use crate::boundingvolume::{make_coveringtree, BoundingBox, Cover, CoveringTree};
use crate::camera::Camera;
use crate::geometry::{Shape, Sphere};
use crate::materials::Material;
use crate::vector::Vec3;
use crate::Color;
use crate::Hittable;

use rand::rngs::ThreadRng;
use rand::Rng;

pub fn gen_scene() -> Box<CoveringTree> {
    let mut rng = rand::thread_rng();
    let mut scene: Vec<BoundingBox> = Vec::new();

    let ground_sphere = Sphere::new(Vec3([0.0, -1000.0, 0.0]), 1000.0);
    let ground = Hittable {
        shape: Shape::Sphere(ground_sphere),
        material: Material::Diffuse {
            albedo: Color::new(0.5, 0.5, 0.5),
        },
    };
    scene.push(ground.make_covering());

    let big_sphere1 = Sphere::new(Vec3([0.0, 1.0, 0.0]), 1.0);
    let glass_sphere = Hittable {
        shape: Shape::Sphere(big_sphere1),
        material: Material::Dielectric {
            refractive_index: 1.5,
        },
    };
    scene.push(glass_sphere.make_covering());
    let big_sphere2 = Sphere::new(Vec3([-4.0, 1.0, 0.0]), 1.0);
    let matte_sphere = Hittable {
        shape: Shape::Sphere(big_sphere2),
        material: Material::Diffuse {
            albedo: Color::new(0.4, 0.2, 0.1),
        },
    };
    scene.push(matte_sphere.make_covering());
    let big_sphere3 = Sphere::new(Vec3([4.0, 1.0, 0.0]), 1.0);
    let metal_sphere = Hittable {
        shape: Shape::Sphere(big_sphere3),
        material: Material::Metal {
            albedo: Color::new(0.7, 0.6, 0.5),
            fuzz: 0.0,
        },
    };
    scene.push(metal_sphere.make_covering());

    for x in -11..11 {
        for z in -11..11 {
            let location = Vec3([
                x as f64 + 0.9 * rng.gen::<f64>(),
                0.2,
                z as f64 + 0.9 * rng.gen::<f64>(),
            ]);
            let hittable = gen_hittable(&mut rng, location);
            scene.push(hittable.make_covering());
        }
    }

    let mut bboxed = scene.into_boxed_slice();

    println!("number of BoundingBox: {}", bboxed.len());

    make_coveringtree(&mut bboxed)
}

pub fn default_camera() -> Camera {
    Camera::build(
        Vec3([0.0, 0.0, 0.0]),
        Vec3([13.0, 1.5, 3.0]),
        1.0,
        0.1,
        512,
        512,
    )
}

fn gen_hittable(rng: &mut ThreadRng, location: Vec3) -> Hittable {
    let small_sphere = Sphere::new(location, 0.2);
    let material: Material;

    let probability: f64 = rng.gen();

    if probability < 0.8 {
        let (r, g, b) = rng.gen::<(f64, f64, f64)>();
        let albedo = Color::new(r, g, b);
        material = Material::Diffuse { albedo };
    } else if probability < 0.95 {
        let albedo = Color::new(
            rng.gen_range(0.5..1.0),
            rng.gen_range(0.5..1.0),
            rng.gen_range(0.5..1.0),
        );
        let fuzz = rng.gen_range(0.0..0.5);
        material = Material::Metal { albedo, fuzz };
    } else {
        material = Material::Dielectric {
            refractive_index: rng.gen_range(1.0..2.0),
        };
    }

    Hittable {
        shape: Shape::Sphere(small_sphere),
        material,
    }
}

pub fn debug_scene() -> Box<CoveringTree> {
    let mut scene: Vec<BoundingBox> = Vec::new();
    let big_sphere2 = Sphere::new(Vec3([0.0, 0.0, 0.0]), 5.0);
    let matte_sphere = Hittable {
        shape: Shape::Sphere(big_sphere2),
        material: Material::Diffuse {
            albedo: Color::new(0.4, 0.2, 0.1),
        },
    };
    scene.push(matte_sphere.make_covering());

    let mut bboxed = scene.into_boxed_slice();

    println!("number of BoundingBox: {}", bboxed.len());

    make_coveringtree(&mut bboxed)
}

pub fn debug_camera() -> Camera {
    Camera::build(
        Vec3([0.0, 0.0, 0.0]),
        Vec3([10.0, 0.0, 0.0]),
        1.0,
        0.1,
        512,
        512,
    )
}
