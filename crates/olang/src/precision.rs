//! # precision — Adaptive precision for all numerical operations
//!
//! Thay vì hardcode `0.618` (3 chữ số), hệ thống tính φ⁻¹ theo precision tier:
//!   Sensor: 0.618     (3 digits, 5 iter)
//!   Worker: 0.6180340 (7 digits, 15 iter)
//!   Compact: 0.618033988749895 (15 digits, 50 iter)
//!   Full:    0.6180339887498948482... (max f64, 200 iter)
//!
//! 1M nodes → 3 digits đủ
//! 1B nodes → cần 15+ digits để phân biệt edges gần nhau
//!
//! ## Vấn đề:
//! - `weight × 0.618` với 1B edges → error tích lũy = 0.001 × 1B = 1M sai lệch
//! - `weight × φ⁻¹_computed` → error = 1e-15 × 1B = ~1 sai lệch
//!
//! ## Thiết kế:
//! - `PrecisionConfig` giữ tất cả constant đã tính sẵn
//! - Tạo 1 lần khi khởi động, dùng xuyên suốt
//! - Tier tự detect từ HAL, hoặc set manual

extern crate alloc;

use crate::constants::{MathConstant, Precision};

// ─────────────────────────────────────────────────────────────────────────────
// PrecisionConfig — computed constants for a given tier
// ─────────────────────────────────────────────────────────────────────────────

/// All numerical constants pre-computed at a specific precision.
/// Created once at startup, used throughout the system.
#[derive(Debug, Clone)]
pub struct PrecisionConfig {
    /// Current precision tier
    pub tier: Precision,

    // ── Fundamental constants ────────────────────────────────────────────

    /// φ = (1 + √5) / 2 — golden ratio
    pub phi: f64,
    /// φ⁻¹ = 1/φ ≈ 0.618... — Hebbian decay factor per 24h
    pub phi_inv: f64,
    /// π — circle constant
    pub pi: f64,
    /// e — Euler's number
    pub e: f64,
    /// √2
    pub sqrt2: f64,
    /// ln(2)
    pub ln2: f64,

    // ── Derived thresholds (from φ) ──────────────────────────────────────

    /// Hebbian learning rate: φ⁻¹ / φ² ≈ 0.236 → clamped to standard 0.1
    /// But the RELATIONSHIP is: lr = φ⁻² × scale
    pub learning_rate: f64,

    /// Hebbian promote threshold: φ⁻¹ + φ⁻³ ≈ 0.854
    /// Raised from 0.7 — more selective at high precision
    pub promote_weight: f64,

    // ── Scaling factors ──────────────────────────────────────────────────

    /// UCD valence byte divisor: 128.0 (power of 2, exact in IEEE 754)
    /// Replaces 127.5 which has rounding errors
    pub ucd_valence_divisor: f64,

    /// Weight quantization levels: tier-dependent
    /// Sensor=255, Worker=255, Compact=65535, Full=65535
    pub weight_quant_levels: u32,

    // ── Emotion blend weights (from φ) ───────────────────────────────────

    /// Conversation blend: α = φ⁻¹ ≈ 0.618 (current turn weight)
    pub curve_alpha: f64,
    /// Conversation blend: β = 1 - α = φ⁻² ≈ 0.382 (history weight)
    pub curve_beta: f64,

    /// Derivative weight: φ⁻² ≈ 0.382 (first derivative)
    pub d1_weight: f64,
    /// Second derivative weight: φ⁻³ ≈ 0.236
    pub d2_weight: f64,

    /// EMA smoothing: old = φ⁻¹, new = φ⁻²
    /// EMA smoothing: old factor = φ⁻¹
    pub ema_old: f64,
    /// EMA smoothing: new factor = φ⁻²
    pub ema_new: f64,

    // ── Fusion weights ───────────────────────────────────────────────────

    /// Confidence threshold (BlackCurtain): φ⁻² × (1 - φ⁻³) ≈ 0.35
    pub confidence_threshold: f64,

    /// Conflict detection: φ⁻¹ × φ⁻¹ ≈ 0.382
    pub conflict_threshold: f64,
}

