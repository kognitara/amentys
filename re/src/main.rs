#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;
use alloc::boxed::Box;
use core::panic::PanicInfo;
use core::ptr;
use os_terminal::Terminal;
use os_terminal::font::BitmapFont;
use ra::println;
use x86_64::VirtAddr;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use xmas_elf::ElfFile;
///
/// # Panics
/// fail is not a valid framebuffer address.
#[allow(clippy::too_many_lines)]
#[cfg_attr(not(test), unsafe(no_mangle))]
pub extern "C" fn _start(_info: *const ()) -> ! {
    x86_64::instructions::interrupts::disable();
    re::gdt::init();
    re::interrupts::init();
    re::syscall::init();
    re::init_heap();
    if let Some(framebuffer_response) = re::FRAMEBUFFER_REQUEST.response()
        && let Some(framebuffer) = framebuffer_response.framebuffers().first()
    {
        let size_u64 = framebuffer.pitch * framebuffer.height;
        let size = usize::try_from(size_u64).unwrap_or(0);

        // SAFETY: `framebuffer.address()` is a raw MMIO address guaranteed to be valid by the boot protocol.
        unsafe {
            ptr::write_bytes(framebuffer.address(), 0, size);
        }

        // We create the os-terminal compatible drawing target
        let screen_data = ra::ScreenData {
            ptr: framebuffer.address().cast::<u8>(),
            width: framebuffer.width,
            height: framebuffer.height,
            pitch: framebuffer.pitch,
        };
        // We lock and initialize the global kernel terminal
        *ra::TERMINAL.lock() = Some(Terminal::new(screen_data, Box::new(BitmapFont)));
    }

    // memory management setup
    let hhdm_offset = re::HHDM_REQUEST
        .response()
        .expect("Error: Limine didn't provide the HHDM offset")
        .offset;
    // SAFETY: The HHDM offset provided by Limine is guaranteed to be valid and corresponds to the initial virtual mapping.
    let memory_map = re::MEMORY_MAP_REQUEST
        .response()
        .expect("Error: Limine didn't provide the Memory Map")
        .entries();
    let mut frame_allocator = re::memory::BootInfoFrameAllocator::new(memory_map);

    let phys_mem_offset = VirtAddr::new(hhdm_offset);

    // SAFETY: L'offset HHDM fourni par Limine correspond à la cartographie virtuelle initiale.
    let mut mapper = unsafe { re::memory::init_paging(phys_mem_offset) };

    // 4. Loading the Maât ELF module and preparing the user-space environment
    if let Some(modules_response) = re::MODULE_REQUEST.response() {
        for module in modules_response.modules() {
            if module.path() == "maat" {
                let elf_data: &[u8] = module.data();
                let elf = ElfFile::new(elf_data).expect("failed to parse the Maât ELF module");
                let entry_point = elf.header.pt2.entry_point();

                // each program header describes a segment to be loaded into memory
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
                            // 1. Check if the virtual page already exists in our page table
                            let is_mapped = mapper.translate_page(page).is_ok();

                            let frame = if is_mapped {
                                // 2. If it already exists, just retrieve the corresponding RAM block without re-mapping
                                mapper.translate_page(page).unwrap()
                            } else {
                                // 3. If it doesn't exist, allocate a RAM block and map it
                                let new_frame =
                                    frame_allocator.allocate_frame().expect("Plus de RAM !");

                                // SAFETY: Safe mapping of a new physical frame into the system page table.
                                unsafe {
                                    mapper
                                        .map_to(page, new_frame, flags, &mut frame_allocator)
                                        .expect("Erreur de mappage Page Table")
                                        .flush();
                                }
                                new_frame
                            };

                            // SAFETY: The calculated HHDM address is guaranteed to be valid as it derives from a freshly allocated or existing physical frame.
                            unsafe {
                                let hhdm_addr = phys_mem_offset + frame.start_address().as_u64();

                                // NOTE: Only zero out the page if it was just created.
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

                // 5. Creation of the user-space Ring 3 stack
                let stack_end = VirtAddr::new(0x0000_7FFF_FFFF_F000);
                let stack_start = stack_end - (4096u64 * 4); // Raw 16 KiB stack

                let stack_start_page: Page<Size4KiB> = Page::containing_address(stack_start);
                let stack_end_page: Page<Size4KiB> = Page::containing_address(stack_end - 1u64);

                let stack_flags = PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE;

                for page in Page::range_inclusive(stack_start_page, stack_end_page) {
                    let frame = frame_allocator.allocate_frame().unwrap();

                    // SAFETY: Physical allocation and isolation of the memory stack for the application.
                    unsafe {
                        mapper
                            .map_to(page, frame, stack_flags, &mut frame_allocator)
                            .unwrap()
                            .flush();
                        let hhdm_addr = phys_mem_offset + frame.start_address().as_u64();
                        ptr::write_bytes(hhdm_addr.as_mut_ptr::<u8>(), 0, 4096);
                    }
                }

                // 6. Privilege jump to user-space (hardware IRETQ)
                // SAFETY: Final configuration of the status registers and jump to Ring 3.
                unsafe {
                    core::arch::asm!(
                        "push 0x23", // SS utilisateur (GDT index 0x20 | RPL 3)
                        "push {stack}",
                        "push 0x002", // RFLAGS (Sans l'IF flag puisque l'IDT / PIC ne sont pas là !)
                        "push 0x2B", // CS utilisateur (GDT index 0x28 | RPL 3)
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

#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info.message());
    loop {
        x86_64::instructions::hlt();
    }
}
