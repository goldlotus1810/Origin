//! # curve — ConversationCurve
//!
//! f(x) = α × f_conv(t) + β × f_dn(nodes)
//! α = 0.6  (hội thoại hiện tại)
//! β = 0.4  (ĐN tích lũy từ trước)
//!
//! f_conv(t) = V + 0.5×V'(t) + 0.25×V''(t)
//!   V'(t)  = tốc độ thay đổi
//!   V''(t) = gia tốc thay đổi

extern crate alloc;
use alloc::vec::Vec;

use silk::walk::ResponseTone;

/// α cho f_conv
pub const ALPHA: f32 = 0.6;
/// β cho f_dn
pub const BETA:  f32 = 0.4;

// ─────────────────────────────────────────────────────────────────────────────
// ConversationCurve
// ─────────────────────────────────────────────────────────────────────────────

/// ConversationCurve — theo dõi cảm xúc qua các turns.
#[derive(Debug)]
pub struct ConversationCurve {
    /// Valence qua các turns — append-only
    pub curve:   Vec<f32>,
    /// V'(t) — first derivative
    pub d1:      Vec<f32>,
    /// V''(t) — second derivative
    pub d2:      Vec<f32>,
    /// f_conv hiện tại
    pub fx_conv: f32,
    /// f_dn tích lũy từ ĐN nodes
    pub fx_dn:   f32,
    /// f(x) tổng hợp
    pub fx:      f32,
}

impl ConversationCurve {
    pub fn new() -> Self {
        Self {
            curve:   Vec::new(),
            d1:      Vec::new(),
            d2:      Vec::new(),
            fx_conv: 0.0,
            fx_dn:   0.0,
            fx:      0.0,
        }
    }

    /// Thêm valence mới từ turn hiện tại.
    ///
    /// Tự động tính d1, d2 và cập nhật f(x).
    pub fn push(&mut self, valence: f32) -> f32 {
        self.curve.push(valence);
        let n = self.curve.len();

        // d1 = V(t) - V(t-1)
        if n >= 2 {
            self.d1.push(self.curve[n-1] - self.curve[n-2]);
        }

        // d2 = d1(t) - d1(t-1)
        let d1_len = self.d1.len();
        if d1_len >= 2 {
            self.d2.push(self.d1[d1_len-1] - self.d1[d1_len-2]);
        }

        // f_conv = V + 0.5×d1 + 0.25×d2
        let d1_now = self.d1.last().copied().unwrap_or(0.0);
        let d2_now = self.d2.last().copied().unwrap_or(0.0);
        self.fx_conv = valence + 0.5 * d1_now + 0.25 * d2_now;

        // f(x) = α×f_conv + β×f_dn
        self.fx = ALPHA * self.fx_conv + BETA * self.fx_dn;
        self.fx
    }

    /// Cập nhật f_dn từ ĐN node mới.
    pub fn update_dn(&mut self, dn_affect: f32) {
        // Exponential moving average để f_dn không nhảy đột ngột
        self.fx_dn = self.fx_dn * 0.7 + dn_affect * 0.3;
        self.fx    = ALPHA * self.fx_conv + BETA * self.fx_dn;
    }

    /// ResponseTone từ curve hiện tại.
    pub fn tone(&self) -> ResponseTone {
        silk::walk::response_tone(&self.curve)
    }

    /// Số turns.
    pub fn turn_count(&self) -> usize { self.curve.len() }

    /// Valence hiện tại.
    pub fn current_v(&self) -> f32 {
        self.curve.last().copied().unwrap_or(0.0)
    }

    /// d1 hiện tại.
    pub fn d1_now(&self) -> f32 {
        self.d1.last().copied().unwrap_or(0.0)
    }

    /// d2 hiện tại.
    pub fn d2_now(&self) -> f32 {
        self.d2.last().copied().unwrap_or(0.0)
    }

    /// f(x) hiện tại.
    pub fn fx(&self) -> f32 { self.fx }
}

