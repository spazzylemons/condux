use std::f32::consts::{PI, TAU};

use crate::{
    assets::Asset,
    linalg::{Length, Mtx, Vector},
    octree::Octree,
};

const BAKE_LENGTH_SQ: f32 = 1.0;

const FORWARD_VECTOR_SIZE: f32 = 0.125;

struct Point {
    point: Vector,
    control: Vector,
    control_mid: f32,
    tilt: f32,
    tilt_offset: f32,
}

pub struct Baked {
    pub point: Vector,
    position: f32,
    pub offset: f32,
}

pub struct Spline {
    /// The control points.
    points: Vec<Point>,
    /// The baked points.
    pub baked: Vec<Baked>,
    /// The total tilt, used for interpolation.
    total_tilt: f32,
    /// The approximate length of the spline.
    pub length: f32,
}

impl Spline {
    pub const TRACK_RADIUS: f32 = 2.0;

    const MAX_BAKE_DEPTH: usize = 5;

    pub fn load(asset: &mut Asset) -> Option<Self> {
        // number of points
        let num_points = asset.read_byte()?;
        if num_points < 3 {
            return None;
        }
        let mut points = vec![];
        // TODO div by zero checks?
        for _ in 0..num_points {
            let point = asset.read_vector()?;
            let tilt = (f32::from(asset.read_byte()?) / 256.0) * TAU;
            points.push(Point {
                point,
                control: Vector::default(),
                control_mid: 0.0,
                tilt,
                tilt_offset: 0.0,
            });
        }
        // fix tilts
        let mut total_tilt = points[0].tilt;
        for i in 0..num_points {
            let delta = (points[usize::from((i + 1) % num_points)].tilt
                - points[usize::from(i)].tilt)
                .rem_euclid(TAU);
            points[usize::from(i)].tilt = total_tilt;
            if delta <= PI {
                // move up
                total_tilt += delta;
            } else {
                // move down
                total_tilt += delta - TAU;
            }
        }
        // generate bezier control points
        for i in 0..num_points {
            let a = usize::from(i);
            let b = usize::from((i + 1) % num_points);
            let c = usize::from((i + 2) % num_points);
            let pa = points[a].point;
            let pb = points[b].point;
            let pc = points[c].point;
            let da = pa.dist(pb);
            let db = pb.dist(pc);
            // TODO handle potential divs by zero in this area
            let mid = da / (da + db);
            let fac_a = (mid - 1.0) / (2.0 * mid);
            let fac_b = 1.0 / (2.0 * mid * (1.0 - mid));
            let fac_c = mid / (2.0 * (mid - 1.0));
            points[a].control = (pa * fac_a) + (pb * fac_b) + (pc * fac_c);
            points[a].control_mid = mid;
        }
        let mut spline = Self {
            points,
            baked: vec![],
            total_tilt,
            length: 0.0,
        };
        // for each point, recursively find points to bake
        for i in 0..num_points {
            // bake at control point
            spline.add_baked(f32::from(i));
            // add length to tilt offsets
            spline.points[usize::from(i)].tilt_offset = spline.length;
            // bake in between
            spline.bake_recursive(i, 0.0, 1.0, 0);
        }
        // finish off length measurement
        let final_length = spline.baked[0]
            .point
            .dist(spline.baked[spline.baked.len() - 1].point);
        spline.length += final_length;
        Some(spline)
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    fn num_points(&self) -> u8 {
        // the number of points is always known to be within u8 range
        self.points.len() as u8
    }

    #[must_use]
    pub fn bezier(&self, index: usize, offset: f32) -> Vector {
        let other_index = (index + 2) % self.points.len();
        let fac_a = (1.0 - offset) * (1.0 - offset);
        let fac_b = 2.0 * (1.0 - offset) * offset;
        let fac_c = offset * offset;
        let pa = self.points[index].point;
        let pb = self.points[index].control;
        let pc = self.points[other_index].point;
        pa * fac_a + pb * fac_b + pc * fac_c
    }

    #[must_use]
    pub fn interpolate(&self, offset: f32) -> Vector {
        let offset = offset.rem_euclid(f32::from(self.num_points()));
        let index = offset as usize;
        let offset = offset - offset.floor();
        let prev_index = (index + self.points.len() - 1) % self.points.len();
        let prev_mid = self.points[prev_index].control_mid;
        let next_mid = self.points[index].control_mid;
        let a = self.bezier(prev_index, offset * (1.0 - prev_mid) + prev_mid);
        let b = self.bezier(index, offset * next_mid);
        a * (1.0 - offset) + b * offset
    }

    pub fn add_baked(&mut self, position: f32) {
        let baked = Baked {
            point: self.interpolate(position),
            position,
            offset: 0.0,
        };
        if !self.baked.is_empty() {
            self.length += baked.point.dist(self.baked[self.baked.len() - 1].point);
        }
        self.baked.push(Baked {
            offset: self.length,
            ..baked
        });
    }

    pub fn bake_recursive(&mut self, index: u8, begin: f32, end: f32, depth: usize) {
        if depth >= Self::MAX_BAKE_DEPTH {
            return;
        }

        let v1 = self.interpolate(f32::from(index) + begin);
        let v2 = self.interpolate(f32::from(index) + end);

        if v1.dist_sq(v2) > BAKE_LENGTH_SQ {
            let mid = (begin + end) * 0.5;
            self.bake_recursive(index, begin, mid, depth + 1);
            self.add_baked(f32::from(index) + mid);
            self.bake_recursive(index, mid, end, depth + 1);
        }
    }

    fn convert_baked_offset(&self, baked_offset: f32) -> f32 {
        // binary search
        let mut start = 0;
        let mut end = self.baked.len();
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
        let next_index = (current + 1) % self.baked.len();
        let offset_begin = self.baked[current].offset;
        let mut offset_end = self.baked[next_index].offset;
        let position_begin = self.baked[current].position;
        let mut position_end = self.baked[next_index].position;
        if next_index == 0 {
            offset_end += self.length;
            position_end += f32::from(self.num_points());
        }
        let interp = (baked_offset - offset_begin) / (offset_end - offset_begin);
        (1.0 - interp) * position_begin + interp * position_end
    }

    #[must_use]
    pub fn get_baked(&self, offset: f32) -> Vector {
        self.interpolate(self.convert_baked_offset(offset))
    }

    #[must_use]
    fn floor_div(&self, i: isize) -> (isize, &Point) {
        let n = isize::from(self.num_points());
        let d = i / n;
        let d = if i < 0 && d * i != n { d - 1 } else { d };
        (d, &self.points[(i - d * n) as usize])
    }

    fn get_tilt_offset(&self, i: isize) -> f32 {
        let (n, p) = self.floor_div(i);
        self.length * n as f32 + p.tilt_offset
    }

    fn get_tilt_radian(&self, i: isize) -> f32 {
        let (n, p) = self.floor_div(i);
        self.total_tilt * n as f32 + p.tilt
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
        let index = offset as isize;
        let a = self.lagrange(index - 1, pre_baked);
        let b = self.lagrange(index, pre_baked);
        let offset = offset - offset.floor();
        a * (1.0 - offset) + b * offset
    }

    #[must_use]
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

    #[must_use]
    pub fn get_up_height(&self, octree: &Octree, pos: Vector) -> Option<(Vector, f32)> {
        let offset = octree.find_closest_offset(self, pos);
        let point = self.get_baked(offset);
        let (up, right) = self.get_up_right(offset);
        let d = pos - point;
        if right.dot(&d).abs() > Self::TRACK_RADIUS {
            None
        } else {
            Some((up, up.dot(&d)))
        }
    }

    #[must_use]
    pub fn get_offset_and_dist_sq(&self, point: Vector, index: usize) -> (f32, f32) {
        let next_index = (index + 1) % self.baked.len();
        let offset = self.baked[index].offset;
        let interval = (self.baked[next_index].offset - offset).abs();
        let origin = self.baked[index].point;
        let direction = (self.baked[next_index].point - origin) / interval;
        let proj = point - origin;
        let d = proj.dot(&direction).clamp(0.0, interval);
        let proj = (direction * d) + origin;
        let dist = proj.dist_sq(point);
        (offset + d, dist)
    }
}
