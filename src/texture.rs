use crate::vec3::{Color, Point3, Vec3};

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

/// Perlin noise generator.
pub struct Perlin {
    ranvec: [Vec3; 256],
    perm_x: [usize; 256],
    perm_y: [usize; 256],
    perm_z: [usize; 256],
}

impl Perlin {
    pub fn new() -> Self {
        let mut rng = rand::rng();

        let mut ranvec = [Vec3::ZERO; 256];
        for v in &mut ranvec {
            *v = Vec3::random_range(&mut rng, -1.0, 1.0).unit();
        }

        Self {
            ranvec,
            perm_x: Self::generate_perm(&mut rng),
            perm_y: Self::generate_perm(&mut rng),
            perm_z: Self::generate_perm(&mut rng),
        }
    }

    fn generate_perm(rng: &mut impl rand::Rng) -> [usize; 256] {
        let mut perm = [0usize; 256];
        for (i, p) in perm.iter_mut().enumerate() {
            *p = i;
        }
        for i in (1..256).rev() {
            let target = rng.random_range(0..=i);
            perm.swap(i, target);
        }
        perm
    }

    pub fn noise(&self, p: &Point3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as i64;
        let j = p.y.floor() as i64;
        let k = p.z.floor() as i64;

        let mut c = [[[Vec3::ZERO; 2]; 2]; 2];
        for di in 0..2i64 {
            for dj in 0..2i64 {
                for dk in 0..2i64 {
                    let idx = self.perm_x[((i + di) & 255) as usize]
                        ^ self.perm_y[((j + dj) & 255) as usize]
                        ^ self.perm_z[((k + dk) & 255) as usize];
                    c[di as usize][dj as usize][dk as usize] = self.ranvec[idx];
                }
            }
        }

        Self::trilinear_interp(&c, u, v, w)
    }

    fn trilinear_interp(c: &[[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        // Hermite cubic for smoothing
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0;
        for (i, ci) in c.iter().enumerate() {
            for (j, cij) in ci.iter().enumerate() {
                for (k, cijk) in cij.iter().enumerate() {
                    let weight = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1 - i) as f64 * (1.0 - uu))
                        * (j as f64 * vv + (1 - j) as f64 * (1.0 - vv))
                        * (k as f64 * ww + (1 - k) as f64 * (1.0 - ww))
                        * cijk.dot(weight);
                }
            }
        }
        accum
    }

    /// Turbulence — sum of absolute noise at multiple frequencies.
    pub fn turb(&self, p: &Point3, depth: u32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = *p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }
}

/// Marble-like texture using Perlin noise turbulence.
pub struct Marble {
    perlin: Perlin,
    scale: f64,
    color: Color,
}

impl Marble {
    pub fn new(color: Color, scale: f64) -> Self {
        Self {
            perlin: Perlin::new(),
            scale,
            color,
        }
    }
}

impl Texture for Marble {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        // Marble: sine with turbulence perturbation
        let noise = 0.5 * (1.0 + (self.scale * point.z + 10.0 * self.perlin.turb(point, 7)).sin());
        self.color * noise
    }
}

/// Turbulent Perlin noise texture.
pub struct Turbulence {
    perlin: Perlin,
    scale: f64,
    color: Color,
}

impl Turbulence {
    pub fn new(color: Color, scale: f64) -> Self {
        Self {
            perlin: Perlin::new(),
            scale,
            color,
        }
    }
}

impl Texture for Turbulence {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        self.color * self.perlin.turb(&(*point * self.scale), 7)
    }
}

/// Image texture — loads a PNG/JPG and maps via UV coordinates.
pub struct ImageTexture {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl ImageTexture {
    pub fn load(path: &str) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("Failed to load image '{}': {e}", path))?;
        let rgb = img.to_rgb8();
        let (width, height) = rgb.dimensions();
        Ok(Self {
            pixels: rgb.into_raw(),
            width,
            height,
        })
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _point: &Point3) -> Color {
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0); // Flip V to match image coords

        let i = ((u * self.width as f64) as u32).min(self.width - 1);
        let j = ((v * self.height as f64) as u32).min(self.height - 1);

        let idx = (j * self.width + i) as usize * 3;
        let r = self.pixels[idx] as f64 / 255.0;
        let g = self.pixels[idx + 1] as f64 / 255.0;
        let b = self.pixels[idx + 2] as f64 / 255.0;

        Color::new(r, g, b)
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

    #[test]
    fn test_perlin_bounded() {
        let perlin = Perlin::new();
        for i in 0..100 {
            let p = Point3::new(i as f64 * 0.1, i as f64 * 0.2, i as f64 * 0.3);
            let n = perlin.noise(&p);
            assert!(n >= -1.0 && n <= 1.0, "Perlin noise out of range: {n}");
        }
    }

    #[test]
    fn test_marble_positive() {
        let marble = Marble::new(Color::new(1.0, 1.0, 1.0), 4.0);
        let c = marble.value(0.0, 0.0, &Point3::new(1.0, 2.0, 3.0));
        assert!(c.x >= 0.0 && c.x <= 1.0);
    }
}
