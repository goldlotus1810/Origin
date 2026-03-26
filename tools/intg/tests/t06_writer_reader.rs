//! Integration: Writer v0.05 → Reader parse → data khớp
//!
//! Kiểm tra: Writer ghi records → Reader đọc lại → tất cả fields khớp.
//! Test cả tagged v0.05 format. Append-only verification.

use intg::codepoints::*;
use olang::encoder::encode_codepoint;
use olang::registry::NodeKind;
use olang::storage::reader::OlangReader;
use olang::storage::writer::OlangWriter;

// ── Node write/read roundtrip ────────────────────────────────────────────────

#[test]
fn write_read_node_roundtrip() {
    let mut writer = OlangWriter::new(1000);
    let chain = encode_codepoint(FIRE);

    writer.append_node(&chain, 0, false, 2000).unwrap();

    let data = writer.as_bytes();
    let reader = OlangReader::new(data).expect("reader must parse header");
    let parsed = reader.parse_all().expect("parse must succeed");

    assert_eq!(parsed.nodes.len(), 1, "must have 1 node");
    let node = &parsed.nodes[0];
    assert_eq!(node.layer, 0);
    assert!(!node.is_qr);
    assert_eq!(node.timestamp, 2000);
    assert_eq!(node.chain.chain_hash(), chain.chain_hash(), "chain_hash must match");
}

#[test]
fn write_read_multiple_nodes() {
    let mut writer = OlangWriter::new(1000);
    let cps = [FIRE, SPHERE, MEMBER, ARROW, DROPLET];

    for (i, &cp) in cps.iter().enumerate() {
        let chain = encode_codepoint(cp);
        writer.append_node(&chain, i as u8, false, 2000 + i as i64).unwrap();
    }

    let data = writer.as_bytes();
    let reader = OlangReader::new(data).unwrap();
    let parsed = reader.parse_all().unwrap();

    assert_eq!(parsed.nodes.len(), cps.len(), "all nodes must be written and read");

    for (i, node) in parsed.nodes.iter().enumerate() {
        assert_eq!(node.layer, i as u8, "layer must match for node {i}");
        let expected_hash = encode_codepoint(cps[i]).chain_hash();
        assert_eq!(node.chain.chain_hash(), expected_hash, "hash must match for node {i}");
    }
}

// ── Edge write/read roundtrip ────────────────────────────────────────────────

#[test]
fn write_read_edge_roundtrip() {
    let mut writer = OlangWriter::new(1000);
    let h1 = encode_codepoint(FIRE).chain_hash();
    let h2 = encode_codepoint(DROPLET).chain_hash();

    writer.append_edge(h1, h2, 0x01, 3000);

    let data = writer.as_bytes();
    let reader = OlangReader::new(data).unwrap();
    let parsed = reader.parse_all().unwrap();

    assert_eq!(parsed.edges.len(), 1);
    let edge = &parsed.edges[0];
    assert_eq!(edge.from_hash, h1);
    assert_eq!(edge.to_hash, h2);
    assert_eq!(edge.edge_type, 0x01);
    assert_eq!(edge.timestamp, 3000);
}

// ── Alias write/read roundtrip ───────────────────────────────────────────────

#[test]
fn write_read_alias_roundtrip() {
    let mut writer = OlangWriter::new(1000);
    let h = encode_codepoint(FIRE).chain_hash();

    writer.append_alias("lửa", h, 4000).unwrap();
    writer.append_alias("fire", h, 4001).unwrap();

    let data = writer.as_bytes();
    let reader = OlangReader::new(data).unwrap();
    let parsed = reader.parse_all().unwrap();

    assert_eq!(parsed.aliases.len(), 2);
    assert_eq!(parsed.aliases[0].name, "lửa");
    assert_eq!(parsed.aliases[0].chain_hash, h);
    assert_eq!(parsed.aliases[1].name, "fire");
}

// ── NodeKind write/read roundtrip ────────────────────────────────────────────

