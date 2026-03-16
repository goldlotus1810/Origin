//! # bench — Benchmark HomeOS với multilingual sentiment data
//!
//! Đọc TSV (text \t label) → chạy qua BookReader + HomeRuntime
//! So sánh emotion.valence với ground truth label.

use std::fs;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use agents::book::BookReader;
use runtime::origin::HomeRuntime;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: bench <path/to/file.tsv> [max_rows]");
        eprintln!("  TSV format: text<TAB>label");
        eprintln!("  Labels: positive / neutral / negative");
        std::process::exit(1);
    }

    let path     = &args[1];
    let max_rows = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(200usize);

    println!("○ HomeOS Benchmark");
    println!("File    : {}", path);
    println!("Max rows: {}", max_rows);
    println!("UCD     : {} entries", ucd::table_len());
    println!();

    // Đọc data
    let file = match fs::File::open(path) {
        Ok(f)  => f,
        Err(e) => { eprintln!("Cannot open {}: {}", path, e); std::process::exit(1); }
    };

    let mut rows: Vec<(String, String)> = Vec::new(); // (text, label)
    let reader = BufReader::new(file);
    let mut header_skipped = false;

    for line in reader.lines() {
        let Ok(line) = line else { continue };
        let line = line.trim().to_string();
        if line.is_empty() { continue; }

        // Skip header row
        if !header_skipped {
            header_skipped = true;
            if line.starts_with("text") { continue; }
        }

        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() < 2 { continue; }

        let text  = parts[0].trim().to_string();
        let label = parts[1].trim().to_string();
        if text.is_empty() || label.is_empty() { continue; }

        rows.push((text, label));
        if rows.len() >= max_rows { break; }
    }

    println!("Loaded: {} rows", rows.len());
    println!();

    // Benchmark 1: BookReader accuracy
    println!("── BookReader Sentiment Accuracy ─────────────────");
    let book = BookReader::new();
    let mut correct = 0usize;
    let mut total   = 0usize;
    let mut pos_vals = Vec::new();
    let mut neu_vals = Vec::new();
    let mut neg_vals = Vec::new();

    let t0 = Instant::now();

    for (text, label) in &rows {
        let records = book.read(text);
        if records.is_empty() { continue; }

        let avg_v = records.iter().map(|r| r.emotion.valence).sum::<f32>()
            / records.len() as f32;

        // Classify theo valence
        let predicted = if avg_v > 0.15 { "positive" }
                        else if avg_v < -0.15 { "negative" }
                        else { "neutral" };

        if predicted == label.as_str() { correct += 1; }
        total += 1;

        match label.as_str() {
            "positive" => pos_vals.push(avg_v),
            "neutral"  => neu_vals.push(avg_v),
            "negative" => neg_vals.push(avg_v),
            _          => {}
        }
    }

    let elapsed = t0.elapsed();
    let accuracy = if total > 0 { correct as f32 / total as f32 * 100.0 } else { 0.0 };

    println!("Accuracy : {:.1}% ({}/{})", accuracy, correct, total);
    println!("Speed    : {:.0} rows/s", total as f32 / elapsed.as_secs_f32());
    println!();

    // Valence distribution per label
    fn avg(v: &[f32]) -> f32 {
        if v.is_empty() { 0.0 } else { v.iter().sum::<f32>() / v.len() as f32 }
    }
    println!("── Valence Distribution ──────────────────────────");
    println!("positive: n={:4}  avg_V={:+.3}", pos_vals.len(), avg(&pos_vals));
    println!("neutral : n={:4}  avg_V={:+.3}", neu_vals.len(), avg(&neu_vals));
    println!("negative: n={:4}  avg_V={:+.3}", neg_vals.len(), avg(&neg_vals));

    // Benchmark 2: HomeRuntime pipeline
    println!();
    println!("── HomeRuntime Pipeline ──────────────────────────");
    let mut rt = HomeRuntime::new(0xBEEF_BEEF_u64);
    let sample_size = 20.min(rows.len());
    let t1 = Instant::now();

    for (text, _) in rows.iter().take(sample_size) {
        let _ = rt.process_text(text, t1.elapsed().as_nanos() as i64);
    }

    let rt_elapsed = t1.elapsed();
    println!("Processed: {} texts in {:.1}ms", sample_size, rt_elapsed.as_millis());
    println!("f(x) final: {:.3}", rt.fx());
    println!("Tone final: {:?}", rt.tone());

    // Sample predictions
    println!();
    println!("── Sample Predictions (first 10) ─────────────────");
    for (text, label) in rows.iter().take(10) {
        let records = book.read(text);
        let avg_v = if records.is_empty() { 0.0 }
            else { records.iter().map(|r| r.emotion.valence).sum::<f32>() / records.len() as f32 };
        let predicted = if avg_v > 0.15 { "pos" } else if avg_v < -0.15 { "neg" } else { "neu" };
        let truth = match label.as_str() { "positive" => "pos", "negative" => "neg", _ => "neu" };
        let ok = if predicted == truth { "✓" } else { "✗" };
        let short = if text.len() > 50 { &text[..50] } else { text.as_str() };
        println!("  {} [{:3}→{:3}] {}", ok, truth, predicted, short);
    }

    // Benchmark 3: Per-component benchmarks (lookup/LCA/Silk/FFR)
    println!();
    println!("── Component Benchmarks ──────────────────────────");

    // 3a. UCD Lookup
    {
        let cps: Vec<u32> = vec![0x1F525, 0x1F4A7, 0x2744, 0x1F9E0, 0x2764, 0x1F60A, 0x1F62D, 0x1F4AA];
        let t = Instant::now();
        let iters = 10000;
        for _ in 0..iters {
            for &cp in &cps {
                let _ = olang::encoder::encode_codepoint(cp);
            }
        }
        let elapsed = t.elapsed();
        let ops = (iters * cps.len()) as f64 / elapsed.as_secs_f64();
        println!("UCD lookup   : {:.0} ops/s ({:.1}µs per batch of {})", ops, elapsed.as_micros() as f64 / iters as f64, cps.len());
    }

    // 3b. LCA
    {
        let c1 = olang::encoder::encode_codepoint(0x1F525);
        let c2 = olang::encoder::encode_codepoint(0x1F4A7);
        let t = Instant::now();
        let iters = 10000;
        for _ in 0..iters {
            let _ = olang::lca::lca(&c1, &c2);
        }
        let elapsed = t.elapsed();
        let ops = iters as f64 / elapsed.as_secs_f64();
        println!("LCA          : {:.0} ops/s ({:.1}µs per call)", ops, elapsed.as_micros() as f64 / iters as f64);
    }

    // 3c. Silk co_activate
    {
        use silk::graph::SilkGraph;
        use silk::edge::EmotionTag;

        let mut graph = SilkGraph::new();
        let t = Instant::now();
        let iters = 10000;
        for i in 0..iters {
            graph.co_activate(
                i as u64, (i + 1) as u64,
                EmotionTag::NEUTRAL,
                0.5, i as i64,
            );
        }
        let elapsed = t.elapsed();
        let ops = iters as f64 / elapsed.as_secs_f64();
        println!("Silk activate: {:.0} ops/s ({:.1}µs per call)", ops, elapsed.as_micros() as f64 / iters as f64);
    }

    // 3d. FFR point generation
    {
        let t = Instant::now();
        let iters = 100000u64;
        for n in 0..iters {
            let _ = vsdf::ffr::FfrPoint::at(n);
        }
        let elapsed = t.elapsed();
        let ops = iters as f64 / elapsed.as_secs_f64();
        println!("FFR point    : {:.0} ops/s ({:.1}µs per 1000)", ops, elapsed.as_micros() as f64 / (iters as f64 / 1000.0));
    }

    // 3e. chain_hash
    {
        let chain = olang::encoder::encode_codepoint(0x1F525);
        let t = Instant::now();
        let iters = 100000;
        for _ in 0..iters {
            let _ = chain.chain_hash();
        }
        let elapsed = t.elapsed();
        let ops = iters as f64 / elapsed.as_secs_f64();
        println!("chain_hash   : {:.0} ops/s ({:.1}ns per call)", ops, elapsed.as_nanos() as f64 / iters as f64);
    }

    println!();
    println!("○ Done");
}
