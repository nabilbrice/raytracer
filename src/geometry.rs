use crate::vector::Vec3;
use crate::ray::Ray;

pub const FARAWAY: f64 = 1.0e39;

pub struct Sphere {
    pub orig: Vec3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(centre: Vec3, radius: f64) -> Sphere {
        Sphere {orig: centre, radius}
    }

    pub fn intersect(&self, ray: &Ray) -> f64 {
        let ray_to_centre = ray.orig - self.orig;
        let b = 2.0 * ray_to_centre.dotprod(&ray.dir);
        let c = ray_to_centre.dotprod(&ray_to_centre) - self.radius * self.radius;

        let discrm = b * b - 4.0 * c;
        let sq = discrm.sqrt().max(0.0);

        let t0 = -0.5 * (b + sq);
        let t1 = -0.5 * (b - sq);

        let h = if t1 > t0 {t1} else {t0};

        let hit = (discrm > 0.0) && (h > 0.0);

        if hit { h } else {FARAWAY}

    }

    pub fn normal_at(&self, surface_pos: Vec3) -> Vec3 {
        (surface_pos - self.orig).normalize()
    }
}