use crate::ray::Ray;
use crate::vec3::Point3;

/// Axis-aligned bounding box for BVH acceleration.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Point3,
    pub max: Point3,
}

impl Aabb {
    pub const fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    /// Test whether a ray intersects this AABB using the slab method.
    #[inline]
    pub fn hit(&self, ray: &Ray, mut t_min: f64, mut t_max: f64) -> bool {
        // X axis
        let inv_d = 1.0 / ray.direction.x;
        let mut t0 = (self.min.x - ray.origin.x) * inv_d;
        let mut t1 = (self.max.x - ray.origin.x) * inv_d;
        if inv_d < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }
        t_min = t0.max(t_min);
        t_max = t1.min(t_max);
        if t_max <= t_min {
            return false;
        }

        // Y axis
        let inv_d = 1.0 / ray.direction.y;
        let mut t0 = (self.min.y - ray.origin.y) * inv_d;
        let mut t1 = (self.max.y - ray.origin.y) * inv_d;
        if inv_d < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }
        t_min = t0.max(t_min);
        t_max = t1.min(t_max);
        if t_max <= t_min {
            return false;
        }

        // Z axis
        let inv_d = 1.0 / ray.direction.z;
        let mut t0 = (self.min.z - ray.origin.z) * inv_d;
        let mut t1 = (self.max.z - ray.origin.z) * inv_d;
        if inv_d < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }
        t_min = t0.max(t_min);
        t_max = t1.min(t_max);

        t_max > t_min
    }

    /// Surface area of this AABB (used for SAH).
    pub fn surface_area(&self) -> f64 {
        let d = self.max - self.min;
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }

    /// Return the bounding box of the intersection of two boxes.
    /// If the boxes don't overlap, returns a zero-volume box at the origin.
    pub fn intersection(a: &Aabb, b: &Aabb) -> Aabb {
        let min = a.min.max(b.min);
        let max = a.max.min(b.max);
        // Guard against degenerate boxes where min > max
        if min.x > max.x || min.y > max.y || min.z > max.z {
            Aabb {
                min: Point3::ZERO,
                max: Point3::ZERO,
            }
        } else {
            Aabb { min, max }
        }
    }

    /// Return the bounding box that encloses both boxes.
    pub fn surrounding(a: &Aabb, b: &Aabb) -> Aabb {
        Aabb {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::Vec3;

    #[test]
    fn test_aabb_hit() {
        let bb = Aabb::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(bb.hit(&ray, 0.001, f64::INFINITY));
    }

    #[test]
    fn test_aabb_miss() {
        let bb = Aabb::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3::new(0.0, 5.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(!bb.hit(&ray, 0.001, f64::INFINITY));
    }

    #[test]
    fn test_surface_area() {
        // 2x2x2 cube: surface area = 6 * 4 = 24
        let bb = Aabb::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, 1.0, 1.0));
        assert!((bb.surface_area() - 24.0).abs() < 1e-6);
    }

    #[test]
    fn test_aabb_hit_negative_direction() {
        let bb = Aabb::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, 1.0, 1.0));
        // Ray from +z going in -z direction
        let ray = Ray::new(Point3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(bb.hit(&ray, 0.001, f64::INFINITY));
    }

    #[test]
    fn test_surrounding() {
        let a = Aabb::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(0.0, 0.0, 0.0));
        let b = Aabb::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let s = Aabb::surrounding(&a, &b);
        assert_eq!(s.min, Point3::new(-1.0, -1.0, -1.0));
        assert_eq!(s.max, Point3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_intersection_overlapping() {
        let a = Aabb::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, 1.0, 1.0));
        let b = Aabb::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 2.0, 2.0));
        let i = Aabb::intersection(&a, &b);
        assert_eq!(i.min, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(i.max, Point3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_intersection_non_overlapping() {
        let a = Aabb::new(Point3::new(-2.0, -2.0, -2.0), Point3::new(-1.0, -1.0, -1.0));
        let b = Aabb::new(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 2.0, 2.0));
        let i = Aabb::intersection(&a, &b);
        // Degenerate case: should return zero-volume box
        assert_eq!(i.min, Point3::ZERO);
        assert_eq!(i.max, Point3::ZERO);
    }
}
