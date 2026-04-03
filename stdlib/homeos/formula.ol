// stdlib/homeos/formula.ol — Formula Engine (FE.1-3)
// Ported from Rust: crates/olang/src/mol/formula.rs (848 LOC)
// P_weight values = indices into formula tables, NOT static numbers.
//
// R (0-15) → RelationOp (category theory)
// V (0-7)  → ValenceState (potential energy physics)
// A (0-7)  → ArousalState (damped harmonic oscillator)

// ═══ FE.1: RelationOp (R: 0-15) ═══
// 16 relation types from Category Theory.
// R=0: Identity, R=1: Member, R=2: Subset, R=3: Equality,
// R=4: Order, R=5: Arithmetic, R=6: Logical, R=7: SetOp,
// R=8: Compose, R=9: Causes, R=10: Approximate, R=11: Orthogonal,
// R=12: Aggregate, R=13: Directional, R=14: Bracket, R=15: Inverse

pub fn relation_name(r) {
  if r == 0 { return "identity"; }
  if r == 1 { return "member"; }
  if r == 2 { return "subset"; }
  if r == 3 { return "equality"; }
  if r == 4 { return "order"; }
  if r == 5 { return "arithmetic"; }
  if r == 6 { return "logical"; }
  if r == 7 { return "setop"; }
  if r == 8 { return "compose"; }
  if r == 9 { return "causes"; }
  if r == 10 { return "approximate"; }
  if r == 11 { return "orthogonal"; }
  if r == 12 { return "aggregate"; }
  if r == 13 { return "directional"; }
  if r == 14 { return "bracket"; }
  if r == 15 { return "inverse"; }
  return "identity";
}

// Compose two P_weight values under relation R
pub fn relation_compose(r, a, b) {
  if r == 0 { return a; }                          // Identity: pass through
  if r == 1 { return b; }                          // Member: inherit container
  if r == 2 { return b; }                          // Subset: inherit container
  if r == 3 { if a == b { return a; } return a; }  // Equality: keep a
  if r == 4 { if a > b { return a; } return b; }   // Order: larger wins
  if r == 5 {                                       // Arithmetic: modular add dims
    let sa = _kt_mol_s(a); let sb = _kt_mol_s(b);
    let ra = _kt_mol_r(a); let rb = _kt_mol_r(b);
    let va = _kt_mol_v(a); let vb = _kt_mol_v(b);
    let aa = _kt_mol_a(a); let ab = _kt_mol_a(b);
    let ta = _kt_mol_t(a); let tb = _kt_mol_t(b);
    return _kt_pack((sa + sb) % 16, (ra + rb) % 16, (va + vb) % 8, (aa + ab) % 8, (ta + tb) % 4);
  }
  if r == 6 { return a; }                          // Logical: keep a (AND-like)
  if r == 7 { return a; }                          // SetOp: keep a (union-like)
  if r == 8 { return a; }                          // Compose: g∘f → f first
  if r == 9 { return b; }                          // Causes: effect (b) wins
  if r == 10 { return a; }                         // Approximate: keep a
  if r == 11 { return 0; }                         // Orthogonal: zero (⊥)
  if r == 12 {                                     // Aggregate: sum dims
    let sa = _kt_mol_s(a); let sb = _kt_mol_s(b);
    let s = sa + sb; if s > 15 { s = 15; }
    return _kt_pack(s, _kt_mol_r(a), _kt_mol_v(a), _kt_mol_a(a), _kt_mol_t(a));
  }
  if r == 13 { return b; }                         // Directional: target (b) wins
  if r == 14 { return a; }                         // Bracket: first element
  if r == 15 {                                     // Inverse: invert dims
    return _kt_pack(15 - _kt_mol_s(a), 15 - _kt_mol_r(a), 7 - _kt_mol_v(a), 7 - _kt_mol_a(a), 3 - _kt_mol_t(a));
  }
  return a;
}

pub fn relation_is_symmetric(r) {
  // Symmetric: Equality, Approximate, Orthogonal
  if r == 3 { return 1; }
  if r == 10 { return 1; }
  if r == 11 { return 1; }
  return 0;
}

pub fn relation_is_transitive(r) {
  // Transitive: Subset, Equality, Order, Causes
  if r == 2 { return 1; }
  if r == 3 { return 1; }
  if r == 4 { return 1; }
  if r == 9 { return 1; }
  return 0;
}

