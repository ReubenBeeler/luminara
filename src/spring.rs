use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A helical spring (coil) along the Y axis, implemented via SDF ray marching.
/// `coil_radius` — distance from center axis to tube center.
/// `tube_radius` — thickness of the wire.
/// `pitch` — vertical distance per full turn.
/// `turns` — number of complete turns.
pub struct Spring {
    pub center: Point3,
    pub coil_radius: f64,
    pub tube_radius: f64,
    pub pitch: f64,
    pub turns: f64,
    pub material: Box<dyn Material>,
}

impl Spring {
    pub fn new(
        center: Point3,
        coil_radius: f64,
        tube_radius: f64,
        pitch: f64,
        turns: f64,
        material: Box<dyn Material>,
    ) -> Self {
        Self { center, coil_radius, tube_radius, pitch, turns, material }
    }
}

impl Hittable for Spring {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let o = ray.origin - self.center;
        let d = ray.direction;
        let height = self.pitch * self.turns;
        let two_pi = 2.0 * std::f64::consts::PI;

        let sdf = |p: Vec3| -> f64 {
            // Find the angle in the XZ plane
            let angle = p.z.atan2(p.x); // -PI to PI

            // The helix maps angle to y: y = pitch * (angle / 2pi + k) for integer k
            // Find the k that gives the closest y
            let base_y = self.pitch * angle / two_pi;
            let mut best_dist = f64::INFINITY;

            // Check several turns to find the closest helix point
            let k_min = ((p.y - base_y) / self.pitch).floor() as i32 - 1;
            let k_max = k_min + 3;
            for k in k_min..=k_max {
                let helix_y = base_y + self.pitch * k as f64;
                if helix_y < -self.tube_radius || helix_y > height + self.tube_radius {
                    continue;
                }
                let helix_y_clamped = helix_y.clamp(0.0, height);
                let hy_angle = (helix_y_clamped / self.pitch) * two_pi;
                let hx = self.coil_radius * hy_angle.cos();
                let hz = self.coil_radius * hy_angle.sin();

                let dx = p.x - hx;
                let dy = p.y - helix_y_clamped;
                let dz = p.z - hz;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist < best_dist {
                    best_dist = dist;
                }
            }

            best_dist - self.tube_radius
        };

        // Sphere trace
        let step_min = self.tube_radius * 0.3;
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

                // Gradient normal
                let eps = 1e-4;
                let nx = sdf(local_p + Vec3::new(eps, 0.0, 0.0)) - sdf(local_p - Vec3::new(eps, 0.0, 0.0));
                let ny = sdf(local_p + Vec3::new(0.0, eps, 0.0)) - sdf(local_p - Vec3::new(0.0, eps, 0.0));
                let nz = sdf(local_p + Vec3::new(0.0, 0.0, eps)) - sdf(local_p - Vec3::new(0.0, 0.0, eps));
                let normal = Vec3::new(nx, ny, nz).unit();

                // UV: u = progress along spring, v = angle around tube
                let u = local_p.y / height;
                let v = local_p.z.atan2(local_p.x) / two_pi + 0.5;

                return Some(HitRecord::new(ray, point, normal, t, u, v, self.material.as_ref()));
            }
            t += dist.max(step_min);
        }

        None
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let r = self.coil_radius + self.tube_radius;
        let height = self.pitch * self.turns;
        Some(Aabb::new(
            self.center + Vec3::new(-r, -self.tube_radius, -r),
            self.center + Vec3::new(r, height + self.tube_radius, r),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_spring_hit() {
        let spring = Spring::new(
            Point3::ZERO,
            1.0,  // coil_radius
            0.15, // tube_radius
            1.0,  // pitch
            3.0,  // turns
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // At y=0 (bottom), helix angle=0, so center is at (1,0,0)
        // Shoot a ray at (1,0,z) from the front — should hit the tube
        let ray = Ray::new(Point3::new(1.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(spring.hit(&ray, 0.001, f64::INFINITY).is_some());
    }

    #[test]
    fn test_spring_miss() {
        let spring = Spring::new(
            Point3::ZERO, 1.0, 0.15, 1.0, 3.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        // Ray well above the spring
        let ray = Ray::new(Point3::new(0.0, 10.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(spring.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_spring_bbox() {
        let spring = Spring::new(
            Point3::new(1.0, 0.0, 0.0), 0.5, 0.1, 2.0, 3.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let bbox = spring.bounding_box().unwrap();
        assert!((bbox.min.x - 0.4).abs() < 1e-6);
        assert!((bbox.max.x - 1.6).abs() < 1e-6);
        assert!((bbox.max.y - 6.1).abs() < 1e-6); // height = 2*3 + 0.1
    }
}
