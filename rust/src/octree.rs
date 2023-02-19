use std::mem::zeroed;

use crate::{bindings, linalg::Vector};

const MAX_DEPTH: usize = 3;

fn check_bounds(v: Vector, min: &mut Vector, max: &mut Vector) {
    min.x = v.x.min(min.x);
    min.y = v.y.min(min.y);
    min.z = v.z.min(min.z);
    max.x = v.x.max(max.x);
    max.y = v.y.max(max.y);
    max.z = v.z.max(max.z);
}

fn get_bounds(spline: &bindings::Spline, i: usize) -> (Vector, Vector) {
    let mut min = Vector::MAX;
    let mut max = Vector::MIN;
    for b in [spline.baked[i], spline.baked[(i + 1) % spline.numBaked]] {
        let point = Vector::from(b.point);
        let (up, right) = spline.get_up_right(b.offset);
        let right = right * bindings::SPLINE_TRACK_RADIUS as f32;
        let above = up * bindings::MAX_GRAVITY_HEIGHT as f32;
        let below = up * -bindings::COLLISION_DEPTH as f32;
        check_bounds(above - right + point, &mut min, &mut max);
        check_bounds(above + right + point, &mut min, &mut max);
        check_bounds(below - right + point, &mut min, &mut max);
        check_bounds(below + right + point, &mut min, &mut max);
    }
    (min, max)
}

fn build_octree(child_pool: &mut [bindings::OctreeNode], depth: usize, w: &mut usize) -> bindings::OctreeNode {
    let mut result = bindings::OctreeNode {
        segments: -1,
        vehicles: -1,
        children_index: -1,
    };
    if depth < MAX_DEPTH {
        result.children_index = *w as i32;
        let j = *w;
        *w += 8;
        for i in 0..8 {
            child_pool[i + j] = build_octree(child_pool, depth + 1, w);
        }
    }
    result
}

macro_rules! search_axis {
    ($segment_min:expr, $segment_max:expr, $min:expr, $max:expr, $center:expr, $axis:ident) => {
        if $segment_min.$axis < $center.$axis && $segment_max.$axis < $center.$axis {
            $max.$axis = $center.$axis;
            Some(false)
        } else if $segment_min.$axis > $center.$axis && $segment_max.$axis > $center.$axis {
            $min.$axis = $center.$axis;
            Some(true)
        } else {
            None
        }
    };

    ($point:expr, $min:expr, $max:expr, $center:expr, $axis:ident) => {
        if $point.$axis < $center.$axis {
            $max.$axis = $center.$axis;
            false
        } else {
            $min.$axis = $center.$axis;
            true
        }
    };
}

fn search_octree(segment_min: &Vector, segment_max: &Vector, min: &mut Vector, max: &mut Vector) -> [Option<bool>; 3] {
    let center = (*min + *max) * 0.5;

    [
        search_axis!(segment_min, segment_max, min, max, center, x),
        search_axis!(segment_min, segment_max, min, max, center, y),
        search_axis!(segment_min, segment_max, min, max, center, z),
    ]
}

fn search_existing_octree(point: &Vector, min: &mut Vector, max: &mut Vector) -> [bool; 3] {
    let center = (*min + *max) * 0.5;

    [
        search_axis!(point, min, max, center, x),
        search_axis!(point, min, max, center, y),
        search_axis!(point, min, max, center, z),
    ]
}

fn pool_index(which: &[Option<bool>; 3]) -> Option<usize> {
    Some(usize::from(which[0]?) | (usize::from(which[1]?) << 1) | (usize::from(which[2]?) << 2))
}

fn existing_pool_index(which: &[bool; 3]) -> usize {
    usize::from(which[0]) | (usize::from(which[1]) << 1) | (usize::from(which[2]) << 2)
}

impl bindings::Octree {
    pub fn new(spline: &bindings::Spline) -> Self {
        // decide bounds
        let mut min = Vector::MAX;
        let mut max = Vector::MIN;
        for i in 0..spline.numBaked {
            // get bounds of segment
            let (segment_min, segment_max) = get_bounds(spline, i);
            // update bounds
            check_bounds(segment_min, &mut min, &mut max);
            check_bounds(segment_max, &mut min, &mut max);
        }
        // build structure
        let mut w = 0;
        let mut child_pool = [unsafe { zeroed() }; bindings::OCTREE_POOL_SIZE as usize];
        let mut segment_next = [-1; bindings::MAX_BAKED_POINTS as usize];
        let mut segment_sides = [0; bindings::MAX_BAKED_POINTS as usize];
        let mut root = build_octree(&mut child_pool, 0, &mut w);
        // for each segment, figure out where to put it
        for i in 0..spline.numBaked {
            let (segment_min, segment_max) = get_bounds(spline, i);
            let mut search_min = min;
            let mut search_max = max;
            let mut current = &mut root;
            let mut which;
            loop {
                which = search_octree(&segment_min, &segment_max, &mut search_min, &mut search_max);
                if current.children_index == -1 {
                    break;
                }
                if let Some(index) = pool_index(&which) {
                    current = &mut child_pool[current.children_index as usize + index];
                } else {
                    break;
                }
            }
            // add to list
            for (j, w) in which.iter().enumerate() {
                if let Some(b) = w {
                    segment_sides[i] |= 1 << ((j << 1) as u8 | u8::from(*b));
                }
            }
            segment_next[i] = current.segments;
            current.segments = i as i32;
        }
        let mut min_write = [0.0f32; 3];
        let mut max_write = [0.0f32; 3];
        min.write(&mut min_write as *mut f32);
        max.write(&mut max_write as *mut f32);
        Self {
            min: min_write,
            max: max_write,

            root,
            childPool: child_pool,

            segmentNext: segment_next,
            segmentSides: segment_sides,

            vehicleNext: [-1; bindings::MAX_VEHICLES as usize],
            vehicleSides: [0; bindings::MAX_VEHICLES as usize],
        }
    }

