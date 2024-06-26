use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::ray::Ray;
use crate::rgba_to_color;
use crate::{geometry::Shape, vector::Vec3};
use image::{DynamicImage, GenericImageView};
use rand::{thread_rng, Rng};
use std::f64::consts::PI;

use image::Rgba;

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub enum Material {
    Diffuse {
        albedo: Color,
    },
    Metal {
        albedo: Color,
        fuzz: f64,
    },
    Dielectric {
        refractive_index: f64,
    },
    TextureMap {
        #[serde_as(as = "TextureMapFilePath")]
        map: DynamicImage,
        orient_up: Vec3,
        orient_around: Vec3,
    },
    Emitter {
        albedo: Color,
    },
}

fn load_image(path_to_file: &str) -> image::DynamicImage {
    image::open(path_to_file).expect("cannot open file")
}

serde_with::serde_conv!(
    TextureMapFilePath,
    DynamicImage,
    |_map: &DynamicImage| "texturemap.jpeg",
    |path_to_file: &str| -> Result<_, std::convert::Infallible> { Ok(load_image(path_to_file)) }
);

impl Material {
    pub fn albedo(&self, location: &Vec3) -> Color {
        match self {
            Material::Diffuse { albedo: color } => *color,
            Material::Metal {
                albedo: color,
                fuzz: _,
            } => *color,
            Material::Dielectric {
                refractive_index: _,
            } => Color::new(1.0, 1.0, 1.0),
            Material::TextureMap {
                map: img,
                orient_up,
                orient_around,
            } => {
                let latitude: f64 = orient_up.normalize().dotprod(&location).acos();
                let orient_axes: (Vec3, Vec3) = (
                    orient_around.normalize(),
                    orient_up.normalize().cross(&orient_around.normalize()),
                );
                let longitude: f64 = orient_axes
                    .0
                    .dotprod(&location)
                    .atan2(orient_axes.1.dotprod(&location))
                    + PI;
                let texture_color: Rgba<u8> = get_texture_rgba(&img, longitude, latitude);
                rgba_to_color(texture_color)
            }
            Material::Emitter { albedo: color } => *color,
        }
    }
    pub fn scatter(&self, inc_ray: &Ray, shape: &Shape, scatter_loc: Vec3) -> Ray {
        match *self {
            Material::Diffuse { albedo: _ } => {
                let scatter_dir = shape.normal_at(scatter_loc) + random_vec3();
                return Ray::new(scatter_loc, scatter_dir);
            }
            Material::Metal {
                albedo: _,
                fuzz: fuzziness,
            } => {
                let scatter_normal = shape.normal_at(scatter_loc);
                let scatter_dir: Vec3 =
                    inc_ray.dir - 2.0 * scatter_normal.dotprod(&inc_ray.dir) * scatter_normal;
                let fuzzified_dir = fuzzify(fuzziness, scatter_dir, scatter_normal);
                return Ray::new(scatter_loc, fuzzified_dir);
            }
            Material::Dielectric {
                refractive_index: r_idx,
            } => {
                let scatter_normal = shape.normal_at(scatter_loc);
                let inc_cos = scatter_normal.dotprod(&inc_ray.dir); // -ve the usual for most ray-tracers
                let inc_dir_perp: Vec3 = inc_ray.dir - inc_cos * scatter_normal;
                let mut refract_ratio = r_idx; // default ray going from inside to outside so fewer divisions
                let sign_inc = inc_cos.signum(); // needed for determining scattered ray parallel direction
                if sign_inc < 0.0 {
                    refract_ratio = 1.0 / r_idx
                }; // refract from the outside
                let scatter_dir_perp = refract_ratio * inc_dir_perp;
                let scatter_sin2: f64 = scatter_dir_perp.dotprod(&scatter_dir_perp); // no sqrt needed

                if scatter_sin2 > 1.0 || schlick(inc_cos, refract_ratio) {
                    // total internal reflection
                    let scatter_dir: Vec3 = inc_dir_perp - inc_cos * scatter_normal;
                    return Ray::new(scatter_loc, scatter_dir);
                } else {
                    // refraction
                    // refracted ray goes in the same direction as inc ray so sign of cos is the same
                    let scatter_cos: f64 = sign_inc * (1.0 - scatter_sin2).sqrt();
                    let scatter_dir = scatter_dir_perp + scatter_cos * scatter_normal;
                    return Ray::new(scatter_loc, scatter_dir);
                };
            }
            Material::TextureMap { .. } => {
                let scatter_dir = shape.normal_at(scatter_loc) + random_vec3();
                return Ray::new(scatter_loc, scatter_dir);
            }
            _ => {
                panic!("Attempted to access scatter for Material without scattering implemented")
            }
        }
    }
}

fn schlick(cosine: f64, r_idx: f64) -> bool {
    let mut r0 = (1.0 - r_idx) / (1.0 + r_idx);
    r0 = r0 * r0;
    let reflectance: f64 = r0 + (1.0 - r0) * (1.0 - cosine.abs()).powi(5);
    let drawn_prob = thread_rng().gen_range(0.0..1.0);
    drawn_prob < reflectance
}

fn fuzzify(fuzziness: f64, scatter_dir: Vec3, scatter_normal: Vec3) -> Vec3 {
    let fuzzy_dir = scatter_dir + (fuzziness * random_vec3());
    if fuzzy_dir.dotprod(&scatter_normal) > 0.0 {
        fuzzy_dir
    } else {
        fuzzify(fuzziness, scatter_dir, scatter_normal)
    }
}

fn random_vec3() -> Vec3 {
    let v: (f64, f64, f64) = thread_rng().gen();
    let rand_vec3 = 2.0 * Vec3([v.0 - 0.5, v.1 - 0.5, v.2 - 0.5]);
    if rand_vec3.norm() > 1.0 {
        return random_vec3();
    };
    return rand_vec3.normalize();
}

fn get_texture_rgba(image: &DynamicImage, longitude_rad: f64, latitude_rad: f64) -> Rgba<u8> {
    let dimensions: (u32, u32) = image.dimensions();

    let (pixel_column, pixel_row): (f64, f64) = (
        0.5 * longitude_rad / PI * (dimensions.0 as f64),
        latitude_rad / PI * (dimensions.1 as f64),
    );

    image.get_pixel(
        pixel_column as u32 % dimensions.0,
        pixel_row as u32 % dimensions.1,
    )
}

#[cfg(test)]
mod tests {}
