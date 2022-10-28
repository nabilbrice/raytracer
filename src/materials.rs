use crate::{vector::Vec3, geometry::SurfaceNormal};
use crate::color::Color;
use crate::ray::Ray;
use crate::geometry;
use rand::{Rng, thread_rng};

pub enum Material {
    Diffuse {albedo: Color},
    Metal {albedo: Color, fuzz: f64},
}

// Implement using enums?
impl Material {
    pub fn albedo(&self) -> Color {
        match self {
            &Material::Diffuse{albedo: color} => return color,
            &Material::Metal{albedo: color, fuzz: _} => return color,
        }
    }
    pub fn scatter<T: SurfaceNormal>(&self, inc_ray: &Ray, shape: &T, scatter_loc: Vec3) -> Ray {
        match self {
            &Material::Diffuse{albedo: _} => {
                let scatter_dir = shape.normal_at(scatter_loc) + random_vec3();
                return Ray::new(scatter_loc, scatter_dir)
            },
            &Material::Metal{albedo: _, fuzz: fuzziness} => {
                let scatter_normal = shape.normal_at(scatter_loc);
                let scatter_dir: Vec3 = inc_ray.dir - 2.0 * scatter_normal.dotprod(&inc_ray.dir) * scatter_normal;
                let fuzzified_dir = fuzzify(fuzziness, scatter_dir, scatter_normal);
                return Ray::new(scatter_loc, fuzzified_dir)
            }
        }
    }
}

fn fuzzify(fuzziness: f64, scatter_dir: Vec3, scatter_normal: Vec3) -> Vec3 {
    let fuzzy_dir = scatter_dir + (fuzziness * random_vec3());
    if fuzzy_dir.dotprod(&scatter_normal) > 0.0 {
        return fuzzy_dir
    } else { fuzzify(fuzziness, scatter_dir, scatter_normal) }
}

fn random_vec3() -> Vec3 {
    let v: (f64, f64, f64) = thread_rng().gen();
    let rand_vec3 = 2.0 * Vec3(v.0 - 0.5, v.1 - 0.5, v.2 - 0.5);
    if rand_vec3.norm() > 1.0 {
        return random_vec3();
    };
    return rand_vec3.normalize();
}