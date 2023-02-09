use crate::vector::Vec3;

pub struct Ray {
    pub orig: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(position: Vec3, point_to: Vec3) -> Ray {
        Ray {orig: position, dir: point_to.normalize()}
    }
    pub fn position_at(&self, t: f64) -> Vec3 {
        self.orig + t * self.dir
    }
}