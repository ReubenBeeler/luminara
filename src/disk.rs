use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A disk (finite circular plane) defined by center, normal, and radius.
pub struct Disk {
    pub center: Point3,
    pub normal: Vec3,
    pub radius: f64,
    pub material: Box<dyn Material>,
}

impl Disk {
    pub fn new(center: Point3, normal: Vec3, radius: f64, material: Box<dyn Material>) -> Self {
        let normal = if normal.near_zero() {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            normal.unit()
        };
        Self {
            center,
            normal,
            radius,
            material,
        }
    }
}

impl Hittable for Disk {
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
        if offset.length_squared() > self.radius * self.radius {
            return None;
        }

        // UV: map disk to [0,1] x [0,1] using polar coordinates
        let r = offset.length() / self.radius;
        let theta = offset.z.atan2(offset.x);
        let u = (theta / (2.0 * std::f64::consts::PI)) + 0.5;
        let v = r;

        Some(HitRecord::new(
            ray, point, self.normal, t, u, v, self.material.as_ref(),
        ))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        // Conservative AABB: center +/- radius on all axes
        let r = Vec3::new(self.radius, self.radius, self.radius);
        Some(Aabb::new(self.center - r, self.center + r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_disk_hit() {
        let disk = Disk::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        assert!(disk.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_disk_miss_outside_radius() {
        let disk = Disk::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(5.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        assert!(disk.hit(&ray, 0.001, f64::INFINITY).is_none());
    }
}
