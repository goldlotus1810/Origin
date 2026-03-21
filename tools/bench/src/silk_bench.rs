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
    println!("══ 4. DỰ TOÁN 2B NODES (Phone 16GB Target) ═══════════════════");
    println!();

    // ── Layer breakdown from document ──
    // L0: 5400 UCD nodes (fixed formulas)
    // L1: 37 nodes (LCA of L0 buckets)
    // L2→Ln-1: 2,000,000,000 nodes (learned knowledge)
    // Silk: 72 horizontal (implicit, 0B) + vertical parent pointers

    let l0_nodes = 5_400u64;
    let l1_nodes = 37u64;
    let l2_ln_nodes = 2_000_000_000u64; // 2B as specified by user
    let total_nodes = l0_nodes + l1_nodes + l2_ln_nodes;
    let avg_edges = 3u64;

    println!("  ── Layer distribution ──");
    println!("    L0 (UCD)       : {:>12} nodes (fixed formulas)", format_count(l0_nodes));
    println!("    L1 (LCA)       : {:>12} nodes (37 base concepts)", format_count(l1_nodes));
    println!("    L2→Ln-1        : {:>12} nodes (learned knowledge)", format_count(l2_ln_nodes));
    println!("    Total          : {:>12} nodes", format_count(total_nodes));
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // Model A: OLD — SilkEdge only, 33B/node
    // ══════════════════════════════════════════════════════════════════════
    let old_node_bytes = total_nodes * 33;
    let old_edge_bytes = total_nodes * avg_edges * 46;
    let old_total = old_node_bytes + old_edge_bytes;

    println!("    ╔═══════════════════════════════════════════════════╗");
    println!("    ║  MODEL A: OLD (SilkEdge, 33B/node, 46B/edge)    ║");
    println!("    ╚═══════════════════════════════════════════════════╝");
    println!("    Nodes          : {} × 33B = {}", format_count(total_nodes), format_bytes(old_node_bytes));
    println!("    Edges          : {} × 46B = {}", format_count(total_nodes * avg_edges), format_bytes(old_edge_bytes));
    println!("    ────────────────────────────────────────────────────");
    println!("    TOTAL          : {}  ✗", format_bytes(old_total));
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // Model B: 3-Layer Silk, compact 19B/node
    // ══════════════════════════════════════════════════════════════════════
    let b_node_bytes = total_nodes * 19; // tagged sparse avg
    let b_hebbian = total_nodes * avg_edges * 4 / 100; // 4% learned
    let b_structural = total_nodes * avg_edges / 100; // 1% parent ptrs
    let b_hebb_bytes = b_hebbian * 19;
    let b_struct_bytes = b_structural * 17;
    let b_total = b_node_bytes + b_hebb_bytes + b_struct_bytes;

    println!("    ╔═══════════════════════════════════════════════════╗");
    println!("    ║  MODEL B: 3-Layer Silk (compact, 19B/node)       ║");
    println!("    ╚═══════════════════════════════════════════════════╝");
    println!("    Nodes          : {} × 19B = {}", format_count(total_nodes), format_bytes(b_node_bytes));
    println!("    Implicit Silk  : {} × 0B  = 0 B (95%)", format_count(total_nodes * avg_edges * 95 / 100));
    println!("    HebbianLink    : {} × 19B = {} (4%)", format_count(b_hebbian), format_bytes(b_hebb_bytes));
    println!("    Parent ptrs    : {} × 17B = {} (1%)", format_count(b_structural), format_bytes(b_struct_bytes));
    println!("    Index on disk  : 0 B (rebuilt from mol bytes at boot)");
    println!("    ────────────────────────────────────────────────────");
    println!("    TOTAL          : {}  {}", format_bytes(b_total),
        if b_total < 16_000_000_000 { "✓ FITS 16GB" } else { "✗ VƯỢT 16GB" });
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // Model C: FORMULA + DELTA — Molecule = công thức, not data
    // ══════════════════════════════════════════════════════════════════════
    //
    // From doc "node va silk.md":
    //   L0 = 5400 formulas × 10B
    //   L2+ = compose(ref_L0, ref_L0) + op = 5B (formula ref)
    //   OR delta from parent = 2-3B (90% of L4+ nodes)
    //
    // compact.rs already has DeltaMolecule: [bitmask:1B][changed:1-2B] = 2-3B
    //   → 60% savings vs full for nodes that only vary in V/A

    // L0: full formula (10B each)
    let c_l0 = l0_nodes * 10;
    // L1: LCA results (5B tagged each)
    let c_l1 = l1_nodes * 5;
    // L2-L3 concepts (~2.5% of L2+): compose refs = 5B
    let l2_l3_count = l2_ln_nodes * 25 / 1000; // 2.5%
    let c_l2_l3 = l2_l3_count * 5;
    // L4+ learned (~97.5% of L2+): delta from parent = avg 2.5B
    let l4_plus_count = l2_ln_nodes - l2_l3_count;
    let c_l4_plus = l4_plus_count * 25 / 10; // 2.5B avg
    // chain_hash per node: 8B (needed for lookup)
    let c_hashes = total_nodes * 8;
    // Parent pointers: each L2+ node has 1 parent = varint 4B avg
    let c_parents = l2_ln_nodes * 4;
    // HebbianLink: 2% of edges (only strong discovered connections)
    let c_hebbian_count = l2_ln_nodes * avg_edges * 2 / 100;
    let c_hebb = c_hebbian_count * 19;

    let c_total = c_l0 + c_l1 + c_l2_l3 + c_l4_plus + c_hashes + c_parents + c_hebb;

    println!("    ╔═══════════════════════════════════════════════════╗");
    println!("    ║  MODEL C: FORMULA + DELTA (Molecule = công thức) ║");
    println!("    ╚═══════════════════════════════════════════════════╝");
    println!("    L0 formulas    : {:>5} × 10B  = {}", format_count(l0_nodes), format_bytes(c_l0));
    println!("    L1 LCA         : {:>5} × 5B   = {}", format_count(l1_nodes), format_bytes(c_l1));
    println!("    L2-L3 compose  : {} × 5B   = {} (2.5%)", format_count(l2_l3_count), format_bytes(c_l2_l3));
    println!("    L4+ delta      : {} × 2.5B = {} (97.5%)", format_count(l4_plus_count), format_bytes(c_l4_plus));
    println!("    chain_hash     : {} × 8B   = {}", format_count(total_nodes), format_bytes(c_hashes));
    println!("    Parent ptrs    : {} × 4B   = {}", format_count(l2_ln_nodes), format_bytes(c_parents));
    println!("    Implicit Silk  : {} × 0B   = 0 B (98%)", format_count(l2_ln_nodes * avg_edges * 98 / 100));
    println!("    HebbianLink    : {} × 19B  = {} (2%)", format_count(c_hebbian_count), format_bytes(c_hebb));
    println!("    ────────────────────────────────────────────────────");
    println!("    DISK TOTAL     : {}  {}", format_bytes(c_total),
        if c_total < 16_000_000_000 { "✓ FITS 16GB" } else { "✗ VƯỢT 16GB" });
    println!("    Dư còn         : {}", format_bytes(16_000_000_000u64.saturating_sub(c_total)));
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // Model D: ZERO-HASH — hash = computed from formula, not stored
    // ══════════════════════════════════════════════════════════════════════
    //
    // Key insight: if Molecule = FORMULA, then chain_hash = f(formula).
    // Hash is DETERMINISTIC from the formula → no need to store it!
    // Recompute on demand: hash(parent_hash XOR delta_bytes) = O(1)
    //
    // Node on disk = [parent_offset:3B][delta_mask:1B][changed:0-2B]
    //              = 4-6 bytes (no hash stored!)
    // Parent offset: 3B supports up to 16M nodes per level-group (varint)

    let d_l0 = l0_nodes * 10; // full formulas
    let d_l1 = l1_nodes * 5;
    // L2-L3: compose refs (no hash) = [ref1:2B][ref2:2B][op:1B] = 5B
    let d_l2_l3 = l2_l3_count * 5;
    // L4+: delta (no hash) = [parent_offset:3B][mask:1B][changed:0-2B] = avg 4.5B
    let d_l4_plus = l4_plus_count * 45 / 10; // 4.5B avg
    // Hash: 0 bytes stored! Computed from formula on demand.
    // HebbianLink: still needs from_hash + to_hash (but these are runtime-only)
    // On disk: HebbianLink stored as [parent_offset_a:3B][parent_offset_b:3B][weight:1B] = 7B
    let d_hebbian_count = l2_ln_nodes * avg_edges * 2 / 100;
    let d_hebb = d_hebbian_count * 7; // compact HebbianLink on disk

    let d_total = d_l0 + d_l1 + d_l2_l3 + d_l4_plus + d_hebb;

    println!("    ╔═══════════════════════════════════════════════════╗");
    println!("    ║  MODEL D: ZERO-HASH (hash = computed, not stored)║");
    println!("    ╚═══════════════════════════════════════════════════╝");
    println!("    L0 formulas    : {:>5} × 10B   = {}", format_count(l0_nodes), format_bytes(d_l0));
    println!("    L1 LCA         : {:>5} × 5B    = {}", format_count(l1_nodes), format_bytes(d_l1));
    println!("    L2-L3 compose  : {} × 5B    = {} (2.5%)", format_count(l2_l3_count), format_bytes(d_l2_l3));
    println!("    L4+ delta      : {} × 4.5B  = {} (97.5%)", format_count(l4_plus_count), format_bytes(d_l4_plus));
    println!("    chain_hash     : 0 B (computed from formula — KHÔNG LƯU)");
    println!("    Parent ptrs    : implicit in parent_offset (3B, included above)");
    println!("    Implicit Silk  : {} × 0B    = 0 B (98%)", format_count(l2_ln_nodes * avg_edges * 98 / 100));
    println!("    HebbianLink    : {} × 7B    = {} (compact, 2%)", format_count(d_hebbian_count), format_bytes(d_hebb));
    println!("    ────────────────────────────────────────────────────");
    println!("    DISK TOTAL     : {}  {}", format_bytes(d_total),
        if d_total < 16_000_000_000 { "✓ FITS 16GB" } else { "✗ VƯỢT 16GB" });
    println!("    Dư còn         : {} (cho aliases, runtime, evolution)",
        format_bytes(16_000_000_000u64.saturating_sub(d_total)));
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // Model E: COMPACT-QR — QR nodes compressed to 2 bytes
    // ══════════════════════════════════════════════════════════════════════
    //
    // From CompactQR (molecular.rs):
    //   16 bits = [shape:3][relation:3][time:3][valence:4][arousal:3]
    //   Full 5D position in 2 bytes. Hash computed from 2B on demand.
    //   Mature QR nodes → CompactQR → L2→Ln-1 cold storage.
    //   L0 + L1 stay full resolution for active processing.
    //
    // Lifecycle: Formula → Evaluating → Mature → QR → CompactQR
    //   L0: 5400 full formulas (10B) — always hot
    //   L1: ~50K LCA results (5B tagged) — Memory working set
    //   L2→Ln: 2B CompactQR nodes (2B each!) — cold archive
    //   HebbianLink: only top 0.5% strongest connections stored

    let e_l0 = l0_nodes * 10; // full formulas
    let e_l1 = l1_nodes * 5;  // LCA tagged
    // L2→Ln: CompactQR = 2 bytes per node!
    let e_compact = l2_ln_nodes * 2;
    // HebbianLink: only 0.5% strongest (rest implicit from CompactQR silk_compare)
    let e_hebbian_count = l2_ln_nodes * avg_edges * 5 / 1000; // 0.5%
    let e_hebb = e_hebbian_count * 7; // compact on-disk format
    // Optional: bloom filter for fast hash→offset lookup = 1 bit/node
    let e_bloom = total_nodes / 8; // 1 bit per node

    let e_total = e_l0 + e_l1 + e_compact + e_hebb + e_bloom;

    println!("    ╔═══════════════════════════════════════════════════╗");
    println!("    ║  MODEL E: COMPACT-QR (QR → 2 bytes, L2→Ln)      ║");
    println!("    ╚═══════════════════════════════════════════════════╝");
    println!("    L0 formulas    : {:>5} × 10B  = {}", format_count(l0_nodes), format_bytes(e_l0));
    println!("    L1 LCA (Memory): {:>5} × 5B   = {}", format_count(l1_nodes), format_bytes(e_l1));
    println!("    L2→Ln CompactQR: {} × 2B   = {} ← DNA tri thức", format_count(l2_ln_nodes), format_bytes(e_compact));
    println!("    Silk implicit  : {} × 0B   = 0 B (silk_compare O(1))", format_count(l2_ln_nodes * avg_edges * 995 / 1000));
    println!("    HebbianLink    : {} × 7B   = {} (top 0.5%)", format_count(e_hebbian_count), format_bytes(e_hebb));
    println!("    Bloom filter   : {} (1 bit/node)", format_bytes(e_bloom));
    println!("    chain_hash     : 0 B (compute_hash() từ 2B — deterministic)");
    println!("    ────────────────────────────────────────────────────");
    println!("    DISK TOTAL     : {}  {}", format_bytes(e_total),
        if e_total < 16_000_000_000 { "✓ FITS 16GB" } else { "✗ VƯỢT 16GB" });
    println!("    Dư còn         : {} (cho aliases, runtime, evolution, logs)",
        format_bytes(16_000_000_000u64.saturating_sub(e_total)));
    println!("    Nén so với Old : {:.0}× nhỏ hơn", old_total as f64 / e_total as f64);
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // Visual comparison 5 models
    // ══════════════════════════════════════════════════════════════════════
    let bar_width = 50;
    let max_val = old_total as f64;

    println!("    ── Visual: DISK Storage (2B nodes) ──");
    print_bar("  Old (33B+46B)", old_total, max_val, bar_width);
    print_bar("  3-Layer (19B)", b_total, max_val, bar_width);
    print_bar("  Formula+Delta", c_total, max_val, bar_width);
    print_bar("  Zero-Hash(4.5B)", d_total, max_val, bar_width);
    print_bar("  CompactQR (2B)", e_total, max_val, bar_width);
    // 16GB line
    print!("      16GB limit ");
    let limit_pos = ((16_000_000_000.0 / max_val) * bar_width as f64).min(bar_width as f64) as usize;
    for i in 0..bar_width {
        if i == limit_pos { print!("│"); } else { print!("─"); }
    }
    println!();
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // RAM Working Set for 2B nodes
    // ══════════════════════════════════════════════════════════════════════
    println!("    ── RAM Working Set (2B nodes, Model D) ──");
    println!("    {:>6} {:>12} {:>14} {:>14} {:>14}", "%Active", "Nodes",
        "Old RAM", "ZeroHash RAM", "Savings");
    println!("    {}", "─".repeat(65));

    for &pct in &[1u64, 2, 5] {
        let active = total_nodes * pct / 100;
        let active_edges = active * avg_edges;

        // Old: 33B/node + 46B/edge (all explicit)
        let old_ram = active * 33 + active_edges * 46;

        // Model D in RAM: nodes need hash at runtime (recomputed), delta + index
        // Active node in RAM: 4.5B delta + 8B computed hash + 40B index = ~52.5B
        let new_ram = active * 53 // delta + hash (runtime) + index
            + (active_edges * 2 / 100) * std::mem::size_of::<HebbianLink>() as u64; // 2% hebbian

        let savings = 100.0 * (1.0 - new_ram as f64 / old_ram as f64);
        println!("    {:>5}% {:>12} {:>14} {:>14} {:>13.0}%",
            pct, format_count(active), format_bytes(old_ram), format_bytes(new_ram), savings);
    }
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // Summary table
    // ══════════════════════════════════════════════════════════════════════
    println!("    ── Tóm tắt ──");
    println!("    {:30} {:>12} {:>10}", "Model", "Disk", "Fits 16GB?");
    println!("    {}", "─".repeat(55));
    println!("    {:30} {:>12} {:>10}", "A: Old (SilkEdge)",
        format_bytes(old_total), "✗");
    println!("    {:30} {:>12} {:>10}", "B: 3-Layer (compact 19B)",
        format_bytes(b_total), if b_total < 16_000_000_000 { "✓" } else { "✗" });
    println!("    {:30} {:>12} {:>10}", "C: Formula + Delta (hash lưu)",
        format_bytes(c_total), if c_total < 16_000_000_000 { "✓" } else { "✗" });
    println!("    {:30} {:>12} {:>10}", "D: Zero-Hash (hash tính lại)",
        format_bytes(d_total), if d_total < 16_000_000_000 { "✓" } else { "✗" });
    println!("    {:30} {:>12} {:>10}", "E: CompactQR (2B/node) ★",
        format_bytes(e_total), if e_total < 16_000_000_000 { "✓" } else { "✗" });
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // CompactQR live benchmark
    // ══════════════════════════════════════════════════════════════════════
    println!("    ── CompactQR Live Benchmark ──");
    {
        use olang::molecular::{CompactQR, FormulaTable};

        let mut table = FormulaTable::new();

        // Encode fire codepoint → Molecule → CompactQR (LOSSLESS)
        let fire_chain = olang::encoder::encode_codepoint(0x1F525);
        let fire_mol = fire_chain.0[0];
        let fire_cqr = CompactQR::from_molecule(&fire_mol, &mut table).unwrap();
        println!("    🔥 Fire: {:?} → CompactQR {}", fire_mol, fire_cqr);

        // Roundtrip fidelity (LOSSLESS)
        let back = fire_cqr.to_molecule(&table).unwrap();
        println!("    Roundtrip: {:?} (shape={}, rel={}, V={}, A={}, T={}) [LOSSLESS]",
            back, back.shape_u8(), back.relation_u8(), back.valence_u8(), back.arousal_u8(), back.time_u8());
        assert_eq!(back, fire_mol, "Lossless roundtrip");

        // Silk compare: fire vs water (LOSSLESS — full precision)
        let water_chain = olang::encoder::encode_codepoint(0x1F4A7);
        let water_mol = water_chain.0[0];
        let water_cqr = CompactQR::from_molecule(&water_mol, &mut table).unwrap();
        let (base_match, exact_match, sim) = fire_cqr.silk_compare(water_cqr, &table);
        println!("    🔥↔💧 silk_compare: base={}/5 exact={}/5 strength={:.2}", base_match, exact_match, sim);

        // Evolve: fire → gentle fire (lower valence) — LOSSLESS
        let gentle = fire_cqr.evolve(2, 0x40, &mut table).unwrap(); // dim 2 = valence
        println!("    🔥→🕯️ evolve(V, 0x40): {}", gentle);
        println!("    FormulaTable: {} entries, ~{} bytes RAM", table.len(), table.ram_usage());

        // Speed: from_molecule (lossless — includes table lookup)
        let t = std::time::Instant::now();
        let iters = 1_000_000u64;
        for _ in 0..iters {
            let _ = CompactQR::from_molecule(&fire_mol, &mut table);
        }
        let elapsed = t.elapsed();
        println!("    from_molecule : {:.0} ops/s ({:.1}ns/call) [lossless]",
            iters as f64 / elapsed.as_secs_f64(),
            elapsed.as_nanos() as f64 / iters as f64);

        // Speed: silk_compare (lossless — full 5D)
        let t = std::time::Instant::now();
        for _ in 0..iters {
            let _ = fire_cqr.silk_compare(water_cqr, &table);
        }
        let elapsed = t.elapsed();
        println!("    silk_compare  : {:.0} ops/s ({:.1}ns/call) [lossless]",
            iters as f64 / elapsed.as_secs_f64(),
            elapsed.as_nanos() as f64 / iters as f64);

        // Speed: compute_hash
        let t = std::time::Instant::now();
        for _ in 0..iters {
            let _ = fire_cqr.compute_hash();
        }
        let elapsed = t.elapsed();
        println!("    compute_hash  : {:.0} ops/s ({:.1}ns/call)",
            iters as f64 / elapsed.as_secs_f64(),
            elapsed.as_nanos() as f64 / iters as f64);

        // Speed: from_molecule_lossy (backward compat — no table)
        let t = std::time::Instant::now();
        for _ in 0..iters {
            let _ = CompactQR::from_molecule_lossy(&fire_mol);
        }
        let elapsed = t.elapsed();
        println!("    from_mol_lossy: {:.0} ops/s ({:.1}ns/call) [packed, no table]",
            iters as f64 / elapsed.as_secs_f64(),
            elapsed.as_nanos() as f64 / iters as f64);
    }
    println!();
}

fn print_bar(label: &str, value: u64, max: f64, width: usize) {
    let filled = ((value as f64 / max) * width as f64).min(width as f64) as usize;
    let gb = value as f64 / 1_073_741_824.0;
    print!("    {:>15} ", label);
    for i in 0..width {
        if i < filled { print!("█"); } else { print!("░"); }
    }
    println!(" {:.1} GB", gb);
}
