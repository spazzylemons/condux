#include "platform.h"

#include <citro2d.h>
#include <math.h>

static C3D_RenderTarget *target;

#define DEADZONE 0.03f

void platform_init(int preferred_width, int preferred_height) {
    hidInit();
    gfxInitDefault();
    C3D_Init(C3D_DEFAULT_CMDBUF_SIZE);
    C2D_Init(C2D_DEFAULT_MAX_OBJECTS);
    C2D_Prepare();

    target = C2D_CreateScreenTarget(GFX_TOP, GFX_LEFT);
    C2D_SceneBegin(target);
}

void platform_deinit(void) {
    C2D_Fini();
    C3D_Fini();
    gfxExit();
    hidExit();
}

void platform_line(float x0, float y0, float x1, float y1) {
    C2D_DrawLine(x0, y0, 0xffffffff, x1, y1, 0xffffffff, 1.0f, 0.0f);
}

bool platform_should_run(void) {
    return aptMainLoop();
}

uint64_t platform_time_msec(void) {
    return osGetTime();
}

void platform_start_frame(void) {
    C3D_FrameBegin(C3D_FRAME_SYNCDRAW);
    C2D_TargetClear(target, C2D_Color32f(0.0f, 0.0f, 0.0f, 1.0f));
    C2D_SceneBegin(target);
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

void platform_poll(Controls *controls) {
    controls->buttons = 0;
    hidScanInput();
    u32 held = hidKeysHeld();
    if (held & KEY_UP) controls->buttons |= BTN_UP;
    if (held & KEY_DOWN) controls->buttons |= BTN_DOWN;
    if (held & KEY_LEFT) controls->buttons |= BTN_LEFT;
    if (held & KEY_RIGHT) controls->buttons |= BTN_RIGHT;
    if (held & KEY_B) controls->buttons |= BTN_BACK;
    if (held & KEY_A) controls->buttons |= BTN_OK;
    if (held & KEY_START) controls->buttons |= BTN_PAUSE;
    circlePosition cpos;
    hidCircleRead(&cpos);
    controls->steering = cpos.dx / 156.0f;
    if (fabsf(controls->steering) < DEADZONE) {
        controls->steering = 0.0f;
    }
    if (controls->steering > 1.0f) {
        controls->steering = 1.0f;
    } else if (controls->steering < -1.0f) {
        controls->steering = -1.0f;
    }
}
