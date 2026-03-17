//! # silk-bench — Benchmark 3-Layer Silk Architecture
//!
//! So sánh Old (SilkEdge only) vs New (Implicit + HebbianLink + SilkEdge):
//!   1. Dung lượng (Storage)  — bytes per edge tại scale 1K → 500M
//!   2. Tốc độ (Speed)        — ops/s cho mọi operation
//!   3. RAM thực tế (Memory)  — actual heap usage
//!
//! Chạy: cargo run -p bench --bin silk-bench

use std::time::Instant;

use silk::edge::{EmotionTag, HebbianLink, SilkEdge};
use silk::graph::{MolSummary, SilkGraph, SilkNeighbor};
use silk::index::SilkIndex;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        ○ 3-Layer Silk Architecture Benchmark               ║");
    println!("║  Implicit (0B) + HebbianLink (19B) vs SilkEdge (46B)       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // ── 1. Storage ──────────────────────────────────────────────────────────
    bench_storage();

    // ── 2. Speed ────────────────────────────────────────────────────────────
    bench_speed();

    // ── 3. RAM ──────────────────────────────────────────────────────────────
    bench_ram();

    // ── 4. Projected 500M ───────────────────────────────────────────────────
    bench_projection();

    println!();
    println!("○ Silk benchmark complete.");
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn mol_for(i: u64) -> MolSummary {
    // Distribute across 5D space deterministically
    MolSummary {
        shape: ((i % 8) as u8) + 1,          // 8 shape bases
        relation: ((i / 8 % 8) as u8) + 1,   // 8 relation bases
        valence: ((i * 37) % 256) as u8,      // spread across zones
        arousal: ((i * 53) % 256) as u8,      // spread across zones
        time: ((i % 5) as u8) + 1,            // 5 time bases
    }
}

fn emo(i: u64) -> EmotionTag {
    EmotionTag::new(
        (i as f32 / 1000.0) * 2.0 - 1.0,
        0.5,
        0.5,
        0.7,
    )
}

fn bar(label: &str, value: f64, max: f64, width: usize) {
    let filled = ((value / max) * width as f64).min(width as f64) as usize;
    let bar_str: String = "█".repeat(filled) + &"░".repeat(width.saturating_sub(filled));
    print!("    {:<20} {:>12} {}", label, format_bytes(value as u64), bar_str);
    println!();
}

fn format_bytes(b: u64) -> String {
    if b >= 1_073_741_824 {
        format!("{:.2} GB", b as f64 / 1_073_741_824.0)
    } else if b >= 1_048_576 {
        format!("{:.2} MB", b as f64 / 1_048_576.0)
    } else if b >= 1024 {
        format!("{:.1} KB", b as f64 / 1024.0)
    } else {
        format!("{} B", b)
    }
}

fn format_ops(ops: f64) -> String {
    if ops >= 1_000_000.0 {
        format!("{:.1}M", ops / 1_000_000.0)
    } else if ops >= 1_000.0 {
        format!("{:.0}K", ops / 1_000.0)
    } else {
        format!("{:.0}", ops)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Storage — Per-edge size comparison
// ─────────────────────────────────────────────────────────────────────────────

fn bench_storage() {
    println!("══ 1. DUNG LƯỢNG (Storage) ════════════════════════════════════");
    println!();

    // Struct sizes
    let silk_edge_size = std::mem::size_of::<SilkEdge>();
    let hebbian_size = std::mem::size_of::<HebbianLink>();
    let silk_edge_wire = 46u64; // SilkEdge::to_bytes()
    let hebbian_wire = 19u64;   // HebbianLink::to_bytes()

    println!("  ── Per-edge comparison ──");
    println!("    {:20} {:>8} (RAM)  {:>8} (disk)", "Type", "bytes", "bytes");
    println!("    {}", "─".repeat(50));
    println!("    {:20} {:>8}        {:>8}", "SilkEdge (old)",
        silk_edge_size, silk_edge_wire);
    println!("    {:20} {:>8}        {:>8}", "HebbianLink (new)",
        hebbian_size, hebbian_wire);
    println!("    {:20} {:>8}        {:>8}", "Implicit (5D)", 0, 0);
    println!("    {:20} {:>8}        {:>8}", "MolSummary (index)",
        std::mem::size_of::<MolSummary>(), "5");
    println!();

    let savings_ram = 100.0 * (1.0 - hebbian_size as f64 / silk_edge_size as f64);
    let savings_disk = 100.0 * (1.0 - hebbian_wire as f64 / silk_edge_wire as f64);
    println!("    HebbianLink savings: RAM {:.0}%  Disk {:.0}%", savings_ram, savings_disk);
    println!();

    // Scale comparison
    println!("  ── Storage at scale ──");
    println!("    {:>12} {:>14} {:>14} {:>14} {:>10}", "Edges",
        "SilkEdge(46B)", "Hebbian(19B)", "Implicit(0B)", "Savings");
    println!("    {}", "─".repeat(70));

    for &n in &[1_000u64, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000] {
        let old = n * silk_edge_wire;
        let new = n * hebbian_wire;
        let save = 100.0 * (1.0 - new as f64 / old as f64);
        println!("    {:>12} {:>14} {:>14} {:>14} {:>9.0}%",
            format_count(n),
            format_bytes(old),
            format_bytes(new),
            format_bytes(0),
            save);
    }
    println!();

    // Visual bar chart for 10M edges
    let n = 10_000_000u64;
    let old_bytes = n * silk_edge_wire;
    let new_bytes = n * hebbian_wire;
    println!("  ── Visual: 10M edges ──");
    bar("SilkEdge (old)", old_bytes as f64, old_bytes as f64, 40);
    bar("HebbianLink (new)", new_bytes as f64, old_bytes as f64, 40);
    bar("Implicit (5D)", 0.0, old_bytes as f64, 40);
    println!();
}

fn format_count(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{}B", n / 1_000_000_000)
    } else if n >= 1_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        format!("{}", n)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Speed — Operation benchmarks
// ─────────────────────────────────────────────────────────────────────────────

fn bench_speed() {
    println!("══ 2. TỐC ĐỘ XỬ LÝ (Speed) ══════════════════════════════════");
    println!();

    // 2a. co_activate (old SilkEdge) vs learn (new HebbianLink)
    println!("  ── Insert: co_activate vs learn ──");
    {
        let n = 10_000u64;

        // Old: co_activate → SilkEdge
        let mut g_old = SilkGraph::new();
        let t = Instant::now();
        for i in 0..n {
            g_old.co_activate(i, (i + 1) % n, emo(i), 0.6, i as i64);
        }
        let old_time = t.elapsed();
        let old_ops = n as f64 / old_time.as_secs_f64();

        // New: learn → HebbianLink
        let mut g_new = SilkGraph::new();
        let t = Instant::now();
        for i in 0..n {
            g_new.learn(i, (i + 1) % n, 0.6);
        }
        let new_time = t.elapsed();
        let new_ops = n as f64 / new_time.as_secs_f64();

        // New: learn_mol → HebbianLink with 5D boost
        let mut g_mol = SilkGraph::new();
        let t = Instant::now();
        for i in 0..n {
            let ma = mol_for(i);
            let mb = mol_for((i + 1) % n);
            g_mol.learn_mol(i, (i + 1) % n, Some(&ma), Some(&mb), 0.6);
        }
        let mol_time = t.elapsed();
        let mol_ops = n as f64 / mol_time.as_secs_f64();

        println!("    {} inserts:", format_count(n));
        println!("    {:20} {:>10} ops/s  ({:.2}ms)", "co_activate (old)", format_ops(old_ops), old_time.as_micros() as f64 / 1000.0);
        println!("    {:20} {:>10} ops/s  ({:.2}ms)", "learn (new)", format_ops(new_ops), new_time.as_micros() as f64 / 1000.0);
        println!("    {:20} {:>10} ops/s  ({:.2}ms)", "learn_mol (new+5D)", format_ops(mol_ops), mol_time.as_micros() as f64 / 1000.0);

        let speedup = new_ops / old_ops;
        println!("    → learn is {:.1}× faster than co_activate", speedup);
        println!();
    }

    // 2b. Reinforce (existing edge strengthen)
    println!("  ── Reinforce: existing edge ──");
    {
        let n = 5_000u64;

        // Build graphs first
        let mut g_old = SilkGraph::new();
        let mut g_new = SilkGraph::new();
        for i in 0..n {
            g_old.co_activate(i, (i + 1) % n, emo(i), 0.5, i as i64);
            g_new.learn(i, (i + 1) % n, 0.5);
        }

        // Old: reinforce via co_activate
        let iters = 10u32;
        let t = Instant::now();
        for _ in 0..iters {
            for i in 0..n {
                g_old.co_activate(i, (i + 1) % n, emo(i), 0.8, (n + i) as i64);
            }
        }
        let old_time = t.elapsed();
        let old_ops = (iters as f64 * n as f64) / old_time.as_secs_f64();

        // New: reinforce via learn
        let t = Instant::now();
        for _ in 0..iters {
            for i in 0..n {
                g_new.learn(i, (i + 1) % n, 0.8);
            }
        }
        let new_time = t.elapsed();
        let new_ops = (iters as f64 * n as f64) / new_time.as_secs_f64();

        println!("    {} edges × {} reinforcements:", format_count(n), iters);
        println!("    {:20} {:>10} ops/s  ({:.2}ms)", "co_activate (old)", format_ops(old_ops), old_time.as_micros() as f64 / 1000.0);
        println!("    {:20} {:>10} ops/s  ({:.2}ms)", "learn (new)", format_ops(new_ops), new_time.as_micros() as f64 / 1000.0);
        println!("    → reinforce: {:.1}× speedup", new_ops / old_ops);
        println!();
    }

    // 2c. Lookup: edges_from vs unified_neighbors
    println!("  ── Lookup: edges_from vs unified_neighbors ──");
    {
        let n = 2_000u64;

        // Build graph with all 3 layers
        let mut graph = SilkGraph::new();
        for i in 0..n {
            let m = mol_for(i);
            graph.index_node(i, &m);
            graph.co_activate(i, (i + 1) % n, emo(i), 0.6, i as i64);
            graph.learn(i, (i + 1) % n, 0.6);
            // Cross connections
            if i % 10 == 0 {
                graph.co_activate(i, (i + 100) % n, EmotionTag::NEUTRAL, 0.3, i as i64);
                graph.learn(i, (i + 100) % n, 0.3);
            }
        }

        let query_nodes: Vec<u64> = (0..100).collect();
        let iters = 100u32;

        // Old: edges_from
        let t = Instant::now();
        for _ in 0..iters {
            for &h in &query_nodes {
                let _ = graph.edges_from(h);
            }
        }
        let old_time = t.elapsed();
        let old_ops = (iters as f64 * query_nodes.len() as f64) / old_time.as_secs_f64();

        // New: unified_neighbors
        let t = Instant::now();
        for _ in 0..iters {
            for &h in &query_nodes {
                let m = mol_for(h);
                let _ = graph.unified_neighbors(h, Some(&m));
            }
        }
        let new_time = t.elapsed();
        let new_ops = (iters as f64 * query_nodes.len() as f64) / new_time.as_secs_f64();

        println!("    {} queries × {} iterations:", query_nodes.len(), iters);
        println!("    {:20} {:>10} ops/s  ({:.2}ms)", "edges_from (old)", format_ops(old_ops), old_time.as_micros() as f64 / 1000.0);
        println!("    {:20} {:>10} ops/s  ({:.2}ms)", "unified_neighbors", format_ops(new_ops), new_time.as_micros() as f64 / 1000.0);

        // Sample neighbor counts
        let m0 = mol_for(0);
        let legacy = graph.edges_from(0);
        let unified = graph.unified_neighbors(0, Some(&m0));
        println!("    → Node 0: edges_from={} neighbors, unified={} neighbors", legacy.len(), unified.len());
        println!();
    }

    // 2d. Unified weight vs assoc_weight
    println!("  ── Weight query: assoc_weight vs unified_weight ──");
    {
        let n = 2_000u64;
        let mut graph = SilkGraph::new();
        for i in 0..n {
            let m = mol_for(i);
            graph.index_node(i, &m);
            graph.co_activate(i, (i + 1) % n, emo(i), 0.6, i as i64);
            graph.learn(i, (i + 1) % n, 0.6);
        }

        let pairs: Vec<(u64, u64)> = (0..500).map(|i| (i, (i + 1) % n)).collect();
        let iters = 500u32;

        // Old: assoc_weight
        let t = Instant::now();
        for _ in 0..iters {
            for &(a, b) in &pairs {
                let _ = graph.assoc_weight(a, b);
            }
        }
        let old_time = t.elapsed();
        let old_ops = (iters as f64 * pairs.len() as f64) / old_time.as_secs_f64();

        // New: unified_weight
        let t = Instant::now();
        for _ in 0..iters {
            for &(a, b) in &pairs {
                let ma = mol_for(a);
                let mb = mol_for(b);
                let _ = graph.unified_weight(a, b, Some(&ma), Some(&mb));
            }
        }
        let new_time = t.elapsed();
        let new_ops = (iters as f64 * pairs.len() as f64) / new_time.as_secs_f64();

        println!("    {} pairs × {} iterations:", pairs.len(), iters);
        println!("    {:20} {:>10} ops/s  ({:.2}ms)", "assoc_weight (old)", format_ops(old_ops), old_time.as_micros() as f64 / 1000.0);
        println!("    {:20} {:>10} ops/s  ({:.2}ms)", "unified_weight", format_ops(new_ops), new_time.as_micros() as f64 / 1000.0);
        println!();
    }

    // 2e. SilkIndex operations
    println!("  ── SilkIndex: index_node + implicit_silk ──");
    {
        // Index speed
        let n = 10_000u64;
        let mut idx = SilkIndex::new();
        let t = Instant::now();
        for i in 0..n {
            idx.index_node(i, &mol_for(i));
        }
        let index_time = t.elapsed();
        let index_ops = n as f64 / index_time.as_secs_f64();

        println!("    index_node:");
        println!("    {:20} {:>10} ops/s  ({:.2}ms for {})", "index_node", format_ops(index_ops), index_time.as_micros() as f64 / 1000.0, format_count(n));

        // implicit_silk speed (pure computation, O(1))
        let iters = 100_000u32;
        let a = mol_for(42);
        let b = mol_for(137);
        let t = Instant::now();
        for _ in 0..iters {
            let _ = SilkIndex::implicit_silk(&a, &b);
        }
        let silk_time = t.elapsed();
        let silk_ops = iters as f64 / silk_time.as_secs_f64();

        println!("    implicit_silk (O(1) 5D compare):");
        println!("    {:20} {:>10} ops/s  ({:.0}ns/call)", "implicit_silk", format_ops(silk_ops), silk_time.as_nanos() as f64 / iters as f64);

        // implicit_neighbors speed
        let iters_neigh = 1_000u32;
        let query = mol_for(42);
        let t = Instant::now();
        for _ in 0..iters_neigh {
            let _ = idx.implicit_neighbors(42, &query);
        }
        let neigh_time = t.elapsed();
        let neigh_ops = iters_neigh as f64 / neigh_time.as_secs_f64();

        println!("    implicit_neighbors ({} indexed nodes):", format_count(n));
        println!("    {:20} {:>10} ops/s  ({:.1}µs/call)", "implicit_neighbors", format_ops(neigh_ops), neigh_time.as_micros() as f64 / iters_neigh as f64);
        println!();
    }

    // 2f. Decay
    println!("  ── Decay: decay_all vs decay_learned ──");
    {
        let n = 5_000u64;
        let mut g_old = SilkGraph::new();
        let mut g_new = SilkGraph::new();
        for i in 0..n {
            g_old.co_activate(i, (i + 1) % n, emo(i), 0.6, i as i64);
            g_new.learn(i, (i + 1) % n, 0.6);
        }

        let elapsed_ns = 3_600_000_000_000i64; // 1 hour

        let t = Instant::now();
        g_old.decay_all(elapsed_ns);
        let old_time = t.elapsed();

        let t = Instant::now();
        g_new.decay_learned(elapsed_ns);
        let new_time = t.elapsed();

        println!("    Decay {} edges (1h elapsed):", format_count(n));
        println!("    {:20} {:.2}ms", "decay_all (old)", old_time.as_micros() as f64 / 1000.0);
        println!("    {:20} {:.2}ms", "decay_learned (new)", new_time.as_micros() as f64 / 1000.0);
        println!();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. RAM — Actual memory usage
// ─────────────────────────────────────────────────────────────────────────────

fn bench_ram() {
    println!("══ 3. RAM THỰC TẾ (Memory) ═══════════════════════════════════");
    println!();

    // Struct sizes
    println!("  ── Struct sizes (stack) ──");
    println!("    {:25} {:>6} bytes", "SilkEdge", std::mem::size_of::<SilkEdge>());
    println!("    {:25} {:>6} bytes", "HebbianLink", std::mem::size_of::<HebbianLink>());
    println!("    {:25} {:>6} bytes", "EmotionTag", std::mem::size_of::<EmotionTag>());
    println!("    {:25} {:>6} bytes", "MolSummary", std::mem::size_of::<MolSummary>());
    println!("    {:25} {:>6} bytes", "SilkNeighbor", std::mem::size_of::<SilkNeighbor>());
    println!("    {:25} {:>6} bytes", "SilkGraph (stack)", std::mem::size_of::<SilkGraph>());
    println!("    {:25} {:>6} bytes", "SilkIndex (stack)", std::mem::size_of::<SilkIndex>());
    println!();

    // Actual heap usage at different scales
    println!("  ── Heap usage at scale ──");
    println!("    {:>8} {:>14} {:>14} {:>14} {:>14}", "Nodes",
        "Old(edges)", "New(learned)", "Index(5D)", "Total New");
    println!("    {}", "─".repeat(70));

    for &n in &[100u64, 1_000, 5_000, 10_000, 50_000] {
        // Old: only SilkEdge
        let mut g_old = SilkGraph::new();
        for i in 0..n {
            g_old.co_activate(i, (i + 1) % n, emo(i), 0.6, i as i64);
            if i % 5 == 0 {
                g_old.co_activate(i, (i + 50) % n, EmotionTag::NEUTRAL, 0.3, i as i64);
            }
        }
        let old_mem = g_old.memory_usage();

        // New: HebbianLink + SilkIndex
        let mut g_new = SilkGraph::new();
        for i in 0..n {
            let m = mol_for(i);
            g_new.index_node(i, &m);
            g_new.learn(i, (i + 1) % n, 0.6);
            if i % 5 == 0 {
                g_new.learn(i, (i + 50) % n, 0.3);
            }
        }
        let idx_mem = g_new.index().memory_usage();
        let learned_mem = g_new.learned_count() * std::mem::size_of::<HebbianLink>();
        let new_total = idx_mem + learned_mem;

        println!("    {:>8} {:>14} {:>14} {:>14} {:>14}",
            format_count(n),
            format_bytes(old_mem as u64),
            format_bytes(learned_mem as u64),
            format_bytes(idx_mem as u64),
            format_bytes(new_total as u64));
    }
    println!();

    // Detailed SilkIndex memory breakdown
    println!("  ── SilkIndex memory breakdown ──");
    {
        let mut idx = SilkIndex::new();
        for i in 0u64..10_000 {
            idx.index_node(i, &mol_for(i));
        }
        println!("    10K nodes indexed:");
        println!("    Total memory      : {}", format_bytes(idx.memory_usage() as u64));
        println!("    Per-node overhead  : {:.1} bytes", idx.memory_usage() as f64 / 10_000.0);
        println!("    Node count         : {}", idx.node_count());
        println!();

        // Bucket distribution
        println!("    Bucket distribution (top 10):");
        let mut stats = idx.bucket_stats();
        stats.sort_by(|a, b| b.2.cmp(&a.2));
        for (dim, bucket, count) in stats.iter().take(10) {
            println!("      {:<10} [{}]: {:>6} nodes", dim, bucket, count);
        }
    }
    println!();

    // Edge-per-node ratio analysis
    println!("  ── Edge/node ratio impact ──");
    println!("    {:>8} {:>6} {:>14} {:>14} {:>10}", "Nodes", "E/N", "Old RAM", "New RAM", "Savings");
    println!("    {}", "─".repeat(58));

    let n = 10_000u64;
    for &ratio in &[1u64, 3, 5, 10, 25] {
        let old_size = n * ratio * std::mem::size_of::<SilkEdge>() as u64;
        // New: only ~20% of connections need HebbianLink, rest are implicit
        let hebbian_count = n * ratio * 20 / 100; // 20% learned
        let new_size = hebbian_count * std::mem::size_of::<HebbianLink>() as u64
            + 10_000 * 5 * 8; // index overhead ~40B/node (5 buckets × 8B)
        let savings = 100.0 * (1.0 - new_size as f64 / old_size as f64);
        println!("    {:>8} {:>6} {:>14} {:>14} {:>9.0}%",
            format_count(n), ratio, format_bytes(old_size), format_bytes(new_size), savings);
    }
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Projection — 500M concepts (phone target)
// ─────────────────────────────────────────────────────────────────────────────

fn bench_projection() {
    println!("══ 4. DỰ TOÁN 500M CONCEPTS (Phone Target) ═══════════════════");
    println!();

    let concepts = 500_000_000u64;
    let avg_edges = 3u64; // average connections per node

    // Node storage (33 bytes per concept: 5 mol + 8 hash + 20 metadata)
    let node_bytes = concepts * 33;

    // Old model: all edges are SilkEdge (46 bytes)
    let old_edge_bytes = concepts * avg_edges * 46;
    let old_total = node_bytes + old_edge_bytes;

    // New model: 80% implicit (0B) + 15% HebbianLink (19B) + 5% SilkEdge (46B)
    let implicit_count = concepts * avg_edges * 80 / 100;
    let hebbian_count = concepts * avg_edges * 15 / 100;
    let structural_count = concepts * avg_edges * 5 / 100;

    let implicit_bytes = implicit_count * 0; // 0 bytes!
    let hebbian_bytes = hebbian_count * 19;
    let structural_bytes = structural_count * 46;
    let new_edge_bytes = implicit_bytes + hebbian_bytes + structural_bytes;

    // Index overhead: ~40 bytes per node (5 buckets × 8 bytes per hash entry)
    let index_bytes = concepts * 40;
    let new_total = node_bytes + new_edge_bytes + index_bytes;

    println!("  ── 500M concepts × {} avg edges ──", avg_edges);
    println!();
    println!("    OLD MODEL (SilkEdge only):");
    println!("      Nodes        : {} × 33B = {}", format_count(concepts), format_bytes(node_bytes));
    println!("      Edges        : {} × 46B = {}", format_count(concepts * avg_edges), format_bytes(old_edge_bytes));
    println!("      ────────────────────────────────────");
    println!("      TOTAL        : {}", format_bytes(old_total));
    println!();

    println!("    NEW MODEL (3-Layer Silk):");
    println!("      Nodes        : {} × 33B = {}", format_count(concepts), format_bytes(node_bytes));
    println!("      Implicit     : {} × 0B  = {}", format_count(implicit_count), format_bytes(implicit_bytes));
    println!("      HebbianLink  : {} × 19B = {}", format_count(hebbian_count), format_bytes(hebbian_bytes));
    println!("      Structural   : {} × 46B = {}", format_count(structural_count), format_bytes(structural_bytes));
    println!("      5D Index     : {} × 40B = {}", format_count(concepts), format_bytes(index_bytes));
    println!("      ────────────────────────────────────");
    println!("      TOTAL        : {}", format_bytes(new_total));
    println!();

    let savings_pct = 100.0 * (1.0 - new_total as f64 / old_total as f64);
    let fits_phone = new_total < 16_000_000_000; // 16GB

    println!("    SAVINGS:");
    println!("      Old total    : {}", format_bytes(old_total));
    println!("      New total    : {}", format_bytes(new_total));
    println!("      Reduced      : {} ({:.0}%)", format_bytes(old_total - new_total), savings_pct);
    println!("      Fits 16GB?   : {} {}", if fits_phone { "YES" } else { "NO" },
        if fits_phone { "✓" } else { "✗" });
    println!();

    // Visual comparison
    let max_gb = old_total as f64 / 1_073_741_824.0;
    let new_gb = new_total as f64 / 1_073_741_824.0;
    let bar_width = 50;

    println!("    ── Visual: Total Storage ──");
    let old_fill = bar_width;
    let new_fill = ((new_gb / max_gb) * bar_width as f64) as usize;
    let limit_fill = ((16.0 / max_gb) * bar_width as f64).min(bar_width as f64) as usize;

    print!("      Old  ");
    for _ in 0..old_fill { print!("█"); }
    println!(" {:.1} GB", max_gb);

    print!("      New  ");
    for i in 0..bar_width {
        if i < new_fill { print!("█"); } else { print!("░"); }
    }
    println!(" {:.1} GB", new_gb);

    print!("      16GB ");
    for i in 0..bar_width {
        if i == limit_fill { print!("│"); } else { print!(" "); }
    }
    println!(" ← phone limit");

    println!();

    // Optimized model — DISK vs RAM separation
    // Key insight: SilkIndex is runtime-only (rebuilt from molecules on boot).
    // On DISK: only nodes + learned edges. Index = computed from 5D bytes.
    println!("  ── OPTIMIZED MODEL (Silk = formula, not data) ──");
    println!();
    {
        // Disk: compact node (tagged sparse mol avg ~3B + 8 hash + 8 ts = 19B)
        // Per CLAUDE.md: "1 concept = ~33 bytes (5 mol + 8 hash + 20 metadata)"
        // With tagged sparse: ~3B avg mol + 8 hash + 8 ts = 19B
        let compact_node = 19u64;
        let node_disk = concepts * compact_node;

        // 95% implicit (0B disk) + 4% HebbianLink (19B) + 1% structural parent ptr (8B hash-pair)
        let hebbian_count = concepts * avg_edges * 4 / 100;
        let structural_count = concepts * avg_edges * 1 / 100;
        let hebb_disk = hebbian_count * 19;
        // Structural = parent pointer pairs, compact: from_hash(8)+to_hash(8)+kind(1)=17B
        let struct_disk = structural_count * 17;
        // No index on disk — SilkIndex is rebuilt from node molecules at boot

        let disk_total = node_disk + hebb_disk + struct_disk;
        let fits_disk = disk_total < 16_000_000_000;

        println!("    ── DISK (persistent) ──");
        println!("    Compact nodes  : {} × {}B = {}", format_count(concepts), compact_node, format_bytes(node_disk));
        println!("    Implicit edges : {} × 0B  = 0 B (95% — computed from 5D)", format_count(concepts * avg_edges * 95 / 100));
        println!("    HebbianLink    : {} × 19B = {} (4%)", format_count(hebbian_count), format_bytes(hebb_disk));
        println!("    Parent pointers: {} × 17B = {} (1%)", format_count(structural_count), format_bytes(struct_disk));
        println!("    5D Index       : 0 B (rebuilt at boot from mol bytes)");
        println!("    ────────────────────────────────────");
        println!("    DISK TOTAL     : {} {}", format_bytes(disk_total), if fits_disk { "✓ FITS 16GB" } else { "✗" });
        println!();

        // RAM: only active working set + index for active nodes
        let active_pct = 2u64; // 2% active at any time
        let active = concepts * active_pct / 100;
        let ram_nodes = active * compact_node;
        let ram_index = active * 40; // 5D index for active nodes
        let ram_hebbian = (hebbian_count * active_pct / 100) * std::mem::size_of::<HebbianLink>() as u64;
        let ram_total = ram_nodes + ram_index + ram_hebbian;

        println!("    ── RAM ({}% active = {} nodes) ──", active_pct, format_count(active));
        println!("    Active nodes   : {}", format_bytes(ram_nodes));
        println!("    5D Index       : {}", format_bytes(ram_index));
        println!("    Active Hebbian : {}", format_bytes(ram_hebbian));
        println!("    ────────────────────────────────────");
        println!("    RAM TOTAL      : {}", format_bytes(ram_total));
        println!();
    }

    // RAM estimate for active working set
    println!("  ── RAM Working Set (active subset) ──");
    for &active_pct in &[1u64, 5, 10] {
        let active = concepts * active_pct / 100;
        let active_edges = active * avg_edges;
        let old_ram = active * 33 + active_edges * std::mem::size_of::<SilkEdge>() as u64;
        let new_ram = active * 33 + (active_edges * 20 / 100) * std::mem::size_of::<HebbianLink>() as u64
            + active * 40; // index
        println!("    {}% active ({}):", active_pct, format_count(active));
        println!("      Old RAM: {}   New RAM: {}   Savings: {:.0}%",
            format_bytes(old_ram), format_bytes(new_ram),
            100.0 * (1.0 - new_ram as f64 / old_ram as f64));
    }
    println!();
}
