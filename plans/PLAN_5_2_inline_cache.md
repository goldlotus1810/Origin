# PLAN 5.2 — Inline Caching

**Phụ thuộc:** Phase 3 DONE (không cần Phase 4/5.1)
**Mục tiêu:** Cache kết quả lookup → giảm latency cho Registry, Silk, variable access
**Tham chiếu:** `olang/src/storage/registry.rs`, `silk/src/graph.rs`

---

## Bối cảnh

```
HIỆN TẠI:
  Load("x")      → hash("x") → linear scan var_table mỗi lần
  Registry.get(h) → HashMap lookup mỗi lần
  Silk.implicit() → 5D comparison mỗi lần (O(n) neighbors)

SAU PLAN 5.2:
  Load("x")       → inline cache: nếu hash match → O(1) direct access
  Registry.get(h)  → LRU cache: top-N hot nodes cached
  Silk.implicit()  → cache 5D comparison results (invalidate khi evolve)
```

---

## Tasks

### 5.2.1 — Variable inline cache (~80 LOC ASM)

Inline cache = mỗi Load/Store instruction có "cache slot" gắn liền:

```asm
;; Before (current): every Load does full hash scan
op_load:
    call    hash_name           ;; hash
    call    var_load_hash       ;; linear scan

;; After: inline cache per-site
;;   cache = [cached_hash:8][cached_slot:8]  (16 bytes per Load site)
;;
;; Mỗi Load bytecode instruction nhớ slot cuối cùng tìm thấy.
;; Nếu hash match → trực tiếp truy cập slot → O(1)

op_load_cached:
    call    hash_name           ;; rax = hash
    ;; Check inline cache
    lea     rcx, [ic_table + r13*16]  ;; r13 = PC → cache slot
    cmp     rax, (%rcx)        ;; cached_hash == hash?
    jne     .ic_miss
    ;; Cache HIT
    mov     rsi, 8(%rcx)       ;; cached_slot → direct index
    ;; Load from var_table[slot]
    ...
    jmp     vm_loop
.ic_miss:
    ;; Full lookup
    call    var_load_hash
    ;; Update cache
    mov     (%rcx), rax        ;; save hash
    mov     8(%rcx), rsi       ;; save slot
    jmp     vm_loop
```

IC table: `256 entries × 16 bytes = 4 KB` (pre-allocated).

### 5.2.2 — Registry LRU cache (~100 LOC Olang)

```
registry_cache.ol:

let CACHE_SIZE = 64;  // Fib[?] ≈ 64 → dùng 55 (Fib[10])

pub fn cache_new() {
  return { entries: [], hits: 0, misses: 0 };
}

pub fn cache_get(cache, hash) {
  // Linear search (small N → fast)
  // Move to front on hit (LRU)
  // Miss → return null, caller does full lookup
}

pub fn cache_put(cache, hash, node) {
  // Add to front
  // Evict last if len > CACHE_SIZE
}

pub fn cache_invalidate(cache, hash) {
  // Remove specific entry (khi node bị amend/evolve)
}

pub fn cache_stats(cache) {
  return { hits: cache.hits, misses: cache.misses,
           ratio: cache.hits / (cache.hits + cache.misses) };
}
```

### 5.2.3 — Silk similarity cache (~100 LOC Olang)

```
silk_cache.ol:

// 5D similarity = expensive (5 dimensions × subtract × abs × weight)
// Cache key = (hash_a, hash_b), value = similarity score

pub fn sim_cache_new() {
  return { table: {}, size: 0 };
}

pub fn sim_cache_get(cache, ha, hb) {
  let key = combine_hash(ha, hb);  // order-independent: min,max
  return cache.table[key];         // null if miss
}

pub fn sim_cache_put(cache, ha, hb, score) {
  let key = combine_hash(ha, hb);
  cache.table[key] = score;
}

pub fn sim_cache_invalidate_node(cache, h) {
  // Remove all entries containing h
  // Called when node evolves (5D position changed)
}

fn combine_hash(ha, hb) {
  if ha < hb { return ha * 0x100000000 + hb; }
  return hb * 0x100000000 + ha;
}
```

### 5.2.4 — Dream score memoization (~50 LOC)

```
Dream cluster scoring hiện tại: recalculate toàn bộ mỗi Dream cycle.
Memoize: chỉ recalculate cho nodes thay đổi kể từ last Dream.

dream_cache.ol:

pub fn memo_score(cache, cluster_id, nodes) {
  if not changed_since(cache, cluster_id) {
    return cache.scores[cluster_id];
  }
  let score = calculate_score(nodes);
  cache.scores[cluster_id] = score;
  cache.versions[cluster_id] = current_version();
  return score;
}
```

---

## Rào cản

```
1. Cache invalidation
   → "There are only two hard things in CS: cache invalidation and naming things"
   → Giải pháp: conservative invalidation
     - Variable cache: invalidate on Store (cùng hash)
     - Registry cache: invalidate on insert/amend
     - Silk cache: invalidate on evolve
   → Worst case: miss → full lookup (correctness preserved)

2. Memory overhead
   → IC table: 4 KB (fixed)
   → Registry cache: 55 entries × ~100 bytes = 5.5 KB
   → Silk cache: bounded by max_entries (configurable)
   → Total: < 50 KB → acceptable
```

---

## Test Plan

```
Test 1: Variable IC — load same var 1000 times → verify hit rate > 95%
Test 2: Registry cache — get top-10 nodes repeatedly → hit rate > 90%
Test 3: Silk cache — similarity(a,b) twice → second call is cached
Test 4: Invalidation — store new value → next load gets fresh value
Test 5: Benchmark — measure latency reduction vs no-cache baseline
```

---

## Definition of Done

- [ ] Variable inline cache (ASM, per-Load-site)
- [ ] Registry LRU cache (Olang)
- [ ] Silk similarity cache (Olang)
- [ ] Dream score memoization (Olang)
- [ ] Cache invalidation correctness verified
- [ ] Benchmark showing measurable improvement

## Ước tính: 3-5 ngày
