use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// Axis-aligned rectangle in the XY plane at z = k.
pub struct XyRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl XyRect {
    pub fn new(x0: f64, x1: f64, y0: f64, y1: f64, k: f64, material: Box<dyn Material>) -> Self {
        Self { x0, x1, y0, y1, k, material }
    }
}

impl Hittable for XyRect {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let t = (self.k - ray.origin.z) / ray.direction.z;
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.x + t * ray.direction.x;
        let y = ray.origin.y + t * ray.direction.y;
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        Some(HitRecord::new(ray, ray.at(t), Vec3::new(0.0, 0.0, 1.0), t, u, v, self.material.as_ref()))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.x0, self.y0, self.k - 0.0001),
            Point3::new(self.x1, self.y1, self.k + 0.0001),
        ))
    }
}

/// Axis-aligned rectangle in the XZ plane at y = k.
pub struct XzRect {
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl XzRect {
    pub fn new(x0: f64, x1: f64, z0: f64, z1: f64, k: f64, material: Box<dyn Material>) -> Self {
        Self { x0, x1, z0, z1, k, material }
    }
}

impl Hittable for XzRect {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let t = (self.k - ray.origin.y) / ray.direction.y;
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.x + t * ray.direction.x;
        let z = ray.origin.z + t * ray.direction.z;
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (z - self.z0) / (self.z1 - self.z0);
        Some(HitRecord::new(ray, ray.at(t), Vec3::new(0.0, 1.0, 0.0), t, u, v, self.material.as_ref()))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.x0, self.k - 0.0001, self.z0),
            Point3::new(self.x1, self.k + 0.0001, self.z1),
        ))
    }
}

/// Axis-aligned rectangle in the YZ plane at x = k.
pub struct YzRect {
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl YzRect {
    pub fn new(y0: f64, y1: f64, z0: f64, z1: f64, k: f64, material: Box<dyn Material>) -> Self {
        Self { y0, y1, z0, z1, k, material }
    }
}

impl Hittable for YzRect {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let t = (self.k - ray.origin.x) / ray.direction.x;
        if t < t_min || t > t_max {
            return None;
        }
        let y = ray.origin.y + t * ray.direction.y;
        let z = ray.origin.z + t * ray.direction.z;
        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }
        let u = (y - self.y0) / (self.y1 - self.y0);
        let v = (z - self.z0) / (self.z1 - self.z0);
        Some(HitRecord::new(ray, ray.at(t), Vec3::new(1.0, 0.0, 0.0), t, u, v, self.material.as_ref()))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.k - 0.0001, self.y0, self.z0),
            Point3::new(self.k + 0.0001, self.y1, self.z1),
        ))
    }
}

/// An axis-aligned box made of 6 rectangles.
pub fn make_box(
    p_min: Point3,
    p_max: Point3,
    material_factory: impl Fn() -> Box<dyn Material>,
) -> Vec<Box<dyn Hittable>> {
    vec![
        Box::new(XyRect::new(p_min.x, p_max.x, p_min.y, p_max.y, p_max.z, material_factory())),
        Box::new(XyRect::new(p_min.x, p_max.x, p_min.y, p_max.y, p_min.z, material_factory())),
        Box::new(XzRect::new(p_min.x, p_max.x, p_min.z, p_max.z, p_max.y, material_factory())),
        Box::new(XzRect::new(p_min.x, p_max.x, p_min.z, p_max.z, p_min.y, material_factory())),
        Box::new(YzRect::new(p_min.y, p_max.y, p_min.z, p_max.z, p_max.x, material_factory())),
        Box::new(YzRect::new(p_min.y, p_max.y, p_min.z, p_max.z, p_min.x, material_factory())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_xz_rect_hit() {
        let rect = XzRect::new(0.0, 1.0, 0.0, 1.0, 0.0, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        let ray = Ray::new(Point3::new(0.5, 1.0, 0.5), Vec3::new(0.0, -1.0, 0.0));
        assert!(rect.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_xz_rect_miss() {
        let rect = XzRect::new(0.0, 1.0, 0.0, 1.0, 0.0, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        let ray = Ray::new(Point3::new(2.0, 1.0, 0.5), Vec3::new(0.0, -1.0, 0.0));
        assert!(rect.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_make_box() {
        let sides = make_box(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
            || Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        assert_eq!(sides.len(), 6);
    }
}
