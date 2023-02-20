use std::{ops::{Add, Sub, Mul, AddAssign, SubAssign, MulAssign, Div, DivAssign, Neg}};

use crate::timing::TICK_DELTA;

/// Mixin that provides all length-related methods for both Vector and Quat.
pub trait Length: Sized + Sub<Self, Output = Self> + DivAssign<f32> {
    fn dot(&self, other: &Self) -> f32;

    fn mag_sq(&self) -> f32 {
        self.dot(self)
    }

    fn mag(&self) -> f32 {
        self.mag_sq().sqrt()
    }

    fn dist_sq(self, other: Self) -> f32 {
        (self - other).mag_sq()
    }

    fn dist(self, other: Self) -> f32 {
        self.dist_sq(other).sqrt()
    }

    fn normalized(mut self) -> Self {
        let m = self.mag();
        if m != 0.0 {
            self /= m;
        }
        self
    }
}

macro_rules! auto_assign {
    ($name:ident, $func:ident, $assign_func:ident, $t:ty, $u:ty) => {
        impl $name<$u> for $t {
            type Output = $t;

            fn $func(mut self, rhs: $u) -> $t {
                self.$assign_func(rhs);
                self
            }
        }
    };
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const X_AXIS: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y_AXIS: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z_AXIS: Self = Self::new(0.0, 0.0, 1.0);

    pub const MIN: Self = Self::new(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY);
    pub const MAX: Self = Self::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - other.y * self.z,
            self.z * other.x - other.z * self.x,
            self.x * other.y - other.x * self.y,
        )
    }

    pub fn approach(mut self, strength: f32, to: &Self) -> Self {
        let strength = strength * TICK_DELTA;
        self /= 1.0 + strength;
        self += *to * strength;
        self
    }

    pub fn signed_angle(&self, to: &Self, axis: &Self) -> f32 {
        let cross = self.cross(to);
        let unsigned = cross.mag().atan2(self.dot(to));
        unsigned.copysign(-cross.dot(axis))
    }
}

impl Length for Vector {
    fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(mut self) -> Self {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }
}

impl AddAssign<Vector> for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

auto_assign! { Add, add, add_assign, Vector, Vector }

impl SubAssign<Vector> for Vector {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

auto_assign! { Sub, sub, sub_assign, Vector, Vector }

impl MulAssign<f32> for Vector {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

auto_assign! { Mul, mul, mul_assign, Vector, f32 }

impl MulAssign<Mtx> for Vector {
    fn mul_assign(&mut self, rhs: Mtx) {
        let x = self.x * rhs.xx + self.y * rhs.yx + self.z * rhs.zx;
        let y = self.x * rhs.xy + self.y * rhs.yy + self.z * rhs.zy;
        let z = self.x * rhs.xz + self.y * rhs.yz + self.z * rhs.zz;
        *self = Self::new(x, y, z);
    }
}

auto_assign! { Mul, mul, mul_assign, Vector, Mtx }

impl DivAssign<f32> for Vector {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

auto_assign! { Div, div, div_assign, Vector, f32 }

#[derive(Clone, Copy, Default)]
pub struct Quat {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quat {
    pub const IDENT: Self = Self::new(1.0, 0.0, 0.0, 0.0);

    pub const fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    pub fn axis_angle(axis: &Vector, angle: f32) -> Self {
        let angle = angle * 0.5;
        let s = angle.sin();
        Self::new(angle.cos(), axis.x * s, axis.y * s, axis.z * s)
    }

