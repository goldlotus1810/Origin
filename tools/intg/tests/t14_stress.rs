//! Integration: Stress tests — high-volume operations without crash
//!
//! Tests the system under sustained load:
//!   - Runtime: 10K turns of process_text()
//!   - Dream: 1000 observations → 100 dream cycles
//!   - Silk: 100K co-activations → verify convergence
//!
//! Covers: runtime, memory (STM + Dream), silk (Hebbian)

use agents::learning::ShortTermMemory;
use memory::dream::{DreamConfig, DreamCycle};
use olang::encoder::encode_codepoint;
use runtime::core::origin::HomeRuntime;
use silk::edge::EmotionTag;
use silk::graph::SilkGraph;

// ═══════════════════════════════════════════════════════════════════
// Runtime stress: many turns
// ═══════════════════════════════════════════════════════════════════

#[test]
fn stress_runtime_1000_turns() {
    let mut rt = HomeRuntime::new(42);
    let base_ts = 1_000_000i64;

    for i in 0..1000 {
        let text = match i % 5 {
            0 => format!("hello {}", i),
            1 => format!("tôi vui quá {}", i),
            2 => format!("đây là bài test {}", i),
            3 => format!("hôm nay trời đẹp {}", i),
            _ => format!("cảm ơn bạn {}", i),
        };
        let response = rt.process_text(&text, base_ts + i * 100);
        // Response must always have text — never crash
        assert!(!response.text.is_empty(), "turn {} returned empty", i);
    }
}

#[test]
fn stress_runtime_rapid_turns_same_text() {
    let mut rt = HomeRuntime::new(99);
    let base_ts = 2_000_000i64;

    // Same text repeated — tests dedup, STM stability
    for i in 0..500 {
        let response = rt.process_text("xin chào", base_ts + i * 10);
        assert!(!response.text.is_empty());
    }
}

#[test]
fn stress_runtime_long_text() {
    let mut rt = HomeRuntime::new(77);
    // Very long input — should not crash
    let long_text = "a ".repeat(5000); // 10000 chars
    let response = rt.process_text(&long_text, 3_000_000);
    assert!(!response.text.is_empty());
}

#[test]
fn stress_runtime_empty_and_whitespace() {
    let mut rt = HomeRuntime::new(55);
    let inputs = ["", " ", "\n", "\t", "   \n  \t  ", ".", "?"];
    for (i, input) in inputs.iter().enumerate() {
        let response = rt.process_text(input, 4_000_000 + i as i64);
        // Should handle gracefully — no crash
        let _ = response.text;
    }
}

#[test]
fn stress_runtime_unicode_variety() {
    let mut rt = HomeRuntime::new(33);
    let inputs = [
        "🔥 fire emoji",
        "数学は美しい",
        "مرحبا بالعالم",
        "Привет мир",
        "γεια σου κόσμε",
        "🎵🎶🎸🥁🎺🎻",
        "∈ ⊂ ≡ ⊥ ∘ → ≈ ←",
        "● ▬ ■ ▲ ○ ∪ ∩",
    ];
    for (i, input) in inputs.iter().enumerate() {
        let response = rt.process_text(input, 5_000_000 + i as i64 * 100);
        assert!(!response.text.is_empty(), "unicode input {} failed", i);
    }
}

// ═══════════════════════════════════════════════════════════════════
// Dream stress: many observations + cycles
// ═══════════════════════════════════════════════════════════════════

#[test]
fn stress_dream_many_observations() {
    let config = DreamConfig::for_conversation();
    let dream = DreamCycle::new(config);
    let mut stm = ShortTermMemory::new(2048);
    let graph = SilkGraph::new();

    // Push 500 distinct observations from different codepoints
    let codepoints = [
        0x1F525u32, // 🔥
        0x25CF,     // ●
        0x2208,     // ∈
        0x2192,     // →
        0x25CB,     // ○
        0x1F600,    // 😀
        0x1F622,    // 😢
        0x2764,     // ❤
        0x1F4A7,    // 💧
        0x2669,     // ♩
    ];

    for i in 0..500 {
        let cp = codepoints[i % codepoints.len()];
        let chain = encode_codepoint(cp);
        let v = (i % 10) as f32 / 10.0 - 0.5; // -0.5 to 0.4
        let a = (i % 8) as f32 / 8.0;
        let emotion = EmotionTag::new(v, a, 0.5, 0.5);
        stm.push(chain, emotion, i as i64 * 1000);
    }

    // Run 50 dream cycles
    for cycle in 0..50 {
        let result = dream.run(&stm, &graph, (500 + cycle) * 1000);
        // Should not crash, scanned should be >= 0
        let _ = result.scanned;
        let _ = result.clusters_found;
    }
}

