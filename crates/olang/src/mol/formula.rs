//! # formula — Formula dispatch for R, V, A dimensions
//!
//! Read P_weight values → reconstruct physics formula and behavior.
//! Each dimension index is an INDEX into a formula table:
//!   R (0-15) → RelationOp (category theory operations)
//!   V (0-7)  → ValenceState (potential energy physics)
//!   A (0-7)  → ArousalState (damped oscillator physics)
//!
//! `no_std` compatible — uses `core` only, `f32` for memory efficiency.

use super::molecular::Molecule;
use homemath::sqrtf;

// ─────────────────────────────────────────────────────────────────────────────
// FE.1 — RelationOp dispatch (R: 0-15)
// ─────────────────────────────────────────────────────────────────────────────

/// 16 relation types from Category Theory.
///
/// Each variant maps to a specific mathematical structure.
/// R index in P_weight [S:4][R:4][V:3][A:3][T:2] → 4 bits (0-15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RelationOp {
    /// R=0: a → a (identity morphism)
    Identity = 0,
    /// R=1: a ∈ b (set membership)
    Member = 1,
    /// R=2: a ⊂ b (inclusion / subset)
    Subset = 2,
    /// R=3: a ≡ b (equivalence: reflexive + symmetric + transitive)
    Equality = 3,
    /// R=4: a ≤ b (partial order)
    Order = 4,
    /// R=5: ring (ℤ,+,×) — arithmetic operations
    Arithmetic = 5,
    /// R=6: Boolean (∧,∨,¬) — logical connectives
    Logical = 6,
    /// R=7: A∪B, A∩B — set operations
    SetOp = 7,
    /// R=8: g∘f — function composition
    Compose = 8,
    /// R=9: a → b — causality
    Causes = 9,
    /// R=10: a ≈ b (d < ε) — approximate equality
    Approximate = 10,
    /// R=11: a ⊥ b — orthogonality
    Orthogonal = 11,
    /// R=12: Σ, ∫ — summation / aggregation
    Aggregate = 12,
    /// R=13: a → b — vector / directional arrow
    Directional = 13,
    /// R=14: (a, b) — grouping / bracket
    Bracket = 14,
    /// R=15: a⁻¹ — inverse
    Inverse = 15,
}

impl RelationOp {
    /// Decode R index (0-15) from P_weight into RelationOp.
    pub fn from_r(r: u8) -> Self {
        match r {
            0 => Self::Identity,
            1 => Self::Member,
            2 => Self::Subset,
            3 => Self::Equality,
            4 => Self::Order,
            5 => Self::Arithmetic,
            6 => Self::Logical,
            7 => Self::SetOp,
            8 => Self::Compose,
            9 => Self::Causes,
            10 => Self::Approximate,
            11 => Self::Orthogonal,
            12 => Self::Aggregate,
            13 => Self::Directional,
            14 => Self::Bracket,
            15 => Self::Inverse,
            _ => Self::Identity, // fallback
        }
    }

