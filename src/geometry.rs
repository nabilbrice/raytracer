use serde::{Serialize, Deserialize};

use crate::vector::Vec3;
use crate::ray::Ray;
use std::f64::consts::PI;

pub const FARAWAY: f64 = 1.0e39;

#[derive(Debug, Serialize, Deserialize)]
pub enum Shape {
    Sphere(Sphere),
    Disc(Disc),
    Cylinder(Cylinder),
    TruncCone(TruncCone),
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> f64 {
        match self {
            Shape::Sphere(sphere) => sphere.intersect(ray),
            Shape::Disc(disc) => disc.intersect(ray),
            Shape::Cylinder(cylinder) => cylinder.intersect(ray),
            Shape::TruncCone(cone) => cone.intersect(ray),
        }
    }

    pub fn normal_at(&self, surface_pos: Vec3) -> Vec3 {
        match self {
            Shape::Sphere(sphere) => sphere.normal_at(surface_pos),
            Shape::Disc(disc) => disc.normal_at(surface_pos),
            Shape::Cylinder(cylinder) => cylinder.normal_at(surface_pos),
            Shape::TruncCone(cone) => cone.normal_at(surface_pos),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sphere {
    pub centre: Vec3,
    pub radius: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disc {
    pub centre: Vec3,
    pub normal: Vec3,
    pub radius: f64,
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Cylinder {
    pub centre: Vec3,
    #[serde_as(as = "axis_normalized")]
    pub axis: Vec3,
    pub radius: f64,
    pub height: f64,
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TruncCone {
    pub centre: Vec3,
    #[serde_as(as = "axis_normalized")]
    pub axis: Vec3,
    pub opening_angle: f64, // in radians
    pub centre_radius: f64,
    pub height: f64,
}

serde_with::serde_conv!(
    axis_normalized,
    Vec3,
    | _ | " ",
    |axis: Vec3| -> Result<_, std::convert::Infallible> {Ok(axis.normalize())}
);

impl Sphere {
    pub fn new(centre: Vec3, radius: f64) -> Self {
        Self {centre, radius}
    }

    pub fn intersect(&self, ray: &Ray) -> f64 {
        let ray_to_centre = ray.orig - self.centre;
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
        let t_larger = t_smaller + sq;
        if t_larger > 1.0e-6 { t_larger } else {FARAWAY} // 1.0e-6 to avoid self-intersection
    }

    pub fn normal_at(&self, surface_pos: Vec3) -> Vec3 {
        (surface_pos - self.centre)/self.radius
    }
}

impl Disc {
    pub fn new(centre: Vec3, normal: Vec3, radius: f64) -> Self {
        Self {centre, normal: normal.normalize(), radius}
    }

    pub fn intersect(&self, ray: &Ray) -> f64 {
        if self.normal.dotprod(&ray.dir) == 0.0 {return FARAWAY};
        let h: f64 = (self.centre - ray.orig).dotprod(&self.normal)/self.normal.dotprod(&ray.dir);
        let point_in_disc: Vec3 = ray.position_at(h) - self.centre;
        if point_in_disc.dotprod(&point_in_disc) > self.radius * self.radius {return FARAWAY};
        return h
    }

    pub fn normal_at(&self, _surface_pos: Vec3) -> Vec3 {
        self.normal
    }
}


impl Cylinder {
    pub fn new(centre: Vec3, axis: Vec3, radius: f64, height: f64) -> Self {
        Self {centre, axis: axis.normalize(), radius, height}
    }

    pub fn intersect(&self, ray: &Ray) -> f64 {
        let translated_ray: Ray = Ray::new(ray.orig - self.centre, ray.dir);
        let axis_rayd_cos = self.axis.dotprod(&translated_ray.dir);
        let axis_rayo_cos = self.axis.dotprod(&translated_ray.orig);
        
        let a: f64 = 1.0 - axis_rayd_cos*axis_rayd_cos;
        let b: f64 = 2.0*(translated_ray.orig.dotprod(&translated_ray.dir) - (axis_rayo_cos*axis_rayd_cos));
        let c: f64 = {translated_ray.orig.dotprod(&translated_ray.orig) - (axis_rayo_cos*axis_rayo_cos) 
            - self.radius*self.radius};

        let discrim = b*b - 4.0*a*c;
        if discrim < 0.0 {
            return FARAWAY
        } 
        let sq = discrim.sqrt(); // there are two roots from here

        let t_smaller = -0.5 * (b + sq)/a;
        let surface_height_smaller = translated_ray.position_at(t_smaller).dotprod(&self.axis).powi(2);
        if t_smaller > 0.0 && surface_height_smaller < self.height*self.height {
            return t_smaller;
        };
        let t_larger = t_smaller + sq/a;
        let surface_height = translated_ray.position_at(t_larger).dotprod(&self.axis).powi(2);
        if t_larger < 1.0e-6 || surface_height > self.height*self.height { FARAWAY } else { t_larger } // 1.0e-6 to avoid self-intersection
    }

    pub fn normal_at(&self, surface_pos: Vec3) -> Vec3 {
        let relative_pos = surface_pos - self.centre;
        (relative_pos - (relative_pos.dotprod(&self.axis)) * self.axis).normalize()
    }
}

impl TruncCone {
    pub fn new(centre: Vec3, axis: Vec3, opening_angle: f64, centre_radius: f64, height: f64) -> Self {
        Self {centre, axis: axis.normalize(), opening_angle, centre_radius, height}
    }

    pub fn intersect(&self, ray: &Ray) -> f64 {
        let opening_rad = self.opening_angle / 180.0 * PI;
        let bottom_pos = self.centre_radius / opening_rad.tan() * self.axis - self.centre;
        let translated_ray: Ray = Ray::new(ray.orig + bottom_pos, ray.dir);
        let axis_rayd_cos = self.axis.dotprod(&translated_ray.dir);
        let axis_rayo_cos = self.axis.dotprod(&translated_ray.orig);
        let cosine_sq: f64 = opening_rad.cos().powi(2);
        
        let a: f64 = cosine_sq - axis_rayd_cos*axis_rayd_cos;
        let b: f64 = 2.0*(cosine_sq*translated_ray.orig.dotprod(&translated_ray.dir) - (axis_rayo_cos*axis_rayd_cos));
        let c: f64 = {cosine_sq*translated_ray.orig.dotprod(&translated_ray.orig) - (axis_rayo_cos*axis_rayo_cos)};

        let discrim = b*b - 4.0*a*c;
        if discrim < 0.0 {
            return FARAWAY
        } 
        let sq = discrim.sqrt(); // there are two roots from here

        let t_smaller = -0.5 * (b + sq)/a;
        let surface_height = translated_ray.position_at(t_smaller).dotprod(&self.axis);
        let lower_height = self.centre_radius / opening_rad.cos();
        if t_smaller > 0.0 && check_interval(surface_height, lower_height, self.height) {
            return t_smaller;
        };
        let t_larger = t_smaller + sq/a;
        let surface_height = translated_ray.position_at(t_larger).dotprod(&self.axis);
        if t_larger < 1.0e-8 || !check_interval(surface_height, lower_height, self.height) { FARAWAY } else { t_larger } // 1.0e-6 to avoid self-intersection
    }
    
    pub fn normal_at(&self, surface_pos: Vec3) -> Vec3 {
        let opening_rad = self.opening_angle / 180.0 * PI;
        let axis_pos: Vec3 = self.centre + surface_pos.norm() / opening_rad.cos() * self.axis;
        (axis_pos - surface_pos).normalize()
    }
    
}

fn check_interval(val: f64, lower: f64, upper: f64) -> bool {
    lower <= val && val <= upper
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_normal_test() {
        let sph = Sphere::new(Vec3(0.0,0.0,0.0), 2.0);
        assert_eq!(sph.normal_at(Vec3(2.0,0.0,0.0)), Vec3(1.0,0.0,0.0));
    }

    #[test]
    fn disc_normal_test() {
        let disc = Disc::new(Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 1.0), 1.0);
        assert_eq!(disc.normal_at(Vec3(0.0, 0.5, 0.0)), Vec3(0.0, 0.0, 1.0));
    }

    #[test]
    fn sphere_intersect_test() {
        let sph = Sphere::new(Vec3(0.0,0.0,0.0), 2.0);
        let ray = Ray::new(Vec3(0.0,0.0,-3.0), Vec3(0.0,0.0,1.0));
        assert_eq!(sph.intersect(&ray), 1.0);
    }

    #[test]

    fn sphere_non_intersection_test() {
        let sph = Sphere::new(Vec3(0.0,0.0,0.0), 1.0);
        let ray = Ray::new(Vec3(2.0,0.0,0.0), Vec3(1.0,0.0,0.0));
        assert_eq!(sph.intersect(&ray), FARAWAY);
    }

    #[test]
    fn disc_intersection_test() {
        let disc = Disc::new(Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 1.0), 2.0);
        let ray = Ray::new(Vec3(1.0,0.0,3.0), Vec3(0.0, 0.0, -1.0));
        assert_eq!(ray.position_at(disc.intersect(&ray)), Vec3(1.0, 0.0, 0.0));
    }

    #[test]
    fn cylinder_intersection_test() {
        let cylinder = Cylinder::new(Vec3(0.0,0.0,0.0), Vec3(0.0,0.0,2.0), 1.0, 0.5);
        let ray = Ray::new(Vec3(3.0,3.0,3.0), Vec3(-1.0,-1.0,0.0));
        assert_eq!(ray.position_at(cylinder.intersect(&ray)), Vec3(0.5_f64.sqrt(), 0.5_f64.sqrt(),3.0));
    }
}