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
    /** Controls the maximum speed of the vehicle. */
    float speed;
    /** Controls the acceleration rate of the vehicle. */
    float acceleration;
    /** Controls the turn strength of the vehicle. */
    float handling;
    /** Controls how whichly the vehicle's velocity aligns with its forward vector. */
    float antiDrift;
} VehicleType;

typedef struct VehicleController {
    float (*getSteering) (struct VehicleController *self);
    float (*getPedal) (struct VehicleController *self);
} VehicleController;

typedef struct {
    /** The vehicle's position in global space. */
    Vec position;
    /** TODO might want to make this a quaternion */
    Mtx rotation;
    /** The vehicle's velocity. */
    Vec velocity;
    /** The type of the vehicle. */
    const VehicleType *type;
    /** The vehicle controller. */
    VehicleController *controller;
} Vehicle;

#define MAX_POINTS 64
#define MAX_BAKE_DEPTH 5
#define MAX_BAKED_POINTS (MAX_POINTS * (1 << MAX_BAKE_DEPTH))

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

typedef struct QuadTreeSegment {
    struct QuadTreeSegment *next;
} QuadTreeSegment;

typedef struct QuadTreeNode {
    QuadTreeSegment *minXSegments;
    QuadTreeSegment *minZSegments;
    QuadTreeSegment *maxXSegments;
    QuadTreeSegment *maxZSegments;
    QuadTreeSegment *midSegments;
    struct QuadTreeNode *children;
} QuadTreeNode;

typedef struct {
    float minX, minZ, maxX, maxZ;

    QuadTreeNode root;
    QuadTreeNode childPool[4 + 16 + 64];

    QuadTreeSegment segmentPool[MAX_BAKED_POINTS];
} QuadTree;

#endif
