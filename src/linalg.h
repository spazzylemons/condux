#ifndef CONDUX_LINALG_H
#define CONDUX_LINALG_H

#include "types.h"

void vec_copy(Vec dst, Vec src);
void vec_swap(Vec a, Vec b);
void vec_set(Vec dst, float x, float y, float z);
void vec_add(Vec dst, Vec src);
void vec_sub(Vec dst, Vec src);
void vec_scale(Vec v, float scale);
void vec_normalize(Vec v);
float vec_dot(Vec a, Vec b);
float vec_magnitude_sq(Vec v);
float vec_distance_sq(Vec a, Vec b);

#endif
