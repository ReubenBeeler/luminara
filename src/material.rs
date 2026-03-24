use rand::Rng;

use crate::hit::HitRecord;
use crate::ray::Ray;
use crate::texture::{SolidColor, Texture};
use crate::vec3::{Color, Vec3};

/// Result of scattering a ray off a material.
pub struct Scatter {
    pub ray: Ray,
    pub attenuation: Color,
}

/// Trait for materials that interact with light.
pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter>;

    /// Light emitted by this material. Defaults to black (no emission).
    fn emitted(&self) -> Color {
        Color::ZERO
    }
}

/// Workaround to use `dyn Rng` — we define our own trait object-safe RNG trait.
pub trait RngCore {
    fn next_f64(&mut self) -> f64;
}

impl<R: Rng> RngCore for R {
    fn next_f64(&mut self) -> f64 {
        self.random::<f64>()
    }
}

// --- Lambertian (diffuse) ---

pub struct Lambertian {
    pub texture: Box<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self {
            texture: Box::new(SolidColor::new(albedo)),
        }
    }

    pub fn with_texture(texture: Box<dyn Texture>) -> Self {
        Self { texture }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        let mut rng_adapter = RngAdapter(rng);
        let mut scatter_dir = hit.normal + Vec3::random_unit_vector(&mut rng_adapter);
        if scatter_dir.near_zero() {
            scatter_dir = hit.normal;
        }
        Some(Scatter {
            ray: Ray::with_time(hit.point, scatter_dir, ray.time),
            attenuation: self.texture.value(hit.u, hit.v, &hit.point),
        })
    }
}

// --- Metal ---

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz: fuzz.min(1.0),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        let mut rng_adapter = RngAdapter(rng);
        let reflected = ray.direction.unit().reflect(hit.normal);
        let scattered = reflected + Vec3::random_in_unit_sphere(&mut rng_adapter) * self.fuzz;
        if scattered.dot(hit.normal) > 0.0 {
            Some(Scatter {
                ray: Ray::with_time(hit.point, scattered, ray.time),
                attenuation: self.albedo,
            })
        } else {
            None
        }
    }
}

// --- Dielectric (glass) ---

pub struct Dielectric {
    pub refraction_index: f64,
    pub tint: Color,
    pub roughness: f64,
}

impl Dielectric {
    pub const fn new(refraction_index: f64) -> Self {
        Self {
            refraction_index,
            tint: Color::new(1.0, 1.0, 1.0),
            roughness: 0.0,
        }
    }

    pub const fn rough(refraction_index: f64, tint: Color, roughness: f64) -> Self {
        Self { refraction_index, tint, roughness }
    }

    /// Schlick's approximation for reflectance.
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        let eta_ratio = if hit.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = ray.direction.unit();
        let cos_theta = (-unit_direction).dot(hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = eta_ratio * sin_theta > 1.0;
        let mut direction =
            if cannot_refract || Self::reflectance(cos_theta, eta_ratio) > rng.next_f64() {
                unit_direction.reflect(hit.normal)
            } else {
                unit_direction.refract(hit.normal, eta_ratio)
            };

        // Apply roughness (frosted glass effect)
        if self.roughness > 0.0 {
            let mut rng_adapter = RngAdapter(rng);
            direction += Vec3::random_in_unit_sphere(&mut rng_adapter) * self.roughness;
        }

        // Beer's Law: colored glass absorbs light based on distance traveled inside
        let attenuation = if !hit.front_face && self.tint != Color::new(1.0, 1.0, 1.0) {
            // Inside the object — apply exponential attenuation
            let distance = hit.t;
            Color::new(
                (-(1.0 - self.tint.x) * distance).exp(),
                (-(1.0 - self.tint.y) * distance).exp(),
                (-(1.0 - self.tint.z) * distance).exp(),
            )
        } else {
            self.tint
        };

        Some(Scatter {
            ray: Ray::with_time(hit.point, direction, ray.time),
            attenuation,
        })
    }
}

// --- Emissive (light source) ---

pub struct Emissive {
    pub color: Color,
    pub intensity: f64,
}

impl Emissive {
    pub fn new(color: Color, intensity: f64) -> Self {
        Self { color, intensity }
    }
}

impl Material for Emissive {
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord, _rng: &mut dyn RngCore) -> Option<Scatter> {
        None // Lights don't scatter
    }

    fn emitted(&self) -> Color {
        self.color * self.intensity
    }
}

// --- Blend (mix two materials) ---

/// Randomly chooses between two materials per interaction.
pub struct Blend {
    pub mat_a: Box<dyn Material>,
    pub mat_b: Box<dyn Material>,
    pub ratio: f64, // Probability of choosing mat_a (0.0 to 1.0)
}

