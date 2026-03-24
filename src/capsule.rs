use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable, HittableList};
use crate::material::Material;
use crate::sphere::Sphere;
use crate::cylinder::Cylinder;
use crate::vec3::Point3;

/// A capsule (rounded cylinder): a cylinder capped with hemispheres.
/// Implemented as a composite of a cylinder and two spheres.
pub struct Capsule {
    parts: HittableList,
    bbox: Aabb,
}

impl Capsule {
    pub fn new(
        center: Point3,
        radius: f64,
        height: f64,
        material_factory: impl Fn() -> Box<dyn Material>,
    ) -> Self {
        let mut parts = HittableList::new();

        // Cylinder body
        parts.add(Box::new(Cylinder::new(
            center,
            radius,
            center.y,
            center.y + height,
            material_factory(),
        )));

        // Bottom hemisphere
        parts.add(Box::new(Sphere::new(
            Point3::new(center.x, center.y, center.z),
            radius,
            material_factory(),
        )));

        // Top hemisphere
        parts.add(Box::new(Sphere::new(
            Point3::new(center.x, center.y + height, center.z),
            radius,
            material_factory(),
        )));

        let bbox = Aabb::new(
            Point3::new(center.x - radius, center.y - radius, center.z - radius),
            Point3::new(center.x + radius, center.y + height + radius, center.z + radius),
        );

        Self { parts, bbox }
    }
}

impl Hittable for Capsule {
    fn hit(&self, ray: &crate::ray::Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        self.parts.hit(ray, t_min, t_max)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::ray::Ray;
    use crate::vec3::Color;

    #[test]
    fn test_capsule_hit() {
        let capsule = Capsule::new(
            Point3::new(0.0, 0.0, 0.0),
            0.5,
            2.0,
            || Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Hit the top sphere
        let ray = Ray::new(Point3::new(0.0, 2.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(capsule.hit(&ray, 0.001, f64::INFINITY).is_some());

        // Miss entirely
        let ray = Ray::new(Point3::new(5.0, 5.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(capsule.hit(&ray, 0.001, f64::INFINITY).is_none());
    }
}
