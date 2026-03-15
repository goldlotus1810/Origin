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

/// Window size cho variance detection.
const VARIANCE_WINDOW: usize = 5;

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
    /// Window variance — emotional instability indicator
    pub window_variance: f32,
    /// Emotional instability detected
    pub unstable: bool,
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
            window_variance: 0.0,
            unstable: false,
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

        // Window variance — đo emotional instability
        self.update_window_variance();

        self.fx
    }

    /// Cập nhật f_dn từ ĐN node mới.
    pub fn update_dn(&mut self, dn_affect: f32) {
        // Exponential moving average để f_dn không nhảy đột ngột
        self.fx_dn = self.fx_dn * 0.7 + dn_affect * 0.3;
        self.fx    = ALPHA * self.fx_conv + BETA * self.fx_dn;
    }

    /// ResponseTone từ curve hiện tại.
    ///
    /// Nếu emotional instability detected (window variance cao + d1 đổi chiều):
    /// → Gentle thay vì Celebratory (bảo vệ người dùng đang bất ổn)
    pub fn tone(&self) -> ResponseTone {
        let base = silk::walk::response_tone(&self.curve);
        // Instability override: đang bất ổn → Gentle thay vì Celebratory
        if self.unstable && base == ResponseTone::Celebratory {
            return ResponseTone::Gentle;
        }
        base
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

    /// Window variance hiện tại.
    pub fn window_variance(&self) -> f32 { self.window_variance }

    /// Emotional instability?
    pub fn is_unstable(&self) -> bool { self.unstable }

    // ─────────────────────────────────────────────────────────────────────────
    // Window Variance — emotional instability detection
    // ─────────────────────────────────────────────────────────────────────────

    /// Tính variance trong window gần nhất.
    ///
    /// Instability = variance cao (> 0.04) + d1 đổi chiều (sign change).
    /// Khi instability → Gentle thay vì Celebratory.
    fn update_window_variance(&mut self) {
        let n = self.curve.len();
        if n < VARIANCE_WINDOW {
            self.window_variance = 0.0;
            self.unstable = false;
            return;
        }

        let window = &self.curve[n - VARIANCE_WINDOW..];

        // Mean
        let mean: f32 = window.iter().sum::<f32>() / VARIANCE_WINDOW as f32;

        // Variance = Σ(v - mean)² / N
        let var: f32 = window.iter()
            .map(|v| (v - mean) * (v - mean))
            .sum::<f32>() / VARIANCE_WINDOW as f32;

        self.window_variance = var;

        // D1 sign change trong window
        let d1_window = &self.d1[if self.d1.len() >= VARIANCE_WINDOW - 1 {
            self.d1.len() - (VARIANCE_WINDOW - 1)
        } else {
            0
        }..];

        let sign_changes = d1_window.windows(2)
            .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
            .count();

        // Instability = variance cao + nhiều lần đổi chiều
        self.unstable = var > 0.04 && sign_changes >= 2;
    }
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
    fn curve_window_variance_stable() {
        let mut c = ConversationCurve::new();
        // 5 stable turns → low variance
        for _ in 0..5 {
            c.push(-0.3);
        }
        assert!(c.window_variance() < 0.01,
            "Stable values → low variance: {}", c.window_variance());
        assert!(!c.is_unstable(), "Stable → not unstable");
    }

    #[test]
    fn curve_window_variance_oscillating() {
        let mut c = ConversationCurve::new();
        // Oscillating: -0.5, 0.3, -0.4, 0.2, -0.3 → high variance + sign changes
        let vals = [-0.5, 0.3, -0.4, 0.2, -0.3];
        for v in vals {
            c.push(v);
        }
        assert!(c.window_variance() > 0.04,
            "Oscillating → high variance: {}", c.window_variance());
        assert!(c.is_unstable(),
            "Oscillating with sign changes → unstable");
    }

    #[test]
    fn curve_instability_overrides_celebratory() {
        let mut c = ConversationCurve::new();
        // Build instability: oscillate, then recover
        let vals = [-0.5, 0.3, -0.4, 0.2, 0.5];
        for v in vals {
            c.push(v);
        }
        // Even if base tone would be Celebratory, instability → Gentle
        if c.is_unstable() {
            let tone = c.tone();
            assert_ne!(tone, ResponseTone::Celebratory,
                "Unstable → never Celebratory");
        }
    }

    #[test]
    fn curve_window_variance_too_few_turns() {
        let mut c = ConversationCurve::new();
        c.push(-0.5);
        c.push(0.3);
        assert_eq!(c.window_variance(), 0.0,
            "< 5 turns → variance = 0");
        assert!(!c.is_unstable());
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
