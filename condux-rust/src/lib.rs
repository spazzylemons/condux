#![no_main]
#![no_std]

use core::panic::PanicInfo;

pub mod render;

extern "C" {
    fn abort();
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            abort();
        }
    }
}
