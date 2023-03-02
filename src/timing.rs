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

pub const TICKS_PER_SECOND: u8 = 60;
pub const TICK_DELTA: f32 = 1.0 / (TICKS_PER_SECOND as f32);

/// Wraps Instant for non-WASM targets.
#[cfg(not(target_arch = "wasm32"))]
struct InstantWrapper {
    start: std::time::Instant,
}

#[cfg(not(target_arch = "wasm32"))]
impl InstantWrapper {
    fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    fn millis(&self) -> u128 {
        std::time::Instant::now()
            .duration_since(self.start)
            .as_millis()
    }
}

/// Simulates Instant using Performance.now() for WASM target.
#[cfg(target_arch = "wasm32")]
struct InstantWrapper {
    performance: web_sys::Performance,
    start: f64,
}

#[cfg(target_arch = "wasm32")]
impl InstantWrapper {
    fn new() -> Self {
        let performance = web_sys::window().unwrap().performance().unwrap();
        let start = performance.now();
        Self { performance, start }
    }

    fn millis(&self) -> u128 {
        (self.performance.now() - self.start) as _
    }
}

pub struct Timer {
    start: InstantWrapper,
    num_seconds: u128,
    tick_in_second: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: InstantWrapper::new(),
            num_seconds: 0,
            tick_in_second: 0,
        }
    }

    fn tick_ms(&self) -> u128 {
        (1000 * self.num_seconds)
            + u128::from((1000 * u16::from(self.tick_in_second)) / u16::from(TICKS_PER_SECOND))
    }

    /// Return the number of ticks to run for this frame, as well as interpolation
    pub fn frame_ticks(&mut self) -> (u16, f32) {
        let millis = self.start.millis();
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
