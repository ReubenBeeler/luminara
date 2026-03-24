use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable};
use crate::ray::Ray;

/// CSG boolean operation type.
pub enum CsgOp {
    Union,
    Intersection,
    Difference,
}

/// Constructive Solid Geometry: boolean operations on two convex objects.
///
/// Works by finding entry/exit intervals for each child along a ray,
/// then applying the boolean operation to determine valid intersections.
pub struct Csg {
    op: CsgOp,
    a: Box<dyn Hittable>,
    b: Box<dyn Hittable>,
}

impl Csg {
    pub fn new(op: CsgOp, a: Box<dyn Hittable>, b: Box<dyn Hittable>) -> Self {
        Self { op, a, b }
    }

    pub fn union(a: Box<dyn Hittable>, b: Box<dyn Hittable>) -> Self {
        Self::new(CsgOp::Union, a, b)
    }

    pub fn intersection(a: Box<dyn Hittable>, b: Box<dyn Hittable>) -> Self {
        Self::new(CsgOp::Intersection, a, b)
    }

    pub fn difference(a: Box<dyn Hittable>, b: Box<dyn Hittable>) -> Self {
        Self::new(CsgOp::Difference, a, b)
    }
}

/// Find entry and exit t-values for a ray hitting a convex object.
/// Returns (entry_t, exit_t) or None if no hit.
fn find_interval(obj: &dyn Hittable, ray: &Ray, t_min: f64, t_max: f64) -> Option<(f64, f64)> {
    let entry = obj.hit(ray, t_min, t_max)?;
    let entry_t = entry.t;
    // Find exit by continuing past the entry point
    let exit = obj.hit(ray, entry_t + 0.0001, t_max);
    let exit_t = exit.map(|h| h.t).unwrap_or(entry_t);
    Some((entry_t, exit_t))
}

