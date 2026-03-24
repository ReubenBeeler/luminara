use std::sync::Arc;

use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable, HittableList};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// Load an OBJ file and return a HittableList of triangles.
/// Supports vertices (v) and faces (f) with fan triangulation.
/// Parsed face vertex: position index, optional UV and normal indices.
struct FaceVertex {
    pos: usize,
    uv: Option<usize>,
    normal: Option<usize>,
}

pub fn load_obj(
    content: &str,
    material: Box<dyn Material>,
    scale: f64,
    offset: Point3,
) -> Result<HittableList, String> {
    let mut vertices: Vec<Point3> = Vec::new();
    let mut tex_coords: Vec<[f64; 2]> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut faces: Vec<[FaceVertex; 3]> = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.first() {
            Some(&"v") => {
                if parts.len() < 4 {
                    return Err(format!("Line {}: vertex needs 3 coordinates", line_num + 1));
                }
                let x: f64 = parts[1].parse().map_err(|e| format!("Line {}: {e}", line_num + 1))?;
                let y: f64 = parts[2].parse().map_err(|e| format!("Line {}: {e}", line_num + 1))?;
                let z: f64 = parts[3].parse().map_err(|e| format!("Line {}: {e}", line_num + 1))?;
                vertices.push(Point3::new(
                    x * scale + offset.x,
                    y * scale + offset.y,
                    z * scale + offset.z,
                ));
            }
            Some(&"vt") => {
                if parts.len() < 3 {
                    return Err(format!("Line {}: texture coord needs 2 values", line_num + 1));
                }
                let u: f64 = parts[1].parse().map_err(|e| format!("Line {}: {e}", line_num + 1))?;
                let v: f64 = parts[2].parse().map_err(|e| format!("Line {}: {e}", line_num + 1))?;
                tex_coords.push([u, v]);
            }
            Some(&"vn") => {
                if parts.len() < 4 {
                    return Err(format!("Line {}: normal needs 3 components", line_num + 1));
                }
                let x: f64 = parts[1].parse().map_err(|e| format!("Line {}: {e}", line_num + 1))?;
                let y: f64 = parts[2].parse().map_err(|e| format!("Line {}: {e}", line_num + 1))?;
                let z: f64 = parts[3].parse().map_err(|e| format!("Line {}: {e}", line_num + 1))?;
                normals.push(Vec3::new(x, y, z).unit());
            }
            Some(&"f") => {
                let face_verts: Result<Vec<FaceVertex>, String> = parts[1..]
                    .iter()
                    .map(|p| {
                        let segments: Vec<&str> = p.split('/').collect();
                        let pos = segments[0]
                            .parse::<usize>()
                            .map_err(|e| format!("Line {}: {e}", line_num + 1))?;
                        let uv = if segments.len() >= 2 && !segments[1].is_empty() {
                            Some(segments[1].parse::<usize>()
                                .map_err(|e| format!("Line {}: {e}", line_num + 1))?)
                        } else {
                            None
                        };
                        let normal = if segments.len() >= 3 && !segments[2].is_empty() {
                            Some(segments[2].parse::<usize>()
                                .map_err(|e| format!("Line {}: {e}", line_num + 1))?)
                        } else {
                            None
                        };
                        Ok(FaceVertex { pos, uv, normal })
                    })
                    .collect();
                let face_verts = face_verts?;

                if face_verts.len() < 3 {
                    return Err(format!("Line {}: face needs at least 3 vertices", line_num + 1));
                }
                // Fan triangulation
                for i in 1..face_verts.len() - 1 {
                    faces.push([
                        FaceVertex { pos: face_verts[0].pos, uv: face_verts[0].uv, normal: face_verts[0].normal },
                        FaceVertex { pos: face_verts[i].pos, uv: face_verts[i].uv, normal: face_verts[i].normal },
                        FaceVertex { pos: face_verts[i + 1].pos, uv: face_verts[i + 1].uv, normal: face_verts[i + 1].normal },
                    ]);
                }
            }
            _ => {}
        }
    }

    let shared_mat: Arc<dyn Material> = Arc::from(material);
    let mut list = HittableList::new();

    for face in &faces {
        let v0 = *vertices.get(face[0].pos - 1).ok_or_else(|| format!("Vertex index {} out of range", face[0].pos))?;
        let v1 = *vertices.get(face[1].pos - 1).ok_or_else(|| format!("Vertex index {} out of range", face[1].pos))?;
        let v2 = *vertices.get(face[2].pos - 1).ok_or_else(|| format!("Vertex index {} out of range", face[2].pos))?;

        let smooth_normals = if let (Some(n0), Some(n1), Some(n2)) = (face[0].normal, face[1].normal, face[2].normal) {
            let nn0 = normals.get(n0 - 1).copied();
            let nn1 = normals.get(n1 - 1).copied();
            let nn2 = normals.get(n2 - 1).copied();
            match (nn0, nn1, nn2) {
                (Some(a), Some(b), Some(c)) => Some([a, b, c]),
                _ => None,
            }
        } else {
            None
        };

        let uvs = if let (Some(t0), Some(t1), Some(t2)) = (face[0].uv, face[1].uv, face[2].uv) {
            let tc0 = tex_coords.get(t0 - 1).copied();
            let tc1 = tex_coords.get(t1 - 1).copied();
            let tc2 = tex_coords.get(t2 - 1).copied();
            match (tc0, tc1, tc2) {
                (Some(a), Some(b), Some(c)) => Some([a, b, c]),
                _ => None,
            }
        } else {
            None
        };

        list.add(Box::new(MeshTriangle {
            v0,
            v1,
            v2,
            uvs,
            normals: smooth_normals,
            material: Arc::clone(&shared_mat),
        }));
    }

    eprintln!("Loaded OBJ: {} vertices, {} tex coords, {} normals, {} triangles",
        vertices.len(), tex_coords.len(), normals.len(), faces.len());
    Ok(list)
}

