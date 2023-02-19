#include "collision.h"
#include "linalg.h"
#include "render.h"
#include "spline.h"
#include "state.h"
#include "vehicle.h"

#include <math.h>

#define CAMERA_FOLLOW_DISTANCE 2.5f
#define CAMERA_APPROACH_SPEED 2.0f
#define CAMERA_UP_DISTANCE 0.325f
#define STEERING_FACTOR 0.25f

Spline gSpline;
