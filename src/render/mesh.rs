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
    linalg::{Mtx, Vector},
};

use super::context::RenderContext3d;

#[derive(Clone)]
pub struct Mesh {
    vertices: Vec<Vector>,
    lines: Vec<(u8, u8)>,
}

impl Mesh {
    pub fn load(asset: &mut Asset) -> Option<Self> {
        let num_vertices = asset.read_byte()?;
        let mut vertices = vec![];
        for _ in 0..num_vertices {
            vertices.push(asset.read_vector()?);
        }
        let num_lines = asset.read_byte()?;
        let mut lines = vec![];
        for _ in 0..num_lines {
            let x = asset.read_byte()?;
            if x >= num_vertices {
                return None;
            }
            let y = asset.read_byte()?;
            if y >= num_vertices {
                return None;
            }
            lines.push((x, y));
        }
        Some(Self { vertices, lines })
    }

    pub fn render(&self, context: &mut RenderContext3d, translation: Vector, rotation: Mtx) {
        for (x, y) in &self.lines {
            let a = self.vertices[*x as usize];
            let a = a * rotation + translation;
            let b = self.vertices[*y as usize];
            let b = b * rotation + translation;
            context.line(a, b);
        }
    }
}
