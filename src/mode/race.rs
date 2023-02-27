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

use crate::{
    assets::Asset,
    linalg::{Length, Mtx, Quat, Vector},
    mode::Mode,
    octree::Octree,
    platform::{Buttons, Controls},
    render::context::{RenderContext, RenderContext3d},
    spline::Spline,
    util::{Approach, Interpolate},
    vehicle::{garage::Garage, AIController, Controller, PlayerController, Vehicle},
};

use super::{pause::PauseMode, GlobalGameData};

const CAMERA_FOLLOW_DISTANCE: f32 = 2.5;
const CAMERA_APPROACH_SPEED: f32 = 2.0;
const CAMERA_UP_DISTANCE: f32 = 0.325;
const STEERING_FACTOR: f32 = 0.25;

struct VehicleState {
    vehicle: Vehicle,
    prev_pos: Vector,
    prev_rot: Quat,
    prev_steering: f32,
}

impl VehicleState {
    pub fn new(pos: Vector, model_id: u16, controller: Box<dyn Controller>) -> Self {
        let vehicle = Vehicle::new(pos, model_id, controller);

        let prev_pos = vehicle.position;
        let prev_rot = vehicle.rotation;
        let prev_steering = vehicle.steering;

        Self {
            vehicle,
            prev_pos,
            prev_rot,
            prev_steering,
        }
    }

    fn interpolate(&self, interp: f32) -> (Vector, Mtx) {
        let pos = self.prev_pos.interpolate(self.vehicle.position, interp);

        let prev_vehicle_rot = self.prev_rot;
        let cur_vehicle_rot = self.vehicle.rotation;
        let prev_roll = Quat::axis_angle(&Vector::Z_AXIS, self.prev_steering * STEERING_FACTOR);
        let cur_roll = Quat::axis_angle(&Vector::Z_AXIS, self.vehicle.steering * STEERING_FACTOR);
        let prev_quat = prev_roll * prev_vehicle_rot;
        let cur_quat = cur_roll * cur_vehicle_rot;

        let rot_quat = Quat::slerp(prev_quat, cur_quat, interp);

        (pos, rot_quat.into())
    }

    fn render(&self, interp: f32, garage: &Garage, context: &mut RenderContext3d) {
        let (pos, rot) = self.interpolate(interp);
        self.vehicle.render(garage, context, pos, rot);
    }

    /// Returns true if the vehicle was respawned.
    fn try_respawn(&mut self) -> bool {
        if let Some(timer) = self.vehicle.respawn_timer {
            // decrement timer
            let timer = timer - 1;
            // if we hit zero, respawn
            if timer == 0 {
                // reset vehicle position, including interpolation
                self.vehicle.position = self.vehicle.respawn_point;
                self.prev_pos = self.vehicle.respawn_point;
                // reset the rest of the vehicle state and interpolation
                self.prev_rot = Quat::IDENT;
                self.vehicle.rotation = Quat::IDENT;
                self.vehicle.steering = 0.0;
                self.prev_steering = 0.0;
                self.vehicle.velocity = Vector::ZERO;
                // clear respawn timer
                self.vehicle.respawn_timer = None;
                return true;
            } else {
                // otherwise, just update timer
                self.vehicle.respawn_timer = Some(timer);
            }
        }
        false
    }

    fn update(
        &mut self,
        garage: &Garage,
        spline: &Spline,
        octree: &Octree,
        controls: &Controls,
        walls: bool,
    ) {
        self.prev_pos = self.vehicle.position;
        self.prev_rot = self.vehicle.rotation;
        self.prev_steering = self.vehicle.steering;
        self.vehicle
            .update(garage, spline, octree, &controls, walls);
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

    fn update(&mut self, vehicle: &Vehicle) {
        // only do this if the vehicle won't respawn
        // this lets us see the vehicle fall
        if vehicle.respawn_timer.is_none() {
            // set ourselves to the proper distance
            let tmp = Vector::Z_AXIS * (vehicle.position.dist(self.pos) - CAMERA_FOLLOW_DISTANCE);
            let delta = self.pos - vehicle.position;
            let up = vehicle.up_vector();
            let camera_mtx = Mtx::looking_at(delta, up);
            let translation_global = camera_mtx * tmp;
            self.pos += translation_global;
            // approach target location
            let target = self.target_pos(vehicle);
            self.pos.approach_mut(CAMERA_APPROACH_SPEED, target);
        }
        self.look_at(vehicle);
    }

    #[must_use]
    fn target_pos(&self, vehicle: &Vehicle) -> Vector {
        let offset = Mtx::from(vehicle.rotation) * TARGET_ANGLE;
        vehicle.position - offset * CAMERA_FOLLOW_DISTANCE
    }

    fn teleport(&mut self, vehicle: &Vehicle) {
        self.pos = self.target_pos(vehicle);
        self.look_at(vehicle);
    }
}

pub struct RaceMode {
    vehicle_states: Vec<VehicleState>,

