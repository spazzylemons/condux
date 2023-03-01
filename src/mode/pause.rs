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
    render::{graph::RenderGraph, Font},
};

use super::{GlobalGameData, Mode};

/// An action that a menu option can take.
pub enum MenuAction {
    /// Action switches to previous mode.
    Previous,
    /// Action loads new mode.
    Switch(Box<dyn Fn(&GlobalGameData) -> Box<dyn Mode>>),
    /// Action uses global data but does not switch mode.
    Data(Box<dyn Fn(&GlobalGameData)>),
}

/// An option in the pause menu.
pub struct MenuOption {
    /// The name of the option.
    name: String,
    /// The action to take when this option is selected.
    /// This function takes in the previous mode and returns a new mode to switch to.
    action: MenuAction,
}

impl MenuOption {
    pub fn previous(name: String) -> Self {
        Self {
            name,
            action: MenuAction::Previous,
        }
    }

    pub fn switch<F>(name: String, f: F) -> Self
    where
        F: Fn(&GlobalGameData) -> Box<dyn Mode> + 'static,
    {
        Self {
            name,
            action: MenuAction::Switch(Box::new(f)),
        }
    }

    pub fn data<F>(name: String, f: F) -> Self
    where
        F: Fn(&GlobalGameData) + 'static,
    {
        Self {
            name,
            action: MenuAction::Data(Box::new(f)),
        }
    }
}

/// The pause menu mode.
pub struct PauseMode {
    contains: Box<dyn Mode>,
    options: Vec<MenuOption>,
    selected: usize,
}

impl PauseMode {
    pub fn new(contains: Box<dyn Mode>, options: Vec<MenuOption>) -> Self {
        Self {
            contains,
            options,
            selected: 0,
        }
    }

    const CLIP_WIDTH: f32 = 240.0;

    const OPTION_SCALE: f32 = 3.0;
}

impl Mode for PauseMode {
    fn tick(mut self: Box<Self>, data: &GlobalGameData) -> Box<dyn Mode> {
        if data.pressed.contains(Buttons::PAUSE) {
            // return to contained mode
            return self.contains;
        }

        if data.pressed.contains(Buttons::UP) && self.selected > 0 {
            // previous option
            self.selected -= 1;
        } else if data.pressed.contains(Buttons::DOWN) && self.selected + 1 < self.options.len() {
            // next option
            self.selected += 1;
        }

        if data.pressed.contains(Buttons::OK) {
            // select this option
            let option = &self.options[self.selected];
            return match &option.action {
                MenuAction::Previous => self.contains,
                MenuAction::Switch(f) => f(data),
                MenuAction::Data(f) => {
                    f(data);
                    self
                }
            };
        }

        // stay paused
        self
    }

    fn render(
        &self,
        _interp: f32,
        data: &GlobalGameData,
        graph: &mut RenderGraph,
        width: u16,
        height: u16,
    ) {
        let width_f32 = f32::from(width);
        let height_f32 = f32::from(height);
        let menu_start = width_f32 - Self::CLIP_WIDTH;
        // render what we've paused, clpping out the right side
        // note that pass in 1.0 for interpolation.
        // if we used the given interpolation, the scene would jitter between
        // the previous and current frame
        let mut new_graph = RenderGraph::default();
        self.contains
            .render(1.0, data, &mut new_graph, width, height);
        graph.scissor(0.0, 0.0, menu_start, height_f32, new_graph);
        // divider line
        graph.line(menu_start, 0.0, menu_start, height_f32);
        // draw "PAUSED" text
        graph.text(menu_start + 16.0, 16.0, 4.0, String::from("PAUSED"));
        // draw options
        let mut y = 64.0;
        for (i, option) in self.options.iter().enumerate() {
            let mut x = menu_start + 16.0;
            // show cursor where we're selecting
            if i == self.selected {
                graph.text(x, y, Self::OPTION_SCALE, String::from(">"));
                x += Font::GLYPH_SPACING * 2.0 * Self::OPTION_SCALE;
            }
            graph.text(x, y, Self::OPTION_SCALE, option.name.clone());
            y += Self::OPTION_SCALE * 8.0;
        }
    }
}
