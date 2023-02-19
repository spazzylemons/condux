use std::{mem::zeroed, f32::consts::{TAU, PI}};

use crate::{bindings, linalg::{Vector, Length, Mtx}};

const BAKE_LENGTH_SQ: f32 = 1.0;

const FORWARD_VECTOR_SIZE: f32 = 0.125;

impl bindings::Spline {
    pub fn load(asset: &mut bindings::Asset) -> Option<Self> {
        // number of points
        let num_points = asset.read_byte()? as usize;
        if num_points < 3 || num_points > bindings::MAX_POINTS as usize {
            return None;
        }
        let mut points = [unsafe { zeroed::<bindings::SplinePoint>() }; bindings::MAX_POINTS as usize];
        // TODO div by zero checks?
        for i in 0..num_points {
            let point = asset.read_vector()?;
            point.write(&mut points[i].point as *mut f32);
            points[i].tilt = (f32::from(asset.read_byte()?) / 256.0) * TAU;
        }
        // fix tilts
        let mut total_tilt = points[0].tilt;
        for i in 0..num_points {
            let delta = (points[(i + 1) % num_points].tilt - points[i].tilt).rem_euclid(TAU);
            points[i].tilt = total_tilt;
            if delta <= PI {
                // move up
                total_tilt += delta;
            } else {
                // move down
                total_tilt += delta - TAU;
            }
        }
        // generate bezier control points
        for a in 0..num_points {
            let b = (a + 1) % num_points;
            let c = (a + 2) % num_points;
            let pa = Vector::from(points[a].point);
            let pb = Vector::from(points[b].point);
            let pc = Vector::from(points[c].point);
            let da = pa.dist(pb);
            let db = pb.dist(pc);
            // TODO handle potential divs by zero in this area
            let mid = da / (da + db);
            let fac_a = (mid - 1.0) / (2.0 * mid);
            let fac_b = 1.0 / (2.0 * mid * (1.0 - mid));
            let fac_c = mid / (2.0 * (mid - 1.0));
            ((pa * fac_a) + (pb * fac_b) + (pc * fac_c)).write(&mut points[a].control as *mut f32);
            points[a].controlMid = mid;
        }
        let mut spline = unsafe { zeroed::<Self>() };
        spline.numPoints = num_points as u8;
        spline.numBaked = 0;
        spline.totalTilt = total_tilt;
        spline.points = points;
        // bake - TODO make this be safe code
        spline.length = 0.0;
        // for each point, recursively find points to bake
        for i in 0..num_points {
            // bake at control point
            spline.add_baked(i as f32);
            // add length to tilt offsets
            spline.points[i].tiltOffset = spline.length;
            // bake in between
            spline.bake_recursive(i, 0.0, 1.0, 0);
        }
        // finish off length measurement
        let final_length = Vector::from(spline.baked[0].point).dist(Vector::from(spline.baked[spline.numBaked - 1].point));
        spline.length += final_length;
        Some(spline)
    }

    pub fn bezier(&self, index: usize, offset: f32) -> Vector {
        let other_index = (index + 2) % (self.numPoints as usize);
        let fac_a = (1.0 - offset) * (1.0 - offset);
        let fac_b = 2.0 * (1.0 - offset) * offset;
        let fac_c = offset * offset;
        let pa = Vector::from(self.points[index].point);
        let pb = Vector::from(self.points[index].control);
        let pc = Vector::from(self.points[other_index].point);
        pa * fac_a + pb * fac_b + pc * fac_c
    }

    pub fn interpolate(&self, offset: f32) -> Vector {
        let offset = offset.rem_euclid(self.numPoints.into());
        let index = offset.floor() as usize;
        let offset = offset - index as f32;
        let prev_index = (index + self.numPoints as usize - 1) % self.numPoints as usize;
        let prev_mid = self.points[prev_index].controlMid;
        let next_mid = self.points[index].controlMid;
        let a = self.bezier(prev_index, offset * (1.0 - prev_mid) + prev_mid);
        let b = self.bezier(index, offset * next_mid);
        a * (1.0 - offset) + b * offset
    }

    pub fn add_baked(&mut self, position: f32) {
        self.baked[self.numBaked].position = position;
        self.interpolate(position).write(&mut self.baked[self.numBaked].point as *mut f32);
        if self.numBaked != 0 {
            self.length += Vector::from(self.baked[self.numBaked].point).dist(Vector::from(self.baked[self.numBaked - 1].point));
        }
        self.baked[self.numBaked].offset = self.length;
        self.numBaked += 1;
    }

