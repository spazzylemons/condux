use std::{sync::Mutex, mem::zeroed, ffi::CString, ptr::null_mut, fmt::Write};

use crate::{linalg::{Vector, Mtx}, bindings};

#[derive(Default)]
struct Glyph {
    points: Vec<(u8, u8)>,
    lines: Vec<(u8, u8)>,
}

impl Glyph {
    fn load(asset: &mut bindings::Asset) -> Option<Glyph> {
        let mut ranges = 0u8;
        if !unsafe { bindings::asset_read_byte(asset as *mut bindings::Asset, &mut ranges as *mut u8) } {
            return None;
        }
        let num_points = (ranges & 15) as usize;
        let num_lines = (ranges >> 4) as usize;
        let mut points = Vec::with_capacity(num_points);
        for _ in 0..num_points {
            let mut p = 0u8;
            if !unsafe { bindings::asset_read_byte(asset as *mut bindings::Asset, &mut p as *mut u8) } {
                return None;
            }
            points.push((p & 15, p >> 4));
        }
        let mut lines = Vec::with_capacity(num_lines);
        for _ in 0..num_lines {
            let mut p = 0u8;
            if !unsafe { bindings::asset_read_byte(asset as *mut bindings::Asset, &mut p as *mut u8) } {
                return None;
            }
            let (i, j) = (p & 15, p >> 4);
            if i as usize >= num_points || j as usize >= num_points {
                return None;
            }
            lines.push((i, j));
        }
        Some(Glyph { points, lines })
    }

    fn render(&self, x: f32, y: f32, scale: f32) {
        for (i, j) in &self.lines {
            let (x0, y0) = self.points[*i as usize];
            let (x1, y1) = self.points[*j as usize];
            let x0 = x + f32::from(x0) * scale;
            let x1 = x + f32::from(x1) * scale;
            let y0 = y - f32::from(y0) * scale;
            let y1 = y - f32::from(y1) * scale;
            unsafe {
                bindings::platform_line(x0, y0, x1, y1);
            }
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

impl Renderer {
    pub fn set_camera(&mut self, eye: Vector, at: Vector, up: Vector) {
        self.camera_mtx = Mtx::looking_at(eye - at, up).transposed();
        self.camera_pos = eye;
    }

    pub fn load_glyphs(&mut self) {
        self.glyphs.clear();
        let mut asset = unsafe { zeroed::<bindings::Asset>() };
        let asset_name = CString::new("font.bin").unwrap();
        if unsafe { bindings::asset_load(&mut asset as *mut bindings::Asset, asset_name.as_ptr()) } {
            for _ in 0..95 {
                self.glyphs.push(Glyph::load(&mut asset).unwrap_or_default());
            }
        }
    }

    pub fn line(&self, a: Vector, b: Vector) {
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
        let width = unsafe { bindings::platform_width() } as f32;
        let height = unsafe { bindings::platform_height() } as f32;
        let scale = width.min(height);
        // draw it
        let x0 = scale * (a.x / a.z) + (width / 2.0);
        let y0 = (height / 2.0) - scale * (a.y / a.z);
        let x1 = scale * (b.x / b.z) + (width / 2.0);
        let y1 = (height / 2.0) - scale * (b.y / b.z);
        unsafe {
            bindings::platform_line(x0, y0, x1, y1);
        }
    }

    pub fn load_spline(&mut self, spline: &bindings::Spline) {
        self.spline_points.clear();
        let mut d = 0.0;
        while d < spline.length {
            let mut p = [0.0f32; 3];
            let mut r = [0.0f32; 3];
            unsafe {
                bindings::spline_get_baked(spline as *const bindings::Spline, d, &mut p as *mut f32);
                bindings::spline_get_up_right(spline as *const bindings::Spline, d, null_mut(), &mut r as *mut f32);
            }
            let p = Vector::from(p);
            let r = Vector::from(r) * bindings::SPLINE_TRACK_RADIUS as f32;
            self.spline_points.push((p - r, p + r));
            d += 1.0;
        }
    }

    pub fn render_spline(&self) {
        // in case no points loaded, don't draw
        if self.spline_points.len() == 0 {
            return;
        }

        for i in 0..self.spline_points.len() - 1 {
            let (l1, r1) = self.spline_points[i];
            let (l2, r2) = self.spline_points[i + 1];
            self.line(l1, l2);
            self.line(r1, r2);
            self.line(l1, r1);
        }
        // close the loop
        let (l1, r1) = self.spline_points[self.spline_points.len() - 1];
        let (l2, r2) = self.spline_points[0];
        self.line(l1, l2);
        self.line(r1, r2);
        self.line(l1, r1);
    }

    pub fn writer<'a>(&'a self, x: f32, y: f32, scale: f32) -> RendererWriter<'a> {
        RendererWriter { renderer: self, x, y, scale }
    }
}

pub struct RendererWriter<'a> {
    renderer: &'a Renderer,
    x: f32,
    y: f32,
    scale: f32,
}

