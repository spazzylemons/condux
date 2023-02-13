#include "collision.h"
#include "linalg.h"
#include "macros.h"
#include "spline.h"
#include "render.h"

#include <math.h>

#define MAX_DEPTH 3

static void check_bounds(const Vec v, Vec min, Vec max) {
    if (v[0] < min[0]) min[0] = v[0];
    if (v[1] < min[1]) min[1] = v[1];
    if (v[2] < min[2]) min[2] = v[2];
    if (v[0] > max[0]) max[0] = v[0];
    if (v[1] > max[1]) max[1] = v[1];
    if (v[2] > max[2]) max[2] = v[2];
}

static void get_bounds(const Spline *spline, size_t i, Vec min, Vec max) {
    const SplineBaked *baked[2];
    baked[0] = &spline->baked[i];
    baked[1] = &spline->baked[(i + 1) % spline->numBaked];
    Vec tmp, up, right, point, above, below;
    vec_set(min, INFINITY, INFINITY, INFINITY);
    vec_set(max, -INFINITY, -INFINITY, -INFINITY);
    for (int j = 0; j < 2; j++) {
        vec_copy(point, baked[j]->point);
        spline_get_up_right(spline, baked[j]->offset, up, right);
        vec_scale(right, SPLINE_TRACK_RADIUS);
        vec_copy(above, up);
        vec_scale(above, MAX_GRAVITY_HEIGHT);
        vec_copy(below, up);
        vec_scale(below, -COLLISION_DEPTH);
        vec_copy(tmp, above);
        vec_sub(tmp, right);
        vec_add(tmp, point);
        check_bounds(tmp, min, max);
        vec_copy(tmp, above);
        vec_add(tmp, right);
        vec_add(tmp, point);
        check_bounds(tmp, min, max);
        vec_copy(tmp, below);
        vec_sub(tmp, right);
        vec_add(tmp, point);
        check_bounds(tmp, min, max);
        vec_copy(tmp, below);
        vec_add(tmp, right);
        vec_add(tmp, point);
        check_bounds(tmp, min, max);
    }
}

static void build_octree(OctreeNode *node, OctreeNode *childPool, int depth, size_t *w) {
    // set segments to empty list
    node->segments = -1;
    if (depth >= MAX_DEPTH) {
        // if at max depth, we don't add children
        node->children = NULL;
        return;
    }
    // add children
    node->children = &childPool[*w];
    *w += 8;
    for (int i = 0; i < 8; i++) {
        build_octree(&node->children[i], childPool, depth + 1, w);
    }
}

static void add_node(Octree *tree, const Vec segment_min, const Vec segment_max, int segment) {
    Vec min, max;
    vec_copy(min, tree->min);
    vec_copy(max, tree->max);

    OctreeNode *current = &tree->root;
    int which[3];
    for (;;) {
        Vec center;
        vec_copy(center, min);
        vec_add(center, max);
        vec_scale(center, 0.5f);

        for (int i = 0; i < 3; i++) {
            if (segment_min[i] < center[i] && segment_max[i] < center[i]) {
                which[i] = 0;
                max[i] = center[i];
            } else if (segment_min[i] > center[i] && segment_max[i] > center[i]) {
                which[i] = 1;
                min[i] = center[i];
            } else {
                which[i] = -1;
            }
        }

        if (which[0] < 0 || which[1] < 0 || which[2] < 0 || current->children == NULL) {
            break;
        }

        current = &current->children[which[0] | (which[1] << 1) | (which[2] << 2)];
    }
    // add to list
    for (int i = 0; i < 3; i++) {
        if (which[i] == 0) {
            tree->segmentSides[segment] |= (1 << (2 * i));
        } else if (which[i] == 1) {
            tree->segmentSides[segment] |= (1 << ((2 * i) + 1));
        }
    }
    tree->segmentNext[segment] = current->segments;
    current->segments = segment;
}

void octree_init(Octree *tree, const Spline *spline) {
    // decide bounds
    vec_set(tree->min, INFINITY, INFINITY, INFINITY);
    vec_set(tree->max, -INFINITY, -INFINITY, -INFINITY);
    for (int i = 0; i < spline->numBaked; i++) {
        // get bounds of segment
        Vec min, max;
        get_bounds(spline, i, min, max);
        // update bounds
        check_bounds(min, tree->min, tree->max);
        check_bounds(max, tree->min, tree->max);
    }
    // build structure
    size_t w = 0;
    build_octree(&tree->root, tree->childPool, 0, &w);
    // for each segment, figure out where to put it
    for (int i = 0; i < spline->numBaked; i++) {
        Vec min, max;
        get_bounds(spline, i, min, max);
        add_node(tree, min, max, i);
    }
}
