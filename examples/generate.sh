#!/usr/bin/env bash
#
# Luminara Examples Generator
# Renders a curated gallery of examples demonstrating the ray tracer's capabilities.
#
# Usage:
#   ./examples/generate.sh            # Render all examples
#   ./examples/generate.sh quick      # Fast preview mode (low quality, ~2min total)
#   ./examples/generate.sh <name>     # Render a single example by name
#
# Output goes to examples/renders/

set -euo pipefail
cd "$(dirname "$0")/.."

OUTDIR="examples/renders"
mkdir -p "$OUTDIR"

BIN="cargo run --release --"

# Quality presets
if [[ "${1:-}" == "quick" ]]; then
    W=640; H=360; S=64; EXTRA="-d 16"
    echo "=== Quick preview mode (${W}x${H}, ${S} spp) ==="
    shift || true
else
    W=1280; H=720; S=256; EXTRA="-d 50"
    echo "=== Full showcase mode (${W}x${H}, ${S} spp) ==="
fi

COMMON="-w $W -H $H -s $S $EXTRA -q"
FILTER=""  # optional: only render matching name

if [[ -n "${1:-}" && "${1:-}" != "quick" ]]; then
    FILTER="$1"
fi

rendered=0
skipped=0

render() {
    local name="$1"; shift
    if [[ -n "$FILTER" && "$name" != "$FILTER" ]]; then
        skipped=$((skipped + 1))
        return
    fi
    local out="$OUTDIR/${name}.png"
    echo "  [$((rendered+1))] $name"
    $BIN "$@" $COMMON -o "$out" 2>&1 | grep -E "^(Render|Error)" || true
    rendered=$((rendered + 1))
}

echo ""
echo "--- Scene Showcases ---"

render "01_demo_scene" \
    --seed 42

render "02_cornell_box" \
    examples/scenes/cornell.toml

render "03_showcase" \
    examples/scenes/showcase.toml

render "04_gallery" \
    examples/scenes/gallery.toml

render "05_outdoor" \
    examples/scenes/outdoor.toml

render "06_sunset" \
    examples/scenes/sunset.toml

render "07_architecture" \
    examples/scenes/architecture.toml

render "08_abstract_sdf" \
    examples/scenes/abstract.toml

render "09_shapes" \
    examples/scenes/shapes.toml

render "10_csg_booleans" \
    examples/scenes/csg_demo.toml

echo ""
echo "--- Materials ---"

render "11_pbr_metals" \
    examples/scenes/pbr_showcase.toml

render "12_organic_sss" \
    examples/scenes/organic.toml

render "13_iridescent" \
    examples/scenes/materials_showcase.toml --bloom 0.2

render "14_glass_prism" \
    examples/scenes/prism.toml

echo ""
echo "--- Post-Processing: Artistic Styles ---"

render "15_vintage_photo" \
    examples/scenes/showcase.toml --vintage

render "16_cinematic" \
    examples/scenes/showcase.toml --cinematic

render "17_noir" \
    examples/scenes/showcase.toml --noir

render "18_dreamy" \
    examples/scenes/showcase.toml --dreamy

render "19_retro_crt" \
    examples/scenes/showcase.toml --retro

render "20_comic_book" \
    examples/scenes/showcase.toml --comic

render "21_miniature" \
    examples/scenes/outdoor.toml --miniature

echo ""
echo "--- Post-Processing: Painting & Drawing ---"

render "22_oil_painting" \
    examples/scenes/showcase.toml --oil-paint 4

render "23_watercolor" \
    examples/scenes/showcase.toml --watercolor 5

render "24_pencil_sketch" \
    examples/scenes/showcase.toml --sketch

render "25_crosshatch" \
    examples/scenes/showcase.toml --crosshatch 5

render "26_stipple" \
    examples/scenes/showcase.toml --stipple 4

render "27_pencil_hatching" \
    examples/scenes/showcase.toml --pencil 4

echo ""
echo "--- Post-Processing: Color Effects ---"

render "28_gameboy_palette" \
    examples/scenes/showcase.toml --palette gameboy

render "29_nes_palette" \
    examples/scenes/showcase.toml --palette nes

render "30_cga_palette" \
    examples/scenes/showcase.toml --palette cga

render "31_cyberpunk_palette" \
    examples/scenes/showcase.toml --palette cyberpunk

render "32_inferno_colormap" \
    examples/scenes/showcase.toml --color-map inferno

render "33_viridis_colormap" \
    examples/scenes/showcase.toml --color-map viridis

render "34_thermal_colormap" \
    examples/scenes/showcase.toml --color-map thermal

render "35_neon_colormap" \
    examples/scenes/showcase.toml --color-map neon

render "36_duo_tone_blue_gold" \
    examples/scenes/showcase.toml --duo-tone "0,0,80;255,200,50"

render "37_tri_tone" \
    examples/scenes/showcase.toml --tri-tone "20,0,60;200,50,50;255,220,100"

render "38_split_tone" \
    examples/scenes/showcase.toml --split-tone "40,60,120;255,200,150"

render "39_sepia" \
    examples/scenes/showcase.toml --sepia 0.8

render "40_night_vision" \
    examples/scenes/showcase.toml --night-vision

render "41_color_invert" \
    examples/scenes/showcase.toml --invert

render "42_channel_swap" \
    examples/scenes/showcase.toml --channel-swap gbr

