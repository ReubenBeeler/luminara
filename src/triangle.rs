use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A triangle defined by three vertices.
pub struct Triangle {
    pub v0: Point3,
    pub v1: Point3,
    pub v2: Point3,
    pub material: Box<dyn Material>,
}

impl Triangle {
    pub fn new(v0: Point3, v1: Point3, v2: Point3, material: Box<dyn Material>) -> Self {
        Self { v0, v1, v2, material }
    }
}

impl Hittable for Triangle {
    /// Möller–Trumbore ray-triangle intersection.
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        if a.abs() < 1e-8 {
            return None; // Ray parallel to triangle
        }

        let f = 1.0 / a;
        let s = ray.origin - self.v0;
        let u = f * s.dot(h);

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * ray.direction.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);
        if t < t_min || t > t_max {
            return None;
        }

        let point = ray.at(t);
        let outward_normal = edge1.cross(edge2).unit();
        Some(HitRecord::new(
            ray,
            point,
            outward_normal,
            t,
            u,
            v,
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let min = self.v0.min(self.v1).min(self.v2) - Vec3::new(1e-4, 1e-4, 1e-4);
        let max = self.v0.max(self.v1).max(self.v2) + Vec3::new(1e-4, 1e-4, 1e-4);
        Some(Aabb::new(min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_triangle_hit() {
        let tri = Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(0.2, 0.2, -1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(tri.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_triangle_miss() {
        let tri = Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(2.0, 2.0, -1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(tri.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_triangle_bounding_box() {
        let tri = Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let bb = tri.bounding_box().unwrap();
        assert!(bb.min.x < 0.001);
        assert!(bb.max.x > 0.999);
        assert!(bb.max.y > 0.999);
    }
}
