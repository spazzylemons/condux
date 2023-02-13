#include <math.h>

#include "assets.h"
#include "collision.h"
#include "linalg.h"
#include "macros.h"
#include "platform.h"
#include "render.h"
#include "spline.h"
#include "vehicle.h"

static Spline s;
static QuadTree tree;
static bool has_spline = false;
static bool has_quad_tree = false;
static const VehicleType test_model = { 15.0f, 7.0f, 1.5f, 12.0f };
static Vehicle vehicle;

static float steer = 0.0f;
static float pedal = 0.0f;

static float temp_get_steering(VehicleController *controller) {
    return steer;
}

static float temp_get_pedal(VehicleController *controller) {
    return pedal;
}

static VehicleController player_controller_temp = {
    .getSteering = temp_get_steering,
    .getPedal = temp_get_pedal,
};

WEB_EXPORT("game_init")
void game_init(void) {
    platform_init(640, 480);
    Asset asset;
    if (asset_load(&asset, "course_test1.bin")) {
        has_spline = spline_load(&s, &asset);
        if (has_spline) {
            has_quad_tree = quad_tree_init(&tree, &s);
            spline_get_baked(&s, 0.0f, vehicle.position);
            vehicle.position[1] += 1.0f;
            mtx_copy(vehicle.rotation, gMtxIdentity);
            vec_copy(vehicle.velocity, gVecZero);
            vehicle.controller = &player_controller_temp;
            vehicle.type = &test_model;
        }
    }
}

#ifndef CONDUX_WEB
void game_deinit(void) {
    if (has_spline) {
        spline_free(&s);
        if (has_quad_tree) {
            quad_tree_free(&tree);
        }
    }
    platform_deinit();
}
#endif

static void game_logic(float delta) {
    // temporary to fix variable timestamp problems
    delta = 1.0f / 60.0f;
    Controls controls;
    platform_poll(&controls);
    if (controls.buttons & BTN_OK) {
        pedal = 1.0f;
    } else if (controls.buttons & BTN_BACK) {
        pedal = -1.0f;
    } else {
        pedal = 0.0f;
    }
    steer = controls.steering;
    if (has_quad_tree) {
        vehicle_update(&vehicle, &s, &tree, delta);
    }
}

static void game_render(void) {
    Vec camera_pos;
    Vec forward_vehicle;
    Vec up_vehicle;
    vec_copy(camera_pos, vehicle.position);
    mtx_mul_vec(vehicle.rotation, forward_vehicle, gVecZAxis);
    mtx_mul_vec(vehicle.rotation, up_vehicle, gVecYAxis);
    vec_scale(forward_vehicle, 5.0f);
    vec_sub(camera_pos, forward_vehicle);
    vec_add(camera_pos, up_vehicle);
    set_camera(camera_pos, vehicle.position, up_vehicle);
    // draw something to show where the vehicle is
    Vec points[4] = {
        { 1.0f, 0.0f, 1.0f },
        { 1.0f, 0.0f, -1.0f },
        { -1.0f, 0.0f, -1.0f },
        { -1.0f, 0.0f, 1.0f },
    };
    for (int i = 0; i < 4; i++) {
        Vec p1, p2, q1, q2;
        vec_copy(p1, points[i]);
        vec_copy(p2, points[(i + 1) % 4]);
        mtx_mul_vec(vehicle.rotation, q1, p1);
        mtx_mul_vec(vehicle.rotation, q2, p2);
        vec_add(q1, vehicle.position);
        vec_add(q2, vehicle.position);
        render_line(q1, q2);
    }
    if (has_spline) {
        spline_test_render(&s);
    }
}

WEB_EXPORT("game_loop")
void game_loop(float delta) {
    game_logic(delta);
    game_render();
}

#ifndef CONDUX_WEB
int main(void) {
    game_init();
    while (platform_should_run()) {
        float delta = platform_start_frame();
        game_loop(delta);
        platform_end_frame();
    }
    game_deinit();
}
#endif
