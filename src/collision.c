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
        node->children_index = -1;
        return;
    }
    // add children
    node->children_index = *w;
    *w += 8;
    for (int i = 0; i < 8; i++) {
        build_octree(&childPool[node->children_index + i], childPool, depth + 1, w);
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

        for (uint8_t i = 0; i < 3; i++) {
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

        if (which[0] < 0 || which[1] < 0 || which[2] < 0 || current->children_index == -1) {
            break;
        }

        current = &tree->childPool[current->children_index + (which[0] | (which[1] << 1) | (which[2] << 2))];
    }
    // add to list
    tree->segmentSides[segment] = 0;
    for (uint8_t i = 0; i < 3; i++) {
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

void octree_reset_vehicles(Octree *tree) {
    tree->root.vehicles = -1;
    for (size_t i = 0; i < OCTREE_POOL_SIZE; i++) {
        tree->childPool[i].vehicles = -1;
    }
}

void octree_add_vehicle(Octree *tree, const Vec pos, int index) {
    Vec vehicle_min, vehicle_max;
    vehicle_min[0] = pos[0] - (2.0f * VEHICLE_RADIUS);
    vehicle_min[1] = pos[1] - (2.0f * VEHICLE_RADIUS);
    vehicle_min[2] = pos[2] - (2.0f * VEHICLE_RADIUS);
    vehicle_max[0] = pos[0] + (2.0f * VEHICLE_RADIUS);
    vehicle_max[1] = pos[1] + (2.0f * VEHICLE_RADIUS);
    vehicle_max[2] = pos[2] + (2.0f * VEHICLE_RADIUS);

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

        for (uint8_t i = 0; i < 3; i++) {
            if (vehicle_min[i] < center[i] && vehicle_max[i] < center[i]) {
                which[i] = 0;
                max[i] = center[i];
            } else if (vehicle_min[i] > center[i] && vehicle_max[i] > center[i]) {
                which[i] = 1;
                min[i] = center[i];
            } else {
                which[i] = -1;
            }
        }

        if (which[0] < 0 || which[1] < 0 || which[2] < 0 || current->children_index == -1) {
            break;
        }

        current = &tree->childPool[current->children_index + (which[0] | (which[1] << 1) | (which[2] << 2))];
    }
    // add to list
    tree->vehicleSides[index] = 0;
    for (uint8_t i = 0; i < 3; i++) {
        if (which[i] == 0) {
            tree->vehicleSides[index] |= (1 << (2 * i));
        } else if (which[i] == 1) {
            tree->vehicleSides[index] |= (1 << ((2 * i) + 1));
        }
    }
    tree->vehicleNext[index] = current->vehicles;
    current->vehicles = index;
}

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

uint8_t octree_find_collisions(Octree *tree, const Vec point, uint8_t *out) {
    Vec min, max;
    vec_copy(min, tree->min);
    vec_copy(max, tree->max);

    uint8_t result = 0;

    const OctreeNode *current = &tree->root;
    for (;;) {
        int which[3];
        octree_find_which(point, min, max, which);

        int index = current->vehicles;
        while (index >= 0) {
            if (!(which[0] == 1 && (tree->vehicleSides[index] & 1)) &&
                !(which[0] == 0 && (tree->vehicleSides[index] & 2)) &&
                !(which[1] == 1 && (tree->vehicleSides[index] & 4)) &&
                !(which[1] == 0 && (tree->vehicleSides[index] & 8)) &&
                !(which[2] == 1 && (tree->vehicleSides[index] & 16)) &&
                !(which[2] == 0 && (tree->vehicleSides[index] & 32))) {
                out[result++] = index;
            }
            index = tree->vehicleNext[index];
        }

        if (current->children_index == -1) {
            break;
        }
        current = &tree->childPool[current->children_index + (which[0] | (which[1] << 1) | (which[2] << 2))];
    }

    return result;
}
