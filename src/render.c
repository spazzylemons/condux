#include "assets.h"
#include "linalg.h"
#include "macros.h"
#include "platform.h"
#include "render.h"
#include "spline.h"

#include <math.h>
#include <stdlib.h>

#define CUTOFF 0.01f

static Vec camera_pos = { 0.0f, 0.0f, 0.0f };

static Mtx camera_mtx = {
    { 1.0f, 0.0f, 0.0f },
    { 0.0f, 1.0f, 0.0f },
    { 0.0f, 0.0f, 1.0f },
};

static Vec *spline_points_left = NULL;
static Vec *spline_points_right = NULL;
static size_t num_spline_points = 0;

void set_camera(const Vec eye, const Vec at, const Vec up) {
    Vec delta;
    vec_copy(delta, eye);
    vec_sub(delta, at);
    mtx_look_at(camera_mtx, delta, up);
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

bool mesh_load(Mesh *mesh, Asset *asset) {
    if (!asset_read_byte(asset, &mesh->numVertices)) return false;
    if (mesh->numVertices > MAX_MESH_VERTICES) return false;

    for (uint8_t i = 0; i < mesh->numVertices; i++) {
        if (!asset_read_vec(asset, mesh->vertices[i])) return false;
    }

    if (!asset_read_byte(asset, &mesh->numLines)) return false;
    if (mesh->numLines > MAX_MESH_LINES) return false;

    for (uint8_t i = 0; i < mesh->numLines; i++) {
        if (!asset_read_byte(asset, &mesh->line1[i])) return false;
        if (mesh->line1[i] >= mesh->numVertices) return false;
        if (!asset_read_byte(asset, &mesh->line2[i])) return false;
        if (mesh->line2[i] >= mesh->numVertices) return false;
    }

    return true;
}

void mesh_render(const Mesh *mesh, const Vec translation, const Mtx rotation) {
    for (uint8_t i = 0; i < mesh->numLines; i++) {
        Vec a, b;
        mtx_mul_vec(rotation, a, mesh->vertices[mesh->line1[i]]);
        mtx_mul_vec(rotation, b, mesh->vertices[mesh->line2[i]]);
        vec_add(a, translation);
        vec_add(b, translation);
        render_line(a, b);
    }
}
