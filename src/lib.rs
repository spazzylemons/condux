//! Condux - an antigravity racing game
//! Copyright (C) 2023 spazzylemons
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(target_os = "horizon", feature(allocator_api))]

use ouroboros::self_referencing;

use vehicle::AIController;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod assets;
mod linalg;
mod octree;
mod platform;
#[macro_use]
mod render;
mod spline;
mod state;
mod timing;
mod util;
mod vehicle;

use std::cell::Cell;

use assets::Asset;
use platform::{Buttons, Controls, Impl, Platform};
use render::Mesh;

const DEADZONE: f32 = 0.03;

use crate::{
    octree::Octree,
    render::Renderer,
    spline::Spline,
    state::GameState,
    timing::Timer,
    vehicle::{Model, PlayerController},
};

#[self_referencing]
struct Game {
    /// Platform-specific interface.
    platform: Impl,
    /// The game timer.
    timer: Timer,
    /// The state of the controls.
    controls: Cell<Controls>,
    /// The model of the vehicle.
    model: Model,
    /// The game state.
    #[borrows(controls, model)]
    #[covariant]
    state: GameState<'this>,
}

impl Game {
    #[must_use]
    fn init() -> Self {
        let platform = platform::Impl::init(640, 480);
        let mut renderer = Renderer::new();
        renderer.load_glyphs();

        let mesh = Mesh::load(&mut Asset::load("mesh_vehicle.bin").unwrap()).unwrap();
        let model = Model {
            speed: 15.0,
            acceleration: 7.0,
            handling: 1.5,
            anti_drift: 12.0,
            mesh,
        };

        let spline = Spline::load(&mut Asset::load("course_test1.bin").unwrap()).unwrap();
        let octree = Octree::new(&spline);
        renderer.load_spline(&spline);

        let controls = Cell::new(Controls {
            buttons: Buttons::empty(),
            steering: 0.0,
        });
        let timer = Timer::new(&platform);

        GameBuilder {
            platform,
            timer,
            controls,
            model,
            state_builder: move |controls, model| {
                let mut state = GameState::new(spline, octree, renderer);
                // spawn player
                let spawn = state.spline.get_baked(0.0);
                state.spawn(spawn, model, Box::new(PlayerController { controls }));
                // spawn some other vehicles
                let spawn = state.spline.get_baked(5.0);
                state.spawn(spawn, model, Box::new(AIController::default()));
                let spawn = state.spline.get_baked(10.0);
                state.spawn(spawn, model, Box::new(AIController::default()));
                let spawn = state.spline.get_baked(15.0);
                state.spawn(spawn, model, Box::new(AIController::default()));
                // set camera behind player
                state.teleport_camera(0);
                // return state object
                state
            },
        }
        .build()
    }

    fn should_run(&self) -> bool {
        self.borrow_platform().should_run()
            && !self
                .borrow_controls()
                .get()
                .buttons
                .contains(Buttons::PAUSE)
    }

    fn update_controls(&mut self) {
        // get controls
        let mut new_controls = self.with_platform_mut(|platform| platform.poll());
        // apply deadzone
        if new_controls.steering.abs() < DEADZONE {
            new_controls.steering = 0.0;
        }
        // set controls
        self.borrow_controls().set(new_controls);
    }

    fn iteration(&mut self) {
        self.update_controls();
        // update game state
        let (mut i, interp) = self.with_mut(|fields| fields.timer.frame_ticks(fields.platform));
        while i > 0 {
            i -= 1;
            self.with_state_mut(|state| state.update(0));
        }
        // render frame
        self.with_mut(|fields| {
            let mut frame = fields.platform.start_frame();
            fields.state.render(0, interp, &mut frame);
            frame.finish();
        });
    }
}

pub fn run_game() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut game = Game::init();
        while game.should_run() {
            game.iteration();
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        use std::{cell::RefCell, rc::Rc};

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let mut game = Game::init();

        *g.borrow_mut() = Some(Closure::new(move || {
            if game.should_run() {
                game.iteration();
                request_animation_frame(f.borrow().as_ref().unwrap());
            }
        }));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }
}

#[cfg(target_arch = "wasm32")]
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}
