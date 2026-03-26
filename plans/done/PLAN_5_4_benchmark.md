# PLAN 5.4 — Benchmark

**Phụ thuộc:** 5.1, 5.2, 5.3 (hoặc bất kỳ subset)
**Mục tiêu:** Đo lường performance origin.olang vs Rust binary → validate optimization
**Tham chiếu:** `tools/bench/`

---

## Bối cảnh

```
Mục tiêu performance (từ PLAN_REWRITE.md):
  Logic (emotion, learning): < 2× slower than Rust
  Math (LCA, similarity):    < 5× slower than Rust
  IO (emit, file):           near-native (syscall direct)
```

---

## Benchmark Suite

### 5.4.1 — Micro-benchmarks

```
bench_arithmetic.ol:
  - Add 1M f64 numbers
  - Compare: native VM loop vs JIT'd loop vs Rust

bench_string.ol:
  - Concat 10K strings (10 chars each)
  - Hash 10K strings (FNV-1a)
  - substr 10K times
  - Compare: VM vs Rust String ops

bench_lca.ol:
  - LCA of 10K random chain pairs
  - 5D distance of 10K pairs
  - Compare: VM vs Rust olang::lca()

bench_registry.ol:
  - Insert 10K nodes
  - Lookup 100K times (random keys)
  - Compare: VM registry vs Rust HashMap

bench_silk.ol:
  - co_activate 10K pairs
  - walk_weighted 1K queries
  - Compare: VM silk vs Rust SilkGraph
```

### 5.4.2 — Macro-benchmarks

```
bench_tokenize.ol:
  - tokenize(lexer.ol source) × 1000
  - = real-world compiler workload

bench_parse.ol:
  - parse(tokenize(parser.ol source)) × 100
  - = recursive descent parser workload

bench_compile.ol:
  - Full pipeline: tokenize → parse → analyze → codegen
  - Input: lexer.ol (197 LOC)
  - × 100 iterations

bench_emotion.ol:
  - Process 1000 sentences through emotion pipeline
  - T1-T7 full pipeline
  - = real-world HomeOS workload

bench_dream.ol:
  - STM with 1000 observations → cluster → score
  - = real-world Dream cycle
```

### 5.4.3 — Memory benchmarks

```
bench_memory.ol:
  - Track peak memory over 1000 turns
  - With arena: should be flat
  - Without arena: should grow linearly
  - Report: peak, average, growth rate

bench_molecule_pool.ol:
  - Create/destroy 100K molecules
  - With pool: alloc/free O(1), memory stable
  - Without pool: alloc O(1), no free, memory grows
```

### 5.4.4 — Harness

```
benchmark.ol:

pub fn run_bench(name, fn, iterations) {
  let start = __time_ns();
  let i = 0;
  while i < iterations {
    fn();
    i = i + 1;
  }
  let elapsed = __time_ns() - start;
  let per_op = elapsed / iterations;
  emit(name + ": " + format_ns(elapsed) + " total, " +
       format_ns(per_op) + "/op (" + to_string(iterations) + " iters)\n");
  return { name: name, total_ns: elapsed, per_op_ns: per_op, iters: iterations };
}

// VM builtin needed:
//   __time_ns() → u64 nanoseconds (clock_gettime CLOCK_MONOTONIC)
```

---

## Output Format

```
╔══════════════════════════════════════════════════════════════╗
║ origin.olang Benchmark Report                               ║
╠══════════════════════════════════════════════════════════════╣
║ Benchmark           │ VM (μs)  │ Rust (μs) │ Ratio │ Target ║
╠══════════════════════════════════════════════════════════════╣
║ arithmetic_1M       │ 12,400   │ 2,100     │ 5.9×  │ < 5×   ║
║ string_hash_10K     │ 890      │ 320       │ 2.8×  │ < 5×   ║
║ lca_10K             │ 4,500    │ 1,200     │ 3.8×  │ < 5×   ║
║ tokenize_lexer×1K   │ 45,000   │ 28,000    │ 1.6×  │ < 2×   ║
║ emotion_pipeline×1K │ 23,000   │ 15,000    │ 1.5×  │ < 2×   ║
╠══════════════════════════════════════════════════════════════╣
║ Memory (1K turns)   │ VM (KB)  │ Rust (KB) │ Ratio │        ║
║ peak_memory         │ 2,400    │ 1,800     │ 1.3×  │        ║
║ growth_rate         │ 0 KB/turn│ 0 KB/turn │ =     │        ║
╚══════════════════════════════════════════════════════════════╝
```

---

## Rào cản

```
1. __time_ns() builtin chưa có
   → Thêm: clock_gettime(CLOCK_MONOTONIC) syscall wrapper
   → x86_64: syscall 228 (clock_gettime)
   → ~10 LOC ASM

2. Rust baseline cần compile riêng
   → Viết tương đương Rust benchmarks trong tools/bench/
   → criterion hoặc manual timing

3. Variance cao (cold cache, OS scheduling)
   → Warm-up: chạy 10 iterations trước khi đo
   → Median of 5 runs thay vì mean

4. ⚠️ [THỰC TẾ] origin.olang chưa có interactive mode
   → VM load bytecode → execute → exit 0
   → Benchmark cần run loop, nhưng VM exit ngay
   → Giải pháp: benchmark chạy qua Rust VM (cargo test) trước
   → Hoặc: thêm entry point dispatch vào ASM VM

5. ⚠️ [THỰC TẾ] 7/22 stdlib files không compile được
   → chain.ol, iter.ol: negative numbers (Arith(Sub) in expression position)
   → format.ol, json.ol: typeof keyword chưa hỗ trợ trong expression
   → set.ol: "Enum" là reserved word, dùng làm identifier
   → sort.ol: "Fn" là reserved word, dùng làm type annotation
   → string.ol: "From" là reserved word, dùng làm identifier
   → Benchmark cần stdlib → cần fix parser hoặc rename identifiers
```

---

## Definition of Done

- [ ] __time_ns() VM builtin (x86 + WASM)
- [ ] 5 micro-benchmarks (arithmetic, string, lca, registry, silk)
- [ ] 5 macro-benchmarks (tokenize, parse, compile, emotion, dream)
- [ ] Memory benchmark (peak, growth rate)
- [ ] Rust baseline benchmarks
- [ ] Report generator (formatted table)
- [ ] All logic benchmarks < 2× slower than Rust
- [ ] All math benchmarks < 5× slower than Rust

## Ước tính: 3-5 ngày
