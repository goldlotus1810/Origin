//! # spline — SplineKnot + TimeHistory (FE.4, FE.5)
//!
//! Temporal dimension as append-only spline knots.
//! Each observation of a concept = 1 SplineKnot (24 bytes).
//! TimeHistory accumulates knots → interpolate → predict.
//!
//! FE.4: SplineKnot + TimeHistory structures
//! FE.5: Spline interpolation + prediction

extern crate alloc;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// SplineKnot — 24 bytes, compact, append-only
// ─────────────────────────────────────────────────────────────────────────────

/// A single observation point on the temporal spline.
/// 24 bytes — compact, append-only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplineKnot {
    /// When observed (ms since epoch).
    pub timestamp: u64,
    /// Size / intensity / magnitude.
    pub amplitude: f32,
    /// Hz — oscillation / repetition rate.
    pub frequency: f32,
    /// Radians — position in cycle.
    pub phase: f32,
    /// Seconds — how long the observation lasted.
    pub duration: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// TimeMode — T 2-bit value
// ─────────────────────────────────────────────────────────────────────────────

/// Time mode (from T 2-bit value in P_weight).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeMode {
    /// T=0: no temporal pattern.
    Timeless = 0,
    /// T=1: ordered sequence (reading, steps).
    Sequential = 1,
    /// T=2: repeating cycle (day/night, seasons).
    Cyclical = 2,
    /// T=3: wave pattern (music, heartbeat, dance).
    Rhythmic = 3,
}

impl TimeMode {
    /// Parse from 2-bit T value.
    pub fn from_bits(t: u8) -> Self {
        match t & 0x03 {
            0 => Self::Timeless,
            1 => Self::Sequential,
            2 => Self::Cyclical,
            3 => Self::Rhythmic,
            _ => unreachable!(),
        }
    }

