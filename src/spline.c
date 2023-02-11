#include "assets.h"
#include "linalg.h"
#include "macros.h"
#include "render.h"
#include "spline.h"

#include <math.h>

static Vec tempPoints[30 * MAX_POINTS];

static void bezier(Spline *spline, uint8_t index, float offset, Vec v) {
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

static void interpolate(Spline *spline, float offset, Vec v) {
    int index = (int) offset % spline->numPoints;
    offset = (fmodf(offset, spline->numPoints) - index);
    Vec temp;
    bezier(spline, (index + spline->numPoints - 1) % spline->numPoints, offset * 0.5f + 0.5f, v);
    bezier(spline, index, offset * 0.5f, temp);
    vec_scale(v, 1.0f - offset);
    vec_scale(temp, offset);
    vec_add(v, temp);

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
    float start_tilt = spline->points[0].tilt;
    spline->totalTilt = start_tilt;
    for (int i = 0; i < spline->numPoints; i++) {
        float delta = fmodf(spline->points[(i % spline->numPoints) + 1].tilt - spline->points[i].tilt + 2.0f * PI, 2.0f * PI);
        if (i != 0) {
            // to avoid issues when we hit the last point
            spline->points[i].tilt = spline->totalTilt;
        }
        if (delta <= PI) {
            // move up
            spline->totalTilt += delta;
        } else {
            // move down
            spline->totalTilt += delta - (2.0f * PI);
        }
    }
    spline->points[0].tilt = start_tilt;
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
    int foo = 0;
    bool has_previous = false;
    for (int i = 0; i < spline->numPoints; i++) {
        for (int j = 0; j < 30; j++) {
            interpolate(spline, i + j / 30.0f, tempPoints[foo++]);
        }
    }
    return true;
}

void spline_test_render(Spline *spline) {
    int x = (int) spline->numPoints * 30;
    for (int i = 0; i < x; i++) {
        render_line(tempPoints[i], tempPoints[(i + 1) % x]);
    }
}
