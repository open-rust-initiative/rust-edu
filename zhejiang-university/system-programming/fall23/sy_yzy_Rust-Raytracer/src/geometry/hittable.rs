use crate::{
    geometry::{aabb::AABB, ray::Ray},
    materials::material::*,
    vector::vec3::Vec3,
};
use std::ops::Range;

#[derive(Clone)]
pub struct HitRecord {
    pub point: Vec3,        // 交点
    pub normal: Vec3,       // 法线
    pub t: f64,             // 光线的 t
    pub front_face: bool,   // 朝向
    pub material: Material, // 材质
    pub u: f64,             // 纹理
    pub v: f64,             // 纹理
}

impl Default for HitRecord {
    fn default() -> Self {
        Self {
            point: Default::default(),
            normal: Default::default(),
            t: Default::default(),
            front_face: Default::default(),
            material: Material::Dielectric {
                index_of_refraction: (0.6f64),
            },
            u: Default::default(),
            v: Default::default(),
        }
    }
}

impl HitRecord {
    pub fn with_face_normal(
        material: Material,
        point: Vec3,
        outward_normal: Vec3,
        t: f64,
        ray: &Ray,
        u: f64,
        v: f64,
    ) -> Self {
        let (front_face, normal) = HitRecord::set_face_normal(ray, &outward_normal);
        HitRecord {
            material,
            point,
            normal,
            t,
            front_face,
            u,
            v,
        }
    }
    pub fn set_face_normal(r: &Ray, outward_normal: &Vec3) -> (bool, Vec3) {
        let front_face = Vec3::dot(&r.dir, outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal.clone()
        } else {
            -outward_normal.clone()
        };
        (front_face, normal)
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord>;
    fn bbox(&self) -> Option<AABB>;
}

impl<T> Hittable for Vec<T>
where
    T: Hittable + Sync + Send,
{
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord> {
        let (_closest, hit_record) = self.iter().fold((interval.end, None), |acc, item| {
            if let Some(temp_rec) = item.hit(ray, interval.start..acc.0) {
                (temp_rec.t, Some(temp_rec))
            } else {
                acc
            }
        });

        hit_record
    }

    fn bbox(&self) -> Option<AABB> {
        Some(AABB::default())
    }
}
