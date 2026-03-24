use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub};

use rand::Rng;

/// A 3D vector used for positions, directions, and colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Alias for color values.
pub type Color = Vec3;
/// Alias for point in 3D space.
pub type Point3 = Vec3;

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn unit(self) -> Self {
        let len = self.length();
        debug_assert!(len > 0.0, "Cannot normalize zero-length vector");
        self / len
    }

    pub fn near_zero(self) -> bool {
        const EPS: f64 = 1e-8;
        self.x.abs() < EPS && self.y.abs() < EPS && self.z.abs() < EPS
    }

    pub fn reflect(self, normal: Self) -> Self {
        self - normal * 2.0 * self.dot(normal)
    }

    pub fn refract(self, normal: Self, eta_ratio: f64) -> Self {
        let cos_theta = (-self).dot(normal).min(1.0);
        let r_perp = (self + normal * cos_theta) * eta_ratio;
        let r_parallel = normal * -(1.0 - r_perp.length_squared()).abs().sqrt();
        r_perp + r_parallel
    }

    /// Random vector with components in [0, 1).
    pub fn random(rng: &mut impl Rng) -> Self {
        Self::new(rng.random::<f64>(), rng.random::<f64>(), rng.random::<f64>())
    }

    /// Random vector with components in [min, max).
    pub fn random_range(rng: &mut impl Rng, min: f64, max: f64) -> Self {
        let range = max - min;
        Self::new(
            min + rng.random::<f64>() * range,
            min + rng.random::<f64>() * range,
            min + rng.random::<f64>() * range,
        )
    }

    /// Random point inside the unit sphere.
    pub fn random_in_unit_sphere(rng: &mut impl Rng) -> Self {
        loop {
            let v = Self::random_range(rng, -1.0, 1.0);
            if v.length_squared() < 1.0 {
                return v;
            }
        }
    }

    /// Random unit vector (Lambertian distribution).
    pub fn random_unit_vector(rng: &mut impl Rng) -> Self {
        Self::random_in_unit_sphere(rng).unit()
    }

    /// Random point inside the unit disk (for depth of field).
    pub fn random_in_unit_disk(rng: &mut impl Rng) -> Self {
        loop {
            let v = Self::new(
                rng.random::<f64>() * 2.0 - 1.0,
                rng.random::<f64>() * 2.0 - 1.0,
                0.0,
            );
            if v.length_squared() < 1.0 {
                return v;
            }
        }
    }

    /// Component-wise minimum.
    pub fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    /// Component-wise maximum.
    pub fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    /// Component-wise multiplication (Hadamard product), used for color blending.
    pub fn hadamard(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

// --- Operator implementations ---

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, t: f64) -> Self {
        Self::new(self.x * t, self.y * t, self.z * t)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        v * self
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, t: f64) {
        self.x *= t;
        self.y *= t;
        self.z *= t;
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, t: f64) -> Self {
        Self::new(self.x / t, self.y / t, self.z / t)
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, t: f64) {
        let inv = 1.0 / t;
        self.x *= inv;
        self.y *= inv;
        self.z *= inv;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ops() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        let sum = a + b;
        assert_eq!(sum, Vec3::new(5.0, 7.0, 9.0));

        let diff = b - a;
        assert_eq!(diff, Vec3::new(3.0, 3.0, 3.0));

        let scaled = a * 2.0;
        assert_eq!(scaled, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_dot_cross() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);

        assert!((a.dot(b)).abs() < 1e-10);
        assert_eq!(a.cross(b), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_unit_vector() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let u = v.unit();
        assert!((u.length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_reflect() {
        let v = Vec3::new(1.0, -1.0, 0.0);
        let n = Vec3::new(0.0, 1.0, 0.0);
        let r = v.reflect(n);
        assert!((r.x - 1.0).abs() < 1e-10);
        assert!((r.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_hadamard() {
        let a = Vec3::new(0.5, 0.3, 0.1);
        let b = Vec3::new(0.2, 0.4, 0.6);
        let h = a.hadamard(b);
        assert!((h.x - 0.1).abs() < 1e-10);
        assert!((h.y - 0.12).abs() < 1e-10);
        assert!((h.z - 0.06).abs() < 1e-10);
    }
}
