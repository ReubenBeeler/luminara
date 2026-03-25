use serde::Deserialize;

use crate::aabb::Aabb;
use crate::annulus::Annulus;
use crate::bvh::BvhNode;
use crate::camera::{Camera, CameraConfig};
use crate::bump::BumpMap;
use crate::capsule::Capsule;
use crate::transform::{NonUniformScale, RotateX, RotateY, RotateZ, Scale, Translate};
use crate::cone::Cone;
use crate::constant_medium::ConstantMedium;
use crate::cylinder::Cylinder;
use crate::disk::Disk;
use crate::ellipsoid::Ellipsoid;
use crate::hemisphere::Hemisphere;
use crate::hit::{HitRecord, Hittable, HittableList};
use crate::material::{Anisotropic, Blend, Clearcoat, Dielectric, Emissive, Iridescent, Lambertian, Metal, Microfacet, Subsurface, Toon, Translucent, Transparent, Velvet};
use crate::texture::{Brick, Camo, Checker, Cloud, ColorRamp, Dots, Fbm, GradientTexture, Grid, Hexgrid, ImageTexture, Lava, Marble, MixTexture, Noise, Plasma, Rings, Rust, Spiral, Stripe, Terrain, TransformedTexture, TriPlanar, Turbulence, UvChecker, Voronoi, Wavy, Wood};
use crate::plane::Plane;
use crate::quad::Quad;
use crate::ray::Ray;
use crate::rect::{XyRect, XzRect, YzRect, make_box};
use crate::render::{Background, LightInfo, RenderConfig};
use crate::sphere::{MovingSphere, Sphere};
use crate::obj;
use crate::mobius::Mobius;
use crate::rounded_box::RoundedBox;
use crate::spring::Spring;
use crate::superellipsoid::Superellipsoid;
use crate::torus::Torus;
use crate::triangle::Triangle;
use crate::vec3::{Color, Point3, Vec3};

// --- TOML scene description types ---

#[derive(Deserialize)]
pub struct SceneFile {
    pub render: Option<RenderSettings>,
    pub camera: Option<CameraSettings>,
    #[serde(default)]
    pub sphere: Vec<SphereDesc>,
    #[serde(default)]
    pub moving_sphere: Vec<MovingSphereDesc>,
    #[serde(default)]
    pub plane: Vec<PlaneDesc>,
    #[serde(default)]
    pub triangle: Vec<TriangleDesc>,
    #[serde(default)]
    pub mesh: Vec<MeshDesc>,
    #[serde(default)]
    pub torus: Vec<TorusDesc>,
    #[serde(default)]
    pub capsule: Vec<CapsuleDesc>,
    #[serde(default)]
    pub ellipsoid: Vec<EllipsoidDesc>,
    #[serde(default)]
    pub hemisphere: Vec<HemisphereDesc>,
    #[serde(default)]
    pub light: Vec<LightDesc>,
    #[serde(default)]
    pub fog: Vec<FogDesc>,
    #[serde(default)]
    pub cone: Vec<ConeDesc>,
    #[serde(default)]
    pub cylinder: Vec<CylinderDesc>,
    #[serde(default)]
    pub annulus: Vec<AnnulusDesc>,
    #[serde(default)]
    pub disk: Vec<DiskDesc>,
    #[serde(default)]
    #[serde(rename = "box")]
    pub aabb_box: Vec<BoxDesc>,
    #[serde(default)]
    pub quad: Vec<QuadDesc>,
    #[serde(default)]
    pub rect_xy: Vec<RectXyDesc>,
    #[serde(default)]
    pub rect_xz: Vec<RectXzDesc>,
    #[serde(default)]
    pub rect_yz: Vec<RectYzDesc>,
    #[serde(default)]
    pub rounded_box: Vec<RoundedBoxDesc>,
    #[serde(default)]
    pub superellipsoid: Vec<SuperellipsoidDesc>,
    #[serde(default)]
    pub spring: Vec<SpringDesc>,
    #[serde(default)]
    pub mobius: Vec<MobiusDesc>,
    #[serde(default)]
    pub csg: Vec<CsgDesc>,
}

#[derive(Deserialize)]
pub struct RenderSettings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub samples: Option<u32>,
    pub max_depth: Option<u32>,
    pub seed: Option<u64>,
    pub exposure: Option<f64>,
    pub auto_exposure: Option<bool>,
    pub denoise: Option<bool>,
    pub bloom: Option<f64>,
    pub vignette: Option<f64>,
    pub grain: Option<f64>,
    pub saturation: Option<f64>,
    pub contrast: Option<f64>,
    pub white_balance: Option<f64>,
    pub sharpen: Option<f64>,
    pub hue_shift: Option<f64>,
    pub dither: Option<bool>,
    pub gamma: Option<f64>,
    pub pixel_filter: Option<String>,
    pub adaptive: Option<bool>,
    pub adaptive_threshold: Option<f64>,
    pub firefly_filter: Option<f64>,
    pub lens_distortion: Option<f64>,
    pub chromatic_aberration: Option<f64>,
    pub tone_map: Option<String>,
    pub posterize: Option<u32>,
    pub sepia: Option<f64>,
    pub edge_detect: Option<f64>,
    pub pixelate: Option<u32>,
    pub invert: Option<bool>,
    pub scanlines: Option<f64>,
    pub threshold: Option<f64>,
    pub blur: Option<f64>,
    pub tilt_shift: Option<f64>,
    pub grade_shadows: Option<[f64; 3]>,
    pub grade_highlights: Option<[f64; 3]>,
    pub halftone: Option<u32>,
    pub emboss: Option<f64>,
    pub oil_paint: Option<u32>,
    pub color_map: Option<String>,
    pub solarize: Option<f64>,
    pub duo_tone: Option<String>,
    pub sketch: Option<bool>,
    pub median: Option<u32>,
    pub crosshatch: Option<u32>,
    pub glitch: Option<f64>,
    pub quantize: Option<u32>,
    pub tint: Option<[f64; 3]>,
    pub palette: Option<String>,
    pub radial_blur: Option<f64>,
    pub border: Option<u32>,
    pub border_color: Option<[f64; 3]>,
    pub resize: Option<[u32; 2]>,
    pub rotate: Option<u32>,
    pub background: Option<BackgroundDesc>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum BackgroundDesc {
    #[serde(alias = "sky")]
    Sky,
    #[serde(alias = "solid")]
    Solid { color: [f64; 3] },
    #[serde(alias = "gradient")]
    Gradient { bottom: [f64; 3], top: [f64; 3] },
    #[serde(alias = "black")]
    Black,
    #[serde(alias = "sunset")]
    Sunset,
    #[serde(alias = "sun")]
    Sun {
        direction: [f64; 3],
        sun_color: Option<[f64; 3]>,
        intensity: Option<f64>,
        sky_color: Option<[f64; 3]>,
    },
    #[serde(alias = "starfield")]
    Starfield {
        star_density: Option<f64>,
        star_brightness: Option<f64>,
    },
    #[serde(alias = "env_map")]
    EnvMap {
        file: String,
        intensity: Option<f64>,
    },
}

#[derive(Deserialize)]
pub struct CameraSettings {
    pub look_from: Option<[f64; 3]>,
    pub look_at: Option<[f64; 3]>,
    pub vup: Option<[f64; 3]>,
    pub vfov: Option<f64>,
    pub aperture: Option<f64>,
    pub focus_dist: Option<f64>,
}

#[derive(Deserialize)]
pub struct SphereDesc {
    pub center: [f64; 3],
    pub radius: f64,
    pub material: MaterialDesc,
    pub bump_strength: Option<f64>,
    pub bump_scale: Option<f64>,
    pub normal_map: Option<String>,
    pub normal_map_strength: Option<f64>,
    pub rotate_x: Option<f64>,
    pub rotate_y: Option<f64>,
    pub rotate_z: Option<f64>,
    pub translate: Option<[f64; 3]>,
    pub scale: Option<f64>,
}

