use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::vec3::Vec3;

/// Pre-loaded normal map image data.
pub struct NormalMapData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<[f32; 3]>,
}

/// Wraps an object and perturbs its surface normals using a tangent-space normal map image.
/// The image is interpreted as: R→X, G→Y, B→Z in tangent space (0..255 maps to -1..1).
pub struct NormalMap {
    pub inner: Box<dyn Hittable>,
    width: u32,
    height: u32,
    data: Vec<[f32; 3]>,
    pub strength: f64,
}

impl NormalMap {
    /// Load normal map image data from file. Returns data that can be used with `wrap`.
    pub fn load_image(path: &str) -> Result<NormalMapData, String> {
        let img = image::open(path).map_err(|e| format!("Failed to load normal map '{path}': {e}"))?;
        let rgb = img.to_rgb8();
        let width = rgb.width();
        let height = rgb.height();
        let data: Vec<[f32; 3]> = rgb
            .pixels()
            .map(|p| {
                [
                    p[0] as f32 / 255.0 * 2.0 - 1.0,
                    p[1] as f32 / 255.0 * 2.0 - 1.0,
                    p[2] as f32 / 255.0 * 2.0 - 1.0,
                ]
            })
            .collect();
        Ok(NormalMapData { width, height, data })
    }

    /// Wrap an object with pre-loaded normal map data.
    pub fn wrap(inner: Box<dyn Hittable>, nmap: NormalMapData, strength: f64) -> Self {
        Self { inner, width: nmap.width, height: nmap.height, data: nmap.data, strength }
    }


    fn sample(&self, u: f64, v: f64) -> Vec3 {
        let x = ((u * self.width as f64) as u32).min(self.width - 1);
        let y = (((1.0 - v) * self.height as f64) as u32).min(self.height - 1);
        let idx = (y * self.width + x) as usize;
        let d = &self.data[idx];
        Vec3::new(d[0] as f64, d[1] as f64, d[2] as f64)
    }
}

impl Hittable for NormalMap {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let mut hit = self.inner.hit(ray, t_min, t_max)?;

        let map_normal = self.sample(hit.u, hit.v);

        // Build tangent-space basis from the geometric normal
        let n = hit.normal;
        let (tangent, bitangent) = build_tangent_frame(n);

        // Transform from tangent space to world space
        let perturbed = tangent * map_normal.x + bitangent * map_normal.y + n * map_normal.z;

        // Blend between original normal and mapped normal based on strength
        let blended = (n * (1.0 - self.strength) + perturbed * self.strength).unit();

        hit.normal = blended;
        Some(hit)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.inner.bounding_box()
    }
}

/// Build an orthonormal tangent frame from a normal vector.
fn build_tangent_frame(n: Vec3) -> (Vec3, Vec3) {
    let up = if n.y.abs() < 0.999 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let tangent = up.cross(n).unit();
    let bitangent = n.cross(tangent);
    (tangent, bitangent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tangent_frame() {
        let n = Vec3::new(0.0, 1.0, 0.0);
        let (t, b) = build_tangent_frame(n);
        // Should be orthogonal
        assert!(t.dot(n).abs() < 1e-6);
        assert!(b.dot(n).abs() < 1e-6);
        assert!(t.dot(b).abs() < 1e-6);
        // Should be unit length
        assert!((t.length() - 1.0).abs() < 1e-6);
        assert!((b.length() - 1.0).abs() < 1e-6);
    }
}
