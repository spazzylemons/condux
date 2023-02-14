#ifndef CONDUX_MACROS_H
#define CONDUX_MACROS_H

#define TICKS_PER_SECOND 60

#define MAX_MESH_VERTICES 32

#define MAX_MESH_LINES 64

#define MAX_POINTS 64

#define MAX_BAKE_DEPTH 5

#define MAX_BAKED_POINTS (MAX_POINTS * (1 << MAX_BAKE_DEPTH))

#define MAX_VEHICLES 8

#define PI 3.141592653589793115998f

#define SPLINE_TRACK_RADIUS 2.0f

#define MAX_GRAVITY_HEIGHT 5.0f

#define COLLISION_DEPTH 0.25f

#endif