    pub fn reset_vehicles(&mut self) {
        self.root.vehicles = -1;
        for child in self.childPool.iter_mut() {
            child.vehicles = -1;
        }
    }

    pub fn add_vehicle(&mut self, pos: Vector, index: usize) {
        let vehicle_min = pos - Vector::new(
            2.0 * bindings::VEHICLE_RADIUS as f32,
            2.0 * bindings::VEHICLE_RADIUS as f32,
            2.0 * bindings::VEHICLE_RADIUS as f32,
        );
        let vehicle_max = pos + Vector::new(
            2.0 * bindings::VEHICLE_RADIUS as f32,
            2.0 * bindings::VEHICLE_RADIUS as f32,
            2.0 * bindings::VEHICLE_RADIUS as f32,
        );
        let mut search_min = Vector::from(self.min);
        let mut search_max = Vector::from(self.max);
        let mut current = &mut self.root;
        let mut which;
        loop {
            which = search_octree(&vehicle_min, &vehicle_max, &mut search_min, &mut search_max);
            if current.children_index == -1 {
                break;
            }
            if let Some(index) = pool_index(&which) {
                current = &mut self.childPool[current.children_index as usize + index];
            } else {
                break;
            }
        }
        // add to list
        self.vehicleSides[index] = 0;
        for (j, w) in which.iter().enumerate() {
            if let Some(b) = w {
                self.vehicleSides[index] |= 1 << ((j << 1) as u8 | u8::from(*b));
            }
        }
        self.vehicleNext[index] = current.vehicles;
        current.vehicles = index as i32;
    }

    pub fn find_vehicle_collisions(&self, point: &Vector) -> Vec<usize> {
        let mut search_min = Vector::from(self.min);
        let mut search_max = Vector::from(self.max);
        let mut current = &self.root;
        let mut result = vec![];
        loop {
            let which = search_existing_octree(&point, &mut search_min, &mut search_max);

            let mut index = current.vehicles;
            while index >= 0 {
                let i = index as usize;
                if !(which[0] && (self.vehicleSides[i] & 1) != 0) &&
                    !(!which[0] && (self.vehicleSides[i] & 2) != 0) &&
                    !(which[1] && (self.vehicleSides[i] & 4) != 0) &&
                    !(!which[1] && (self.vehicleSides[i] & 8) != 0) &&
                    !(which[2] && (self.vehicleSides[i] & 16) != 0) &&
                    !(!which[2] && (self.vehicleSides[i] & 32) != 0) {
                    result.push(i);
                }
                index = self.vehicleNext[i];
            }

            if current.children_index == -1 {
                break result;
            }
            current = &self.childPool[current.children_index as usize + existing_pool_index(&which)];
        }
    }

    pub fn find_closest_offset(&self, spline: &bindings::Spline, point: Vector) -> f32 {
        let mut search_min = Vector::from(self.min);
        let mut search_max = Vector::from(self.max);
        let mut current = &self.root;
        let mut result = 0.0;
        let mut best_dist_sq = f32::INFINITY;
        loop {
            let which = search_existing_octree(&point, &mut search_min, &mut search_max);

            let mut index = current.segments;
            while index >= 0 {
                let i = index as usize;
                if !(which[0] && (self.segmentSides[i] & 1) != 0) &&
                    !(!which[0] && (self.segmentSides[i] & 2) != 0) &&
                    !(which[1] && (self.segmentSides[i] & 4) != 0) &&
                    !(!which[1] && (self.segmentSides[i] & 8) != 0) &&
                    !(which[2] && (self.segmentSides[i] & 16) != 0) &&
                    !(!which[2] && (self.segmentSides[i] & 32) != 0) {
                    let (offset, dist_sq) = spline.get_offset_and_dist_sq(point, i);
                    if dist_sq < best_dist_sq {
                        best_dist_sq = dist_sq;
                        result = offset;
                    }
                }
                index = self.segmentNext[i];
            }

            if current.children_index == -1 {
                break result;
            }
            current = &self.childPool[current.children_index as usize + existing_pool_index(&which)];
        }
    }
}

#[no_mangle]
pub extern "C" fn octree_init(tree: *mut bindings::Octree, spline: *const bindings::Spline) {
    *unsafe { &mut *tree } = bindings::Octree::new(unsafe { &*spline });
}
