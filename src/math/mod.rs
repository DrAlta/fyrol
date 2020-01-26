// Clippy complains about normal mathematical symbols like A, B, C for quadratic equation.
#![allow(clippy::many_single_char_names)]

pub mod vec2;
pub mod vec3;
pub mod vec4;
pub mod mat4;
pub mod mat3;
pub mod quat;
pub mod ray;
pub mod plane;
pub mod triangulator;
pub mod frustum;
pub mod aabb;

use vec2::*;
use vec3::*;
use std::ops::{Add, Sub, Mul};
use crate::visitor::{Visit, VisitResult, Visitor};

#[derive(Copy, Clone, Debug)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> Rect<T> where T: PartialOrd + Default + Add<Output=T> + Sub<Output=T> + Mul<Output=T> + Copy {
    pub fn new(x: T, y: T, w: T, h: T) -> Rect<T> {
        Rect { x, y, w, h }
    }

    pub fn default() -> Rect<T> {
        Rect {
            x: T::default(),
            y: T::default(),
            w: T::default(),
            h: T::default(),
        }
    }

    pub fn inflate(&self, dw: T, dh: T) -> Rect<T> {
        Rect {
            x: self.x - dw,
            y: self.y - dh,
            w: self.w + dw + dw,
            h: self.h + dh + dh,
        }
    }

    pub fn deflate(&self, dw: T, dh: T) -> Rect<T> {
        Rect {
            x: self.x + dw,
            y: self.y + dh,
            w: self.w - (dw + dw),
            h: self.h - (dh + dh),
        }
    }

    pub fn contains(&self, x: T, y: T) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
}

impl<T> Visit for Rect<T> where T: Default + Visit + 'static {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        self.x.visit("X", visitor)?;
        self.y.visit("Y", visitor)?;
        self.w.visit("W", visitor)?;
        self.h.visit("H", visitor)?;

        visitor.leave_region()
    }
}

#[derive(Copy, Clone)]
pub enum PlaneClass {
    XY,
    YZ,
    XZ,
}

#[allow(clippy::useless_let_if_seq)]
pub fn classify_plane(normal: Vec3) -> PlaneClass {
    let mut longest = 0.0f32;
    let mut class = PlaneClass::XY;

    if normal.x.abs() > longest {
        longest = normal.x.abs();
        class = PlaneClass::YZ;
    }

    if normal.y.abs() > longest {
        longest = normal.y.abs();
        class = PlaneClass::XZ;
    }

    if normal.z.abs() > longest {
        class = PlaneClass::XY;
    }

    class
}

pub fn get_polygon_normal(polygon: &[Vec3]) -> Result<Vec3, &'static str> {
    let mut normal = Vec3::ZERO;

    for (i, current) in polygon.iter().enumerate() {
        let next = polygon[(i + 1) % polygon.len()];
        normal.x += (current.y - next.y) * (current.z + next.z);
        normal.y += (current.z - next.z) * (current.x + next.x);
        normal.z += (current.x - next.x) * (current.y + next.y);
    }

    if normal.sqr_len() > std::f32::EPSILON {
        return Ok(normal.normalized_unchecked());
    }

    Err("Unable to get normal of degenerated polygon!")
}

pub fn get_signed_triangle_area(v1: Vec2, v2: Vec2, v3: Vec2) -> f32 {
    0.5 * (v1.x * (v3.y - v2.y) + v2.x * (v1.y - v3.y) + v3.x * (v2.y - v1.y))
}

pub fn vec3_to_vec2_by_plane(plane_class: PlaneClass, normal: Vec3, point: Vec3) -> Vec2 {
    match plane_class {
        PlaneClass::XY => {
            if normal.z < 0.0 {
                Vec2::new(point.y, point.x)
            } else {
                Vec2::new(point.x, point.y)
            }
        }
        PlaneClass::XZ => {
            if normal.y < 0.0 {
                Vec2::new(point.x, point.z)
            } else {
                Vec2::new(point.z, point.x)
            }
        }
        PlaneClass::YZ => {
            if normal.x < 0.0 {
                Vec2::new(point.z, point.y)
            } else {
                Vec2::new(point.y, point.z)
            }
        }
    }
}

pub fn is_point_inside_2d_triangle(point: Vec2, pt_a: Vec2, pt_b: Vec2, pt_c: Vec2) -> bool {
    let ba = pt_b - pt_a;
    let ca = pt_c - pt_a;

    let vp = point - pt_a;

    let ba_dot_ba = ba.dot(ba);
    let ca_dot_ba = ca.dot(ba);
    let ca_dot_ca = ca.dot(ca);

    let dot_02 = ca.dot(vp);
    let dot_12 = ba.dot(vp);

    let inv_denom = 1.0 / (ca_dot_ca * ba_dot_ba - ca_dot_ba * ca_dot_ba);

    // calculate barycentric coordinates
    let u = (ba_dot_ba * dot_02 - ca_dot_ba * dot_12) * inv_denom;
    let v = (ca_dot_ca * dot_12 - ca_dot_ba * dot_02) * inv_denom;

    (u >= 0.0) && (v >= 0.0) && (u + v < 1.0)
}


