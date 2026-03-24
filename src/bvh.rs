use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::ray::Ray;

/// A node in the bounding volume hierarchy.
pub struct BvhNode {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
    bbox: Aabb,
}

impl BvhNode {
    /// Build a BVH from a list of hittable objects.
    /// Objects without bounding boxes (e.g. infinite planes) are excluded —
    /// the caller should test those separately.
    pub fn build(mut objects: Vec<Box<dyn Hittable>>) -> Box<dyn Hittable> {
        let axis = longest_axis(&objects);

        objects.sort_by(|a, b| {
            let a_min = a.bounding_box().map(|bb| axis_val(&bb.min, axis)).unwrap_or(0.0);
            let b_min = b.bounding_box().map(|bb| axis_val(&bb.min, axis)).unwrap_or(0.0);
            a_min.partial_cmp(&b_min).unwrap_or(std::cmp::Ordering::Equal)
        });

        match objects.len() {
            0 => panic!("BVH: cannot build from zero objects"),
            1 => objects.into_iter().next().unwrap(),
            2 => {
                let mut iter = objects.into_iter();
                let left = iter.next().unwrap();
                let right = iter.next().unwrap();
                let left_box = left.bounding_box().unwrap();
                let right_box = right.bounding_box().unwrap();
                let bbox = Aabb::surrounding(&left_box, &right_box);
                Box::new(BvhNode { left, right, bbox })
            }
            n => {
                let mid = n / 2;
                let right_half = objects.split_off(mid);
                let left = BvhNode::build(objects);
                let right = BvhNode::build(right_half);
                let left_box = left.bounding_box().unwrap();
                let right_box = right.bounding_box().unwrap();
                let bbox = Aabb::surrounding(&left_box, &right_box);
                Box::new(BvhNode { left, right, bbox })
            }
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        if !self.bbox.hit(ray, t_min, t_max) {
            return None;
        }

        let left_hit = self.left.hit(ray, t_min, t_max);
        let t_max_right = left_hit.as_ref().map_or(t_max, |h| h.t);
        let right_hit = self.right.hit(ray, t_min, t_max_right);

        right_hit.or(left_hit)
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox)
    }
}

/// Determine which axis has the largest extent across all objects.
fn longest_axis(objects: &[Box<dyn Hittable>]) -> usize {
    let mut min_pt = crate::vec3::Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
    let mut max_pt = crate::vec3::Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

    for obj in objects {
        if let Some(bb) = obj.bounding_box() {
            min_pt = min_pt.min(bb.min);
            max_pt = max_pt.max(bb.max);
        }
    }

    let extent = max_pt - min_pt;
    if extent.x >= extent.y && extent.x >= extent.z {
        0
    } else if extent.y >= extent.z {
        1
    } else {
        2
    }
}

fn axis_val(v: &crate::vec3::Vec3, axis: usize) -> f64 {
    match axis {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::sphere::Sphere;
    use crate::vec3::{Color, Point3, Vec3};

    #[test]
    fn test_bvh_basic_hit() {
        let objects: Vec<Box<dyn Hittable>> = vec![
            Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))))),
            Box::new(Sphere::new(Point3::new(2.0, 0.0, -1.0), 0.5, Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))))),
        ];
        let bvh = BvhNode::build(objects);

        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(bvh.hit(&ray, 0.001, f64::INFINITY).is_some());

        let ray_miss = Ray::new(Point3::new(0.0, 5.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(bvh.hit(&ray_miss, 0.001, f64::INFINITY).is_none());
    }
}
