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
    fn emitted(&self, _u: f64, _v: f64, _point: &crate::vec3::Point3) -> Color {
        Color::ZERO
    }

    /// Whether this material is specular (mirror/glass). Specular materials
    /// should not use direct light sampling (NEE).
    fn is_specular(&self) -> bool {
        false
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
    fn is_specular(&self) -> bool {
        true
    }

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
    /// Dispersion strength (0 = none, typical = 0.01-0.05 for prism effects).
    pub dispersion: f64,
}

impl Dielectric {
    pub const fn new(refraction_index: f64) -> Self {
        Self {
            refraction_index,
            tint: Color::new(1.0, 1.0, 1.0),
            roughness: 0.0,
            dispersion: 0.0,
        }
    }

    pub const fn rough(refraction_index: f64, tint: Color, roughness: f64) -> Self {
        Self { refraction_index, tint, roughness, dispersion: 0.0 }
    }

    pub const fn dispersive(refraction_index: f64, tint: Color, roughness: f64, dispersion: f64) -> Self {
        Self { refraction_index, tint, roughness, dispersion }
    }

    /// Schlick's approximation for reflectance.
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn is_specular(&self) -> bool {
        true
    }

    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        // Stochastic dispersion: pick a random wavelength channel and adjust IOR
        let (ior, channel_attenuation) = if self.dispersion > 0.0 {
            let channel = (rng.next_f64() * 3.0) as u32;
            let ior_offset = match channel {
                0 => -self.dispersion,       // Red: lower IOR
                1 => 0.0,                    // Green: base IOR
                _ => self.dispersion,        // Blue: higher IOR
            };
            let channel_color = match channel {
                0 => Color::new(3.0, 0.0, 0.0),
                1 => Color::new(0.0, 3.0, 0.0),
                _ => Color::new(0.0, 0.0, 3.0),
            };
            (self.refraction_index + ior_offset, channel_color)
        } else {
            (self.refraction_index, Color::new(1.0, 1.0, 1.0))
        };

        let eta_ratio = if hit.front_face {
            1.0 / ior
        } else {
            ior
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
        let mut attenuation = if !hit.front_face && self.tint != Color::new(1.0, 1.0, 1.0) {
            let distance = hit.t;
            Color::new(
                (-(1.0 - self.tint.x) * distance).exp(),
                (-(1.0 - self.tint.y) * distance).exp(),
                (-(1.0 - self.tint.z) * distance).exp(),
            )
        } else {
            self.tint
        };

        // Apply channel selection for dispersion
        if self.dispersion > 0.0 {
            attenuation = attenuation.hadamard(channel_attenuation);
        }

        Some(Scatter {
            ray: Ray::with_time(hit.point, direction, ray.time),
            attenuation,
        })
    }
}

// --- Emissive (light source) ---

pub struct Emissive {
    pub texture: Box<dyn Texture>,
    pub intensity: f64,
}

impl Emissive {
    pub fn new(color: Color, intensity: f64) -> Self {
        Self {
            texture: Box::new(SolidColor::new(color)),
            intensity,
        }
    }

    pub fn with_texture(texture: Box<dyn Texture>, intensity: f64) -> Self {
        Self { texture, intensity }
    }
}

impl Material for Emissive {
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord, _rng: &mut dyn RngCore) -> Option<Scatter> {
        None // Lights don't scatter
    }

    fn emitted(&self, u: f64, v: f64, point: &crate::vec3::Point3) -> Color {
        self.texture.value(u, v, point) * self.intensity
    }
}

// --- Microfacet (Cook-Torrance GGX) ---

pub struct Microfacet {
    pub albedo: Box<dyn Texture>,
    pub roughness: f64,
    pub metallic: f64,
}

impl Microfacet {
    pub fn new(albedo: Color, roughness: f64, metallic: f64) -> Self {
        Self {
            albedo: Box::new(SolidColor::new(albedo)),
            roughness: roughness.clamp(0.01, 1.0),
            metallic: metallic.clamp(0.0, 1.0),
        }
    }

    pub fn with_texture(texture: Box<dyn Texture>, roughness: f64, metallic: f64) -> Self {
        Self {
            albedo: texture,
            roughness: roughness.clamp(0.01, 1.0),
            metallic: metallic.clamp(0.0, 1.0),
        }
    }

