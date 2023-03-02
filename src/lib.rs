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

use render::{context::GenericBaseContext, Font};
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

use platform::{Buttons, Controls, Impl, Platform};

const DEADZONE: f32 = 0.03;

use crate::{render::graph::RenderGraph, timing::Timer};

/// Structure of game update for non-WASM targets.
#[cfg(not(target_arch = "wasm32"))]
struct GameUpdate {
    /// Sends render graph
    sender: std::sync::mpsc::SyncSender<RenderUpdate>,
    /// Holds update data
    receiver: std::sync::mpsc::Receiver<PlatformUpdate>,
}

/// Implementation of game update for non-WASM targets.
#[cfg(not(target_arch = "wasm32"))]
impl GameUpdate {
    /// Sends the update.
    fn send_update(&self, update: RenderUpdate) -> PlatformUpdate {
        // send new update
        self.sender.send(update).unwrap();
        // receive new update
        self.receiver.recv().unwrap()
    }
}

/// Structure of game update for WASM target. Not multithreaded.
#[cfg(target_arch = "wasm32")]
struct GameUpdate {
    /// Platform to use
    platform: RefCell<Impl>,
    /// The font to use
    font: Font,
}

/// Implementation of game update for WASM target.
#[cfg(target_arch = "wasm32")]
impl GameUpdate {
    fn send_update(&self, update: RenderUpdate) -> PlatformUpdate {
        let mut platform = self.platform.borrow_mut();
        let result = generate_update(&mut *platform);
        match update {
            RenderUpdate::Graph(graph) => {
                let mut ctx = GenericBaseContext::new(&mut *platform);
                graph.render(&self.font, &mut ctx);
                ctx.finish();
            }
        }
        result
    }
}

/// Contains information sent from the render thread to the game thread.
#[derive(Clone, Copy)]
struct PlatformUpdate {
    controls: Controls,
    width: u16,
    height: u16,
    #[cfg(not(target_arch = "wasm32"))]
    should_run: bool,
}

/// Contains information sent from the game thread to the render thread.
enum RenderUpdate {
    /// Here is a scene to draw.
    Graph(RenderGraph),
    /// Game has ended
    #[cfg(not(target_arch = "wasm32"))]
    End,
}

struct Game {
    /// The game timer.
    timer: Timer,
    /// The game mode.
    mode: Box<dyn Mode>,
    /// The global game data.
    data: GlobalGameData,
    /// The game update information.
    update: GameUpdate,
    /// The last seen value of update.
    last_update: PlatformUpdate,
}

impl Game {
    #[must_use]
    fn init(update: GameUpdate) -> Self {
        // let platform = platform::Impl::init(640, 480);
        let mut data = GlobalGameData::default();
        data.garage.load_hardcoded();
        data.walls.set(true);

        #[cfg(not(target_arch = "wasm32"))]
        data.should_run.set(true);

        let last_update = update.send_update(RenderUpdate::Graph(RenderGraph::default()));
        let timer = Timer::new();

        Self {
            timer,
            mode: Box::new(TitleMode),
            data,
            update,
            last_update,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn should_run(&self) -> bool {
        self.data.should_run.get() && self.last_update.should_run
    }

    fn update_controls(&mut self) {
        // get controls
        let mut controls = self.last_update.controls;
        // apply deadzone
        if controls.steering.abs() < DEADZONE {
            controls.steering = 0.0;
        }
        // cancel out up/down and left/right
        if controls.buttons.contains(Buttons::UP | Buttons::DOWN) {
            controls.buttons &= !(Buttons::UP | Buttons::DOWN);
        }
        if controls.buttons.contains(Buttons::LEFT | Buttons::RIGHT) {
            controls.buttons &= !(Buttons::LEFT | Buttons::RIGHT);
        }
        // determine which buttons were pressed
        self.data.pressed = controls.buttons & !self.data.controls.buttons;
        self.data.controls = controls;
    }

    fn iteration(mut self) -> Self {
        self.update_controls();
        // update game state
        let (mut i, interp) = self.timer.frame_ticks();
        while i > 0 {
            i -= 1;
            self.mode = self.mode.tick(&self.data);
            // clear pressed buttons to avoid triggering stuff if we need to run multiple frames
            self.data.pressed = Buttons::empty();
        }
        // render frame
        let mut graph = RenderGraph::default();
        self.mode.render(
            interp,
            &self.data,
            &mut graph,
            self.last_update.width,
            self.last_update.height,
        );
        self.last_update = self.update.send_update(RenderUpdate::Graph(graph));
        // return game
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn end(self) {
        self.update.send_update(RenderUpdate::End);
    }
}

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

/// Runs a frame using requestAnimationFrame.
#[cfg(target_arch = "wasm32")]
fn create_closure(keep_alive: Rc<RefCell<Closure<dyn FnMut()>>>, game: Game) -> impl FnOnce() {
    move || {
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

fn generate_update(platform: &mut Impl) -> PlatformUpdate {
    PlatformUpdate {
        controls: platform.poll(),
        width: platform.width(),
        height: platform.height(),
        #[cfg(not(target_arch = "wasm32"))]
        should_run: platform.should_run(),
    }
}

pub fn run_game() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (render_tx, render_rx) = std::sync::mpsc::sync_channel::<RenderUpdate>(0);
        let (platform_tx, platform_rx) = std::sync::mpsc::sync_channel::<PlatformUpdate>(0);
        let update = GameUpdate {
            sender: render_tx,
            receiver: platform_rx,
        };
        let mut platform = Impl::init(640, 480);

        // game thread
        let game_thread = std::thread::spawn(move || {
            let mut game = Game::init(update);
            while game.should_run() {
                game = game.iteration();
            }
            // send end message
            game.end();
        });
        // render thread runs here
        let font = Font::new().unwrap();
        loop {
            // perform update exchange
            let render_update = render_rx.recv().unwrap();
            platform_tx.send(generate_update(&mut platform)).unwrap();
            // get new messages
            match render_update {
                RenderUpdate::End => break,
                RenderUpdate::Graph(graph) => {
                    let mut ctx = GenericBaseContext::new(&mut platform);
                    graph.render(&font, &mut ctx);
                    ctx.finish();
                }
            }
        }
        // join game thread when done
        game_thread.join().unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    {
        let font = Font::new().unwrap();
        // implementation of game update for WASM
        let game_update = GameUpdate {
            platform: RefCell::new(Impl::init(640, 480)),
            font,
        };
        // keeps the closure alive
        let keep_alive = Rc::new(RefCell::new(Closure::once(|| ())));
        // runs the infinite loop
        create_closure(keep_alive.clone(), Game::init(game_update))();
    }
}

#[cfg(target_arch = "wasm32")]
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}
