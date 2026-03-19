//! Integration: Agents → Memory pipeline
//!
//! SecurityGate.check() → ContentEncoder.encode() → STM.push() →
//! Hebbian co_activate strengthening.

use agents::encoder::{ContentEncoder, ContentInput};
use agents::gate::{GateVerdict, SecurityGate};
use agents::learning::ShortTermMemory;
use silk::edge::EmotionTag;
use silk::graph::SilkGraph;

#[test]
fn gate_allows_normal_text() {
    let gate = SecurityGate::new();
    let verdict = gate.check_text("tôi vui hôm nay");
    assert!(matches!(verdict, GateVerdict::Allow), "normal text must be allowed, got {:?}", verdict);
}

#[test]
fn gate_blocks_crisis_text() {
    let gate = SecurityGate::new();
    let verdict = gate.check_text("tôi muốn tự tử");
    assert!(!matches!(verdict, GateVerdict::Allow), "crisis text must NOT be allowed, got {:?}", verdict);
}

#[test]
fn encoder_produces_valid_chain() {
    let encoder = ContentEncoder::new();
    let input = ContentInput::Text { content: "lửa cháy".into(), timestamp: 1000 };
    let result = encoder.encode(input);
    assert!(!result.chain.is_empty());
    assert_ne!(result.chain.chain_hash(), 0);
}

#[test]
fn encoder_deterministic() {
    let encoder = ContentEncoder::new();
    let r1 = encoder.encode(ContentInput::Text { content: "lửa cháy".into(), timestamp: 1000 });
    let r2 = encoder.encode(ContentInput::Text { content: "lửa cháy".into(), timestamp: 1000 });
    assert_eq!(r1.chain.chain_hash(), r2.chain.chain_hash(), "same text → same chain");
}

#[test]
fn stm_push_and_retrieve() {
    let mut stm = ShortTermMemory::new(512);
    let encoder = ContentEncoder::new();
    let result = encoder.encode(ContentInput::Text { content: "buồn".into(), timestamp: 1000 });
    let emo = EmotionTag { valence: -0.7, arousal: 0.5, dominance: 0.3, intensity: 0.8 };
    stm.push(result.chain, emo, 1000);
    assert!(!stm.is_empty());
}

#[test]
fn stm_multiple_pushes_fire_count() {
    let mut stm = ShortTermMemory::new(512);
    let encoder = ContentEncoder::new();
    // Push same chain multiple times → fire_count should increase (deduplication)
    let result = encoder.encode(ContentInput::Text { content: "buồn".into(), timestamp: 1000 });
    let emo = EmotionTag { valence: -0.5, arousal: 0.5, dominance: 0.0, intensity: 0.5 };
    stm.push(result.chain.clone(), emo, 1000);
    stm.push(result.chain.clone(), emo, 2000);
    stm.push(result.chain.clone(), emo, 3000);
    // STM deduplicates by chain_hash → len = 1 but fire_count > 1
    assert_eq!(stm.len(), 1, "STM deduplicates by chain_hash");
    let obs = stm.all();
    assert!(obs[0].fire_count >= 3, "fire_count must be >= 3, got {}", obs[0].fire_count);
}

#[test]
fn hebbian_strengthening_through_co_activate() {
    let mut graph = SilkGraph::new();
    let encoder = ContentEncoder::new();
    let r1 = encoder.encode(ContentInput::Text { content: "buồn".into(), timestamp: 1000 });
    let r2 = encoder.encode(ContentInput::Text { content: "mất việc".into(), timestamp: 1000 });
    let h1 = r1.chain.chain_hash();
    let h2 = r2.chain.chain_hash();
    let emo = EmotionTag { valence: -0.65, arousal: 0.45, dominance: 0.0, intensity: 0.8 };
    graph.co_activate(h1, h2, emo, 0.8, 1000);
    let w1 = graph.find_edge(h1, h2, silk::edge::EdgeKind::Assoc).map(|e| e.weight).unwrap_or(0.0);
    graph.co_activate(h1, h2, emo, 0.8, 2000);
    let w2 = graph.find_edge(h1, h2, silk::edge::EdgeKind::Assoc).map(|e| e.weight).unwrap_or(0.0);
    assert!(w2 >= w1, "Hebbian must strengthen: {w1} → {w2}");
}
