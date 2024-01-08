use std::ops::Neg;

use crate::geometry::{hittable::*, ray::*};
use crate::textures::texture::Texture;
use crate::vector::vec3::Vec3;

use rand::Rng;
// 材质
#[non_exhaustive]
#[derive(Clone)]
pub enum Material {
    Lambertian { albedo: Texture }, // 以某种概率分布衰减，albedo / p
    Metal { albedo: Vec3, fuzz: f64 },
    Dielectric { index_of_refraction: f64 },
    DiffuseLight(Texture),
    Isotropic { albedo: Texture },
}
pub struct Scattered {
    pub attenuation: Vec3, // 颜色衰减
    pub scattered: Ray,    // 散射光
}

impl Material {
    pub fn scatter(&self, r_in: &Ray, hit_record: &HitRecord) -> Option<Scattered> {
        match self {
            Material::Lambertian { albedo } => {
                // 散射方向
                let mut scatter_direction = hit_record.normal + Vec3::random_unit_vector();

                if scatter_direction.near_zero() {
                    scatter_direction = hit_record.normal;
                }

                Some(Scattered {
                    attenuation: albedo.color(hit_record.u, hit_record.v, hit_record.point),
                    scattered: Ray {
                        orig: hit_record.point,
                        dir: scatter_direction,
                    },
                })
            }
            Material::Metal { albedo, fuzz } => {
                let reflected: Vec3 = Vec3::reflect(r_in.dir.unit_vector_self(), hit_record.normal);
                let scattered = Ray {
                    orig: hit_record.point,
                    dir: reflected + *fuzz * Vec3::random_unit_vector(),
                };
                if scattered.dir.dot_self(&hit_record.normal) > 0.0 {
                    Some(Scattered {
                        attenuation: *albedo,
                        scattered,
                    })
                } else {
                    None
                }
            }
            Material::Dielectric {
                index_of_refraction,
            } => {
                let mut rng = rand::thread_rng();

                let attenuation = Vec3::one();
                let refraction_ratio: f64 = if hit_record.front_face {
                    1.0 / index_of_refraction
                } else {
                    index_of_refraction.clone()
                };

                let unit_direction = r_in.dir.unit_vector_self();

                let cos_theta = Vec3::dot(&-unit_direction, &hit_record.normal).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let cannot_refract = refraction_ratio * sin_theta > 1.0;
                let direction = if cannot_refract
                    || Vec3::reflectance(cos_theta, refraction_ratio) > rng.gen::<f64>()
                {
                    // print!("1");
                    Vec3::reflect(unit_direction, hit_record.normal)
                } else {
                    // print!("2");
                    Vec3::refract(unit_direction, hit_record.normal, refraction_ratio)
                };
                // print!("{}", direction);
                Some(Scattered {
                    attenuation,
                    scattered: Ray {
                        orig: hit_record.point,
                        dir: direction,
                    },
                })
            }
            Material::DiffuseLight(_) => None,
            Material::Isotropic { albedo } => {
                let scattered = Ray {
                    orig: hit_record.point,
                    dir: Vec3::random_unit_vector(),
                };
                let attenuation = albedo.color(hit_record.u, hit_record.v, hit_record.point);
                Some(Scattered {
                    attenuation,
                    scattered,
                })
            }
        }
    }
    pub fn emitted(&self, u: f64, v: f64, point: Vec3) -> Vec3 {
        match self {
            Material::DiffuseLight(texture) => texture.color(u, v, point),
            _ => Vec3::new(0.0, 0.0, 0.0),
        }
    }
}
