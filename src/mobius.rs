use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A Möbius strip — a non-orientable surface with one half-twist.
/// `radius` — radius of the strip's central circle.
/// `width` — half-width of the strip.
/// `thickness` — thickness of the surface (for ray intersection).
pub struct Mobius {
    pub center: Point3,
    pub radius: f64,
    pub width: f64,
    pub thickness: f64,
    pub material: Box<dyn Material>,
}

impl Mobius {
    pub fn new(center: Point3, radius: f64, width: f64, thickness: f64, material: Box<dyn Material>) -> Self {
        Self { center, radius, width, thickness, material }
    }
}

impl Hittable for Mobius {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let o = ray.origin - self.center;
        let d = ray.direction;
        let r = self.radius;
        let w = self.width;
        let th = self.thickness;

        // SDF for a Möbius strip:
        // Parametric: P(u,v) = (r + v*cos(u/2))*cos(u), (r + v*cos(u/2))*sin(u), v*sin(u/2)
        // where u in [0, 2pi], v in [-w, w]
        // We approximate the SDF by finding the nearest point on the strip.
        let sdf = |p: Vec3| -> f64 {
            let angle = p.z.atan2(p.x);
            let mut best = f64::INFINITY;

            // Check two candidate angles (the strip crosses each angle twice due to the twist)
            for offset in [-std::f64::consts::PI, 0.0, std::f64::consts::PI] {
                let u = angle + offset;
                let cos_u = u.cos();
                let sin_u = u.sin();
                let cos_half = (u / 2.0).cos();
                let sin_half = (u / 2.0).sin();

                // Project point onto the strip at this angle
                // Center of strip at this angle
                let cx = r * cos_u;
                let cz = r * sin_u;

                // Local frame: radial direction and twist direction
                let radial = Vec3::new(cos_u * cos_half, 0.0, sin_u * cos_half);
                let twist = Vec3::new(0.0, sin_half, 0.0);
                let strip_normal_dir = Vec3::new(cos_u * cos_half, sin_half, sin_u * cos_half);

                let dp = Vec3::new(p.x - cx, p.y, p.z - cz);

                // v parameter: project dp onto the strip direction
                let strip_dir = strip_normal_dir.unit();
                let v_param = dp.x * strip_dir.x + dp.y * strip_dir.y + dp.z * strip_dir.z;
                let v_clamped = v_param.clamp(-w, w);

                // Point on strip surface
                let sx = cx + v_clamped * strip_dir.x;
                let sy = v_clamped * strip_dir.y;
                let sz = cz + v_clamped * strip_dir.z;

                let dx = p.x - sx;
                let dy = p.y - sy;
                let dz = p.z - sz;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt() - th;

                let _ = (radial, twist); // suppress unused warnings
                if dist < best {
                    best = dist;
                }
            }
            best
        };

        // Sphere trace
        let mut t = t_min;
        for _ in 0..512 {
            if t > t_max {
                break;
            }
            let p = o + d * t;
            let dist = sdf(p);
            if dist < 1e-4 {
                let point = ray.at(t);
                let local_p = point - self.center;

                let eps = 1e-4;
                let nx = sdf(local_p + Vec3::new(eps, 0.0, 0.0)) - sdf(local_p - Vec3::new(eps, 0.0, 0.0));
                let ny = sdf(local_p + Vec3::new(0.0, eps, 0.0)) - sdf(local_p - Vec3::new(0.0, eps, 0.0));
                let nz = sdf(local_p + Vec3::new(0.0, 0.0, eps)) - sdf(local_p - Vec3::new(0.0, 0.0, eps));
                let normal = Vec3::new(nx, ny, nz).unit();

                let u = (local_p.z.atan2(local_p.x) / (2.0 * std::f64::consts::PI)) + 0.5;
                let v = local_p.y / (w + th) * 0.5 + 0.5;

                return Some(HitRecord::new(ray, point, normal, t, u, v, self.material.as_ref()));
            }
            t += dist.max(th * 0.3);
        }

        None
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let extent = self.radius + self.width + self.thickness;
        let y_extent = self.width + self.thickness;
        Some(Aabb::new(
            self.center - Vec3::new(extent, y_extent, extent),
            self.center + Vec3::new(extent, y_extent, extent),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_mobius_hit() {
        let m = Mobius::new(
            Point3::ZERO, 1.0, 0.3, 0.05,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray aimed at the strip from the side (x=1 is where the strip center is at angle 0)
        let ray = Ray::new(Point3::new(1.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(m.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_mobius_miss() {
        let m = Mobius::new(
            Point3::ZERO, 1.0, 0.3, 0.05,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(0.0, 5.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(m.hit(&ray, 0.001, f64::INFINITY).is_none());
    }
}
