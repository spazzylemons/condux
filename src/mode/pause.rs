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
    platform::Buttons,
    render::context::{RenderContext, ScissorContext},
};

use super::{GlobalGameData, Mode};

pub struct PauseMode {
    contains: Box<dyn Mode>,
}

impl PauseMode {
    pub fn new(contains: Box<dyn Mode>) -> Self {
        Self { contains }
    }

    const CLIP_WIDTH: f32 = 200.0;
}

impl Mode for PauseMode {
    fn tick(self: Box<Self>, data: &GlobalGameData) -> Box<dyn Mode> {
        if data.pressed.contains(Buttons::PAUSE) {
            // return to contained mode
            self.contains
        } else {
            // stay paused
            self
        }
    }

    fn render(&self, _interp: f32, data: &GlobalGameData, context: &mut dyn RenderContext) {
        let width = f32::from(context.width());
        let height = f32::from(context.height());
        let menu_start = width - Self::CLIP_WIDTH;
        // render what we've paused, clpping out the right side
        // note that pass in 1.0 for interpolation.
        // if we used the given interpolation, the scene would jitter between
        // the previous and current frame
        let mut new_context = ScissorContext::new(context, 0.0, 0.0, menu_start, height);
        self.contains.render(1.0, data, &mut new_context);
        // divider line
        context.line(menu_start, 0.0, menu_start, height);
        // draw "PAUSED" text
        data.font
            .write(context, menu_start + 8.0, 8.0, 4.0, "PAUSED");
        // options will be implemented later
    }
}
