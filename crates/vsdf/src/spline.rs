//! # spline — Vector Spline
//!
//! QT6: "HAI CHIỀU TỒN TẠI"
//!   SDF = hữu hình (khoảng cách, hình dạng)
//!   Vector Spline = vô hình (ánh sáng, gió, nhiệt, âm, cảm xúc)
//!
//! VectorSpline = chuỗi Bezier cubic.
//! evaluate(t) → f32 với t ∈ [0, 1].
//!
//! Dùng cho:
//!   intensity_spline(t) — ánh sáng thay đổi theo thời gian
//!   force_spline(t)     — gió
//!   temp_spline(t)      — nhiệt
//!   freq_spline(t)      — âm thanh
//!   emotion_spline(t)   — cảm xúc V/A/D/I theo thời gian

extern crate alloc;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// Bezier cubic — 1 đoạn cong
// ─────────────────────────────────────────────────────────────────────────────

/// Bezier cubic với 4 control points: p0, p1, p2, p3.
///
/// evaluate(t) = (1-t)³p0 + 3(1-t)²t·p1 + 3(1-t)t²·p2 + t³·p3
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezierSegment {
    pub p0: f32,
    pub p1: f32,
    pub p2: f32,
    pub p3: f32,
}

impl BezierSegment {
    /// Tạo đoạn phẳng tại giá trị val.
    pub fn flat(val: f32) -> Self {
        Self { p0: val, p1: val, p2: val, p3: val }
    }

    /// Tạo đoạn linear từ a đến b.
    pub fn linear(a: f32, b: f32) -> Self {
        Self { p0: a, p1: a + (b - a) / 3.0, p2: a + 2.0 * (b - a) / 3.0, p3: b }
    }

    /// Evaluate tại t ∈ [0, 1].
    ///
    /// Dùng De Casteljau — numerically stable.
    #[inline]
    pub fn evaluate(self, t: f32) -> f32 {
        let u  = 1.0 - t;
        let t2 = t * t;
        let u2 = u * u;
        u2 * u * self.p0
            + 3.0 * u2 * t * self.p1
            + 3.0 * u * t2 * self.p2
            + t2 * t * self.p3
    }

    /// Đạo hàm tại t — tốc độ thay đổi.
    #[inline]
    pub fn derivative(self, t: f32) -> f32 {
        let u = 1.0 - t;
        3.0 * (u * u * (self.p1 - self.p0)
             + 2.0 * u * t * (self.p2 - self.p1)
             + t * t * (self.p3 - self.p2))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VectorSpline — chuỗi Bezier cubic
// ─────────────────────────────────────────────────────────────────────────────

/// Chuỗi Bezier cubic liên tiếp.
///
/// Lưu tối đa MAX_SEGMENTS đoạn (không_std — Vec có bounded push).
/// evaluate(t) với t ∈ [0, 1] map qua toàn bộ spline.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct VectorSpline {
    pub segments: Vec<BezierSegment>,
}

impl VectorSpline {
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }

    /// Spline phẳng tại giá trị val.
    pub fn flat(val: f32) -> Self {
        let mut s = Self::new();
        s.segments.push(BezierSegment::flat(val));
        s
    }

    /// Spline linear từ a đến b.
    pub fn linear(a: f32, b: f32) -> Self {
        let mut s = Self::new();
        s.segments.push(BezierSegment::linear(a, b));
        s
    }

    /// Thêm segment vào cuối.
    pub fn push(&mut self, seg: BezierSegment) {
        self.segments.push(seg);
    }

    /// Evaluate tại t ∈ [0, 1] — map qua toàn bộ spline.
    ///
    /// Phân bố đều: mỗi segment chiếm 1/N khoảng t.
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let n = self.segments.len();
        if n == 0 { return 0.0; }
        if n == 1 { return self.segments[0].evaluate(t); }

