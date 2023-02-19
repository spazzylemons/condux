#include "assets.h"
#include "collision.h"
#include "linalg.h"
#include "macros.h"
#include "render.h"
#include "spline.h"

#include <math.h>

#define BAKE_LENGTH_SQ 1.0f

#define FORWARD_VEC_SIZE 0.125f

static void bezier(const Spline *spline, uint8_t index, float offset, Vec v) {
    uint8_t otherIndex = (index + 2) % spline->numPoints;
    vec_scaled_copy(v, spline->points[index].point, (1.0f - offset) * (1.0f - offset));
    vec_scaled_add(v, spline->points[index].control, 2.0f * (1.0f - offset) * offset);
    vec_scaled_add(v, spline->points[otherIndex].point, offset * offset);
}

static void interpolate(const Spline *spline, float offset, Vec v) {
    uint8_t index = offset;
    offset -= index;
    index %= spline->numPoints;
    uint8_t prevIndex = (index + spline->numPoints - 1) % spline->numPoints;
    float prevMid = spline->points[prevIndex].controlMid;
    float nextMid = spline->points[index].controlMid;
    Vec temp;
    bezier(spline, prevIndex, offset * (1.0f - prevMid) + prevMid, v);
    bezier(spline, index, offset * nextMid, temp);
    vec_scale(v, 1.0f - offset);
    vec_scaled_add(v, temp, offset);
}

static void add_baked(Spline *spline, float position) {
    SplineBaked *baked = &spline->baked[spline->numBaked];
    baked->position = position;
    interpolate(spline, position, baked->point);
    if (spline->numBaked != 0) {
        spline->length += sqrtf(vec_distance_sq(baked->point, spline->baked[spline->numBaked - 1].point));
    }
    baked->offset = spline->length;
    ++spline->numBaked;
}

static void bake_recursive(Spline *spline, int index, float begin, float end, int depth) {
    if (depth >= MAX_BAKE_DEPTH) return;

    Vec v1, v2;
    interpolate(spline, index + begin, v1);
    interpolate(spline, index + end, v2);

    float segment_length_squared = vec_distance_sq(v1, v2);
    if (segment_length_squared > BAKE_LENGTH_SQ) {
        float mid = (begin + end) * 0.5f;
        // recurse on either end of midpoint - in-order to avoid sorting
        bake_recursive(spline, index, begin, mid, depth + 1);
        add_baked(spline, index + mid);
        bake_recursive(spline, index, mid, end, depth + 1);
    }
}

static void bake(Spline *spline) {
    // set starting size of spline to zero
    spline->length = 0.0f;
    // index to write baked points to
    spline->numBaked = 0;
    // for each point, recursively find points to bake
    for (uint8_t i = 0; i < spline->numPoints; i++) {
        // bake at control point
        add_baked(spline, i);
        // add length to tilt offsets
        spline->points[i].tiltOffset = spline->length;
        // bake in between
        bake_recursive(spline, i, 0.0f, 1.0f, 0);
    }
    // finish off length measurement
    spline->length += sqrtf(vec_distance_sq(spline->baked[0].point, spline->baked[spline->numBaked - 1].point));
}

static void generate_controls(Spline *spline) {
    // generate bezier control points
    for (uint8_t a = 0; a < spline->numPoints; a++) {
        uint8_t b = (a + 1) % spline->numPoints;
        uint8_t c = (a + 2) % spline->numPoints;
        float da = sqrtf(vec_distance_sq(spline->points[a].point, spline->points[b].point));
        float db = sqrtf(vec_distance_sq(spline->points[b].point, spline->points[c].point));
        // TODO handle potential divs by zero in this area
        float mid = da / (da + db);
        float fac_a = (mid - 1.0f) / (2.0f * mid);
        float fac_b = 1.0f / (2.0f * mid * (1.0f - mid));
        float fac_c = mid / (2.0f * (mid - 1.0f));
        vec_scaled_copy(spline->points[a].control, spline->points[a].point, fac_a);
        vec_scaled_add(spline->points[a].control, spline->points[b].point, fac_b);
        vec_scaled_add(spline->points[a].control, spline->points[c].point, fac_c);
        spline->points[a].controlMid = mid;
    }
}

bool spline_load(Spline *spline, Asset *asset) {
    // static buffers to hold data before building spline
    // this allows verifying input without modifying the existing spline
    // so if this function fails, no invaraints are invalidated
    static uint8_t numPoints;
    static Vec points[MAX_POINTS];
    static float tilts[MAX_POINTS];
    // number of points
    if (!asset_read_byte(asset, &numPoints)) return false;
    if (numPoints < 3 || numPoints > MAX_POINTS) return false;
    // read points in
    for (int i = 0; i < numPoints; i++) {
        if (!asset_read_vec(asset, points[i])) return false;
        uint8_t tilt_int;
        if (!asset_read_byte(asset, &tilt_int)) return false;
        tilts[i] = (tilt_int / 256.0f) * (2.0f * PI);
    }
    // TODO handle div by zero tests
    // otherwise data looks good
    spline->numPoints = numPoints;
    for (uint8_t i = 0; i < numPoints; i++) {
        vec_copy(spline->points[i].point, points[i]);
        spline->points[i].tilt = tilts[i];
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
    bake(spline);
    return true;
}

static float convert_baked_offset(const Spline *spline, float baked_offset) {
    // binary search
    size_t start = 0;
    size_t end = spline->numBaked;
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
    float offsetBegin = spline->baked[current].offset;
    float offsetEnd = spline->baked[(current + 1) % spline->numBaked].offset;
    float positionBegin = spline->baked[current].position;
    float positionEnd = spline->baked[(current + 1) % spline->numBaked].position;
    if (current == spline->numBaked - 1) {
        offsetEnd += spline->length;
        positionEnd += spline->numPoints;
    }
    float interp = (baked_offset - offsetBegin) / (offsetEnd - offsetBegin);
    float result = (1.0f - interp) * positionBegin + interp * positionEnd;
    return result;
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
    float a = lagrange(spline, index - 1, pre_baked);
    float b = lagrange(spline, index, pre_baked);
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

static float get_closest(const Spline *spline, const Octree *tree, const Vec point) {
    Vec min, max;
    vec_copy(min, tree->min);
    vec_copy(max, tree->max);

    const OctreeNode *current = &tree->root;
    float distance = INFINITY;
    float nearest = 0.0f;

    for (;;) {
        int which[3];
        octree_find_which(point, min, max, which);

        int segment = current->segments;
        while (segment >= 0) {
            if (!(which[0] == 1 && (tree->segmentSides[segment] & 1)) &&
                !(which[0] == 0 && (tree->segmentSides[segment] & 2)) &&
                !(which[1] == 1 && (tree->segmentSides[segment] & 4)) &&
                !(which[1] == 0 && (tree->segmentSides[segment] & 8)) &&
                !(which[2] == 1 && (tree->segmentSides[segment] & 16)) &&
                !(which[2] == 0 && (tree->segmentSides[segment] & 32))) {
                get_distance(spline, point, segment, &distance, &nearest);
            }
            segment = tree->segmentNext[segment];
        }

        if (current->children_index == -1) {
            break;
        }
        current = &tree->childPool[current->children_index + (which[0] | (which[1] << 1) | (which[2] << 2))];
    }

    return nearest;
}

bool spline_get_up_height(const Spline *spline, const Octree *tree, const Vec pos, Vec up, float *height) {
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
