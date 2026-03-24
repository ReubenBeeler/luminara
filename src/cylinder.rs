use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A finite cylinder aligned along the Y axis.
pub struct Cylinder {
    pub center: Point3,
    pub radius: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub material: Box<dyn Material>,
}

impl Cylinder {
    pub fn new(center: Point3, radius: f64, y_min: f64, y_max: f64, material: Box<dyn Material>) -> Self {
        Self { center, radius, y_min, y_max, material }
    }
}

impl Hittable for Cylinder {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        // Solve for ray-cylinder intersection in XZ plane
        let dx = ray.direction.x;
        let dz = ray.direction.z;
        let ox = ray.origin.x - self.center.x;
        let oz = ray.origin.z - self.center.z;

        let a = dx * dx + dz * dz;
        let half_b = ox * dx + oz * dz;
        let c = ox * ox + oz * oz - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Try both roots
        for sign in [-1.0, 1.0] {
            let root = (-half_b + sign * sqrtd) / a;
            if root < t_min || root > t_max {
                continue;
            }

            let y = ray.origin.y + root * ray.direction.y;
            if y < self.y_min || y > self.y_max {
                continue;
            }

            let point = ray.at(root);
            let outward_normal = Vec3::new(
                (point.x - self.center.x) / self.radius,
                0.0,
                (point.z - self.center.z) / self.radius,
            );

            // UV mapping: theta around cylinder, v along height
            let theta = (-(point.z - self.center.z)).atan2(point.x - self.center.x) + std::f64::consts::PI;
            let u = theta / (2.0 * std::f64::consts::PI);
            let v = (y - self.y_min) / (self.y_max - self.y_min);

            return Some(HitRecord::new(
                ray, point, outward_normal, root, u, v, self.material.as_ref(),
            ));
        }

        None
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.center.x - self.radius, self.y_min, self.center.z - self.radius),
            Point3::new(self.center.x + self.radius, self.y_max, self.center.z + self.radius),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_cylinder_hit() {
        let cyl = Cylinder::new(
            Point3::new(0.0, 0.0, 0.0), 1.0, 0.0, 2.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(3.0, 1.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = cyl.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.point.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cylinder_miss_above() {
        let cyl = Cylinder::new(
            Point3::new(0.0, 0.0, 0.0), 1.0, 0.0, 2.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(3.0, 5.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        assert!(cyl.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_cylinder_bounding_box() {
        let cyl = Cylinder::new(
            Point3::new(1.0, 0.0, 2.0), 0.5, -1.0, 3.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let bb = cyl.bounding_box().unwrap();
        assert!((bb.min.x - 0.5).abs() < 1e-6);
        assert!((bb.max.x - 1.5).abs() < 1e-6);
        assert!((bb.min.y - -1.0).abs() < 1e-6);
        assert!((bb.max.y - 3.0).abs() < 1e-6);
    }
}
