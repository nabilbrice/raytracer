pub mod vector;
pub mod color;
pub mod ray;
pub mod geometry;
pub mod materials;
pub mod camera;
pub mod config;

use std::fs::File;
use std::io::{Write, BufWriter};
use rayon::prelude::*;

use vector::Vec3;
use ray::Ray;
use color::{Color, NUMBER_OF_BINS};
use geometry::{Shape, FARAWAY};
use materials::{Material};
use serde::{Serialize, Deserialize};
use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]
pub struct Hittable {
    shape: Shape,
    material: Material,
}


pub fn raytrace(ray: &Ray, scene: &Vec<Hittable>, scatter_depth: u8, rng: &mut impl Rng) -> Color {
    if scatter_depth == 0 {
        return Color::new([0.0;color::NUMBER_OF_BINS]);
    }

    let (hit_obj, param) = scene.iter()
                        .map(|hittable| {(hittable, hittable.shape.intersect(ray))})
                        .min_by(|x,y| {x.1.total_cmp(&y.1)})
                        .unwrap();


        if param != FARAWAY {
            let scatter_loc: Vec3 = ray.position_at(param);
            if let Material::Emitter(emitter) = &hit_obj.material {
                match emitter {
                    _ => {
                        let cosine: f64 = ray.dir.dotprod(&hit_obj.shape.normal_at(scatter_loc));
                        return emitter.spectrum(&scatter_loc) * cosine.abs() },
                }
            }
            let scatter_ray: Ray = hit_obj.material.scatter(ray, &hit_obj.shape, scatter_loc, rng);
            let obj_relative_loc: Vec3;
            match &hit_obj.shape {
                Shape::Sphere(sphere) => obj_relative_loc = (scatter_loc - sphere.centre).normalize(),
                Shape::Disc(disc) => obj_relative_loc = scatter_loc - disc.centre,
                Shape::Cylinder(cylinder) => {obj_relative_loc = (scatter_loc - cylinder.centre).normalize()},
                Shape::TruncCone(cone) => {obj_relative_loc = (scatter_loc - cone.centre).normalize()},
            }
            return hit_obj.material.spectrum(&obj_relative_loc) * raytrace(&scatter_ray, scene, scatter_depth - 1, rng)
        }
    

    // Current calculation for sky color when no intersection is made

    Color::new([0.0;NUMBER_OF_BINS])

    // let t = 0.5 * (ray.dir.1 + 1.0);

    // (1.0 - t) * Color::new([1.0,1.0,1.0]) + t* Color::new([0.5, 0.7, 1.0])
    // (1.0 - t) * Color::new([1.0;NUMBER_OF_BINS]) + t * Color::new([0.5,0.5,0.5,0.5, 0.7,0.7,0.7,0.7, 1.0,1.0,1.0,1.0])

}

pub fn render_into_file(vis_file: File, tot_file: File, cam: &camera::Camera, scene: &Vec<Hittable>, spp: u32) {
    let mut vis_stream = BufWriter::new(vis_file);
    let mut tot_stream = BufWriter::new(tot_file);
    for j in 0..cam.vert_res {
        for i in 0..cam.horiz_res {
            let mut pixel_color: Color = (0..spp).into_par_iter()
                .map_init(|| (rand::thread_rng(), rand::thread_rng()),
                    |rng, _| {
                        let focus_loc = cam.get_focus_loc(&mut rng.0);
                        let ray = Ray::new(focus_loc, cam.get_sample_loc(i,j) - focus_loc);
                        raytrace(&ray, &scene, 10, &mut rng.1)
                    }
                ).reduce(|| Color::new([0.0;NUMBER_OF_BINS]), 
                    |a: Color,b: Color| (a + b));
        
            pixel_color = (1.0/(spp as f64)) * pixel_color; // no Div defined for Color

            writeln!(tot_stream, "{}", &pixel_color)
                .expect("Unable to write total information");

            let color = color_to_ppm(pixel_color);
            
            writeln!(vis_stream, "{} {} {}", color[0], color[1], color[2])
                 .expect("Unable to write colors.");
        };
        eprint!("\rScanlines: {} out of {}",j , cam.vert_res);
    };
    tot_stream.flush().expect("Cannot flush total buffer");
    vis_stream.flush().expect("Cannot flush visualise buffer.");
    eprintln!("");
}


pub fn color_to_ppm(col: Color) -> [u8;3] {
    let mut red: f64 = 0.0;
    let mut green: f64 = 0.0;
    let mut blue: f64 = 0.0;
    for i in 0..4 {
       red += col.bin[i]*0.25;
       green += col.bin[i+4]*0.25;
       blue += col.bin[i+8]*0.25;
    }
    [(255.0 * red.sqrt()).clamp(0.0,255.0) as u8, (255.0*green.sqrt()).clamp(0.0,255.0) as u8, (255.0 * blue.sqrt()).clamp(0.0,255.0) as u8]
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

pub fn logspace<const N: usize>(start: f64, stop: f64) -> [f64;N] {
    let multiplier: f64 = 10.0_f64.powf((stop/start).log10()/N as f64);
    let mut array: [f64;N] = [0.0;N];
    for i in 0..N {
        array[i] = start * multiplier.powi(i as i32);
    }
    array
}