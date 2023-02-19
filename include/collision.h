#ifndef CONDUX_COLLISION_H
#define CONDUX_COLLISION_H

#include "types.h"

#include <stdbool.h>

void octree_init(Octree *tree, const Spline *spline);

void octree_find_which(const Vec point, Vec min, Vec max, int *which);

#endif
