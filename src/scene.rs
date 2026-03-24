use serde::Deserialize;

use crate::aabb::Aabb;
use crate::bvh::BvhNode;
use crate::camera::{Camera, CameraConfig};
use crate::bump::BumpMap;
use crate::capsule::Capsule;
use crate::transform::{RotateY, Translate};
use crate::cone::Cone;
use crate::constant_medium::ConstantMedium;
use crate::cylinder::Cylinder;
use crate::disk::Disk;
use crate::ellipsoid::Ellipsoid;
use crate::hit::{HitRecord, Hittable, HittableList};
use crate::material::{Blend, Dielectric, Emissive, Lambertian, Metal};
use crate::texture::{Checker, Dots, GradientTexture, Grid, ImageTexture, Marble, Rings, Stripe, Turbulence, UvChecker, Wood};
use crate::plane::Plane;
use crate::ray::Ray;
use crate::rect::{XyRect, XzRect, YzRect, make_box};
use crate::render::{Background, RenderConfig};
use crate::sphere::Sphere;
use crate::obj;
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
    pub fog: Vec<FogDesc>,
    #[serde(default)]
    pub cone: Vec<ConeDesc>,
    #[serde(default)]
    pub cylinder: Vec<CylinderDesc>,
    #[serde(default)]
    pub disk: Vec<DiskDesc>,
    #[serde(default)]
    #[serde(rename = "box")]
    pub aabb_box: Vec<BoxDesc>,
    #[serde(default)]
    pub rect_xy: Vec<RectXyDesc>,
    #[serde(default)]
    pub rect_xz: Vec<RectXzDesc>,
    #[serde(default)]
    pub rect_yz: Vec<RectYzDesc>,
}

#[derive(Deserialize)]
pub struct RenderSettings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub samples: Option<u32>,
    pub max_depth: Option<u32>,
    pub seed: Option<u64>,
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
    #[serde(alias = "sun")]
    Sun {
        direction: [f64; 3],
        sun_color: Option<[f64; 3]>,
        intensity: Option<f64>,
        sky_color: Option<[f64; 3]>,
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
}

#[derive(Deserialize)]
pub struct CylinderDesc {
    pub center: [f64; 3],
    pub radius: f64,
    pub height: f64,
    pub material: MaterialDesc,
}

#[derive(Deserialize)]
pub struct BoxDesc {
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub material: MaterialDesc,
    pub rotate_y: Option<f64>,
    pub translate: Option<[f64; 3]>,
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
    },
    #[serde(alias = "emissive")]
    Emissive { color: [f64; 3], intensity: Option<f64> },
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
    #[serde(alias = "blend")]
    BlendMat {
        material_a: Box<MaterialDesc>,
        material_b: Box<MaterialDesc>,
        ratio: Option<f64>,
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
}

/// A scene world that uses BVH for bounded objects and linear scan for unbounded ones.
pub struct SceneWorld {
    bvh: Option<Box<dyn Hittable>>,
    unbounded: Vec<Box<dyn Hittable>>,
}

