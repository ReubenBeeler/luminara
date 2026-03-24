mod aabb;
mod bump;
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
mod transform;
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
    max_depth: Option<u32>,
    threads: Option<usize>,
    seed: Option<u64>,
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
    if let Some(d) = cli.max_depth {
        render_config.max_depth = d;
    }
    if let Some(s) = cli.seed {
        render_config.seed = s;
    }
    if let Some(t) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(t)
            .build_global()
            .ok();
    }

    // Validate render config
    if render_config.width == 0 || render_config.height == 0 {
        eprintln!("Error: width and height must be > 0");
        std::process::exit(1);
    }
    if render_config.samples_per_pixel == 0 {
        eprintln!("Error: samples must be > 0");
        std::process::exit(1);
    }

    eprintln!(
        "Rendering {}x{} @ {} spp, max depth {}",
        render_config.width, render_config.height, render_config.samples_per_pixel, render_config.max_depth
    );

    let pixels = render::render(&render_config, &camera, &world);

    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    let sqrt_spp = (render_config.samples_per_pixel as f64).sqrt().ceil() as u64;
    let total_rays = render_config.width as u64 * render_config.height as u64 * sqrt_spp * sqrt_spp;
    let mrays_per_sec = total_rays as f64 / secs / 1_000_000.0;
    eprintln!("Rendered in {secs:.2}s ({mrays_per_sec:.1} Mrays/s)");

    // Save output
    let out = cli.output.unwrap_or_else(|| PathBuf::from("output.png"));
    let out_str = out.to_string_lossy();

    if out_str == "-" || out_str.ends_with(".ppm") {
        // Write PPM to stdout or file
        use std::io::Write;
        let w = render_config.width;
        let h = render_config.height;
        let mut ppm = format!("P3\n{w} {h}\n255\n");
        for chunk in pixels.chunks(4) {
            ppm.push_str(&format!("{} {} {}\n", chunk[0], chunk[1], chunk[2]));
        }
        if out_str == "-" {
            std::io::stdout().write_all(ppm.as_bytes()).unwrap();
            eprintln!("Written to stdout (PPM)");
        } else {
            std::fs::write(&out, ppm).unwrap_or_else(|e| panic!("Failed to write: {e}"));
            eprintln!("Saved to {}", out.display());
        }
    } else {
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
}

fn parse_args(args: &[String]) -> CliArgs {
    let mut cli = CliArgs {
        scene: None,
        output: None,
        width: None,
        height: None,
        samples: None,
        max_depth: None,
        threads: None,
        seed: None,
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
            "--seed" => {
                i += 1;
                if i < args.len() {
                    cli.seed = args[i].parse().ok();
                }
            }
            "-t" | "--threads" => {
                i += 1;
                if i < args.len() {
                    cli.threads = args[i].parse().ok();
                }
            }
            "-d" | "--depth" => {
                i += 1;
                if i < args.len() {
                    cli.max_depth = args[i].parse().ok();
                }
            }
            "-V" | "--version" => {
                eprintln!("Luminara {} — a physically-based ray tracer", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-h" | "--help" => {
                eprintln!("Usage: luminara [scene.toml] [options]");
                eprintln!();
                eprintln!("  scene.toml        Scene description file (optional, uses demo scene if omitted)");
                eprintln!("  -o, --output      Output file path (default: output.png)");
                eprintln!("  -w, --width       Override render width");
                eprintln!("      --height      Override render height");
                eprintln!("  -s, --samples     Override samples per pixel");
                eprintln!("  -d, --depth       Override max ray bounce depth");
                eprintln!("  -t, --threads     Number of render threads (default: all cores)");
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
