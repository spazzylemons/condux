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

use assets::Asset;
use render::Mesh;

use crate::{state::GameState, timing::Timer, vehicle::{PlayerController, EmptyController, VehicleType}, render::Renderer, spline::Spline, octree::Octree};

fn main() {
    unsafe {
        bindings::platform_init(640, 480);
    }
    let mut renderer = Renderer::new();
    renderer.load_glyphs();
    let mut asset = Asset::load("mesh_vehicle.bin").unwrap();
    let mesh = Mesh::load(&mut asset).unwrap();
    let test_type = VehicleType {
        speed: 15.0,
        acceleration: 7.0,
        handling: 1.5,
        anti_drift: 12.0,
        mesh,
    };
    asset = Asset::load("course_test1.bin").unwrap();
    let spline = Spline::load(&mut asset).unwrap();
    let octree = Octree::new(&spline);
    let mut state = GameState::new(spline, octree, renderer);
    state.renderer.load_spline(&state.spline);
    let controls = Mutex::new(bindings::Controls {
        buttons: 0,
        steering: 0.0,
    });
    let player = PlayerController { controls: &controls };
    let empty = EmptyController;
    let spawn = state.spline.get_baked(0.0);
    state.spawn(spawn, &test_type, &player);
    let spawn = state.spline.get_baked(5.0);
    state.spawn(spawn, &test_type, &empty);
    let spawn = state.spline.get_baked(10.0);
    state.spawn(spawn, &test_type, &empty);
    let spawn = state.spline.get_baked(15.0);
    state.spawn(spawn, &test_type, &empty);
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