    pub spline: Spline,
    pub octree: Octree,

    camera: CameraState,
    prev_camera: CameraState,

    pub camera_focus: usize,

    pub walls: bool,
}

// (0, sin(PI / -8), cos(PI / -8))
// trigonometry is not const fn in Rust
const TARGET_ANGLE: Vector = Vector::new(0.0, -0.382_683_43, 0.923_879_5);

fn adjust_normal(up: Vector, normal: Vector) -> Vector {
    (normal - up * normal.dot(&up)).normalized()
}

impl RaceMode {
    #[must_use]
    pub fn new(spline: Spline, octree: Octree, camera_focus: usize) -> Self {
        Self {
            vehicle_states: vec![],
            spline,
            octree,
            camera: CameraState::default(),
            prev_camera: CameraState::default(),
            camera_focus,
            walls: true,
        }
    }

    #[must_use]
    pub fn initialized(garage: &Garage) -> Self {
        let spline = Spline::load(&mut Asset::load("course_test1.bin").unwrap()).unwrap();
        let octree = Octree::new(&spline);
        let mut mode = Self::new(spline, octree, 0);
        // spawn player
        let spawn = mode.spline.get_baked(0.0);
        let model = garage.get_id("default").unwrap();
        mode.spawn(spawn, model, Box::new(PlayerController::default()));
        // spawn some other vehicles
        let spawn = mode.spline.get_baked(5.0);
        mode.spawn(spawn, model, Box::new(AIController::default()));
        let spawn = mode.spline.get_baked(10.0);
        mode.spawn(spawn, model, Box::new(AIController::default()));
        let spawn = mode.spline.get_baked(15.0);
        mode.spawn(spawn, model, Box::new(AIController::default()));
        // set camera behind player
        mode.teleport_camera();
        mode
    }

    pub fn spawn(&mut self, pos: Vector, model_id: u16, controller: Box<dyn Controller>) {
        self.vehicle_states
            .push(VehicleState::new(pos, model_id, controller));
    }

    fn update_camera_pos(&mut self) {
        if self.camera_focus >= self.vehicle_states.len() {
            return;
        }

        self.prev_camera = self.camera.clone();
        self.camera
            .update(&self.vehicle_states[self.camera_focus].vehicle);
    }

    pub fn teleport_camera(&mut self) {
        if self.camera_focus >= self.vehicle_states.len() {
            return;
        }

        self.camera
            .teleport(&self.vehicle_states[self.camera_focus].vehicle);
        // update prev camera as well
        self.prev_camera = self.camera.clone();
    }
}

impl Mode for RaceMode {
    fn tick(mut self: Box<Self>, data: &GlobalGameData) -> Box<dyn Mode> {
        if data.pressed.contains(Buttons::PAUSE) {
            // return paused
            return Box::new(PauseMode::new(self));
        }
        // check all vehicles that may need to respawn
        let mut need_to_reset_camera = false;
        for (i, state) in self.vehicle_states.iter_mut().enumerate() {
            if state.try_respawn() && i == self.camera_focus {
                need_to_reset_camera = true;
            }
        }
        if need_to_reset_camera {
            self.teleport_camera();
        }

        // run physics on all vehicles
        let mut total_translations = vec![];
        let mut original_velocity = vec![];
        let mut momentum_neighbors = vec![];

        self.octree.reset_vehicles();

        for (i, state) in self.vehicle_states.iter_mut().enumerate() {
            state.update(
                &data.garage,
                &self.spline,
                &self.octree,
                &data.controls,
                self.walls,
            );

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
        self.update_camera_pos();
        self
    }

    fn render(&self, interp: f32, data: &GlobalGameData, context: &mut dyn RenderContext) {
        let interp_camera_pos = self.prev_camera.pos.interpolate(self.camera.pos, interp);
        let interp_camera_target = self
            .prev_camera
            .target
            .interpolate(self.camera.target, interp);
        let interp_camera_up = self.prev_camera.up.interpolate(self.camera.up, interp);

        let mut context_3d = RenderContext3d::new(
            context,
            interp_camera_pos,
            interp_camera_target,
            interp_camera_up,
        );

        for state in &self.vehicle_states {
            state.render(interp, &data.garage, &mut context_3d);
        }

        self.spline.render(&mut context_3d, self.walls);

        if self.camera_focus < self.vehicle_states.len() {
            let vehicle = &self.vehicle_states[self.camera_focus].vehicle;
            let speed = vehicle.signed_speed();
            data.font
                .write(context, 6.0, 18.0, 2.0, &format!("SPEED {:.2}", speed));
        }
    }
}
