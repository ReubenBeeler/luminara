# Luminara

A physically-based ray tracer written in Rust. Luminara renders photorealistic 3D scenes from declarative TOML scene descriptions, with support for multiple material types, depth of field, and multithreaded rendering.

## What it does

Luminara traces rays of light through a virtual scene, simulating how photons interact with surfaces to produce realistic images. It supports:

- **Geometry**: Spheres and infinite planes
- **Materials**: Lambertian (diffuse), metallic (with configurable fuzz), and dielectric (glass with refraction)
- **Camera**: Configurable field of view, position, depth of field (aperture/focus distance)
- **Rendering**: Multithreaded via Rayon, gamma-corrected output, anti-aliasing through multisampling
- **Output**: PNG images via the `image` crate
- **Scenes**: Declarative TOML format, plus a built-in demo scene

## Usage

```bash
# Render the built-in demo scene (random spheres)
cargo run --release

# Render a custom scene
cargo run --release -- scenes/showcase.toml -o my_render.png

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
material = { type = "lambertian", color = [0.8, 0.8, 0.8] }
```

## Architecture

| Module | Purpose |
|--------|---------|
| `vec3` | 3D vector math, colors, points |
| `ray` | Ray definition (origin + direction) |
| `hit` | Hit records, `Hittable` trait, scene list |
| `material` | Material trait + Lambertian, Metal, Dielectric |
| `sphere` | Sphere intersection |
| `plane` | Infinite plane intersection |
| `camera` | Perspective camera with depth of field |
| `render` | Multithreaded ray tracing engine |
| `scene` | TOML scene loading + demo scene builder |

## Design decisions

- **Pure Rust, minimal dependencies**: Only `rand`, `rayon`, `image`, `toml`, and `serde`. No graphics API, no GPU — just math and parallelism.
- **Trait-based extensibility**: `Hittable` and `Material` traits make adding new geometry and materials straightforward.
- **Physically-based**: Schlick's approximation for Fresnel, proper refraction via Snell's law, Lambertian scattering, gamma correction.
- **Deterministic per-row seeding**: Each row gets its own RNG seeded by row index, making renders reproducible regardless of thread scheduling.

## What's next

- BVH (bounding volume hierarchy) acceleration for scenes with many objects
- Triangle meshes and OBJ loading
- Texture mapping (solid, image, procedural)
- Area lights and emissive materials
- Progress bar during rendering

---

*Built by Claude Code in the Climagine workspace.*
