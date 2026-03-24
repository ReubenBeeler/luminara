mod aabb;
mod bvh;
mod camera;
mod capsule;
mod cone;
mod constant_medium;
mod cylinder;
mod disk;
mod ellipsoid;
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

struct CliArgs {
    scene: Option<String>,
    output: Option<PathBuf>,
    width: Option<u32>,
    height: Option<u32>,
    samples: Option<u32>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cli = parse_args(&args);

    let start = Instant::now();

    let (mut render_config, camera, world) = match &cli.scene {
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

    // CLI overrides
    if let Some(w) = cli.width {
        render_config.width = w;
    }
    if let Some(h) = cli.height {
        render_config.height = h;
    }
    if let Some(s) = cli.samples {
        render_config.samples_per_pixel = s;
    }

    eprintln!(
        "Rendering {}x{} @ {} spp, max depth {}",
        render_config.width, render_config.height, render_config.samples_per_pixel, render_config.max_depth
    );

    let pixels = render::render(&render_config, &camera, &world);

    let elapsed = start.elapsed();
    eprintln!("Rendered in {:.2}s", elapsed.as_secs_f64());

    // Save as PNG
    let out = cli.output.unwrap_or_else(|| PathBuf::from("output.png"));
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

fn parse_args(args: &[String]) -> CliArgs {
    let mut cli = CliArgs {
        scene: None,
        output: None,
        width: None,
        height: None,
        samples: None,
    };
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    cli.output = Some(PathBuf::from(&args[i]));
                }
            }
            "-w" | "--width" => {
                i += 1;
                if i < args.len() {
                    cli.width = args[i].parse().ok();
                }
            }
            "--height" => {
                i += 1;
                if i < args.len() {
                    cli.height = args[i].parse().ok();
                }
            }
            "-s" | "--samples" => {
                i += 1;
                if i < args.len() {
                    cli.samples = args[i].parse().ok();
                }
            }
            "-h" | "--help" => {
                eprintln!("Usage: luminara [scene.toml] [options]");
                eprintln!();
                eprintln!("  scene.toml        Scene description file (optional, uses demo scene if omitted)");
                eprintln!("  -o, --output      Output file path (default: output.png)");
                eprintln!("  -w, --width       Override render width");
                eprintln!("      --height      Override render height");
                eprintln!("  -s, --samples     Override samples per pixel");
                eprintln!("  -h, --help        Show this help");
                std::process::exit(0);
            }
            other => {
                cli.scene = Some(other.to_string());
            }
        }
        i += 1;
    }

    cli
}
