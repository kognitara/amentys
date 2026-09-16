#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;
use core::panic::PanicInfo;
use ra::println;

use linked_list_allocator::LockedHeap;
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    // Safety: This is required to initialize the heap
    unsafe {
        let heap_ptr = core::ptr::addr_of_mut!(HEAP).cast::<u8>();
        ALLOCATOR.lock().init(heap_ptr, HEAP_SIZE);
    }
}
#[allow(clippy::too_many_lines)]
#[cfg_attr(not(test), unsafe(no_mangle))]
pub extern "C" fn _start(_info: *const ()) -> ! {
    init_heap();
    loop {
        core::hint::spin_loop();
    }
}

#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info.message());
    loop {
        core::hint::spin_loop();
    }
}
