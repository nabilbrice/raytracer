use serde::{Serialize, Deserialize};

use crate::rgba_to_color;
use crate::{vector::Vec3, geometry::Surface};
use crate::geometry::Sphere;
use crate::color::Color;
use crate::ray::Ray;
use rand::{Rng, thread_rng};
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, RgbImage};
use std::f64::consts::PI;

use image::Rgba;

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
}

fn load_image(path_to_file: &str) -> image::DynamicImage {
    image::open(path_to_file).unwrap()
}

serde_with::serde_conv!(
    TextureMapFilePath,
    DynamicImage,
    |_map: &DynamicImage| "texturemap.jpeg",
    |path_to_file: &str| -> Result<_, std::convert::Infallible> {Ok(load_image(path_to_file))}
);


// Implement using enums?
impl Material {
    pub fn albedo(&self, location: &Vec3) -> Color {
        match self {
            Material::Diffuse{albedo: color} => *color,
            Material::Metal{albedo: color, fuzz: _} => *color,
            Material::Dielectric { refractive_index: _ } => Color::new(1.0, 1.0, 1.0),
            Material::TextureMap {map: img, orient_up, orient_around} => {
                let latitude: f64 = PI * orient_up.dotprod(&location) + PI;
                let longitude: f64 = PI * orient_around.dotprod(&location) + PI;
                let texture_color: Rgba<u8> = get_texture_rgba(&img, longitude, latitude);
                rgba_to_color(texture_color)
            },
        }
    }
    pub fn scatter<T: Surface>(&self, inc_ray: &Ray, shape: &T, scatter_loc: Vec3) -> Ray {
        match *self {
            Material::Diffuse{albedo: _} => {
                let scatter_dir = shape.normal_at(scatter_loc) + random_vec3();
                return Ray::new(scatter_loc, scatter_dir)
            },
            Material::Metal{albedo: _, fuzz: fuzziness} => {
                let scatter_normal = shape.normal_at(scatter_loc);
                let scatter_dir: Vec3 = inc_ray.dir - 2.0 * scatter_normal.dotprod(&inc_ray.dir) * scatter_normal;
                let fuzzified_dir = fuzzify(fuzziness, scatter_dir, scatter_normal);
                return Ray::new(scatter_loc, fuzzified_dir)
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
                }
                else {
                    // refraction
                    // refracted ray goes in the same direction as inc ray so sign of cos is the same
                    let scatter_cos: f64 = sign_inc * (1.0 - scatter_sin2).sqrt();
                    let scatter_dir = scatter_dir_perp + scatter_cos * scatter_normal;
                    return Ray::new(scatter_loc, scatter_dir);
                };
            },
            Material::TextureMap { .. } => {
                let scatter_dir = shape.normal_at(scatter_loc) + random_vec3();
                return Ray::new(scatter_loc, scatter_dir)
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

fn fuzzify(fuzziness: f64, scatter_dir: Vec3, scatter_normal: Vec3) -> Vec3 {
    let fuzzy_dir = scatter_dir + (fuzziness * random_vec3());
    if fuzzy_dir.dotprod(&scatter_normal) > 0.0 {
        fuzzy_dir
    } else { fuzzify(fuzziness, scatter_dir, scatter_normal) }
}

fn random_vec3() -> Vec3 {
    let v: (f64, f64, f64) = thread_rng().gen();
    let rand_vec3 = 2.0 * Vec3(v.0 - 0.5, v.1 - 0.5, v.2 - 0.5);
    if rand_vec3.norm() > 1.0 {
        return random_vec3();
    };
    return rand_vec3.normalize();
}

fn get_texture_rgba(image: &DynamicImage, longitude_rad: f64, latitude_rad: f64) -> Rgba<u8> {
    let dimensions: (u32, u32) = image.dimensions();

    let (pixel_column, pixel_row): (f64, f64) =
        (longitude_rad / PI * (dimensions.0 as f64), 0.5 * latitude_rad / PI * (dimensions.1 as f64));

    image.get_pixel(pixel_column as u32 % dimensions.0, pixel_row as u32 % dimensions.1)
}

#[test]
fn test_scattering() {
    let mat = Material::Dielectric{ refractive_index: 1.5};
    let sph = Sphere::new(Vec3(0.0,0.0,0.0), 0.5, mat);
    let incoming_ray = Ray::new(Vec3(0.42,0.0,-10.0),Vec3(0.0,0.0,1.0));
    let intersection = sph.intersect(&incoming_ray);
    let point = incoming_ray.position_at(intersection);
    let scattered_ray = sph.material.scatter(&incoming_ray, &sph, point);
    println!("{:?}", scattered_ray.orig);
    println!("{:?}", scattered_ray.dir);
    let second_intersection = sph.intersect(&scattered_ray);
    let second_point = scattered_ray.position_at(second_intersection);
    let second_scattered_ray = sph.material.scatter(&scattered_ray, &sph, second_point);
    println!("{:?}",second_scattered_ray.orig);
    println!("{:?}",second_scattered_ray.dir);
    let third_intersection = sph.intersect(&second_scattered_ray);
    let third_point = second_scattered_ray.position_at(third_intersection);
    let third_scattered_ray = sph.material.scatter(&second_scattered_ray, &sph, third_point);
    println!("{:?}",third_scattered_ray.orig);
    println!("{:?}",third_scattered_ray.dir);
    let trial_ray = Ray::new(Vec3(0.4,0.0,0.0), Vec3(0.0,0.0,-1.0));
    let trial_intersection = sph.intersect(&trial_ray);
    let trial_point = trial_ray.position_at(trial_intersection);
    let new_ray = sph.material.scatter(&trial_ray, &sph, trial_point);
    println!("{:?}", new_ray.orig);
}