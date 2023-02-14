#include "collision.h"
#include "linalg.h"
#include "render.h"
#include "spline.h"
#include "state.h"
#include "vehicle.h"

#include <math.h>

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

static void adjust_normal(Vec v, const Vec up, const Vec normal) {
    Vec tmp;
    vec_copy(v, normal);
    vec_copy(tmp, up);
    vec_scale(tmp, vec_dot(normal, up));
    vec_sub(v, tmp);
    vec_normalize(v);
}

void game_state_update(float delta) {
    static Vec totalTranslations[MAX_VEHICLES];
    static Vec originalVelocity[MAX_VEHICLES];
    static uint8_t momentumNeighbors[MAX_VEHICLES][MAX_VEHICLES];
    static uint8_t numMomentumNeighbors[MAX_VEHICLES];
    static uint8_t octreeCollisions[MAX_VEHICLES];

    // first, run physics on all vehicles
    octree_reset_vehicles(&octree);
    for (uint8_t i = 0; i < numVehicles; i++) {
        vec_copy(prevPos[i], vehicles[i].position);
        quat_copy(prevRot[i], vehicles[i].rotation);
        vehicle_update(&vehicles[i], &gSpline, &octree, delta);

        vec_copy(totalTranslations[i], gVecZero);
        vec_copy(originalVelocity[i], vehicles[i].velocity);
        vec_copy(vehicles[i].velocity, gVecZero);
        momentumNeighbors[i][0] = i;
        numMomentumNeighbors[i] = 1;

        octree_add_vehicle(&octree, vehicles[i].position, i);
    }
    // next, find any collisions between vehicles
    for (uint8_t i = 0; i < numVehicles; i++) {
        uint8_t n = octree_find_collisions(&octree, vehicles[i].position, octreeCollisions);
        for (uint8_t x = 0; x < n; x++) {
            uint8_t j = octreeCollisions[x];
            if (j <= i) continue;
            // measure collision vector
            Vec normal;
            vec_copy(normal, vehicles[i].position);
            vec_sub(normal, vehicles[j].position);
            // measure distance
            float length = sqrtf(vec_magnitude_sq(normal));
            float depth = VEHICLE_RADIUS + VEHICLE_RADIUS - length;
            if (depth <= 0.0f) continue;
            vec_scale(normal, 1.0f / length);
            Vec upI, upJ, normalI, normalJ;
            vehicle_up_vector(&vehicles[i], upI);
            vehicle_up_vector(&vehicles[j], upJ);
            adjust_normal(normalI, upI, normal);;
            adjust_normal(normalJ, upJ, normal);
            depth /= 2.0f;
            vec_scale(normalI, depth);
            vec_scale(normalJ, depth);
            vec_add(totalTranslations[i], normalI);
            vec_sub(totalTranslations[j], normalJ);
            momentumNeighbors[i][numMomentumNeighbors[i]++] = j;
            momentumNeighbors[j][numMomentumNeighbors[j]++] = i;
        }
    }
    // attempt to resolve collisions and transfer momentum
    for (uint8_t i = 0; i < numVehicles; i++) {
        vec_add(vehicles[i].position, totalTranslations[i]);
        vec_scale(originalVelocity[i], 1.0f / numMomentumNeighbors[i]);
        for (uint8_t j = 0; j < numMomentumNeighbors[i]; j++) {
            uint8_t neighbor = momentumNeighbors[i][j];
            vec_add(vehicles[neighbor].velocity, originalVelocity[i]);
        }
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
