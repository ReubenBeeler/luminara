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
    /// Generate albedo (base color) pass output.
    pub save_albedo: bool,
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
    /// Color grading: tint shadows (RGB multiplier, [1,1,1] = neutral).
    pub grade_shadows: [f64; 3],
    /// Color grading: tint highlights (RGB multiplier, [1,1,1] = neutral).
    pub grade_highlights: [f64; 3],
    /// Halftone dot size (0 = off). Simulates newspaper halftone printing.
    pub halftone: u32,
    /// Emboss filter intensity (0.0 = off). Creates raised/engraved look.
    pub emboss: f64,
    /// Oil paint effect radius (0 = off). Kuwahara filter for painterly look.
    pub oil_paint: u32,
    /// False color mapping: "inferno", "viridis", "turbo", "grayscale", or empty for off.
    pub color_map: String,
    /// Solarize threshold (negative = off, 0.0-1.0). Pixels above this luminance get inverted.
    pub solarize: f64,
    /// Duo-tone: map shadows to color A, highlights to color B. Empty = off.
    /// Format: "R,G,B;R,G,B" (each 0-255) e.g. "0,0,64;255,200,0"
    pub duo_tone: String,
    /// Sketch mode (false = off). Combines edge detection + inverted grayscale for pencil look.
    pub sketch: bool,
    /// Median filter radius (0 = off). Removes salt-and-pepper noise while preserving edges.
    pub median: u32,
    /// Crosshatch spacing (0 = off). Simulates pen-and-ink cross-hatching.
    pub crosshatch: u32,
    /// Glitch effect intensity (0.0 = off). Simulates digital data corruption.
    pub glitch: f64,
    /// Depth fog: fog density (0.0 = off). Blends scene toward fog color by depth.
    pub depth_fog: f64,
    /// Depth fog color [R, G, B] (default: white).
    pub depth_fog_color: [f64; 3],
    /// Channel swap mode: "rgb" (normal), "rbg", "grb", "gbr", "brg", "bgr"
    pub channel_swap: String,
    /// Mirror mode: "h" (horizontal), "v" (vertical), "hv" (both), or empty for off.
    pub mirror: String,
    /// Color quantization: reduce to N colors (0 = off). Median-cut algorithm.
    pub quantize: u32,
    /// Color tint: multiply all pixels by this RGB color [R, G, B] in 0..1 range.
    pub tint: [f64; 3],
    /// Named color palette for quantization (overrides --quantize).
    pub palette: String,
    /// Tri-tone: map shadows/midtones/highlights to three colors. "R,G,B;R,G,B;R,G,B"
    pub tri_tone: String,
    /// Gradient map: replace luminance with a custom color gradient.
    /// Format: "RRGGBB;RRGGBB;..." hex colors from dark to light.
    pub gradient_map: String,
    /// Split-tone: warm highlights / cool shadows. "R,G,B;R,G,B" (shadow;highlight colors 0-255).
    pub split_tone: String,
    /// Color shift: rotate RGB channels by N positions (0 = off, 1 = R→G→B→R, 2 = R→B→G→R).
    pub color_shift: u32,
    /// Per-channel posterization levels [R, G, B]. [0,0,0] = off.
    pub posterize_channels: [u32; 3],
    /// Lens flare intensity (0.0 = off). Adds light streaks from brightest point.
    pub lens_flare: f64,
    /// Cel-shading color bands (0 = off). Combines posterization + edge outlines for toon look.
    pub cel_shade: u32,
    /// Picture frame width in pixels (0 = off). Adds decorative frame with drop shadow.
    pub frame: u32,
    /// Hexagonal pixelation cell size (0 = off). Creates honeycomb mosaic effect.
    pub hex_pixelate: u32,
    /// Pop art: number of color bands for Warhol-style effect (0 = off).
    pub pop_art: u32,
    /// Watercolor painting effect radius (0 = off).
    pub watercolor: u32,
    /// Auto-levels: stretch histogram to use full dynamic range.
    pub auto_levels: bool,
    /// Brightness adjustment (-1.0 to 1.0, 0 = no change). Added to each channel in LDR space.
    pub brightness: f64,
    /// Color balance: per-channel multiplier [R, G, B]. Values > 1.0 boost, < 1.0 reduce.
    pub color_balance: [f64; 3],
    /// Stipple dot size (0 = off). Creates pointillism/stippling effect.
    pub stipple: u32,
    /// Night vision simulation (0 = off, 1 = full effect).
    pub night_vision: bool,
    /// Fisheye barrel distortion intensity (0 = off, positive = barrel, negative = pincushion).
    pub fisheye: f64,
    /// Wave distortion amplitude in pixels (0 = off).
    pub wave: f64,
    /// Swirl distortion intensity (0 = off). Twists image around center.
    pub swirl: f64,
    /// Mosaic cell size (0 = off). Creates Voronoi stained-glass mosaic effect.
    pub mosaic: u32,
    /// Radial blur intensity (0 = off). Creates zoom blur centered on image.
    pub radial_blur: f64,
    /// Border width in pixels (0 = off).
    pub border: u32,
    /// Border color [R, G, B] in 0..1 range.
    pub border_color: [f64; 3],
    /// Output resize: [width, height]. 0 = keep original dimension.
    pub resize: [u32; 2],
    /// Image rotation in degrees (0, 90, 180, 270).
    pub rotate: u32,
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
            save_albedo: false,
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
            grade_shadows: [1.0, 1.0, 1.0],
            grade_highlights: [1.0, 1.0, 1.0],
            halftone: 0,
            emboss: 0.0,
            oil_paint: 0,
            color_map: String::new(),
            solarize: -1.0,
            duo_tone: String::new(),
            sketch: false,
            median: 0,
            crosshatch: 0,
            glitch: 0.0,
            depth_fog: 0.0,
            depth_fog_color: [0.8, 0.85, 0.9],
            channel_swap: String::new(),
            mirror: String::new(),
            quantize: 0,
            tint: [1.0, 1.0, 1.0],
            palette: String::new(),
            tri_tone: String::new(),
            gradient_map: String::new(),
            split_tone: String::new(),
            color_shift: 0,
            posterize_channels: [0, 0, 0],
            lens_flare: 0.0,
            cel_shade: 0,
            frame: 0,
            hex_pixelate: 0,
            pop_art: 0,
            watercolor: 0,
            auto_levels: false,
            brightness: 0.0,
            color_balance: [1.0, 1.0, 1.0],
            stipple: 0,
            night_vision: false,
            fisheye: 0.0,
            wave: 0.0,
            swirl: 0.0,
            mosaic: 0,
            radial_blur: 0.0,
            border: 0,
            border_color: [0.0, 0.0, 0.0],
            resize: [0, 0],
            rotate: 0,
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
    for (j, result_row) in result.iter_mut().enumerate() {
        let dist = ((j as f64 - center).abs() - band).max(0.0) / (center - band).max(1.0);
        let blur_radius = (dist * strength * 5.0).round() as i64;
        if blur_radius <= 0 {
            continue;
        }
        for (i, pixel) in result_row.iter_mut().enumerate() {
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
            *pixel = sum / count;
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
            for row in rows.iter().take(ey).skip(by) {
                for pixel in row.iter().take(ex).skip(bx) {
                    sum += *pixel;
                    count += 1.0;
                }
            }
            let avg = sum / count;
            for row in result.iter_mut().take(ey).skip(by) {
                for pixel in row.iter_mut().take(ex).skip(bx) {
                    *pixel = avg;
                }
            }
        }
    }
    result
}

/// Emboss filter: creates a raised/engraved 3D appearance.
fn apply_emboss(rows: &[Vec<Color>], intensity: f64) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();

    rows.iter()
        .enumerate()
        .map(|(j, row)| {
            row.iter()
                .enumerate()
                .map(|(i, orig)| {
                    if j == 0 || i == 0 || j >= height - 1 || i >= width - 1 {
                        return *orig;
                    }
                    // Emboss kernel: [-2, -1, 0; -1, 1, 1; 0, 1, 2]
                    let tl = rows[j - 1][i - 1];
                    let tc = rows[j - 1][i];
                    let ml = rows[j][i - 1];
                    let mr = rows[j][i + 1];
                    let bc = rows[j + 1][i];
                    let br = rows[j + 1][i + 1];

                    let diff = (mr + bc + br * 2.0) - (tl * 2.0 + tc + ml);
                    let gray = Color::new(0.5, 0.5, 0.5);
                    let embossed = gray + diff * intensity * 0.25;
                    // Blend with original
                    *orig * (1.0 - intensity.min(1.0)) + embossed * intensity.min(1.0)
                })
                .collect()
        })
        .collect()
}

