use crate::{vector::Vec3, geometry::SurfaceNormal};
use crate::color::Color;
use crate::ray::Ray;
use crate::geometry;
use rand::{Rng, thread_rng};

pub enum Material {
    Diffuse {albedo: Color},
    Metal {albedo: Color},
}

// Implement using enums?
impl Material {
    pub fn albedo(&self) -> Color {
        match self {
            &Material::Diffuse{albedo: color} => return color,
            &Material::Metal{albedo: color} => return color,
        }
    }
    pub fn scatter<T: SurfaceNormal>(&self, inc_ray: &Ray, shape: &T, scatter_loc: Vec3) -> Ray {
        match self {
            &Material::Diffuse{albedo: _} => {
                let scatter_dir = shape.normal_at(scatter_loc) + random_vec3();
                return Ray::new(scatter_loc, scatter_dir)
            },
            &Material::Metal{albedo: _} => {
                let scatter_normal = shape.normal_at(scatter_loc);
                let scatter_dir: Vec3 = inc_ray.dir - 2.0 * scatter_normal.dotprod(&inc_ray.dir) * scatter_normal;
                return Ray::new(scatter_loc, scatter_dir)
            }
        }
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