//! # domains — Domain Knowledge Seeds
//!
//! Mỗi domain = 1 module, seed nodes + edges + aliases.
//! Tất cả chain từ encode_codepoint(cp) — KHÔNG hardcode.

pub mod algorithms;
pub mod biology;
pub mod chemistry;
pub mod math;
pub mod philosophy;
pub mod physics;

use olang::encoder::encode_codepoint;
use olang::log::{EventLog, LogEvent};
use olang::registry::Registry;
use olang::writer::OlangWriter;

/// One domain node to seed.
pub struct SeedNode {
    /// Internal name (used for edge references).
    pub name: &'static str,
    /// Unicode codepoint (representative).
    pub codepoint: u32,
    /// Aliases: multilingual names, LaTeX notation, etc.
    pub aliases: &'static [&'static str],
}

/// One domain edge to seed.
pub struct SeedEdge {
    /// Source node name.
    pub from: &'static str,
    /// Target node name.
    pub to: &'static str,
    /// Relation byte (EdgeKind).
    pub relation: u8,
}

/// Seed a domain into writer/registry.
///
/// Returns (nodes_created, edges_created).
#[allow(clippy::too_many_arguments)]
pub fn seed_domain(
    domain_name: &str,
    nodes: &[SeedNode],
    edges: &[SeedEdge],
    layer: u8,
    writer: &mut OlangWriter,
    registry: &mut Registry,
    log: &mut EventLog,
    ts: i64,
) -> (usize, usize) {
    let mut node_count = 0usize;
    let mut edge_count = 0usize;

    // Phase 1: Seed nodes
    for node in nodes {
        let chain = encode_codepoint(node.codepoint);
        let hash = chain.chain_hash();

        // Skip if already registered (idempotent)
        if registry.lookup_hash(hash).is_some() {
            // Still register new aliases
            for &alias in node.aliases {
                if registry.lookup_name(alias).is_none() {
                    registry.register_alias(alias, hash);
                    let _ = writer.append_alias(alias, hash, ts);
                }
            }
            continue;
        }

        // QT8: file TRƯỚC
        let offset = match writer.append_node(&chain, layer, false, ts) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("[{}] write error {}: {:?}", domain_name, node.name, e);
                continue;
            }
        };

        // Registry SAU
        registry.insert(&chain, layer, offset, ts, false);
        registry.register_alias(node.name, hash);
        for &alias in node.aliases {
            registry.register_alias(alias, hash);
            let _ = writer.append_alias(alias, hash, ts);
        }

        log.append(LogEvent::NodeCreated {
            chain_hash: hash,
            layer,
            file_offset: offset,
            timestamp: ts,
        });

        let uname = ucd::lookup(node.codepoint).map(|e| e.name).unwrap_or("?");
        println!("[{}] ✓ {} (U+{:04X} {})", domain_name, node.name, node.codepoint, uname);
        node_count += 1;
    }

    // Phase 2: Seed edges (relationships between nodes)
    for edge in edges {
        let from_hash = match registry.lookup_name(edge.from) {
            Some(h) => h,
            None => continue, // source not found
        };
        let to_hash = match registry.lookup_name(edge.to) {
            Some(h) => h,
            None => continue, // target not found
        };

        writer.append_edge(from_hash, to_hash, edge.relation, ts);

        log.append(LogEvent::EdgeCreated {
            from_hash,
            to_hash,
            edge_type: edge.relation,
            timestamp: ts,
        });

        edge_count += 1;
    }

    println!(
        "[{}] {} nodes, {} edges",
        domain_name, node_count, edge_count
    );
    (node_count, edge_count)
}
