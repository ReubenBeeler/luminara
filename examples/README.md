# Luminara Examples

A gallery of 90 renders demonstrating what Luminara can do.

## Generating

```bash
# Full quality (1280x720, 256 spp) — takes a while
./examples/generate.sh

# Quick preview (640x360, 64 spp) — much faster
./examples/generate.sh quick

# Render a single example by name
./examples/generate.sh 45_glitch
```

Output goes to `examples/renders/`.

## Gallery

### Scenes (01–10)

| # | Name | Description |
|---|------|-------------|
| 01 | Demo Scene | Built-in random spheres scene |
| 02 | Cornell Box | Classic radiosity test scene with colored walls |
| 03 | Showcase | Glass, metal, marble spheres on checkerboard |
| 04 | Gallery | All geometry types, textures, fog, tinted glass |
| 05 | Outdoor | Sunlit scene with frosted glass, UV globe |
| 06 | Sunset | Sunset sky background preset |
| 07 | Architecture | Brick, marble, brushed metal, blackbody lights |
| 08 | Abstract SDF | Rounded box, superellipsoid, spring, Möbius strip |
| 09 | Shapes | Every primitive geometry type |
| 10 | CSG Booleans | Union, intersection, difference operations |

### Materials (11–14)

| # | Name | Description |
|---|------|-------------|
| 11 | PBR Metals | Roughness and metallic parameter gradients |
| 12 | Organic SSS | Subsurface scattering: wax, jade, skin, marble |
| 13 | Iridescent | Thin-film interference with bloom |
| 14 | Glass Prism | Chromatic dispersion rainbow effects |

### Artistic Presets (15–21)

| # | Name | Description |
|---|------|-------------|
| 15 | Vintage | Sepia + grain + vignette |
| 16 | Cinematic | Bloom + warm tint + vignette |
| 17 | Noir | Black & white + high contrast + vignette |
| 18 | Dreamy | Bloom + blur + saturation |
| 19 | Retro CRT | Scanlines + pixelate + chromatic aberration |
| 20 | Comic Book | Cel-shade + halftone + contrast |
| 21 | Miniature | Tilt-shift + saturation boost |

### Painting & Drawing (22–27)

| # | Name | Description |
|---|------|-------------|
| 22 | Oil Painting | Kuwahara filter |
| 23 | Watercolor | Watercolor blur effect |
| 24 | Pencil Sketch | Grayscale + edge detection |
| 25 | Crosshatch | Pen-and-ink style |
| 26 | Stipple | Pointillism dot effect |
| 27 | Pencil Hatching | Cross-hatching drawing |

### Color Effects (28–44)

| # | Name | Description |
|---|------|-------------|
| 28 | Game Boy | 4-color green palette |
| 29 | NES | NES color palette |
| 30 | CGA | CGA 4-color palette |
| 31 | Cyberpunk | Neon cyberpunk palette |
| 32 | Inferno | Inferno scientific colormap |
| 33 | Viridis | Viridis scientific colormap |
| 34 | Thermal | Thermal/heat vision |
| 35 | Neon | Neon glow colormap |
| 36 | Duo-tone | Blue-gold two-color toning |
| 37 | Tri-tone | Three-color toning |
| 38 | Split Tone | Shadow/highlight toning |
| 39 | Sepia | Classic sepia tone |
| 40 | Night Vision | Green-tinted amplification |
| 41 | Color Invert | Negative image |
| 42 | Channel Swap | GBR channel rotation |
| 43 | Quantize 8 | Median-cut to 8 colors |
| 44 | Solarize | Partial tone inversion |

### Distortion & Texture (45–58)

| # | Name | Description |
|---|------|-------------|
| 45 | Glitch | Digital glitch artifacts |
| 46 | Mosaic | Voronoi stained glass |
| 47 | Pixelate | Block pixel art |
| 48 | Hex Pixelate | Hexagonal cells |
| 49 | Dot Matrix | Printer dots |
| 50 | Color Halftone | CMYK rotated screens |
| 51 | Emboss | Raised relief effect |
| 52 | Swirl | Twist distortion |
| 53 | Wave Ripple | Sine wave distortion |
| 54 | Fisheye | Barrel distortion |
| 55 | Kaleidoscope | Mirror symmetry (8 segments) |
| 56 | Frosted Glass | Displacement blur |
| 57 | Spin Blur | Rotational motion blur |
| 58 | Radial Blur | Zoom blur from center |

### Enhancement & Grading (59–71)

| # | Name | Description |
|---|------|-------------|
| 59 | Bloom | Glow around bright areas |
| 60 | High Contrast | Punchy contrast + saturation |
| 61 | Desaturated | Full grayscale |
| 62 | Warm | Warm white balance + vignette |
| 63 | Cool | Cool white balance + vignette |
| 64 | Heavy Vignette | Strong edge darkening |
| 65 | Film Grain | Analog film look |
| 66 | Chromatic Aberration | Color fringing |
| 67 | Lens Distortion | Barrel distortion |
| 68 | Posterize | Reduced color levels |
| 69 | Cel Shade | Cartoon shading bands |
| 70 | Lens Flare | Bright streak from light |
| 71 | Edge Detect | Outline/wireframe effect |

### Combined Effects (72–80)

| # | Name | Description |
|---|------|-------------|
| 72 | Retro Gaming | Pixelate + Game Boy palette + border |
| 73 | VHS Aesthetic | Glitch + aberration + grain |
| 74 | Cyberpunk Neon | Bloom + cyberpunk palette + contrast |
| 75 | Lo-fi Print | Halftone + sepia + grain |
| 76 | Surveillance | Night vision + grain + vignette + quantize |
| 77 | Pop Art | Warhol-style color bands |
| 78 | Tiled Grid | 3×2 repeated image |
| 79 | Framed Art | Oil paint + picture frame |
| 80 | Noir Crosshatch | Film noir + crosshatch |

### Render Passes (81–83)

| # | Name | Description |
|---|------|-------------|
| 81 | Depth Pass | Distance-from-camera buffer |
| 82 | Normals Pass | Surface normal directions |
| 83 | Albedo Pass | Base color (no lighting) |

### Tone Mapping (84–87)

| # | Name | Description |
|---|------|-------------|
| 84 | ACES | Academy Color Encoding System |
| 85 | Reinhard | Simple Reinhard mapping |
| 86 | Filmic | Uncharted 2 filmic curve |
| 87 | None | Raw linear (clamped) |

### Special Scenes (88–90)

| # | Name | Description |
|---|------|-------------|
| 88 | Motion Blur | Moving spheres with time sampling |
| 89 | Toon Shading | Cel-shaded scene |
| 90 | Space | Space/starfield scene |
