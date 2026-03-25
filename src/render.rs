use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
    Disk {
        center: Point3,
        normal: Vec3,
        radius: f64,
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

                // Bilinear interpolation
                let fx = u * *width as f64 - 0.5;
                let fy = v * *height as f64 - 0.5;
                let x0 = (fx.floor() as i64).rem_euclid(*width as i64) as u32;
                let y0 = fy.floor().clamp(0.0, *height as f64 - 1.0) as u32;
                let x1 = (x0 + 1) % width;
                let y1 = (y0 + 1).min(height - 1);
                let tx = fx - fx.floor();
                let ty = fy - fy.floor();
                let w = *width as usize;
                let sample = |xi: u32, yi: u32| -> Color {
                    let idx = (yi as usize * w + xi as usize) * 3;
                    if idx + 2 < pixels.len() {
                        Color::new(pixels[idx] as f64, pixels[idx + 1] as f64, pixels[idx + 2] as f64)
                    } else {
                        Color::ZERO
                    }
                };
                let c00 = sample(x0, y0);
                let c10 = sample(x1, y0);
                let c01 = sample(x0, y1);
                let c11 = sample(x1, y1);
                let c = c00 * ((1.0 - tx) * (1.0 - ty))
                    + c10 * (tx * (1.0 - ty))
                    + c01 * ((1.0 - tx) * ty)
                    + c11 * (tx * ty);
                c * *intensity
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

/// Pixel reconstruction filter for anti-aliasing.
#[derive(Default, Clone, Copy)]
pub enum PixelFilter {
    /// Box filter — uniform weighting (fastest, default).
    #[default]
    Box,
    /// Triangle (tent) filter — linear falloff from center.
    Triangle,
    /// Gaussian filter — smooth bell curve weighting.
    Gaussian,
    /// Mitchell-Netravali filter — balanced sharpness/smoothness.
    Mitchell,
}

impl PixelFilter {
    /// Evaluate the filter weight for a sample at offset (dx, dy) from pixel center.
    /// Offsets are in [-0.5, 0.5] range within the pixel.
    fn weight(&self, dx: f64, dy: f64) -> f64 {
        match self {
            PixelFilter::Box => 1.0,
            PixelFilter::Triangle => {
                (1.0 - dx.abs() * 2.0).max(0.0) * (1.0 - dy.abs() * 2.0).max(0.0)
            }
            PixelFilter::Gaussian => {
                let sigma = 0.35;
                let r2 = dx * dx + dy * dy;
                (-r2 / (2.0 * sigma * sigma)).exp()
            }
            PixelFilter::Mitchell => {
                mitchell_1d(dx * 2.0) * mitchell_1d(dy * 2.0)
            }
        }
    }
}

/// Mitchell-Netravali 1D filter with B=1/3, C=1/3.
fn mitchell_1d(x: f64) -> f64 {
    let x = x.abs();
    let (b, c) = (1.0 / 3.0, 1.0 / 3.0);
    if x < 1.0 {
        ((12.0 - 9.0 * b - 6.0 * c) * x * x * x
            + (-18.0 + 12.0 * b + 6.0 * c) * x * x
            + (6.0 - 2.0 * b))
            / 6.0
    } else if x < 2.0 {
        ((-b - 6.0 * c) * x * x * x
            + (6.0 * b + 30.0 * c) * x * x
            + (-12.0 * b - 48.0 * c) * x
            + (8.0 * b + 24.0 * c))
            / 6.0
    } else {
        0.0
    }
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
    pub save_hdr: bool,
    /// Optional crop region: (x, y, width, height) in pixels
    pub crop: Option<(u32, u32, u32, u32)>,
    /// Bloom intensity (0.0 = off). Adds glow around bright areas.
    pub bloom: f64,
    /// Vignette strength (0.0 = off). Darkens edges for cinematic look.
    pub vignette: f64,
    /// Film grain intensity (0.0 = off). Adds photographic noise.
    pub grain: f64,
    /// Saturation adjustment (1.0 = normal, 0.0 = grayscale, >1.0 = boosted).
    pub saturation: f64,
    /// Contrast adjustment (1.0 = normal, <1 = lower contrast, >1 = higher).
    pub contrast: f64,
    /// White balance temperature shift (0 = neutral, negative = cooler/blue, positive = warmer/orange).
    pub white_balance: f64,
    /// Sharpen intensity (0.0 = off). Enhances detail via unsharp mask.
    pub sharpen: f64,
    /// Hue rotation in degrees (0 = no change, 180 = complementary colors).
    pub hue_shift: f64,
    /// Enable ordered dithering to reduce banding in 8-bit output.
    pub dither: bool,
    /// Custom gamma value (0 = use sRGB transfer function, >0 = simple power curve).
    pub gamma: f64,
    /// Pixel reconstruction filter for anti-aliasing.
    pub pixel_filter: PixelFilter,
    /// Chromatic aberration strength (0.0 = off). Shifts RGB channels radially.
    pub chromatic_aberration: f64,
    /// Lens distortion (0.0 = off, positive = barrel, negative = pincushion).
    pub lens_distortion: f64,
    /// Generate depth pass output.
    pub save_depth: bool,
    /// Generate normal pass output.
    pub save_normals: bool,
    /// Firefly removal threshold (0.0 = off). Higher = more aggressive.
    /// Replaces pixels whose luminance exceeds neighbors by this factor.
    pub firefly_filter: f64,
    /// Enable adaptive sampling: concentrate samples on noisy regions.
    pub adaptive: bool,
    /// Adaptive sampling noise threshold (0.01 = aggressive, 0.1 = conservative).
    /// Pixels with variance below this stop early.
    pub adaptive_threshold: f64,
    /// Maximum render time in seconds (0 = no limit). Render stops when exceeded.
    pub time_limit: f64,
    /// Posterize level (0 = off, 2-256 = number of discrete color levels per channel).
    pub posterize: u32,
    /// Sepia tone intensity (0.0 = off, 1.0 = full sepia).
    pub sepia: f64,
    /// Edge detection / outline strength (0.0 = off). Darkens edges for toon look.
    pub edge_detect: f64,
    /// Pixelate block size (0 = off, N = NxN pixel blocks for retro look).
    pub pixelate: u32,
    /// Invert colors (false = normal, true = negative image).
    pub invert: bool,
    /// Scanline intensity (0.0 = off). Simulates CRT scanlines.
    pub scanlines: f64,
    /// Black-and-white threshold (negative = off, 0.0-1.0 = luminance threshold).
    pub threshold: f64,
    /// Gaussian blur radius (0 = off). Softens the entire image.
    pub blur: f64,
    /// Tilt-shift effect: blur amount at edges (0 = off). Simulates miniature photography.
    pub tilt_shift: f64,
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
            save_hdr: false,
            crop: None,
            bloom: 0.0,
            vignette: 0.0,
            grain: 0.0,
            saturation: 1.0,
            contrast: 1.0,
            white_balance: 0.0,
            sharpen: 0.0,
            hue_shift: 0.0,
            dither: false,
            gamma: 0.0,
            pixel_filter: PixelFilter::default(),
            firefly_filter: 0.0,
            chromatic_aberration: 0.0,
            lens_distortion: 0.0,
            save_depth: false,
            save_normals: false,
            adaptive: false,
            adaptive_threshold: 0.03,
            time_limit: 0.0,
            posterize: 0,
            sepia: 0.0,
            edge_detect: 0.0,
            pixelate: 0,
            invert: false,
            scanlines: 0.0,
            threshold: -1.0,
            blur: 0.0,
            tilt_shift: 0.0,
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

/// Apply bloom (glow) post-processing to HDR image data.
/// Extracts bright pixels above a luminance threshold, applies a multi-pass
/// Gaussian blur, then blends the result back into the original image.
/// Remove firefly pixels whose luminance is far above their neighbors.
/// Uses a 3x3 neighborhood median comparison.
fn remove_fireflies(rows: &[Vec<Color>], threshold: f64) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();
    let mut result = rows.to_vec();

    let luminance = |c: &Color| 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;

    for j in 0..height {
        for i in 0..width {
            let center_lum = luminance(&rows[j][i]);
            if center_lum < 0.01 {
                continue;
            }

            // Collect neighbor luminances (3x3, excluding center)
            let mut neighbors = Vec::with_capacity(8);
            for dj in [j.wrapping_sub(1), j, j + 1] {
                for di in [i.wrapping_sub(1), i, i + 1] {
                    if dj < height && di < width && !(dj == j && di == i) {
                        neighbors.push(luminance(&rows[dj][di]));
                    }
                }
            }
            if neighbors.is_empty() {
                continue;
            }

            // Compute median of neighbors
            neighbors.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let median = neighbors[neighbors.len() / 2];

            // If center pixel is much brighter than median neighbor, replace it
            if center_lum > median * threshold + 0.1 {
                // Replace with average of neighbors
                let mut avg = Color::ZERO;
                for dj in [j.wrapping_sub(1), j, j + 1] {
                    for di in [i.wrapping_sub(1), i, i + 1] {
                        if dj < height && di < width && !(dj == j && di == i) {
                            avg += rows[dj][di];
                        }
                    }
                }
                result[j][i] = avg / neighbors.len() as f64;
            }
        }
    }
    result
}

fn apply_bloom(rows: &[Vec<Color>], intensity: f64) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();
    let threshold = 1.0; // Extract pixels brighter than 1.0 (pre-tonemapping)

    // Step 1: Extract bright pixels
    let mut bright: Vec<Vec<Color>> = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|c| {
                    let lum = 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
                    if lum > threshold {
                        *c - Color::new(threshold, threshold, threshold)
                    } else {
                        Color::ZERO
                    }
                })
                .collect()
        })
        .collect();

    // Step 2: Multi-pass downscale + blur for wide glow
    // We do 4 passes of a 5-tap Gaussian blur, which approximates a large kernel
    let kernel = [0.06136, 0.24477, 0.38774, 0.24477, 0.06136];
    for _ in 0..4 {
        // Horizontal pass
        let mut temp = vec![vec![Color::ZERO; width]; height];
        for (temp_row, bright_row) in temp.iter_mut().zip(bright.iter()) {
            for (x, temp_px) in temp_row.iter_mut().enumerate() {
                let mut sum = Color::ZERO;
                for (k, &w) in kernel.iter().enumerate() {
                    let sx = (x as i64 + k as i64 - 2).clamp(0, width as i64 - 1) as usize;
                    sum += bright_row[sx] * w;
                }
                *temp_px = sum;
            }
        }
        // Vertical pass
        for (y, bright_row) in bright.iter_mut().enumerate() {
            for (x, bright_px) in bright_row.iter_mut().enumerate() {
                let mut sum = Color::ZERO;
                for (k, &w) in kernel.iter().enumerate() {
                    let sy = (y as i64 + k as i64 - 2).clamp(0, height as i64 - 1) as usize;
                    sum += temp[sy][x] * w;
                }
                *bright_px = sum;
            }
        }
    }

    // Step 3: Blend bloom back into original
    rows.iter()
        .zip(bright.iter())
        .map(|(orig_row, bloom_row)| {
            orig_row
                .iter()
                .zip(bloom_row.iter())
                .map(|(orig, bloom)| *orig + *bloom * intensity)
                .collect()
        })
        .collect()
}