    pub fn slerp(a: Self, b: Self, t: f32) -> Self {
        let cos_half_theta = a.dot(&b);
        if cos_half_theta >= 1.0 {
            // if angle 0, don't interpolate
            a
        } else {
            let half_theta = cos_half_theta.acos();
            let sign_half_theta = (1.0 - cos_half_theta * cos_half_theta).sqrt();
            // avoid divide by zero, use fallback approach in that case
            let (ra, rb) = if sign_half_theta.abs() < 1e-6 {
                (0.5, 0.5)
            } else {
                (
                    ((1.0 - t) * half_theta).sin() / sign_half_theta,
                    (t * half_theta).sin() / sign_half_theta,
                )
            };
            a * ra + b * rb
        }
    }
}

impl Length for Quat {
    fn dot(&self, other: &Self) -> f32 {
        self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Neg for Quat {
    type Output = Self;

    fn neg(mut self) -> Self {
        self.w = -self.w;
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }
}

impl AddAssign<Quat> for Quat {
    fn add_assign(&mut self, rhs: Self) {
        self.w += rhs.w;
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

auto_assign! { Add, add, add_assign, Quat, Quat }

impl SubAssign<Quat> for Quat {
    fn sub_assign(&mut self, rhs: Self) {
        self.w -= rhs.w;
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

auto_assign! { Sub, sub, sub_assign, Quat, Quat }

impl MulAssign<f32> for Quat {
    fn mul_assign(&mut self, rhs: f32) {
        self.w *= rhs;
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

auto_assign! { Mul, mul, mul_assign, Quat, f32 }

impl MulAssign<Quat> for Quat {
    fn mul_assign(&mut self, rhs: Self) {
        let w = self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z;
        let x = self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y;
        let y = self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x;
        let z = self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w;
        *self = Self::new(w, x, y, z);
    }
}

auto_assign! { Mul, mul, mul_assign, Quat, Quat }

impl DivAssign<f32> for Quat {
    fn div_assign(&mut self, rhs: f32) {
        self.w /= rhs;
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

auto_assign! { Div, div, div_assign, Quat, f32 }

#[derive(Copy, Clone)]
pub struct Mtx {
    pub xx: f32,
    pub xy: f32,
    pub xz: f32,
    pub yx: f32,
    pub yy: f32,
    pub yz: f32,
    pub zx: f32,
    pub zy: f32,
    pub zz: f32,
}

impl Mtx {
    pub const IDENT: Self = Self::new(
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
    );

    pub const fn new(xx: f32, xy: f32, xz: f32, yx: f32, yy: f32, yz: f32, zx: f32, zy: f32, zz: f32) -> Mtx {
        Self { xx, xy, xz, yx, yy, yz, zx, zy, zz }
    }

    pub fn transposed(mut self) -> Self {
        let t = self.yx;
        self.yx = self.xy;
        self.xy = t;
        let t = self.zx;
        self.zx = self.xz;
        self.xz = t;
        let t = self.yz;
        self.yz = self.zy;
        self.zy = t;
        self
    }

    pub fn looking_at(at: Vector, up: Vector) -> Self {
        let z = -at.normalized();
        let x = up.cross(&z).normalized();
        let y = z.cross(&x);

        Self::new(
            x.x, x.y, x.z,
            y.x, y.y, y.z,
            z.x, z.y, z.z,
        )
    }

    pub fn axis_angle(axis: &Vector, angle: f32) -> Self {
        let cos = angle.cos();
        let a = axis.x * axis.x;
        let xx = a + cos * (1.0 - a);
        let a = axis.y * axis.y;
        let yy = a + cos * (1.0 - a);
        let a = axis.z * axis.z;
        let zz = a + cos * (1.0 - a);
        let sin = angle.sin();
        let cos = 1.0 - cos;
        let a = axis.x * axis.y * cos;
        let b = axis.z * sin;
        let xy = a - b;
        let yx = a + b;
        let a = axis.x * axis.z * cos;
        let b = axis.y * sin;
        let xz = a + b;
        let zx = a - b;
        let a = axis.y * axis.z * cos;
        let b = axis.x * sin;
        let yz = a - b;
        let zy = a + b;
        Self::new(xx, xy, xz, yx, yy, yz, zx, zy, zz)
    }
}

impl From<Quat> for Mtx {
    fn from(q: Quat) -> Self {
        let a = q.x * q.x;
        let b = q.y * q.y;
        let c = q.z * q.z;
        let xx = 1.0 - 2.0 * (b + c);
        let yy = 1.0 - 2.0 * (a + c);
        let zz = 1.0 - 2.0 * (a + b);
        let a = q.x * q.y;
        let b = q.z * q.w;
        let xy = 2.0 * (a - b);
        let yx = 2.0 * (a + b);
        let a = q.x * q.z;
        let b = q.y * q.w;
        let xz = 2.0 * (a + b);
        let zx = 2.0 * (a - b);
        let a = q.y * q.z;
        let b = q.x * q.w;
        let yz = 2.0 * (a - b);
        let zy = 2.0 * (a + b);
        Self::new(xx, xy, xz, yx, yy, yz, zx, zy, zz)
    }
}

impl MulAssign<Mtx> for Mtx {
    fn mul_assign(&mut self, rhs: Mtx) {
        let x = self.xx;
        let y = self.xy;
        let z = self.xz;
        let xx = x * rhs.xx + y * rhs.yx + z * rhs.zx;
        let xy = x * rhs.xy + y * rhs.yy + z * rhs.zy;
        let xz = x * rhs.xz + y * rhs.yz + z * rhs.zz;
        let x = self.yx;
        let y = self.yy;
        let z = self.yz;
        let yx = x * rhs.xx + y * rhs.yx + z * rhs.zx;
        let yy = x * rhs.xy + y * rhs.yy + z * rhs.zy;
        let yz = x * rhs.xz + y * rhs.yz + z * rhs.zz;
        let x = self.zx;
        let y = self.zy;
        let z = self.zz;
        let zx = x * rhs.xx + y * rhs.yx + z * rhs.zx;
        let zy = x * rhs.xy + y * rhs.yy + z * rhs.zy;
        let zz = x * rhs.xz + y * rhs.yz + z * rhs.zz;
        *self = Self::new(xx, xy, xz, yx, yy, yz, zx, zy, zz);
    }
}

auto_assign! { Mul, mul, mul_assign, Mtx, Mtx }

impl Mul<Vector> for Mtx {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Vector {
        rhs * self
    }
}
