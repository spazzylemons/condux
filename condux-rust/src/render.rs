use crate::linalg::{Vector, Mtx};

extern "C" {
    static mut camera_mtx: [[f32; 3]; 3];
    static mut camera_pos: [f32; 3];
}

#[no_mangle]
extern "C" fn set_camera(eye: *const f32, at: *const f32, up: *const f32) {
    let delta = Vector::from(eye) - Vector::from(at);
    let mtx = Mtx::look_at(delta, Vector::from(up)).transposed();
    unsafe {
        mtx.write_to_ptr(camera_mtx.as_mut_ptr());
        Vector::from(eye).write_to_ptr(camera_pos.as_mut_ptr());
    }
}
