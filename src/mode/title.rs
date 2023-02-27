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

use crate::render::context::RenderContext;

use super::{race::RaceMode, GlobalGameData, Mode};

pub struct TitleMode;

impl Mode for TitleMode {
    fn tick(self: Box<Self>, data: &GlobalGameData) -> Box<dyn Mode> {
        if !data.pressed.is_empty() {
            // transition to race
            Box::new(RaceMode::initialized(&data.garage))
        } else {
            self
        }
    }

    fn render(&self, _interp: f32, data: &GlobalGameData, context: &mut dyn RenderContext) {
        // draw some text
        data.font
            .write(context, 4.0, 4.0, 4.0, "press any button to start");
    }
}
