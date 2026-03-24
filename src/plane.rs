use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// An infinite plane defined by a point and a normal.
pub struct Plane {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Box<dyn Material>,
}

impl Plane {
    pub fn new(point: Point3, normal: Vec3, material: Box<dyn Material>) -> Self {
        let normal = if normal.near_zero() {
            Vec3::new(0.0, 1.0, 0.0) // Fallback to up vector
        } else {
            normal.unit()
        };
        Self {
            point,
            normal,
            material,
        }
    }
}

impl Hittable for Plane {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let denom = self.normal.dot(ray.direction);

        // Ray is parallel to the plane.
        if denom.abs() < 1e-8 {
            return None;
        }

        let t = (self.point - ray.origin).dot(self.normal) / denom;
        if t < t_min || t > t_max {
            return None;
        }

        let point = ray.at(t);
        Some(HitRecord::new(
            ray,
            point,
            self.normal,
            t,
            0.0,
            0.0,
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        None // Infinite planes have no finite bounding box
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
    fn plane_hit_from_above() {
        let plane = Plane::new(Point3::ZERO, Vec3::new(0.0, 1.0, 0.0), test_mat());
        let ray = Ray::new(Point3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = plane.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some());
        assert!((hit.unwrap().t - 5.0).abs() < 1e-6);
    }

    #[test]
    fn plane_miss_parallel() {
        let plane = Plane::new(Point3::ZERO, Vec3::new(0.0, 1.0, 0.0), test_mat());
        let ray = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        assert!(plane.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn plane_no_bounding_box() {
        let plane = Plane::new(Point3::ZERO, Vec3::new(0.0, 1.0, 0.0), test_mat());
        assert!(plane.bounding_box().is_none());
    }

    #[test]
    fn plane_zero_normal_fallback() {
        let plane = Plane::new(Point3::ZERO, Vec3::ZERO, test_mat());
        // Should fallback to up normal
        assert!((plane.normal.y - 1.0).abs() < 1e-6);
    }
}
