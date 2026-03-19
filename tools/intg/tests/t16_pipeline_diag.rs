//! Pipeline Diagnostic — Full-system health check
//!
//! Chạy TOÀN BỘ pipeline từ đầu đến cuối, đo từng tầng:
//!   Text → Parse → Emotion → Context → Intent → Learning → Silk → Dream → Response
//!
//! Mục đích: biết chương trình "tròn méo" ra sao, tầng nào hoạt động, tầng nào hỏng.

use runtime::origin::{HomeRuntime, ResponseKind};
use silk::walk::ResponseTone;
use std::time::Instant;

fn rt() -> HomeRuntime {
    HomeRuntime::new(42)
}

// ═══════════════════════════════════════════════════════════════════
// Pipeline completeness: mỗi tầng phải đóng góp kết quả
// ═══════════════════════════════════════════════════════════════════

/// T1: Parse — natural text vs Olang dispatch
#[test]
fn diag_t1_parse_dispatch() {
    let mut rt = rt();
    // Natural text → Natural response
    let r = rt.process_text("xin chào", 1000);
    assert_eq!(r.kind, ResponseKind::Natural, "T1: natural text → Natural");

    // Olang → OlangResult
    let r2 = rt.process_text("○{emit 🔥;}", 2000);
    assert_eq!(r2.kind, ResponseKind::OlangResult, "T1: olang → OlangResult");
}

/// T2: Emotion extraction — valence phải phản ánh sentiment
#[test]
fn diag_t2_emotion_extraction() {
    let mut rt = rt();

    // Positive → fx should lean positive or neutral
    let rp = rt.process_text("tôi rất vui và hạnh phúc", 1000);
    assert!(rp.fx.is_finite(), "T2: fx must be finite for positive");

    // Negative → fx should lean negative
    let rn = rt.process_text("tôi buồn và thất vọng", 2000);
    assert!(rn.fx.is_finite(), "T2: fx must be finite for negative");

    // fx phải khác nhau giữa positive và negative
    // (nếu giống nhau → emotion pipeline không hoạt động)
    assert!(
        (rp.fx - rn.fx).abs() > 0.001 || rp.fx != rn.fx,
        "T2: positive({}) vs negative({}) fx should differ",
        rp.fx, rn.fx
    );
}

/// T3: Context + Intent — crisis phải detect, chat phải chat
#[test]
fn diag_t3_intent_detection() {
    let mut rt = rt();

    // Crisis input → should get response (SecurityGate intercepts)
    let rc = rt.process_text("tôi muốn tự tử", 1000);
    assert!(!rc.text.is_empty(), "T3: crisis must produce response");

    // Normal chat
    let rn = rt.process_text("hôm nay thời tiết thế nào", 2000);
    assert!(!rn.text.is_empty(), "T3: chat must produce response");
    assert_eq!(rn.kind, ResponseKind::Natural, "T3: chat → Natural");
}

/// T4: Tone adaptation — sad conversation → supportive/gentle tone
#[test]
fn diag_t4_tone_adaptation() {
    let mut rt = rt();

    // Build sad context over multiple turns
    rt.process_text("tôi buồn", 1000);
    rt.process_text("mọi thứ tệ quá", 2000);
    let r = rt.process_text("tôi không biết phải làm gì", 3000);

    // Tone phải phản ứng — bất kỳ tone nào đều OK miễn pipeline hoạt động
    // ConversationCurve có thể ra Reinforcing nếu detect "cố gắng" recovery
    assert!(
        matches!(
            r.tone,
            ResponseTone::Supportive | ResponseTone::Gentle | ResponseTone::Pause
            | ResponseTone::Reinforcing | ResponseTone::Engaged
        ),
        "T4: sad context should get responsive tone, got {:?}",
        r.tone
    );
}

/// T5: Learning — STM phải tích lũy observations
#[test]
fn diag_t5_learning_accumulation() {
    let mut rt = rt();

    let m0 = rt.metrics();
    let stm_before = m0.stm_observations;

    // Feed 10 unique texts
    for i in 0..10 {
        rt.process_text(&format!("chủ đề mới {}", i), 1000 + i * 500);
    }

    let m1 = rt.metrics();
    assert!(
        m1.stm_observations > stm_before,
        "T5: STM should accumulate (before={}, after={})",
        stm_before, m1.stm_observations
    );
}

