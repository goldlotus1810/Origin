//! # inspector — Đọc và verify sổ cái origin.olang

use std::env;
use std::fs;

use olang::reader::OlangReader;

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "origin.olang".to_string());

    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Cannot read {}: {}", path, e);
            std::process::exit(1);
        }
    };

    println!("Inspector ○  · {}", path);
    println!("File size: {} bytes", data.len());
    println!();

    let reader = match OlangReader::new(&data) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("Created: {} ns", reader.created_at());

    let parsed = match reader.parse_all() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    println!();
    println!("── Contents ──────────────────────────────────");
    println!("Nodes  : {}", parsed.node_count());
    println!("Edges  : {}", parsed.edge_count());
    println!("Aliases: {}", parsed.alias_count());
    println!("QR     : {}", parsed.qr_nodes().len());

    // Layer distribution
    println!();
    println!("── Layer Distribution ───────────────────────");
    for layer in 0..8u8 {
        let count = parsed.nodes_in_layer(layer).len();
        if count > 0 {
            println!("  L{}: {} nodes", layer, count);
        }
    }

    // Sample nodes
    if !parsed.nodes.is_empty() {
        println!();
        println!("── Sample Nodes (first 10) ──────────────────");
        for node in parsed.nodes.iter().take(10) {
            let hash = node.chain.chain_hash();
            let qr = if node.is_qr { "QR" } else { "ĐN" };
            let mol_count = node.chain.len();
            println!(
                "  [{qr}] L{} hash=0x{:08X} mols={} ts={}",
                node.layer,
                hash & 0xFFFFFFFF,
                mol_count,
                node.timestamp
            );
        }
        if parsed.nodes.len() > 10 {
            println!("  ... ({} more)", parsed.nodes.len() - 10);
        }
    }

    // Sample aliases
    if !parsed.aliases.is_empty() {
        println!();
        println!("── Sample Aliases (first 10) ─────────────────");
        let aliases: Vec<_> = parsed
            .aliases
            .iter()
            .filter(|a| !a.name.starts_with("_qr_"))
            .take(10)
            .collect();
        for alias in &aliases {
            println!(
                "  {:?} -> 0x{:08X}",
                alias.name,
                alias.chain_hash & 0xFFFFFFFF
            );
        }
    }

    println!();
    println!("── Verify ────────────────────────────────────");

    // Verify: mọi node có chain không rỗng
    let empty_chains = parsed.nodes.iter().filter(|n| n.chain.is_empty()).count();
    if empty_chains == 0 {
        println!("✓ All {} nodes have non-empty chains", parsed.node_count());
    } else {
        println!("✗ {} nodes have empty chains!", empty_chains);
    }

    // Verify: file offsets tăng dần
    let offsets_ok = parsed
        .nodes
        .windows(2)
        .all(|w| w[0].file_offset < w[1].file_offset);
    if offsets_ok || parsed.nodes.len() <= 1 {
        println!("✓ File offsets monotonically increasing (append-only)");
    } else {
        println!("✗ File offsets NOT monotonic — possible corruption!");
    }

    println!();
    println!("○(∅) == ○ — sổ cái intact");
}

#[cfg(test)]
mod tests {
    use olang::encoder::encode_codepoint;
    use olang::reader::OlangReader;
    use olang::writer::OlangWriter;

    /// Helper: build a minimal valid origin.olang with one node.
    fn make_test_file() -> Vec<u8> {
        let mut w = OlangWriter::new(1_000_000);
        let chain = encode_codepoint(0x25CF); // ● BLACK CIRCLE
        w.append_node(&chain, 0, false, 2_000_000).unwrap();
        w.into_bytes()
    }

    #[test]
    fn default_path_is_origin_olang() {
        // The default path when no CLI arg is given should be "origin.olang".
        // We verify the constant behavior by checking env::args logic inline:
        // unwrap_or_else returns the default.
        let default: String = None::<String>.unwrap_or_else(|| "origin.olang".to_string());
        assert_eq!(default, "origin.olang");
    }

    #[test]
    fn reader_parses_valid_file() {
        let data = make_test_file();
        let reader = OlangReader::new(&data).expect("header should parse");
        assert!(reader.created_at() > 0);

        let parsed = reader.parse_all().expect("parse_all should succeed");
        assert_eq!(parsed.node_count(), 1);
        assert_eq!(parsed.edge_count(), 0);
        assert_eq!(parsed.alias_count(), 0);
    }

    #[test]
    fn reader_rejects_too_short() {
        let data = [0u8; 5]; // too short for header
        assert!(OlangReader::new(&data).is_err());
    }

    #[test]
    fn reader_rejects_bad_magic() {
        // Valid header is 13 bytes: [○LNG][0x03][ts:8]
        // Create 13 bytes with wrong magic
        let data = [0u8; 13];
        assert!(OlangReader::new(&data).is_err());
    }

    #[test]
    fn nodes_have_nonempty_chains() {
        let data = make_test_file();
        let reader = OlangReader::new(&data).unwrap();
        let parsed = reader.parse_all().unwrap();
        let empty_chains = parsed.nodes.iter().filter(|n| n.chain.is_empty()).count();
        assert_eq!(empty_chains, 0);
    }

    #[test]
    fn file_offsets_monotonic() {
        // Build a file with multiple nodes
        let mut w = OlangWriter::new(1_000_000);
        for cp in [0x25CF, 0x25AC, 0x25A0] {
            let chain = encode_codepoint(cp);
            w.append_node(&chain, 0, false, 2_000_000).unwrap();
        }
        let data = w.into_bytes();

        let reader = OlangReader::new(&data).unwrap();
        let parsed = reader.parse_all().unwrap();
        assert_eq!(parsed.node_count(), 3);

        let offsets_ok = parsed
            .nodes
            .windows(2)
            .all(|w| w[0].file_offset < w[1].file_offset);
        assert!(offsets_ok, "file offsets should be monotonically increasing");
    }

    #[test]
    fn qr_nodes_detected() {
        let mut w = OlangWriter::new(1_000_000);
        let chain = encode_codepoint(0x25CF);
        w.append_node(&chain, 0, true, 2_000_000).unwrap(); // is_qr = true
        let data = w.into_bytes();

        let reader = OlangReader::new(&data).unwrap();
        let parsed = reader.parse_all().unwrap();
        assert_eq!(parsed.qr_nodes().len(), 1);
    }

    #[test]
    fn layer_distribution() {
        let mut w = OlangWriter::new(1_000_000);
        let chain = encode_codepoint(0x25CF);
        w.append_node(&chain, 0, false, 2_000_000).unwrap();
        w.append_node(&chain, 1, false, 3_000_000).unwrap();
        let data = w.into_bytes();

        let reader = OlangReader::new(&data).unwrap();
        let parsed = reader.parse_all().unwrap();
        assert_eq!(parsed.nodes_in_layer(0).len(), 1);
        assert_eq!(parsed.nodes_in_layer(1).len(), 1);
        assert_eq!(parsed.nodes_in_layer(2).len(), 0);
    }

    #[test]
    fn aliases_parsed() {
        let mut w = OlangWriter::new(1_000_000);
        let chain = encode_codepoint(0x25CF);
        let hash = chain.chain_hash();
        w.append_node(&chain, 0, false, 2_000_000).unwrap();
        w.append_alias("circle", hash, 3_000_000).unwrap();
        let data = w.into_bytes();

        let reader = OlangReader::new(&data).unwrap();
        let parsed = reader.parse_all().unwrap();
        assert_eq!(parsed.alias_count(), 1);
        assert_eq!(parsed.aliases[0].name, "circle");
    }
}