    /// GGX/Trowbridge-Reitz normal distribution function.
    #[cfg(test)]
    fn ggx_d(n_dot_h: f64, alpha: f64) -> f64 {
        let a2 = alpha * alpha;
        let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
        a2 / (std::f64::consts::PI * denom * denom)
    }

    /// Schlick-GGX geometry function (single direction).
    fn schlick_g1(n_dot_v: f64, k: f64) -> f64 {
        n_dot_v / (n_dot_v * (1.0 - k) + k)
    }

    /// Smith's geometry function using Schlick-GGX for both directions.
    fn geometry(n_dot_v: f64, n_dot_l: f64, roughness: f64) -> f64 {
        let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
        Self::schlick_g1(n_dot_v, k) * Self::schlick_g1(n_dot_l, k)
    }

    /// Fresnel-Schlick approximation.
    fn fresnel(cos_theta: f64, f0: Color) -> Color {
        let t = (1.0 - cos_theta).max(0.0).powi(5);
        f0 * (1.0 - t) + Color::new(1.0, 1.0, 1.0) * t
    }

    /// Sample a microfacet normal using GGX importance sampling.
    fn sample_ggx_normal(normal: &Vec3, alpha: f64, rng: &mut dyn RngCore) -> Vec3 {
        let r1 = rng.next_f64();
        let r2 = rng.next_f64();

        // GGX importance sampling in tangent space
        let theta = (alpha * alpha * r1 / (1.0 - r1 + 1e-10)).sqrt().atan();
        let phi = 2.0 * std::f64::consts::PI * r2;

        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let h_local = Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);

        // Build orthonormal basis from normal
        let up = if normal.y.abs() < 0.999 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let tangent = up.cross(*normal).unit();
        let bitangent = normal.cross(tangent);

        (tangent * h_local.x + bitangent * h_local.y + *normal * h_local.z).unit()
    }
}

impl Material for Microfacet {
    fn is_specular(&self) -> bool {
        // Treat highly metallic, low-roughness surfaces as specular for NEE
        self.metallic > 0.9 && self.roughness < 0.1
    }

    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        let alpha = self.roughness * self.roughness;
        let v = (-ray.direction).unit();
        let n = hit.normal;
        let albedo = self.albedo.value(hit.u, hit.v, &hit.point);

        let n_dot_v = n.dot(v).max(0.001);

        // F0: base reflectance (0.04 for dielectrics, albedo for metals)
        let f0 = Color::new(0.04, 0.04, 0.04) * (1.0 - self.metallic)
            + albedo * self.metallic;

        // Decide: specular or diffuse sampling based on Fresnel
        let fresnel_weight = Self::fresnel(n_dot_v, f0);
        let specular_prob = (fresnel_weight.x + fresnel_weight.y + fresnel_weight.z) / 3.0;
        let specular_prob = specular_prob.clamp(0.1, 0.9);

        if rng.next_f64() < specular_prob {
            // Specular path: sample GGX microfacet normal
            let h = Self::sample_ggx_normal(&n, alpha, rng);
            let l = v.reflect_around(h);

            let n_dot_l = n.dot(l);
            if n_dot_l <= 0.0 {
                return None;
            }
            let n_dot_h = n.dot(h).max(0.0);
            let v_dot_h = v.dot(h).max(0.0);

            let g = Self::geometry(n_dot_v, n_dot_l, self.roughness);
            let f = Self::fresnel(v_dot_h, f0);

            // Cook-Torrance specular BRDF * cos_theta / pdf (D cancels with GGX IS pdf)
            // weight = BRDF * n_dot_l / pdf_l = G * F * v_dot_h / (n_dot_v * n_dot_h)
            let weight = if n_dot_h > 0.001 {
                f * (g * v_dot_h / (n_dot_v * n_dot_h))
            } else {
                Color::ZERO
            };

            Some(Scatter {
                ray: Ray::with_time(hit.point, l, ray.time),
                attenuation: weight / specular_prob,
            })
        } else {
            // Diffuse path: Lambertian sampling
            let mut rng_adapter = RngAdapter(rng);
            let mut scatter_dir = n + Vec3::random_unit_vector(&mut rng_adapter);
            if scatter_dir.near_zero() {
                scatter_dir = n;
            }

            // Diffuse contribution: (1 - F) * (1 - metallic) * albedo / pi
            // Weighted by 1 / (1 - specular_prob) for correct MC weighting
            let k_d = (Color::new(1.0, 1.0, 1.0) - fresnel_weight) * (1.0 - self.metallic);
            let diffuse_attenuation = k_d.hadamard(albedo) / (1.0 - specular_prob);

            Some(Scatter {
                ray: Ray::with_time(hit.point, scatter_dir, ray.time),
                attenuation: diffuse_attenuation,
            })
        }
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

