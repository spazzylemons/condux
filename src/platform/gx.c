#include "../platform.h"

#include <gccore.h>
#include <malloc.h>
#include <string.h>

u64 gettime(void);
u32 diff_usec(u64 start, u64 end);

#define FIFO_SIZE (256 * 1024)

static GXRModeObj *screen_mode;
static void *backbuffer;

static u16 colors[] ATTRIBUTE_ALIGN(32) = { 65535 };

static u64 last_time;

void platform_init(int preferred_width, int preferred_height) {
    VIDEO_Init();
    screen_mode = VIDEO_GetPreferredMode(NULL);
    backbuffer = MEM_K0_TO_K1(SYS_AllocateFramebuffer(screen_mode));
    VIDEO_Configure(screen_mode);
    VIDEO_SetNextFramebuffer(backbuffer);
    VIDEO_SetBlack(TRUE);
    VIDEO_Flush();
    // initialize GX
    void *fifo_buffer = MEM_K0_TO_K1(memalign(32, FIFO_SIZE));
    memset(fifo_buffer, 0, FIFO_SIZE);
    GX_Init(fifo_buffer, FIFO_SIZE);
    // black background to copy with
    GXColor color = {0, 0, 0, 255};
    GX_SetCopyClear(color, 0x00ffffff);
    // set gamma of screen copy
    GX_SetDispCopyGamma(GX_GM_1_0);
    // initialize GX based on preferred screen mode
    GX_SetViewport(0.0f, 0.0f, screen_mode->fbWidth, screen_mode->efbHeight, 0.0f, 1.0f);
    GX_SetDispCopyYScale((float) screen_mode->xfbHeight / (float) screen_mode->efbHeight);
    GX_SetDispCopySrc(0, 0, screen_mode->fbWidth, screen_mode->efbHeight);
    GX_SetDispCopyDst(screen_mode->fbWidth, screen_mode->xfbHeight);
    GX_SetCopyFilter(screen_mode->aa, screen_mode->sample_pattern, GX_TRUE, screen_mode->vfilter);
    GX_SetFieldMode(screen_mode->field_rendering, screen_mode->viHeight == 2 * screen_mode->xfbHeight);
    // run one copy to clear the buffer
    GX_CopyDisp(backbuffer, GX_TRUE);
    // set up vertex format
    GX_ClearVtxDesc();
    // direct draw lines, use index for color
    GX_SetVtxDesc(GX_VA_POS, GX_DIRECT);
    GX_SetVtxDesc(GX_VA_CLR0, GX_INDEX8);
    // 2D lines, RGB color
    GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XY, GX_F32, 0);
    GX_SetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGB, GX_RGB565, 0);
    // load color array
    GX_SetArray(GX_VA_CLR0, colors, sizeof(u16));
    GX_SetNumChans(1);
    GX_SetNumTexGens(0);
    GX_SetTevOrder(GX_TEVSTAGE0, GX_TEXCOORDNULL, GX_TEXMAP_NULL, GX_COLOR0A0);
    GX_SetTevOp(GX_TEVSTAGE0, GX_PASSCLR);
    // don't need to invalidate the vertex cache while drawing, as all of the
    // data that changes each frame is loaded directly
    GX_InvVtxCache();
    // setup the orthographic view matrix
    Mtx view;
    guMtxIdentity(view);
    GX_LoadPosMtxImm(view, GX_PNMTX0);
    // setup the orthographic projection matrix
    Mtx44 proj;
    guOrtho(proj, 0, screen_mode->efbHeight - 1, 0, screen_mode->fbWidth - 1, 0, 300);
    GX_LoadProjectionMtx(proj, GX_ORTHOGRAPHIC);
    // load current time
    last_time = gettime();
}

void platform_deinit(void) {
    // nothing to close on this platform yet
}

void platform_line(float x0, float y0, float x1, float y1) {
    GX_Begin(GX_LINES, GX_VTXFMT0, 2);
    GX_Position2f32(x0, y0);
    GX_Color1x8(0);
    GX_Position2f32(x1, y1);
    GX_Color1x8(0);
    GX_End();
}

bool platform_should_run(void) {
    return true;
}

float platform_start_frame(void) {
    u64 new_time = gettime();
    float result = (float) diff_usec(last_time, new_time) / 1000000.0f;
    last_time = new_time;
    return result;
}

void platform_end_frame(void) {
    GX_DrawDone();
    VIDEO_WaitVSync();
    VIDEO_SetBlack(FALSE);
    VIDEO_Flush();
    GX_SetColorUpdate(GX_TRUE);
    GX_CopyDisp(backbuffer, GX_TRUE);
    GX_Flush();
}

int platform_width(void) {
    return screen_mode->fbWidth;
}

int platform_height(void) {
    return screen_mode->efbHeight;
}
