use serde::{Serialize, Deserialize};

use crate::{rgba_to_color, logspace};
use crate::{vector::Vec3, geometry::Shape};
use crate::color::{Color, NUMBER_OF_BINS};
use crate::ray::Ray;
use rand::{Rng, thread_rng};
use image::{DynamicImage, GenericImageView};
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};

use image::Rgba;

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub enum Emitter {
    Blackbody {temperature: f64},
    TemperatureMap {
        #[serde_as(as = "TemperatureMapFilePath")]
        map: Vec<Vec<f64>>,
        orient_up: Vec3,
        orient_around: Vec3,
    },
}

fn blackbody(temperature: f64) -> Color {
    // The flux is not using a normalization
    let flux = |energy: f64| energy.powi(3)/((energy / temperature).exp() - 1.0);
    let mut bin = [0.0;NUMBER_OF_BINS];
    let energy_bins = logspace::<NUMBER_OF_BINS>(0.1,20.0);
    for i in 0..NUMBER_OF_BINS {
        bin[i] = flux(energy_bins[i])
    }
    Color::new(bin) 
}

impl Emitter {
    pub fn spectrum(&self, location: &Vec3) -> Color {
        match self {
            Emitter::Blackbody {temperature} => { blackbody(*temperature) },
            Emitter::TemperatureMap{map, orient_up, orient_around} => {
                let latitude: f64 = orient_up.normalize().dotprod(location).acos();
                let orient_axes: (Vec3, Vec3) = (orient_around.normalize(), orient_up.normalize().cross(&orient_around.normalize()));
                let longitude: f64 = orient_axes.0.dotprod(location).atan2(orient_axes.1.dotprod(location)) + PI;

                let temperature = get_temperature(map, latitude, longitude)*10.0;
                blackbody(temperature)
            }
        }
    }
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub enum Material {
    Diffuse {albedo: Color},
    Metal {albedo: Color, fuzz: f64},
    Dielectric {refractive_index: f64},
    TextureMap {
        #[serde_as(as = "TextureMapFilePath")]
        map: DynamicImage,
        orient_up: Vec3,
        orient_around: Vec3,
    },
    Emitter(Emitter),
}

fn load_image(path_to_file: &str) -> image::DynamicImage {
    image::open(path_to_file).expect("cannot open file")
}

fn load_data(path_to_file: &str) -> Vec<Vec<f64>> {
    let f = BufReader::new(File::open(path_to_file).unwrap());

    let temperatures: Vec<Vec<f64>> = f.lines()
        .map(|line| line.unwrap().split("    ") // tempmap has 4 space delimiter
            .map(|value| value.parse().unwrap_or(0.0)).collect())
        .collect();
    temperatures
}

serde_with::serde_conv!(
    TextureMapFilePath,
    DynamicImage,
    |_map: &DynamicImage| "texturemap.jpeg",
    |path_to_file: &str| -> Result<_, std::convert::Infallible> {Ok(load_image(path_to_file))}
);

serde_with::serde_conv!(
    TemperatureMapFilePath,
    Vec<Vec<f64>>,
    |_map: &[Vec<f64>]| "tempmap.dat",
    |path_to_file: &str| -> Result<_, std::convert::Infallible> {Ok(load_data(path_to_file))}
);

impl Material {
    pub fn spectrum(&self, location: &Vec3) -> Color {
        match self {
            Material::Diffuse{albedo: color} => *color,
            Material::Metal{albedo: color, fuzz: _} => *color,
            Material::Dielectric { refractive_index: _ } => Color::new([1.0;NUMBER_OF_BINS]),
            Material::TextureMap {map: img, orient_up, orient_around} => {
                let latitude: f64 = orient_up.normalize().dotprod(location).acos();
                let orient_axes: (Vec3, Vec3) = (orient_around.normalize(), orient_up.normalize().cross(&orient_around.normalize()));
                let longitude: f64 = orient_axes.0.dotprod(location).atan2(orient_axes.1.dotprod(location)) + PI;
                let texture_color: Rgba<u8> = get_texture_rgba(img, longitude, latitude);
                rgba_to_color(texture_color)
            },
            Material::Emitter(emitter) => emitter.spectrum(location),
        }
    }
    pub fn scatter(&self, inc_ray: &Ray, shape: &Shape, scatter_loc: Vec3, rng: &mut impl Rng) -> Ray {
        match *self {
            Material::Diffuse{albedo: _} => {
                let scatter_dir = shape.normal_at(scatter_loc) + random_vec3(rng);
                Ray::new(scatter_loc, scatter_dir)
            },
            Material::Metal{albedo: _, fuzz: fuzziness} => {
                let scatter_normal = shape.normal_at(scatter_loc);
                let scatter_dir: Vec3 = inc_ray.dir - 2.0 * scatter_normal.dotprod(&inc_ray.dir) * scatter_normal;
                let fuzzified_dir = fuzzify(fuzziness, scatter_dir, scatter_normal, rng);
                Ray::new(scatter_loc, fuzzified_dir)
            },
            Material::Dielectric{refractive_index: r_idx} => {
                let scatter_normal = shape.normal_at(scatter_loc);
                let inc_cos = scatter_normal.dotprod(&inc_ray.dir); // -ve the usual for most ray-tracers
                let inc_dir_perp: Vec3 = inc_ray.dir - inc_cos*scatter_normal;
                let mut refract_ratio = r_idx; // default ray going from inside to outside so fewer divisions
                let sign_inc = inc_cos.signum(); // needed for determining scattered ray parallel direction
                if sign_inc < 0.0 {refract_ratio = 1.0/r_idx}; // refract from the outside
                let scatter_dir_perp = refract_ratio * inc_dir_perp;
                let scatter_sin2: f64 = scatter_dir_perp.dotprod(&scatter_dir_perp); // no sqrt needed

                if scatter_sin2 > 1.0 || schlick(inc_cos, refract_ratio)  {
                    // total internal reflection
                    let scatter_dir: Vec3 = inc_dir_perp - inc_cos * scatter_normal;
                    return Ray::new(scatter_loc, scatter_dir);
                };
                // refraction
                // refracted ray goes in the same direction as inc ray so sign of cos is the same
                let scatter_cos: f64 = sign_inc * (1.0 - scatter_sin2).sqrt();
                let scatter_dir = scatter_dir_perp + scatter_cos * scatter_normal;
                Ray::new(scatter_loc, scatter_dir)
            },
            Material::TextureMap { .. } => {
                let scatter_dir = shape.normal_at(scatter_loc) + random_vec3(rng);
                Ray::new(scatter_loc, scatter_dir)
            },
            _ => {
                panic!("Attempted to access scatter for Material without scattering implemented")
            },
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

fn fuzzify(fuzziness: f64, scatter_dir: Vec3, scatter_normal: Vec3, rng: &mut impl Rng) -> Vec3 {
    let fuzzy_dir = scatter_dir + (fuzziness * random_vec3(rng));
    if fuzzy_dir.dotprod(&scatter_normal) > 0.0 {
        fuzzy_dir
    } else { fuzzify(fuzziness, scatter_dir, scatter_normal, rng) }
}

fn random_vec3(rng: &mut impl Rng) -> Vec3 {
    let v: (f64, f64, f64) = rng.gen();
    let rand_vec3 = 2.0 * Vec3(v.0 - 0.5, v.1 - 0.5, v.2 - 0.5);
    if rand_vec3.norm() > 1.0 {
        return random_vec3(rng);
    };
    rand_vec3.normalize()
}

fn get_texture_rgba(image: &DynamicImage, longitude_rad: f64, latitude_rad: f64) -> Rgba<u8> {
    let dimensions: (u32, u32) = image.dimensions();

    let (pixel_column, pixel_row): (f64, f64) =
        (0.5 * longitude_rad / PI * (dimensions.0 as f64), latitude_rad / PI * (dimensions.1 as f64));

    image.get_pixel(pixel_column as u32 % dimensions.0, pixel_row as u32 % dimensions.1)
}

fn get_temperature(map: &[Vec<f64>], latitude_rad: f64, longitude_rad: f64) -> f64 {
    let row_access = (latitude_rad / PI * (map.len() as f64)) as usize % map.len();
    let column_access = (0.5 * longitude_rad / PI * (map[row_access].len() as f64)) as usize % map[row_access].len();
    map[row_access][column_access]
}

#[cfg(test)]
mod tests {
}