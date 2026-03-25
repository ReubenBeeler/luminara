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
        Self { even, odd, scale: if scale.abs() < 1e-10 { 1.0 } else { scale } }
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

/// Stripe pattern along a specified axis.
pub struct Stripe {
    pub color1: Color,
    pub color2: Color,
    pub scale: f64,
    pub axis: usize, // 0=X, 1=Y, 2=Z
}

impl Stripe {
    pub fn new(color1: Color, color2: Color, scale: f64, axis: usize) -> Self {
        Self { color1, color2, scale: if scale.abs() < 1e-10 { 1.0 } else { scale }, axis: axis.min(2) }
    }
}

impl Texture for Stripe {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        let val = match self.axis {
            0 => point.x,
            1 => point.y,
            _ => point.z,
        };
        if ((val / self.scale).floor() as i64) % 2 == 0 {
            self.color1
        } else {
            self.color2
        }
    }
}

/// Gradient texture — blends between two colors along an axis.
pub struct GradientTexture {
    pub color1: Color,
    pub color2: Color,
    pub axis: usize,
    pub min_val: f64,
    pub max_val: f64,
}

impl GradientTexture {
    pub fn new(color1: Color, color2: Color, axis: usize, min_val: f64, max_val: f64) -> Self {
        let range = max_val - min_val;
        let (min_val, max_val) = if range.abs() < 1e-10 {
            (min_val, min_val + 1.0)
        } else {
            (min_val, max_val)
        };
        Self { color1, color2, axis: axis.min(2), min_val, max_val }
    }
}

impl Texture for GradientTexture {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        let val = match self.axis {
            0 => point.x,
            1 => point.y,
            _ => point.z,
        };
        let t = ((val - self.min_val) / (self.max_val - self.min_val)).clamp(0.0, 1.0);
        self.color1 * (1.0 - t) + self.color2 * t
    }
}

/// Wood-grain rings in the XZ plane.
pub struct Rings {
    pub color1: Color,
    pub color2: Color,
    pub scale: f64,
}

impl Rings {
    pub fn new(color1: Color, color2: Color, scale: f64) -> Self {
        Self { color1, color2, scale: if scale.abs() < 1e-10 { 1.0 } else { scale } }
    }
}

impl Texture for Rings {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        let dist = (point.x * point.x + point.z * point.z).sqrt() * self.scale;
        if (dist.floor() as i64) % 2 == 0 {
            self.color1
        } else {
            self.color2
        }
    }
}

/// Wood texture — Perlin-perturbed concentric rings.
pub struct Wood {
    color1: Color,
    color2: Color,
    scale: f64,
    perlin: Perlin,
}

impl Wood {
    pub fn new(color1: Color, color2: Color, scale: f64) -> Self {
        Self {
            color1,
            color2,
            scale: if scale.abs() < 1e-10 { 4.0 } else { scale },
            perlin: Perlin::new(),
        }
    }
}

impl Texture for Wood {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        let dist = (point.x * point.x + point.z * point.z).sqrt() * self.scale;
        let noise = self.perlin.turb(point, 4) * 10.0;
        let ring = ((dist + noise).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        self.color1 * ring + self.color2 * (1.0 - ring)
    }
}

/// 3D polka-dot pattern.
pub struct Dots {
    pub dot_color: Color,
    pub bg_color: Color,
    pub scale: f64,
    pub radius: f64,
}

impl Dots {
    pub fn new(dot_color: Color, bg_color: Color, scale: f64, radius: f64) -> Self {
        Self {
            dot_color,
            bg_color,
            scale: if scale.abs() < 1e-10 { 1.0 } else { scale },
            radius: radius.clamp(0.01, 0.49),
        }
    }
}

impl Texture for Dots {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        let inv = 1.0 / self.scale;
        let fx = (point.x * inv).fract().abs() - 0.5;
        let fy = (point.y * inv).fract().abs() - 0.5;
        let fz = (point.z * inv).fract().abs() - 0.5;

        let dist_sq = fx * fx + fy * fy + fz * fz;
        if dist_sq < self.radius * self.radius {
            self.dot_color
        } else {
            self.bg_color
        }
    }
}

/// 3D grid pattern with thin lines.
pub struct Grid {
    pub line_color: Color,
    pub bg_color: Color,
    pub scale: f64,
    pub line_width: f64,
}

impl Grid {
    pub fn new(line_color: Color, bg_color: Color, scale: f64, line_width: f64) -> Self {
        Self {
            line_color,
            bg_color,
            scale: if scale.abs() < 1e-10 { 1.0 } else { scale },
            line_width: line_width.clamp(0.01, 0.5),
        }
    }
}

impl Texture for Grid {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        let inv = 1.0 / self.scale;
        let fx = (point.x * inv).fract().abs();
        let fy = (point.y * inv).fract().abs();
        let fz = (point.z * inv).fract().abs();

        let hw = self.line_width * 0.5;
        if fx < hw || fx > 1.0 - hw || fy < hw || fy > 1.0 - hw || fz < hw || fz > 1.0 - hw {
            self.line_color
        } else {
            self.bg_color
        }
    }
}

/// UV-based checkerboard that uses texture coordinates instead of world position.
pub struct UvChecker {
    pub even: Color,
    pub odd: Color,
    pub frequency: f64,
}

impl UvChecker {
    pub fn new(even: Color, odd: Color, frequency: f64) -> Self {
        Self { even, odd, frequency: if frequency.abs() < 1e-10 { 10.0 } else { frequency } }
    }
}

