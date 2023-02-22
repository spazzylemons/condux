//! Condux - an antigravity racing game
//! Copyright (C) 2023 spazzylemons
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::cell::Cell;

use crate::{
    linalg::{Length, Mtx, Quat, Vector},
    octree::Octree,
    platform::{Buttons, Controls},
    render::Mesh,
    spline::{CollisionState, Spline},
    timing::TICK_DELTA,
};

const GRAVITY_APPROACH_SPEED: f32 = 5.0;
const GRAVITY_STRENGTH: f32 = 6.0;
const FRICTION_COEFFICIENT: f32 = 0.1;
const GRAVITY_FALLOFF_POINT: f32 = 2.0;
const STEERING_APPROACH_SPEED: f32 = 6.0;
const GRAVITY_TERMINAL_VELOCITY: f32 = 8.0;

pub struct Model {
    pub speed: f32,
    pub acceleration: f32,
    pub handling: f32,
    pub anti_drift: f32,
    pub mesh: Mesh,
}

pub struct Vehicle<'a> {
    pub position: Vector,
    pub rotation: Quat,
    pub velocity: Vector,
    pub steering: f32,
    pub ty: &'a Model,
    pub controller: Box<dyn Controller + 'a>,
    /// When containing a value, the vehicle cannot be controlled, does not collide
    /// with the track, and will be respawned when the timer reaches zero.
    pub respawn_timer: Option<u8>,
    /// The location to respawn to.
    pub respawn_point: Vector,
}

impl<'a> Vehicle<'a> {
    pub const RADIUS: f32 = 0.3;

    pub const MAX_GRAVITY_HEIGHT: f32 = 8.0;

    pub const COLLISION_DEPTH: f32 = 0.25;

    /// If height above track is at or below this, the vehicle snaps to the ground.
    /// This prevents the vehicle from bumping around.
    const GRAVITY_SNAP: f32 = 0.05;

    /// This is the number of frames that respawn_timer is set to.
    const RESPAWN_TIMER_INIT: u8 = 60;

    #[must_use]
    pub fn up_vector(&self) -> Vector {
        Mtx::from(self.rotation) * Vector::Y_AXIS
    }

    #[must_use]
    pub fn forward_vector(&self) -> Vector {
        Mtx::from(self.rotation) * Vector::Z_AXIS
    }

    #[must_use]
    pub fn gravity(&self) -> Vector {
        let up = self.up_vector();
        // amount of gravity being experienced
        up * self.velocity.dot(&up)
    }

    #[must_use]
    pub fn velocity_without_gravity(&self) -> Vector {
        self.velocity - self.gravity()
    }

    fn handle_steering(&mut self) {
        // only if we're not going to respawn
        if self.respawn_timer.is_some() {
            return;
        }
        let steering = self.controller.steering();
        // local rotate for steering
        let steering_rotation =
            Quat::axis_angle(&Vector::Y_AXIS, -steering * self.ty.handling * TICK_DELTA);
        self.rotation = steering_rotation * self.rotation;
        // smooth steering visual
        self.steering = steering * STEERING_APPROACH_SPEED * TICK_DELTA
            + self.steering / (1.0 + (STEERING_APPROACH_SPEED * TICK_DELTA));
    }

    fn apply_acceleration_no_speed_cap(&mut self, without: &mut Vector, forward: Vector) {
        // only if we're not going to respawn
        if self.respawn_timer.is_some() {
            return;
        }
        let pedal = self.controller.pedal();
        *without += forward * (pedal * self.ty.acceleration * TICK_DELTA);
    }

    fn approach_aligned_without_gravity(&mut self, forward: Vector, without: &mut Vector) {
        let f = forward.normalized();
        let b = -f;

        let length = without.mag();
        *without = without.normalized();

        let anti_drift = self.ty.anti_drift;
        let v = if f.dot(without) > b.dot(without) {
            without.approach(anti_drift, &f)
        } else {
            without.approach(anti_drift, &b)
        }
        .normalized();
        *without = v * length;
    }