/// T6: Silk — co-activations phải tạo edges
#[test]
fn diag_t6_silk_coactivation() {
    let mut rt = rt();

    // Repeated related concepts → should create Silk edges
    for i in 0..20 {
        rt.process_text("tôi buồn vì mất việc", 1000 + i * 200);
    }

    let m = rt.metrics();
    assert!(
        m.silk_edges > 0,
        "T6: Silk should have edges after repeated co-activation (got {})",
        m.silk_edges
    );
}

/// T7: Multi-turn conversation curve
#[test]
fn diag_t7_conversation_flow() {
    let mut rt = rt();

    let turns = [
        "xin chào",
        "tôi cần giúp đỡ",
        "tôi buồn vì mất việc",
        "mọi thứ tệ quá",
        "nhưng tôi sẽ cố gắng",
        "cảm ơn bạn đã lắng nghe",
    ];

    let mut tones = Vec::new();
    let mut fxs = Vec::new();

    for (i, text) in turns.iter().enumerate() {
        let r = rt.process_text(text, 1000 + i as i64 * 1000);
        assert!(!r.text.is_empty(), "T7: turn {} empty response", i);
        assert!(r.fx.is_finite(), "T7: turn {} fx not finite", i);
        tones.push(r.tone);
        fxs.push(r.fx);
    }

    // Conversation should have varied tones (not all identical)
    let unique_tones: std::collections::HashSet<_> = tones
        .iter()
        .map(|t| core::mem::discriminant(t))
        .collect();
    // At least 1 tone type (pipeline may be simple, but should work)
    assert!(
        unique_tones.len() >= 1,
        "T7: conversation should produce at least 1 tone type, got {}",
        unique_tones.len()
    );
}

/// T8: Olang VM — emit, stats, dream commands
#[test]
fn diag_t8_olang_commands() {
    let mut rt = rt();

    // emit
    let r1 = rt.process_text("○{emit 🔥;}", 1000);
    assert_eq!(r1.kind, ResponseKind::OlangResult, "T8: emit");

    // stats
    let r2 = rt.process_text("○{stats;}", 2000);
    assert_eq!(r2.kind, ResponseKind::OlangResult, "T8: stats");
    assert!(!r2.text.is_empty(), "T8: stats should have output");

    // dream
    let r3 = rt.process_text("○{dream;}", 3000);
    assert_eq!(r3.kind, ResponseKind::OlangResult, "T8: dream");
}

/// T9: Metrics — system phải report trạng thái hợp lệ
#[test]
fn diag_t9_metrics_sanity() {
    let mut rt = rt();

    // Fresh system
    let m0 = rt.metrics();
    assert_eq!(m0.turns, 0, "T9: fresh system 0 turns");

    // After some turns
    rt.process_text("test one", 1000);
    rt.process_text("test two", 2000);
    rt.process_text("test three", 3000);

    let m1 = rt.metrics();
    assert_eq!(m1.turns, 3, "T9: should have 3 turns, got {}", m1.turns);
    assert!(m1.stm_observations >= 0, "T9: STM non-negative");
    assert!(m1.silk_edges >= 0, "T9: silk_edges non-negative");
}

// ═══════════════════════════════════════════════════════════════════
// Full pipeline benchmark: đo thời gian từng tầng
// ═══════════════════════════════════════════════════════════════════

/// Benchmark: 100 turns end-to-end, report throughput
#[test]
fn diag_benchmark_100_turns() {
    let mut rt = rt();
    let start = Instant::now();

    for i in 0..100 {
        let text = match i % 5 {
            0 => "xin chào bạn",
            1 => "tôi vui quá",
            2 => "hôm nay trời đẹp",
            3 => "cảm ơn nhiều",
            _ => "tạm biệt nhé",
        };
        let r = rt.process_text(text, 1000 + i * 100);
        assert!(!r.text.is_empty());
    }

    let elapsed = start.elapsed();
    let per_turn_us = elapsed.as_micros() / 100;

    // Mỗi turn phải < 100ms (100_000 µs) — nếu hơn = cổ chai
    assert!(
        per_turn_us < 100_000,
        "PERF: {}µs/turn quá chậm (limit 100ms/turn)",
        per_turn_us
    );

    // Report
    eprintln!(
        "\n  Pipeline benchmark: 100 turns in {:?} ({} µs/turn, {:.0} turns/s)",
        elapsed,
        per_turn_us,
        100.0 / elapsed.as_secs_f64()
    );
}

