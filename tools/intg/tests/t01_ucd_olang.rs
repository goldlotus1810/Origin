//! Integration: UCD → Olang encoder → Registry roundtrip

use intg::codepoints::*;
use olang::encoder::encode_codepoint;
use olang::registry::{NodeKind, Registry};

#[test]
fn encode_fire_produces_valid_chain() {
    let chain = encode_codepoint(FIRE);
    assert!(!chain.is_empty());
    assert_eq!(chain.len(), 1);
    let mol = chain.first().unwrap();
    assert_eq!(mol.shape_base(), olang::molecular::ShapeBase::Sphere, "FIRE shape base = Sphere");
    assert!(mol.valence_u8() >= 0xC0, "FIRE valence must be high");
    assert!(mol.arousal_u8() >= 0xC0, "FIRE arousal must be high");
}

#[test]
fn encode_sphere_sdf_primitive() {
    let chain = encode_codepoint(SPHERE);
    let mol = chain.first().unwrap();
    assert_eq!(mol.shape_base(), olang::molecular::ShapeBase::Sphere, "● = Sphere base");
}

#[test]
fn encode_member_relation_primitive() {
    let chain = encode_codepoint(MEMBER);
    let mol = chain.first().unwrap();
    assert_eq!(mol.relation_base(), olang::molecular::RelationBase::Member, "∈ = Member base");
}

#[test]
fn encode_arrow_causes_relation() {
    let chain = encode_codepoint(ARROW);
    let mol = chain.first().unwrap();
    assert_eq!(mol.relation_base(), olang::molecular::RelationBase::Causes, "→ = Causes base");
}

#[test]
fn encode_musical_note() {
    let chain = encode_codepoint(MUSICAL);
    assert_eq!(chain.len(), 1, "♩ encodes to 1 molecule");
}

#[test]
fn chain_hashes_are_unique_across_codepoints() {
    let cps = [FIRE, SPHERE, MEMBER, ARROW, TORUS, HAPPY, SAD, DROPLET, MUSICAL];
    let hashes: Vec<u64> = cps.iter().map(|&cp| encode_codepoint(cp).chain_hash()).collect();
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(hashes[i], hashes[j],
                "hash collision: 0x{:05X} and 0x{:05X}", cps[i], cps[j]);
        }
    }
}

#[test]
fn encode_is_deterministic() {
    let chain1 = encode_codepoint(FIRE);
    let chain2 = encode_codepoint(FIRE);
    assert_eq!(chain1.chain_hash(), chain2.chain_hash());
    assert_eq!(chain1.to_bytes(), chain2.to_bytes());
}

#[test]
fn registry_insert_and_lookup() {
    let mut reg = Registry::new();
    let chain = encode_codepoint(FIRE);
    let hash = chain.chain_hash();
    reg.insert(&chain, 0, 0, 1000, false);
    let entry = reg.lookup_hash(hash).expect("lookup must return entry");
    assert_eq!(entry.chain_hash, hash);
    assert_eq!(entry.layer, 0);
}

#[test]
fn registry_alias_roundtrip() {
    let mut reg = Registry::new();
    let chain = encode_codepoint(FIRE);
    let hash = chain.chain_hash();
    reg.insert(&chain, 0, 0, 1000, false);
    reg.register_alias("lửa", hash);
    reg.register_alias("fire", hash);
    assert_eq!(reg.lookup_name("lửa"), Some(hash));
    assert_eq!(reg.lookup_name("fire"), Some(hash));
    assert_eq!(reg.lookup_name("nonexistent"), None);
}

#[test]
fn registry_insert_with_kind() {
    let mut reg = Registry::new();
    let chain = encode_codepoint(FIRE);
    reg.insert_with_kind(&chain, 0, 0, 1000, false, NodeKind::Knowledge);
    let entry = reg.lookup_hash(chain.chain_hash()).unwrap();
    assert_eq!(entry.kind, NodeKind::Knowledge);
}

#[test]
fn registry_multiple_codepoints() {
    let mut reg = Registry::new();
    let cps = [FIRE, SPHERE, MEMBER, ARROW, TORUS];
    for &cp in &cps {
        let chain = encode_codepoint(cp);
        reg.insert(&chain, 0, 0, 1000, false);
    }
    assert_eq!(reg.len(), cps.len());
    for &cp in &cps {
        let hash = encode_codepoint(cp).chain_hash();
        assert!(reg.lookup_hash(hash).is_some(), "0x{:05X} must be in registry", cp);
    }
}

#[test]
fn ucd_lookup_matches_encoder_molecule() {
    let entry = ucd::lookup(FIRE).expect("FIRE must be in UCD");
    let chain = encode_codepoint(FIRE);
    let mol = chain.first().unwrap();
    // v2: Molecule packs raw u8 values into quantized bits, so compare quantized
    assert_eq!(mol.shape(), entry.shape >> 4, "shape quantized match");
    assert_eq!(mol.relation(), entry.relation >> 4, "relation quantized match");
    assert_eq!(mol.valence(), entry.valence >> 5, "valence quantized match");
    assert_eq!(mol.arousal(), entry.arousal >> 5, "arousal quantized match");
    assert_eq!(mol.time(), entry.time >> 6, "time quantized match");
}
