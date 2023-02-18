use std::{sync::Mutex};

use crate::bindings;

static TIMER: Mutex<Option<Timer>> = Mutex::new(None);

pub struct Timer {
    start_ms: u64,
    num_seconds: u64,
    tick_in_second: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_ms: unsafe { bindings::platform_time_msec() },
            num_seconds: 0,
            tick_in_second: 0,
        }
    }

    fn tick_ms(&self) -> u64 {
        self.start_ms
            + (1000 * self.num_seconds)
            + (1000 * u64::from(self.tick_in_second)) / (bindings::TICKS_PER_SECOND as u64)
    }

    /// Return the number of ticks to run for this frame, as well as interpolation
    pub fn frame_ticks(&mut self) -> (u16, f32) {
        let millis = unsafe { bindings::platform_time_msec() };
        let mut ticks = 0;
        while millis >= self.tick_ms() {
            self.tick_in_second += 1;
            if self.tick_in_second == bindings::TICKS_PER_SECOND as u8 {
                self.tick_in_second = 0;
                self.num_seconds += 1;
            }
            ticks += 1;
        }
        let interp = (1.0 - (((self.tick_ms() - millis) as f32)
            * ((bindings::TICKS_PER_SECOND as f32) / 1000.0))).clamp(0.0, 1.0);
        (ticks, interp)
    }
}

#[no_mangle]
pub extern "C" fn timing_init() {
    *TIMER.lock().unwrap() = Some(Timer::new());
}

#[no_mangle]
pub extern "C" fn timing_num_ticks(interpolation: *mut f32) -> u16 {
    let (ticks, interp) = TIMER.lock().unwrap().as_mut().unwrap().frame_ticks();
    unsafe {
        *interpolation = interp;
    }
    ticks
}
