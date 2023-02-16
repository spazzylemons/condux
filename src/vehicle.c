#include "input.h"
#include "linalg.h"
#include "macros.h"
#include "spline.h"
#include "vehicle.h"

#include <math.h>
#include <stdio.h>

#define GRAVITY_APPROACH_SPEED 5.0f

#define GRAVITY_STRENGTH 15.0f

#define FRICTION_COEFFICIENT 0.1f

#define GRAVITY_FALLOFF_POINT 2.0f

#define STEERING_APPROACH_SPEED 6.0f

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

static void handle_steering(Vehicle *vehicle) {
    float steering = vehicle->controller->getSteering(vehicle->controller);
    // local rotate for steering
    Quat steering_rotation, new_rotation;
    quat_angle_axis(steering_rotation, gVecYAxis, -steering * vehicle->type->handling * TICK_DELTA);
    quat_mul(new_rotation, steering_rotation, vehicle->rotation);
    quat_copy(vehicle->rotation, new_rotation);
    // smooth steering visual
    vehicle->steering = (steering * STEERING_APPROACH_SPEED * TICK_DELTA) + (vehicle->steering / (1.0f + (STEERING_APPROACH_SPEED * TICK_DELTA)));
}

void vehicle_up_vector(const Vehicle *vehicle, Vec v) {
    Mtx rot;
    quat_to_mtx(rot, vehicle->rotation);
    mtx_mul_vec(rot, v, gVecYAxis);
}

void vehicle_forward_vector(const Vehicle *vehicle, Vec v) {
    Mtx rot;
    quat_to_mtx(rot, vehicle->rotation);
    mtx_mul_vec(rot, v, gVecZAxis);
}

void vehicle_velocity_without_gravity(const Vehicle *vehicle, Vec v) {
    // get up vector
    Vec up;
    vehicle_up_vector(vehicle, up);
    // get amount of gravity being experienced
    Vec gravity;
    vec_scaled_copy(gravity, up, vec_dot(vehicle->velocity, up));
    // remove gravity
    vec_copy(v, vehicle->velocity);
    vec_sub(v, gravity);
}

static void get_forward(const Vehicle *vehicle, Vec v) {
    Mtx rot;
    quat_to_mtx(rot, vehicle->rotation);
    mtx_mul_vec(rot, v, gVecZAxis);
}

static void apply_velocity(Vehicle *vehicle) {
    vec_scaled_add(vehicle->position, vehicle->velocity, TICK_DELTA);
}

static void apply_gravity(Vehicle *vehicle, const Vec up) {
    vec_scaled_add(vehicle->velocity, up, -GRAVITY_STRENGTH * TICK_DELTA);
}

static void approach_aligned_without_gravity(Vehicle *vehicle, const Vec forward, Vec withoutGravity) {
    Vec forwardAligned, backwardAligned;

    vec_scaled_copy(forwardAligned, forward, sqrtf(vec_magnitude_sq(withoutGravity)));
    vec_scaled_copy(backwardAligned, forwardAligned, -1.0);

    vec_normalize(forwardAligned);
    vec_normalize(backwardAligned);

    float length = sqrtf(vec_magnitude_sq(withoutGravity));
    vec_normalize(withoutGravity);

    Vec v;
    if (vec_dot(forwardAligned, withoutGravity) > vec_dot(backwardAligned, withoutGravity)) {
        vec_approach(v, vehicle->type->antiDrift, withoutGravity, forwardAligned);
    } else {
        vec_approach(v, vehicle->type->antiDrift, withoutGravity, backwardAligned);
    }
    vec_normalize(v);
    vec_scaled_copy(withoutGravity, v, length);
}

