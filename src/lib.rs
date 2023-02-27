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

use mode::{title::TitleMode, GlobalGameData, Mode};

use render::Font;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod assets;
mod linalg;
mod mode;
mod octree;
mod platform;
mod render;
mod spline;
mod timing;
mod util;
mod vehicle;

use platform::{Buttons, Impl, Platform};

const DEADZONE: f32 = 0.03;

use crate::timing::Timer;

struct Game {
    /// Platform-specific interface.
    platform: Impl,
    /// The game timer.
    timer: Timer,
    /// The game mode.
    mode: Box<dyn Mode>,
    /// The global game data.
    data: GlobalGameData,
}

impl Game {
    #[must_use]
    fn init() -> Self {
        let platform = platform::Impl::init(640, 480);
        let mut data = GlobalGameData::default();
        data.garage.load_hardcoded();
        data.font = Font::new().unwrap();
        let timer = Timer::new(&platform);

        Self {
            platform,
            timer,
            mode: Box::new(TitleMode),
            data,
        }
    }

    fn should_run(&self) -> bool {
        self.platform.should_run()
    }

    fn update_controls(&mut self) {
        // get controls
        let mut controls = self.platform.poll();
        // apply deadzone
        if controls.steering.abs() < DEADZONE {
            controls.steering = 0.0;
        }
        // determine which buttons were pressed
        self.data.pressed = controls.buttons & !self.data.controls.buttons;
        self.data.controls = controls;
    }

    fn iteration(mut self) -> Self {
        self.update_controls();
        // update game state
        let (mut i, interp) = self.timer.frame_ticks(&self.platform);
        while i > 0 {
            i -= 1;
            self.mode = self.mode.tick(&self.data);
            // clear pressed buttons to avoid triggering stuff if we need to run multiple frames
            self.data.pressed = Buttons::empty();
        }
        // render frame
        let mut context = self.platform.start_frame();
        self.mode.render(interp, &self.data, &mut context);
        context.finish();
        self
    }
}

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

/// Runs a frame using requestAnimationFrame.
#[cfg(target_arch = "wasm32")]
fn create_closure(keep_alive: Rc<RefCell<Closure<dyn FnMut()>>>, game: Game) -> impl FnOnce() {
    move || {
        if game.should_run() {
            // iterate game
            let game = game.iteration();
            // create new closure to run
            let closure = create_closure(keep_alive.clone(), game);
            // store in keey_alive to prevent it from being dropped
            keep_alive.replace(Closure::once(closure));
            // call this closure on next animation frame
            request_animation_frame(&keep_alive.borrow());
        }
    }
}

pub fn run_game() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut game = Game::init();
        while game.should_run() {
            game = game.iteration();
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // keeps the closure alive
        let keep_alive = Rc::new(RefCell::new(Closure::once(|| ())));
        // runs the infinite loop
        create_closure(keep_alive.clone(), Game::init())();
    }
}

#[cfg(target_arch = "wasm32")]
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}
