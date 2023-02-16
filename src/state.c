#include "collision.h"
#include "linalg.h"
#include "render.h"
#include "spline.h"
#include "state.h"
#include "vehicle.h"

#include <math.h>

#define CAMERA_FOLLOW_DISTANCE 2.5f
#define CAMERA_APPROACH_SPEED 2.0f
#define CAMERA_UP_DISTANCE 0.325f
#define STEERING_FACTOR 0.25f

Spline gSpline;
static Octree octree;

static uint8_t numVehicles;
static Vehicle vehicles[MAX_VEHICLES];
static Vec prevPos[MAX_VEHICLES];
static Quat prevRot[MAX_VEHICLES];
static float prevSteering[MAX_VEHICLES];

static Vec cameraPos, prevCameraPos;
static Vec cameraTarget, prevCameraTarget;
static Vec cameraUp, prevCameraUp;

// (0, sin(PI / -8), cos(PI / -8))
static const Vec targetAngle = { 0.0f, -0.3826834323650898f, 0.9238795325112867f };

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
    vec_copy(v, normal);
    vec_scaled_add(v, up, -vec_dot(normal, up));
    vec_normalize(v);
}

static void camera_target_pos(Vec dst, const Vehicle *vehicle) {
    Vec offset;
    Mtx offsetMtx;
    vec_copy(dst, vehicle->position);
    quat_to_mtx(offsetMtx, vehicle->rotation);
    mtx_mul_vec(offsetMtx, offset, targetAngle);
    vec_scaled_add(dst, offset, -CAMERA_FOLLOW_DISTANCE);
}

static void camera_look_at_target(const Vehicle *vehicle) {
    vec_copy(cameraTarget, vehicle->position);
    vehicle_up_vector(vehicle, cameraUp);
    vec_scaled_add(cameraTarget, cameraUp, CAMERA_UP_DISTANCE);
}

static void update_camera_pos(const Vehicle *vehicle) {
    vec_copy(prevCameraPos, cameraPos);
    vec_copy(prevCameraTarget, cameraTarget);
    vec_copy(prevCameraUp, cameraUp);

    // set ourselves to the proper distance
    Vec tmp, translationGlobal, delta, up, target;
    vec_scaled_copy(tmp, gVecZAxis, sqrtf(vec_distance_sq(vehicle->position, cameraPos)) - CAMERA_FOLLOW_DISTANCE);
    Mtx cameraMtx;
    vec_copy(delta, cameraPos);
    vec_sub(delta, vehicle->position);
    vehicle_up_vector(vehicle, up);
    mtx_look_at(cameraMtx, delta, up);
    mtx_mul_vec(cameraMtx, translationGlobal, tmp);
    vec_add(cameraPos, translationGlobal);
    // approach target location
    camera_target_pos(target, vehicle);
    vec_approach(tmp, CAMERA_APPROACH_SPEED, cameraPos, target);
    vec_copy(cameraPos, tmp);
    camera_look_at_target(vehicle);
}

void game_state_teleport_camera(uint8_t cameraFocus) {
    if (cameraFocus < numVehicles) {
        const Vehicle *vehicle = &vehicles[cameraFocus];
        camera_target_pos(cameraPos, vehicle);
        camera_look_at_target(vehicle);
    }
}

void game_state_update(uint8_t cameraFocus) {
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
        prevSteering[i] = vehicles[i].steering;
        vehicle_update(&vehicles[i], &gSpline, &octree);

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
    // now, run camera logic
    if (cameraFocus < numVehicles) {
        const Vehicle *vehicle = &vehicles[cameraFocus];
        update_camera_pos(vehicle);
    }
}

static void interpolate_vehicle(uint8_t i, float interpolation, Vec pos, Mtx rot) {
    vec_scaled_copy(pos, vehicles[i].position, interpolation);
    vec_scaled_add(pos, prevPos[i], 1.0f - interpolation);

    Quat prevVehicleRot, curVehicleRot, prevRoll, curRoll, prevQuat, curQuat;
    quat_copy(prevVehicleRot, prevRot[i]);
    quat_copy(curVehicleRot, vehicles[i].rotation);
    quat_angle_axis(prevRoll, gVecZAxis, prevSteering[i] * STEERING_FACTOR);
    quat_angle_axis(curRoll, gVecZAxis, vehicles[i].steering * STEERING_FACTOR);
    quat_mul(prevQuat, prevRoll, prevVehicleRot);
    quat_mul(curQuat, curRoll, curVehicleRot);

    Quat rot_quat;
    quat_slerp(rot_quat, prevQuat, curQuat, interpolation);
    quat_to_mtx(rot, rot_quat);
}

void game_state_render(uint8_t uiFocus, float interpolation) {
    Vec interpCameraPos, interpCameraTarget, interpCameraUp;
    vec_scaled_copy(interpCameraPos, cameraPos, interpolation);
    vec_scaled_add(interpCameraPos, prevCameraPos, 1.0f - interpolation);
    vec_scaled_copy(interpCameraTarget, cameraTarget, interpolation);
    vec_scaled_add(interpCameraTarget, prevCameraTarget, 1.0f - interpolation);
    vec_scaled_copy(interpCameraUp, cameraUp, interpolation);
    vec_scaled_add(interpCameraUp, prevCameraUp, 1.0f - interpolation);

    set_camera(interpCameraPos, interpCameraTarget, interpCameraUp);

    for (uint8_t i = 0; i < numVehicles; i++) {
        Vec pos;
        Mtx rot;
        interpolate_vehicle(i, interpolation, pos, rot);
        mesh_render(&vehicles[i].type->mesh, pos, rot);
    }

    render_spline();

    if (uiFocus < numVehicles) {
        const Vehicle *vehicle = &vehicles[uiFocus];
        Vec v, forward;
        vehicle_velocity_without_gravity(vehicle, v);
        vehicle_forward_vector(vehicle, forward);
        float speed = sqrtf(vec_magnitude_sq(v));
        // if moving opposite where we're facing, flip reported speed
        if (vec_dot(v, forward) < 0.0f) speed *= -1.0f;
        render_text(6.0f, 18.0f, 2.0f, "SPEED %.2f", speed);
    }
}
