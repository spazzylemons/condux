#include "linalg.h"

#include <math.h>

void vec_copy(Vec dst, Vec src) {
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

void vec_add(Vec dst, Vec src) {
    dst[0] += src[0];
    dst[1] += src[1];
    dst[2] += src[2];
}

void vec_sub(Vec dst, Vec src) {
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

float vec_dot(Vec a, Vec b) {
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

float vec_magnitude_sq(Vec v) {
    return v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
}

float vec_distance_sq(Vec a, Vec b) {
    Vec c;
    vec_copy(c, a);
    vec_sub(c, b);
    return vec_magnitude_sq(c);
}
