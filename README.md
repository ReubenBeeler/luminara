# Luminara

A physically-based ray tracer written in Rust. Luminara renders photorealistic 3D scenes from declarative TOML scene descriptions, with support for multiple material types, depth of field, and multithreaded rendering.

## What it does

Luminara traces rays of light through a virtual scene, simulating how photons interact with surfaces to produce realistic images. It supports:

- **Geometry**: Spheres, ellipsoids, infinite planes, disks, triangles, cylinders, cones, capsules, axis-aligned rectangles, boxes, and OBJ triangle meshes
- **Materials**: Lambertian (diffuse), metallic (with configurable fuzz), dielectric (glass with tint and roughness), emissive (light sources)
- **Textures**: Solid color, 3D checkerboard, UV checkerboard, stripes, gradient, Perlin marble, turbulence, and image textures (PNG/JPG)
- **Volumetrics**: Constant-density fog/smoke with isotropic scattering
- **Camera**: Configurable field of view, position, depth of field (aperture/focus distance)
- **Rendering**: Multithreaded via Rayon, stratified sampling, ACES tone mapping, sRGB gamma, progress indicator
- **Acceleration**: BVH with Surface Area Heuristic for O(log n) ray intersection
- **Backgrounds**: Sky gradient, sun+sky with directional sun disk, solid color, custom gradient, or black
- **Output**: PNG images, PPM format, or stdout piping (`-o -`)
- **Scenes**: Declarative TOML format, plus a built-in demo scene

## Usage

```bash
# Render the built-in demo scene (random spheres)
cargo run --release

# Render a custom scene
cargo run --release -- scenes/showcase.toml -o my_render.png

# Override resolution and samples from CLI
cargo run --release -- scenes/gallery.toml -w 1920 --height 1080 -s 256

# Render the Cornell Box
cargo run --release -- scenes/cornell.toml -o cornell.png

# See all options
cargo run --release -- --help
```

### CLI options

| Flag | Description |
|------|-------------|
| `-o`, `--output` | Output file path (default: output.png) |
| `-w`, `--width` | Override render width |
| `--height` | Override render height |
| `-s`, `--samples` | Override samples per pixel |
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
| `lambertian` | `color` |
| `metal` | `color`, `fuzz` (optional, 0.0-1.0) |
| `dielectric` | `refraction_index`, `tint` (optional), `roughness` (optional, 0.0-1.0) |
| `emissive` | `color`, `intensity` (optional, default 1.0) |
| `checker` | `color1`, `color2`, `scale` (optional) |
| `stripe` | `color1`, `color2`, `scale` (optional), `axis` (optional, x/y/z) |
| `gradient_tex` | `color1`, `color2`, `axis` (optional), `min`/`max` (optional) |
| `marble` | `color`, `scale` (optional) |
| `turbulence` | `color`, `scale` (optional) |
| `image` | `file` (path to PNG/JPG) |
| `uv_checker` | `color1`, `color2`, `frequency` (optional) |

## Architecture

| Module | Purpose |
|--------|---------|
| `vec3` | 3D vector math, colors, points |
| `ray` | Ray definition (origin + direction) |
| `aabb` | Axis-aligned bounding boxes |
| `bvh` | Bounding volume hierarchy acceleration |
| `hit` | Hit records, `Hittable` trait, scene list |
| `material` | Material trait + Lambertian, Metal, Dielectric, Emissive |
| `texture` | Texture trait + SolidColor, Checker, Stripe, Gradient, Marble, Turbulence, Image |
| `sphere` | Sphere intersection with UV mapping |
| `plane` | Infinite plane intersection |
| `disk` | Finite circular plane |
| `triangle` | Triangle intersection (Möller-Trumbore) |
| `cylinder` | Finite Y-axis cylinder |
| `cone` | Finite Y-axis cone |
| `rect` | Axis-aligned rectangles (XY, XZ, YZ) and box builder |
| `constant_medium` | Volumetric fog/smoke with isotropic scattering |
| `obj` | OBJ file loading with fan triangulation |
| `camera` | Perspective camera with depth of field |
| `render` | Stratified sampling, ACES tone mapping, progress |
| `scene` | TOML scene loading, BVH construction, demo scene |

## Design decisions

- **Pure Rust, minimal dependencies**: Only `rand`, `rayon`, `image`, `toml`, and `serde`. No graphics API, no GPU — just math and parallelism.
- **Trait-based extensibility**: `Hittable`, `Material`, and `Texture` traits make adding new geometry, materials, and textures straightforward.
- **BVH acceleration**: Bounded objects are organized in a bounding volume hierarchy for logarithmic ray intersection. Unbounded objects (infinite planes) are tested separately.
- **Physically-based**: Schlick's approximation for Fresnel, proper refraction via Snell's law, Lambertian scattering, emissive light transport.
- **HDR pipeline**: ACES filmic tone mapping prevents harsh clamping of bright emissive surfaces. Stratified (jittered) sampling reduces noise.
- **Deterministic per-row seeding**: Each row gets its own RNG seeded by row index, making renders reproducible regardless of thread scheduling.
- **Input validation**: Guards against zero-length normals, zero-scale textures, and missing image files to prevent NaN propagation.

## Included scenes

- **showcase.toml**: Glass, metal, marble, and matte spheres on a checkerboard ground with emissive light
- **cornell.toml**: Classic Cornell Box with colored walls, area light, and two boxes
- **gallery.toml**: Feature showcase with all geometry types, textures, fog, tinted glass, and multiple lights
- **outdoor.toml**: Sunlit outdoor scene with frosted glass, UV globe, cylinder, and cone
- **everything.toml**: Comprehensive demo of every geometry, material, texture, and effect

## What's next

- Normal mapping
- Constructive solid geometry (CSG)
- Importance sampling for faster convergence
- HDR environment maps
- Motion blur

---

*Built by Claude Code in the Climagine workspace.*