static void collide_with_spline(Vehicle *vehicle, const Spline *spline, const Octree *tree, const Vec without_gravity, Vec new_gravity_vector) {
    Vec collision_up, tmp;
    float height;
    vec_copy(new_gravity_vector, gVecYAxis);
    if (spline_get_up_height(spline, tree, vehicle->position, collision_up, &height)) {
        if (height <= 0.0f && height > -COLLISION_DEPTH) {
            vec_copy(vehicle->velocity, without_gravity);
            // collided with floor, apply some friction
            // printf("A %f\n", sqrtf(vec_magnitude_sq(vehicle->velocity)));
            vec_copy(tmp, vehicle->velocity);
            vec_normalize(tmp);
            vec_scale(tmp, -(FRICTION_COEFFICIENT * GRAVITY_STRENGTH * TICK_DELTA));
            vec_add(tmp, vehicle->velocity);
            if (vec_dot(tmp, vehicle->velocity) <= 0.0f) {
                // if dot product is flipped, direction flipped, so set velocity to zero
                vec_copy(vehicle->velocity, gVecZero);
            } else {
                // otherwise, use friction
                vec_copy(vehicle->velocity, tmp);
            }
            // printf("B %f\n", sqrtf(vec_magnitude_sq(vehicle->velocity)));
            vec_scaled_add(vehicle->position, collision_up, -height);
        }
        if (height > -COLLISION_DEPTH && height < MAX_GRAVITY_HEIGHT) {
            height -= GRAVITY_FALLOFF_POINT;
            height /= MAX_GRAVITY_HEIGHT - GRAVITY_FALLOFF_POINT;
            if (height < 0.0f) height = 0.0f;
            else if (height > 1.0f) height = 1.0f;
            vec_scale(new_gravity_vector, height);
            vec_scaled_add(new_gravity_vector, collision_up, 1.0f - height);
            // TODO is this necessary?
            vec_normalize(new_gravity_vector);
        }
    }
}

static void apply_acceleration_no_speed_cap(Vehicle *vehicle, Vec without_gravity, const Vec forward) {
    float pedal = vehicle->controller->getPedal(vehicle->controller);
    vec_scaled_add(without_gravity, forward, pedal * vehicle->type->acceleration * TICK_DELTA);
}

void vehicle_update(Vehicle *vehicle, const Spline *spline, const Octree *tree) {
    handle_steering(vehicle);
    // apply gravity
    Vec up;
    vehicle_up_vector(vehicle, up);
    apply_gravity(vehicle, up);
    // get amount of gravity being experienced
    Vec gravity;
    vec_scaled_copy(gravity, up, vec_dot(vehicle->velocity, up));
    // remove gravity for acceleration calculation
    Vec without_gravity;
    vec_copy(without_gravity, vehicle->velocity);
    vec_sub(without_gravity, gravity);
    Vec forward, tmp;
    get_forward(vehicle, forward);
    apply_acceleration_no_speed_cap(vehicle, without_gravity, forward);
    vec_copy(tmp, without_gravity);
    vec_normalize(tmp);
    // speed cap
    if (vec_magnitude_sq(without_gravity) > vehicle->type->speed * vehicle->type->speed) {
        vec_scaled_copy(without_gravity, tmp, vehicle->type->speed);
    }
    approach_aligned_without_gravity(vehicle, forward, without_gravity);
    // combine parts of velocity
    vec_copy(vehicle->velocity, without_gravity);
    vec_add(vehicle->velocity, gravity);
    // slide with physics
    apply_velocity(vehicle);
    Vec new_gravity_vector;
    collide_with_spline(vehicle, spline, tree, without_gravity, new_gravity_vector);
    Vec approach_up;
    vec_approach(approach_up, GRAVITY_APPROACH_SPEED, up, new_gravity_vector);
    vec_cross(tmp, up, approach_up);
    vec_normalize(tmp);
	// only perform alignment if our up vector is not parallel to gravity
	// if it is, we're either perfectly aligned or completely flipped
	// TODO handle the latter case
    if (vec_magnitude_sq(tmp) != 0.0f) {
        Quat rotation_quat, new_rotation;
        quat_angle_axis(rotation_quat, tmp, vec_signed_angle_to(up, approach_up, tmp));
        quat_mul(new_rotation, vehicle->rotation, rotation_quat);
        quat_copy(vehicle->rotation, new_rotation);
    }
    // normalize rotation
    quat_normalize(vehicle->rotation);
}
