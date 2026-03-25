use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A superellipsoid — a generalized shape controlled by two exponents (e1, e2).
/// e1 controls NS curvature, e2 controls EW curvature.
/// (1,1) = sphere, (0.1,0.1) = cube, (2,2) = diamond/octahedron,
/// (0.1,1) = cylinder, (1,0.1) = pillow.
pub struct Superellipsoid {
    pub center: Point3,
    pub scale: Vec3,
    pub e1: f64,
    pub e2: f64,
    pub material: Box<dyn Material>,
}

impl Superellipsoid {
    pub fn new(center: Point3, scale: Vec3, e1: f64, e2: f64, material: Box<dyn Material>) -> Self {
        Self { center, scale, e1, e2, material }
    }
}

impl Hittable for Superellipsoid {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let o = Vec3::new(
            (ray.origin.x - self.center.x) / self.scale.x,
            (ray.origin.y - self.center.y) / self.scale.y,
            (ray.origin.z - self.center.z) / self.scale.z,
        );
        let d = Vec3::new(
            ray.direction.x / self.scale.x,
            ray.direction.y / self.scale.y,
            ray.direction.z / self.scale.z,
        );

        let e1 = self.e1;
        let e2 = self.e2;

        // SDF for superellipsoid: ( (|x|^(2/e2) + |z|^(2/e2))^(e2/e1) + |y|^(2/e1) )^(e1/2) - 1
        let sdf = |p: Vec3| -> f64 {
            let ax = p.x.abs();
            let ay = p.y.abs();
            let az = p.z.abs();
            let exp2 = 2.0 / e2;
            let xz = ax.powf(exp2) + az.powf(exp2);
            let exp1 = 2.0 / e1;
            let val = xz.powf(e2 / e1) + ay.powf(exp1);
            val.powf(e1 / 2.0) - 1.0
        };

        // Sphere trace
        let mut t = t_min;
        for _ in 0..256 {
            if t > t_max {
                break;
            }
            let p = o + d * t;
            let dist = sdf(p);
            if dist.abs() < 1e-5 {
                let point = ray.at(t);
                let local_p = Vec3::new(
                    (point.x - self.center.x) / self.scale.x,
                    (point.y - self.center.y) / self.scale.y,
                    (point.z - self.center.z) / self.scale.z,
                );

                // Gradient normal
                let eps = 1e-5;
                let nx = sdf(local_p + Vec3::new(eps, 0.0, 0.0)) - sdf(local_p - Vec3::new(eps, 0.0, 0.0));
                let ny = sdf(local_p + Vec3::new(0.0, eps, 0.0)) - sdf(local_p - Vec3::new(0.0, eps, 0.0));
                let nz = sdf(local_p + Vec3::new(0.0, 0.0, eps)) - sdf(local_p - Vec3::new(0.0, 0.0, eps));
                let normal = Vec3::new(
                    nx / self.scale.x,
                    ny / self.scale.y,
                    nz / self.scale.z,
                ).unit();

                // Spherical UV
                let u = 0.5 + local_p.z.atan2(local_p.x) / (2.0 * std::f64::consts::PI);
                let v = 0.5 + local_p.y.asin().clamp(-1.0, 1.0) / std::f64::consts::PI;

                return Some(HitRecord::new(ray, point, normal, t, u, v, self.material.as_ref()));
            }
            t += dist.max(1e-4);
        }

        None
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(Aabb::new(
            self.center - self.scale,
            self.center + self.scale,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_superellipsoid_sphere_hit() {
        // e1=1, e2=1 is a sphere
        let se = Superellipsoid::new(
            Point3::ZERO,
            Vec3::new(1.0, 1.0, 1.0),
            1.0, 1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(0.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = se.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.t - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_superellipsoid_miss() {
        let se = Superellipsoid::new(
            Point3::ZERO,
            Vec3::new(1.0, 1.0, 1.0),
            0.5, 0.5,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let ray = Ray::new(Point3::new(5.0, 5.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(se.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn test_superellipsoid_bbox() {
        let se = Superellipsoid::new(
            Point3::new(1.0, 2.0, 3.0),
            Vec3::new(0.5, 1.0, 0.5),
            0.3, 0.3,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        );
        let bbox = se.bounding_box().unwrap();
        assert!((bbox.min.x - 0.5).abs() < 1e-6);
        assert!((bbox.max.y - 3.0).abs() < 1e-6);
    }
}
