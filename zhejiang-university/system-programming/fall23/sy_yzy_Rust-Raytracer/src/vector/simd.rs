// use std::arch::x86_64::*;

// #[repr(align(32))]
// #[derive(Debug, Clone, Copy)]
// pub union F64x4 {
//     pub simd: [f64; 4],
//     pub simd_pd: __m256d,
// }

// impl F64x4 {
//     pub fn new(simd_pd: __m256d) -> Self {
//         Self {
//             simd_pd: simd_pd,
//         }
//     }

//     pub fn splat(value: f64) -> Self {
//         Self {
//             simd_pd: unsafe { _mm256_set1_pd(value) },
//         }
//     }

//     pub fn load_unaligned(data: &[f64; 4]) -> Self {
//         Self {
//             simd_pd: unsafe { _mm256_loadu_pd(data.as_ptr()) },
//         }
//     }

//     pub fn store_unaligned(self, data: &mut [f64; 4]) {
//         unsafe {
//             _mm256_storeu_pd(data.as_mut_ptr(), self.simd_pd);
//         }
//     }
// }

// #[derive(Debug, Clone, Copy)]
// pub struct Vec3d {
//     pub e: F64x4,
// }

// impl Vec3d {
//     pub fn new() -> Self {
//         Self {
//             e: F64x4::new(unsafe { _mm256_setzero_pd() }),
//         }
//     }

//     pub fn splat(value: f64) -> Self {
//         Self {
//             e: F64x4::splat(value),
//         }
//     }

//     pub fn splat_zero() -> Self {
//         Self::splat(0.0)
//     }

//     pub fn splat_one() -> Self {
//         Self::splat(1.0)
//     }

//     pub fn from_simd(simd: __m256d) -> Self {
//         Self {
//             e: F64x4::new(simd),
//         }
//     }

//     pub fn from_f64(x: f64) -> Self {
//         Self {
//             e: F64x4::new(unsafe { _mm256_set_pd(x, x, x, 0.0) }),
//         }
//     }

//     pub fn from_f64_xyz(x: f64, y: f64, z: f64) -> Self {
//         Self {
//             e: F64x4::new(unsafe { _mm256_set_pd(x, y, z, 0.0) }),
//         }
//     }

//     pub fn unary_minus(&self) -> Self {
//         Self {
//             e: F64x4::new(unsafe { _mm256_sub_pd(_mm256_setzero_pd(), self.e.simd_pd) }),
//         }
//     }

//     pub fn get(&self, i: usize) -> f64 {
//         if i < 3 {
//             self.e.simd[i]
//         } else {
//             0.0
//         }
//     }

//     pub fn x(&self) -> f64 {
//         self.e.simd[0]
//     }

//     pub fn y(&self) -> f64 {
//         self.e.simd[1]
//     }

//     pub fn z(&self) -> f64 {
//         self.e.simd[2]
//     }

//     pub fn sum3(&self) -> f64 {
//         self.x() + self.y() + self.z()
//     }

//     pub fn multiply(&self, scalar: f64) -> Self {
//         Self {
//             e: F64x4::new(unsafe { _mm256_mul_pd(self.e.simd_pd, _mm256_set1_pd(scalar)) }),
//         }
//     }

//     pub fn divide(&self, scalar: f64) -> Self {
//         Self {
//             e: F64x4::new(unsafe { _mm256_div_pd(self.e.simd_pd, _mm256_set1_pd(scalar)) }),
//         }
//     }

//     pub fn dot(&self, other: &Self) -> f64 {
//         let product = Self {
//             e: F64x4::new(unsafe { _mm256_mul_pd(self.e.simd_pd, other.e.simd_pd) }),
//         };
//         product.sum3()
//     }

//     pub fn subtract(&self, other: &Self) -> Self {
//         Self {
//             e: F64x4::new(unsafe { _mm256_sub_pd(self.e.simd_pd, other.e.simd_pd) }),
//         }
//     }

//     pub fn add(&self, other: &Self) -> Self {
//         Self {
//             e: F64x4::new(unsafe { _mm256_add_pd(self.e.simd_pd, other.e.simd_pd) }),
//         }
//     }

//     pub fn add_assign(&mut self, other: &Self) {
//         self.e.simd_pd = unsafe { _mm256_add_pd(self.e.simd_pd, other.e.simd_pd) };
//     }

//     pub fn multiply_assign(&mut self, other: &Self) {
//         self.e.simd_pd = unsafe { _mm256_mul_pd(self.e.simd_pd, other.e.simd_pd) };
//     }

//     pub fn divide_assign(&mut self, other: &Self) {
//         self.e.simd_pd = unsafe { _mm256_div_pd(self.e.simd_pd, other.e.simd_pd) };
//     }

//     pub fn sqrt(&self) -> Self {
//         Self {
//             e: F64x4::new(unsafe { _mm256_sqrt_pd(self.e.simd_pd) }),
//         }
//     }

//     pub fn normalize(&self) -> Self {
//         let length = self.length();
//         self.divide(length)
//     }

//     pub fn length_squared(&self) -> f64 {
//         let product = Self {
//             e: F64x4::new(unsafe { _mm256_mul_pd(self.e.simd_pd, self.e.simd_pd) }),
//         };
//         product.sum3()
//     }

//     pub fn length(&self) -> f64 {
//         self.length_squared().sqrt()
//     }

//     pub fn cross(&self, other: &Self) -> Self {
//         let a_yzx = Self {
//             e: F64x4::new(unsafe { _mm256_permute_pd(self.e.simd_pd, 0b0101) }),
//         };
//         let b_zxy = Self {
//             e: F64x4::new(unsafe { _mm256_permute_pd(other.e.simd_pd, 0b1001) }),
//         };
//         let a_zxy = Self {
//             e: F64x4::new(unsafe { _mm256_permute_pd(self.e.simd_pd, 0b1001) }),
//         };
//         let b_yzx = Self {
//             e: F64x4::new(unsafe { _mm256_permute_pd(other.e.simd_pd, 0b0101) }),
//         };

//         let c = a_yzx.multiply(b_zxy).subtract(&a_zxy.multiply(b_yzx));
//         c
//     }
// }