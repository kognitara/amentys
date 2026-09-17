#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use core::arch::asm;
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub struct SyscallWriter;
impl Write for SyscallWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // safety: aa
        unsafe {
            asm!(
            "syscall",
            in("rax") 1, in("rdi") 0, in("rsi") s.as_ptr() as u64, in("rdx") s.len() as u64,
            out("rcx") _, out("r11") _,
            );
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print { ($($arg:tt)*) => { let _ = core::fmt::write(&mut $crate::SyscallWriter, core::format_args!($($arg)*)); }; }
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => ($crate::print!("{}\r\n", core::format_args!($($arg)*)));
}

#[allow(dead_code)]
const RESET: &str = "\x1b[0m";
#[allow(dead_code)]
const CYAN: &str = "\x1b[36;1m";
#[allow(dead_code)]
const GREEN: &str = "\x1b[32;1m";

fn get_screen_size() -> (u32, u32) {
    let mut packed_dims: u64;
    // Safety: get screen size
    unsafe {
        asm!(
        "syscall",
        inlateout("rax") 2u64 => packed_dims,
        out("rcx") _,
        out("r11") _
        );
    }
    (
        (packed_dims >> 32) as u32,
        (packed_dims & 0xFFFF_FFFF) as u32,
    )
}
fn flush_screen(buffer: &[u32]) {
    // Safety: clear screen
    unsafe {
        // Safety: flush screen
        asm!("syscall", in("rax") 3, in("rdi") 0, in("rsi") buffer.as_ptr() as u64, out("rcx") _, out("r11") _);
    }
}

#[cfg_attr(not(test), unsafe(no_mangle))]
pub extern "C" fn _start() -> ! {
    // Initialisation d'un Tas de 8 MiB exclusif à Maât
    const HEAP_SIZE: usize = 8 * 1024 * 1024;
    #[repr(C, align(4096))]
    struct Heap([u8; HEAP_SIZE]);
    static mut HEAP: Heap = Heap([0; HEAP_SIZE]);
    // Safety: init heap
    unsafe {
        ALLOCATOR
            .lock()
            .init(core::ptr::addr_of_mut!(HEAP).cast::<u8>(), HEAP_SIZE);
    }

    let (width, height) = get_screen_size();

    // Allocation du buffer vidéo dans l'espace utilisateur
    let total_pixels = (width * height) as usize;
    let mut buffer: Vec<u32> = alloc::vec![0; total_pixels];

    // Dessin du fond graphique (Dégradé vertical)
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) as usize;

            // Calcul d'un cyan stylisé qui s'assombrit vers le bas
            let intensity = 255 - ((y * 255) / height);
            let r = 0x00; // Rouge = 0
            let g = intensity / 2; // Vert modéré
            let b = intensity; // Bleu dominant (Cyan)

            // Format XRGB (0x00RRGGBB)
            buffer[index] = (r << 16) | (g << 8) | b;
        }
    }
    // Envoi du buffer au noyau pour affichage immédiat
    flush_screen(&buffer);

    loop {
        core::hint::spin_loop();
    }
}

#[cfg_attr(not(test), panic_handler)]
#[allow(dead_code)]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
