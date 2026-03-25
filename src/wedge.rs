use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A wedge (triangular prism with right-angled triangle cross-section).
/// Base in XZ plane, ramp rises along +Y from front to back.
pub struct Wedge {
    /// Bottom-front-left corner
    min: Point3,
    /// Top-back-right corner (y = max height at back)
    max: Point3,
    material: Box<dyn Material>,
}

impl Wedge {
    pub fn new(min: Point3, max: Point3, material: Box<dyn Material>) -> Self {
        Self { min, max, material }
    }
}

impl Hittable for Wedge {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        // The wedge is an axis-aligned box where the top face is a ramp:
        // y <= (z - min.z) / (max.z - min.z) * (max.y - min.y) + min.y
        // We test ray against 5 faces: bottom, left, right, front, back, ramp

        let mut closest_t = t_max;
        let mut best: Option<(Point3, Vec3, f64, f64)> = None;

        let depth = self.max.z - self.min.z;
        let height = self.max.y - self.min.y;
        let width = self.max.x - self.min.x;

        // Bottom face (y = min.y, XZ rectangle)
        if ray.direction.y.abs() > 1e-10 {
            let t = (self.min.y - ray.origin.y) / ray.direction.y;
            if t > t_min && t < closest_t {
                let p = ray.at(t);
                if p.x >= self.min.x && p.x <= self.max.x && p.z >= self.min.z && p.z <= self.max.z {
                    let u = (p.x - self.min.x) / width;
                    let v = (p.z - self.min.z) / depth;
                    closest_t = t;
                    best = Some((p, Vec3::new(0.0, -1.0, 0.0), u, v));
                }
            }
        }

        // Back face (z = max.z, XY rectangle up to max.y)
        if ray.direction.z.abs() > 1e-10 {
            let t = (self.max.z - ray.origin.z) / ray.direction.z;
            if t > t_min && t < closest_t {
                let p = ray.at(t);
                if p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y {
                    let u = (p.x - self.min.x) / width;
                    let v = (p.y - self.min.y) / height;
                    closest_t = t;
                    best = Some((p, Vec3::new(0.0, 0.0, 1.0), u, v));
                }
            }
        }

        // Front face (z = min.z, just bottom edge — degenerate, skip since height=0 at front)

        // Left face (x = min.x, triangle in YZ)
        if ray.direction.x.abs() > 1e-10 {
            let t = (self.min.x - ray.origin.x) / ray.direction.x;
            if t > t_min && t < closest_t {
                let p = ray.at(t);
                let ramp_y = (p.z - self.min.z) / depth * height + self.min.y;
                if p.z >= self.min.z && p.z <= self.max.z && p.y >= self.min.y && p.y <= ramp_y {
                    let u = (p.z - self.min.z) / depth;
                    let v = (p.y - self.min.y) / height;
                    closest_t = t;
                    best = Some((p, Vec3::new(-1.0, 0.0, 0.0), u, v));
                }
            }
        }

        // Right face (x = max.x, triangle in YZ)
        if ray.direction.x.abs() > 1e-10 {
            let t = (self.max.x - ray.origin.x) / ray.direction.x;
            if t > t_min && t < closest_t {
                let p = ray.at(t);
                let ramp_y = (p.z - self.min.z) / depth * height + self.min.y;
                if p.z >= self.min.z && p.z <= self.max.z && p.y >= self.min.y && p.y <= ramp_y {
                    let u = (p.z - self.min.z) / depth;
                    let v = (p.y - self.min.y) / height;
                    closest_t = t;
                    best = Some((p, Vec3::new(1.0, 0.0, 0.0), u, v));
                }
            }
        }

        // Ramp face: plane y = slope * (z - min.z) + min.y
        // Normal: (-slope, 1, 0) normalized, where slope = height/depth
        let slope = height / depth;
        let normal = Vec3::new(0.0, 1.0, -slope).unit();
        // Plane equation: normal · (p - point_on_ramp) = 0
        // Use point_on_ramp = (min.x, min.y, min.z)
        let denom = ray.direction.dot(normal);
        if denom.abs() > 1e-10 {
            let t = (Point3::new(self.min.x, self.min.y, self.min.z) - ray.origin).dot(normal) / denom;
            if t > t_min && t < closest_t {
                let p = ray.at(t);
                if p.x >= self.min.x && p.x <= self.max.x && p.z >= self.min.z && p.z <= self.max.z {
                    let expected_y = (p.z - self.min.z) / depth * height + self.min.y;
                    if (p.y - expected_y).abs() < 0.01 {
                        let u = (p.x - self.min.x) / width;
                        let v = (p.z - self.min.z) / depth;
                        closest_t = t;
                        best = Some((p, normal, u, v));
                    }
                }
            }
        }

        best.map(|(point, outward_normal, u, v)| {
            HitRecord::new(ray, point, outward_normal, closest_t, u, v, self.material.as_ref())
        })
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(Aabb::new(self.min, self.max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_wedge_hit_ramp() {
        let wedge = Wedge::new(
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(1.0, 2.0, 4.0),
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray from above hitting the ramp surface at z=2 (mid-depth, y=1)
        let ray = Ray::new(Point3::new(0.0, 5.0, 2.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = wedge.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some(), "Should hit ramp surface");
    }

    #[test]
    fn test_wedge_hit_bottom() {
        let wedge = Wedge::new(
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(1.0, 2.0, 4.0),
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray from below hitting bottom face
        let ray = Ray::new(Point3::new(0.0, -1.0, 2.0), Vec3::new(0.0, 1.0, 0.0));
        let hit = wedge.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some(), "Should hit bottom face");
        let h = hit.unwrap();
        assert!((h.point.y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_wedge_miss() {
        let wedge = Wedge::new(
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(1.0, 2.0, 4.0),
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray going away
        let ray = Ray::new(Point3::new(5.0, 5.0, 5.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(wedge.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_wedge_bounding_box() {
        let wedge = Wedge::new(
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(1.0, 2.0, 4.0),
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let bb = wedge.bounding_box().unwrap();
        assert!((bb.min.x - -1.0).abs() < 1e-6);
        assert!((bb.max.y - 2.0).abs() < 1e-6);
    }
}
