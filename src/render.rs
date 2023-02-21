use std::{fmt::Write};

use crate::{linalg::{Vector, Mtx}, spline::Spline, assets::Asset, platform::{Frame, Platform}};

pub struct Mesh {
    vertices: Vec<Vector>,
    lines: Vec<(u8, u8)>,
}

#[derive(Default)]
struct Glyph {
    points: Vec<(u8, u8)>,
    lines: Vec<(u8, u8)>,
}

const GLYPH_SPACING: f32 = 5.0;

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

    fn render(&self, x: f32, y: f32, scale: f32, frame: &mut Frame) {
        for (i, j) in &self.lines {
            let (x0, y0) = self.points[*i as usize];
            let (x1, y1) = self.points[*j as usize];
            let x0 = x + f32::from(x0) * scale;
            let x1 = x + f32::from(x1) * scale;
            let y0 = y - f32::from(y0) * scale;
            let y1 = y - f32::from(y1) * scale;
            frame.line(x0, y0, x1, y1);
        }
    }
}

pub struct Renderer {
    camera_pos: Vector,
    camera_mtx: Mtx,
    spline_points: Vec<(Vector, Vector)>,
    glyphs: Vec<Glyph>,
}

const CUTOFF: f32 = 0.01;

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            camera_pos: Vector::ZERO,
            camera_mtx: Mtx::IDENT,
            spline_points: vec![],
            glyphs: vec![],
        }
    }

    pub fn set_camera(&mut self, eye: Vector, at: Vector, up: Vector) {
        self.camera_mtx = Mtx::looking_at(eye - at, up).transposed();
        self.camera_pos = eye;
    }

    pub fn load_glyphs(&mut self) {
        self.glyphs.clear();
        if let Some(mut asset) = Asset::load("font.bin") {
            for _ in 0..95 {
                self.glyphs.push(Glyph::load(&mut asset).unwrap_or_default());
            }
        }
    }

    pub fn line(&self, a: Vector, b: Vector, frame: &mut Frame) {
        // perform camera transform
        let a = (a - self.camera_pos) * self.camera_mtx;
        let b = (b - self.camera_pos) * self.camera_mtx;
        if a.z < CUTOFF && b.z < CUTOFF {
            // lies entirely behind camera, don't draw it
            return;
        }
        // sort endpoints
        let (a, b) = if a.z > b.z { (b, a) } else { (a, b) };
        let a = if a.z < CUTOFF && b.z > CUTOFF {
            // if line crosses, we need to cut the line
            let n = (b.z - CUTOFF) / (b.z - a.z);
            (a * n) + (b * (1.0 - n))
        } else {
            // no cut
            a
        };
        // adjust for screen res
        let width = f32::from(frame.platform.width());
        let height = f32::from(frame.platform.height());
        let scale = width.min(height);
        // draw it
        let x0 = scale * (a.x / a.z) + (width / 2.0);
        let y0 = (height / 2.0) - scale * (a.y / a.z);
        let x1 = scale * (b.x / b.z) + (width / 2.0);
        let y1 = (height / 2.0) - scale * (b.y / b.z);
        frame.line(x0, y0, x1, y1);
    }

    pub fn load_spline(&mut self, spline: &Spline) {
        self.spline_points.clear();
        let mut d = 0.0;
        while d < spline.length {
            let p = spline.get_baked(d);
            let (_, r) = spline.get_up_right(d);
            let r = r * Spline::TRACK_RADIUS;
            self.spline_points.push((p - r, p + r));
            d += 1.0;
        }
    }

    pub fn render_spline(&self, frame: &mut Frame) {
        // in case no points loaded, don't draw
        if self.spline_points.is_empty() {
            return;
        }

        for i in 0..self.spline_points.len() - 1 {
            let (l1, r1) = self.spline_points[i];
            let (l2, r2) = self.spline_points[i + 1];
            self.line(l1, l2, frame);
            self.line(r1, r2, frame);
            self.line(l1, r1, frame);
        }
        // close the loop
        let (l1, r1) = self.spline_points[self.spline_points.len() - 1];
        let (l2, r2) = self.spline_points[0];
        self.line(l1, l2, frame);
        self.line(r1, r2, frame);
        self.line(l1, r1, frame);
    }

    pub fn writer<'a, 'b, 'c>(&'a self, x: f32, y: f32, scale: f32, frame: &'b mut Frame<'c>) -> RendererWriter<'a, 'b, 'c> {
        RendererWriter { renderer: self, x, y, scale, frame }
    }
}

pub struct RendererWriter<'a, 'b, 'c> {
    renderer: &'a Renderer,
    x: f32,
    y: f32,
    scale: f32,
    frame: &'b mut Frame<'c>,
}

impl<'a, 'b, 'c> Write for RendererWriter<'a, 'b, 'c> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.chars() {
            let codepoint = u32::from(c);
            if codepoint >= 0x20 {
                let codepoint = (codepoint - 0x20) as usize;
                if codepoint < self.renderer.glyphs.len() {
                    self.renderer.glyphs[codepoint].render(self.x, self.y, self.scale, self.frame);
                }
            }
            self.x += GLYPH_SPACING * self.scale;
        }
        Ok(())
    }
}

macro_rules! render_write {
    ($dst:expr, $x:expr, $y:expr, $scale:expr, $frame:expr, $($arg:tt)*) => {
        write!((($dst).writer($x, $y, $scale, $frame)), $($arg)*).unwrap()
    };
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

    pub fn render(&self, renderer: &Renderer, translation: Vector, rotation: Mtx, frame: &mut Frame) {
        for (x, y) in &self.lines {
            let a = self.vertices[*x as usize];
            let a = a * rotation + translation;
            let b = self.vertices[*y as usize];
            let b = b * rotation + translation;
            renderer.line(a, b, frame);
        }
    }
}
