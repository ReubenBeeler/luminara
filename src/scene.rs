use serde::Deserialize;

use crate::aabb::Aabb;
use crate::bvh::BvhNode;
use crate::camera::{Camera, CameraConfig};
use crate::hit::{HitRecord, Hittable, HittableList};
use crate::material::{Dielectric, Emissive, Lambertian, Metal};
use crate::texture::{Checker, Marble, Turbulence};
use crate::plane::Plane;
use crate::ray::Ray;
use crate::render::RenderConfig;
use crate::sphere::Sphere;
use crate::obj;
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
}

#[derive(Deserialize)]
pub struct RenderSettings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub samples: Option<u32>,
    pub max_depth: Option<u32>,
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
#[serde(tag = "type")]
pub enum MaterialDesc {
    #[serde(alias = "lambertian")]
    Lambertian { color: [f64; 3] },
    #[serde(alias = "metal")]
    Metal { color: [f64; 3], fuzz: Option<f64> },
    #[serde(alias = "dielectric")]
    Dielectric { refraction_index: f64 },
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

        if let Some(ref bvh) = self.bvh {
            if let Some(hit) = bvh.hit(ray, t_min, closest) {
                closest = hit.t;
                best_hit = Some(hit);
            }
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
    }

    // Camera
    let mut cam_config = CameraConfig::default();
    cam_config.aspect_ratio = render_config.width as f64 / render_config.height as f64;
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
        world.add(Box::new(Sphere::new(arr_to_vec3(s.center), s.radius, mat)));
    }

    for p in &scene.plane {
        let mat = build_material(&p.material);
        world.add(Box::new(Plane::new(
            arr_to_vec3(p.point),
            arr_to_vec3(p.normal),
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
        MaterialDesc::Dielectric { refraction_index } => {
            Box::new(Dielectric::new(*refraction_index))
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
    }
}

/// Build the classic "random spheres" demo scene.
pub fn demo_scene() -> (RenderConfig, Camera, SceneWorld) {
    let render_config = RenderConfig {
        width: 800,
        height: 450,
        samples_per_pixel: 64,
        max_depth: 50,
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
