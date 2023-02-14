#ifndef CONDUX_TYPES_H
#define CONDUX_TYPES_H

#include "macros.h"

#include <stddef.h>
#include <stdint.h>

typedef float Vec[3];
typedef float Quat[4];
typedef float Mtx[3][3];

typedef struct {
    Vec point;
    Vec control;
    float tilt;
    float tiltOffset;
} SplinePoint;

typedef struct {
    Vec point;
    float position;
    float offset;
} SplineBaked;

typedef struct {
    const char *name;
    size_t size;
    const char *data;
} AssetEntry;

typedef struct {
    const AssetEntry *entry;
    size_t index;
} Asset;

typedef struct {
    uint8_t numVertices;
    Vec vertices[MAX_MESH_VERTICES];

    uint8_t numLines;
    uint8_t line1[MAX_MESH_LINES];
    uint8_t line2[MAX_MESH_LINES];
} Mesh;

typedef struct {
    /** Controls the maximum speed of the vehicle. */
    float speed;
    /** Controls the acceleration rate of the vehicle. */
    float acceleration;
    /** Controls the turn strength of the vehicle. */
    float handling;
    /** Controls how whichly the vehicle's velocity aligns with its forward vector. */
    float antiDrift;
    /** The model used to render the vehicle. */
    Mesh mesh;
} VehicleType;

typedef struct VehicleController {
    float (*getSteering) (struct VehicleController *self);
    float (*getPedal) (struct VehicleController *self);
} VehicleController;

typedef struct {
    /** The vehicle's position in global space. */
    Vec position;
    /** The vehicle's rotation in global space. */
    Quat rotation;
    /** The vehicle's velocity. */
    Vec velocity;
    /** The type of the vehicle. */
    const VehicleType *type;
    /** The vehicle controller. */
    VehicleController *controller;
} Vehicle;

typedef struct {
    uint8_t numVehicles;
    Vehicle previous[MAX_VEHICLES];
    Vehicle current[MAX_VEHICLES];
} VehicleSet;

typedef struct {
    /** The number of control points on the spline. */
    uint8_t numPoints;
    /** The number of baked points on the spline. */
    size_t numBaked;
    /** The total tilt, used for interpolation. */
    float totalTilt;
    /** The approximate length of the spline. */
    float length;
    /** The control points. */
    SplinePoint points[MAX_POINTS];
    /** The baked points. */
    SplineBaked baked[MAX_BAKED_POINTS];
} Spline;

typedef struct OctreeNode {
    int segments;
    struct OctreeNode *children;
} OctreeNode;

typedef struct {
    Vec min, max;

    OctreeNode root;
    OctreeNode childPool[8 + 64 + 512];

    int segmentNext[MAX_BAKED_POINTS];
    uint8_t segmentSides[MAX_BAKED_POINTS];
} Octree;

#endif
