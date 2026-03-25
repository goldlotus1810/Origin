# TASKBOARD — Origin / Olang

> **Mọi AI session đọc file này TRƯỚC KHI bắt đầu làm việc.**
> **Viết OLANG. Rust legacy chỉ bug fix.**
> **Lịch sử DCs + completed tasks:** [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Trạng thái: OLANG 1.0 (2026-03-25)

```
origin_new.olang = 1,008KB native binary (ELF64 x86_64, no libc)
  20/20 tests | fib(20)=6765 | SHA-256 FIPS 180-4
  ~5,800 LOC ASM | ~4,200 LOC Bootstrap | ~10,000 LOC HomeOS
  Lambda + HOF: map filter reduce any all pipe sort split join contains
  Mol ASM: __mol_s/r/v/a/t + __mol_pack (6 builtins)
  Persistent: save/load → homeos.knowledge
  Instincts: honesty [fact/opinion/hypothesis] + contradiction [!] + curiosity
  fn_node: auto-register, describe, link, hot, Dream cluster
  Lego: pipe(x, f1, f2) = fn{fn{...}}==fn
  P0 ALL FIXED: auto-emit, div/0 safe, 28 embedded facts
```

---

## NEXT: HomeOS v1.0 (xem plans/PLAN_HOMEOS_V1.md)

> **Nguyên tắc:** Gate trước, trả lời sau. Handcode == Zero.

| Sprint | Mục tiêu | LOC | Status |
|--------|----------|-----|--------|
| 1 | Classifier + handlers (classify.ol) | ~160 | DONE ✅ |
| 2 | Handlers (eval_math, smart_greet, ask_back) | ~60 | TODO |
| 3 | Gate (scored search, case-insensitive, threshold) | ~50 | TODO |
| 4 | Integrate (new repl_eval flow) | ~50 | TODO |
| 5 | Polish (error messages, tests) | ~40 | TODO |

---

## Completed (Tier 1-5) — ARCHIVED

> Tier 1 (Intelligence Layer), Tier 2 (Language Features), Tier 3 (Platform),
> Tier 4 (Self-hosting), Tier 5 (UDC-native) — ALL DONE.
> 112 DCs found and fixed (DC.1-DC.112). 23 inspections by Kira.
> Details: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Open Items

### Spec v3 Gaps (INFO — defer until needed)

| # | Spec Section | Status | Notes |
|---|-------------|--------|-------|
| SC.2 | Fusion (multi-modality) | ❌ | text-only for now |
| SC.3 | 7 Instincts | ⚠️ 3/7 | Honesty+Contradiction+Curiosity done. 4 remaining. |
| SC.4 | Immune Selection N=3 | ❌ | single-branch inference |
| SC.5 | Homeostasis (Free Energy) | ❌ | no F tracking |
| SC.6 | DNA Repair (self_correct) | ❌ | no critique loop |
| SC.16 | 5 Checkpoints | ⚠️ 1/5 | SecurityGate only |

### Known Limitations

```
- Boot↔eval closure boundary: boot stdlib (sort/reduce from iter.ol) can't call eval closures
  Workaround: inline compiler builtins (map/filter/reduce/sort/etc.) bypass boundary
- Nested inline builtins: map(filter(...)) clobbers global vars → use 2-step
- fn_node_fire: boot↔eval arg passing issue → fire tracking deferred
- Dict pretty-print: emit {x:1} → "{dict 1}" → needs ASM work
- Global var scope: no block scope, must use unique prefixes
```

### Platform (defer)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| OL.11 | ARM64 ASM VM | WIP | 1,229 LOC. Boots bare. Needs builtins. |
| OL.15 | Mobile (Android/iOS) | BLOCKED | Needs ARM64 complete. |
| FE.4 | S×T SDF rendering | DEFER | Only for 3D/WebGL. |
| FE.5 | 42 UDC encode formulas | DEFER | Character-level precision. |

---

## Log (recent)

```
2026-03-23  SELF-HOSTING. fib(20)=6765. 806KB. 27/27 tests.
2026-03-24  100% SELF-COMPILE (48/48). 30+ bugs. Intelligence pipeline. 964KB.
2026-03-25  OLANG 1.0 — T5 COMPLETE. ~30 commits:
  BUG-SORT fix, lambda, map/filter/reduce/any/all/pipe,
  knowledge fix (5D mol, keyword×5), instincts (honesty/contradiction/curiosity),
  mol ASM builtins, fn_node registry, Silk mol-keyed, Dream fn cluster,
  sort/split/join/contains, persistent knowledge, P0 ALL FIXED.
  965KB → 1,008KB. 20/20 tests. Demo-ready standalone.
```