/// Apply unsharp mask sharpening to HDR image data.
/// Subtracts a blurred version from the original to enhance edges.
/// Apply barrel/pincushion lens distortion.
/// Positive k = barrel (edges bend outward), negative k = pincushion (edges bend inward).
fn apply_lens_distortion(rows: &[Vec<Color>], k: f64) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    let max_r = (cx * cx + cy * cy).sqrt();

    let mut result = vec![vec![Color::ZERO; width]; height];

    for (j, row) in result.iter_mut().enumerate() {
        for (i, pixel) in row.iter_mut().enumerate() {
            let dx = (i as f64 + 0.5 - cx) / max_r;
            let dy = (j as f64 + 0.5 - cy) / max_r;
            let r2 = dx * dx + dy * dy;
            // Brown-Conrady distortion model (radial only)
            let factor = 1.0 + k * r2;
            let src_x = cx + dx * factor * max_r;
            let src_y = cy + dy * factor * max_r;

            // Bilinear sample all channels
            let r = sample_channel_bilinear(rows, src_x, src_y, width, height, 0);
            let g = sample_channel_bilinear(rows, src_x, src_y, width, height, 1);
            let b = sample_channel_bilinear(rows, src_x, src_y, width, height, 2);
            *pixel = Color::new(r, g, b);
        }
    }
    result
}

