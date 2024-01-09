use super::hittable::Hittable;
use crate::materials::material::*;
use crate::vector::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub orig: Vec3,
    pub dir: Vec3,
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            orig: Default::default(),
            dir: Default::default(),
        }
    }
}

impl Ray {
    pub fn ray(a: Vec3, b: Vec3) -> Ray {
        Ray { orig: a, dir: b }
    }

    pub fn origin(self) -> Vec3 {
        self.orig
    }

    pub fn direction(self) -> Vec3 {
        self.dir
    }

    pub fn point_at_parameter(self, t: f64) -> Vec3 {
        self.orig + self.dir * t
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.orig + t * self.dir
    }
    pub fn color<T>(&self, depth: u32, world: &T, miss_color: &Option<Vec3>) -> Vec3
    where
        T: Hittable + std::marker::Sync,
    {
        // depth == 0 means we're done
        if depth <= 0 {
            return Vec3::zero();
        }
        // if we hit something
        if let Some(rec) = world.hit(&self, (0.001)..f64::INFINITY) {
            let color_from_emission = rec.material.emitted(rec.u, rec.v, rec.point);

            let Some(Scattered {
                attenuation,
                scattered,
            }) = rec.material.scatter(self, &rec)
            else {
                return color_from_emission;
            };

            // recurse to follow more bounces
            let color_from_scatter = attenuation * scattered.color(depth - 1, world, miss_color);
            return color_from_emission + color_from_scatter;
        }

        miss_color.unwrap_or_else(|| {
            // this is sky because we missed everything
            let a = 0.5 * (self.dir.unit_vector_self().y() + 1.0);
            return (1.0 - a) * Vec3::new(1.0, 1.0, 1.0) + a * Vec3::new(0.5, 0.7, 1.0);
        })
    }
}