// ═══ FE.2: ValenceState (V: 0-7) ═══
// Physics potential energy landscape.
// V=0-2: Barriers (repulsion), V=3-4: Flat (neutral), V=5-7: Wells (attraction)

pub fn valence_name(v) {
  if v == 0 { return "high_barrier"; }    // U = +0.85
  if v == 1 { return "low_barrier"; }     // U = +0.40
  if v == 2 { return "mild_barrier"; }    // U = +0.15
  if v == 3 { return "flat"; }            // U = 0.00
  if v == 4 { return "mild_well"; }       // U = -0.05
  if v == 5 { return "shallow_well"; }    // U = -0.35
  if v == 6 { return "deep_well"; }       // U = -0.75
  if v == 7 { return "very_deep_well"; }  // U = -0.95
  return "flat";
}

// Approach tendency: F = -dU/dx
// Negative → avoidance, Positive → approach, 0 → neutral
pub fn approach_tendency(v) {
  if v == 0 { return 0 - 850; }  // strong avoidance (*1000)
  if v == 1 { return 0 - 400; }
  if v == 2 { return 0 - 150; }
  if v == 3 { return 0; }        // neutral
  if v == 4 { return 50; }
  if v == 5 { return 350; }
  if v == 6 { return 750; }
  if v == 7 { return 950; }      // strong approach
  return 0;
}

// Potential energy U(x) for valence state (*1000 for integer math)
pub fn valence_potential(v) {
  if v == 0 { return 850; }
  if v == 1 { return 400; }
  if v == 2 { return 150; }
  if v == 3 { return 0; }
  if v == 4 { return 0 - 50; }
  if v == 5 { return 0 - 350; }
  if v == 6 { return 0 - 750; }
  if v == 7 { return 0 - 950; }
  return 0;
}

// ═══ FE.3: ArousalState (A: 0-7) ═══
// Damped harmonic oscillator energy regimes.
// A=0-2: Low energy (overdamped), A=3-4: Equilibrium, A=5-7: High energy (underdamped)

pub fn arousal_name(a) {
  if a == 0 { return "ground_state"; }    // E=0.02, γ=100
  if a == 1 { return "heat_death"; }      // E=0.05, γ=50
  if a == 2 { return "overdamped"; }      // E=0.08, γ=30
  if a == 3 { return "equilibrium"; }     // E=0.20, γ=3.0
  if a == 4 { return "mild_eq"; }         // E=0.50, γ=1.0
  if a == 5 { return "excited_low"; }     // E=0.70, γ=0.3
  if a == 6 { return "excited_high"; }    // E=0.90, γ=0.05
  if a == 7 { return "supercritical"; }   // E=0.98, γ=0
  return "equilibrium";
}

// Urgency level (0-1000): how urgent is this arousal state?
pub fn urgency(a) {
  if a == 0 { return 20; }
  if a == 1 { return 50; }
  if a == 2 { return 80; }
  if a == 3 { return 200; }
  if a == 4 { return 500; }
  if a == 5 { return 700; }
  if a == 6 { return 900; }
  if a == 7 { return 950; }
  return 200;
}

// Does this arousal level need urgent attention? (φ⁻¹ = 0.618 threshold)
pub fn needs_urgent(a) {
  return urgency(a) > 618;  // golden ratio threshold
}

// Is this a crisis level? (A ≥ 6)
pub fn is_crisis_arousal(a) {
  return a >= 6;
}

// ═══ FE.Unified: FormulaState ═══
// Combine R + V + A dispatch from a molecule

pub fn formula_from_mol(mol) {
  let r = _kt_mol_r(mol);
  let v = _kt_mol_v(mol);
  let a = _kt_mol_a(mol);
  return [r, v, a, urgency(a), approach_tendency(v)];
  // [0]=R, [1]=V, [2]=A, [3]=urgency, [4]=approach
}

pub fn formula_needs_urgent(f) {
  return __array_get(f, 3) > 618;
}

pub fn formula_approach(f) {
  return __array_get(f, 4);
}

pub fn formula_describe(mol) {
  let r = _kt_mol_r(mol);
  let v = _kt_mol_v(mol);
  let a = _kt_mol_a(mol);
  return relation_name(r) + " | " + valence_name(v) + " | " + arousal_name(a)
       + " | urgency=" + __to_string(urgency(a))
       + " | approach=" + __to_string(approach_tendency(v));
}
