use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A box with rounded edges, implemented via SDF ray marching.
/// `half_size` defines the half-extents of the inner box.
/// `radius` defines the rounding radius applied to edges and corners.
pub struct RoundedBox {
    pub center: Point3,
    pub half_size: Vec3,
    pub radius: f64,
    pub material: Box<dyn Material>,
}

impl RoundedBox {
    pub fn new(center: Point3, half_size: Vec3, radius: f64, material: Box<dyn Material>) -> Self {
        Self { center, half_size, radius, material }
    }
}

impl Hittable for RoundedBox {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let o = ray.origin - self.center;
        let d = ray.direction;
        let b = self.half_size;
        let r = self.radius;

        let sdf = |p: Vec3| -> f64 {
            let q = Vec3::new(p.x.abs() - b.x, p.y.abs() - b.y, p.z.abs() - b.z);
            let outside = Vec3::new(q.x.max(0.0), q.y.max(0.0), q.z.max(0.0)).length();
            let inside = q.x.max(q.y.max(q.z)).min(0.0);
            outside + inside - r
        };

        // Sphere march
        let mut t = t_min;
        for _ in 0..256 {
            if t > t_max {
                break;
            }
            let p = o + d * t;
            let dist = sdf(p);
            if dist < 1e-5 {
                // Found surface
                let point = ray.at(t);
                let local_p = point - self.center;

                // Gradient normal
                let eps = 1e-5;
                let nx = sdf(local_p + Vec3::new(eps, 0.0, 0.0)) - sdf(local_p - Vec3::new(eps, 0.0, 0.0));
                let ny = sdf(local_p + Vec3::new(0.0, eps, 0.0)) - sdf(local_p - Vec3::new(0.0, eps, 0.0));
                let nz = sdf(local_p + Vec3::new(0.0, 0.0, eps)) - sdf(local_p - Vec3::new(0.0, 0.0, eps));
                let normal = Vec3::new(nx, ny, nz).unit();

                // UV mapping: box-style projection based on dominant face
                let total = b + Vec3::new(r, r, r);
                let u;
                let v;
                let abs_n = Vec3::new(normal.x.abs(), normal.y.abs(), normal.z.abs());
                if abs_n.x > abs_n.y && abs_n.x > abs_n.z {
                    u = (local_p.z / total.z + 1.0) * 0.5;
                    v = (local_p.y / total.y + 1.0) * 0.5;
                } else if abs_n.y > abs_n.z {
                    u = (local_p.x / total.x + 1.0) * 0.5;
                    v = (local_p.z / total.z + 1.0) * 0.5;
                } else {
                    u = (local_p.x / total.x + 1.0) * 0.5;
                    v = (local_p.y / total.y + 1.0) * 0.5;
                }

                return Some(HitRecord::new(
                    ray, point, normal, t, u, v, self.material.as_ref(),
                ));
            }
            // Advance by the distance to the surface (sphere tracing)
            t += dist.max(1e-4);
        }

        None
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let extent = self.half_size + Vec3::new(self.radius, self.radius, self.radius);
        Some(Aabb::new(
            self.center - extent,
            self.center + extent,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_rounded_box_hit() {
        let rb = RoundedBox::new(
            Point3::ZERO,
            Vec3::new(1.0, 1.0, 1.0),
            0.2,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = rb.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.t - 3.8).abs() < 0.1); // ~5 - 1.2 (half_size + radius)
    }

    #[test]
    fn test_rounded_box_miss() {
        let rb = RoundedBox::new(
            Point3::ZERO,
            Vec3::new(0.5, 0.5, 0.5),
            0.1,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(5.0, 5.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(rb.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_rounded_box_bbox() {
        let rb = RoundedBox::new(
            Point3::new(1.0, 2.0, 3.0),
            Vec3::new(0.5, 0.5, 0.5),
            0.1,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let bbox = rb.bounding_box().unwrap();
        assert!((bbox.min.x - 0.4).abs() < 1e-6);
        assert!((bbox.max.x - 1.6).abs() < 1e-6);
    }
}