render "43_color_quantize_8" \
    examples/scenes/showcase.toml --quantize 8

render "44_solarize" \
    examples/scenes/showcase.toml --solarize 0.5

echo ""
echo "--- Post-Processing: Distortion & Texture ---"

render "45_glitch" \
    examples/scenes/showcase.toml --glitch 0.6

render "46_mosaic" \
    examples/scenes/showcase.toml --mosaic 12

render "47_pixelate" \
    examples/scenes/showcase.toml --pixelate 8

render "48_hex_pixelate" \
    examples/scenes/showcase.toml --hex-pixelate 10

render "49_dot_matrix" \
    examples/scenes/showcase.toml --dot-matrix 6

render "50_color_halftone" \
    examples/scenes/showcase.toml --color-halftone 5

render "51_emboss" \
    examples/scenes/showcase.toml --emboss 0.8

render "52_swirl" \
    examples/scenes/showcase.toml --swirl 3.0

render "53_wave_ripple" \
    examples/scenes/showcase.toml --wave 10

render "54_fisheye" \
    examples/scenes/showcase.toml --fisheye 0.6

render "55_kaleidoscope" \
    examples/scenes/showcase.toml --kaleidoscope 8

render "56_frosted_glass" \
    examples/scenes/showcase.toml --frosted-glass 5

render "57_spin_blur" \
    examples/scenes/showcase.toml --spin-blur 5

render "58_radial_blur" \
    examples/scenes/showcase.toml --radial-blur 0.4

echo ""
echo "--- Post-Processing: Enhancement & Grading ---"

render "59_bloom_glow" \
    examples/scenes/showcase.toml --bloom 0.5

render "60_high_contrast" \
    examples/scenes/showcase.toml --contrast 1.8 --saturation 1.3

render "61_desaturated" \
    examples/scenes/showcase.toml --saturation 0.0

render "62_warm_tint" \
    examples/scenes/showcase.toml --warm --vignette 0.3

render "63_cool_tint" \
    examples/scenes/showcase.toml --cool --vignette 0.3

render "64_heavy_vignette" \
    examples/scenes/showcase.toml --vignette 0.8

render "65_film_grain" \
    examples/scenes/showcase.toml --grain 0.15 --vignette 0.2

render "66_chromatic_aberration" \
    examples/scenes/showcase.toml --ca 0.01

render "67_lens_distortion" \
    examples/scenes/showcase.toml --distortion 0.5

render "68_posterize" \
    examples/scenes/showcase.toml --posterize 4

render "69_cel_shade" \
    examples/scenes/showcase.toml --cel-shade 5

render "70_lens_flare" \
    examples/scenes/showcase.toml --lens-flare 0.5

render "71_edge_detect" \
    examples/scenes/showcase.toml --edge-detect 1.0

echo ""
echo "--- Post-Processing: Combined Effects ---"

render "72_retro_gaming" \
    examples/scenes/showcase.toml --pixelate 6 --palette gameboy --border 4 --border-color "0.1,0.2,0.1"

render "73_vhs_aesthetic" \
    examples/scenes/showcase.toml --glitch 0.3 --ca 0.008 --grain 0.1 --saturation 0.7 --contrast 1.2

render "74_cyberpunk_neon" \
    examples/scenes/showcase.toml --bloom 0.4 --palette cyberpunk --contrast 1.5 --vignette 0.4

render "75_lo_fi_print" \
    examples/scenes/showcase.toml --color-halftone 4 --sepia 0.3 --grain 0.05

render "76_surveillance_cam" \
    examples/scenes/showcase.toml --night-vision --grain 0.2 --vignette 0.6 --quantize 16

render "77_pop_art" \
    examples/scenes/showcase.toml --pop-art 4

render "78_tiled_grid" \
    examples/scenes/showcase.toml --tile 3,2

render "79_framed_art" \
    examples/scenes/showcase.toml --oil-paint 3 --frame 20 --saturation 1.2

render "80_noir_crosshatch" \
    examples/scenes/showcase.toml --noir --crosshatch 4

echo ""
echo "--- Render Passes ---"

render "81_depth_pass" \
    examples/scenes/showcase.toml --save-depth "$OUTDIR/81_depth_pass_raw.png"

render "82_normals_pass" \
    examples/scenes/showcase.toml --save-normals "$OUTDIR/82_normals_pass_raw.png"

render "83_albedo_pass" \
    examples/scenes/showcase.toml --save-albedo "$OUTDIR/83_albedo_pass_raw.png"

echo ""
echo "--- Tone Mapping Comparison ---"

render "84_tonemapping_aces" \
    examples/scenes/showcase.toml --tone-map aces

render "85_tonemapping_reinhard" \
    examples/scenes/showcase.toml --tone-map reinhard

render "86_tonemapping_filmic" \
    examples/scenes/showcase.toml --tone-map filmic

render "87_tonemapping_none" \
    examples/scenes/showcase.toml --tone-map none

echo ""
echo "--- Special Scenes ---"

render "88_motion_blur" \
    examples/scenes/motion.toml

render "89_toon_shading" \
    examples/scenes/toon.toml

render "90_space" \
    examples/scenes/space.toml

echo ""
echo "========================================"
echo "  Showcase complete!"
echo "  Rendered: $rendered examples"
if [[ $skipped -gt 0 ]]; then
    echo "  Skipped:  $skipped (filtered)"
fi
echo "  Output:   $OUTDIR/"
echo "========================================"
