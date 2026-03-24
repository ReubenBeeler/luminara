use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A torus (donut) centered at the origin in the XZ plane.
/// major_radius: distance from center to tube center
/// minor_radius: radius of the tube
pub struct Torus {
    pub center: Point3,
    pub major_radius: f64,
    pub minor_radius: f64,
    pub material: Box<dyn Material>,
}

impl Torus {
    pub fn new(center: Point3, major_radius: f64, minor_radius: f64, material: Box<dyn Material>) -> Self {
        Self { center, major_radius, minor_radius, material }
    }
}

impl Hittable for Torus {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        // Transform ray to torus local space
        let o = ray.origin - self.center;
        let d = ray.direction;

        let r_maj = self.major_radius;
        let r_min = self.minor_radius;

        // Torus equation: (sqrt(x^2 + z^2) - R)^2 + y^2 = r^2
        // Substituting ray parametric form and expanding gives a quartic in t.
        // We use ray marching with Newton refinement for robustness.

        let step = r_min * 0.5;
        let max_steps = ((t_max - t_min) / step).min(500.0) as usize;

        let sdf = |p: Vec3| -> f64 {
            let q = (p.x * p.x + p.z * p.z).sqrt() - r_maj;
            (q * q + p.y * p.y).sqrt() - r_min
        };

        let mut t = t_min;
        let mut prev_dist = sdf(o + d * t);

        for _ in 0..max_steps {
            t += step;
            if t > t_max {
                break;
            }

            let p = o + d * t;
            let dist = sdf(p);

            // Check for sign change (ray crossed the surface)
            if prev_dist > 0.0 && dist < 0.0 {
                // Bisect to find the exact crossing
                let mut lo = t - step;
                let mut hi = t;
                for _ in 0..20 {
                    let mid = (lo + hi) * 0.5;
                    if sdf(o + d * mid) > 0.0 {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }

                let t_hit = (lo + hi) * 0.5;
                if t_hit < t_min || t_hit > t_max {
                    prev_dist = dist;
                    continue;
                }

                let point = ray.at(t_hit);
                let local_p = point - self.center;

                // Compute normal via gradient
                let eps = 1e-5;
                let nx = sdf(local_p + Vec3::new(eps, 0.0, 0.0)) - sdf(local_p - Vec3::new(eps, 0.0, 0.0));
                let ny = sdf(local_p + Vec3::new(0.0, eps, 0.0)) - sdf(local_p - Vec3::new(0.0, eps, 0.0));
                let nz = sdf(local_p + Vec3::new(0.0, 0.0, eps)) - sdf(local_p - Vec3::new(0.0, 0.0, eps));
                let normal = Vec3::new(nx, ny, nz).unit();

                // UV mapping
                let theta = local_p.z.atan2(local_p.x); // angle around major axis
                let xz_dist = (local_p.x * local_p.x + local_p.z * local_p.z).sqrt();
                let phi = local_p.y.atan2(xz_dist - r_maj); // angle around tube

                let u = (theta / (2.0 * std::f64::consts::PI)) + 0.5;
                let v = (phi / (2.0 * std::f64::consts::PI)) + 0.5;

                return Some(HitRecord::new(
                    ray, point, normal, t_hit, u, v, self.material.as_ref(),
                ));
            }

            prev_dist = dist;
        }

        None
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let r = self.major_radius + self.minor_radius;
        Some(Aabb::new(
            self.center - Vec3::new(r, self.minor_radius, r),
            self.center + Vec3::new(r, self.minor_radius, r),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_torus_hit() {
        let torus = Torus::new(
            Point3::ZERO, 1.0, 0.3,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray hitting the front of the torus
        let ray = Ray::new(Point3::new(1.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(torus.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_torus_miss_above() {
        let torus = Torus::new(
            Point3::ZERO, 1.0, 0.3,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray well above the torus should miss
        let ray = Ray::new(Point3::new(0.0, 5.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(torus.hit(&ray, 0.001, f64::INFINITY).is_none());
    }
}
