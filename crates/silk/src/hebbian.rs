//! # hebbian — Hebbian Learning Engine
//!
//! "Neurons that fire together, wire together."
//!
//! ## Co-activation:
//!   weight += reward × (1 - w) × lr   (lr = 0.1)
//!   emotion = blend(edge.emotion, new_emotion, intensity)
//!
//! ## Decay (24h):
//!   weight × φ⁻¹   (φ = 1.618...)
//!   "Không dùng → quên"
//!
//! ## Fibonacci threshold:
//!   Promote khi: weight ≥ 0.7 AND fire_count ≥ Fib[depth]
//!   Depth càng sâu → threshold càng cao

use crate::edge::EmotionTag;

/// Learning rate Hebbian.
pub const LR: f32 = 0.1;

/// Golden ratio φ = 1.618...
pub const PHI: f32 = 1.618_034;

/// φ⁻¹ = 1/φ ≈ 0.618 — decay factor mỗi 24h.
pub const PHI_INV: f32 = 0.618_034;

/// Weight threshold để promote lên nhánh mới.
pub const PROMOTE_WEIGHT: f32 = 0.7;

/// Nanoseconds trong 24 giờ.
pub const NS_PER_DAY: i64 = 86_400_000_000_000;

// ─────────────────────────────────────────────────────────────────────────────
// Fibonacci sequence
// ─────────────────────────────────────────────────────────────────────────────

/// Fibonacci(n) — threshold cho tầng thứ n.
///
/// Fib[0]=1, Fib[1]=1, Fib[2]=2, Fib[3]=3, Fib[4]=5, ...
/// Tầng càng sâu → cần co-activate nhiều hơn mới promote.
pub fn fib(n: usize) -> u32 {
    match n {
        0 | 1 => 1,
        _ => {
            let mut a = 1u32;
            let mut b = 1u32;
            for _ in 2..=n {
                let c = a.saturating_add(b);
                a = b;
                b = c;
            }
            b
        }
    }
}

/// Kiểm tra có thể promote không.
///
/// promote = weight ≥ 0.7 AND fire_count ≥ Fib[depth]
pub fn should_promote(weight: f32, fire_count: u32, depth: usize) -> bool {
    weight >= PROMOTE_WEIGHT && fire_count >= fib(depth)
}

// ─────────────────────────────────────────────────────────────────────────────
// Hebbian update
// ─────────────────────────────────────────────────────────────────────────────

/// Cập nhật weight khi co-activation xảy ra.
///
/// weight += reward × (1 - w) × lr
/// reward ∈ [0.0, 1.0] — thường = intensity của EmotionTag
pub fn hebbian_strengthen(weight: f32, reward: f32) -> f32 {
    let delta = reward * (1.0 - weight) * LR;
    (weight + delta).min(1.0)
}

/// Decay weight sau khoảng thời gian elapsed_ns.
///
/// Số 24h periods = elapsed_ns / NS_PER_DAY
/// weight × φ⁻¹^periods
pub fn hebbian_decay(weight: f32, elapsed_ns: i64) -> f32 {
    if elapsed_ns <= 0 {
        return weight;
    }
    let days = elapsed_ns as f32 / NS_PER_DAY as f32;
    // weight × φ⁻¹^days
    let factor = libm::powf(PHI_INV, days);
    (weight * factor).max(0.0)
}

