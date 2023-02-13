#include "assets.h"
#include "collision.h"
#include "linalg.h"
#include "macros.h"
#include "render.h"
#include "spline.h"

#include <math.h>
#include <stdlib.h>

#define BAKE_LENGTH_SQ 1.0f

#define FORWARD_VEC_SIZE 0.125f

static void bezier(const Spline *spline, uint8_t index, float offset, Vec v) {
    float fac_a = (1.0f - offset) * (1.0f - offset);
    float fac_b = 2.0f * (1.0f - offset) * offset;
    float fac_c = offset * offset;
    uint8_t other_index = (index + 2) % spline->numPoints;
    Vec temp;
    vec_copy(v, spline->points[index].point);
    vec_scale(v, fac_a);
    vec_copy(temp, spline->points[index].control);
    vec_scale(temp, fac_b);
    vec_add(v, temp);
    vec_copy(temp, spline->points[other_index].point);
    vec_scale(temp, fac_c);
    vec_add(v, temp);
}

static void interpolate(const Spline *spline, float offset, Vec v) {
    offset = fmodf(offset, spline->numPoints);
    int index = offset;
    offset -= index;
    Vec temp;
    bezier(spline, (index + spline->numPoints - 1) % spline->numPoints, offset * 0.5f + 0.5f, v);
    bezier(spline, index, offset * 0.5f, temp);
    vec_scale(v, 1.0f - offset);
    vec_scale(temp, offset);
    vec_add(v, temp);
}

static void find_baked_recursive(Spline *spline, int index, float begin, float end, int depth) {
    if (depth >= MAX_BAKE_DEPTH) return;

    Vec v1, v2;
    interpolate(spline, index + begin, v1);
    interpolate(spline, index + end, v2);

    float segment_length_squared = vec_distance_sq(v1, v2);
    if (segment_length_squared > BAKE_LENGTH_SQ) {
        float mid = (begin + end) * 0.5f;
        ++spline->numBaked;
        // recurse on either end of midpoint
        find_baked_recursive(spline, index, begin, mid, depth + 1);
        find_baked_recursive(spline, index, mid, end, depth + 1);
    }
}

static void find_baked_size(Spline *spline) {
    // start at zero
    spline->numBaked = 0;
    // for each point, recursively find points to bake
    for (int i = 0; i < spline->numPoints; i++) {
        // bake at control point
        ++spline->numBaked;
        // bake in between
        find_baked_recursive(spline, i, 0.0f, 1.0f, 0);
    }
}

static void add_baked(Spline *spline, float position, size_t *w) {
    SplineBaked *baked = &spline->baked[*w];
    baked->position = position;
    interpolate(spline, position, baked->point);
    if (*w) {
        spline->length += sqrtf(vec_distance_sq(baked->point, spline->baked[*w - 1].point));
    }
    baked->offset = spline->length;
    ++*w;
}

static void bake_recursive(Spline *spline, int index, float begin, float end, int depth, size_t *w) {
    if (depth >= MAX_BAKE_DEPTH) return;

    Vec v1, v2;
    interpolate(spline, index + begin, v1);
    interpolate(spline, index + end, v2);

    float segment_length_squared = vec_distance_sq(v1, v2);
    if (segment_length_squared > BAKE_LENGTH_SQ) {
        float mid = (begin + end) * 0.5f;
        // recurse on either end of midpoint - in-order to avoid sorting
        bake_recursive(spline, index, begin, mid, depth + 1, w);
        add_baked(spline, index + mid, w);
        bake_recursive(spline, index, mid, end, depth + 1, w);
    }
}

static void bake(Spline *spline) {
    // index to write to
    size_t w = 0;
    // set starting size of spline to zero
    spline->length = 0.0f;
    // for each point, recursively find points to bake
    for (int i = 0; i < spline->numPoints; i++) {
        // bake at control point
        add_baked(spline, i, &w);
        // add length to tilt offsets
        spline->points[i].tiltOffset = spline->length;
        // bake in between
        bake_recursive(spline, i, 0.0f, 1.0f, 0, &w);
    }
    // finish off length measurement
    spline->length += sqrtf(vec_distance_sq(spline->baked[0].point, spline->baked[spline->numBaked - 1].point));
}

