//! Integration: Olang encode → chain_hash → Silk co_activate → lookup

use intg::codepoints::*;
use intg::{encode_and_hash, mol_summary_of};
use olang::encoder::encode_codepoint;
use olang::registry::Registry;
use silk::edge::EmotionTag;
use silk::graph::SilkGraph;

#[test]
fn co_activate_creates_edge() {
    let mut graph = SilkGraph::new();
    let (_, h_fire) = encode_and_hash(FIRE);
    let (_, h_drop) = encode_and_hash(DROPLET);
    let emo = EmotionTag { valence: -0.5, arousal: 0.6, dominance: 0.0, intensity: 0.8 };
    graph.co_activate(h_fire, h_drop, emo, 0.8, 1000);
    let neighbors = graph.neighbors(h_fire);
    assert!(neighbors.contains(&h_drop), "FIRE must have DROPLET as neighbor");
}

#[test]
fn co_activate_strengthens_existing_edge() {
    let mut graph = SilkGraph::new();
    let (_, h_fire) = encode_and_hash(FIRE);
    let (_, h_drop) = encode_and_hash(DROPLET);
    let emo = EmotionTag { valence: -0.5, arousal: 0.6, dominance: 0.0, intensity: 0.8 };
    graph.co_activate(h_fire, h_drop, emo, 0.5, 1000);
    let w1 = graph.find_edge(h_fire, h_drop, silk::edge::EdgeKind::Assoc).map(|e| e.weight).unwrap_or(0.0);
    graph.co_activate(h_fire, h_drop, emo, 0.5, 2000);
    let w2 = graph.find_edge(h_fire, h_drop, silk::edge::EdgeKind::Assoc).map(|e| e.weight).unwrap_or(0.0);
    assert!(w2 >= w1, "repeated co_activate must strengthen: {w1} → {w2}");
}

#[test]
fn co_activate_mol_with_similarity() {
    let mut graph = SilkGraph::new();
    let (_, h_fire) = encode_and_hash(FIRE);
    let (_, h_happy) = encode_and_hash(HAPPY);
    let ms_fire = mol_summary_of(FIRE);
    let ms_happy = mol_summary_of(HAPPY);
    let emo = EmotionTag { valence: 0.5, arousal: 0.5, dominance: 0.0, intensity: 0.5 };
    graph.co_activate_mol(h_fire, h_happy, Some(ms_fire), Some(ms_happy), emo, 0.5, 1000);
    let neighbors = graph.neighbors(h_fire);
    assert!(neighbors.contains(&h_happy));
}

#[test]
fn registry_hash_matches_silk_hash() {
    let mut reg = Registry::new();
    let mut graph = SilkGraph::new();
    let chain_fire = encode_codepoint(FIRE);
    let chain_drop = encode_codepoint(DROPLET);
    let h_fire = chain_fire.chain_hash();
    let h_drop = chain_drop.chain_hash();
    reg.insert(&chain_fire, 0, 0, 1000, false);
    reg.insert(&chain_drop, 0, 100, 1000, false);
    let emo = EmotionTag { valence: -0.3, arousal: 0.4, dominance: 0.0, intensity: 0.6 };
    graph.co_activate(h_fire, h_drop, emo, 0.7, 1000);
    let reg_hash = reg.lookup_hash(h_fire).unwrap().chain_hash;
    let neighbors = graph.neighbors(reg_hash);
    assert!(neighbors.contains(&h_drop), "Registry hash must match Silk graph hash");
}

#[test]
fn co_activate_same_layer_enforced() {
    let mut graph = SilkGraph::new();
    let (_, h_fire) = encode_and_hash(FIRE);
    let (_, h_drop) = encode_and_hash(DROPLET);
    let emo = EmotionTag { valence: 0.0, arousal: 0.5, dominance: 0.0, intensity: 0.5 };
    assert!(graph.co_activate_same_layer(h_fire, h_drop, 0, 0, emo, 0.5, 1000));
    assert!(!graph.co_activate_same_layer(h_fire, h_drop, 0, 1, emo, 0.5, 2000));
}

#[test]
fn implicit_silk_similarity() {
    let ms_fire = mol_summary_of(FIRE);
    let ms_happy = mol_summary_of(HAPPY);
    let sim = ms_fire.similarity(&ms_happy);
    assert!((0.0..=1.0).contains(&sim), "similarity must be in [0,1]");
}