/// Apply chromatic aberration by radially shifting R, G, B channels.
/// Red shifts outward, blue shifts inward, green stays centered.
fn apply_chromatic_aberration(rows: &[Vec<Color>], strength: f64) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    let mut result = vec![vec![Color::ZERO; width]; height];

    for j in 0..height {
        for i in 0..width {
            let dx = i as f64 + 0.5 - cx;
            let dy = j as f64 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            // Scale shift by distance from center (stronger at edges)
            let shift = strength * (dist / max_dist);

            // Red channel: sample shifted outward
            let r_x = cx + dx * (1.0 + shift);
            let r_y = cy + dy * (1.0 + shift);
            let r = sample_channel_bilinear(rows, r_x, r_y, width, height, 0);

            // Green channel: no shift
            let g = rows[j][i].y;

            // Blue channel: sample shifted inward
            let b_x = cx + dx * (1.0 - shift);
            let b_y = cy + dy * (1.0 - shift);
            let b = sample_channel_bilinear(rows, b_x, b_y, width, height, 2);

            result[j][i] = Color::new(r, g, b);
        }
    }
    result
}

/// Bilinear sample a single color channel from the HDR image.
/// channel: 0=R, 1=G, 2=B
fn sample_channel_bilinear(rows: &[Vec<Color>], x: f64, y: f64, width: usize, height: usize, channel: usize) -> f64 {
    let x = x - 0.5;
    let y = y - 0.5;
    let x0 = (x.floor() as isize).clamp(0, width as isize - 1) as usize;
    let y0 = (y.floor() as isize).clamp(0, height as isize - 1) as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);
    let fx = (x - x.floor()).clamp(0.0, 1.0);
    let fy = (y - y.floor()).clamp(0.0, 1.0);

    let get = |row: usize, col: usize| match channel {
        0 => rows[row][col].x,
        1 => rows[row][col].y,
        _ => rows[row][col].z,
    };

    let top = get(y0, x0) * (1.0 - fx) + get(y0, x1) * fx;
    let bot = get(y1, x0) * (1.0 - fx) + get(y1, x1) * fx;
    top * (1.0 - fy) + bot * fy
}

fn apply_sharpen(rows: &[Vec<Color>], intensity: f64) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();

    // 3x3 box blur for the "unsharp" reference
    rows.iter()
        .enumerate()
        .map(|(j, row)| {
            row.iter()
                .enumerate()
                .map(|(i, orig)| {
                    let mut blur = Color::ZERO;
                    let mut count = 0.0;
                    for dy in -1i64..=1 {
                        for dx in -1i64..=1 {
                            let ny = (j as i64 + dy).clamp(0, height as i64 - 1) as usize;
                            let nx = (i as i64 + dx).clamp(0, width as i64 - 1) as usize;
                            blur += rows[ny][nx];
                            count += 1.0;
                        }
                    }
                    blur /= count;
                    // Unsharp mask: original + intensity * (original - blur)
                    let sharpened = *orig + (*orig - blur) * intensity;
                    // Prevent negative values
                    Color::new(sharpened.x.max(0.0), sharpened.y.max(0.0), sharpened.z.max(0.0))
                })
                .collect()
        })
        .collect()
}

/// Sobel edge detection: detects edges and darkens them for an outline effect.
fn apply_edge_detect(rows: &[Vec<Color>], strength: f64) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();

    let lum = |c: &Color| 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;

    rows.iter()
        .enumerate()
        .map(|(j, row)| {
            row.iter()
                .enumerate()
                .map(|(i, orig)| {
                    if j == 0 || j == height - 1 || i == 0 || i == width - 1 {
                        return *orig;
                    }
                    // Sobel kernels
                    let tl = lum(&rows[j - 1][i - 1]);
                    let tc = lum(&rows[j - 1][i]);
                    let tr = lum(&rows[j - 1][i + 1]);
                    let ml = lum(&rows[j][i - 1]);
                    let mr = lum(&rows[j][i + 1]);
                    let bl = lum(&rows[j + 1][i - 1]);
                    let bc = lum(&rows[j + 1][i]);
                    let br = lum(&rows[j + 1][i + 1]);

                    let gx = -tl - 2.0 * ml - bl + tr + 2.0 * mr + br;
                    let gy = -tl - 2.0 * tc - tr + bl + 2.0 * bc + br;
                    let edge = (gx * gx + gy * gy).sqrt().min(1.0);

                    // Darken by edge strength
                    *orig * (1.0 - edge * strength)
                })
                .collect()
        })
        .collect()
}

