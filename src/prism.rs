use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A triangular prism with equilateral triangle base, aligned along the Y axis.
pub struct Prism {
    /// Three vertices of the bottom triangle
    v0: Point3,
    v1: Point3,
    v2: Point3,
    /// Height of the prism
    height: f64,
    material: Box<dyn Material>,
}

impl Prism {
    /// Create a prism centered at `center` with equilateral triangle base of given `side`
    /// length and `height` along Y.
    pub fn new(center: Point3, side: f64, height: f64, material: Box<dyn Material>) -> Self {
        // Equilateral triangle vertices in XZ plane centered at (center.x, center.z)
        let r = side / 3.0_f64.sqrt(); // circumradius
        let v0 = Point3::new(center.x, center.y, center.z + r);
        let v1 = Point3::new(
            center.x - side / 2.0,
            center.y,
            center.z - r / 2.0,
        );
        let v2 = Point3::new(
            center.x + side / 2.0,
            center.y,
            center.z - r / 2.0,
        );
        Self { v0, v1, v2, height, material }
    }
}

impl Hittable for Prism {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let mut closest_t = t_max;
        let mut best_hit: Option<(Point3, Vec3, f64, f64)> = None;

        let dy = Vec3::new(0.0, self.height, 0.0);
        let top0 = self.v0 + dy;
        let top1 = self.v1 + dy;
        let top2 = self.v2 + dy;

        // Bottom cap (triangle: v0, v1, v2, normal -Y)
        if let Some((t, u, v)) = ray_triangle(ray, self.v0, self.v1, self.v2).filter(|(t, _, _)| *t > t_min && *t < closest_t) {
            closest_t = t;
            best_hit = Some((ray.at(t), Vec3::new(0.0, -1.0, 0.0), u, v));
        }
        // Top cap (triangle: top0, top2, top1, normal +Y — reversed winding)
        if let Some((t, u, v)) = ray_triangle(ray, top0, top2, top1).filter(|(t, _, _)| *t > t_min && *t < closest_t) {
            closest_t = t;
            best_hit = Some((ray.at(t), Vec3::new(0.0, 1.0, 0.0), u, v));
        }

        // Three side quads (each as two triangles)
        let bot = [self.v0, self.v1, self.v2];
        let top = [top0, top1, top2];
        for i in 0..3 {
            let j = (i + 1) % 3;
            let edge = bot[j] - bot[i];
            let up = Vec3::new(0.0, 1.0, 0.0);
            let normal = edge.cross(up).unit();

            // First triangle of quad
            if let Some((t, u, v)) = ray_triangle(ray, bot[i], bot[j], top[j]).filter(|(t, _, _)| *t > t_min && *t < closest_t) {
                closest_t = t;
                best_hit = Some((ray.at(t), normal, u, v));
            }
            // Second triangle of quad
            if let Some((t, u, v)) = ray_triangle(ray, bot[i], top[j], top[i]).filter(|(t, _, _)| *t > t_min && *t < closest_t) {
                closest_t = t;
                best_hit = Some((ray.at(t), normal, u, v));
            }
        }

        best_hit.map(|(point, outward_normal, u, v)| {
            HitRecord::new(ray, point, outward_normal, closest_t, u, v, self.material.as_ref())
        })
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let xs = [self.v0.x, self.v1.x, self.v2.x];
        let zs = [self.v0.z, self.v1.z, self.v2.z];
        let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_z = zs.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_z = zs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        Some(Aabb::new(
            Point3::new(min_x, self.v0.y, min_z),
            Point3::new(max_x, self.v0.y + self.height, max_z),
        ))
    }
}

/// Möller–Trumbore ray-triangle intersection. Returns (t, u, v).
fn ray_triangle(ray: &Ray, v0: Point3, v1: Point3, v2: Point3) -> Option<(f64, f64, f64)> {
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
    if t > 1e-10 {
        Some((t, u, v))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_prism_hit_side() {
        let prism = Prism::new(
            Point3::new(0.0, 0.0, 0.0),
            2.0,
            3.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Shoot ray from front toward center
        let ray = Ray::new(Point3::new(0.0, 1.5, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = prism.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some(), "Ray should hit prism side");
    }

    #[test]
    fn test_prism_miss() {
        let prism = Prism::new(
            Point3::new(0.0, 0.0, 0.0),
            1.0,
            2.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray far above
        let ray = Ray::new(Point3::new(0.0, 10.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(prism.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_prism_bounding_box() {
        let prism = Prism::new(
            Point3::new(0.0, 0.0, 0.0),
            2.0,
            3.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let bb = prism.bounding_box().unwrap();
        assert!(bb.min.y < 0.01);
        assert!((bb.max.y - 3.0).abs() < 0.01);
    }
}
