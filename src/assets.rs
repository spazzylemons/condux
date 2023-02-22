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

use include_dir::{include_dir, Dir};

use crate::linalg::Vector;

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

pub struct Asset {
    contents: &'static [u8],
    index: usize,
}

impl Asset {
    #[must_use]
    pub fn load(name: &str) -> Option<Self> {
        let contents = ASSETS.get_file(name)?.contents();
        Some(Self { contents, index: 0 })
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.index >= self.contents.len() {
            return None;
        }
        let b = self.contents[self.index];
        self.index += 1;
        Some(b)
    }

    pub fn read_fixed(&mut self) -> Option<f32> {
        let lo = self.read_byte()?;
        let hi = self.read_byte()?;
        Some(f32::from(i16::from(lo) | (i16::from(hi) << 8)) / 256.0)
    }

    pub fn read_vector(&mut self) -> Option<Vector> {
        let x = self.read_fixed()?;
        let y = self.read_fixed()?;
        let z = self.read_fixed()?;
        Some(Vector::new(x, y, z))
    }
}
