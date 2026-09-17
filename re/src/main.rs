#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;
use alloc::boxed::Box;
use core::panic::PanicInfo;
use core::ptr;
use os_terminal::Terminal;
use os_terminal::font::BitmapFont;
use x86_64::VirtAddr;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use xmas_elf::ElfFile;
///
/// # Panics
/// fail is not a valid framebuffer address.
#[cfg_attr(not(test), unsafe(no_mangle))]
#[allow(clippy::too_many_lines)]
pub extern "C" fn _start(_info: *const ()) -> ! {
    x86_64::instructions::interrupts::disable();
    re::gdt::init();
    re::interrupts::init();
    re::syscall::init();

    // 1. Initialisation de la Pagination
    let hhdm_offset = re::HHDM_REQUEST
        .response()
        .expect("Error: Limine didn't provide the HHDM offset")
        .offset;

    let memory_map = re::MEMORY_MAP_REQUEST
        .response()
        .expect("Error: Limine didn't provide the Memory Map")
        .entries();

    let mut frame_allocator = re::memory::BootInfoFrameAllocator::new(memory_map);
    let phys_mem_offset = VirtAddr::new(hhdm_offset);

    // SAFETY: L'offset HHDM fourni par Limine est valide et correspond à la cartographie virtuelle initiale.
    let mut mapper = unsafe { re::memory::init_paging(phys_mem_offset) };

    // 2. Initialisation du Tas (Heap) : Doit être fait APRÈS la pagination
    re::init_heap();

    // 3. Initialisation du Framebuffer et du Terminal (Nécessite le Tas pour Box::new)
    if let Some(framebuffer_response) = re::FRAMEBUFFER_REQUEST.response()
        && let Some(framebuffer) = framebuffer_response.framebuffers().first()
    {
        // --- NOUVEAU : On empaquette les dimensions pour Maât ---
        let packed_dims = ((framebuffer.width) << 32) | (framebuffer.height);
        re::syscall::SCREEN_DIMS.store(packed_dims, core::sync::atomic::Ordering::Relaxed);
        // --------------------------------------------------------
        let size_u64 = framebuffer.pitch * framebuffer.height;
        let size = usize::try_from(size_u64).unwrap_or(0);

        // SAFETY: L'adresse du framebuffer est garantie par le protocole d'amorçage.
        unsafe {
            ptr::write_bytes(framebuffer.address(), 0, size);
        }

        let screen_data = ra::ScreenData {
            ptr: framebuffer.address().cast::<u8>(),
            width: framebuffer.width,
            height: framebuffer.height,
            pitch: framebuffer.pitch,
        };
        *ra::TERMINAL.lock() = Some(Terminal::new(screen_data, Box::new(BitmapFont)));
    }

    // 4. Chargement du module ELF Maât et préparation de l'espace utilisateur
    if let Some(modules_response) = re::MODULE_REQUEST.response() {
        for module in modules_response.modules() {
            let path = module.path();
            if path.ends_with("maat") || path.ends_with("/maat") {
                let elf_data: &[u8] = module.data();
                let elf = ElfFile::new(elf_data).expect("failed to parse the Maât ELF module");
                let entry_point = elf.header.pt2.entry_point();

                for ph in elf.program_iter() {
                    if ph.get_type() == Ok(xmas_elf::program::Type::Load) {
                        let start_addr = VirtAddr::new(ph.virtual_addr());
                        let end_addr = start_addr + ph.mem_size();

                        let start_page: Page<Size4KiB> = Page::containing_address(start_addr);
                        let end_page: Page<Size4KiB> = Page::containing_address(end_addr - 1u64);

                        let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
                        if ph.flags().is_write() {
                            flags |= PageTableFlags::WRITABLE;
                        }

                        for page in Page::range_inclusive(start_page, end_page) {
                            let is_mapped = mapper.translate_page(page).is_ok();

                            let frame = if is_mapped {
                                mapper.translate_page(page).unwrap()
                            } else {
                                let new_frame =
                                    frame_allocator.allocate_frame().expect("Plus de RAM !");

                                // SAFETY: Mappage sécurisé d'une nouvelle frame physique.
                                unsafe {
                                    mapper
                                        .map_to(page, new_frame, flags, &mut frame_allocator)
                                        .expect("Erreur de mappage Page Table")
                                        .flush();
                                }
                                new_frame
                            };

                            // SAFETY: L'adresse HHDM calculée est garantie valide.
                            unsafe {
                                let hhdm_addr = phys_mem_offset + frame.start_address().as_u64();

                                if !is_mapped {
                                    ptr::write_bytes(hhdm_addr.as_mut_ptr::<u8>(), 0, 4096);
                                }

                                let page_start_vaddr = page.start_address();
                                let page_end_vaddr = page_start_vaddr + 4096u64;
                                let segment_vaddr_start = start_addr;
                                let segment_vaddr_end = start_addr + ph.file_size();

                                if page_start_vaddr < segment_vaddr_end
                                    && page_end_vaddr > segment_vaddr_start
                                {
                                    let copy_start_vaddr =
                                        core::cmp::max(page_start_vaddr, segment_vaddr_start);
                                    let copy_end_vaddr =
                                        core::cmp::min(page_end_vaddr, segment_vaddr_end);
                                    let copy_size =
                                        usize::try_from(copy_end_vaddr - copy_start_vaddr)
                                            .unwrap_or(0);

                                    let file_offset =
                                        ph.offset() + (copy_start_vaddr - segment_vaddr_start);
                                    let page_offset = copy_start_vaddr - page_start_vaddr;

                                    let src_ptr = elf_data
                                        .as_ptr()
                                        .add(usize::try_from(file_offset).unwrap_or(0));
                                    let dst_ptr = hhdm_addr
                                        .as_mut_ptr::<u8>()
                                        .add(usize::try_from(page_offset).unwrap_or(0));

                                    ptr::copy_nonoverlapping(src_ptr, dst_ptr, copy_size);
                                }
                            }
                        }
                    }
                }

                // 5. Création de la pile (Stack) Ring 3 pour Maât
                let stack_end = VirtAddr::new(0x0000_7FFF_FFFF_F000);
                let stack_start = stack_end - (4096u64 * 4);

                let stack_start_page: Page<Size4KiB> = Page::containing_address(stack_start);
                let stack_end_page: Page<Size4KiB> = Page::containing_address(stack_end - 1u64);

                let stack_flags = PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE;

                for page in Page::range_inclusive(stack_start_page, stack_end_page) {
                    let frame = frame_allocator.allocate_frame().unwrap();

                    // SAFETY: Allocation physique et isolation de la pile mémoire.
                    unsafe {
                        mapper
                            .map_to(page, frame, stack_flags, &mut frame_allocator)
                            .unwrap()
                            .flush();
                        let hhdm_addr = phys_mem_offset + frame.start_address().as_u64();
                        ptr::write_bytes(hhdm_addr.as_mut_ptr::<u8>(), 0, 4096);
                    }
                }

                // 6. Saut de privilège vers l'espace utilisateur (IRETQ)
                // SAFETY: Configuration finale des registres d'état et basculement en Ring 3.
                unsafe {
                    core::arch::asm!(
                    "push 0x23",
                    "push {stack}",
                    "push 0x002",
                    "push 0x2B",
                    "push {entry}",
                    "iretq",
                    stack = in(reg) stack_end.as_u64(),
                    entry = in(reg) entry_point,
                    options(noreturn)
                    );
                }
            }
        }
    }
    loop {
        x86_64::instructions::hlt();
    }
}
use core::fmt::Write;

pub struct SerialPort;

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                // SAFETY:LLe port série exige un retour chariot (\r) avant le saut de ligne (\n)
                unsafe {
                    core::arch::asm!("out dx, al", in("dx") 0x3F8_u16, in("al") b'\r');
                }
            }
            // SAFETY: The calculated HHDM address is guaranteed to be valid as it derives from a freshly allocated or existing physical frame.
            unsafe {
                core::arch::asm!("out dx, al", in("dx") 0x3F8_u16, in("al") byte);
            }
        }
        Ok(())
    }
}

#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    let _ = writeln!(SerialPort, "\n\n[KERNEL PANIC] {info}");

    loop {
        x86_64::instructions::hlt();
    }
}
