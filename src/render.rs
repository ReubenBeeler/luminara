use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rayon::prelude::*;

use crate::camera::Camera;
use crate::hit::Hittable;
use crate::material::RngCore;
use crate::ray::Ray;
use crate::vec3::{Color, Point3, Vec3};

/// Information about a light source for direct light sampling (NEE).
pub struct LightInfo {
    pub center: Point3,
    pub radius: f64,
    pub emission: Color,
}

/// Background environment for rays that miss all objects.
#[derive(Default)]
pub enum Background {
    /// Sky gradient from white (bottom) to blue (top)
    #[default]
    SkyGradient,
    /// Solid color background
    Solid(Color),
    /// Custom gradient from bottom to top
    Gradient { bottom: Color, top: Color },
    /// Physical sky with sun
    Sun {
        direction: crate::vec3::Vec3,
        sun_color: Color,
        sun_intensity: f64,
        sky_color: Color,
    },
    /// Starfield with procedural stars
    Starfield {
        star_density: f64,
        star_brightness: f64,
    },
    /// Environment map (equirectangular HDR image)
    EnvMap {
        pixels: Arc<Vec<f32>>,
        width: u32,
        height: u32,
        intensity: f64,
    },
}

impl Background {
    fn color(&self, ray: &Ray) -> Color {
        match self {
            Background::SkyGradient => {
                let unit_dir = ray.direction.unit();
                let t = 0.5 * (unit_dir.y + 1.0);
                Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
            }
            Background::Solid(c) => *c,
            Background::Gradient { bottom, top } => {
                let unit_dir = ray.direction.unit();
                let t = 0.5 * (unit_dir.y + 1.0);
                *bottom * (1.0 - t) + *top * t
            }
            Background::Starfield { star_density, star_brightness } => {
                let unit_dir = ray.direction.unit();
                // Dark blue space
                let t = 0.5 * (unit_dir.y + 1.0);
                let space = Color::new(0.0, 0.0, 0.02) * (1.0 - t) + Color::new(0.02, 0.0, 0.05) * t;

                // Procedural stars using hash of quantized direction
                let scale = 500.0 * star_density;
                let ix = (unit_dir.x * scale).floor() as i64;
                let iy = (unit_dir.y * scale).floor() as i64;
                let iz = (unit_dir.z * scale).floor() as i64;
                let hash = (ix.wrapping_mul(73856093) ^ iy.wrapping_mul(19349663) ^ iz.wrapping_mul(83492791)) as u64;
                let star_val = (hash % 10000) as f64 / 10000.0;

                if star_val > 0.997 {
                    let intensity = star_val * star_brightness;
                    // Color variation
                    let r_hash = ((hash >> 8) % 100) as f64 / 100.0;
                    let star_color = if r_hash < 0.3 {
                        Color::new(1.0, 0.9, 0.8) // warm
                    } else if r_hash < 0.6 {
                        Color::new(0.9, 0.9, 1.0) // cool
                    } else {
                        Color::new(1.0, 1.0, 1.0) // white
                    };
                    space + star_color * intensity
                } else {
                    space
                }
            }
            Background::EnvMap { pixels, width, height, intensity } => {
                let unit_dir = ray.direction.unit();
                // Equirectangular mapping: direction -> (u, v)
                let theta = (-unit_dir.y).acos();
                let phi = (-unit_dir.z).atan2(unit_dir.x) + std::f64::consts::PI;
                let u = phi / (2.0 * std::f64::consts::PI);
                let v = theta / std::f64::consts::PI;

                let i = ((u * *width as f64) as u32).min(width - 1);
                let j = ((v * *height as f64) as u32).min(height - 1);
                let idx = (j * width + i) as usize * 3;

                if idx + 2 < pixels.len() {
                    Color::new(
                        pixels[idx] as f64 * intensity,
                        pixels[idx + 1] as f64 * intensity,
                        pixels[idx + 2] as f64 * intensity,
                    )
                } else {
                    Color::ZERO
                }
            }
            Background::Sun { direction, sun_color, sun_intensity, sky_color } => {
                let unit_dir = ray.direction.unit();
                let t = 0.5 * (unit_dir.y + 1.0);
                let sky = Color::new(1.0, 1.0, 1.0) * (1.0 - t) + *sky_color * t;

                // Sun disk
                let sun_dot = unit_dir.dot(direction.unit());
                if sun_dot > 0.9995 {
                    // Sharp sun disk
                    *sun_color * *sun_intensity
                } else if sun_dot > 0.995 {
                    // Sun glow halo
                    let glow = (sun_dot - 0.995) / (0.9995 - 0.995);
                    sky + *sun_color * (*sun_intensity * 0.3 * glow)
                } else {
                    sky
                }
            }
        }
    }
}

