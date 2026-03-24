use crate::aabb::Aabb;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// Record of a ray-object intersection.
pub struct HitRecord<'a> {
    pub point: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
    pub material: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    /// Construct a hit record, ensuring the normal always points against the ray.
    pub fn new(
        ray: &Ray,
        point: Point3,
        outward_normal: Vec3,
        t: f64,
        u: f64,
        v: f64,
        material: &'a dyn Material,
    ) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self {
            point,
            normal,
            t,
            u,
            v,
            front_face,
            material,
        }
    }
}

/// Trait for anything a ray can intersect.
pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>>;

    /// Return the axis-aligned bounding box for this object.
    /// Returns None for unbounded objects (e.g. infinite planes).
    fn bounding_box(&self) -> Option<Aabb>;
}

/// A list of hittable objects.
pub struct HittableList {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let mut closest = t_max;
        let mut best_hit = None;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray, t_min, closest) {
                closest = hit.t;
                best_hit = Some(hit);
            }
        }

        best_hit
    }

    fn bounding_box(&self) -> Option<Aabb> {
        if self.objects.is_empty() {
            return None;
        }

        let mut result: Option<Aabb> = None;
        for object in &self.objects {
            if let Some(bbox) = object.bounding_box() {
                result = Some(match result {
                    Some(existing) => Aabb::surrounding(&existing, &bbox),
                    None => bbox,
                });
            }
        }
        result
    }
}
