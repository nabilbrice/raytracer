// pub mod boundingvolume;
pub mod boundingvolume;
pub mod camera;
pub mod color;
pub mod config;
pub mod geometry;
#[macro_use]
pub mod intervals;
pub mod materials;
pub mod ray;
pub mod scenegen;
pub mod vector;

use std::fs::File;
use std::io::{BufWriter, Write};

use boundingvolume::{tree_filter, BoundingBox, CoveringTree};
use color::Color;
use geometry::Shape;
use materials::Material;
use ray::Ray;
use serde::{Deserialize, Serialize};
use vector::Vec3;

#[derive(Debug, Serialize, Deserialize)]
pub struct Hittable {
    shape: Shape,
    material: Material,
}

pub fn cmp_intersection(a: Option<f64>, b: Option<f64>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.partial_cmp(&b).unwrap(),
        (Some(_), None) => std::cmp::Ordering::Less, // None is larger so < is returned
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

pub fn raytrace(ray: &Ray, scene: &[Hittable], scatter_depth: u8) -> Color {
    let mut color = Color::new(1.0, 1.0, 1.0);

    let mut ray = ray;
    let mut scatter_ray: Ray;
    for _ in 1..=scatter_depth {
        if let Some((hit_obj, Some(param))) = scene
            .iter()
            .map(|hittable| (hittable, hittable.shape.intersect(ray)))
            .min_by(|x, y| cmp_intersection(x.1, y.1))
        {
            let scatter_loc: Vec3 = ray.position_at(param);
            if let Material::Emitter { albedo } = hit_obj.material {
                let cosine: f64 = ray.dir.dotprod(&hit_obj.shape.normal_at(scatter_loc));
                return albedo * cosine.abs();
            };
            scatter_ray = hit_obj.material.scatter(ray, &hit_obj.shape, scatter_loc);
            let obj_relative_loc: Vec3;
            match &hit_obj.shape {
                Shape::Sphere(sphere) => {
                    obj_relative_loc = (scatter_loc - sphere.centre).normalize()
                }
                Shape::Disc(disc) => obj_relative_loc = scatter_loc - disc.centre,
                _ => todo!(),
            }
            color = color * hit_obj.material.albedo(&obj_relative_loc);
            ray = &scatter_ray;
        } else {
            let t = 0.5 * (ray.dir[1] + 1.0);
            let sky_color = (1.0 - t)
                * Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                }
                + t * Color {
                    r: 0.5,
                    g: 0.7,
                    b: 1.0,
                };

            return color * sky_color;
        }
    }

    color
}

pub fn accel_raytrace(ray: &Ray, tree: &Box<CoveringTree>, scatter_depth: u8) -> Color {
    let mut color = Color::new(1.0, 1.0, 1.0);

    let mut ray = ray;
    let mut scatter_ray: Ray;
    for _ in 1..=scatter_depth {
        let mut subscene = Vec::<(&Hittable, Option<f64>)>::new();
        tree_filter(tree, &mut subscene, ray);

        if let Some((hit_obj, Some(param))) =
            subscene.iter().min_by(|x, y| cmp_intersection(x.1, y.1))
        {
            let scatter_loc: Vec3 = ray.position_at(*param);
            if let Material::Emitter { albedo } = hit_obj.material {
                let cosine: f64 = ray.dir.dotprod(&hit_obj.shape.normal_at(scatter_loc));
                return albedo * cosine.abs();
            };
            scatter_ray = hit_obj.material.scatter(ray, &hit_obj.shape, scatter_loc);
            let obj_relative_loc: Vec3;
            match &hit_obj.shape {
                Shape::Sphere(sphere) => {
                    obj_relative_loc = (scatter_loc - sphere.centre).normalize()
                }
                Shape::Disc(disc) => obj_relative_loc = scatter_loc - disc.centre,
                _ => todo!(),
            }
            color = color * hit_obj.material.albedo(&obj_relative_loc);
            ray = &scatter_ray;
        } else {
            let t = 0.5 * (ray.dir[1] + 1.0);
            let sky_color = (1.0 - t)
                * Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                }
                + t * Color {
                    r: 0.5,
                    g: 0.7,
                    b: 1.0,
                };

            return color * sky_color;
        }
    }

    color
}

pub fn render_into_file(file: &mut File, cam: &camera::Camera, scene: &[Hittable], spp: u32) {
    let mut vis_stream = BufWriter::new(file);
    for j in 0..cam.vert_res {
        for i in 0..cam.horiz_res {
            let mut pixel_color: Color = (0..spp)
                .map(|_| cam.get_focus_loc())
                .map(|focus_loc| Ray::new(focus_loc, cam.get_sample_loc(i, j) - focus_loc))
                .fold(Color::new(0.0, 0.0, 0.0), |acc, r| {
                    acc + raytrace(&r, &scene, 10)
                });

            pixel_color = (1.0 / (spp as f64)) * pixel_color; // no Div defined for Color
            let color = color_to_ppm(pixel_color);

            writeln!(vis_stream, "{} {} {}", color.0, color.1, color.2)
                .expect("Unable to write colors.");
        }
        eprint!("\rScanline: {} out of {}", j, cam.vert_res);
    }
    eprintln!("");
}

pub fn accel_render_into_file(
    file: &mut File,
    cam: &camera::Camera,
    tree: Box<CoveringTree>,
    spp: u32,
) {
    let mut vis_stream = BufWriter::new(file);
    for j in 0..cam.vert_res {
        for i in 0..cam.horiz_res {
            let mut pixel_color: Color = (0..spp)
                .map(|_| cam.get_focus_loc())
                .map(|focus_loc| Ray::new(focus_loc, cam.get_sample_loc(i, j) - focus_loc))
                .fold(Color::new(0.0, 0.0, 0.0), |acc, r| {
                    acc + accel_raytrace(&r, &tree, 10)
                });

            pixel_color = (1.0 / (spp as f64)) * pixel_color; // no Div defined for Color
            let color = color_to_ppm(pixel_color);

            writeln!(vis_stream, "{} {} {}", color.0, color.1, color.2)
                .expect("Unable to write colors.");
        }
        eprint!("\rScanline: {} out of {}", j, cam.vert_res);
    }
    eprintln!("");
}

pub fn color_to_ppm(col: Color) -> (u8, u8, u8) {
    (
        (255.0 * col.r.sqrt()) as u8,
        (255.0 * col.g.sqrt()) as u8,
        (255.0 * col.b.sqrt()) as u8,
    )
}

pub fn rgba_to_color(rgba: image::Rgba<u8>) -> Color {
    Color::new(
        (rgba[0] as f64) / 255.0,
        (rgba[1] as f64) / 255.0,
        (rgba[2] as f64) / 255.0,
    )
}
