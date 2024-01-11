use std::ops::Range;

use crate::geometry::{
    aabb::AABB,
    hittable::{HitRecord, Hittable},
    ray::Ray,
};

pub struct HittableList {
    list: Vec<Box<dyn Hittable>>,
    aabb: AABB,
}

impl Default for HittableList {
    fn default() -> Self {
        Self {
            list: Default::default(),
            aabb: Default::default(),
        }
    }
}

unsafe impl Send for HittableList {}
unsafe impl Sync for HittableList {}

impl HittableList {
    pub fn new(list: Vec<Box<dyn Hittable>>) -> HittableList {
        let aabb = HittableList::bounding_box(&list);
        HittableList { list, aabb }
    }
    fn bounding_box(list: &Vec<Box<dyn Hittable>>) -> AABB {
        let mut aabb = AABB::default();
        for item in list {
            match item.bbox() {
                Some(shape_box) => aabb = AABB::from_boxes(&aabb, &shape_box),
                None => {}
            }
        }
        aabb
    }
    pub fn into_objects(self) -> Vec<Box<dyn Hittable>> {
        self.list
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, interval: Range<f64>) -> Option<HitRecord> {
        // print!("hit!");
        let mut temp_rec = HitRecord::default();
        let mut hit_anything = false;
        let mut closest_so_far = interval.end;

        for object in &self.list {
            let now = object.hit(
                r,
                Range {
                    start: interval.start,
                    end: closest_so_far,
                },
            );
            match now {
                Some(hitrec) => {
                    // print!("hit!!!");
                    hit_anything = true;
                    closest_so_far = hitrec.t;
                    temp_rec = hitrec;
                }
                None => {}
            };
        }
        if hit_anything {
            Some(temp_rec)
        } else {
            None
        }
    }

    fn bbox(&self) -> Option<super::AABB> {
        Some(self.aabb)
    }
}
