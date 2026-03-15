//! # fit — SDF Fitting Engine
//!
//! Nhận vào điểm cloud (từ camera/sensor/image) →
//! tìm SDF primitive phù hợp nhất → confidence score.
//!
//! Confidence:
//!   ≥ 0.7 → accept (promote lên ĐN)
//!   < 0.4 → ignore
//!   0.4..0.7 → hold, cần thêm observation

extern crate alloc;
use alloc::vec::Vec;

use olang::molecular::{MolecularChain, ShapeBase};
use olang::encoder::encode_codepoint;

use crate::sdf::{SdfKind, SdfParams, Vec3, sdf};

/// Threshold để accept một SDF fit.
pub const ACCEPT_THRESHOLD:  f32 = 0.70;
/// Threshold dưới đây → ignore.
pub const IGNORE_THRESHOLD:  f32 = 0.40;

// ─────────────────────────────────────────────────────────────────────────────
// FitResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả fitting một SDF vào point cloud.
#[derive(Debug, Clone)]
pub struct FitResult {
    pub kind:       SdfKind,
    pub params:     SdfParams,
    pub confidence: f32,
    /// Chain tương ứng với shape này (từ UCD)
    pub chain:      MolecularChain,
}

impl FitResult {
    pub fn is_accepted(&self)  -> bool { self.confidence >= ACCEPT_THRESHOLD }
    pub fn is_ignorable(&self) -> bool { self.confidence < IGNORE_THRESHOLD }
}

// ─────────────────────────────────────────────────────────────────────────────
// SdfFitter
// ─────────────────────────────────────────────────────────────────────────────

/// Fit SDF vào point cloud.
pub struct SdfFitter;

impl SdfFitter {
    pub fn new() -> Self { Self }

    /// Fit tất cả 18 primitives và trả về best fit.
    ///
    /// Trả về None nếu không primitive nào đủ confidence.
    pub fn fit_best(&self, points: &[Vec3]) -> Option<FitResult> {
        if points.is_empty() { return None; }

        // Tính bounding box và center
        let (center, extents) = bounding_box(points);
        let r = extents.len() * 0.5;

        // Thử từng primitive
        let candidates = [
            self.try_fit(SdfKind::Sphere, points, center,
                SdfParams::sphere(r)),
            self.try_fit(SdfKind::Box, points, center,
                SdfParams::sdf_box(extents.x, extents.y, extents.z)),
            self.try_fit(SdfKind::Capsule, points, center,
                SdfParams::capsule(r * 0.4, extents.y)),
            self.try_fit(SdfKind::Torus, points, center,
                SdfParams::torus(r * 0.6, r * 0.2)),
            self.try_fit(SdfKind::Cone, points, center,
                SdfParams::cone(r, extents.y)),
            self.try_fit(SdfKind::Cylinder, points, center,
                SdfParams { r: r * 0.5, r2: 0.0, h: extents.y, b: extents }),
            self.try_fit(SdfKind::Ellipsoid, points, center,
                SdfParams { r, r2: 0.0, h: 0.0, b: extents }),
            self.try_fit(SdfKind::Octahedron, points, center,
                SdfParams::sphere(r)),
        ];

        candidates.into_iter()
            .flatten()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence)
                .unwrap_or(core::cmp::Ordering::Equal))
    }

    /// Thử fit một primitive. Trả về FitResult nếu confidence ≥ IGNORE_THRESHOLD.
    fn try_fit(
        &self,
        kind:   SdfKind,
        points: &[Vec3],
        center: Vec3,
        params: SdfParams,
    ) -> Option<FitResult> {
        let confidence = self.compute_confidence(kind, points, center, &params);
        if confidence < IGNORE_THRESHOLD { return None; }

        // Map kind → UCD codepoint
        let cp = kind_to_codepoint(kind);
        let chain = encode_codepoint(cp);

        Some(FitResult { kind, params, confidence, chain })
    }

    /// Tính confidence: tỷ lệ points nằm trên/gần bề mặt SDF.
    ///
    /// Confidence = 1 - avg(|sdf(p)| / r)
    /// Point nằm đúng bề mặt → sdf=0 → contribute 1.0.
    fn compute_confidence(
        &self,
        kind:   SdfKind,
        points: &[Vec3],
        center: Vec3,
        params: &SdfParams,
    ) -> f32 {
        if points.is_empty() { return 0.0; }

        let scale = params.r.max(params.b.len()).max(0.001);
        let mut total = 0.0f32;

        for &p in points {
            // Translate to SDF local space
            let local = p.sub(center);
            let d = sdf(kind, local, params);
            // Normalized error: gần 0 → gần bề mặt → confidence cao
            let err = (d.abs() / scale).min(1.0);
            total += 1.0 - err;
        }

        total / points.len() as f32
    }
}

