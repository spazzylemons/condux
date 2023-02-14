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

WEB_EXPORT("game_init")
void game_init(void) {
    platform_init(640, 480);
    input_init();
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
    input_poll();
    game_state_update(delta);
}

static void game_render(float interpolation) {
    game_state_render(0, interpolation);
    render_spline();
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