#[derive(Deserialize)]
pub struct MovingSphereDesc {
    pub center0: [f64; 3],
    pub center1: [f64; 3],
    pub radius: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct PlaneDesc {
    pub point: [f64; 3],
    pub normal: [f64; 3],
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct TriangleDesc {
    pub v0: [f64; 3],
    pub v1: [f64; 3],
    pub v2: [f64; 3],
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct MeshDesc {
    pub file: String,
    pub material: MaterialDesc,
    pub scale: Option<f64>,
    pub offset: Option<[f64; 3]>,
}

#[derive(Deserialize)]
pub struct AnnulusDesc {
    pub center: [f64; 3],
    pub normal: [f64; 3],
    pub inner_radius: f64,
    pub outer_radius: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct DiskDesc {
    pub center: [f64; 3],
    pub normal: [f64; 3],
    pub radius: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct TorusDesc {
    pub center: [f64; 3],
    pub major_radius: f64,
    pub minor_radius: f64,
    pub material: MaterialDesc,
    pub bump_strength: Option<f64>,
    pub bump_scale: Option<f64>,
}

#[derive(Deserialize)]
pub struct RoundedBoxDesc {
    pub center: [f64; 3],
    pub half_size: [f64; 3],
    pub radius: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct MobiusDesc {
    pub center: [f64; 3],
    pub radius: f64,
    pub width: f64,
    pub thickness: Option<f64>,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct SpringDesc {
    pub center: [f64; 3],
    pub coil_radius: f64,
    pub tube_radius: f64,
    pub pitch: f64,
    pub turns: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct SuperellipsoidDesc {
    pub center: [f64; 3],
    pub scale: [f64; 3],
    pub e1: f64,
    pub e2: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct CapsuleDesc {
    pub center: [f64; 3],
    pub radius: f64,
    pub height: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct EllipsoidDesc {
    pub center: [f64; 3],
    pub radii: [f64; 3],
    pub material: MaterialDesc,
    pub bump_strength: Option<f64>,
    pub bump_scale: Option<f64>,
}

#[derive(Deserialize)]
pub struct LightDesc {
    pub position: [f64; 3],
    pub color: Option<[f64; 3]>,
    pub intensity: Option<f64>,
    pub radius: Option<f64>,
}

#[derive(Deserialize)]
pub struct HemisphereDesc {
    pub center: [f64; 3],
    pub radius: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct FogDesc {
    pub center: [f64; 3],
    pub radius: f64,
    pub density: f64,
    pub color: [f64; 3],
}

#[derive(Deserialize)]
pub struct ConeDesc {
    pub center: [f64; 3],
    pub radius: f64,
    pub height: f64,
    pub material: MaterialDesc,
    pub rotate_x: Option<f64>,
    pub rotate_y: Option<f64>,
    pub rotate_z: Option<f64>,
    pub translate: Option<[f64; 3]>,
}

#[derive(Deserialize)]
pub struct CylinderDesc {
    pub center: [f64; 3],
    pub radius: f64,
    pub height: f64,
    pub material: MaterialDesc,
    pub bump_strength: Option<f64>,
    pub bump_scale: Option<f64>,
    pub normal_map: Option<String>,
    pub normal_map_strength: Option<f64>,
    pub rotate_x: Option<f64>,
    pub rotate_y: Option<f64>,
    pub rotate_z: Option<f64>,
    pub translate: Option<[f64; 3]>,
}

#[derive(Deserialize)]
pub struct BoxDesc {
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub material: MaterialDesc,
    pub rotate_x: Option<f64>,
    pub rotate_y: Option<f64>,
    pub rotate_z: Option<f64>,
    pub translate: Option<[f64; 3]>,
    pub scale: Option<f64>,
    pub scale_xyz: Option<[f64; 3]>,
}

#[derive(Deserialize)]
pub struct QuadDesc {
    pub q: [f64; 3],
    pub u: [f64; 3],
    pub v: [f64; 3],
    pub material: MaterialDesc,
    pub normal_map: Option<String>,
    pub normal_map_strength: Option<f64>,
}

#[derive(Deserialize)]
pub struct RectXyDesc {
    pub x: [f64; 2],
    pub y: [f64; 2],
    pub k: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct RectXzDesc {
    pub x: [f64; 2],
    pub z: [f64; 2],
    pub k: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct RectYzDesc {
    pub y: [f64; 2],
    pub z: [f64; 2],
    pub k: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct CsgDesc {
    pub operation: String,
    pub a: CsgChild,
    pub b: CsgChild,
}

#[derive(Deserialize)]
pub struct CsgChild {
    pub shape: String,
    pub center: [f64; 3],
    pub radius: Option<f64>,
    pub size: Option<[f64; 3]>,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum MaterialDesc {
    #[serde(alias = "lambertian", alias = "matte")]
    Lambertian { color: [f64; 3] },
    #[serde(alias = "metal")]
    Metal { color: [f64; 3], fuzz: Option<f64> },
    #[serde(alias = "dielectric")]
    Dielectric {
        refraction_index: f64,
        tint: Option<[f64; 3]>,
        roughness: Option<f64>,
        dispersion: Option<f64>,
    },
    #[serde(alias = "emissive")]
    Emissive {
        color: [f64; 3],
        intensity: Option<f64>,
        texture: Option<String>,
    },
    #[serde(alias = "microfacet", alias = "pbr")]
    Microfacet {
        color: [f64; 3],
        roughness: Option<f64>,
        metallic: Option<f64>,
        texture: Option<String>,
    },
    #[serde(alias = "checker")]
    Checker {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
    },
    #[serde(alias = "marble")]
    Marble {
        color: [f64; 3],
        scale: Option<f64>,
    },
    #[serde(alias = "turbulence")]
    Turbulence {
        color: [f64; 3],
        scale: Option<f64>,
    },
    #[serde(alias = "image")]
    Image {
        file: String,
        uv_offset: Option<[f64; 2]>,
        uv_rotation: Option<f64>,
        uv_tile: Option<[f64; 2]>,
    },
    #[serde(alias = "mirror")]
    Mirror,
    #[serde(alias = "glass")]
    Glass,
    #[serde(alias = "wood")]
    Wood {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
    },
    #[serde(alias = "rings")]
    Rings {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
    },
    #[serde(alias = "toon", alias = "cel")]
    Toon {
        color: [f64; 3],
        bands: Option<u32>,
        specular: Option<f64>,
    },
    #[serde(alias = "anisotropic", alias = "brushed")]
    Anisotropic {
        color: [f64; 3],
        roughness_u: Option<f64>,
        roughness_v: Option<f64>,
        tangent_axis: Option<usize>,
    },
    /// Blackbody emitter — specify light color by temperature in Kelvin
    #[serde(alias = "blackbody")]
    Blackbody {
        temperature: f64,
        intensity: Option<f64>,
    },
    #[serde(alias = "clearcoat")]
    Clearcoat {
        color: [f64; 3],
        coat_gloss: Option<f64>,
        coat_ior: Option<f64>,
    },
    #[serde(alias = "velvet")]
    Velvet {
        color: [f64; 3],
        sheen: Option<f64>,
    },
    #[serde(alias = "iridescent")]
    Iridescent {
        color: Option<[f64; 3]>,
        thickness: Option<f64>,
        film_ior: Option<f64>,
        roughness: Option<f64>,
    },
    #[serde(alias = "translucent")]
    Translucent {
        color: [f64; 3],
        translucency: Option<f64>,
        scatter_width: Option<f64>,
    },
    #[serde(alias = "subsurface", alias = "sss")]
    Subsurface {
        color: [f64; 3],
        mean_free_path: Option<f64>,
        scatter_color: Option<[f64; 3]>,
    },
    #[serde(alias = "blend")]
    BlendMat {
        material_a: Box<MaterialDesc>,
        material_b: Box<MaterialDesc>,
        ratio: Option<f64>,
    },
    #[serde(alias = "opacity", alias = "alpha")]
    Opacity {
        material: Box<MaterialDesc>,
        opacity: f64,
    },
    #[serde(alias = "dots")]
    Dots {
        dot_color: [f64; 3],
        bg_color: [f64; 3],
        scale: Option<f64>,
        radius: Option<f64>,
    },
    #[serde(alias = "grid")]
    Grid {
        line_color: [f64; 3],
        bg_color: [f64; 3],
        scale: Option<f64>,
        line_width: Option<f64>,
    },
    #[serde(alias = "uv_checker")]
    UvChecker {
        color1: [f64; 3],
        color2: [f64; 3],
        frequency: Option<f64>,
    },
    #[serde(alias = "gradient_tex")]
    GradientTex {
        color1: [f64; 3],
        color2: [f64; 3],
        axis: Option<String>,
        min: Option<f64>,
        max: Option<f64>,
    },
    #[serde(alias = "stripe")]
    Stripe {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
        axis: Option<String>,
    },
    #[serde(alias = "voronoi")]
    Voronoi {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
    },
    #[serde(alias = "noise")]
    Noise {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
    },
    #[serde(alias = "spiral")]
    Spiral {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
        arms: Option<u32>,
    },
    #[serde(alias = "hexgrid")]
    Hexgrid {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
        line_width: Option<f64>,
    },
    #[serde(alias = "plasma")]
    Plasma {
        scale: Option<f64>,
    },
    #[serde(alias = "terrain", alias = "earth")]
    Terrain {
        scale: Option<f64>,
    },
    #[serde(alias = "rust", alias = "patina", alias = "oxidized")]
    Rust {
        scale: Option<f64>,
    },
    #[serde(alias = "brick", alias = "bricks")]
    Brick {
        brick_color: Option<[f64; 3]>,
        mortar_color: Option<[f64; 3]>,
        scale: Option<f64>,
        mortar_width: Option<f64>,
    },
    #[serde(alias = "camo", alias = "camouflage")]
    Camo {
        color1: Option<[f64; 3]>,
        color2: Option<[f64; 3]>,
        color3: Option<[f64; 3]>,
        scale: Option<f64>,
    },
    #[serde(alias = "lava", alias = "magma")]
    Lava {
        scale: Option<f64>,
    },
    #[serde(alias = "cloud", alias = "clouds")]
    Cloud {
        color: Option<[f64; 3]>,
        sky_color: Option<[f64; 3]>,
        scale: Option<f64>,
        density: Option<f64>,
        octaves: Option<u32>,
    },
    #[serde(alias = "tri_planar", alias = "triplanar")]
    TriPlanar {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
        sharpness: Option<f64>,
    },
    #[serde(alias = "mix_color")]
    MixColor {
        color1: [f64; 3],
        color2: [f64; 3],
        factor: Option<f64>,
    },
    #[serde(alias = "wavy", alias = "wave")]
    Wavy {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
        waves: Option<u32>,
    },
    #[serde(alias = "fbm", alias = "fractal")]
    Fbm {
        color1: [f64; 3],
        color2: [f64; 3],
        scale: Option<f64>,
        octaves: Option<u32>,
    },
    #[serde(alias = "color_ramp", alias = "ramp")]
    ColorRamp {
        /// Array of [position, r, g, b] stops
        stops: Vec<[f64; 4]>,
        axis: Option<usize>,
        min_val: Option<f64>,
        max_val: Option<f64>,
    },
}

/// A scene world that uses BVH for bounded objects and linear scan for unbounded ones.
pub struct SceneWorld {
    bvh: Option<Box<dyn Hittable>>,
    unbounded: Vec<Box<dyn Hittable>>,
    pub bounded_count: usize,
    pub unbounded_count: usize,
    pub lights: Vec<LightInfo>,
}

impl SceneWorld {
    /// Build a SceneWorld from a HittableList, partitioning objects into
    /// bounded (accelerated via BVH) and unbounded (tested linearly).
    pub fn from_list(list: HittableList, lights: Vec<LightInfo>) -> Self {
        let mut bounded = Vec::new();
        let mut unbounded = Vec::new();

        for obj in list.objects {
            if obj.bounding_box().is_some() {
                bounded.push(obj);
            } else {
                unbounded.push(obj);
            }
        }

        let bounded_count = bounded.len();
        let unbounded_count = unbounded.len();

        let bvh = if bounded.is_empty() {
            None
        } else {
            Some(BvhNode::build(bounded))
        };

        Self { bvh, unbounded, bounded_count, unbounded_count, lights }
    }

    /// Total number of objects in the scene.
    pub fn object_count(&self) -> usize {
        self.bounded_count + self.unbounded_count
    }
}

impl Hittable for SceneWorld {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let mut closest = t_max;
        let mut best_hit = None;

        if let Some(hit) = self.bvh.as_ref().and_then(|bvh| bvh.hit(ray, t_min, closest)) {
            closest = hit.t;
            best_hit = Some(hit);
        }

        for obj in &self.unbounded {
            if let Some(hit) = obj.hit(ray, t_min, closest) {
                closest = hit.t;
                best_hit = Some(hit);
            }
        }

        best_hit
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.bvh.as_ref().and_then(|b| b.bounding_box())
    }
}

fn arr_to_vec3(a: [f64; 3]) -> Vec3 {
    Vec3::new(a[0], a[1], a[2])
}

/// Load a scene from a TOML string.
pub fn load_scene(toml_str: &str) -> Result<(RenderConfig, Camera, SceneWorld), String> {
    let scene: SceneFile = toml::from_str(toml_str).map_err(|e| format!("TOML parse error: {e}"))?;

    // Render config
    let mut render_config = RenderConfig::default();
    if let Some(r) = &scene.render {
        if let Some(w) = r.width {
            render_config.width = w;
        }
        if let Some(h) = r.height {
            render_config.height = h;
        }
        if let Some(s) = r.samples {
            render_config.samples_per_pixel = s;
        }
        if let Some(d) = r.max_depth {
            render_config.max_depth = d;
        }
        if let Some(s) = r.seed {
            render_config.seed = s;
        }
        if let Some(e) = r.exposure {
            render_config.exposure = e;
        }
        if let Some(ae) = r.auto_exposure {
            render_config.auto_exposure = ae;
        }
        if let Some(dn) = r.denoise {
            render_config.denoise = dn;
        }
        if let Some(bloom) = r.bloom {
            render_config.bloom = bloom;
        }
        if let Some(vignette) = r.vignette {
            render_config.vignette = vignette;
        }
        if let Some(grain) = r.grain {
            render_config.grain = grain;
        }
        if let Some(saturation) = r.saturation {
            render_config.saturation = saturation;
        }
        if let Some(contrast) = r.contrast {
            render_config.contrast = contrast;
        }
        if let Some(wb) = r.white_balance {
            render_config.white_balance = wb;
        }
        if let Some(sharpen) = r.sharpen {
            render_config.sharpen = sharpen;
        }
        if let Some(hue_shift) = r.hue_shift {
            render_config.hue_shift = hue_shift;
        }
        if let Some(dither) = r.dither {
            render_config.dither = dither;
        }
        if let Some(gamma) = r.gamma {
            render_config.gamma = gamma;
        }
        if let Some(ref pf) = r.pixel_filter {
            render_config.pixel_filter = match pf.as_str() {
                "triangle" | "tent" => crate::render::PixelFilter::Triangle,
                "gaussian" | "gauss" => crate::render::PixelFilter::Gaussian,
                "mitchell" => crate::render::PixelFilter::Mitchell,
                _ => crate::render::PixelFilter::Box,
            };
        }
        if let Some(adaptive) = r.adaptive {
            render_config.adaptive = adaptive;
        }
        if let Some(threshold) = r.adaptive_threshold {
            render_config.adaptive_threshold = threshold;
        }
        if let Some(ff) = r.firefly_filter {
            render_config.firefly_filter = ff;
        }
        if let Some(ld) = r.lens_distortion {
            render_config.lens_distortion = ld;
        }
        if let Some(ca) = r.chromatic_aberration {
            render_config.chromatic_aberration = ca;
        }
        if let Some(p) = r.posterize {
            render_config.posterize = p;
        }
        if let Some(s) = r.sepia {
            render_config.sepia = s;
        }
        if let Some(ed) = r.edge_detect {
            render_config.edge_detect = ed;
        }
        if let Some(px) = r.pixelate {
            render_config.pixelate = px;
        }
        if let Some(inv) = r.invert {
            render_config.invert = inv;
        }
        if let Some(sl) = r.scanlines {
            render_config.scanlines = sl;
        }
        if let Some(th) = r.threshold {
            render_config.threshold = th;
        }
        if let Some(bl) = r.blur {
            render_config.blur = bl;
        }
        if let Some(ts) = r.tilt_shift {
            render_config.tilt_shift = ts;
        }
        if let Some(gs) = r.grade_shadows {
            render_config.grade_shadows = gs;
        }
        if let Some(gh) = r.grade_highlights {
            render_config.grade_highlights = gh;
        }
        if let Some(ht) = r.halftone {
            render_config.halftone = ht;
        }
        if let Some(em) = r.emboss {
            render_config.emboss = em;
        }
        if let Some(op) = r.oil_paint {
            render_config.oil_paint = op;
        }
        if let Some(ref cm) = r.color_map {
            render_config.color_map = cm.clone();
        }
        if let Some(s) = r.solarize {
            render_config.solarize = s;
        }
        if let Some(ref dt) = r.duo_tone {
            render_config.duo_tone = dt.clone();
        }
        if r.sketch == Some(true) {
            render_config.sketch = true;
        }
        if let Some(m) = r.median {
            render_config.median = m;
        }
        if let Some(ch) = r.crosshatch {
            render_config.crosshatch = ch;
        }
        if let Some(gl) = r.glitch {
            render_config.glitch = gl;
        }
        if let Some(q) = r.quantize {
            render_config.quantize = q;
        }
        if let Some(t) = r.tint {
            render_config.tint = t;
        }
        if let Some(ref p) = r.palette {
            render_config.palette = p.clone();
        }
        if let Some(rb) = r.radial_blur {
            render_config.radial_blur = rb;
        }
        if let Some(b) = r.border {
            render_config.border = b;
        }
        if let Some(bc) = r.border_color {
            render_config.border_color = bc;
        }
        if let Some(rs) = r.resize {
            render_config.resize = rs;
        }
        if let Some(rot) = r.rotate {
            render_config.rotate = rot;
        }
        if let Some(ref tm) = r.tone_map {
            render_config.tone_map = match tm.as_str() {
                "reinhard" => crate::render::ToneMap::Reinhard,
                "filmic" | "uncharted2" => crate::render::ToneMap::Filmic,
                "none" => crate::render::ToneMap::None,
                _ => crate::render::ToneMap::Aces,
            };
        }
        if let Some(bg) = &r.background {
            render_config.background = match bg {
                BackgroundDesc::Sky => Background::SkyGradient,
                BackgroundDesc::Solid { color } => {
                    Background::Solid(Color::new(color[0], color[1], color[2]))
                }
                BackgroundDesc::Gradient { bottom, top } => Background::Gradient {
                    bottom: Color::new(bottom[0], bottom[1], bottom[2]),
                    top: Color::new(top[0], top[1], top[2]),
                },
                BackgroundDesc::Black => Background::Solid(Color::ZERO),
                BackgroundDesc::Sunset => Background::Sun {
                    direction: Vec3::new(0.3, 0.15, -1.0),
                    sun_color: Color::new(1.0, 0.6, 0.2),
                    sun_intensity: 30.0,
                    sky_color: Color::new(0.8, 0.4, 0.2),
                },
                BackgroundDesc::Sun { direction, sun_color, intensity, sky_color } => {
                    Background::Sun {
                        direction: arr_to_vec3(*direction),
                        sun_color: sun_color.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(1.0, 0.95, 0.85)),
                        sun_intensity: intensity.unwrap_or(20.0),
                        sky_color: sky_color.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(0.5, 0.7, 1.0)),
                    }
                }
                BackgroundDesc::Starfield { star_density, star_brightness } => {
                    Background::Starfield {
                        star_density: star_density.unwrap_or(1.0),
                        star_brightness: star_brightness.unwrap_or(5.0),
                    }
                }
                BackgroundDesc::EnvMap { file, intensity } => {
                    Background::load_env_map(file, intensity.unwrap_or(1.0))?
                }
            };
        }
    }

    // Camera
    let mut cam_config = CameraConfig {
        aspect_ratio: render_config.width as f64 / render_config.height as f64,
        ..CameraConfig::default()
    };
    if let Some(c) = &scene.camera {
        if let Some(lf) = c.look_from {
            cam_config.look_from = arr_to_vec3(lf);
        }
        if let Some(la) = c.look_at {
            cam_config.look_at = arr_to_vec3(la);
        }
        if let Some(vup) = c.vup {
            cam_config.vup = arr_to_vec3(vup);
        }
        if let Some(vfov) = c.vfov {
            cam_config.vfov_degrees = vfov;
        }
        if let Some(ap) = c.aperture {
            cam_config.aperture = ap;
        }
        if let Some(fd) = c.focus_dist {
            cam_config.focus_dist = fd;
        }
    }
    let camera = Camera::new(cam_config);

    // Objects
    let mut world = HittableList::new();
    let mut lights = Vec::new();

    for s in &scene.sphere {
        // Track emissive spheres for NEE
        match &s.material {
            MaterialDesc::Emissive { color, intensity, .. } => {
                let emission_color = Color::new(color[0], color[1], color[2]);
                let int = intensity.unwrap_or(1.0);
                lights.push(LightInfo::Sphere {
                    center: arr_to_vec3(s.center),
                    radius: s.radius,
                    emission: emission_color * int,
                });
            }
            MaterialDesc::Blackbody { temperature, intensity } => {
                let emission_color = blackbody_to_rgb(*temperature);
                let int = intensity.unwrap_or(1.0);
                lights.push(LightInfo::Sphere {
                    center: arr_to_vec3(s.center),
                    radius: s.radius,
                    emission: emission_color * int,
                });
            }
            _ => {}
        }
        let mat = build_material(&s.material);
        let mut obj: Box<dyn Hittable> = Box::new(Sphere::new(arr_to_vec3(s.center), s.radius, mat));
        if let Some(strength) = s.bump_strength {
            let scale = s.bump_scale.unwrap_or(4.0);
            obj = Box::new(BumpMap::new(obj, strength, scale));
        }
        if let Some(ref nmap_path) = s.normal_map {
            let strength = s.normal_map_strength.unwrap_or(1.0);
            match crate::normal_map::NormalMap::load_image(nmap_path) {
                Ok(nmap_data) => obj = Box::new(crate::normal_map::NormalMap::wrap(obj, nmap_data, strength)),
                Err(e) => eprintln!("Warning: {e}"),
            }
        }
        if let Some(angle) = s.rotate_x {
            obj = Box::new(RotateX::new(obj, angle));
        }
        if let Some(angle) = s.rotate_y {
            obj = Box::new(RotateY::new(obj, angle));
        }
        if let Some(angle) = s.rotate_z {
            obj = Box::new(RotateZ::new(obj, angle));
        }
        if let Some(offset) = s.translate {
            obj = Box::new(Translate::new(obj, arr_to_vec3(offset)));
        }
        if let Some(s_val) = s.scale {
            obj = Box::new(Scale::new(obj, s_val));
        }
        world.add(obj);
    }

    for ms in &scene.moving_sphere {
        let mat = build_material(&ms.material);
        world.add(Box::new(MovingSphere::new(
            arr_to_vec3(ms.center0),
            arr_to_vec3(ms.center1),
            ms.radius,
            mat,
        )));
    }

    for p in &scene.plane {
        let mat = build_material(&p.material);
        world.add(Box::new(Plane::new(
            arr_to_vec3(p.point),
            arr_to_vec3(p.normal),
            mat,
        )));
    }

    for t in &scene.torus {
        let mat = build_material(&t.material);
        let obj: Box<dyn Hittable> = Box::new(Torus::new(arr_to_vec3(t.center), t.major_radius, t.minor_radius, mat));
        if let Some(strength) = t.bump_strength {
            let scale = t.bump_scale.unwrap_or(4.0);
            world.add(Box::new(BumpMap::new(obj, strength, scale)));
        } else {
            world.add(obj);
        }
    }

    for rb in &scene.rounded_box {
        let mat = build_material(&rb.material);
        world.add(Box::new(RoundedBox::new(
            arr_to_vec3(rb.center),
            arr_to_vec3(rb.half_size),
            rb.radius,
            mat,
        )));
    }

    for mb in &scene.mobius {
        let mat = build_material(&mb.material);
        world.add(Box::new(Mobius::new(
            arr_to_vec3(mb.center),
            mb.radius,
            mb.width,
            mb.thickness.unwrap_or(0.05),
            mat,
        )));
    }

    for sp in &scene.spring {
        let mat = build_material(&sp.material);
        world.add(Box::new(Spring::new(
            arr_to_vec3(sp.center),
            sp.coil_radius,
            sp.tube_radius,
            sp.pitch,
            sp.turns,
            mat,
        )));
    }

    for se in &scene.superellipsoid {
        let mat = build_material(&se.material);
        world.add(Box::new(Superellipsoid::new(
            arr_to_vec3(se.center),
            arr_to_vec3(se.scale),
            se.e1,
            se.e2,
            mat,
        )));
    }

    for c in &scene.capsule {
        let center = arr_to_vec3(c.center);
        world.add(Box::new(Capsule::new(
            center,
            c.radius,
            c.height,
            || build_material(&c.material),
        )));
    }

    for e in &scene.ellipsoid {
        let mat = build_material(&e.material);
        let obj: Box<dyn Hittable> = Box::new(Ellipsoid::new(arr_to_vec3(e.center), arr_to_vec3(e.radii), mat));
        if let Some(strength) = e.bump_strength {
            let scale = e.bump_scale.unwrap_or(4.0);
            world.add(Box::new(BumpMap::new(obj, strength, scale)));
        } else {
            world.add(obj);
        }
    }

    for l in &scene.light {
        let color = l.color.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(1.0, 1.0, 1.0));
        let intensity = l.intensity.unwrap_or(10.0);
        let radius = l.radius.unwrap_or(0.1);
        let center = arr_to_vec3(l.position);
        world.add(Box::new(Sphere::new(
            center,
            radius,
            Box::new(Emissive::new(color, intensity)),
        )));
        lights.push(LightInfo::Sphere {
            center,
            radius,
            emission: color * intensity,
        });
    }

    for h in &scene.hemisphere {
        let mat = build_material(&h.material);
        world.add(Box::new(Hemisphere::new(arr_to_vec3(h.center), h.radius, mat)));
    }

    for f in &scene.fog {
        let boundary = Box::new(Sphere::new(
            arr_to_vec3(f.center),
            f.radius,
            Box::new(Lambertian::new(Color::ZERO)), // dummy material
        ));
        world.add(Box::new(ConstantMedium::new(
            boundary,
            f.density,
            Color::new(f.color[0], f.color[1], f.color[2]),
        )));
    }

    for c in &scene.cone {
        let mat = build_material(&c.material);
        let center = arr_to_vec3(c.center);
        let mut obj: Box<dyn Hittable> = Box::new(Cone::new(
            center, c.radius, center.y, center.y + c.height, mat,
        ));
        if let Some(angle) = c.rotate_x { obj = Box::new(RotateX::new(obj, angle)); }
        if let Some(angle) = c.rotate_y { obj = Box::new(RotateY::new(obj, angle)); }
        if let Some(angle) = c.rotate_z { obj = Box::new(RotateZ::new(obj, angle)); }
        if let Some(offset) = c.translate { obj = Box::new(Translate::new(obj, arr_to_vec3(offset))); }
        world.add(obj);
    }

    for a in &scene.annulus {
        let mat = build_material(&a.material);
        world.add(Box::new(Annulus::new(
            arr_to_vec3(a.center),
            arr_to_vec3(a.normal),
            a.inner_radius,
            a.outer_radius,
            mat,
        )));
    }

    for d in &scene.disk {
        // Track emissive disks for NEE
        match &d.material {
            MaterialDesc::Emissive { color, intensity, .. } => {
                let emission_color = Color::new(color[0], color[1], color[2]);
                let int = intensity.unwrap_or(1.0);
                lights.push(LightInfo::Disk {
                    center: arr_to_vec3(d.center),
                    normal: arr_to_vec3(d.normal).unit(),
                    radius: d.radius,
                    emission: emission_color * int,
                });
            }
            MaterialDesc::Blackbody { temperature, intensity, .. } => {
                let emission_color = blackbody_to_rgb(*temperature);
                let int = intensity.unwrap_or(1.0);
                lights.push(LightInfo::Disk {
                    center: arr_to_vec3(d.center),
                    normal: arr_to_vec3(d.normal).unit(),
                    radius: d.radius,
                    emission: emission_color * int,
                });
            }
            _ => {}
        }
        let mat = build_material(&d.material);
        world.add(Box::new(Disk::new(
            arr_to_vec3(d.center),
            arr_to_vec3(d.normal),
            d.radius,
            mat,
        )));
    }

    for c in &scene.cylinder {
        let mat = build_material(&c.material);
        let center = arr_to_vec3(c.center);
        let mut obj: Box<dyn Hittable> = Box::new(Cylinder::new(
            center,
            c.radius,
            center.y,
            center.y + c.height,
            mat,
        ));
        if let Some(strength) = c.bump_strength {
            let scale = c.bump_scale.unwrap_or(4.0);
            obj = Box::new(BumpMap::new(obj, strength, scale));
        }
        if let Some(ref nmap_path) = c.normal_map {
            let strength = c.normal_map_strength.unwrap_or(1.0);
            match crate::normal_map::NormalMap::load_image(nmap_path) {
                Ok(nmap_data) => obj = Box::new(crate::normal_map::NormalMap::wrap(obj, nmap_data, strength)),
                Err(e) => eprintln!("Warning: {e}"),
            }
        }
        if let Some(angle) = c.rotate_x { obj = Box::new(RotateX::new(obj, angle)); }
        if let Some(angle) = c.rotate_y { obj = Box::new(RotateY::new(obj, angle)); }
        if let Some(angle) = c.rotate_z { obj = Box::new(RotateZ::new(obj, angle)); }
        if let Some(offset) = c.translate { obj = Box::new(Translate::new(obj, arr_to_vec3(offset))); }
        world.add(obj);
    }

    for t in &scene.triangle {
        let mat = build_material(&t.material);
        world.add(Box::new(Triangle::new(
            arr_to_vec3(t.v0),
            arr_to_vec3(t.v1),
            arr_to_vec3(t.v2),
            mat,
        )));
    }

    for q in &scene.quad {
        // Track emissive quads for NEE
        let origin = arr_to_vec3(q.q);
        let u_vec = arr_to_vec3(q.u);
        let v_vec = arr_to_vec3(q.v);
        match &q.material {
            MaterialDesc::Emissive { color, intensity, .. } => {
                let emission_color = Color::new(color[0], color[1], color[2]);
                let int = intensity.unwrap_or(1.0);
                lights.push(LightInfo::Rect {
                    origin,
                    u: u_vec,
                    v: v_vec,
                    normal: u_vec.cross(v_vec).unit(),
                    emission: emission_color * int,
                });
            }
            MaterialDesc::Blackbody { temperature, intensity } => {
                let emission_color = blackbody_to_rgb(*temperature);
                let int = intensity.unwrap_or(1.0);
                lights.push(LightInfo::Rect {
                    origin,
                    u: u_vec,
                    v: v_vec,
                    normal: u_vec.cross(v_vec).unit(),
                    emission: emission_color * int,
                });
            }
            _ => {}
        }
        let mat = build_material(&q.material);
        let mut obj: Box<dyn Hittable> = Box::new(Quad::new(origin, u_vec, v_vec, mat));
        if let Some(ref nmap_path) = q.normal_map {
            let strength = q.normal_map_strength.unwrap_or(1.0);
            match crate::normal_map::NormalMap::load_image(nmap_path) {
                Ok(nmap_data) => obj = Box::new(crate::normal_map::NormalMap::wrap(obj, nmap_data, strength)),
                Err(e) => eprintln!("Warning: {e}"),
            }
        }
        world.add(obj);
    }

    for m in &scene.mesh {
        let mat = build_material(&m.material);
        let scale = m.scale.unwrap_or(1.0);
        let offset = m.offset.map(arr_to_vec3).unwrap_or(Vec3::ZERO);
        let content = std::fs::read_to_string(&m.file)
            .map_err(|e| format!("Failed to read mesh '{}': {e}", m.file))?;
        let mesh_list = if m.file.ends_with(".ply") {
            obj::load_ply(&content, mat, scale, offset)?
        } else {
            obj::load_obj(&content, mat, scale, offset)?
        };
        for obj in mesh_list.objects {
            world.add(obj);
        }
    }

    for b in &scene.aabb_box {
        let sides = make_box(
            arr_to_vec3(b.min),
            arr_to_vec3(b.max),
            || build_material(&b.material),
        );

        let has_transform = b.rotate_x.is_some() || b.rotate_y.is_some()
            || b.rotate_z.is_some() || b.translate.is_some() || b.scale.is_some()
            || b.scale_xyz.is_some();
        if has_transform {
            let mut box_list = HittableList::new();
            for side in sides {
                box_list.add(side);
            }
            let mut obj: Box<dyn Hittable> = Box::new(box_list);
            if let Some([sx, sy, sz]) = b.scale_xyz {
                obj = Box::new(NonUniformScale::new(obj, sx, sy, sz));
            } else if let Some(factor) = b.scale {
                obj = Box::new(Scale::new(obj, factor));
            }
            if let Some(angle) = b.rotate_x {
                obj = Box::new(RotateX::new(obj, angle));
            }
            if let Some(angle) = b.rotate_y {
                obj = Box::new(RotateY::new(obj, angle));
            }
            if let Some(angle) = b.rotate_z {
                obj = Box::new(RotateZ::new(obj, angle));
            }
            if let Some(offset) = b.translate {
                obj = Box::new(Translate::new(obj, arr_to_vec3(offset)));
            }
            world.add(obj);
        } else {
            for side in sides {
                world.add(side);
            }
        }
    }

    for r in &scene.rect_xy {
        let mat = build_material(&r.material);
        world.add(Box::new(XyRect::new(r.x[0], r.x[1], r.y[0], r.y[1], r.k, mat)));
    }

    for r in &scene.rect_xz {
        // Track emissive XZ rects for NEE
        if let MaterialDesc::Emissive { color, intensity, .. } = &r.material {
            let emission_color = Color::new(color[0], color[1], color[2]);
            let int = intensity.unwrap_or(1.0);
            lights.push(LightInfo::Rect {
                origin: Point3::new(r.x[0], r.k, r.z[0]),
                u: Vec3::new(r.x[1] - r.x[0], 0.0, 0.0),
                v: Vec3::new(0.0, 0.0, r.z[1] - r.z[0]),
                normal: Vec3::new(0.0, -1.0, 0.0),
                emission: emission_color * int,
            });
        }
        let mat = build_material(&r.material);
        world.add(Box::new(XzRect::new(r.x[0], r.x[1], r.z[0], r.z[1], r.k, mat)));
    }

    for r in &scene.rect_yz {
        let mat = build_material(&r.material);
        world.add(Box::new(YzRect::new(r.y[0], r.y[1], r.z[0], r.z[1], r.k, mat)));
    }

    for c in &scene.csg {
        let a = build_csg_child(&c.a);
        let b = build_csg_child(&c.b);
        let csg = match c.operation.as_str() {
            "union" => crate::csg::Csg::union(a, b),
            "intersection" | "intersect" => crate::csg::Csg::intersection(a, b),
            "difference" | "subtract" => crate::csg::Csg::difference(a, b),
            other => {
                eprintln!("Warning: unknown CSG operation '{other}', using union");
                crate::csg::Csg::union(a, b)
            }
        };
        world.add(Box::new(csg));
    }

    Ok((render_config, camera, SceneWorld::from_list(world, lights)))
}

fn build_csg_child(child: &CsgChild) -> Box<dyn Hittable> {
    let mat = build_material(&child.material);
    let center = arr_to_vec3(child.center);
    match child.shape.as_str() {
        "sphere" => {
            let radius = child.radius.unwrap_or(1.0);
            Box::new(Sphere::new(center, radius, mat))
        }
        "box" => {
            let size = child.size.map(arr_to_vec3).unwrap_or(Vec3::new(1.0, 1.0, 1.0));
            let half = size * 0.5;
            let sides = make_box(center - half, center + half, || build_material(&child.material));
            let mut list = HittableList::new();
            for side in sides {
                list.add(side);
            }
            Box::new(list)
        }
        "cylinder" => {
            let radius = child.radius.unwrap_or(1.0);
            let height = child.size.map(|s| s[1]).unwrap_or(2.0);
            Box::new(Cylinder::new(center, radius, center.y - height / 2.0, center.y + height / 2.0, mat))
        }
        other => {
            eprintln!("Warning: unknown CSG shape '{other}', using unit sphere");
            Box::new(Sphere::new(center, 1.0, mat))
        }
    }
}

/// Convert blackbody temperature (Kelvin) to normalized RGB color.
/// Uses Tanner Helland's approximation of the Planckian locus.
fn blackbody_to_rgb(temp: f64) -> Color {
    let temp = temp.clamp(1000.0, 40000.0) / 100.0;

    let r = if temp <= 66.0 {
        1.0
    } else {
        let t = temp - 60.0;
        (329.698727446 * t.powf(-0.1332047592) / 255.0).clamp(0.0, 1.0)
    };

    let g = if temp <= 66.0 {
        let t = temp;
        (99.4708025861 * t.ln() - 161.1195681661).clamp(0.0, 255.0) / 255.0
    } else {
        let t = temp - 60.0;
        (288.1221695283 * t.powf(-0.0755148492) / 255.0).clamp(0.0, 1.0)
    };

    let b = if temp >= 66.0 {
        1.0
    } else if temp <= 19.0 {
        0.0
    } else {
        let t = temp - 10.0;
        (138.5177312231 * t.ln() - 305.0447927307).clamp(0.0, 255.0) / 255.0
    };

    Color::new(r, g, b)
}

fn build_material(desc: &MaterialDesc) -> Box<dyn crate::material::Material> {
    match desc {
        MaterialDesc::Lambertian { color } => {
            Box::new(Lambertian::new(Color::new(color[0], color[1], color[2])))
        }
        MaterialDesc::Metal { color, fuzz } => Box::new(Metal::new(
            Color::new(color[0], color[1], color[2]),
            fuzz.unwrap_or(0.0),
        )),
        MaterialDesc::Dielectric { refraction_index, tint, roughness, dispersion } => {
            let tint_color = tint.map(|t| Color::new(t[0], t[1], t[2])).unwrap_or(Color::new(1.0, 1.0, 1.0));
            let r = roughness.unwrap_or(0.0);
            let d = dispersion.unwrap_or(0.0);
            if d > 0.0 {
                Box::new(Dielectric::dispersive(*refraction_index, tint_color, r, d))
            } else if r > 0.0 || tint.is_some() {
                Box::new(Dielectric::rough(*refraction_index, tint_color, r))
            } else {
                Box::new(Dielectric::new(*refraction_index))
            }
        }
        MaterialDesc::Emissive { color, intensity, texture } => {
            let int = intensity.unwrap_or(1.0);
            if let Some(file) = texture {
                match ImageTexture::load(file) {
                    Ok(tex) => Box::new(Emissive::with_texture(Box::new(tex), int)),
                    Err(e) => {
                        eprintln!("Warning: failed to load emissive texture '{file}': {e}, using solid color");
                        Box::new(Emissive::new(Color::new(color[0], color[1], color[2]), int))
                    }
                }
            } else {
                Box::new(Emissive::new(
                    Color::new(color[0], color[1], color[2]),
                    int,
                ))
            }
        }
        MaterialDesc::Microfacet { color, roughness, metallic, texture } => {
            let r = roughness.unwrap_or(0.5);
            let m = metallic.unwrap_or(0.0);
            if let Some(file) = texture {
                match ImageTexture::load(file) {
                    Ok(tex) => Box::new(Microfacet::with_texture(Box::new(tex), r, m)),
                    Err(e) => {
                        eprintln!("Warning: failed to load PBR texture '{file}': {e}, using solid color");
                        Box::new(Microfacet::new(Color::new(color[0], color[1], color[2]), r, m))
                    }
                }
            } else {
                Box::new(Microfacet::new(
                    Color::new(color[0], color[1], color[2]),
                    r,
                    m,
                ))
            }
        }
        MaterialDesc::Checker { color1, color2, scale } => {
            Box::new(Lambertian::with_texture(Box::new(Checker::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(1.0),
            ))))
        }
        MaterialDesc::Marble { color, scale } => {
            Box::new(Lambertian::with_texture(Box::new(Marble::new(
                Color::new(color[0], color[1], color[2]),
                scale.unwrap_or(4.0),
            ))))
        }
        MaterialDesc::Turbulence { color, scale } => {
            Box::new(Lambertian::with_texture(Box::new(Turbulence::new(
                Color::new(color[0], color[1], color[2]),
                scale.unwrap_or(4.0),
            ))))
        }
        MaterialDesc::Image { file, uv_offset, uv_rotation, uv_tile } => {
            let tex = ImageTexture::load(file)
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to load image texture '{file}': {e}, using fallback");
                    ImageTexture::fallback()
                });
            let has_transform = uv_offset.is_some() || uv_rotation.is_some() || uv_tile.is_some();
            if has_transform {
                let off = uv_offset.unwrap_or([0.0, 0.0]);
                let rot = uv_rotation.unwrap_or(0.0);
                let tile = uv_tile.unwrap_or([1.0, 1.0]);
                Box::new(Lambertian::with_texture(Box::new(TransformedTexture::new(
                    Box::new(tex), off[0], off[1], rot, tile[0], tile[1],
                ))))
            } else {
                Box::new(Lambertian::with_texture(Box::new(tex)))
            }
        }
        MaterialDesc::Mirror => {
            Box::new(Metal::new(Color::new(0.95, 0.95, 0.95), 0.0))
        }
        MaterialDesc::Glass => {
            Box::new(Dielectric::new(1.5))
        }
        MaterialDesc::Wood { color1, color2, scale } => {
            Box::new(Lambertian::with_texture(Box::new(Wood::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(4.0),
            ))))
        }
        MaterialDesc::Rings { color1, color2, scale } => {
            Box::new(Lambertian::with_texture(Box::new(Rings::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(4.0),
            ))))
        }
        MaterialDesc::BlendMat { material_a, material_b, ratio } => {
            let a = build_material(material_a);
            let b = build_material(material_b);
            Box::new(Blend::new(a, b, ratio.unwrap_or(0.5)))
        }
        MaterialDesc::Opacity { material, opacity } => {
            let inner = build_material(material);
            let transparent = Box::new(Transparent);
            Box::new(Blend::new(inner, transparent, opacity.clamp(0.0, 1.0)))
        }
        MaterialDesc::Dots { dot_color, bg_color, scale, radius } => {
            Box::new(Lambertian::with_texture(Box::new(Dots::new(
                Color::new(dot_color[0], dot_color[1], dot_color[2]),
                Color::new(bg_color[0], bg_color[1], bg_color[2]),
                scale.unwrap_or(1.0),
                radius.unwrap_or(0.25),
            ))))
        }
        MaterialDesc::Grid { line_color, bg_color, scale, line_width } => {
            Box::new(Lambertian::with_texture(Box::new(Grid::new(
                Color::new(line_color[0], line_color[1], line_color[2]),
                Color::new(bg_color[0], bg_color[1], bg_color[2]),
                scale.unwrap_or(1.0),
                line_width.unwrap_or(0.05),
            ))))
        }
        MaterialDesc::UvChecker { color1, color2, frequency } => {
            Box::new(Lambertian::with_texture(Box::new(UvChecker::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                frequency.unwrap_or(10.0),
            ))))
        }
        MaterialDesc::GradientTex { color1, color2, axis, min, max } => {
            let axis_idx = match axis.as_deref() {
                Some("x" | "X") => 0,
                Some("y" | "Y") => 1,
                _ => 2,
            };
            Box::new(Lambertian::with_texture(Box::new(GradientTexture::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                axis_idx,
                min.unwrap_or(0.0),
                max.unwrap_or(1.0),
            ))))
        }
        MaterialDesc::Stripe { color1, color2, scale, axis } => {
            let axis_idx = match axis.as_deref() {
                Some("x" | "X") => 0,
                Some("y" | "Y") => 1,
                _ => 2,
            };
            Box::new(Lambertian::with_texture(Box::new(Stripe::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(1.0),
                axis_idx,
            ))))
        }
        MaterialDesc::Voronoi { color1, color2, scale } => {
            Box::new(Lambertian::with_texture(Box::new(Voronoi::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(1.0),
            ))))
        }
        MaterialDesc::Noise { color1, color2, scale } => {
            Box::new(Lambertian::with_texture(Box::new(Noise::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(4.0),
            ))))
        }
        MaterialDesc::Spiral { color1, color2, scale, arms } => {
            Box::new(Lambertian::with_texture(Box::new(Spiral::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(1.0),
                arms.unwrap_or(2),
            ))))
        }
        MaterialDesc::Hexgrid { color1, color2, scale, line_width } => {
            Box::new(Lambertian::with_texture(Box::new(Hexgrid::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(1.0),
                line_width.unwrap_or(0.1),
            ))))
        }
        MaterialDesc::Fbm { color1, color2, scale, octaves } => {
            Box::new(Lambertian::with_texture(Box::new(Fbm::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(1.0),
                octaves.unwrap_or(6),
            ))))
        }
        MaterialDesc::Plasma { scale } => {
            Box::new(Lambertian::with_texture(Box::new(Plasma::new(scale.unwrap_or(2.0)))))
        }
        MaterialDesc::Terrain { scale } => {
            Box::new(Lambertian::with_texture(Box::new(Terrain::new(scale.unwrap_or(1.0)))))
        }
        MaterialDesc::Rust { scale } => {
            Box::new(Lambertian::with_texture(Box::new(Rust::new(scale.unwrap_or(2.0)))))
        }
        MaterialDesc::Brick { brick_color, mortar_color, scale, mortar_width } => {
            let bc = brick_color.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(0.7, 0.2, 0.15));
            let mc = mortar_color.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(0.8, 0.8, 0.75));
            Box::new(Lambertian::with_texture(Box::new(Brick::new(
                bc, mc, scale.unwrap_or(3.0), mortar_width.unwrap_or(0.05),
            ))))
        }
        MaterialDesc::Camo { color1, color2, color3, scale } => {
            let c1 = color1.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(0.2, 0.35, 0.15));
            let c2 = color2.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(0.35, 0.25, 0.1));
            let c3 = color3.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(0.1, 0.15, 0.08));
            Box::new(Lambertian::with_texture(Box::new(Camo::new(c1, c2, c3, scale.unwrap_or(3.0)))))
        }
        MaterialDesc::Lava { scale } => {
            Box::new(Lambertian::with_texture(Box::new(Lava::new(scale.unwrap_or(2.0)))))
        }
        MaterialDesc::Cloud { color, sky_color, scale, density, octaves } => {
            let c = color.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(1.0, 1.0, 1.0));
            let sc = sky_color.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(0.5, 0.7, 1.0));
            Box::new(Lambertian::with_texture(Box::new(Cloud::new(
                c, sc, scale.unwrap_or(1.0), density.unwrap_or(0.5), octaves.unwrap_or(6),
            ))))
        }
        MaterialDesc::TriPlanar { color1, color2, scale, sharpness } => {
            let inner = Box::new(Checker {
                even: Color::new(color1[0], color1[1], color1[2]),
                odd: Color::new(color2[0], color2[1], color2[2]),
                scale: 1.0,
            });
            Box::new(Lambertian::with_texture(Box::new(TriPlanar::new(
                inner,
                scale.unwrap_or(1.0),
                sharpness.unwrap_or(2.0),
            ))))
        }
        MaterialDesc::MixColor { color1, color2, factor } => {
            Box::new(Lambertian::with_texture(Box::new(MixTexture::new(
                Box::new(crate::texture::SolidColor::new(Color::new(color1[0], color1[1], color1[2]))),
                Box::new(crate::texture::SolidColor::new(Color::new(color2[0], color2[1], color2[2]))),
                factor.unwrap_or(0.5),
            ))))
        }
        MaterialDesc::Wavy { color1, color2, scale, waves } => {
            Box::new(Lambertian::with_texture(Box::new(Wavy::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(1.0),
                waves.unwrap_or(3),
            ))))
        }
        MaterialDesc::ColorRamp { stops, axis, min_val, max_val } => {
            let color_stops: Vec<(f64, Color)> = stops.iter()
                .map(|s| (s[0], Color::new(s[1], s[2], s[3])))
                .collect();
            Box::new(Lambertian::with_texture(Box::new(ColorRamp::new(
                color_stops,
                axis.unwrap_or(1),
                min_val.unwrap_or(0.0),
                max_val.unwrap_or(1.0),
            ))))
        }
        MaterialDesc::Toon { color, bands, specular } => {
            Box::new(Toon::new(
                Color::new(color[0], color[1], color[2]),
                bands.unwrap_or(3),
                specular.unwrap_or(0.5),
            ))
        }
        MaterialDesc::Anisotropic { color, roughness_u, roughness_v, tangent_axis } => {
            Box::new(Anisotropic::new(
                Color::new(color[0], color[1], color[2]),
                roughness_u.unwrap_or(0.1),
                roughness_v.unwrap_or(0.5),
                tangent_axis.unwrap_or(1),
            ))
        }
        MaterialDesc::Blackbody { temperature, intensity } => {
            let color = blackbody_to_rgb(*temperature);
            Box::new(Emissive::new(color, intensity.unwrap_or(1.0)))
        }
        MaterialDesc::Clearcoat { color, coat_gloss, coat_ior } => {
            Box::new(Clearcoat::new(
                Color::new(color[0], color[1], color[2]),
                coat_gloss.unwrap_or(0.8),
                coat_ior.unwrap_or(1.5),
            ))
        }
        MaterialDesc::Velvet { color, sheen } => {
            Box::new(Velvet::new(
                Color::new(color[0], color[1], color[2]),
                sheen.unwrap_or(1.0),
            ))
        }
        MaterialDesc::Iridescent { color, thickness, film_ior, roughness } => {
            let c = color.unwrap_or([0.9, 0.9, 0.9]);
            Box::new(Iridescent::new(
                Color::new(c[0], c[1], c[2]),
                thickness.unwrap_or(400.0),
                film_ior.unwrap_or(1.4),
                roughness.unwrap_or(0.05),
            ))
        }
        MaterialDesc::Translucent { color, translucency, scatter_width } => {
            Box::new(Translucent::new(
                Color::new(color[0], color[1], color[2]),
                translucency.unwrap_or(0.5),
                scatter_width.unwrap_or(0.8),
            ))
        }
        MaterialDesc::Subsurface { color, mean_free_path, scatter_color } => {
            let sc = scatter_color.unwrap_or([0.8, 0.5, 0.5]);
            Box::new(Subsurface::new(
                Color::new(color[0], color[1], color[2]),
                mean_free_path.unwrap_or(0.5),
                Color::new(sc[0], sc[1], sc[2]),
            ))
        }
    }
}