pub fn clampf(v: f32, min: f32, max: f32) -> f32 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

pub fn wrapf(mut n: f32, mut min_limit: f32, mut max_limit: f32) -> f32 {
    if n >= min_limit && n <= max_limit {
        return n;
    }

    if max_limit == 0.0 && min_limit == 0.0 {
        return 0.0;
    }

    max_limit -= min_limit;

    let offset = min_limit;
    min_limit = 0.0;
    n -= offset;

    let num_of_max = (n / max_limit).abs().floor();

    if n >= max_limit {
        n -= num_of_max * max_limit;
    } else if n < min_limit {
        n += (num_of_max + 1.0) * max_limit;
    }

    n + offset
}

pub fn lerpf(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn get_farthest_point(points: &[Vec3], dir: Vec3) -> Vec3
{
    let mut n_farthest = 0;
    let mut max_dot = -std::f32::MAX;
    for (i, point) in points.iter().enumerate() {
        let dot = dir.dot(point);
        if dot > max_dot {
            n_farthest = i;
            max_dot = dot
        }
    }
    points[n_farthest]
}

pub fn get_barycentric_coords(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> (f32, f32, f32)
{
    let v0 = *b - *a;
    let v1 = *c - *a;
    let v2 = *p - *a;

    let d00 = v0.dot(&v0);
    let d01 = v0.dot(&v1);
    let d11 = v1.dot(&v1);
    let d20 = v2.dot(&v0);
    let d21 = v2.dot(&v1);
    let denom = d00 * d11 - d01 * d01;

    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    (u, v, w)
}

pub fn get_barycentric_coords_2d(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> (f32, f32, f32)
{
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;

    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let denom = d00 * d11 - d01 * d01;

    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    (u, v, w)
}

pub fn is_point_inside_triangle(p: &Vec3, vertices: &[Vec3; 3]) -> bool {
    let ba = vertices[1] - vertices[0];
    let ca = vertices[2] - vertices[0];
    let vp = *p - vertices[0];

    let ba_dot_ba = ba.dot(&ba);
    let ca_dot_ba = ca.dot(&ba);
    let ca_dot_ca = ca.dot(&ca);

    let dot02 = ca.dot(&vp);
    let dot12 = ba.dot(&vp);

    let inv_denom = 1.0 / (ca_dot_ca * ba_dot_ba - ca_dot_ba * ca_dot_ba);

    // Calculate barycentric coordinates
    let u = (ba_dot_ba * dot02 - ca_dot_ba * dot12) * inv_denom;
    let v = (ca_dot_ca * dot12 - ca_dot_ba * dot02) * inv_denom;

    (u >= 0.0) && (v >= 0.0) && (u + v < 1.0)
}

pub fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<[f32; 2]> {
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        // No real roots
        None
    } else {
        // Dont care if quadratic equation has only one root (discriminant == 0), this is edge-case
        // which requires additional branching instructions which is not good for branch-predictor in CPU.
        let _2a = 2.0 * a;
        let discr_root = discriminant.sqrt();
        let r1 = (-b + discr_root) / _2a;
        let r2 = (-b - discr_root) / _2a;
        Some([r1, r2])
    }
}

pub fn spherical_to_cartesian(azimuth: f32, elevation: f32, radius: f32) -> Vec3 {
    let x = radius * elevation.sin() * azimuth.sin();
    let y = radius * elevation.cos();
    let z = -radius * elevation.sin() * azimuth.cos();
    Vec3::new(x, y, z)
}

#[derive(Clone, PartialEq, Eq)]
#[repr(C)]
pub struct TriangleDefinition {
    pub indices: [u32; 3]
}

pub trait PositionProvider: Sized {
    fn position(&self) -> Vec3;
}

impl PositionProvider for Vec3 {
    fn position(&self) -> Vec3 {
        *self
    }
}

/// Tries to find a point closest to given point.
///
/// # Notes
///
/// O(n) complexity.
pub fn get_closest_point<P: PositionProvider>(points: &[P], point: Vec3) -> Option<usize> {
    let mut closest_sqr_distance = std::f32::MAX;
    let mut closest_index = None;
    for (i, vertex) in points.iter().enumerate() {
        let sqr_distance = (vertex.position() - point).sqr_len();
        if sqr_distance < closest_sqr_distance {
            closest_sqr_distance = sqr_distance;
            closest_index = Some(i);
        }
    }
    closest_index
}

pub fn get_closest_point_triangles<P: PositionProvider>(points: &[P], triangles: &[TriangleDefinition], triangle_indices: &[u32], point: Vec3) -> Option<usize> {
    let mut closest_sqr_distance = std::f32::MAX;
    let mut closest_index = None;
    for triangle_index in triangle_indices {
        let triangle = triangles.get(*triangle_index as usize).unwrap();
        for point_index in triangle.indices.iter() {
            let vertex = points.get(*point_index as usize).unwrap();
            let sqr_distance = (vertex.position() - point).sqr_len();
            if sqr_distance < closest_sqr_distance {
                closest_sqr_distance = sqr_distance;
                closest_index = Some(*point_index as usize);
            }
        }
    }
    closest_index
}