    /// Apply relation: compose two P_weight values under this operation.
    ///
    /// Returns a new packed u16 representing the composed result.
    /// Each RelationOp defines its own composition semantics.
    pub fn compose(&self, a: u16, b: u16) -> u16 {
        match self {
            // Identity: pass through a
            Self::Identity => a,

            // Member/Subset: result inherits container (b)
            Self::Member | Self::Subset => b,

            // Equality: if equal return a, else XOR blend
            Self::Equality => {
                if a == b { a } else { a ^ b }
            }

            // Order: return the larger value
            Self::Order => {
                if a >= b { a } else { b }
            }

            // Arithmetic (ring): add dimensions mod range
            Self::Arithmetic => {
                let sa = (a >> 12) & 0xF;
                let ra = (a >> 8) & 0xF;
                let va = (a >> 5) & 0x7;
                let aa = (a >> 2) & 0x7;
                let ta = a & 0x3;

                let sb = (b >> 12) & 0xF;
                let rb = (b >> 8) & 0xF;
                let vb = (b >> 5) & 0x7;
                let ab = (b >> 2) & 0x7;
                let tb = b & 0x3;

                let s = (sa + sb) & 0xF;
                let r = (ra + rb) & 0xF;
                let v = (va + vb) & 0x7;
                let ac = (aa + ab) & 0x7;
                let t = (ta + tb) & 0x3;

                (s << 12) | (r << 8) | (v << 5) | (ac << 2) | t
            }

            // Logical: AND of bits (conjunction)
            Self::Logical => a & b,

            // SetOp: OR of bits (union)
            Self::SetOp => a | b,

            // Compose: apply b then a (g∘f semantics — a wraps b)
            Self::Compose => {
                // Use a's R dimension, blend others from b
                let r_a = (a >> 8) & 0xF;
                (b & 0xF0FF) | (r_a << 8)
            }

            // Causes: result = b (effect inherits from cause)
            Self::Causes => b,

            // Approximate: average (midpoint blend)
            Self::Approximate => {
                // Blend each dimension: (a_dim + b_dim) / 2
                let sa = (a >> 12) & 0xF;
                let sb = (b >> 12) & 0xF;
                let ra = (a >> 8) & 0xF;
                let rb = (b >> 8) & 0xF;
                let va = (a >> 5) & 0x7;
                let vb = (b >> 5) & 0x7;
                let aa = (a >> 2) & 0x7;
                let ab = (b >> 2) & 0x7;
                let ta = a & 0x3;
                let tb = b & 0x3;

                let s = (sa + sb) / 2;
                let r = (ra + rb) / 2;
                let v = (va + vb) / 2;
                let ac = (aa + ab) / 2;
                let t = (ta + tb) / 2;

                (s << 12) | (r << 8) | (v << 5) | (ac << 2) | t
            }

            // Orthogonal: XOR (independent dimensions)
            Self::Orthogonal => a ^ b,

            // Aggregate: sum → clamp each dim to max
            Self::Aggregate => {
                let sa = (a >> 12) & 0xF;
                let sb = (b >> 12) & 0xF;
                let ra = (a >> 8) & 0xF;
                let rb = (b >> 8) & 0xF;
                let va = (a >> 5) & 0x7;
                let vb = (b >> 5) & 0x7;
                let aa = (a >> 2) & 0x7;
                let ab = (b >> 2) & 0x7;
                let ta = a & 0x3;
                let tb = b & 0x3;

                let s = core::cmp::min(sa + sb, 0xF);
                let r = core::cmp::min(ra + rb, 0xF);
                let v = core::cmp::min(va + vb, 0x7);
                let ac = core::cmp::min(aa + ab, 0x7);
                let t = core::cmp::min(ta + tb, 0x3);

                (s << 12) | (r << 8) | (v << 5) | (ac << 2) | t
            }

            // Directional: b (arrow target)
            Self::Directional => b,

            // Bracket: preserve a (grouping is transparent)
            Self::Bracket => a,

            // Inverse: bitwise NOT within valid range
            Self::Inverse => {
                let s = 0xF - ((a >> 12) & 0xF);
                let r = 0xF - ((a >> 8) & 0xF);
                let v = 0x7 - ((a >> 5) & 0x7);
                let ac = 0x7 - ((a >> 2) & 0x7);
                let t = 0x3 - (a & 0x3);
                (s << 12) | (r << 8) | (v << 5) | (ac << 2) | t
            }
        }
    }

    /// Whether the relation is symmetric: a R b ⟺ b R a.
    pub fn is_symmetric(&self) -> bool {
        matches!(
            self,
            Self::Identity
                | Self::Equality
                | Self::Arithmetic
                | Self::Logical
                | Self::SetOp
                | Self::Approximate
                | Self::Orthogonal
        )
    }

