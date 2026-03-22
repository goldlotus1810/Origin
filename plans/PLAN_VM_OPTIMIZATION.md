# PLAN — VM Optimization: Từ 10s → <0.5s cho bootstrap self-hosting

**Ngày:** 2026-03-22
**Vấn đề:** OlangVM quá chậm — i3-4150 chạy 100% CPU, tokenize 200 dòng mất 10s+, self-hosting timeout.
**Mục tiêu:** 50-100x faster. Self-hosting tests hoàn thành trong <30s.
**Tham chiếu:** `plans/PLAN_FORMULA_ENGINE.md`, `docs/HomeOS_SPEC_v3.md` Section IX

---

## Kiểm tra xung đột Spec v3.1

```
✅ APPROVED (không xung đột):
  Opt 1: String compare không allocate    — Bloom Filter precedent (Spec IX.C, line 663)
  Opt 2: Builtin dispatch table           — SDF/Instinct dispatch implicit (Spec I.1, line 159)
  Opt 3: Scope variable cache             — PLAN_5_2 đã approved, Lazy Eval (Spec IX.A, line 647)
  Opt 4: Small-chain SSO                  — 2B architecture (Spec I.2, line 72)
  Opt 5: Keyword hash builtin             — Bloom Filter precedent
  Opt 6: Micro-opts (step batch, flags)   — Lazy Eval (Spec IX.A)

✅ APPROVED (kết hợp KnowTree — Spec IX.I, IX.J mới thêm):
  Opt 7: KnowTree Sampling               — Lấy mẫu từ cây đã học (Spec IX.I)
  Opt 8: Bellman Path Optimization        — Tối ưu đường tìm trong KnowTree (Spec IX.J)

  Cycle: KnowTree (học) → Sampling (lấy mẫu) → Bellman (tối ưu tìm) → feed back KnowTree
  Hiện tại: KnowTree chưa mature → fallback json/udc.json
  Sau:      KnowTree mature → lấy mẫu trực tiếp → nhanh hơn nữa
  Discount factor = φ⁻¹ ≈ 0.618 (đã trong spec line 384)
```

---

## Phase 1 — Quick Wins (không đổi kiến trúc, ~130 LOC)

### Opt 1: String Compare Không Allocate (5-10x)

**Vấn đề:** `char_at(source, pos)` gọi `chain_to_string()` → allocate String mỗi lần.
200 dòng source × ~25 chars/dòng = 5000 `char_at` calls = 5000 String allocations.

**Fix:**

```rust
// TRƯỚC (allocate String mỗi lần):
"__str_char_at" => {
    let s = chain_to_string(&source).unwrap();  // ALLOCATE
    let ch = s.chars().nth(i).unwrap();           // ITERATE ALL
    // ...
}

// SAU (zero allocation):
"__str_char_at" => {
    if i < source.0.len() {
        let mol = source.0[i];           // O(1) index
        let byte = (mol & 0xFF) as u8;
        stack.push(MolecularChain(vec![str_byte_mol(byte)]));  // 1 mol
    }
}
```

Tương tự cho `__str_substr`: slice `source.0[start..end].to_vec()` thay vì decode + re-encode.

**String comparison không allocate:**

```rust
// TRƯỚC: chain_to_string(&a) rồi compare String
// SAU: compare u16 slices trực tiếp
fn chain_cmp_order(a: &MolecularChain, b: &MolecularChain) -> core::cmp::Ordering {
    for (ma, mb) in a.0.iter().zip(b.0.iter()) {
        let ba = (ma & 0xFF) as u8;
        let bb = (mb & 0xFF) as u8;
        match ba.cmp(&bb) {
            core::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }
    a.0.len().cmp(&b.0.len())
}
```

**Files:** `crates/olang/src/exec/vm.rs`
**LOC:** ~60
**Risk:** Thấp

---

### Opt 5: Keyword Hash Builtin (1.5x)

**Vấn đề:** `is_keyword(text)` trong lexer.ol loop 28 entries × full string compare.
500 identifiers × 28 = 14,000 comparisons mỗi tokenize.

**Fix:** Thêm builtin `__str_is_keyword` dùng FNV-1a hash:

```rust
"__str_is_keyword" => {
    let s = vm_pop!(stack, events);
    let bytes: Vec<u8> = s.0.iter().map(|&b| (b & 0xFF) as u8).collect();
    let is_kw = matches!(&bytes[..],
        b"let" | b"fn" | b"if" | b"else" | b"loop" | b"while" |
        b"for" | b"in" | b"return" | b"break" | b"continue" |
        b"emit" | b"type" | b"union" | b"impl" | b"trait" |
        b"match" | b"try" | b"catch" | b"spawn" | b"select" |
        b"timeout" | b"from" | b"use" | b"mod" | b"pub" |
        b"true" | b"false"
    );
    let _ = stack.push(if is_kw {
        MolecularChain::from_number(1.0)
    } else {
        MolecularChain::empty()
    });
}
```