impl SceneWorld {
    /// Build a SceneWorld from a HittableList, partitioning objects into
    /// bounded (accelerated via BVH) and unbounded (tested linearly).
    pub fn from_list(list: HittableList) -> Self {
        let mut bounded = Vec::new();
        let mut unbounded = Vec::new();

        for obj in list.objects {
            if obj.bounding_box().is_some() {
                bounded.push(obj);
            } else {
                unbounded.push(obj);
            }
        }

        let bvh = if bounded.is_empty() {
            None
        } else {
            Some(BvhNode::build(bounded))
        };

        Self { bvh, unbounded }
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
                BackgroundDesc::Sun { direction, sun_color, intensity, sky_color } => {
                    Background::Sun {
                        direction: arr_to_vec3(*direction),
                        sun_color: sun_color.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(1.0, 0.95, 0.85)),
                        sun_intensity: intensity.unwrap_or(20.0),
                        sky_color: sky_color.map(|c| Color::new(c[0], c[1], c[2])).unwrap_or(Color::new(0.5, 0.7, 1.0)),
                    }
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

    for s in &scene.sphere {
        let mat = build_material(&s.material);
        let sphere: Box<dyn Hittable> = Box::new(Sphere::new(arr_to_vec3(s.center), s.radius, mat));
        if let Some(strength) = s.bump_strength {
            let scale = s.bump_scale.unwrap_or(4.0);
            world.add(Box::new(BumpMap::new(sphere, strength, scale)));
        } else {
            world.add(sphere);
        }
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
        world.add(Box::new(Torus::new(arr_to_vec3(t.center), t.major_radius, t.minor_radius, mat)));
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
        world.add(Box::new(Ellipsoid::new(arr_to_vec3(e.center), arr_to_vec3(e.radii), mat)));
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
        world.add(Box::new(Cone::new(
            center,
            c.radius,
            center.y,
            center.y + c.height,
            mat,
        )));
    }

    for d in &scene.disk {
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
        world.add(Box::new(Cylinder::new(
            center,
            c.radius,
            center.y,
            center.y + c.height,
            mat,
        )));
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

    for m in &scene.mesh {
        let mat = build_material(&m.material);
        let scale = m.scale.unwrap_or(1.0);
        let offset = m.offset.map(arr_to_vec3).unwrap_or(Vec3::ZERO);
        let content = std::fs::read_to_string(&m.file)
            .map_err(|e| format!("Failed to read mesh '{}': {e}", m.file))?;
        let mesh_list = obj::load_obj(&content, mat, scale, offset)?;
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

        if b.rotate_y.is_some() || b.translate.is_some() {
            // Wrap all sides into a HittableList, then transform
            let mut box_list = HittableList::new();
            for side in sides {
                box_list.add(side);
            }
            let mut obj: Box<dyn Hittable> = Box::new(box_list);
            if let Some(angle) = b.rotate_y {
                obj = Box::new(RotateY::new(obj, angle));
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
        let mat = build_material(&r.material);
        world.add(Box::new(XzRect::new(r.x[0], r.x[1], r.z[0], r.z[1], r.k, mat)));
    }

    for r in &scene.rect_yz {
        let mat = build_material(&r.material);
        world.add(Box::new(YzRect::new(r.y[0], r.y[1], r.z[0], r.z[1], r.k, mat)));
    }

    Ok((render_config, camera, SceneWorld::from_list(world)))
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
        MaterialDesc::Dielectric { refraction_index, tint, roughness } => {
            let tint_color = tint.map(|t| Color::new(t[0], t[1], t[2])).unwrap_or(Color::new(1.0, 1.0, 1.0));
            let r = roughness.unwrap_or(0.0);
            if r > 0.0 || tint.is_some() {
                Box::new(Dielectric::rough(*refraction_index, tint_color, r))
            } else {
                Box::new(Dielectric::new(*refraction_index))
            }
        }
        MaterialDesc::Emissive { color, intensity } => {
            Box::new(Emissive::new(
                Color::new(color[0], color[1], color[2]),
                intensity.unwrap_or(1.0),
            ))
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
        MaterialDesc::Image { file } => {
            let tex = ImageTexture::load(file)
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to load image texture '{file}': {e}, using fallback");
                    ImageTexture::fallback()
                });
            Box::new(Lambertian::with_texture(Box::new(tex)))
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
                _ => 2, // Default to Z for unknown axes
            };
            Box::new(Lambertian::with_texture(Box::new(Stripe::new(
                Color::new(color1[0], color1[1], color1[2]),
                Color::new(color2[0], color2[1], color2[2]),
                scale.unwrap_or(1.0),
                axis_idx,
            ))))
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

    (render_config, camera, SceneWorld::from_list(world))
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
"#;
        let result = load_scene(toml);
        assert!(result.is_ok(), "Every geometry type should parse: {:?}", result.err());
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
"#;
        let result = load_scene(toml);
        assert!(result.is_ok(), "Every material type should parse: {:?}", result.err());
    }
}
