use crate::vector::Vec3;
use crate::color::Color;
use crate::ray::Ray;
use crate::geometry;
use rand::{Rng, thread_rng};
pub struct Material {
    pub albedo: Color,
}

// Implement using enums?
pub enum Composition {
    Diffuse,
    Metal,
}

impl Material {
    pub fn new(albedo: Color) -> Material {
        Material{albedo}
    }
}

pub fn lambertian<T: geometry::SurfaceNormal>(inc_ray: &Ray, shape: &T, scatter_loc: Vec3) -> Ray {
    let scatter_dir = shape.normal_at(scatter_loc) + random_vec3();
    Ray::new(scatter_loc, scatter_dir)
}

fn random_vec3() -> Vec3 {
    let v: (f64, f64, f64) = thread_rng().gen();
    let rand_vec3 = 2.0 * Vec3(v.0 - 0.5, v.1 - 0.5, v.2 - 0.5);
    if rand_vec3.norm() > 1.0 {
        return random_vec3();
    };
    return rand_vec3.normalize();
}