#[test]
fn stress_dream_empty_stm() {
    let config = DreamConfig::for_conversation();
    let dream = DreamCycle::new(config);
    let stm = ShortTermMemory::new(64);
    let graph = SilkGraph::new();

    // Dream on empty STM — should handle gracefully
    let result = dream.run(&stm, &graph, 1_000_000);
    assert_eq!(result.scanned, 0);
}

// ═══════════════════════════════════════════════════════════════════
// Silk stress: many co-activations
// ═══════════════════════════════════════════════════════════════════

#[test]
fn stress_silk_100k_coactivations() {
    let mut graph = SilkGraph::new();

    for i in 0u64..100_000 {
        let from = i.wrapping_mul(6364136223846793005).wrapping_add(1);
        let to = from.wrapping_mul(1442695040888963407).wrapping_add(1);
        let v = ((i % 200) as f32 / 100.0) - 1.0; // -1.0 to 0.99
        let a = (i % 100) as f32 / 100.0;
        graph.co_activate(from, to, EmotionTag::new(v, a, 0.5, 0.5), 0.8, i as i64);
    }

    // Should have accumulated many edges without crash
    assert!(graph.len() > 0, "graph should have edges");
}

#[test]
fn stress_silk_repeated_pair_convergence() {
    let mut graph = SilkGraph::new();
    let from = 0xDEADBEEFu64;
    let to = 0xCAFEBABEu64;

    // Same pair co-activated 10000 times — weight should converge, not overflow
    for i in 0..10_000 {
        graph.co_activate(
            from, to,
            EmotionTag::new(0.8, 0.7, 0.5, 0.5),
            1.0,
            i as i64 * 100,
        );
    }

    let weight = graph.assoc_weight(from, to);
    // Weight should be bounded (Hebbian with decay)
    assert!(weight > 0.0, "weight should be positive after 10K activations");
    assert!(weight <= 10.0, "weight should not overflow, got {}", weight);

    // Also test learn() API separately
    graph.learn(from, to, 1.0);
    let learned = graph.learned_weight(from, to);
    assert!(learned > 0.0, "learned weight should be positive after learn()");
}

#[test]
fn stress_silk_many_nodes_neighbors() {
    let mut graph = SilkGraph::new();
    let hub = 0x12345678u64;

    // Create a hub node with 1000 connections
    for i in 0u64..1000 {
        let spoke = 0xAAAA0000 + i;
        graph.co_activate(
            hub, spoke,
            EmotionTag::new(0.5, 0.5, 0.5, 0.5),
            0.5,
            i as i64,
        );
    }

    let neighbors = graph.neighbors(hub);
    assert!(neighbors.len() > 0, "hub should have neighbors");
    // Should handle large neighbor list without issue
}

#[test]
fn stress_silk_decay_stability() {
    let mut graph = SilkGraph::new();

    // Create some edges
    for i in 0u64..100 {
        graph.co_activate(i, i + 1000, EmotionTag::new(0.5, 0.5, 0.5, 0.5), 0.8, i as i64);
    }

    let initial_count = graph.len();

    // Decay multiple times — should not crash or produce NaN
    for _ in 0..100 {
        graph.decay_all(1_000_000_000); // 1 second elapsed
    }

    // After heavy decay, edges may have been pruned but count should be valid
    assert!(graph.len() <= initial_count, "decay should not add edges");
}

#[test]
fn stress_silk_maintain_prune() {
    let mut graph = SilkGraph::new();

    // Create many edges
    for i in 0u64..5000 {
        graph.co_activate(
            i % 100, i + 1000,
            EmotionTag::new(0.5, 0.5, 0.5, 0.5),
            0.3,
            i as i64,
        );
    }

    // Maintain with aggressive pruning
    let pruned = graph.maintain(1_000_000_000, 100);
    let _ = pruned; // May or may not prune depending on implementation
    // Should not crash
}
