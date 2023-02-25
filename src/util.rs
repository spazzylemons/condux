use std::ops::{AddAssign, DivAssign, MulAssign};

use crate::timing::TICK_DELTA;

pub trait Approach: Sized + DivAssign<f32> + AddAssign<Self> + MulAssign<f32> {
    #[must_use]
    fn approach(mut self, mut strength: f32, mut to: Self) -> Self {
        strength *= TICK_DELTA;
        to *= strength;
        strength += 1.0;
        self /= strength;
        self += to;
        self
    }

    fn approach_mut(&mut self, mut strength: f32, mut to: Self) {
        strength *= TICK_DELTA;
        to *= strength;
        strength += 1.0;
        *self /= strength;
        *self += to;
    }
}

impl<T: Sized + DivAssign<f32> + AddAssign<T> + MulAssign<f32>> Approach for T {}
