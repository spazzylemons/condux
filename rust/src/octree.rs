use crate::{linalg::Vector, spline::Spline, vehicle::Vehicle};

const MAX_DEPTH: usize = 3;

struct OctreeListEntry {
    index: usize,
    sides: u8,
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

fn select_which<'a>(entries: &'a Vec<OctreeListEntry>, which: &'a [bool; 3]) -> impl Iterator<Item = usize> + 'a {
    entries.iter().filter(|entry| {
        (!which[0] || (entry.sides & 1) == 0) &&
        (which[0] || (entry.sides & 2) == 0) &&
        (!which[1] || (entry.sides & 4) == 0) &&
        (which[1] || (entry.sides & 8) == 0) &&
        (!which[2] || (entry.sides & 16) == 0) &&
        (which[2] || (entry.sides & 32) == 0)
    }).map(|entry| entry.index)
}

fn sides_to_bitmask(which: &[Option<bool>; 3]) -> u8 {
    let mut sides = 0;
    for (j, w) in which.iter().enumerate() {
        if let Some(b) = w {
            sides |= 1 << ((j << 1) as u8 | u8::from(*b));
        }
    }
    sides
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
    for b in [&spline.baked[i], &spline.baked[(i + 1) % spline.baked.len()]] {
        let point = Vector::from(b.point);
        let (up, right) = spline.get_up_right(b.offset);
        let right = right * Spline::TRACK_RADIUS;
        let above = up * Vehicle::MAX_GRAVITY_HEIGHT;
        let below = up * -Vehicle::COLLISION_DEPTH;
        check_bounds(above - right + point, &mut min, &mut max);
        check_bounds(above + right + point, &mut min, &mut max);
        check_bounds(below - right + point, &mut min, &mut max);
        check_bounds(below + right + point, &mut min, &mut max);
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

impl OctreeNode {
    fn extract_child(&mut self, which: &[Option<bool>; 3]) -> Option<&mut Self> {
        Some(&mut self.children.as_mut()?[pool_index(which)?])
    }

    fn add<F>(&mut self, mut min: Vector, mut max: Vector, segment_min: &Vector, segment_max: &Vector, func: F)
    where F: FnOnce(&mut Self, u8) {
        let which = search_octree(segment_min, segment_max, &mut min, &mut max);
        if let Some(child) = self.extract_child(&which) {
            child.add(min, max, segment_min, segment_max, func)
        } else {
            let bitmask = sides_to_bitmask(&which);
            func(self, bitmask);
        }
    }
}

impl Octree {
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
            root.add(min, max, &segment_min, &segment_max, |node, sides| {
                node.segments.push(OctreeListEntry {
                    index: i,
                    sides,
                });
            });
        }
        Self {
            min,
            max,

            root,
        }
    }

    pub fn reset_vehicles(&mut self) {
        self.root.reset_vehicles();
    }

    pub fn add_vehicle(&mut self, pos: Vector, index: usize) {
        let vehicle_min = pos - Vector::new(
            2.0 * Vehicle::RADIUS,
            2.0 * Vehicle::RADIUS,
            2.0 * Vehicle::RADIUS,
        );
        let vehicle_max = pos + Vector::new(
            2.0 * Vehicle::RADIUS,
            2.0 * Vehicle::RADIUS,
            2.0 * Vehicle::RADIUS,
        );
        self.root.add(self.min, self.max, &vehicle_min, &vehicle_max, |node, sides| {
            node.vehicles.push(OctreeListEntry { index, sides });
        });
    }

    pub fn find_vehicle_collisions(&self, point: &Vector) -> Vec<usize> {
        let mut search_min = self.min;
        let mut search_max = self.max;
        let mut current = &self.root;
        let mut result = vec![];
        loop {
            let which = search_existing_octree(&point, &mut search_min, &mut search_max);

            for index in select_which(&current.vehicles, &which) {
                result.push(index);
            }

            if let Some(children) = &current.children {
                current = &children[existing_pool_index(&which)];
            } else {
                break result;
            }
        }
    }

    pub fn find_closest_offset(&self, spline: &Spline, point: Vector) -> f32 {
        let mut search_min = self.min;
        let mut search_max = self.max;
        let mut current = &self.root;
        let mut result = 0.0;
        let mut best_dist_sq = f32::INFINITY;
        loop {
            let which = search_existing_octree(&point, &mut search_min, &mut search_max);

            for index in select_which(&current.segments, &which) {
                let (offset, dist_sq) = spline.get_offset_and_dist_sq(point, index);
                if dist_sq < best_dist_sq {
                    best_dist_sq = dist_sq;
                    result = offset;
                }
            }

            if let Some(children) = &current.children {
                current = &children[existing_pool_index(&which)];
            } else {
                break result;
            }
        }
    }
}