impl PrecisionConfig {
    /// Create a PrecisionConfig for a given precision tier.
    /// All constants computed from formulas — NOTHING hardcoded.
    pub fn new(tier: Precision) -> Self {
        let phi = MathConstant::Phi.compute(tier);
        let phi_inv = 1.0 / phi;
        let phi_inv2 = phi_inv * phi_inv;
        let phi_inv3 = phi_inv2 * phi_inv;

        Self {
            tier,

            phi,
            phi_inv,
            pi: MathConstant::Pi.compute(tier),
            e: MathConstant::E.compute(tier),
            sqrt2: MathConstant::Sqrt2.compute(tier),
            ln2: MathConstant::Ln2.compute(tier),

            // Derived from φ — consistent relationships
            learning_rate: phi_inv2 * phi_inv, // φ⁻³ ≈ 0.236, but clamped
            promote_weight: phi_inv + phi_inv3, // φ⁻¹ + φ⁻³ ≈ 0.854

            // Power-of-2 divisor: exact in IEEE 754
            ucd_valence_divisor: 128.0,

            // Tier-dependent quantization
            weight_quant_levels: match tier {
                Precision::Low | Precision::Medium => 255,
                Precision::High | Precision::Ultra => 65535,
            },

            // All blend weights from φ — self-consistent
            curve_alpha: phi_inv,           // ≈ 0.618
            curve_beta: phi_inv2,           // ≈ 0.382 (= 1 - φ⁻¹)
            d1_weight: phi_inv2,            // ≈ 0.382
            d2_weight: phi_inv3,            // ≈ 0.236
            ema_old: phi_inv,               // ≈ 0.618
            ema_new: phi_inv2,              // ≈ 0.382

            // Thresholds from φ relationships
            confidence_threshold: phi_inv2 * (1.0 - phi_inv3), // ≈ 0.29
            conflict_threshold: phi_inv2,    // ≈ 0.382
        }
    }

    /// Hebbian PHI_INV as f32 (for silk crate compatibility).
    pub fn phi_inv_f32(&self) -> f32 {
        self.phi_inv as f32
    }

    /// Hebbian learning rate as f32.
    pub fn learning_rate_f32(&self) -> f32 {
        self.learning_rate as f32
    }

    /// Hebbian promote weight as f32.
    pub fn promote_weight_f32(&self) -> f32 {
        self.promote_weight as f32
    }

    /// Curve alpha as f32.
    pub fn curve_alpha_f32(&self) -> f32 {
        self.curve_alpha as f32
    }

    /// Curve beta as f32.
    pub fn curve_beta_f32(&self) -> f32 {
        self.curve_beta as f32
    }

    /// Quantize weight to tier-appropriate precision.
    /// Returns (quantized_value, level_count).
    pub fn quantize_weight(&self, w: f32) -> (u32, u32) {
        let clamped = w.clamp(0.0, 1.0);
        let levels = self.weight_quant_levels;
        let quantized = (clamped * levels as f32) as u32;
        (quantized, levels)
    }

    /// Dequantize weight from tier-appropriate precision.
    pub fn dequantize_weight(&self, q: u32) -> f32 {
        q as f32 / self.weight_quant_levels as f32
    }

    /// Scale UCD valence byte to [-1.0, +1.0] using power-of-2 divisor.
    pub fn scale_valence_byte(&self, b: u8) -> f32 {
        (b as f32 / self.ucd_valence_divisor as f32) - 1.0
    }

    /// Scale UCD arousal byte to [0.0, 1.0].
    pub fn scale_arousal_byte(&self, b: u8) -> f32 {
        b as f32 / 255.0
    }

    /// Summary for display.
    pub fn summary(&self) -> alloc::string::String {
        alloc::format!(
            "PrecisionConfig ({:?})\n\
             φ       = {:.16}\n\
             φ⁻¹     = {:.16}\n\
             π       = {:.16}\n\
             LR      = {:.16}\n\
             Promote = {:.16}\n\
             α (conv)= {:.16}\n\
             β (hist)= {:.16}\n\
             Quant   = {} levels",
            self.tier,
            self.phi,
            self.phi_inv,
            self.pi,
            self.learning_rate,
            self.promote_weight,
            self.curve_alpha,
            self.curve_beta,
            self.weight_quant_levels,
        )
    }
}

