#ifndef CONDUX_COLLISION_H
#define CONDUX_COLLISION_H

#include "types.h"

#include <stdbool.h>

void octree_init(Octree *tree, const Spline *spline);

void octree_reset_vehicles(Octree *tree);

void octree_add_vehicle(Octree *tree, const Vec pos, int index);

void octree_find_which(const Vec point, Vec min, Vec max, int *which);

uint8_t octree_find_collisions(Octree *tree, const Vec pos, uint8_t *out);

#endif
