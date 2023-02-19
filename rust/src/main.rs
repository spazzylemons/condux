pub mod assets;
pub mod bindings;
pub mod linalg;
pub mod octree;
#[macro_use]
pub mod render;
pub mod spline;
pub mod state;
pub mod timing;
pub mod vehicle;

use std::{sync::Mutex, mem::zeroed};

#[cfg(target_os = "horizon")]
use ctru::prelude::*;

use crate::{state::GameState, timing::Timer, vehicle::{PlayerController, EmptyController}, render::Renderer};

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
        bindings::platform_init(640, 480);
    }
    let mut renderer = Renderer::new();
    renderer.load_glyphs();
    let mut asset = bindings::Asset::load("mesh_vehicle.bin").unwrap();
    let mesh = bindings::Mesh::load(&mut asset).unwrap();
    let test_type = bindings::VehicleType {
        speed: 15.0,
        acceleration: 7.0,
        handling: 1.5,
        antiDrift: 12.0,
        mesh,
    };
    asset = bindings::Asset::load("course_test1.bin").unwrap();
    let spline = bindings::Spline::load(&mut asset).unwrap();
    let octree = bindings::Octree::new(&spline);
    let mut state = GameState::new(spline, octree, renderer);
    state.renderer.load_spline(&state.spline);
    let controls = Mutex::new(bindings::Controls {
        buttons: 0,
        steering: 0.0,
    });
    let player = PlayerController { controls: &controls };
    let empty = EmptyController;
    let spawn = state.spline.get_baked(0.0);
    assert!(state.spawn(spawn, &test_type, &player));
    let spawn = state.spline.get_baked(5.0);
    assert!(state.spawn(spawn, &test_type, &empty));
    let spawn = state.spline.get_baked(10.0);
    assert!(state.spawn(spawn, &test_type, &empty));
    let spawn = state.spline.get_baked(15.0);
    assert!(state.spawn(spawn, &test_type, &empty));
    state.teleport_camera(0);
    let mut timer = Timer::new();
    while unsafe { bindings::platform_should_run() } {
        let mut temp_controls = unsafe { zeroed() };
        unsafe { bindings::platform_poll(&mut temp_controls) };
        *controls.lock().unwrap() = temp_controls;
        let (mut i, interp) = timer.frame_ticks();
        while i > 0 {
            i -= 1;
            state.update(0);
        }
        unsafe { bindings::platform_start_frame() };
        state.render(0, interp);
        unsafe { bindings::platform_end_frame() };
        if (controls.lock().unwrap().buttons & bindings::BTN_PAUSE as u8) != 0 {
            break;
        }
    }
    unsafe {
        bindings::platform_deinit();
    }
}