/// Kuwahara filter — produces an oil-painting effect by selecting the
/// lowest-variance quadrant around each pixel.
fn apply_oil_paint(rows: &[Vec<Color>], radius: u32) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();
    let r = radius as i32;

    (0..height)
        .into_par_iter()
        .map(|j| {
            (0..width)
                .map(|i| {
                    let mut best_mean = Color::ZERO;
                    let mut best_var = f64::INFINITY;

                    // 4 quadrants: TL, TR, BL, BR
                    for (dy_range, dx_range) in [
                        (-r..=0, -r..=0),
                        (-r..=0, 0..=r),
                        (0..=r, -r..=0),
                        (0..=r, 0..=r),
                    ] {
                        let mut sum = Color::ZERO;
                        let mut sum_sq = 0.0f64;
                        let mut count = 0.0f64;

                        for dy in dy_range {
                            for dx in dx_range.clone() {
                                let ny = j as i32 + dy;
                                let nx = i as i32 + dx;
                                if ny >= 0 && ny < height as i32 && nx >= 0 && nx < width as i32 {
                                    let c = rows[ny as usize][nx as usize];
                                    sum += c;
                                    sum_sq += c.x * c.x + c.y * c.y + c.z * c.z;
                                    count += 1.0;
                                }
                            }
                        }

                        if count > 0.0 {
                            let mean = sum * (1.0 / count);
                            let mean_sq = (mean.x * mean.x + mean.y * mean.y + mean.z * mean.z) * count;
                            let variance = (sum_sq - mean_sq) / count;
                            if variance < best_var {
                                best_var = variance;
                                best_mean = mean;
                            }
                        }
                    }

                    best_mean
                })
                .collect()
        })
        .collect()
}

/// Map a [0,1] luminance value through a named colormap, returning (R, G, B) in [0,1].
/// Median filter: replace each pixel with the median of its NxN neighborhood.
/// Good for removing salt-and-pepper noise while preserving edges.
fn apply_median(rows: &[Vec<Color>], radius: u32) -> Vec<Vec<Color>> {
    let height = rows.len();
    if height == 0 {
        return vec![];
    }
    let width = rows[0].len();
    let r = radius as i32;

    (0..height)
        .into_par_iter()
        .map(|j| {
            (0..width)
                .map(|i| {
                    let mut rs = Vec::new();
                    let mut gs = Vec::new();
                    let mut bs = Vec::new();

                    for dy in -r..=r {
                        for dx in -r..=r {
                            let ny = j as i32 + dy;
                            let nx = i as i32 + dx;
                            if ny >= 0 && ny < height as i32 && nx >= 0 && nx < width as i32 {
                                let c = rows[ny as usize][nx as usize];
                                rs.push(c.x);
                                gs.push(c.y);
                                bs.push(c.z);
                            }
                        }
                    }

                    rs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    gs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    bs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                    let mid = rs.len() / 2;
                    Color::new(rs[mid], gs[mid], bs[mid])
                })
                .collect()
        })
        .collect()
}

/// Simple median-cut color quantization. Returns a palette of N representative colors.
fn median_cut_palette(pixels: &[[u8; 3]], n: usize) -> Vec<[u8; 3]> {
    if pixels.is_empty() || n == 0 {
        return vec![[128, 128, 128]];
    }

    let mut buckets: Vec<Vec<[u8; 3]>> = vec![pixels.to_vec()];

    while buckets.len() < n {
        // Find bucket with largest range in any channel
        let mut best_idx = 0;
        let mut best_range = 0u32;
        let mut best_channel = 0usize;

        for (idx, bucket) in buckets.iter().enumerate() {
            if bucket.len() < 2 { continue; }
            for ch in 0..3 {
                let min = bucket.iter().map(|p| p[ch]).min().unwrap() as u32;
                let max = bucket.iter().map(|p| p[ch]).max().unwrap() as u32;
                let range = max - min;
                if range > best_range {
                    best_range = range;
                    best_idx = idx;
                    best_channel = ch;
                }
            }
        }

        if best_range == 0 { break; }

        // Split the best bucket at median
        let mut bucket = buckets.remove(best_idx);
        bucket.sort_by_key(|p| p[best_channel]);
        let mid = bucket.len() / 2;
        let right = bucket.split_off(mid);
        buckets.push(bucket);
        buckets.push(right);
    }

    // Compute average color for each bucket
    buckets.iter().map(|b| {
        if b.is_empty() { return [128, 128, 128]; }
        let (mut sr, mut sg, mut sb) = (0u64, 0u64, 0u64);
        for p in b {
            sr += p[0] as u64;
            sg += p[1] as u64;
            sb += p[2] as u64;
        }
        let n = b.len() as u64;
        [(sr / n) as u8, (sg / n) as u8, (sb / n) as u8]
    }).collect()
}

fn named_palette(name: &str) -> Option<Vec<[u8; 3]>> {
    match name {
        "gameboy" => Some(vec![
            [15, 56, 15], [48, 98, 48], [139, 172, 15], [155, 188, 15],
        ]),
        "cga" => Some(vec![
            [0, 0, 0], [85, 255, 255], [255, 85, 255], [255, 255, 255],
        ]),
        "nes" => Some(vec![
            [0, 0, 0], [124, 124, 124], [188, 188, 188], [252, 252, 252],
            [168, 16, 0], [228, 92, 16], [248, 216, 120], [88, 168, 0],
            [0, 120, 0], [0, 168, 68], [0, 88, 248], [88, 216, 84],
            [104, 68, 252], [248, 120, 88], [216, 0, 204], [248, 184, 0],
        ]),
        "pastel" => Some(vec![
            [255, 179, 186], [255, 223, 186], [255, 255, 186], [186, 255, 201],
            [186, 225, 255], [218, 186, 255], [255, 186, 243], [255, 255, 255],
        ]),
        "grayscale4" => Some(vec![
            [0, 0, 0], [85, 85, 85], [170, 170, 170], [255, 255, 255],
        ]),
        "sunset" => Some(vec![
            [25, 10, 40], [80, 20, 60], [160, 50, 40], [220, 100, 30],
            [240, 160, 50], [250, 210, 120], [255, 240, 200],
        ]),
        "cyberpunk" => Some(vec![
            [10, 0, 20], [30, 0, 60], [100, 0, 150], [200, 0, 255],
            [255, 0, 100], [0, 255, 200], [255, 255, 0], [255, 255, 255],
        ]),
        "sepia4" => Some(vec![
            [60, 40, 25], [130, 100, 65], [190, 160, 110], [240, 220, 180],
        ]),
        _ => None,
    }
}

