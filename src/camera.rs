use rand::Rng;

use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: f64,
}

pub struct CameraConfig {
    pub look_from: Point3,
    pub look_at: Point3,
    pub vup: Vec3,
    pub vfov_degrees: f64,
    pub aspect_ratio: f64,
    pub aperture: f64,
    pub focus_dist: f64,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            look_from: Point3::new(0.0, 1.0, 3.0),
            look_at: Point3::new(0.0, 0.0, 0.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            vfov_degrees: 40.0,
            aspect_ratio: 16.0 / 9.0,
            aperture: 0.0,
            focus_dist: 1.0,
        }
    }
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        let theta = config.vfov_degrees.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = config.aspect_ratio * viewport_height;

        let w = (config.look_from - config.look_at).unit();
        let u = config.vup.cross(w).unit();
        let v = w.cross(u);

        let horizontal = u * viewport_width * config.focus_dist;
        let vertical = v * viewport_height * config.focus_dist;
        let lower_left_corner =
            config.look_from - horizontal / 2.0 - vertical / 2.0 - w * config.focus_dist;

        Self {
            origin: config.look_from,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius: config.aperture / 2.0,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64, rng: &mut impl Rng) -> Ray {
        let rd = Vec3::random_in_unit_disk(rng) * self.lens_radius;
        let offset = self.u * rd.x + self.v * rd.y;
        Ray::new(
            self.origin + offset,
            self.lower_left_corner + self.horizontal * s + self.vertical * t
                - self.origin
                - offset,
        )
    }
}
