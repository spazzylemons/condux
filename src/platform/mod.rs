use bitflags::bitflags;

pub type Point = (f32, f32);
pub type Line = (Point, Point);

pub struct Frame<'a> {
    lines: Vec<Line>,
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

#[derive(Clone, Copy)]
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
    fn init(preferred_width: u16, preferred_height: u16) -> Self where Self: Sized;

    fn should_run(&self) -> bool;

    fn time_msec(&self) -> u64;

    fn start_frame(&mut self) -> Frame<'_> where Self: Sized {
        Frame {
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
#[cfg(not(target_os = "horizon"))]
pub mod sdl;

#[cfg(target_os = "horizon")]
pub type Impl = ctr::CitroPlatform;
#[cfg(not(target_os = "horizon"))]
pub type Impl = sdl::SdlPlatform;
