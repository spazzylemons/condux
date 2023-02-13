#ifndef CONDUX_SPLINE_H
#define CONDUX_SPLINE_H

#include "types.h"

#include <stdbool.h>

bool spline_load(Spline *spline, Asset *asset);

float spline_get_tilt(const Spline *spline, float offset);

void spline_get_baked(const Spline *spline, float offset, Vec v);

void spline_get_up_right(const Spline *spline, float offset, Vec up, Vec right);

bool spline_get_up_height(const Spline *spline, const QuadTree *tree, const Vec pos, Vec up, float *height);

#endif