impl<'a> Write for RendererWriter<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.chars() {
            let codepoint = u32::from(c);
            if codepoint >= 0x20 {
                let codepoint = (codepoint - 0x20) as usize;
                if codepoint < self.renderer.glyphs.len() {
                    self.renderer.glyphs[codepoint].render(self.x, self.y, self.scale);
                }
            }
            self.x += (bindings::GLYPH_SPACING as f32) * self.scale;
        }
        Ok(())
    }
}

macro_rules! render_write {
    ($dst:expr, $x:expr, $y:expr, $scale:expr, $($arg:tt)*) => {
        write!((($dst).writer($x, $y, $scale)), $($arg)*).unwrap()
    };
}

pub static RENDERER: Mutex<Renderer> = Mutex::new(Renderer {
    camera_pos: Vector::ZERO,
    camera_mtx: Mtx::IDENT,
    spline_points: vec![],
    glyphs: vec![],
});

impl bindings::Mesh {
    pub fn load(asset: &mut bindings::Asset) -> Option<bindings::Mesh> {
        let mut mesh = unsafe { zeroed::<bindings::Mesh>() };
        let mut num_vertices = 0u8;
        if !unsafe { bindings::asset_read_byte(asset as *mut bindings::Asset, &mut num_vertices as *mut u8) } {
            return None;
        }
        mesh.numVertices = num_vertices;
        let num_vertices = num_vertices as usize;
        if num_vertices > bindings::MAX_MESH_VERTICES as usize {
            return None;
        }
        for i in 0..num_vertices {
            if !unsafe { bindings::asset_read_vec(asset as *mut bindings::Asset, &mut mesh.vertices[i] as *mut f32) } {
                return None;
            }
        }
        let mut num_lines = 0u8;
        if !unsafe { bindings::asset_read_byte(asset as *mut bindings::Asset, &mut num_lines as *mut u8) } {
            return None;
        }
        mesh.numLines = num_lines;
        let num_lines = num_lines as usize;
        if num_lines > bindings::MAX_MESH_LINES as usize {
            return None;
        }
        for i in 0..num_lines {
            if !unsafe { bindings::asset_read_byte(asset as *mut bindings::Asset, &mut mesh.line1[i] as *mut u8) } {
                return None;
            }
            if mesh.line1[i] >= mesh.numVertices {
                return None;
            }
            if !unsafe { bindings::asset_read_byte(asset as *mut bindings::Asset, &mut mesh.line2[i] as *mut u8) } {
                return None;
            }
            if mesh.line2[i] >= mesh.numVertices {
                return None;
            }
        }
        Some(mesh)
    }

    pub fn render(&self, translation: Vector, rotation: Mtx) {
        for i in 0..self.numLines as usize {
            let a = Vector::from(self.vertices[self.line1[i] as usize]);
            let a = a * rotation + translation;
            let b = Vector::from(self.vertices[self.line2[i] as usize]);
            let b = b * rotation + translation;
            RENDERER.lock().unwrap().line(a, b);
        }
    }
}

#[no_mangle]
pub extern "C" fn render_init() {
    RENDERER.lock().unwrap().load_glyphs();
}

#[no_mangle]
pub extern "C" fn render_line(a: *const f32, b: *const f32) {
    RENDERER.lock().unwrap().line(Vector::from(a), Vector::from(b));
}

#[no_mangle]
pub extern "C" fn render_load_spline(spline: *const bindings::Spline) {
    RENDERER.lock().unwrap().load_spline(unsafe { &*spline });
}

#[no_mangle]
pub extern "C" fn render_spline() {
    RENDERER.lock().unwrap().render_spline();
}

#[no_mangle]
pub extern "C" fn mesh_load(mesh: *mut bindings::Mesh, asset: *mut bindings::Asset) -> bool {
    match bindings::Mesh::load(unsafe { &mut *asset }) {
        Some(v) => {
            unsafe {
                *mesh = v;
            }
            true
        },

        None => false
    }
}

#[no_mangle]
pub extern "C" fn mesh_render(mesh: *const bindings::Mesh, translation: *const f32, rotation: *const [f32; 3]) {
    unsafe { &*mesh }.render(Vector::from(translation), Mtx::from(rotation));
}