impl Hittable for Csg {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        match self.op {
            CsgOp::Union => {
                // Union: nearest hit from either child
                let hit_a = self.a.hit(ray, t_min, t_max);
                let t_limit = hit_a.as_ref().map_or(t_max, |h| h.t);
                let hit_b = self.b.hit(ray, t_min, t_limit);
                hit_b.or(hit_a)
            }
            CsgOp::Intersection => {
                // Intersection: find where ray is inside BOTH objects
                let (a_entry, a_exit) = find_interval(&*self.a, ray, t_min, t_max)?;
                let (b_entry, b_exit) = find_interval(&*self.b, ray, t_min, t_max)?;

                // Overlap interval
                let overlap_start = a_entry.max(b_entry);
                let overlap_end = a_exit.min(b_exit);

                if overlap_start > overlap_end {
                    return None; // No overlap
                }

                if overlap_start < t_min || overlap_start > t_max {
                    return None;
                }

                // Return the hit at the start of the overlap
                // The surface we enter is whichever object we entered last
                if a_entry >= b_entry {
                    // Entering A is the start of the overlap — use A's hit
                    self.a.hit(ray, a_entry - 0.0001, a_entry + 0.0001)
                } else {
                    // Entering B is the start of the overlap — use B's hit
                    self.b.hit(ray, b_entry - 0.0001, b_entry + 0.0001)
                }
            }
            CsgOp::Difference => {
                // Difference (A - B): where ray is inside A but NOT inside B
                let (a_entry, a_exit) = find_interval(&*self.a, ray, t_min, t_max)?;

                // Check if B overlaps with A's interval
                let b_interval = find_interval(&*self.b, ray, t_min, t_max);

                match b_interval {
                    None => {
                        // B doesn't intersect — just return A's entry
                        self.a.hit(ray, t_min, t_max)
                    }
                    Some((b_entry, b_exit)) => {
                        // Case 1: B covers A's entry — the visible surface starts at B's exit
                        // with inverted normal (we see the inside of B)
                        if b_entry <= a_entry && b_exit > a_entry && b_exit < a_exit {
                            // Re-hit B at exit, flip its normal
                            if let Some(mut hit) = self.b.hit(ray, b_exit - 0.0001, b_exit + 0.0001) {
                                hit.normal = -hit.normal;
                                hit.front_face = !hit.front_face;
                                return Some(hit);
                            }
                            return None;
                        }
                        // Case 2: B starts after A's entry — A's entry is valid
                        if b_entry > a_entry {
                            return self.a.hit(ray, t_min, t_max);
                        }
                        // Case 3: B fully covers A — no visible surface
                        if b_entry <= a_entry && b_exit >= a_exit {
                            return None;
                        }
                        // Default: return A's hit
                        self.a.hit(ray, t_min, t_max)
                    }
                }
            }
        }
    }

    fn bounding_box(&self) -> Option<Aabb> {
        match self.op {
            CsgOp::Union => {
                match (self.a.bounding_box(), self.b.bounding_box()) {
                    (Some(a), Some(b)) => Some(Aabb::surrounding(&a, &b)),
                    (Some(a), None) | (None, Some(a)) => Some(a),
                    (None, None) => None,
                }
            }
            CsgOp::Intersection => {
                // Intersection is bounded by the smaller of the two
                match (self.a.bounding_box(), self.b.bounding_box()) {
                    (Some(a), Some(b)) => Some(Aabb::intersection(&a, &b)),
                    _ => None,
                }
            }
            CsgOp::Difference => {
                // Difference is bounded by A
                self.a.bounding_box()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::sphere::Sphere;
    use crate::vec3::{Color, Point3, Vec3};

    fn white_sphere(center: Point3, radius: f64) -> Box<dyn Hittable> {
        Box::new(Sphere::new(center, radius, Box::new(Lambertian::new(Color::new(1.0, 1.0, 1.0)))))
    }

    #[test]
    fn union_hits_either() {
        let csg = Csg::union(
            white_sphere(Point3::new(0.0, 0.0, -2.0), 1.0),
            white_sphere(Point3::new(3.0, 0.0, -2.0), 1.0),
        );
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(csg.hit(&ray, 0.001, f64::INFINITY).is_some());

        let ray2 = Ray::new(Point3::new(3.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(csg.hit(&ray2, 0.001, f64::INFINITY).is_some());

        let ray_miss = Ray::new(Point3::new(10.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(csg.hit(&ray_miss, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn union_returns_nearest() {
        let csg = Csg::union(
            white_sphere(Point3::new(0.0, 0.0, -5.0), 1.0),
            white_sphere(Point3::new(0.0, 0.0, -2.0), 1.0),
        );
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = csg.hit(&ray, 0.001, f64::INFINITY).unwrap();
        // Should hit the nearer sphere (at z=-2, t=1.0)
        assert!(hit.t < 2.0, "Should hit nearer sphere, got t={}", hit.t);
    }

    #[test]
    fn intersection_overlapping_spheres() {
        // Two overlapping spheres — intersection should hit only in overlap region
        let csg = Csg::intersection(
            white_sphere(Point3::new(0.0, 0.0, -2.0), 1.5),
            white_sphere(Point3::new(0.0, 0.0, -3.0), 1.5),
        );
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = csg.hit(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some(), "Should hit intersection of overlapping spheres");
    }

    #[test]
    fn intersection_non_overlapping_misses() {
        // Two non-overlapping spheres — intersection should miss
        let csg = Csg::intersection(
            white_sphere(Point3::new(0.0, 0.0, -2.0), 0.5),
            white_sphere(Point3::new(5.0, 0.0, -2.0), 0.5),
        );
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(csg.hit(&ray, 0.001, f64::INFINITY).is_none());
    }

    #[test]
    fn difference_carves_hole() {
        // Large sphere minus a small sphere at the same center
        // Ray through center should NOT hit (interior carved out)
        let csg = Csg::difference(
            white_sphere(Point3::new(0.0, 0.0, -3.0), 2.0),
            white_sphere(Point3::new(0.0, 0.0, -3.0), 1.5),
        );
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = csg.hit(&ray, 0.001, f64::INFINITY);
        // B fully covers A's entry, B exits before A — should see B's exit surface
        // The hit should exist (seeing the carved interior wall)
        assert!(hit.is_some(), "Should see carved interior of difference");
    }

    #[test]
    fn difference_no_overlap_returns_a() {
        // A - B where B doesn't overlap A — should just return A
        let csg = Csg::difference(
            white_sphere(Point3::new(0.0, 0.0, -3.0), 1.0),
            white_sphere(Point3::new(10.0, 0.0, -3.0), 1.0),
        );
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = csg.hit(&ray, 0.001, f64::INFINITY).unwrap();
        assert!((hit.t - 2.0).abs() < 0.1, "Should hit A at t~2.0, got {}", hit.t);
    }

    #[test]
    fn csg_has_bounding_box() {
        let u = Csg::union(
            white_sphere(Point3::new(0.0, 0.0, 0.0), 1.0),
            white_sphere(Point3::new(3.0, 0.0, 0.0), 1.0),
        );
        assert!(u.bounding_box().is_some());

        let d = Csg::difference(
            white_sphere(Point3::new(0.0, 0.0, 0.0), 1.0),
            white_sphere(Point3::new(0.5, 0.0, 0.0), 0.5),
        );
        assert!(d.bounding_box().is_some());
    }
}
