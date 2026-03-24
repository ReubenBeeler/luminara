use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A finite cone aligned along the Y axis, with apex at the top.
/// The cone narrows from radius at y_min to zero at y_max (apex).
pub struct Cone {
    pub center: Point3,
    pub radius: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub material: Box<dyn Material>,
}

impl Cone {
    pub fn new(center: Point3, radius: f64, y_min: f64, y_max: f64, material: Box<dyn Material>) -> Self {
        Self { center, radius, y_min, y_max, material }
    }
}

impl Hittable for Cone {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let height = self.y_max - self.y_min;
        if height.abs() < 1e-10 {
            return None;
        }

        // Cone equation: x^2 + z^2 = (r * (y_max - y) / height)^2
        let k = (self.radius / height) * (self.radius / height);
        let ox = ray.origin.x - self.center.x;
        let oz = ray.origin.z - self.center.z;
        let oy = ray.origin.y - self.y_max; // relative to apex

        let dx = ray.direction.x;
        let dz = ray.direction.z;
        let dy = ray.direction.y;

        let a = dx * dx + dz * dz - k * dy * dy;
        let half_b = ox * dx + oz * dz - k * oy * dy;
        let c = ox * ox + oz * oz - k * oy * oy;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

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
            let r_at_y = self.radius * (self.y_max - y) / height;
            let nx = (point.x - self.center.x) / r_at_y.max(1e-10);
            let nz = (point.z - self.center.z) / r_at_y.max(1e-10);
            let ny = self.radius / height;
            let outward_normal = Vec3::new(nx, ny, nz).unit();

            let theta = (-(point.z - self.center.z)).atan2(point.x - self.center.x) + std::f64::consts::PI;
            let u = theta / (2.0 * std::f64::consts::PI);
            let v = (y - self.y_min) / height;

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
    fn test_cone_hit() {
        let cone = Cone::new(
            Point3::new(0.0, 0.0, 0.0), 1.0, 0.0, 2.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray from the side hitting the cone body
        let ray = Ray::new(Point3::new(3.0, 0.5, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        assert!(cone.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_cone_miss_above_apex() {
        let cone = Cone::new(
            Point3::new(0.0, 0.0, 0.0), 1.0, 0.0, 2.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(3.0, 5.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        assert!(cone.hit(&ray, 0.001, f64::INFINITY).is_none());
    }
}
