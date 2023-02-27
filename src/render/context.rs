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
    linalg::{Mtx, Vector},
    platform::Platform,
};

pub type Point2d = (f32, f32);
pub type Line2d = (Point2d, Point2d);

pub trait RenderContext {
    fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32);

    fn width(&self) -> u16;

    fn height(&self) -> u16;
}

pub struct GenericBaseContext<'a, P>
where
    P: Platform,
{
    lines: Vec<Line2d>,
    platform: &'a mut P,
}

impl<'a, P> GenericBaseContext<'a, P>
where
    P: Platform + Sized,
{
    pub fn new(platform: &'a mut P) -> Self {
        Self {
            lines: vec![],
            platform,
        }
    }

    pub fn finish(self) {
        self.platform.end_frame(&self.lines);
    }
}

impl<'a, P> RenderContext for GenericBaseContext<'a, P>
where
    P: Platform,
{
    fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.lines.push(((x0, y0), (x1, y1)));
    }

    fn width(&self) -> u16 {
        self.platform.width()
    }

    fn height(&self) -> u16 {
        self.platform.height()
    }
}

pub struct ScissorContext<'a> {
    parent: &'a mut dyn RenderContext,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl<'a> ScissorContext<'a> {
    pub fn new(
        parent: &'a mut dyn RenderContext,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    ) -> Self {
        Self {
            parent,
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    fn clip(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Option<(Point2d, Point2d)> {
        // liang-barsky
        let dx = x1 - x0;
        let dy = y1 - y0;
        let clip = (0.0, 1.0);
        let clip = test_edge(clip, -dx, x0 - self.min_x)?;
        let clip = test_edge(clip, dx, self.max_x - x0)?;
        let clip = test_edge(clip, -dy, y0 - self.min_y)?;
        let clip = test_edge(clip, dy, self.max_y - y0)?;
        let (min, max) = clip;
        Some((
            (x0 + min * dx, y0 + min * dy),
            (x0 + max * dx, y0 + max * dy),
        ))
    }
}

fn test_edge((mut min, mut max): (f32, f32), p: f32, q: f32) -> Option<(f32, f32)> {
    if p.abs() == 0.0 {
        // parallel
        if q < 0.0 {
            // outside
            return None;
        }
    } else {
        let r = q / p;
        if p < 0.0 {
            if r > max {
                return None;
            } else if r > min {
                min = r;
            }
        } else {
            if r < min {
                return None;
            } else if r < max {
                max = r;
            }
        }
    }
    Some((min, max))
}

impl<'a> RenderContext for ScissorContext<'a> {
    fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        if let Some(((x0, y0), (x1, y1))) = self.clip(x0, y0, x1, y1) {
            self.parent.line(x0, y0, x1, y1);
        }
    }

    fn width(&self) -> u16 {
        self.parent.width()
    }

    fn height(&self) -> u16 {
        self.parent.height()
    }
}

pub struct RenderContext3d<'a> {
    context: &'a mut dyn RenderContext,
    camera_pos: Vector,
    camera_mtx: Mtx,
}

impl<'a> RenderContext3d<'a> {
    const CUTOFF: f32 = 0.01;

    pub fn new(context: &'a mut dyn RenderContext, eye: Vector, at: Vector, up: Vector) -> Self {
        Self {
            context,
            camera_pos: eye,
            camera_mtx: Mtx::looking_at(eye - at, up).transposed(),
        }
    }

    pub fn line(&mut self, a: Vector, b: Vector) {
        // perform camera transform
        let a = (a - self.camera_pos) * self.camera_mtx;
        let b = (b - self.camera_pos) * self.camera_mtx;
        if a.z < Self::CUTOFF && b.z < Self::CUTOFF {
            // lies entirely behind camera, don't draw it
            return;
        }
        // sort endpoints
        let (a, b) = if a.z > b.z { (b, a) } else { (a, b) };
        let a = if a.z < Self::CUTOFF && b.z > Self::CUTOFF {
            // if line crosses, we need to cut the line
            let n = (b.z - Self::CUTOFF) / (b.z - a.z);
            (a * n) + (b * (1.0 - n))
        } else {
            // no cut
            a
        };
        // adjust for screen res
        let width = f32::from(self.context.width());
        let height = f32::from(self.context.height());
        let scale = width.min(height);
        // draw it
        let x0 = scale * (a.x / a.z) + (width / 2.0);
        let y0 = (height / 2.0) - scale * (a.y / a.z);
        let x1 = scale * (b.x / b.z) + (width / 2.0);
        let y1 = (height / 2.0) - scale * (b.y / b.z);
        self.context.line(x0, y0, x1, y1);
    }
}
