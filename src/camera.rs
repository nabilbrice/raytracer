use crate::vector::Vec3;
use rand::{Rng, thread_rng};

pub struct Camera {
    pub lookat: Vec3, // focal plane location
    pub lookfrom: Vec3, // lens location
    pub lookup: Vec3, // up direction for the camera
    pub focal_distance: f64,
    pub inv_focal_length: f64, // 1/f sets field of view, fewer divisions
    pub aperture: f64,
    pub horiz_arm: Vec3,
    pub vert_arm: Vec3,
    pub horiz_res: u32, // number of horizontal pixels
    pub vert_res: u32, // number of vertical pixels
    pub aspect_ratio: f64,
}

impl Camera {
    pub fn build(lookat: Vec3, lookfrom: Vec3, inv_focal_length: f64, aperture: f64,
        horiz_res: u32, vert_res: u32) -> Camera {
            let lookup = Vec3(0.0, 1.0, 0.0); // fiducial value, normalize first
            let pointing_direction: Vec3 = lookat - lookfrom;
            let focal_distance = pointing_direction.norm();
            // upside down because flipped through aperture
            let horiz_arm: Vec3 = lookup.cross(&pointing_direction).normalize();
            let vert_arm: Vec3 = pointing_direction.cross(&horiz_arm).normalize();
            let aspect_ratio: f64 = horiz_res as f64 / vert_res as f64;
            Camera {
                lookat, lookfrom, lookup, focal_distance, inv_focal_length, aperture,
                horiz_arm, vert_arm,
                horiz_res, vert_res, aspect_ratio
            }

    }

    pub fn get_focus_loc(&self, rng: &mut impl Rng) -> Vec3 {
        let rng_scalars = random_in_disc(rng);

        let nudged_lookfrom: Vec3 = self.lookfrom
                            + self.aperture * rng_scalars[0]*self.horiz_arm.normalize() 
                            + self.aperture * rng_scalars[1]*self.vert_arm.normalize(); 

        nudged_lookfrom
    }

    pub fn get_sample_loc(&self, i: u32, j:u32) -> Vec3 {
        let rng_scalars: [f64; 2] = thread_rng().gen();

        let horiz_increm = 1.0/f64::from(self.horiz_res);
        let vert_increm = 1.0/f64::from(self.vert_res);
        let horiz_nudge: Vec3 = (rng_scalars[0] * horiz_increm) * self.horiz_arm;
        let vert_nudge: Vec3 = (rng_scalars[1] * vert_increm) * self.vert_arm;

        let horiz_span = self.inv_focal_length * self.focal_distance * self.horiz_arm;
        let vert_span = self.inv_focal_length * self.focal_distance * self.vert_arm;

        let grid_h_offset = -0.5 + f64::from(i)*horiz_increm;
        let grid_v_offset = 0.5 - f64::from(j)*vert_increm;

        self.lookat + (grid_h_offset * horiz_span) + (grid_v_offset * vert_span) 
        + horiz_nudge + vert_nudge
    }

}

fn random_in_disc(rng: &mut impl Rng) -> [f64;2] {
    let rng_scalars: [f64; 2] = rng.gen();

    let radius2: f64 = rng_scalars[0]*rng_scalars[0] + rng_scalars[1]*rng_scalars[1]; // rejection condition
    if radius2 > 1.0 {
        return random_in_disc(rng);
    };
    rng_scalars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_in_disc_test() {
        let mut rng = thread_rng();
        let point = random_in_disc(&mut rng);
        assert!(point[0]*point[0] + point[1]*point[1] <= 1.0, "picked point out of disc")
    }
}