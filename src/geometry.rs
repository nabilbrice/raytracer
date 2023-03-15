use serde::{Serialize, Deserialize};

use crate::vector::Vec3;
use crate::ray::Ray;
use crate::materials::Material;

pub const FARAWAY: f64 = 1.0e39;

#[derive(Debug, Serialize, Deserialize)]
pub struct Sphere {
    pub orig: Vec3,
    pub radius: f64,
    pub material: Material,
}

impl Sphere {
    pub fn new(centre: Vec3, radius: f64, material: Material) -> Self {
        Self {orig: centre, radius, material}
    }

    pub fn intersect(&self, ray: &Ray) -> f64 {
        let ray_to_centre = ray.orig - self.orig;
        let b = 2.0 * ray_to_centre.dotprod(&ray.dir);
        let c = ray_to_centre.dotprod(&ray_to_centre) - self.radius * self.radius;

        let discrm = b * b - 4.0 * c;
        if discrm < 0.0 {
            return FARAWAY;
        };
        let sq = discrm.sqrt(); // there are two roots from here

        let t_smaller = -0.5 * (b + sq);
        if t_smaller > 0.0 {
            return t_smaller;
        };
        let h = t_smaller + sq;
        if h > 1.0e-6 { h } else {FARAWAY} // 1.0e-6 to avoid self-intersection
    }
}

impl Surface for Sphere {
    fn normal_at(&self, surface_pos: Vec3) -> Vec3 {
        (surface_pos - self.orig) / self.radius // cheaper hack than .normalize()
    }
}

pub trait Surface {
    fn normal_at(&self, surface_pos: Vec3) -> Vec3;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn normal_test() {
        let mat = Material::Diffuse { albedo: Color{r: 1.0,g: 1.0,b: 1.0} };
        let sph = Sphere::new(Vec3(0.0,0.0,0.0), 2.0, mat);
        assert_eq!(sph.normal_at(Vec3(2.0,0.0,0.0)), Vec3(1.0,0.0,0.0));
    }

    #[test]
    fn intersect_test() {
        let mat = Material::Diffuse{albedo: Color{r: 1.0, g: 1.0, b: 1.0}};
        let sph = Sphere::new(Vec3(0.0,0.0,0.0), 2.0, mat);
        let ray = Ray::new(Vec3(0.0,0.0,-3.0), Vec3(0.0,0.0,1.0));
        assert_eq!(sph.intersect(&ray), 1.0);
    }
}