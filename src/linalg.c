#include "linalg.h"

#include <math.h>

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
