//! # seed_domains — Seed domain knowledge into origin.olang
//!
//! Seeds: math (LaTeX), physics, chemistry, biology, philosophy, algorithms.
//! Appends to existing origin.olang (incremental).
//! Tất cả chain từ encode_codepoint(cp) — KHÔNG hardcode.

mod domains;

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use olang::log::EventLog;
use olang::registry::Registry;
use olang::writer::OlangWriter;

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}

fn main() {
    println!("[seed_domains] HomeOS Domain Knowledge Seeder");
    println!("[seed_domains] Domains: math, physics, chemistry, biology, philosophy, algorithms");

    println!("[seed_domains] UCD: {} entries", ucd::table_len());

    let ts = now_ns();

    // Load existing origin.olang or create new
    let mut writer = if let Ok(existing) = fs::read("origin.olang") {
        println!(
            "[seed_domains] Loading existing origin.olang ({} bytes)",
            existing.len()
        );
        OlangWriter::from_existing(existing)
    } else {
        println!("[seed_domains] No existing file — creating new origin.olang");
        OlangWriter::new(ts)
    };

    let mut registry = Registry::new();
    let mut log = EventLog::new(String::from("origin.olang.log"));

    // Layer 4 = domain knowledge
    let layer = 4u8;

    let mut total_nodes = 0usize;
    let mut total_edges = 0usize;

    // ── Math ────────────────────────────────────────────────────────────────
    println!("\n=== MATH (LaTeX + Unicode) ===");
    let math_nodes: Vec<&domains::SeedNode> = domains::math::all_nodes();
    let math_nodes_owned: Vec<domains::SeedNode> = math_nodes
        .iter()
        .map(|n| domains::SeedNode {
            name: n.name,
            codepoint: n.codepoint,
            aliases: n.aliases,
        })
        .collect();
    let (n, e) = domains::seed_domain(
        "math",
        &math_nodes_owned,
        domains::math::MATH_EDGES,
        layer,
        &mut writer,
        &mut registry,
        &mut log,
        ts,
    );
    total_nodes += n;
    total_edges += e;

    // ── Physics ─────────────────────────────────────────────────────────────
    println!("\n=== PHYSICS ===");
    let phys_nodes: Vec<&domains::SeedNode> = domains::physics::all_nodes();
    let phys_owned: Vec<domains::SeedNode> = phys_nodes
        .iter()
        .map(|n| domains::SeedNode {
            name: n.name,
            codepoint: n.codepoint,
            aliases: n.aliases,
        })
        .collect();
    let (n, e) = domains::seed_domain(
        "physics",
        &phys_owned,
        domains::physics::PHYSICS_EDGES,
        layer,
        &mut writer,
        &mut registry,
        &mut log,
        ts,
    );
    total_nodes += n;
    total_edges += e;

    // ── Chemistry ───────────────────────────────────────────────────────────
    println!("\n=== CHEMISTRY ===");
    let chem_nodes: Vec<&domains::SeedNode> = domains::chemistry::all_nodes();
    let chem_owned: Vec<domains::SeedNode> = chem_nodes
        .iter()
        .map(|n| domains::SeedNode {
            name: n.name,
            codepoint: n.codepoint,
            aliases: n.aliases,
        })
        .collect();
    let (n, e) = domains::seed_domain(
        "chemistry",
        &chem_owned,
        domains::chemistry::CHEMISTRY_EDGES,
        layer,
        &mut writer,
        &mut registry,
        &mut log,
        ts,
    );
    total_nodes += n;
    total_edges += e;

    // ── Biology ─────────────────────────────────────────────────────────────
    println!("\n=== BIOLOGY ===");
    let bio_nodes: Vec<&domains::SeedNode> = domains::biology::all_nodes();
    let bio_owned: Vec<domains::SeedNode> = bio_nodes
        .iter()
        .map(|n| domains::SeedNode {
            name: n.name,
            codepoint: n.codepoint,
            aliases: n.aliases,
        })
        .collect();
    let (n, e) = domains::seed_domain(
        "biology",
        &bio_owned,
        domains::biology::BIOLOGY_EDGES,
        layer,
        &mut writer,
        &mut registry,
        &mut log,
        ts,
    );
    total_nodes += n;
    total_edges += e;

    // ── Philosophy ──────────────────────────────────────────────────────────
    println!("\n=== PHILOSOPHY ===");
    let phil_nodes: Vec<&domains::SeedNode> = domains::philosophy::all_nodes();
    let phil_owned: Vec<domains::SeedNode> = phil_nodes
        .iter()
        .map(|n| domains::SeedNode {
            name: n.name,
            codepoint: n.codepoint,
            aliases: n.aliases,
        })
        .collect();
    let (n, e) = domains::seed_domain(
        "philosophy",
        &phil_owned,
        domains::philosophy::PHILOSOPHY_EDGES,
        layer,
        &mut writer,
        &mut registry,
        &mut log,
        ts,
    );
    total_nodes += n;
    total_edges += e;

    // ── Algorithms ──────────────────────────────────────────────────────────
    println!("\n=== ALGORITHMS ===");
    let algo_nodes: Vec<&domains::SeedNode> = domains::algorithms::all_nodes();
    let algo_owned: Vec<domains::SeedNode> = algo_nodes
        .iter()
        .map(|n| domains::SeedNode {
            name: n.name,
            codepoint: n.codepoint,
            aliases: n.aliases,
        })
        .collect();
    let (n, e) = domains::seed_domain(
        "algorithms",
        &algo_owned,
        domains::algorithms::ALGORITHM_EDGES,
        layer,
        &mut writer,
        &mut registry,
        &mut log,
        ts,
    );
    total_nodes += n;
    total_edges += e;

    // ── Summary ─────────────────────────────────────────────────────────────
    println!("\n[seed_domains] ════════════════════════════════");
    println!("[seed_domains] Total nodes : {}", total_nodes);
    println!("[seed_domains] Total edges : {}", total_edges);
    println!("[seed_domains] Registry    : {} entries", registry.len());
    println!("[seed_domains] Aliases     : {}", registry.alias_count());
    println!("[seed_domains] File size   : {} bytes", writer.size());

    // Write file
    let bytes = writer.as_bytes().to_vec();
    fs::write("origin.olang", &bytes).expect("write origin.olang");
    println!("[seed_domains] ✓ origin.olang ({} bytes)", bytes.len());

    // Verify roundtrip
    let reader = olang::reader::OlangReader::new(&bytes).expect("parse");
    let parsed = reader.parse_all().expect("parse all");
    println!(
        "[seed_domains] ✓ Roundtrip: {} nodes, {} edges, {} aliases",
        parsed.node_count(),
        parsed.edges.len(),
        parsed.alias_count()
    );

    println!("[seed_domains] Done ✓");
}
