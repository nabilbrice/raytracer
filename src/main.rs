use std::fs::OpenOptions;
use std::io::prelude::*;
use std::collections::HashMap;
use std::cmp::min;
use rand::{Rng, random};
use std::process::abort;

mod vector; //call a local module into this one with ; instead of {}
use crate::vector::Vec3; // use the specific name here

mod ray;
use crate::ray::Ray;

mod geometry;
use crate::geometry::{Sphere, FARAWAY};

fn main() {

    // Image
    let image_width: u32 = 256;
    let image_height: u32 = 256;

    // Scene
    let sphere1 = Sphere::new(Vec3(0.0, 0.0, 1.0), 0.5);
    let sphere2 = Sphere::new(Vec3(0.0, 1.0, 1.0), 1.0);
    let ground_sphere= Sphere::new(Vec3(0.0, -100.5, 1.0), 100.0);

    let scene = vec![sphere1, ground_sphere];

    // Camera
    let viewport_height = 2.0;
    let viewport_width = 2.0;

    let cam_origin = Vec3(0.0, 0.5, 0.0);
    let horizontal = Vec3(viewport_width, 0.0, 0.0);
    let vertical = Vec3(0.0, viewport_height, 0.0);

    // Camera_TEST
    let cam = Camera::new(cam_origin, horizontal, vertical,
    1.0, image_width, image_height);

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
    // Render
    for j in 0..max_j {
        for i in 0..max_i {
            let mut pixel_color = Vec3(0.0, 0.0, 0.0);
            for iter in 1..10 {
                let sample_position = cam.get_sample_loc(i,j);
                let ray_direction = sample_position - cam.eye_loc;

                let r = Ray::new(sample_position, ray_direction);
                pixel_color += raytrace(&r, &scene, 20)
            }
            pixel_color = pixel_color/10.0;
            let color = vec3_to_rgb(&pixel_color);

            writeln!(file, "{} {} {}", color.0, color.1, color.2)
                .expect("Unable to write colors.");

        };
    };
}

fn raytrace(ray: &Ray, scene: &Vec<Sphere>, scatter_depth: u8) -> Vec3 {
    let mut ray_color = Vec3(0.0, 0.0, 0.0);
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
    if param != FARAWAY && param > 1.0e-8 {
        let hit_obj = hit_rec.unwrap();
        // let surface_normal = hittable.normal_at(ray.position_at(param));
        // return 0.5 * surface_normal + Vec3(0.5, 0.5, 0.5);
        let scatter_loc: Vec3 = ray.position_at(param);
        let scatter_dir = hit_obj.normal_at(scatter_loc)
                                + random_vec3();
        let scatter_ray: Ray = Ray::new(scatter_loc, scatter_dir);
        let scatter_color = raytrace(&scatter_ray, scene, scatter_depth - 1);

        ray_color = 0.5 * scatter_color;
        return ray_color;
    }
    else {

        // Current calculation for sky color when no intersection is made
        let t = 0.5 * (ray.dir.1 + 1.0);

        ray_color += (1.0 - t) * Vec3(1.0, 1.0, 1.0) + t * Vec3(0.5, 0.7, 1.0);

        return ray_color;
    }
    
}

struct Camera {
    position: Vec3,
    horiz_arm: Vec3,
    vert_arm: Vec3,
    direction: Vec3,
    eye_loc: Vec3,
    horiz_res: u32,
    vert_res: u32,
    aspect_ratio: f64,
}

impl Camera {
    fn new(position: Vec3, horiz_arm: Vec3, vert_arm: Vec3,
        focal_length: f64, horiz_res: u32, vert_res: u32) -> Camera {
            let direction = horiz_arm.cross(&vert_arm).normalize();
            let eye_loc: Vec3 = position - (focal_length * direction);
            let aspect_ratio: f64 = horiz_arm.norm()/vert_arm.norm();
            Camera {
                position, horiz_arm, vert_arm,
                direction, eye_loc, horiz_res, vert_res, aspect_ratio
            }

    }

    fn get_sample_loc(&self, i: u32, j:u32) -> Vec3 {
        let mut rng = rand::thread_rng();
        let h_rng: f64 = rng.gen();
        let v_rng: f64 = rng.gen();

        let horiz_increm = 1.0/f64::from(self.horiz_res);
        let vert_increm = 1.0/f64::from(self.vert_res);
        let horiz_nudge: Vec3 = (h_rng * horiz_increm) * self.horiz_arm;
        let vert_nudge: Vec3 = (v_rng * vert_increm) * self.vert_arm;

        let horiz_span = self.horiz_arm;
        let vert_span = self.vert_arm;

        let grid_h_offset = -0.5 + f64::from(i)*horiz_increm;
        let grid_v_offset = 0.5 - f64::from(j)*vert_increm;

        self.position + (grid_h_offset * horiz_span) + (grid_v_offset * vert_span) 
        + horiz_nudge + vert_nudge
    }

}

fn vec3_to_rgb(vec: &Vec3) -> (u8, u8, u8) {
    ((255.0*vec.0) as u8, (255.0*vec.1) as u8, (255.0*vec.2) as u8)
}

fn random_vec3() -> Vec3 {
    let mut rng = rand::thread_rng();
    let v0: f64 = rng.gen();
    let v1: f64 = rng.gen();
    let v2: f64 = rng.gen();
    let rand_vec3 = 2.0 * Vec3(v0 - 0.5, v1 - 0.5, v2 - 0.5);
    if rand_vec3.norm() > 1.0 {
        return random_vec3();
    };
    return rand_vec3;
}