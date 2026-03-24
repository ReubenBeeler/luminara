use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rayon::prelude::*;

use crate::camera::Camera;
use crate::hit::Hittable;
use crate::material::RngCore;
use crate::ray::Ray;
use crate::vec3::Color;

pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 450,
            samples_per_pixel: 100,
            max_depth: 50,
        }
    }
}

/// Trace a single ray through the scene.
fn ray_color(ray: &Ray, world: &dyn Hittable, rng: &mut dyn RngCore, depth: u32) -> Color {
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
                    .hadamard(ray_color(&scatter.ray, world, rng, depth - 1));
        }
        return emitted;
    }

    // Sky gradient
    let unit_dir = ray.direction.unit();
    let t = 0.5 * (unit_dir.y + 1.0);
    Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
}

/// Render the scene and return a flat Vec of RGBA bytes.
pub fn render(
    config: &RenderConfig,
    camera: &Camera,
    world: &dyn Hittable,
) -> Vec<u8> {
    let width = config.width as usize;
    let height = config.height as usize;

    let rows: Vec<Vec<Color>> = (0..height)
        .into_par_iter()
        .map(|j| {
            // Each thread gets its own RNG seeded from row index.
            let mut rng = SmallRng::seed_from_u64(j as u64 * 31337);
            let y = (height - 1 - j) as f64;

            (0..width)
                .map(|i| {
                    let mut color = Color::ZERO;
                    for _ in 0..config.samples_per_pixel {
                        let u = (i as f64 + rng.random::<f64>()) / (width - 1) as f64;
                        let v = (y + rng.random::<f64>()) / (height - 1) as f64;
                        let ray = camera.get_ray(u, v, &mut rng);
                        color += ray_color(&ray, world, &mut rng, config.max_depth);
                    }
                    color / config.samples_per_pixel as f64
                })
                .collect()
        })
        .collect();

    // Convert to RGBA bytes with gamma correction (gamma = 2.0).
    let mut pixels = Vec::with_capacity(width * height * 4);
    for row in &rows {
        for color in row {
            let r = (color.x.sqrt().clamp(0.0, 0.999) * 256.0) as u8;
            let g = (color.y.sqrt().clamp(0.0, 0.999) * 256.0) as u8;
            let b = (color.z.sqrt().clamp(0.0, 0.999) * 256.0) as u8;
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    pixels
}
