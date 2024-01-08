use std::{cmp::Ordering, ops::Range};

use crate::{
    geometry::{
        aabb::AABB,
        hittable::{HitRecord, Hittable},
        hittable_list::HittableList,
        ray::Ray,
    },
    global::Random,
};

pub struct BVH {
    bbox: Option<AABB>,
    left: Option<Box<dyn Hittable>>,
    right: Option<Box<dyn Hittable>>,
}

impl Default for BVH {
    fn default() -> Self {
        Self {
            bbox: Default::default(),
            left: Default::default(),
            right: Default::default(),
        }
    }
}

fn cmp_geometry_by(axis: usize, a: &dyn Hittable, b: &dyn Hittable) -> Ordering {
    let box_a = a.bbox().expect("No bounding box in bvh_node constructor");
    let box_b = b.bbox().expect("No bounding box in bvh_node constructor");

    box_a.min()[axis]
        .partial_cmp(&box_b.min()[axis])
        .expect("Bounding box contains NaN")
}

impl BVH {
    #[must_use]
    pub fn new(objects: HittableList) -> Self {
        let objects = objects.into_objects();
        if objects.is_empty() {
            Self::default()
        } else {
            let mut objects: Vec<_> = objects.into_iter().map(Some).collect();
            let count = objects.len();
            Self::new_internal(&mut objects, 0..count)
        }
    }

    fn new_internal(objects: &mut Vec<Option<Box<dyn Hittable>>>, index: Range<usize>) -> Self {
        let count = index.end - index.start;

        if count == 1 {
            let left = objects[index.start].take().unwrap();
            let bbox = left
                .bbox()
                .expect("No bounding box in bvh_node constructor.");
            Self {
                bbox: Some(bbox),
                left: Some(left),
                right: None,
            }
        } else if count == 2 {
            let left = objects[index.start].take().unwrap();
            let right = objects[index.start + 1].take().unwrap();
            let left_bbox = left
                .bbox()
                .expect("No bounding box in bvh_node constructor.");
            let right_bbox = right
                .bbox()
                .expect("No bounding box in bvh_node constructor.");
            Self {
                bbox: Some(left_bbox | &right_bbox),
                left: Some(left),
                right: Some(right),
            }
        } else {
            let axis = *Random::choose(&[0, 1, 2]);
            objects[index.clone()].sort_by(|a, b| {
                cmp_geometry_by(
                    axis,
                    a.as_ref().unwrap().as_ref(),
                    b.as_ref().unwrap().as_ref(),
                )
            });
            let mid = index.start + count / 2;
            let left = Box::new(Self::new_internal(objects, index.start..mid));
            let right = Box::new(Self::new_internal(objects, mid..index.end));
            Self {
                bbox: Some(left.bbox.as_ref().unwrap() | right.bbox.as_ref().unwrap()),
                left: Some(left),
                right: Some(right),
            }
        }
    }
}

/// Bounding Volume Hierarchies
impl Hittable for BVH {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord> {
        let bbox = self.bbox.as_ref()?;
        if !bbox.hit(ray, interval.clone()) {
            return None;
        }

        let hit_left = self
            .left
            .as_ref()
            .and_then(|left| left.hit(ray, interval.clone()));
        let hit_right = self.right.as_ref().and_then(|right| {
            let right_limit =
                interval.start..hit_left.as_ref().map_or(interval.end, |record| record.t);
            right.hit(ray, right_limit)
        });

        hit_right.or(hit_left)
    }

    fn bbox(&self) -> Option<AABB> {
        self.bbox.clone()
    }
}
