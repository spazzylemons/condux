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

use std::fmt::Write;

use crate::{
    linalg::{Length, Mtx, Quat, Vector},
    octree::Octree,
    platform::Frame,
    render::Renderer,
    spline::Spline,
    vehicle::{Controller, Model, Vehicle},
};

const CAMERA_FOLLOW_DISTANCE: f32 = 2.5;
const CAMERA_APPROACH_SPEED: f32 = 2.0;
const CAMERA_UP_DISTANCE: f32 = 0.325;
const STEERING_FACTOR: f32 = 0.25;

struct VehicleState<'a> {
    vehicle: Vehicle<'a>,
    prev_pos: Vector,
    prev_rot: Quat,
    prev_steering: f32,
}

fn target_pos(vehicle: &Vehicle) -> Vector {
    let offset = Mtx::from(vehicle.rotation) * TARGET_ANGLE;
    vehicle.position - offset * CAMERA_FOLLOW_DISTANCE
}

impl<'a> VehicleState<'a> {
    fn interpolate(&self, interp: f32) -> (Vector, Mtx) {
        let pos = (self.vehicle.position * interp) + (self.prev_pos * (1.0 - interp));

        let prev_vehicle_rot = self.prev_rot;
        let cur_vehicle_rot = self.vehicle.rotation;
        let prev_roll = Quat::axis_angle(&Vector::Z_AXIS, self.prev_steering * STEERING_FACTOR);
        let cur_roll = Quat::axis_angle(&Vector::Z_AXIS, self.vehicle.steering * STEERING_FACTOR);
        let prev_quat = prev_roll * prev_vehicle_rot;
        let cur_quat = cur_roll * cur_vehicle_rot;

        let rot_quat = Quat::slerp(prev_quat, cur_quat, interp);

        (pos, rot_quat.into())
    }
}

#[derive(Clone, Default)]
struct CameraState {
    pos: Vector,
    target: Vector,
    up: Vector,
}

impl CameraState {
    fn look_at(&mut self, vehicle: &Vehicle) {
        self.target = vehicle.position;
        self.up = vehicle.up_vector();
        self.target += self.up * CAMERA_UP_DISTANCE;
    }
}

pub struct GameState<'a> {
    vehicle_states: Vec<VehicleState<'a>>,

    pub spline: Spline,
    pub octree: Octree,

    camera: CameraState,
    prev_camera: CameraState,

    pub renderer: Renderer,
}

// (0, sin(PI / -8), cos(PI / -8))
// trigonometry is not const fn in Rust
const TARGET_ANGLE: Vector = Vector::new(0.0, -0.382_683_43, 0.923_879_5);

fn adjust_normal(up: Vector, normal: Vector) -> Vector {
    (normal - up * normal.dot(&up)).normalized()
}

impl<'a> GameState<'a> {
    #[must_use]
    pub fn new(spline: Spline, octree: Octree, renderer: Renderer) -> Self {
        Self {
            vehicle_states: vec![],
            spline,
            octree,
            camera: CameraState::default(),
            prev_camera: CameraState::default(),
            renderer,
        }
    }

    pub fn spawn(&mut self, pos: Vector, ty: &'a Model, controller: Box<dyn Controller + 'a>) {
        let vehicle = Vehicle {
            position: pos,
            rotation: Quat::IDENT,
            velocity: Vector::ZERO,
            ty,
            controller,
            steering: 0.0,
        };

        let prev_pos = vehicle.position;
        let prev_rot = vehicle.rotation;
        let prev_steering = vehicle.steering;

        let vehicle_state = VehicleState {
            vehicle,
            prev_pos,
            prev_rot,
            prev_steering,
        };
        self.vehicle_states.push(vehicle_state);
    }

    fn update_camera_pos(&mut self, focus: usize) {
        if focus >= self.vehicle_states.len() {
            return;
        }

        self.prev_camera = self.camera.clone();
        // set ourselves to the proper distance
        let tmp = Vector::Z_AXIS
            * (self.vehicle_states[focus]
                .vehicle
                .position
                .dist(self.camera.pos)
                - CAMERA_FOLLOW_DISTANCE);
        let delta = self.camera.pos - self.vehicle_states[focus].vehicle.position;
        let up = self.vehicle_states[focus].vehicle.up_vector();
        let camera_mtx = Mtx::looking_at(delta, up);
        let translation_global = camera_mtx * tmp;
        self.camera.pos += translation_global;
        // approach target location
        let target = target_pos(&self.vehicle_states[focus].vehicle);
        self.camera.pos = self.camera.pos.approach(CAMERA_APPROACH_SPEED, &target);
        self.camera.look_at(&self.vehicle_states[focus].vehicle);
    }