impl Default for ConversationCurve {
    fn default() -> Self { Self::new() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_empty() {
        let c = ConversationCurve::new();
        assert_eq!(c.turn_count(), 0);
        assert_eq!(c.fx(), 0.0);
    }

    #[test]
    fn curve_single_turn() {
        let mut c = ConversationCurve::new();
        c.push(-0.6);
        assert_eq!(c.turn_count(), 1);
        assert_eq!(c.current_v(), -0.6);
        assert_eq!(c.d1_now(), 0.0, "1 turn → d1=0");
    }

    #[test]
    fn curve_derivative_correct() {
        let mut c = ConversationCurve::new();
        c.push(-0.1);
        c.push(-0.3);
        assert!((c.d1_now() - (-0.2)).abs() < 0.001,
            "d1 = -0.3 - (-0.1) = -0.2");
    }

    #[test]
    fn curve_d2_correct() {
        let mut c = ConversationCurve::new();
        c.push(-0.1);
        c.push(-0.3);
        c.push(-0.6);
        // d1[0] = -0.2, d1[1] = -0.3, d2 = -0.3 - (-0.2) = -0.1
        assert!((c.d2_now() - (-0.1)).abs() < 0.001,
            "d2 ≈ -0.1, got {}", c.d2_now());
    }

    #[test]
    fn curve_fx_formula() {
        let mut c = ConversationCurve::new();
        c.push(-0.1);
        c.push(-0.3);
        // f_conv = V + 0.5×d1 + 0.25×d2
        //       = -0.3 + 0.5×(-0.2) + 0.25×0 = -0.3 - 0.1 = -0.4
        // f(x) = 0.6×(-0.4) + 0.4×0 = -0.24
        assert!((c.fx() - (-0.24)).abs() < 0.01,
            "f(x) ≈ -0.24, got {}", c.fx());
    }

    #[test]
    fn curve_dn_update() {
        let mut c = ConversationCurve::new();
        c.push(0.0);
        c.update_dn(-0.5); // ĐN node buồn
        // f_dn = 0×0.7 + (-0.5)×0.3 = -0.15
        // f(x) = 0.6×0 + 0.4×(-0.15) = -0.06
        assert!(c.fx() < 0.0, "f_dn âm ảnh hưởng f(x)");
    }

    #[test]
    fn curve_tone_falling_supportive() {
        let mut c = ConversationCurve::new();
        c.push(-0.1);
        c.push(-0.3);
        c.push(-0.6);
        assert_eq!(c.tone(), ResponseTone::Supportive,
            "Đang buồn xuống → Supportive");
    }

    #[test]
    fn curve_tone_rising_reinforcing() {
        let mut c = ConversationCurve::new();
        c.push(-0.5);
        c.push(-0.2);
        c.push(0.1);
        assert_eq!(c.tone(), ResponseTone::Reinforcing,
            "Đang hồi phục → Reinforcing");
    }

    #[test]
    fn curve_tone_stable_sad_gentle() {
        let mut c = ConversationCurve::new();
        c.push(-0.4);
        c.push(-0.4);
        c.push(-0.4);
        assert_eq!(c.tone(), ResponseTone::Gentle,
            "Buồn ổn định → Gentle");
    }

    #[test]
    fn curve_accumulates_history() {
        let mut c = ConversationCurve::new();
        let values = [-0.1f32, -0.2, -0.5, -0.3, 0.0, 0.2];
        for &v in &values {
            c.push(v);
        }
        assert_eq!(c.turn_count(), 6);
        assert_eq!(c.d1.len(), 5, "5 d1 values cho 6 turns");
        assert_eq!(c.d2.len(), 4, "4 d2 values cho 5 d1");
    }

    #[test]
    fn curve_fx_influenced_by_dn() {
        let mut c1 = ConversationCurve::new();
        let mut c2 = ConversationCurve::new();

        c1.push(0.0);
        c2.push(0.0);
        c2.update_dn(-0.8); // c2 có ĐN âm

        assert!(c2.fx() < c1.fx(),
            "ĐN âm → f(x) thấp hơn: {} < {}", c2.fx(), c1.fx());
    }
}
