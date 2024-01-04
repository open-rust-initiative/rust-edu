use crate::geometry::ray::*;
use crate::global::degrees_to_radians;
use crate::vector::vec3::{Point, Vec3};

pub struct Camera {
    pub origin: Point,
    pub lower_left_corner: Point,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
    pub lens_radius: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            lower_left_corner: Default::default(),
            horizontal: Default::default(),
            vertical: Default::default(),
            u: Default::default(),
            v: Default::default(),
            w: Default::default(),
            lens_radius: Default::default(),
        }
    }
}

impl Camera {
    pub fn new(
        lookfrom: Point,
        lookat: Point,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
    ) -> Camera {
        let theta = degrees_to_radians(vfov);
        let h = (theta / 2.0).tan();
        let focus_dist = (lookfrom - lookat).length();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).unit_vector_self();
        let u = vup.cross_self(&w).unit_vector_self();
        let v = w.cross_self(&u);

        let origin = lookfrom;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        let lens_radius = aperture / 2.0;

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            w,
            lens_radius,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let p = self.lens_radius * Vec3::random_in_unit_disk();
        let offset = self.u * p.x() + self.v * p.y();

        Ray {
            orig: self.origin + offset,
            dir: self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.origin
                - offset,
        }
    }
}
