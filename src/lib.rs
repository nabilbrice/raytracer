pub mod vector; //call a local module into this one with ; instead of {}
pub mod color;
pub mod ray;
pub mod geometry;
pub mod materials;
pub mod camera;
pub mod config;

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
        return hit_obj.material.albedo() * raytrace(&scatter_ray, scene, scatter_depth - 1)
    }
    // Current calculation for sky color when no intersection is made
    let t = 0.5 * (ray.dir.1 + 1.0);

    (1.0 - t) * Color{r: 1.0, g: 1.0, b: 1.0} + t* Color{r: 0.5, g: 0.7, b: 1.0}

}