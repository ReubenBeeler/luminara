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
    panorama: bool,
    // Basis vectors for panorama mode
    forward: Vec3,
    right: Vec3,
    up: Vec3,
}

pub struct CameraConfig {
    pub look_from: Point3,
    pub look_at: Point3,
    pub vup: Vec3,
    pub vfov_degrees: f64,
    pub aspect_ratio: f64,
    pub aperture: f64,
    pub focus_dist: f64,
    pub panorama: bool,
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
            panorama: false,
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
            panorama: config.panorama,
            forward: -w,
            right: u,
            up: v,
        }
    }

    pub fn set_panorama(&mut self, panorama: bool) {
        self.panorama = panorama;
    }

    pub fn get_ray(&self, s: f64, t: f64, rng: &mut impl Rng) -> Ray {
        if self.panorama {
            // Equirectangular panoramic: s maps to longitude, t maps to latitude
            let theta = std::f64::consts::PI * (1.0 - t); // latitude: 0=top(pi) to 1=bottom(0)
            let phi = 2.0 * std::f64::consts::PI * s - std::f64::consts::PI; // longitude: -pi to pi
            let dir = self.forward * (theta.sin() * phi.cos())
                + self.right * (theta.sin() * phi.sin())
                + self.up * theta.cos();
            Ray::with_time(self.origin, dir.unit(), rng.random::<f64>())
        } else {
            let rd = Vec3::random_in_unit_disk(rng) * self.lens_radius;
            let offset = self.u * rd.x + self.v * rd.y;
            Ray::with_time(
                self.origin + offset,
                self.lower_left_corner + self.horizontal * s + self.vertical * t
                    - self.origin
                    - offset,
                rng.random::<f64>(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn camera_ray_through_center() {
        // Zero aperture camera looking along -Z
        let config = CameraConfig {
            look_from: Point3::new(0.0, 0.0, 0.0),
            look_at: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            vfov_degrees: 90.0,
            aspect_ratio: 2.0,
            aperture: 0.0,
            focus_dist: 1.0,
            panorama: false,
        };
        let camera = Camera::new(config);
        let mut rng = SmallRng::seed_from_u64(0);

        // Ray through center (s=0.5, t=0.5) should go roughly along -Z
        let ray = camera.get_ray(0.5, 0.5, &mut rng);
        let dir = ray.direction.unit();
        assert!(
            (dir.x).abs() < 1e-6,
            "Center ray x should be ~0, got {}",
            dir.x
        );
        assert!(
            (dir.y).abs() < 1e-6,
            "Center ray y should be ~0, got {}",
            dir.y
        );
        assert!(
            dir.z < 0.0,
            "Center ray should go in -Z direction, got {}",
            dir.z
        );

        // Ray through bottom-left corner (s=0, t=0) should go left and down
        let ray_bl = camera.get_ray(0.0, 0.0, &mut rng);
        let dir_bl = ray_bl.direction.unit();
        assert!(dir_bl.x < 0.0, "Bottom-left ray should go left");
        assert!(dir_bl.y < 0.0, "Bottom-left ray should go down");

        // Ray through top-right (s=1, t=1) should go right and up
        let ray_tr = camera.get_ray(1.0, 1.0, &mut rng);
        let dir_tr = ray_tr.direction.unit();
        assert!(dir_tr.x > 0.0, "Top-right ray should go right");
        assert!(dir_tr.y > 0.0, "Top-right ray should go up");
    }

    #[test]
    fn camera_ray_origin_with_zero_aperture() {
        let config = CameraConfig {
            look_from: Point3::new(1.0, 2.0, 3.0),
            look_at: Point3::new(0.0, 0.0, 0.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            vfov_degrees: 40.0,
            aspect_ratio: 1.0,
            aperture: 0.0,
            focus_dist: 1.0,
            panorama: false,
        };
        let camera = Camera::new(config);
        let mut rng = SmallRng::seed_from_u64(99);

        let ray = camera.get_ray(0.5, 0.5, &mut rng);
        // With zero aperture, origin should be exactly look_from
        assert!((ray.origin.x - 1.0).abs() < 1e-6);
        assert!((ray.origin.y - 2.0).abs() < 1e-6);
        assert!((ray.origin.z - 3.0).abs() < 1e-6);
    }
}
