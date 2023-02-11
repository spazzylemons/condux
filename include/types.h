#ifndef CONDUX_TYPES_H
#define CONDUX_TYPES_H

#include <stddef.h>
#include <stdint.h>

typedef float Vec[3];
typedef float Mtx[3][3];

typedef struct {
    Vec point;
    Vec control;
    float tilt;
} SplinePoint;

typedef struct {
    const char *name;
    size_t size;
    const char *data;
} AssetEntry;

typedef struct {
    const AssetEntry *entry;
    size_t index;
} Asset;

#define MAX_POINTS 64

typedef struct {
    /** The number of control points on the spline. */
    uint8_t numPoints;
    /** The total tilt, used for interpolation. */
    float totalTilt;
    /** The control points. */
    SplinePoint points[MAX_POINTS];
} Spline;

#endif
