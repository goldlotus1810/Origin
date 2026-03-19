# PLAN 6.2 — Self-Optimize

**Phụ thuộc:** 5.1 (JIT), 5.4 (benchmark), 6.1 (self-update)
**Mục tiêu:** LeoAI profile runtime → phát hiện bottleneck → viết Olang optimization → apply
**Tham chiếu:** `agents/src/hierarchy/leo.rs`, `stdlib/homeos/leo.ol`

---

## Bối cảnh

```
HIỆN TẠI:
  LeoAI có 7 bản năng + self-programming capability
  program(source) → parse → compile → VM → learn
  NHƯNG: chỉ học TRI THỨC, chưa tối ưu CHÍNH MÌNH

SAU PLAN 6.2:
  LeoAI profile: "hàm blend_emotion() chạy 10,000 lần/ngày, tốn 40% CPU"
  LeoAI viết: "optimize blend_emotion: precompute weights"
  LeoAI test: "benchmark trước: 15μs/call, sau: 3μs/call → 5× faster"
  LeoAI install: "o install optimized_emotion.ol"
  → Sinh linh tự cải thiện bản thân
```

---

## Thiết kế

### Profiling data collection

```
Runtime tự động thu thập:

profile_data = {
  function_calls: {
    "blend_emotion": { count: 10000, total_ns: 150000000, avg_ns: 15000 },
    "walk_weighted": { count: 5000,  total_ns: 200000000, avg_ns: 40000 },
    ...
  },
  hot_loops: {
    pc_0x1234: { iterations: 50000, total_ns: 300000000 },
    ...
  },
  memory: {
    peak_kb: 2400,
    arena_resets: 1000,
    promotes: 500,
    pool_allocs: 10000
  },
  cache: {
    var_ic_hit_rate: 0.95,
    registry_hit_rate: 0.87,
    silk_hit_rate: 0.92
  }
}
```

### LeoAI optimization cycle

```
Triggered: mỗi Dream cycle (offline, không block user)

1. ANALYZE profile_data
   → Sort functions by total_ns DESC
   → Top-5 = optimization candidates

2. HYPOTHESIZE
   → Dùng Curiosity instinct: "tại sao blend_emotion chậm?"
   → Dùng Analogy instinct: "blend giống matrix_mul → có thể vectorize"
   → Dùng Abstraction instinct: "10 blend calls cùng weights → precompute"

3. GENERATE optimization Olang code
   → program_experiment(hypothesis, dim, val)
   → Viết optimized version + benchmark

4. VALIDATE
   → Chạy benchmark: before vs after
   → Honesty instinct: "chỉ apply nếu >= 20% improvement"
   → Contradiction instinct: "output phải giống hệt original"

5. PROPOSE
   → Tạo Proposal: "optimize blend_emotion: precompute shared weights"
   → AAM review → approve/reject
   → Approved → install optimized version

6. LEARN
   → Ghi nhớ pattern: "precompute → speedup khi repeated calls"
   → Silk: co_activate("precompute", "repeated_calls", weight=0.9)
   → Next time: tự động recognize similar pattern
```

### Optimization patterns (dạy LeoAI nhận biết)

```
Pattern                          Fix                           Expected
───────────────────────────────────────────────────────────────────────────
Repeated computation with        Memoize/precompute            2-10×
  same inputs

Hot loop with constant           Hoist constant outside        1.5-3×
  expression

String concat in loop            StringBuilder pattern         5-50×

Linear search on sorted data     Binary search                 O(n)→O(log n)

Redundant Silk walk              Cache walk results            2-5×

Repeated tokenize same source    Cache tokenize result         10-100×
```

---

## Tasks

### 6.2.1 — Profiler (~150 LOC Olang)

```
profiler.ol:

pub fn profile_start(name) {
  let entry = get_or_create(name);
  entry.start_ns = __time_ns();
}

pub fn profile_end(name) {
  let entry = profiles[name];
  let elapsed = __time_ns() - entry.start_ns;
  entry.count = entry.count + 1;
  entry.total_ns = entry.total_ns + elapsed;
}

pub fn profile_report() {
  // Sort by total_ns DESC, return top-20
}

pub fn profile_hot_functions(threshold_pct) {
  // Return functions consuming > threshold% of total time
}
```

### 6.2.2 — LeoAI optimizer extension (~200 LOC)

```
optimizer.ol:

pub fn optimize_cycle(profile, knowledge) {
  let candidates = profile_hot_functions(10);  // > 10% of total

  for fn_name in candidates {
    let hypothesis = generate_hypothesis(fn_name, profile);
    if hypothesis != null {
      let optimized = generate_optimization(fn_name, hypothesis);
      let result = validate_optimization(fn_name, optimized);
      if result.speedup >= 1.2 {  // >= 20% improvement
        propose_optimization(fn_name, optimized, result);
      }
    }
  }
}

fn generate_hypothesis(fn_name, profile) {
  let fn_profile = profile[fn_name];

  // Pattern: many calls, short duration → inline candidate
  if fn_profile.count > 1000 && fn_profile.avg_ns < 100 {
    return { type: "inline", reason: "frequent short function" };
  }

  // Pattern: few calls, long duration → algorithmic issue
  if fn_profile.count < 100 && fn_profile.avg_ns > 100000 {
    return { type: "algorithm", reason: "slow function, check complexity" };
  }

  // Pattern: called with same args repeatedly
  if fn_profile.unique_args < fn_profile.count * 0.1 {
    return { type: "memoize", reason: "repeated args" };
  }

  return null;
}
```

### 6.2.3 — AAM approval gate

```
Mọi self-modification PHẢI qua AAM:
  1. LeoAI propose optimization
  2. AAM show user: "LeoAI muốn tối ưu blend_emotion (5× faster). Approve?"
  3. User approve → install
  4. User reject → discard, learn "user không muốn thay đổi này"
```

---

## Rào cản

```
1. Correctness verification
   → Optimization KHÔNG ĐƯỢC thay đổi output
   → Test: run original + optimized trên 100 inputs → assert identical
   → Nếu 1 output khác → reject

2. Infinite optimization loop
   → LeoAI optimize → optimize optimization → optimize²...
   → Giải pháp: cooldown = Fib[n] Dream cycles giữa optimize cùng function
   → Max 3 optimization rounds per function

3. LeoAI viết code sai
   → Sandbox: chạy trong sandbox trước khi install
   → Rollback: giữ version cũ (6.1 module versioning)
   → Worst case: "o rollback module_name"
```

---

## Definition of Done

- [ ] Profiler: function-level + loop-level timing
- [ ] Hot function detection (> 10% of total time)
- [ ] LeoAI hypothesis generation (3+ patterns)
- [ ] Optimization code generation (memoize, inline, precompute)
- [ ] Validation: correctness + speedup measurement
- [ ] AAM approval gate
- [ ] Test: LeoAI detects + optimizes synthetic hot function

## Ước tính: 2-3 tuần
