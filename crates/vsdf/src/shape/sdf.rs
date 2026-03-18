//! # sdf — 18 SDF Primitives
//!
//! Mỗi primitive = signed distance function f(p) → distance.
//! f(p) < 0 = bên trong, f(p) = 0 = bề mặt, f(p) > 0 = bên ngoài.
//!
//! SDF byte trong Molecule = primitive index (0x01..0x12).
//! Confidence score: 0.0..1.0 — độ tự tin của fit.

use homemath::{cosf, fabsf, fmaxf, fminf, sinf, sqrtf};

// ─────────────────────────────────────────────────────────────────────────────
// Vec3 — điểm trong không gian 3D
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn len(self) -> f32 {
        sqrtf(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn dot(self, o: Self) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn abs(self) -> Self {
        Self::new(fabsf(self.x), fabsf(self.y), fabsf(self.z))
    }

    pub fn max_scalar(self, s: f32) -> Self {
        Self::new(fmaxf(self.x, s), fmaxf(self.y, s), fmaxf(self.z, s))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }

    pub fn scale(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    pub fn max_comp(self) -> f32 {
        fmaxf(fmaxf(self.x, self.y), self.z)
    }

    pub fn clamp(self, lo: f32, hi: f32) -> Self {
        Self::new(
            fmaxf(lo, fminf(hi, self.x)),
            fmaxf(lo, fminf(hi, self.y)),
            fmaxf(lo, fminf(hi, self.z)),
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SdfKind — 18 primitives
// ─────────────────────────────────────────────────────────────────────────────

/// 18 SDF primitives — map trực tiếp với ShapeBase UCD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SdfKind {
    Sphere = 0x01,
    Box = 0x02,
    Cone = 0x03,
    Torus = 0x04,
    Capsule = 0x05,
    Cylinder = 0x06,
    Ellipsoid = 0x07,
    Pyramid = 0x08,
    Plane = 0x09,
    RoundBox = 0x0A,
    Link = 0x0B,
    HexPrism = 0x0C,
    TriPrism = 0x0D,
    SolidAngle = 0x0E,
    CutSphere = 0x0F,
    CutHollow = 0x10,
    DeathStar = 0x11,
    Octahedron = 0x12,
}

impl SdfKind {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Sphere),
            0x02 => Some(Self::Box),
            0x03 => Some(Self::Cone),
            0x04 => Some(Self::Torus),
            0x05 => Some(Self::Capsule),
            0x06 => Some(Self::Cylinder),
            0x07 => Some(Self::Ellipsoid),
            0x08 => Some(Self::Pyramid),
            0x09 => Some(Self::Plane),
            0x0A => Some(Self::RoundBox),
            0x0B => Some(Self::Link),
            0x0C => Some(Self::HexPrism),
            0x0D => Some(Self::TriPrism),
            0x0E => Some(Self::SolidAngle),
            0x0F => Some(Self::CutSphere),
            0x10 => Some(Self::CutHollow),
            0x11 => Some(Self::DeathStar),
            0x12 => Some(Self::Octahedron),
            _ => None,
        }
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SDF functions — f(p, params) → distance
// ─────────────────────────────────────────────────────────────────────────────

/// f(p) → signed distance
pub fn sdf(kind: SdfKind, p: Vec3, params: &SdfParams) -> f32 {
    match kind {
        SdfKind::Sphere => sphere(p, params.r),
        SdfKind::Box => sdf_box(p, params.b),
        SdfKind::Cone => cone(p, params.h, params.r, params.r2),
        SdfKind::Torus => torus(p, params.r, params.r2),
        SdfKind::Capsule => capsule(
            p,
            Vec3::new(0.0, -params.h, 0.0),
            Vec3::new(0.0, params.h, 0.0),
            params.r,
        ),
        SdfKind::Cylinder => cylinder(p, params.r, params.h),
        SdfKind::Ellipsoid => ellipsoid(p, params.b),
        SdfKind::Pyramid => pyramid(p, params.h),
        SdfKind::Plane => plane(p, Vec3::new(0.0, 1.0, 0.0), 0.0),
        SdfKind::RoundBox => round_box(p, params.b, params.r2),
        SdfKind::Link => link(p, params.h, params.r, params.r2),
        SdfKind::HexPrism => hex_prism(p, Vec3::new(params.r, 0.0, params.h)),
        SdfKind::TriPrism => tri_prism(p, Vec3::new(params.r, 0.0, params.h)),
        SdfKind::SolidAngle => solid_angle(p, params.r, params.h),
        SdfKind::CutSphere => cut_sphere(p, params.r, params.h),
        SdfKind::CutHollow => cut_hollow_sphere(p, params.r, params.h, params.r2),
        SdfKind::DeathStar => death_star(p, params.r, params.r2, params.h),
        SdfKind::Octahedron => octahedron(p, params.r),
    }
}

/// Parameters cho SDF primitives.
#[derive(Debug, Clone, Copy)]
pub struct SdfParams {
    pub r: f32,  // radius 1
    pub r2: f32, // radius 2 / rounding
    pub h: f32,  // height / half-height
    pub b: Vec3, // box half-extents
}

impl SdfParams {
    pub fn sphere(r: f32) -> Self {
        Self {
            r,
            r2: 0.0,
            h: 0.0,
            b: Vec3::new(r, r, r),
        }
    }
    pub fn sdf_box(bx: f32, by: f32, bz: f32) -> Self {
        Self {
            r: 0.0,
            r2: 0.0,
            h: 0.0,
            b: Vec3::new(bx, by, bz),
        }
    }
    pub fn torus(r1: f32, r2: f32) -> Self {
        Self {
            r: r1,
            r2,
            h: 0.0,
            b: Vec3::new(r1, r2, 0.0),
        }
    }
    pub fn capsule(r: f32, h: f32) -> Self {
        Self {
            r,
            r2: 0.0,
            h,
            b: Vec3::new(r, h, r),
        }
    }
    pub fn cone(r: f32, h: f32) -> Self {
        Self {
            r,
            r2: r * 0.5,
            h,
            b: Vec3::new(r, h, r),
        }
    }
}

impl Default for SdfParams {
    fn default() -> Self {
        Self::sphere(1.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 18 SDF implementations
// ─────────────────────────────────────────────────────────────────────────────

fn sphere(p: Vec3, r: f32) -> f32 {
    p.len() - r
}

fn sdf_box(p: Vec3, b: Vec3) -> f32 {
    let q = p.abs().sub(b);
    q.max_scalar(0.0).len() + fminf(q.max_comp(), 0.0)
}

fn round_box(p: Vec3, b: Vec3, r: f32) -> f32 {
    let q = p.abs().sub(b);
    q.max_scalar(0.0).len() + fminf(q.max_comp(), 0.0) - r
}

fn torus(p: Vec3, r1: f32, r2: f32) -> f32 {
    let q = Vec3::new(sqrtf(p.x * p.x + p.z * p.z) - r1, p.y, 0.0);
    q.len() - r2
}

fn capsule(p: Vec3, a: Vec3, b: Vec3, r: f32) -> f32 {
    let pa = p.sub(a);
    let ba = b.sub(a);
    let h = fmaxf(0.0, fminf(1.0, pa.dot(ba) / ba.dot(ba)));
    pa.sub(ba.scale(h)).len() - r
}

fn cone(p: Vec3, h: f32, r1: f32, r2: f32) -> f32 {
    let q = Vec3::new(sqrtf(p.x * p.x + p.z * p.z), p.y, 0.0);
    let k1 = Vec3::new(r2, h, 0.0);
    let k2 = Vec3::new(r2 - r1, 2.0 * h, 0.0);
    let ca_x = q.x - fminf(q.x, if q.y < 0.0 { r1 } else { r2 });
    let ca_y = fabsf(q.y) - h;
    let ca = Vec3::new(ca_x, ca_y, 0.0);
    let t = fmaxf(
        0.0,
        fminf(
            1.0,
            (k1.x * (q.x - r1) + k1.y * q.y) / (k1.x * k1.x + k1.y * k1.y),
        ),
    );
    let cb = Vec3::new(q.x - r1 - k2.x * t, q.y - k2.y * t, 0.0);
    let s = if cb.x < 0.0 && ca_y < 0.0 {
        -1.0f32
    } else {
        1.0
    };
    s * sqrtf(fminf(ca.x * ca.x + ca.y * ca.y, cb.x * cb.x + cb.y * cb.y))
}

fn cylinder(p: Vec3, r: f32, h: f32) -> f32 {
    let d = Vec3::new(sqrtf(p.x * p.x + p.z * p.z) - r, fabsf(p.y) - h, 0.0);
    fminf(fmaxf(d.x, d.y), 0.0) + Vec3::new(fmaxf(d.x, 0.0), fmaxf(d.y, 0.0), 0.0).len()
}

fn ellipsoid(p: Vec3, r: Vec3) -> f32 {
    let k0 = Vec3::new(p.x / r.x, p.y / r.y, p.z / r.z).len();
    let k1 = Vec3::new(p.x / (r.x * r.x), p.y / (r.y * r.y), p.z / (r.z * r.z)).len();
    k0 * (k0 - 1.0) / k1
}

fn pyramid(p: Vec3, h: f32) -> f32 {
    let m2 = h * h + 0.25;
    let px = fabsf(p.x);
    let pz = fabsf(p.z);
    let (px, pz) = if pz > px { (pz, px) } else { (px, pz) };
    let px = px - 0.5;
    let qx = pz + px;
    let qy = p.y - h;
    let qz = pz - px;
    let d = fmaxf(-qy, fmaxf(qx * h - qz * 0.5 * 0.5, 0.0));
    fminf(
        fmaxf(sqrtf(fmaxf(0.0, fmaxf(px * m2, qy * 0.5 + d * d))) - d, 0.0),
        sqrtf(qx * qx + qy * qy),
    )
}

fn plane(p: Vec3, n: Vec3, h: f32) -> f32 {
    p.dot(n) + h
}

fn link(p: Vec3, le: f32, r1: f32, r2: f32) -> f32 {
    let q = Vec3::new(p.x, fmaxf(fabsf(p.y) - le, 0.0), p.z);
    Vec3::new(sqrtf(q.x * q.x + q.y * q.y) - r1, q.z, 0.0).len() - r2
}

fn hex_prism(p: Vec3, h: Vec3) -> f32 {
    let k = Vec3::new(-0.8660254, 0.5, 0.57735);
    let px = fabsf(p.x);
    let py = fabsf(p.y);
    let pz = fabsf(p.z);
    let t = 2.0 * fminf(k.x * px + k.y * py, 0.0);
    let qx = px - t * k.x;
    let qy = py - t * k.y;
    let dx = Vec3::new(qx - fmaxf(-h.x, fminf(h.x, qx)), qy - h.x, 0.0).len()
        * (if qy > h.x { 1.0 } else { -1.0 });
    let dz = pz - h.z;
    fminf(fmaxf(dx, dz), 0.0) + Vec3::new(fmaxf(dx, 0.0), fmaxf(dz, 0.0), 0.0).len()
}

fn tri_prism(p: Vec3, h: Vec3) -> f32 {
    let q = p.abs();
    fmaxf(
        q.z - h.z,
        fmaxf(q.x * 0.866025 + p.y * 0.5, -p.y) - h.x * 0.5,
    )
}

fn solid_angle(p: Vec3, r: f32, angle: f32) -> f32 {
    let c = Vec3::new(sinf(angle), cosf(angle), 0.0);
    let q = Vec3::new(sqrtf(p.x * p.x + p.z * p.z), p.y, 0.0);
    let l = q.len() - r;
    let m = (Vec3::new(q.x - c.x * fmaxf(0.0, fminf(q.len(), r)), q.y - c.y, 0.0)).len();
    fmaxf(l, m * (if c.y * q.x > c.x * q.y { -1.0 } else { 1.0 }))
}

fn cut_sphere(p: Vec3, r: f32, h: f32) -> f32 {
    let w = sqrtf(fmaxf(0.0, r * r - h * h));
    let q = Vec3::new(sqrtf(p.x * p.x + p.z * p.z), p.y, 0.0);
    let s = fmaxf(
        (h - r) * q.x * q.x + w * w * (h + r - 2.0 * q.y),
        h * q.x - w * q.y,
    );
    if s < 0.0 {
        q.len() - r
    } else if q.x < w {
        h - q.y
    } else {
        Vec3::new(q.x - w, q.y - h, 0.0).len()
    }
}

fn cut_hollow_sphere(p: Vec3, r: f32, h: f32, t: f32) -> f32 {
    let w = sqrtf(fmaxf(0.0, r * r - h * h));
    let q = Vec3::new(sqrtf(p.x * p.x + p.z * p.z), p.y, 0.0);
    if h * q.x < w * q.y {
        Vec3::new(q.x - w, q.y - h, 0.0).len() - t
    } else {
        fabsf(q.len() - r) - t
    }
}

fn death_star(p: Vec3, ra: f32, rb: f32, d: f32) -> f32 {
    let a = sphere(p, ra);
    let b = sphere(p.sub(Vec3::new(d, 0.0, 0.0)), -rb);
    fmaxf(a, -b)
}

fn octahedron(p: Vec3, s: f32) -> f32 {
    let p = p.abs();
    let m = p.x + p.y + p.z - s;
    let (qx, qy, qz) = if 3.0 * p.x < m {
        (p.x, p.y, p.z)
    } else if 3.0 * p.y < m {
        (p.y, p.x, p.z)
    } else if 3.0 * p.z < m {
        (p.z, p.x, p.y)
    } else {
        return m * 0.57735027;
    };
    let k = fmaxf(0.0, fminf(qy + qz - s * 0.5, s * 0.5));
    Vec3::new(qx, qy - k, qz - k).len()
}

// ─────────────────────────────────────────────────────────────────────────────
// SDF boolean ops
// ─────────────────────────────────────────────────────────────────────────────

pub fn union(a: f32, b: f32) -> f32 {
    fminf(a, b)
}
pub fn subtract(a: f32, b: f32) -> f32 {
    fmaxf(-b, a)
}
pub fn intersect(a: f32, b: f32) -> f32 {
    fmaxf(a, b)
}
pub fn smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let h = fmaxf(k - fabsf(a - b), 0.0) / k;
    fminf(a, b) - h * h * k * 0.25
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_center_inside() {
        let d = sphere(Vec3::ZERO, 1.0);
        assert!((d - (-1.0)).abs() < 1e-5, "center → d=-r: {}", d);
    }

    #[test]
    fn sphere_surface_zero() {
        let d = sphere(Vec3::new(1.0, 0.0, 0.0), 1.0);
        assert!(d.abs() < 1e-5, "surface → d=0: {}", d);
    }

    #[test]
    fn sphere_outside_positive() {
        let d = sphere(Vec3::new(2.0, 0.0, 0.0), 1.0);
        assert!((d - 1.0).abs() < 1e-5, "outside → d=1: {}", d);
    }

    #[test]
    fn box_inside() {
        let d = sdf_box(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        assert!(d < 0.0, "inside box → d<0: {}", d);
    }

    #[test]
    fn box_outside() {
        let d = sdf_box(Vec3::new(2.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        assert!((d - 1.0).abs() < 1e-5, "outside box → d=1: {}", d);
    }

    #[test]
    fn torus_center_inside() {
        // Torus(r1=2, r2=0.5) — center of tube at (2,0,0)
        let d = torus(Vec3::new(2.0, 0.0, 0.0), 2.0, 0.5);
        assert!(d < 0.0, "inside torus tube → d<0: {}", d);
    }

    #[test]
    fn capsule_center() {
        let a = Vec3::new(0.0, -1.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        let d = capsule(Vec3::ZERO, a, b, 0.5);
        assert!((d - (-0.5)).abs() < 1e-5, "capsule center → d=-r: {}", d);
    }

    #[test]
    fn all_18_primitives_compile() {
        // Verify tất cả 18 primitives return finite values
        let p = Vec3::new(0.5, 0.5, 0.5);
        for b in 0x01u8..=0x12 {
            let kind = SdfKind::from_byte(b).unwrap();
            let params = SdfParams {
                r: 1.0,
                r2: 0.3,
                h: 1.0,
                b: Vec3::new(1.0, 1.0, 1.0),
            };
            let d = sdf(kind, p, &params);
            assert!(
                d.is_finite(),
                "SDF 0x{:02X} phải trả finite value: {}",
                b,
                d
            );
        }
    }

    #[test]
    fn sdf_kind_roundtrip() {
        for b in 0x01u8..=0x12 {
            let k = SdfKind::from_byte(b).unwrap();
            assert_eq!(k.as_byte(), b);
        }
        assert!(SdfKind::from_byte(0x00).is_none());
        assert!(SdfKind::from_byte(0x13).is_none());
    }

    #[test]
    fn boolean_ops() {
        assert_eq!(union(-1.0, 2.0), -1.0);
        assert_eq!(intersect(-1.0, 2.0), 2.0);
        assert_eq!(subtract(-1.0, 2.0), -1.0);
        assert!(smooth_union(-1.0, 2.0, 0.5) < -1.0 + 1e-3);
    }

    #[test]
    fn octahedron_center() {
        let d = octahedron(Vec3::ZERO, 1.0);
        assert!(d < 0.0, "inside octahedron → d<0: {}", d);
    }
}
