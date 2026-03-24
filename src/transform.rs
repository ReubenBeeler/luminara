use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::vec3::Vec3;

/// Translates (moves) an object by an offset vector.
pub struct Translate {
    inner: Box<dyn Hittable>,
    offset: Vec3,
}

impl Translate {
    pub fn new(inner: Box<dyn Hittable>, offset: Vec3) -> Self {
        Self { inner, offset }
    }
}

impl Hittable for Translate {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let moved_ray = Ray::new(ray.origin - self.offset, ray.direction);
        let mut hit = self.inner.hit(&moved_ray, t_min, t_max)?;
        hit.point = hit.point + self.offset;
        Some(hit)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.inner.bounding_box().map(|bb| Aabb::new(bb.min + self.offset, bb.max + self.offset))
    }
}

/// Rotates an object around the Y axis.
pub struct RotateY {
    inner: Box<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Option<Aabb>,
}

impl RotateY {
    pub fn new(inner: Box<dyn Hittable>, angle_degrees: f64) -> Self {
        let radians = angle_degrees.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        let bbox = inner.bounding_box().map(|bb| {
            let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
            let mut max = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        let x = i as f64 * bb.max.x + (1 - i) as f64 * bb.min.x;
                        let y = j as f64 * bb.max.y + (1 - j) as f64 * bb.min.y;
                        let z = k as f64 * bb.max.z + (1 - k) as f64 * bb.min.z;

                        let new_x = cos_theta * x + sin_theta * z;
                        let new_z = -sin_theta * x + cos_theta * z;

                        min = min.min(Vec3::new(new_x, y, new_z));
                        max = max.max(Vec3::new(new_x, y, new_z));
                    }
                }
            }
            Aabb::new(min, max)
        });

        Self { inner, sin_theta, cos_theta, bbox }
    }
}

impl Hittable for RotateY {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        // Rotate ray into object space (inverse rotation)
        let origin = Vec3::new(
            self.cos_theta * ray.origin.x - self.sin_theta * ray.origin.z,
            ray.origin.y,
            self.sin_theta * ray.origin.x + self.cos_theta * ray.origin.z,
        );
        let direction = Vec3::new(
            self.cos_theta * ray.direction.x - self.sin_theta * ray.direction.z,
            ray.direction.y,
            self.sin_theta * ray.direction.x + self.cos_theta * ray.direction.z,
        );

        let rotated_ray = Ray::new(origin, direction);
        let mut hit = self.inner.hit(&rotated_ray, t_min, t_max)?;

        // Rotate hit point and normal back to world space
        hit.point = Vec3::new(
            self.cos_theta * hit.point.x + self.sin_theta * hit.point.z,
            hit.point.y,
            -self.sin_theta * hit.point.x + self.cos_theta * hit.point.z,
        );
        hit.normal = Vec3::new(
            self.cos_theta * hit.normal.x + self.sin_theta * hit.normal.z,
            hit.normal.y,
            -self.sin_theta * hit.normal.x + self.cos_theta * hit.normal.z,
        );

        Some(hit)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.bbox
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::sphere::Sphere;
    use crate::vec3::{Color, Point3};

    #[test]
    fn test_translate() {
        let sphere = Sphere::new(Point3::ZERO, 1.0, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        let translated = Translate::new(Box::new(sphere), Vec3::new(5.0, 0.0, 0.0));

        // Ray aimed at translated position should hit
        let ray = Ray::new(Point3::new(5.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(translated.hit(&ray, 0.001, f64::INFINITY).is_some());

        // Ray aimed at original position should miss
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(translated.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_rotate_y() {
        let sphere = Sphere::new(Point3::new(2.0, 0.0, 0.0), 0.5, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        let rotated = RotateY::new(Box::new(sphere), 90.0);

        // After 90° rotation, sphere at (2,0,0) should be at (0,0,-2)
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(rotated.hit(&ray, 0.001, f64::INFINITY).is_some());
    }
}
