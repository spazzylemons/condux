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

use std::sync::Arc;

use crate::{
    linalg::{Length, Mtx, Quat, Vector},
    octree::Octree,
    platform::{Buttons, Controls},
    render::{graph::RenderGraph3d, Mesh},
    spline::Spline,
    timing::TICK_DELTA,
    util::Approach,
};

use self::garage::Garage;

pub mod garage;

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
    pub mesh: Arc<Mesh>,
}

pub struct ControllerGuidance<'a> {
    /// The horizontal value.
    pub horizontal: f32,
    /// The vehicle position.
    pub position: Vector,
    /// The signed magnitude of the vehicle's speed.
    pub speed: f32,
    /// The vehicle up vector.
    pub up: Vector,
    /// The vehicle forward vector.
    pub forward: Vector,
    /// A point somewhat ahead to target.
    pub target: Vector,
    /// The state of the controls.
    pub controls: &'a Controls,
}

pub struct Vehicle {
    pub position: Vector,
    pub rotation: Quat,
    pub velocity: Vector,
    pub steering: f32,
    model_id: u16,
    controller: Box<dyn Controller>,
    /// When containing a value, the vehicle cannot be controlled, does not collide
    /// with the track, and will be respawned when the timer reaches zero.
    pub respawn_timer: Option<u8>,
    /// The location to respawn to.
    pub respawn_point: Vector,
    /// The last seen spline horizontal.
    last_horizontal: f32,
    /// The last seen spline offset.
    last_offset: f32,
}

impl Vehicle {
    pub const RADIUS: f32 = 0.3;

    pub const MAX_GRAVITY_HEIGHT: f32 = 8.0;

    pub const COLLISION_DEPTH: f32 = 0.25;

    /// If height above track is at or below this, the vehicle snaps to the ground.
    /// This prevents the vehicle from bumping around.
    const GRAVITY_SNAP: f32 = 0.05;

    /// This is the number of frames that respawn_timer is set to.
    const RESPAWN_TIMER_INIT: u8 = 60;

    /// How far ahead we look on the spline for guiding AI.
    const GUIDANCE_LOOKAHEAD: f32 = 4.0;

    pub fn new(pos: Vector, model_id: u16, controller: Box<dyn Controller>) -> Self {
        Self {
            position: pos,
            rotation: Quat::IDENT,
            velocity: Vector::ZERO,
            model_id,
            controller,
            steering: 0.0,
            respawn_timer: None,
            respawn_point: pos,
            last_horizontal: 0.0,
            last_offset: 0.0,
        }
    }

