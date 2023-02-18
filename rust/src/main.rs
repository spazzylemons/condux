pub mod bindings;
pub mod timing;

#[cfg(target_os = "horizon")]
use ctru::prelude::*;

#[cfg(target_os = "horizon")]
fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::init().expect("fail 1");
    let hid = Hid::init().expect("fail 2");
    let apt = Apt::init().expect("fail 3");

    // don't name this _, or we'll drop the console
    let _console = Console::init(gfx.top_screen.borrow_mut());

    game_main();

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::KEY_START) {
            break;
        }

        gfx.flush_buffers();
        gfx.swap_buffers();

        gfx.wait_for_vblank();
    }
}

#[cfg(not(target_os = "horizon"))]
fn main() {
    game_main();
}

fn game_main() {
    unsafe {
        bindings::game_init();
        while bindings::platform_should_run() {
            bindings::platform_start_frame();
            bindings::game_loop();
            bindings::platform_end_frame();
            if (bindings::gControls.buttons & (bindings::BTN_PAUSE as u8)) != 0 {
                break;
            }
        }
        bindings::game_deinit();
    }
}
