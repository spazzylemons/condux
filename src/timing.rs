use crate::platform::Platform;

pub const TICKS_PER_SECOND: u8 = 60;
pub const TICK_DELTA: f32 = 1.0 / (TICKS_PER_SECOND as f32);

pub struct Timer {
    start_ms: u64,
    num_seconds: u64,
    tick_in_second: u8,
}

impl Timer {
    pub fn new(platform: &dyn Platform) -> Self {
        Self {
            start_ms: platform.time_msec(),
            num_seconds: 0,
            tick_in_second: 0,
        }
    }

    fn tick_ms(&self) -> u64 {
        self.start_ms
            + (1000 * self.num_seconds)
            + (1000 * u64::from(self.tick_in_second)) / (TICKS_PER_SECOND as u64)
    }

    /// Return the number of ticks to run for this frame, as well as interpolation
    pub fn frame_ticks(&mut self, platform: &dyn Platform) -> (u16, f32) {
        let millis = platform.time_msec();
        let mut ticks = 0;
        while millis >= self.tick_ms() {
            self.tick_in_second += 1;
            if self.tick_in_second == TICKS_PER_SECOND {
                self.tick_in_second = 0;
                self.num_seconds += 1;
            }
            ticks += 1;
        }
        let interp = (1.0 - (((self.tick_ms() - millis) as f32)
            * ((TICKS_PER_SECOND as f32) / 1000.0))).clamp(0.0, 1.0);
        (ticks, interp)
    }
}
