use crate::{bindings, linalg::{Quat, Mtx, Vector, Length}};

const GRAVITY_APPROACH_SPEED: f32 = 5.0;
const GRAVITY_STRENGTH: f32 = 15.0;
const FRICTION_COEFFICIENT: f32 = 0.1;
const GRAVITY_FALLOFF_POINT: f32 = 2.0;
const STEERING_APPROACH_SPEED: f32 = 6.0;

pub struct Vehicle {
    pub position: Vector,
    pub rotation: Quat,
    pub velocity: Vector,
    pub steering: f32,
    pub ty: &'static bindings::VehicleType,
    pub controller: &'static mut bindings::VehicleController,
}

impl Vehicle {
    pub fn up_vector(&self) -> Vector {
        Mtx::from(self.rotation) * Vector::Y_AXIS
    }

    pub fn forward_vector(&self) -> Vector {
        Mtx::from(self.rotation) * Vector::Z_AXIS
    }

    pub fn gravity(&self) -> Vector {
        let up = self.up_vector();
        // amount of gravity being experienced
        up * self.velocity.dot(&up)
    }

    pub fn velocity_without_gravity(&self) -> Vector {
        self.velocity - self.gravity()
    }

    fn handle_steering(&mut self) {
        let steering = unsafe { (&*self.controller).getSteering.unwrap()(self.controller) };
        // local rotate for steering
        let steering_rotation = Quat::axis_angle(&Vector::Y_AXIS, -steering * self.ty.handling * (bindings::TICK_DELTA as f32));
        self.rotation = steering_rotation * self.rotation;
        // smooth steering visual
        self.steering = steering * STEERING_APPROACH_SPEED * (bindings::TICK_DELTA as f32)
            + self.steering / (1.0 + (STEERING_APPROACH_SPEED * (bindings::TICK_DELTA as f32)));
    }

    fn apply_acceleration_no_speed_cap(&mut self, without: &mut Vector, forward: Vector) {
        let pedal = unsafe { (&*self.controller).getPedal.unwrap()(self.controller) };
        *without += forward * (pedal * self.ty.acceleration * (bindings::TICK_DELTA as f32));
    }

    fn approach_aligned_without_gravity(&mut self, forward: Vector, without: &mut Vector) {
        let f = forward.normalized();
        let b = -f;

        let length = without.mag();
        *without = without.normalized();

        let anti_drift = self.ty.antiDrift;
        let v = if f.dot(without) > b.dot(without) {
            without.approach(anti_drift, &f)
        } else {
            without.approach(anti_drift, &b)
        }.normalized();
        *without = v * length;
    }

    fn update_physics(&mut self) {
        self.handle_steering();
        // precalculate up and forward vectors
        let up = self.up_vector();
        let forward = self.forward_vector();
        // apply gravity
        self.velocity -= up * (GRAVITY_STRENGTH * (bindings::TICK_DELTA as f32));
        // get amount of gravity being experienced
        let gravity = self.gravity();
        let mut without = self.velocity - gravity;
        self.apply_acceleration_no_speed_cap(&mut without, forward);
        // speed cap
        let speed = self.ty.speed;
        if without.mag_sq() > speed * speed {
            without = without.normalized() * speed;
        }
        self.approach_aligned_without_gravity(forward, &mut without);
        // combine parts of velocity
        self.velocity = gravity + without;
        // slide with physics
        self.position += self.velocity * (bindings::TICK_DELTA as f32);
    }

    fn collide_with_spline(&mut self, spline: &bindings::Spline, tree: &bindings::Octree) -> Vector {
        let mut up_pass = [0.0f32, 0.0f32, 0.0f32];
        let mut position_write = [0.0f32, 0.0f32, 0.0f32];
        let mut height = 0.0f32;
        let mut new_gravity_vector = Vector::Y_AXIS;
        self.position.write(&mut position_write as *mut f32);
        if unsafe { bindings::spline_get_up_height(spline as *const bindings::Spline, tree as *const bindings::Octree, &mut position_write as *mut f32, &mut up_pass as *mut f32, &mut height as *mut f32) } {
            if height <= 0.0 && height > -bindings::COLLISION_DEPTH as f32 {
                self.velocity = self.velocity_without_gravity();
                // collided with floor, apply some friction
                let with_friction = self.velocity - self.velocity.normalized() * FRICTION_COEFFICIENT * GRAVITY_STRENGTH * (bindings::TICK_DELTA as f32);
                if with_friction.dot(&self.velocity) <= 0.0 {
                    // if dot product is flipped, direction flipped, so set velocity to zero
                    self.velocity = Vector::ZERO;
                } else {
                    // otherwise, use friction
                    self.velocity = with_friction;
                }
                self.position -= Vector::from(up_pass) * height;
            }
            if height > -bindings::COLLISION_DEPTH as f32 && height < bindings::MAX_GRAVITY_HEIGHT as f32 {
                height -= GRAVITY_FALLOFF_POINT;
                height /= bindings::MAX_GRAVITY_HEIGHT as f32 - GRAVITY_FALLOFF_POINT;
                height = height.clamp(0.0, 1.0);
                new_gravity_vector *= height;
                new_gravity_vector += Vector::from(up_pass) * (1.0 - height);
                // TODO is this necessary?
                new_gravity_vector = new_gravity_vector.normalized();
            }
        }
        new_gravity_vector
    }

    fn update_collision(&mut self, spline: &bindings::Spline, tree: &bindings::Octree) {
        let new_gravity_vector = self.collide_with_spline(spline, tree);
        let up = self.up_vector();
        let approach_up = up.approach(GRAVITY_APPROACH_SPEED, &new_gravity_vector);
        let alignment = up.cross(&new_gravity_vector).normalized();
        // only perform alignment if our up vector is not parallel to gravity
        // if it is, we're either perfectly aligned or completely flipped
        // TODO handle the latter case
        if alignment.mag_sq() > 0.0 {
            let rotation_quat = Quat::axis_angle(&alignment, up.signed_angle(&approach_up, &alignment));
            self.rotation *= rotation_quat;
        }
    }

    pub fn update(&mut self, spline: &bindings::Spline, tree: &bindings::Octree) {
        self.update_physics();
        self.update_collision(spline, tree);
        // normalize rotation
        self.rotation = self.rotation.normalized();
    }
}
