#include "assets.h"
#include "linalg.h"
#include "macros.h"
#include "platform.h"
#include "render.h"
#include "spline.h"

#include <math.h>
#include <stdlib.h>

static Vec *spline_points_left = NULL;
static Vec *spline_points_right = NULL;
static size_t num_spline_points = 0;

void render_load_spline(const Spline *spline) {
    size_t count = 2.0f + spline->length;
    Vec *left = malloc(sizeof(Vec) * count);
    Vec *right = malloc(sizeof(Vec) * count);
    if (left == NULL || right == NULL) {
        free(left);
        free(right);
        // failure to allocate
        return;
    }
    free(spline_points_left);
    free(spline_points_right);
    spline_points_left = left;
    spline_points_right = right;
    // load baked points
    float d = 0.0f;
    size_t index = 0;
    while (d < spline->length) {
        Vec p, r;
        spline_get_baked(spline, d, p);
        spline_get_up_right(spline, d, NULL, r);
        vec_scale(r, SPLINE_TRACK_RADIUS);
        vec_copy(spline_points_left[index], p);
        vec_sub(spline_points_left[index], r);
        vec_copy(spline_points_right[index], p);
        vec_add(spline_points_right[index], r);
        ++index;
        d += 1.0f;
    }
    vec_copy(spline_points_left[index], spline_points_left[0]);
    vec_copy(spline_points_right[index], spline_points_right[0]);
    num_spline_points = index;
}

void render_spline(void) {
    for (size_t i = 0; i < num_spline_points; i++) {
        render_line(spline_points_left[i], spline_points_left[i + 1]);
        render_line(spline_points_right[i], spline_points_right[i + 1]);
        render_line(spline_points_left[i], spline_points_right[i]);
    }
}

void render_deinit(void) {
    free(spline_points_left);
    free(spline_points_right);
    spline_points_left = NULL;
    spline_points_right = NULL;
}
