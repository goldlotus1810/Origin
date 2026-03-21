//! # functional — Chạy code thật, verify kết quả thật
//!
//! KHÔNG scan source. GỌI functions, kiểm tra output.
//! Nếu code sai → phát hiện NGAY tại runtime.

use std::time::Instant;
use crate::CheckResult;

// ═══════════════════════════════════════════════════════════════
// F1: UCD lookup — encode codepoints, verify 5D values hợp lý
// ═══════════════════════════════════════════════════════════════

pub fn check_ucd_encode_real() -> CheckResult {
    println!("  [F1] UCD — encode real codepoints...");
    let check_start = Instant::now();
    let mut details = Vec::new();
    let mut fails = 0;

    // Test known anchors from v2 spec
    let test_cases: Vec<(u32, &str, &str)> = vec![
        (0x1F525, "FIRE", "high valence, high arousal"),
        (0x1F60A, "SMILING FACE", "high valence, medium arousal"),
        (0x2764,  "HEART", "high valence"),
        (0x1F480, "SKULL", "low valence"),
        (0x1F3B5, "MUSICAL NOTE", "medium valence"),
        (0x25CF,  "BLACK CIRCLE", "neutral, shape=Sphere"),
        (0x25A0,  "BLACK SQUARE", "shape=Box expected"),
        (0x2660,  "SPADE SUIT", "should exist in table"),
    ];

    let start = Instant::now();

    for (cp, name, expected) in &test_cases {
        match ucd::lookup(*cp) {
            Some(entry) => {
                let v = entry.valence;
                let a = entry.arousal;
                let s = entry.shape;
                let r = entry.relation;
                let t = entry.time;

                // Verify values are non-zero (not fallback defaults for known chars)
                let is_default = s == 0x01 && r == 0x01 && v == 0x80 && a == 0x80 && t == 0x03;
                if is_default {
                    fails += 1;
                    details.push(format!("⚠️  U+{:04X} {} — ALL DEFAULTS (Sphere/neutral) → {}", cp, name, expected));
                } else {
                    details.push(format!("✅ U+{:04X} {} — S={} R={} V=0x{:02X} A=0x{:02X} T={}",
                        cp, name, s, r, v, a, t));
                }
            }
            None => {
                fails += 1;
                details.push(format!("❌ U+{:04X} {} — NOT FOUND in UCD table", cp, name));
            }
        }
    }

    let elapsed = start.elapsed();
    details.push(format!("Lookup time: {:?} for {} entries", elapsed, test_cases.len()));

    // Test coverage: how many of 8,846 expected L0 codepoints exist?
    let mut found = 0;
    let mut missing = 0;
    for cp in 0x2190..=0x27FF {  // Arrows + misc symbols (should be SDF)
        if ucd::lookup(cp).is_some() { found += 1; } else { missing += 1; }
    }
    for cp in 0x1F600..=0x1F64F {  // Emoticons
        if ucd::lookup(cp).is_some() { found += 1; } else { missing += 1; }
    }
    details.push(format!("Sample coverage: {} found, {} missing (arrows+emoticons)", found, missing));

    let check_elapsed = check_start.elapsed();
    details.push(format!("Check time: {:?}", check_elapsed));

    if fails == 0 && missing < 50 {
        CheckResult::pass("F1 UCD Encode", &format!("OK — {} anchors, {} coverage [{:?}]", test_cases.len(), found, check_elapsed))
            .with_details(details)
    } else {
        CheckResult::fail("F1 UCD Encode", &format!(
            "{} anchor issues, {} missing [{:?}]", fails, missing, check_elapsed
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// F2: Molecule encode → verify roundtrip, size, values
// ═══════════════════════════════════════════════════════════════

pub fn check_molecule_roundtrip() -> CheckResult {
    println!("  [F2] Molecule — encode roundtrip...");
    let check_start = Instant::now();
    let mut details = Vec::new();
    let mut fails = 0;

    let test_cps = [0x1F525u32, 0x1F60A, 0x2764, 0x25CF, 0x2208, 0x222B];

    let start = Instant::now();

    for cp in &test_cps {
        let chain = olang::mol::encoder::encode_codepoint(*cp);
        let bytes = chain.to_bytes();
        let hash = chain.chain_hash();

        if chain.is_empty() {
            fails += 1;
            details.push(format!("❌ U+{:04X} — encode returned EMPTY chain", cp));
            continue;
        }

        // Verify bytes are valid (non-zero for important dims)
        let mol = olang::mol::molecular::Molecule::from_u16(chain.0[0]);
        let size = std::mem::size_of_val(&mol);
        details.push(format!("✅ U+{:04X} — {} mol(s), {}B wire, hash={:016X}, struct={}B RAM",
            cp, chain.len(), bytes.len(), hash, size));

        // Roundtrip: bytes → Molecule → bytes
        if chain.len() == 1 {
            let wire = mol.to_bytes();
            let rebuilt = olang::mol::molecular::Molecule::from_bytes_v2(&wire);
            if rebuilt.to_bytes() != wire {
                fails += 1;
                details.push(format!("  ❌ roundtrip MISMATCH for U+{:04X}", cp));
            }
        }
    }

    let elapsed = start.elapsed();
    details.push(format!("Encode time: {:?} for {} codepoints", elapsed, test_cps.len()));

    // Measure struct size
    let mol_size = std::mem::size_of::<olang::mol::molecular::Molecule>();
    details.push(format!("Molecule struct size: {} bytes (v2 target: 2 bytes)", mol_size));

    let check_elapsed = check_start.elapsed();
    details.push(format!("Check time: {:?}", check_elapsed));

    if fails == 0 {
        CheckResult::pass("F2 Molecule Roundtrip", &format!(
            "OK — {} roundtrips, struct={}B [{:?}]", test_cps.len(), mol_size, check_elapsed
        ))
            .with_details(details)
    } else {
        CheckResult::fail("F2 Molecule Roundtrip", &format!(
            "{} roundtrip failures [{:?}]", fails, check_elapsed
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// F3: LCA — verify compose rules per dimension
// ═══════════════════════════════════════════════════════════════

pub fn check_lca_rules() -> CheckResult {
    println!("  [F3] LCA — compose rule verification...");
    let check_start = Instant::now();
    let mut details = Vec::new();
    let mut fails = 0;

    // Create 2 molecules with known different values
    let mol_a = olang::mol::molecular::Molecule::raw(0x01, 0x02, 0x20, 0xE0, 0x01); // low V, high A
    let mol_b = olang::mol::molecular::Molecule::raw(0x03, 0x04, 0xE0, 0x40, 0x03); // high V, low A

    let chain_a = olang::mol::molecular::MolecularChain::single(mol_a);
    let chain_b = olang::mol::molecular::MolecularChain::single(mol_b);

    let result = olang::mol::lca::lca(&chain_a, &chain_b);

    if result.is_empty() {
        return CheckResult::fail("F3 LCA Rules", "LCA returned EMPTY chain");
    }

    let r = olang::mol::molecular::Molecule::from_u16(result.0[0]);

    // v2 rules:
    //   V = amplify → should be >= max(0x20, 0xE0) = 0xE0, NOT average 0x80
    //   A = max()   → should be max(0xE0, 0x40) = 0xE0, NOT average 0x90
    let v_avg = (0x20u16 + 0xE0u16) / 2;  // 0x80 = wrong
    let a_max = 0xE0u8;  // correct
    let a_avg = (0xE0u16 + 0x40u16) / 2;  // 0x90 = wrong

    // Check Valence: should NOT be average
    let rv = r.valence_u8();
    let ra = r.arousal_u8();
    let v_is_avg = (rv as i16 - v_avg as i16).unsigned_abs() < 5;
    if v_is_avg {
        fails += 1;
        details.push(format!("❌ V = 0x{:02X} ≈ avg(0x20,0xE0)=0x{:02X} — should AMPLIFY, not average",
            rv, v_avg));
    } else {
        details.push(format!("✅ V = 0x{:02X} (not avg 0x{:02X})", rv, v_avg));
    }

    // Check Arousal: should be max
    if ra != a_max {
        let a_is_avg = (ra as i16 - a_avg as i16).unsigned_abs() < 5;
        if a_is_avg {
            fails += 1;
            details.push(format!("❌ A = 0x{:02X} ≈ avg(0xE0,0x40)=0x{:02X} — should be max()=0xE0",
                ra, a_avg));
        } else {
            details.push(format!("⚠️  A = 0x{:02X} (not max 0x{:02X}, not avg 0x{:02X})",
                ra, a_max, a_avg));
        }
    } else {
        details.push(format!("✅ A = 0x{:02X} = max(0xE0,0x40)", ra));
    }

    details.push(format!("LCA result: S={} R={} V=0x{:02X} A=0x{:02X} T={}",
        r.shape_u8(), r.relation_u8(), rv, ra, r.time_u8()));

    let check_elapsed = check_start.elapsed();
    details.push(format!("Check time: {:?}", check_elapsed));

    if fails == 0 {
        CheckResult::pass("F3 LCA Rules", &format!("OK — compose rules correct [{:?}]", check_elapsed))
            .with_details(details)
    } else {
        CheckResult::fail("F3 LCA Rules", &format!("{} compose rule violations [{:?}]", fails, check_elapsed))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// F4: Hebbian — verify strengthen/decay math properties
// ═══════════════════════════════════════════════════════════════

pub fn check_hebbian_math() -> CheckResult {
    println!("  [F4] Hebbian — mathematical properties...");
    let check_start = Instant::now();
    let mut details = Vec::new();
    let mut fails = 0;

    // Property 1: strengthen should increase weight (monotonic)
    let w0 = 0.5f32;
    let w1 = silk::hebbian::hebbian_strengthen(w0, 1.0);
    if w1 <= w0 {
        fails += 1;
        details.push(format!("❌ strengthen({}, reward=1.0) = {} — NOT monotonic increasing", w0, w1));
    } else {
        details.push(format!("✅ strengthen({}, reward=1.0) = {} — monotonic ✓", w0, w1));
    }

    // Property 2: strengthen output ∈ [0.0, 1.0] (bounded)
    for &w in &[0.0f32, 0.1, 0.5, 0.9, 0.99, 1.0] {
        let result = silk::hebbian::hebbian_strengthen(w, 1.0);
        if !(0.0..=1.0).contains(&result) {
            fails += 1;
            details.push(format!("❌ strengthen({}) = {} — OUT OF BOUNDS [0,1]", w, result));
        }
    }
    details.push("✅ strengthen bounded [0,1] for all test inputs".into());

    // Property 3: decay should decrease weight
    let decayed = silk::hebbian::hebbian_decay(0.8, 86_400_000_000_000); // 1 day in ns
    if decayed >= 0.8 {
        fails += 1;
        details.push(format!("❌ decay(0.8, 1day) = {} — NOT decreasing", decayed));
    } else {
        details.push(format!("✅ decay(0.8, 1day) = {} — decreasing ✓", decayed));
    }

    // Property 4: decay output ∈ [0.0, 1.0]
    if !(0.0..=1.0).contains(&decayed) {
        fails += 1;
        details.push(format!("❌ decay output {} — OUT OF BOUNDS", decayed));
    }

    // Property 5: Fibonacci sequence correct
    let fib_vals: Vec<u32> = (0..10).map(silk::hebbian::fib).collect();
    let expected = [1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
    if fib_vals != expected {
        fails += 1;
        details.push(format!("❌ Fibonacci: {:?} ≠ {:?}", fib_vals, expected));
    } else {
        details.push(format!("✅ Fibonacci(0..10) = {:?}", fib_vals));
    }

    // Property 6: φ⁻¹ ≈ 0.618
    let phi_inv = silk::hebbian::PHI_INV;
    if (phi_inv - 0.618).abs() > 0.001 {
        fails += 1;
        details.push(format!("❌ φ⁻¹ = {} — expected ≈ 0.618", phi_inv));
    } else {
        details.push(format!("✅ φ⁻¹ = {} ≈ 0.618", phi_inv));
    }

    // Property 7: should_promote threshold
    let promotes_at_phi = silk::hebbian::should_promote(silk::hebbian::PROMOTE_WEIGHT, 8, 5);
    let no_promote_low = silk::hebbian::should_promote(0.3, 8, 5);
    if !promotes_at_phi {
        fails += 1;
        details.push(format!("❌ should_promote at PROMOTE_WEIGHT={} — returned false", silk::hebbian::PROMOTE_WEIGHT));
    }
    if no_promote_low {
        fails += 1;
        details.push("❌ should_promote at w=0.3 — returned true (should be false)".into());
    }
    details.push(format!("✅ promote threshold = {} (φ⁻¹+φ⁻³ ≈ 0.854)", silk::hebbian::PROMOTE_WEIGHT));

    let check_elapsed = check_start.elapsed();
    details.push(format!("Check time: {:?}", check_elapsed));

    if fails == 0 {
        CheckResult::pass("F4 Hebbian Math", &format!("OK — 7 properties [{:?}]", check_elapsed))
            .with_details(details)
    } else {
        CheckResult::fail("F4 Hebbian Math", &format!("{} violations [{:?}]", fails, check_elapsed))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// F5: Emotion pipeline — sentence_affect with known inputs
// ═══════════════════════════════════════════════════════════════

pub fn check_emotion_pipeline() -> CheckResult {
    println!("  [F5] Emotion — sentence_affect real test...");
    let check_start = Instant::now();
    let mut details = Vec::new();
    let mut fails = 0;

    let test_cases = [
        ("tôi rất vui hôm nay", "positive"),
        ("buồn quá", "negative"),
        ("bình thường thôi", "neutral"),
    ];

    let start = Instant::now();

    for (text, expected_tone) in &test_cases {
        let result = context::emotion::sentence_affect(text);

        details.push(format!("{}: V={:.2} A={:.2} D={:.2} I={:.2}",
            text, result.valence, result.arousal, result.dominance, result.intensity));

        // Basic sanity: values should be bounded
        if result.valence < -1.0 || result.valence > 1.0 {
            fails += 1;
            details.push(format!("  ❌ valence {} OUT OF BOUNDS [-1,1]", result.valence));
        }
        if result.arousal < 0.0 || result.arousal > 1.0 {
            fails += 1;
            details.push(format!("  ❌ arousal {} OUT OF BOUNDS [0,1]", result.arousal));
        }

        // Tone direction check
        match *expected_tone {
            "positive" => {
                if result.valence <= 0.0 {
                    details.push(format!("  ⚠️  expected positive valence, got {}", result.valence));
                }
            }
            "negative" => {
                if result.valence >= 0.0 {
                    details.push(format!("  ⚠️  expected negative valence, got {}", result.valence));
                }
            }
            _ => {}
        }
    }

    let elapsed = start.elapsed();
    details.push(format!("sentence_affect time: {:?} for {} inputs", elapsed, test_cases.len()));

    let check_elapsed = check_start.elapsed();
    details.push(format!("Check time: {:?}", check_elapsed));

    if fails == 0 {
        CheckResult::pass("F5 Emotion Pipeline", &format!(
            "OK — {} texts, bounds OK [{:?}]", test_cases.len(), check_elapsed
        ))
            .with_details(details)
    } else {
        CheckResult::fail("F5 Emotion Pipeline", &format!("{} violations [{:?}]", fails, check_elapsed))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// F6: Performance — measure key operations, find bottlenecks
// ═══════════════════════════════════════════════════════════════

pub fn check_performance() -> CheckResult {
    println!("  [F6] PERF — bottleneck detection...");
    let mut details = Vec::new();

    // Benchmark: UCD lookup
    let start = Instant::now();
    for cp in 0x1F600..0x1F680 {
        let _ = ucd::lookup(cp);
    }
    let ucd_time = start.elapsed();
    let ucd_per = ucd_time.as_nanos() / 128;
    details.push(format!("UCD lookup: {:?} / 128 calls = {}ns/call", ucd_time, ucd_per));

    // Benchmark: encode_codepoint
    let start = Instant::now();
    for cp in 0x1F600..0x1F680 {
        let _ = olang::mol::encoder::encode_codepoint(cp);
    }
    let encode_time = start.elapsed();
    let encode_per = encode_time.as_nanos() / 128;
    details.push(format!("encode_codepoint: {:?} / 128 calls = {}ns/call", encode_time, encode_per));

    // Benchmark: LCA
    let chain_a = olang::mol::encoder::encode_codepoint(0x1F525);
    let chain_b = olang::mol::encoder::encode_codepoint(0x1F60A);
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = olang::mol::lca::lca(&chain_a, &chain_b);
    }
    let lca_time = start.elapsed();
    let lca_per = lca_time.as_nanos() / 1000;
    details.push(format!("LCA: {:?} / 1000 calls = {}ns/call", lca_time, lca_per));

    // Benchmark: sentence_affect
    let start = Instant::now();
    for _ in 0..100 {
        let _ = context::emotion::sentence_affect("tôi rất vui hôm nay");
    }
    let affect_time = start.elapsed();
    let affect_per = affect_time.as_micros() / 100;
    details.push(format!("sentence_affect: {:?} / 100 calls = {}µs/call", affect_time, affect_per));

    // Benchmark: Hebbian strengthen
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = silk::hebbian::hebbian_strengthen(0.5, 1.0);
    }
    let heb_time = start.elapsed();
    let heb_per = heb_time.as_nanos() / 10000;
    details.push(format!("hebbian_strengthen: {:?} / 10K calls = {}ns/call", heb_time, heb_per));

    // Benchmark: chain_hash
    let chain = olang::mol::encoder::encode_codepoint(0x1F525);
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = chain.chain_hash();
    }
    let hash_time = start.elapsed();
    let hash_per = hash_time.as_nanos() / 10000;
    details.push(format!("chain_hash: {:?} / 10K calls = {}ns/call", hash_time, hash_per));

    // Find bottleneck
    let times = [
        ("UCD lookup", ucd_per),
        ("encode_codepoint", encode_per),
        ("LCA", lca_per),
        ("sentence_affect", affect_per * 1000), // convert µs→ns
        ("hebbian_strengthen", heb_per),
        ("chain_hash", hash_per),
    ];

    // Thresholds — ngưỡng tối đa chấp nhận được
    // HomeOS real-time: user gõ → phải phản hồi < 50ms
    // Mỗi operation trong pipeline phải nhanh hơn budget
    let thresholds_ns: &[(&str, u128)] = &[
        ("UCD lookup",          1_000),       // < 1µs — O(log n) binary search
        ("encode_codepoint",    5_000),       // < 5µs — lookup + construct
        ("LCA",                 10_000),      // < 10µs — 5D compose
        ("sentence_affect",     10_000_000),  // < 10ms — full NLP pipeline
        ("hebbian_strengthen",  500),         // < 500ns — pure math
        ("chain_hash",          500),         // < 500ns — FNV-1a
    ];

    let mut fails = 0;
    let mut slowest_name = "";
    let mut slowest_ns: u128 = 0;

    for (name, per_ns) in &times {
        let threshold = thresholds_ns.iter()
            .find(|(n, _)| *n == *name)
            .map(|(_, t)| *t)
            .unwrap_or(100_000); // default 100µs

        let over = *per_ns > threshold;
        let ratio = if threshold > 0 { *per_ns as f64 / threshold as f64 } else { 0.0 };

        if over {
            fails += 1;
            details.push(format!("❌ {} = {}ns/call > {}ns threshold ({:.1}x over)",
                name, per_ns, threshold, ratio));
        } else {
            details.push(format!("✅ {} = {}ns/call < {}ns threshold ({:.0}% budget)",
                name, per_ns, threshold, ratio * 100.0));
        }

        if *per_ns > slowest_ns {
            slowest_ns = *per_ns;
            slowest_name = name;
        }
    }

    // Total pipeline budget: 50ms for full T1→T7
    let total_ns = times.iter().map(|(_, t)| t).sum::<u128>();
    let total_ms = total_ns as f64 / 1_000_000.0;
    details.push(format!("Total pipeline estimate: {:.2}ms (budget: 50ms)", total_ms));
    if total_ms > 50.0 {
        fails += 1;
        details.push("❌ Total > 50ms — user will feel lag".into());
    }

    details.push(format!("BOTTLENECK: {} ({}ns/call)", slowest_name, slowest_ns));

    if fails == 0 {
        CheckResult::pass("F6 Performance", &format!(
            "OK — all within budget, total {:.2}ms", total_ms
        ))
            .with_details(details)
    } else {
        CheckResult::fail("F6 Performance", &format!(
            "{} operations exceed threshold — bottleneck: {}", fails, slowest_name
        ))
            .with_details(details)
    }
}
