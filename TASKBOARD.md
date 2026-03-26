# TASKBOARD — Origin / Olang

> **Master Plan:** [`plans/MASTER_PLAN_HOMEOS_V1.md`](plans/MASTER_PLAN_HOMEOS_V1.md)
> **Lich su:** [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Trang thai: Olang 1.0 + VM Scope Fix (2026-03-25)

```
origin_new.olang = 1,021KB | 20/20 tests | 3 doi self-build
VM scope fix: eval vars preserved across boot→boot chains ✅
Silk: ALIVE (17 edges/5 turns). Mol compose: WORKING.
```

---

## NEXT: GD.1 — KnowTree (Xuong Song)

> Tree IS index. Walk tree. Khong scan array.

| # | Task | Effort | Status |
|---|------|--------|--------|
| N.1 | Lazy char node (hash table) | ~40 LOC | TODO |
| N.2 | Word node: chain(char nodes) | ~40 LOC | TODO |
| N.3 | Fact node: chain(word nodes) | ~30 LOC | TODO |
| N.4 | Tree search: walk path | ~40 LOC | TODO |
| N.5 | Replace __knowledge[] | ~30 LOC | TODO |
| N.6 | learn/respond via tree | ~20 LOC | TODO |

## LATER: GD.2-5

```
GD.2: Neuron model (STM→Silk→Dream→QR)     ~150 LOC
GD.3: Skills + Instincts (7 Skills QT4)     ~100 LOC
GD.4: ConversationCurve (f, f', f'')         ~80 LOC
GD.5: Agents + ISL (AAM→LeoAI→Workers)     ~200 LOC
```

## Done (v1.0 + v1.1 fixes)

```
✅ Olang 1.0 (lambda, HOF, pipe, sort, split, join, contains, UTF-8)
✅ VM scope fix (eval↔boot boundary)
✅ 7/7 instincts, 5/5 checkpoints, SC.4+5+6
✅ Greeting/goodbye router, gate, math strip
✅ Knowledge: CI search, gate zero-score, no dupes
✅ Dict pretty-print, persistent save/load
✅ All P0+P1 blockers resolved
✅ 117 DCs fixed (Kira #24)
✅ Tier 1-5 + PLAN_REWRITE ALL DONE
```

## Log

```
2026-03-23  SELF-HOSTING. 806KB.
2026-03-24  100% SELF-COMPILE. 964KB.
2026-03-25  OLANG 1.0 + HomeOS 1.0. ~50 commits. 1,021KB.
            VM scope fix (CRITICAL). KnowTree plan ready.
```
