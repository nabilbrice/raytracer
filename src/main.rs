use std::fs;
use std::fs::{OpenOptions, File};
use std::io::prelude::*;
use std::time::{Duration, Instant};

use raytracer::color::Color;
use raytracer::ray::Ray;
use raytracer::geometry::{Sphere};
use raytracer::camera::Camera;

use raytracer::config::Config;

use raytracer::raytrace;
fn main() {
    let config_contents = fs::read("./scene.json")
        .expect("unable to read scene file");

    let deser_config = serde_json::from_slice::<Config>(&config_contents)
        .expect("unable to deserialize scene information");

    let scene = deser_config.spheres;
    let cam = deser_config.camera.setup();

    // Write header to file
    let header = format!("P3\n{} {}\n255\n",&cam.horiz_res,&cam.vert_res);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("./image.ppm")
        .expect("Unable to open file");

    write!(file, "{}", header)
        .expect("Unable to write header to ppm");

    let spp: u32 = 10; // samples per pixel
    // Render
    println!("Starting render...");
    println!("Computing with {} samples", &cam.horiz_res*&cam.vert_res*spp);
    let timer = Instant::now();
    render_into_file(&mut file, &cam, &scene, spp);
    println!("Render finished in {}s", timer.elapsed().as_secs());
}

fn color_to_ppm(col: Color) -> (u8, u8, u8) {
    ((255.0 * col.r.sqrt()) as u8, (255.0*col.g.sqrt()) as u8, (255.0 * col.b.sqrt()) as u8)
}

fn render_into_file(file: &mut File, cam: &Camera, scene: &Vec<Sphere>, spp: u32) {
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
};
}