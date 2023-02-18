use crate::bindings;

#[no_mangle]
pub extern "C" fn input_init() {
    unsafe {
        bindings::gControls.buttons = 0;
        bindings::gControls.steering = 0.0;
    }
}

#[no_mangle]
pub extern "C" fn input_poll() {
    unsafe {
        bindings::platform_poll(&mut bindings::gControls);
    }
}
