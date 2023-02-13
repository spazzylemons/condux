#ifndef CONDUX_LINALG_H
#define CONDUX_LINALG_H

#include "types.h"

extern const Vec gVecZero;
extern const Vec gVecXAxis;
extern const Vec gVecYAxis;
extern const Vec gVecZAxis;

void vec_copy(Vec dst, const Vec src);
void vec_swap(Vec a, Vec b);
void vec_set(Vec dst, float x, float y, float z);
void vec_add(Vec dst, const Vec src);
void vec_sub(Vec dst, const Vec src);
void vec_scale(Vec v, float scale);
void vec_normalize(Vec v);
void vec_cross(Vec dst, const Vec a, const Vec b);
float vec_dot(const Vec a, const Vec b);
float vec_magnitude_sq(const Vec v);
float vec_distance_sq(const Vec a, const Vec b);
float vec_signed_angle_to(const Vec v, const Vec to, const Vec axis);

extern const Mtx gMtxIdentity;

void mtx_copy(Mtx dst, const Mtx src);
void mtx_transpose(Mtx m);
void mtx_look_at(Mtx m, const Vec at, const Vec up);
void mtx_angle_axis(Mtx m, const Vec axis, float angle);
void mtx_mul(Mtx dst, const Mtx a, const Mtx b);
void mtx_mul_vec(const Mtx m, Vec dst, const Vec src);

#endif