use std::sync::atomic::{AtomicUsize, Ordering};

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rayon::prelude::*;

use crate::camera::Camera;
use crate::hit::Hittable;
use crate::material::RngCore;
use crate::ray::Ray;
use crate::vec3::Color;

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
        }
    }
}

pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
    pub background: Background,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 450,
            samples_per_pixel: 100,
            max_depth: 50,
            background: Background::default(),
        }
    }
}

/// Trace a single ray through the scene.
fn ray_color(ray: &Ray, world: &dyn Hittable, bg: &Background, rng: &mut dyn RngCore, depth: u32) -> Color {
    if depth == 0 {
        return Color::ZERO;
    }

    // 0.001 to avoid shadow acne
    if let Some(hit) = world.hit(ray, 0.001, f64::INFINITY) {
        let emitted = hit.material.emitted();
        if let Some(scatter) = hit.material.scatter(ray, &hit, rng) {
            return emitted
                + scatter
                    .attenuation
                    .hadamard(ray_color(&scatter.ray, world, bg, rng, depth - 1));
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
) -> Vec<u8> {
    let width = config.width as usize;
    let height = config.height as usize;

    let rows_done = AtomicUsize::new(0);
    let sqrt_spp = (config.samples_per_pixel as f64).sqrt().ceil() as u32;
    let actual_spp = sqrt_spp * sqrt_spp;

    let rows: Vec<Vec<Color>> = (0..height)
        .into_par_iter()
        .map(|j| {
            let mut rng = SmallRng::seed_from_u64(j as u64 * 31337);
            let y = (height - 1 - j) as f64;

            let row: Vec<Color> = (0..width)
                .map(|i| {
                    let mut color = Color::ZERO;
                    for sy in 0..sqrt_spp {
                        for sx in 0..sqrt_spp {
                            let u = (i as f64 + (sx as f64 + rng.random::<f64>()) / sqrt_spp as f64) / (width - 1) as f64;
                            let v = (y + (sy as f64 + rng.random::<f64>()) / sqrt_spp as f64) / (height - 1) as f64;
                            let ray = camera.get_ray(u, v, &mut rng);
                            color += ray_color(&ray, world, &config.background, &mut rng, config.max_depth);
                        }
                    }
                    color / actual_spp as f64
                })
                .collect();

            let done = rows_done.fetch_add(1, Ordering::Relaxed) + 1;
            #[allow(clippy::manual_is_multiple_of)]
            if done % 20 == 0 || done == height {
                let pct = done * 100 / height;
                eprint!("\rProgress: {pct:3}% [{done}/{height} rows]");
            }

            row
        })
        .collect();

    eprintln!();

    let total_rays = width as u64 * height as u64 * actual_spp as u64;
    eprintln!("Primary rays: {total_rays} ({actual_spp} spp, {sqrt_spp}x{sqrt_spp} stratified)");

    // Convert to RGBA bytes with ACES tone mapping + gamma correction.
    let mut pixels = Vec::with_capacity(width * height * 4);
    for row in &rows {
        for color in row {
            let r = linear_to_srgb(aces_tonemap(color.x));
            let g = linear_to_srgb(aces_tonemap(color.y));
            let b = linear_to_srgb(aces_tonemap(color.z));
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    pixels
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

/// Convert linear [0,1] to sRGB byte with gamma 2.2.
fn linear_to_srgb(x: f64) -> u8 {
    (x.powf(1.0 / 2.2).clamp(0.0, 0.999) * 256.0) as u8
}
