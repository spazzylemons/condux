use std::{sync::Mutex, mem::zeroed};

use crate::{bindings, linalg::{Vector, Quat, Mtx, Length}};

const CAMERA_FOLLOW_DISTANCE: f32 = 2.5;
const CAMERA_APPROACH_SPEED: f32 = 2.0;
const CAMERA_UP_DISTANCE: f32 = 0.325;
const STEERING_FACTOR: f32 = 0.25;

struct VehicleState {
    vehicle: bindings::Vehicle,
    prev_pos: Vector,
    prev_rot: Quat,
    prev_steering: f32,
}

impl VehicleState {
    fn interpolate(&self, interp: f32) -> (Vector, Mtx) {
        let pos = (self.vehicle.position() * interp) + (self.prev_pos * (1.0 - interp));

        let prev_vehicle_rot = self.prev_rot;
        let cur_vehicle_rot = self.vehicle.rotation();
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
    fn look_at(&mut self, vehicle: &bindings::Vehicle) {
        self.target = vehicle.position();
        self.up = vehicle.up_vector();
        self.target += self.up * CAMERA_UP_DISTANCE;
    }
}

pub struct GameState {
    vehicle_states: Vec<VehicleState>,

    octree: bindings::Octree,

    camera: CameraState,
    prev_camera: CameraState,
}

// (0, sin(PI / -8), cos(PI / -8))
// trigonometry is not const fn in Rust
const TARGET_ANGLE: Vector = Vector::new(0.0, -0.3826834323650898, 0.9238795325112867);

fn adjust_normal(up: Vector, normal: Vector) -> Vector {
    (normal - up * normal.dot(&up)).normalized()
}

impl GameState {
    pub fn new(octree: bindings::Octree) -> Self {
        Self {
            vehicle_states: vec![],
            octree,
            camera: CameraState::default(),
            prev_camera: CameraState::default(),
        }
    }

    pub fn spawn(&mut self, pos: Vector, ty: *const bindings::VehicleType, controller: *mut bindings::VehicleController) -> bool {
        if self.vehicle_states.len() == bindings::MAX_VEHICLES as usize {
            return false;
        }
    
        let mut vehicle = bindings::Vehicle {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            type_: ty,
            controller,
            steering: 0.0,
        };
        vehicle.set_position(pos);
        vehicle.set_rotation(Quat::IDENT);
    
        let prev_pos = vehicle.position();
        let prev_rot = vehicle.rotation();
        let prev_steering = vehicle.steering;
    
        let vehicle_state = VehicleState { vehicle, prev_pos, prev_rot, prev_steering };
        self.vehicle_states.push(vehicle_state);

        true
    }

    fn target_pos(&self, vehicle: &bindings::Vehicle) -> Vector {
        let offset = Mtx::from(vehicle.rotation()) * TARGET_ANGLE;
        vehicle.position() - offset * CAMERA_FOLLOW_DISTANCE
    }

    fn update_camera_pos(&mut self, focus: usize) {
        if focus >= self.vehicle_states.len() {
            return;
        }

        self.prev_camera = self.camera.clone();
        // set ourselves to the proper distance
        let tmp = Vector::Z_AXIS * (self.vehicle_states[focus].vehicle.position().dist(self.camera.pos) - CAMERA_FOLLOW_DISTANCE);
        let delta = self.camera.pos - self.vehicle_states[focus].vehicle.position();
        let up = self.vehicle_states[focus].vehicle.up_vector();
        let camera_mtx = Mtx::looking_at(delta, up);
        let translation_global = camera_mtx * tmp;
        self.camera.pos += translation_global;
        // approach target location
        let target = self.target_pos(&self.vehicle_states[focus].vehicle);
        self.camera.pos = self.camera.pos.approach(CAMERA_APPROACH_SPEED, &target);
        self.camera.look_at(&self.vehicle_states[focus].vehicle);
    }

    pub fn teleport_camera(&mut self, focus: usize) {
        if focus >= self.vehicle_states.len() {
            return;
        }

        let vehicle = &self.vehicle_states[focus].vehicle;
        self.camera.pos = self.target_pos(vehicle);
        self.camera.look_at(vehicle);
    }

