#ifndef CONDUX_SPLINE_H
#define CONDUX_SPLINE_H

#include "types.h"

bool spline_load(Spline *spline, Asset *asset);

void spline_free(const Spline *spline);

float spline_get_tilt(const Spline *spline, float offset);

void spline_get_baked(const Spline *spline, float offset, Vec v);

void spline_get_up_right(const Spline *spline, float offset, Vec up, Vec right);

void spline_test_render(Spline *spline);

#endif
