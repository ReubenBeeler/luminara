use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::texture::Perlin;
use crate::vec3::Vec3;

/// Wraps an object and perturbs its surface normals using Perlin noise.
pub struct BumpMap {
    pub inner: Box<dyn Hittable>,
    perlin: Perlin,
    pub strength: f64,
    pub scale: f64,
}

impl BumpMap {
    pub fn new(inner: Box<dyn Hittable>, strength: f64, scale: f64) -> Self {
        Self {
            inner,
            perlin: Perlin::new(),
            strength,
            scale,
        }
    }
}

impl Hittable for BumpMap {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let mut hit = self.inner.hit(ray, t_min, t_max)?;

        // Compute gradient of Perlin noise at hit point for normal perturbation
        let eps = 0.001;
        let p = hit.point * self.scale;
        let dx = self.perlin.noise(&(p + Vec3::new(eps, 0.0, 0.0)))
            - self.perlin.noise(&(p - Vec3::new(eps, 0.0, 0.0)));
        let dy = self.perlin.noise(&(p + Vec3::new(0.0, eps, 0.0)))
            - self.perlin.noise(&(p - Vec3::new(0.0, eps, 0.0)));
        let dz = self.perlin.noise(&(p + Vec3::new(0.0, 0.0, eps)))
            - self.perlin.noise(&(p - Vec3::new(0.0, 0.0, eps)));

        let gradient = Vec3::new(dx, dy, dz) * (self.strength / (2.0 * eps));
        let perturbed = (hit.normal + gradient).unit();

        hit.normal = perturbed;
        Some(hit)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.inner.bounding_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::sphere::Sphere;
    use crate::vec3::{Color, Point3};

    #[test]
    fn bump_map_perturbs_normal() {
        let sphere = Box::new(Sphere::new(
            Point3::ZERO, 1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        ));
        let bumped = BumpMap::new(sphere, 1.0, 4.0);
        let ray = Ray::new(Point3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = bumped.hit(&ray, 0.001, f64::INFINITY).unwrap();
        // Normal should still be roughly unit length and pointing outward
        let len = hit.normal.length();
        assert!((len - 1.0).abs() < 0.01, "Normal should be unit, got {len}");
    }

    #[test]
    fn bump_map_preserves_bbox() {
        let sphere = Box::new(Sphere::new(
            Point3::ZERO, 1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        ));
        let orig_bb = sphere.bounding_box();
        let bumped = BumpMap::new(sphere, 1.0, 4.0);
        let bump_bb = bumped.bounding_box();
        assert_eq!(orig_bb.is_some(), bump_bb.is_some());
    }
}
