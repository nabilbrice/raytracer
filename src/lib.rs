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
use geometry::{Shape, FARAWAY};
use materials::Material;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Hittable {
    shape: Shape,
    material: Material,
}


pub fn raytrace(ray: &Ray, scene: &Vec<Hittable>, scatter_depth: u8) -> Color {
    if scatter_depth == 0 {
        return Color::new([0.0;color::NUMBER_OF_BINS]);
    }

    let (hit_obj, param) = scene.iter()
                        .map(|hittable| {(hittable, hittable.shape.intersect(ray))})
                        .min_by(|x,y| {x.1.total_cmp(&y.1)})
                        .unwrap();


        if param != FARAWAY {
            let scatter_loc: Vec3 = ray.position_at(param);
            if let Material::Emitter{albedo} = hit_obj.material {
                    let cosine: f64 = ray.dir.dotprod(&hit_obj.shape.normal_at(scatter_loc));
                    return albedo * cosine.abs() };
            let scatter_ray: Ray = hit_obj.material.scatter(ray, &hit_obj.shape, scatter_loc);
            let obj_relative_loc: Vec3;
            match &hit_obj.shape {
                Shape::Sphere(sphere) => obj_relative_loc = (scatter_loc - sphere.centre).normalize(),
                Shape::Disc(disc) => obj_relative_loc = scatter_loc - disc.centre,
            }
            return hit_obj.material.albedo(&obj_relative_loc) * raytrace(&scatter_ray, scene, scatter_depth - 1)
        }
    

    // Current calculation for sky color when no intersection is made

    let t = 0.5 * (ray.dir.1 + 1.0);

    // (1.0 - t) * Color::new([1.0,1.0,1.0]) + t* Color::new([0.5, 0.7, 1.0])
    (1.0 - t) * Color::new([1.0;12]) + t * Color::new([0.5,0.5,0.5,0.5, 0.7,0.7,0.7,0.7, 1.0,1.0,1.0,1.0])

}

pub fn render_into_file(file: &mut File, cam: &camera::Camera, scene: &Vec<Hittable>, spp: u32) {
    for j in 0..cam.vert_res {
        for i in 0..cam.horiz_res {
            let mut pixel_color: Color = (0..spp).map(|_| cam.get_focus_loc())
                .map(|focus_loc| {
                    Ray::new(focus_loc, cam.get_sample_loc(i,j) - focus_loc)
                })
                .fold(Color::new([0.0;color::NUMBER_OF_BINS]), |acc, r| {
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
    let mut red: f64 = 0.0;
    let mut green: f64 = 0.0;
    let mut blue: f64 = 0.0;
    for i in 0..4 {
       red += col.bin[i]*0.25;
       green += col.bin[i+4]*0.25;
       blue += col.bin[i+8]*0.25;
    }
    ((255.0 * red.sqrt()) as u8, (255.0*green.sqrt()) as u8, (255.0 * blue.sqrt()) as u8)
}

pub fn rgba_to_color(rgba: image::Rgba<u8>) -> Color {
    let mut color = Color::new([0.0;color::NUMBER_OF_BINS]);
    for i in 0..4 {
       color.bin[i] += (rgba[0] as f64) / 255.0;
       color.bin[i + 4] += (rgba[1] as f64) / 255.0;
       color.bin[i + 8] += (rgba[2] as f64) / 255.0;
    };
    color
}