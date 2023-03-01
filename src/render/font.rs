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

use crate::assets::Asset;

use super::context::RenderContext;

#[derive(Default)]
struct Glyph {
    points: Vec<(u8, u8)>,
    lines: Vec<(u8, u8)>,
}

impl Glyph {
    fn load(asset: &mut Asset) -> Option<Glyph> {
        let ranges = asset.read_byte()?;
        let num_points = (ranges & 15) as usize;
        let num_lines = (ranges >> 4) as usize;
        let mut points = Vec::with_capacity(num_points);
        for _ in 0..num_points {
            let p = asset.read_byte()?;
            points.push((p & 15, p >> 4));
        }
        let mut lines = Vec::with_capacity(num_lines);
        for _ in 0..num_lines {
            let p = asset.read_byte()?;
            let (i, j) = (p & 15, p >> 4);
            if i as usize >= num_points || j as usize >= num_points {
                return None;
            }
            lines.push((i, j));
        }
        Some(Glyph { points, lines })
    }

    fn render(&self, context: &mut dyn RenderContext, x: f32, y: f32, scale: f32) {
        for (i, j) in &self.lines {
            let (x0, y0) = self.points[*i as usize];
            let (x1, y1) = self.points[*j as usize];
            let x0 = x + f32::from(x0) * scale;
            let x1 = x + f32::from(x1) * scale;
            let y0 = y + f32::from(y0) * scale;
            let y1 = y + f32::from(y1) * scale;
            context.line(x0, y0, x1, y1);
        }
    }
}

#[derive(Default)]
pub struct Font {
    glyphs: Vec<Glyph>,
}

impl Font {
    pub const GLYPH_SPACING: f32 = 5.0;

    pub fn new() -> Option<Self> {
        let mut asset = Asset::load("font.bin")?;
        let mut glyphs = vec![];
        for _ in 0..95 {
            glyphs.push(Glyph::load(&mut asset)?);
        }
        Some(Self { glyphs })
    }

    pub fn write(&self, context: &mut dyn RenderContext, mut x: f32, y: f32, scale: f32, s: &str) {
        for c in s.chars() {
            let codepoint = u32::from(c);
            if codepoint >= 0x20 {
                let codepoint = (codepoint - 0x20) as usize;
                if let Some(glyph) = self.glyphs.get(codepoint) {
                    glyph.render(context, x, y, scale);
                }
            }
            x += Self::GLYPH_SPACING * scale;
        }
    }
}