static void generate_controls(Spline *spline) {
    // generate bezier control points
    for (int a = 0; a < spline->numPoints; a++) {
        int b = (a + 1) % spline->numPoints;
        int c = (a + 2) % spline->numPoints;
        float da = sqrtf(vec_distance_sq(spline->points[a].point, spline->points[b].point));
        float db = sqrtf(vec_distance_sq(spline->points[b].point, spline->points[c].point));
        // TODO handle potential divs by zero in this area
        float mid = da / (da + db);
        float fac_a = (mid - 1.0f) / (2.0f * mid);
        float fac_b = (1.0f) / (2.0f * mid * (1.0f - mid));
        float fac_c = mid / (2.0f * (mid - 1.0f));
        Vec temp;
        vec_copy(spline->points[a].control, spline->points[a].point);
        vec_scale(spline->points[a].control, fac_a);
        vec_copy(temp, spline->points[b].point);
        vec_scale(temp, fac_b);
        vec_add(spline->points[a].control, temp);
        vec_copy(temp, spline->points[c].point);
        vec_scale(temp, fac_c);
        vec_add(spline->points[a].control, temp);
    }
}

static bool bake_points(Spline *spline) {
    // first bake pass to see how many points we need
    find_baked_size(spline);
    // bake points
    bake(spline);
    return true;
}

bool spline_load(Spline *spline, Asset *asset) {
    // number of points
    if (!asset_read_byte(asset, &spline->numPoints)) return false;
    if (spline->numPoints < 3 || spline->numPoints > MAX_POINTS) return false;
    // read points in
    for (int i = 0; i < spline->numPoints; i++) {
        if (!asset_read_vec(asset, spline->points[i].point)) return false;
        uint8_t tilt_int;
        if (!asset_read_byte(asset, &tilt_int)) return false;
        spline->points[i].tilt = (tilt_int / 256.0f) * (2.0f * PI);
    }
    // fix tilts
    spline->totalTilt = spline->points[0].tilt;
    for (int i = 0; i < spline->numPoints; i++) {
        float delta = fmodf(spline->points[(i + 1) % spline->numPoints].tilt - spline->points[i].tilt + 2.0f * PI, 2.0f * PI);
        spline->points[i].tilt = spline->totalTilt;
        if (delta <= PI) {
            // move up
            spline->totalTilt += delta;
        } else {
            // move down
            spline->totalTilt += delta - (2.0f * PI);
        }
    }
    generate_controls(spline);
    if (!bake_points(spline)) return false;
    return true;
}

static float convert_baked_offset(const Spline *spline, float baked_offset) {
    // binary search
    size_t start = 0;
    size_t end = spline->numBaked - 1;
    size_t current = (start + end) / 2;
    while (start < current) {
        if (baked_offset <= spline->baked[current].offset) {
            end = current;
        } else {
            start = current;
        }
        current = (start + end) / 2;
    }
    // interpolate
    float offset_begin = spline->baked[current].offset;
    float offset_end = spline->baked[current + 1].offset;
    float interp = (baked_offset - offset_begin) / (offset_end - offset_begin);
    return (1.0f - interp) * spline->baked[current].position + interp * spline->baked[current + 1].position;
}

void spline_get_baked(const Spline *spline, float offset, Vec v) {
    interpolate(spline, convert_baked_offset(spline, offset), v);
}

static float get_tilt_offset(const Spline *spline, int i) {
    int n = i / spline->numPoints;
    return spline->length * n + spline->points[i - n * spline->numPoints].tiltOffset;
}

static float get_tilt_radian(const Spline *spline, int i) {
    int n = i / spline->numPoints;
    return spline->totalTilt * n + spline->points[i - n * spline->numPoints].tilt;
}

static float lagrange(const Spline *spline, int i, float x) {
    // TODO optimize
    float x0 = get_tilt_offset(spline, i);
    float x1 = get_tilt_offset(spline, i + 1);
    float x2 = get_tilt_offset(spline, i + 2);
    float y0 = get_tilt_radian(spline, i);
    float y1 = get_tilt_radian(spline, i + 1);
    float y2 = get_tilt_radian(spline, i + 2);
	float result = (y0 * (x - x1) / (x0 - x1) * (x - x2) / (x0 - x2));
	result += (y1 * (x - x0) / (x1 - x0) * (x - x2) / (x1 - x2));
	result += (y2 * (x - x0) / (x2 - x0) * (x - x1) / (x2 - x1));
    return result;
}

float spline_get_tilt(const Spline *spline, float offset) {
    float pre_baked = fmodf(offset, spline->length);
    offset = convert_baked_offset(spline, offset);
    int index = offset;
    float a = lagrange(spline, index + spline->numPoints - 1, pre_baked + spline->length);
    float b = lagrange(spline, index + spline->numPoints, pre_baked + spline->length);
    offset -= index;
    return a * (1.0f - offset) + b * offset;
}

