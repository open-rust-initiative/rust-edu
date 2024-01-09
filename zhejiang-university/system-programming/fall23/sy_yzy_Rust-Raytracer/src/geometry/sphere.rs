use std::{f64::consts::PI, ops::Range};

use crate::{
    geometry::{hittable::*, ray::*},
    materials::material::Material,
    textures::texture::Texture,
    vector::vec3::Vec3,
};

use super::AABB;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Material,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Default::default(),
            radius: Default::default(),
            material: Material::Lambertian {
                albedo: Texture::SolidColor(Vec3::new(0.0, 1., 1.)),
            },
        }
    }
}
unsafe impl Send for Sphere {}

impl Sphere {
    pub fn new(center: Vec3, radius: f64, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
    fn get_sphere_uv(&self, p: Vec3) -> (f64, f64) {
        let theta = (-p.y()).acos();
        let phi = (-p.z()).atan2(p.x()) + PI;

        let u = phi / (2. * PI);
        let v = theta / PI;
        (u, v)
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord> {
        let oc = ray.orig - self.center;
        let a = ray.dir.squared_length();
        let half_b = oc.dot_self(&ray.dir);
        let c = oc.squared_length() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        // 找到最近的交点
        let mut root = (-half_b - sqrtd) / a;
        if !interval.contains(&root) {
            root = (-half_b + sqrtd) / a;
            if !interval.contains(&root) {
                return None;
            }
        }

        let t = root;
        let point = ray.at(t);
        let outward_normal = (point - self.center) / self.radius;
        let (u, v) = self.get_sphere_uv(outward_normal);

        let rec =
            HitRecord::with_face_normal(self.material.clone(), point, outward_normal, t, ray, u, v);
        // print!("hit point = {}", point);
        Some(rec)
    }

    fn bbox(&self) -> Option<super::AABB> {
        let r = self.radius;
        let c = self.center;
        Some(AABB::from_points(
            &(c - Vec3::new(r, r, r)),
            &(c + Vec3::new(r, r, r)),
        ))
    }
}
