use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// An annulus (ring/washer): a disk with a hole in the center.
pub struct Annulus {
    center: Point3,
    normal: Vec3,
    inner_radius: f64,
    outer_radius: f64,
    material: Box<dyn Material>,
}

impl Annulus {
    pub fn new(
        center: Point3,
        normal: Vec3,
        inner_radius: f64,
        outer_radius: f64,
        material: Box<dyn Material>,
    ) -> Self {
        let normal = if normal.near_zero() {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            normal.unit()
        };
        Self {
            center,
            normal,
            inner_radius,
            outer_radius,
            material,
        }
    }
}

impl Hittable for Annulus {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() < 1e-8 {
            return None;
        }

        let t = (self.center - ray.origin).dot(self.normal) / denom;
        if t < t_min || t > t_max {
            return None;
        }

        let point = ray.at(t);
        let offset = point - self.center;
        let dist_sq = offset.length_squared();

        if dist_sq < self.inner_radius * self.inner_radius
            || dist_sq > self.outer_radius * self.outer_radius
        {
            return None;
        }

        // UV: radial and angular coordinates
        let r = dist_sq.sqrt();
        let u_radial = (r - self.inner_radius) / (self.outer_radius - self.inner_radius);
        let theta = offset.z.atan2(offset.x);
        let u_angular = (theta / (2.0 * std::f64::consts::PI)) + 0.5;

        Some(HitRecord::new(
            ray,
            point,
            self.normal,
            t,
            u_angular,
            u_radial,
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let r = Vec3::new(self.outer_radius, self.outer_radius, self.outer_radius);
        Some(Aabb::new(self.center - r, self.center + r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    fn test_mat() -> Box<dyn Material> {
        Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)))
    }

    #[test]
    fn annulus_hit_in_ring() {
        let ring = Annulus::new(
            Point3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
            0.5,
            1.0,
            test_mat(),
        );
        // Hit at x=0.75 — between inner (0.5) and outer (1.0) radius
        let ray = Ray::new(Point3::new(0.75, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        assert!(ring.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn annulus_miss_in_hole() {
        let ring = Annulus::new(
            Point3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
            0.5,
            1.0,
            test_mat(),
        );
        // Hit at center — inside the hole
        let ray = Ray::new(Point3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        assert!(ring.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn annulus_miss_outside() {
        let ring = Annulus::new(
            Point3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
            0.5,
            1.0,
            test_mat(),
        );
        let ray = Ray::new(Point3::new(2.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        assert!(ring.hit(&ray, 0.001, f64::INFINITY).is_none());
    }
}