impl Blend {
    pub fn new(mat_a: Box<dyn Material>, mat_b: Box<dyn Material>, ratio: f64) -> Self {
        Self { mat_a, mat_b, ratio: ratio.clamp(0.0, 1.0) }
    }
}

impl Material for Blend {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        if rng.next_f64() < self.ratio {
            self.mat_a.scatter(ray, hit, rng)
        } else {
            self.mat_b.scatter(ray, hit, rng)
        }
    }

    fn emitted(&self) -> Color {
        // Blend emissions by ratio
        self.mat_a.emitted() * self.ratio + self.mat_b.emitted() * (1.0 - self.ratio)
    }
}

/// Adapter to use our `RngCore` trait with functions expecting `impl Rng`.
struct RngAdapter<'a>(&'a mut dyn RngCore);

impl rand::RngCore for RngAdapter<'_> {
    fn next_u32(&mut self) -> u32 {
        (self.0.next_f64() * u32::MAX as f64) as u32
    }

    fn next_u64(&mut self) -> u64 {
        (self.0.next_f64() * u64::MAX as f64) as u64
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest.iter_mut() {
            *byte = (self.0.next_f64() * 256.0) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hit::HitRecord;
    use crate::ray::Ray;
    use crate::vec3::{Color, Point3, Vec3};
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    /// Helper: build a HitRecord for a surface at the origin with normal pointing up (+Y),
    /// hit by a downward ray.
    fn make_hit_record(material: &dyn Material) -> HitRecord<'_> {
        let ray = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let point = Point3::new(0.0, 0.0, 0.0);
        let outward_normal = Vec3::new(0.0, 1.0, 0.0);
        HitRecord::new(&ray, point, outward_normal, 1.0, 0.5, 0.5, material)
    }

    #[test]
    fn lambertian_scatter_in_correct_hemisphere() {
        let mat = Lambertian::new(Color::new(0.8, 0.2, 0.2));
        let hit = make_hit_record(&mat);
        let incoming_ray = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let mut rng = SmallRng::seed_from_u64(42);

        for _ in 0..100 {
            let scatter = mat.scatter(&incoming_ray, &hit, &mut rng).unwrap();
            // Scattered ray direction should be in the same hemisphere as the normal
            assert!(
                scatter.ray.direction.dot(Vec3::new(0.0, 1.0, 0.0)) >= 0.0,
                "Lambertian scatter went below surface: {:?}",
                scatter.ray.direction
            );
            // Attenuation should match albedo
            assert!((scatter.attenuation.x - 0.8).abs() < 1e-6);
            assert!((scatter.attenuation.y - 0.2).abs() < 1e-6);
            assert!((scatter.attenuation.z - 0.2).abs() < 1e-6);
        }
    }

    #[test]
    fn metal_reflection_direction() {
        let mat = Metal::new(Color::new(0.9, 0.9, 0.9), 0.0);
        let hit = make_hit_record(&mat);
        // Incoming ray at 45 degrees
        let incoming_ray = Ray::new(
            Point3::new(-1.0, 1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0).unit(),
        );
        let mut rng = SmallRng::seed_from_u64(42);

        let scatter = mat.scatter(&incoming_ray, &hit, &mut rng).unwrap();
        // With zero fuzz, reflected direction should be (1, 1, 0) normalized
        let expected = Vec3::new(1.0, 1.0, 0.0).unit();
        let dir = scatter.ray.direction.unit();
        assert!(
            (dir.x - expected.x).abs() < 1e-6
                && (dir.y - expected.y).abs() < 1e-6
                && (dir.z - expected.z).abs() < 1e-6,
            "Metal reflection incorrect: got {:?}, expected {:?}",
            dir,
            expected
        );
    }

    #[test]
    fn metal_reflection_absorbed_when_below_surface() {
        // With high fuzz and a grazing angle, scatter can go below surface -> None
        let mat = Metal::new(Color::new(0.9, 0.9, 0.9), 1.0);
        // Ray nearly parallel to surface
        let incoming_ray = Ray::new(
            Point3::new(-1.0, 0.01, 0.0),
            Vec3::new(1.0, -0.01, 0.0).unit(),
        );
        let mat_for_hit = Lambertian::new(Color::new(0.5, 0.5, 0.5));
        let hit = make_hit_record(&mat_for_hit);
        let mut rng = SmallRng::seed_from_u64(42);
        // Run many times; at least some should be None (absorbed)
        let mut got_none = false;
        for _ in 0..200 {
            if mat.scatter(&incoming_ray, &hit, &mut rng).is_none() {
                got_none = true;
                break;
            }
        }
        assert!(got_none, "Expected at least one absorbed scatter with high fuzz at grazing angle");
    }

    #[test]
    fn dielectric_produces_scatter() {
        let mat = Dielectric::new(1.5);
        let hit = make_hit_record(&mat);
        let incoming_ray = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let mut rng = SmallRng::seed_from_u64(42);

        for _ in 0..100 {
            let scatter = mat.scatter(&incoming_ray, &hit, &mut rng).unwrap();
            // Attenuation is always white
            assert!((scatter.attenuation.x - 1.0).abs() < 1e-6);
            assert!((scatter.attenuation.y - 1.0).abs() < 1e-6);
            assert!((scatter.attenuation.z - 1.0).abs() < 1e-6);
            // Direction should be non-zero
            assert!(scatter.ray.direction.length() > 1e-6);
        }
    }

    #[test]
    fn dielectric_total_internal_reflection() {
        // High eta_ratio with steep angle should cause total internal reflection
        let mat = Dielectric::new(2.5);
        // Simulate hitting from inside (front_face = false)
        let incoming_ray = Ray::new(
            Point3::new(0.0, -1.0, 0.0),
            Vec3::new(0.8, 0.6, 0.0).unit(), // steep angle from inside
        );
        let point = Point3::new(0.0, 0.0, 0.0);
        let outward_normal = Vec3::new(0.0, 1.0, 0.0);
        let mat_ref: &dyn Material = &mat;
        let hit = HitRecord::new(&incoming_ray, point, outward_normal, 1.0, 0.5, 0.5, mat_ref);
        let mut rng = SmallRng::seed_from_u64(42);

        // Should always produce a scatter (dielectric always returns Some)
        let scatter = mat.scatter(&incoming_ray, &hit, &mut rng).unwrap();
        assert!(scatter.ray.direction.length() > 1e-6);
    }

    #[test]
    fn emissive_does_not_scatter() {
        let mat = Emissive::new(Color::new(1.0, 0.8, 0.6), 2.0);
        let hit = make_hit_record(&mat);
        let incoming_ray = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let mut rng = SmallRng::seed_from_u64(42);

        assert!(mat.scatter(&incoming_ray, &hit, &mut rng).is_none());
    }

    #[test]
    fn emissive_emits_correct_color() {
        let mat = Emissive::new(Color::new(1.0, 0.5, 0.0), 3.0);
        let emitted = mat.emitted();
        assert!((emitted.x - 3.0).abs() < 1e-6);
        assert!((emitted.y - 1.5).abs() < 1e-6);
        assert!((emitted.z - 0.0).abs() < 1e-6);
    }

    #[test]
    fn lambertian_emits_black() {
        let mat = Lambertian::new(Color::new(0.5, 0.5, 0.5));
        let emitted = mat.emitted();
        assert!((emitted.x).abs() < 1e-6);
        assert!((emitted.y).abs() < 1e-6);
        assert!((emitted.z).abs() < 1e-6);
    }

    #[test]
    fn schlick_reflectance_at_normal() {
        // At normal incidence, reflectance = ((1-n)/(1+n))^2
        let r = Dielectric::reflectance(1.0, 1.0 / 1.5);
        let ratio = 1.0_f64 / 1.5;
        let expected = ((1.0 - ratio) / (1.0 + ratio)).powi(2);
        assert!((r - expected).abs() < 1e-6);
    }

    #[test]
    fn schlick_reflectance_at_grazing() {
        // At grazing angle (cosine near 0), reflectance approaches 1
        let r = Dielectric::reflectance(0.01, 1.0 / 1.5);
        assert!(r > 0.9, "Grazing angle should have high reflectance, got {r}");
    }

    #[test]
    fn blend_chooses_between_materials() {
        let mat = Blend::new(
            Box::new(Lambertian::new(Color::new(1.0, 0.0, 0.0))),
            Box::new(Metal::new(Color::new(0.0, 0.0, 1.0), 0.0)),
            0.5,
        );
        let hit = make_hit_record(&mat);
        let incoming = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let mut rng = SmallRng::seed_from_u64(42);

        let mut got_red = false;
        let mut got_blue = false;
        for _ in 0..100 {
            if let Some(scatter) = mat.scatter(&incoming, &hit, &mut rng) {
                if scatter.attenuation.x > 0.5 {
                    got_red = true;
                }
                if scatter.attenuation.z > 0.5 {
                    got_blue = true;
                }
            }
        }
        assert!(got_red, "Blend should sometimes pick material A (red)");
        assert!(got_blue, "Blend should sometimes pick material B (blue)");
    }
}
