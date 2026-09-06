#![cfg_attr(not(test), no_std)]
#![feature(abi_x86_interrupt)]
use limine::request::{FramebufferRequest, HhdmRequest, MemmapRequest, ModulesRequest};

/// Module principal of the kernel
pub mod gdt;
/// Module for handling interrupts
pub mod interrupts;
/// Module for handling memory management
pub mod memory;
/// Module for handling system calls
pub mod syscall;
pub mod time;

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

/// Global Limine requests for various system information.
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
/// Global Limine requests for various system information.
pub static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();
/// Global Limine requests for various system information.
pub static MODULE_REQUEST: ModulesRequest = ModulesRequest::new();
/// Global Limine requests for various system information.
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
