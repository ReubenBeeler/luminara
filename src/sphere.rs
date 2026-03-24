use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material: Box<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, material: Box<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
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

        // Find the nearest root in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let point = ray.at(root);
        let outward_normal = (point - self.center) / self.radius;
        // Spherical UV mapping
        let theta = (-outward_normal.y).acos();
        let phi = (-outward_normal.z).atan2(outward_normal.x) + std::f64::consts::PI;
        let u = phi / (2.0 * std::f64::consts::PI);
        let v = theta / std::f64::consts::PI;
        Some(HitRecord::new(
            ray,
            point,
            outward_normal,
            root,
            u,
            v,
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let r = Vec3::new(self.radius, self.radius, self.radius);
        Some(Aabb::new(self.center - r, self.center + r))
    }
}

/// A sphere that moves linearly between two centers over time [0, 1].
pub struct MovingSphere {
    pub center0: Point3,
    pub center1: Point3,
    pub radius: f64,
    pub material: Box<dyn Material>,
}

impl MovingSphere {
    pub fn new(center0: Point3, center1: Point3, radius: f64, material: Box<dyn Material>) -> Self {
        Self { center0, center1, radius, material }
    }

    fn center_at(&self, time: f64) -> Point3 {
        self.center0 + (self.center1 - self.center0) * time
    }
}

impl Hittable for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let center = self.center_at(ray.time);
        let oc = ray.origin - center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
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
        let outward_normal = (point - center) / self.radius;
        let theta = (-outward_normal.y).acos();
        let phi = (-outward_normal.z).atan2(outward_normal.x) + std::f64::consts::PI;
        let u = phi / (2.0 * std::f64::consts::PI);
        let v = theta / std::f64::consts::PI;
        Some(HitRecord::new(ray, point, outward_normal, root, u, v, self.material.as_ref()))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let r = Vec3::new(self.radius, self.radius, self.radius);
        let box0 = Aabb::new(self.center0 - r, self.center0 + r);
        let box1 = Aabb::new(self.center1 - r, self.center1 + r);
        Some(Aabb::surrounding(&box0, &box1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    fn test_sphere() -> Sphere {
        Sphere::new(
            Point3::new(1.0, 2.0, 3.0),
            0.5,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        )
    }

    #[test]
    fn bounding_box_is_correct() {
        let sphere = test_sphere();
        let bbox = sphere.bounding_box().unwrap();
        // min should be center - radius
        assert!((bbox.min.x - 0.5).abs() < 1e-6);
        assert!((bbox.min.y - 1.5).abs() < 1e-6);
        assert!((bbox.min.z - 2.5).abs() < 1e-6);
        // max should be center + radius
        assert!((bbox.max.x - 1.5).abs() < 1e-6);
        assert!((bbox.max.y - 2.5).abs() < 1e-6);
        assert!((bbox.max.z - 3.5).abs() < 1e-6);
    }

    #[test]
    fn uv_coordinates_in_unit_range() {
        let sphere = Sphere::new(
            Point3::new(0.0, 0.0, 0.0),
            1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );

        // Test rays from multiple directions to sample different UV points
        let directions = [
            Vec3::new(0.0, 0.0, -1.0),  // front
            Vec3::new(0.0, 0.0, 1.0),   // back
            Vec3::new(1.0, 0.0, 0.0),   // right
            Vec3::new(-1.0, 0.0, 0.0),  // left
            Vec3::new(0.0, 1.0, 0.0),   // top (nearly)
            Vec3::new(0.0, -1.0, 0.0),  // bottom (nearly)
            Vec3::new(1.0, 1.0, 1.0),   // diagonal
            Vec3::new(-1.0, -1.0, -1.0),
        ];

        for dir in &directions {
            let origin = Point3::new(0.0, 0.0, 0.0) - *dir * 3.0;
            let ray = Ray::new(origin, *dir);
            if let Some(hit) = sphere.hit(&ray, 0.001, f64::INFINITY) {
                assert!(
                    hit.u >= 0.0 && hit.u <= 1.0,
                    "u={} out of [0,1] for dir {:?}",
                    hit.u,
                    dir
                );
                assert!(
                    hit.v >= 0.0 && hit.v <= 1.0,
                    "v={} out of [0,1] for dir {:?}",
                    hit.v,
                    dir
                );
            }
        }
    }

    #[test]
    fn moving_sphere_at_time_zero() {
        let ms = MovingSphere::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // At time=0, center is at origin
        let ray = Ray::with_time(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0), 0.0);
        assert!(ms.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn moving_sphere_at_time_one() {
        let ms = MovingSphere::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // At time=1, center is at (10,0,0)
        let ray = Ray::with_time(Point3::new(10.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0), 1.0);
        assert!(ms.hit(&ray, 0.001, f64::INFINITY).is_some());
        // Should miss at origin at time=1
        let ray2 = Ray::with_time(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0), 1.0);
        assert!(ray2.origin.x.abs() < 0.001); // just verifying
        assert!(ms.hit(&ray2, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn moving_sphere_bbox_encompasses_both() {
        let ms = MovingSphere::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let bb = ms.bounding_box().unwrap();
        assert!(bb.min.x <= -1.0);
        assert!(bb.max.x >= 11.0);
    }

    #[test]
    fn sphere_hit_returns_none_for_miss() {
        let sphere = test_sphere();
        // Ray that misses entirely
        let ray = Ray::new(Point3::new(100.0, 100.0, 100.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(sphere.hit(&ray, 0.001, f64::INFINITY).is_none());
    }
}