    /// Whether the relation is transitive: a R b ∧ b R c → a R c.
    pub fn is_transitive(&self) -> bool {
        matches!(
            self,
            Self::Identity
                | Self::Subset
                | Self::Equality
                | Self::Order
                | Self::Compose
                | Self::Causes
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FE.2 — ValenceState dispatch (V: 0-7)
// ─────────────────────────────────────────────────────────────────────────────

/// Potential energy landscape type.
///
/// Maps V index to a specific potential energy formula.
/// V index in P_weight: 3 bits (0-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ValenceKind {
    /// V=0: U >> 0, strong repulsion (hate/horror)
    /// Formula: U(r) = +k·q1·q2/r (Coulomb repulsion, same charge)
    HighBarrier = 0,
    /// V=1: U > 0, mild repulsion (annoying/difficult)
    /// Formula: U(x) = U0·exp(-x²/2σ²) (Gaussian barrier, low U0)
    LowBarrier = 1,
    /// V=2: U > 0, slight repulsion (adverse/ambiguous)
    /// Formula: U(x) = ε·exp(-x²/2σ²), ε small
    MildBarrier = 2,
    /// V=3: U ≈ 0, no force (neutral)
    /// Formula: U(x) = const → F(x) = -dU/dx = 0
    Flat = 3,
    /// V=4: U ≈ 0, slight attraction (neutral+)
    /// Formula: U(x) = const → F(x) = 0
    MildWell = 4,
    /// V=5: U < 0, mild attraction (pleasant/helpful)
    /// Formula: U(r) = -ε·(σ/r)^6 (Van der Waals, attractive part)
    ShallowWell = 5,
    /// V=6: U << 0, strong attraction (joy/love)
    /// Formula: U = -V0 + ½kx² (parabolic well, V0 >> kT)
    DeepWell = 6,
    /// V=7: U <<< 0, very strong attraction (ecstasy/bliss)
    /// Formula: U(x) = -V0·sech²(x/a) + V_barrier
    VeryDeepWell = 7,
}

/// Potential energy state derived from V dimension.
///
/// Encodes the physics of approach/avoidance behavior.
/// Force F = -dU/dx: positive = attract, negative = repel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValenceState {
    /// Type of potential landscape
    pub kind: ValenceKind,
    /// Potential energy U(x) value (dimensionless)
    /// Negative = well (attract), Positive = barrier (repel), Zero = flat
    pub potential: f32,
    /// Force F = -dU/dx
    /// Positive = attract (approach), Negative = repel (avoid)
    pub force: f32,
}

impl ValenceState {
    /// Decode V index (0-7) from P_weight into ValenceState.
    pub fn from_v(v: u8) -> Self {
        match v {
            0 => Self {
                kind: ValenceKind::HighBarrier,
                potential: 0.85,
                force: -0.9,
            },
            1 => Self {
                kind: ValenceKind::LowBarrier,
                potential: 0.4,
                force: -0.4,
            },
            2 => Self {
                kind: ValenceKind::MildBarrier,
                potential: 0.15,
                force: -0.15,
            },
            3 => Self {
                kind: ValenceKind::Flat,
                potential: 0.0,
                force: 0.0,
            },
            4 => Self {
                kind: ValenceKind::MildWell,
                potential: 0.0,
                force: 0.0,
            },
            5 => Self {
                kind: ValenceKind::ShallowWell,
                potential: -0.35,
                force: 0.35,
            },
            6 => Self {
                kind: ValenceKind::DeepWell,
                potential: -0.75,
                force: 0.8,
            },
            7 => Self {
                kind: ValenceKind::VeryDeepWell,
                potential: -0.95,
                force: 0.95,
            },
            _ => Self {
                kind: ValenceKind::Flat,
                potential: 0.0,
                force: 0.0,
            },
        }
    }

