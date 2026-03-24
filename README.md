# Luminara

A physically-based ray tracer written in Rust. Luminara renders photorealistic 3D scenes from declarative TOML scene descriptions, with support for multiple material types, depth of field, and multithreaded rendering.

## What it does

Luminara traces rays of light through a virtual scene, simulating how photons interact with surfaces to produce realistic images. It supports:

- **Geometry**: Spheres, infinite planes, triangles, and OBJ triangle meshes
- **Materials**: Lambertian (diffuse), metallic (with configurable fuzz), dielectric (glass), and emissive (light sources)
- **Textures**: Solid color and 3D checkerboard patterns, with UV mapping for spheres
- **Camera**: Configurable field of view, position, depth of field (aperture/focus distance)
- **Rendering**: Multithreaded via Rayon, gamma-corrected output, anti-aliasing, progress indicator
- **Acceleration**: BVH (bounding volume hierarchy) for O(log n) ray intersection
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
| `texture` | Texture trait + SolidColor, Checker |
| `sphere` | Sphere intersection with UV mapping |
| `plane` | Infinite plane intersection |
| `triangle` | Triangle intersection (Möller-Trumbore) |
| `obj` | OBJ file loading with fan triangulation |
| `camera` | Perspective camera with depth of field |
| `render` | Multithreaded ray tracing engine with progress |
| `scene` | TOML scene loading, BVH construction, demo scene |

## Design decisions

- **Pure Rust, minimal dependencies**: Only `rand`, `rayon`, `image`, `toml`, and `serde`. No graphics API, no GPU — just math and parallelism.
- **Trait-based extensibility**: `Hittable`, `Material`, and `Texture` traits make adding new geometry, materials, and textures straightforward.
- **BVH acceleration**: Bounded objects are organized in a bounding volume hierarchy for logarithmic ray intersection. Unbounded objects (infinite planes) are tested separately.
- **Physically-based**: Schlick's approximation for Fresnel, proper refraction via Snell's law, Lambertian scattering, gamma correction, emissive light transport.
- **Deterministic per-row seeding**: Each row gets its own RNG seeded by row index, making renders reproducible regardless of thread scheduling.

## Included scenes

- **showcase.toml**: Glass, metal, and matte spheres on a checkerboard ground with a glowing light sphere
- **cornell.toml**: Classic Cornell Box with colored walls, area light, and two objects

## What's next

- Image textures (PNG/JPG mapped onto surfaces)
- Perlin noise procedural textures
- Constructive solid geometry (CSG)
- Importance sampling for faster convergence
- HDR environment maps

---

*Built by Claude Code in the Climagine workspace.*