    fn update_physics(&mut self) {
        self.handle_steering();
        // precalculate up and forward vectors
        let up = self.up_vector();
        let forward = self.forward_vector();
        // apply gravity
        self.velocity -= up * (GRAVITY_STRENGTH * TICK_DELTA);
        // get amount of gravity being experienced
        let mut gravity = self.gravity();
        let mut without = self.velocity - gravity;
        self.apply_acceleration_no_speed_cap(&mut without, forward);
        // speed cap
        let speed = self.ty.speed;
        if without.mag_sq() > speed * speed {
            without = without.normalized() * speed;
        }
        self.approach_aligned_without_gravity(forward, &mut without);
        // limit gravity
        if gravity.mag_sq() > GRAVITY_TERMINAL_VELOCITY * GRAVITY_TERMINAL_VELOCITY {
            gravity = gravity.normalized() * GRAVITY_TERMINAL_VELOCITY;
        }
        // combine parts of velocity
        self.velocity = gravity + without;
        // slide with physics
        self.position += self.velocity * TICK_DELTA;
    }

    fn collide_with_spline(&mut self, spline: &Spline, octree: &Octree) -> Vector {
        let mut new_gravity_vector = Vector::Y_AXIS;
        // only do this if the respawn timer is none
        if self.respawn_timer.is_none() {
            // check collision
            match spline.get_collision(octree, self.position) {
                CollisionState::Gravity { up, height } => {
                    if height <= Self::GRAVITY_SNAP {
                        self.velocity = self.velocity_without_gravity();
                        // collided with floor, apply some friction
                        let with_friction = self.velocity
                            - self.velocity.normalized()
                                * FRICTION_COEFFICIENT
                                * GRAVITY_STRENGTH
                                * TICK_DELTA;
                        if with_friction.dot(&self.velocity) <= 0.0 {
                            // if dot product is flipped, direction flipped, so set velocity to zero
                            self.velocity = Vector::ZERO;
                        } else {
                            // otherwise, use friction
                            self.velocity = with_friction;
                        }
                        self.position -= up * height;
                    }
                    let height = height - GRAVITY_FALLOFF_POINT;
                    let height = height / Self::MAX_GRAVITY_HEIGHT - GRAVITY_FALLOFF_POINT;
                    let height = height.clamp(0.0, 1.0);
                    new_gravity_vector *= height;
                    new_gravity_vector += up * (1.0 - height);
                    // TODO is this necessary?
                    new_gravity_vector = new_gravity_vector.normalized();
                }

                CollisionState::InBounds => {
                    // nothing to do, we're in bounds
                }

                CollisionState::OutOfBounds => {
                    // we're out of bounds, we'll signal that we need a respawn
                    self.respawn_timer = Some(Self::RESPAWN_TIMER_INIT);
                }
            }
        }
        new_gravity_vector
    }

    fn update_collision(&mut self, spline: &Spline, octree: &Octree) {
        let new_gravity_vector = self.collide_with_spline(spline, octree);
        let up = self.up_vector();
        let approach_up = up.approach(GRAVITY_APPROACH_SPEED, &new_gravity_vector);
        let alignment = up.cross(&new_gravity_vector).normalized();
        // only perform alignment if our up vector is not parallel to gravity
        // if it is, we're either perfectly aligned or completely flipped
        // TODO handle the latter case
        if alignment.mag_sq() > 0.0 {
            let rotation_quat =
                Quat::axis_angle(&alignment, up.signed_angle(&approach_up, &alignment));
            self.rotation *= rotation_quat;
        }
    }

    pub fn update(&mut self, spline: &Spline, octree: &Octree) {
        self.update_physics();
        self.update_collision(spline, octree);
        // normalize rotation
        self.rotation = self.rotation.normalized();
    }
}

pub trait Controller {
    fn pedal(&self) -> f32;
    fn steering(&self) -> f32;
}

pub struct PlayerController<'a> {
    pub controls: &'a Cell<Controls>,
}

impl<'a> Controller for PlayerController<'a> {
    fn pedal(&self) -> f32 {
        let controls = self.controls.get();
        if controls.buttons.contains(Buttons::BACK) {
            -1.0
        } else if controls.buttons.contains(Buttons::OK) {
            1.0
        } else {
            0.0
        }
    }

    fn steering(&self) -> f32 {
        self.controls.get().steering
    }
}

pub struct EmptyController;

impl Controller for EmptyController {
    fn pedal(&self) -> f32 {
        0.0
    }

    fn steering(&self) -> f32 {
        0.0
    }
}
