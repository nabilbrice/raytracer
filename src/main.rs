use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::Instant;

use raytracer::config::Config;

use raytracer::render_into_file;
fn main() {
    let config_contents = fs::read("./scene.json")
        .expect("unable to read scene file");

    let de_config = serde_json::from_slice::<Config>(&config_contents)
        .expect("unable to deserialize scene information");

    let scene = de_config.spheres;
    let cam = de_config.camera.setup();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("./image.ppm")
        .expect("Unable to open file to write");

    let header = format!("P3\n{} {}\n255\n",&cam.horiz_res,&cam.vert_res);
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