/// Build the classic "random spheres" demo scene.
pub fn demo_scene() -> (RenderConfig, Camera, SceneWorld) {
    let render_config = RenderConfig {
        width: 800,
        height: 450,
        samples_per_pixel: 64,
        max_depth: 50,
        background: Background::default(),
        seed: 31337,
        quiet: false,
        exposure: 1.0,
        tone_map: crate::render::ToneMap::default(),
        auto_exposure: false,
        denoise: false,
        save_hdr: false,
        crop: None,
        bloom: 0.0,
        vignette: 0.0,
        grain: 0.0,
        saturation: 1.0,
        contrast: 1.0,
        white_balance: 0.0,
        sharpen: 0.0,
        hue_shift: 0.0,
        dither: false,
        gamma: 0.0,
        pixel_filter: crate::render::PixelFilter::default(),
        firefly_filter: 0.0,
        chromatic_aberration: 0.0,
        lens_distortion: 0.0,
        save_depth: false,
        save_normals: false,
        save_albedo: false,
        adaptive: false,
        adaptive_threshold: 0.03,
        time_limit: 0.0,
        posterize: 0,
        sepia: 0.0,
        edge_detect: 0.0,
        pixelate: 0,
        invert: false,
        scanlines: 0.0,
        threshold: -1.0,
        blur: 0.0,
        tilt_shift: 0.0,
        grade_shadows: [1.0, 1.0, 1.0],
        grade_highlights: [1.0, 1.0, 1.0],
        halftone: 0,
        emboss: 0.0,
        oil_paint: 0,
        color_map: String::new(),
        solarize: -1.0,
        duo_tone: String::new(),
        sketch: false,
        median: 0,
        crosshatch: 0,
        glitch: 0.0,
        depth_fog: 0.0,
        depth_fog_color: [0.8, 0.85, 0.9],
        channel_swap: String::new(),
        mirror: String::new(),
        quantize: 0,
        tint: [1.0, 1.0, 1.0],
        palette: String::new(),
        radial_blur: 0.0,
        border: 0,
        border_color: [0.0, 0.0, 0.0],
        resize: [0, 0],
        rotate: 0,
    };

    let cam_config = CameraConfig {
        look_from: Point3::new(13.0, 2.0, 3.0),
        look_at: Point3::new(0.0, 0.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),
        vfov_degrees: 20.0,
        aspect_ratio: render_config.width as f64 / render_config.height as f64,
        aperture: 0.1,
        focus_dist: 10.0,
    };
    let camera = Camera::new(cam_config);

    let mut world = HittableList::new();

    // Ground
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
    )));

    // Three showcase spheres
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        Box::new(Dielectric::new(1.5)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Box::new(Lambertian::new(Color::new(0.4, 0.2, 0.1))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Box::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0)),
    )));

    // Small random spheres
    let mut rng = rand::rng();
    use rand::Rng;
    for a in -5..5 {
        for b in -5..5 {
            let choose_mat: f64 = rng.random();
            let center = Point3::new(
                a as f64 + 0.9 * rng.random::<f64>(),
                0.2,
                b as f64 + 0.9 * rng.random::<f64>(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let material: Box<dyn crate::material::Material> = if choose_mat < 0.8 {
                    let albedo = Color::random(&mut rng).hadamard(Color::random(&mut rng));
                    Box::new(Lambertian::new(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = Color::random_range(&mut rng, 0.5, 1.0);
                    let fuzz = rng.random::<f64>() * 0.5;
                    Box::new(Metal::new(albedo, fuzz))
                } else {
                    Box::new(Dielectric::new(1.5))
                };
                world.add(Box::new(Sphere::new(center, 0.2, material)));
            }
        }
    }

    (render_config, camera, SceneWorld::from_list(world, Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_simple_scene() {
        let toml = r#"
[render]
width = 400
height = 200
samples = 16
max_depth = 10

[camera]
look_from = [0.0, 1.0, 5.0]
look_at = [0.0, 0.0, 0.0]
vfov = 60.0

[[sphere]]
center = [0.0, 0.0, 0.0]
radius = 1.0
[sphere.material]
type = "lambertian"
color = [0.8, 0.2, 0.1]

[[sphere]]
center = [2.0, 0.0, 0.0]
radius = 0.5
[sphere.material]
type = "metal"
color = [0.9, 0.9, 0.9]
fuzz = 0.1
"#;
        let (render_config, _camera, _world) = load_scene(toml).unwrap();
        assert_eq!(render_config.width, 400);
        assert_eq!(render_config.height, 200);
        assert_eq!(render_config.samples_per_pixel, 16);
        assert_eq!(render_config.max_depth, 10);
    }

    #[test]
    fn materials_created_from_toml() {
        let toml = r#"
[[sphere]]
center = [0.0, 0.0, 0.0]
radius = 1.0
[sphere.material]
type = "dielectric"
refraction_index = 1.5

[[sphere]]
center = [3.0, 0.0, 0.0]
radius = 1.0
[sphere.material]
type = "emissive"
color = [1.0, 1.0, 1.0]
intensity = 5.0
"#;
        let result = load_scene(toml);
        assert!(result.is_ok(), "Scene should parse: {:?}", result.err());
    }

    #[test]
    fn invalid_toml_returns_error() {
        let bad_toml = "this is {{{{ not valid TOML at all";
        let result = load_scene(bad_toml);
        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.contains("TOML parse error"), "Error was: {e}"),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn missing_material_type_returns_error() {
        let toml = r#"
[[sphere]]
center = [0.0, 0.0, 0.0]
radius = 1.0
[sphere.material]
color = [1.0, 0.0, 0.0]
"#;
        let result = load_scene(toml);
        assert!(result.is_err());
    }

    #[test]
    fn every_geometry_type_parses() {
        let toml = r#"
[[sphere]]
center = [0.0, 0.0, 0.0]
radius = 1.0
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[plane]]
point = [0.0, 0.0, 0.0]
normal = [0.0, 1.0, 0.0]
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[triangle]]
v0 = [0.0, 0.0, 0.0]
v1 = [1.0, 0.0, 0.0]
v2 = [0.0, 1.0, 0.0]
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[cylinder]]
center = [0.0, 0.0, 0.0]
radius = 0.5
height = 1.0
material = { type = "metal", color = [0.5, 0.5, 0.5] }

[[cone]]
center = [0.0, 0.0, 0.0]
radius = 0.5
height = 1.0
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[disk]]
center = [0.0, 0.0, 0.0]
normal = [0.0, 1.0, 0.0]
radius = 1.0
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[capsule]]
center = [0.0, 0.0, 0.0]
radius = 0.3
height = 1.0
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[ellipsoid]]
center = [0.0, 0.0, 0.0]
radii = [1.0, 0.5, 0.7]
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[box]]
min = [0.0, 0.0, 0.0]
max = [1.0, 1.0, 1.0]
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[fog]]
center = [0.0, 1.0, 0.0]
radius = 2.0
density = 0.5
color = [0.8, 0.8, 0.8]

