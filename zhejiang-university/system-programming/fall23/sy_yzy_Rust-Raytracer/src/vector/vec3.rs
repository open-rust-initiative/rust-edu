use rand::prelude::*;
use std::ops::{self, Index, IndexMut, Neg};

use crate::global::{Random, clamp};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec3 {
    e: [f64; 3],
}

impl Vec3 {
    pub fn zero() -> Vec3 {
        Vec3 { e: [0.0, 0.0, 0.0] }
    }
    pub fn one() -> Vec3 {
        Vec3 { e: [1.0, 1.0, 1.0] }
    }
    pub fn splat(num: f64) -> Vec3 {
        Vec3 { e: [num, num, num] }
    }
    pub fn new(e0: f64, e1: f64, e2: f64) -> Vec3 {
        Vec3 { e: [e0, e1, e2] }
    }

    pub fn x(self) -> f64 {
        self.e[0]
    }

    pub fn y(self) -> f64 {
        self.e[1]
    }

    pub fn z(self) -> f64 {
        self.e[2]
    }

    pub fn r(self) -> f64 {
        self.e[0]
    }

    pub fn g(self) -> f64 {
        self.e[1]
    }

    pub fn b(self) -> f64 {
        self.e[2]
    }

    pub fn clamp(mut self, v1 : Vec3, v2 : Vec3) -> Self {
        self.e[0] = clamp(self.e[0], v1.e[0], v2.e[0]);
        self.e[1] = clamp(self.e[1], v1.e[1], v2.e[1]);
        self.e[2] = clamp(self.e[2], v1.e[2], v2.e[2]);
        self
    }

    pub fn squared_length(self) -> f64 {
        self.e[0] * self.e[0] + self.e[1] * self.e[1] + self.e[2] * self.e[2]
    }
    pub fn length(self) -> f64 {
        (self.e[0] * self.e[0] + self.e[1] * self.e[1] + self.e[2] * self.e[2]).sqrt()
    }
    pub fn unit_vector_self(self) -> Vec3 {
        self / self.length()
    }
    pub fn unit_vector(v: &Vec3) -> Vec3 {
        *v / v.length()
    }
    pub fn dot_self(self, v2: &Vec3) -> f64 {
        self.e[0] * v2.e[0] + self.e[1] * v2.e[1] + self.e[2] * v2.e[2]
    }
    pub fn dot(v1: &Vec3, v2: &Vec3) -> f64 {
        v1.e[0] * v2.e[0] + v1.e[1] * v2.e[1] + v1.e[2] * v2.e[2]
    }
    pub fn cross_self(self, v2: &Vec3) -> Vec3 {
        Vec3 {
            e: [
                self.e[1] * v2.e[2] - self.e[2] * v2.e[1],
                self.e[2] * v2.e[0] - self.e[0] * v2.e[2],
                self.e[0] * v2.e[1] - self.e[1] * v2.e[0],
            ],
        }
    }

    pub fn cross(v1: &Vec3, v2: &Vec3) -> Vec3 {
        Vec3 {
            e: [
                v1.e[1] * v2.e[2] - v1.e[2] * v2.e[1],
                v1.e[2] * v2.e[0] - v1.e[0] * v2.e[2],
                v1.e[0] * v2.e[1] - v1.e[1] * v2.e[0],
            ],
        }
    }

    pub fn random() -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3 {
            e: [rng.gen(), rng.gen(), rng.gen()],
        }
    }

    pub fn near_zero(self) -> bool {
        let esp = 1e-8;
        self[0].abs() < esp && self[1].abs() < esp && self[2].abs() < esp
    }

    pub fn random_init(min: f64, max: f64) -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3 {
            e: [
                rng.gen_range(min, max),
                rng.gen_range(min, max),
                rng.gen_range(min, max),
            ],
        }
    }

    pub fn random_in_unit_box() -> Self {
        Self::new(
            Random::range(-1.0..1.0),
            Random::range(-1.0..1.0),
            Random::range(-1.0..1.0),
        )
    }

    fn random_in_unit_sphere() -> Vec3 {
        let mut rng = rand::thread_rng();
        loop {
            let vec = Vec3::new(
                rng.gen_range(-1.0, 1.0),
                rng.gen_range(-1.0, 1.0),
                rng.gen_range(-1.0, 1.0),
            );

            if vec.squared_length() < 1. {
                break vec;
            }
        }
    }
    // 平面圆内一点
    pub fn random_in_unit_disk() -> Vec3 {
        let mut rng = rand::thread_rng();
        loop {
            let vec = Vec3::new(
                rng.gen_range(-1.0, 1.0),
                rng.gen_range(-1.0, 1.0),
                0.0
            );

            if vec.squared_length() < 1. {
                break vec;
            }
        }
    }

    pub fn random_unit_vector() -> Vec3 {
        return Self::random_in_unit_sphere().unit_vector_self();
    }
    // 求镜面反射光线方向向量 (入射光方向向量，法线方向向量)
    pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
        return v - 2.0 * Vec3::dot(&v, &n) * n;
    }
    // 聂耳定律 -> 求折射光的方向（入射光方向，法线，折射率）
    pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f64) -> Vec3 {
        let cos_theta = Vec3::dot(&-uv, &n).min(1.0);
        let r_out_perp = etai_over_etat * (uv + cos_theta * n);
        let r_out_parallel: Vec3 = -(1.0 - r_out_perp.squared_length()).abs().sqrt() * n;
        return r_out_perp + r_out_parallel;
    }

    pub fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        // Use Schlick's approximation for reflectance.
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 = r0 * r0;
        return r0 + (1. - r0) * ((1. - cosine).powf(5.0));
    }

    pub fn new_min(v1 : Vec3, v2 : Vec3) -> Vec3 {
        Vec3::new(v1[0].min(v2[0]), v1[1].min(v2[1]), v1[2].min(v2[2]))
    }
    pub fn new_max(v1 : Vec3, v2 : Vec3) -> Vec3 {
        Vec3::new(v1[0].max(v2[0]), v1[1].max(v2[1]), v1[2].max(v2[2]))
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.e[index]
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.e[index]
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec3 {
            e: [-self.e[0], -self.e[1], -self.e[2]],
        }
    }
}

impl ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            e: [
                self.e[0] + rhs.e[0],
                self.e[1] + rhs.e[1],
                self.e[2] + rhs.e[2],
            ],
        }
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            e: [
                self.e[0] - rhs.e[0],
                self.e[1] - rhs.e[1],
                self.e[2] - rhs.e[2],
            ],
        }
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            e: [rhs.e[0] * self, rhs.e[1] * self, rhs.e[2] * self],
        }
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3 {
            e: [self.e[0] * rhs, self.e[1] * rhs, self.e[2] * rhs],
        }
    }
}

impl ops::Mul for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec3 {
            e: [
                self.e[0] * rhs.e[0],
                self.e[1] * rhs.e[1],
                self.e[2] * rhs.e[2],
            ],
        }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        let k = 1.0 / rhs;

        Vec3 {
            e: [self.e[0] * k, self.e[1] * k, self.e[2] * k],
        }
    }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.e[0], self.e[1], self.e[2])
    }
}

pub type Color = Vec3;
pub type Point = Vec3;
