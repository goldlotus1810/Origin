//! # system-bench — Benchmark tổng quát HomeOS
//!
//! Kiểm tra toàn bộ pipeline: UCD → Encode → Silk → KnowTree → Runtime
//! Đo: node count, silk edges, CPU time, memory, hash uniqueness
//!
//! Chạy: cargo run -p bench --bin system-bench

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use olang::encoder::encode_codepoint;
use olang::lca::lca;
use olang::molecular::{MolecularChain, ShapeBase, RelationBase, TimeDim};
use runtime::origin::HomeRuntime;
use silk::edge::EmotionTag;
use silk::graph::SilkGraph;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           ○ HomeOS System Benchmark                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // ── 1. UCD Statistics ─────────────────────────────────────────────────────
    bench_ucd_stats();

    // ── 2. Hierarchical Encoding Distribution ─────────────────────────────────
    bench_hierarchical_distribution();

    // ── 3. Hash Uniqueness ────────────────────────────────────────────────────
    bench_hash_uniqueness();

    // ── 4. Encode + LCA Performance ───────────────────────────────────────────
    bench_encode_lca_perf();

    // ── 5. Silk Graph Stress ──────────────────────────────────────────────────
    bench_silk_stress();

    // ── 6. Full Pipeline (Runtime) ────────────────────────────────────────────
    bench_runtime_pipeline();

    // ── 7. KnowTree Storage ──────────────────────────────────────────────────
    bench_knowtree();

    // ── 8. Memory Footprint ───────────────────────────────────────────────────
    bench_memory_footprint();

    println!();
    println!("○ System benchmark complete.");
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. UCD Statistics
// ─────────────────────────────────────────────────────────────────────────────

