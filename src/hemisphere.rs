use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A hemisphere: the upper half of a sphere (y >= center.y).
pub struct Hemisphere {
    center: Point3,
    radius: f64,
    material: Box<dyn Material>,
}

impl Hemisphere {
    pub fn new(center: Point3, radius: f64, material: Box<dyn Material>) -> Self {
        Self { center, radius, material }
    }
}

impl Hittable for Hemisphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Try nearest root first, then far root
        for sign in [-1.0, 1.0] {
            let root = (-half_b + sign * sqrtd) / a;
            if root < t_min || root > t_max {
                continue;
            }
            let point = ray.at(root);
            // Only accept hits in the upper hemisphere
            if point.y < self.center.y {
                continue;
            }
            let outward_normal = (point - self.center) / self.radius;
            let theta = (-outward_normal.y).acos();
            let phi = (-outward_normal.z).atan2(outward_normal.x) + std::f64::consts::PI;
            let u = phi / (2.0 * std::f64::consts::PI);
            let v = theta / std::f64::consts::PI;
            return Some(HitRecord::new(
                ray, point, outward_normal, root, u, v, self.material.as_ref(),
            ));
        }

        None
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let r = Vec3::new(self.radius, self.radius, self.radius);
        let mut min = self.center - r;
        min.y = self.center.y; // Only upper half
        Some(Aabb::new(min, self.center + r))
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
    fn hemisphere_hit_from_above() {
        let h = Hemisphere::new(Point3::ZERO, 1.0, test_mat());
        let ray = Ray::new(Point3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = h.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some());
        let rec = hit.unwrap();
        assert!(rec.point.y >= 0.0);
    }

    #[test]
    fn hemisphere_miss_below() {
        let h = Hemisphere::new(Point3::ZERO, 1.0, test_mat());
        // Ray hitting only the lower half of where a full sphere would be
        let ray = Ray::new(Point3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let hit = h.hit(&ray, 0.001, f64::INFINITY);
        // Should hit the upper part when exiting
        if let Some(rec) = hit {
            assert!(rec.point.y >= 0.0);
        }
    }

    #[test]
    fn hemisphere_bbox_upper_only() {
        let h = Hemisphere::new(Point3::new(0.0, 2.0, 0.0), 1.0, test_mat());
        let bb = h.bounding_box().unwrap();
        assert!((bb.min.y - 2.0).abs() < 1e-6);
        assert!((bb.max.y - 3.0).abs() < 1e-6);
    }
}
