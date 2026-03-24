use crate::vec3::{Color, Point3};

/// Trait for procedural and sampled textures.
pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, point: &Point3) -> Color;
}

/// A solid color texture.
pub struct SolidColor {
    pub color: Color,
}

impl SolidColor {
    pub const fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _point: &Point3) -> Color {
        self.color
    }
}

/// A 3D checkerboard pattern.
pub struct Checker {
    pub even: Color,
    pub odd: Color,
    pub scale: f64,
}

impl Checker {
    pub fn new(even: Color, odd: Color, scale: f64) -> Self {
        Self { even, odd, scale }
    }
}

impl Texture for Checker {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        let inv_scale = 1.0 / self.scale;
        let x = (point.x * inv_scale).floor() as i64;
        let y = (point.y * inv_scale).floor() as i64;
        let z = (point.z * inv_scale).floor() as i64;
        if (x + y + z) % 2 == 0 {
            self.even
        } else {
            self.odd
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_color() {
        let tex = SolidColor::new(Color::new(0.5, 0.5, 0.5));
        let c = tex.value(0.0, 0.0, &Point3::ZERO);
        assert_eq!(c, Color::new(0.5, 0.5, 0.5));
    }

    #[test]
    fn test_checker() {
        let tex = Checker::new(Color::new(1.0, 1.0, 1.0), Color::new(0.0, 0.0, 0.0), 1.0);
        let c1 = tex.value(0.0, 0.0, &Point3::new(0.5, 0.5, 0.5));
        let c2 = tex.value(0.0, 0.0, &Point3::new(1.5, 0.5, 0.5));
        assert_eq!(c1, Color::new(1.0, 1.0, 1.0)); // even (0+0+0)
        assert_eq!(c2, Color::new(0.0, 0.0, 0.0)); // odd (1+0+0)
    }
}