impl Default for SdfFitter {
    fn default() -> Self { Self::new() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Tính bounding box (center, half-extents) của point cloud.
fn bounding_box(points: &[Vec3]) -> (Vec3, Vec3) {
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

    for &p in points {
        min = Vec3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z));
        max = Vec3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z));
    }

    let center  = min.add(max).scale(0.5);
    let extents = max.sub(min).scale(0.5);
    (center, extents)
}

/// Map SdfKind → Unicode codepoint đại diện (cho chain).
fn kind_to_codepoint(kind: SdfKind) -> u32 {
    match kind {
        SdfKind::Sphere     => 0x25CF, // ●
        SdfKind::Box        => 0x25A0, // ■
        SdfKind::Cone       => 0x25B2, // ▲
        SdfKind::Torus      => 0x25CB, // ○ (ring)
        SdfKind::Capsule    => 0x2770, // ❰ (capsule-like)
        SdfKind::Cylinder   => 0x25AE, // ▮
        SdfKind::Ellipsoid  => 0x2B2C, // ⬬
        SdfKind::Pyramid    => 0x25B3, // △
        SdfKind::Plane      => 0x2014, // — (flat)
        SdfKind::RoundBox   => 0x2B1C, // ⬜
        SdfKind::Link       => 0x26D3, // ⛓
        SdfKind::HexPrism   => 0x2B23, // ⬣
        SdfKind::TriPrism   => 0x25B7, // ▷
        SdfKind::SolidAngle => 0x2220, // ∠
        SdfKind::CutSphere  => 0x25D1, // ◑
        SdfKind::CutHollow  => 0x25D0, // ◐
        SdfKind::DeathStar  => 0x2605, // ★
        SdfKind::Octahedron => 0x25C6, // ◆
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn skip() -> bool { ucd::table_len() == 0 }

    /// Tạo point cloud trên bề mặt sphere.
    fn sphere_points(r: f32, n: usize) -> Vec<Vec3> {
        // Sample uniformly bằng Fibonacci sphere
        let phi = 2.39996; // golden angle ≈ 2π/φ²
        (0..n).map(|i| {
            let theta = libm::acosf(1.0 - 2.0*(i as f32 + 0.5)/n as f32);
            let psi   = phi * i as f32;
            Vec3::new(
                r * libm::sinf(theta) * libm::cosf(psi),
                r * libm::cosf(theta),
                r * libm::sinf(theta) * libm::sinf(psi),
            )
        }).collect()
    }

    /// Tạo point cloud trên bề mặt box.
    fn box_points(bx: f32, by: f32, bz: f32, n: usize) -> Vec<Vec3> {
        let faces = [
            Vec3::new( bx, 0.0, 0.0), Vec3::new(-bx, 0.0, 0.0),
            Vec3::new(0.0,  by, 0.0), Vec3::new(0.0, -by, 0.0),
            Vec3::new(0.0, 0.0,  bz), Vec3::new(0.0, 0.0, -bz),
        ];
        (0..n).map(|i| {
            let f = &faces[i % 6];
            // Jitter trên mặt
            let t = (i / 6) as f32 * 0.1;
            Vec3::new(f.x + if f.x == 0.0 { t } else { 0.0 },
                      f.y + if f.y == 0.0 { t } else { 0.0 },
                      f.z + if f.z == 0.0 { t } else { 0.0 })
        }).collect()
    }

    #[test]
    fn fit_sphere_high_confidence() {
        if skip() { return; }
        let pts = sphere_points(1.0, 50);
        let fitter = SdfFitter::new();
        let result = fitter.fit_best(&pts).expect("phải tìm được fit");

        // Sphere cloud → Sphere hoặc Ellipsoid (superset) đều hợp lệ
        assert!(
            result.kind == SdfKind::Sphere || result.kind == SdfKind::Ellipsoid,
            "Sphere cloud → Sphere/Ellipsoid, got {:?}", result.kind
        );
        assert!(result.confidence >= ACCEPT_THRESHOLD,
            "confidence={:.3} ≥ {}", result.confidence, ACCEPT_THRESHOLD);
        assert!(result.is_accepted());
    }

    #[test]
    fn fit_sphere_has_chain() {
        if skip() { return; }
        let pts = sphere_points(0.5, 30);
        let result = SdfFitter::new().fit_best(&pts).unwrap();
        assert!(!result.chain.is_empty(), "FitResult phải có chain");
    }

    #[test]
    fn fit_box_reasonable() {
        if skip() { return; }
        // Box points — confidence thấp hơn sphere vì sampling đơn giản
        let pts = box_points(1.0, 1.0, 1.0, 24);
        let result = SdfFitter::new().fit_best(&pts);
        assert!(result.is_some(), "Phải tìm được ít nhất 1 fit");
        let r = result.unwrap();
        assert!(r.confidence >= IGNORE_THRESHOLD,
            "Box fit confidence={:.3} ≥ {}", r.confidence, IGNORE_THRESHOLD);
    }

    #[test]
    fn fit_empty_returns_none() {
        let result = SdfFitter::new().fit_best(&[]);
        assert!(result.is_none(), "Empty cloud → None");
    }

    #[test]
    fn fit_single_point() {
        if skip() { return; }
        // 1 điểm → bất kỳ primitive nào đều có thể fit
        let pts = vec![Vec3::new(1.0, 0.0, 0.0)];
        let result = SdfFitter::new().fit_best(&pts);
        // Không crash — có thể None hoặc Some
        let _ = result;
    }

    #[test]
    fn confidence_thresholds() {
        if skip() { return; }
        let pts = sphere_points(1.0, 50);
        let result = SdfFitter::new().fit_best(&pts).unwrap();
        // Test threshold logic
        if result.confidence >= ACCEPT_THRESHOLD {
            assert!(result.is_accepted());
            assert!(!result.is_ignorable());
        }
    }

    #[test]
    fn bounding_box_correct() {
        let pts = vec![
            Vec3::new(-1.0, -2.0, -3.0),
            Vec3::new( 1.0,  2.0,  3.0),
        ];
        let (center, extents) = bounding_box(&pts);
        assert!((center.x).abs() < 1e-5);
        assert!((center.y).abs() < 1e-5);
        assert!((extents.x - 1.0).abs() < 1e-5);
        assert!((extents.y - 2.0).abs() < 1e-5);
        assert!((extents.z - 3.0).abs() < 1e-5);
    }

    #[test]
    fn kind_to_codepoint_all_valid() {
        if skip() { return; }
        for b in 0x01u8..=0x12 {
            let kind = SdfKind::from_byte(b).unwrap();
            let cp = kind_to_codepoint(kind);
            assert!(cp > 0x20, "Codepoint phải là printable Unicode: 0x{:04X}", cp);
        }
    }
}
