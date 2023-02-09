#include <math.h>

#include "linalg.h"
#include "macros.h"
#include "platform.h"
#include "render.h"

static float angle;

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

WEB_EXPORT("game_loop")
void game_loop(float delta) {
    angle = fmodf(delta + angle, 2.0f * PI);
    Vec lines[8];
    int j = 0;
    float cos_angle = cosf(angle);
    float sin_angle = sinf(angle);
    for (int z = -1; z <= 1; z += 2) {
        for (int y = -1; y <= 1; y += 2) {
            for (int x = -1; x <= 1; x += 2) {
                float px = 3.0f * x;
                float py = 3.0f * y;
                float pz = 20.0f * z;
                vec_set(lines[j++], px * cos_angle + pz * sin_angle, py, pz * cos_angle - px * sin_angle + 10.0f);
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
