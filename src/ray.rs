use crate::vec3::{Point3, Vec3};

/// A ray: origin + direction * t, with a time component for motion blur.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub time: f64,
}

impl Ray {
    pub const fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction, time: 0.0 }
    }

    pub const fn with_time(origin: Point3, direction: Vec3, time: f64) -> Self {
        Self { origin, direction, time }
    }

    #[inline(always)]
    pub fn at(self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }
}
