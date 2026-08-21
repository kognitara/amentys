#![cfg_attr(not(test), no_main, no_std)]

use core::panic::PanicInfo;
use x86_64::instructions::hlt;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let _hu = hu::Hu::new();
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
