use rand::Rng;

use crate::hit::HitRecord;
use crate::ray::Ray;
use crate::vec3::{Color, Vec3};

/// Result of scattering a ray off a material.
pub struct Scatter {
    pub ray: Ray,
    pub attenuation: Color,
}

/// Trait for materials that interact with light.
pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter>;
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
    pub albedo: Color,
}

impl Lambertian {
    pub const fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        let mut rng_adapter = RngAdapter(rng);
        let mut scatter_dir = hit.normal + Vec3::random_unit_vector(&mut rng_adapter);
        if scatter_dir.near_zero() {
            scatter_dir = hit.normal;
        }
        Some(Scatter {
            ray: Ray::new(hit.point, scatter_dir),
            attenuation: self.albedo,
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
                ray: Ray::new(hit.point, scattered),
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
}

impl Dielectric {
    pub const fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
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
        let direction =
            if cannot_refract || Self::reflectance(cos_theta, eta_ratio) > rng.next_f64() {
                unit_direction.reflect(hit.normal)
            } else {
                unit_direction.refract(hit.normal, eta_ratio)
            };

        Some(Scatter {
            ray: Ray::new(hit.point, direction),
            attenuation: Color::new(1.0, 1.0, 1.0),
        })
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
