use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// An ellipsoid centered at a point with three axis radii.
pub struct Ellipsoid {
    pub center: Point3,
    pub radii: Vec3,
    pub material: Box<dyn Material>,
}

impl Ellipsoid {
    pub fn new(center: Point3, radii: Vec3, material: Box<dyn Material>) -> Self {
        Self { center, radii, material }
    }
}

impl Hittable for Ellipsoid {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        // Transform ray into unit-sphere space by dividing by radii
        let inv_r = Vec3::new(1.0 / self.radii.x, 1.0 / self.radii.y, 1.0 / self.radii.z);
        let oc = Vec3::new(
            (ray.origin.x - self.center.x) * inv_r.x,
            (ray.origin.y - self.center.y) * inv_r.y,
            (ray.origin.z - self.center.z) * inv_r.z,
        );
        let dir = Vec3::new(
            ray.direction.x * inv_r.x,
            ray.direction.y * inv_r.y,
            ray.direction.z * inv_r.z,
        );

        let a = dir.length_squared();
        let half_b = oc.dot(dir);
        let c = oc.length_squared() - 1.0;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let point = ray.at(root);
        // Normal: gradient of ellipsoid equation
        let outward_normal = Vec3::new(
            (point.x - self.center.x) / (self.radii.x * self.radii.x),
            (point.y - self.center.y) / (self.radii.y * self.radii.y),
            (point.z - self.center.z) / (self.radii.z * self.radii.z),
        ).unit();

        let theta = (-outward_normal.y).acos();
        let phi = (-outward_normal.z).atan2(outward_normal.x) + std::f64::consts::PI;
        let u = phi / (2.0 * std::f64::consts::PI);
        let v = theta / std::f64::consts::PI;

        Some(HitRecord::new(ray, point, outward_normal, root, u, v, self.material.as_ref()))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(Aabb::new(self.center - self.radii, self.center + self.radii))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_ellipsoid_hit() {
        let ell = Ellipsoid::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 1.0, 1.0),
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(ell.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_ellipsoid_bounding_box() {
        let ell = Ellipsoid::new(
            Point3::new(1.0, 2.0, 3.0),
            Vec3::new(0.5, 1.0, 0.3),
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let bb = ell.bounding_box().unwrap();
        assert!((bb.min.x - 0.5).abs() < 1e-6);
        assert!((bb.max.y - 3.0).abs() < 1e-6);
    }
}
