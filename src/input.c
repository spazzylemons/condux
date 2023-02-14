#include "input.h"

Controls gControls;

void input_init(void) {
    gControls.buttons = 0;
    gControls.steering = 0.0f;
}

void input_poll(void) {
    platform_poll(&gControls);
}
