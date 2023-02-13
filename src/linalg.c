#include "linalg.h"

#include <math.h>
#include <stdio.h>

const Vec gVecZero = { 0.0f, 0.0f, 0.0f };
const Vec gVecXAxis = { 1.0f, 0.0f, 0.0f };
const Vec gVecYAxis = { 0.0f, 1.0f, 0.0f };
const Vec gVecZAxis = { 0.0f, 0.0f, 1.0f };

void vec_copy(Vec dst, const Vec src) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

void vec_swap(Vec a, Vec b) {
    float temp;
    temp = a[0];
    a[0] = b[0];
    b[0] = temp;
    temp = a[1];
    a[1] = b[1];
    b[1] = temp;
    temp = a[2];
    a[2] = b[2];
    b[2] = temp;
}

void vec_set(Vec dst, float x, float y, float z) {
    dst[0] = x;
    dst[1] = y;
    dst[2] = z;
}

void vec_add(Vec dst, const Vec src) {
    dst[0] += src[0];
    dst[1] += src[1];
    dst[2] += src[2];
}

void vec_sub(Vec dst, const Vec src) {
    dst[0] -= src[0];
    dst[1] -= src[1];
    dst[2] -= src[2];
}

void vec_scale(Vec v, float scale) {
    v[0] *= scale;
    v[1] *= scale;
    v[2] *= scale;
}

void vec_normalize(Vec v) {
    float m = vec_magnitude_sq(v);
    if (m == 0.0f) return;
    vec_scale(v, 1.0f / sqrtf(m));
}

void vec_cross(Vec dst, const Vec a, const Vec b) {
    dst[0] = a[1] * b[2] - b[1] * a[2];
    dst[1] = a[2] * b[0] - b[2] * a[0];
    dst[2] = a[0] * b[1] - b[0] * a[1];
}

float vec_dot(const Vec a, const Vec b) {
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

float vec_magnitude_sq(const Vec v) {
    return v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
}

float vec_distance_sq(const Vec a, const Vec b) {
    Vec c;
    vec_copy(c, a);
    vec_sub(c, b);
    return vec_magnitude_sq(c);
}

float vec_signed_angle_to(const Vec v, const Vec to, const Vec axis) {
    Vec cross;
    vec_cross(cross, v, to);
    float unsigned_angle = atan2f(sqrtf(vec_magnitude_sq(cross)), vec_dot(v, to));
    float sign = vec_dot(cross, axis);
    return (sign > 0.0f) ? -unsigned_angle : unsigned_angle;
}

const Quat gQuatIdentity = { 1.0f, 0.0f, 0.0f, 0.0f };

void quat_copy(Quat dst, const Quat src) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
    dst[3] = src[3];
}

void quat_add(Quat dst, const Quat src) {
    dst[0] += src[0];
    dst[1] += src[1];
    dst[2] += src[2];
    dst[3] += src[3];
}

void quat_scale(Quat dst, float scale) {
    dst[0] *= scale;
    dst[1] *= scale;
    dst[2] *= scale;
    dst[3] *= scale;
}

void quat_mul(Quat dst, const Quat a, const Quat b) {
    dst[0] = a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3];
    dst[1] = a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2];
    dst[2] = a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1];
    dst[3] = a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0];
}

void quat_angle_axis(Quat q, const Vec axis, float angle) {
    angle *= 0.5f;
    q[0] = cosf(angle);
    float s = sinf(angle);
    q[1] = axis[0] * s;
    q[2] = axis[1] * s;
    q[3] = axis[2] * s;
}

void quat_to_mtx(Mtx m, const Quat q) {
    float a, b, c;

    a = q[1] * q[1];
    b = q[2] * q[2];
    c = q[3] * q[3];
    m[0][0] = 1.0f - 2.0f * (b + c);
    m[1][1] = 1.0f - 2.0f * (a + c);
    m[2][2] = 1.0f - 2.0f * (a + b);

    a = q[1] * q[2];
    b = q[3] * q[0];
    m[0][1] = 2.0f * (a - b);
    m[1][0] = 2.0f * (a + b);

    a = q[1] * q[3];
    b = q[2] * q[0];
    m[0][2] = 2.0f * (a + b);
    m[2][0] = 2.0f * (a - b);

    a = q[2] * q[3];
    b = q[1] * q[0];
    m[1][2] = 2.0f * (a - b);
    m[2][1] = 2.0f * (a + b);
}

