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

void mtx_mul_vec(const Mtx m, Vec dst, const Vec src) {
    dst[0] = src[0] * m[0][0] + src[1] * m[1][0] + src[2] * m[2][0];
    dst[1] = src[0] * m[0][1] + src[1] * m[1][1] + src[2] * m[2][1];
    dst[2] = src[0] * m[0][2] + src[1] * m[1][2] + src[2] * m[2][2];
}
