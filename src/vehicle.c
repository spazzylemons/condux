#include "linalg.h"
#include "macros.h"
#include "spline.h"
#include "vehicle.h"

#include <math.h>

#define GRAVITY_APPROACH_SPEED 5.0f

#define GRAVITY_STRENGTH 15.0f

#define FRICTION_COEFFICIENT 0.1f

#define GRAVITY_FALLOFF_POINT 2.0f

static void handle_steering(Vehicle *vehicle, float delta) {
    float steering = vehicle->controller->getSteering(vehicle->controller);
    // local rotate for steering
    Quat steering_rotation, new_rotation;
    quat_angle_axis(steering_rotation, gVecYAxis, -steering * vehicle->type->handling * delta);
    quat_mul(new_rotation, steering_rotation, vehicle->rotation);
    quat_copy(vehicle->rotation, new_rotation);
}

static void get_up(const Vehicle *vehicle, Vec v) {
    Mtx rot;
    quat_to_mtx(rot, vehicle->rotation);
    mtx_mul_vec(rot, v, gVecYAxis);
}

static void get_forward(const Vehicle *vehicle, Vec v) {
    Mtx rot;
    quat_to_mtx(rot, vehicle->rotation);
    mtx_mul_vec(rot, v, gVecZAxis);
}

static void apply_approach(float delta, float strength, Vec dst, const Vec from, const Vec to) {
    vec_copy(dst, to);
    vec_scale(dst, strength * delta);
    vec_add(dst, from);
    vec_scale(dst, 1.0f / (1.0f + (strength * delta)));
}

static void apply_velocity(Vehicle *vehicle, float delta) {
    Vec tmp;
    vec_copy(tmp, vehicle->velocity);
    vec_scale(tmp, delta);
    vec_add(vehicle->position, tmp);
}

static void apply_gravity(Vehicle *vehicle, float delta, const Vec up) {
    Vec tmp;
    vec_copy(tmp, up);
    vec_scale(tmp, GRAVITY_STRENGTH * delta);
    vec_sub(vehicle->velocity, tmp);
}

static void approach_aligned_without_gravity(Vehicle *vehicle, const Vec forward, const Vec normalized, Vec without_gravity, float delta) {
    Vec forward_aligned, backward_aligned;
    Vec forward_aligned_normalized, backward_aligned_normalized;

    vec_copy(forward_aligned, forward);
    vec_scale(forward_aligned, sqrtf(vec_magnitude_sq(without_gravity)));

    vec_set(backward_aligned, -forward_aligned[0], -forward_aligned[1], -forward_aligned[2]);

    vec_copy(forward_aligned_normalized, forward_aligned);
    vec_copy(backward_aligned_normalized, backward_aligned);

    vec_normalize(forward_aligned_normalized);
    vec_normalize(backward_aligned_normalized);

    if (vec_dot(forward_aligned_normalized, normalized) > vec_dot(backward_aligned_normalized, normalized)) {
        Vec v;
        apply_approach(delta, vehicle->type->antiDrift, v, without_gravity, forward_aligned);
        vec_copy(without_gravity, v);
    } else {
        Vec v;
        apply_approach(delta, vehicle->type->antiDrift, v, without_gravity, backward_aligned);
        vec_copy(without_gravity, v);
    }
}

static void collide_with_spline(Vehicle *vehicle, const Spline *spline, const Octree *tree, const Vec without_gravity, float delta, Vec new_gravity_vector) {
    Vec collision_up, tmp;
    float height;
    vec_copy(new_gravity_vector, gVecYAxis);
    if (spline_get_up_height(spline, tree, vehicle->position, collision_up, &height)) {
        if (height <= 0.0f && height > -COLLISION_DEPTH) {
            vec_copy(vehicle->velocity, without_gravity);
            // collided with floor, apply some friction
            vec_copy(tmp, vehicle->velocity);
            vec_normalize(tmp);
            vec_scale(tmp, -(FRICTION_COEFFICIENT * GRAVITY_STRENGTH * delta));
            vec_add(tmp, vehicle->velocity);
            if (vec_dot(tmp, vehicle->velocity) <= 0.0f) {
                // if dot product is flipped, direction flipped, so set velocity to zero
                vec_copy(vehicle->velocity, gVecZero);
            } else {
                // otherwise, use friction
                vec_copy(vehicle->velocity, tmp);
            }
            vec_copy(tmp, collision_up);
            vec_scale(tmp, height);
            vec_sub(vehicle->position, tmp);
        }
        if (height > -COLLISION_DEPTH && height < MAX_GRAVITY_HEIGHT) {
            height -= GRAVITY_FALLOFF_POINT;
            height /= MAX_GRAVITY_HEIGHT - GRAVITY_FALLOFF_POINT;
            if (height < 0.0f) height = 0.0f;
            else if (height > 1.0f) height = 1.0f;
            vec_copy(tmp, collision_up);
            vec_scale(tmp, 1.0f - height);
            vec_scale(new_gravity_vector, height);
            vec_add(new_gravity_vector, tmp);
            // TODO is this necessary?
            vec_normalize(new_gravity_vector);
        }
    }
}

static void apply_acceleration_no_speed_cap(Vehicle *vehicle, Vec without_gravity, const Vec forward, float delta) {
    float pedal = vehicle->controller->getPedal(vehicle->controller);
    Vec tmp;
    vec_copy(tmp, forward);
    vec_scale(tmp, pedal * vehicle->type->acceleration * delta);
    vec_add(without_gravity, tmp);
}

void vehicle_update(Vehicle *vehicle, const Spline *spline, const Octree *tree, float delta) {
    handle_steering(vehicle, delta);
    // apply gravity
    Vec up;
    get_up(vehicle, up);
    apply_gravity(vehicle, delta, up);
    // get amount of gravity being experienced
    Vec gravity;
    vec_copy(gravity, up);
    vec_scale(gravity, vec_dot(vehicle->velocity, up));
    // remove gravity for acceleration calculation
    Vec without_gravity;
    vec_copy(without_gravity, vehicle->velocity);
    vec_sub(without_gravity, gravity);
    Vec forward, tmp;
    get_forward(vehicle, forward);
    apply_acceleration_no_speed_cap(vehicle, without_gravity, forward, delta);
    vec_copy(tmp, without_gravity);
    vec_normalize(tmp);
    // speed cap
    if (vec_magnitude_sq(without_gravity) > vehicle->type->speed * vehicle->type->speed) {
        vec_copy(without_gravity, tmp);
        vec_scale(without_gravity, vehicle->type->speed);
    }
    approach_aligned_without_gravity(vehicle, forward, tmp, without_gravity, delta);
    // combine parts of velocity
    vec_copy(vehicle->velocity, without_gravity);
    vec_add(vehicle->velocity, gravity);
    // slide with physics
    apply_velocity(vehicle, delta);
    Vec new_gravity_vector;
    collide_with_spline(vehicle, spline, tree, without_gravity, delta, new_gravity_vector);
    Vec approach_up;
    apply_approach(delta, GRAVITY_APPROACH_SPEED, approach_up, up, new_gravity_vector);
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