**Tích hợp:** Lowering detect `is_keyword(x)` → emit `__str_is_keyword` thay vì user fn call.
Hoặc: sửa `lexer.ol` gọi builtin trực tiếp.

**Files:** `crates/olang/src/exec/vm.rs`, `stdlib/bootstrap/lexer.ol`
**LOC:** ~40
**Risk:** Thấp

---

### Opt 6: Micro-Optimizations (1.2x)

**6a. Batch step checking:**

```rust
// TRƯỚC: check mỗi step
if steps >= self.max_steps { ... }

// SAU: check mỗi 256 steps
if steps & 0xFF == 0 && steps >= self.max_steps { ... }
```

**6b. EarlyReturn flag thay scan events:**

```rust
// TRƯỚC (scan toàn bộ events mỗi step):
// events.iter().any(|e| matches!(e, VmEvent::EarlyReturn))

// SAU (boolean flag):
let mut early_return = false;
// ... khi set: early_return = true;
// ... khi check: if early_return { ... }
```

**Files:** `crates/olang/src/exec/vm.rs`
**LOC:** ~30
**Risk:** Rất thấp

---

## Phase 2 — Medium Effort (~300 LOC)

### Opt 3: Scope Variable Cache (2-4x)

**Vấn đề:** `LoadLocal("pos")` scan 3-4 scopes × 10-15 vars = 45 string comparisons.
Trong while loop, `pos`, `ch`, `line`, `col` truy cập liên tục.

**Fix: Inline cache** (giống V8 inline cache, đã approved trong PLAN_5_2):

```rust
struct ScopeCache {
    entries: [(u32, usize, usize); 8],  // (name_hash, scope_idx, slot_idx)
    hits: u32,
    misses: u32,
}

// LoadLocal: check cache trước
let hash = fnv1a_hash(name);
if let Some((si, vi)) = cache.lookup(hash) {
    // HIT: O(1)
    return scopes[si][vi].1.clone();
}
// MISS: linear scan, rồi update cache
```

**Invalidation:** Khi `Store(x)` hoặc `ScopeEnd`, invalidate cache entries liên quan.
Conservative: miss → full lookup (correctness preserved).

**Fibonacci threshold:** Cache size = 8 (gần Fib(6)=8, theo spec line 384).

**Files:** `crates/olang/src/exec/vm.rs`
**LOC:** ~100
**Risk:** Thấp

---

### Opt 2: Builtin Dispatch Table (2-3x)

**Vấn đề:** `Op::Call("__eq")` match 207+ string arms tuần tự.

**Fix:** Thêm `Op::CallBuiltin(u8)` vào IR:

```rust
// ir.rs
enum Op {
    CallBuiltin(u8),  // NEW: index vào dispatch table
    Call(String),      // giữ cho user functions
    // ...
}

// Builtin IDs
const BID_EQ: u8 = 0;
const BID_CMP_LT: u8 = 1;
const BID_CHAR_AT: u8 = 2;
const BID_ARRAY_GET: u8 = 3;
// ... top 32 builtins

// VM dispatch:
Op::CallBuiltin(id) => {
    BUILTIN_TABLE[id as usize](stack, events, ...);
}
```

**Lowering:** semantic.rs detect `__eq` etc. → emit `CallBuiltin(BID_EQ)`.

**Files:** `crates/olang/src/exec/ir.rs`, `crates/olang/src/exec/vm.rs`, `crates/olang/src/lang/semantic.rs`
**LOC:** ~200
**Risk:** Trung bình

---

## Phase 3 — Architectural (~300 LOC)

### Opt 4: Small-Chain SSO (2-3x allocation reduction)

**Vấn đề:** `MolecularChain::from_number(42.0)` allocate Vec<u16> (4 elements).
Mỗi arithmetic op = 2 pop + 1 push = 3 Vec allocations.
Hàng triệu ops = hàng triệu heap allocations.

**Fix: Inline storage** cho chains ≤ 4 molecules:

```rust
pub struct MolecularChain {
    len: u8,            // 0-4: inline, 5+: heap
    inline: [u16; 4],   // 8 bytes, zero allocation
    heap: Vec<u16>,     // chỉ dùng khi len > 4
}
```

Covers: tất cả numbers (4 mols), single chars (1 mol), small strings (≤4 chars).
~90% chains trong VM là ≤ 4 molecules → 90% zero heap allocation.

