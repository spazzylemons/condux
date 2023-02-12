#ifndef CONDUX_COLLISION_H
#define CONDUX_COLLISION_H

#include "types.h"

#include <stdbool.h>

bool quad_tree_init(QuadTree *tree, const Spline *spline);

void quad_tree_free(const QuadTree *tree);

void quad_tree_test_print_colliding(const QuadTree *tree, float x, float z);

#endif
