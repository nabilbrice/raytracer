use std::fs::OpenOptions;
use std::io::prelude::*;
use rand::{Rng, thread_rng};

mod vector; //call a local module into this one with ; instead of {}
use crate::vector::Vec3; // use the specific name here

mod color;
use crate::color::Color;

mod ray;
use crate::ray::Ray;

mod geometry;
use crate::geometry::{Sphere, FARAWAY};

fn main() {

    // Image
    let image_width: u32 = 512;
    let image_height: u32 = 512;

    // Scene
    let sphere1 = Sphere::new(Vec3(0.0, 0.0, 2.0), 0.5);
    let sphere2 = Sphere::new(Vec3(0.7, -0.25, 0.7), 0.25);
    let sphere3 = Sphere::new(Vec3(-0.7, 0.0, 0.7), 0.5);
    let ground_sphere= Sphere::new(Vec3(0.0, -100.5, 1.0), 100.0);

    let scene = vec![sphere1, ground_sphere, sphere2, sphere3];

    // Camera
    let viewport_height = 2.0;
    let viewport_width = 2.0;

    let cam_origin = Vec3(0.0, 0.5, 0.0);
    let horizontal = Vec3(viewport_width, 0.0, 0.0);
    let vertical = Vec3(0.0, viewport_height, 0.0);

    // Camera_TEST
    let cam = Camera::new(cam_origin, horizontal, vertical,
    10.0, image_width, image_height);

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
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            for iter in 1..10 {
                let sample_position = cam.get_sample_loc(i,j);
                let ray_direction = sample_position - cam.eye_loc;

                let r = Ray::new(sample_position, ray_direction);
                pixel_color += raytrace(&r, &scene, 40)
            }
            pixel_color = (1.0/10.0) * pixel_color; // no Div defined for Color
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
        // Attenuation
        let scatter_dir = hit_obj.normal_at(scatter_loc)
                                + random_vec3();
        // Scattered Ray is generated
        let scatter_ray: Ray = Ray::new(scatter_loc, scatter_dir);
        let albedo = Color::new(0.8, 0.8, 0.1);
        let scatter_color: Color = albedo * raytrace(&scatter_ray, scene, scatter_depth - 1);

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
        let rng_scalars: [f64; 2] = thread_rng().gen();

        let horiz_increm = 1.0/f64::from(self.horiz_res);
        let vert_increm = 1.0/f64::from(self.vert_res);
        let horiz_nudge: Vec3 = (rng_scalars[0] * horiz_increm) * self.horiz_arm;
        let vert_nudge: Vec3 = (rng_scalars[1] * vert_increm) * self.vert_arm;

        let horiz_span = self.horiz_arm;
        let vert_span = self.vert_arm;

        let grid_h_offset = -0.5 + f64::from(i)*horiz_increm;
        let grid_v_offset = 0.5 - f64::from(j)*vert_increm;

        self.position + (grid_h_offset * horiz_span) + (grid_v_offset * vert_span) 
        + horiz_nudge + vert_nudge
    }

}

fn color_to_ppm(col: Color) -> (u8, u8, u8) {
    ((255.0 * col.r) as u8, (255.0*col.g) as u8, (255.0 * col.b) as u8)
}

fn random_vec3() -> Vec3 {
    let v: (f64, f64, f64) = thread_rng().gen();
    let rand_vec3 = 2.0 * Vec3(v.0 - 0.5, v.1 - 0.5, v.2 - 0.5);
    if rand_vec3.norm() > 1.0 {
        return random_vec3();
    };
    return rand_vec3.normalize();
}