**Spec alignment:** 2B per molecule (Spec I.2 line 72), inline = consistent.

**Files:** `crates/olang/src/mol/molecular.rs`, `crates/olang/src/exec/vm.rs`
**LOC:** ~300
**Risk:** Cao (changes fundamental type, many callsites)

---

## Kết hợp Formula Engine

### Thay Hardcode → Tra cứu UDC Database

**Vấn đề hiện tại (PLAN_FORMULA_ENGINE):**
```rust
// HARDCODE — values cố định, không dùng UDC database
pub fn eval_valence(v: u8) -> ValenceState {
    match v {
        6 => ValenceState { potential: -0.75, force: 0.8, ... },
        // ...
    }
}
```

**Giải pháp: Tra cứu từ `json/udc.json` + `udc_p_table.bin`**

```rust
/// Tính ValenceState từ UDC database thay vì hardcode.
/// Lấy tất cả ký tự có V=v, tính trung bình potential từ P_weight.
///
/// Spec alignment: "Đọc giá trị → biết công thức → biết hình dạng"
/// (Spec I.1, line 35: "1 hàm. 1 điểm. Ra tất cả.")
pub fn eval_valence_from_udc(v: u8, p_table: &[u16; 65536]) -> ValenceState {
    // Lấy tất cả entries có V field = v
    let matching: Vec<u16> = p_table.iter()
        .filter(|&&pw| (pw >> 5) & 0x07 == v as u16)
        .copied()
        .collect();

    if matching.is_empty() {
        return FALLBACK_VALENCE[v as usize]; // hardcode chỉ làm fallback
    }

    // Tính từ dữ liệu thực
    let avg_potential = compute_potential_from_entries(&matching, v);
    let force = -d_potential(avg_potential); // F = -dU/dx

    ValenceState {
        potential: avg_potential,
        force,
        barrier: avg_potential.abs(),
        kind: valence_kind_from_index(v),
    }
}
```

**Tương tự cho R, A, T dimensions.**

**Lợi ích:**
- Giá trị chính xác từ 8,846 UDC chars thay vì hardcode
- Tự động cập nhật khi UDC database thay đổi
- Spec-compliant: "Đọc giá trị → biết công thức"

---

## Thứ tự thực hiện

```
Week 1: Phase 1 (Opt 1 + 5 + 6)       → ~130 LOC, 5-15x faster
Week 2: Phase 2 (Opt 3 + 2)            → ~300 LOC, thêm 4-12x
Week 3: Phase 3 (Opt 4)                → ~300 LOC, thêm 2-3x
Week 3: Formula Engine UDC integration → ~200 LOC

Tổng: ~930 LOC, ước tính 50-100x faster
```

---

## Spec Amendments cần thêm vào V3.md

## Phase 4 — KnowTree Integration (Opt 7 + 8, ~250 LOC)

### Opt 7: KnowTree Sampling — Thay hardcode bằng lấy mẫu (2-3x cho Formula Engine)

**Vấn đề:** `eval_valence(6)` trả về `potential: -0.75` HARDCODE.
Không dùng UDC database, không học, không cập nhật.

**Giải pháp: 3 tầng fallback**

```
Tầng 1 (mature): KnowTree L0-L3 đã học
  → Lấy K nodes có V=6 (K = Fib(n) theo maturity)
  → Tính trung bình potential từ P_weight thực
  → Feed back: strengthen Hebbian edge cho nodes hay dùng

Tầng 2 (bootstrap): json/udc.json + udc_p_table.bin
  → Scan 8,846 entries, filter V=6
  → Tính từ dữ liệu gốc
  → Cache kết quả (Fibonacci-sized cache)

Tầng 3 (emergency): Hardcode constants hiện tại
  → Fallback cuối cùng nếu cả 2 tầng trên rỗng
```

**Rust implementation:**

```rust
pub fn eval_valence_adaptive(v: u8, knowtree: Option<&KnowTree>, p_table: &[u16]) -> ValenceState {
    // Tầng 1: KnowTree (nếu mature)
    if let Some(kt) = knowtree {
        let maturity = kt.generation();  // gen0..gen3
        let k = fibonacci_sample_size(maturity); // Fib(3)=2..Fib(10)=55
        let samples = kt.sample_by_dimension(Dim::V, v, k);
        if !samples.is_empty() {
            return compute_valence_from_samples(&samples);
        }
    }

    // Tầng 2: UDC database
    let matching: Vec<u16> = p_table.iter()
        .filter(|&&pw| pw != 0 && (pw >> 5) & 0x07 == v as u16)
        .copied().collect();
    if !matching.is_empty() {
        return compute_valence_from_pweights(&matching, v);
    }

    // Tầng 3: Hardcode fallback
    HARDCODE_VALENCE[v as usize]
}

fn fibonacci_sample_size(gen: u8) -> usize {
    match gen {
        0 => 2,   // Fib(3) — UDC gốc, stable
        1 => 5,   // Fib(5)
        2 => 13,  // Fib(7)
        _ => 55,  // Fib(10) — mới học, cần nhiều mẫu
    }
}
```

