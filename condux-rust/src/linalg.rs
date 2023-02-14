use core::{slice, ops::{Add, Sub, Mul, Div, Neg}};
use num_traits::real::Real;

#[derive(Clone, Copy)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector {
    const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    const X_AXIS: Self = Self::new(1.0, 0.0, 0.0);
    const Y_AXIS: Self = Self::new(0.0, 1.0, 0.0);
    const Z_AXIS: Self = Self::new(0.0, 0.0, 1.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn write_to_ptr(self, ptr: *mut f32) {
        let entries = unsafe { slice::from_raw_parts_mut(ptr, 3) };
        entries[0] = self.x;
        entries[1] = self.y;
        entries[2] = self.z;
    }

    pub fn normalized(self) -> Self {
        let m = self.magnitude_sq();
        if m == 0.0 {
            self
        } else {
            self / m.sqrt()
        }
    }

    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn magnitude_sq(self) -> f32 {
        self.dot(self)
    }

    pub fn distance_sq(self, rhs: Self) -> f32 {
        (self - rhs).magnitude_sq()
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - rhs.y * self.z,
            self.z * rhs.x - rhs.z * self.x,
            self.x * rhs.y - rhs.x * self.y,
        )
    }

    pub fn signed_angle_to(self, to: Self, axis: Self) -> f32 {
        let cross = self.cross(to);
        let unsigned = cross.magnitude_sq().sqrt().atan2(self.dot(to));
        let sign = cross.dot(axis);
        if sign > 0.0 {
            -unsigned
        } else {
            unsigned
        }
    }
}

impl From<*const f32> for Vector {
    fn from(value: *const f32) -> Self {
        let entries = unsafe { slice::from_raw_parts(value, 3) };
        Self::new(
            entries[0],
            entries[1],
            entries[2],
        )
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add<Vector> for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
        )
    }
}

impl Sub<Vector> for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
        )
    }
}

impl Mul<f32> for Vector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(
            self.x * rhs,
            self.y * rhs,
            self.z * rhs,
        )
    }
}

impl Div<f32> for Vector {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self::new(
            self.x / rhs,
            self.y / rhs,
            self.z / rhs,
        )
    }
}

impl Mul<Mtx> for Vector {
    type Output = Self;

    fn mul(self, rhs: Mtx) -> Self {
        Self::new(
            self.x * rhs.m00 + self.y * rhs.m10 + self.z * rhs.m20,
            self.x * rhs.m01 + self.y * rhs.m11 + self.z * rhs.m21,
            self.x * rhs.m02 + self.y * rhs.m12 + self.z * rhs.m22,
        )
    }
}

