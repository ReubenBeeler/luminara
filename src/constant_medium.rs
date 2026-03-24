use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::{Material, RngCore, Scatter};
use crate::ray::Ray;
use crate::vec3::{Color, Vec3};

/// A constant-density volume (fog/smoke) wrapping a boundary shape.
/// Rays passing through have a probability of scattering based on density.
pub struct ConstantMedium {
    boundary: Box<dyn Hittable>,
    neg_inv_density: f64,
    phase_function: Isotropic,
}

impl ConstantMedium {
    pub fn new(boundary: Box<dyn Hittable>, density: f64, color: Color) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Isotropic { albedo: color },
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        // Find entry and exit points of the ray through the boundary
        let hit1 = self.boundary.hit(ray, f64::NEG_INFINITY, f64::INFINITY)?;
        let hit2 = self.boundary.hit(ray, hit1.t + 0.0001, f64::INFINITY)?;

        let mut t1 = hit1.t.max(t_min);
        let t2 = hit2.t.min(t_max);

        if t1 >= t2 {
            return None;
        }

        if t1 < 0.0 {
            t1 = 0.0;
        }

        let ray_length = ray.direction.length();
        let distance_inside = (t2 - t1) * ray_length;

        // Use a simple deterministic approach based on ray parameters for thread safety
        // We use a hash of the ray to get a pseudo-random value
        let hash_val = (ray.origin.x * 73856093.0 + ray.origin.y * 19349663.0
            + ray.origin.z * 83492791.0 + ray.direction.x * 47392381.0)
            .fract()
            .abs();
        let hit_distance = self.neg_inv_density * (1.0 - hash_val).max(1e-10).ln();

        if hit_distance > distance_inside {
            return None;
        }

        let t = t1 + hit_distance / ray_length;
        let point = ray.at(t);

        Some(HitRecord::new(
            ray,
            point,
            Vec3::new(1.0, 0.0, 0.0), // arbitrary normal
            t,
            0.0,
            0.0,
            &self.phase_function,
        ))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.boundary.bounding_box()
    }
}

/// Isotropic scattering material — scatters in a random direction.
pub struct Isotropic {
    pub albedo: Color,
}

impl Material for Isotropic {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        let mut rng_adapter = IsotropicRng(rng);
        Some(Scatter {
            ray: Ray::with_time(hit.point, Vec3::random_in_unit_sphere(&mut rng_adapter), ray.time),
            attenuation: self.albedo,
        })
    }
}

struct IsotropicRng<'a>(&'a mut dyn RngCore);

impl rand::RngCore for IsotropicRng<'_> {
    fn next_u32(&mut self) -> u32 {
        (self.0.next_f64() * u32::MAX as f64) as u32
    }
    fn next_u64(&mut self) -> u64 {
        (self.0.next_f64() * u64::MAX as f64) as u64
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest.iter_mut() {
            *byte = (self.0.next_f64() * 256.0) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::sphere::Sphere;
    use crate::vec3::Point3;

    #[test]
    fn constant_medium_has_boundary_bbox() {
        let boundary = Box::new(Sphere::new(
            Point3::ZERO, 1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        ));
        let medium = ConstantMedium::new(boundary, 0.5, Color::new(1.0, 1.0, 1.0));
        let bb = medium.bounding_box().unwrap();
        assert!((bb.min.x - -1.0).abs() < 1e-4);
        assert!((bb.max.x - 1.0).abs() < 1e-4);
    }

    #[test]
    fn constant_medium_ray_through() {
        let boundary = Box::new(Sphere::new(
            Point3::ZERO, 5.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        ));
        // Very high density — should almost always scatter
        let medium = ConstantMedium::new(boundary, 100.0, Color::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3::new(0.0, 0.0, -10.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = medium.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some());
    }
}
