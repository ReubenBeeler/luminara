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
