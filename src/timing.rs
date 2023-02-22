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

use crate::platform::{Impl, Platform};

pub const TICKS_PER_SECOND: u8 = 60;
pub const TICK_DELTA: f32 = 1.0 / (TICKS_PER_SECOND as f32);

pub struct Timer {
    start_ms: u64,
    num_seconds: u64,
    tick_in_second: u8,
}

impl Timer {
    pub fn new(platform: &Impl) -> Self {
        Self {
            start_ms: platform.time_msec(),
            num_seconds: 0,
            tick_in_second: 0,
        }
    }

    fn tick_ms(&self) -> u64 {
        self.start_ms
            + (1000 * self.num_seconds)
            + (1000 * u64::from(self.tick_in_second)) / u64::from(TICKS_PER_SECOND)
    }

    /// Return the number of ticks to run for this frame, as well as interpolation
    pub fn frame_ticks(&mut self, platform: &Impl) -> (u16, f32) {
        let millis = platform.time_msec();
        let mut ticks = 0;
        while millis >= self.tick_ms() {
            self.tick_in_second += 1;
            if self.tick_in_second == TICKS_PER_SECOND {
                self.tick_in_second = 0;
                self.num_seconds += 1;
            }
            ticks += 1;
        }
        let interp = (1.0
            - (((self.tick_ms() - millis) as f32) * (f32::from(TICKS_PER_SECOND) / 1000.0)))
            .clamp(0.0, 1.0);
        (ticks, interp)
    }
}