/// Tone mapping algorithm selection.
#[derive(Default, Clone, Copy)]
pub enum ToneMap {
    #[default]
    Aces,
    Reinhard,
    Filmic,
    None,
}

pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
    pub background: Background,
    pub seed: u64,
    pub quiet: bool,
    pub exposure: f64,
    pub tone_map: ToneMap,
    pub auto_exposure: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 450,
            samples_per_pixel: 100,
            max_depth: 50,
            background: Background::default(),
            seed: 31337,
            quiet: false,
            exposure: 1.0,
            tone_map: ToneMap::default(),
            auto_exposure: false,
        }
    }
}

/// Compute the log-average luminance of the rendered image.
/// Uses geometric mean: exp(avg(log(delta + luminance)))
fn compute_log_average_luminance(rows: &[Vec<Color>]) -> f64 {
    let delta = 1e-4; // Small constant to avoid log(0)
    let mut sum = 0.0;
    let mut count = 0u64;
    for row in rows {
        for color in row {
            let luminance = 0.2126 * color.x + 0.7152 * color.y + 0.0722 * color.z;
            sum += (delta + luminance.max(0.0)).ln();
            count += 1;
        }
    }
    if count == 0 {
        return 0.0;
    }
    (sum / count as f64).exp()
}

/// Sample direct illumination from a random light source (Next Event Estimation).
fn sample_direct_light(
    hit_point: &Point3,
    normal: &Vec3,
    albedo: &Color,
    lights: &[LightInfo],
    world: &dyn Hittable,
    rng: &mut dyn RngCore,
) -> Color {
    if lights.is_empty() {
        return Color::ZERO;
    }

    // Pick a random light
    let light_idx = (rng.next_f64() * lights.len() as f64) as usize;
    let light_idx = light_idx.min(lights.len() - 1);
    let light = &lights[light_idx];

    // Skip if hit point is inside the light sphere
    let to_center = light.center - *hit_point;
    if to_center.length_squared() < light.radius * light.radius {
        return Color::ZERO;
    }

    // Sample a random point on the light sphere using rejection sampling
    let mut lx;
    let mut ly;
    let mut lz;
    loop {
        lx = rng.next_f64() * 2.0 - 1.0;
        ly = rng.next_f64() * 2.0 - 1.0;
        lz = rng.next_f64() * 2.0 - 1.0;
        if lx * lx + ly * ly + lz * lz < 1.0 {
            break;
        }
    }
    let len = (lx * lx + ly * ly + lz * lz).sqrt();
    let light_point = Point3::new(
        light.center.x + light.radius * lx / len,
        light.center.y + light.radius * ly / len,
        light.center.z + light.radius * lz / len,
    );

    let to_light = light_point - *hit_point;
    let dist_sq = to_light.length_squared();
    let dist = dist_sq.sqrt();
    let dir = to_light / dist;

    // Cosine at the shading point
    let cos_theta = normal.dot(dir);
    if cos_theta <= 0.0 {
        return Color::ZERO;
    }

    // Cosine at the light surface (light normal points outward from center)
    let light_normal = (light_point - light.center) / light.radius;
    let cos_theta_light = (-dir).dot(light_normal);
    if cos_theta_light <= 0.0 {
        return Color::ZERO; // Back face of light
    }

    // Shadow ray: check if path to light is unoccluded
    let shadow_ray = crate::ray::Ray::new(*hit_point, dir);
    if let Some(shadow_hit) = world.hit(&shadow_ray, 0.001, dist - 0.001) {
        // Something blocks the light
        let _ = shadow_hit;
        return Color::ZERO;
    }

    // Light area: 4 * pi * r^2
    let area = 4.0 * std::f64::consts::PI * light.radius * light.radius;

    // Lambertian BRDF = albedo / pi
    // Monte Carlo estimate: emission * (albedo/pi) * cos_theta * cos_theta_light * area / dist^2 * N_lights
    light.emission.hadamard(*albedo)
        * (cos_theta * cos_theta_light * area * lights.len() as f64
            / (std::f64::consts::PI * dist_sq))
}

