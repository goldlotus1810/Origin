// stdlib/homeos/benchmark.ol — Benchmark harness for origin.olang
// PLAN 5.4: Measure and compare performance of VM operations.
// Uses __time_ns() builtin (clock_gettime CLOCK_MONOTONIC).

// ── Harness ─────────────────────────────────────────────────────────

pub fn run_bench(name, bench_fn, iterations) {
  // Warm-up: 10 iterations (avoid cold cache effects)
  let warmup = 10;
  if warmup > iterations { warmup = iterations; }
  let w = 0;
  while w < warmup {
    bench_fn();
    w = w + 1;
  }
  // Measure
  let start = time();
  let i = 0;
  while i < iterations {
    bench_fn();
    i = i + 1;
  }
  let elapsed = time() - start;
  let per_op = elapsed / iterations;
  return {
    name: name,
    total_ms: elapsed,
    per_op_ms: per_op,
    iterations: iterations
  };
}

pub fn report(results) {
  // Print formatted benchmark report
  emit "=== Benchmark Report ===\n";
  let i = 0;
  while i < len(results) {
    let r = results[i];
    emit r.name;
    emit ": ";
    emit r.total_ms;
    emit "ms total, ";
    emit r.per_op_ms;
    emit "ms/op (";
    emit r.iterations;
    emit " iters)\n";
    i = i + 1;
  }
  emit "========================\n";
}

// ── Micro-benchmarks ────────────────────────────────────────────────

pub fn bench_arithmetic(n) {
  // Add N f64 numbers: sum = 0 + 1 + 2 + ... + (n-1)
  let sum = 0.0;
  let i = 0;
  while i < n {
    sum = sum + i;
    i = i + 1;
  }
  return sum;
}

pub fn bench_arithmetic_mul(n) {
  // Multiply chain: product = 1 * 1.001 * 1.001 * ...
  let product = 1.0;
  let i = 0;
  while i < n {
    product = product * 1.001;
    i = i + 1;
  }
  return product;
}

pub fn bench_string_concat(n) {
  // Concatenate N short strings
  let result = "";
  let i = 0;
  while i < n {
    result = result + "x";
    i = i + 1;
  }
  return len(result);
}

pub fn bench_hash(n) {
  // Hash N different strings
  let i = 0;
  let total = 0;
  while i < n {
    let h = hash_combine(i, i * 7 + 13);
    total = total + h;
    i = i + 1;
  }
  return total;
}

pub fn bench_array_ops(n) {
  // Push N items, then access each
  let arr = [];
  let i = 0;
  while i < n {
    push(arr, i * 2 + 1);
    i = i + 1;
  }
  // Sum all elements
  let sum = 0;
  i = 0;
  while i < n {
    sum = sum + arr[i];
    i = i + 1;
  }
  return sum;
}

// ── Macro-benchmarks ────────────────────────────────────────────────

pub fn bench_fibonacci(n) {
  // Fibonacci sequence (iterative) — classic benchmark
  let a = 0;
  let b = 1;
  let i = 0;
  while i < n {
    let tmp = a + b;
    a = b;
    b = tmp;
    i = i + 1;
  }
  return a;
}

pub fn bench_sieve(n) {
  // Simple prime counting up to N (sieve-like)
  let count = 0;
  let i = 2;
  while i < n {
    let is_prime = true;
    let j = 2;
    while j * j <= i {
      if i - (i / j) * j == 0 {
        is_prime = false;
        j = i;  // break
      }
      j = j + 1;
    }
    if is_prime { count = count + 1; }
    i = i + 1;
  }
  return count;
}

pub fn bench_matrix_mul(size) {
  // Multiply two size×size matrices (flat arrays)
  let n = size;
  let a = [];
  let b = [];
  let c = [];
  // Initialize
  let i = 0;
  while i < n * n {
    push(a, i + 1);
    push(b, (i + 1) * 2);
    push(c, 0);
    i = i + 1;
  }
  // Multiply: c[i][j] = sum(a[i][k] * b[k][j])
  i = 0;
  while i < n {
    let j = 0;
    while j < n {
      let sum = 0;
      let k = 0;
      while k < n {
        sum = sum + a[i * n + k] * b[k * n + j];
        k = k + 1;
      }
      c[i * n + j] = sum;
      j = j + 1;
    }
    i = i + 1;
  }
  return c[0];
}

// ── Memory benchmarks ───────────────────────────────────────────────

pub fn bench_alloc_pattern(turns, allocs_per_turn) {
  // Simulate turn-based allocation pattern
  // With arena: memory should be stable
  // Without arena: memory grows linearly
  let total_allocs = 0;
  let t = 0;
  while t < turns {
    let i = 0;
    while i < allocs_per_turn {
      let data = [0, 0, 0, 0, 0];  // 5 values per alloc
      total_allocs = total_allocs + 1;
      i = i + 1;
    }
    // In real arena: arena_reset() here
    t = t + 1;
  }
  return total_allocs;
}

// ── Helpers ─────────────────────────────────────────────────────────

fn hash_combine(a, b) {
  // Simple hash combine for benchmarking
  return (a * 2654435761 + b) - floor((a * 2654435761 + b) / 4294967296) * 4294967296;
}
