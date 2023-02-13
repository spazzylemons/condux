#include "collision.h"
#include "linalg.h"
#include "macros.h"
#include "spline.h"
#include "render.h"

#include <math.h>

#define QUAD_TREE_MAX_DEPTH 3

static void check_bounds(const Vec v, float *dims) {
    if (v[0] < dims[0]) dims[0] = v[0];
    if (v[2] < dims[1]) dims[1] = v[2];
    if (v[0] > dims[2]) dims[2] = v[0];
    if (v[2] > dims[3]) dims[3] = v[2];
}

static void get_bounds(const Spline *spline, size_t i, float *dims) {
    const SplineBaked *baked[2];
    baked[0] = &spline->baked[i];
    baked[1] = &spline->baked[(i + 1) % spline->numBaked];
    Vec tmp, up, right, point, above, below;
    dims[0] = INFINITY;
    dims[1] = INFINITY;
    dims[2] = -INFINITY;
    dims[3] = -INFINITY;
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
        check_bounds(tmp, dims);
        vec_copy(tmp, above);
        vec_add(tmp, right);
        vec_add(tmp, point);
        check_bounds(tmp, dims);
        vec_copy(tmp, below);
        vec_sub(tmp, right);
        vec_add(tmp, point);
        check_bounds(tmp, dims);
        vec_copy(tmp, below);
        vec_add(tmp, right);
        vec_add(tmp, point);
        check_bounds(tmp, dims);
    }
}

static void build_quad_tree(QuadTreeNode *node, QuadTreeNode *childPool, int depth, size_t *w) {
    // set segments to empty list
    node->minXSegments = NULL;
    node->minZSegments = NULL;
    node->maxXSegments = NULL;
    node->maxZSegments = NULL;
    node->midSegments = NULL;
    if (depth >= QUAD_TREE_MAX_DEPTH) {
        // if at max depth, we don't add children
        node->children = NULL;
        return;
    }
    // add children
    node->children = &childPool[*w];
    *w += 4;
    for (int i = 0; i < 4; i++) {
        build_quad_tree(&node->children[i], childPool, depth + 1, w);
    }
}

static void add_node(QuadTree *tree, const float *dims, QuadTreeSegment *segment) {
    float min_x = tree->minX;
    float min_z = tree->minZ;
    float max_x = tree->maxX;
    float max_z = tree->maxZ;

    QuadTreeNode *current = &tree->root;
    int which_x, which_z;
    for (;;) {
        float center_x = (min_x + max_x) * 0.5f;
        float center_z = (min_z + max_z) * 0.5f;

        if (dims[0] < center_x && dims[2] < center_x) {
            which_x = 0;
            max_x = center_x;
        } else if (dims[0] > center_x && dims[2] > center_x) {
            which_x = 1;
            min_x = center_x;
        } else {
            which_x = -1;
        }

        if (dims[1] < center_z && dims[3] < center_z) {
            which_z = 0;
            max_z = center_z;
        } else if (dims[1] > center_z && dims[3] > center_z) {
            which_z = 1;
            min_z = center_z;
        } else {
            which_z = -1;
        }

        if (which_x < 0 || which_z < 0) {
            break;
        }

        if (current->children == NULL) {
            break;
        }
        current = &current->children[which_x | (which_z << 1)];
    }
    // add to list
    if (which_x == 0 && which_z == -1) {
        segment->next = current->minXSegments;
        current->minXSegments = segment;
    } else if (which_x == 1 && which_z == -1) {
        segment->next = current->maxXSegments;
        current->maxXSegments = segment;
    } else if (which_x == -1 && which_z == 0) {
        segment->next = current->minZSegments;
        current->minZSegments = segment;
    } else if (which_x == -1 && which_z == 1) {
        segment->next = current->maxZSegments;
        current->maxZSegments = segment;
    } else {
        segment->next = current->midSegments;
        current->midSegments = segment;
    }
}

bool quad_tree_init(QuadTree *tree, const Spline *spline) {
    // decide bounds of quadtree
    tree->minX = INFINITY;
    tree->minZ = INFINITY;
    tree->maxX = -INFINITY;
    tree->maxZ = -INFINITY;
    for (int i = 0; i < spline->numBaked; i++) {
        // get bounds of segment
        float dims[4];
        get_bounds(spline, i, dims);
        // update bounds
        if (dims[0] < tree->minX) tree->minX = dims[0];
        if (dims[1] < tree->minZ) tree->minZ = dims[1];
        if (dims[2] > tree->maxX) tree->maxX = dims[2];
        if (dims[3] > tree->maxZ) tree->maxZ = dims[3];
    }
    // build quadtree structure
    size_t w = 0;
    build_quad_tree(&tree->root, tree->childPool, 0, &w);
    // for each segment, figure out where to put it
    for (int i = 0; i < spline->numBaked; i++) {
        float dims[4];
        get_bounds(spline, i, dims);
        add_node(tree, dims, &tree->segmentPool[i]);
    }
    // for testing, print out the current tree
    return true;
}
