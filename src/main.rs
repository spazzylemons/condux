#![cfg_attr(target_os = "horizon", feature(allocator_api))]

pub mod assets;
pub mod linalg;
pub mod platform;
pub mod octree;
#[macro_use]
pub mod render;
pub mod spline;
pub mod state;
pub mod timing;
pub mod vehicle;

use std::sync::Mutex;

use assets::Asset;
use platform::{Platform, Controls, Buttons};
use render::Mesh;

const DEADZONE: f32 = 0.03;

use crate::{state::GameState, timing::Timer, vehicle::{PlayerController, EmptyController, VehicleType}, render::Renderer, spline::Spline, octree::Octree};

fn main() {
    let mut platform = platform::PlatformImpl::init(640, 480).unwrap();
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
    let controls = Mutex::new(Controls {
        buttons: Buttons::empty(),
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
    let mut timer = Timer::new(&platform);
    while platform.should_run() {
        let mut new_controls = platform.poll();
        // apply deadzone
        if new_controls.steering.abs() < DEADZONE {
            new_controls.steering = 0.0;
        }
        *controls.lock().unwrap() = new_controls;
        let (mut i, interp) = timer.frame_ticks(&platform);
        while i > 0 {
            i -= 1;
            state.update(0);
        }
        let mut frame = platform.start_frame();
        state.render(0, interp, &mut frame);
        frame.finish();
        if controls.lock().unwrap().buttons.contains(Buttons::PAUSE) {
            break;
        }
    }
}
