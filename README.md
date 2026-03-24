# Luminara

A physically-based ray tracer written in Rust. Luminara renders photorealistic 3D scenes from declarative TOML scene descriptions, with support for multiple material types, depth of field, and multithreaded rendering.

## What it does

Luminara traces rays of light through a virtual scene, simulating how photons interact with surfaces to produce realistic images. It supports:

- **Geometry**: Spheres, infinite planes, triangles, cylinders, axis-aligned rectangles, boxes, and OBJ triangle meshes
- **Materials**: Lambertian (diffuse), metallic (with configurable fuzz), dielectric (glass), and emissive (light sources)
- **Textures**: Solid color, 3D checkerboard, Perlin marble, turbulence, and image textures (PNG/JPG)
- **Camera**: Configurable field of view, position, depth of field (aperture/focus distance)
- **Rendering**: Multithreaded via Rayon, stratified sampling, ACES tone mapping, sRGB gamma, progress indicator
- **Acceleration**: BVH (bounding volume hierarchy) for O(log n) ray intersection
- **Backgrounds**: Sky gradient, solid color, custom gradient, or black
- **Output**: PNG images via the `image` crate
- **Scenes**: Declarative TOML format, plus a built-in demo scene

## Usage

```bash
# Render the built-in demo scene (random spheres)
cargo run --release

# Render a custom scene
cargo run --release -- scenes/showcase.toml -o my_render.png

# Render the Cornell Box
cargo run --release -- scenes/cornell.toml -o cornell.png

# See options
cargo run --release -- --help
```

## Scene format

Scenes are TOML files. Example:

```toml
[render]
width = 800
height = 450
samples = 64
max_depth = 50

[camera]
look_from = [3.0, 2.0, 6.0]
look_at = [0.0, 0.5, 0.0]
vfov = 30.0
aperture = 0.05
focus_dist = 7.0

[[sphere]]
center = [0.0, 1.0, 0.0]
radius = 1.0
material = { type = "dielectric", refraction_index = 1.5 }

[[sphere]]
center = [2.5, 0.7, -1.0]
radius = 0.7
material = { type = "metal", color = [0.8, 0.7, 0.6], fuzz = 0.0 }

[[plane]]
point = [0.0, 0.0, 0.0]
normal = [0.0, 1.0, 0.0]
material = { type = "checker", color1 = [0.9, 0.9, 0.9], color2 = [0.2, 0.2, 0.2], scale = 1.0 }

# Emissive light
[[sphere]]
center = [0.0, 5.0, 0.0]
radius = 1.0
material = { type = "emissive", color = [1.0, 1.0, 1.0], intensity = 10.0 }

# Triangle
[[triangle]]
v0 = [0.0, 0.0, 0.0]
v1 = [1.0, 0.0, 0.0]
v2 = [0.0, 1.0, 0.0]
material = { type = "lambertian", color = [0.8, 0.2, 0.2] }

# OBJ mesh
[[mesh]]
file = "models/bunny.obj"
material = { type = "metal", color = [0.9, 0.9, 0.9], fuzz = 0.1 }
scale = 10.0
offset = [0.0, 0.0, 0.0]
```

## Architecture

| Module | Purpose |
|--------|---------|
| `vec3` | 3D vector math, colors, points |
| `ray` | Ray definition (origin + direction) |
| `aabb` | Axis-aligned bounding boxes |
| `bvh` | Bounding volume hierarchy acceleration |
| `hit` | Hit records, `Hittable` trait, scene list |
| `material` | Material trait + Lambertian, Metal, Dielectric, Emissive |
| `texture` | Texture trait + SolidColor, Checker, Marble, Turbulence, Image |
| `sphere` | Sphere intersection with UV mapping |
| `plane` | Infinite plane intersection |
| `triangle` | Triangle intersection (Möller-Trumbore) |
| `cylinder` | Finite Y-axis cylinder intersection |
| `rect` | Axis-aligned rectangles (XY, XZ, YZ) and box builder |
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

## Included scenes

- **showcase.toml**: Glass, metal, marble, and matte spheres on a checkerboard ground with a glowing light
- **cornell.toml**: Classic Cornell Box with colored walls, area light, and two boxes
- **gallery.toml**: Feature showcase with all geometry types, textures, and multiple light sources

## What's next

- Normal mapping
- Constructive solid geometry (CSG)
- Importance sampling for faster convergence
- HDR environment maps
- Depth of field bokeh shapes

---

*Built by Claude Code in the Climagine workspace.*
