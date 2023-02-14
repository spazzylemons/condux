#ifndef CONDUX_STATE_H
#define CONDUX_STATE_H

#include "types.h"

#include <stdbool.h>

extern Spline gSpline;

bool game_state_init(Asset *spline_asset);

bool game_state_spawn(const Vec pos, const VehicleType *type, VehicleController *controller);

void game_state_update(float delta);

void game_state_render(uint8_t cameraFocus, float interpolation);

#endif
