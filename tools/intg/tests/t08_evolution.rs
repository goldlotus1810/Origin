//! Integration: Molecule.evolve() → new chain → Registry → Silk
//!
//! Kiểm tra: 🔥 encode → evolve(Valence, 0x40) → new Molecule valid.
//! consistency_check ≥ 3/4. dimension_delta() đúng chiều thay đổi.
//! New chain → Registry insert → Silk liên kết với parent.

use intg::codepoints::*;
use olang::encoder::encode_codepoint;
use olang::molecular::Dimension;
use olang::registry::Registry;
use silk::edge::EmotionTag;
use silk::graph::SilkGraph;

#[test]
fn evolve_valence_creates_new_molecule() {
    let chain = encode_codepoint(FIRE);
    let mol = chain.first().unwrap();

    let result = mol.evolve(Dimension::Valence, 0x40);

    assert!(result.valid, "evolve(Valence, 0x40) must be valid (consistency ≥ 3)");
    assert_ne!(
        result.molecule.valence(),
        mol.valence(),
        "evolved molecule must have different valence"
    );
    assert_eq!(result.molecule.valence(), olang::molecular::Molecule::pack(0, 0, 0x40, 0, 0).valence());
    assert_eq!(result.molecule.shape(), mol.shape(), "shape must be unchanged");
    assert_eq!(result.molecule.relation(), mol.relation(), "relation must be unchanged");
}

#[test]
fn evolve_shape_creates_new_molecule() {
    let chain = encode_codepoint(FIRE);
    let mol = chain.first().unwrap();

    // Evolve shape to Cone (0x04)
    let result = mol.evolve(Dimension::Shape, 0x04);
    assert_eq!(result.molecule.shape(), olang::molecular::Molecule::pack(0x04, 0, 0, 0, 0).shape());
}

#[test]
fn evolve_time_creates_new_molecule() {
    let chain = encode_codepoint(FIRE);
    let mol = chain.first().unwrap();

    // Evolve time to Static (0x01)
    let result = mol.evolve(Dimension::Time, 0x01);
    assert_eq!(result.molecule.time(), olang::molecular::Molecule::pack(0, 0, 0, 0, 0x01).time());
}

// ── dimension_delta detects correct change ───────────────────────────────────

#[test]
fn dimension_delta_detects_valence_change() {
    let chain = encode_codepoint(FIRE);
    let mol = chain.first().unwrap();

    let result = mol.evolve(Dimension::Valence, 0x40);
    let deltas = mol.dimension_delta(&result.molecule);

    assert!(
        deltas.iter().any(|(dim, _, _)| *dim == Dimension::Valence),
        "dimension_delta must detect Valence change"
    );
    // Only valence should change
    assert!(
        !deltas.iter().any(|(dim, _, _)| *dim == Dimension::Shape),
        "Shape should not be in deltas"
    );
}

#[test]
fn dimension_delta_empty_for_identical() {
    let chain = encode_codepoint(FIRE);
    let mol = chain.first().unwrap();
    let deltas = mol.dimension_delta(&mol);
    assert!(deltas.is_empty(), "identical molecules should have no deltas");
}

// ── chain evolve_and_apply → new chain_hash ──────────────────────────────────

#[test]
fn chain_evolution_changes_hash() {
    let chain = encode_codepoint(FIRE);
    let original_hash = chain.chain_hash();

    if let Some((new_chain, result)) = chain.evolve_and_apply(0, Dimension::Valence, 0x40) {
        let new_hash = new_chain.chain_hash();
        assert_ne!(original_hash, new_hash, "evolved chain must have different hash");
        assert!(result.valid, "evolution must be valid");
    } else {
        panic!("evolve_and_apply must succeed for valid evolution");
    }
}

// ── Evolution → Registry + Silk integration ──────────────────────────────────

#[test]
fn evolved_chain_registers_and_links_to_parent() {
    let mut reg = Registry::new();
    let mut graph = SilkGraph::new();

    // Original: 🔥
    let chain_fire = encode_codepoint(FIRE);
    let h_fire = chain_fire.chain_hash();
    reg.insert(&chain_fire, 0, 0, 1000, false);

    // Evolve: 🔥 → "lửa nhẹ" (valence 0x40)
    let (new_chain, _result) = chain_fire
        .evolve_and_apply(0, Dimension::Valence, 0x40)
        .expect("evolution must succeed");
    let h_evolved = new_chain.chain_hash();

    // Register evolved chain
    reg.insert(&new_chain, 0, 100, 2000, false);
    assert!(reg.lookup_hash(h_evolved).is_some(), "evolved chain must be in registry");
    assert_ne!(h_fire, h_evolved, "parent and child hashes must differ");

    // Link in Silk
    let emo = EmotionTag {
        valence: 0.3,
        arousal: 0.3,
        dominance: 0.0,
        intensity: 0.5,
    };
    graph.co_activate(h_fire, h_evolved, emo, 0.7, 2000);

    let neighbors = graph.neighbors(h_fire);
    assert!(
        neighbors.contains(&h_evolved),
        "evolved chain must be linked to parent in Silk"
    );
}

#[test]
fn multiple_evolutions_from_same_parent() {
    let chain_fire = encode_codepoint(FIRE);
    let h_fire = chain_fire.chain_hash();

    // Evolve valence
    let (chain_v, _) = chain_fire
        .evolve_and_apply(0, Dimension::Valence, 0x40)
        .unwrap();
    // Evolve time
    let (chain_t, _) = chain_fire
        .evolve_and_apply(0, Dimension::Time, 0x01)
        .unwrap();

    let h_v = chain_v.chain_hash();
    let h_t = chain_t.chain_hash();

    assert_ne!(h_v, h_t, "different evolutions must produce different hashes");
    assert_ne!(h_v, h_fire, "evolved V must differ from parent");
    assert_ne!(h_t, h_fire, "evolved T must differ from parent");
}
