//! # t17_formula_engine — End-to-end: P_weight → Formula → Behavior
//!
//! FE.7: Reading P_weight reconstructs the formula and determines behavior.
//! Tests the full chain: Molecule bits → dimension extraction → formula dispatch → behavior.

use olang::mol::formula::{ArousalState, FormulaState, RelationOp, ValenceState};
use olang::mol::molecular::Molecule;
use olang::mol::spline::{SplineKnot, TimeHistory};
use vsdf::render::parametric::{self, ParametricSdf};

// ─────────────────────────────────────────────────────────────────────────────
// 1. R dispatch round-trip: every R value (0-15) → RelationOp variant
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t17_r_dispatch_roundtrip() {
    // Expected mapping: R index → RelationOp variant
    let expected = [
        RelationOp::Identity,    // 0
        RelationOp::Member,      // 1
        RelationOp::Subset,      // 2
        RelationOp::Equality,    // 3
        RelationOp::Order,       // 4
        RelationOp::Arithmetic,  // 5
        RelationOp::Logical,     // 6
        RelationOp::SetOp,       // 7
        RelationOp::Compose,     // 8
        RelationOp::Causes,      // 9
        RelationOp::Approximate, // 10
        RelationOp::Orthogonal,  // 11
        RelationOp::Aggregate,   // 12
        RelationOp::Directional, // 13
        RelationOp::Bracket,     // 14
        RelationOp::Inverse,     // 15
    ];

    for r in 0u8..16 {
        // Create molecule with R in the R field: [S:4][R:4][V:3][A:3][T:2]
        // R occupies bits 8-11, so shift r into that position.
        let bits = (r as u16) << 8;
        let mol = Molecule::from_u16(bits);

        // Verify extraction
        assert_eq!(
            mol.relation(),
            r,
            "Molecule::relation() should extract R={} from bits {:#06x}",
            r,
            bits
        );

        // Verify dispatch
        let rop = RelationOp::from_r(mol.relation());
        assert_eq!(
            rop, expected[r as usize],
            "R={} should map to {:?}, got {:?}",
            r, expected[r as usize], rop
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. V approach tendency signs
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t17_v_approach_tendency_signs() {
    // V=0-2 → negative force (repel)
    for v in 0..=2u8 {
        let state = ValenceState::from_v(v);
        assert!(
            state.approach_tendency() < 0.0,
            "V={} should repel (negative tendency), got {}",
            v,
            state.approach_tendency()
        );
    }

    // V=3-4 → near zero (neutral)
    for v in 3..=4u8 {
        let state = ValenceState::from_v(v);
        assert!(
            state.approach_tendency().abs() < 0.01,
            "V={} should be near zero (neutral), got {}",
            v,
            state.approach_tendency()
        );
    }

    // V=5-7 → positive force (attract)
    for v in 5..=7u8 {
        let state = ValenceState::from_v(v);
        assert!(
            state.approach_tendency() > 0.0,
            "V={} should attract (positive tendency), got {}",
            v,
            state.approach_tendency()
        );
    }

    // Monotonicity: tendency should increase with V
    let mut prev = ValenceState::from_v(0).approach_tendency();
    for v in 1..=7u8 {
        let curr = ValenceState::from_v(v).approach_tendency();
        assert!(
            curr >= prev,
            "V={}: tendency {} should be >= prev {}",
            v,
            curr,
            prev
        );
        prev = curr;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. A urgency thresholds
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t17_a_urgency_thresholds() {
    // A=0-3 → low urgency (< 0.3)
    for a in 0..=3u8 {
        let state = ArousalState::from_a(a);
        assert!(
            state.urgency() < 0.3,
            "A={} should have low urgency (< 0.3), got {}",
            a,
            state.urgency()
        );
    }

    // A=7 → urgency > 0.618 (phi^-1, the golden ratio threshold)
    let urgent = ArousalState::from_a(7);
    assert!(
        urgent.urgency() > 0.618,
        "A=7 should have urgency > 0.618 (phi^-1), got {}",
        urgent.urgency()
    );

    // Monotonicity: urgency should increase with A
    let mut prev = ArousalState::from_a(0).urgency();
    for a in 1..=7u8 {
        let curr = ArousalState::from_a(a).urgency();
        assert!(
            curr >= prev,
            "A={}: urgency {} should be >= prev {}",
            a,
            curr,
            prev
        );
        prev = curr;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. T spline accumulation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t17_t_spline_accumulation() {
    let mut history = TimeHistory::new();
    history.push(TimeHistory::observe_text("hello", 1000));
    history.push(TimeHistory::observe_text("hello again", 2000));

    let pred = history.predict();
    assert!(
        pred.familiarity > 0.0,
        "2 observations should give familiarity > 0, got {}",
        pred.familiarity
    );
    assert!(
        pred.learning_rate > 0.0,
        "learning_rate should be positive, got {}",
        pred.learning_rate
    );

    // Familiarity should be 2/10 = 0.2
    assert!(
        (pred.familiarity - 0.2).abs() < 1e-5,
        "familiarity should be 0.2, got {}",
        pred.familiarity
    );

    // With only 2 knots (< 3), learning_rate stays at 1.0
    assert!(
        (pred.learning_rate - 1.0).abs() < 1e-5,
        "learning_rate with 2 knots should be 1.0, got {}",
        pred.learning_rate
    );

    // avg_amplitude should be positive (text lengths > 0)
    assert!(
        pred.avg_amplitude > 0.0,
        "avg_amplitude should be positive, got {}",
        pred.avg_amplitude
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. T x S snowman — parametric SDF integration
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t17_txs_snowman() {
    let shapes = parametric::snowman();
    assert_eq!(shapes.len(), 3, "snowman should have 3 parts");

    // Center of body (0,0,0) should be inside (negative SDF)
    let d = parametric::sdf_union(&shapes, [0.0, 0.0, 0.0], 0.0);
    assert!(d < 0.0, "inside body: expected negative SDF, got {}", d);

    // Far away should be outside (positive SDF)
    let d = parametric::sdf_union(&shapes, [100.0, 0.0, 0.0], 0.0);
    assert!(d > 0.0, "outside: expected positive SDF, got {}", d);

    // Each part is a sphere (shape=0)
    for (i, part) in shapes.iter().enumerate() {
        assert_eq!(part.shape, 0, "part {} should be sphere (shape=0)", i);
    }

    // Middle part center (phase=4.0) should be inside
    let d_mid = parametric::sdf_union(&shapes, [0.0, 4.0, 0.0], 0.0);
    assert!(
        d_mid < 0.0,
        "inside middle: expected negative SDF, got {}",
        d_mid
    );

    // Head center (phase=7.0) should be inside
    let d_head = parametric::sdf_union(&shapes, [0.0, 7.0, 0.0], 0.0);
    assert!(
        d_head < 0.0,
        "inside head: expected negative SDF, got {}",
        d_head
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. FormulaState from real UCD codepoint
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t17_formula_state_from_ucd_codepoint() {
    // Encode fire emoji U+1F525 → get FormulaState → verify behavior
    let fire = olang::mol::encoder::encode_codepoint(0x1F525);
    if !fire.is_empty() {
        let mol = Molecule::from_u16(fire.0[0]);
        let fs = FormulaState::from_molecule(&mol);

        // Fire should have *some* arousal (urgency > 0)
        assert!(
            fs.urgency() > 0.0,
            "fire emoji should have some urgency, got {}",
            fs.urgency()
        );

        // FormulaState fields should be consistent with molecule dimensions
        assert_eq!(
            fs.relation,
            RelationOp::from_r(mol.relation()),
            "relation should match"
        );
        assert_eq!(
            fs.valence,
            ValenceState::from_v(mol.valence()),
            "valence should match"
        );
        assert_eq!(
            fs.arousal,
            ArousalState::from_a(mol.arousal()),
            "arousal should match"
        );
    }

    // Also test a basic math symbol: + U+002B
    let plus = olang::mol::encoder::encode_codepoint(0x002B);
    if !plus.is_empty() {
        let mol = Molecule::from_u16(plus.0[0]);
        let fs = FormulaState::from_molecule(&mol);
        // Plus is a math operator, should have some relation type
        // (just verify it doesn't panic and produces valid state)
        let _ = fs.approach_tendency();
        let _ = fs.urgency();
        let _ = fs.needs_urgent();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Formula determines compose behavior — different R → different compose
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t17_formula_determines_compose_behavior() {
    let rop_arith = RelationOp::from_r(5); // Arithmetic
    let rop_logical = RelationOp::from_r(6); // Logical

    assert_eq!(rop_arith, RelationOp::Arithmetic);
    assert_eq!(rop_logical, RelationOp::Logical);

    // Arithmetic compose differs from Logical compose
    let a: u16 = 0x1234;
    let b: u16 = 0x5678;
    let result_arith = rop_arith.compose(a, b);
    let result_logical = rop_logical.compose(a, b);

    assert_ne!(
        result_arith, result_logical,
        "Arithmetic compose ({:#06x}) should differ from Logical compose ({:#06x}) for a={:#06x}, b={:#06x}",
        result_arith, result_logical, a, b
    );

    // Verify specific semantics:
    // Logical = AND → a & b
    assert_eq!(
        result_logical,
        a & b,
        "Logical compose should be AND: {:#06x} & {:#06x} = {:#06x}",
        a,
        b,
        a & b
    );

    // Identity should return a
    assert_eq!(
        RelationOp::Identity.compose(a, b),
        a,
        "Identity compose should return a"
    );

    // SetOp = OR → a | b
    assert_eq!(
        RelationOp::SetOp.compose(a, b),
        a | b,
        "SetOp compose should be OR"
    );

    // Orthogonal = XOR → a ^ b
    assert_eq!(
        RelationOp::Orthogonal.compose(a, b),
        a ^ b,
        "Orthogonal compose should be XOR"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Full pipeline: Molecule → FormulaState → behavior predicates
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t17_full_pipeline_molecule_to_behavior() {
    // Construct molecule with known dimensions:
    // S=0, R=9 (Causes), V=7 (VeryDeepWell → strong attract), A=7 (Supercritical → urgent), T=3
    let bits: u16 = (0u16 << 12) | (9u16 << 8) | (7u16 << 5) | (7u16 << 2) | 3u16;
    let mol = Molecule::from_u16(bits);

    assert_eq!(mol.shape(), 0);
    assert_eq!(mol.relation(), 9);
    assert_eq!(mol.valence(), 7);
    assert_eq!(mol.arousal(), 7);
    assert_eq!(mol.time(), 3);

    let fs = FormulaState::from_molecule(&mol);

    // R=9 → Causes
    assert_eq!(fs.relation, RelationOp::Causes);

    // V=7 → strong approach
    assert!(
        fs.approach_tendency() > 0.5,
        "V=7 should give strong approach tendency, got {}",
        fs.approach_tendency()
    );

    // A=7 → urgent
    assert!(
        fs.needs_urgent(),
        "A=7 should be urgent (> 0.618), urgency = {}",
        fs.urgency()
    );

    // Contrast: calm molecule (A=0, V=3 neutral)
    let calm_bits: u16 = (0u16 << 12) | (0u16 << 8) | (3u16 << 5) | (0u16 << 2) | 0u16;
    let calm_mol = Molecule::from_u16(calm_bits);
    let calm_fs = FormulaState::from_molecule(&calm_mol);

    assert!(!calm_fs.needs_urgent(), "A=0 should not be urgent");
    assert!(
        calm_fs.approach_tendency().abs() < 0.01,
        "V=3 should be neutral"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. ParametricSdf from SplineKnot — T provides parameters to S
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn t17_parametric_sdf_from_spline_knot() {
    let knot = SplineKnot {
        timestamp: 5000,
        amplitude: 2.5,
        frequency: 0.5,
        phase: 1.0,
        duration: 0.3,
    };

    // S=0 (sphere) + T knot → ParametricSdf
    let psdf = ParametricSdf::from_s_and_t(0, &knot);
    assert_eq!(psdf.shape, 0);
    assert!((psdf.amplitude - 2.5).abs() < 1e-5);
    assert!((psdf.frequency - 0.5).abs() < 1e-5);
    assert!((psdf.phase - 1.0).abs() < 1e-5);

    // Evaluate: center (adjusted for phase offset) should be inside
    let d = psdf.eval([0.0, 1.0, 0.0], 0.0);
    assert!(
        d < 0.0,
        "point at phase offset should be inside sphere, got {}",
        d
    );

    // Far away should be outside
    let d_far = psdf.eval([50.0, 50.0, 50.0], 0.0);
    assert!(d_far > 0.0, "far point should be outside, got {}", d_far);
}
