# TASKBOARD — Origin / Olang

> **Doc TRUOC KHI lam viec. Viet OLANG.**
> **Master Plan:** [`plans/MASTER_PLAN_HOMEOS_V1.md`](plans/MASTER_PLAN_HOMEOS_V1.md)
> **Lich su:** [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Trang thai: OLANG 1.0 + HomeOS 1.0 (2026-03-25)

```
origin_new.olang = 1,021KB (1,021,393 bytes, ELF64 x86_64, no libc)
  20/20 tests | fib(20)=6765 | 3 doi self-build verified
  5,987 LOC ASM | 3,748 LOC Bootstrap | 451 LOC REPL | 10,042 LOC HomeOS
  Total: 21,559 LOC (VM + Olang)
```

---

## RELEASE ROADMAP

### v1.0 "It speaks" — DONE ✅ (2026-03-25)

Binary chay doc lap. REPL. Knowledge. Emotion. 7 instincts. Persistent.

### v1.1 "It understands" — IN PROGRESS

> Knowledge search DUNG. Gate QUYET DINH: tra loi / hoi lai / im.

| # | Task | Effort | Status |
|---|------|--------|--------|
| FIX-1 | Case-insensitive word[] match | ~10 LOC | TODO |
| FIX-2 | Short word disambiguation | ~5 LOC | TODO |
| FIX-3 | Gate zero-score → ask_back | ~5 LOC | TODO |
| FIX-4 | No persistence dupes on restart | ~3 LOC | TODO |
| KS.1 | knowledge_search_scored → {text, score} | ~20 LOC | TODO |
| GT.1 | gate_decide threshold (HIGH/LOW/ZERO) | ~15 LOC | TODO |
| IN.1 | Instinct → action refactor | ~50 LOC | TODO |
| CP.1 | Compose v2: f(knowledge, emotion, context) | ~40 LOC | TODO |
| TS.1 | 50+ integration test scenarios | ~50 LOC | TODO |

### v1.2 "It reasons" — PLANNED

> Formula dispatch. Mol = behavior, khong chi number.

| # | Task | Effort | Status |
|---|------|--------|--------|
| FE.1 | r_eval(R, a, b) → compose by relation | ~60 LOC | TODO |
| FE.2 | v_behavior(V) → approach/avoid/neutral | ~20 LOC | TODO |
| FE.3 | a_urgency(A) → calm/alert/urgent | ~20 LOC | TODO |
| FE.4 | Causal chain in Silk | ~30 LOC | TODO |
| FE.5 | Analogy engine 5D delta | ~40 LOC | TODO |

### v1.3 "It grows" — PLANNED

> Self-improvement. Dream → skill. Auto-optimize.

| # | Task | Effort | Status |
|---|------|--------|--------|
| DR.1 | fn_node_fire wiring | ~15 LOC | TODO |
| DR.2 | Dream cluster → skill promote | ~30 LOC | TODO |
| DR.3 | Knowledge compression | ~30 LOC | TODO |
| DR.4 | Auto-save on exit | ~10 LOC | TODO |

### v2.0 "It connects" — FUTURE

> Multi-device. Browser. Mobile.

| # | Task | Status |
|---|------|--------|
| ARM64 VM complete | WIP (1,229 LOC skeleton) |
| WASM update | DONE (3KB, basic) |
| ISL protocol | TODO |
| Server mode | TODO |

---

## Spec v3 Compliance

| # | Section | Status |
|---|---------|--------|
| SC.1 | SecurityGate 3-layer | ✅ 12 patterns |
| SC.3 | 7 Instincts | ✅ 7/7 active |
| SC.4 | Immune Selection N=3 | ✅ 3 candidates |
| SC.5 | Homeostasis (Free Energy) | ✅ FE tracking |
| SC.6 | DNA Repair | ✅ Self-correction |
| SC.16 | 5 Checkpoints | ✅ 5/5 |
| SC.2 | Fusion (multi-modal) | ❌ text-only |

---

## Known Limitations

```
- Boot↔eval boundary: boot functions can't call eval closures → inline builtins
- Nested inline builtins: map(filter(...)) clobbers vars → 2-step
- set_at/push auto-emit noise → use sort() builtin
- Global var scope: no block scope → unique prefixes
- fn_node_fire: not wired (boot↔eval arg issue)
```

---

## Log

```
2026-03-23  SELF-HOSTING. 806KB.
2026-03-24  100% SELF-COMPILE. Intelligence pipeline. 964KB.
2026-03-25  OLANG 1.0 + HomeOS 1.0. ~50 commits. 1,021KB.
            T5 complete. All P0+P1 fixed. 7/7 instincts. 5/5 checkpoints.
            UTF-8 pipeline. Dict pretty-print. Persistent knowledge.
            Classifier → Router → Gate architecture.
            MASTER PLAN written. Release roadmap: v1.0→v1.1→v1.2→v1.3→v2.0.
```
