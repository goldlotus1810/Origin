//! # parametric — T x S Integration (FE.8)
//!
//! T provides parameters (amplitude, phase, frequency) to SDF primitives (S).
//! S x T = concrete shape with size, position, motion.
//!
//! 18 SDF primitives x T spline knots = infinite shapes from finite ingredients.

use crate::shape::sdf::{self, SdfKind, SdfParams, Vec3};
use homemath::sinf;
use olang::mol::spline::SplineKnot;

// ─────────────────────────────────────────────────────────────────────────────
// ParametricSdf — SDF with T parameters
// ─────────────────────────────────────────────────────────────────────────────

/// SDF with T parameters — T provides size/position/motion to SDF primitives.
#[derive(Debug, Clone, Copy)]
pub struct ParametricSdf {
    /// S value (SdfKind byte) — which SDF primitive.
    pub shape: u8,
    /// T.amplitude -> radius/size.
    pub amplitude: f32,
    /// T.phase -> rotation/position offset (Y axis).
    pub phase: f32,
    /// T.frequency -> oscillation/motion.
    pub frequency: f32,
}

impl ParametricSdf {
    /// Create from Molecule S value + SplineKnot T.
    pub fn from_s_and_t(s: u8, knot: &SplineKnot) -> Self {
        Self {
            shape: s,
            amplitude: knot.amplitude,
            phase: knot.phase,
            frequency: knot.frequency,
        }
    }

    /// Convert T parameters to SdfParams for the given shape.
    fn to_sdf_params(&self) -> SdfParams {
        let r = self.amplitude;
        SdfParams {
            r,
            r2: r * 0.3,
            h: r * 2.0,
            b: Vec3::new(r, r, r),
        }
    }

    /// Map shape byte (0-based, matching ShapeBase) to SdfKind (1-based).
    fn sdf_kind(&self) -> SdfKind {
        // ShapeBase is 0-based, SdfKind is 1-based (0x01..0x12).
        // Map: ShapeBase 0 (Sphere) -> SdfKind 0x01, etc.
        let byte = self.shape.wrapping_add(1);
        SdfKind::from_byte(byte).unwrap_or(SdfKind::Sphere)
    }

