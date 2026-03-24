use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::ray::Ray;

/// A node in the bounding volume hierarchy.
pub struct BvhNode {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
    bbox: Aabb,
}

const SAH_BUCKET_COUNT: usize = 12;

impl BvhNode {
    /// Build a BVH from a list of hittable objects using SAH.
    pub fn build(mut objects: Vec<Box<dyn Hittable>>) -> Box<dyn Hittable> {
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
                // Find best split using SAH across all 3 axes
                let (best_axis, best_mid) = find_best_split(&objects);

                objects.sort_by(|a, b| {
                    let a_c = a.bounding_box().map(|bb| centroid_axis(&bb, best_axis)).unwrap_or(0.0);
                    let b_c = b.bounding_box().map(|bb| centroid_axis(&bb, best_axis)).unwrap_or(0.0);
                    a_c.partial_cmp(&b_c).unwrap_or(std::cmp::Ordering::Equal)
                });

                let mid = best_mid.clamp(1, n - 1);
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

/// Find the best axis and split position using Surface Area Heuristic.
fn find_best_split(objects: &[Box<dyn Hittable>]) -> (usize, usize) {
    let n = objects.len();

    // Compute overall bounding box and centroid bounds
    let mut overall_bb = objects[0].bounding_box().unwrap();
    let mut centroid_min = crate::vec3::Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
    let mut centroid_max = crate::vec3::Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

    for obj in objects {
        if let Some(bb) = obj.bounding_box() {
            overall_bb = Aabb::surrounding(&overall_bb, &bb);
            let c = (bb.min + bb.max) * 0.5;
            centroid_min = centroid_min.min(c);
            centroid_max = centroid_max.max(c);
        }
    }

    let overall_sa = overall_bb.surface_area();
    if overall_sa < 1e-10 {
        return (0, n / 2);
    }

    let mut best_axis = 0;
    let mut best_cost = f64::INFINITY;
    let mut best_mid = n / 2;

    for axis in 0..3 {
        let extent = axis_val(&centroid_max, axis) - axis_val(&centroid_min, axis);
        if extent < 1e-10 {
            continue;
        }

        // Bucket sort centroids
        let mut buckets = vec![(0usize, None::<Aabb>); SAH_BUCKET_COUNT];

        for obj in objects {
            if let Some(bb) = obj.bounding_box() {
                let c = (axis_val(&bb.min, axis) + axis_val(&bb.max, axis)) * 0.5;
                let b = ((c - axis_val(&centroid_min, axis)) / extent * SAH_BUCKET_COUNT as f64) as usize;
                let b = b.min(SAH_BUCKET_COUNT - 1);
                buckets[b].0 += 1;
                buckets[b].1 = Some(match buckets[b].1 {
                    Some(existing) => Aabb::surrounding(&existing, &bb),
                    None => bb,
                });
            }
        }

        // Evaluate SAH cost at each split
        for split in 1..SAH_BUCKET_COUNT {
            let mut left_count = 0;
            let mut left_bb: Option<Aabb> = None;
            for bucket in &buckets[..split] {
                left_count += bucket.0;
                if let Some(bb) = bucket.1 {
                    left_bb = Some(match left_bb {
                        Some(existing) => Aabb::surrounding(&existing, &bb),
                        None => bb,
                    });
                }
            }

            let mut right_count = 0;
            let mut right_bb: Option<Aabb> = None;
            for bucket in &buckets[split..] {
                right_count += bucket.0;
                if let Some(bb) = bucket.1 {
                    right_bb = Some(match right_bb {
                        Some(existing) => Aabb::surrounding(&existing, &bb),
                        None => bb,
                    });
                }
            }

            if left_count == 0 || right_count == 0 {
                continue;
            }

            let left_sa = left_bb.map(|b| b.surface_area()).unwrap_or(0.0);
            let right_sa = right_bb.map(|b| b.surface_area()).unwrap_or(0.0);

            let cost = 1.0 + (left_count as f64 * left_sa + right_count as f64 * right_sa) / overall_sa;

            if cost < best_cost {
                best_cost = cost;
                best_axis = axis;
                best_mid = left_count;
            }
        }
    }

    (best_axis, best_mid)
}

fn centroid_axis(bb: &Aabb, axis: usize) -> f64 {
    (axis_val(&bb.min, axis) + axis_val(&bb.max, axis)) * 0.5
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

    #[test]
    fn test_bvh_many_objects() {
        let mut objects: Vec<Box<dyn Hittable>> = Vec::new();
        for i in 0..50 {
            objects.push(Box::new(Sphere::new(
                Point3::new(i as f64 * 2.0, 0.0, 0.0),
                0.5,
                Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))),
            )));
        }
        let bvh = BvhNode::build(objects);

        // Hit the first sphere
        let ray = Ray::new(Point3::new(0.0, 0.0, -2.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(bvh.hit(&ray, 0.001, f64::INFINITY).is_some());

        // Miss all spheres
        let ray_miss = Ray::new(Point3::new(0.0, 5.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(bvh.hit(&ray_miss, 0.001, f64::INFINITY).is_none());
    }
}
