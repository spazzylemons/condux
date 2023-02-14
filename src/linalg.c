#include "linalg.h"

#include <math.h>
#include <stdio.h>

const Vec gVecZero = { 0.0f, 0.0f, 0.0f };
const Vec gVecXAxis = { 1.0f, 0.0f, 0.0f };
const Vec gVecYAxis = { 0.0f, 1.0f, 0.0f };
const Vec gVecZAxis = { 0.0f, 0.0f, 1.0f };

const Quat gQuatIdentity = { 1.0f, 0.0f, 0.0f, 0.0f };

const Mtx gMtxIdentity = {
    { 1.0f, 0.0f, 0.0f },
    { 0.0f, 1.0f, 0.0f },
    { 0.0f, 0.0f, 1.0f },
};