    /// Approach tendency: F > 0 means approach, F < 0 means avoid.
    pub fn approach_tendency(&self) -> f32 {
        self.force
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FE.3 — ArousalState dispatch (A: 0-7)
// ─────────────────────────────────────────────────────────────────────────────

/// Energy regime type for the damped oscillator model.
///
/// Maps A index to a specific oscillation/energy formula.
/// A index in P_weight: 3 bits (0-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ArousalKind {
    /// A=0: E=E₀, frozen, zero-point energy
    /// Formula: E₀ = ½·ℏ·ω₀
    GroundState = 0,
    /// A=1: S→S_max, entropy maximum, exhausted
    /// Formula: η = W/Q → 0 (Carnot efficiency → 0)
    HeatDeath = 1,
    /// A=2: γ >> ω₀, slow monotonic decay
    /// Formula: x(t) = (C₁+C₂·t)·exp(-γ·t), γ >> ω₀
    Overdamped = 2,
    /// A=3: ΔG=0, thermal equilibrium
    /// Formula: P(E) = exp(-E/kT)/Z (Boltzmann distribution)
    Equilibrium = 3,
    /// A=4: ΔG≈0, slight activity
    MildEquilibrium = 4,
    /// A=5: E > E_th, mild oscillation
    /// Formula: x(t) = A₀·exp(-γ·t)·cos(ω_d·t), γ small
    ExcitedLow = 5,
    /// A=6: E >> E_th, strong oscillation / resonance
    /// Formula: |X(ω)| = F₀/√((ω₀²−ω²)² + 4γ²ω²)
    ExcitedHigh = 6,
    /// A=7: E >>> E_th, positive feedback, explosion
    /// Formula: R(t) = R₀·exp(λ·t) (chain reaction)
    Supercritical = 7,
}

/// Energy/oscillation state derived from A dimension.
///
/// Models the system as a damped harmonic oscillator:
///   x'' + 2γx' + ω₀²x = F(t)/m
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArousalState {
    /// Type of energy regime
    pub kind: ArousalKind,
    /// Energy level E/E_threshold ratio (dimensionless, [0.0, 1.0])
    pub energy: f32,
    /// Damping coefficient γ
    /// γ > ω₀ → overdamped, γ < ω₀ → underdamped, γ = 0 → no damping
    pub damping: f32,
}

impl ArousalState {
    /// Decode A index (0-7) from P_weight into ArousalState.
    pub fn from_a(a: u8) -> Self {
        match a {
            0 => Self {
                kind: ArousalKind::GroundState,
                energy: 0.02,
                damping: 100.0,
            },
            1 => Self {
                kind: ArousalKind::HeatDeath,
                energy: 0.05,
                damping: 50.0,
            },
            2 => Self {
                kind: ArousalKind::Overdamped,
                energy: 0.08,
                damping: 30.0,
            },
            3 => Self {
                kind: ArousalKind::Equilibrium,
                energy: 0.2,
                damping: 3.0,
            },
            4 => Self {
                kind: ArousalKind::MildEquilibrium,
                energy: 0.5,
                damping: 1.0,
            },
            5 => Self {
                kind: ArousalKind::ExcitedLow,
                energy: 0.7,
                damping: 0.3,
            },
            6 => Self {
                kind: ArousalKind::ExcitedHigh,
                energy: 0.9,
                damping: 0.05,
            },
            7 => Self {
                kind: ArousalKind::Supercritical,
                energy: 0.98,
                damping: 0.0,
            },
            _ => Self {
                kind: ArousalKind::Equilibrium,
                energy: 0.5,
                damping: 1.0,
            },
        }
    }

    /// Urgency level: 0.0 = calm, 1.0 = urgent.
    ///
    /// urgency > 0.618 → needs_urgent (golden ratio threshold)
    /// urgency > 0.8  → trigger SecurityGate (crisis check)
    pub fn urgency(&self) -> f32 {
        match self.kind {
            ArousalKind::GroundState => 0.02,
            ArousalKind::HeatDeath => 0.05,
            ArousalKind::Overdamped => 0.1,
            ArousalKind::Equilibrium => 0.2,
            ArousalKind::MildEquilibrium => 0.4,
            ArousalKind::ExcitedLow => 0.6,
            ArousalKind::ExcitedHigh => 0.85,
            ArousalKind::Supercritical => 0.95,
        }
    }

