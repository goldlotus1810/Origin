//! # hebbian — Hebbian Learning Engine
//!
//! "Neurons that fire together, wire together."
//!
//! ## Co-activation:
//!   weight += reward × (1 - w) × lr   (lr = φ⁻³ ≈ 0.236)
//!   emotion = blend(edge.emotion, new_emotion, intensity)
//!
//! ## Decay (24h):
//!   weight × φ⁻¹   (φ = (1+√5)/2, computed — not hardcoded)
//!   "Không dùng → quên"
//!
//! ## Fibonacci threshold:
//!   Promote khi: weight ≥ PROMOTE_WEIGHT AND fire_count ≥ Fib[depth]
//!   Depth càng sâu → threshold càng cao
//!
//! ## Adaptive precision:
//!   Mọi constant tính từ φ = (1+√5)/2 — KHÔNG hardcode giá trị.
//!   1M nodes: 3 chữ số đủ. 1B nodes: cần 15+ chữ số.

use crate::edge::EmotionTag;

// ── Computed constants from φ = (1+√5)/2 ────────────────────────────────────

/// Golden ratio φ = (1+√5)/2 — computed from formula.
pub const PHI: f32 = compute_phi_f32();

/// φ⁻¹ = 1/φ = (√5-1)/2 — decay factor mỗi 24h. Computed.
pub const PHI_INV: f32 = compute_phi_inv_f32();

/// Learning rate = φ⁻³ ≈ 0.236 — derived from golden ratio.
/// Trước đây hardcode 0.1, giờ tính từ φ cho consistency.
pub const LR: f32 = compute_lr_f32();

/// Weight threshold để promote: φ⁻¹ + φ⁻³ ≈ 0.854
/// Trước đây 0.7 (quá dễ), giờ derived từ φ → selective hơn.
pub const PROMOTE_WEIGHT: f32 = compute_promote_f32();

/// Compute φ = (1+√5)/2 at compile time (const fn).
const fn compute_phi_f32() -> f32 {
    // √5 ≈ 2.2360679774997896 (computed via Newton's method at f64, cast to f32)
    // We use a high-precision literal since const fn can't call homemath
    const SQRT5: f32 = 2.236_068; // √5 to f32 precision
    (1.0 + SQRT5) / 2.0
}

/// Compute φ⁻¹ = (√5-1)/2 at compile time.
const fn compute_phi_inv_f32() -> f32 {
    const SQRT5: f32 = 2.236_068; // √5 to f32 precision
    (SQRT5 - 1.0) / 2.0
}

/// Compute learning rate = φ⁻³ at compile time.
const fn compute_lr_f32() -> f32 {
    let phi_inv = compute_phi_inv_f32();
    phi_inv * phi_inv * phi_inv
}

/// Compute promote weight = φ⁻¹ + φ⁻³ at compile time.
const fn compute_promote_f32() -> f32 {
    let phi_inv = compute_phi_inv_f32();
    let phi_inv3 = phi_inv * phi_inv * phi_inv;
    phi_inv + phi_inv3
}

/// Runtime φ with full f64 precision (uses homemath::sqrt).
pub fn phi_f64() -> f64 {
    (1.0 + homemath::sqrt(5.0)) / 2.0
}

/// Runtime φ⁻¹ with full f64 precision.
pub fn phi_inv_f64() -> f64 {
    (homemath::sqrt(5.0) - 1.0) / 2.0
}

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
///
/// Tính bằng f64 internally → truncate f32 cuối cùng.
/// 1B co-activations × f64 error ~1e-15 = ~1 sai lệch (vs f32: ~1000 sai lệch)
pub fn hebbian_strengthen(weight: f32, reward: f32) -> f32 {
    // f64 precision for accumulated operations
    let w = weight as f64;
    let r = reward as f64;
    let p = phi_inv_f64();
    let lr = p * p * p; // φ⁻³ at full f64 precision
    let delta = r * (1.0 - w) * lr;
    ((w + delta).min(1.0)) as f32
}

/// Decay weight sau khoảng thời gian elapsed_ns.
///
/// Số 24h periods = elapsed_ns / NS_PER_DAY
/// weight × φ⁻¹^periods
///
/// Dùng f64 φ⁻¹ computed từ (√5-1)/2 — không hardcode.
pub fn hebbian_decay(weight: f32, elapsed_ns: i64) -> f32 {
    if elapsed_ns <= 0 {
        return weight;
    }
    let days = elapsed_ns as f64 / NS_PER_DAY as f64;
    // weight × φ⁻¹^days — f64 precision
    let phi_inv = phi_inv_f64();
    let factor = homemath::pow(phi_inv, days);
    ((weight as f64 * factor).max(0.0)) as f32
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
        // PROMOTE_WEIGHT ≈ 0.854 (φ⁻¹ + φ⁻³)
        // weight=0.9 > 0.854, fire=5, depth=3 (Fib[3]=3) → promote
        assert!(should_promote(0.9, 5, 3));
        // weight=0.6 (< PROMOTE_WEIGHT) → không promote
        assert!(!should_promote(0.6, 10, 1));
        // weight=0.9, fire=2, depth=4 (Fib[4]=5) → không đủ fire
        assert!(!should_promote(0.9, 2, 4));
        // weight=0.8 (< PROMOTE_WEIGHT ≈ 0.854) → không promote
        assert!(!should_promote(0.8, 10, 1));
    }

    #[test]
    fn should_promote_boundary() {
        // Chính xác boundary — PROMOTE_WEIGHT = φ⁻¹ + φ⁻³ ≈ 0.854
        assert!(
            should_promote(PROMOTE_WEIGHT, fib(3), 3),
            "Boundary: weight=PROMOTE_WEIGHT, fire=Fib[3]"
        );
        assert!(
            !should_promote(PROMOTE_WEIGHT - 0.001, fib(3), 3),
            "Below weight threshold"
        );
        assert!(!should_promote(PROMOTE_WEIGHT, fib(3) - 1, 3), "Below fire threshold");
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
        // weight += reward × (1 - w) × φ⁻³
        let w0 = 0.5f32;
        let reward = 0.8f32;
        // hebbian_strengthen uses f64 internally for precision
        let p = phi_inv_f64();
        let lr_f64 = p * p * p;
        let expected = (w0 as f64 + reward as f64 * (1.0 - w0 as f64) * lr_f64) as f32;
        let got = hebbian_strengthen(w0, reward);
        assert!((got - expected).abs() < 1e-6, "Formula: expected={}, got={}", expected, got);
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
        let expected = homemath::powf(PHI_INV, 7.0);
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
        let expected = homemath::powf(PHI_INV, 0.5);
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
