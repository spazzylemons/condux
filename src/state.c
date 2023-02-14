#include "collision.h"
#include "linalg.h"
#include "render.h"
#include "spline.h"
#include "state.h"
#include "vehicle.h"

Spline gSpline;
static Octree octree;

static uint8_t numVehicles;
static Vehicle vehicles[MAX_VEHICLES];
static Vec prevPos[MAX_VEHICLES];
static Quat prevRot[MAX_VEHICLES];

bool game_state_init(Asset *spline_asset) {
    if (!spline_load(&gSpline, spline_asset)) {
        return false;
    }

    octree_init(&octree, &gSpline);

    numVehicles = 0;

    return true;
}

bool game_state_spawn(const Vec pos, const VehicleType *type, VehicleController *controller) {
    if (numVehicles == MAX_VEHICLES) return false;

    Vehicle *vehicle = &vehicles[numVehicles];
    vec_copy(vehicle->position, pos);
    quat_copy(vehicle->rotation, gQuatIdentity);
    vec_copy(vehicle->velocity, gVecZero);
    vehicle->type = type;
    vehicle->controller = controller;
    vec_copy(prevPos[numVehicles], pos);
    vec_copy(prevRot[numVehicles], gQuatIdentity);
    ++numVehicles;

    return true;
}

void game_state_update(float delta) {
    for (uint8_t i = 0; i < numVehicles; i++) {
        vec_copy(prevPos[i], vehicles[i].position);
        quat_copy(prevRot[i], vehicles[i].rotation);
        vehicle_update(&vehicles[i], &gSpline, &octree, delta);
    }
}

static void interpolate_vehicle(uint8_t i, float interpolation, Vec pos, Mtx rot) {
    Vec tmp;
    vec_copy(tmp, vehicles[i].position);
    vec_scale(tmp, interpolation);
    vec_copy(pos, prevPos[i]);
    vec_scale(pos, 1.0f - interpolation);
    vec_add(pos, tmp);

    Quat rot_quat;
    quat_slerp(rot_quat, prevRot[i], vehicles[i].rotation, interpolation);
    quat_to_mtx(rot, rot_quat);
}

void game_state_render(uint8_t cameraFocus, float interpolation) {
    if (cameraFocus < numVehicles) {
        Vec pos;
        Mtx rot;
        interpolate_vehicle(cameraFocus, interpolation, pos, rot);

        Vec cam, forward, up;

        mtx_mul_vec(rot, forward, gVecZAxis);
        mtx_mul_vec(rot, up, gVecYAxis);

        vec_scale(forward, 3.0f);
        vec_copy(cam, pos);
        vec_sub(cam, forward);
        vec_add(cam, up);
        set_camera(cam, pos, up);
    }

    for (uint8_t i = 0; i < numVehicles; i++) {
        Vec pos;
        Mtx rot;
        interpolate_vehicle(i, interpolation, pos, rot);
        mesh_render(&vehicles[i].type->mesh, pos, rot);
    }
}