/// Separable Gaussian blur in HDR space.
fn apply_blur(rows: &[Vec<Color>], sigma: f64) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 || sigma < 0.5 {
        return rows.to_vec();
    }
    let width = rows[0].len();
    let radius = (sigma * 3.0).ceil() as i64;

    // Build 1D Gaussian kernel
    let mut kernel = Vec::with_capacity((2 * radius + 1) as usize);
    let mut sum = 0.0;
    for i in -radius..=radius {
        let w = (-0.5 * (i as f64 / sigma).powi(2)).exp();
        kernel.push(w);
        sum += w;
    }
    for w in &mut kernel {
        *w /= sum;
    }

    // Horizontal pass
    let mut temp: Vec<Vec<Color>> = vec![vec![Color::ZERO; width]; height];
    for (j, row) in temp.iter_mut().enumerate() {
        for (i, pixel) in row.iter_mut().enumerate() {
            let mut c = Color::ZERO;
            for (k, &w) in kernel.iter().enumerate() {
                let x = (i as i64 + k as i64 - radius).clamp(0, width as i64 - 1) as usize;
                c += rows[j][x] * w;
            }
            *pixel = c;
        }
    }

    // Vertical pass
    let mut result: Vec<Vec<Color>> = vec![vec![Color::ZERO; width]; height];
    for (j, row) in result.iter_mut().enumerate() {
        for (i, pixel) in row.iter_mut().enumerate() {
            let mut c = Color::ZERO;
            for (k, &w) in kernel.iter().enumerate() {
                let y = (j as i64 + k as i64 - radius).clamp(0, height as i64 - 1) as usize;
                c += temp[y][i] * w;
            }
            *pixel = c;
        }
    }

    result
}

/// Tilt-shift: blur top and bottom of image, leaving center sharp.
fn apply_tilt_shift(rows: &[Vec<Color>], strength: f64) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return rows.to_vec();
    }
    let width = rows[0].len();
    let center = height as f64 / 2.0;
    let band = height as f64 * 0.15; // sharp band = 30% of image height

    let mut result = rows.to_vec();
    for j in 0..height {
        let dist = ((j as f64 - center).abs() - band).max(0.0) / (center - band).max(1.0);
        let blur_radius = (dist * strength * 5.0).round() as i64;
        if blur_radius <= 0 {
            continue;
        }
        for i in 0..width {
            let mut sum = Color::ZERO;
            let mut count = 0.0;
            for dy in -blur_radius..=blur_radius {
                for dx in -blur_radius..=blur_radius {
                    let ny = (j as i64 + dy).clamp(0, height as i64 - 1) as usize;
                    let nx = (i as i64 + dx).clamp(0, width as i64 - 1) as usize;
                    sum += rows[ny][nx];
                    count += 1.0;
                }
            }
            result[j][i] = sum / count;
        }
    }
    result
}

/// Pixelate: average NxN blocks for a retro pixel-art effect.
fn apply_pixelate(rows: &[Vec<Color>], block: u32) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 || block < 2 {
        return rows.to_vec();
    }
    let width = rows[0].len();
    let bs = block as usize;
    let mut result: Vec<Vec<Color>> = rows.to_vec();

    // Average each block
    for by in (0..height).step_by(bs) {
        for bx in (0..width).step_by(bs) {
            let mut sum = Color::ZERO;
            let mut count = 0.0;
            let ey = (by + bs).min(height);
            let ex = (bx + bs).min(width);
            for y in by..ey {
                for x in bx..ex {
                    sum += rows[y][x];
                    count += 1.0;
                }
            }
            let avg = sum / count;
            for y in by..ey {
                for x in bx..ex {
                    result[y][x] = avg;
                }
            }
        }
    }
    result
}