    fn emitted(&self, u: f64, v: f64, point: &crate::vec3::Point3) -> Color {
        // Blend emissions by ratio
        self.mat_a.emitted(u, v, point) * self.ratio + self.mat_b.emitted(u, v, point) * (1.0 - self.ratio)
    }
}

// --- Iridescent (thin-film interference) ---

/// Iridescent material that simulates thin-film interference effects
/// seen in soap bubbles, oil slicks, and beetle shells. Color shifts
/// based on viewing angle due to constructive/destructive interference.
pub struct Iridescent {
    /// Base reflectivity
    pub base_color: Color,
    /// Film thickness in nanometers (controls which wavelengths interfere)
    pub thickness: f64,
    /// Film refractive index
    pub film_ior: f64,
    /// Roughness of the reflection
    pub roughness: f64,
}

impl Iridescent {
    pub fn new(base_color: Color, thickness: f64, film_ior: f64, roughness: f64) -> Self {
        Self {
            base_color,
            thickness: thickness.max(100.0),
            film_ior: film_ior.max(1.0),
            roughness: roughness.clamp(0.0, 1.0),
        }
    }

    /// Compute thin-film interference color for a given angle.
    /// Uses simplified model: phase difference -> spectral color.
    fn thin_film_color(cos_theta: f64, thickness: f64, film_ior: f64) -> Color {
        // Snell's law: angle inside film
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let sin_theta_film = sin_theta / film_ior;
        let cos_theta_film = (1.0 - sin_theta_film * sin_theta_film).max(0.0).sqrt();

        // Optical path difference (2 * thickness * n * cos(theta_film))
        let opd = 2.0 * thickness * film_ior * cos_theta_film;

        // Approximate spectral response for RGB wavelengths (nm)
        let wavelengths = [650.0, 532.0, 450.0]; // R, G, B
        let mut color = [0.0f64; 3];
        for (i, &wl) in wavelengths.iter().enumerate() {
            // Phase difference
            let phase = 2.0 * std::f64::consts::PI * opd / wl;
            // Reflectance from interference (simplified)
            let r = (phase / 2.0).sin().powi(2);
            color[i] = r;
        }

        Color::new(color[0], color[1], color[2])
    }
}

impl Material for Iridescent {
    fn is_specular(&self) -> bool {
        true
    }

    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        let unit_dir = ray.direction.unit();
        let cos_theta = (-unit_dir).dot(hit.normal).abs().min(1.0);

        // Compute thin-film interference color
        let film_color = Self::thin_film_color(cos_theta, self.thickness, self.film_ior);

        // Blend with base color
        let attenuation = self.base_color.hadamard(film_color);

        // Reflect with optional roughness
        let mut direction = unit_dir.reflect(hit.normal);
        if self.roughness > 0.0 {
            let mut rng_adapter = RngAdapter(rng);
            direction += Vec3::random_in_unit_sphere(&mut rng_adapter) * self.roughness;
            if direction.dot(hit.normal) <= 0.0 {
                return None; // Absorbed
            }
        }

        Some(Scatter {
            ray: Ray::with_time(hit.point, direction, ray.time),
            attenuation,
        })
    }
}

// --- Clearcoat (lacquered surface — base diffuse + glossy clear layer) ---

/// Clearcoat material simulates a glossy transparent layer over a
/// colored base, like automotive paint, polished wood, or lacquer.
/// Combines diffuse base with Fresnel-weighted specular reflection.
pub struct Clearcoat {
    pub base_color: Color,
    /// Clearcoat glossiness (0 = matte base only, 1 = highly glossy coat)
    pub coat_gloss: f64,
    /// Clearcoat IOR (typically 1.5 for lacquer)
    pub coat_ior: f64,
}

