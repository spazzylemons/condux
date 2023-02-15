use crate::{linalg::{Vector, Mtx}, platform::{platform_width, platform_height, platform_line}, assets::{Asset, asset_read_byte, asset_read_vec}};

#[repr(C)]
pub struct Mesh {
    pub num_vertices: u8,
    pub vertices: [[f32; 3]; Mesh::MAX_VERTICES],

    pub num_lines: u8,
    pub line1: [u8; Mesh::MAX_LINES],
    pub line2: [u8; Mesh::MAX_LINES],
}

impl Mesh {
    pub const MAX_VERTICES: usize = 32;
    pub const MAX_LINES: usize = 64;
}

// TODO safety here
static mut CAMERA_MTX: Mtx = Mtx::IDENT;
static mut CAMERA_POS: Vector = Vector::ZERO;

const CUTOFF: f32 = 0.01;

#[no_mangle]
extern "C" fn set_camera(eye: *const f32, at: *const f32, up: *const f32) {
    let delta = Vector::from(eye) - Vector::from(at);
    let mtx = Mtx::look_at(delta, Vector::from(up)).transposed();
    unsafe {
        CAMERA_MTX = mtx;
        CAMERA_POS = eye.into();
    }
}

fn render_line_rust(a: Vector, b: Vector) {
    // perform camera transform
    let c = unsafe { CAMERA_POS };
    let m = unsafe { CAMERA_MTX };
    let a = (a - c) * m;
    let b = (b - c) * m;
    if a.z < CUTOFF && b.z < CUTOFF {
        // lies entirely behind camera, don't draw it
        return;
    }
    // sort endpoints
    let (mut a, b) = if a.z > b.z { (b, a) } else { (a, b) };
    if a.z < CUTOFF && b.z > CUTOFF {
        // if line crosses, we need to cut the line
        let n = (b.z - CUTOFF) / (b.z - a.z);
        a = (a * n) + (b * (1.0 - n));
    }
    // adjust for screen res
    let width = unsafe { platform_width() } as f32;
    let height = unsafe { platform_height() } as f32;
    let scale = if width < height { width } else { height };
    // draw it
    let x0 = scale * (a.x / a.z) + (width / 2.0);
    let y0 = (height / 2.0) - scale * (a.y / a.z);
    let x1 = scale * (b.x / b.z) + (width / 2.0);
    let y1 = (height / 2.0) - scale * (b.y / b.z);
    unsafe {
        platform_line(x0, y0, x1, y1);
    }
}

#[no_mangle]
extern "C" fn render_line(a: *const f32, b: *const f32) {
    let a = Vector::from(a);
    let b = Vector::from(b);
    render_line_rust(a, b);
}

#[no_mangle]
extern "C" fn mesh_load(mesh: *mut Mesh, asset: *mut Asset) -> bool {
    let mesh = unsafe { &mut *mesh };
    if !unsafe { asset_read_byte(asset, &mut mesh.num_vertices as *mut u8) } {
        return false;
    }
    if mesh.num_vertices as usize > Mesh::MAX_VERTICES {
        return false;
    }
    for i in 0..mesh.num_vertices {
        if !unsafe { asset_read_vec(asset, &mut mesh.vertices[i as usize] as *mut f32) } {
            return false;
        }
    }
    if !unsafe { asset_read_byte(asset, &mut mesh.num_lines as *mut u8) } {
        return false;
    }
    if mesh.num_lines as usize > Mesh::MAX_LINES {
        return false;
    }
    for i in 0..mesh.num_lines {
        if !unsafe { asset_read_byte(asset, &mut mesh.line1[i as usize] as *mut u8) } {
            return false;
        }
        if mesh.line1[i as usize] >= mesh.num_vertices {
            return false;
        }
        if !unsafe { asset_read_byte(asset, &mut mesh.line2[i as usize] as *mut u8) } {
            return false;
        }
        if mesh.line2[i as usize] >= mesh.num_vertices {
            return false;
        }
    }
    true
}

#[no_mangle]
extern "C" fn mesh_render(mesh: *const Mesh, translation: *const f32, rotation: *const [f32; 3]) {
    let translation = Vector::from(translation);
    let rotation = Mtx::from(rotation);
    let mesh = unsafe { &*mesh };
    for i in 0..mesh.num_lines {
        let v1 = Vector::from(mesh.vertices[mesh.line1[i as usize] as usize]);
        let v2 = Vector::from(mesh.vertices[mesh.line2[i as usize] as usize]);
        render_line_rust(v1 * rotation + translation, v2 * rotation + translation);
    }
}