[[quad]]
q = [0.0, 0.0, 0.0]
u = [1.0, 0.0, 0.0]
v = [0.0, 0.0, 1.0]
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[moving_sphere]]
center0 = [0.0, 0.0, 0.0]
center1 = [1.0, 0.0, 0.0]
radius = 0.5
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[hemisphere]]
center = [0.0, 0.0, 0.0]
radius = 1.0
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[annulus]]
center = [0.0, 0.0, 0.0]
normal = [0.0, 1.0, 0.0]
inner_radius = 0.5
outer_radius = 1.0
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[torus]]
center = [0.0, 0.0, 0.0]
major_radius = 1.0
minor_radius = 0.3
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }
"#;
        let result = load_scene(toml);
        assert!(result.is_ok(), "Every geometry type should parse: {:?}", result.err());
    }

    #[test]
    #[test]
    fn blackbody_produces_valid_colors() {
        // Candle light (warm)
        let warm = super::blackbody_to_rgb(1800.0);
        assert!(warm.x > warm.y && warm.y > warm.z, "1800K should be warm (R>G>B)");

        // Daylight (neutral white)
        let daylight = super::blackbody_to_rgb(6500.0);
        assert!(daylight.x > 0.5 && daylight.y > 0.5 && daylight.z > 0.5,
            "6500K should be near white");

        // Blue sky (cool)
        let cool = super::blackbody_to_rgb(15000.0);
        assert!(cool.z >= cool.x, "15000K should be cool (B>=R)");

        // All values in range
        for temp in [1000, 2000, 3000, 4000, 5000, 6000, 8000, 10000, 20000] {
            let c = super::blackbody_to_rgb(temp as f64);
            assert!(c.x >= 0.0 && c.x <= 1.0, "R out of range at {temp}K: {}", c.x);
            assert!(c.y >= 0.0 && c.y <= 1.0, "G out of range at {temp}K: {}", c.y);
            assert!(c.z >= 0.0 && c.z <= 1.0, "B out of range at {temp}K: {}", c.z);
        }
    }

    #[test]
    fn every_material_type_parses() {
        let toml = r#"
[[sphere]]
center = [0.0, 0.0, 0.0]
radius = 1.0
material = { type = "lambertian", color = [0.5, 0.5, 0.5] }

[[sphere]]
center = [2.0, 0.0, 0.0]
radius = 1.0
material = { type = "metal", color = [0.5, 0.5, 0.5], fuzz = 0.1 }

[[sphere]]
center = [4.0, 0.0, 0.0]
radius = 1.0
material = { type = "dielectric", refraction_index = 1.5, tint = [0.9, 0.9, 1.0], roughness = 0.1 }

[[sphere]]
center = [6.0, 0.0, 0.0]
radius = 1.0
material = { type = "emissive", color = [1.0, 1.0, 1.0], intensity = 5.0 }

[[sphere]]
center = [8.0, 0.0, 0.0]
radius = 1.0
material = { type = "checker", color1 = [0.9, 0.9, 0.9], color2 = [0.1, 0.1, 0.1] }

[[sphere]]
center = [10.0, 0.0, 0.0]
radius = 1.0
material = { type = "stripe", color1 = [1.0, 0.0, 0.0], color2 = [0.0, 0.0, 1.0] }

[[sphere]]
center = [12.0, 0.0, 0.0]
radius = 1.0
material = { type = "marble", color = [0.9, 0.9, 0.9] }

[[sphere]]
center = [14.0, 0.0, 0.0]
radius = 1.0
material = { type = "turbulence", color = [0.7, 0.5, 0.3] }

[[sphere]]
center = [16.0, 0.0, 0.0]
radius = 1.0
material = { type = "dots", dot_color = [1.0, 0.0, 0.0], bg_color = [1.0, 1.0, 1.0] }

[[sphere]]
center = [18.0, 0.0, 0.0]
radius = 1.0
material = { type = "grid", line_color = [0.0, 0.0, 0.0], bg_color = [1.0, 1.0, 1.0] }

[[sphere]]
center = [20.0, 0.0, 0.0]
radius = 1.0
material = { type = "uv_checker", color1 = [0.0, 0.0, 1.0], color2 = [1.0, 1.0, 0.0] }

[[sphere]]
center = [22.0, 0.0, 0.0]
radius = 1.0
material = { type = "rings", color1 = [0.6, 0.3, 0.1], color2 = [0.3, 0.15, 0.05] }

[[sphere]]
center = [24.0, 0.0, 0.0]
radius = 1.0
material = { type = "wood", color1 = [0.6, 0.3, 0.1], color2 = [0.3, 0.15, 0.05] }

[[sphere]]
center = [26.0, 0.0, 0.0]
radius = 1.0
material = { type = "voronoi", color1 = [0.8, 0.4, 0.1], color2 = [0.2, 0.1, 0.05] }

[[sphere]]
center = [28.0, 0.0, 0.0]
radius = 1.0
material = { type = "iridescent", color = [0.9, 0.9, 1.0], thickness = 400, film_ior = 1.4 }

[[sphere]]
center = [30.0, 0.0, 0.0]
radius = 1.0
material = { type = "translucent", color = [0.7, 0.9, 0.7], translucency = 0.5, scatter_width = 0.8 }

[[sphere]]
center = [32.0, 0.0, 0.0]
radius = 1.0
material = { type = "velvet", color = [0.5, 0.1, 0.1], sheen = 1.2 }

[[sphere]]
center = [34.0, 0.0, 0.0]
radius = 1.0
material = { type = "clearcoat", color = [0.8, 0.1, 0.05], coat_gloss = 0.9 }

[[sphere]]
center = [36.0, 0.0, 0.0]
radius = 1.0
material = { type = "blackbody", temperature = 3200, intensity = 5.0 }

[[sphere]]
center = [38.0, 0.0, 0.0]
radius = 1.0
material = { type = "anisotropic", color = [0.8, 0.8, 0.85], roughness_u = 0.05, roughness_v = 0.4 }

[[sphere]]
center = [40.0, 0.0, 0.0]
radius = 1.0
material = { type = "toon", color = [0.2, 0.5, 0.9], bands = 4, specular = 0.8 }

[[sphere]]
center = [42.0, 0.0, 0.0]
radius = 1.0
material = { type = "hexgrid", color1 = [0.9, 0.9, 0.9], color2 = [0.2, 0.2, 0.2] }

[[sphere]]
center = [44.0, 0.0, 0.0]
radius = 1.0
material = { type = "subsurface", color = [0.9, 0.7, 0.6], mean_free_path = 0.3, scatter_color = [0.9, 0.4, 0.3] }

[[sphere]]
center = [46.0, 0.0, 0.0]
radius = 1.0
material = { type = "color_ramp", stops = [[0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]], axis = 1, min_val = -1, max_val = 1 }

[[sphere]]
center = [48.0, 0.0, 0.0]
radius = 1.0

[sphere.material]
type = "opacity"
opacity = 0.5
[sphere.material.material]
type = "lambertian"
color = [0.8, 0.2, 0.2]

[[sphere]]
center = [50.0, 0.0, 0.0]
radius = 1.0
material = { type = "fbm", color1 = [0.5, 0.5, 0.5], color2 = [0.2, 0.2, 0.2] }

[[sphere]]
center = [52.0, 0.0, 0.0]
radius = 1.0
material = { type = "wavy", color1 = [0.5, 0.0, 0.0], color2 = [0.0, 0.0, 0.5] }

[[sphere]]
center = [54.0, 0.0, 0.0]
radius = 1.0
material = { type = "mix_color", color1 = [1.0, 0.0, 0.0], color2 = [0.0, 0.0, 1.0], factor = 0.5 }

[[sphere]]
center = [56.0, 0.0, 0.0]
radius = 1.0
material = { type = "tri_planar", color1 = [0.5, 0.5, 0.5], color2 = [0.2, 0.2, 0.2] }

[[sphere]]
center = [58.0, 0.0, 0.0]
radius = 1.0
material = { type = "cloud" }

[[sphere]]
center = [60.0, 0.0, 0.0]
radius = 1.0
material = { type = "lava" }

[[sphere]]
center = [62.0, 0.0, 0.0]
radius = 1.0
material = { type = "camo" }

[[sphere]]
center = [64.0, 0.0, 0.0]
radius = 1.0
material = { type = "brick" }

[[sphere]]
center = [66.0, 0.0, 0.0]
radius = 1.0
material = { type = "rust" }

[[sphere]]
center = [68.0, 0.0, 0.0]
radius = 1.0
material = { type = "terrain" }
"#;
        let result = load_scene(toml);
        assert!(result.is_ok(), "Every material type should parse: {:?}", result.err());
    }
}