impl Clearcoat {
    pub fn new(base_color: Color, coat_gloss: f64, coat_ior: f64) -> Self {
        Self {
            base_color,
            coat_gloss: coat_gloss.clamp(0.0, 1.0),
            coat_ior: coat_ior.max(1.0),
        }
    }
}

impl Material for Clearcoat {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        let unit_dir = ray.direction.unit();
        let cos_theta = (-unit_dir).dot(hit.normal).abs().min(1.0);

        // Fresnel reflectance for the clearcoat layer (Schlick)
        let r0 = ((1.0 - self.coat_ior) / (1.0 + self.coat_ior)).powi(2);
        let fresnel = r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5);

        // Probability of reflecting off the clearcoat vs diffusing from base
        let coat_prob = fresnel * self.coat_gloss;

        if rng.next_f64() < coat_prob {
            // Specular reflection off clearcoat
            let reflected = unit_dir.reflect(hit.normal);
            // Add slight roughness based on inverse gloss
            let roughness = (1.0 - self.coat_gloss) * 0.3;
            let direction = if roughness > 0.001 {
                let mut rng_adapter = RngAdapter(rng);
                let fuzzed = reflected + Vec3::random_in_unit_sphere(&mut rng_adapter) * roughness;
                if fuzzed.dot(hit.normal) > 0.0 { fuzzed } else { reflected }
            } else {
                reflected
            };
            Some(Scatter {
                ray: Ray::with_time(hit.point, direction, ray.time),
                attenuation: Color::new(1.0, 1.0, 1.0), // Clear coat reflects white
            })
        } else {
            // Diffuse reflection from base
            let mut rng_adapter = RngAdapter(rng);
            let mut scatter_dir = hit.normal + Vec3::random_unit_vector(&mut rng_adapter);
            if scatter_dir.near_zero() {
                scatter_dir = hit.normal;
            }
            Some(Scatter {
                ray: Ray::with_time(hit.point, scatter_dir, ray.time),
                attenuation: self.base_color,
            })
        }
    }
}

// --- Velvet (soft fabric with rim lighting) ---

/// Velvet material that produces a characteristic rim-lighting effect.
/// Brighter at grazing angles, darker when viewed head-on — simulates
/// the appearance of velvet, suede, and other short-fiber fabrics.
pub struct Velvet {
    pub color: Color,
    /// Sheen intensity at grazing angles (0.0 - 1.0)
    pub sheen: f64,
}

impl Velvet {
    pub fn new(color: Color, sheen: f64) -> Self {
        Self { color, sheen: sheen.clamp(0.0, 2.0) }
    }
}

impl Material for Velvet {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        let mut rng_adapter = RngAdapter(rng);
        let mut scatter_dir = hit.normal + Vec3::random_unit_vector(&mut rng_adapter);
        if scatter_dir.near_zero() {
            scatter_dir = hit.normal;
        }

        // Velvet BRDF: enhanced reflectance at grazing angles
        let cos_theta = (-ray.direction.unit()).dot(hit.normal).abs();
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        // Rim lighting: intensity increases as viewing angle becomes more grazing
        let rim = sin_theta.powf(1.5) * self.sheen;
        let attenuation = self.color * (1.0 + rim);

        Some(Scatter {
            ray: Ray::with_time(hit.point, scatter_dir, ray.time),
            attenuation,
        })
    }
}

// --- Translucent (subsurface scattering approximation) ---

/// Translucent material that allows light to pass through and scatter
/// underneath the surface. Approximates subsurface scattering for
/// materials like wax, jade, skin, leaves, and thin fabric.
pub struct Translucent {
    pub color: Color,
    /// Fraction of light that transmits through (0 = opaque, 1 = fully translucent)
    pub translucency: f64,
    /// How much the transmitted light scatters (0 = straight through, 1 = fully diffuse)
    pub scatter_width: f64,
}

impl Translucent {
    pub fn new(color: Color, translucency: f64, scatter_width: f64) -> Self {
        Self {
            color,
            translucency: translucency.clamp(0.0, 1.0),
            scatter_width: scatter_width.clamp(0.0, 1.0),
        }
    }
}

