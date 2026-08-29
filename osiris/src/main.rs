#![cfg_attr(not(test), no_main, no_std)]

use core::panic::PanicInfo;
use x86_64::instructions::hlt;

pub extern "C" fn _start(_info: *const ()) -> ! {
    loop {
        hlt();
    }
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        hlt();
    }
}