        // Map t → segment index + local t
        let scaled = t * n as f32;
        let idx    = (scaled as usize).min(n - 1);
        let local  = scaled - idx as f32;
        self.segments[idx].evaluate(local)
    }

    /// Đạo hàm tại t — tốc độ thay đổi.
    pub fn derivative(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let n = self.segments.len();
        if n == 0 { return 0.0; }
        let scaled = t * n as f32;
        let idx    = (scaled as usize).min(n - 1);
        let local  = scaled - idx as f32;
        // Chain rule: d/dt = d/d_local × d_local/dt = deriv × n
        self.segments[idx].derivative(local) * n as f32
    }

    /// Giá trị tối đa trong spline.
    pub fn max_val(&self) -> f32 {
        self.segments.iter()
            .flat_map(|s| [s.p0, s.p1, s.p2, s.p3])
            .fold(f32::NEG_INFINITY, f32::max)
    }

    /// Giá trị tối thiểu.
    pub fn min_val(&self) -> f32 {
        self.segments.iter()
            .flat_map(|s| [s.p0, s.p1, s.p2, s.p3])
            .fold(f32::INFINITY, f32::min)
    }

    /// Số segments.
    pub fn len(&self) -> usize { self.segments.len() }

    /// Rỗng?
    pub fn is_empty(&self) -> bool { self.segments.is_empty() }
}

impl Default for VectorSpline {
    fn default() -> Self { Self::new() }
}

// ─────────────────────────────────────────────────────────────────────────────
// SLI — Spline Layer Index (từ spec MASTER.md)
// ─────────────────────────────────────────────────────────────────────────────

/// SLI: Spline Layer Index cho một Node.
///
/// Từ spec: "SLI chỉ là layer + spline".
/// Size thay đổi theo layer: L0→2B, L1→3B, L2→5B, L3→9B.
///
/// Đây là representation compact — encode/decode cho binary format.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliHeader {
    pub layer:    u8,
    pub n_splines: u8,
}

