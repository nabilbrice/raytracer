use serde::{Serialize, Deserialize};

use crate::geometry::Sphere;
use crate::vector::Vec3;
use crate::camera::Camera;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub camera: SetupCamera,
    pub spheres: Vec<Sphere>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupCamera {
    lookat: Vec3,
    lookfrom: Vec3,
    inv_focal_length: f64,
    aperture: f64,
    horiz_res: u32,
    vert_res: u32,
}

impl SetupCamera {
    pub fn setup(&self) -> Camera {
        Camera::build(self.lookat, self.lookfrom, self.inv_focal_length, self.aperture,
        self.horiz_res, self.vert_res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use crate::materials::Material;
    use crate::vector::Vec3;
    use crate::color::Color;

    #[test]
    fn load_config_test() {
        let config_contents = fs::read("./scene.json").expect("unable to read message");
        let deser = serde_json::from_slice::<Config>(&config_contents);
    }
}