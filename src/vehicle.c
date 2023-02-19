#include "input.h"
#include "linalg.h"
#include "macros.h"
#include "spline.h"
#include "vehicle.h"

static float player_controller_steering(VehicleController *controller) {
    return gControls.steering;
}

static float player_controller_pedal(VehicleController *controller) {
    if (gControls.buttons & BTN_BACK) {
        return -1.0f;
    } else if (gControls.buttons & BTN_OK) {
        return 1.0f;
    } else {
        return 0.0f;
    }
}

VehicleController gPlayerController = {
    .getSteering = player_controller_steering,
    .getPedal = player_controller_pedal,
};

static float empty_controller_callback(VehicleController *controller) {
    return 0.0f;
}

VehicleController gEmptyController = {
    .getSteering = empty_controller_callback,
    .getPedal = empty_controller_callback,
};