fn bench_ucd_stats() {
    println!("── 1. UCD Statistics ─────────────────────────────────────────");

    let total = ucd::table_len();
    println!("  Total UCD entries     : {}", total);

    // Count per group
    let mut group_counts: HashMap<u8, usize> = HashMap::new();
    let mut shape_base_counts: HashMap<u8, usize> = HashMap::new();
    let mut relation_base_counts: HashMap<u8, usize> = HashMap::new();
    let mut time_base_counts: HashMap<u8, usize> = HashMap::new();

    // Sample all UCD entries via encode
    let test_ranges: &[(u32, u32)] = &[
        (0x2190, 0x21FF), // Arrows
        (0x2200, 0x22FF), // Math Operators
        (0x2300, 0x23FF), // Misc Technical
        (0x2500, 0x257F), // Box Drawing
        (0x25A0, 0x25FF), // Geometric Shapes
        (0x2600, 0x26FF), // Misc Symbols
        (0x2700, 0x27BF), // Dingbats
        (0x2900, 0x297F), // Supp Arrows-B
        (0x2980, 0x29FF), // Misc Math Symbols-B
        (0x2A00, 0x2AFF), // Supp Math Operators
        (0x1D100, 0x1D1FF), // Musical Symbols
        (0x1F300, 0x1F5FF), // Misc Symbols & Pictographs
        (0x1F600, 0x1F64F), // Emoticons
        (0x1F680, 0x1F6FF), // Transport & Map
        (0x1F900, 0x1F9FF), // Supp Symbols & Pictographs
    ];

    let mut encoded_count = 0u32;
    for &(start, end) in test_ranges {
        for cp in start..=end {
            if let Some(entry) = ucd::lookup(cp) {
                group_counts.entry(entry.group).and_modify(|c| *c += 1).or_insert(1);

                let sb = ((entry.shape.wrapping_sub(1)) % 8) + 1;
                let rb = ((entry.relation.wrapping_sub(1)) % 8) + 1;
                let tb = if entry.time == 0 { 0 } else { ((entry.time - 1) % 5) + 1 };
                shape_base_counts.entry(sb).and_modify(|c| *c += 1).or_insert(1);
                relation_base_counts.entry(rb).and_modify(|c| *c += 1).or_insert(1);
                time_base_counts.entry(tb).and_modify(|c| *c += 1).or_insert(1);
                encoded_count += 1;
            }
        }
    }

    let group_names = ["", "SDF", "MATH", "EMOTICON", "MUSICAL", "MISC"];
    println!("  Encoded from ranges   : {}", encoded_count);
    println!();
    println!("  Groups:");
    let mut groups: Vec<_> = group_counts.iter().collect();
    groups.sort_by_key(|&(k, _)| *k);
    for (g, count) in &groups {
        let name = group_names.get(**g as usize).unwrap_or(&"?");
        println!("    {} ({:02X}): {:5}", name, g, count);
    }

    let shape_names = [
        "", "Sphere", "Plane", "Box", "Cone", "Torus", "Union", "Intersect", "Subtract",
    ];
    println!();
    println!("  Shape bases:");
    let mut shapes: Vec<_> = shape_base_counts.iter().collect();
    shapes.sort_by_key(|&(k, _)| *k);
    for (s, count) in &shapes {
        let name = shape_names.get(**s as usize).unwrap_or(&"?");
        println!("    {} ({:02X}): {:5}", name, s, count);
    }

    let rel_names = [
        "", "Member", "Contains", "Equiv", "Orthogonal", "Compose",
        "Causes", "Similar", "DerivedFrom",
    ];
    println!();
    println!("  Relation bases:");
    let mut rels: Vec<_> = relation_base_counts.iter().collect();
    rels.sort_by_key(|&(k, _)| *k);
    for (r, count) in &rels {
        let name = rel_names.get(**r as usize).unwrap_or(&"?");
        println!("    {} ({:02X}): {:5}", name, r, count);
    }

    let time_names = ["", "Static", "Slow", "Medium", "Fast", "Instant"];
    println!();
    println!("  Time bases:");
    let mut times: Vec<_> = time_base_counts.iter().collect();
    times.sort_by_key(|&(k, _)| *k);
    for (t, count) in &times {
        let name = time_names.get(**t as usize).unwrap_or(&"?");
        println!("    {} ({:02X}): {:5}", name, t, count);
    }
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Hierarchical Encoding Distribution
// ─────────────────────────────────────────────────────────────────────────────

fn bench_hierarchical_distribution() {
    println!("── 2. Hierarchical Byte Distribution ────────────────────────");

    let mut unique_shapes: HashSet<u8> = HashSet::new();
    let mut unique_relations: HashSet<u8> = HashSet::new();
    let mut unique_times: HashSet<u8> = HashSet::new();
    let mut unique_molecules: HashSet<[u8; 5]> = HashSet::new();

    let ranges: &[(u32, u32)] = &[
        (0x2190, 0x21FF), (0x2200, 0x22FF), (0x25A0, 0x25FF),
        (0x2600, 0x26FF), (0x2700, 0x27BF), (0x1D100, 0x1D1FF),
        (0x1F300, 0x1F5FF), (0x1F600, 0x1F64F), (0x1F900, 0x1F9FF),
    ];

    let mut total = 0u32;
    for &(start, end) in ranges {
        for cp in start..=end {
            if let Some(entry) = ucd::lookup(cp) {
                unique_shapes.insert(entry.shape);
                unique_relations.insert(entry.relation);
                unique_times.insert(entry.time);
                unique_molecules.insert([
                    entry.shape, entry.relation, entry.valence, entry.arousal, entry.time,
                ]);
                total += 1;
            }
        }
    }

    println!("  Entries sampled       : {}", total);
    println!("  Unique shape bytes    : {}", unique_shapes.len());
    println!("  Unique relation bytes : {}", unique_relations.len());
    println!("  Unique time bytes     : {}", unique_times.len());
    println!("  Unique 5-byte molecules: {}", unique_molecules.len());
    println!(
        "  Diversity ratio       : {:.1}% (molecules/entries)",
        unique_molecules.len() as f64 / total.max(1) as f64 * 100.0
    );

    // Show max sub-index per base
    println!();
    println!("  Max sub-index per shape base:");
    for base in 1u8..=8 {
        let max_sub = unique_shapes
            .iter()
            .filter(|&&s| s > 0 && ((s - 1) % 8) + 1 == base)
            .map(|&s| (s - 1) / 8)
            .max()
            .unwrap_or(0);
        let name = ["", "Sphere", "Plane", "Box", "Cone", "Torus", "Union", "Intersect", "Subtract"];
        println!("    {}: {} variants (sub 0..{})", name[base as usize], max_sub + 1, max_sub);
    }
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Hash Uniqueness
// ─────────────────────────────────────────────────────────────────────────────

fn bench_hash_uniqueness() {
    println!("── 3. Hash Uniqueness ────────────────────────────────────────");

    let mut hashes: HashSet<u64> = HashSet::new();
    let mut total = 0u32;
    let mut collisions = 0u32;

    let ranges: &[(u32, u32)] = &[
        (0x2190, 0x21FF), (0x2200, 0x22FF), (0x2300, 0x23FF),
        (0x25A0, 0x25FF), (0x2600, 0x26FF), (0x2700, 0x27BF),
        (0x2900, 0x297F), (0x2980, 0x29FF), (0x2A00, 0x2AFF),
        (0x1D100, 0x1D1FF), (0x1F300, 0x1F5FF), (0x1F600, 0x1F64F),
        (0x1F680, 0x1F6FF), (0x1F900, 0x1F9FF),
    ];

    for &(start, end) in ranges {
        for cp in start..=end {
            let chain = encode_codepoint(cp);
            if chain.0.is_empty() {
                continue;
            }
            let hash = chain.chain_hash();
            if !hashes.insert(hash) {
                collisions += 1;
            }
            total += 1;
        }
    }

    println!("  Encoded codepoints : {}", total);
    println!("  Unique hashes      : {}", hashes.len());
    println!("  Collisions         : {}", collisions);
    println!(
        "  Collision rate     : {:.3}%",
        collisions as f64 / total.max(1) as f64 * 100.0
    );

    // Before hierarchical: ~5279 entries → ~100 unique hashes (98% collision)
    // After hierarchical: should be much better
    let uniqueness = hashes.len() as f64 / total.max(1) as f64 * 100.0;
    println!(
        "  Uniqueness         : {:.1}% {}",
        uniqueness,
        if uniqueness > 90.0 { "(excellent)" }
        else if uniqueness > 50.0 { "(good)" }
        else if uniqueness > 10.0 { "(moderate)" }
        else { "(poor — pre-hierarchical level)" }
    );
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Encode + LCA Performance
// ─────────────────────────────────────────────────────────────────────────────

fn bench_encode_lca_perf() {
    println!("── 4. Encode + LCA Performance ──────────────────────────────");

    let cps: Vec<u32> = vec![
        0x1F525, 0x1F4A7, 0x2744, 0x1F9E0, 0x2764, 0x1F60A, 0x1F62D, 0x1F4AA,
        0x25CF, 0x25CB, 0x25A0, 0x25B2, 0x2208, 0x2286, 0x2192, 0x2190,
    ];

    // Encode benchmark
    let iters = 50_000u32;
    let t = Instant::now();
    for _ in 0..iters {
        for &cp in &cps {
            let _ = encode_codepoint(cp);
        }
    }
    let elapsed = t.elapsed();
    let ops_per_sec = (iters as f64 * cps.len() as f64) / elapsed.as_secs_f64();
    println!(
        "  encode_codepoint : {:>10.0} ops/s  ({:.1}µs/batch of {})",
        ops_per_sec,
        elapsed.as_micros() as f64 / iters as f64,
        cps.len()
    );

    // LCA benchmark
    let pairs: Vec<(MolecularChain, MolecularChain)> = cps
        .windows(2)
        .map(|w| (encode_codepoint(w[0]), encode_codepoint(w[1])))
        .collect();

    let t = Instant::now();
    for _ in 0..iters {
        for (a, b) in &pairs {
            let _ = lca(a, b);
        }
    }
    let elapsed = t.elapsed();
    let ops_per_sec = (iters as f64 * pairs.len() as f64) / elapsed.as_secs_f64();
    println!(
        "  LCA              : {:>10.0} ops/s  ({:.1}µs/batch of {})",
        ops_per_sec,
        elapsed.as_micros() as f64 / iters as f64,
        pairs.len()
    );

    // chain_hash benchmark
    let chains: Vec<MolecularChain> = cps.iter().map(|&cp| encode_codepoint(cp)).collect();
    let t = Instant::now();
    let hash_iters = 200_000u32;
    for _ in 0..hash_iters {
        for c in &chains {
            let _ = c.chain_hash();
        }
    }
    let elapsed = t.elapsed();
    let ops_per_sec = (hash_iters as f64 * chains.len() as f64) / elapsed.as_secs_f64();
    println!(
        "  chain_hash       : {:>10.0} ops/s  ({:.0}ns/call)",
        ops_per_sec,
        elapsed.as_nanos() as f64 / (hash_iters as f64 * chains.len() as f64)
    );

    // UCD lookup benchmark
    let t = Instant::now();
    for _ in 0..iters {
        for &cp in &cps {
            let _ = ucd::lookup(cp);
        }
    }
    let elapsed = t.elapsed();
    let ops_per_sec = (iters as f64 * cps.len() as f64) / elapsed.as_secs_f64();
    println!(
        "  ucd::lookup      : {:>10.0} ops/s  ({:.1}µs/batch of {})",
        ops_per_sec,
        elapsed.as_micros() as f64 / iters as f64,
        cps.len()
    );

    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Silk Graph Stress
// ─────────────────────────────────────────────────────────────────────────────

fn bench_silk_stress() {
    println!("── 5. Silk Graph Stress ───────────────────────────────────────");

    let mut graph = SilkGraph::new();

    // Phase A: Build graph with 1000 co-activations
    let t = Instant::now();
    let n = 1000u64;
    for i in 0..n {
        let emo = EmotionTag {
            valence: (i as f32 / n as f32) * 2.0 - 1.0,
            arousal: 0.5,
            dominance: 0.5,
            intensity: 0.7,
        };
        graph.co_activate(i, (i + 1) % n, emo, 0.6, i as i64);
        // Also create some cross-connections
        if i % 10 == 0 {
            graph.co_activate(i, (i + 100) % n, EmotionTag::NEUTRAL, 0.3, i as i64);
        }
    }
    let build_time = t.elapsed();

    println!("  Build 1000+100 co-activations:");
    println!("    Time           : {:.2}ms", build_time.as_micros() as f64 / 1000.0);
    println!("    Nodes          : {}", graph.node_count());
    println!("    Total edges    : {}", graph.len());
    println!("    Associative    : {}", graph.assoc_count());
    println!("    Structural     : {}", graph.structural_count());

    // Phase B: Reinforce existing edges (Hebbian)
    let t = Instant::now();
    for i in 0..500u64 {
        graph.co_activate(i, i + 1, EmotionTag::NEUTRAL, 0.8, (n + i) as i64);
    }
    let reinforce_time = t.elapsed();
    println!("  Reinforce 500 edges:");
    println!("    Time           : {:.2}ms", reinforce_time.as_micros() as f64 / 1000.0);
    println!("    Total edges    : {} (should stay same)", graph.len());

    // Phase C: Maintain/prune
    let t = Instant::now();
    let pruned = graph.maintain(1_000_000_000, 800); // 1s elapsed, max 800 edges
    let prune_time = t.elapsed();
    println!("  Maintain (max=800):");
    println!("    Time           : {:.2}ms", prune_time.as_micros() as f64 / 1000.0);
    println!("    Pruned         : {}", pruned);
    println!("    Remaining      : {}", graph.len());

    // Phase D: Lookup speed
    let t = Instant::now();
    let iters = 10_000u32;
    for _ in 0..iters {
        for i in 0..100u64 {
            let _ = graph.edges_from(i);
        }
    }
    let lookup_time = t.elapsed();
    let ops_per_sec = (iters as f64 * 100.0) / lookup_time.as_secs_f64();
    println!("  edges_from() lookup:");
    println!(
        "    Speed          : {:.0} ops/s ({:.1}µs/100 lookups)",
        ops_per_sec,
        lookup_time.as_micros() as f64 / iters as f64
    );

    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Full Pipeline (Runtime)
// ─────────────────────────────────────────────────────────────────────────────

fn bench_runtime_pipeline() {
    println!("── 6. Full Pipeline (HomeRuntime) ─────────────────────────────");

    let mut rt = HomeRuntime::new(0xBE_0001);

    // Vietnamese conversation simulation
    let inputs = [
        "xin chào",
        "tôi đang học về vũ trụ",
        "sao lại có lực hấp dẫn?",
        "nước chảy từ cao xuống thấp",
        "lửa cháy nóng và sáng",
        "tôi buồn vì mất việc",
        "nhưng tôi sẽ cố gắng",
        "hôm nay trời đẹp quá",
        "âm nhạc làm tôi vui hơn",
        "toán học thật thú vị",
        "1 + 2 bằng bao nhiêu?",
        "tại sao bầu trời xanh?",
        "cảm ơn bạn đã lắng nghe",
        "tôi thích đọc sách",
        "mưa rơi trên mái nhà",
        "gió thổi lá bay",
        "mặt trời lặn rồi",
        "tôi cần nghỉ ngơi",
        "ngày mai sẽ tốt hơn",
        "chúc ngủ ngon",
    ];

    let t = Instant::now();
    let mut responses = Vec::new();
    for (i, text) in inputs.iter().enumerate() {
        let ts = (i as i64 + 1) * 1000;
        let resp = rt.process_text(text, ts);
        responses.push((text, resp));
    }
    let pipeline_time = t.elapsed();

    println!("  Processed {} turns in {:.1}ms", inputs.len(), pipeline_time.as_millis());
    println!(
        "  Throughput        : {:.0} turns/s ({:.1}ms/turn)",
        inputs.len() as f64 / pipeline_time.as_secs_f64(),
        pipeline_time.as_millis() as f64 / inputs.len() as f64,
    );
    println!();

    // State after conversation
    println!("  ── Runtime State ──");
    println!("  Turn count        : {}", rt.turn_count());
    println!("  STM observations  : {}", rt.stm_len());
    println!("  Silk nodes        : {}", rt.silk_node_count());
    println!("  Silk edges total  : {}", rt.silk_edge_count());
    println!("  Silk associative  : {}", rt.silk_assoc_count());
    println!("  Silk structural   : {}", rt.silk_structural_count());
    println!();

    // Emotion curve
    println!("  ── Emotion Curve ──");
    println!("  f(x) final        : {:+.3}", rt.fx());
    println!("  Valence now       : {:+.3}", rt.curve_valence());
    println!("  Velocity (d1)     : {:+.3}", rt.curve_d1());
    println!("  Acceleration (d2) : {:+.3}", rt.curve_d2());
    println!("  Window variance   : {:.4}", rt.curve_variance());
    println!("  Unstable          : {}", rt.curve_unstable());
    println!("  Tone              : {:?}", rt.tone());
    println!();

    // KnowTree
    println!("  ── KnowTree ──");
    println!("  Total nodes       : {}", rt.knowtree().total_nodes());
    println!("  Total edges       : {}", rt.knowtree().total_edges());
    println!("  L2 sentences      : {}", rt.knowtree_sentences());
    println!("  L3 concepts       : {}", rt.knowtree_concepts());
    println!("  RAM usage         : ~{}KB", rt.knowtree().ram_usage() / 1024);
    println!("  Disk usage        : ~{}KB", rt.knowtree().disk_usage() / 1024);
    println!("  Cache hit rate    : {:.1}%", rt.knowtree().cache_hit_rate() * 100.0);
    println!();

    // BodyStore
    println!("  ── BodyStore (SDF + Spline) ──");
    println!("  Total bodies      : {}", rt.body_count());
    println!("  Bodies with SDF   : {}", rt.bodies_with_shape());
    println!("  Body RAM          : ~{}KB", rt.body_store().ram_usage() / 1024);
    println!();

    // Dream stats
    println!("  ── Dream ──");
    println!("  Dream cycles      : {}", rt.dream_cycles());
    println!("  Approved proposals: {}", rt.dream_approved());
    println!("  L3 created        : {}", rt.dream_l3_concepts());
    println!("  Next dream at     : turn {}", rt.dream_fib_interval());
    println!();

    // Sample responses
    println!("  ── Sample Responses (first 5) ──");
    for (text, resp) in responses.iter().take(5) {
        let short = if resp.text.len() > 60 {
            format!("{}...", &resp.text[..resp.text.char_indices().nth(60).map(|(i,_)|i).unwrap_or(resp.text.len())])
        } else {
            resp.text.clone()
        };
        println!("  IN : {}", text);
        println!("  OUT: [{:?}|{:?}|fx={:+.2}] {}", resp.kind, resp.tone, resp.fx, short);
        println!();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. KnowTree Stress
// ─────────────────────────────────────────────────────────────────────────────

fn bench_knowtree() {
    println!("── 7. KnowTree Stress ────────────────────────────────────────");

    use olang::knowtree::{KnowTreeLegacy, text_to_word_hashes};
    use olang::molecular::Molecule;

    let mut kt = KnowTreeLegacy::for_pc();

    // Store 100 sentences
    let sentences = [
        "lửa cháy rực rỡ trong đêm tối",
        "nước chảy từ nguồn ra biển lớn",
        "gió thổi mạnh qua đồng cỏ xanh",
        "mặt trời chiếu sáng khắp nơi",
        "mưa rơi nhẹ nhàng trên lá cây",
        "tuyết phủ trắng xóa núi cao",
        "sóng biển vỗ bờ đá xám",
        "chim hót véo von trong vườn",
        "hoa nở rực rỡ mùa xuân",
        "trăng sáng vằng vặc đêm rằm",
    ];

    let t = Instant::now();
    let mut all_hashes = Vec::new();

    for round in 0..10 {
        for (i, text) in sentences.iter().enumerate() {
            let chain = MolecularChain::single(Molecule::raw(
                ShapeBase::Sphere.encode(round as u8),
                RelationBase::Member.encode(i as u8),
                0x80 + (round * 5) as u8,
                0x60 + (i * 3) as u8,
                TimeDim::Medium.as_byte(),
            ));
            let words = text_to_word_hashes(text);
            let hash = kt.store_sentence(&chain, None, &words, (round * 10 + i) as i64);
            all_hashes.push(hash);
        }
    }
    let store_time = t.elapsed();

    println!("  Stored 100 sentences in {:.1}ms", store_time.as_micros() as f64 / 1000.0);
    println!("  Sentences         : {}", kt.sentences());
    println!("  Total nodes       : {}", kt.total_nodes());
    println!("  Total edges       : {}", kt.total_edges());
    println!("  RAM usage         : ~{}KB", kt.ram_usage() / 1024);

    // Lookup speed
    let t = Instant::now();
    let mut found = 0u32;
    for _ in 0..1000 {
        for &h in &all_hashes {
            if kt.lookup(h, 2).is_some() {
                found += 1;
            }
        }
    }
    let lookup_time = t.elapsed();
    println!(
        "  Lookup speed      : {:.0} ops/s ({} found/100K)",
        (1000.0 * all_hashes.len() as f64) / lookup_time.as_secs_f64(),
        found
    );
    println!("  Cache hit rate    : {:.1}%", kt.cache_hit_rate() * 100.0);

    // Store concepts (LCA)
    let t = Instant::now();
    for chunk in all_hashes.chunks(5) {
        let concept_chain = MolecularChain::single(Molecule::raw(
            ShapeBase::Sphere.as_byte(),
            RelationBase::DerivedFrom.as_byte(),
            0x80,
            0x80,
            TimeDim::Medium.as_byte(),
        ));
        kt.store_concept(&concept_chain, None, 3, chunk, 9999);
    }
    let concept_time = t.elapsed();
    println!("  Stored {} concepts in {:.1}ms", kt.concepts(), concept_time.as_micros() as f64 / 1000.0);
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Memory Footprint
// ─────────────────────────────────────────────────────────────────────────────

fn bench_memory_footprint() {
    println!("── 8. Memory Footprint ───────────────────────────────────────");

    use olang::molecular::Molecule;
    use silk::edge::SilkEdge;
    use olang::compact::CompactNode;

    println!("  Core struct sizes:");
    println!("    Molecule         : {} bytes", std::mem::size_of::<Molecule>());
    println!("    MolecularChain   : {} bytes (stack)", std::mem::size_of::<MolecularChain>());
    println!("    EmotionTag       : {} bytes", std::mem::size_of::<EmotionTag>());
    println!("    SilkEdge         : {} bytes", std::mem::size_of::<SilkEdge>());
    println!("    CompactNode      : {} bytes", std::mem::size_of::<CompactNode>());
    println!("    SilkGraph        : {} bytes (stack)", std::mem::size_of::<SilkGraph>());
    println!();

    // Estimate for typical usage
    let typical_nodes = 1000u64;
    let typical_edges = 5000u64;
    let node_bytes = typical_nodes * std::mem::size_of::<CompactNode>() as u64;
    let edge_bytes = typical_edges * std::mem::size_of::<SilkEdge>() as u64;
    println!("  Estimate for {} nodes + {} edges:", typical_nodes, typical_edges);
    println!("    CompactNodes     : ~{}KB", node_bytes / 1024);
    println!("    SilkEdges        : ~{}KB", edge_bytes / 1024);
    println!("    Total            : ~{}KB", (node_bytes + edge_bytes) / 1024);
    println!();

    // FFR point generation
    println!("  FFR benchmark:");
    let t = Instant::now();
    let iters = 10_000u64;
    for n in 0..iters {
        let _ = vsdf::ffr::FfrPoint::at(n);
    }
    let elapsed = t.elapsed();
    println!(
        "    FfrPoint::at()   : {:.0} Mops/s ({:.0}ns/call)",
        iters as f64 / elapsed.as_secs_f64() / 1_000_000.0,
        elapsed.as_nanos() as f64 / iters as f64
    );
    println!();
}
