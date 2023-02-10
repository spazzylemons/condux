#include <math.h>

#include "linalg.h"
#include "macros.h"
#include "platform.h"
#include "render.h"

static float angle;

static float lookAngle;
static Vec lookPos = { 0.0f, 0.0f, 0.0f };

WEB_EXPORT("game_init")
void game_init(void) {
    angle = 0.0f;
    platform_init(640, 480);
}

#ifndef CONDUX_WEB
void game_deinit(void) {
    platform_deinit();
}
#endif

static void game_logic(float delta) {
    angle = fmodf(delta + angle, 2.0f * PI);
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
    vec_scale(lookForwardVec, 4.0f * delta);
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
    Vec lines[8];
    int j = 0;
    float cos_angle = cosf(angle);
    float sin_angle = sinf(angle);
    for (int z = -1; z <= 1; z += 2) {
        for (int y = -1; y <= 1; y += 2) {
            for (int x = -1; x <= 1; x += 2) {
                float px = x;
                float py = y;
                float pz = z;
                vec_set(lines[j++], px * cos_angle + pz * sin_angle, py, pz * cos_angle - px * sin_angle + 5.0f);
            }
        }
    }
    render_line(lines[0], lines[1]);
    render_line(lines[1], lines[3]);
    render_line(lines[3], lines[2]);
    render_line(lines[2], lines[0]);
    render_line(lines[4], lines[5]);
    render_line(lines[5], lines[7]);
    render_line(lines[7], lines[6]);
    render_line(lines[6], lines[4]);
    render_line(lines[0], lines[4]);
    render_line(lines[1], lines[5]);
    render_line(lines[2], lines[6]);
    render_line(lines[3], lines[7]);
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
