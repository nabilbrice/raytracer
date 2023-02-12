use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};

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
    let randomcolor: Color = Color::new(thread_rng().gen(), thread_rng().gen(), thread_rng().gen());
    let redish: Color = Color::new(0.9, 0.3, 0.3);
    let greenish: Color = Color::new(0.3, 0.9, 0.3);
    let bluish: Color = Color::new(0.3, 0.3, 0.9);
    let yellowish = Color::new(0.8, 0.6, 0.2);
    let material1 = Diffuse{albedo: redish};
    let material2 = Diffuse{albedo: greenish};
    let material3 = Diffuse{albedo: bluish};
    let material4 = Diffuse{albedo: Color::new(0.3, 0.3, 0.3)};
    let metal1 = Metal{albedo: Color::new(0.8, 0.8, 0.8), fuzz: 0.1};
    let metal2 = Metal{albedo: randomcolor, fuzz: 0.3};
    
    let glass1 = Dielectric{refractive_index: 1.5};
    let glass2 = Dielectric{refractive_index: 1.5};

    let sphere1 = Sphere::new(Vec3(0.0, 0.0, 4.0), 0.5, metal2);
    let sphere2 = Sphere::new(Vec3(0.7, -0.25, 0.7), 0.25, metal1);
    let sphere3 = Sphere::new(Vec3(-0.5, 0.0, 0.7), 0.5, glass1);
    let ground_sphere= Sphere::new(Vec3(0.0, -100.5, 1.0), 100.0, material2);

    let inverted_sphere = Sphere::new(Vec3(0.0, 0.0, 2.0), -0.45, glass2);

    let scene = vec![sphere1, ground_sphere, sphere2, sphere3];

    // Camera
    let cam_lookfrom = Vec3(0.0, 1.5, -5.0);
    let aperture_radius = 0.2;

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
    let spp: u32 = 50; // samples per pixel
    // Render
    println!("Starting render...");
    println!("Computing with {} samples", image_height*image_width*spp);
    let timer = Instant::now();
    for j in 0..max_j {
        for i in 0..max_i {
            let mut pixel_color: Color = (0..spp).map(|_| cam.get_focus_loc())
                .map(|focus_loc| {
                    Ray::new(focus_loc, cam.get_sample_loc(i,j) - focus_loc)
                })
                .fold(Color::new(0.0,0.0,0.0), |acc, r| {
                    acc + raytrace(&r, &scene, 10)
                });

            pixel_color = (1.0/(spp as f64)) * pixel_color; // no Div defined for Color
            let color = color_to_ppm(pixel_color);

            writeln!(file, "{} {} {}", color.0, color.1, color.2)
                .expect("Unable to write colors.");


        };
    };
    println!("Render finished in {}s", timer.elapsed().as_secs());
}

fn raytrace(ray: &Ray, scene: &Vec<Sphere>, scatter_depth: u8) -> Color {
    if scatter_depth == 0 {
        return Color::new(0.0,0.0,0.0);
    }

    let (hit_obj, param) = scene.iter()
                        .map(|hittable| {(hittable, hittable.intersect(ray))})
                        .min_by(|x,y| {x.1.total_cmp(&y.1)})
                        .unwrap();

    if param != FARAWAY {
        let scatter_loc: Vec3 = ray.position_at(param);
        let scatter_ray: Ray = hit_obj.material.scatter(ray, hit_obj, scatter_loc);
        return hit_obj.material.albedo() * raytrace(&scatter_ray, scene, scatter_depth - 1)
    }
    // Current calculation for sky color when no intersection is made
    let t = 0.5 * (ray.dir.1 + 1.0);

    (1.0 - t) * Color{r: 1.0, g: 1.0, b: 1.0} + t* Color{r: 0.5, g: 0.7, b: 1.0}

}


fn color_to_ppm(col: Color) -> (u8, u8, u8) {
    ((255.0 * col.r.sqrt()) as u8, (255.0*col.g.sqrt()) as u8, (255.0 * col.b.sqrt()) as u8)
}