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

#[cfg(not(any(target_os = "horizon", target_arch = "wasm32")))]
pub mod editor;

pub mod loading;
pub mod pause;
pub mod race;
pub mod title;

use std::cell::Cell;

use crate::{
    platform::{Buttons, Controls},
    render::graph::RenderGraph,
    vehicle::garage::Garage,
};

pub struct GlobalGameData {
    /// The last state of the controls.
    pub controls: Controls,
    /// The buttons that have been pressed this frame.
    pub pressed: Buttons,
    /// The vehicle models.
    pub garage: Garage,
    /// If true, enable walls. Stored in a Cell so it may be modified easily.
    pub walls: Cell<bool>,
    /// If false, the game stops running. Not present on web target.
    #[cfg(not(target_arch = "wasm32"))]
    pub should_run: Cell<bool>,
    /// The mouse state, used for the editor.
    #[cfg(not(any(target_arch = "wasm32", target_os = "horizon")))]
    pub mouse_state: sdl2::mouse::MouseState,
    /// The mouse scroll wheel this frame.
    #[cfg(not(any(target_arch = "wasm32", target_os = "horizon")))]
    pub scroll_wheel: i32,
}

impl Default for GlobalGameData {
    fn default() -> Self {
        Self {
            controls: Controls::default(),
            pressed: Buttons::default(),
            garage: Garage::default(),
            walls: Cell::<bool>::default(),
            #[cfg(not(target_arch = "wasm32"))]
            should_run: Cell::<bool>::default(),
            #[cfg(not(any(target_arch = "wasm32", target_os = "horizon")))]
            mouse_state: sdl2::mouse::MouseState::from_sdl_state(0),
            #[cfg(not(any(target_arch = "wasm32", target_os = "horizon")))]
            scroll_wheel: 0,
        }
    }
}

/// A game mode.
pub trait Mode {
    /// Update the state. If a new mode is to be transitioned to, then returns
    /// the new mode to replace this with, which should have the same lifetime.
    fn tick(self: Box<Self>, data: &GlobalGameData) -> Box<dyn Mode>;

    /// Render this mode.
    fn render(
        &self,
        interp: f32,
        data: &GlobalGameData,
        graph: &mut RenderGraph,
        width: u16,
        height: u16,
    );
}