#[no_mangle]
extern "C" fn vec_copy(dst: *mut f32, src: *const f32) {
    Vector::from(src).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn vec_set(dst: *mut f32, x: f32, y: f32, z: f32) {
    Vector::new(x, y, z).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn vec_add(dst: *mut f32, src: *const f32) {
    (Vector::from(dst as *const f32) + Vector::from(src)).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn vec_sub(dst: *mut f32, src: *const f32) {
    (Vector::from(dst as *const f32) - Vector::from(src)).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn vec_scale(v: *mut f32, scale: f32) {
    (Vector::from(v as *const f32) * scale).write_to_ptr(v);
}

#[no_mangle]
extern "C" fn vec_normalize(v: *mut f32) {
    Vector::from(v as *const f32).normalized().write_to_ptr(v);
}

#[no_mangle]
extern "C" fn vec_cross(dst: *mut f32, a: *const f32, b: *const f32) {
    Vector::from(a).cross(Vector::from(b)).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn vec_dot(a: *const f32, b: *const f32) -> f32 {
    Vector::from(a).dot(Vector::from(b))
}

#[no_mangle]
extern "C" fn vec_magnitude_sq(v: *const f32) -> f32 {
    Vector::from(v).magnitude_sq()
}

#[no_mangle]
extern "C" fn vec_distance_sq(a: *const f32, b: *const f32) -> f32 {
    Vector::from(a).distance_sq(Vector::from(b))
}

#[no_mangle]
extern "C" fn vec_signed_angle_to(v: *const f32, to: *const f32, axis: *const f32) -> f32 {
    Vector::from(v).signed_angle_to(Vector::from(to), Vector::from(axis))
}

#[derive(Clone, Copy)]
pub struct Quat {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quat {
    const IDENT: Self = Self::new(1.0, 0.0, 0.0, 0.0);

    pub const fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    pub fn write_to_ptr(self, ptr: *mut f32) {
        let entries = unsafe { slice::from_raw_parts_mut(ptr, 4) };
        entries[0] = self.w;
        entries[1] = self.x;
        entries[2] = self.y;
        entries[3] = self.z;
    }

    pub fn normalized(self) -> Self {
        let m = self.magnitude_sq();
        if m == 0.0 {
            self
        } else {
            self / m.sqrt()
        }
    }

    pub fn dot(self, rhs: Self) -> f32 {
        self.w * rhs.w + self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn magnitude_sq(self) -> f32 {
        self.dot(self)
    }

    pub fn distance_sq(self, rhs: Self) -> f32 {
        (self - rhs).magnitude_sq()
    }

    pub fn axis_angle(axis: Vector, angle: f32) -> Self {
        let angle = angle * 0.5;
        let s = angle.sin();
        Self::new(
            angle.cos(),
            axis.x * s,
            axis.y * s,
            axis.z * s,
        )
    }

    pub fn slerp(a: Quat, b: Quat, t: f32) -> Self {
        let cos_half_theta = a.dot(b);
        // if angle 0, don't interpolate
        if cos_half_theta.abs() >= 1.0 {
            a
        } else {
            let half_theta = cos_half_theta.acos();
            let sin_half_theta = (1.0 - cos_half_theta * cos_half_theta).sqrt();
            // avoid divide by zero, use fallback approach in that case
            let (ra, rb) = if sin_half_theta.abs() < 1e-6 {
                // average the quaternions as fallback
                (0.5, 0.5)
            } else {
                (
                    ((1.0 - t) * half_theta).sin() / sin_half_theta,
                    (t * half_theta).sin() / sin_half_theta,
                )
            };
            (a * ra) + (b * rb)
        }
    }
}

impl From<*const f32> for Quat {
    fn from(value: *const f32) -> Self {
        let entries = unsafe { slice::from_raw_parts(value, 4) };
        Self::new(
            entries[0],
            entries[1],
            entries[2],
            entries[3],
        )
    }
}

impl Add<Quat> for Quat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.w + rhs.w,
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
        )
    }
}

impl Sub<Quat> for Quat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.w - rhs.w,
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
        )
    }
}

impl Mul<f32> for Quat {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(
            self.w * rhs,
            self.x * rhs,
            self.y * rhs,
            self.z * rhs,
        )
    }
}

impl Div<f32> for Quat {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self::new(
            self.w / rhs,
            self.x / rhs,
            self.y / rhs,
            self.z / rhs,
        )
    }
}

impl Mul<Quat> for Quat {
    type Output = Self;

    fn mul(self, rhs: Quat) -> Self {
        Self::new(
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        )
    }
}