    pub fn update(&mut self, focus: usize) {
        // first, run physics on all vehicles
        let mut total_translations = vec![];
        let mut original_velocity = vec![];
        let mut momentum_neighbors = vec![];

        unsafe {
            bindings::octree_reset_vehicles(&mut self.octree as *mut bindings::Octree);
        }

        for (i, state) in self.vehicle_states.iter_mut().enumerate() {
            state.prev_pos = state.vehicle.position();
            state.prev_rot = state.vehicle.rotation();
            state.prev_steering = state.vehicle.steering;
            state.vehicle.update(unsafe { &mut bindings::gSpline }, &mut self.octree);

            total_translations.push(Vector::ZERO);
            original_velocity.push(state.vehicle.velocity());
            state.vehicle.set_velocity(Vector::ZERO);
            momentum_neighbors.push(vec![i]);

            unsafe {
                bindings::octree_add_vehicle(&mut self.octree as *mut bindings::Octree, &mut state.vehicle.position as *mut f32, i as i32);
            }
        }

        // next, find any collisions between vehicles
        for i in 0..self.vehicle_states.len() {
            let mut octree_collisions = [0u8; bindings::MAX_VEHICLES as usize];
            let n = unsafe {
                bindings::octree_find_collisions(&mut self.octree as *mut bindings::Octree, &mut self.vehicle_states[i].vehicle.position as *mut f32, &mut octree_collisions as *mut u8)
            };
            for j in &octree_collisions[0..n as usize] {
                let j = *j as usize;
                if j <= i {
                    continue;
                }
                // measure collision vector
                let normal = self.vehicle_states[i].vehicle.position() - self.vehicle_states[j].vehicle.position();
                // measure distance
                let length = normal.mag();
                let depth = (bindings::VEHICLE_RADIUS + bindings::VEHICLE_RADIUS) as f32 - length;
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
            let new_position = self.vehicle_states[i].vehicle.position() + total_translations[i];
            self.vehicle_states[i].vehicle.set_position(new_position);
            let velocity = original_velocity[i] / (momentum_neighbors[i].len() as f32);
            for &j in &momentum_neighbors[i] {
                let new_velocity = self.vehicle_states[j].vehicle.velocity() + velocity;
                self.vehicle_states[j].vehicle.set_velocity(new_velocity);
            }
        }
        // now, run camera logic
        self.update_camera_pos(focus);
    }

    pub fn render(&mut self, ui_focus: usize, interp: f32) {
        let interp_camera_pos = (self.camera.pos * interp) + (self.prev_camera.pos * (1.0 - interp));
        let interp_camera_target = (self.camera.target * interp) + (self.prev_camera.target * (1.0 - interp));
        let interp_camera_up = (self.camera.up * interp) + (self.prev_camera.up * (1.0 - interp));
        
        let mut interp_camera_pos_write = [0.0f32; 3];
        interp_camera_pos.write(&mut interp_camera_pos_write as *mut f32);
        let mut interp_camera_target_write = [0.0f32; 3];
        interp_camera_target.write(&mut interp_camera_target_write as *mut f32);
        let mut interp_camera_up_write = [0.0f32; 3];
        interp_camera_up.write(&mut interp_camera_up_write as *mut f32);
        
        unsafe {
            bindings::set_camera(
                &mut interp_camera_pos_write as *mut f32,
                &mut interp_camera_target_write as *mut f32,
                &mut interp_camera_up_write as *mut f32
            );
        }

        for state in &self.vehicle_states {
            let (pos, rot) = state.interpolate(interp);
            let mut pos_write = [0.0f32; 3];
            let mut rot_write = [[0.0f32; 3]; 3];
            pos.write(&mut pos_write as *mut f32);
            rot.write(&mut rot_write as *mut [f32; 3]);
            unsafe {
                bindings::mesh_render(&(*state.vehicle.type_).mesh as *const bindings::Mesh, &mut pos_write as *mut f32, &mut rot_write as *mut [f32; 3]);
            }
        }

        unsafe {
            bindings::render_spline();
        }

        if ui_focus < self.vehicle_states.len() {
            let vehicle = &self.vehicle_states[ui_focus].vehicle;
            let v = vehicle.velocity_without_gravity();
            let forward = vehicle.forward_vector();
            let mut speed = v.mag();
            // if moving opposite where we're facing, flip reported speed
            if v.dot(&forward) < 0.0 {
                speed *= -1.0;
            }
            unsafe {
                let ptr = std::ffi::CString::new("SPEED %.2f").unwrap();
                bindings::render_text(6.0, 18.0, 2.0, ptr.as_ptr(), speed as std::ffi::c_double);
            }
        }
    }
}

unsafe impl Send for GameState{}

static STATE: Mutex<Option<GameState>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn game_state_init(spline_asset: *mut bindings::Asset) -> bool {
    unsafe {
        if !bindings::spline_load(&mut bindings::gSpline as *mut bindings::Spline, spline_asset) {
            return false;
        }

        let mut octree = zeroed::<bindings::Octree>();
        bindings::octree_init(&mut octree as *mut bindings::Octree, &bindings::gSpline as *const bindings::Spline);

        let state = GameState::new(octree);

        *STATE.lock().unwrap() = Some(state);

        true
    }
}

#[no_mangle]
pub extern "C" fn game_state_spawn(pos: *const f32, ty: *const bindings::VehicleType, controller: *mut bindings::VehicleController) -> bool {
    STATE.lock().unwrap().as_mut().unwrap().spawn(Vector::from(pos), ty, controller)
}

#[no_mangle]
pub extern "C" fn game_state_teleport_camera(focus: u8) {
    STATE.lock().unwrap().as_mut().unwrap().teleport_camera(focus as usize);
}

#[no_mangle]
pub extern "C" fn game_state_update(focus: u8) {
    STATE.lock().unwrap().as_mut().unwrap().update(focus as usize);
}

#[no_mangle]
pub extern "C" fn game_state_render(focus: u8, interp: f32) {
    STATE.lock().unwrap().as_mut().unwrap().render(focus as usize, interp);
}
