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

use crate::render::context::{GenericBaseContext, Line2d};

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

pub trait Platform {
    fn init(preferred_width: u16, preferred_height: u16) -> Self
    where
        Self: Sized;

    fn should_run(&self) -> bool;

    fn time_msec(&self) -> u64;

    fn start_frame(&mut self) -> GenericBaseContext<'_, Self>
    where
        Self: Sized,
    {
        GenericBaseContext::<'_, Self>::new(self)
    }

    fn end_frame(&mut self, lines: &[Line2d]);

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
