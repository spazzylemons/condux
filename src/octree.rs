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

use crate::{linalg::Vector, spline::Spline, vehicle::Vehicle};

const MAX_DEPTH: usize = 3;

struct OctreeListEntry {
    index: usize,
    min: Vector,
    max: Vector,
}

struct OctreeNode {
    segments: Vec<OctreeListEntry>,
    vehicles: Vec<OctreeListEntry>,
    children: Option<Box<[OctreeNode; 8]>>,
}

impl OctreeNode {
    fn reset_vehicles(&mut self) {
        self.vehicles.clear();
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.reset_vehicles();
            }
        }
    }
}

pub struct Octree {
    min: Vector,
    max: Vector,

    root: OctreeNode,
}

fn select_which<'a>(
    entries: &'a [OctreeListEntry],
    point: &'a Vector,
) -> impl Iterator<Item = usize> + 'a {
    entries
        .iter()
        .filter(move |entry| {
            entry.min.x <= point.x
                && entry.max.x >= point.x
                && entry.min.y <= point.y
                && entry.max.y >= point.y
                && entry.min.z <= point.z
                && entry.max.z >= point.z
        })
        .map(|entry| entry.index)
}

fn check_bounds(v: Vector, min: &mut Vector, max: &mut Vector) {
    min.x = v.x.min(min.x);
    min.y = v.y.min(min.y);
    min.z = v.z.min(min.z);
    max.x = v.x.max(max.x);
    max.y = v.y.max(max.y);
    max.z = v.z.max(max.z);
}

fn get_bounds(spline: &Spline, i: usize) -> (Vector, Vector) {
    let mut min = Vector::MAX;
    let mut max = Vector::MIN;
    for b in [
        &spline.baked[i],
        &spline.baked[(i + 1) % spline.baked.len()],
    ] {
        let (up, right) = spline.get_up_right(b.offset);
        let right = right * Spline::BOUNDS_RADIUS;
        let above = up * Vehicle::MAX_GRAVITY_HEIGHT;
        let below = up * -Vehicle::COLLISION_DEPTH;
        check_bounds(above - right + b.point, &mut min, &mut max);
        check_bounds(above + right + b.point, &mut min, &mut max);
        check_bounds(below - right + b.point, &mut min, &mut max);
        check_bounds(below + right + b.point, &mut min, &mut max);
    }
    (min, max)
}

fn build_octree(depth: usize) -> OctreeNode {
    let mut result = OctreeNode {
        segments: vec![],
        vehicles: vec![],
        children: None,
    };
    if depth < MAX_DEPTH {
        result.children = Some(Box::new([
            build_octree(depth + 1),
            build_octree(depth + 1),
            build_octree(depth + 1),
            build_octree(depth + 1),
            build_octree(depth + 1),
            build_octree(depth + 1),
            build_octree(depth + 1),
            build_octree(depth + 1),
        ]));
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

fn search_octree(
    segment_min: &Vector,
    segment_max: &Vector,
    min: &mut Vector,
    max: &mut Vector,
) -> [Option<bool>; 3] {
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

fn pool_index(which: [Option<bool>; 3]) -> Option<usize> {
    Some(usize::from(which[0]?) | (usize::from(which[1]?) << 1) | (usize::from(which[2]?) << 2))
}

fn existing_pool_index(which: [bool; 3]) -> usize {
    usize::from(which[0]) | (usize::from(which[1]) << 1) | (usize::from(which[2]) << 2)
}

impl OctreeNode {
    fn extract_child(&mut self, which: [Option<bool>; 3]) -> Option<&mut Self> {
        Some(&mut self.children.as_mut()?[pool_index(which)?])
    }

    fn add<F>(
        &mut self,
        mut min: Vector,
        mut max: Vector,
        segment_min: &Vector,
        segment_max: &Vector,
        func: F,
    ) where
        F: FnOnce(&mut Self),
    {
        let which = search_octree(segment_min, segment_max, &mut min, &mut max);
        if let Some(child) = self.extract_child(which) {
            child.add(min, max, segment_min, segment_max, func);
        } else {
            func(self);
        }
    }
}

impl Octree {
    #[must_use]
    pub fn new(spline: &Spline) -> Self {
        // decide bounds
        let mut min = Vector::MAX;
        let mut max = Vector::MIN;
        for i in 0..spline.baked.len() {
            // get bounds of segment
            let (segment_min, segment_max) = get_bounds(spline, i);
            // update bounds
            check_bounds(segment_min, &mut min, &mut max);
            check_bounds(segment_max, &mut min, &mut max);
        }
        // build structure
        let mut root = build_octree(0);
        // for each segment, figure out where to put it
        for i in 0..spline.baked.len() {
            let (segment_min, segment_max) = get_bounds(spline, i);
            root.add(min, max, &segment_min, &segment_max, |node| {
                node.segments.push(OctreeListEntry {
                    index: i,
                    min: segment_min,
                    max: segment_max,
                });
            });
        }
        Self { min, max, root }
    }

    pub fn reset_vehicles(&mut self) {
        self.root.reset_vehicles();
    }

    pub fn add_vehicle(&mut self, pos: Vector, index: usize) {
        let vehicle_min = pos
            - Vector::new(
                2.0 * Vehicle::RADIUS,
                2.0 * Vehicle::RADIUS,
                2.0 * Vehicle::RADIUS,
            );
        let vehicle_max = pos
            + Vector::new(
                2.0 * Vehicle::RADIUS,
                2.0 * Vehicle::RADIUS,
                2.0 * Vehicle::RADIUS,
            );
        self.root
            .add(self.min, self.max, &vehicle_min, &vehicle_max, |node| {
                node.vehicles.push(OctreeListEntry {
                    index,
                    min: vehicle_min,
                    max: vehicle_max,
                });
            });
    }

    #[must_use]
    pub fn find_vehicle_collisions(&self, point: &Vector) -> Vec<usize> {
        let mut search_min = self.min;
        let mut search_max = self.max;
        let mut current = &self.root;
        let mut result = vec![];
        loop {
            let which = search_existing_octree(point, &mut search_min, &mut search_max);

            for index in select_which(&current.vehicles, point) {
                result.push(index);
            }

            if let Some(children) = &current.children {
                current = &children[existing_pool_index(which)];
            } else {
                break result;
            }
        }
    }

    #[must_use]
    pub fn find_closest_offset(&self, spline: &Spline, point: Vector) -> Option<f32> {
        let mut search_min = self.min;
        let mut search_max = self.max;
        let mut current = &self.root;
        let mut result = None;
        let mut best_dist_sq = f32::INFINITY;
        loop {
            let which = search_existing_octree(&point, &mut search_min, &mut search_max);

            for index in select_which(&current.segments, &point) {
                let (offset, dist_sq) = spline.get_offset_and_dist_sq(point, index);
                if dist_sq < best_dist_sq {
                    best_dist_sq = dist_sq;
                    result = Some(offset);
                }
            }

            if let Some(children) = &current.children {
                current = &children[existing_pool_index(which)];
            } else {
                break result;
            }
        }
    }
}
