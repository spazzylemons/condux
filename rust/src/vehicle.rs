use crate::{bindings, linalg::{Quat, Mtx, Vector, Length}};

const GRAVITY_APPROACH_SPEED: f32 = 5.0;
const GRAVITY_STRENGTH: f32 = 15.0;
const FRICTION_COEFFICIENT: f32 = 0.1;
const GRAVITY_FALLOFF_POINT: f32 = 2.0;
const STEERING_APPROACH_SPEED: f32 = 6.0;

impl bindings::Vehicle {
    pub fn position(&self) -> Vector {
        Vector::from(self.position)
    }

    pub fn set_position(&mut self, v: Vector) {
        self.position = v.into();
    }

    pub fn rotation(&self) -> Quat {
        Quat::from(self.rotation)
    }

    pub fn set_rotation(&mut self, q: Quat) {
        self.rotation = q.into();
    }

    pub fn velocity(&self) -> Vector {
        Vector::from(self.velocity)
    }

    pub fn set_velocity(&mut self, v: Vector) {
        self.velocity = v.into();
    }

    pub fn up_vector(&self) -> Vector {
        Mtx::from(self.rotation()) * Vector::Y_AXIS
    }

    pub fn forward_vector(&self) -> Vector {
        Mtx::from(self.rotation()) * Vector::Z_AXIS
    }

    pub fn gravity(&self) -> Vector {
        let up = self.up_vector();
        // amount of gravity being experienced
        up * self.velocity().dot(&up)
    }

    pub fn velocity_without_gravity(&self) -> Vector {
        self.velocity() - self.gravity()
    }

    fn handle_steering(&mut self) {
        let steering = unsafe { (&*self.controller).getSteering.unwrap()(self.controller) };
        // local rotate for steering
        let steering_rotation = Quat::axis_angle(&Vector::Y_AXIS, -steering * (unsafe { &*self.type_ }).handling * (bindings::TICK_DELTA as f32));
        let new_rotation = steering_rotation * self.rotation();
        self.set_rotation(new_rotation);
        // smooth steering visual
        self.steering = steering * STEERING_APPROACH_SPEED * (bindings::TICK_DELTA as f32)
            + self.steering / (1.0 + (STEERING_APPROACH_SPEED * (bindings::TICK_DELTA as f32)));
    }

    fn apply_acceleration_no_speed_cap(&mut self, without: &mut Vector, forward: Vector) {
        let pedal = unsafe { (&*self.controller).getPedal.unwrap()(self.controller) };
        *without += forward * (pedal * (unsafe { &*self.type_ }).acceleration * (bindings::TICK_DELTA as f32));
    }

    fn approach_aligned_without_gravity(&mut self, forward: Vector, without: &mut Vector) {
        let f = forward.normalized();
        let b = -f;

        let length = without.mag();
        *without = without.normalized();

        let anti_drift = unsafe { &*self.type_ }.antiDrift;
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
        self.set_velocity(self.velocity() - (up * (GRAVITY_STRENGTH * (bindings::TICK_DELTA as f32))));
        // get amount of gravity being experienced
        let gravity = self.gravity();
        let mut without = self.velocity() - gravity;
        self.apply_acceleration_no_speed_cap(&mut without, forward);
        // speed cap
        let speed = unsafe { &*self.type_ }.speed;
        if without.mag_sq() > speed * speed {
            without = without.normalized() * speed;
        }
        self.approach_aligned_without_gravity(forward, &mut without);
        // combine parts of velocity
        self.set_velocity(gravity + without);
        // slide with physics
        self.set_position(self.position() + (self.velocity() * (bindings::TICK_DELTA as f32)));
    }

    fn collide_with_spline(&mut self, spline: &bindings::Spline, tree: &bindings::Octree) -> Vector {
        let mut up_pass = [0.0f32, 0.0f32, 0.0f32];
        let mut height = 0.0f32;
        let mut new_gravity_vector = Vector::Y_AXIS;
        if unsafe { bindings::spline_get_up_height(spline as *const bindings::Spline, tree as *const bindings::Octree, &mut self.position as *mut f32, &mut up_pass as *mut f32, &mut height as *mut f32) } {
            if height <= 0.0 && height > -bindings::COLLISION_DEPTH as f32 {
                self.set_velocity(self.velocity_without_gravity());
                // collided with floor, apply some friction
                let with_friction = self.velocity() - self.velocity().normalized() * FRICTION_COEFFICIENT * GRAVITY_STRENGTH * (bindings::TICK_DELTA as f32);
                if with_friction.dot(&self.velocity()) <= 0.0 {
                    // if dot product is flipped, direction flipped, so set velocity to zero
                    self.set_velocity(Vector::ZERO);
                } else {
                    // otherwise, use friction
                    self.set_velocity(with_friction);
                }
                self.set_position(self.position() + Vector::from(up_pass) * -height);
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
            let new_rotation = self.rotation() * rotation_quat;
            self.set_rotation(new_rotation);
        }
    }

    pub fn update(&mut self, spline: &bindings::Spline, tree: &bindings::Octree) {
        self.update_physics();
        self.update_collision(spline, tree);
        // normalize rotation
        self.set_rotation(self.rotation().normalized());
    }
}

#[no_mangle]
pub extern "C" fn vehicle_up_vector(vehicle: *const bindings::Vehicle, v: *mut f32) {
    (unsafe { &*vehicle }).up_vector().write(v);
}

#[no_mangle]
pub extern "C" fn vehicle_forward_vector(vehicle: *const bindings::Vehicle, v: *mut f32) {
    (unsafe { &*vehicle }).forward_vector().write(v);
}

#[no_mangle]
pub extern "C" fn vehicle_velocity_without_gravity(vehicle: *const bindings::Vehicle, v: *mut f32) {
    (unsafe { &*vehicle }).velocity_without_gravity().write(v);
}

#[no_mangle]
pub extern "C" fn vehicle_update(vehicle: *mut bindings::Vehicle, spline: *const bindings::Spline, tree: *const bindings::Octree) {
    unsafe {
        (&mut *vehicle).update(&*spline, &*tree);
    }
}
