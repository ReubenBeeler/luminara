# Luminara

![Thumbnail](./thumbnail.png)

A physically-based ray tracer written in Rust. Luminara renders photorealistic 3D scenes from declarative TOML scene descriptions, with support for multiple material types, depth of field, and multithreaded rendering.

## What it does

Luminara traces rays of light through a virtual scene, simulating how photons interact with surfaces to produce realistic images. It supports:

- **Geometry**: Spheres, moving spheres (motion blur), ellipsoids, tori, infinite planes, disks, triangles, quads (parallelograms), cylinders, cones, capsules, hemispheres, annuli, axis-aligned rectangles, boxes, rounded boxes, superellipsoids, springs/helices, Möbius strips, prisms, wedges, OBJ and PLY triangle meshes
- **Materials**: Lambertian (diffuse), metallic, dielectric (glass with Beer's Law, tint, roughness, dispersion), emissive, blackbody (Kelvin temperature), microfacet/PBR (Cook-Torrance GGX), iridescent (thin-film interference), translucent, subsurface scattering (random-walk SSS), velvet (rim lighting), clearcoat (lacquer), anisotropic (brushed metal), toon (cel-shading), blend (mix two materials)
- **Textures**: Solid color, 3D checkerboard, UV checkerboard, stripes, gradient, rings, wood, dots, grid, Perlin marble, turbulence, Voronoi, spiral, hexgrid, noise, color ramp (multi-stop gradient), FBM (fractal Brownian motion), wavy (sine interference), mix (blend two textures), tri-planar mapping, UV transforms (offset/rotation/tiling), cloud, lava, camouflage, brick, rust/patina, terrain, plasma, and image textures (PNG/JPG)
- **Volumetrics**: Constant-density fog/smoke with isotropic scattering
- **Camera**: Configurable field of view, position, depth of field (aperture/focus distance)
- **Motion blur**: Moving spheres with per-ray time sampling
- **Rendering**: Multithreaded via Rayon, stratified sampling, Next Event Estimation (direct light sampling for sphere, rect, and disk lights), adaptive sampling (variance-based early termination), Russian roulette path termination, pixel reconstruction filters (box, triangle, Gaussian, Mitchell-Netravali), ACES/Reinhard/Filmic tone mapping, sRGB gamma, progress indicator with ETA, Mrays/s stats, time-budgeted rendering
- **Post-processing**: Bloom (glow), vignette, film grain, saturation, contrast, white balance, hue shift, sharpening, chromatic aberration, bilateral denoising, ordered dithering, custom gamma, firefly removal, lens distortion, posterize, sepia tone, edge detection/outlines, pixelate, color inversion, CRT scanlines, B&W threshold, Gaussian blur, tilt-shift, color grading (shadows/highlights), halftone dots, emboss, oil paint (Kuwahara filter), false color mapping, solarize, duo-tone, pencil sketch, median filter, crosshatch, digital glitch, depth fog, channel swap, color quantization (median-cut), color tint, named palettes (gameboy/cga/nes/pastel/cyberpunk/etc), radial blur, border frame, resize, rotation, warm/cool presets, ASCII art output, mosaic, swirl, wave, fisheye, night vision, stipple, watercolor, auto-levels, brightness, color balance, thermal/neon color maps, gradient map, split toning, color shift, per-channel posterize, lens flare, cel-shade, picture frame, tri-tone, pop art, hex pixelate, pencil cross-hatching
- **Camera modes**: Perspective (standard), panoramic (360° equirectangular)
- **Presets**: Vintage (sepia+grain+vignette), cinematic (bloom+warm+vignette), retro (scanlines+pixelate+aberration), dreamy (bloom+blur+saturation)
- **Output**: PNG, PPM, JPEG, Radiance HDR (.hdr), OpenEXR (.exr), depth pass, normal pass, albedo pass, JSON stats, stdout piping
- **Acceleration**: BVH with Surface Area Heuristic for O(log n) ray intersection
- **CSG**: Constructive Solid Geometry — union, intersection, and difference operations on convex primitives
- **Backgrounds**: Sky gradient, sun+sky with directional sun disk, sunset preset, solid color, custom gradient, starfield, HDRI environment maps (bilinear interpolated), or black
- **Transforms**: Translate, rotate (X/Y/Z-axis), uniform and non-uniform scale
- **Scenes**: Declarative TOML format, plus a built-in demo scene

## Usage

```bash
# Render the built-in demo scene (random spheres)
cargo run --release

# Render a custom scene
cargo run --release -- examples/scenes/showcase.toml -o my_render.png

# Override resolution and samples from CLI
cargo run --release -- examples/scenes/gallery.toml -w 1920 --height 1080 -s 256

# Render the Cornell Box
cargo run --release -- examples/scenes/cornell.toml -o cornell.png

# See all options
cargo run --release -- --help
```

### CLI options

| Flag | Description |
|------|-------------|
| `-o`, `--output` | Output file path (default: output.png, `-` for stdout PPM) |
| `-w`, `--width` | Override render width |
| `-H`, `--height` | Override render height |
| `-s`, `--samples` | Override samples per pixel |
| `-d`, `--depth` | Override max ray bounce depth |
| `-t`, `--threads` | Number of render threads (default: all cores) |
| `--seed` | Set RNG seed for deterministic rendering |
| `-e`, `--exposure` | Exposure multiplier (default: 1.0) |
| `--auto-exposure` | Automatically compute exposure from scene luminance |
| `--tone-map` | Tone mapping: aces, reinhard, filmic, none |
| `-q`, `--quiet` | Suppress progress output |
| `--denoise` | Apply bilateral denoiser to reduce noise |
| `--bloom N` | Add bloom glow effect (intensity, e.g. 0.3) |
| `--vignette N` | Darken edges for cinematic look (e.g. 0.5) |
| `--grain N` | Add film grain noise (e.g. 0.1) |
| `--sharpen N` | Sharpen details (e.g. 0.5) |
| `--saturation N` | Saturation (1.0 = normal, 0.0 = grayscale) |
| `--contrast N` | Contrast (1.0 = normal) |
| `--white-balance N` | Color temperature shift (negative = cool, positive = warm) |
| `--hue-shift N` | Rotate hue in degrees |
| `--dither` | Apply ordered dithering to reduce banding |
| `--gamma N` | Custom gamma (0 = sRGB default) |
| `--ca N` | Chromatic aberration strength (e.g. 0.005) |
| `--filter F` | Pixel filter: box, triangle, gaussian, mitchell |
| `--adaptive` | Adaptive sampling: fewer samples on smooth areas |
| `--adaptive-threshold N` | Noise threshold for adaptive (default 0.03) |
| `--save-hdr F` | Save HDR data to Radiance .hdr file |
| `--save-depth F` | Save depth pass to image file |
| `--save-normals F` | Save normal pass to image file |
| `--save-albedo F` | Save albedo (base color) pass to image file |
| `--crop X,Y,W,H` | Render only a sub-region of the image |
| `--info` | Show scene info and post-processing pipeline |
| `--firefly-filter N` | Remove firefly outlier pixels (e.g. 3.0) |
| `--lens-distortion N` | Barrel/pincushion distortion (positive/negative) |
| `--time-limit N` | Max render time in seconds |
| `--posterize N` | Reduce to N color levels per channel (retro look) |
| `--sepia N` | Sepia tone intensity (0.0-1.0) |
| `--edge-detect N` | Edge detection / outline effect strength |
| `--pixelate N` | NxN pixel block averaging (pixel art) |
| `--invert` | Invert colors (negative image) |
| `--emboss N` | Emboss effect intensity (raised/engraved look) |
| `--oil-paint N` | Oil painting Kuwahara filter radius (e.g. 3) |
| `--color-map S` | False color: inferno, viridis, turbo, heat, grayscale |
| `--solarize N` | Solarize at luminance threshold (0.0-1.0) |
| `--duo-tone S` | Two-color toning ("R,G,B;R,G,B" e.g. "0,0,64;255,200,0") |
| `--sketch` | Pencil sketch effect (grayscale + edge detection) |
| `--median N` | Median filter for noise removal (edge-preserving) |
| `--crosshatch N` | Pen-and-ink crosshatch spacing (e.g. 4) |
| `--glitch N` | Digital glitch effect intensity (e.g. 0.5) |
| `--depth-fog N` | Atmospheric depth fog density (e.g. 0.1) |
| `--channel-swap S` | Swap RGB channels: rbg, grb, gbr, brg, bgr |
| `--quantize N` | Reduce to N colors via median-cut quantization |
| `--tint R,G,B` | Multiply all pixels by RGB color (0-1 each) |
| `--palette S` | Named palette: gameboy, cga, nes, pastel, cyberpunk, etc. |
| `--radial-blur N` | Zoom blur from center (e.g. 0.5) |
| `--border N` | Add N-pixel border frame |
| `--border-color R,G,B` | Border color (0-1 each, default: black) |
| `--resize WxH` | Resize output with bilinear interpolation |
| `--rotate N` | Rotate output (90, 180, 270 degrees) |
| `--warm` | Warm white balance preset |
| `--cool` | Cool white balance preset |
| `--ascii` | Print ASCII art to terminal |
| `--mosaic N` | Voronoi stained-glass mosaic (cell size) |
| `--swirl N` | Swirl/twist distortion |
| `--wave N` | Wave/ripple distortion (amplitude) |
| `--fisheye N` | Barrel (positive) or pincushion (negative) distortion |
| `--night-vision` | Green-tinted night vision simulation |
| `--stipple N` | Pointillism/stipple dot effect |
| `--watercolor N` | Watercolor painting effect |
| `--auto-levels` | Auto-stretch histogram for full dynamic range |
| `--brightness N` | Brightness adjustment (-1.0 to 1.0) |
| `--color-balance R,G,B` | Per-channel level adjustment |
| `--fog-color R,G,B` | Depth fog color |
| `--panorama` | 360° equirectangular panoramic camera |
| `--vintage` | Vintage photo preset |
| `--cinematic` | Cinematic look preset |
| `--retro` | Retro CRT preset (scanlines + pixelate + aberration) |
| `--dreamy` | Dreamy/ethereal preset (bloom + blur + saturation) |
| `--tri-tone S` | Three-color toning ("R,G,B;R,G,B;R,G,B") |
| `--gradient-map S` | Custom gradient color map ("RRGGBB;RRGGBB;...") |
| `--split-tone S` | Split toning ("R,G,B;R,G,B" shadow;highlight) |
| `--color-shift N` | Rotate RGB channels (1=right, 2=left) |
| `--posterize-channels R,G,B` | Per-channel posterization levels |
| `--lens-flare N` | Lens flare streaks from brightest point |
| `--cel-shade N` | Cel-shading/toon (N color bands + outlines) |
| `--frame N` | Picture frame with bevel (N pixel width) |
| `--pop-art N` | Warhol-style pop art color bands |
| `--save-json F` | Save render statistics as JSON |
| `--benchmark` | Run built-in benchmark and report Mrays/s |
| `--list-scenes` | List available .toml scene files |
| `-p`, `--preview` | Quick preview (1/4 res, low samples) |
| `-V`, `--version` | Show version |
| `-h`, `--help` | Show help |

## Scene format

Scenes are TOML files. Example:

```toml
[render]
width = 800
height = 450
samples = 64
max_depth = 50
background = { type = "sky" }

[camera]
look_from = [3.0, 2.0, 6.0]
look_at = [0.0, 0.5, 0.0]
vfov = 30.0
aperture = 0.05
focus_dist = 7.0

[[sphere]]
center = [0.0, 1.0, 0.0]
radius = 1.0
material = { type = "dielectric", refraction_index = 1.5, tint = [0.9, 1.0, 0.9] }

[[cylinder]]
center = [2.0, 0.0, 0.0]
radius = 0.3
height = 2.0
material = { type = "metal", color = [0.8, 0.8, 0.8], fuzz = 0.1 }

[[cone]]
center = [-2.0, 0.0, 0.0]
radius = 0.5
height = 1.5
material = { type = "lambertian", color = [0.7, 0.2, 0.2] }

[[plane]]
point = [0.0, 0.0, 0.0]
normal = [0.0, 1.0, 0.0]
material = { type = "checker", color1 = [0.9, 0.9, 0.9], color2 = [0.2, 0.2, 0.2], scale = 1.0 }

[[sphere]]
center = [0.0, 5.0, 0.0]
radius = 1.0
material = { type = "emissive", color = [1.0, 1.0, 1.0], intensity = 10.0 }

[[box]]
min = [1.0, 0.0, -2.0]
max = [2.0, 1.0, -1.0]
material = { type = "marble", color = [0.9, 0.9, 0.9], scale = 4.0 }

[[fog]]
center = [0.0, 1.0, 0.0]
radius = 3.0
density = 0.2
color = [0.8, 0.8, 0.8]
```

### Material types

| Type | Parameters |
|------|-----------|
| `lambertian` / `matte` | `color` |
| `metal` | `color`, `fuzz` (optional, 0.0-1.0) |
| `dielectric` | `refraction_index`, `tint` (optional), `roughness` (optional, 0.0-1.0), `dispersion` (optional, 0.01-0.05 for prism effects) |
| `emissive` | `color`, `intensity` (optional, default 1.0), `texture` (optional, image path) |
| `microfacet` / `pbr` | `color`, `roughness` (optional, 0.01-1.0), `metallic` (optional, 0.0-1.0) |
| `mirror` | *(none — perfect reflector)* |
| `glass` | *(none — standard glass, IOR 1.5)* |
| `blend` | `material_a`, `material_b`, `ratio` (optional, default 0.5) |
| `checker` | `color1`, `color2`, `scale` (optional) |
| `uv_checker` | `color1`, `color2`, `frequency` (optional) |
| `stripe` | `color1`, `color2`, `scale` (optional), `axis` (optional) |
| `grid` | `line_color`, `bg_color`, `scale` (optional), `line_width` (optional) |
| `dots` | `dot_color`, `bg_color`, `scale` (optional), `radius` (optional) |
| `rings` | `color1`, `color2`, `scale` (optional) |
| `wood` | `color1`, `color2`, `scale` (optional) |
| `gradient_tex` | `color1`, `color2`, `axis` (optional), `min`/`max` (optional) |
| `marble` | `color`, `scale` (optional) |
| `turbulence` | `color`, `scale` (optional) |
| `iridescent` | `color` (optional), `thickness` (nm, optional), `film_ior` (optional), `roughness` (optional) |
| `translucent` | `color`, `translucency` (optional, 0.0-1.0), `scatter_width` (optional, 0.0-1.0) |
| `velvet` | `color`, `sheen` (optional, rim lighting intensity) |
| `clearcoat` | `color`, `coat_gloss` (optional), `coat_ior` (optional) |
| `anisotropic` / `brushed` | `color`, `roughness_u` (optional), `roughness_v` (optional), `tangent_axis` (optional) |
| `toon` / `cel` | `color`, `bands` (optional, shading steps), `specular` (optional) |
| `blackbody` | `temperature` (Kelvin), `intensity` (optional) |
| `subsurface` / `sss` | `color`, `mean_free_path` (optional, world units), `scatter_color` (optional, per-channel absorption) |
| `voronoi` | `color1`, `color2`, `scale` (optional) |
| `spiral` | `color1`, `color2`, `scale` (optional), `arms` (optional) |
| `hexgrid` | `color1`, `color2`, `scale` (optional), `line_width` (optional) |
| `noise` | `color1`, `color2`, `scale` (optional) |
| `color_ramp` / `ramp` | `stops` ([[pos,r,g,b],...]), `axis` (optional), `min_val`/`max_val` (optional) |
| `image` | `file` (path to PNG/JPG), `uv_offset` (optional [u,v]), `uv_rotation` (optional, degrees), `uv_tile` (optional [u,v]) |
| `fbm` / `fractal` | `color1`, `color2`, `scale` (optional), `octaves` (optional) |
| `wavy` / `wave` | `color1`, `color2`, `scale` (optional), `waves` (optional) |
| `mix_color` | `color1`, `color2`, `factor` (optional, 0.0-1.0) |
| `tri_planar` | `color1`, `color2`, `scale` (optional), `sharpness` (optional) |
| `opacity` | `base` (material), `alpha` (material), `threshold` (optional) |

## Architecture

| Module | Purpose |
|--------|---------|
| `vec3` | 3D vector math, colors, points |
| `ray` | Ray definition (origin + direction + time) |
| `aabb` | Axis-aligned bounding boxes |
| `bvh` | Bounding volume hierarchy acceleration |
| `hit` | Hit records, `Hittable` trait, scene list |
| `material` | Material trait + 14 material types (Lambertian, Metal, Dielectric, Emissive, Microfacet, Blend, Transparent, Iridescent, Translucent, Subsurface, Velvet, Clearcoat, Anisotropic, Toon) |
| `csg` | Constructive Solid Geometry (union, intersection, difference) |
| `texture` | Texture trait + 22 procedural/image textures |
| `sphere` | Sphere and MovingSphere with UV mapping |
| `ellipsoid` | Ellipsoid with 3-axis radii |
| `torus` | Torus via SDF ray marching |
| `plane` | Infinite plane with UV projection |
| `quad` | Parallelogram (arbitrary quad) primitive |
| `disk` | Finite circular plane |
| `triangle` | Triangle intersection (Möller-Trumbore) |
| `cylinder` | Finite Y-axis cylinder |
| `cone` | Finite Y-axis cone |
| `capsule` | Rounded cylinder (cylinder + hemispheres) |
| `prism` | Triangular prism geometry |
| `wedge` | Wedge (ramp) geometry |
| `rect` | Axis-aligned rectangles (XY, XZ, YZ) and box builder |
| `transform` | Translate, RotateX/Y/Z, Scale wrappers |
| `bump` | Perlin noise bump mapping |
| `constant_medium` | Volumetric fog/smoke with isotropic scattering |
| `obj` | OBJ and PLY file loading with fan triangulation |
| `camera` | Perspective camera with depth of field |
| `render` | Stratified/adaptive sampling, pixel filters, tone mapping, post-processing pipeline, AOV passes |
| `scene` | TOML scene loading, BVH construction, demo scene |

## Design decisions

- **Pure Rust, minimal dependencies**: Only `rand`, `rayon`, `image`, `toml`, and `serde`. No graphics API, no GPU — just math and parallelism.
- **Trait-based extensibility**: `Hittable`, `Material`, and `Texture` traits make adding new geometry, materials, and textures straightforward.
- **BVH acceleration**: Bounded objects are organized in a bounding volume hierarchy for logarithmic ray intersection. Unbounded objects (infinite planes) are tested separately.
- **Physically-based**: Cook-Torrance GGX microfacet BRDF, Schlick's approximation for Fresnel, Beer's Law volumetric absorption, proper refraction via Snell's law, Lambertian scattering, emissive light transport with Next Event Estimation.
- **HDR pipeline**: ACES, Reinhard, and Uncharted 2 filmic tone mapping. Auto-exposure via log-average luminance. Stratified (jittered) sampling reduces noise.
- **Deterministic per-row seeding**: Each row gets its own RNG seeded by row index, making renders reproducible regardless of thread scheduling.
- **Input validation**: Guards against zero-length normals, zero-scale textures, and missing image files to prevent NaN propagation.

## Included scenes

- **showcase.toml**: Glass, metal, marble, and matte spheres on a checkerboard ground with emissive light
- **cornell.toml**: Classic Cornell Box with colored walls, area light, and two boxes
- **gallery.toml**: Feature showcase with all geometry types, textures, fog, tinted glass, and multiple lights
- **outdoor.toml**: Sunlit outdoor scene with frosted glass, UV globe, cylinder, and cone
- **everything.toml**: Comprehensive demo of every geometry, material, texture, and effect
- **sunset.toml**: Sunset background preset scene
- **motion.toml**: Motion blur demo with moving spheres and quad geometry
- **csg_demo.toml**: CSG boolean operations (union, intersection, difference) with PBR materials
- **pbr_showcase.toml**: PBR material roughness and metallic parameter gradients
- **prism.toml**: Chromatic dispersion rainbow effects in glass
- **materials_showcase.toml**: Iridescent and translucent materials with bloom and vignette
- **organic.toml**: Subsurface scattering showcase (wax, jade, skin, marble)
- **toon.toml**: Cel-shaded rendering with edge detection and posterize
- **retro.toml**: Pixel art retro style with pixelate and posterize
- **cinematic.toml**: Movie-style orange/teal color grading with filmic tone mapping
- **architecture.toml**: Brick walls, marble floors, brushed metal, and blackbody lighting
- **abstract.toml**: SDF geometry showcase (rounded box, superellipsoid, spring, Möbius strip)
- **shapes.toml**: Complete geometry gallery with every primitive type

## What's next

- Multiple importance sampling (MIS) for better light-surface interaction
- Photon mapping for caustics
- Importance-sampled environment map lighting

---

*Built by Claude Code in the Climagine workspace.*
