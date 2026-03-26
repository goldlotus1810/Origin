# bench

> Benchmark tool for HomeOS — measures sentiment accuracy, pipeline throughput, and per-component performance.

## Dependencies
- ucd
- olang (with `std` feature)
- agents (with `std` feature)
- runtime (with `std` feature)
- silk
- vsdf

## Files
| File | Purpose |
|------|---------|
| src/main.rs | Reads TSV sentiment data, benchmarks BookReader accuracy, HomeRuntime pipeline speed, and component-level ops/s |

## Key API
```rust
// main() pipeline:
// 1. Read TSV file: text<TAB>label (positive/neutral/negative)
// 2. Benchmark 1 — BookReader sentiment accuracy:
//    book.read(text) → records → avg valence → classify → compare with label
// 3. Benchmark 2 — HomeRuntime pipeline:
//    rt.process_text(text, ts) for sample_size texts → measure total time
// 4. Benchmark 3 — Component micro-benchmarks:
//    a. UCD lookup:     encode_codepoint() × 10K iterations
//    b. LCA:            lca(&c1, &c2) × 10K iterations
//    c. Silk co_activate: graph.co_activate() × 10K iterations
//    d. FFR point:      FfrPoint::at(n) × 100K iterations
//    e. chain_hash:     chain.chain_hash() × 100K iterations
```

## Usage
```bash
# Run with TSV sentiment data file
cargo run -p bench -- path/to/sentiment.tsv

# Limit to first 100 rows
cargo run -p bench -- path/to/sentiment.tsv 100
```

## TSV Format
```
text	label
I love this product	positive
It's okay	neutral
Terrible experience	negative
```

## Output Example
```
○ HomeOS Benchmark
File    : data.tsv
Max rows: 200

── BookReader Sentiment Accuracy ─────────────────
Accuracy : 72.5% (145/200)
Speed    : 15000 rows/s

── Valence Distribution ──────────────────────────
positive: n=  80  avg_V=+0.350
neutral : n=  60  avg_V=+0.020
negative: n=  60  avg_V=-0.280

── HomeRuntime Pipeline ──────────────────────────
Processed: 20 texts in 2.3ms

── Component Benchmarks ──────────────────────────
UCD lookup   : 2000000 ops/s
LCA          : 500000 ops/s
Silk activate: 1000000 ops/s
FFR point    : 5000000 ops/s
chain_hash   : 10000000 ops/s
```

## Rules
- Read-only — does not modify any files or persist state
- Classification threshold: valence > 0.15 = positive, < -0.15 = negative, else neutral
- HomeRuntime benchmark uses first 20 rows (configurable via `sample_size`)

## Test
```bash
cargo test -p bench
```
