mod aabb;
mod annulus;
mod bump;
mod bvh;
mod camera;
mod capsule;
mod cone;
mod constant_medium;
mod csg;
mod cylinder;
mod disk;
mod ellipsoid;
mod hemisphere;
mod hit;
mod material;
mod mobius;
mod normal_map;
mod obj;
mod plane;
mod prism;
mod pyramid;
mod quad;
mod ray;
mod rect;
mod render;
mod rounded_box;
mod scene;
mod superellipsoid;
mod sphere;
mod spring;
mod texture;
mod torus;
mod transform;
mod triangle;
mod vec3;
mod wedge;

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
    exposure: Option<f64>,
    auto_exposure: bool,
    tone_map: Option<String>,
    preview: bool,
    quiet: bool,
    info_only: bool,
    benchmark: bool,
    list_scenes: bool,
    denoise: bool,
    save_hdr: Option<PathBuf>,
    crop: Option<String>,
    bloom: Option<f64>,
    vignette: Option<f64>,
    grain: Option<f64>,
    saturation: Option<f64>,
    contrast: Option<f64>,
    white_balance: Option<f64>,
    sharpen: Option<f64>,
    hue_shift: Option<f64>,
    dither: bool,
    gamma: Option<f64>,
    adaptive: bool,
    adaptive_threshold: Option<f64>,
    time_limit: Option<f64>,
    firefly_filter: Option<f64>,
    lens_distortion: Option<f64>,
    chromatic_aberration: Option<f64>,
    pixel_filter: Option<String>,
    save_depth: Option<PathBuf>,
    save_normals: Option<PathBuf>,
    save_albedo: Option<PathBuf>,
    posterize: Option<u32>,
    sepia: Option<f64>,
    edge_detect: Option<f64>,
    pixelate: Option<u32>,
    invert: bool,
    scanlines: Option<f64>,
    threshold: Option<f64>,
    blur: Option<f64>,
    tilt_shift: Option<f64>,
    halftone: Option<u32>,
    emboss: Option<f64>,
    oil_paint: Option<u32>,
    color_map: Option<String>,
    solarize: Option<f64>,
    duo_tone: Option<String>,
    sketch: bool,
    median: Option<u32>,
    crosshatch: Option<u32>,
    glitch: Option<f64>,
    depth_fog: Option<f64>,
    fog_color: Option<[f64; 3]>,
    channel_swap: Option<String>,
    mirror: Option<String>,
    quantize: Option<u32>,
    tint: Option<[f64; 3]>,
    palette: Option<String>,
    ascii: bool,
    tri_tone: Option<String>,
    gradient_map: Option<String>,
    split_tone: Option<String>,
    color_shift: Option<u32>,
    posterize_channels: Option<[u32; 3]>,
    lens_flare: Option<f64>,
    cel_shade: Option<u32>,
    frame: Option<u32>,
    hex_pixelate: Option<u32>,
    pencil: Option<u32>,
    dot_matrix: Option<u32>,
    noise_overlay: Option<f64>,
    color_halftone: Option<u32>,
    kaleidoscope: Option<u32>,
    pop_art: Option<u32>,
    watercolor: Option<u32>,
    auto_levels: bool,
    brightness: Option<f64>,
    color_balance: Option<[f64; 3]>,
    stipple: Option<u32>,
    night_vision: bool,
    fisheye: Option<f64>,
    wave: Option<f64>,
    swirl: Option<f64>,
    mosaic: Option<u32>,
    radial_blur: Option<f64>,
    border: Option<u32>,
    border_color: Option<[f64; 3]>,
    resize: Option<[u32; 2]>,
    warm: bool,
    cool: bool,
    rotate: Option<u32>,
    panorama: bool,
    save_json: Option<PathBuf>,
    vintage: bool,
    cinematic: bool,
    retro: bool,
    dreamy: bool,
    noir: bool,
    miniature: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cli = parse_args(&args);

    // List available scenes
    if cli.list_scenes {
        let scene_dir = std::path::Path::new("scenes");
        if scene_dir.is_dir() {
            let mut scenes: Vec<String> = std::fs::read_dir(scene_dir)
                .unwrap_or_else(|_| { eprintln!("Cannot read scenes/ directory"); std::process::exit(1); })
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
                .map(|e| e.path().display().to_string())
                .collect();
            scenes.sort();
            eprintln!("Available scenes ({}):", scenes.len());
            for s in &scenes {
                eprintln!("  {s}");
            }
        } else {
            eprintln!("No scenes/ directory found");
        }
        std::process::exit(0);
    }

    let start = Instant::now();

    let (mut render_config, mut camera, world) = match &cli.scene {
        Some(path) => {
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: cannot read scene file '{}': {}", path, e);
                    std::process::exit(1);
                }
            };
            match scene::load_scene(&content) {
                Ok(scene) => scene,
                Err(e) => {
                    eprintln!("Error: invalid scene '{}': {}", path, e);
                    std::process::exit(1);
                }
            }
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
    if let Some(e) = cli.exposure {
        render_config.exposure = e;
    }
    render_config.quiet = cli.quiet;
    if cli.preview {
        // Preview mode: 1/4 resolution, 4 samples, reduced depth
        render_config.width = (render_config.width / 4).max(1);
        render_config.height = (render_config.height / 4).max(1);
        render_config.samples_per_pixel = render_config.samples_per_pixel.min(4);
        render_config.max_depth = render_config.max_depth.min(10);
    }
    if cli.auto_exposure {
        render_config.auto_exposure = true;
    }
    if cli.denoise {
        render_config.denoise = true;
    }
    if let Some(bloom) = cli.bloom {
        render_config.bloom = bloom;
    }
    if let Some(vignette) = cli.vignette {
        render_config.vignette = vignette;
    }
    if let Some(grain) = cli.grain {
        render_config.grain = grain;
    }
    if let Some(saturation) = cli.saturation {
        render_config.saturation = saturation;
    }
    if let Some(contrast) = cli.contrast {
        render_config.contrast = contrast;
    }
    if let Some(wb) = cli.white_balance {
        render_config.white_balance = wb;
    }
    if let Some(sharpen) = cli.sharpen {
        render_config.sharpen = sharpen;
    }
    if let Some(hue_shift) = cli.hue_shift {
        render_config.hue_shift = hue_shift;
    }
    if cli.dither {
        render_config.dither = true;
    }
    if let Some(ref pf) = cli.pixel_filter {
        render_config.pixel_filter = match pf.as_str() {
            "triangle" | "tent" => render::PixelFilter::Triangle,
            "gaussian" | "gauss" => render::PixelFilter::Gaussian,
            "mitchell" => render::PixelFilter::Mitchell,
            _ => render::PixelFilter::Box,
        };
    }
    if let Some(ld) = cli.lens_distortion {
        render_config.lens_distortion = ld;
    }
    if let Some(ff) = cli.firefly_filter {
        render_config.firefly_filter = ff;
    }
    if let Some(ca) = cli.chromatic_aberration {
        render_config.chromatic_aberration = ca;
    }
    if let Some(p) = cli.posterize {
        render_config.posterize = p;
    }
    if let Some(s) = cli.sepia {
        render_config.sepia = s;
    }
    if let Some(ed) = cli.edge_detect {
        render_config.edge_detect = ed;
    }
    if let Some(px) = cli.pixelate {
        render_config.pixelate = px;
    }
    if cli.invert {
        render_config.invert = true;
    }
    if let Some(sl) = cli.scanlines {
        render_config.scanlines = sl;
    }
    if let Some(th) = cli.threshold {
        render_config.threshold = th;
    }
    if let Some(bl) = cli.blur {
        render_config.blur = bl;
    }
    if let Some(ts) = cli.tilt_shift {
        render_config.tilt_shift = ts;
    }
    if let Some(ht) = cli.halftone {
        render_config.halftone = ht;
    }
    if let Some(e) = cli.emboss {
        render_config.emboss = e;
    }
    if let Some(op) = cli.oil_paint {
        render_config.oil_paint = op;
    }
    if let Some(ref cm) = cli.color_map {
        render_config.color_map = cm.clone();
    }
    if let Some(s) = cli.solarize {
        render_config.solarize = s;
    }
    if let Some(ref dt) = cli.duo_tone {
        render_config.duo_tone = dt.clone();
    }
    if cli.sketch {
        render_config.sketch = true;
    }
    if let Some(m) = cli.median {
        render_config.median = m;
    }
    if let Some(ch) = cli.crosshatch {
        render_config.crosshatch = ch;
    }
    if let Some(gl) = cli.glitch {
        render_config.glitch = gl;
    }
    if let Some(df) = cli.depth_fog {
        render_config.depth_fog = df;
    }
    if let Some(fc) = cli.fog_color {
        render_config.depth_fog_color = fc;
    }
    if let Some(ref cs) = cli.channel_swap {
        render_config.channel_swap = cs.clone();
    }
    if let Some(ref m) = cli.mirror {
        render_config.mirror = m.clone();
    }
    if let Some(q) = cli.quantize {
        render_config.quantize = q;
    }
    if let Some(t) = cli.tint {
        render_config.tint = t;
    }
    if let Some(ref p) = cli.palette {
        render_config.palette = p.clone();
    }
    if let Some(ref tt) = cli.tri_tone {
        render_config.tri_tone = tt.clone();
    }
    if let Some(ref gm) = cli.gradient_map {
        render_config.gradient_map = gm.clone();
    }
    if let Some(ref st) = cli.split_tone {
        render_config.split_tone = st.clone();
    }
    if let Some(cs) = cli.color_shift {
        render_config.color_shift = cs;
    }
    if let Some(pc) = cli.posterize_channels {
        render_config.posterize_channels = pc;
    }
    if let Some(lf) = cli.lens_flare {
        render_config.lens_flare = lf;
    }
    if let Some(cs) = cli.cel_shade {
        render_config.cel_shade = cs;
    }
    if let Some(f) = cli.frame {
        render_config.frame = f;
    }
    if let Some(hp) = cli.hex_pixelate {
        render_config.hex_pixelate = hp;
    }
    if let Some(p) = cli.pencil {
        render_config.pencil = p;
    }
    if let Some(dm) = cli.dot_matrix {
        render_config.dot_matrix = dm;
    }
    if let Some(no) = cli.noise_overlay {
        render_config.noise_overlay = no;
    }
    if let Some(ch) = cli.color_halftone {
        render_config.color_halftone = ch;
    }
    if let Some(k) = cli.kaleidoscope {
        render_config.kaleidoscope = k;
    }
    if let Some(pa) = cli.pop_art {
        render_config.pop_art = pa;
    }
    if let Some(wc) = cli.watercolor {
        render_config.watercolor = wc;
    }
    if cli.auto_levels {
        render_config.auto_levels = true;
    }
    if let Some(br) = cli.brightness {
        render_config.brightness = br;
    }
    if let Some(cb) = cli.color_balance {
        render_config.color_balance = cb;
    }
    if let Some(st) = cli.stipple {
        render_config.stipple = st;
    }
    if cli.night_vision {
        render_config.night_vision = true;
    }
    if let Some(fe) = cli.fisheye {
        render_config.fisheye = fe;
    }
    if let Some(w) = cli.wave {
        render_config.wave = w;
    }
    if let Some(s) = cli.swirl {
        render_config.swirl = s;
    }
    if let Some(m) = cli.mosaic {
        render_config.mosaic = m;
    }
    if let Some(rb) = cli.radial_blur {
        render_config.radial_blur = rb;
    }
    if let Some(b) = cli.border {
        render_config.border = b;
    }
    if let Some(bc) = cli.border_color {
        render_config.border_color = bc;
    }
    if let Some(r) = cli.resize {
        render_config.resize = r;
    }
    if cli.warm {
        render_config.tint = [1.0, 0.92, 0.82];
    }
    if cli.cool {
        render_config.tint = [0.85, 0.92, 1.0];
    }
    if let Some(r) = cli.rotate {
        render_config.rotate = r;
    }
    if cli.panorama {
        camera.set_panorama(true);
    }
    // Presets
    if cli.vintage {
        render_config.sepia = 0.6;
        render_config.grain = 0.08;
        render_config.vignette = 0.4;
        render_config.contrast = 1.1;
        render_config.saturation = 0.7;
    }
    if cli.cinematic {
        render_config.bloom = 0.06;
        render_config.vignette = 0.3;
        render_config.contrast = 1.15;
        render_config.tint = [1.0, 0.95, 0.88];
        render_config.grain = 0.02;
    }
    if cli.retro {
        render_config.scanlines = 0.3;
        render_config.pixelate = 3;
        render_config.vignette = 0.5;
        render_config.grain = 0.06;
        render_config.chromatic_aberration = 2.0;
        render_config.saturation = 0.8;
    }
    if cli.dreamy {
        render_config.bloom = 0.12;
        render_config.blur = 0.8;
        render_config.saturation = 1.2;
        render_config.brightness = 0.05;
        render_config.tint = [1.0, 0.98, 1.05]; // slight purple tint
        render_config.vignette = 0.2;
    }
    if cli.noir {
        render_config.saturation = 0.0; // full desaturation
        render_config.contrast = 1.4;
        render_config.vignette = 0.6;
        render_config.grain = 0.05;
        render_config.sharpen = 0.3;
    }
    if cli.miniature {
        render_config.tilt_shift = 2.0;
        render_config.saturation = 1.4;
        render_config.contrast = 1.2;
        render_config.vignette = 0.15;
    }
    if cli.save_depth.is_some() {
        render_config.save_depth = true;
    }
    if cli.save_normals.is_some() {
        render_config.save_normals = true;
    }
    if cli.save_albedo.is_some() {
        render_config.save_albedo = true;
    }
    if cli.adaptive {
        render_config.adaptive = true;
    }
    if let Some(tl) = cli.time_limit {
        render_config.time_limit = tl;
    }
    if let Some(threshold) = cli.adaptive_threshold {
        render_config.adaptive_threshold = threshold;
    }
    if let Some(gamma) = cli.gamma {
        render_config.gamma = gamma;
    }
    if cli.save_hdr.is_some() {
        render_config.save_hdr = true;
    }
    if let Some(ref crop_str) = cli.crop {
        match parse_crop(crop_str) {
            Some(crop) => render_config.crop = Some(crop),
            None => {
                eprintln!("Error: invalid crop format '{}', expected X,Y,W,H", crop_str);
                std::process::exit(1);
            }
        }
    }
    if let Some(ref tm) = cli.tone_map {
        render_config.tone_map = match tm.as_str() {
            "aces" => render::ToneMap::Aces,
            "reinhard" => render::ToneMap::Reinhard,
            "filmic" | "uncharted2" => render::ToneMap::Filmic,
            "none" => render::ToneMap::None,
            other => {
                eprintln!("Warning: unknown tone map '{other}', using ACES");
                render::ToneMap::Aces
            }
        };
    }
    if let Some(t) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(t)
            .build_global()
            .ok();
    }

    if cli.info_only {
        let sqrt_spp = (render_config.samples_per_pixel as f64).sqrt().ceil() as u64;
        let actual_spp = sqrt_spp * sqrt_spp;
        let total_rays = render_config.width as u64 * render_config.height as u64 * actual_spp;
        eprintln!("Scene: {}", cli.scene.as_deref().unwrap_or("demo"));
        eprintln!("Resolution: {}x{}", render_config.width, render_config.height);
        eprintln!("Samples: {} ({}x{} stratified = {})", render_config.samples_per_pixel, sqrt_spp, sqrt_spp, actual_spp);
        eprintln!("Max depth: {}", render_config.max_depth);
        eprintln!("Total primary rays: {}", total_rays);
        eprintln!("Objects: {} ({} bounded/BVH, {} unbounded)", world.object_count(), world.bounded_count, world.unbounded_count);
        eprintln!("Lights: {}", world.lights.len());
        eprintln!("Seed: {}", render_config.seed);
        eprintln!("Exposure: {}{}", render_config.exposure, if render_config.auto_exposure { " (auto)" } else { "" });
        let tm_name = match render_config.tone_map {
            render::ToneMap::Aces => "ACES",
            render::ToneMap::Reinhard => "Reinhard",
            render::ToneMap::Filmic => "Filmic (Uncharted 2)",
            render::ToneMap::None => "None",
        };
        eprintln!("Tone mapping: {tm_name}");

        // Post-processing pipeline summary
        let mut pp = Vec::new();
        if render_config.denoise { pp.push("denoise".to_string()); }
        if render_config.bloom > 0.0 { pp.push(format!("bloom({:.2})", render_config.bloom)); }
        if render_config.sharpen > 0.0 { pp.push(format!("sharpen({:.2})", render_config.sharpen)); }
        if render_config.firefly_filter > 0.0 { pp.push(format!("firefly({:.1})", render_config.firefly_filter)); }
        if render_config.chromatic_aberration > 0.0 { pp.push(format!("ca({:.3})", render_config.chromatic_aberration)); }
        if render_config.vignette > 0.0 { pp.push(format!("vignette({:.2})", render_config.vignette)); }
        if render_config.white_balance.abs() > 1e-6 { pp.push(format!("wb({:.1})", render_config.white_balance)); }
        if render_config.hue_shift.abs() > 1e-6 { pp.push(format!("hue({:.0}°)", render_config.hue_shift)); }
        if (render_config.saturation - 1.0).abs() > 1e-6 { pp.push(format!("sat({:.2})", render_config.saturation)); }
        if (render_config.contrast - 1.0).abs() > 1e-6 { pp.push(format!("contrast({:.2})", render_config.contrast)); }
        if render_config.grain > 0.0 { pp.push(format!("grain({:.2})", render_config.grain)); }
        if render_config.dither { pp.push("dither".to_string()); }
        if render_config.gamma > 0.0 { pp.push(format!("gamma({:.1})", render_config.gamma)); }
        if render_config.adaptive { pp.push(format!("adaptive(thr={:.3})", render_config.adaptive_threshold)); }
        if render_config.lens_distortion.abs() > 1e-6 { pp.push(format!("lens({:.3})", render_config.lens_distortion)); }
        if render_config.blur > 0.0 { pp.push(format!("blur({:.1})", render_config.blur)); }
        if render_config.tilt_shift > 0.0 { pp.push(format!("tilt-shift({:.1})", render_config.tilt_shift)); }
        if render_config.pixelate >= 2 { pp.push(format!("pixelate({})", render_config.pixelate)); }
        if render_config.edge_detect > 0.0 { pp.push(format!("edges({:.1})", render_config.edge_detect)); }
        if render_config.emboss > 0.0 { pp.push(format!("emboss({:.1})", render_config.emboss)); }
        if render_config.oil_paint > 0 { pp.push(format!("oil-paint({})", render_config.oil_paint)); }
        if !render_config.color_map.is_empty() { pp.push(format!("color-map({})", render_config.color_map)); }
        if render_config.solarize >= 0.0 { pp.push(format!("solarize({:.2})", render_config.solarize)); }
        if !render_config.duo_tone.is_empty() { pp.push("duo-tone".to_string()); }
        if render_config.sketch { pp.push("sketch".to_string()); }
        if render_config.median > 0 { pp.push(format!("median({})", render_config.median)); }
        if render_config.crosshatch > 0 { pp.push(format!("crosshatch({})", render_config.crosshatch)); }
        if render_config.glitch > 0.0 { pp.push(format!("glitch({:.1})", render_config.glitch)); }
        if render_config.depth_fog > 0.0 { pp.push(format!("depth-fog({:.2})", render_config.depth_fog)); }
        if !render_config.channel_swap.is_empty() { pp.push(format!("channel-swap({})", render_config.channel_swap)); }
        if !render_config.mirror.is_empty() { pp.push(format!("mirror({})", render_config.mirror)); }
        if render_config.quantize >= 2 { pp.push(format!("quantize({})", render_config.quantize)); }
        if render_config.tint[0] < 1.0 || render_config.tint[1] < 1.0 || render_config.tint[2] < 1.0 {
            pp.push(format!("tint({:.2},{:.2},{:.2})", render_config.tint[0], render_config.tint[1], render_config.tint[2]));
        }
        if !render_config.palette.is_empty() { pp.push(format!("palette({})", render_config.palette)); }
        if !render_config.tri_tone.is_empty() { pp.push("tri-tone".to_string()); }
        if !render_config.gradient_map.is_empty() { pp.push("gradient-map".to_string()); }
        if !render_config.split_tone.is_empty() { pp.push("split-tone".to_string()); }
        if render_config.color_shift > 0 { pp.push(format!("color-shift({})", render_config.color_shift)); }
        if render_config.lens_flare > 0.0 { pp.push(format!("lens-flare({:.2})", render_config.lens_flare)); }
        if render_config.cel_shade >= 2 { pp.push(format!("cel-shade({})", render_config.cel_shade)); }
        if render_config.frame > 0 { pp.push(format!("frame({}px)", render_config.frame)); }
        if render_config.hex_pixelate >= 2 { pp.push(format!("hex-pixelate({})", render_config.hex_pixelate)); }
        if render_config.pencil > 0 { pp.push(format!("pencil({})", render_config.pencil)); }
        if render_config.dot_matrix >= 2 { pp.push(format!("dot-matrix({})", render_config.dot_matrix)); }
        if render_config.noise_overlay > 0.0 { pp.push(format!("noise({:.2})", render_config.noise_overlay)); }
        if render_config.color_halftone >= 2 { pp.push(format!("color-halftone({})", render_config.color_halftone)); }
        if render_config.kaleidoscope >= 2 { pp.push(format!("kaleidoscope({})", render_config.kaleidoscope)); }
        if render_config.posterize_channels.iter().any(|&l| l >= 2) {
            pp.push(format!("posterize-ch({},{},{})", render_config.posterize_channels[0], render_config.posterize_channels[1], render_config.posterize_channels[2]));
        }
        if render_config.pop_art >= 2 { pp.push(format!("pop-art({})", render_config.pop_art)); }
        if render_config.watercolor > 0 { pp.push(format!("watercolor({})", render_config.watercolor)); }
        if render_config.auto_levels { pp.push("auto-levels".to_string()); }
        if render_config.brightness.abs() > 1e-6 { pp.push(format!("brightness({:.2})", render_config.brightness)); }
        if (render_config.color_balance[0] - 1.0).abs() > 1e-6 || (render_config.color_balance[1] - 1.0).abs() > 1e-6 || (render_config.color_balance[2] - 1.0).abs() > 1e-6 {
            pp.push(format!("color-balance({:.2},{:.2},{:.2})", render_config.color_balance[0], render_config.color_balance[1], render_config.color_balance[2]));
        }
        if render_config.stipple > 0 { pp.push(format!("stipple({})", render_config.stipple)); }
        if render_config.night_vision { pp.push("night-vision".to_string()); }
        if render_config.fisheye.abs() > 1e-6 { pp.push(format!("fisheye({:.2})", render_config.fisheye)); }
        if render_config.wave > 0.0 { pp.push(format!("wave({:.0})", render_config.wave)); }
        if render_config.swirl.abs() > 1e-6 { pp.push(format!("swirl({:.1})", render_config.swirl)); }
        if render_config.mosaic > 0 { pp.push(format!("mosaic({})", render_config.mosaic)); }
        if render_config.radial_blur > 0.0 { pp.push(format!("radial-blur({:.1})", render_config.radial_blur)); }
        if render_config.border > 0 { pp.push(format!("border({}px)", render_config.border)); }
        if render_config.resize[0] > 0 || render_config.resize[1] > 0 { pp.push(format!("resize({}x{})", render_config.resize[0], render_config.resize[1])); }
        if render_config.rotate > 0 { pp.push(format!("rotate({}°)", render_config.rotate)); }
        if render_config.posterize >= 2 { pp.push(format!("posterize({})", render_config.posterize)); }
        if render_config.sepia > 0.0 { pp.push(format!("sepia({:.1})", render_config.sepia)); }
        if render_config.threshold >= 0.0 { pp.push(format!("threshold({:.2})", render_config.threshold)); }
        if render_config.halftone >= 2 { pp.push(format!("halftone({})", render_config.halftone)); }
        if render_config.invert { pp.push("invert".to_string()); }
        if render_config.scanlines > 0.0 { pp.push(format!("scanlines({:.1})", render_config.scanlines)); }
        if render_config.grade_shadows != [1.0, 1.0, 1.0] { pp.push("grade-shadows".to_string()); }
        if render_config.grade_highlights != [1.0, 1.0, 1.0] { pp.push("grade-highlights".to_string()); }
        if pp.is_empty() {
            eprintln!("Post-processing: none");
        } else {
            eprintln!("Post-processing: {}", pp.join(" → "));
        }

        // Output files
        if render_config.save_hdr { eprintln!("HDR output: enabled"); }
        if render_config.save_depth { eprintln!("Depth pass: enabled"); }
        if render_config.save_normals { eprintln!("Normal pass: enabled"); }
        if render_config.save_albedo { eprintln!("Albedo pass: enabled"); }

        std::process::exit(0);
    }

    // Benchmark mode: run a standardized performance test
    if cli.benchmark {
        eprintln!("Benchmark: rendering demo scene at 400x225 @ 64 spp...");
        let (mut bench_config, bench_cam, bench_world) = scene::demo_scene();
        bench_config.width = 400;
        bench_config.height = 225;
        bench_config.samples_per_pixel = 64;
        bench_config.quiet = true;

        let bench_start = Instant::now();
        let _ = render::render(&bench_config, &bench_cam, &bench_world, &bench_world.lights);
        let bench_elapsed = bench_start.elapsed().as_secs_f64();

        let sqrt_spp = (bench_config.samples_per_pixel as f64).sqrt().ceil() as u64;
        let actual_spp = sqrt_spp * sqrt_spp;
        let total_rays = 400u64 * 225 * actual_spp;
        let mrays = total_rays as f64 / bench_elapsed / 1_000_000.0;
        let threads = rayon::current_num_threads();
        eprintln!("Benchmark result: {mrays:.1} Mrays/s ({bench_elapsed:.2}s, {threads} threads)");
        eprintln!("  {total_rays} primary rays, {actual_spp} spp (stratified {sqrt_spp}x{sqrt_spp})");
        std::process::exit(0);
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

    let num_threads = rayon::current_num_threads();
    let (out_w, out_h) = if let Some((cx, cy, cw, ch)) = render_config.crop {
        let cw = cw.min(render_config.width.saturating_sub(cx));
        let ch = ch.min(render_config.height.saturating_sub(cy));
        (cw, ch)
    } else {
        (render_config.width, render_config.height)
    };
    let total_pixels = out_w as u64 * out_h as u64;
    if let Some((cx, cy, _, _)) = render_config.crop {
        eprintln!(
            "Rendering crop {out_w}x{out_h} at ({cx},{cy}) from {}x{} @ {} spp, max depth {}, {} threads",
            render_config.width, render_config.height,
            render_config.samples_per_pixel, render_config.max_depth, num_threads
        );
    } else {
        eprintln!(
            "Rendering {}x{} ({:.1}MP) @ {} spp, max depth {}, {} threads",
            render_config.width, render_config.height,
            total_pixels as f64 / 1_000_000.0,
            render_config.samples_per_pixel, render_config.max_depth, num_threads
        );
    }

    let result = render::render(&render_config, &camera, &world, &world.lights);
    let pixels = result.pixels;

    // Adjust output dimensions for rotation
    let (out_w, out_h) = if render_config.rotate == 90 || render_config.rotate == 270 {
        (out_h, out_w)
    } else {
        (out_w, out_h)
    };
    // Adjust output dimensions if resize was applied
    let (out_w, out_h) = if render_config.resize[0] > 0 || render_config.resize[1] > 0 {
        let rw = if render_config.resize[0] > 0 { render_config.resize[0] } else { out_w };
        let rh = if render_config.resize[1] > 0 { render_config.resize[1] } else { out_h };
        (rw, rh)
    } else {
        (out_w, out_h)
    };
    // Adjust output dimensions for picture frame
    let (out_w, out_h) = if render_config.frame > 0 {
        (out_w + render_config.frame * 2, out_h + render_config.frame * 2)
    } else {
        (out_w, out_h)
    };

    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    let mrays_per_sec = result.total_rays as f64 / result.render_time_secs / 1_000_000.0;
    eprintln!("Rendered in {secs:.2}s (trace: {:.2}s, {mrays_per_sec:.1} Mrays/s, {} total rays)",
        result.render_time_secs, result.total_rays);

    // Save output
    let out = cli.output.unwrap_or_else(|| PathBuf::from("output.png"));
    let out_str = out.to_string_lossy();

    if out_str == "-" || out_str.ends_with(".ppm") {
        // Write PPM to stdout or file
        use std::io::Write;
        let mut ppm = format!("P3\n{out_w} {out_h}\n255\n");
        for chunk in pixels.chunks(4) {
            ppm.push_str(&format!("{} {} {}\n", chunk[0], chunk[1], chunk[2]));
        }
        if out_str == "-" {
            if let Err(e) = std::io::stdout().write_all(ppm.as_bytes()) {
                eprintln!("Error: failed to write to stdout: {e}");
                std::process::exit(1);
            }
            eprintln!("Written to stdout (PPM)");
        } else {
            if let Err(e) = std::fs::write(&out, ppm) {
                eprintln!("Error: failed to write '{}': {e}", out.display());
                std::process::exit(1);
            }
            eprintln!("Saved to {}", out.display());
        }
    } else {
        let is_jpeg = out_str.ends_with(".jpg") || out_str.ends_with(".jpeg");
        if is_jpeg {
            // JPEG doesn't support alpha — convert RGBA to RGB
            let rgb_pixels: Vec<u8> = pixels.chunks(4).flat_map(|c| &c[..3]).copied().collect();
            if let Err(e) = image::save_buffer(&out, &rgb_pixels, out_w, out_h, image::ColorType::Rgb8) {
                eprintln!("Error: failed to save image '{}': {e}", out.display());
                std::process::exit(1);
            }
        } else if let Err(e) = image::save_buffer(&out, &pixels, out_w, out_h, image::ColorType::Rgba8) {
            eprintln!("Error: failed to save image '{}': {e}", out.display());
            std::process::exit(1);
        }
        eprintln!("Saved to {}", out.display());
    }

    // ASCII art output
    if cli.ascii {
        const ASCII_CHARS: &[u8] = b" .:-=+*#%@";
        let term_width: usize = 80;
        let aspect = out_h as f64 / out_w as f64;
        // Terminal chars are ~2x taller than wide, so halve the height
        let term_height = ((term_width as f64 * aspect) * 0.5).round() as usize;
        let mut ascii_art = String::with_capacity((term_width + 1) * term_height);
        for row in 0..term_height {
            for col in 0..term_width {
                let px = (col as f64 / term_width as f64 * out_w as f64) as usize;
                let py = (row as f64 / term_height as f64 * out_h as f64) as usize;
                let idx = (py * out_w as usize + px) * 4;
                let r = pixels[idx] as f64;
                let g = pixels[idx + 1] as f64;
                let b = pixels[idx + 2] as f64;
                let lum = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0;
                let ci = (lum * (ASCII_CHARS.len() - 1) as f64).round() as usize;
                ascii_art.push(ASCII_CHARS[ci.min(ASCII_CHARS.len() - 1)] as char);
            }
            ascii_art.push('\n');
        }
        eprintln!("\n{ascii_art}");
    }

    // Save render statistics as JSON if requested
    if let Some(ref json_path) = cli.save_json {
        let json = format!(
            "{{\n  \"width\": {},\n  \"height\": {},\n  \"samples\": {},\n  \"max_depth\": {},\n  \"total_rays\": {},\n  \"render_time_secs\": {:.4},\n  \"total_time_secs\": {:.4},\n  \"mrays_per_sec\": {:.2},\n  \"threads\": {},\n  \"output\": \"{}\"\n}}",
            out_w, out_h, render_config.samples_per_pixel, render_config.max_depth,
            result.total_rays, result.render_time_secs, secs, mrays_per_sec,
            num_threads, out.display()
        );
        if let Err(e) = std::fs::write(json_path, json) {
            eprintln!("Warning: failed to write JSON stats '{}': {e}", json_path.display());
        } else {
            eprintln!("Stats saved to {}", json_path.display());
        }
    }

    // Save HDR data if requested
    // Save depth pass
    if let (Some(depth_path), Some(depth_data)) = (&cli.save_depth, &result.depth_pass) {
        // Normalize depth to [0, 255] using max depth
        let max_depth = depth_data.iter().cloned().fold(0.0f32, f32::max);
        let norm = if max_depth > 0.0 { 255.0 / max_depth } else { 1.0 };
        let depth_pixels: Vec<u8> = depth_data
            .iter()
            .flat_map(|d| {
                let v = (d * norm).clamp(0.0, 255.0) as u8;
                [v, v, v, 255]
            })
            .collect();
        match image::RgbaImage::from_raw(out_w, out_h, depth_pixels) {
            Some(img) => {
                if let Err(e) = img.save(depth_path) {
                    eprintln!("Error: failed to save depth pass '{}': {e}", depth_path.display());
                } else {
                    eprintln!("Saved depth pass to {}", depth_path.display());
                }
            }
            None => eprintln!("Error: depth pass buffer size mismatch"),
        }
    }

    // Save normal pass
    if let (Some(normal_path), Some(normal_data)) = (&cli.save_normals, &result.normal_pass) {
        let normal_pixels: Vec<u8> = normal_data
            .chunks(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect();
        match image::RgbaImage::from_raw(out_w, out_h, normal_pixels) {
            Some(img) => {
                if let Err(e) = img.save(normal_path) {
                    eprintln!("Error: failed to save normal pass '{}': {e}", normal_path.display());
                } else {
                    eprintln!("Saved normal pass to {}", normal_path.display());
                }
            }
            None => eprintln!("Error: normal pass buffer size mismatch"),
        }
    }

    // Save albedo pass
    if let (Some(albedo_path), Some(albedo_data)) = (&cli.save_albedo, &result.albedo_pass) {
        let albedo_pixels: Vec<u8> = albedo_data
            .chunks(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect();
        match image::RgbaImage::from_raw(out_w, out_h, albedo_pixels) {
            Some(img) => {
                if let Err(e) = img.save(albedo_path) {
                    eprintln!("Error: failed to save albedo pass '{}': {e}", albedo_path.display());
                } else {
                    eprintln!("Saved albedo pass to {}", albedo_path.display());
                }
            }
            None => eprintln!("Error: albedo pass buffer size mismatch"),
        }
    }

    if let (Some(hdr_path), Some(hdr_data)) = (&cli.save_hdr, &result.hdr_data) {
        let ext = hdr_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "exr" => {
                // Use image crate's EXR writer
                let rgb32f = image::Rgb32FImage::from_raw(out_w, out_h, hdr_data.clone());
                match rgb32f {
                    Some(img) => {
                        if let Err(e) = img.save(hdr_path) {
                            eprintln!("Error: failed to save EXR '{}': {e}", hdr_path.display());
                            std::process::exit(1);
                        }
                        eprintln!("Saved EXR to {}", hdr_path.display());
                    }
                    None => {
                        eprintln!("Error: HDR data size mismatch for EXR output");
                        std::process::exit(1);
                    }
                }
            }
            _ => {
                // Default: Radiance HDR format
                if let Err(e) = write_radiance_hdr(hdr_path, out_w, out_h, hdr_data) {
                    eprintln!("Error: failed to save HDR '{}': {e}", hdr_path.display());
                    std::process::exit(1);
                }
                eprintln!("Saved HDR to {}", hdr_path.display());
            }
        }
    }
}

/// Write HDR data in Radiance RGBE (.hdr) format.
fn write_radiance_hdr(
    path: &std::path::Path,
    width: u32,
    height: u32,
    data: &[f32],
) -> Result<(), String> {
    use std::io::Write;

    let mut buf = Vec::new();
    // Radiance header
    write!(buf, "#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y {} +X {}\n", height, width)
        .map_err(|e| e.to_string())?;

    // Validate data length
    let expected_len = width as usize * height as usize * 3;
    if data.len() < expected_len {
        return Err(format!("HDR data too short: {} < {}", data.len(), expected_len));
    }

    // Convert each pixel to RGBE encoding
    for i in 0..(width as usize * height as usize) {
        let r = data[i * 3].max(0.0);
        let g = data[i * 3 + 1].max(0.0);
        let b = data[i * 3 + 2].max(0.0);

        let max_val = r.max(g).max(b);
        if max_val < 1e-32 {
            buf.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            // frexp equivalent: find exponent such that max_val = mantissa * 2^exp
            let exp = max_val.log2().ceil() as i32;
            let scale = 256.0 / 2.0_f32.powi(exp);
            let re = (r * scale).min(255.0) as u8;
            let ge = (g * scale).min(255.0) as u8;
            let be = (b * scale).min(255.0) as u8;
            let ee = (exp + 128) as u8;
            buf.extend_from_slice(&[re, ge, be, ee]);
        }
    }

    std::fs::write(path, &buf).map_err(|e| e.to_string())
}

fn parse_crop(s: &str) -> Option<(u32, u32, u32, u32)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return None;
    }
    let x = parts[0].trim().parse().ok()?;
    let y = parts[1].trim().parse().ok()?;
    let w = parts[2].trim().parse().ok()?;
    let h = parts[3].trim().parse().ok()?;
    Some((x, y, w, h))
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
        exposure: None,
        auto_exposure: false,
        tone_map: None,
        preview: false,
        quiet: false,
        info_only: false,
        benchmark: false,
        list_scenes: false,
        denoise: false,
        save_hdr: None,
        crop: None,
        bloom: None,
        vignette: None,
        grain: None,
        saturation: None,
        contrast: None,
        white_balance: None,
        sharpen: None,
        hue_shift: None,
        dither: false,
        gamma: None,
        adaptive: false,
        adaptive_threshold: None,
        time_limit: None,
        firefly_filter: None,
        lens_distortion: None,
        chromatic_aberration: None,
        pixel_filter: None,
        save_depth: None,
        save_normals: None,
        save_albedo: None,
        posterize: None,
        sepia: None,
        edge_detect: None,
        pixelate: None,
        invert: false,
        scanlines: None,
        threshold: None,
        blur: None,
        tilt_shift: None,
        halftone: None,
        emboss: None,
        oil_paint: None,
        color_map: None,
        solarize: None,
        duo_tone: None,
        sketch: false,
        median: None,
        crosshatch: None,
        glitch: None,
        depth_fog: None,
        fog_color: None,
        channel_swap: None,
        mirror: None,
        quantize: None,
        tint: None,
        palette: None,
        ascii: false,
        tri_tone: None,
        gradient_map: None,
        split_tone: None,
        color_shift: None,
        posterize_channels: None,
        lens_flare: None,
        cel_shade: None,
        frame: None,
        hex_pixelate: None,
        pencil: None,
        dot_matrix: None,
        noise_overlay: None,
        color_halftone: None,
        kaleidoscope: None,
        pop_art: None,
        watercolor: None,
        auto_levels: false,
        brightness: None,
        color_balance: None,
        stipple: None,
        night_vision: false,
        fisheye: None,
        wave: None,
        swirl: None,
        mosaic: None,
        radial_blur: None,
        border: None,
        border_color: None,
        resize: None,
        warm: false,
        cool: false,
        rotate: None,
        panorama: false,
        save_json: None,
        vintage: false,
        cinematic: false,
        retro: false,
        dreamy: false,
        noir: false,
        miniature: false,
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
            "-H" | "--height" => {
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
            "--auto-exposure" => {
                cli.auto_exposure = true;
            }
            "-p" | "--preview" => {
                cli.preview = true;
            }
            "--tone-map" => {
                i += 1;
                if i < args.len() {
                    cli.tone_map = Some(args[i].clone());
                }
            }
            "-q" | "--quiet" => {
                cli.quiet = true;
            }
            "--denoise" => {
                cli.denoise = true;
            }
            "--bloom" => {
                i += 1;
                if i < args.len() {
                    cli.bloom = args[i].parse().ok();
                }
            }
            "--vignette" => {
                i += 1;
                if i < args.len() {
                    cli.vignette = args[i].parse().ok();
                }
            }
            "--grain" => {
                i += 1;
                if i < args.len() {
                    cli.grain = args[i].parse().ok();
                }
            }
            "--saturation" => {
                i += 1;
                if i < args.len() {
                    cli.saturation = args[i].parse().ok();
                }
            }
            "--contrast" => {
                i += 1;
                if i < args.len() {
                    cli.contrast = args[i].parse().ok();
                }
            }
            "--white-balance" | "--wb" => {
                i += 1;
                if i < args.len() {
                    cli.white_balance = args[i].parse().ok();
                }
            }
            "--sharpen" => {
                i += 1;
                if i < args.len() {
                    cli.sharpen = args[i].parse().ok();
                }
            }
            "--hue-shift" => {
                i += 1;
                if i < args.len() {
                    cli.hue_shift = args[i].parse().ok();
                }
            }
            "--dither" => {
                cli.dither = true;
            }
            "--filter" | "--pixel-filter" => {
                i += 1;
                if i < args.len() {
                    cli.pixel_filter = Some(args[i].clone());
                }
            }
            "--save-depth" => {
                i += 1;
                if i < args.len() {
                    cli.save_depth = Some(PathBuf::from(&args[i]));
                }
            }
            "--save-normals" => {
                i += 1;
                if i < args.len() {
                    cli.save_normals = Some(PathBuf::from(&args[i]));
                }
            }
            "--save-albedo" => {
                i += 1;
                if i < args.len() {
                    cli.save_albedo = Some(PathBuf::from(&args[i]));
                }
            }
            "--lens-distortion" | "--distortion" => {
                i += 1;
                if i < args.len() {
                    cli.lens_distortion = args[i].parse().ok();
                }
            }
            "--firefly-filter" | "--firefly" => {
                i += 1;
                if i < args.len() {
                    cli.firefly_filter = args[i].parse().ok();
                }
            }
            "--chromatic-aberration" | "--ca" => {
                i += 1;
                if i < args.len() {
                    cli.chromatic_aberration = args[i].parse().ok();
                }
            }
            "--adaptive" => {
                cli.adaptive = true;
            }
            "--time-limit" | "--max-time" => {
                i += 1;
                if i < args.len() {
                    cli.time_limit = args[i].parse().ok();
                }
            }
            "--adaptive-threshold" => {
                i += 1;
                if i < args.len() {
                    cli.adaptive_threshold = args[i].parse().ok();
                }
            }
            "--gamma" => {
                i += 1;
                if i < args.len() {
                    cli.gamma = args[i].parse().ok();
                }
            }
            "--save-hdr" => {
                i += 1;
                if i < args.len() {
                    cli.save_hdr = Some(PathBuf::from(&args[i]));
                }
            }
            "--crop" => {
                i += 1;
                if i < args.len() {
                    cli.crop = Some(args[i].clone());
                }
            }
            "--posterize" => {
                i += 1;
                if i < args.len() {
                    cli.posterize = args[i].parse().ok();
                }
            }
            "--sepia" => {
                i += 1;
                if i < args.len() {
                    cli.sepia = args[i].parse().ok();
                }
            }
            "--edge-detect" | "--outline" => {
                i += 1;
                if i < args.len() {
                    cli.edge_detect = args[i].parse().ok();
                }
            }
            "--pixelate" => {
                i += 1;
                if i < args.len() {
                    cli.pixelate = args[i].parse().ok();
                }
            }
            "--invert" => {
                cli.invert = true;
            }
            "--scanlines" => {
                i += 1;
                if i < args.len() {
                    cli.scanlines = args[i].parse().ok();
                }
            }
            "--threshold" | "--bw" => {
                i += 1;
                if i < args.len() {
                    cli.threshold = args[i].parse().ok();
                }
            }
            "--blur" => {
                i += 1;
                if i < args.len() {
                    cli.blur = args[i].parse().ok();
                }
            }
            "--tilt-shift" => {
                i += 1;
                if i < args.len() {
                    cli.tilt_shift = args[i].parse().ok();
                }
            }
            "--halftone" => {
                i += 1;
                if i < args.len() {
                    cli.halftone = args[i].parse().ok();
                }
            }
            "--emboss" => {
                i += 1;
                if i < args.len() {
                    cli.emboss = args[i].parse().ok();
                }
            }
            "--oil-paint" => {
                i += 1;
                if i < args.len() {
                    cli.oil_paint = args[i].parse().ok();
                }
            }
            "--color-map" | "--colormap" => {
                i += 1;
                if i < args.len() {
                    cli.color_map = Some(args[i].clone());
                }
            }
            "--solarize" => {
                i += 1;
                if i < args.len() {
                    cli.solarize = args[i].parse().ok();
                }
            }
            "--duo-tone" | "--duotone" => {
                i += 1;
                if i < args.len() {
                    cli.duo_tone = Some(args[i].clone());
                }
            }
            "--sketch" => {
                cli.sketch = true;
            }
            "--median" => {
                i += 1;
                if i < args.len() {
                    cli.median = args[i].parse().ok();
                }
            }
            "--crosshatch" => {
                i += 1;
                if i < args.len() {
                    cli.crosshatch = args[i].parse().ok();
                }
            }
            "--glitch" => {
                i += 1;
                if i < args.len() {
                    cli.glitch = args[i].parse().ok();
                }
            }
            "--fog-color" => {
                i += 1;
                if i < args.len() {
                    let parts: Vec<f64> = args[i].split(',').filter_map(|s| s.trim().parse().ok()).collect();
                    if parts.len() == 3 {
                        cli.fog_color = Some([parts[0], parts[1], parts[2]]);
                    }
                }
            }
            "--depth-fog" => {
                i += 1;
                if i < args.len() {
                    cli.depth_fog = args[i].parse().ok();
                }
            }
            "--channel-swap" => {
                i += 1;
                if i < args.len() {
                    cli.channel_swap = Some(args[i].clone());
                }
            }
            "--mirror" | "--flip" => {
                i += 1;
                if i < args.len() {
                    cli.mirror = Some(args[i].clone());
                }
            }
            "--quantize" => {
                i += 1;
                if i < args.len() {
                    cli.quantize = args[i].parse().ok();
                }
            }
            "--tint" => {
                i += 1;
                if i < args.len() {
                    let parts: Vec<f64> = args[i].split(',').filter_map(|s| s.trim().parse().ok()).collect();
                    if parts.len() == 3 {
                        cli.tint = Some([parts[0], parts[1], parts[2]]);
                    }
                }
            }
            "--palette" => {
                i += 1;
                if i < args.len() {
                    cli.palette = Some(args[i].clone());
                }
            }
            "--ascii" => {
                cli.ascii = true;
            }
            "--warm" => {
                cli.warm = true;
            }
            "--cool" => {
                cli.cool = true;
            }
            "--rotate" => {
                i += 1;
                if i < args.len() {
                    cli.rotate = args[i].parse().ok();
                }
            }
            "--panorama" | "--pano" => {
                cli.panorama = true;
            }
            "--vintage" => {
                cli.vintage = true;
            }
            "--retro" | "--crt" => {
                cli.retro = true;
            }
            "--dreamy" => {
                cli.dreamy = true;
            }
            "--noir" => {
                cli.noir = true;
            }
            "--miniature" => {
                cli.miniature = true;
            }
            "--cinematic" => {
                cli.cinematic = true;
            }
            "--save-json" => {
                i += 1;
                if i < args.len() {
                    cli.save_json = Some(PathBuf::from(&args[i]));
                }
            }
            "--tri-tone" => {
                i += 1;
                if i < args.len() {
                    cli.tri_tone = Some(args[i].clone());
                }
            }
            "--gradient-map" => {
                i += 1;
                if i < args.len() {
                    cli.gradient_map = Some(args[i].clone());
                }
            }
            "--split-tone" => {
                i += 1;
                if i < args.len() {
                    cli.split_tone = Some(args[i].clone());
                }
            }
            "--color-shift" => {
                i += 1;
                if i < args.len() {
                    cli.color_shift = args[i].parse().ok();
                }
            }
            "--posterize-channels" => {
                i += 1;
                if i < args.len() {
                    let parts: Vec<u32> = args[i].split(',').filter_map(|x| x.trim().parse().ok()).collect();
                    if parts.len() == 3 {
                        cli.posterize_channels = Some([parts[0], parts[1], parts[2]]);
                    }
                }
            }
            "--lens-flare" => {
                i += 1;
                if i < args.len() {
                    cli.lens_flare = args[i].parse().ok();
                }
            }
            "--cel-shade" | "--toon" => {
                i += 1;
                if i < args.len() {
                    cli.cel_shade = args[i].parse().ok();
                }
            }
            "--frame" => {
                i += 1;
                if i < args.len() {
                    cli.frame = args[i].parse().ok();
                }
            }
            "--hex-pixelate" | "--hex" => {
                i += 1;
                if i < args.len() {
                    cli.hex_pixelate = args[i].parse().ok();
                }
            }
            "--pencil" => {
                i += 1;
                if i < args.len() {
                    cli.pencil = args[i].parse().ok();
                }
            }
            "--dot-matrix" => {
                i += 1;
                if i < args.len() {
                    cli.dot_matrix = args[i].parse().ok();
                }
            }
            "--noise-overlay" | "--noise" => {
                i += 1;
                if i < args.len() {
                    cli.noise_overlay = args[i].parse().ok();
                }
            }
            "--color-halftone" => {
                i += 1;
                if i < args.len() {
                    cli.color_halftone = args[i].parse().ok();
                }
            }
            "--kaleidoscope" => {
                i += 1;
                if i < args.len() {
                    cli.kaleidoscope = args[i].parse().ok();
                }
            }
            "--pop-art" => {
                i += 1;
                if i < args.len() {
                    cli.pop_art = args[i].parse().ok();
                }
            }
            "--watercolor" => {
                i += 1;
                if i < args.len() {
                    cli.watercolor = args[i].parse().ok();
                }
            }
            "--auto-levels" => {
                cli.auto_levels = true;
            }
            "--brightness" => {
                i += 1;
                if i < args.len() {
                    cli.brightness = args[i].parse().ok();
                }
            }
            "--color-balance" => {
                i += 1;
                if i < args.len() {
                    let parts: Vec<f64> = args[i].split(',').filter_map(|s| s.trim().parse().ok()).collect();
                    if parts.len() == 3 {
                        cli.color_balance = Some([parts[0], parts[1], parts[2]]);
                    }
                }
            }
            "--stipple" => {
                i += 1;
                if i < args.len() {
                    cli.stipple = args[i].parse().ok();
                }
            }
            "--night-vision" | "--nv" => {
                cli.night_vision = true;
            }
            "--fisheye" => {
                i += 1;
                if i < args.len() {
                    cli.fisheye = args[i].parse().ok();
                }
            }
            "--wave" => {
                i += 1;
                if i < args.len() {
                    cli.wave = args[i].parse().ok();
                }
            }
            "--swirl" => {
                i += 1;
                if i < args.len() {
                    cli.swirl = args[i].parse().ok();
                }
            }
            "--mosaic" => {
                i += 1;
                if i < args.len() {
                    cli.mosaic = args[i].parse().ok();
                }
            }
            "--radial-blur" => {
                i += 1;
                if i < args.len() {
                    cli.radial_blur = args[i].parse().ok();
                }
            }
            "--border" => {
                i += 1;
                if i < args.len() {
                    cli.border = args[i].parse().ok();
                }
            }
            "--border-color" => {
                i += 1;
                if i < args.len() {
                    let parts: Vec<f64> = args[i].split(',').filter_map(|s| s.trim().parse().ok()).collect();
                    if parts.len() == 3 {
                        cli.border_color = Some([parts[0], parts[1], parts[2]]);
                    }
                }
            }
            "--resize" => {
                i += 1;
                if i < args.len() {
                    let parts: Vec<&str> = args[i].split('x').collect();
                    if parts.len() == 2 && parts[0].parse::<u32>().is_ok() && parts[1].parse::<u32>().is_ok() {
                        let w: u32 = parts[0].parse().unwrap();
                        let h: u32 = parts[1].parse().unwrap();
                        cli.resize = Some([w, h]);
                    }
                }
            }
            "--info" => {
                cli.info_only = true;
            }
            "--benchmark" | "--bench" => {
                cli.benchmark = true;
            }
            "--list-scenes" | "--scenes" => {
                cli.list_scenes = true;
            }
            "--seed" => {
                i += 1;
                if i < args.len() {
                    cli.seed = args[i].parse().ok();
                }
            }
            "-e" | "--exposure" => {
                i += 1;
                if i < args.len() {
                    cli.exposure = args[i].parse().ok();
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
                eprintln!("  14 materials, 29 textures, 33 geometry types, 72 post-processing effects");
                std::process::exit(0);
            }
            "-h" | "--help" => {
                eprintln!("Usage: luminara [scene.toml] [options]");
                eprintln!();
                eprintln!("  scene.toml        Scene description file (optional, uses demo scene if omitted)");
                eprintln!("  -o, --output      Output file path (default: output.png, '-' for stdout PPM)");
                eprintln!("  -w, --width       Override render width");
                eprintln!("  -H, --height      Override render height");
                eprintln!("  -s, --samples     Override samples per pixel");
                eprintln!("  -d, --depth       Override max ray bounce depth");
                eprintln!("  -t, --threads     Number of render threads (default: all cores)");
                eprintln!("      --seed        Set RNG seed for deterministic rendering");
                eprintln!("  -e, --exposure    Exposure multiplier (default: 1.0)");
                eprintln!("      --auto-exposure  Automatically compute exposure from scene luminance");
                eprintln!("      --tone-map TM    Tone mapping: aces, reinhard, filmic, none (default: aces)");
                eprintln!("      --denoise     Apply bilateral denoiser to reduce noise");
                eprintln!("      --bloom N     Add bloom glow effect (intensity, e.g. 0.3)");
                eprintln!("      --vignette N  Darken edges for cinematic look (e.g. 0.5)");
                eprintln!("      --grain N     Add film grain noise (e.g. 0.1)");
                eprintln!("      --saturation N  Color saturation (1.0=normal, 0=grayscale)");
                eprintln!("      --contrast N  Contrast adjustment (1.0=normal, >1=more)");
                eprintln!("      --wb N        White balance (-=cooler/blue, +=warmer/orange)");
                eprintln!("      --sharpen N   Sharpen details (e.g. 0.5)");
                eprintln!("      --hue-shift N Rotate hue in degrees (e.g. 30, 180)");
                eprintln!("      --dither      Apply ordered dithering to reduce banding");
                eprintln!("      --filter F    Pixel filter: box, triangle, gaussian, mitchell");
                eprintln!("      --save-depth F   Save depth pass to image file");
                eprintln!("      --save-normals F Save normal pass to image file");
                eprintln!("      --save-albedo F Save albedo (base color) pass to image file");
                eprintln!("      --distortion N  Lens distortion (+barrel, -pincushion, e.g. 0.3)");
                eprintln!("      --firefly N   Remove firefly outliers (threshold, e.g. 5.0)");
                eprintln!("      --ca N        Chromatic aberration strength (e.g. 0.005)");
                eprintln!("      --time-limit N  Max render time in seconds");
                eprintln!("      --adaptive    Adaptive sampling: fewer samples on smooth areas");
                eprintln!("      --adaptive-threshold N  Noise threshold (default 0.03)");
                eprintln!("      --gamma N     Custom gamma (0=sRGB default, 2.2=simple)");
                eprintln!("      --save-hdr F  Save HDR data to Radiance .hdr file");
                eprintln!("      --crop X,Y,W,H  Render only a sub-region of the image");
                eprintln!("  -p, --preview     Quick preview (1/4 res, low samples)");
                eprintln!("  -q, --quiet       Suppress progress output");
                eprintln!("      --info        Show scene info without rendering");
                eprintln!("      --benchmark   Run standardized performance benchmark");
                eprintln!("      --emboss N    Emboss effect intensity (e.g. 0.5)");
                eprintln!("      --oil-paint N Oil painting Kuwahara filter radius (e.g. 3)");
                eprintln!("      --color-map S False color: inferno, viridis, turbo, heat, thermal, neon, grayscale");
                eprintln!("      --solarize N  Solarize at luminance threshold (0.0-1.0)");
                eprintln!("      --duo-tone S  Two-color toning (e.g. \"0,0,64;255,200,0\")");
                eprintln!("      --sketch      Pencil sketch effect (grayscale + edge detection)");
                eprintln!("      --median N    Median filter radius for noise removal (e.g. 1)");
                eprintln!("      --crosshatch N Pen-and-ink crosshatch spacing (e.g. 4)");
                eprintln!("      --glitch N    Digital glitch effect intensity (e.g. 0.5)");
                eprintln!("      --depth-fog N Atmospheric depth fog density (e.g. 0.1)");
                eprintln!("      --fog-color R,G,B  Fog color (0-1 each, default: 0.8,0.85,0.9)");
                eprintln!("      --channel-swap S Swap RGB channels: rbg, grb, gbr, brg, bgr");
                eprintln!("      --quantize N  Reduce to N colors via median-cut quantization");
                eprintln!("      --tint R,G,B  Multiply all pixels by RGB color (0-1 each)");
                eprintln!("      --palette S   Named color palette: gameboy, cga, nes, pastel,");
                eprintln!("                    grayscale4, sunset, cyberpunk, sepia4");
                eprintln!("      --ascii       Print ASCII art rendering to terminal");
                eprintln!("      --tri-tone S  Three-color toning (\"R,G,B;R,G,B;R,G,B\")");
                eprintln!("      --gradient-map S  Custom gradient color map (\"RRGGBB;RRGGBB;...\")");
                eprintln!("      --split-tone S  Split toning (\"R,G,B;R,G,B\" shadow;highlight)");
                eprintln!("      --color-shift N  Rotate RGB channels (1=right, 2=left)");
                eprintln!("      --posterize-channels R,G,B  Per-channel posterization levels");
                eprintln!("      --lens-flare N  Lens flare streaks from brightest point");
                eprintln!("      --cel-shade N  Cel-shading/toon (N color bands + outlines)");
                eprintln!("      --frame N     Picture frame with bevel (N pixel width)");
                eprintln!("      --hex-pixelate N  Hexagonal pixelation (cell size)");
                eprintln!("      --pencil N    Pencil cross-hatching drawing (line spacing)");
                eprintln!("      --dot-matrix N  Dot matrix printer effect (cell size)");
                eprintln!("      --noise-overlay N  Procedural noise texture overlay");
                eprintln!("      --color-halftone N  CMYK color halftone with rotated screens");
                eprintln!("      --kaleidoscope N  Kaleidoscope mirror symmetry (N segments)");
                eprintln!("      --pop-art N   Warhol-style pop art color bands");
                eprintln!("      --watercolor N  Watercolor painting effect (blur radius)");
                eprintln!("      --auto-levels Auto-stretch histogram for full dynamic range");
                eprintln!("      --brightness N Brightness adjustment (-1.0 to 1.0, 0 = no change)");
                eprintln!("      --color-balance R,G,B  Per-channel level adjustment (e.g. 1.2,1.0,0.8)");
                eprintln!("      --stipple N   Pointillism/stipple dot effect (cell size in px)");
                eprintln!("      --night-vision Night vision (green-tinted luminance amplification)");
                eprintln!("      --fisheye N   Barrel distortion (positive) or pincushion (negative)");
                eprintln!("      --wave N      Wave/ripple distortion amplitude in pixels");
                eprintln!("      --swirl N     Swirl/twist distortion (e.g. 2.0, negative for CCW)");
                eprintln!("      --mosaic N    Voronoi stained-glass mosaic (cell size in px)");
                eprintln!("      --radial-blur N  Zoom blur from center (e.g. 0.5)");
                eprintln!("      --border N    Add N-pixel border frame");
                eprintln!("      --border-color R,G,B  Border color (0-1 each, default: black)");
                eprintln!("      --resize WxH  Resize output (bilinear), e.g. 1920x1080");
                eprintln!("      --warm        Warm white balance preset");
                eprintln!("      --cool        Cool white balance preset");
                eprintln!("      --rotate N    Rotate output image (90, 180, 270 degrees)");
                eprintln!("      --panorama    360° equirectangular panoramic camera");
                eprintln!("      --vintage     Vintage photo preset (sepia + grain + vignette)");
                eprintln!("      --cinematic   Cinematic preset (bloom + warm tint + vignette)");
                eprintln!("      --retro       Retro CRT preset (scanlines + pixelate + aberration)");
                eprintln!("      --dreamy      Dreamy/ethereal preset (bloom + blur + saturation)");
                eprintln!("      --noir        Film noir preset (B&W + high contrast + vignette)");
                eprintln!("      --miniature   Tilt-shift miniature preset (blur + saturation)");
                eprintln!("      --save-json F Save render statistics as JSON to file");
                eprintln!("      --list-scenes List available scene files");
                eprintln!("  -V, --version     Show version");
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
