use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::Instant;
use clap::Parser;

use raytracer::config::Config;

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

    let cli_args = Cli::parse();
    let spp: u32 = cli_args.samples_per_pixel; // samples per pixel, default set at 10
    // Render
    println!("Starting render...");
    println!("Computing with {} samples", &cam.horiz_res*&cam.vert_res*spp);
    let timer = Instant::now();
    raytracer::render_into_file(&mut file, &cam, &scene, spp);
    println!("Render finished in {}s", timer.elapsed().as_secs());
}

#[derive(Parser)]
#[command(author="Nabil", version="0.1.0", about, long_about=None)]
pub struct Cli{
    #[arg(short='s', long="samples", default_value_t=10)]
    pub samples_per_pixel: u32
}