impl Material for Translucent {
    fn scatter(&self, ray: &Ray, hit: &HitRecord, rng: &mut dyn RngCore) -> Option<Scatter> {
        if rng.next_f64() < self.translucency {
            // Transmitted: scatter through the surface
            let through = ray.direction.unit();
            let mut rng_adapter = RngAdapter(rng);
            let random_below = -hit.normal + Vec3::random_unit_vector(&mut rng_adapter);
            let mut dir = through * (1.0 - self.scatter_width) + random_below * self.scatter_width;
            if dir.near_zero() {
                dir = -hit.normal;
            }
            Some(Scatter {
                ray: Ray::with_time(hit.point, dir, ray.time),
                attenuation: self.color,
            })
        } else {
            // Reflected: diffuse reflection on the surface
            let mut rng_adapter = RngAdapter(rng);
            let mut scatter_dir = hit.normal + Vec3::random_unit_vector(&mut rng_adapter);
            if scatter_dir.near_zero() {
                scatter_dir = hit.normal;
            }
            Some(Scatter {
                ray: Ray::with_time(hit.point, scatter_dir, ray.time),
                attenuation: self.color,
            })
        }
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
        let p = Point3::new(0.0, 0.0, 0.0);
        let emitted = mat.emitted(0.5, 0.5, &p);
        assert!((emitted.x - 3.0).abs() < 1e-6);
        assert!((emitted.y - 1.5).abs() < 1e-6);
        assert!((emitted.z - 0.0).abs() < 1e-6);
    }

