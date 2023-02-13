#ifndef CONDUX_VEHICLE_H
#define CONDUX_VEHICLE_H

#include "types.h"

void vehicle_update(Vehicle *vehicle, const Spline *spline, const QuadTree *tree, float delta);

#endif