/// Trace a single ray through the scene.
///
/// `skip_emission`: when true, don't count emitted light on first hit.
/// This prevents double-counting when NEE already sampled this light.
fn ray_color(
    ray: &Ray,
    world: &dyn Hittable,
    lights: &[LightInfo],
    bg: &Background,
    rng: &mut dyn RngCore,
    depth: u32,
    skip_emission: bool,
) -> Color {
    if depth == 0 {
        return Color::ZERO;
    }

    // 0.001 to avoid shadow acne
    if let Some(hit) = world.hit(ray, 0.001, f64::INFINITY) {
        let emitted = if skip_emission {
            Color::ZERO
        } else {
            hit.material.emitted(hit.u, hit.v, &hit.point)
        };

        if let Some(scatter) = hit.material.scatter(ray, &hit, rng) {
            // Russian Roulette: probabilistically terminate low-contribution paths
            // after a minimum number of bounces, without introducing bias.
            let max_component = scatter.attenuation.x.max(scatter.attenuation.y).max(scatter.attenuation.z);
            let survival_prob = max_component.clamp(0.05, 1.0);
            if depth < 47 && rng.next_f64() > survival_prob {
                // Path terminated — return only emitted light
                return emitted;
            }
            let rr_weight = if depth < 47 { 1.0 / survival_prob } else { 1.0 };

            let use_nee = !hit.material.is_specular() && !lights.is_empty();

            // For diffuse materials, add direct light sampling (NEE)
            let direct = if use_nee {
                sample_direct_light(
                    &hit.point,
                    &hit.normal,
                    &scatter.attenuation,
                    lights,
                    world,
                    rng,
                )
            } else {
                Color::ZERO
            };

            // For the indirect bounce after NEE, skip emission to avoid double-counting
            let indirect = scatter
                .attenuation
                .hadamard(ray_color(&scatter.ray, world, lights, bg, rng, depth - 1, use_nee))
                * rr_weight;
            let result = emitted + direct + indirect;
            // Guard against NaN propagation from degenerate geometry or materials
            return if result.x.is_nan() || result.y.is_nan() || result.z.is_nan() {
                Color::ZERO
            } else {
                result
            };
        }
        return emitted;
    }

    bg.color(ray)
}

