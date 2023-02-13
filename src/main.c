#include "assets.h"
#include "collision.h"
#include "linalg.h"
#include "macros.h"
#include "platform.h"
#include "render.h"
#include "spline.h"
#include "timing.h"
#include "vehicle.h"

#include <math.h>
#include <string.h>

static Spline s;
static QuadTree tree;
static bool has_spline = false;
static bool has_quad_tree = false;
static const VehicleType test_model = { 15.0f, 7.0f, 1.5f, 12.0f };
static Vehicle vehicle;
static Vehicle previous_vehicle;
static Mesh vehicle_mesh;
static bool has_mesh = false;

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
        render_load_spline(&s);
        if (has_spline) {
            has_quad_tree = quad_tree_init(&tree, &s);
            spline_get_baked(&s, 0.0f, vehicle.position);
            vehicle.position[1] += 1.0f;
            quat_copy(vehicle.rotation, gQuatIdentity);
            vec_copy(vehicle.velocity, gVecZero);
            vehicle.controller = &player_controller_temp;
            vehicle.type = &test_model;
            memcpy(&previous_vehicle, &vehicle, sizeof(Vehicle));
        }
    }
    if (asset_load(&asset, "mesh_vehicle.bin")) {
        has_mesh = mesh_load(&vehicle_mesh, &asset);
    }
    timing_init();
}

#ifndef CONDUX_WEB
void game_deinit(void) {
    render_deinit();
    platform_deinit();
}
#endif

static void game_logic(void) {
    float delta = 1.0f / TICKS_PER_SECOND;
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
        memcpy(&previous_vehicle, &vehicle, sizeof(Vehicle));
        vehicle_update(&vehicle, &s, &tree, delta);
    }
}

static void game_render(float interpolation) {
    Vec vehicle_pos, tmp_vec;
    vec_copy(tmp_vec, vehicle.position);
    vec_scale(tmp_vec, interpolation);
    vec_copy(vehicle_pos, previous_vehicle.position);
    vec_scale(vehicle_pos, 1.0f - interpolation);
    vec_add(vehicle_pos, tmp_vec);

    Quat vehicle_rot;
    quat_slerp(vehicle_rot, previous_vehicle.rotation, vehicle.rotation, interpolation);

    Vec camera_pos;
    Vec forward_vehicle;
    Vec up_vehicle;
    vec_copy(camera_pos, vehicle_pos);
    Mtx vehicle_rotation;
    quat_to_mtx(vehicle_rotation, vehicle_rot);
    mtx_mul_vec(vehicle_rotation, forward_vehicle, gVecZAxis);
    mtx_mul_vec(vehicle_rotation, up_vehicle, gVecYAxis);
    vec_scale(forward_vehicle, 5.0f);
    vec_sub(camera_pos, forward_vehicle);
    vec_add(camera_pos, up_vehicle);
    set_camera(camera_pos, vehicle_pos, up_vehicle);
    // draw something to show where the vehicle is
    if (has_mesh) {
        mesh_render(&vehicle_mesh, vehicle_pos, vehicle_rotation);
    }
    render_spline();
}

WEB_EXPORT("game_loop")
void game_loop(void) {
    float interpolation;
    uint16_t i = timing_num_ticks(&interpolation);
    while (i--) {
        game_logic();
    }
    // TODO interpolation
    game_render(interpolation);
}

#ifndef CONDUX_WEB
int main(void) {
    game_init();
    while (platform_should_run()) {
        platform_start_frame();
        game_loop();
        platform_end_frame();
    }
    game_deinit();
}
#endif
