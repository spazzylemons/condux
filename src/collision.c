#include "collision.h"
#include "linalg.h"
#include "macros.h"
#include "spline.h"
#include "render.h"

void octree_find_which(const Vec point, Vec min, Vec max, int *which) {
    Vec center;
    vec_copy(center, min);
    vec_add(center, max);
    vec_scale(center, 0.5f);

    for (uint8_t i = 0; i < 3; i++) {
        if (point[i] < center[i]) {
            which[i] = 0;
            max[i] = center[i];
        } else {
            which[i] = 1;
            min[i] = center[i];
        }
    }
}
