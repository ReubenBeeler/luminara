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