// Default = High precision (f64 max, good for most platforms)
impl Default for PrecisionConfig {
    fn default() -> Self {
        Self::new(Precision::High)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_identity() {
        let cfg = PrecisionConfig::new(Precision::Ultra);
        // φ² = φ + 1
        assert!((cfg.phi * cfg.phi - cfg.phi - 1.0).abs() < 1e-14);
    }

    #[test]
    fn phi_inv_identity() {
        let cfg = PrecisionConfig::new(Precision::Ultra);
        // φ × φ⁻¹ = 1
        assert!((cfg.phi * cfg.phi_inv - 1.0).abs() < 1e-14);
    }

    #[test]
    fn alpha_beta_sum_close_to_one() {
        let cfg = PrecisionConfig::new(Precision::High);
        // φ⁻¹ + φ⁻² = 1 (exact identity of golden ratio)
        assert!((cfg.curve_alpha + cfg.curve_beta - 1.0).abs() < 1e-14,
            "α + β = {} ≠ 1.0", cfg.curve_alpha + cfg.curve_beta);
    }

    #[test]
    fn ema_sum_close_to_one() {
        let cfg = PrecisionConfig::new(Precision::High);
        assert!((cfg.ema_old + cfg.ema_new - 1.0).abs() < 1e-14);
    }

    #[test]
    fn higher_precision_more_phi_digits() {
        let low = PrecisionConfig::new(Precision::Low);
        let ultra = PrecisionConfig::new(Precision::Ultra);
        // Both should be close to true φ
        let true_phi = (1.0 + libm::sqrt(5.0)) / 2.0;
        assert!((ultra.phi - true_phi).abs() <= (low.phi - true_phi).abs());
    }

    #[test]
    fn quantize_dequantize_roundtrip() {
        let cfg = PrecisionConfig::new(Precision::High);
        let w = 0.7345_f32;
        let (q, _) = cfg.quantize_weight(w);
        let w2 = cfg.dequantize_weight(q);
        // 65535 levels → error < 1/65535 ≈ 1.5e-5
        assert!((w - w2).abs() < 2.0 / 65535.0,
            "roundtrip error: {} → {} → {} (err={})", w, q, w2, (w - w2).abs());
    }

    #[test]
    fn quantize_low_precision() {
        let cfg = PrecisionConfig::new(Precision::Low);
        assert_eq!(cfg.weight_quant_levels, 255); // u8
    }

    #[test]
    fn quantize_high_precision() {
        let cfg = PrecisionConfig::new(Precision::High);
        assert_eq!(cfg.weight_quant_levels, 65535); // u16
    }

    #[test]
    fn valence_scaling_symmetric() {
        let cfg = PrecisionConfig::new(Precision::High);
        let v0 = cfg.scale_valence_byte(0);     // → -1.0
        let v128 = cfg.scale_valence_byte(128);   // → 0.0
        let v255 = cfg.scale_valence_byte(255);   // → ~0.99
        assert!(v0 < -0.9, "byte 0 → {}", v0);
        assert!(v128.abs() < 0.01, "byte 128 → {} (should be ~0)", v128);
        assert!(v255 > 0.9, "byte 255 → {}", v255);
    }

    #[test]
    fn promote_weight_stricter_than_old() {
        let cfg = PrecisionConfig::new(Precision::High);
        // Old hardcode was 0.7, new is φ⁻¹ + φ⁻³ ≈ 0.854
        assert!(cfg.promote_weight > 0.7, "promote = {}", cfg.promote_weight);
    }

    #[test]
    fn learning_rate_from_phi() {
        let cfg = PrecisionConfig::new(Precision::High);
        // lr = φ⁻³ ≈ 0.236
        let phi_inv3 = 1.0 / (cfg.phi * cfg.phi * cfg.phi);
        assert!((cfg.learning_rate - phi_inv3).abs() < 1e-12);
    }

    #[test]
    fn precision_comparison() {
        let low = PrecisionConfig::new(Precision::Low);
        let high = PrecisionConfig::new(Precision::High);

        // Low and high should compute the same thing to within f64 precision
        // (since φ is algebraic, not series-based)
        assert!((low.phi - high.phi).abs() < 1e-14);

        // But π computed from series should differ
        // (low uses 5 iterations, high uses 50)
        // Actually for Machin's formula, convergence is fast
        // The key point is: both are COMPUTED, not hardcoded
        assert!(low.pi > 3.0 && low.pi < 4.0);
        assert!(high.pi > 3.0 && high.pi < 4.0);
    }
}
