use rustRaytracer::{
    camera::Camera,
    geometry::{Hittable, HittableList, Sphere},
    materials::Material,
    renderer::Renderer,
    vector::{Point, Vec3},
};
use std::io;

fn main() -> io::Result<()> {
    let mut world: Vec<Box<dyn Hittable>> = vec![];

    let material_ground = Material::Lambertian {
        albedo: Vec3::new(0.8, 0.8, 0.0).into(),
    };
    let material_center = Material::Lambertian {
        albedo: Vec3::new(0.1, 0.2, 0.5).into(),
    };
    let material_left = Material::Dielectric {
        index_of_refraction: 1.5,
    };
    let material_right = Material::Metal {
        albedo: Vec3::new(0.8, 0.6, 0.2),
        fuzz: 0.0,
    };

    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, 0.0, -1.0),
        0.5,
        material_center,
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left.clone(),
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        -0.4,
        material_left,
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    )));

    let lookfrom = Point::new(-2.0, 2.0, 1.0);
    let lookat = Point::new(0.0, 0.0, -1.0);
    let vup = Point::new(0.0, 1.0, 0.0);
    let vof = 20.0;
    let aspect_ratio = 16.0 / 9.0;
    let aperture = 0.0;

    let hitable_list = HittableList::new(world);
    let cam = Camera::new(lookfrom, lookat, vup, vof, aspect_ratio, aperture);
    let mut renderer = Renderer::new(cam, hitable_list.into());
    renderer.set_photo_name("Sphere3.bmp".to_owned());
    renderer.set_max_depth(20);
    renderer.set_samples_per_pixel(400);
    renderer.render();
    Ok(())
}