    pub fn bake_recursive(&mut self, index: usize, begin: f32, end: f32, depth: usize) {
        if depth >= bindings::MAX_BAKE_DEPTH as usize {
            return;
        }

        let v1 = self.interpolate(index as f32 + begin);
        let v2 = self.interpolate(index as f32 + end);

        if v1.dist_sq(v2) > BAKE_LENGTH_SQ {
            let mid = (begin + end) * 0.5;
            self.bake_recursive(index, begin, mid, depth + 1);
            self.add_baked(index as f32 + mid);
            self.bake_recursive(index, mid, end, depth + 1);
        }
    }

    fn convert_baked_offset(&self, baked_offset: f32) -> f32 {
        // binary search
        let mut start = 0;
        let mut end = self.numBaked;
        let mut current = (start + end) / 2;
        while start < current {
            if baked_offset <= self.baked[current].offset {
                end = current;
            } else {
                start = current;
            }
            current = (start + end) / 2;
        }
        // interpolate
        let next_index = (current + 1) % self.numBaked;
        let offset_begin = self.baked[current].offset;
        let mut offset_end = self.baked[next_index].offset;
        let position_begin = self.baked[current].position;
        let mut position_end = self.baked[next_index].position;
        if next_index == 0 {
            offset_end += self.length;
            position_end += self.numPoints as f32;
        }
        let interp = (baked_offset - offset_begin) / (offset_end - offset_begin);
        (1.0 - interp) * position_begin + interp * position_end
    }

    pub fn get_baked(&self, offset: f32) -> Vector {
        self.interpolate(self.convert_baked_offset(offset))
    }

    fn floor_div(&self, i: isize) -> (isize, &bindings::SplinePoint) {
        let n = self.numPoints as isize;
        let d = i / n;
        let d = if i < 0 && d * i != n {
            d - 1
        } else {
            d
        };
        (d, &self.points[(i - d * n) as usize])
    }

    fn get_tilt_offset(&self, i: isize) -> f32 {
        let (n, p) = self.floor_div(i);
        self.length * n as f32 + p.tiltOffset
    }

    fn get_tilt_radian(&self, i: isize) -> f32 {
        let (n, p) = self.floor_div(i);
        self.totalTilt * n as f32 + p.tilt
    }

    fn lagrange(&self, i: isize, x: f32) -> f32 {
        // TODO optimize
        let x0 = self.get_tilt_offset(i);
        let x1 = self.get_tilt_offset(i + 1);
        let x2 = self.get_tilt_offset(i + 2);
        let y0 = self.get_tilt_radian(i);
        let y1 = self.get_tilt_radian(i + 1);
        let y2 = self.get_tilt_radian(i + 2);
        (y0 * (x - x1) / (x0 - x1) * (x - x2) / (x0 - x2))
            + (y1 * (x - x0) / (x1 - x0) * (x - x2) / (x1 - x2))
            + (y2 * (x - x0) / (x2 - x0) * (x - x1) / (x2 - x1))
    }

    fn get_tilt(&self, offset: f32) -> f32 {
        let pre_baked = offset.rem_euclid(self.length);
        let offset = self.convert_baked_offset(offset);
        let index = offset.floor() as isize;
        let a = self.lagrange(index - 1, pre_baked);
        let b = self.lagrange(index, pre_baked);
        let offset = offset - index as f32;
        a * (1.0 - offset) + b * offset
    }

    pub fn get_up_right(&self, offset: f32) -> (Vector, Vector) {
        let sa = (offset - FORWARD_VECTOR_SIZE).rem_euclid(self.length);
        let sb = (offset + FORWARD_VECTOR_SIZE).rem_euclid(self.length);
        let target = (self.get_baked(sb) - self.get_baked(sa)).normalized();
        let look = Mtx::looking_at(target, Vector::Y_AXIS);
        let tilt = Mtx::axis_angle(&target, self.get_tilt(offset));
        let up = (Vector::Y_AXIS * look) * tilt;
        let right = (Vector::X_AXIS * look) * tilt;
        (up, right)
    }

    pub fn get_up_height(&self, octree: &bindings::Octree, pos: Vector) -> Option<(Vector, f32)> {
        let offset = octree.find_closest_offset(self, pos);
        let point = self.get_baked(offset);
        let (up, right) = self.get_up_right(offset);
        let d = pos - point;
        if right.dot(&d).abs() > bindings::SPLINE_TRACK_RADIUS as f32 {
            None
        } else {
            Some((up, up.dot(&d)))
        }
    }

    pub fn get_offset_and_dist_sq(&self, point: Vector, index: usize) -> (f32, f32) {
        let next_index = (index + 1) % self.numBaked;
        let offset = self.baked[index].offset;
        let interval = self.baked[next_index].offset - offset;
        let origin = Vector::from(self.baked[index].point);
        let direction = (Vector::from(self.baked[next_index].point) - origin) / interval;
        let proj = point - origin;
        let d = proj.dot(&direction).max(interval).min(0.0);
        let proj = (direction * d) + origin;
        let dist = proj.dist_sq(point);
        (offset + d, dist)
    }
}