void quat_slerp(Quat dst, const Quat a, const Quat b, float t) {
    float cos_half_theta = quat_dot(a, b);
    // if angle 0, don't interpolate
    if (fabsf(cos_half_theta) >= 1.0f) {
        quat_copy(dst, a);
        return;
    }
    float half_theta = acosf(cos_half_theta);
    float sin_half_theta = sqrtf(1.0f - cos_half_theta * cos_half_theta);
    // avoid divide by zero, use fallback approach in that case
    float ra, rb;
    if (fabsf(sin_half_theta) < 1e-6) {
        // average the quaternions as fallback
        ra = 0.5f;
        rb = 0.5f;
    } else {
        ra = sinf((1.0f - t) * half_theta) / sin_half_theta;
        rb = sinf(t * half_theta) / sin_half_theta;
    }
    // scale quaternions and add
    Quat tmp;
    quat_copy(tmp, a);
    quat_scale(tmp, ra);
    quat_copy(dst, b);
    quat_scale(dst, rb);
    quat_add(dst, tmp);
}

void quat_normalize(Quat q) {
    float m = quat_magnitude_sq(q);
    if (m == 0.0f) return;
    quat_scale(q, 1.0f / sqrtf(m));
}

float quat_dot(const Quat a, const Quat b) {
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
}

float quat_magnitude_sq(const Quat q) {
    return q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
}

const Mtx gMtxIdentity = {
    { 1.0f, 0.0f, 0.0f },
    { 0.0f, 1.0f, 0.0f },
    { 0.0f, 0.0f, 1.0f },
};

void mtx_copy(Mtx dst, const Mtx src) {
    dst[0][0] = src[0][0];
    dst[0][1] = src[0][1];
    dst[0][2] = src[0][2];
    dst[1][0] = src[1][0];
    dst[1][1] = src[1][1];
    dst[1][2] = src[1][2];
    dst[2][0] = src[2][0];
    dst[2][1] = src[2][1];
    dst[2][2] = src[2][2];
}

void mtx_transpose(Mtx m) {
    float t;

    t = m[1][0];
    m[1][0] = m[0][1];
    m[0][1] = t;

    t = m[2][1];
    m[2][1] = m[1][2];
    m[1][2] = t;

    t = m[2][0];
    m[2][0] = m[0][2];
    m[0][2] = t;
}

void mtx_look_at(Mtx m, const Vec at, const Vec up) {
    // negate while copying this one
    m[2][0] = -at[0];
    m[2][1] = -at[1];
    m[2][2] = -at[2];
    vec_normalize(m[2]);

    vec_cross(m[0], up, m[2]);
    vec_normalize(m[0]);

    vec_cross(m[1], m[2], m[0]);
}

void mtx_angle_axis(Mtx m, const Vec axis, float angle) {
    float cos_angle = cosf(angle);
    float a, b;

    a = axis[0] * axis[0];
    m[0][0] = a + cos_angle * (1.0f - a);
    a = axis[1] * axis[1];
    m[1][1] = a + cos_angle * (1.0f - a);
    a = axis[2] * axis[2];
    m[2][2] = a + cos_angle * (1.0f - a);

    float sin_angle = sinf(angle);
    cos_angle = 1.0f - cos_angle;

    a = axis[0] * axis[1] * cos_angle;
    b = axis[2] * sin_angle;
    m[0][1] = a - b;
    m[1][0] = a + b;

    a = axis[0] * axis[2] * cos_angle;
    b = axis[1] * sin_angle;
    m[0][2] = a + b;
    m[2][0] = a - b;

    a = axis[1] * axis[2] * cos_angle;
    b = axis[0] * sin_angle;
    m[1][2] = a - b;
    m[2][1] = a + b;
}

void mtx_mul(Mtx dst, const Mtx a, const Mtx b) {
    register float x, y, z;
    x = a[0][0];
    y = a[0][1];
    z = a[0][2];
    dst[0][0] = x * b[0][0] + y * b[1][0] + z * b[2][0];
    dst[0][1] = x * b[0][1] + y * b[1][1] + z * b[2][1];
    dst[0][2] = x * b[0][2] + y * b[1][2] + z * b[2][2];
    x = a[1][0];
    y = a[1][1];
    z = a[1][2];
    dst[1][0] = x * b[0][0] + y * b[1][0] + z * b[2][0];
    dst[1][1] = x * b[0][1] + y * b[1][1] + z * b[2][1];
    dst[1][2] = x * b[0][2] + y * b[1][2] + z * b[2][2];
    x = a[2][0];
    y = a[2][1];
    z = a[2][2];
    dst[2][0] = x * b[0][0] + y * b[1][0] + z * b[2][0];
    dst[2][1] = x * b[0][1] + y * b[1][1] + z * b[2][1];
    dst[2][2] = x * b[0][2] + y * b[1][2] + z * b[2][2];
}

void mtx_mul_vec(const Mtx m, Vec dst, const Vec src) {
    dst[0] = src[0] * m[0][0] + src[1] * m[1][0] + src[2] * m[2][0];
    dst[1] = src[0] * m[0][1] + src[1] * m[1][1] + src[2] * m[2][1];
    dst[2] = src[0] * m[0][2] + src[1] * m[1][2] + src[2] * m[2][2];
}
