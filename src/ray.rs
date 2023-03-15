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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn position_test() {
        let ray1 = Ray::new(Vec3(0.0,0.0,0.0), Vec3(1.0,0.0,0.0));
        assert_eq!(ray1.position_at(0.5), Vec3(0.5,0.0,0.0))
    }
}