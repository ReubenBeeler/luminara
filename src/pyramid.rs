use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A square-base pyramid aligned along the Y axis.
pub struct Pyramid {
    /// Center of the base
    base_center: Point3,
    /// Half-width of the square base
    half_base: f64,
    /// Height of the pyramid
    height: f64,
    material: Box<dyn Material>,
}

impl Pyramid {
    pub fn new(base_center: Point3, base_size: f64, height: f64, material: Box<dyn Material>) -> Self {
        Self {
            base_center,
            half_base: base_size / 2.0,
            height,
            material,
        }
    }
}

/// Möller–Trumbore ray-triangle intersection. Returns (t, u, v).
fn ray_tri(ray: &Ray, v0: Point3, v1: Point3, v2: Point3) -> Option<(f64, f64, f64)> {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = ray.direction.cross(edge2);
    let a = edge1.dot(h);
    if a.abs() < 1e-10 {
        return None;
    }
    let f = 1.0 / a;
    let s = ray.origin - v0;
    let u = f * s.dot(h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(edge1);
    let v = f * ray.direction.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * edge2.dot(q);
    if t > 1e-10 { Some((t, u, v)) } else { None }
}

impl Hittable for Pyramid {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let cx = self.base_center.x;
        let cy = self.base_center.y;
        let cz = self.base_center.z;
        let hb = self.half_base;

        // Base corners (Y = cy)
        let b0 = Point3::new(cx - hb, cy, cz - hb);
        let b1 = Point3::new(cx + hb, cy, cz - hb);
        let b2 = Point3::new(cx + hb, cy, cz + hb);
        let b3 = Point3::new(cx - hb, cy, cz + hb);
        // Apex
        let apex = Point3::new(cx, cy + self.height, cz);

        let mut closest_t = t_max;
        let mut best: Option<(Point3, Vec3, f64, f64)> = None;

        // 6 triangles: 2 for base, 4 for sides
        let tris: [(Point3, Point3, Point3); 6] = [
            // Base (two triangles, normal -Y)
            (b0, b2, b1),
            (b0, b3, b2),
            // Side faces
            (b0, b1, apex),
            (b1, b2, apex),
            (b2, b3, apex),
            (b3, b0, apex),
        ];

        for (a, b, c) in &tris {
            if let Some((t, u, v)) = ray_tri(ray, *a, *b, *c).filter(|(t, _, _)| *t > t_min && *t < closest_t) {
                let edge1 = *b - *a;
                let edge2 = *c - *a;
                let normal = edge1.cross(edge2).unit();
                closest_t = t;
                best = Some((ray.at(t), normal, u, v));
            }
        }

        best.map(|(point, outward_normal, u, v)| {
            HitRecord::new(ray, point, outward_normal, closest_t, u, v, self.material.as_ref())
        })
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(
                self.base_center.x - self.half_base,
                self.base_center.y,
                self.base_center.z - self.half_base,
            ),
            Point3::new(
                self.base_center.x + self.half_base,
                self.base_center.y + self.height,
                self.base_center.z + self.half_base,
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_pyramid_hit_side() {
        let pyr = Pyramid::new(
            Point3::new(0.0, 0.0, 0.0),
            2.0,
            3.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray from front hitting a side face
        let ray = Ray::new(Point3::new(0.0, 1.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(pyr.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_pyramid_hit_base() {
        let pyr = Pyramid::new(
            Point3::new(0.0, 0.0, 0.0),
            2.0,
            3.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray from below hitting base
        let ray = Ray::new(Point3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        assert!(pyr.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_pyramid_miss() {
        let pyr = Pyramid::new(
            Point3::new(0.0, 0.0, 0.0),
            1.0,
            2.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray going away
        let ray = Ray::new(Point3::new(5.0, 5.0, 5.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(pyr.hit(&ray, 0.001, f64::INFINITY).is_none());
    }
}
