use crate::Hittable;
use crate::camera::Camera;
use crate::vector::Vec3;
use crate::geometry::{Shape,Sphere};
use crate::materials::Material;
use crate::Color;

use rand::Rng;
use rand::rngs::ThreadRng;


pub fn gen_scene() -> Vec<Hittable> {
    let mut rng = rand::thread_rng();
    let mut scene: Vec<Hittable> = Vec::new();

    let ground_sphere = Sphere::new(Vec3(0.0, -1000.0, 0.0), 1000.0);
    let ground = Hittable{shape: Shape::Sphere(ground_sphere), material: Material::Diffuse{albedo: Color::new(0.5, 0.5, 0.5)}};
    scene.push(ground);

    let big_sphere1 = Sphere::new(Vec3(0.0, 1.0, 0.0), 1.0);
    let glass_sphere = Hittable{shape: Shape::Sphere(big_sphere1), material: Material::Dielectric{refractive_index: 1.5}};
    scene.push(glass_sphere);
    let big_sphere2 = Sphere::new(Vec3(-4.0, 1.0, 0.0), 1.0);
    let matte_sphere = Hittable{shape: Shape::Sphere(big_sphere2), material: Material::Diffuse{albedo: Color::new(0.4, 0.2, 0.1)}};
    scene.push(matte_sphere);
    let big_sphere3 = Sphere::new(Vec3(4.0, 1.0, 0.0), 1.0);
    let metal_sphere = Hittable{shape: Shape::Sphere(big_sphere3), material: Material::Metal{albedo: Color::new(0.7, 0.6, 0.5), fuzz: 0.0}};
    scene.push(metal_sphere);


    for x in -11..11 {
        for z in -11..11 {
            let location = Vec3(x as f64 + 0.9*rng.gen::<f64>(), 0.2, z as f64 + 0.9*rng.gen::<f64>());
            let hittable = gen_hittable(&mut rng, location);
            scene.push(hittable);
        }
    };

    scene
}

pub fn default_camera() -> Camera {
    Camera::build(Vec3(0.0, 0.0, 0.0), Vec3(13.0, 2.0, 3.0), 1.0, 0.1, 512, 512)
}

fn gen_hittable(rng: &mut ThreadRng, location: Vec3) -> Hittable {
    let small_sphere = Sphere::new(location, 0.2);
    let material: Material;

    let probability: f64 = rng.gen();

    if probability < 0.8 {
        let (r,g,b) = rng.gen::<(f64,f64,f64)>();
        let albedo = Color::new(r, g, b);
        material = Material::Diffuse{albedo};
    } else if probability < 0.95 {
        let albedo = Color::new(rng.gen_range(0.5..1.0), rng.gen_range(0.5..1.0), rng.gen_range(0.5..1.0));
        let fuzz = rng.gen_range(0.0..0.5);
        material = Material::Metal{albedo, fuzz};
    } else {
        material = Material::Dielectric{refractive_index: rng.gen_range(1.0..2.0)};
    }

    Hittable{shape: Shape::Sphere(small_sphere), material}
}