    fn guidance<'a>(&self, spline: &Spline, controls: &'a Controls) -> ControllerGuidance<'a> {
        ControllerGuidance {
            horizontal: self.last_horizontal,
            position: self.position,
            speed: self.signed_speed(),
            up: self.up_vector(),
            forward: self.forward_vector(),
            target: spline.get_baked(self.last_offset + Self::GUIDANCE_LOOKAHEAD),
            controls,
        }
    }

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

    fn handle_steering(&mut self, model: &Model) {
        // only if we're not going to respawn
        if self.respawn_timer.is_some() {
            return;
        }
        let steering = self.controller.steering();
        // local rotate for steering
        let steering_rotation =
            Quat::axis_angle(&Vector::Y_AXIS, -steering * model.handling * TICK_DELTA);
        self.rotation = steering_rotation * self.rotation;
        // smooth steering visual
        self.steering = steering * STEERING_APPROACH_SPEED * TICK_DELTA
            + self.steering / (1.0 + (STEERING_APPROACH_SPEED * TICK_DELTA));
    }

    fn apply_acceleration_no_speed_cap(
        &mut self,
        model: &Model,
        without: &mut Vector,
        forward: Vector,
    ) {
        // only if we're not going to respawn
        if self.respawn_timer.is_some() {
            return;
        }
        let pedal = match self.controller.pedal() {
            Pedal::Accel => 1.0,
            Pedal::Brake => -1.0,
            Pedal::Neutral => 0.0,
        };
        *without += forward * (pedal * model.acceleration * TICK_DELTA);
    }

    fn approach_aligned_without_gravity(
        &mut self,
        model: &Model,
        forward: Vector,
        without: &mut Vector,
    ) {
        let f = forward.normalized();
        let b = -f;

        let length = without.mag();
        *without = without.normalized();

        let anti_drift = model.anti_drift;
        let v = if f.dot(without) > b.dot(without) {
            without.approach(anti_drift, f)
        } else {
            without.approach(anti_drift, b)
        }
        .normalized();
        *without = v * length;
    }

    fn update_physics(&mut self, model: &Model) {
        self.handle_steering(model);
        // precalculate up and forward vectors
        let up = self.up_vector();
        let forward = self.forward_vector();
        // apply gravity
        self.velocity -= up * (GRAVITY_STRENGTH * TICK_DELTA);
        // get amount of gravity being experienced
        let mut gravity = self.gravity();
        let mut without = self.velocity - gravity;
        self.apply_acceleration_no_speed_cap(model, &mut without, forward);
        // speed cap
        let speed = model.speed;
        if without.mag_sq() > speed * speed {
            without = without.normalized() * speed;
        }
        self.approach_aligned_without_gravity(model, forward, &mut without);
        // limit gravity
        if gravity.mag_sq() > GRAVITY_TERMINAL_VELOCITY * GRAVITY_TERMINAL_VELOCITY {
            gravity = gravity.normalized() * GRAVITY_TERMINAL_VELOCITY;
        }
        // combine parts of velocity
        self.velocity = gravity + without;
        // slide with physics
        self.position += self.velocity * TICK_DELTA;
    }

    fn collide_with_spline(&mut self, spline: &Spline, octree: &Octree, walls: bool) -> Vector {
        let mut new_gravity_vector = Vector::Y_AXIS;
        // only do this if the respawn timer is none
        if self.respawn_timer.is_none() {
            // check collision
            if let Some(state) = spline.get_collision(octree, self.position) {
                let height = state.height;
                let horizontal = state.horizontal;
                if horizontal.abs() <= Spline::TRACK_RADIUS {
                    if walls {
                        // account for vehicle radius with wall collision
                        let adjusted_radius = Spline::TRACK_RADIUS - Self::RADIUS;
                        if horizontal.abs() > Spline::TRACK_RADIUS - Self::RADIUS
                            && height <= Spline::WALL_HEIGHT
                        {
                            let right = state.right;
                            self.position -=
                                right * (horizontal.abs() - adjusted_radius).copysign(horizontal);
                            // remove velocity that's parallel to the wall
                            self.velocity = self.velocity_without_gravity();
                            self.velocity -= right * self.velocity.dot(&right);
                        }
                    }
                    let up = state.up;
                    // update guidance info
                    self.last_offset = state.offset;
                    self.last_horizontal = horizontal / -Spline::TRACK_RADIUS;
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
            } else {
                // we're out of bounds, we'll signal that we need a respawn
                self.respawn_timer = Some(Self::RESPAWN_TIMER_INIT);
            }
        }
        new_gravity_vector
    }

    fn update_collision(&mut self, spline: &Spline, octree: &Octree, walls: bool) {
        let new_gravity_vector = self.collide_with_spline(spline, octree, walls);
        let up = self.up_vector();
        let approach_up = up.approach(GRAVITY_APPROACH_SPEED, new_gravity_vector);
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

    fn update_controller(&mut self, spline: &Spline, controls: &Controls) {
        self.controller.update(&self.guidance(spline, controls));
    }

    pub fn update(
        &mut self,
        garage: &Garage,
        spline: &Spline,
        octree: &Octree,
        controls: &Controls,
        walls: bool,
    ) {
        if let Some(model) = garage.get_model(self.model_id) {
            self.update_controller(spline, controls);
            self.update_physics(model);
            self.update_collision(spline, octree, walls);
            // normalize rotation
            self.rotation = self.rotation.normalized();
        }
    }

    #[must_use]
    pub fn signed_speed(&self) -> f32 {
        let v = self.velocity_without_gravity();
        let f = self.forward_vector();
        v.mag().copysign(v.dot(&f))
    }

    pub fn render(&self, garage: &Garage, graph: &mut RenderGraph3d, pos: Vector, rot: Mtx) {
        if let Some(model) = garage.get_model(self.model_id) {
            graph.mesh(pos, rot, model.mesh.clone());
        }
    }
}

#[derive(Clone, Copy)]
pub enum Pedal {
    Accel,
    Brake,
    Neutral,
}

impl Default for Pedal {
    fn default() -> Self {
        Self::Neutral
    }
}

pub trait Controller: Send {
    fn pedal(&self) -> Pedal;

    fn steering(&self) -> f32;

    fn update(&mut self, _guidance: &ControllerGuidance) {
        // default implementation if no update logic needed
    }
}

#[derive(Default)]
pub struct PlayerController {
    next_pedal: Pedal,
    next_steering: f32,
}

impl Controller for PlayerController {
    fn pedal(&self) -> Pedal {
        self.next_pedal
    }

    fn steering(&self) -> f32 {
        self.next_steering
    }

    fn update(&mut self, guidance: &ControllerGuidance) {
        self.next_pedal = if guidance.controls.buttons.contains(Buttons::BACK) {
            Pedal::Brake
        } else if guidance.controls.buttons.contains(Buttons::OK) {
            Pedal::Accel
        } else {
            Pedal::Neutral
        };
        self.next_steering = guidance.controls.steering;
    }
}

pub struct EmptyController;

impl Controller for EmptyController {
    fn pedal(&self) -> Pedal {
        Pedal::Neutral
    }

    fn steering(&self) -> f32 {
        0.0
    }
}

#[derive(Default)]
pub struct AIController {
    next_pedal: Pedal,
    next_steering: f32,
}

impl AIController {
    const BRAKE_RADIUS: f32 = 0.2;
    const STEER_STRENGTH: f32 = 5.0;
    const CAREFUL_ANGLE_RADS: f32 = 0.65;
    const MIN_SPEED: f32 = 7.0;
    const STEER_INTERP_STRENGTH: f32 = 10.0;
}

impl Controller for AIController {
    fn pedal(&self) -> Pedal {
        self.next_pedal
    }

    fn steering(&self) -> f32 {
        self.next_steering
    }

    fn update(&mut self, guidance: &ControllerGuidance) {
        // find vector that we want to be facing
        let target_direction = (guidance.target - guidance.position).normalized();
        // find angle between that and where we're actually facing
        let angle = guidance
            .forward
            .signed_angle(&target_direction, &guidance.up);
        // use this angle to decide steering
        let new_steering = (-angle * Self::STEER_STRENGTH).clamp(-1.0, 1.0);
        // interpolate for smoother steering
        self.next_steering
            .approach_mut(Self::STEER_INTERP_STRENGTH, new_steering);
        if guidance.speed < Self::MIN_SPEED {
            // if moving rather slowly, accelerate so we don't go backwards
            self.next_pedal = Pedal::Accel;
        } else if angle.abs() >= Self::CAREFUL_ANGLE_RADS {
            // if making a tight turn, brake
            self.next_pedal = Pedal::Brake;
        } else if guidance.horizontal.abs() >= Self::BRAKE_RADIUS {
            // if far from center, brake
            self.next_pedal = Pedal::Brake;
        } else {
            // otherwise, accelerate
            self.next_pedal = Pedal::Accel;
        }
    }
}
