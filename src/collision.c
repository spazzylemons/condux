#include "collision.h"
#include "linalg.h"
#include "macros.h"
#include "spline.h"
#include "render.h"

#include <math.h>
#include <stdlib.h>
#include <stdio.h>

#define QUAD_TREE_MAX_DEPTH 3

static void check_bounds(const Vec v, float *dims) {
    if (v[0] < dims[0]) dims[0] = v[0];
    if (v[2] < dims[1]) dims[1] = v[2];
    if (v[0] > dims[2]) dims[2] = v[0];
    if (v[2] > dims[3]) dims[3] = v[2];
}

static void get_bounds(const Spline *spline, size_t i, float *dims) {
    const SplineBaked *b1 = &spline->baked[i];
    const SplineBaked *b2 = &spline->baked[(i + 1) % spline->numBaked];
    Vec up[2], right[2];
    Vec point[2];
    vec_copy(point[0], b1->point);
    vec_copy(point[1], b2->point);
    spline_get_up_right(spline, b1->offset, up[0], right[0]);
    spline_get_up_right(spline, b2->offset, up[1], right[1]);
    vec_scale(right[0], SPLINE_TRACK_RADIUS);
    vec_scale(right[1], SPLINE_TRACK_RADIUS);
    Vec test_point;
    dims[0] = INFINITY;
    dims[1] = INFINITY;
    dims[2] = -INFINITY;
    dims[3] = -INFINITY;
    for (int j = 0; j < 2; j++) {
        vec_copy(test_point, up[j]);
        vec_scale(test_point, MAX_GRAVITY_HEIGHT);
        vec_sub(test_point, right[j]);
        vec_add(test_point, point[j]);
        check_bounds(test_point, dims);
        vec_copy(test_point, up[j]);
        vec_scale(test_point, MAX_GRAVITY_HEIGHT);
        vec_add(test_point, right[j]);
        vec_add(test_point, point[j]);
        check_bounds(test_point, dims);
        vec_copy(test_point, up[j]);
        vec_scale(test_point, -COLLISION_DEPTH);
        vec_sub(test_point, right[j]);
        vec_add(test_point, point[j]);
        check_bounds(test_point, dims);
        vec_copy(test_point, up[j]);
        vec_scale(test_point, -COLLISION_DEPTH);
        vec_add(test_point, right[j]);
        vec_add(test_point, point[j]);
        check_bounds(test_point, dims);
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
    while (current->children != NULL) {
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
        segment->next = current->minZSegments;
        current->maxZSegments = segment;
    } else {
        segment->next = current->midSegments;
        current->midSegments = segment;
    }
}

static void print_segment_list(QuadTree *tree, const QuadTreeSegment *segment, int depth, const char *s) {
    for (int i = 0; i < depth; i++) printf("  ");
    printf("%s", s);
    size_t size = 0;
    while (segment != NULL) {
        size_t index = ((uintptr_t) segment - (uintptr_t) tree->segmentPool) / sizeof(QuadTreeSegment);
        printf(" %zu", index);
        segment = segment->next;
        ++size;
    }
    printf(" TOTAL %zu\n", size);
}

static void testing_print_tree(QuadTree *tree, const QuadTreeNode *node, int depth) {
    print_segment_list(tree, node->minXSegments, depth, "MIN_X");
    print_segment_list(tree, node->minZSegments, depth, "MIN_Z");
    print_segment_list(tree, node->minXSegments, depth, "MIN_X");
    print_segment_list(tree, node->maxZSegments, depth, "MAX_Z");
    print_segment_list(tree, node->midSegments, depth, "MID");
    if (node->children != NULL) {
        for (int i = 0; i < 4; i++) {
            testing_print_tree(tree, &node->children[i], depth + 1);
        }
    }
}

bool quad_tree_init(QuadTree *tree, const Spline *spline) {
    // allocate segments
    tree->segmentPool = malloc(sizeof(QuadTreeSegment) * spline->numBaked);
    if (!tree->segmentPool) return false;
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
    testing_print_tree(tree, &tree->root, 0);
    return true;
}

void quad_tree_free(const QuadTree *tree) {
    free(tree->segmentPool);
}

void quad_tree_test_print_colliding(const QuadTree *tree, float x, float z) {
    float min_x = tree->minX;
    float min_z = tree->minZ;
    float max_x = tree->maxX;
    float max_z = tree->maxZ;

    const QuadTreeNode *current = &tree->root;
    size_t num = 0;
    while (current->children != NULL) {

        float center_x = (min_x + max_x) * 0.5f;
        float center_z = (min_z + max_z) * 0.5f;

        int which_x, which_z;

        int which = 0;

        if (x < center_x) {
            which_x = 0;
            max_x = center_x;
        } else {
            which_x = 1;
            min_x = center_x;
        }

        if (z < center_z) {
            which_z = 0;
            max_z = center_z;
        } else {
            which_z = 1;
            min_z = center_z;
        }

        const QuadTreeSegment *segment = (which_x == 0) ? current->minXSegments : current->maxXSegments;
        while (segment != NULL) {
            ++num;
            segment = segment->next;
        }
        segment = (which_z == 0) ? current->minZSegments : current->maxZSegments;
        while (segment != NULL) {
            ++num;
            segment = segment->next;
        }
        segment = current->midSegments;
        while (segment != NULL) {
            ++num;
            segment = segment->next;
        }

        current = &current->children[which_x | (which_z << 1)];
    }
    printf("collision with %zu segments\n", num);
}