/// Render the scene and return a flat Vec of RGBA bytes.
pub fn render(
    config: &RenderConfig,
    camera: &Camera,
    world: &dyn Hittable,
    lights: &[LightInfo],
) -> Vec<u8> {
    let width = config.width as usize;
    let height = config.height as usize;

    let rows_done = AtomicUsize::new(0);
    let sqrt_spp = (config.samples_per_pixel as f64).sqrt().ceil() as u32;
    let actual_spp = sqrt_spp * sqrt_spp;
    let start_time = Instant::now();

    let rows: Vec<Vec<Color>> = (0..height)
        .into_par_iter()
        .map(|j| {
            let mut rng = SmallRng::seed_from_u64(j as u64 * config.seed);
            let y = (height - 1 - j) as f64;

            let row: Vec<Color> = (0..width)
                .map(|i| {
                    let mut color = Color::ZERO;
                    for sy in 0..sqrt_spp {
                        for sx in 0..sqrt_spp {
                            let u = (i as f64 + (sx as f64 + rng.random::<f64>()) / sqrt_spp as f64) / (width - 1) as f64;
                            let v = (y + (sy as f64 + rng.random::<f64>()) / sqrt_spp as f64) / (height - 1) as f64;
                            let ray = camera.get_ray(u, v, &mut rng);
                            let sample = ray_color(&ray, world, lights, &config.background, &mut rng, config.max_depth, false);
                            // Clamp per-sample luminance to prevent firefly artifacts
                            let luminance = 0.2126 * sample.x + 0.7152 * sample.y + 0.0722 * sample.z;
                            if luminance > 100.0 {
                                let scale = 100.0 / luminance;
                                color += sample * scale;
                            } else {
                                color += sample;
                            }
                        }
                    }
                    color / actual_spp as f64
                })
                .collect();

            let done = rows_done.fetch_add(1, Ordering::Relaxed) + 1;
            if !config.quiet {
                #[allow(clippy::manual_is_multiple_of)]
                if done % 20 == 0 || done == height {
                    let pct = done * 100 / height;
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let rays_so_far = done as u64 * width as u64 * actual_spp as u64;
                    let mrays = rays_so_far as f64 / elapsed / 1_000_000.0;
                    let eta = if done < height {
                        let remaining = elapsed / done as f64 * (height - done) as f64;
                        format!(" ETA {:.0}s", remaining)
                    } else {
                        String::new()
                    };
                    eprint!("\rProgress: {pct:3}% [{done}/{height} rows] {mrays:.1} Mrays/s{eta}   ");
                }
            }

            row
        })
        .collect();

    if !config.quiet {
        eprintln!();
        let total_rays = width as u64 * height as u64 * actual_spp as u64;
        eprintln!("Primary rays: {total_rays} ({actual_spp} spp, {sqrt_spp}x{sqrt_spp} stratified)");
    }

    // Auto-exposure: compute exposure from scene luminance if not manually overridden.
    let exposure = if config.auto_exposure {
        let log_avg = compute_log_average_luminance(&rows);
        let auto_exp = if log_avg > 1e-6 {
            0.18 / log_avg // Key value mapping
        } else {
            1.0
        };
        if !config.quiet {
            eprintln!("Auto-exposure: {auto_exp:.3} (scene avg luminance: {log_avg:.4})");
        }
        auto_exp * config.exposure // Allow manual fine-tuning on top
    } else {
        config.exposure
    };

    // Convert to RGBA bytes with exposure, tone mapping + gamma correction.
    let tone_fn: fn(f64) -> f64 = match config.tone_map {
        ToneMap::Aces => aces_tonemap,
        ToneMap::Reinhard => reinhard_tonemap,
        ToneMap::Filmic => filmic_tonemap,
        ToneMap::None => |x: f64| x.clamp(0.0, 1.0),
    };
    let mut pixels = Vec::with_capacity(width * height * 4);
    for row in &rows {
        for color in row {
            let r = linear_to_srgb(tone_fn(color.x * exposure));
            let g = linear_to_srgb(tone_fn(color.y * exposure));
            let b = linear_to_srgb(tone_fn(color.z * exposure));
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    pixels
}

impl Background {
    /// Load an environment map from an image file (HDR, PNG, JPG).
    pub fn load_env_map(path: &str, intensity: f64) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("Failed to load env map '{path}': {e}"))?;
        let rgb = img.to_rgb32f();
        let width = rgb.width();
        let height = rgb.height();
        let pixels: Vec<f32> = rgb.into_raw();
        Ok(Background::EnvMap {
            pixels: Arc::new(pixels),
            width,
            height,
            intensity,
        })
    }
}

/// ACES filmic tone mapping curve.
/// Maps HDR values to [0, 1] with a pleasing S-curve.
fn aces_tonemap(x: f64) -> f64 {
    let x = x.max(0.0);
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    ((x * (a * x + b)) / (x * (c * x + d) + e)).clamp(0.0, 1.0)
}

/// Reinhard tone mapping: simple x / (1 + x) curve.
fn reinhard_tonemap(x: f64) -> f64 {
    let x = x.max(0.0);
    x / (1.0 + x)
}