#[test]
fn write_read_node_kind_roundtrip() {
    let mut writer = OlangWriter::new(1000);
    let h = encode_codepoint(FIRE).chain_hash();

    writer.append_node_kind(h, NodeKind::Knowledge as u8, 5000);

    let data = writer.as_bytes();
    let reader = OlangReader::new(data).unwrap();
    let parsed = reader.parse_all().unwrap();

    assert_eq!(parsed.node_kinds.len(), 1);
    assert_eq!(parsed.node_kinds[0].chain_hash, h);
    assert_eq!(parsed.node_kinds[0].kind, NodeKind::Knowledge as u8);
}

// ── STM record write/read roundtrip ──────────────────────────────────────────

#[test]
fn write_read_stm_roundtrip() {
    let mut writer = OlangWriter::new(1000);
    let h = encode_codepoint(SAD).chain_hash();

    writer.append_stm(h, -0.7, 0.5, 0.3, 0.8, 3, 0, 0, 6000);

    let data = writer.as_bytes();
    let reader = OlangReader::new(data).unwrap();
    let parsed = reader.parse_all().unwrap();

    assert_eq!(parsed.stm_records.len(), 1);
    let stm = &parsed.stm_records[0];
    assert_eq!(stm.chain_hash, h);
    assert!((stm.valence - (-0.7)).abs() < 0.01);
    assert!((stm.arousal - 0.5).abs() < 0.01);
}

// ── Hebbian write/read roundtrip ─────────────────────────────────────────────

#[test]
fn write_read_hebbian_roundtrip() {
    let mut writer = OlangWriter::new(1000);
    let h1 = encode_codepoint(FIRE).chain_hash();
    let h2 = encode_codepoint(DROPLET).chain_hash();

    writer.append_hebbian(h1, h2, 200, 5, 7000);

    let data = writer.as_bytes();
    let reader = OlangReader::new(data).unwrap();
    let parsed = reader.parse_all().unwrap();

    assert_eq!(parsed.hebbian_records.len(), 1);
    let heb = &parsed.hebbian_records[0];
    assert_eq!(heb.from_hash, h1);
    assert_eq!(heb.to_hash, h2);
    assert_eq!(heb.weight, 200);
    assert_eq!(heb.fire_count, 5);
}

// ── Curve write/read roundtrip ───────────────────────────────────────────────

#[test]
fn write_read_curve_roundtrip() {
    let mut writer = OlangWriter::new(1000);

    writer.append_curve(-0.5, 0.3, 8000);

    let data = writer.as_bytes();
    let reader = OlangReader::new(data).unwrap();
    let parsed = reader.parse_all().unwrap();

    assert_eq!(parsed.curve_records.len(), 1);
    let crv = &parsed.curve_records[0];
    assert!((crv.valence - (-0.5)).abs() < 0.01);
    assert!((crv.fx_dn - 0.3).abs() < 0.01);
}

// ── Append-only: ghi thêm → cũ+mới đều đúng ────────────────────────────────

#[test]
fn append_preserves_previous_records() {
    let mut writer = OlangWriter::new(1000);

    let chain1 = encode_codepoint(FIRE);
    writer.append_node(&chain1, 0, false, 2000).unwrap();

    let chain2 = encode_codepoint(SPHERE);
    writer.append_node(&chain2, 0, false, 3000).unwrap();

    let chain3 = encode_codepoint(MEMBER);
    writer.append_node(&chain3, 1, false, 4000).unwrap();

    let data = writer.as_bytes();
    let reader = OlangReader::new(data).unwrap();
    let parsed = reader.parse_all().unwrap();

    assert_eq!(parsed.nodes.len(), 3, "all appended nodes must be present");
    assert_eq!(parsed.nodes[0].chain.chain_hash(), chain1.chain_hash());
    assert_eq!(parsed.nodes[1].chain.chain_hash(), chain2.chain_hash());
    assert_eq!(parsed.nodes[2].chain.chain_hash(), chain3.chain_hash());
}
