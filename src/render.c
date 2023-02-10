#include "linalg.h"
#include "platform.h"
#include "render.h"

#include <math.h>
#include <stdio.h>

#define CUTOFF 0.01f

static Vec camera_pos = { 0.0f, 0.0f, 0.0f };

static Mtx camera_mtx = {
    1.0f, 0.0f, 0.0f,
    0.0f, 1.0f, 0.0f,
    0.0f, 0.0f, 1.0f,
};

void set_camera(const Vec eye, const Vec at, const Vec up) {
    vec_copy(camera_mtx[2], at);
    vec_sub(camera_mtx[2], eye);
    vec_normalize(camera_mtx[2]);
    vec_cross(camera_mtx[0], up, camera_mtx[2]);
    vec_normalize(camera_mtx[0]);
    vec_cross(camera_mtx[1], camera_mtx[2], camera_mtx[0]);
    mtx_transpose(camera_mtx);
    vec_copy(camera_pos, eye);
}

void render_line(const Vec a, const Vec b) {
    // perform camera transform
    Vec p, q, t;
    vec_copy(t, a);
    vec_sub(t, camera_pos);
    mtx_mul_vec(camera_mtx, p, t);
    vec_copy(t, b);
    vec_sub(t, camera_pos);
    mtx_mul_vec(camera_mtx, q, t);
    if (p[2] < CUTOFF && q[2] < CUTOFF) {
        // lies entirely behind camera, don't draw it
        return;
    }
    // sort endpoints
    if (p[2] > q[2]) {
        a = q;
        b = p;
    } else {
        a = p;
        b = q;
    }
    if (a[2] < CUTOFF && b[2] > CUTOFF) {
        // if line crosses, we need to cut the line
        float n = (b[2] - CUTOFF) / (b[2] - a[2]);
        Vec d;
        vec_copy(t, a);
        vec_copy(d, b);
        vec_scale(t, n);
        vec_scale(d, 1.0f - n);
        vec_add(t, d);
        a = t;
    }
    // adjust for screen res
    float width = platform_width();
    float height = platform_height();
    float scale = width < height ? width : height;
    // draw it
    float x0 = scale * (a[0] / a[2]) + (width / 2);
    float y0 = (height / 2) - scale * (a[1] / a[2]);
    float x1 = scale * (b[0] / b[2]) + (width / 2);
    float y1 = (height / 2) - scale * (b[1] / b[2]);
    platform_line(x0, y0, x1, y1);
}
