#ifndef CONDUX_VEHICLE_H
#define CONDUX_VEHICLE_H

#include "types.h"

extern VehicleController gPlayerController;
extern VehicleController gEmptyController;

void vehicle_update(Vehicle *vehicle, const Spline *spline, const Octree *tree);

void vehicle_velocity_without_gravity(const Vehicle *vehicle, Vec v);

void vehicle_up_vector(const Vehicle *vehicle, Vec v);

void vehicle_forward_vector(const Vehicle *vehicle, Vec v);

#endif