/// Build an orthonormal basis from a given direction (Frisvad's method).
fn build_onb(n: Vec3) -> (Vec3, Vec3) {
    if n.z < -0.9999999 {
        return (Vec3::new(0.0, -1.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
    }
    let a = 1.0 / (1.0 + n.z);
    let b = -n.x * n.y * a;
    (
        Vec3::new(1.0 - n.x * n.x * a, b, -n.x),
        Vec3::new(b, 1.0 - n.y * n.y * a, -n.y),
    )
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
            let dist_to_center_sq = to_center.length_squared();
            if dist_to_center_sq < radius * radius {
                return Color::ZERO;
            }
            let dist_to_center = dist_to_center_sq.sqrt();

            // Solid-angle cone sampling: sample directions within the cone
            // subtended by the sphere as seen from the hit point.
            let sin_theta_max = radius / dist_to_center;
            let cos_theta_max = (1.0 - sin_theta_max * sin_theta_max).max(0.0).sqrt();

            // Sample uniform direction within cone
            let r1 = rng.next_f64();
            let r2 = rng.next_f64();
            let cos_theta = 1.0 - r1 * (1.0 - cos_theta_max);
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            let phi = 2.0 * std::f64::consts::PI * r2;

            // Build local frame with z-axis toward light center
            let w = to_center / dist_to_center;
            let (a_vec, b_vec) = build_onb(w);

            let dir = a_vec * (sin_theta * phi.cos())
                + b_vec * (sin_theta * phi.sin())
                + w * cos_theta;
            let dir = dir.unit();

            // Intersect ray with sphere to find the actual point
            let oc = *hit_point - *center;
            let b_coeff = oc.dot(dir);
            let c_coeff = oc.length_squared() - radius * radius;
            let discriminant = b_coeff * b_coeff - c_coeff;
            let t = if discriminant > 0.0 {
                let sqrt_d = discriminant.sqrt();
                let t1 = -b_coeff - sqrt_d;
                if t1 > 0.001 { t1 } else { -b_coeff + sqrt_d }
            } else {
                dist_to_center // fallback
            };

            let p = *hit_point + dir * t;
            let n = (p - *center).unit();
            // Solid angle PDF → convert to area measure for consistency
            let solid_angle = 2.0 * std::f64::consts::PI * (1.0 - cos_theta_max);
            let a = solid_angle * t * t / (-dir).dot(n).abs().max(0.001);
            (p, n, a, *emission)
        }
        LightInfo::Rect { origin, u, v, normal: n, emission } => {
            let s = rng.next_f64();
            let t = rng.next_f64();
            let p = *origin + *u * s + *v * t;
            let a = u.cross(*v).length();
            (p, *n, a, *emission)
        }
        LightInfo::Disk { center, normal: n, radius, emission } => {
            // Uniform sampling on disk: rejection-free concentric mapping
            let r1 = rng.next_f64();
            let r2 = rng.next_f64();
            let r = r1.sqrt() * radius;
            let theta = 2.0 * std::f64::consts::PI * r2;
            // Build tangent frame from normal
            let w = *n;
            let a_vec = if w.x.abs() > 0.9 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
            let u_tan = w.cross(a_vec).unit();
            let v_tan = w.cross(u_tan);
            let p = *center + u_tan * (r * theta.cos()) + v_tan * (r * theta.sin());
            let area = std::f64::consts::PI * radius * radius;
            (p, *n, area, *emission)
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

/// Result of rendering: contains both LDR output and optional HDR data.
pub struct RenderResult {
    pub pixels: Vec<u8>,
    /// HDR float data as [R, G, B] per pixel, row-major. Only populated if requested.
    pub hdr_data: Option<Vec<f32>>,
    /// Depth pass: linear depth values per pixel (0 = camera, larger = farther).
    pub depth_pass: Option<Vec<f32>>,
    /// Normal pass: world-space normals as [R, G, B] per pixel.
    pub normal_pass: Option<Vec<u8>>,
    /// Total rays traced (primary + secondary bounces).
    pub total_rays: u64,
    /// Render time in seconds (excluding post-processing).
    pub render_time_secs: f64,
}

/// Render the scene and return LDR pixels and optionally HDR data.
pub fn render(
    config: &RenderConfig,
    camera: &Camera,
    world: &dyn Hittable,
    lights: &[LightInfo],
) -> RenderResult {
    let full_width = config.width as usize;
    let full_height = config.height as usize;

    // Crop region (defaults to full image)
    let (crop_x, crop_y, crop_w, crop_h) = match config.crop {
        Some((cx, cy, cw, ch)) => (
            (cx as usize).min(full_width),
            (cy as usize).min(full_height),
            (cw as usize).min(full_width),
            (ch as usize).min(full_height),
        ),
        None => (0, 0, full_width, full_height),
    };
    let width = crop_w;
    let height = crop_h;

    let rows_done = AtomicUsize::new(0);
    let total_rays_counter = Arc::new(AtomicUsize::new(0));
    let sqrt_spp = (config.samples_per_pixel as f64).sqrt().ceil() as u32;
    let actual_spp = sqrt_spp * sqrt_spp;
    // For adaptive sampling: minimum samples before checking variance
    let adaptive_min_samples: u32 = if config.adaptive { (actual_spp / 4).max(16) } else { 0 };
    let adaptive_threshold = config.adaptive_threshold;
    let time_limit = config.time_limit;
    let time_expired = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();

    let rows: Vec<Vec<Color>> = (0..height)
        .into_par_iter()
        .map(|j| {
            // Check time limit (check every row, not every pixel for performance)
            if time_limit > 0.0 && start_time.elapsed().as_secs_f64() > time_limit {
                time_expired.store(true, Ordering::Relaxed);
            }
            let expired = time_expired.load(Ordering::Relaxed);

            let global_j = j + crop_y;
            let mut rng = SmallRng::seed_from_u64(global_j as u64 * config.seed);
            let y = (full_height - 1 - global_j) as f64;
            let mut row_rays: u64 = 0;

            let row: Vec<Color> = (0..width)
                .map(|i| {
                    let global_i = i + crop_x;

                    // If time limit exceeded, use minimal sampling (1 spp)
                    if expired {
                        let u_coord = (global_i as f64 + 0.5) / (full_width - 1) as f64;
                        let v_coord = (y + 0.5) / (full_height - 1) as f64;
                        let ray = camera.get_ray(u_coord, v_coord, &mut rng);
                        let sample = ray_color(&ray, world, lights, &config.background, &mut rng, config.max_depth, false);
                        row_rays += 1;
                        let luminance = 0.2126 * sample.x + 0.7152 * sample.y + 0.0722 * sample.z;
                        return if luminance > 100.0 { sample * (100.0 / luminance) } else { sample };
                    }

                    if config.adaptive {
                        // Adaptive sampling: use Welford's online algorithm to track
                        // per-pixel luminance variance. Stop early when converged.
                        let mut mean = Color::ZERO;
                        let mut m2_lum = 0.0_f64; // running sum of squared luminance deviations
                        let mut mean_lum = 0.0_f64;
                        let mut n: u32 = 0;

                        for sy in 0..sqrt_spp {
                            for sx in 0..sqrt_spp {
                                let u_coord = (global_i as f64 + (sx as f64 + rng.random::<f64>()) / sqrt_spp as f64) / (full_width - 1) as f64;
                                let v_coord = (y + (sy as f64 + rng.random::<f64>()) / sqrt_spp as f64) / (full_height - 1) as f64;
                                let ray = camera.get_ray(u_coord, v_coord, &mut rng);
                                let sample = ray_color(&ray, world, lights, &config.background, &mut rng, config.max_depth, false);

                                // Clamp per-sample luminance to prevent firefly artifacts
                                let luminance = 0.2126 * sample.x + 0.7152 * sample.y + 0.0722 * sample.z;
                                let clamped = if luminance > 100.0 {
                                    sample * (100.0 / luminance)
                                } else {
                                    sample
                                };

                                n += 1;
                                let lum = 0.2126 * clamped.x + 0.7152 * clamped.y + 0.0722 * clamped.z;
                                // Welford's update for luminance variance
                                let delta_lum = lum - mean_lum;
                                mean_lum += delta_lum / n as f64;
                                let delta_lum2 = lum - mean_lum;
                                m2_lum += delta_lum * delta_lum2;

                                mean = mean + (clamped - mean) / n as f64;

                                // Check convergence after minimum samples
                                if n >= adaptive_min_samples && n > 1 {
                                    let variance = m2_lum / (n - 1) as f64;
                                    // Compare standard error to threshold
                                    let std_error = (variance / n as f64).sqrt();
                                    if std_error < adaptive_threshold {
                                        break;
                                    }
                                }
                            }
                            // Check if inner loop broke early (n < expected)
                            let expected = (sy + 1) * sqrt_spp;
                            if n < expected {
                                break;
                            }
                        }

                        row_rays += n as u64;
                        mean
                    } else {
                        // Non-adaptive: stratified sampling with pixel filter
                        let mut color = Color::ZERO;
                        let mut total_weight = 0.0;
                        for sy in 0..sqrt_spp {
                            for sx in 0..sqrt_spp {
                                let jx = (sx as f64 + rng.random::<f64>()) / sqrt_spp as f64;
                                let jy = (sy as f64 + rng.random::<f64>()) / sqrt_spp as f64;
                                let u_coord = (global_i as f64 + jx) / (full_width - 1) as f64;
                                let v_coord = (y + jy) / (full_height - 1) as f64;
                                let ray = camera.get_ray(u_coord, v_coord, &mut rng);
                                let sample = ray_color(&ray, world, lights, &config.background, &mut rng, config.max_depth, false);
                                // Clamp per-sample luminance to prevent firefly artifacts
                                let luminance = 0.2126 * sample.x + 0.7152 * sample.y + 0.0722 * sample.z;
                                let clamped = if luminance > 100.0 {
                                    sample * (100.0 / luminance)
                                } else {
                                    sample
                                };
                                // Apply pixel filter weight based on distance from pixel center
                                let w = config.pixel_filter.weight(jx - 0.5, jy - 0.5);
                                color += clamped * w;
                                total_weight += w;
                            }
                        }
                        row_rays += actual_spp as u64;
                        if total_weight > 0.0 { color / total_weight } else { color }
                    }
                })
                .collect();

            total_rays_counter.fetch_add(row_rays as usize, Ordering::Relaxed);
            let done = rows_done.fetch_add(1, Ordering::Relaxed) + 1;
            if !config.quiet {
                #[allow(clippy::manual_is_multiple_of)]
                if done % 20 == 0 || done == height {
                    let pct = done * 100 / height;
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let rays_so_far = total_rays_counter.load(Ordering::Relaxed) as f64;
                    let mrays = rays_so_far / elapsed / 1_000_000.0;
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
        let total_rays = total_rays_counter.load(Ordering::Relaxed) as u64;
        if config.adaptive {
            let max_rays = width as u64 * height as u64 * actual_spp as u64;
            let savings = 100.0 * (1.0 - total_rays as f64 / max_rays as f64);
            let avg_spp = total_rays as f64 / (width as u64 * height as u64) as f64;
            eprintln!("Adaptive: {total_rays} rays ({avg_spp:.1} avg spp, max {actual_spp}, {savings:.1}% saved)");
        } else {
            eprintln!("Primary rays: {total_rays} ({actual_spp} spp, {sqrt_spp}x{sqrt_spp} stratified)");
        }
    }

    let render_time_secs = start_time.elapsed().as_secs_f64();
    let final_total_rays = total_rays_counter.load(Ordering::Relaxed) as u64;

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

    // Optional firefly removal pass (before bloom to prevent glow on outliers)
    let rows = if config.firefly_filter > 0.0 {
        if !config.quiet {
            eprint!("Removing fireflies...");
        }
        let cleaned = remove_fireflies(&rows, config.firefly_filter);
        if !config.quiet {
            eprintln!(" done");
        }
        cleaned
    } else {
        rows
    };

    // Optional bloom pass (operates on HDR data before tone mapping)
    let rows = if config.bloom > 0.0 {
        if !config.quiet {
            eprint!("Applying bloom...");
        }
        let bloomed = apply_bloom(&rows, config.bloom);
        if !config.quiet {
            eprintln!(" done");
        }
        bloomed
    } else {
        rows
    };

    // Optional sharpen pass (operates on HDR data before tone mapping)
    let rows = if config.sharpen > 0.0 {
        if !config.quiet {
            eprint!("Sharpening...");
        }
        let sharpened = apply_sharpen(&rows, config.sharpen);
        if !config.quiet {
            eprintln!(" done");
        }
        sharpened
    } else {
        rows
    };

    // Optional Gaussian blur pass
    let rows = if config.blur > 0.0 {
        if !config.quiet {
            eprint!("Blurring...");
        }
        let blurred = apply_blur(&rows, config.blur);
        if !config.quiet {
            eprintln!(" done");
        }
        blurred
    } else {
        rows
    };

    // Optional tilt-shift pass
    let rows = if config.tilt_shift > 0.0 {
        if !config.quiet {
            eprint!("Applying tilt-shift...");
        }
        let shifted = apply_tilt_shift(&rows, config.tilt_shift);
        if !config.quiet {
            eprintln!(" done");
        }
        shifted
    } else {
        rows
    };

    // Optional pixelation pass
    let rows = if config.pixelate >= 2 {
        if !config.quiet {
            eprint!("Pixelating...");
        }
        let pixelated = apply_pixelate(&rows, config.pixelate);
        if !config.quiet {
            eprintln!(" done");
        }
        pixelated
    } else {
        rows
    };

    // Optional edge detection / outline pass
    let rows = if config.edge_detect > 0.0 {
        if !config.quiet {
            eprint!("Detecting edges...");
        }
        let edged = apply_edge_detect(&rows, config.edge_detect);
        if !config.quiet {
            eprintln!(" done");
        }
        edged
    } else {
        rows
    };

    // Optional lens distortion pass
    let rows = if config.lens_distortion.abs() > 1e-6 {
        if !config.quiet {
            eprint!("Applying lens distortion...");
        }
        let distorted = apply_lens_distortion(&rows, config.lens_distortion);
        if !config.quiet {
            eprintln!(" done");
        }
        distorted
    } else {
        rows
    };

    // Optional chromatic aberration pass (operates on HDR data before tone mapping)
    let rows = if config.chromatic_aberration > 0.0 {
        if !config.quiet {
            eprint!("Applying chromatic aberration...");
        }
        let ca = apply_chromatic_aberration(&rows, config.chromatic_aberration);
        if !config.quiet {
            eprintln!(" done");
        }
        ca
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

    // Optionally collect HDR float data before tone mapping
    let hdr_data = if config.save_hdr {
        let mut hdr = Vec::with_capacity(width * height * 3);
        for row in &rows {
            for color in row {
                // Apply exposure to HDR data too
                hdr.push((color.x * exposure) as f32);
                hdr.push((color.y * exposure) as f32);
                hdr.push((color.z * exposure) as f32);
            }
        }
        Some(hdr)
    } else {
        None
    };

    // Convert to RGBA bytes with exposure, tone mapping + gamma correction.
    let tone_fn: fn(f64) -> f64 = match config.tone_map {
        ToneMap::Aces => aces_tonemap,
        ToneMap::Reinhard => reinhard_tonemap,
        ToneMap::Filmic => filmic_tonemap,
        ToneMap::None => |x: f64| x.clamp(0.0, 1.0),
    };
    let mut pixels = Vec::with_capacity(width * height * 4);
    for (j, row) in rows.iter().enumerate() {
        for (i, color) in row.iter().enumerate() {
            // Vignette: darken edges based on distance from center
            let vignette_factor = if config.vignette > 0.0 {
                let cx = (i as f64 + 0.5) / width as f64 - 0.5;
                let cy = (j as f64 + 0.5) / height as f64 - 0.5;
                let dist_sq = cx * cx + cy * cy;
                // Smooth falloff: 1 at center, decreasing toward edges
                (1.0 - dist_sq * config.vignette * 4.0).max(0.0)
            } else {
                1.0
            };
            let (mut cr, mut cg, mut cb) = (color.x * exposure * vignette_factor,
                                              color.y * exposure * vignette_factor,
                                              color.z * exposure * vignette_factor);

            // White balance: shift color temperature in HDR space
            if config.white_balance.abs() > 1e-6 {
                let wb = config.white_balance;
                // Warm: boost red, reduce blue. Cool: boost blue, reduce red.
                cr *= 1.0 + wb * 0.1;
                cb *= 1.0 - wb * 0.1;
                // Slight green adjustment for natural look
                cg *= 1.0 + wb * 0.02;
            }

            // Hue rotation in HDR space (Rodrigues' rotation around luminance axis)
            if config.hue_shift.abs() > 1e-6 {
                let angle = config.hue_shift.to_radians();
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                // Rotation matrix for hue shift (around the (1,1,1)/sqrt(3) axis)
                let k = 1.0 / 3.0;
                let sqrt3_inv = 1.0_f64 / 3.0_f64.sqrt();
                let r = cr * (cos_a + k * (1.0 - cos_a))
                    + cg * (k * (1.0 - cos_a) - sqrt3_inv * sin_a)
                    + cb * (k * (1.0 - cos_a) + sqrt3_inv * sin_a);
                let g = cr * (k * (1.0 - cos_a) + sqrt3_inv * sin_a)
                    + cg * (cos_a + k * (1.0 - cos_a))
                    + cb * (k * (1.0 - cos_a) - sqrt3_inv * sin_a);
                let b = cr * (k * (1.0 - cos_a) - sqrt3_inv * sin_a)
                    + cg * (k * (1.0 - cos_a) + sqrt3_inv * sin_a)
                    + cb * (cos_a + k * (1.0 - cos_a));
                cr = r.max(0.0);
                cg = g.max(0.0);
                cb = b.max(0.0);
            }

            // Saturation adjustment in HDR space (before tone mapping)
            if (config.saturation - 1.0).abs() > 1e-6 {
                let lum = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;
                cr = lum + (cr - lum) * config.saturation;
                cg = lum + (cg - lum) * config.saturation;
                cb = lum + (cb - lum) * config.saturation;
            }

            // Apply tone mapping
            let rf = tone_fn(cr);
            let gf = tone_fn(cg);
            let bf = tone_fn(cb);

            // Optional ordered dithering before 8-bit quantization
            let dither_offset = if config.dither {
                // 4x4 Bayer matrix normalized to [-0.5, 0.5] / 255
                const BAYER4: [[f64; 4]; 4] = [
                    [ 0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0],
                    [12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0],
                    [ 3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0],
                    [15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0],
                ];
                (BAYER4[j % 4][i % 4] - 0.5) / 255.0
            } else {
                0.0
            };

            let mut r = gamma_correct(rf, config.gamma, dither_offset);
            let mut g = gamma_correct(gf, config.gamma, dither_offset);
            let mut b = gamma_correct(bf, config.gamma, dither_offset);

            // Contrast: pivot around middle gray (128) in sRGB space
            if (config.contrast - 1.0).abs() > 1e-6 {
                r = ((((r as f64 / 255.0) - 0.5) * config.contrast + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
                g = ((((g as f64 / 255.0) - 0.5) * config.contrast + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
                b = ((((b as f64 / 255.0) - 0.5) * config.contrast + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
            }

            // Sepia tone: warm brownish tint
            if config.sepia > 0.0 {
                let rf = r as f64 / 255.0;
                let gf = g as f64 / 255.0;
                let bf = b as f64 / 255.0;
                let sr = (rf * 0.393 + gf * 0.769 + bf * 0.189).min(1.0);
                let sg = (rf * 0.349 + gf * 0.686 + bf * 0.168).min(1.0);
                let sb = (rf * 0.272 + gf * 0.534 + bf * 0.131).min(1.0);
                let s = config.sepia;
                r = ((rf * (1.0 - s) + sr * s) * 255.0) as u8;
                g = ((gf * (1.0 - s) + sg * s) * 255.0) as u8;
                b = ((bf * (1.0 - s) + sb * s) * 255.0) as u8;
            }

            // Posterize: reduce color levels per channel
            if config.posterize >= 2 {
                let levels = config.posterize as f64;
                let posterize_ch = |c: u8| -> u8 {
                    let f = c as f64 / 255.0;
                    let q = (f * (levels - 1.0)).round() / (levels - 1.0);
                    (q * 255.0).clamp(0.0, 255.0) as u8
                };
                r = posterize_ch(r);
                g = posterize_ch(g);
                b = posterize_ch(b);
            }

            // Color inversion
            if config.invert {
                r = 255 - r;
                g = 255 - g;
                b = 255 - b;
            }

            // Black-and-white threshold
            if config.threshold >= 0.0 {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                let bw = if lum >= config.threshold { 255u8 } else { 0u8 };
                r = bw;
                g = bw;
                b = bw;
            }

            // CRT scanlines: darken alternating rows
            if config.scanlines > 0.0 {
                let factor = if j % 2 == 0 { 1.0 } else { 1.0 - config.scanlines.min(1.0) };
                r = (r as f64 * factor) as u8;
                g = (g as f64 * factor) as u8;
                b = (b as f64 * factor) as u8;
            }

            // Film grain: add deterministic luminance noise
            if config.grain > 0.0 {
                let hash = (i as u64).wrapping_mul(73856093)
                    ^ (j as u64).wrapping_mul(19349663)
                    ^ config.seed;
                let noise = ((hash & 0xFFFF) as f64 / 65535.0 - 0.5) * config.grain * 255.0;
                r = (r as f64 + noise).clamp(0.0, 255.0) as u8;
                g = (g as f64 + noise).clamp(0.0, 255.0) as u8;
                b = (b as f64 + noise).clamp(0.0, 255.0) as u8;
            }

            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    // Generate AOV passes (depth, normals) with single-ray-per-pixel, parallelized
    let (depth_pass, normal_pass) = if config.save_depth || config.save_normals {
        if !config.quiet {
            eprint!("Generating AOV passes...");
        }

        // Per-row parallel AOV generation
        let aov_rows: Vec<(Vec<f32>, Vec<u8>)> = (0..height)
            .into_par_iter()
            .map(|j| {
                let global_j = j + crop_y;
                let y = (full_height - 1 - global_j) as f64;
                let mut row_depth = vec![0.0f32; width];
                let mut row_normals = vec![0u8; width * 3];

                for i in 0..width {
                    let global_i = i + crop_x;
                    let u_coord = (global_i as f64 + 0.5) / (full_width - 1) as f64;
                    let v_coord = (y + 0.5) / (full_height - 1) as f64;
                    let mut rng = SmallRng::seed_from_u64(
                        (global_j as u64 * full_width as u64 + global_i as u64).wrapping_mul(config.seed),
                    );
                    let ray = camera.get_ray(u_coord, v_coord, &mut rng);

                    if let Some(hit) = world.hit(&ray, 0.001, f64::INFINITY) {
                        row_depth[i] = hit.t as f32;
                        row_normals[i * 3] = ((hit.normal.x * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
                        row_normals[i * 3 + 1] = ((hit.normal.y * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
                        row_normals[i * 3 + 2] = ((hit.normal.z * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
                    }
                }
                (row_depth, row_normals)
            })
            .collect();

        let depth_buf = if config.save_depth {
            Some(aov_rows.iter().flat_map(|(d, _)| d.iter().copied()).collect())
        } else {
            None
        };
        let normal_buf = if config.save_normals {
            Some(aov_rows.iter().flat_map(|(_, n)| n.iter().copied()).collect())
        } else {
            None
        };

        if !config.quiet {
            eprintln!(" done");
        }
        (depth_buf, normal_buf)
    } else {
        (None, None)
    };

    RenderResult { pixels, hdr_data, depth_pass, normal_pass, total_rays: final_total_rays, render_time_secs }
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
#[cfg(test)]
fn linear_to_srgb(x: f64) -> u8 {
    gamma_correct(x, 0.0, 0.0)
}

fn gamma_correct(x: f64, gamma: f64, dither: f64) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let s = if gamma > 0.0 {
        x.powf(1.0 / gamma)
    } else if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    ((s + dither).clamp(0.0, 0.999) * 256.0) as u8
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

    #[test]
    fn pixel_filter_box_uniform() {
        let f = PixelFilter::Box;
        assert_eq!(f.weight(0.0, 0.0), 1.0);
        assert_eq!(f.weight(0.4, 0.4), 1.0);
        assert_eq!(f.weight(-0.5, -0.5), 1.0);
    }

    #[test]
    fn pixel_filter_gaussian_peaks_center() {
        let f = PixelFilter::Gaussian;
        let center = f.weight(0.0, 0.0);
        let edge = f.weight(0.4, 0.4);
        assert!(center > edge, "Gaussian should peak at center");
    }

    #[test]
    fn pixel_filter_mitchell_positive_center() {
        let f = PixelFilter::Mitchell;
        let center = f.weight(0.0, 0.0);
        assert!(center > 0.5, "Mitchell center weight should be significant, got {center}");
    }

    #[test]
    fn firefly_removal_replaces_outlier() {
        // Create a 5x5 image with one bright outlier pixel
        let dim = Color::new(0.1, 0.1, 0.1);
        let mut rows = vec![vec![dim; 5]; 5];
        rows[2][2] = Color::new(100.0, 100.0, 100.0); // Firefly

        let result = remove_fireflies(&rows, 5.0);
        let lum = 0.2126 * result[2][2].x + 0.7152 * result[2][2].y + 0.0722 * result[2][2].z;
        assert!(lum < 1.0, "Firefly should be removed, got luminance {lum}");
    }

    #[test]
    fn chromatic_aberration_preserves_center() {
        // Center pixel should be minimally affected
        let white = Color::new(1.0, 1.0, 1.0);
        let rows = vec![vec![white; 5]; 5];
        let result = apply_chromatic_aberration(&rows, 0.01);
        // Center pixel (2,2) should still be close to white
        let c = result[2][2];
        assert!((c.x - 1.0).abs() < 0.1, "Center R should be ~1.0, got {}", c.x);
        assert!((c.y - 1.0).abs() < 0.1, "Center G should be ~1.0, got {}", c.y);
    }
}