impl Texture for UvChecker {
    fn value(&self, u: f64, v: f64, _point: &Point3) -> Color {
        let su = (u * self.frequency).floor() as i64;
        let sv = (v * self.frequency).floor() as i64;
        if (su + sv) % 2 == 0 {
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

    /// Fallback 1x1 magenta texture for when image loading fails.
    pub fn fallback() -> Self {
        Self {
            pixels: vec![255, 0, 255],
            width: 1,
            height: 1,
        }
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

/// Voronoi (cell) texture pattern — creates organic cell-like patterns.
pub struct Voronoi {
    pub color1: Color,
    pub color2: Color,
    pub scale: f64,
}

impl Voronoi {
    pub fn new(color1: Color, color2: Color, scale: f64) -> Self {
        Self { color1, color2, scale: if scale.abs() < 1e-10 { 1.0 } else { scale } }
    }

    /// Hash a 3D integer coordinate to a pseudo-random point offset.
    fn hash_point(ix: i64, iy: i64, iz: i64) -> Vec3 {
        // Simple hash using large primes
        let n = ix.wrapping_mul(73856093) ^ iy.wrapping_mul(19349663) ^ iz.wrapping_mul(83492791);
        let fx = ((n & 0xFF) as f64) / 255.0;
        let fy = (((n >> 8) & 0xFF) as f64) / 255.0;
        let fz = (((n >> 16) & 0xFF) as f64) / 255.0;
        Vec3::new(fx, fy, fz)
    }
}

impl Texture for Voronoi {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        let p = *point / self.scale;
        let ix = p.x.floor() as i64;
        let iy = p.y.floor() as i64;
        let iz = p.z.floor() as i64;

        let mut min_dist = f64::INFINITY;

        // Check surrounding 27 cells
        for di in -1..=1 {
            for dj in -1..=1 {
                for dk in -1..=1 {
                    let ci = ix + di;
                    let cj = iy + dj;
                    let ck = iz + dk;
                    let offset = Self::hash_point(ci, cj, ck);
                    let cell_point = Vec3::new(ci as f64 + offset.x, cj as f64 + offset.y, ck as f64 + offset.z);
                    let dist = (p - cell_point).length_squared();
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
            }
        }

        let t = min_dist.sqrt().min(1.0);
        self.color1 * (1.0 - t) + self.color2 * t
    }
}

/// Hexagonal grid pattern in the XZ plane.
pub struct Hexgrid {
    pub color1: Color,
    pub color2: Color,
    pub scale: f64,
    pub line_width: f64,
}

impl Hexgrid {
    pub fn new(color1: Color, color2: Color, scale: f64, line_width: f64) -> Self {
        Self {
            color1,
            color2,
            scale: if scale.abs() < 1e-10 { 1.0 } else { scale },
            line_width: line_width.clamp(0.01, 0.5),
        }
    }
}

impl Texture for Hexgrid {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        let inv = 1.0 / self.scale;
        let x = point.x * inv;
        let z = point.z * inv;

        // Hexagonal coordinate conversion
        let sqrt3 = 3.0_f64.sqrt();
        // Axial hex coordinates
        let q = (2.0 / 3.0) * x;
        let r = (-1.0 / 3.0) * x + (sqrt3 / 3.0) * z;

        // Round to nearest hex center (cube coordinate rounding)
        let s = -q - r;
        let mut rq = q.round();
        let mut rr = r.round();
        let rs = s.round();
        let dq = (rq - q).abs();
        let dr = (rr - r).abs();
        let ds = (rs - s).abs();
        if dq > dr && dq > ds {
            rq = -rr - rs;
        } else if dr > ds {
            rr = -rq - rs;
        }

        // Distance from nearest hex center (in axial coordinates)
        let dist_q = q - rq;
        let dist_r = r - rr;
        // Convert back to cartesian distance
        let cx = dist_q * 1.5;
        let cz = dist_q * sqrt3 / 2.0 + dist_r * sqrt3;
        let dist = (cx * cx + cz * cz).sqrt();

        // Hex edge detection: if close to hex boundary, show line color
        let hex_radius = sqrt3 / 3.0;
        if dist > hex_radius * (1.0 - self.line_width) {
            self.color2
        } else {
            self.color1
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

    #[test]
    fn test_perlin_bounded() {
        let perlin = Perlin::new();
        for i in 0..100 {
            let p = Point3::new(i as f64 * 0.1, i as f64 * 0.2, i as f64 * 0.3);
            let n = perlin.noise(&p);
            assert!((-1.0..=1.0).contains(&n), "Perlin noise out of range: {n}");
        }
    }

    #[test]
    fn test_voronoi_in_range() {
        let tex = Voronoi::new(Color::new(1.0, 0.0, 0.0), Color::new(0.0, 0.0, 1.0), 1.0);
        for i in 0..20 {
            let p = Point3::new(i as f64 * 0.37, i as f64 * 0.53, i as f64 * 0.17);
            let c = tex.value(0.0, 0.0, &p);
            assert!(c.x >= 0.0 && c.x <= 1.0, "r={} out of range", c.x);
            assert!(c.z >= 0.0 && c.z <= 1.0, "b={} out of range", c.z);
        }
    }

    #[test]
    fn test_marble_positive() {
        let marble = Marble::new(Color::new(1.0, 1.0, 1.0), 4.0);
        let c = marble.value(0.0, 0.0, &Point3::new(1.0, 2.0, 3.0));
        assert!(c.x >= 0.0 && c.x <= 1.0);
    }
}
