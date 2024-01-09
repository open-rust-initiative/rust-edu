use crate::geometry::ray::Ray;
use crate::tools::interval::Interval;
use crate::vector::{Point, Vec3};
use std::cmp::{max, min};
use std::mem::swap;
use std::ops::{BitOr, Range};

// Axis-Aligned Bounding Boxes
// 三维空间的盒子，最小坐标 + 最大坐标 的 矩形
#[derive(Clone, Copy)]
pub struct AABB {
    pub axis: [Interval; 3],
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            axis: Default::default(),
        }
    }
}

impl AABB {
    pub fn new() -> AABB {
        AABB {
            axis: [Interval::new(); 3],
        }
    }

    pub fn from_point(p: &Vec3) -> AABB {
        AABB {
            axis: [
                Interval::new_with_values(p.x(), p.x()),
                Interval::new_with_values(p.y(), p.y()),
                Interval::new_with_values(p.z(), p.z()),
            ],
        }
    }

    pub fn from_points(a: &Vec3, b: &Vec3) -> AABB {
        AABB {
            axis: [
                Interval::new_with_values(a.x().min(b.x()), a.x().max(b.x())),
                Interval::new_with_values(a.y().min(b.y()), a.y().max(b.y())),
                Interval::new_with_values(a.z().min(b.z()), a.z().max(b.z())),
            ],
        }
    }

    pub fn from_boxes(box0: &AABB, box1: &AABB) -> AABB {
        AABB {
            axis: [
                Interval::new_with_intervals(&box0.axis[0], &box1.axis[0]),
                Interval::new_with_intervals(&box0.axis[1], &box1.axis[1]),
                Interval::new_with_intervals(&box0.axis[2], &box1.axis[2]),
            ],
        }
    }

    pub fn from_intervals(ix: &Interval, iy: &Interval, iz: &Interval) -> AABB {
        AABB {
            axis: [ix.clone(), iy.clone(), iz.clone()],
        }
    }

    pub fn min(&self) -> Vec3 {
        Vec3::new(self.axis[0].min, self.axis[1].min, self.axis[2].min)
    }

    pub fn max(&self) -> Vec3 {
        Vec3::new(self.axis[0].max, self.axis[1].max, self.axis[2].max)
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(
            self.axis[1].average(),
            self.axis[2].average(),
            self.axis[3].average(),
        )
    }

    pub fn x(&self) -> &Interval {
        &self.axis[0]
    }

    pub fn y(&self) -> &Interval {
        &self.axis[1]
    }

    pub fn z(&self) -> &Interval {
        &self.axis[2]
    }

    pub fn x_mut(&mut self) -> &mut Interval {
        &mut self.axis[0]
    }

    pub fn y_mut(&mut self) -> &mut Interval {
        &mut self.axis[1]
    }

    pub fn z_mut(&mut self) -> &mut Interval {
        &mut self.axis[2]
    }

    pub fn pad(&self) -> AABB {
        // Return an AABB that has no side narrower than some delta, padding if necessary.
        let delta = 0.0001;
        let new_x = if self.x().size() >= delta {
            self.x().clone()
        } else {
            self.x().expand(delta)
        };
        let new_y = if self.y().size() >= delta {
            self.y().clone()
        } else {
            self.y().expand(delta)
        };
        let new_z = if self.z().size() >= delta {
            self.z().clone()
        } else {
            self.z().expand(delta)
        };

        AABB::from_intervals(&new_x, &new_y, &new_z)
    }

    // old version
    // pub fn hit_old(&self, r: &Ray, ray_t: &mut Interval) -> bool {
    //     // 判断光线 与 AABB 是否相交，判断与三个面的交面，是否有重合
    //     for i in 0..3 {
    //         let t0 = min(
    //             (self.axis[i].min - r.origin()[i]) / r.direction()[i],
    //             (self.axis[i].max - r.origin()[i]) / r.direction()[i],
    //         );
    //         let t1 = max(
    //             (self.axis[i].min - r.origin()[i]) / r.direction()[i],
    //             (self.axis[i].max - r.origin()[i]) / r.direction()[i],
    //         );
    //         ray_t.min = max(t0, ray_t.min);
    //         rayt.max = min(t1, ray_t.max);
    //         if ray_t.max <= ray_t.min {
    //             return false;
    //         }
    //     }
    //     true
    // }

    // Andrew Kensler 优化
    pub fn hit(&self, r: &Ray, mut ray_t: Range<f64>) -> bool {
        for i in 0..3 {
            let inv_d = 1.0 / r.direction()[i];
            let mut t0 = (self.axis[i].min - r.origin()[i]) * inv_d;
            let mut t1 = (self.axis[i].max - r.origin()[i]) * inv_d;
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            if t0 > ray_t.start {
                ray_t.start = t0;
            }
            if t1 < ray_t.end {
                ray_t.end = t1;
            }
            if ray_t.end <= ray_t.start {
                return false;
            }
        }
        true
    }
}

impl BitOr<Self> for &AABB {
    type Output = AABB;

    fn bitor(self, rhs: Self) -> Self::Output {
        let min = Point::new_min(self.min(), rhs.min());
        let max = Point::new_max(self.max(), rhs.max());

        AABB::from_points(&min, &max)
    }
}

impl BitOr<&Self> for AABB {
    type Output = Self;

    fn bitor(self, rhs: &Self) -> Self::Output {
        &self | rhs
    }
}

// 包围和
pub fn surrounding_box(box0: &AABB, box1: &AABB) -> AABB {
    AABB::from_boxes(box0, box1)
}

impl std::ops::Add<Vec3> for AABB {
    type Output = AABB;

    fn add(self, offset: Vec3) -> AABB {
        AABB {
            axis: [
                self.x().clone() + offset.x(),
                self.y().clone() + offset.y(),
                self.z().clone() + offset.z(),
            ],
        }
    }
}

impl std::ops::Add<AABB> for Vec3 {
    type Output = AABB;

    fn add(self, bbox: AABB) -> AABB {
        bbox + self
    }
}
