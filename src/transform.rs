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
        hit.point += self.offset;
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

/// Rotates an object around the X axis.
pub struct RotateX {
    inner: Box<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Option<Aabb>,
}

impl RotateX {
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

                        let new_y = cos_theta * y - sin_theta * z;
                        let new_z = sin_theta * y + cos_theta * z;

                        min = min.min(Vec3::new(x, new_y, new_z));
                        max = max.max(Vec3::new(x, new_y, new_z));
                    }
                }
            }
            Aabb::new(min, max)
        });

        Self { inner, sin_theta, cos_theta, bbox }
    }
}

impl Hittable for RotateX {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let origin = Vec3::new(
            ray.origin.x,
            self.cos_theta * ray.origin.y + self.sin_theta * ray.origin.z,
            -self.sin_theta * ray.origin.y + self.cos_theta * ray.origin.z,
        );
        let direction = Vec3::new(
            ray.direction.x,
            self.cos_theta * ray.direction.y + self.sin_theta * ray.direction.z,
            -self.sin_theta * ray.direction.y + self.cos_theta * ray.direction.z,
        );

        let rotated_ray = Ray::with_time(origin, direction, ray.time);
        let mut hit = self.inner.hit(&rotated_ray, t_min, t_max)?;

        hit.point = Vec3::new(
            hit.point.x,
            self.cos_theta * hit.point.y - self.sin_theta * hit.point.z,
            self.sin_theta * hit.point.y + self.cos_theta * hit.point.z,
        );
        hit.normal = Vec3::new(
            hit.normal.x,
            self.cos_theta * hit.normal.y - self.sin_theta * hit.normal.z,
            self.sin_theta * hit.normal.y + self.cos_theta * hit.normal.z,
        );

        Some(hit)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.bbox
    }
}

/// Rotates an object around the Z axis.
pub struct RotateZ {
    inner: Box<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Option<Aabb>,
}

impl RotateZ {
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

                        let new_x = cos_theta * x - sin_theta * y;
                        let new_y = sin_theta * x + cos_theta * y;

                        min = min.min(Vec3::new(new_x, new_y, z));
                        max = max.max(Vec3::new(new_x, new_y, z));
                    }
                }
            }
            Aabb::new(min, max)
        });

        Self { inner, sin_theta, cos_theta, bbox }
    }
}

impl Hittable for RotateZ {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let origin = Vec3::new(
            self.cos_theta * ray.origin.x + self.sin_theta * ray.origin.y,
            -self.sin_theta * ray.origin.x + self.cos_theta * ray.origin.y,
            ray.origin.z,
        );
        let direction = Vec3::new(
            self.cos_theta * ray.direction.x + self.sin_theta * ray.direction.y,
            -self.sin_theta * ray.direction.x + self.cos_theta * ray.direction.y,
            ray.direction.z,
        );

        let rotated_ray = Ray::with_time(origin, direction, ray.time);
        let mut hit = self.inner.hit(&rotated_ray, t_min, t_max)?;

        hit.point = Vec3::new(
            self.cos_theta * hit.point.x - self.sin_theta * hit.point.y,
            self.sin_theta * hit.point.x + self.cos_theta * hit.point.y,
            hit.point.z,
        );
        hit.normal = Vec3::new(
            self.cos_theta * hit.normal.x - self.sin_theta * hit.normal.y,
            self.sin_theta * hit.normal.x + self.cos_theta * hit.normal.y,
            hit.normal.z,
        );

        Some(hit)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.bbox
    }
}

/// Uniformly scales an object around the origin.
pub struct Scale {
    inner: Box<dyn Hittable>,
    factor: f64,
    inv_factor: f64,
}

impl Scale {
    pub fn new(inner: Box<dyn Hittable>, factor: f64) -> Self {
        let factor = if factor.abs() < 1e-10 { 1.0 } else { factor };
        Self { inner, factor, inv_factor: 1.0 / factor }
    }
}

impl Hittable for Scale {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let scaled_ray = Ray::new(ray.origin * self.inv_factor, ray.direction * self.inv_factor);
        let mut hit = self.inner.hit(&scaled_ray, t_min, t_max)?;
        hit.point *= self.factor;
        hit.t *= self.factor;
        Some(hit)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.inner.bounding_box().map(|bb| Aabb::new(bb.min * self.factor, bb.max * self.factor))
    }
}

/// Non-uniformly scales an object along each axis independently.
pub struct NonUniformScale {
    inner: Box<dyn Hittable>,
    scale: Vec3,
    inv_scale: Vec3,
}

