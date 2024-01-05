use rand::prelude::*;
use rustRaytracer::{
    camera::Camera,
    geometry::{Hittable, HittableList, Sphere},
    materials::Material::{self},
    renderer::Renderer,
    textures::Texture,
    vector::{Color, Point, Vec3},
};
use std::io;

fn main() -> io::Result<()> {
    let mut Box: Vec<Box<dyn Hittable>> = vec![];

    let ground_material = Material::Lambertian {
        albedo: Texture::Checker {
            even: Vec3::new(1.0, 1.0, 1.0),
            odd: Vec3::new(0.0, 0.0, 0.0),
            scale: 1.0,
        },
    };
    Box.push(Box::new(Sphere::new(
        Point::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material.clone(),
    )));

    let mut rng = rand::thread_rng();

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat: f64 = rng.gen();

            let center = Point::new(
                a as f64 + 0.9 * rng.gen::<f64>(),
                0.2,
                b as f64 + 0.9 * rng.gen::<f64>(),
            );

            if (center - Point::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Material::Lambertian {
                        albedo: albedo.into(),
                    };
                    Box.push(Box::new(Sphere::new(center, 0.2, sphere_material.clone())));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::random_init(0.5, 1.0);
                    let fuzz = rng.gen_range(0.0, 0.5);
                    let sphere_material = Material::Metal { albedo, fuzz };
                    Box.push(Box::new(Sphere::new(center, 0.2, sphere_material.clone())));
                } else {
                    // glass
                    let sphere_material = Material::Dielectric {
                        index_of_refraction: 1.5,
                    };
                    Box.push(Box::new(Sphere::new(center, 0.2, sphere_material.clone())));
                }
            }
        }
    }

    let material1 = Material::Dielectric {
        index_of_refraction: 1.5,
    };
    Box.push(Box::new(Sphere::new(
        Point::new(0.0, 1.0, 0.0),
        1.0,
        material1.clone(),
    )));

    let material2 = Material::Lambertian {
        albedo: Color::new(0.4, 0.2, 0.1).into(),
    };
    Box.push(Box::new(Sphere::new(
        Point::new(-4.0, 1.0, 0.0),
        1.0,
        material2.clone(),
    )));

    let material3 = Material::Metal {
        albedo: Color::new(0.7, 0.6, 0.5),
        fuzz: 0.0,
    };
    Box.push(Box::new(Sphere::new(
        Point::new(4.0, 1.0, 0.0),
        1.0,
        material3.clone(),
    )));

    let lookfrom = Point::new(13.0, 4.0, 3.0);
    let lookat = Point::new(0.0, 0.0, 0.0);
    let vup = Point::new(0.0, 1.0, 0.0);
    let vof = 20.0;
    let aspect_ratio = 16.0 / 9.0;
    let aperture = 0.1;

    let hitable_list = HittableList::new(Box);
    let cam = Camera::new(lookfrom, lookat, vup, vof, aspect_ratio, aperture);
    let mut renderer = Renderer::new(cam, hitable_list.into());
    renderer.set_photo_name("Book1_2.bmp".to_owned());
    renderer.set_width_and_ratio(1200, aspect_ratio);
    renderer.set_samples_per_pixel(1024);
    renderer.set_max_depth(5);
    renderer.render();

    Ok(())
}
