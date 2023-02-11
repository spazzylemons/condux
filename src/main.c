#include <math.h>

#include "assets.h"
#include "linalg.h"
#include "macros.h"
#include "platform.h"
#include "render.h"
#include "spline.h"

static float lookAngle;
static Vec lookPos = { 0.0f, 0.0f, 0.0f };
static Spline s;
static bool has_spline = false;

WEB_EXPORT("game_init")
void game_init(void) {
    platform_init(640, 480);
    Asset asset;
    if (asset_load(&asset, "course_test1.bin")) {
        has_spline = spline_load(&s, &asset);
    }
}

#ifndef CONDUX_WEB
void game_deinit(void) {
    if (has_spline) {
        spline_free(&s);
    }
    platform_deinit();
}
#endif

static void game_logic(float delta) {
    Controls controls;
    platform_poll(&controls);
    lookAngle += controls.steering * delta;
    if (lookAngle > 2.0f * PI) {
        lookAngle -= 2.0f * PI;
    } else if (lookAngle < 0.0f) {
        lookAngle += 2.0f * PI;
    }
    Vec lookForwardVec;
    vec_set(lookForwardVec, sinf(lookAngle), 0.0f, cosf(lookAngle));
    vec_scale(lookForwardVec, 10.0f * delta);
    if (controls.buttons & BTN_UP) {
        vec_add(lookPos, lookForwardVec);
    } else if (controls.buttons & BTN_DOWN) {
        vec_sub(lookPos, lookForwardVec);
    }
    if (controls.buttons & BTN_BACK) {
        lookPos[1] -= 4.0f * delta;
    }
    if (controls.buttons & BTN_OK) {
        lookPos[1] += 4.0f * delta;
    }
}

static void game_render(void) {
    // render from this camera position
    Vec lookForwardVec;
    vec_set(lookForwardVec, sinf(lookAngle), 0.0f, cosf(lookAngle));
    vec_add(lookForwardVec, lookPos);
    set_camera(lookPos, lookForwardVec, gVecYAxis);
    // render a cube
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
