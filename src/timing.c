#include "macros.h"
#include "platform.h"
#include "timing.h"

static uint64_t start_ms;
static uint64_t num_seconds = 0;
static uint8_t tick_in_second = 0;

static uint64_t tick_ms(void) {
    return start_ms + (1000 * num_seconds) + (1000 * (uint64_t) tick_in_second) / TICKS_PER_SECOND;
}

void timing_init(void) {
    start_ms = platform_time_msec();
}

uint16_t timing_num_ticks(float *interpolation) {
    uint64_t millis = platform_time_msec();
    uint16_t result = 0;
    while (millis >= tick_ms()) {
        if (++tick_in_second == TICKS_PER_SECOND) {
            tick_in_second = 0;
            ++num_seconds;
        }
        ++result;
    }
    float interp = 1.0f - ((tick_ms() - millis) * (TICKS_PER_SECOND / 1000.0f));
    // clamp just in case
    if (interp < 0.0f) interp = 0.0f;
    *interpolation = interp;
    // return number of ticks to run
    return result;
}