**Tương tự cho eval_arousal, eval_relation, eval_time.**

**Files:** `crates/olang/src/mol/formula.rs` (hoặc tạo `formula_adaptive.rs`)
**LOC:** ~150
**Risk:** Thấp (fallback = giữ nguyên hành vi cũ)

---

### Opt 8: Bellman Path — Tối ưu tìm kiếm KnowTree (2-3x cho lookups)

**Vấn đề:** Tìm node trong KnowTree = traverse cây L0→L1→L2→L3.
Hiện tại: scan tuyến tính hoặc hash lookup.
Sau khi KnowTree lớn (>100K nodes): cần tối ưu đường đi.

**Giải pháp: Q-table nhỏ gọn**

```rust
/// Q-table cho KnowTree traversal.
/// Mỗi node lưu "đi hướng nào nhanh nhất cho từng loại query".
struct KnowTreePathCache {
    /// (node_hash, query_dim, best_child_idx, q_value)
    entries: [(u64, u8, u8, f32); 55],  // Fib(10) = 55 entries
}

impl KnowTreePathCache {
    fn lookup(&self, node: u64, dim: u8) -> Option<u8> {
        self.entries.iter()
            .find(|(h, d, _, q)| *h == node && *d == dim && *q > 0.3)
            .map(|(_, _, child, _)| *child)
    }

    fn update(&mut self, node: u64, dim: u8, child: u8, hit: bool) {
        // Q(s,a) = Q(s,a) + α × (reward - Q(s,a))
        // α = 0.1, reward = 1.0 nếu hit, 0.0 nếu miss
        // Decay: Q *= φ⁻¹ mỗi cycle (giống Hebbian decay)
        let alpha = 0.1f32;
        let reward = if hit { 1.0 } else { 0.0 };
        let phi_inv = 0.618f32;

        if let Some(entry) = self.entries.iter_mut()
            .find(|(h, d, _, _)| *h == node && *d == dim)
        {
            entry.2 = child;
            entry.3 = entry.3 + alpha * (reward - entry.3);
        } else {
            // Evict lowest Q entry
            if let Some(min) = self.entries.iter_mut()
                .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap())
            {
                *min = (node, dim, child, reward * alpha);
            }
        }

        // Decay all entries (φ⁻¹)
        for e in &mut self.entries {
            e.3 *= phi_inv;
        }
    }
}
```

**Files:** `crates/olang/src/storage/knowtree.rs` (thêm cache)
**LOC:** ~100
**Risk:** Thấp (cache miss = full lookup, correctness preserved)

---

## Thứ tự thực hiện (cập nhật)

```
Phase 1: Opt 1 + 5 + 6                 → ~130 LOC, 5-15x faster
Phase 2: Opt 3 + 2                      → ~300 LOC, thêm 4-12x
Phase 3: Opt 4 (SSO)                    → ~300 LOC, thêm 2-3x
Phase 4: Opt 7 + 8 (KnowTree + Bellman) → ~250 LOC, thêm 2-3x + adaptive

Tổng: ~980 LOC, ước tính 50-150x faster
     + hệ thống TỰ HỌC: càng dùng → càng nhanh
```

---

## Spec Amendments (đã thêm vào V3.md Section IX)

### Section IX.H — String Fingerprinting (NEW)

```
String equality dùng FNV-1a hash fingerprint:
  h(A) ≠ h(B) → A ≠ B (O(1), deterministic)
  h(A) = h(B) → compare full (fallback)
  Collision rate < 0.001% cho 8,846 chars.
  Tương thích Bloom Filter (Section IX.C).
```

### Section IX.I — Builtin Dispatch Tables (NEW)

```
Builtin operations dùng dispatch table O(1):
  CallBuiltin(id) → function_table[id]()
  Tương thích SDF dispatch (18 primitives, Section I.1)
  và Instinct dispatch (7 instincts, Section V.2).
```

### Section X — Cache Invalidation Rules (NEW, sau Checkpoint 5)

```
Variable Cache:
  Store(x) → invalidate cache[hash(x)]
  ScopeEnd → invalidate scope entries
  Conservative: miss → full lookup (correctness preserved)

Fibonacci sizing: cache = 8 entries (Fib(6)=8)
```