/// Uncharted 2 filmic tone mapping (John Hable).
/// Produces a cinematic look with deep blacks and soft highlights.
fn filmic_tonemap(x: f64) -> f64 {
    let x = x.max(0.0);
    // Uncharted 2 tone mapping formula
    fn uc2(v: f64) -> f64 {
        let a = 0.15; // Shoulder strength
        let b = 0.50; // Linear strength
        let c = 0.10; // Linear angle
        let d = 0.20; // Toe strength
        let e = 0.02; // Toe numerator
        let f = 0.30; // Toe denominator
        ((v * (a * v + c * b) + d * e) / (v * (a * v + b) + d * f)) - e / f
    }
    let w = 11.2; // Linear white point
    (uc2(x) / uc2(w)).clamp(0.0, 1.0)
}

/// Convert linear [0,1] to sRGB byte using the official piecewise transfer function.
fn linear_to_srgb(x: f64) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let s = if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (s.clamp(0.0, 0.999) * 256.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aces_tonemap_zero() {
        assert_eq!(aces_tonemap(0.0), 0.0);
    }

    #[test]
    fn aces_tonemap_bounded() {
        for i in 0..100 {
            let x = i as f64 * 0.5;
            let y = aces_tonemap(x);
            assert!(y >= 0.0 && y <= 1.0, "aces_tonemap({x}) = {y} out of [0,1]");
        }
    }

    #[test]
    fn aces_tonemap_monotonic() {
        let mut prev = 0.0;
        for i in 1..100 {
            let x = i as f64 * 0.1;
            let y = aces_tonemap(x);
            assert!(y >= prev, "ACES should be monotonically increasing");
            prev = y;
        }
    }

    #[test]
    fn reinhard_tonemap_basic() {
        assert_eq!(reinhard_tonemap(0.0), 0.0);
        assert!((reinhard_tonemap(1.0) - 0.5).abs() < 1e-6);
        // Monotonic
        let mut prev = 0.0;
        for i in 1..50 {
            let x = i as f64;
            let y = reinhard_tonemap(x);
            assert!(y > prev);
            assert!(y < 1.0);
            prev = y;
        }
    }

    #[test]
    fn filmic_tonemap_zero() {
        assert!((filmic_tonemap(0.0)).abs() < 0.01);
    }

    #[test]
    fn filmic_tonemap_bounded() {
        for i in 0..100 {
            let x = i as f64 * 0.5;
            let y = filmic_tonemap(x);
            assert!(y >= 0.0 && y <= 1.0, "filmic_tonemap({x}) = {y} out of [0,1]");
        }
    }

    #[test]
    fn filmic_tonemap_monotonic() {
        let mut prev = 0.0;
        for i in 1..100 {
            let x = i as f64 * 0.1;
            let y = filmic_tonemap(x);
            assert!(y >= prev, "Filmic should be monotonically increasing");
            prev = y;
        }
    }

    #[test]
    fn log_avg_luminance_basic() {
        // Uniform gray image: all pixels = (0.5, 0.5, 0.5)
        let gray = Color::new(0.5, 0.5, 0.5);
        let rows = vec![vec![gray; 10]; 10];
        let avg = compute_log_average_luminance(&rows);
        let expected_lum = 0.2126 * 0.5 + 0.7152 * 0.5 + 0.0722 * 0.5;
        assert!((avg - expected_lum).abs() < 0.01, "Expected ~{expected_lum}, got {avg}");
    }

    #[test]
    fn log_avg_luminance_dark_scene() {
        // Very dark scene should have low luminance
        let dark = Color::new(0.01, 0.01, 0.01);
        let rows = vec![vec![dark; 10]; 10];
        let avg = compute_log_average_luminance(&rows);
        assert!(avg < 0.05, "Dark scene should have low avg luminance, got {avg}");
    }

    #[test]
    fn srgb_black() {
        assert_eq!(linear_to_srgb(0.0), 0);
    }

    #[test]
    fn srgb_white() {
        assert_eq!(linear_to_srgb(1.0), 255);
    }

    #[test]
    fn srgb_monotonic() {
        let mut prev = 0u8;
        for i in 1..=100 {
            let x = i as f64 / 100.0;
            let y = linear_to_srgb(x);
            assert!(y >= prev, "sRGB should be monotonically increasing");
            prev = y;
        }
    }

    #[test]
    fn srgb_clamps_negative() {
        assert_eq!(linear_to_srgb(-1.0), 0);
    }

    #[test]
    fn nee_direct_light_nonzero() {
        use crate::material::{Lambertian, RngCore};
        use crate::vec3::{Color, Point3, Vec3};
        use crate::sphere::Sphere;
        use crate::hit::HittableList;
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        // Build a simple scene: ground plane (sphere) + light above
        let mut world = HittableList::new();
        world.add(Box::new(Sphere::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            Box::new(Lambertian::new(Color::new(0.8, 0.8, 0.8))),
        )));
        world.add(Box::new(Sphere::new(
            Point3::new(0.0, 5.0, 0.0),
            1.0,
            Box::new(crate::material::Emissive::new(Color::new(1.0, 1.0, 1.0), 10.0)),
        )));

        let lights = vec![LightInfo {
            center: Point3::new(0.0, 5.0, 0.0),
            radius: 1.0,
            emission: Color::new(10.0, 10.0, 10.0),
        }];

        let mut rng = SmallRng::seed_from_u64(42);
        let hit_point = Point3::new(0.0, 0.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let albedo = Color::new(0.8, 0.8, 0.8);

        // Sample many times; average should be positive
        let mut total = Color::ZERO;
        let n = 1000;
        for _ in 0..n {
            total += sample_direct_light(&hit_point, &normal, &albedo, &lights, &world, &mut rng);
        }
        let avg = total / n as f64;
        assert!(avg.x > 0.0, "Direct light sampling should produce positive value, got {}", avg.x);
        assert!(avg.y > 0.0, "Direct light sampling should produce positive value, got {}", avg.y);
    }

    #[test]
    fn nee_no_lights_returns_zero() {
        use crate::material::RngCore;
        use crate::vec3::{Color, Point3, Vec3};
        use crate::hit::HittableList;
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        let world = HittableList::new();
        let lights: Vec<LightInfo> = vec![];
        let mut rng = SmallRng::seed_from_u64(42);
        let hit_point = Point3::new(0.0, 0.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let albedo = Color::new(0.8, 0.8, 0.8);

        let result = sample_direct_light(&hit_point, &normal, &albedo, &lights, &world, &mut rng);
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn nee_occluded_returns_zero() {
        use crate::material::{Lambertian, RngCore};
        use crate::vec3::{Color, Point3, Vec3};
        use crate::sphere::Sphere;
        use crate::hit::HittableList;
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        // Light above, but a sphere blocks the path
        let mut world = HittableList::new();
        // Blocker sphere between hit point and light
        world.add(Box::new(Sphere::new(
            Point3::new(0.0, 2.5, 0.0),
            1.0,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        )));
        // Light sphere
        world.add(Box::new(Sphere::new(
            Point3::new(0.0, 5.0, 0.0),
            0.5,
            Box::new(crate::material::Emissive::new(Color::new(1.0, 1.0, 1.0), 10.0)),
        )));

        let lights = vec![LightInfo {
            center: Point3::new(0.0, 5.0, 0.0),
            radius: 0.5,
            emission: Color::new(10.0, 10.0, 10.0),
        }];

        let mut rng = SmallRng::seed_from_u64(42);
        let hit_point = Point3::new(0.0, 0.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let albedo = Color::new(0.8, 0.8, 0.8);

        // All samples should be zero since the blocker fully occludes the light
        let mut total = Color::ZERO;
        for _ in 0..100 {
            total += sample_direct_light(&hit_point, &normal, &albedo, &lights, &world, &mut rng);
        }
        assert_eq!(total.x, 0.0, "Occluded light should contribute zero");
    }
}