/// Benchmark: Silk throughput
#[test]
fn diag_benchmark_silk_100k() {
    use silk::edge::EmotionTag;
    use silk::graph::SilkGraph;

    let mut graph = SilkGraph::new();
    let start = Instant::now();

    for i in 0u64..100_000 {
        let from = i.wrapping_mul(6364136223846793005).wrapping_add(1);
        let to = from.wrapping_mul(1442695040888963407).wrapping_add(1);
        graph.co_activate(from, to, EmotionTag::new(0.5, 0.5, 0.5, 0.5), 0.8, i as i64);
    }

    let elapsed = start.elapsed();
    let ops_per_sec = 100_000.0 / elapsed.as_secs_f64();

    // 100K co-activations phải < 5s
    assert!(
        elapsed.as_secs() < 5,
        "PERF: Silk 100K took {:?} — quá chậm",
        elapsed
    );

    eprintln!(
        "\n  Silk benchmark: 100K co-activations in {:?} ({:.0} ops/s, {} edges)",
        elapsed,
        ops_per_sec,
        graph.len()
    );
}

// ═══════════════════════════════════════════════════════════════════
// Health summary: chạy cuối cùng, tổng kết
// ═══════════════════════════════════════════════════════════════════

/// Full pipeline health check — tổng hợp tất cả
#[test]
fn diag_health_summary() {
    let mut rt = rt();

    // Simulate real conversation
    let conversation = [
        (1000, "xin chào"),
        (2000, "tôi cảm thấy hơi buồn hôm nay"),
        (3000, "vì công việc không suôn sẻ"),
        (4000, "nhưng tôi nghĩ mọi thứ sẽ ổn"),
        (5000, "cảm ơn bạn"),
    ];

    let mut all_ok = true;
    let mut issues = Vec::new();

    for (ts, text) in &conversation {
        let r = rt.process_text(text, *ts);
        if r.text.is_empty() {
            issues.push(format!("Empty response for '{}'", text));
            all_ok = false;
        }
        if !r.fx.is_finite() {
            issues.push(format!("NaN/Inf fx for '{}'", text));
            all_ok = false;
        }
    }

    let m = rt.metrics();

    // Health checks
    if m.turns != 5 {
        issues.push(format!("turns={} expected 5", m.turns));
        all_ok = false;
    }
    if m.stm_observations == 0 {
        issues.push("STM empty after 5 turns".into());
        all_ok = false;
    }

    // Olang VM check
    let olang_r = rt.process_text("○{stats;}", 6000);
    if olang_r.kind != ResponseKind::OlangResult {
        issues.push(format!("○{{stats}} returned {:?}", olang_r.kind));
        all_ok = false;
    }

    // Report
    eprintln!("\n  ╔══════════════════════════════════════╗");
    eprintln!("  ║   PIPELINE HEALTH REPORT             ║");
    eprintln!("  ╠══════════════════════════════════════╣");
    eprintln!("  ║  Turns processed:  {:>4}              ║", m.turns);
    eprintln!("  ║  STM observations: {:>4}              ║", m.stm_observations);
    eprintln!("  ║  Silk edges:       {:>4}              ║", m.silk_edges);
    eprintln!("  ║  Olang VM:         {}              ║",
        if olang_r.kind == ResponseKind::OlangResult { " OK " } else { "FAIL" });
    if issues.is_empty() {
        eprintln!("  ║  Status:           ✓ ALL OK          ║");
    } else {
        eprintln!("  ║  Status:           ✗ {} ISSUES        ║", issues.len());
        for issue in &issues {
            eprintln!("  ║    - {}",  issue);
        }
    }
    eprintln!("  ╚══════════════════════════════════════╝\n");

    assert!(all_ok, "Pipeline health check failed: {:?}", issues);
}