    /// Evaluate SDF at point `p` with T parameters applied.
    ///
    /// - `p`: point in 3D space.
    /// - `time`: elapsed time in seconds (for motion/oscillation).
    pub fn eval(&self, p: [f32; 3], time: f32) -> f32 {
        let r = self.amplitude;
        if r <= 0.0 {
            // Degenerate: no size → infinite distance
            return f32::MAX;
        }

        // Apply motion from T.frequency
        let motion_offset = if self.frequency > 0.0 {
            sinf(self.frequency * time * core::f32::consts::TAU) * r * 0.1
        } else {
            0.0
        };

        // Apply phase as Y-axis position offset (for stacking shapes)
        let offset_y = self.phase + motion_offset;
        let p_local = Vec3::new(p[0], p[1] - offset_y, p[2]);

        let kind = self.sdf_kind();
        let params = self.to_sdf_params();
        sdf::sdf(kind, p_local, &params)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CSG composition
// ─────────────────────────────────────────────────────────────────────────────

/// Compose multiple ParametricSdf via CSG union.
///
/// Returns minimum signed distance across all shapes.
pub fn sdf_union(shapes: &[ParametricSdf], p: [f32; 3], time: f32) -> f32 {
    shapes
        .iter()
        .map(|s| s.eval(p, time))
        .fold(f32::MAX, |a, b| if a < b { a } else { b })
}

/// Compose via smooth union for organic blending.
pub fn sdf_smooth_union(shapes: &[ParametricSdf], p: [f32; 3], time: f32, k: f32) -> f32 {
    shapes
        .iter()
        .map(|s| s.eval(p, time))
        .fold(f32::MAX, |a, b| sdf::smooth_union(a, b, k))
}

// ─────────────────────────────────────────────────────────────────────────────
// Examples
// ─────────────────────────────────────────────────────────────────────────────

/// Example: create snowman (3 spheres stacked via phase offset).
///
/// ```text
///      o    head   (r=1.5, phase=7.0)
///     O     middle (r=2.0, phase=4.0)
///    O      body   (r=3.0, phase=0.0)
/// ```
pub fn snowman() -> [ParametricSdf; 3] {
    [
        // Body: sphere at origin
        ParametricSdf {
            shape: 0,
            amplitude: 3.0,
            phase: 0.0,
            frequency: 0.0,
        },
        // Middle: sphere stacked above
        ParametricSdf {
            shape: 0,
            amplitude: 2.0,
            phase: 4.0,
            frequency: 0.0,
        },
        // Head: sphere at top
        ParametricSdf {
            shape: 0,
            amplitude: 1.5,
            phase: 7.0,
            frequency: 0.0,
        },
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parametric_sdf_eval_sphere() {
        // Sphere (shape=0) with radius 2.0 at origin
        let psdf = ParametricSdf {
            shape: 0,
            amplitude: 2.0,
            phase: 0.0,
            frequency: 0.0,
        };

        // Center → d = -r = -2.0
        let d_center = psdf.eval([0.0, 0.0, 0.0], 0.0);
        assert!(
            (d_center - (-2.0)).abs() < 1e-3,
            "center: expected -2.0, got {}",
            d_center
        );

        // Surface → d ≈ 0
        let d_surface = psdf.eval([2.0, 0.0, 0.0], 0.0);
        assert!(d_surface.abs() < 1e-3, "surface: expected ~0, got {}", d_surface);

        // Outside → d > 0
        let d_outside = psdf.eval([4.0, 0.0, 0.0], 0.0);
        assert!(d_outside > 0.0, "outside: expected >0, got {}", d_outside);
    }

    #[test]
    fn parametric_sdf_union_snowman() {
        let parts = snowman();

        // Inside body (origin) → negative distance
        let d_body = sdf_union(&parts, [0.0, 0.0, 0.0], 0.0);
        assert!(d_body < 0.0, "inside body: expected <0, got {}", d_body);

        // Inside middle (at phase=4.0) → negative
        let d_mid = sdf_union(&parts, [0.0, 4.0, 0.0], 0.0);
        assert!(d_mid < 0.0, "inside middle: expected <0, got {}", d_mid);

        // Inside head (at phase=7.0) → negative
        let d_head = sdf_union(&parts, [0.0, 7.0, 0.0], 0.0);
        assert!(d_head < 0.0, "inside head: expected <0, got {}", d_head);

        // Far outside → positive
        let d_far = sdf_union(&parts, [20.0, 20.0, 20.0], 0.0);
        assert!(d_far > 0.0, "far outside: expected >0, got {}", d_far);
    }

    #[test]
    fn parametric_t_cross_s_different_sizes() {
        // Same shape (sphere), different T → different sizes
        let small = ParametricSdf {
            shape: 0,
            amplitude: 1.0,
            phase: 0.0,
            frequency: 0.0,
        };
        let large = ParametricSdf {
            shape: 0,
            amplitude: 5.0,
            phase: 0.0,
            frequency: 0.0,
        };

        // At distance 2.0 from center:
        // small sphere (r=1.0): outside → d > 0
        // large sphere (r=5.0): inside → d < 0
        let p = [2.0, 0.0, 0.0];
        let d_small = small.eval(p, 0.0);
        let d_large = large.eval(p, 0.0);

        assert!(d_small > 0.0, "small sphere at r=2: expected >0, got {}", d_small);
        assert!(d_large < 0.0, "large sphere at r=2: expected <0, got {}", d_large);
    }

    #[test]
    fn parametric_from_s_and_t() {
        let knot = SplineKnot {
            timestamp: 1000,
            amplitude: 3.0,
            frequency: 1.0,
            phase: 2.0,
            duration: 0.5,
        };
        let psdf = ParametricSdf::from_s_and_t(0, &knot);
        assert_eq!(psdf.shape, 0);
        assert!((psdf.amplitude - 3.0).abs() < 1e-5);
        assert!((psdf.frequency - 1.0).abs() < 1e-5);
        assert!((psdf.phase - 2.0).abs() < 1e-5);
    }

    #[test]
    fn parametric_box_eval() {
        // Box (shape=1) with half-extent 2.0
        let psdf = ParametricSdf {
            shape: 1,
            amplitude: 2.0,
            phase: 0.0,
            frequency: 0.0,
        };
        // Center → inside
        let d = psdf.eval([0.0, 0.0, 0.0], 0.0);
        assert!(d < 0.0, "inside box: expected <0, got {}", d);

        // Outside
        let d = psdf.eval([5.0, 0.0, 0.0], 0.0);
        assert!(d > 0.0, "outside box: expected >0, got {}", d);
    }

    #[test]
    fn parametric_motion_oscillation() {
        // Sphere with frequency → position oscillates over time
        let psdf = ParametricSdf {
            shape: 0,
            amplitude: 1.0,
            phase: 0.0,
            frequency: 1.0, // 1 Hz oscillation
        };
        let d_t0 = psdf.eval([0.0, 0.0, 0.0], 0.0);
        let d_t1 = psdf.eval([0.0, 0.0, 0.0], 0.25); // quarter cycle

        // With frequency, distance at center should differ at different times
        // (because the sphere moves along Y due to motion offset)
        // At t=0, sin(0)=0 → no offset. At t=0.25, sin(pi/2)=1 → offset.
        assert!(
            (d_t0 - d_t1).abs() > 1e-5,
            "motion should change distance: t0={}, t1={}",
            d_t0,
            d_t1
        );
    }
}
