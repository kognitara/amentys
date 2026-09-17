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

pub fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024 * 1024;

    // Force le compilateur à aligner ce bloc sur une vraie frame physique (4 KiB)
    #[repr(C, align(4096))]
    struct Heap([u8; HEAP_SIZE]);

    // On réserve le bloc statique aligné
    static mut HEAP: Heap = Heap([0; HEAP_SIZE]);

    // SAFETY: Initialisation sécurisée de l'allocateur de mémoire.
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