    pub fn teleport_camera(&mut self, focus: usize) {
        if focus >= self.vehicle_states.len() {
            return;
        }

        let vehicle = &self.vehicle_states[focus].vehicle;
        self.camera.pos = target_pos(vehicle);
        self.camera.look_at(vehicle);
    }

    pub fn update(&mut self, focus: usize) {
        // first, run physics on all vehicles
        let mut total_translations = vec![];
        let mut original_velocity = vec![];
        let mut momentum_neighbors = vec![];

        self.octree.reset_vehicles();

        for (i, state) in self.vehicle_states.iter_mut().enumerate() {
            state.prev_pos = state.vehicle.position;
            state.prev_rot = state.vehicle.rotation;
            state.prev_steering = state.vehicle.steering;
            state.vehicle.update(&self.spline, &self.octree);

            total_translations.push(Vector::ZERO);
            original_velocity.push(state.vehicle.velocity);
            state.vehicle.velocity = Vector::ZERO;
            momentum_neighbors.push(vec![i]);

            self.octree.add_vehicle(state.vehicle.position, i);
        }

        // next, find any collisions between vehicles
        for i in 0..self.vehicle_states.len() {
            let collisions = self
                .octree
                .find_vehicle_collisions(&self.vehicle_states[i].vehicle.position);
            for j in collisions {
                if j <= i {
                    continue;
                }
                // measure collision vector
                let normal = self.vehicle_states[i].vehicle.position
                    - self.vehicle_states[j].vehicle.position;
                // measure distance
                let length = normal.mag();
                let depth = (Vehicle::RADIUS + Vehicle::RADIUS) - length;
                if depth <= 0.0 {
                    continue;
                }
                let normal = normal / length;
                let up_i = self.vehicle_states[i].vehicle.up_vector();
                let up_j = self.vehicle_states[j].vehicle.up_vector();
                let depth = depth * 0.5;
                total_translations[i] += adjust_normal(up_i, normal) * depth;
                total_translations[j] -= adjust_normal(up_j, normal) * depth;
                momentum_neighbors[i].push(j);
                momentum_neighbors[j].push(i);
            }
        }

        // attempt to resolve collisions and transfer momentum
        for i in 0..self.vehicle_states.len() {
            self.vehicle_states[i].vehicle.position += total_translations[i];
            let velocity = original_velocity[i] / (momentum_neighbors[i].len() as f32);
            for &j in &momentum_neighbors[i] {
                self.vehicle_states[j].vehicle.velocity += velocity;
            }
        }
        // now, run camera logic
        self.update_camera_pos(focus);
    }

    pub fn render(&mut self, ui_focus: usize, interp: f32, frame: &mut Frame) {
        let interp_camera_pos =
            (self.camera.pos * interp) + (self.prev_camera.pos * (1.0 - interp));
        let interp_camera_target =
            (self.camera.target * interp) + (self.prev_camera.target * (1.0 - interp));
        let interp_camera_up = (self.camera.up * interp) + (self.prev_camera.up * (1.0 - interp));

        self.renderer
            .set_camera(interp_camera_pos, interp_camera_target, interp_camera_up);

        for state in &self.vehicle_states {
            let (pos, rot) = state.interpolate(interp);
            state
                .vehicle
                .ty
                .mesh
                .render(&self.renderer, pos, rot, frame);
        }

        self.renderer.render_spline(frame);

        if ui_focus < self.vehicle_states.len() {
            let vehicle = &self.vehicle_states[ui_focus].vehicle;
            let v = vehicle.velocity_without_gravity();
            let forward = vehicle.forward_vector();
            let mut speed = v.mag();
            // if moving opposite where we're facing, flip reported speed
            if v.dot(&forward) < 0.0 {
                speed *= -1.0;
            }
            render_write!(self.renderer, 6.0, 18.0, 2.0, frame, "SPEED {:.2}", speed);
        }
    }
}
