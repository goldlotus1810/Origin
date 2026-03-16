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
