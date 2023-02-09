#include "../platform.h"

#include <citro2d.h>

static C3D_RenderTarget *target;

static u64 last_time;

void platform_init(int preferred_width, int preferred_height) {
    gfxInitDefault();
    C3D_Init(C3D_DEFAULT_CMDBUF_SIZE);
    C2D_Init(C2D_DEFAULT_MAX_OBJECTS);
    C2D_Prepare();

    target = C2D_CreateScreenTarget(GFX_TOP, GFX_LEFT);
    C2D_SceneBegin(target);
    last_time = osGetTime();
}

void platform_deinit(void) {
    C2D_Fini();
    C3D_Fini();
    gfxExit();
}

void platform_line(float x0, float y0, float x1, float y1) {
    C2D_DrawLine(x0, y0, 0xffffffff, x1, y1, 0xffffffff, 1.0f, 0.0f);
}

bool platform_should_run(void) {
    return aptMainLoop();
}

float platform_start_frame(void) {
    C3D_FrameBegin(C3D_FRAME_SYNCDRAW);
    C2D_TargetClear(target, C2D_Color32f(0.0f, 0.0f, 0.0f, 1.0f));
    C2D_SceneBegin(target);
    u64 new_time = osGetTime();
    float result = (new_time - last_time) / 1000.0f;
    last_time = new_time;
    return result;
}

void platform_end_frame(void) {
    C3D_FrameEnd(0);
}

int platform_width(void) {
    return 400;
}

int platform_height(void) {
    return 240;
}