impl NonUniformScale {
    pub fn new(inner: Box<dyn Hittable>, sx: f64, sy: f64, sz: f64) -> Self {
        let sx = if sx.abs() < 1e-10 { 1.0 } else { sx };
        let sy = if sy.abs() < 1e-10 { 1.0 } else { sy };
        let sz = if sz.abs() < 1e-10 { 1.0 } else { sz };
        Self {
            inner,
            scale: Vec3::new(sx, sy, sz),
            inv_scale: Vec3::new(1.0 / sx, 1.0 / sy, 1.0 / sz),
        }
    }
}

impl Hittable for NonUniformScale {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        // Transform ray into scaled object space
        let scaled_origin = Vec3::new(
            ray.origin.x * self.inv_scale.x,
            ray.origin.y * self.inv_scale.y,
            ray.origin.z * self.inv_scale.z,
        );
        let scaled_dir = Vec3::new(
            ray.direction.x * self.inv_scale.x,
            ray.direction.y * self.inv_scale.y,
            ray.direction.z * self.inv_scale.z,
        );
        let scaled_ray = Ray::new(scaled_origin, scaled_dir);
        let mut hit = self.inner.hit(&scaled_ray, t_min, t_max)?;

        // Transform hit point back to world space
        hit.point = Vec3::new(
            hit.point.x * self.scale.x,
            hit.point.y * self.scale.y,
            hit.point.z * self.scale.z,
        );
        // Transform normal: use inverse-transpose (= inverse scale for diagonal matrices)
        let n = Vec3::new(
            hit.normal.x * self.inv_scale.x,
            hit.normal.y * self.inv_scale.y,
            hit.normal.z * self.inv_scale.z,
        );
        hit.normal = n.unit();
        // Adjust t by the direction scale factor
        hit.t *= scaled_dir.length() / ray.direction.length();
        Some(hit)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.inner.bounding_box().map(|bb| {
            let min = Vec3::new(
                bb.min.x * self.scale.x,
                bb.min.y * self.scale.y,
                bb.min.z * self.scale.z,
            );
            let max = Vec3::new(
                bb.max.x * self.scale.x,
                bb.max.y * self.scale.y,
                bb.max.z * self.scale.z,
            );
            // Handle negative scales
            Aabb::new(min.min(max), min.max(max))
        })
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
    fn test_scale() {
        let sphere = Sphere::new(Point3::ZERO, 1.0, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        let scaled = Scale::new(Box::new(sphere), 2.0);

        // Ray should hit at scaled distance
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = scaled.hit(&ray, 0.001, f64::INFINITY).unwrap();
        assert!((hit.point.z - -2.0).abs() < 0.01);

        // Bounding box should be doubled
        let bb = scaled.bounding_box().unwrap();
        assert!((bb.max.x - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_rotate_y() {
        let sphere = Sphere::new(Point3::new(2.0, 0.0, 0.0), 0.5, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        let rotated = RotateY::new(Box::new(sphere), 90.0);

        // After 90° rotation, sphere at (2,0,0) should be at (0,0,-2)
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(rotated.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_rotate_x() {
        let sphere = Sphere::new(Point3::new(0.0, 2.0, 0.0), 0.5, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        let rotated = RotateX::new(Box::new(sphere), 90.0);

        // After 90° X rotation, sphere at (0,2,0) should be at (0,0,2)
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(rotated.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_rotate_z() {
        let sphere = Sphere::new(Point3::new(2.0, 0.0, 0.0), 0.5, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        let rotated = RotateZ::new(Box::new(sphere), 90.0);

        // After 90° Z rotation, sphere at (2,0,0) should be at (0,2,0)
        let ray = Ray::new(Point3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        assert!(rotated.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_non_uniform_scale() {
        // Unit sphere at origin, scaled by (2, 1, 1) → ellipsoid stretching along X
        let sphere = Sphere::new(Point3::ZERO, 1.0, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        let scaled = NonUniformScale::new(Box::new(sphere), 2.0, 1.0, 1.0);

        // Should be hittable at x=1.5 (within 2x scale)
        let ray = Ray::new(Point3::new(1.5, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(scaled.hit(&ray, 0.001, f64::INFINITY).is_some(), "Should hit stretched ellipsoid");

        // Should miss at y=1.5 (beyond 1x scale)
        let ray = Ray::new(Point3::new(0.0, 1.5, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(scaled.hit(&ray, 0.001, f64::INFINITY).is_none(), "Should miss above unit-scaled Y");

        // Bounding box should reflect non-uniform scale
        let bb = scaled.bounding_box().unwrap();
        assert!((bb.max.x - 2.0).abs() < 0.01, "X bound should be 2.0, got {}", bb.max.x);
        assert!((bb.max.y - 1.0).abs() < 0.01, "Y bound should be 1.0, got {}", bb.max.y);
    }
}
