mod aabb;
mod bvh;
mod camera;
mod cylinder;
mod disk;
mod hit;
mod material;
mod obj;
mod plane;
mod ray;
mod rect;
mod render;
mod scene;
mod sphere;
mod texture;
mod triangle;
mod vec3;

use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (scene_name, output_path) = parse_args(&args);

    let start = Instant::now();

    let (render_config, camera, world) = match &scene_name {
        Some(path) => {
            let content = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("Failed to read scene file '{}': {}", path, e));
            scene::load_scene(&content).unwrap_or_else(|e| panic!("Scene error: {}", e))
        }
        None => {
            eprintln!("No scene file provided, rendering demo scene...");
            scene::demo_scene()
        }
    };

    eprintln!(
        "Rendering {}x{} @ {} spp, max depth {}",
        render_config.width, render_config.height, render_config.samples_per_pixel, render_config.max_depth
    );

    let pixels = render::render(&render_config, &camera, &world);

    let elapsed = start.elapsed();
    eprintln!("Rendered in {:.2}s", elapsed.as_secs_f64());

    // Save as PNG
    let out = output_path.unwrap_or_else(|| PathBuf::from("output.png"));
    image::save_buffer(
        &out,
        &pixels,
        render_config.width,
        render_config.height,
        image::ColorType::Rgba8,
    )
    .unwrap_or_else(|e| panic!("Failed to save image: {}", e));

    eprintln!("Saved to {}", out.display());
}

fn parse_args(args: &[String]) -> (Option<String>, Option<PathBuf>) {
    let mut scene = None;
    let mut output = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output = Some(PathBuf::from(&args[i]));
                }
            }
            "-h" | "--help" => {
                eprintln!("Usage: luminara [scene.toml] [-o output.png]");
                eprintln!();
                eprintln!("  scene.toml    Scene description file (optional, uses demo scene if omitted)");
                eprintln!("  -o, --output  Output file path (default: output.png)");
                std::process::exit(0);
            }
            other => {
                scene = Some(other.to_string());
            }
        }
        i += 1;
    }

    (scene, output)
}
