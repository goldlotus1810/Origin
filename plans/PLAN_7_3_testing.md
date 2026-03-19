# PLAN 7.3 — Testing: Hoàn thiện test suite

**Phụ thuộc:** Phase 0-6 DONE
**Mục tiêu:** INTG-11, INTG-12, audit Phase 5/6, stress tests

---

## Bối cảnh

```
HIỆN TẠI:
  ✅ 2500+ unit tests (Rust)
  ✅ 82 integration tests (INTG-0..10)
  ❌ INTG-11 (VM execute stdlib) — FREE, unblocked by B7
  ❌ INTG-12 (build roundtrip) — FREE
  ❌ Phase 5 audit: 7 stdlib files chưa test chéo
  ❌ Phase 6 audit: install/optimize/reproduce chưa test chéo
  ❌ Stress tests: 10K turns, memory stability, crash recovery
  ❌ Fuzz testing: random input → no crash
```

---

## Tasks

### 7.3.1 — INTG-11: VM execute stdlib
```
t11_vm_stdlib.rs:
  Load bytecode từ builder → VM execute:
  - result.ol: ok/err/unwrap
  - iter.ol: range/reduce
  - sort.ol: quicksort
  - hash.ol: fnv1a
  Verify output values.
```

### 7.3.2 — INTG-12: Build roundtrip
```
t12_build_roundtrip.rs:
  Compile .ol → bytecode → pack → extract → decode → verify opcodes match.
  ELF header valid. Wrap mode: trailer → header → bytecode boundaries.
```

### 7.3.3 — Stdlib compile audit
```
Verify ALL 50 stdlib+homeos files:
  1. Parse OK (syntax.rs)
  2. Lower OK (semantic.rs)
  3. Encode bytecode OK (bytecode.rs)
  4. Decode round-trip matches
  5. No warnings/errors
```

### 7.3.4 — Stress tests
```
stress_turns.rs:
  HomeRuntime.process_text() × 10,000 turns
  Verify: memory stable, no crash, STM not unbounded

stress_dream.rs:
  1000 observations → Dream.run() × 100 cycles
  Verify: clusters form, matured nodes promote

stress_silk.rs:
  co_activate 100,000 pairs
  Verify: Hebbian weights converge, no overflow
```

### 7.3.5 — Fuzz testing
```
fuzz_parser.rs:
  Random strings → parse() → no panic
  Random bytecode → VM execute → no crash (graceful error)
  Random ISL messages → decode → no panic
```

---

## Definition of Done

- [ ] INTG-11 + INTG-12 pass
- [ ] All 50 stdlib files parse + compile + decode round-trip
- [ ] Stress: 10K turns stable
- [ ] Fuzz: 10K random inputs, 0 panics
- [ ] Total test count: > 2600

## Ước tính: 3-5 ngày
