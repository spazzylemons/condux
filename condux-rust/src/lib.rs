#![no_main]
#![no_std]

use core::panic::PanicInfo;

pub mod assets;
pub mod linalg;
pub mod platform;
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
