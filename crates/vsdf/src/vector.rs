//! # vector — VectorField
//!
//! QT6: VÔ HÌNH = Vector Spline
//!
//! Mỗi loại trường vật lý/cảm xúc là:
//!   Vec3 (direction) + VectorSpline (intensity over time)
//!
//! ```text
//! Ánh sáng  = Vec3(direction) + intensity_spline(t)
//! Gió       = Vec3(direction) + force_spline(t)
//! Nhiệt     = Vec3(source)    + temp_spline(t)
//! Âm thanh  = Vec3(position)  + freq_spline(t)
//! Cảm xúc   = Vec4(V,A,D,I)  + 4 × spline(t)
//! Trọng lực = Vec3(0,-1,0)    + g_spline
//! ```

extern crate alloc;
use crate::sdf::Vec3;
use crate::spline::VectorSpline;

// ─────────────────────────────────────────────────────────────────────────────
// FieldKind — loại trường
// ─────────────────────────────────────────────────────────────────────────────

/// Loại trường vật lý/cảm xúc.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FieldKind {
    Light = 0x01,   // ánh sáng
    Wind = 0x02,    // gió
    Heat = 0x03,    // nhiệt
    Audio = 0x04,   // âm thanh
    Emotion = 0x05, // cảm xúc
    Gravity = 0x06, // trọng lực
    Custom = 0xFF,
}

impl FieldKind {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Light),
            0x02 => Some(Self::Wind),
            0x03 => Some(Self::Heat),
            0x04 => Some(Self::Audio),
            0x05 => Some(Self::Emotion),
            0x06 => Some(Self::Gravity),
            0xFF => Some(Self::Custom),
            _ => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VectorField — Vec3 + intensity spline
// ─────────────────────────────────────────────────────────────────────────────

/// Trường vật lý: hướng Vec3 + cường độ thay đổi theo thời gian.
///
/// evaluate(t) → Vec3 với magnitude = spline.evaluate(t)
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct VectorField {
    pub kind: FieldKind,
    /// Hướng (normalized)
    pub direction: Vec3,
    /// Cường độ theo thời gian
    pub intensity: VectorSpline,
    /// Ambient minimum (không bao giờ về 0 hoàn toàn)
    pub ambient: f32,
}

impl VectorField {
    /// Tạo ánh sáng mặt trời với chu kỳ ngày.
    ///
    /// t=0 → bình minh, t=0.5 → giữa trưa, t=1 → hoàng hôn.
    pub fn sunlight() -> Self {
        use crate::spline::BezierSegment;
        let mut intensity = VectorSpline::new();
        // Bình minh lên
        intensity.push(BezierSegment {
            p0: 0.05,
            p1: 0.05,
            p2: 0.95,
            p3: 0.95,
        });
        // Giữa trưa xuống
        intensity.push(BezierSegment {
            p0: 0.95,
            p1: 0.95,
            p2: 0.05,
            p3: 0.05,
        });

        Self {
            kind: FieldKind::Light,
            direction: Vec3::new(0.6, 0.8, 0.2), // từ góc trên
            intensity,
            ambient: 0.05,
        }
    }

    /// Gió mặc định.
    pub fn wind(dir: Vec3, strength: f32) -> Self {
        Self {
            kind: FieldKind::Wind,
            direction: dir,
            intensity: VectorSpline::flat(strength),
            ambient: 0.0,
        }
    }

    /// Nhiệt từ nguồn.
    pub fn heat(source: Vec3, temp: f32) -> Self {
        Self {
            kind: FieldKind::Heat,
            direction: source,
            intensity: VectorSpline::flat(temp),
            ambient: 0.15,
        }
    }

    /// Trọng lực chuẩn (0, -1, 0) × g.
    pub fn gravity(g: f32) -> Self {
        Self {
            kind: FieldKind::Gravity,
            direction: Vec3::new(0.0, -1.0, 0.0),
            intensity: VectorSpline::flat(g),
            ambient: 0.0,
        }
    }

