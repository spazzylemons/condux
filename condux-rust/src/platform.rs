#[repr(C)]
pub struct Controls {
    pub buttons: u8,
    pub steering: f32,
}

extern "C" {
    pub fn platform_init(preferred_width: i32, preferred_height: i32);
    pub fn platform_deinit();
    pub fn platform_line(x0: f32, y0: f32, x1: f32, y1: f32);
    pub fn platform_should_run() -> bool;
    pub fn platform_time_msec() -> u64;
    pub fn platform_start_frame();
    pub fn platform_end_frame();
    pub fn platform_width() -> i32;
    pub fn platform_height() -> i32;
    pub fn platform_poll(controls: *mut Controls);
}