#[no_mangle]
extern "C" fn quat_copy(dst: *mut f32, src: *const f32) {
    Quat::from(src).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn quat_add(dst: *mut f32, src: *const f32) {
    (Quat::from(dst as *const f32) + Quat::from(src)).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn quat_scale(dst: *mut f32, scale: f32) {
    (Quat::from(dst as *const f32) * scale).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn quat_mul(dst: *mut f32, a: *const f32, b: *const f32) {
    (Quat::from(a) * Quat::from(b)).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn quat_angle_axis(dst: *mut f32, axis: *const f32, angle: f32) {
    Quat::axis_angle(Vector::from(axis), angle).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn quat_slerp(dst: *mut f32, a: *const f32, b: *const f32, t: f32) {
    Quat::slerp(Quat::from(a), Quat::from(b), t).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn quat_normalize(dst: *mut f32) {
    Quat::from(dst as *const f32).normalized().write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn quat_dot(a: *const f32, b: *const f32) -> f32 {
    Quat::from(a).dot(Quat::from(b))
}

#[no_mangle]
extern "C" fn quat_magnitude_sq(a: *const f32) -> f32 {
    Quat::from(a).magnitude_sq()
}

pub struct Mtx {
    m00: f32,
    m01: f32,
    m02: f32,
    m10: f32,
    m11: f32,
    m12: f32,
    m20: f32,
    m21: f32,
    m22: f32,
}

impl Mtx {
    pub const IDENTITY: Self = Self::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);

    pub const fn new(m00: f32, m01: f32, m02: f32, m10: f32, m11: f32, m12: f32, m20: f32, m21: f32, m22: f32,) -> Self {
        Self { m00, m01, m02, m10, m11, m12, m20, m21, m22 }
    }

    pub fn write_to_ptr(self, ptr: *mut [f32; 3]) {
        let entries = unsafe { slice::from_raw_parts_mut(ptr, 3) };
        entries[0][0] = self.m00;
        entries[0][1] = self.m01;
        entries[0][2] = self.m02;
        entries[1][0] = self.m10;
        entries[1][1] = self.m11;
        entries[1][2] = self.m12;
        entries[2][0] = self.m20;
        entries[2][1] = self.m21;
        entries[2][2] = self.m22;
    }

    pub fn transposed(mut self) -> Self {
        let t = self.m10;
        self.m10 = self.m01;
        self.m01 = t;
        let t = self.m21;
        self.m21 = self.m12;
        self.m12 = t;
        let t = self.m20;
        self.m20 = self.m02;
        self.m02 = t;
        self
    }

    pub fn look_at(at: Vector, up: Vector) -> Self {
        let z = -at.normalized();
        let x = up.cross(z).normalized();
        let y = z.cross(x);

        Self::new(x.x, x.y, x.z, y.x, y.y, y.z, z.x, z.y, z.z)
    }
}

impl From<*const [f32; 3]> for Mtx {
    fn from(value: *const [f32; 3]) -> Self {
        let entries = unsafe { slice::from_raw_parts(value, 3) };
        Self::new(
            entries[0][0],
            entries[0][1],
            entries[0][2],
            entries[1][0],
            entries[1][1],
            entries[1][2],
            entries[2][0],
            entries[2][1],
            entries[2][2],
        )
    }
}

impl From<Quat> for Mtx {
    fn from(q: Quat) -> Self {
        let a = q.x * q.x;
        let b = q.y * q.y;
        let c = q.z * q.z;
        let m00 = 1.0 - 2.0 * (b + c);
        let m11 = 1.0 - 2.0 * (a + c);
        let m22 = 1.0 - 2.0 * (a + b);
    
        let a = q.x * q.y;
        let b = q.z * q.w;
        let m01 = 2.0 * (a - b);
        let m10 = 2.0 * (a + b);
    
        let a = q.x * q.z;
        let b = q.y * q.w;
        let m02 = 2.0 * (a + b);
        let m20 = 2.0 * (a - b);
    
        let a = q.y * q.z;
        let b = q.x * q.w;
        let m12 = 2.0 * (a - b);
        let m21 = 2.0 * (a + b);

        Self::new(m00, m01, m02, m10, m11, m12, m20, m21, m22)
    }
}

impl Mul<Mtx> for Mtx {
    type Output = Self;

    fn mul(self, rhs: Mtx) -> Self {
        Self::new(
            self.m00 * rhs.m00 + self.m01 * rhs.m10 + self.m02 * rhs.m20,
            self.m00 * rhs.m01 + self.m01 * rhs.m11 + self.m02 * rhs.m21,
            self.m00 * rhs.m02 + self.m01 * rhs.m12 + self.m02 * rhs.m22,
            self.m10 * rhs.m00 + self.m11 * rhs.m10 + self.m12 * rhs.m20,
            self.m10 * rhs.m01 + self.m11 * rhs.m11 + self.m12 * rhs.m21,
            self.m10 * rhs.m02 + self.m11 * rhs.m12 + self.m12 * rhs.m22,
            self.m20 * rhs.m00 + self.m21 * rhs.m10 + self.m22 * rhs.m20,
            self.m20 * rhs.m01 + self.m21 * rhs.m11 + self.m22 * rhs.m21,
            self.m20 * rhs.m02 + self.m21 * rhs.m12 + self.m22 * rhs.m22,
        )
    }
}

#[no_mangle]
extern "C" fn quat_to_mtx(m: *mut [f32; 3], q: *const f32) {
    Mtx::from(Quat::from(q)).write_to_ptr(m);
}

#[no_mangle]
extern "C" fn mtx_copy(dst: *mut [f32; 3], src: *const [f32; 3]) {
    Mtx::from(src).write_to_ptr(dst);
}

#[no_mangle]
extern "C" fn mtx_transpose(m: *mut [f32; 3]) {
    Mtx::from(m as *const [f32; 3]).transposed().write_to_ptr(m);
}

#[no_mangle]
extern "C" fn mtx_look_at(m: *mut [f32; 3], at: *const f32, up: *const f32) {
    Mtx::look_at(Vector::from(at), Vector::from(up)).write_to_ptr(m);
}

#[no_mangle]
extern "C" fn mtx_angle_axis(m: *mut [f32; 3], axis: *const f32, angle: f32) {
    Mtx::from(Quat::axis_angle(Vector::from(axis), angle)).write_to_ptr(m);
}

#[no_mangle]
extern "C" fn mtx_mul(dst: *mut [f32; 3], a: *const [f32; 3], b: *const [f32; 3]) {
    (Mtx::from(a) * Mtx::from(b)).write_to_ptr(dst);
}


#[no_mangle]
extern "C" fn mtx_mul_vec(m: *const [f32; 3], dst: *mut f32, src: *const f32) {
    (Vector::from(src) * Mtx::from(m)).write_to_ptr(dst);
}
