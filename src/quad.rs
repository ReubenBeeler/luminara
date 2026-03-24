use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A parallelogram defined by a corner point Q and two edge vectors u and v.
/// The quad spans from Q to Q+u, Q+v, and Q+u+v.
pub struct Quad {
    q: Point3,
    u: Vec3,
    v: Vec3,
    normal: Vec3,
    d: f64,
    w: Vec3,
    material: Box<dyn Material>,
}

impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, material: Box<dyn Material>) -> Self {
        let n = u.cross(v);
        let normal = n.unit();
        let d = normal.dot(q);
        let w = n / n.dot(n);
        Self { q, u, v, normal, d, w, material }
    }
}

impl Hittable for Quad {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let denom = self.normal.dot(ray.direction);

        // Ray is parallel to the quad plane
        if denom.abs() < 1e-8 {
            return None;
        }

        let t = (self.d - self.normal.dot(ray.origin)) / denom;
        if t < t_min || t > t_max {
            return None;
        }

        let intersection = ray.at(t);
        let planar_hit = intersection - self.q;
        let alpha = self.w.dot(planar_hit.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hit));

        // Check if the hit is inside the parallelogram [0,1] x [0,1]
        if !(0.0..=1.0).contains(&alpha) || !(0.0..=1.0).contains(&beta) {
            return None;
        }

        Some(HitRecord::new(
            ray,
            intersection,
            self.normal,
            t,
            alpha,
            beta,
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let p0 = self.q;
        let p1 = self.q + self.u;
        let p2 = self.q + self.v;
        let p3 = self.q + self.u + self.v;
        let min = p0.min(p1).min(p2).min(p3) - Vec3::new(1e-4, 1e-4, 1e-4);
        let max = p0.max(p1).max(p2).max(p3) + Vec3::new(1e-4, 1e-4, 1e-4);
        Some(Aabb::new(min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    fn test_mat() -> Box<dyn Material> {
        Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)))
    }

    #[test]
    fn test_quad_hit_center() {
        // XZ quad at y=0
        let quad = Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            test_mat(),
        );
        let ray = Ray::new(Point3::new(0.5, 1.0, 0.5), Vec3::new(0.0, -1.0, 0.0));
        let hit = quad.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some());
        let rec = hit.unwrap();
        assert!((rec.t - 1.0).abs() < 1e-6);
        assert!((rec.u - 0.5).abs() < 1e-6);
        assert!((rec.v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_quad_miss_outside() {
        let quad = Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            test_mat(),
        );
        let ray = Ray::new(Point3::new(2.0, 1.0, 0.5), Vec3::new(0.0, -1.0, 0.0));
        assert!(quad.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_quad_miss_parallel() {
        let quad = Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            test_mat(),
        );
        // Ray parallel to the quad
        let ray = Ray::new(Point3::new(0.5, 1.0, 0.5), Vec3::new(1.0, 0.0, 0.0));
        assert!(quad.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_quad_tilted() {
        // Tilted quad in space: normal is (1,-1,0)
        let quad = Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            test_mat(),
        );
        // Ray coming from +x/-y direction, hitting the center of the quad
        let hit = quad.hit(
            &Ray::new(Point3::new(1.5, -0.5, 0.5), Vec3::new(-1.0, 1.0, 0.0)),
            0.001,
            f64::INFINITY,
        );
        assert!(hit.is_some());
    }

    #[test]
    fn test_quad_bounding_box() {
        let quad = Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
            test_mat(),
        );
        let bb = quad.bounding_box().unwrap();
        assert!(bb.min.x < 0.001);
        assert!(bb.max.x > 1.999);
        assert!(bb.max.y > 2.999);
    }
}
