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

use bitflags::bitflags;

pub type Point = (f32, f32);
pub type Line = (Point, Point);

pub struct GenericFrame<'a, P>
where
    P: Platform,
{
    lines: Vec<Line>,
    pub platform: &'a mut P,
}

pub type Frame<'a> = GenericFrame<'a, Impl>;

bitflags! {
    #[derive(Default)]
    pub struct Buttons: u8 {
        const UP    = 1 << 0;
        const DOWN  = 1 << 1;
        const LEFT  = 1 << 2;
        const RIGHT = 1 << 3;
        const OK    = 1 << 4;
        const BACK  = 1 << 5;
        const PAUSE = 1 << 6;
    }
}

#[derive(Clone, Copy, Default)]
pub struct Controls {
    pub buttons: Buttons,
    pub steering: f32,
}

impl<'a> Frame<'a> {
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.lines.push(((x0, y0), (x1, y1)));
    }

    pub fn finish(self) {
        self.platform.end_frame(&self.lines);
    }
}

pub trait Platform {
    fn init(preferred_width: u16, preferred_height: u16) -> Self
    where
        Self: Sized;

    fn should_run(&self) -> bool;

    fn time_msec(&self) -> u64;

    fn start_frame(&mut self) -> GenericFrame<'_, Self>
    where
        Self: Sized,
    {
        GenericFrame::<'_, Self> {
            lines: vec![],
            platform: self,
        }
    }

    fn end_frame(&mut self, lines: &[Line]);

    fn width(&self) -> u16;

    fn height(&self) -> u16;

    fn poll(&mut self) -> Controls;
}

#[cfg(target_os = "horizon")]
pub mod ctr;

#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(not(any(target_os = "horizon", target_arch = "wasm32")))]
pub mod sdl;

#[cfg(target_os = "horizon")]
pub type Impl = ctr::CitroPlatform;

#[cfg(target_arch = "wasm32")]
pub type Impl = web::WebPlatform;

#[cfg(not(any(target_os = "horizon", target_arch = "wasm32")))]
pub type Impl = sdl::SdlPlatform;
