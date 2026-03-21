//! Integration: Quy Tắc Bất Biến từ CLAUDE.md

use intg::codepoints::*;
use olang::encoder::encode_codepoint;
use olang::molecular::Dimension;
use olang::registry::Registry;
use olang::storage::reader::OlangReader;
use olang::storage::writer::OlangWriter;
use silk::edge::EmotionTag;
use silk::graph::SilkGraph;

/// QT① 5 nhóm Unicode = nền tảng
#[test]
fn qt1_five_unicode_groups() {
    assert_eq!(ucd::lookup(SPHERE).unwrap().group, 0x01, "● = SDF");
    assert_eq!(ucd::lookup(MEMBER).unwrap().group, 0x02, "∈ = MATH");
    assert_eq!(ucd::lookup(FIRE).unwrap().group, 0x03, "🔥 = EMOTICON");
    assert!(ucd::lookup(MUSICAL).unwrap().group > 0, "♩ must belong to a group");
}

/// QT④ Molecule chỉ từ encode_codepoint()
#[test]
fn qt4_molecule_from_encode_only() {
    for &cp in &[FIRE, SPHERE, MEMBER, ARROW, HAPPY, SAD, DROPLET] {
        let chain = encode_codepoint(cp);
        assert!(!chain.is_empty(), "0x{:05X}", cp);
        assert_ne!(chain.chain_hash(), 0, "0x{:05X}", cp);
    }
}

/// QT⑤ chain từ LCA hoặc UCD
#[test]
fn qt5_chain_from_lca() {
    let a = encode_codepoint(FIRE);
    let b = encode_codepoint(DROPLET);
    let parent = olang::lca::lca(&a, &b);
    assert!(!parent.is_empty());
}

/// QT⑥ chain_hash tự sinh, deterministic
#[test]
fn qt6_chain_hash_deterministic() {
    let chain = encode_codepoint(FIRE);
    assert_eq!(chain.chain_hash(), chain.chain_hash());
    assert_ne!(chain.chain_hash(), 0);
}

/// QT⑧ Node phải register
#[test]
fn qt8_node_must_register() {
    let mut reg = Registry::new();
    let chain = encode_codepoint(FIRE);
    assert!(reg.lookup_hash(chain.chain_hash()).is_none());
    reg.insert(&chain, 0, 0, 1000, false);
    assert!(reg.lookup_hash(chain.chain_hash()).is_some());
}

/// QT⑩ Append-only
#[test]
fn qt10_append_only() {
    let mut writer = OlangWriter::new(1000);
    let c1 = encode_codepoint(FIRE);
    let c2 = encode_codepoint(SPHERE);
    writer.append_node(&c1, 0, false, 2000).unwrap();
    let s1 = writer.as_bytes().len();
    writer.append_node(&c2, 0, false, 3000).unwrap();
    let s2 = writer.as_bytes().len();
    assert!(s2 > s1, "writer must only grow");
    let reader = OlangReader::new(writer.as_bytes()).unwrap();
    let parsed = reader.parse_all().unwrap();
    assert_eq!(parsed.nodes.len(), 2);
}

/// QT⑪ Silk cùng tầng
#[test]
fn qt11_silk_same_layer() {
    let mut graph = SilkGraph::new();
    let (_, h1) = intg::encode_and_hash(FIRE);
    let (_, h2) = intg::encode_and_hash(DROPLET);
    let emo = EmotionTag { valence: 0.0, arousal: 0.5, dominance: 0.0, intensity: 0.5 };
    assert!(graph.co_activate_same_layer(h1, h2, 0, 0, emo, 0.5, 1000));
    assert!(!graph.co_activate_same_layer(h1, h2, 0, 2, emo, 0.5, 2000));
}

/// QT⑬ Silk mang EmotionTag
#[test]
fn qt13_silk_carries_emotion() {
    let mut graph = SilkGraph::new();
    let (_, h1) = intg::encode_and_hash(FIRE);
    let (_, h2) = intg::encode_and_hash(HAPPY);
    let emo = EmotionTag { valence: 0.8, arousal: 0.9, dominance: 0.5, intensity: 0.7 };
    graph.co_activate(h1, h2, emo, 0.7, 1000);
    assert!(graph.neighbors(h1).contains(&h2));
}

/// QT⑰ Fibonacci
#[test]
fn qt17_fibonacci() {
    use silk::hebbian::fib;
    assert_eq!(fib(0), 1);
    assert_eq!(fib(1), 1);
    assert_eq!(fib(2), 2);
    assert_eq!(fib(5), 8);
    assert_eq!(fib(7), 21);
}

/// QT⑱ Gate: không bịa
#[test]
fn qt18_gate_no_fabrication() {
    let gate = agents::gate::SecurityGate::new();
    assert!(matches!(gate.check_text("hello"), agents::gate::GateVerdict::Allow));
}

/// Evolution consistency ≥ 3/4
#[test]
fn evolution_consistency() {
    let chain = encode_codepoint(FIRE);
    let mol = chain.first().unwrap();
    let result = mol.evolve(Dimension::Valence, 0x40);
    assert!(result.consistency >= 3, "consistency = {}", result.consistency);
}
