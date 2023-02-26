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

use crate::{
    assets::Asset,
    linalg::Vector,
    octree::Octree,
    platform::{Buttons, Controls, Frame},
    render::Renderer,
    spline::Spline,
    vehicle::{AIController, PlayerController},
};

use super::{race::RaceMode, Mode};

pub struct TitleMode;

impl Mode for TitleMode {
    fn tick(&mut self, _controls: Controls, pressed: Buttons) -> Option<Box<dyn Mode>> {
        if !pressed.is_empty() {
            // transition to race
            let spline = Spline::load(&mut Asset::load("course_test1.bin").unwrap()).unwrap();
            let octree = Octree::new(&spline);
            let mut mode = RaceMode::new(spline, octree, 0);
            // spawn player
            let spawn = mode.spline.get_baked(0.0);
            mode.spawn(spawn, "default", Box::new(PlayerController::default()));
            // spawn some other vehicles
            let spawn = mode.spline.get_baked(5.0);
            mode.spawn(spawn, "default", Box::new(AIController::default()));
            let spawn = mode.spline.get_baked(10.0);
            mode.spawn(spawn, "default", Box::new(AIController::default()));
            let spawn = mode.spline.get_baked(15.0);
            mode.spawn(spawn, "default", Box::new(AIController::default()));
            // set camera behind player
            mode.teleport_camera();
            // send mode transition
            Some(Box::new(mode))
        } else {
            None
        }
    }

    fn camera(&self, _interp: f32) -> (Vector, Vector, Vector) {
        (Vector::ZERO, Vector::Z_AXIS, Vector::Y_AXIS)
    }

    fn render(&self, _interp: f32, renderer: &Renderer, frame: &mut Frame) {
        // draw some text
        renderer.write(4.0, 30.0, 4.0, frame, "press any button to start");
    }
}