    /// Encode as 2-bit value.
    pub fn as_bits(self) -> u8 {
        self as u8
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TimeHistory — append-only knot sequence
// ─────────────────────────────────────────────────────────────────────────────

/// Append-only history of spline knots for a concept.
pub struct TimeHistory {
    knots: Vec<SplineKnot>,
}

impl TimeHistory {
    /// Create empty history.
    pub fn new() -> Self {
        Self {
            knots: Vec::new(),
        }
    }

    /// Append a knot.
    pub fn push(&mut self, knot: SplineKnot) {
        self.knots.push(knot);
    }

    /// Number of observations.
    pub fn len(&self) -> usize {
        self.knots.len()
    }

    /// Whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.knots.is_empty()
    }

    /// Slice of all knots.
    pub fn knots(&self) -> &[SplineKnot] {
        &self.knots
    }

    /// Create knot from text observation.
    pub fn observe_text(text: &str, timestamp: u64) -> SplineKnot {
        SplineKnot {
            timestamp,
            amplitude: text.len() as f32 / 100.0, // longer text = higher amplitude
            frequency: 0.0,                        // text is not periodic
            phase: 0.0,
            duration: text.len() as f32 * 0.05, // ~50ms per char reading time
        }
    }

    /// Create knot from sensor measurement.
    pub fn observe_sensor(value: f32, freq: f32, timestamp: u64) -> SplineKnot {
        SplineKnot {
            timestamp,
            amplitude: value,
            frequency: freq,
            phase: 0.0,
            duration: 0.001, // sensor sample = instantaneous
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FE.5: Spline Interpolation + Prediction
// ─────────────────────────────────────────────────────────────────────────────

/// Prediction derived from TimeHistory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimePrediction {
    /// 0.0 = new, 1.0 = well-known. Saturates at 10 observations.
    pub familiarity: f32,
    /// Decreases with familiarity (1/sqrt(n)).
    pub learning_rate: f32,
    /// Average intensity across all knots.
    pub avg_amplitude: f32,
    /// 0.0 = random, 1.0 = perfectly periodic.
    pub periodicity: f32,
}

impl TimeHistory {
    /// Interpolate amplitude at time `t` (linear between knots).
    ///
    /// Returns 0.0 if no knots. Clamps to first/last knot outside range.
    pub fn amplitude_at(&self, t: u64) -> f32 {
        let n = self.knots.len();
        if n == 0 {
            return 0.0;
        }
        if n == 1 {
            return self.knots[0].amplitude;
        }

        // Before first knot → clamp
        if t <= self.knots[0].timestamp {
            return self.knots[0].amplitude;
        }
        // After last knot → clamp
        if t >= self.knots[n - 1].timestamp {
            return self.knots[n - 1].amplitude;
        }

        // Find bracketing knots and lerp
        for i in 0..n - 1 {
            let k0 = &self.knots[i];
            let k1 = &self.knots[i + 1];
            if t >= k0.timestamp && t <= k1.timestamp {
                let dt = (k1.timestamp - k0.timestamp) as f32;
                if dt < 1.0 {
                    return k0.amplitude;
                }
                let frac = (t - k0.timestamp) as f32 / dt;
                return k0.amplitude + (k1.amplitude - k0.amplitude) * frac;
            }
        }

        // Fallback (should not reach here)
        self.knots[n - 1].amplitude
    }

    /// Predict behavior from history.
    pub fn predict(&self) -> TimePrediction {
        let n = self.knots.len();
        let n_f = n.max(1) as f32;
        TimePrediction {
            familiarity: (n as f32 / 10.0).min(1.0),
            learning_rate: if n < 3 { 1.0 } else { 1.0 / sqrt_f32(n_f) },
            avg_amplitude: self.knots.iter().map(|k| k.amplitude).sum::<f32>() / n_f,
            periodicity: self.detect_periodicity(),
        }
    }

    /// Detect if observations have periodic pattern.
    ///
    /// Uses variance of inter-arrival deltas: low variance = periodic.
    /// Returns 0.0 for < 3 knots, otherwise 1/(1+cv) where cv = coefficient of variation.
    fn detect_periodicity(&self) -> f32 {
        let n = self.knots.len();
        if n < 3 {
            return 0.0;
        }

        // Compute inter-arrival deltas
        let mut deltas = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            let dt = (self.knots[i + 1].timestamp - self.knots[i].timestamp) as f32;
            deltas.push(dt);
        }

        let count = deltas.len() as f32;
        let mean = deltas.iter().sum::<f32>() / count;
        if mean < 1.0 {
            return 0.0; // all at same time, no periodicity signal
        }

        let variance = deltas.iter().map(|d| (d - mean) * (d - mean)).sum::<f32>() / count;
        let std_dev = sqrt_f32(variance);
        let cv = std_dev / mean; // coefficient of variation

        // cv=0 → perfectly periodic (return 1.0), cv→∞ → random (return 0.0)
        1.0 / (1.0 + cv)
    }
}

/// no_std-compatible sqrt for f32.
#[inline]
fn sqrt_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    // Newton-Raphson, 8 iterations — good for f32 precision
    let mut guess = x;
    let mut i = 0;
    while i < 8 {
        guess = 0.5 * (guess + x / guess);
        i += 1;
    }
    guess
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spline_knot_create_store() {
        let knot = SplineKnot {
            timestamp: 1000,
            amplitude: 2.5,
            frequency: 440.0,
            phase: 0.0,
            duration: 1.0,
        };
        assert_eq!(knot.timestamp, 1000);
        assert!((knot.amplitude - 2.5).abs() < 1e-5);
        assert!((knot.frequency - 440.0).abs() < 1e-5);

        // Size check: 8 + 4 + 4 + 4 + 4 = 24 bytes
        assert_eq!(core::mem::size_of::<SplineKnot>(), 24);
    }

    #[test]
    fn spline_time_history_append_predict() {
        let mut hist = TimeHistory::new();
        assert!(hist.is_empty());

        // Push 5 knots
        for i in 0..5 {
            hist.push(SplineKnot {
                timestamp: i * 100,
                amplitude: (i as f32 + 1.0) * 0.5,
                frequency: 0.0,
                phase: 0.0,
                duration: 0.1,
            });
        }
        assert_eq!(hist.len(), 5);
        assert_eq!(hist.knots().len(), 5);

        let pred = hist.predict();
        assert!((pred.familiarity - 0.5).abs() < 1e-5); // 5/10 = 0.5
        assert!(pred.learning_rate < 1.0); // n=5 ≥ 3 → 1/sqrt(5)
        assert!(pred.avg_amplitude > 0.0);
    }

    #[test]
    fn spline_amplitude_interpolation() {
        let mut hist = TimeHistory::new();
        hist.push(SplineKnot {
            timestamp: 0,
            amplitude: 1.0,
            frequency: 0.0,
            phase: 0.0,
            duration: 0.1,
        });
        hist.push(SplineKnot {
            timestamp: 100,
            amplitude: 3.0,
            frequency: 0.0,
            phase: 0.0,
            duration: 0.1,
        });

        // Midpoint should be 2.0
        let mid = hist.amplitude_at(50);
        assert!((mid - 2.0).abs() < 1e-3, "midpoint amplitude: {}", mid);

        // At start
        let start = hist.amplitude_at(0);
        assert!((start - 1.0).abs() < 1e-3);

        // At end
        let end = hist.amplitude_at(100);
        assert!((end - 3.0).abs() < 1e-3);

        // Before start → clamp
        let before = hist.amplitude_at(0);
        assert!((before - 1.0).abs() < 1e-3);

        // After end → clamp
        let after = hist.amplitude_at(200);
        assert!((after - 3.0).abs() < 1e-3);
    }

    #[test]
    fn spline_observe_text() {
        let knot = TimeHistory::observe_text("hello world", 5000);
        assert_eq!(knot.timestamp, 5000);
        assert!((knot.amplitude - 0.11).abs() < 0.01); // 11/100 = 0.11
        assert!((knot.frequency - 0.0).abs() < 1e-5);
        assert!(knot.duration > 0.0);
    }

    #[test]
    fn spline_observe_sensor() {
        let knot = TimeHistory::observe_sensor(42.0, 100.0, 9000);
        assert_eq!(knot.timestamp, 9000);
        assert!((knot.amplitude - 42.0).abs() < 1e-5);
        assert!((knot.frequency - 100.0).abs() < 1e-5);
        assert!((knot.duration - 0.001).abs() < 1e-5);
    }

    #[test]
    fn spline_periodicity_detection() {
        let mut hist = TimeHistory::new();
        // Perfectly periodic: every 100ms
        for i in 0..10 {
            hist.push(SplineKnot {
                timestamp: i * 100,
                amplitude: 1.0,
                frequency: 0.0,
                phase: 0.0,
                duration: 0.01,
            });
        }
        let pred = hist.predict();
        assert!(
            pred.periodicity > 0.9,
            "perfectly periodic should be ~1.0: {}",
            pred.periodicity
        );

        // Familiarity at 10 observations should be 1.0
        assert!((pred.familiarity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn spline_time_mode_roundtrip() {
        for t in 0u8..=3 {
            let mode = TimeMode::from_bits(t);
            assert_eq!(mode.as_bits(), t);
        }
        assert_eq!(TimeMode::from_bits(0), TimeMode::Timeless);
        assert_eq!(TimeMode::from_bits(3), TimeMode::Rhythmic);
    }

    #[test]
    fn spline_empty_history_predict() {
        let hist = TimeHistory::new();
        let pred = hist.predict();
        assert!((pred.familiarity - 0.0).abs() < 1e-5);
        assert!((pred.learning_rate - 1.0).abs() < 1e-5);
        assert!((pred.avg_amplitude - 0.0).abs() < 1e-5);
        assert!((pred.periodicity - 0.0).abs() < 1e-5);
    }

    #[test]
    fn spline_single_knot_amplitude() {
        let mut hist = TimeHistory::new();
        hist.push(SplineKnot {
            timestamp: 500,
            amplitude: 7.0,
            frequency: 0.0,
            phase: 0.0,
            duration: 0.1,
        });
        // Single knot → always returns that amplitude
        assert!((hist.amplitude_at(0) - 7.0).abs() < 1e-5);
        assert!((hist.amplitude_at(500) - 7.0).abs() < 1e-5);
        assert!((hist.amplitude_at(9999) - 7.0).abs() < 1e-5);
    }
}