    /// Evaluate tại t → Vec3 với magnitude = intensity(t) + ambient.
    pub fn evaluate(&self, t: f32) -> Vec3 {
        let mag = self.intensity.evaluate(t) + self.ambient;
        self.direction.scale(mag)
    }

    /// Cường độ scalar tại t.
    pub fn intensity_at(&self, t: f32) -> f32 {
        self.intensity.evaluate(t) + self.ambient
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EmotionField — Vec4(V,A,D,I) × spline
// ─────────────────────────────────────────────────────────────────────────────

/// Cảm xúc theo thời gian — 4 splines cho V/A/D/I.
///
/// Khác VectorField: không có hướng Vec3 mà có 4 chiều cảm xúc,
/// mỗi chiều có spline riêng.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct EmotionField {
    pub valence: VectorSpline,   // V: tích cực/tiêu cực
    pub arousal: VectorSpline,   // A: kích động
    pub dominance: VectorSpline, // D: kiểm soát
    pub intensity: VectorSpline, // I: mạnh/yếu
}

/// Snapshot EmotionField tại t.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct EmotionSample {
    pub t: f32,
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub intensity: f32,
}

impl EmotionField {
    /// Field cảm xúc trung tính.
    pub fn neutral() -> Self {
        Self {
            valence: VectorSpline::flat(0.0),
            arousal: VectorSpline::flat(0.5),
            dominance: VectorSpline::flat(0.5),
            intensity: VectorSpline::flat(0.1),
        }
    }

    /// Tạo từ 4 giá trị ban đầu (spline phẳng).
    pub fn constant(v: f32, a: f32, d: f32, i: f32) -> Self {
        Self {
            valence: VectorSpline::flat(v),
            arousal: VectorSpline::flat(a),
            dominance: VectorSpline::flat(d),
            intensity: VectorSpline::flat(i),
        }
    }

    /// Tạo arc cảm xúc: bắt đầu → đỉnh → kết thúc.
    ///
    /// Dùng cho một cuộc hội thoại: đau buồn → hồi phục → bình yên.
    pub fn arc(start_v: f32, peak_v: f32, end_v: f32, duration_t: f32) -> Self {
        use crate::spline::BezierSegment;

        let mut v_spline = VectorSpline::new();
        v_spline.push(BezierSegment {
            p0: start_v,
            p1: start_v + (peak_v - start_v) * 0.5,
            p2: peak_v,
            p3: peak_v,
        });
        v_spline.push(BezierSegment {
            p0: peak_v,
            p1: peak_v,
            p2: end_v + (peak_v - end_v) * 0.5,
            p3: end_v,
        });

        // Arousal: cao lúc peak, thấp lúc ổn định
        let a_peak = (peak_v.abs() * 0.8 + 0.2).min(1.0);
        let a_end = 0.3_f32;
        let mut a_spline = VectorSpline::new();
        a_spline.push(BezierSegment::linear(0.5, a_peak));
        a_spline.push(BezierSegment::linear(a_peak, a_end));

        let _ = duration_t; // normalized, caller scale nếu cần

        Self {
            valence: v_spline,
            arousal: a_spline,
            dominance: VectorSpline::linear(0.3, 0.6),
            intensity: VectorSpline::linear((start_v.abs() * 0.8).max(0.1), 0.15),
        }
    }

    /// Sample tại t ∈ [0, 1].
    pub fn sample(&self, t: f32) -> EmotionSample {
        EmotionSample {
            t,
            valence: self.valence.evaluate(t),
            arousal: self.arousal.evaluate(t),
            dominance: self.dominance.evaluate(t),
            intensity: self.intensity.evaluate(t),
        }
    }

