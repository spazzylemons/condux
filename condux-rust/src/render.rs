extern "C" {
    pub fn vec_copy(dst: *mut f32, src: *const f32);

    pub fn vec_sub(dst: *mut f32, src: *const f32);

    pub fn mtx_look_at(m: *mut [f32; 3], at: *const f32, up: *const f32);

    pub fn mtx_transpose(m: *mut [f32; 3]);

    static mut camera_mtx: [[f32; 3]; 3];
    static mut camera_pos: [f32; 3];
}

#[no_mangle]
extern "C" fn set_camera(eye: *const f32, at: *const f32, up: *const f32) {
    let mut delta = [0.0, 0.0, 0.0];
    unsafe {
        vec_copy((&mut delta).as_mut_ptr(), eye);
        vec_sub((&mut delta).as_mut_ptr(), at);
        mtx_look_at(camera_mtx.as_mut_ptr(), (&delta).as_ptr(), up);
        mtx_transpose(camera_mtx.as_mut_ptr());
        vec_copy(camera_pos.as_mut_ptr(), eye);
    }
}
