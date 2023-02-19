#include "assets.h"
#include "collision.h"
#include "input.h"
#include "linalg.h"
#include "macros.h"
#include "platform.h"
#include "render.h"
#include "spline.h"
#include "state.h"
#include "timing.h"
#include "vehicle.h"

#include <math.h>
#include <stdlib.h>
#include <string.h>

static VehicleType test_model = { 15.0f, 7.0f, 1.5f, 12.0f };

#ifdef CONDUX_WEB
void __wasm_call_ctors(void);
#endif

WEB_EXPORT("game_init")
void game_init(void) {
#ifdef CONDUX_WEB
    // prevents linker magic that we don't want
    __wasm_call_ctors();
#endif
    platform_init(640, 480);
    input_init();
    render_init();
    Asset asset;
    if (!asset_load(&asset, "mesh_vehicle.bin")) abort();
    if (!mesh_load(&test_model.mesh, &asset)) abort();
    if (!asset_load(&asset, "course_test1.bin")) abort();
    if (!game_state_init(&asset)) abort();
    render_load_spline(&gSpline);
    Vec spawn;
    spline_get_baked(&gSpline, 0.0f, spawn);
    game_state_spawn(spawn, &test_model, &gPlayerController);
    spline_get_baked(&gSpline, 5.0f, spawn);
    game_state_spawn(spawn, &test_model, &gEmptyController);
    spline_get_baked(&gSpline, 10.0f, spawn);
    game_state_spawn(spawn, &test_model, &gEmptyController);
    spline_get_baked(&gSpline, 15.0f, spawn);
    game_state_spawn(spawn, &test_model, &gEmptyController);
    game_state_teleport_camera(0);
    timing_init();
}

#ifndef CONDUX_WEB
void game_deinit(void) {
    platform_deinit();
}
#endif

static void game_logic(void) {
    input_poll();
    game_state_update(0);
}

static void game_render(float interpolation) {
    game_state_render(0, interpolation);
}

WEB_EXPORT("game_loop")
void game_loop(void) {
    float interpolation;
    uint16_t i = timing_num_ticks(&interpolation);
    while (i--) {
        game_logic();
    }
    game_render(interpolation);
}
