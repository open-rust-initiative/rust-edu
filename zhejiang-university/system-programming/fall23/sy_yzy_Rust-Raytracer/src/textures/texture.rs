use std::{io, path::Path};

use crate::vector::vec3::Vec3;
use image::{DynamicImage, GenericImageView};

#[derive(Clone)]
pub enum Texture {
    /*
    固定颜色
    棋盘
    图像
    */
    SolidColor(Vec3),
    Checker { even: Vec3, odd: Vec3, scale: f64 },
    Image(DynamicImage),
}
impl Texture {
    pub fn load_image<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        use image::io::Reader as ImageReader;

        let img = ImageReader::open(path)?.decode().unwrap();

        Ok(Self::Image(img))
    }
    pub fn color(&self, u: f64, v: f64, point: Vec3) -> Vec3 {
        match self {
            Texture::SolidColor(color) => *color,
            Texture::Checker { even, odd, scale } => {
                let x_integer = (scale.recip() * point.x()).floor() as i32;
                let y_integer = (scale.recip() * point.y()).floor() as i32;
                let z_integer = (scale.recip() * point.z()).floor() as i32;

                let is_even = (x_integer + y_integer + z_integer) % 2 == 0;

                if is_even {
                    *even
                } else {
                    *odd
                }
            }
            Texture::Image(image) => {
                // 读取贴图的信息
                if image.height() <= 0 {
                    return Vec3::new(0., 1., 1.);
                }
                let u = u.clamp(0.0, 1.0);
                let v = 1.0 - v.clamp(0.0, 1.0); // Flip V to image coordinates

                let i: u32 = (u * image.width() as f64) as u32;
                let j: u32 = (v * image.height() as f64) as u32;

                let pixel = image.get_pixel(i, j);

                let color_scale = 1.0 / 255.0;
                return Vec3::new(
                    color_scale * pixel[0] as f64,
                    color_scale * pixel[1] as f64,
                    color_scale * pixel[2] as f64,
                );
            }
        }
    }
}

impl From<Vec3> for Texture {
    fn from(value: Vec3) -> Self {
        Self::SolidColor(value)
    }
}
