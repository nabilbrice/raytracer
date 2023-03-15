pub mod vector;
pub mod color;
pub mod ray;
pub mod geometry;
pub mod materials;
pub mod camera;
pub mod config;

use std::fs::File;
use std::io::Write;

use vector::Vec3;
use ray::Ray;
use color::Color;
use geometry::{Sphere, FARAWAY};


pub fn raytrace(ray: &Ray, scene: &Vec<Sphere>, scatter_depth: u8) -> Color {
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
        let obj_relative_loc = (scatter_loc - hit_obj.orig).normalize();
        return hit_obj.material.albedo(&obj_relative_loc) * raytrace(&scatter_ray, scene, scatter_depth - 1)
    }
    // Current calculation for sky color when no intersection is made
    let t = 0.5 * (ray.dir.1 + 1.0);

    (1.0 - t) * Color{r: 1.0, g: 1.0, b: 1.0} + t* Color{r: 0.5, g: 0.7, b: 1.0}

}

pub fn render_into_file(file: &mut File, cam: &camera::Camera, scene: &Vec<Sphere>, spp: u32) {
    for j in 0..cam.vert_res {
        for i in 0..cam.horiz_res {
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
        eprint!("\rScanlines remaining: {}", cam.vert_res - j);
    };
    eprintln!("");
}

pub fn color_to_ppm(col: Color) -> (u8, u8, u8) {
    ((255.0 * col.r.sqrt()) as u8, (255.0*col.g.sqrt()) as u8, (255.0 * col.b.sqrt()) as u8)
}

pub fn rgba_to_color(rgba: image::Rgba<u8>) -> Color {
    Color::new((rgba[0] as f64) / 255.0, (rgba[1] as f64) / 255.0, (rgba[2] as f64) / 255.0)
}