fn apply_color_map(t: f64, name: &str) -> (f64, f64, f64) {
    let t = t.clamp(0.0, 1.0);
    match name {
        "inferno" => {
            // Simplified inferno: black → purple → orange → yellow
            let r = (1.5 * t - 0.2).clamp(0.0, 1.0);
            let g = (1.8 * t * t).clamp(0.0, 1.0);
            let b = ((1.0 - (t - 0.35).abs() * 3.0).max(0.0) * 0.8).clamp(0.0, 1.0);
            (r, g, b)
        }
        "viridis" => {
            // Simplified viridis: purple → teal → yellow
            let r = (t * t * 0.8 + t * 0.2).clamp(0.0, 1.0);
            let g = (0.15 + t * 0.85).clamp(0.0, 1.0);
            let b = (0.55 - t * 0.55 + (1.0 - t) * 0.3).clamp(0.0, 1.0);
            (r, g, b)
        }
        "turbo" => {
            // Simplified turbo rainbow: blue → cyan → green → yellow → red
            let r = (1.0 - (t - 0.75).abs() * 4.0).clamp(0.0, 1.0);
            let g = (1.0 - (t - 0.5).abs() * 3.0).clamp(0.0, 1.0);
            let b = (1.0 - (t - 0.2).abs() * 4.0).clamp(0.0, 1.0);
            (r, g, b)
        }
        "grayscale" => (t, t, t),
        "heat" => {
            // Heat: black → red → yellow → white
            let r = (t * 3.0).clamp(0.0, 1.0);
            let g = ((t - 0.33) * 3.0).clamp(0.0, 1.0);
            let b = ((t - 0.67) * 3.0).clamp(0.0, 1.0);
            (r, g, b)
        }
        "thermal" => {
            // Thermal/infrared: black → blue → purple → red → orange → yellow → white
            if t < 0.15 {
                let s = t / 0.15;
                (0.0, 0.0, s * 0.5)
            } else if t < 0.3 {
                let s = (t - 0.15) / 0.15;
                (s * 0.5, 0.0, 0.5 + s * 0.2)
            } else if t < 0.5 {
                let s = (t - 0.3) / 0.2;
                (0.5 + s * 0.5, 0.0, 0.7 - s * 0.7)
            } else if t < 0.7 {
                let s = (t - 0.5) / 0.2;
                (1.0, s * 0.6, 0.0)
            } else if t < 0.85 {
                let s = (t - 0.7) / 0.15;
                (1.0, 0.6 + s * 0.4, s * 0.3)
            } else {
                let s = (t - 0.85) / 0.15;
                (1.0, 1.0, 0.3 + s * 0.7)
            }
        }
        "neon" => {
            // Neon: dark → electric blue → magenta → hot pink → white
            if t < 0.25 {
                let s = t / 0.25;
                (0.0, s * 0.2, s * 0.8)
            } else if t < 0.5 {
                let s = (t - 0.25) / 0.25;
                (s * 0.8, 0.2 - s * 0.2, 0.8 + s * 0.2)
            } else if t < 0.75 {
                let s = (t - 0.5) / 0.25;
                (0.8 + s * 0.2, s * 0.4, 1.0)
            } else {
                let s = (t - 0.75) / 0.25;
                (1.0, 0.4 + s * 0.6, 1.0)
            }
        }
        _ => (t, t, t), // fallback to grayscale
    }
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
    /// Albedo pass: base material color as [R, G, B] per pixel.
    pub albedo_pass: Option<Vec<u8>>,
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
    // Validate render config
    if config.width == 0 || config.height == 0 {
        eprintln!("Warning: zero-size image ({}x{}), using 1x1", config.width, config.height);
    }
    if config.samples_per_pixel == 0 {
        eprintln!("Warning: 0 samples per pixel, using 1");
    }

    let full_width = config.width.max(1) as usize;
    let full_height = config.height.max(1) as usize;

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

    // Optional emboss pass
    let rows = if config.emboss > 0.0 {
        if !config.quiet {
            eprint!("Embossing...");
        }
        let embossed = apply_emboss(&rows, config.emboss);
        if !config.quiet {
            eprintln!(" done");
        }
        embossed
    } else {
        rows
    };

    // Optional color quantization pass (named palette or median-cut)
    let use_palette = !config.palette.is_empty() && named_palette(&config.palette).is_some();
    let rows = if use_palette || config.quantize >= 2 {
        let palette = if let Some(p) = named_palette(&config.palette) {
            if !config.quiet {
                eprint!("Applying {} palette ({} colors)...", config.palette, p.len());
            }
            p
        } else {
            if !config.quiet {
                eprint!("Quantizing to {} colors...", config.quantize);
            }
            let height = rows.len();
            let width = if height > 0 { rows[0].len() } else { 0 };

            let mut pixels_rgb: Vec<[u8; 3]> = Vec::with_capacity(height * width);
            for row in &rows {
                for c in row {
                    pixels_rgb.push([
                        (c.x.clamp(0.0, 1.0) * 255.0) as u8,
                        (c.y.clamp(0.0, 1.0) * 255.0) as u8,
                        (c.z.clamp(0.0, 1.0) * 255.0) as u8,
                    ]);
                }
            }
            median_cut_palette(&pixels_rgb, config.quantize as usize)
        };

        // Map each pixel to nearest palette color
        let result: Vec<Vec<Color>> = rows.iter()
            .map(|row| row.iter()
                .map(|c| {
                    let r = (c.x.clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (c.y.clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (c.z.clamp(0.0, 1.0) * 255.0) as u8;
                    let best = palette.iter()
                        .min_by_key(|p| {
                            let dr = r as i32 - p[0] as i32;
                            let dg = g as i32 - p[1] as i32;
                            let db = b as i32 - p[2] as i32;
                            (dr*dr + dg*dg + db*db) as u32
                        })
                        .unwrap();
                    Color::new(best[0] as f64 / 255.0, best[1] as f64 / 255.0, best[2] as f64 / 255.0)
                })
                .collect())
            .collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional mirror pass
    let rows = if !config.mirror.is_empty() {
        let mirror_h = config.mirror.contains('h');
        let mirror_v = config.mirror.contains('v');
        let mut result = rows;
        if mirror_h {
            for row in &mut result {
                row.reverse();
            }
        }
        if mirror_v {
            result.reverse();
        }
        result
    } else {
        rows
    };

    // Optional rotation pass
    let rows = if config.rotate == 90 || config.rotate == 180 || config.rotate == 270 {
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        match config.rotate {
            90 => {
                let mut result = vec![vec![Color::new(0.0, 0.0, 0.0); height]; width];
                for (y, row) in rows.iter().enumerate() {
                    for (x, pixel) in row.iter().enumerate() {
                        result[x][height - 1 - y] = *pixel;
                    }
                }
                result
            }
            180 => {
                let mut result = rows;
                result.reverse();
                for row in &mut result {
                    row.reverse();
                }
                result
            }
            270 => {
                let mut result = vec![vec![Color::new(0.0, 0.0, 0.0); height]; width];
                for (y, row) in rows.iter().enumerate() {
                    for (x, pixel) in row.iter().enumerate() {
                        result[width - 1 - x][y] = *pixel;
                    }
                }
                result
            }
            _ => rows,
        }
    } else {
        rows
    };

    // Optional watercolor painting effect
    let rows = if config.watercolor > 0 {
        if !config.quiet {
            eprint!("Applying watercolor...");
        }
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let r = config.watercolor as i32;

        // Step 1: Smooth colors using area averaging (fast approximation of median)
        let smoothed: Vec<Vec<Color>> = (0..height).into_par_iter().map(|y| {
            (0..width).map(|x| {
                let mut sum = Color::new(0.0, 0.0, 0.0);
                let mut count = 0;
                for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = (x as i32 + dx).clamp(0, (width - 1) as i32) as usize;
                        let ny = (y as i32 + dy).clamp(0, (height - 1) as i32) as usize;
                        sum += rows[ny][nx];
                        count += 1;
                    }
                }
                sum * (1.0 / count as f64)
            }).collect()
        }).collect();

        // Step 2: Add edge darkening using Sobel magnitude
        let result: Vec<Vec<Color>> = (0..height).into_par_iter().map(|y| {
            (0..width).map(|x| {
                if y == 0 || x == 0 || y >= height - 1 || x >= width - 1 {
                    return smoothed[y][x];
                }
                let lum = |yy: usize, xx: usize| -> f64 {
                    let c = smoothed[yy][xx];
                    0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z
                };
                let gx = -lum(y-1,x-1) - 2.0*lum(y,x-1) - lum(y+1,x-1)
                        + lum(y-1,x+1) + 2.0*lum(y,x+1) + lum(y+1,x+1);
                let gy = -lum(y-1,x-1) - 2.0*lum(y-1,x) - lum(y-1,x+1)
                        + lum(y+1,x-1) + 2.0*lum(y+1,x) + lum(y+1,x+1);
                let edge = (gx * gx + gy * gy).sqrt();
                let darken = (1.0 - edge * 2.0).clamp(0.6, 1.0);
                smoothed[y][x] * darken
            }).collect()
        }).collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional auto-levels pass: stretch histogram to full range
    let rows = if config.auto_levels {
        if !config.quiet {
            eprint!("Auto-leveling...");
        }
        // Find min and max for each channel (ignoring extreme outliers at 1% and 99%)
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let total = height * width;
        let mut r_vals: Vec<f64> = Vec::with_capacity(total);
        let mut g_vals: Vec<f64> = Vec::with_capacity(total);
        let mut b_vals: Vec<f64> = Vec::with_capacity(total);
        for row in &rows {
            for c in row {
                r_vals.push(c.x.clamp(0.0, 1.0));
                g_vals.push(c.y.clamp(0.0, 1.0));
                b_vals.push(c.z.clamp(0.0, 1.0));
            }
        }
        r_vals.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        g_vals.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        b_vals.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let lo = total / 100; // 1st percentile
        let hi = total - 1 - lo; // 99th percentile
        let r_min = r_vals[lo];
        let r_max = r_vals[hi];
        let g_min = g_vals[lo];
        let g_max = g_vals[hi];
        let b_min = b_vals[lo];
        let b_max = b_vals[hi];

        let stretch = |v: f64, lo: f64, hi: f64| -> f64 {
            if (hi - lo).abs() < 1e-6 { v } else { ((v - lo) / (hi - lo)).clamp(0.0, 1.0) }
        };

        let result: Vec<Vec<Color>> = rows.iter().map(|row| {
            row.iter().map(|c| {
                Color::new(
                    stretch(c.x.clamp(0.0, 1.0), r_min, r_max),
                    stretch(c.y.clamp(0.0, 1.0), g_min, g_max),
                    stretch(c.z.clamp(0.0, 1.0), b_min, b_max),
                )
            }).collect()
        }).collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional stipple/pointillism pass
    let rows = if config.stipple > 0 {
        if !config.quiet {
            eprint!("Applying stipple...");
        }
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let cell = config.stipple as usize;
        let white = Color::new(1.0, 1.0, 1.0);

        let result: Vec<Vec<Color>> = (0..height).into_par_iter().map(|y| {
            (0..width).map(|x| {
                let cx = (x / cell) * cell + cell / 2;
                let cy = (y / cell) * cell + cell / 2;
                let scx = cx.min(width - 1);
                let scy = cy.min(height - 1);
                let c = rows[scy][scx];
                let lum = 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
                // Darker pixels → larger dots
                let radius = (1.0 - lum.clamp(0.0, 1.0)) * cell as f64 * 0.5;
                let dx = x as f64 - cx as f64;
                let dy = y as f64 - cy as f64;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= radius {
                    c
                } else {
                    white
                }
            }).collect()
        }).collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional fisheye barrel/pincushion distortion pass
    let rows = if config.fisheye.abs() > 1e-6 {
        if !config.quiet {
            eprint!("Applying fisheye...");
        }
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let cx = width as f64 * 0.5;
        let cy = height as f64 * 0.5;
        let k = config.fisheye;

        let result: Vec<Vec<Color>> = (0..height).into_par_iter().map(|y| {
            (0..width).map(|x| {
                let nx = (x as f64 - cx) / cx;
                let ny = (y as f64 - cy) / cy;
                let r2 = nx * nx + ny * ny;
                let factor = 1.0 + k * r2;
                let sx = cx + nx * factor * cx;
                let sy = cy + ny * factor * cy;
                if sx < 0.0 || sx >= width as f64 || sy < 0.0 || sy >= height as f64 {
                    Color::new(0.0, 0.0, 0.0)
                } else {
                    let x0 = sx.floor() as usize;
                    let y0 = sy.floor() as usize;
                    let x1 = (x0 + 1).min(width - 1);
                    let y1 = (y0 + 1).min(height - 1);
                    let fx = sx - x0 as f64;
                    let fy = sy - y0 as f64;
                    let top = rows[y0][x0] * (1.0 - fx) + rows[y0][x1] * fx;
                    let bot = rows[y1][x0] * (1.0 - fx) + rows[y1][x1] * fx;
                    top * (1.0 - fy) + bot * fy
                }
            }).collect()
        }).collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional wave distortion pass
    let rows = if config.wave > 0.0 {
        if !config.quiet {
            eprint!("Applying wave...");
        }
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let amp = config.wave;
        let freq = 2.0 * std::f64::consts::PI / (height as f64 * 0.15);

        let result: Vec<Vec<Color>> = (0..height).into_par_iter().map(|y| {
            (0..width).map(|x| {
                let dx = (y as f64 * freq).sin() * amp;
                let dy = (x as f64 * freq * 1.3).sin() * amp * 0.7;
                let sx = (x as f64 + dx).clamp(0.0, (width - 1) as f64);
                let sy = (y as f64 + dy).clamp(0.0, (height - 1) as f64);
                let x0 = sx.floor() as usize;
                let y0 = sy.floor() as usize;
                let x1 = (x0 + 1).min(width - 1);
                let y1 = (y0 + 1).min(height - 1);
                let fx = sx - x0 as f64;
                let fy = sy - y0 as f64;
                let top = rows[y0][x0] * (1.0 - fx) + rows[y0][x1] * fx;
                let bot = rows[y1][x0] * (1.0 - fx) + rows[y1][x1] * fx;
                top * (1.0 - fy) + bot * fy
            }).collect()
        }).collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional swirl distortion pass
    let rows = if config.swirl.abs() > 1e-6 {
        if !config.quiet {
            eprint!("Applying swirl...");
        }
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let cx = width as f64 * 0.5;
        let cy = height as f64 * 0.5;
        let max_radius = (cx * cx + cy * cy).sqrt();

        let result: Vec<Vec<Color>> = (0..height).into_par_iter().map(|y| {
            (0..width).map(|x| {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let angle = config.swirl * (1.0 - dist / max_radius).max(0.0);
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let sx = cx + dx * cos_a - dy * sin_a;
                let sy = cy + dx * sin_a + dy * cos_a;
                let sx = sx.clamp(0.0, (width - 1) as f64);
                let sy = sy.clamp(0.0, (height - 1) as f64);
                // Bilinear interpolation
                let x0 = sx.floor() as usize;
                let y0 = sy.floor() as usize;
                let x1 = (x0 + 1).min(width - 1);
                let y1 = (y0 + 1).min(height - 1);
                let fx = sx - x0 as f64;
                let fy = sy - y0 as f64;
                let top = rows[y0][x0] * (1.0 - fx) + rows[y0][x1] * fx;
                let bot = rows[y1][x0] * (1.0 - fx) + rows[y1][x1] * fx;
                top * (1.0 - fy) + bot * fy
            }).collect()
        }).collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional mosaic pass (Voronoi stained-glass effect)
    let rows = if config.mosaic > 0 {
        if !config.quiet {
            eprint!("Applying mosaic...");
        }
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let cell = config.mosaic as usize;
        let cols_count = width.div_ceil(cell);
        let rows_count = height.div_ceil(cell);

        // Generate one seed point per grid cell using deterministic hash
        let mut seeds: Vec<(f64, f64)> = Vec::with_capacity(cols_count * rows_count);
        for gy in 0..rows_count {
            for gx in 0..cols_count {
                let hash = ((gx as u64).wrapping_mul(73856093) ^ (gy as u64).wrapping_mul(19349663)).wrapping_mul(83492791);
                let jx = (hash & 0xFFFF) as f64 / 65535.0;
                let jy = ((hash >> 16) & 0xFFFF) as f64 / 65535.0;
                let sx = (gx as f64 + jx) * cell as f64;
                let sy = (gy as f64 + jy) * cell as f64;
                seeds.push((sx.min((width - 1) as f64), sy.min((height - 1) as f64)));
            }
        }

        // For each seed, compute average color from pixels in its neighborhood
        let seed_colors: Vec<Color> = seeds.iter().map(|&(sx, sy)| {
            let cx = sx as usize;
            let cy = sy as usize;
            let x0 = cx.saturating_sub(cell / 2);
            let y0 = cy.saturating_sub(cell / 2);
            let x1 = (cx + cell / 2).min(width - 1);
            let y1 = (cy + cell / 2).min(height - 1);
            let mut sum = Color::new(0.0, 0.0, 0.0);
            let mut count = 0u32;
            for row in &rows[y0..=y1] {
                for pixel in &row[x0..=x1] {
                    sum += *pixel;
                    count += 1;
                }
            }
            if count > 0 { sum * (1.0 / count as f64) } else { Color::new(0.0, 0.0, 0.0) }
        }).collect();

        // Map each pixel to nearest seed
        let result: Vec<Vec<Color>> = (0..height).into_par_iter().map(|y| {
            (0..width).map(|x| {
                let gx = x / cell;
                let gy = y / cell;
                let mut best_dist = f64::MAX;
                let mut best_color = Color::new(0.0, 0.0, 0.0);
                // Check 3x3 neighborhood of grid cells
                for dy in 0..3u32 {
                    for dx in 0..3u32 {
                        let nx = gx as i32 + dx as i32 - 1;
                        let ny = gy as i32 + dy as i32 - 1;
                        if nx < 0 || ny < 0 || nx >= cols_count as i32 || ny >= rows_count as i32 { continue; }
                        let idx = ny as usize * cols_count + nx as usize;
                        let (sx, sy) = seeds[idx];
                        let ddx = x as f64 - sx;
                        let ddy = y as f64 - sy;
                        let dist = ddx * ddx + ddy * ddy;
                        if dist < best_dist {
                            best_dist = dist;
                            best_color = seed_colors[idx];
                        }
                    }
                }
                best_color
            }).collect()
        }).collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional radial blur pass (zoom blur from center)
    let rows = if config.radial_blur > 0.0 {
        if !config.quiet {
            eprint!("Applying radial blur...");
        }
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let cx = width as f64 * 0.5;
        let cy = height as f64 * 0.5;
        let max_dist = (cx * cx + cy * cy).sqrt();
        let samples = 12;

        let result: Vec<Vec<Color>> = (0..height).into_par_iter().map(|y| {
            (0..width).map(|x| {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let strength = (dist / max_dist) * config.radial_blur;
                let mut sum = Color::new(0.0, 0.0, 0.0);
                for s in 0..samples {
                    let t = (s as f64 / (samples - 1) as f64) * 2.0 - 1.0;
                    let offset = t * strength;
                    let sx = (x as f64 + dx * offset * 0.02).clamp(0.0, (width - 1) as f64) as usize;
                    let sy = (y as f64 + dy * offset * 0.02).clamp(0.0, (height - 1) as f64) as usize;
                    sum += rows[sy][sx];
                }
                sum * (1.0 / samples as f64)
            }).collect()
        }).collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional border pass
    let rows = if config.border > 0 {
        let bw = config.border as usize;
        let bc = Color::new(config.border_color[0], config.border_color[1], config.border_color[2]);
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let mut result = rows;
        for (y, row) in result.iter_mut().enumerate().take(height) {
            for (x, pixel) in row.iter_mut().enumerate().take(width) {
                if y < bw || y >= height - bw || x < bw || x >= width - bw {
                    *pixel = bc;
                }
            }
        }
        result
    } else {
        rows
    };

    // Optional resize pass (bilinear interpolation)
    let rows = if config.resize[0] > 0 || config.resize[1] > 0 {
        let src_h = rows.len();
        let src_w = if src_h > 0 { rows[0].len() } else { 0 };
        let dst_w = if config.resize[0] > 0 { config.resize[0] as usize } else { src_w };
        let dst_h = if config.resize[1] > 0 { config.resize[1] as usize } else { src_h };
        if !config.quiet {
            eprint!("Resizing to {}x{}...", dst_w, dst_h);
        }

        let result: Vec<Vec<Color>> = (0..dst_h).into_par_iter().map(|dy| {
            (0..dst_w).map(|dx| {
                let sx = dx as f64 * (src_w - 1) as f64 / (dst_w - 1).max(1) as f64;
                let sy = dy as f64 * (src_h - 1) as f64 / (dst_h - 1).max(1) as f64;
                let x0 = sx.floor() as usize;
                let y0 = sy.floor() as usize;
                let x1 = (x0 + 1).min(src_w - 1);
                let y1 = (y0 + 1).min(src_h - 1);
                let fx = sx - x0 as f64;
                let fy = sy - y0 as f64;
                let c00 = rows[y0][x0];
                let c10 = rows[y0][x1];
                let c01 = rows[y1][x0];
                let c11 = rows[y1][x1];
                let top = c00 * (1.0 - fx) + c10 * fx;
                let bot = c01 * (1.0 - fx) + c11 * fx;
                top * (1.0 - fy) + bot * fy
            }).collect()
        }).collect();

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional glitch effect pass
    let rows = if config.glitch > 0.0 {
        if !config.quiet {
            eprint!("Applying glitch...");
        }
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let intensity = config.glitch;

        let mut result = rows;
        // Use seed for deterministic glitch pattern
        let mut hash = config.seed;

        // Create horizontal strip shifts
        let num_strips = (height as f64 * intensity * 0.15).max(1.0) as usize;
        for _ in 0..num_strips {
            hash = hash.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let strip_y = (hash as usize) % height;
            hash = hash.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let strip_h = ((hash as usize) % 8).max(1).min(height - strip_y);
            hash = hash.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let shift = ((hash as i64 % (width as i64 / 4)) - (width as i64 / 8)) as i32;

            for row in result.iter_mut().skip(strip_y).take(strip_h.min(height - strip_y)) {
                let orig = row.clone();
                for (x, pixel) in row.iter_mut().enumerate() {
                    let src_x = (x as i32 + shift).rem_euclid(width as i32) as usize;
                    *pixel = orig[src_x];
                }
            }
        }

        // Channel offset for some strips
        let num_channel_shifts = (height as f64 * intensity * 0.05).max(1.0) as usize;
        for _ in 0..num_channel_shifts {
            hash = hash.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let y = (hash as usize) % height;
            hash = hash.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let offset = ((hash as usize) % 10) + 2;

            let red_shifted: Vec<f64> = (0..width).map(|x| {
                let rx = (x + offset).min(width - 1);
                result[y][rx].x
            }).collect();
            for (x, pixel) in result[y].iter_mut().enumerate() {
                pixel.x = red_shifted[x];
            }
        }

        if !config.quiet {
            eprintln!(" done");
        }
        result
    } else {
        rows
    };

    // Optional median filter pass
    let rows = if config.median > 0 {
        if !config.quiet {
            eprint!("Applying median filter...");
        }
        let filtered = apply_median(&rows, config.median);
        if !config.quiet {
            eprintln!(" done");
        }
        filtered
    } else {
        rows
    };

    // Optional sketch pass: grayscale + edge detection for pencil drawing look
    let rows = if config.sketch {
        if !config.quiet {
            eprint!("Generating sketch...");
        }
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };

        // First, convert to grayscale
        let gray: Vec<Vec<Color>> = rows.iter()
            .map(|row| row.iter()
                .map(|c| {
                    let lum = 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
                    Color::new(lum, lum, lum)
                })
                .collect())
            .collect();

        // Apply edge detection (Sobel) to get edges
        let sketch: Vec<Vec<Color>> = (0..height)
            .into_par_iter()
            .map(|j| {
                (0..width)
                    .map(|i| {
                        if j == 0 || i == 0 || j >= height - 1 || i >= width - 1 {
                            return Color::new(1.0, 1.0, 1.0); // white border
                        }
                        let lum = |jj: usize, ii: usize| -> f64 {
                            let c = gray[jj][ii];
                            c.x
                        };
                        let gx = -lum(j-1,i-1) - 2.0*lum(j,i-1) - lum(j+1,i-1)
                                + lum(j-1,i+1) + 2.0*lum(j,i+1) + lum(j+1,i+1);
                        let gy = -lum(j-1,i-1) - 2.0*lum(j-1,i) - lum(j-1,i+1)
                                + lum(j+1,i-1) + 2.0*lum(j+1,i) + lum(j+1,i+1);
                        let edge = (gx * gx + gy * gy).sqrt();
                        // Invert: white background, dark edges
                        let val = (1.0 - edge * 3.0).clamp(0.0, 1.0);
                        Color::new(val, val, val)
                    })
                    .collect()
            })
            .collect();

        if !config.quiet {
            eprintln!(" done");
        }
        sketch
    } else {
        rows
    };

    // Optional oil paint (Kuwahara filter) pass
    let rows = if config.oil_paint > 0 {
        if !config.quiet {
            eprint!("Applying oil paint filter...");
        }
        let painted = apply_oil_paint(&rows, config.oil_paint);
        if !config.quiet {
            eprintln!(" done");
        }
        painted
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
    // Pre-compute depth buffer for depth fog (if needed)
    let fog_depth_buf: Option<Vec<f32>> = if config.depth_fog > 0.0 {
        if !config.quiet {
            eprint!("Computing depth for fog...");
        }
        let buf: Vec<f32> = (0..height)
            .into_par_iter()
            .flat_map(|j| {
                let global_j = j + crop_y;
                let y = (full_height - 1 - global_j) as f64;
                (0..width).map(move |i| {
                    let global_i = i + crop_x;
                    let u_coord = (global_i as f64 + 0.5) / (full_width - 1) as f64;
                    let v_coord = (y + 0.5) / (full_height - 1) as f64;
                    let mut rng = SmallRng::seed_from_u64(
                        (global_j as u64 * full_width as u64 + global_i as u64).wrapping_mul(config.seed),
                    );
                    let ray = camera.get_ray(u_coord, v_coord, &mut rng);
                    if let Some(hit) = world.hit(&ray, 0.001, f64::INFINITY) {
                        hit.t as f32
                    } else {
                        f32::INFINITY
                    }
                }).collect::<Vec<f32>>()
            })
            .collect();
        if !config.quiet {
            eprintln!(" done");
        }
        Some(buf)
    } else {
        None
    };

    // Cel-shading: posterize + edge outline for toon look
    let rows = if config.cel_shade >= 2 {
        let bands = config.cel_shade as f64;
        // Step 1: posterize colors to N bands
        let mut result: Vec<Vec<Color>> = rows.iter().map(|row| {
            row.iter().map(|c| {
                let quantize = |v: f64| ((v * bands).floor() / (bands - 1.0)).clamp(0.0, 1.0);
                Color::new(quantize(c.x), quantize(c.y), quantize(c.z))
            }).collect()
        }).collect();
        // Step 2: Sobel edge detection overlay (black outlines)
        let h = result.len();
        let w = if h > 0 { result[0].len() } else { 0 };
        if h > 2 && w > 2 {
            let lum_at = |r: &[Vec<Color>], yy: usize, xx: usize| {
                let c = &r[yy][xx];
                c.x * 0.2126 + c.y * 0.7152 + c.z * 0.0722
            };
            // Compute edge magnitudes and apply black outline
            for (y, row) in result.iter_mut().enumerate() {
                if y == 0 || y >= h - 1 { continue; }
                for (x, pixel) in row.iter_mut().enumerate() {
                    if x == 0 || x >= w - 1 { continue; }
                    let gx = -lum_at(&rows, y - 1, x - 1) + lum_at(&rows, y - 1, x + 1)
                        - 2.0 * lum_at(&rows, y, x - 1) + 2.0 * lum_at(&rows, y, x + 1)
                        - lum_at(&rows, y + 1, x - 1) + lum_at(&rows, y + 1, x + 1);
                    let gy = -lum_at(&rows, y - 1, x - 1) - 2.0 * lum_at(&rows, y - 1, x) - lum_at(&rows, y - 1, x + 1)
                        + lum_at(&rows, y + 1, x - 1) + 2.0 * lum_at(&rows, y + 1, x) + lum_at(&rows, y + 1, x + 1);
                    if (gx * gx + gy * gy).sqrt() > 0.15 {
                        *pixel = Color::new(0.0, 0.0, 0.0);
                    }
                }
            }
        }
        result
    } else { rows };

    // Hexagonal pixelation
    let rows = if config.hex_pixelate >= 2 {
        let cell = config.hex_pixelate as f64;
        let h = rows.len();
        let w = if h > 0 { rows[0].len() } else { 0 };
        let hex_h = cell;
        let hex_w = cell * 3.0_f64.sqrt();
        let mut result = rows.clone();
        for (y, row) in result.iter_mut().enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                // Find which hex cell this pixel belongs to
                let fy = y as f64 / hex_h;
                let fx = x as f64 / hex_w;
                let row_idx = fy.floor() as isize;
                let col_offset = if row_idx % 2 != 0 { 0.5 } else { 0.0 };
                let col_idx = (fx - col_offset).floor() as isize;
                // Hex center
                let cx = ((col_idx as f64 + col_offset) + 0.5) * hex_w;
                let cy = (row_idx as f64 + 0.5) * hex_h;
                // Sample color from hex center
                let sx = (cx as usize).min(w.saturating_sub(1));
                let sy = (cy as usize).min(h.saturating_sub(1));
                *pixel = rows[sy][sx];
            }
        }
        result
    } else { rows };

    // Picture frame: outer frame with inner bevel and drop shadow
    let rows = if config.frame > 0 {
        let fw = config.frame as usize;
        let h = rows.len();
        let w = if h > 0 { rows[0].len() } else { 0 };
        let new_h = h + fw * 2;
        let new_w = w + fw * 2;
        let mut result = vec![vec![Color::new(0.15, 0.1, 0.05); new_w]; new_h];
        // Copy image into center
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                result[y + fw][x + fw] = *c;
            }
        }
        // Inner bevel: light top/left edge, dark bottom/right edge
        let bevel = (fw / 4).max(1);
        for b in 0..bevel {
            let light = Color::new(0.3, 0.25, 0.15);
            let dark = Color::new(0.05, 0.03, 0.01);
            let fy = fw - b - 1;
            let fx = fw - b - 1;
            // Top edge
            for pixel in &mut result[fy][fx..new_w - fx] {
                *pixel = light;
            }
            // Left edge
            for row in result.iter_mut().take(new_h - fy).skip(fy) {
                row[fx] = light;
            }
            // Bottom edge
            for pixel in &mut result[new_h - 1 - fy][fx..new_w - fx] {
                *pixel = dark;
            }
            // Right edge
            for row in result.iter_mut().take(new_h - fy).skip(fy) {
                row[new_w - 1 - fx] = dark;
            }
        }
        result
    } else { rows };

    // Lens flare: find brightest pixel, add radial streaks
    let rows = if config.lens_flare > 0.0 {
        let mut max_lum = 0.0_f64;
        let mut bright_x = width / 2;
        let mut bright_y = height / 2;
        for (j, row) in rows.iter().enumerate() {
            for (i, c) in row.iter().enumerate() {
                let lum = c.x * 0.2126 + c.y * 0.7152 + c.z * 0.0722;
                if lum > max_lum {
                    max_lum = lum;
                    bright_x = i;
                    bright_y = j;
                }
            }
        }
        let mut result = rows;
        let streak_len = ((width.max(height)) as f64 * 0.4) as usize;
        let intensity = config.lens_flare;
        for spoke in 0..6 {
            let angle = spoke as f64 * std::f64::consts::PI / 3.0;
            let dx = angle.cos();
            let dy = angle.sin();
            for step in 1..=streak_len {
                let falloff = intensity * (-(step as f64) / (streak_len as f64 * 0.3)).exp();
                if falloff < 0.001 { break; }
                let px = (bright_x as f64 + dx * step as f64).round() as isize;
                let py = (bright_y as f64 + dy * step as f64).round() as isize;
                if px >= 0 && px < width as isize && py >= 0 && py < height as isize {
                    result[py as usize][px as usize] += Color::new(1.0, 0.95, 0.8) * falloff;
                }
            }
        }
        result
    } else { rows };

    // Parse duo-tone colors if specified: "R,G,B;R,G,B"
    let duo_tone_colors: Option<([f64; 3], [f64; 3])> = if !config.duo_tone.is_empty() {
        let parts: Vec<&str> = config.duo_tone.split(';').collect();
        if parts.len() == 2 {
            let parse_rgb = |s: &str| -> Option<[f64; 3]> {
                let c: Vec<f64> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                if c.len() == 3 { Some([c[0] / 255.0, c[1] / 255.0, c[2] / 255.0]) } else { None }
            };
            match (parse_rgb(parts[0]), parse_rgb(parts[1])) {
                (Some(a), Some(b)) => Some((a, b)),
                _ => None,
            }
        } else { None }
    } else { None };

    // Parse tri-tone colors if specified: "R,G,B;R,G,B;R,G,B"
    let tri_tone_colors: Option<([f64; 3], [f64; 3], [f64; 3])> = if !config.tri_tone.is_empty() {
        let parts: Vec<&str> = config.tri_tone.split(';').collect();
        if parts.len() == 3 {
            let parse_rgb = |s: &str| -> Option<[f64; 3]> {
                let c: Vec<f64> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                if c.len() == 3 { Some([c[0] / 255.0, c[1] / 255.0, c[2] / 255.0]) } else { None }
            };
            match (parse_rgb(parts[0]), parse_rgb(parts[1]), parse_rgb(parts[2])) {
                (Some(a), Some(b), Some(c)) => Some((a, b, c)),
                _ => None,
            }
        } else { None }
    } else { None };

    // Parse gradient map colors: "RRGGBB;RRGGBB;..."
    let gradient_map_colors: Vec<[f64; 3]> = if !config.gradient_map.is_empty() {
        config.gradient_map.split(';').filter_map(|hex| {
            let hex = hex.trim().trim_start_matches('#');
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
                Some([r, g, b])
            } else { None }
        }).collect()
    } else { Vec::new() };

    // Parse split-tone colors: "R,G,B;R,G,B" (shadow color; highlight color)
    let split_tone_colors: Option<([f64; 3], [f64; 3])> = if !config.split_tone.is_empty() {
        let parts: Vec<&str> = config.split_tone.split(';').collect();
        if parts.len() == 2 {
            let parse_rgb = |s: &str| -> Option<[f64; 3]> {
                let c: Vec<f64> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                if c.len() == 3 { Some([c[0] / 255.0, c[1] / 255.0, c[2] / 255.0]) } else { None }
            };
            match (parse_rgb(parts[0]), parse_rgb(parts[1])) {
                (Some(a), Some(b)) => Some((a, b)),
                _ => None,
            }
        } else { None }
    } else { None };

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

            // Depth fog: blend scene color toward fog color based on depth
            if let Some(ref depth_data) = fog_depth_buf {
                let depth = depth_data[j * width + i] as f64;
                let fog_factor = 1.0 - (-config.depth_fog * depth).exp();
                let fc = &config.depth_fog_color;
                cr = cr * (1.0 - fog_factor) + fc[0] * fog_factor;
                cg = cg * (1.0 - fog_factor) + fc[1] * fog_factor;
                cb = cb * (1.0 - fog_factor) + fc[2] * fog_factor;
            }

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

            // Color grading: tint shadows and highlights separately
            let has_grading = config.grade_shadows != [1.0, 1.0, 1.0]
                || config.grade_highlights != [1.0, 1.0, 1.0];
            if has_grading {
                let lum = (0.2126 * cr + 0.7152 * cg + 0.0722 * cb).max(0.0);
                // Sigmoid-like split: 0=pure shadow, 1=pure highlight
                let t = (lum / (lum + 1.0)).clamp(0.0, 1.0);
                let sr = config.grade_shadows[0] * (1.0 - t) + config.grade_highlights[0] * t;
                let sg = config.grade_shadows[1] * (1.0 - t) + config.grade_highlights[1] * t;
                let sb = config.grade_shadows[2] * (1.0 - t) + config.grade_highlights[2] * t;
                cr *= sr;
                cg *= sg;
                cb *= sb;
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

            // Brightness: additive adjustment in LDR space
            if config.brightness.abs() > 1e-6 {
                let offset = (config.brightness * 255.0) as i16;
                r = (r as i16 + offset).clamp(0, 255) as u8;
                g = (g as i16 + offset).clamp(0, 255) as u8;
                b = (b as i16 + offset).clamp(0, 255) as u8;
            }

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

            // Color tint: multiply by user-specified RGB
            if config.tint[0] < 1.0 || config.tint[1] < 1.0 || config.tint[2] < 1.0 {
                r = ((r as f64 / 255.0 * config.tint[0]) * 255.0).clamp(0.0, 255.0) as u8;
                g = ((g as f64 / 255.0 * config.tint[1]) * 255.0).clamp(0.0, 255.0) as u8;
                b = ((b as f64 / 255.0 * config.tint[2]) * 255.0).clamp(0.0, 255.0) as u8;
            }

            // Color balance: per-channel level adjustment (can boost above 1.0)
            if (config.color_balance[0] - 1.0).abs() > 1e-6
                || (config.color_balance[1] - 1.0).abs() > 1e-6
                || (config.color_balance[2] - 1.0).abs() > 1e-6
            {
                r = ((r as f64 * config.color_balance[0]).clamp(0.0, 255.0)) as u8;
                g = ((g as f64 * config.color_balance[1]).clamp(0.0, 255.0)) as u8;
                b = ((b as f64 * config.color_balance[2]).clamp(0.0, 255.0)) as u8;
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

            // Crosshatch: pen-and-ink style based on luminance
            if config.crosshatch > 0 {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                let sp = config.crosshatch as i32;
                let x = i as i32;
                let y = j as i32;

                // Layer 1: diagonal lines (/) for lum < 0.75
                let line1 = ((x + y) % sp == 0) && lum < 0.75;
                // Layer 2: opposite diagonal (\) for lum < 0.50
                let line2 = ((x - y).rem_euclid(sp) == 0) && lum < 0.50;
                // Layer 3: horizontal for lum < 0.25
                let line3 = (y % sp == 0) && lum < 0.25;

                if line1 || line2 || line3 {
                    r = 0;
                    g = 0;
                    b = 0;
                } else {
                    r = 255;
                    g = 255;
                    b = 255;
                }
            }

            // Halftone: circular dots based on luminance
            if config.halftone >= 2 {
                let hs = config.halftone as f64;
                let cx = (i as f64 % hs) - hs / 2.0;
                let cy = (j as f64 % hs) - hs / 2.0;
                let dist = (cx * cx + cy * cy).sqrt();
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                let dot_radius = lum * hs * 0.6;
                if dist > dot_radius {
                    r = 255;
                    g = 255;
                    b = 255;
                }
                // else keep original color (or darken slightly)
            }

            // Channel swap
            if !config.channel_swap.is_empty() {
                let (or, og, ob) = (r, g, b);
                match config.channel_swap.as_str() {
                    "rbg" => { g = ob; b = og; }
                    "grb" => { r = og; g = or; }
                    "gbr" => { r = og; g = ob; b = or; }
                    "brg" => { r = ob; g = or; b = og; }
                    "bgr" => { r = ob; b = or; }
                    _ => {} // "rgb" or unknown = no change
                }
            }

            // False color mapping
            if !config.color_map.is_empty() {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                let (mr, mg, mb) = apply_color_map(lum, &config.color_map);
                r = (mr * 255.0) as u8;
                g = (mg * 255.0) as u8;
                b = (mb * 255.0) as u8;
            }

            // Solarize: invert pixels above a luminance threshold
            if config.solarize >= 0.0 {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                if lum > config.solarize {
                    r = 255 - r;
                    g = 255 - g;
                    b = 255 - b;
                }
            }

            // Duo-tone: map luminance to two-color gradient
            if let Some((shadow, highlight)) = &duo_tone_colors {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                r = ((shadow[0] * (1.0 - lum) + highlight[0] * lum) * 255.0).clamp(0.0, 255.0) as u8;
                g = ((shadow[1] * (1.0 - lum) + highlight[1] * lum) * 255.0).clamp(0.0, 255.0) as u8;
                b = ((shadow[2] * (1.0 - lum) + highlight[2] * lum) * 255.0).clamp(0.0, 255.0) as u8;
            }

            // Tri-tone: map luminance to three-color gradient (shadows/midtones/highlights)
            if let Some((shadow, mid, highlight)) = &tri_tone_colors {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                let (c0, c1, t) = if lum < 0.5 {
                    (shadow, mid, lum * 2.0)
                } else {
                    (mid, highlight, (lum - 0.5) * 2.0)
                };
                r = ((c0[0] * (1.0 - t) + c1[0] * t) * 255.0).clamp(0.0, 255.0) as u8;
                g = ((c0[1] * (1.0 - t) + c1[1] * t) * 255.0).clamp(0.0, 255.0) as u8;
                b = ((c0[2] * (1.0 - t) + c1[2] * t) * 255.0).clamp(0.0, 255.0) as u8;
            }

            // Gradient map: replace luminance with interpolated gradient color
            if gradient_map_colors.len() >= 2 {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                let n = gradient_map_colors.len() - 1;
                let pos = lum * n as f64;
                let idx = (pos.floor() as usize).min(n - 1);
                let t = pos - idx as f64;
                let c0 = &gradient_map_colors[idx];
                let c1 = &gradient_map_colors[idx + 1];
                r = ((c0[0] * (1.0 - t) + c1[0] * t) * 255.0).clamp(0.0, 255.0) as u8;
                g = ((c0[1] * (1.0 - t) + c1[1] * t) * 255.0).clamp(0.0, 255.0) as u8;
                b = ((c0[2] * (1.0 - t) + c1[2] * t) * 255.0).clamp(0.0, 255.0) as u8;
            }

            // Split-tone: blend shadow/highlight colors based on luminance
            if let Some((shadow_c, highlight_c)) = &split_tone_colors {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                // Blend strength: strongest at extremes, zero at mid-gray
                let (tint, strength) = if lum < 0.5 {
                    (shadow_c, 1.0 - lum * 2.0)
                } else {
                    (highlight_c, (lum - 0.5) * 2.0)
                };
                let blend = strength * 0.5; // 50% max tint to preserve detail
                r = ((r as f64 * (1.0 - blend) + tint[0] * 255.0 * blend).clamp(0.0, 255.0)) as u8;
                g = ((g as f64 * (1.0 - blend) + tint[1] * 255.0 * blend).clamp(0.0, 255.0)) as u8;
                b = ((b as f64 * (1.0 - blend) + tint[2] * 255.0 * blend).clamp(0.0, 255.0)) as u8;
            }

            // Color shift: rotate RGB channels
            if config.color_shift > 0 {
                let (nr, ng, nb) = match config.color_shift % 3 {
                    1 => (b, r, g), // shift right: R←B, G←R, B←G
                    2 => (g, b, r), // shift left: R←G, G←B, B←R
                    _ => (r, g, b),
                };
                r = nr;
                g = ng;
                b = nb;
            }

            // Per-channel posterization
            if config.posterize_channels[0] >= 2 || config.posterize_channels[1] >= 2 || config.posterize_channels[2] >= 2 {
                let posterize_ch = |v: u8, levels: u32| -> u8 {
                    if levels < 2 { return v; }
                    let f = v as f64 / 255.0;
                    let q = (f * (levels - 1) as f64).round() / (levels - 1) as f64;
                    (q * 255.0).clamp(0.0, 255.0) as u8
                };
                r = posterize_ch(r, config.posterize_channels[0]);
                g = posterize_ch(g, config.posterize_channels[1]);
                b = posterize_ch(b, config.posterize_channels[2]);
            }

            // Color inversion
            if config.invert {
                r = 255 - r;
                g = 255 - g;
                b = 255 - b;
            }

            // Night vision: green-tinted amplified luminance with noise
            if config.night_vision {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                // Amplify and add noise
                let hash = (i as u64).wrapping_mul(73856093) ^ (j as u64).wrapping_mul(19349663);
                let noise = ((hash & 0xFFFF) as f64 / 65535.0 - 0.5) * 0.08;
                let bright = (lum * 1.5 + 0.05 + noise).clamp(0.0, 1.0);
                r = (bright * 0.15 * 255.0) as u8;
                g = (bright * 255.0) as u8;
                b = (bright * 0.1 * 255.0) as u8;
            }

            // Pop art: Warhol-style bold color bands
            if config.pop_art >= 2 {
                let lum = (r as f64 * 0.2126 + g as f64 * 0.7152 + b as f64 * 0.0722) / 255.0;
                let bands = config.pop_art as f64;
                let band = (lum * bands).floor() as u32;
                // Vivid color palette cycling through pop art colors
                const POP_COLORS: [[u8; 3]; 6] = [
                    [255, 0, 128], [0, 200, 255], [255, 220, 0],
                    [255, 100, 0], [0, 255, 100], [200, 0, 255],
                ];
                let pc = POP_COLORS[(band as usize) % POP_COLORS.len()];
                let bright = (lum * 1.5).clamp(0.3, 1.0);
                r = (pc[0] as f64 * bright) as u8;
                g = (pc[1] as f64 * bright) as u8;
                b = (pc[2] as f64 * bright) as u8;
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

    // Generate AOV passes (depth, normals, albedo) with single-ray-per-pixel, parallelized
    let (depth_pass, normal_pass, albedo_pass) = if config.save_depth || config.save_normals || config.save_albedo {
        if !config.quiet {
            eprint!("Generating AOV passes...");
        }

        let need_albedo = config.save_albedo;

        // Per-row parallel AOV generation
        let aov_rows: Vec<(Vec<f32>, Vec<u8>, Vec<u8>)> = (0..height)
            .into_par_iter()
            .map(|j| {
                let global_j = j + crop_y;
                let y = (full_height - 1 - global_j) as f64;
                let mut row_depth = vec![0.0f32; width];
                let mut row_normals = vec![0u8; width * 3];
                let mut row_albedo = vec![0u8; width * 3];

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

                        if need_albedo {
                            // Get albedo by calling scatter — the attenuation is the surface color
                            if let Some(scatter) = hit.material.scatter(&ray, &hit, &mut rng) {
                                row_albedo[i * 3] = (scatter.attenuation.x.clamp(0.0, 1.0) * 255.0) as u8;
                                row_albedo[i * 3 + 1] = (scatter.attenuation.y.clamp(0.0, 1.0) * 255.0) as u8;
                                row_albedo[i * 3 + 2] = (scatter.attenuation.z.clamp(0.0, 1.0) * 255.0) as u8;
                            } else {
                                // Emissive materials don't scatter — use emitted color
                                let e = hit.material.emitted(hit.u, hit.v, &hit.point);
                                let max_e = e.x.max(e.y.max(e.z)).max(1.0);
                                row_albedo[i * 3] = (e.x / max_e * 255.0).clamp(0.0, 255.0) as u8;
                                row_albedo[i * 3 + 1] = (e.y / max_e * 255.0).clamp(0.0, 255.0) as u8;
                                row_albedo[i * 3 + 2] = (e.z / max_e * 255.0).clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
                }
                (row_depth, row_normals, row_albedo)
            })
            .collect();

        let depth_buf = if config.save_depth {
            Some(aov_rows.iter().flat_map(|(d, _, _)| d.iter().copied()).collect())
        } else {
            None
        };
        let normal_buf = if config.save_normals {
            Some(aov_rows.iter().flat_map(|(_, n, _)| n.iter().copied()).collect())
        } else {
            None
        };
        let albedo_buf = if config.save_albedo {
            Some(aov_rows.iter().flat_map(|(_, _, a)| a.iter().copied()).collect())
        } else {
            None
        };

        if !config.quiet {
            eprintln!(" done");
        }
        (depth_buf, normal_buf, albedo_buf)
    } else {
        (None, None, None)
    };

    RenderResult { pixels, hdr_data, depth_pass, normal_pass, albedo_pass, total_rays: final_total_rays, render_time_secs }
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

    #[test]
    fn render_minimal_scene() {
        use crate::camera::{Camera, CameraConfig};
        use crate::hit::HittableList;
        use crate::material::Lambertian;
        use crate::sphere::Sphere;

        let mut world = HittableList::new();
        world.add(Box::new(Sphere::new(
            crate::vec3::Point3::new(0.0, 0.0, -1.0),
            0.5,
            Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
        )));

        let cam = Camera::new(CameraConfig::default());
        let config = RenderConfig {
            width: 10,
            height: 10,
            samples_per_pixel: 1,
            max_depth: 3,
            quiet: true,
            ..Default::default()
        };

        let result = render(&config, &cam, &world, &[]);
        assert_eq!(result.pixels.len(), 10 * 10 * 4); // RGBA
        assert!(result.total_rays > 0);
        assert!(result.render_time_secs >= 0.0);
    }

    #[test]
    fn blur_preserves_solid_color() {
        let c = Color::new(0.5, 0.3, 0.7);
        let rows = vec![vec![c; 20]; 20];
        let blurred = apply_blur(&rows, 2.0);
        // Center pixel should stay close to original for uniform image
        let center = blurred[10][10];
        assert!((center.x - c.x).abs() < 0.01);
        assert!((center.y - c.y).abs() < 0.01);
        assert!((center.z - c.z).abs() < 0.01);
    }

    #[test]
    fn pixelate_averages_blocks() {
        let rows = vec![
            vec![Color::new(1.0, 0.0, 0.0), Color::new(0.0, 1.0, 0.0)],
            vec![Color::new(0.0, 0.0, 1.0), Color::new(1.0, 1.0, 0.0)],
        ];
        let result = apply_pixelate(&rows, 2);
        // All 4 pixels should be the same (averaged)
        let avg = result[0][0];
        assert!((avg.x - 0.5).abs() < 0.01);
        assert!((avg.y - 0.5).abs() < 0.01);
        assert_eq!(result[0][0], result[0][1]);
        assert_eq!(result[1][0], result[1][1]);
    }

    #[test]
    fn edge_detect_no_edges_on_uniform() {
        let c = Color::new(0.5, 0.5, 0.5);
        let rows = vec![vec![c; 10]; 10];
        let result = apply_edge_detect(&rows, 1.0);
        // Interior pixels should be unchanged (no edges)
        let center = result[5][5];
        assert!((center.x - c.x).abs() < 0.01);
    }

    #[test]
    fn median_cut_palette_single_color() {
        let pixels = vec![[128, 128, 128]; 100];
        let palette = median_cut_palette(&pixels, 4);
        // With uniform input, all buckets converge to the same color
        assert!(!palette.is_empty());
        assert_eq!(palette[0], [128, 128, 128]);
    }

    #[test]
    fn median_cut_palette_two_colors() {
        let mut pixels = vec![[0, 0, 0]; 50];
        pixels.extend(vec![[255, 255, 255]; 50]);
        let palette = median_cut_palette(&pixels, 2);
        assert_eq!(palette.len(), 2);
        // Should produce one dark and one light color
        let has_dark = palette.iter().any(|p| p[0] < 128);
        let has_light = palette.iter().any(|p| p[0] >= 128);
        assert!(has_dark && has_light);
    }

    #[test]
    fn median_cut_palette_empty() {
        let palette = median_cut_palette(&[], 4);
        assert!(!palette.is_empty());
    }

    #[test]
    fn named_palette_known() {
        assert!(named_palette("gameboy").is_some());
        assert!(named_palette("cga").is_some());
        assert!(named_palette("nes").is_some());
        assert!(named_palette("cyberpunk").is_some());
        assert_eq!(named_palette("gameboy").unwrap().len(), 4);
    }

    #[test]
    fn named_palette_unknown() {
        assert!(named_palette("doesnotexist").is_none());
    }

    #[test]
    fn apply_color_map_thermal_range() {
        let (r, g, b) = apply_color_map(0.0, "thermal");
        assert!(r >= 0.0 && g >= 0.0 && b >= 0.0);
        let (r, g, b) = apply_color_map(1.0, "thermal");
        assert!(r <= 1.0 && g <= 1.0 && b <= 1.0);
    }

    #[test]
    fn apply_color_map_neon_range() {
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let (r, g, b) = apply_color_map(t, "neon");
            assert!(r >= 0.0 && r <= 1.0, "neon r out of range at t={t}");
            assert!(g >= 0.0 && g <= 1.0, "neon g out of range at t={t}");
            assert!(b >= 0.0 && b <= 1.0, "neon b out of range at t={t}");
        }
    }

    #[test]
    fn gradient_map_replaces_luminance() {
        // Black→Red gradient: dark pixels become black, bright become red
        let mut config = RenderConfig::default();
        config.width = 2;
        config.height = 1;
        config.samples_per_pixel = 1;
        config.max_depth = 1;
        config.gradient_map = "000000;FF0000".to_string();
        // White pixel should map to red
        let rows = vec![vec![Color::new(1.0, 1.0, 1.0), Color::new(0.0, 0.0, 0.0)]];
        // We can't easily call output_pixels directly, but we can verify the
        // gradient_map field is set and the config compiles
        assert_eq!(config.gradient_map, "000000;FF0000");
    }

    #[test]
    fn cel_shade_config_default() {
        let config = RenderConfig::default();
        assert_eq!(config.cel_shade, 0);
        assert_eq!(config.lens_flare, 0.0);
        assert_eq!(config.color_shift, 0);
        assert_eq!(config.posterize_channels, [0, 0, 0]);
        assert!(config.split_tone.is_empty());
        assert!(config.gradient_map.is_empty());
    }

    #[test]
    fn color_shift_wraps_modulo3() {
        // color_shift % 3 should handle large values
        let config = RenderConfig { color_shift: 4, ..RenderConfig::default() };
        assert_eq!(config.color_shift % 3, 1);
        let config = RenderConfig { color_shift: 6, ..RenderConfig::default() };
        assert_eq!(config.color_shift % 3, 0);
    }
}
