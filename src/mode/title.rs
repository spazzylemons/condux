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

use crate::{platform::Buttons, render::context::RenderContext};

use super::{race::RaceMode, GlobalGameData, Mode};

pub struct TitleMode;

impl Mode for TitleMode {
    fn tick(self: Box<Self>, data: &GlobalGameData) -> Box<dyn Mode> {
        // if pressed pause, then consider that a signal to quit, except on web
        // since it doesn't make logical sense to stop running on web
        #[cfg(not(target_arch = "wasm32"))]
        if data.pressed.contains(Buttons::BACK) {
            data.should_run.set(false);
        }

        if data.pressed.contains(Buttons::OK) {
            // transition to race
            Box::new(RaceMode::initialized(&data.garage))
        } else {
            self
        }
    }

    fn render(&self, _interp: f32, data: &GlobalGameData, context: &mut dyn RenderContext) {
        // draw some text
        let center = f32::from(context.width()) * 0.5;
        data.font
            .write_centered(context, center, 32.0, 6.0, "CONDUX");
        data.font
            .write_centered(context, center, 120.0, 4.0, "Press OK to start");

        #[cfg(not(target_arch = "wasm32"))]
        data.font
            .write_centered(context, center, 160.0, 4.0, "Press Back to quit");
    }
}