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
pub enum LightInfo {
    Sphere {
        center: Point3,
        radius: f64,
        emission: Color,
    },
    Rect {
        /// Corner of the rectangle
        origin: Point3,
        /// Two edge vectors defining the rectangle
        u: Vec3,
        v: Vec3,
        /// Normal of the rectangle
        normal: Vec3,
        emission: Color,
    },
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
    pub denoise: bool,
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
            denoise: false,
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

/// Edge-preserving bilateral denoiser operating on HDR image data.
/// Uses a spatial Gaussian kernel combined with a range (color similarity) kernel
/// to smooth noise while preserving edges and detail.
fn bilateral_denoise(rows: &[Vec<Color>]) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();

    // Bilateral filter parameters
    let radius: i32 = 3; // 7x7 kernel
    let sigma_spatial = 2.0_f64;
    let sigma_range = 0.15_f64; // Color similarity threshold (in linear HDR space)
    let spatial_denom = -1.0 / (2.0 * sigma_spatial * sigma_spatial);
    let range_denom = -1.0 / (2.0 * sigma_range * sigma_range);

    let mut result = vec![vec![Color::ZERO; width]; height];

    // Process rows in parallel for performance
    result.par_iter_mut().enumerate().for_each(|(j, out_row)| {
        for i in 0..width {
            let center = rows[j][i];
            let center_lum = 0.2126 * center.x + 0.7152 * center.y + 0.0722 * center.z;
            let mut sum = Color::ZERO;
            let mut weight_sum = 0.0_f64;

            for dy in -radius..=radius {
                let ny = j as i32 + dy;
                if ny < 0 || ny >= height as i32 {
                    continue;
                }
                let ny = ny as usize;
                for dx in -radius..=radius {
                    let nx = i as i32 + dx;
                    if nx < 0 || nx >= width as i32 {
                        continue;
                    }
                    let nx = nx as usize;

                    let neighbor = rows[ny][nx];
                    let neighbor_lum = 0.2126 * neighbor.x + 0.7152 * neighbor.y + 0.0722 * neighbor.z;

                    // Spatial weight (Gaussian based on pixel distance)
                    let dist_sq = (dx * dx + dy * dy) as f64;
                    let w_spatial = (dist_sq * spatial_denom).exp();

                    // Range weight (Gaussian based on luminance difference)
                    let lum_diff = center_lum - neighbor_lum;
                    let w_range = (lum_diff * lum_diff * range_denom).exp();

                    let w = w_spatial * w_range;
                    sum += neighbor * w;
                    weight_sum += w;
                }
            }

            out_row[i] = if weight_sum > 0.0 {
                sum / weight_sum
            } else {
                center
            };
        }
    });

    result
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

    // Sample a point on the selected light
    let (light_point, light_normal, area, emission) = match light {
        LightInfo::Sphere { center, radius, emission } => {
            // Skip if hit point is inside the light sphere
            let to_center = *center - *hit_point;
            if to_center.length_squared() < radius * radius {
                return Color::ZERO;
            }
            // Sample random point on sphere surface
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
            let p = Point3::new(
                center.x + radius * lx / len,
                center.y + radius * ly / len,
                center.z + radius * lz / len,
            );
            let n = (p - *center) / *radius;
            let a = 4.0 * std::f64::consts::PI * radius * radius;
            (p, n, a, *emission)
        }
        LightInfo::Rect { origin, u, v, normal: n, emission } => {
            let s = rng.next_f64();
            let t = rng.next_f64();
            let p = *origin + *u * s + *v * t;
            let a = u.cross(*v).length();
            (p, *n, a, *emission)
        }
    };

    let to_light = light_point - *hit_point;
    let dist_sq = to_light.length_squared();
    let dist = dist_sq.sqrt();
    let dir = to_light / dist;

    // Cosine at the shading point
    let cos_theta = normal.dot(dir);
    if cos_theta <= 0.0 {
        return Color::ZERO;
    }

    // Cosine at the light surface
    let cos_theta_light = (-dir).dot(light_normal).abs();
    if cos_theta_light <= 0.001 {
        return Color::ZERO;
    }

    // Shadow ray
    let shadow_ray = crate::ray::Ray::new(*hit_point, dir);
    if world.hit(&shadow_ray, 0.001, dist - 0.001).is_some() {
        return Color::ZERO;
    }

    // Monte Carlo: emission * (albedo/pi) * cos_theta * cos_theta_light * area / dist^2 * N_lights
    emission.hadamard(*albedo)
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

    // Optional bilateral denoising pass (operates on HDR data before tone mapping)
    let rows = if config.denoise {
        if !config.quiet {
            eprint!("Denoising...");
        }
        let denoised = bilateral_denoise(&rows);
        if !config.quiet {
            eprintln!(" done");
        }
        denoised
    } else {
        rows
    };

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

        let lights = vec![LightInfo::Sphere {
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

        let lights = vec![LightInfo::Sphere {
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

    #[test]
    fn bilateral_denoise_preserves_uniform() {
        // Uniform image should be unchanged by denoising
        let rows = vec![vec![Color::new(0.5, 0.5, 0.5); 10]; 10];
        let result = bilateral_denoise(&rows);
        for row in &result {
            for c in row {
                assert!((c.x - 0.5).abs() < 1e-6);
                assert!((c.y - 0.5).abs() < 1e-6);
                assert!((c.z - 0.5).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn bilateral_denoise_smooths_gradual_noise() {
        // Slightly noisy image — noise within range sigma should be smoothed
        let mut rows = vec![vec![Color::new(0.5, 0.5, 0.5); 10]; 10];
        // Add mild noise within the range sigma
        rows[5][5] = Color::new(0.6, 0.6, 0.6);
        rows[5][6] = Color::new(0.4, 0.4, 0.4);
        let result = bilateral_denoise(&rows);
        // Neighbors should pull the noisy pixels closer to 0.5
        assert!((result[5][5].x - 0.5).abs() < 0.05, "Mild noise should be smoothed toward mean");
        assert!((result[5][6].x - 0.5).abs() < 0.05, "Mild noise should be smoothed toward mean");
    }

    #[test]
    fn bilateral_denoise_preserves_edges() {
        // Sharp edge: left half dark, right half bright
        let mut rows = vec![vec![Color::ZERO; 20]; 10];
        for row in &mut rows {
            for i in 10..20 {
                row[i] = Color::new(1.0, 1.0, 1.0);
            }
        }
        let result = bilateral_denoise(&rows);
        // Deep in dark region should stay dark
        assert!(result[5][2].x < 0.05, "Dark region should stay dark");
        // Deep in bright region should stay bright
        assert!(result[5][17].x > 0.95, "Bright region should stay bright");
    }

    #[test]
    fn bilateral_denoise_empty() {
        let rows: Vec<Vec<Color>> = vec![];
        let result = bilateral_denoise(&rows);
        assert!(result.is_empty());
    }
}
