use serde::{Serialize, Deserialize};
use std::ops::Deref;

use crate::vector::Vec3;
use crate::ray::Ray;
use crate::intervals;
use crate::intervals::Interval;

#[derive(Debug, Serialize, Deserialize)]
pub enum Shape {
    Sphere(Sphere),
    Disc(Disc),
    #[serde(skip_serializing, skip_deserializing)]
    BoundVolume(BoundBox),
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> Option<f64> {
        match self {
            Shape::Sphere(sphere) => sphere.intersect(ray),
            Shape::Disc(disc) => disc.intersect(ray),
            Shape::BoundVolume(bbox) => bbox.intersect(ray),
            _ => unreachable!(),
        }
    }

    pub fn normal_at(&self, surface_pos: Vec3) -> Vec3 {
        match self {
            Shape::Sphere(sphere) => sphere.normal_at(surface_pos),
            Shape::Disc(disc) => disc.normal_at(surface_pos),
            _ => todo!(),
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

#[derive(Debug)]
pub struct BoundBox([Interval;3]);

impl Deref for BoundBox {
    type Target = [Interval;3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn cover(bbox1: &BoundBox, bbox2: &BoundBox) -> BoundBox {
    BoundBox(
        [intervals::cover(&bbox1[0], &bbox2[0]),
        intervals::cover(&bbox1[1], &bbox2[1]),
        intervals::cover(&bbox1[2], &bbox2[2])])
}

impl BoundBox {
    pub fn intersect(&self, ray: &Ray) -> Option<f64> {
        let mut times = self.iter().zip(ray.orig.iter()).zip(ray.dir.iter())
            .map(|((interval,orig),dir)| Interval::new((interval.start - orig)/dir, (interval.end - orig)/dir));

        let intersection01 = intervals::intersection(&times.next().unwrap(), &times.next().unwrap());
        if intersection01.is_none() {
            return None
        };
        let intersection012 = intervals::intersection(&intersection01.unwrap(), &times.next().unwrap());
        match intersection012 {
            Some(interval) => Some(interval.start),
            None => None,
        }
    }

}

impl Sphere {
    pub fn new(centre: Vec3, radius: f64) -> Self {
        Self {centre, radius}
    }

    pub fn intersect(&self, ray: &Ray) -> Option<f64> {
        let ray_to_centre = ray.orig - self.centre;
        let b = 2.0 * ray_to_centre.dotprod(&ray.dir);
        let c = ray_to_centre.dotprod(&ray_to_centre) - self.radius * self.radius;

        let discrm = b * b - 4.0 * c;
        if discrm < 0.0 {
            return Option::None;
        };
        let sq = discrm.sqrt(); // there are two roots from here

        let t_smaller = -0.5 * (b + sq);
        if t_smaller > 0.0 {
            return Some(t_smaller);
        };
        let t_larger = t_smaller + sq;
        if t_larger > 1.0e-6 { Some(t_larger) } else {Option::None} // 1.0e-6 to avoid self-intersection
    }

    pub fn normal_at(&self, surface_pos: Vec3) -> Vec3 {
        (surface_pos - self.centre)/self.radius
    }
}

impl Disc {
    pub fn new(centre: Vec3, normal: Vec3, radius: f64) -> Self {
        Self {centre, normal: normal.normalize(), radius}
    }

    pub fn intersect(&self, ray: &Ray) -> Option<f64> {
        if self.normal.dotprod(&ray.dir) == 0.0 {return None};
        let h: f64 = (self.centre - ray.orig).dotprod(&self.normal)/self.normal.dotprod(&ray.dir);
        let point_in_disc: Vec3 = ray.position_at(h) - self.centre;
        if point_in_disc.dotprod(&point_in_disc) > self.radius * self.radius {return None};
        return Some(h)
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
        let sph = Sphere::new(Vec3([0.0,0.0,0.0]), 2.0);
        assert_eq!(sph.normal_at(Vec3([2.0,0.0,0.0])), Vec3([1.0,0.0,0.0]));
    }

    #[test]
    fn disc_normal_test() {
        let disc = Disc::new(Vec3([0.0, 0.0, 0.0]), Vec3([0.0, 0.0, 1.0]), 1.0);
        assert_eq!(disc.normal_at(Vec3([0.0, 0.5, 0.0])), Vec3([0.0, 0.0, 1.0]));
    }

    #[test]
    fn sphere_intersect_test() {
        let sph = Sphere::new(Vec3([0.0,0.0,0.0]), 2.0);
        let ray = Ray::new(Vec3([0.0,0.0,-3.0]), Vec3([0.0,0.0,1.0]));
        assert_eq!(sph.intersect(&ray), Some(1.0));
    }

    #[test]
    fn sphere_non_intersection_test() {
        let sph = Sphere::new(Vec3([0.0,0.0,0.0]), 1.0);
        let ray = Ray::new(Vec3([2.0,0.0,0.0]), Vec3([1.0,0.0,0.0]));
        assert_eq!(sph.intersect(&ray), Option::None);
    }

    #[test]
    fn disc_intersection_test() {
        let disc = Disc::new(Vec3([0.0, 0.0, 0.0]), Vec3([0.0, 0.0, 1.0]), 2.0);
        let ray = Ray::new(Vec3([1.0,0.0,3.0]), Vec3([0.0, 0.0, -1.0]));
        assert_eq!(ray.position_at(disc.intersect(&ray).unwrap()), Vec3([1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_bbox_cover() {
        let bbox1 = BoundBox([Interval::new(0.0,1.0), Interval::new(0.0,1.0), Interval::new(0.0,1.0)]);
        let bbox2 = BoundBox([Interval::new(-1.0,1.0), Interval::new(0.0,2.0), Interval::new(-1.0,0.0)]);
        let bbox3 = BoundBox([Interval::new(-1.0,1.0), Interval::new(0.0,2.0), Interval::new(-1.0,1.0)]);

        let new_bbox = cover(&bbox1, &bbox2);

        assert_eq!(new_bbox[0].start, bbox3[0].start);
        assert_eq!(new_bbox[0].end, bbox3[0].end);
        assert_eq!(new_bbox[1].start, bbox3[1].start);
        assert_eq!(new_bbox[1].end, bbox3[1].end);
        assert_eq!(new_bbox[2].start, bbox3[2].start);
        assert_eq!(new_bbox[2].end, bbox3[2].end);

    }
}
