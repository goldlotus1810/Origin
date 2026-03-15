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

    println!();
    println!("○ Done");
}