    #[test]
    fn lambertian_emits_black() {
        let mat = Lambertian::new(Color::new(0.5, 0.5, 0.5));
        let p = Point3::new(0.0, 0.0, 0.0);
        let emitted = mat.emitted(0.5, 0.5, &p);
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

    #[test]
    fn emissive_with_texture() {
        use crate::texture::Checker;
        let tex = Checker::new(Color::new(1.0, 0.0, 0.0), Color::new(0.0, 1.0, 0.0), 1.0);
        let mat = Emissive::with_texture(Box::new(tex), 5.0);
        let p = Point3::new(0.0, 0.0, 0.0);
        let emitted = mat.emitted(0.5, 0.5, &p);
        // Should emit non-zero and be scaled by intensity
        let luminance = emitted.x + emitted.y + emitted.z;
        assert!(luminance > 0.0, "Textured emissive should emit light");
        assert!((emitted.x / 5.0 + emitted.y / 5.0).abs() < 1.01, "Should be one of the checker colors scaled by 5");
    }

    #[test]
    fn microfacet_scatter_valid() {
        let mat = Microfacet::new(Color::new(0.9, 0.1, 0.1), 0.3, 1.0);
        let hit = make_hit_record(&mat);
        let incoming = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let mut rng = SmallRng::seed_from_u64(42);

        let mut scatter_count = 0;
        for _ in 0..200 {
            if let Some(scatter) = mat.scatter(&incoming, &hit, &mut rng) {
                // Attenuation components should be non-negative
                assert!(scatter.attenuation.x >= 0.0, "Negative attenuation R: {}", scatter.attenuation.x);
                assert!(scatter.attenuation.y >= 0.0, "Negative attenuation G: {}", scatter.attenuation.y);
                assert!(scatter.attenuation.z >= 0.0, "Negative attenuation B: {}", scatter.attenuation.z);
                // Direction should be non-zero
                assert!(scatter.ray.direction.length() > 1e-6);
                scatter_count += 1;
            }
        }
        assert!(scatter_count > 50, "Should scatter most of the time, got {scatter_count}/200");
    }

    #[test]
    fn microfacet_ggx_normalized() {
        // GGX D function at n_dot_h = 1 (perfect alignment) should peak
        let d_peak = Microfacet::ggx_d(1.0, 0.5 * 0.5);
        let d_off = Microfacet::ggx_d(0.5, 0.5 * 0.5);
        assert!(d_peak > d_off, "GGX should peak at n_dot_h = 1");
        assert!(d_peak > 0.0, "GGX D should be positive");
    }

    #[test]
    fn microfacet_dielectric_has_diffuse() {
        // Non-metallic microfacet should have diffuse component
        let mat = Microfacet::new(Color::new(0.8, 0.2, 0.1), 0.5, 0.0);
        let hit = make_hit_record(&mat);
        let incoming = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let mut rng = SmallRng::seed_from_u64(42);

        // With metallic=0, we should get diffuse scatters with colored attenuation
        let mut got_diffuse = false;
        for _ in 0..100 {
            if let Some(scatter) = mat.scatter(&incoming, &hit, &mut rng) {
                if scatter.attenuation.x > scatter.attenuation.y {
                    got_diffuse = true; // Albedo color bleeding through
                }
            }
        }
        assert!(got_diffuse, "Non-metallic should produce colored diffuse scatters");
    }

    #[test]
    fn dielectric_dispersion_separates_channels() {
        let mat = Dielectric::dispersive(1.5, Color::new(1.0, 1.0, 1.0), 0.0, 0.03);
        let mat_ref: &dyn Material = &mat;
        let incoming = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = HitRecord::new(&incoming, Point3::ZERO, Vec3::new(0.0, 1.0, 0.0), 1.0, 0.5, 0.5, mat_ref);
        let mut rng = SmallRng::seed_from_u64(42);

        let mut got_red_only = false;
        let mut got_green_only = false;
        let mut got_blue_only = false;
        for _ in 0..300 {
            if let Some(scatter) = mat.scatter(&incoming, &hit, &mut rng) {
                let a = scatter.attenuation;
                if a.x > 1.0 && a.y < 0.1 && a.z < 0.1 { got_red_only = true; }
                if a.y > 1.0 && a.x < 0.1 && a.z < 0.1 { got_green_only = true; }
                if a.z > 1.0 && a.x < 0.1 && a.y < 0.1 { got_blue_only = true; }
            }
        }
        assert!(got_red_only, "Dispersion should produce red-only samples");
        assert!(got_green_only, "Dispersion should produce green-only samples");
        assert!(got_blue_only, "Dispersion should produce blue-only samples");
    }

    #[test]
    fn iridescent_color_varies_with_angle() {
        let mat = Iridescent::new(Color::new(1.0, 1.0, 1.0), 400.0, 1.4, 0.0);
        // Normal incidence
        let c1 = Iridescent::thin_film_color(1.0, 400.0, 1.4);
        // Grazing angle
        let c2 = Iridescent::thin_film_color(0.3, 400.0, 1.4);
        // Colors should differ (that's the whole point of iridescence)
        let diff = (c1.x - c2.x).abs() + (c1.y - c2.y).abs() + (c1.z - c2.z).abs();
        assert!(diff > 0.01, "Iridescent colors should vary with angle, diff = {diff}");

        // Test scattering works
        let hit = make_hit_record(&mat);
        let incoming = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let mut rng = SmallRng::seed_from_u64(42);
        let scatter = mat.scatter(&incoming, &hit, &mut rng).unwrap();
        assert!(scatter.ray.direction.length() > 1e-6);
        assert!(scatter.attenuation.x >= 0.0);
    }

    #[test]
    fn translucent_transmits_and_reflects() {
        let mat = Translucent::new(Color::new(0.8, 0.9, 0.7), 0.5, 0.8);
        let hit = make_hit_record(&mat);
        let incoming = Ray::new(Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let mut rng = SmallRng::seed_from_u64(42);

        let mut transmitted = 0;
        let mut reflected = 0;
        for _ in 0..200 {
            let scatter = mat.scatter(&incoming, &hit, &mut rng).unwrap();
            // Check if ray goes below surface (transmitted) or above (reflected)
            if scatter.ray.direction.dot(Vec3::new(0.0, 1.0, 0.0)) < 0.0 {
                transmitted += 1;
            } else {
                reflected += 1;
            }
            // Attenuation should match color
            assert!((scatter.attenuation.x - 0.8).abs() < 1e-6);
        }
        assert!(transmitted > 30, "Should have some transmitted rays, got {transmitted}");
        assert!(reflected > 30, "Should have some reflected rays, got {reflected}");
    }

    #[test]
    fn specular_flag_correct() {
        let lamb = Lambertian::new(Color::new(0.5, 0.5, 0.5));
        assert!(!lamb.is_specular(), "Lambertian should not be specular");

        let metal = Metal::new(Color::new(0.9, 0.9, 0.9), 0.0);
        assert!(metal.is_specular(), "Metal should be specular");

        let glass = Dielectric::new(1.5);
        assert!(glass.is_specular(), "Dielectric should be specular");

        let light = Emissive::new(Color::new(1.0, 1.0, 1.0), 1.0);
        assert!(!light.is_specular(), "Emissive should not be specular");
    }
}