    /// Tốc độ thay đổi valence tại t (đạo hàm).
    pub fn valence_rate(&self, t: f32) -> f32 {
        self.valence.derivative(t)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── VectorField ───────────────────────────────────────────────────────────

    #[test]
    fn sunlight_day_cycle() {
        let sun = VectorField::sunlight();

        let dawn = sun.intensity_at(0.0);
        let noon = sun.intensity_at(0.5);
        let dusk = sun.intensity_at(1.0);

        assert!(dawn < 0.3, "Bình minh mờ: {}", dawn);
        assert!(noon > 0.8, "Giữa trưa sáng: {}", noon);
        assert!(dusk < 0.3, "Hoàng hôn mờ: {}", dusk);
    }

    #[test]
    fn sunlight_ambient_floor() {
        let sun = VectorField::sunlight();
        // Ambient đảm bảo không bao giờ hoàn toàn tối
        for t in [0.0f32, 0.05, 0.95, 1.0] {
            let i = sun.intensity_at(t);
            assert!(
                i >= sun.ambient,
                "Ambient floor tại t={}: {} >= {}",
                t,
                i,
                sun.ambient
            );
        }
    }

    #[test]
    fn gravity_direction_down() {
        let g = VectorField::gravity(9.8);
        let vec = g.evaluate(0.5);
        assert!(vec.y < 0.0, "Trọng lực hướng xuống: {}", vec.y);
        assert!((vec.x).abs() < 1e-5, "Không có thành phần ngang: {}", vec.x);
    }

    #[test]
    fn wind_constant_direction() {
        let dir = Vec3::new(1.0, 0.0, 0.0); // gió từ đông
        let wind = VectorField::wind(dir, 0.5);
        let v = wind.evaluate(0.3);
        assert!(v.x > 0.0, "Gió theo chiều X+: {}", v.x);
        assert!((v.y).abs() < 1e-5);
    }

    #[test]
    fn heat_positive() {
        let h = VectorField::heat(Vec3::new(0.0, 1.0, 0.0), 0.8);
        assert!(h.intensity_at(0.5) > 0.0, "Nhiệt > 0");
    }

    // ── EmotionField ──────────────────────────────────────────────────────────

    #[test]
    fn emotion_neutral() {
        let e = EmotionField::neutral();
        let s = e.sample(0.5);
        assert!((s.valence).abs() < 0.01, "Neutral V=0: {}", s.valence);
        assert!(
            (s.arousal - 0.5).abs() < 0.01,
            "Neutral A=0.5: {}",
            s.arousal
        );
    }

    #[test]
    fn emotion_constant() {
        let e = EmotionField::constant(-0.6, 0.4, 0.3, 0.55);
        let s = e.sample(0.5);
        assert!((s.valence - (-0.6)).abs() < 0.01);
        assert!((s.arousal - 0.4).abs() < 0.01);
        assert!((s.dominance - 0.3).abs() < 0.01);
        assert!((s.intensity - 0.55).abs() < 0.01);
    }

    #[test]
    fn emotion_arc_shape() {
        // Buồn → đau → hồi phục
        let arc = EmotionField::arc(-0.3, -0.7, 0.1, 1.0);

        let start = arc.sample(0.0);
        let peak = arc.sample(0.5);
        let end = arc.sample(1.0);

        // Đau nhất ở peak
        assert!(
            peak.valence < start.valence,
            "Peak buồn hơn đầu: {} < {}",
            peak.valence,
            start.valence
        );
        // Cuối hồi phục
        assert!(
            end.valence > peak.valence,
            "Cuối tốt hơn peak: {} > {}",
            end.valence,
            peak.valence
        );
    }

    #[test]
    fn emotion_arc_arousal_follows_intensity() {
        let arc = EmotionField::arc(-0.2, -0.8, 0.2, 1.0);
        let peak = arc.sample(0.5);
        // Lúc peak buồn nhất → arousal cao
        assert!(peak.arousal > 0.4, "Arousal cao lúc peak: {}", peak.arousal);
    }

    #[test]
    fn emotion_valence_rate_direction() {
        // Arc từ buồn → ít buồn hơn → dương rate ở cuối
        let arc = EmotionField::arc(-0.7, -0.7, 0.2, 1.0);
        let rate_late = arc.valence_rate(0.8);
        assert!(rate_late > 0.0, "Đang hồi phục → rate dương: {}", rate_late);
    }

    #[test]
    fn emotion_sample_t_field() {
        let e = EmotionField::neutral();
        let s = e.sample(0.3);
        assert!((s.t - 0.3).abs() < 1e-5, "t field giữ đúng: {}", s.t);
    }
}
