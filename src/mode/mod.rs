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

pub mod pause;
pub mod race;
pub mod title;

use crate::{
    platform::{Buttons, Controls},
    render::{context::RenderContext, Font},
    vehicle::garage::Garage,
};

#[derive(Default)]
pub struct GlobalGameData {
    /// The last state of the controls.
    pub controls: Controls,
    /// The buttons that have been pressed this frame.
    pub pressed: Buttons,
    /// The vehicle models.
    pub garage: Garage,
    /// The font.
    pub font: Font,
}

/// A game mode.
pub trait Mode {
    /// Update the state. If a new mode is to be transitioned to, then returns
    /// the new mode to replace this with, which should have the same lifetime.
    fn tick(self: Box<Self>, data: &GlobalGameData) -> Box<dyn Mode>;

    /// Render this mode.
    fn render(&self, interp: f32, data: &GlobalGameData, context: &mut dyn RenderContext);
}
