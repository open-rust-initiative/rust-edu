use core::fmt;
use rand::{thread_rng, Rng};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use std::ops::Range;

use crate::{
    camera::Camera,
    geometry::{hittable::Hittable, hittable_list::HittableList, ray::Ray},
    global::clamp,
    vector::{Color, Vec3},
};

pub struct Renderer {
    photoname: String,
    cam: Camera,
    world: HittableList,
    aspect_ratio: f64,
    image_width: u32,
    image_height: u32,
    samples_per_pixel: u32,
    max_depth: u32,
    background: Color,
}

impl fmt::Display for Renderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.photoname, self.image_height)
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            photoname: Default::default(),
            cam: Default::default(),
            world: Default::default(),
            aspect_ratio: Default::default(),
            image_width: Default::default(),
            image_height: Default::default(),
            samples_per_pixel: Default::default(),
            max_depth: Default::default(),
            background: Default::default(),
        }
    }
}

impl Renderer {
    pub fn new(c: Camera, hitlist: HittableList) -> Self {
        Renderer {
            photoname: "Img.bmp".to_string(),
            cam: c,
            world: hitlist,
            aspect_ratio: 16.0 / 9.0,
            image_width: 1200,
            image_height: (1200.0 / (16.0 / 9.0)) as u32,
            samples_per_pixel: 64,
            max_depth: 10,
            background: Color::default(),
        }
    }

    pub fn set_camera(&mut self, c: Camera) {
        self.cam = c;
    }

    pub fn set_photo_name(&mut self, name: String) {
        self.photoname = name;
    }

    pub fn set_samples_per_pixel(&mut self, samples: u32) {
        self.samples_per_pixel = samples;
    }

    pub fn set_max_depth(&mut self, depth: u32) {
        self.max_depth = depth;
    }

    pub fn set_background(&mut self, c: Color) {
        self.background = c;
    }

    pub fn set_width_and_ratio(&mut self, width: u32, ratio: f64) {
        self.image_width = width;
        self.aspect_ratio = ratio;
        self.image_height = (width as f64 / ratio) as u32;
    }

    pub fn set_image_size(&mut self, width: u32, height: u32) {
        self.image_width = width;
        self.image_height = height;
        self.aspect_ratio = width as f64 / height as f64;
    }

    pub fn render(self) {
        let mut buffer: Vec<u8> = vec![0; (self.image_width * self.image_height * 3) as usize];

        buffer
            .par_chunks_mut(3)
            .enumerate()
            .for_each(|(idx, pixel)| {
                let x = idx as u32 % self.image_width;
                let y = self.image_height - idx as u32 / self.image_width;

                // 计算像素的颜色
                let mut pixel_color = self.simple_random_sampling(x, y);
                Self::set_rgb(&mut pixel_color, self.samples_per_pixel, pixel);
            });

        image::save_buffer(
            self.photoname,
            &buffer,
            self.image_width,
            self.image_height,
            image::ColorType::Rgb8,
        )
        .unwrap();
    }
    fn simple_random_sampling(&self, i: u32, j: u32) -> Color {
        let mut rng = thread_rng();
        let mut pixel_color = Color::default();
        for _ in 0..self.samples_per_pixel {
            let u = (i as f64 + rng.gen::<f64>()) / (self.image_width - 1) as f64;
            let v = (j as f64 + rng.gen::<f64>()) / (self.image_height - 1) as f64;
            let ray = self.cam.get_ray(u, v);
            pixel_color = pixel_color + self.ray_color(&ray, self.max_depth);
        }

        pixel_color
    }
    fn ray_color(&self, ray: &Ray, depth: u32) -> Color {
        if depth == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }
        if let Some(hit_record) = self.world.hit(
            ray,
            Range {
                start: 0.001,
                end: std::f64::INFINITY,
            },
        ) {
            let scatter_result = hit_record.material.scatter(ray, &hit_record);
            let emit = hit_record
                .material
                .emitted(hit_record.u, hit_record.v, hit_record.point);
            match scatter_result {
                Some(res) => {
                    return emit + res.attenuation * self.ray_color(&res.scattered, depth - 1);
                }
                None => {
                    return Vec3::zero();
                }
            }
        }

        let unit_direction = ray.dir.unit_vector_self();
        let a = 0.5 * (unit_direction.y() + 1.0);
        return (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0);
    }

    fn set_rgb(pixel_color: &mut Color, samples_per_pixel: u32, pixel: &mut [u8]) {
        let scale = 1.0 / samples_per_pixel as f64;
        pixel_color[0] = (pixel_color[0] * scale).sqrt();
        pixel_color[1] = (pixel_color[1] * scale).sqrt();
        pixel_color[2] = (pixel_color[2] * scale).sqrt();

        // Write the translated [0,255] value of each color component.
        pixel[0] = (256.0 * clamp(pixel_color.x(), 0.0, 0.99999)) as u8;
        pixel[1] = (256.0 * clamp(pixel_color.y(), 0.0, 0.9999)) as u8;
        pixel[2] = (256.0 * clamp(pixel_color.z(), 0.0, 0.9999)) as u8;
    }
}