    /// Natural oscillation frequency (Hz).
    ///
    /// ω₀ = 1.0 (normalized), actual freq depends on damping regime.
    /// Overdamped: no oscillation (0 Hz effective).
    /// Underdamped: ω_d = √(ω₀² − γ²).
    pub fn oscillation_freq(&self) -> f32 {
        let omega0: f32 = 1.0;
        let g = self.damping;
        if g >= omega0 {
            // Overdamped or critically damped: no oscillation
            0.0
        } else {
            // Underdamped: ω_d = √(ω₀² − γ²)
            sqrtf(omega0 * omega0 - g * g)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unified FormulaState
// ─────────────────────────────────────────────────────────────────────────────

/// Unified formula state extracted from a Molecule's R, V, A dimensions.
///
/// Combines relation dispatch, valence physics, and arousal physics
/// into a single queryable struct.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormulaState {
    /// Relation dispatch from R dimension (0-15).
    pub relation: RelationOp,
    /// Valence state from V dimension (0-7).
    pub valence: ValenceState,
    /// Arousal state from A dimension (0-7).
    pub arousal: ArousalState,
}

impl FormulaState {
    /// Extract FormulaState from a Molecule's P_weight bits.
    pub fn from_molecule(mol: &Molecule) -> Self {
        Self {
            relation: RelationOp::from_r(mol.relation()),
            valence: ValenceState::from_v(mol.valence()),
            arousal: ArousalState::from_a(mol.arousal()),
        }
    }

    /// Approach tendency from valence: >0 approach, <0 avoid.
    pub fn approach_tendency(&self) -> f32 {
        self.valence.approach_tendency()
    }

    /// Urgency from arousal: 0.0=calm, 1.0=urgent.
    pub fn urgency(&self) -> f32 {
        self.arousal.urgency()
    }

    /// Whether this state needs urgent response (golden ratio threshold).
    pub fn needs_urgent(&self) -> bool {
        self.arousal.urgency() > 0.618
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── FE.1: RelationOp tests ──────────────────────────────────────────

    #[test]
    fn formula_from_r_all_values() {
        assert_eq!(RelationOp::from_r(0), RelationOp::Identity);
        assert_eq!(RelationOp::from_r(1), RelationOp::Member);
        assert_eq!(RelationOp::from_r(2), RelationOp::Subset);
        assert_eq!(RelationOp::from_r(3), RelationOp::Equality);
        assert_eq!(RelationOp::from_r(4), RelationOp::Order);
        assert_eq!(RelationOp::from_r(5), RelationOp::Arithmetic);
        assert_eq!(RelationOp::from_r(6), RelationOp::Logical);
        assert_eq!(RelationOp::from_r(7), RelationOp::SetOp);
        assert_eq!(RelationOp::from_r(8), RelationOp::Compose);
        assert_eq!(RelationOp::from_r(9), RelationOp::Causes);
        assert_eq!(RelationOp::from_r(10), RelationOp::Approximate);
        assert_eq!(RelationOp::from_r(11), RelationOp::Orthogonal);
        assert_eq!(RelationOp::from_r(12), RelationOp::Aggregate);
        assert_eq!(RelationOp::from_r(13), RelationOp::Directional);
        assert_eq!(RelationOp::from_r(14), RelationOp::Bracket);
        assert_eq!(RelationOp::from_r(15), RelationOp::Inverse);
        // Out of range → fallback to Identity
        assert_eq!(RelationOp::from_r(16), RelationOp::Identity);
        assert_eq!(RelationOp::from_r(255), RelationOp::Identity);
    }

    #[test]
    fn formula_compose_identity() {
        let a: u16 = 0b1010_0011_101_010_01; // S=10, R=3, V=5, A=2, T=1
        let b: u16 = 0b0101_1100_010_111_10; // S=5, R=12, V=2, A=7, T=2
        // Identity returns a
        assert_eq!(RelationOp::Identity.compose(a, b), a);
    }

    #[test]
    fn formula_compose_order_returns_larger() {
        let a: u16 = 100;
        let b: u16 = 200;
        assert_eq!(RelationOp::Order.compose(a, b), b);
        assert_eq!(RelationOp::Order.compose(b, a), b);
        assert_eq!(RelationOp::Order.compose(a, a), a);
    }

    #[test]
    fn formula_compose_logical_and() {
        let a: u16 = 0xFF00;
        let b: u16 = 0x0FF0;
        assert_eq!(RelationOp::Logical.compose(a, b), 0x0F00);
    }

    #[test]
    fn formula_compose_setop_union() {
        let a: u16 = 0xFF00;
        let b: u16 = 0x00FF;
        assert_eq!(RelationOp::SetOp.compose(a, b), 0xFFFF);
    }

    #[test]
    fn formula_compose_equality_same() {
        let a: u16 = 42;
        assert_eq!(RelationOp::Equality.compose(a, a), a);
    }

    #[test]
    fn formula_compose_equality_different() {
        let a: u16 = 0b1010;
        let b: u16 = 0b0110;
        assert_eq!(RelationOp::Equality.compose(a, b), a ^ b);
    }

    #[test]
    fn formula_compose_inverse() {
        // S=15, R=15, V=7, A=7, T=3 → all max
        let all_max: u16 = (0xF << 12) | (0xF << 8) | (0x7 << 5) | (0x7 << 2) | 0x3;
        let result = RelationOp::Inverse.compose(all_max, 0);
        // Inverse of all-max should be all-zero
        assert_eq!(result, 0);

        // Inverse of all-zero should be all-max
        let result2 = RelationOp::Inverse.compose(0, 0);
        assert_eq!(result2, all_max);
    }

    #[test]
    fn formula_compose_arithmetic_wraps() {
        // S=8, R=8, V=4, A=4, T=2
        let a: u16 = (8 << 12) | (8 << 8) | (4 << 5) | (4 << 2) | 2;
        let b: u16 = (8 << 12) | (8 << 8) | (4 << 5) | (4 << 2) | 2;
        let result = RelationOp::Arithmetic.compose(a, b);
        // S: (8+8)&0xF = 0, R: (8+8)&0xF = 0, V: (4+4)&0x7 = 0,
        // A: (4+4)&0x7 = 0, T: (2+2)&0x3 = 0
        assert_eq!(result, 0);
    }

    #[test]
    fn formula_symmetric_relations() {
        assert!(RelationOp::Identity.is_symmetric());
        assert!(RelationOp::Equality.is_symmetric());
        assert!(RelationOp::Approximate.is_symmetric());
        assert!(RelationOp::Orthogonal.is_symmetric());
        // Non-symmetric
        assert!(!RelationOp::Member.is_symmetric());
        assert!(!RelationOp::Subset.is_symmetric());
        assert!(!RelationOp::Causes.is_symmetric());
        assert!(!RelationOp::Inverse.is_symmetric());
    }

    #[test]
    fn formula_transitive_relations() {
        assert!(RelationOp::Identity.is_transitive());
        assert!(RelationOp::Subset.is_transitive());
        assert!(RelationOp::Equality.is_transitive());
        assert!(RelationOp::Order.is_transitive());
        assert!(RelationOp::Compose.is_transitive());
        assert!(RelationOp::Causes.is_transitive());
        // Non-transitive
        assert!(!RelationOp::Member.is_transitive());
        assert!(!RelationOp::Approximate.is_transitive());
        assert!(!RelationOp::Orthogonal.is_transitive());
    }

    // ── FE.2: ValenceState tests ────────────────────────────────────────

    #[test]
    fn formula_from_v_all_values() {
        assert_eq!(ValenceState::from_v(0).kind, ValenceKind::HighBarrier);
        assert_eq!(ValenceState::from_v(1).kind, ValenceKind::LowBarrier);
        assert_eq!(ValenceState::from_v(2).kind, ValenceKind::MildBarrier);
        assert_eq!(ValenceState::from_v(3).kind, ValenceKind::Flat);
        assert_eq!(ValenceState::from_v(4).kind, ValenceKind::MildWell);
        assert_eq!(ValenceState::from_v(5).kind, ValenceKind::ShallowWell);
        assert_eq!(ValenceState::from_v(6).kind, ValenceKind::DeepWell);
        assert_eq!(ValenceState::from_v(7).kind, ValenceKind::VeryDeepWell);
        // Out of range → fallback to Flat
        assert_eq!(ValenceState::from_v(8).kind, ValenceKind::Flat);
        assert_eq!(ValenceState::from_v(255).kind, ValenceKind::Flat);
    }

    #[test]
    fn formula_valence_approach_signs() {
        // Barriers: negative force (repel/avoid)
        assert!(ValenceState::from_v(0).approach_tendency() < 0.0);
        assert!(ValenceState::from_v(1).approach_tendency() < 0.0);
        assert!(ValenceState::from_v(2).approach_tendency() < 0.0);

        // Flat: zero force
        assert_eq!(ValenceState::from_v(3).approach_tendency(), 0.0);
        assert_eq!(ValenceState::from_v(4).approach_tendency(), 0.0);

        // Wells: positive force (attract/approach)
        assert!(ValenceState::from_v(5).approach_tendency() > 0.0);
        assert!(ValenceState::from_v(6).approach_tendency() > 0.0);
        assert!(ValenceState::from_v(7).approach_tendency() > 0.0);
    }

    #[test]
    fn formula_valence_monotonic_potential() {
        // Potential should decrease from V=0 (high barrier) to V=7 (deep well)
        let potentials: [f32; 8] = core::array::from_fn(|i| {
            ValenceState::from_v(i as u8).potential
        });
        for i in 0..7 {
            assert!(
                potentials[i] >= potentials[i + 1],
                "potential[{}]={} should >= potential[{}]={}",
                i, potentials[i], i + 1, potentials[i + 1]
            );
        }
    }

    #[test]
    fn formula_valence_force_magnitude_increases() {
        // |force| should increase away from center (V=3,4)
        let f0 = ValenceState::from_v(0).force.abs();
        let f1 = ValenceState::from_v(1).force.abs();
        let f3 = ValenceState::from_v(3).force.abs();
        let f6 = ValenceState::from_v(6).force.abs();
        let f7 = ValenceState::from_v(7).force.abs();

        assert!(f0 > f1, "|f0|={} should > |f1|={}", f0, f1);
        assert!(f1 > f3, "|f1|={} should > |f3|={}", f1, f3);
        assert!(f7 > f6, "|f7|={} should > |f6|={}", f7, f6);
        assert_eq!(f3, 0.0);
    }

    // ── FE.3: ArousalState tests ────────────────────────────────────────

    #[test]
    fn formula_from_a_all_values() {
        assert_eq!(ArousalState::from_a(0).kind, ArousalKind::GroundState);
        assert_eq!(ArousalState::from_a(1).kind, ArousalKind::HeatDeath);
        assert_eq!(ArousalState::from_a(2).kind, ArousalKind::Overdamped);
        assert_eq!(ArousalState::from_a(3).kind, ArousalKind::Equilibrium);
        assert_eq!(ArousalState::from_a(4).kind, ArousalKind::MildEquilibrium);
        assert_eq!(ArousalState::from_a(5).kind, ArousalKind::ExcitedLow);
        assert_eq!(ArousalState::from_a(6).kind, ArousalKind::ExcitedHigh);
        assert_eq!(ArousalState::from_a(7).kind, ArousalKind::Supercritical);
        // Out of range → fallback to Equilibrium
        assert_eq!(ArousalState::from_a(8).kind, ArousalKind::Equilibrium);
        assert_eq!(ArousalState::from_a(255).kind, ArousalKind::Equilibrium);
    }

    #[test]
    fn formula_arousal_urgency_monotonic() {
        // Urgency should increase with A index
        let urgencies: [f32; 8] = core::array::from_fn(|i| {
            ArousalState::from_a(i as u8).urgency()
        });
        for i in 0..7 {
            assert!(
                urgencies[i] <= urgencies[i + 1],
                "urgency[{}]={} should <= urgency[{}]={}",
                i, urgencies[i], i + 1, urgencies[i + 1]
            );
        }
    }

    #[test]
    fn formula_arousal_urgency_thresholds() {
        // A=0..2: low urgency (< 0.2)
        assert!(ArousalState::from_a(0).urgency() < 0.2);
        assert!(ArousalState::from_a(1).urgency() < 0.2);
        assert!(ArousalState::from_a(2).urgency() < 0.2);

        // A=3,4: moderate (0.2..0.5)
        let u3 = ArousalState::from_a(3).urgency();
        let u4 = ArousalState::from_a(4).urgency();
        assert!(u3 >= 0.2 && u3 <= 0.5);
        assert!(u4 >= 0.2 && u4 <= 0.5);

        // A=6,7: high urgency (> 0.618, needs_urgent)
        assert!(ArousalState::from_a(6).urgency() > 0.618);
        assert!(ArousalState::from_a(7).urgency() > 0.618);
    }

    #[test]
    fn formula_arousal_energy_monotonic() {
        // Energy should increase with A index
        let energies: [f32; 8] = core::array::from_fn(|i| {
            ArousalState::from_a(i as u8).energy
        });
        for i in 0..7 {
            assert!(
                energies[i] <= energies[i + 1],
                "energy[{}]={} should <= energy[{}]={}",
                i, energies[i], i + 1, energies[i + 1]
            );
        }
    }

    #[test]
    fn formula_arousal_oscillation_freq() {
        // Heavily damped → 0 Hz
        assert_eq!(ArousalState::from_a(0).oscillation_freq(), 0.0);
        assert_eq!(ArousalState::from_a(1).oscillation_freq(), 0.0);
        assert_eq!(ArousalState::from_a(2).oscillation_freq(), 0.0);
        assert_eq!(ArousalState::from_a(3).oscillation_freq(), 0.0);
        assert_eq!(ArousalState::from_a(4).oscillation_freq(), 0.0);

        // Underdamped → positive frequency
        assert!(ArousalState::from_a(5).oscillation_freq() > 0.0);
        assert!(ArousalState::from_a(6).oscillation_freq() > 0.0);

        // Supercritical: damping=0 → ω_d = ω₀ = 1.0
        let freq7 = ArousalState::from_a(7).oscillation_freq();
        assert!((freq7 - 1.0).abs() < 0.001);
    }

    // ── Unified FormulaState tests ──────────────────────────────────────

    #[test]
    fn formula_state_from_molecule() {
        // Pack: S=5, R=9(Causes), V=6(DeepWell), A=7(Supercritical), T=1
        // Note: pack() quantizes from u8 via >>4/>>5/>>6
        // To get R=9 in 4-bit: need input where input>>4 = 9, so input = 9<<4 = 144
        // To get V=6 in 3-bit: need input where input>>5 = 6, so input = 6<<5 = 192
        // To get A=7 in 3-bit: need input where input>>5 = 7, so input = 7<<5 = 224
        let mol = Molecule::pack(5 << 4, 9 << 4, 6 << 5, 7 << 5, 1 << 6);
        let state = FormulaState::from_molecule(&mol);

        assert_eq!(state.relation, RelationOp::Causes);
        assert_eq!(state.valence.kind, ValenceKind::DeepWell);
        assert_eq!(state.arousal.kind, ArousalKind::Supercritical);
    }

    #[test]
    fn formula_state_needs_urgent() {
        // A=7 (Supercritical) → urgency=0.95 > 0.618
        let mol = Molecule::pack(0, 0, 0, 7 << 5, 0);
        let state = FormulaState::from_molecule(&mol);
        assert!(state.needs_urgent());

        // A=0 (GroundState) → urgency=0.02 < 0.618
        let mol_calm = Molecule::pack(0, 0, 0, 0, 0);
        let state_calm = FormulaState::from_molecule(&mol_calm);
        assert!(!state_calm.needs_urgent());
    }

    #[test]
    fn formula_state_approach_tendency() {
        // V=6 (DeepWell) → force=0.8 > 0 (approach)
        let mol = Molecule::pack(0, 0, 6 << 5, 0, 0);
        let state = FormulaState::from_molecule(&mol);
        assert!(state.approach_tendency() > 0.0);

        // V=0 (HighBarrier) → force=-0.9 < 0 (avoid)
        let mol_repel = Molecule::pack(0, 0, 0, 0, 0);
        let state_repel = FormulaState::from_molecule(&mol_repel);
        assert!(state_repel.approach_tendency() < 0.0);
    }
}
