use std::error::Error;

use bitflags::bitflags;

pub struct Frame<'a> {
    lines: Vec<((f32, f32), (f32, f32))>,
    pub platform: &'a mut dyn Platform,
}

bitflags! {
    pub struct Buttons: u8 {
        const UP = 1 << 0;
        const DOWN = 1 << 1;
        const LEFT = 1 << 2;
        const RIGHT = 1 << 3;
        const OK = 1 << 4;
        const BACK = 1 << 5;
        const PAUSE = 1 << 6;
    }
}

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
    fn init(preferred_width: u16, preferred_height: u16) -> Result<Self, Box<dyn Error>>
    where Self: Sized;

    fn should_run(&self) -> bool;

    fn time_msec(&self) -> u64;

    fn start_frame<'a>(&'a mut self) -> Frame<'a> where Self: Sized {
        Frame {
            lines: vec![],
            platform: self,
        }
    }

    fn end_frame<'a>(&mut self, lines: &[((f32, f32), (f32, f32))]);

    fn width(&self) -> u16;

    fn height(&self) -> u16;

    fn poll(&mut self) -> Controls;
}

#[cfg(target_os = "horizon")]
pub mod ctr;
#[cfg(not(target_os = "horizon"))]
pub mod sdl;

#[cfg(target_os = "horizon")]
pub type PlatformImpl = ctr::CitroPlatform;
#[cfg(not(target_os = "horizon"))]
pub type PlatformImpl = sdl::SdlPlatform;
