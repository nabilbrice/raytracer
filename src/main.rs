use std::fs::OpenOptions;
use std::io::prelude::*;

mod vector; //call a local module into this one with ; instead of {}
use crate::vector::Vec3; // use the specific name here

mod color;
use crate::color::Color;

mod ray;
use crate::ray::Ray;

mod geometry;
use crate::geometry::{Sphere, FARAWAY};

mod materials;
use crate::materials::Material::{Diffuse, Metal, Dielectric};

mod camera;
use crate::camera::Camera;

fn main() {

    // Image
    let image_width: u32 = 512;
    let image_height: u32 = 512;

    // Scene
    let redish: Color = Color::new(0.9, 0.3, 0.3);
    let greenish: Color = Color::new(0.3, 0.9, 0.3);
    let bluish: Color = Color::new(0.3, 0.3, 0.9);
    let yellowish = Color::new(0.8, 0.6, 0.2);
    let material1 = Diffuse{albedo: redish};
    let material2 = Diffuse{albedo: greenish};
    let material3 = Diffuse{albedo: bluish};
    let material4 = Diffuse{albedo: Color::new(0.3, 0.3, 0.3)};
    let metal1 = Metal{albedo: Color::new(0.8, 0.8, 0.8), fuzz: 0.1};
    let metal2 = Metal{albedo: yellowish, fuzz: 0.3};
    
    let glass1 = Dielectric{refractive_index: 2.5};
    let glass2 = Dielectric{refractive_index: 2.5};

    let sphere1 = Sphere::new(Vec3(0.0, 0.0, 4.0), 0.5, glass1);
    let sphere2 = Sphere::new(Vec3(0.7, -0.25, 0.7), 0.25, metal1);
    let sphere3 = Sphere::new(Vec3(-0.5, 0.0, 0.7), 0.5, metal2);
    let ground_sphere= Sphere::new(Vec3(0.0, -100.5, 1.0), 100.0, material2);

    let inverted_sphere = Sphere::new(Vec3(0.0, 0.0, 2.0), -0.45, glass2);

    let scene = vec![sphere1, ground_sphere, sphere2, sphere3];

    // Camera
    let cam_lookfrom = Vec3(0.0, 1.5, -5.0);
    let aperture_radius = 0.3;

    // Camera_TEST
    let cam = Camera::new(Vec3(0.0, 0.0, 1.0), cam_lookfrom,
                1.0, aperture_radius, image_width, image_height);

    // Write header to file
    let header = format!("P3\n{} {}\n255\n",&image_width,&image_height);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("./image.ppm")
        .expect("Unable to open file");

    write!(file, "{}", header)
        .expect("Unable to write header to ppm");

    let max_j = image_height;
    let max_i = image_width;
    let spp: i32 = 50; // samples per pixel
    // Render
    for j in 0..max_j {
        for i in 0..max_i {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            for _iter in 1..spp {
                let sample_position = cam.get_sample_loc(i,j);
                let lens_sample_loc = cam.get_focus_loc();
                let ray_direction = sample_position - lens_sample_loc;

                let r = Ray::new(lens_sample_loc, ray_direction);
                pixel_color += raytrace(&r, &scene, 10)
            }
            pixel_color = (1.0/(spp as f64)) * pixel_color; // no Div defined for Color
            let color = color_to_ppm(pixel_color);

            writeln!(file, "{} {} {}", color.0, color.1, color.2)
                .expect("Unable to write colors.");


        };
        print!("{} % \r", (100.0 * f64::from(j) / f64::from(max_j)) as u32);
    };
}

fn raytrace(ray: &Ray, scene: &Vec<Sphere>, scatter_depth: u8) -> Color {
    let mut ray_color: Color = Color::new(0.0, 0.0, 0.0);
    if scatter_depth == 0 {
        return ray_color;
    }
    let mut param = FARAWAY;
    let mut hit_rec = None;
    for hittable in scene {
        let test_param = hittable.intersect(ray);

        if test_param < param {
            param = test_param;
            hit_rec = Some(hittable);
        }
    }
    if param != FARAWAY && param > 1.0e-6 {
        let hit_obj = hit_rec.expect("hit object is None!");
        // let surface_normal = hittable.normal_at(ray.position_at(param));
        // return 0.5 * surface_normal + Vec3(0.5, 0.5, 0.5);
        let scatter_loc: Vec3 = ray.position_at(param);
        // Scattered Ray is generated, currently diffuse material hardcoded
        let scatter_ray: Ray = hit_obj.material.scatter(&ray, hit_obj, scatter_loc);

        let scatter_color: Color = hit_obj.material.albedo() * raytrace(&scatter_ray, scene, scatter_depth - 1);

        return scatter_color;
    }
    else {

        // Current calculation for sky color when no intersection is made
        let t = 0.5 * (ray.dir.1 + 1.0);

        ray_color += (1.0 - t) * Color{r: 1.0, g: 1.0, b: 1.0}
                    + t* Color{r: 0.5, g: 0.7, b: 1.0};

        return ray_color;
    }
    
}


fn color_to_ppm(col: Color) -> (u8, u8, u8) {
    ((255.0 * col.r) as u8, (255.0*col.g) as u8, (255.0 * col.b) as u8)
}