use serde::{Serialize, Deserialize};

use crate::vector::Vec3;
use crate::ray::Ray;

pub const FARAWAY: f64 = 1.0e39;

#[derive(Debug, Serialize, Deserialize)]
pub enum Shape {
    Sphere(Sphere),
    Disc(Disc),
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> f64 {
        match self {
            Shape::Sphere(sphere) => sphere.intersect(ray),
            Shape::Disc(disc) => disc.intersect(ray),
        }
    }

    pub fn normal_at(&self, surface_pos: Vec3) -> Vec3 {
        match self {
            Shape::Sphere(sphere) => sphere.normal_at(surface_pos),
            Shape::Disc(disc) => disc.normal_at(surface_pos),
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
        let h = t_smaller + sq;
        if h > 1.0e-6 { h } else {FARAWAY} // 1.0e-6 to avoid self-intersection
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
    fn disc_intersection_test() {
        let disc = Disc::new(Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 1.0), 2.0);
        let ray = Ray::new(Vec3(1.0,0.0,3.0), Vec3(0.0, 0.0, -1.0));
        assert_eq!(ray.position_at(disc.intersect(&ray)), Vec3(1.0, 0.0, 0.0));
    }
}