/// Blend emotion của edge với emotion mới.
///
/// Edge "nhớ" cảm xúc — emotion mới blend vào với weight = intensity.
pub fn blend_emotion(current: EmotionTag, new_emotion: EmotionTag, intensity: f32) -> EmotionTag {
    current.blend(new_emotion, 1.0 - intensity) // new blends in
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Fibonacci ────────────────────────────────────────────────────────────

    #[test]
    fn fib_sequence() {
        assert_eq!(fib(0), 1);
        assert_eq!(fib(1), 1);
        assert_eq!(fib(2), 2);
        assert_eq!(fib(3), 3);
        assert_eq!(fib(4), 5);
        assert_eq!(fib(5), 8);
        assert_eq!(fib(6), 13);
        assert_eq!(fib(7), 21);
        assert_eq!(fib(10), 89);
    }

    #[test]
    fn promote_threshold_deeper_harder() {
        // Tầng càng sâu → Fib càng lớn → khó promote hơn
        assert!(fib(2) < fib(5), "Fib[2] < Fib[5]");
        assert!(fib(5) < fib(10), "Fib[5] < Fib[10]");
    }

    #[test]
    fn should_promote_basic() {
        // weight=0.8, fire=5, depth=3 (Fib[3]=3) → promote
        assert!(should_promote(0.8, 5, 3));
        // weight=0.6 (< 0.7) → không promote
        assert!(!should_promote(0.6, 10, 1));
        // weight=0.8, fire=2, depth=4 (Fib[4]=5) → không đủ fire
        assert!(!should_promote(0.8, 2, 4));
    }

    #[test]
    fn should_promote_boundary() {
        // Chính xác boundary
        assert!(
            should_promote(0.7, fib(3), 3),
            "Boundary: weight=0.7, fire=Fib[3]"
        );
        assert!(!should_promote(0.699, fib(3), 3), "Below weight threshold");
        assert!(!should_promote(0.7, fib(3) - 1, 3), "Below fire threshold");
    }

    // ── Hebbian strengthen ───────────────────────────────────────────────────

    #[test]
    fn strengthen_increases_weight() {
        let w0 = 0.3f32;
        let w1 = hebbian_strengthen(w0, 1.0);
        assert!(w1 > w0, "Strengthen phải tăng weight");
    }

    #[test]
    fn strengthen_never_exceeds_one() {
        let w = hebbian_strengthen(0.99, 1.0);
        assert!(w <= 1.0, "Weight không vượt quá 1.0");
    }

    #[test]
    fn strengthen_formula() {
        // weight += reward × (1 - w) × lr
        let w0 = 0.5f32;
        let reward = 0.8f32;
        let expected = w0 + reward * (1.0 - w0) * LR;
        let got = hebbian_strengthen(w0, reward);
        assert!((got - expected).abs() < 1e-6, "Formula đúng");
    }

    #[test]
    fn strengthen_high_weight_slow() {
        // weight cao → tăng chậm hơn
        let delta_low = hebbian_strengthen(0.1, 1.0) - 0.1;
        let delta_high = hebbian_strengthen(0.9, 1.0) - 0.9;
        assert!(
            delta_low > delta_high,
            "Weight thấp tăng nhanh hơn weight cao"
        );
    }

    // ── Hebbian decay ────────────────────────────────────────────────────────

    #[test]
    fn decay_decreases_weight() {
        let w0 = 0.8f32;
        let w1 = hebbian_decay(w0, NS_PER_DAY); // 1 ngày
        assert!(w1 < w0, "Decay phải giảm weight");
        assert!(w1 >= 0.0, "Weight không âm");
    }

    #[test]
    fn decay_one_day_phi_inv() {
        let w0 = 1.0f32;
        let w1 = hebbian_decay(w0, NS_PER_DAY);
        assert!(
            (w1 - PHI_INV).abs() < 0.001,
            "Sau 1 ngày: weight × φ⁻¹ ≈ {}, got {}",
            PHI_INV,
            w1
        );
    }

    #[test]
    fn decay_no_elapsed_no_change() {
        let w = 0.7f32;
        assert_eq!(hebbian_decay(w, 0), w, "0 thời gian → không decay");
        assert_eq!(hebbian_decay(w, -100), w, "Negative time → không decay");
    }

    #[test]
    fn decay_multiple_days() {
        let w0 = 1.0f32;
        let w7 = hebbian_decay(w0, NS_PER_DAY * 7); // 1 tuần
        let expected = libm::powf(PHI_INV, 7.0);
        assert!(
            (w7 - expected).abs() < 0.001,
            "7 ngày: w7={} ≈ φ⁻⁷={}",
            w7,
            expected
        );
    }

    #[test]
    fn decay_partial_day() {
        let w0 = 1.0f32;
        let w_half = hebbian_decay(w0, NS_PER_DAY / 2); // 12h
        let expected = libm::powf(PHI_INV, 0.5);
        assert!((w_half - expected).abs() < 0.001);
    }

    // ── Emotion blend ────────────────────────────────────────────────────────

    #[test]
    fn blend_emotion_high_intensity() {
        // blend_emotion(current, new, intensity)
        // = current.blend(new, 1.0 - intensity)
        // = current × (1-intensity) + new × intensity
        // intensity=0.9 → new dominates (90%)
        let current = EmotionTag::new(0.0, 0.0, 0.5, 0.5);
        let new_emo = EmotionTag::new(1.0, 1.0, 0.5, 0.5);
        let blended = blend_emotion(current, new_emo, 0.9);
        assert!(
            blended.valence > 0.5,
            "High intensity → new_emo dominates: val={}",
            blended.valence
        );
    }

    #[test]
    fn blend_emotion_low_intensity() {
        // intensity=0.1 → current dominates (90%)
        let current = EmotionTag::new(-1.0, 0.0, 0.5, 0.5);
        let new_emo = EmotionTag::new(1.0, 1.0, 0.5, 0.5);
        let blended = blend_emotion(current, new_emo, 0.1);
        assert!(
            blended.valence < 0.0,
            "Low intensity → current dominates: val={}",
            blended.valence
        );
    }
}
