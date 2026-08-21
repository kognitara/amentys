#![cfg_attr(not(test), no_std)]
#![feature(abi_x86_interrupt)]
use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest, ModulesRequest};
use os_terminal::{DrawTarget, Rgb, Terminal};
use spin::Mutex;

/// Module principal of the kernel
pub mod gdt;
/// Module for handling interrupts
pub mod interrupts;
/// Module for handling memory management
pub mod memory;
/// Module for handling system calls
pub mod syscall;

use linked_list_allocator::LockedHeap;

/// Declares the official global allocator of our OS
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initializes the kernel heap.
pub fn init_heap() {
    const HEAP_SIZE: usize = 100 * 1024; // 100 Ko de RAM dédiés au noyau
    // On réserve un bloc statique rempli de zéros dans le binaire (.bss)
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    // SAFETY: On donne ce bloc exclusif à notre allocateur global
    unsafe {
        let heap_ptr = core::ptr::addr_of_mut!(HEAP).cast::<u8>();
        ALLOCATOR.lock().init(heap_ptr, HEAP_SIZE);
    }
}
/// Represents the framebuffer data for the screen.
///
/// # Fields
/// - `ptr`: A pointer to the framebuffer memory.
/// - `width`: The width of the screen in pixels.
/// - `height`: The height of the screen in pixels.
/// - `pitch`: The number of bytes in a single row of the framebuffer.
pub struct ScreenData {
    pub ptr: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
}
// SAFETY: The framebuffer pointer is managed exclusively by the kernel and is safe to send between threads.
unsafe impl Send for ScreenData {}
// SAFETY: Concurrent access to the framebuffer is strictly protected by the global Mutex (TERMINAL).
unsafe impl Sync for ScreenData {}

impl DrawTarget for ScreenData {
    fn size(&self) -> (usize, usize) {
        (
            usize::try_from(self.width).unwrap_or(0),
            usize::try_from(self.height).unwrap_or(0),
        )
    }

    #[inline]
    fn draw_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        let screen_x = u64::try_from(x).unwrap_or(0);
        let screen_y = u64::try_from(y).unwrap_or(0);
        let offset = (screen_y * self.pitch) + (screen_x * 4);
        let offset_usize = usize::try_from(offset).unwrap_or(0);
        #[allow(clippy::cast_ptr_alignment)]
        // SAFETY: L'offset est calculé mathématiquement pour correspondre aux limites du Framebuffer vidéo.
        unsafe {
            let pixel_ptr = self.ptr.add(offset_usize).cast::<u32>();
            let raw_color =
                (u32::from(color.0) << 16) | (u32::from(color.1) << 8) | u32::from(color.2);
            *pixel_ptr = raw_color;
        }
    }
}
/// A global mutex-protected terminal instance that can be safely accessed across threads.
pub static TERMINAL: Mutex<Option<Terminal<ScreenData>>> = Mutex::new(None);
/// Global Limine requests for various system information.
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
/// Global Limine requests for various system information.
pub static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();
/// Global Limine requests for various system information.
pub static MODULE_REQUEST: ModulesRequest = ModulesRequest::new();
/// Global Limine requests for various system information.
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