void spline_get_up_right(const Spline *spline, float offset, Vec up, Vec right) {
    float sa = fmodf(offset - FORWARD_VEC_SIZE + spline->length, spline->length);
    float sb = fmodf(offset + FORWARD_VEC_SIZE + spline->length, spline->length);
    Vec target, temp;
    spline_get_baked(spline, sb, target);
    spline_get_baked(spline, sa, temp);
    vec_sub(target, temp);
    vec_normalize(target);
    Mtx look, rot;
    // TODO would have issues for track going directly upwards
    mtx_look_at(look, target, gVecYAxis);
    float tilt = spline_get_tilt(spline, offset);
    if (up != NULL) {
        mtx_mul_vec(look, temp, gVecYAxis);
        mtx_angle_axis(rot, target, tilt);
        mtx_mul_vec(rot, up, temp);
    }
    if (right != NULL) {
        mtx_mul_vec(look, temp, gVecXAxis);
        mtx_angle_axis(rot, target, tilt);
        mtx_mul_vec(rot, right, temp);
    }
}

static void get_distance(const Spline *spline, const Vec point, int i, float *distance, float *nearest) {
    float offset = spline->baked[i].offset;
    float interval = spline->baked[(i + 1) % spline->numBaked].offset - offset;
    Vec origin;
    vec_copy(origin, spline->baked[i].point);
    Vec direction;
    vec_copy(direction, spline->baked[(i + 1) % spline->numBaked].point);
    vec_sub(direction, origin);
    vec_scale(direction, 1.0f / interval);
    Vec proj;
    vec_copy(proj, point);
    vec_sub(proj, origin);
    float d = vec_dot(proj, direction);
    if (d < 0.0f) d = 0.0f;
    else if (d > interval) d = interval;
    vec_copy(proj, direction);
    vec_scale(proj, d);
    vec_add(proj, origin);
    float dist = vec_distance_sq(proj, point);
    if (dist < *distance) {
        *nearest = offset + d;
        *distance = dist;
    }
}

static int get_segment_id(const QuadTree *tree, const QuadTreeSegment *segment) {
    return ((uintptr_t) segment - (uintptr_t) tree->segmentPool) / sizeof(QuadTreeSegment);
}

static float get_closest(const Spline *spline, const QuadTree *tree, const Vec point) {
    // quadtree for search
    float min_x = tree->minX;
    float min_z = tree->minZ;
    float max_x = tree->maxX;
    float max_z = tree->maxZ;

    const QuadTreeNode *current = &tree->root;
    float distance = INFINITY;
    float nearest = 0.0f;

    for (;;) {
        float center_x = (min_x + max_x) * 0.5f;
        float center_z = (min_z + max_z) * 0.5f;

        int which_x, which_z;

        if (point[0] < center_x) {
            which_x = 0;
            max_x = center_x;
        } else {
            which_x = 1;
            min_x = center_x;
        }

        if (point[2] < center_z) {
            which_z = 0;
            max_z = center_z;
        } else {
            which_z = 1;
            min_z = center_z;
        }

        const QuadTreeSegment *segment = (which_x == 0) ? current->minXSegments : current->maxXSegments;
        while (segment != NULL) {
            get_distance(spline, point, get_segment_id(tree, segment), &distance, &nearest);
            segment = segment->next;
        }
        segment = (which_z == 0) ? current->minZSegments : current->maxZSegments;
        while (segment != NULL) {
            get_distance(spline, point, get_segment_id(tree, segment), &distance, &nearest);
            segment = segment->next;
        }
        segment = current->midSegments;
        while (segment != NULL) {
            get_distance(spline, point, get_segment_id(tree, segment), &distance, &nearest);
            segment = segment->next;
        }

        if (current->children == NULL) {
            break;
        }
        current = &current->children[which_x | (which_z << 1)];
    }

    return nearest;
}

bool spline_get_up_height(const Spline *spline, const QuadTree *tree, const Vec pos, Vec up, float *height) {
    float offset = get_closest(spline, tree, pos);
    Vec point;
    spline_get_baked(spline, offset, point);
    Vec right;
    spline_get_up_right(spline, offset, up, right);
    Vec d;
    vec_copy(d, pos);
    vec_sub(d, point);
    float side_distance = vec_dot(right, d);
    if (side_distance < -SPLINE_TRACK_RADIUS || side_distance > SPLINE_TRACK_RADIUS) {
        return false;
    }
    *height = vec_dot(up, d);
    return true;
}