/// A triangle that shares its material via Arc for mesh efficiency.
/// Optionally stores per-vertex normals for smooth (Phong) shading.
struct MeshTriangle {
    v0: Point3,
    v1: Point3,
    v2: Point3,
    uvs: Option<[[f64; 2]; 3]>,
    normals: Option<[Vec3; 3]>,
    material: Arc<dyn Material>,
}

impl Hittable for MeshTriangle {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        if a.abs() < 1e-8 {
            return None;
        }

        let f = 1.0 / a;
        let s = ray.origin - self.v0;
        let u = f * s.dot(h);

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * ray.direction.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);
        if t < t_min || t > t_max {
            return None;
        }

        let point = ray.at(t);
        let outward_normal = if let Some([n0, n1, n2]) = self.normals {
            (n0 * (1.0 - u - v) + n1 * u + n2 * v).unit()
        } else {
            edge1.cross(edge2).unit()
        };
        // Interpolate UV coordinates if available
        let (tex_u, tex_v) = if let Some([uv0, uv1, uv2]) = self.uvs {
            let w = 1.0 - u - v;
            (uv0[0] * w + uv1[0] * u + uv2[0] * v,
             uv0[1] * w + uv1[1] * u + uv2[1] * v)
        } else {
            (u, v)
        };
        Some(HitRecord::new(
            ray,
            point,
            outward_normal,
            t,
            tex_u,
            tex_v,
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self) -> Option<Aabb> {
        let min = self.v0.min(self.v1).min(self.v2) - Vec3::new(1e-4, 1e-4, 1e-4);
        let max = self.v0.max(self.v1).max(self.v2) + Vec3::new(1e-4, 1e-4, 1e-4);
        Some(Aabb::new(min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;
    use crate::vec3::Color;

    #[test]
    fn test_load_quad_triangulates() {
        let obj = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let mat = Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        let list = load_obj(obj, mat, 1.0, Point3::ZERO).unwrap();
        assert_eq!(list.objects.len(), 2);
    }

    #[test]
    fn test_load_triangle() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let mat = Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        let list = load_obj(obj, mat, 1.0, Point3::ZERO).unwrap();
        assert_eq!(list.objects.len(), 1);
    }

    #[test]
    fn test_scale_and_offset() {
        let obj = "v 1 1 1\nv 2 1 1\nv 1 2 1\nf 1 2 3\n";
        let mat = Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        let list = load_obj(obj, mat, 2.0, Point3::new(1.0, 0.0, 0.0)).unwrap();
        assert_eq!(list.objects.len(), 1);
        // Verify the bounding box reflects the scale and offset
        let bb = list.objects[0].bounding_box().unwrap();
        assert!(bb.min.x > 2.9); // 1*2+1 = 3
        assert!(bb.max.x < 5.1); // 2*2+1 = 5
    }

    #[test]
    fn test_load_with_uvs() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nf 1/1 2/2 3/3\n";
        let mat = Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        let list = load_obj(obj, mat, 1.0, Point3::ZERO).unwrap();
        assert_eq!(list.objects.len(), 1);
    }

    #[test]
    fn test_load_full_format() {
        // f v/vt/vn format
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nvn 0 0 1\nvn 0 0 1\nvn 0 0 1\nf 1/1/1 2/2/2 3/3/3\n";
        let mat = Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        let list = load_obj(obj, mat, 1.0, Point3::ZERO).unwrap();
        assert_eq!(list.objects.len(), 1);
    }

    #[test]
    fn test_load_with_normals() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 1\nvn 0 0 1\nvn 0 0 1\nf 1//1 2//2 3//3\n";
        let mat = Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        let list = load_obj(obj, mat, 1.0, Point3::ZERO).unwrap();
        assert_eq!(list.objects.len(), 1);
    }

    #[test]
    fn test_obj_with_comments() {
        let obj = "# comment\nv 0 0 0\nv 1 0 0\nv 0 1 0\n\nf 1 2 3\n";
        let mat = Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        let list = load_obj(obj, mat, 1.0, Point3::ZERO).unwrap();
        assert_eq!(list.objects.len(), 1);
    }
}