impl SliHeader {
    /// Size (bytes) theo layer.
    pub fn byte_size(layer: u8) -> usize {
        match layer {
            0 => 2,
            1 => 3,
            2 => 5,
            _ => 9,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── BezierSegment ─────────────────────────────────────────────────────────

    #[test]
    fn bezier_flat_constant() {
        let seg = BezierSegment::flat(0.75);
        assert!((seg.evaluate(0.0) - 0.75).abs() < 1e-5);
        assert!((seg.evaluate(0.5) - 0.75).abs() < 1e-5);
        assert!((seg.evaluate(1.0) - 0.75).abs() < 1e-5);
    }

    #[test]
    fn bezier_linear_endpoints() {
        let seg = BezierSegment::linear(0.0, 1.0);
        assert!((seg.evaluate(0.0) - 0.0).abs() < 1e-5, "t=0 → 0.0");
        assert!((seg.evaluate(1.0) - 1.0).abs() < 1e-5, "t=1 → 1.0");
    }

    #[test]
    fn bezier_linear_midpoint() {
        let seg = BezierSegment::linear(0.0, 1.0);
        let mid = seg.evaluate(0.5);
        assert!((mid - 0.5).abs() < 0.01, "Linear midpoint ≈ 0.5: {}", mid);
    }

    #[test]
    fn bezier_custom() {
        // Cong lên ở giữa
        let seg = BezierSegment { p0: 0.0, p1: 0.8, p2: 0.8, p3: 0.0 };
        let mid = seg.evaluate(0.5);
        assert!(mid > 0.4, "Cong lên giữa: {} > 0.4", mid);
        assert!((seg.evaluate(0.0)).abs() < 1e-5);
        assert!((seg.evaluate(1.0)).abs() < 1e-5);
    }

    #[test]
    fn bezier_derivative_linear() {
        let seg = BezierSegment::linear(0.0, 1.0);
        let d   = seg.derivative(0.5);
        // Derivative của linear ≈ hằng số 1.0
        assert!((d - 1.0).abs() < 0.05, "Linear deriv ≈ 1.0: {}", d);
    }

    // ── VectorSpline ──────────────────────────────────────────────────────────

    #[test]
    fn spline_flat() {
        let s = VectorSpline::flat(0.5);
        assert!((s.evaluate(0.0) - 0.5).abs() < 1e-5);
        assert!((s.evaluate(0.5) - 0.5).abs() < 1e-5);
        assert!((s.evaluate(1.0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn spline_linear() {
        let s = VectorSpline::linear(0.0, 1.0);
        assert!((s.evaluate(0.0) - 0.0).abs() < 1e-4);
        assert!((s.evaluate(1.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn spline_empty_zero() {
        let s = VectorSpline::new();
        assert_eq!(s.evaluate(0.5), 0.0, "Empty spline → 0");
        assert_eq!(s.derivative(0.5), 0.0);
    }

    #[test]
    fn spline_multi_segment() {
        let mut s = VectorSpline::new();
        // Đoạn 1: 0→1, đoạn 2: 1→0 (giống sin một chu kỳ)
        s.push(BezierSegment::linear(0.0, 1.0));
        s.push(BezierSegment::linear(1.0, 0.0));
        assert_eq!(s.len(), 2);

        // t=0 → đầu đoạn 1 = 0
        assert!((s.evaluate(0.0) - 0.0).abs() < 1e-4);
        // t=0.5 → giữa = ~1.0 (đầu đoạn 2)
        let mid = s.evaluate(0.5);
        assert!((mid - 1.0).abs() < 0.1, "Mid ≈ 1.0: {}", mid);
        // t=1.0 → cuối đoạn 2 = 0
        assert!((s.evaluate(1.0) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn spline_t_clamp() {
        let s = VectorSpline::linear(0.0, 1.0);
        // t ngoài [0,1] được clamp
        assert!((s.evaluate(-0.5) - s.evaluate(0.0)).abs() < 1e-5);
        assert!((s.evaluate(1.5) - s.evaluate(1.0)).abs() < 1e-5);
    }

    #[test]
    fn spline_max_min() {
        let mut s = VectorSpline::new();
        s.push(BezierSegment { p0: -0.5, p1: 0.8, p2: 0.3, p3: 0.1 });
        assert!(s.max_val() >= 0.8);
        assert!(s.min_val() <= -0.5);
    }

    #[test]
    fn spline_derivative_direction() {
        // Spline đang tăng → derivative dương
        let s = VectorSpline::linear(0.0, 1.0);
        assert!(s.derivative(0.5) > 0.0, "Tăng → deriv dương");

        // Spline đang giảm → derivative âm
        let s2 = VectorSpline::linear(1.0, 0.0);
        assert!(s2.derivative(0.5) < 0.0, "Giảm → deriv âm");
    }

    #[test]
    fn spline_sunlight_day_cycle() {
        // Mô phỏng ánh sáng một ngày: 0 sáng → peak giữa trưa → 0 tối
        // t=0: 0h, t=1: 24h
        let mut s = VectorSpline::new();
        s.push(BezierSegment { p0: 0.0, p1: 0.0, p2: 1.0, p3: 1.0 }); // sáng sớm
        s.push(BezierSegment { p0: 1.0, p1: 1.0, p2: 0.0, p3: 0.0 }); // chiều tối

        let dawn  = s.evaluate(0.0);
        let noon  = s.evaluate(0.5);
        let dusk  = s.evaluate(1.0);

        assert!(dawn  < 0.1,  "Rạng sáng: {}", dawn);
        assert!(noon  > 0.8,  "Giữa trưa: {}", noon);
        assert!(dusk  < 0.1,  "Chập tối:  {}", dusk);
    }

    #[test]
    fn sli_byte_size() {
        assert_eq!(SliHeader::byte_size(0), 2);
        assert_eq!(SliHeader::byte_size(1), 3);
        assert_eq!(SliHeader::byte_size(2), 5);
        assert_eq!(SliHeader::byte_size(3), 9);
        assert_eq!(SliHeader::byte_size(9), 9); // L3+
    }
}
