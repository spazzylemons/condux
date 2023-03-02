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
    linalg::{Length, Mtx, Quat, Vector},
    platform::Buttons,
    render::graph::{RenderGraph, RenderGraph3d},
    spline::Spline,
};

use super::{loading::LoadingMode, title::TitleMode, GlobalGameData, Mode};

pub struct EditorMode {
    spline: Spline,
    focus_pos: Vector,
    rotation: Quat,
    last_mouse_x: i32,
    last_mouse_y: i32,
}

impl EditorMode {
    pub fn load() -> LoadingMode<EditorMode> {
        LoadingMode::new(|| {
            let spline = Spline::load(&mut Asset::load("course_test1.bin").unwrap()).unwrap();
            Self {
                spline,
                focus_pos: Vector::Z_AXIS,
                rotation: Quat::IDENT,
                last_mouse_x: 0,
                last_mouse_y: 0,
            }
        })
    }
}

impl Mode for EditorMode {
    fn tick(mut self: Box<Self>, data: &GlobalGameData) -> Box<dyn Mode> {
        // if back button pressed, quit editor
        if data.pressed.contains(Buttons::BACK) {
            return Box::new(TitleMode);
        }

        // if mouse down, pan camera
        if data.mouse_state.left() {
            let rotation_mtx = Mtx::from(self.rotation);
            let local_x = rotation_mtx * Vector::X_AXIS;
            let dx = data.mouse_state.x() - self.last_mouse_x;
            let dy = data.mouse_state.y() - self.last_mouse_y;
            self.rotation *= Quat::axis_angle(&Vector::Y_AXIS, (dx as f32) * 0.015);
            self.rotation *= Quat::axis_angle(&local_x, (dy as f32) * 0.015);

            let rotation_mtx = Mtx::from(self.rotation);
            let our_forward = rotation_mtx * Vector::Z_AXIS;

            let our_up = rotation_mtx * Vector::Y_AXIS;
            let test_up = if our_up.y > 0.02 {
                Some(Vector::Y_AXIS)
            } else if our_up.y < -0.02 {
                Some(-Vector::Y_AXIS)
            } else {
                None
            };

            if let Some(test_up) = test_up {
                let test_right = test_up.cross(&our_forward);
                let our_right = rotation_mtx * Vector::X_AXIS;
                let angle = our_right.signed_angle(&test_right, &our_forward);
                self.rotation *= Quat::axis_angle(&our_forward, angle);
            }
            self.rotation = self.rotation.normalized();
        }

        if data.scroll_wheel != 0 {
            let rotation_mtx = Mtx::from(self.rotation);
            let our_forward = rotation_mtx * Vector::Z_AXIS;

            self.focus_pos += our_forward * (data.scroll_wheel as f32) * 4.0;
        }

        // update mouse position
        self.last_mouse_x = data.mouse_state.x();
        self.last_mouse_y = data.mouse_state.y();

        self
    }

    fn render(
        &self,
        _interp: f32,
        _data: &GlobalGameData,
        graph: &mut RenderGraph,
        _width: u16,
        _height: u16,
    ) {
        // TODO interpolation
        let rotation_mtx = Mtx::from(self.rotation);
        let camera_pos = (rotation_mtx * (Vector::Z_AXIS * -10.0)) + self.focus_pos;
        let camera_up = rotation_mtx * Vector::Y_AXIS;
        let mut graph_3d = RenderGraph3d::new(camera_pos, self.focus_pos, camera_up);
        self.spline.render(&mut graph_3d, false);
        graph.graph_3d(graph_3d);
    }
}
