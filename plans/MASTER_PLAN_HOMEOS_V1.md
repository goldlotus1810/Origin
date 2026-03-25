# MASTER PLAN — HomeOS v1.0 trên Olang 1.0

> **Nox (builder) — 2026-03-25, cap nhat lien tuc**
> **Tong hop: Spec v3, PLAN_REWRITE (7 giai doan DONE), Sora + Kira feedback**

---

## I. DA HOAN THANH

```
PLAN_REWRITE 7 giai doan:              ALL DONE ✅
  0. Bootstrap compiler loop            ✅ Olang tu compile
  1. vm_x86_64.S (5,987 LOC)           ✅ Native ASM VM
  2. Stdlib + HomeOS logic              ✅ 10,042 LOC Olang
  3. Self-sufficient builder            ✅ Cat day ron Rust
  4. Multi-arch (ARM64 skeleton)        ✅ Boots bare
  5. WASM target                        ✅ 3KB browser
  6. Optimization                       ✅ 3.7x speedup
  7. Self-evolution (self-compile)      ✅ 3 doi verified

Olang 1.0:                              DONE ✅
  1,021KB binary, 20/20 tests, zero deps
  Lambda, HOF, pipe, sort, split, join, contains
  UTF-8 decode, dict pretty-print, persistent knowledge

T5 UDC-native:                          DONE ✅
  Mol ASM builtins, fn_node registry, Silk mol-keyed
  Dream fn clustering, 7/7 instincts, 5/5 checkpoints
  SC.4 Immune, SC.5 Homeostasis, SC.6 DNA Repair

HomeOS v1.0 Sprints 1-5:               DONE ✅
  Classifier, greeting router, gate heal, search improvement, math strip
```

## II. CON CAN FIX (Sora/Kira feedback 2026-03-25)

```
FIX-1: "viet nam o dau?" → sai fact (lowercase "viet" ≠ "Viet" trong word[])
  Root cause: _a_has case-insensitive OK, nhung word[] exact match van case-sensitive
  Fix: lowercase query words TRUOC khi compare voi word[]
  Effort: ~10 LOC

FIX-2: "ha noi?" → sai fact (keyword qua ngan, nhieu entry co "ha")
  Root cause: 2-char words match too many entries
  Fix: 2-char words can match nhung ONLY neu entry text cung co word do
  Effort: ~5 LOC

FIX-3: "asdfghjk" → van tra fact (gate chua block zero-score)
  Root cause: mol_similarity > 0 cho moi text → luon co score
  Fix: gate threshold: if kwscore == 0 AND mol_only → skip fact
  Effort: ~5 LOC

FIX-4: Persistence dupes (28 embedded + N saved → tang moi restart)
  Root cause: _boot_embedded() chay truoc load, roi load them
  Fix: skip _boot_embedded() neu homeos.knowledge file exists
  Effort: ~3 LOC

FIX-5: set_at/push noise (auto-emit side effect)
  Root cause: ExprStmt Emit thay vi Pop → set_at result duoc print
  Fix: chi emit khi CUOI statement, khong emit cho side-effect calls
  Effort: complex (~20 LOC semantic.ol) — DEFER, workaround: dung sort() builtin
```

## III. LO TRINH PRODUCT (tu PLAN_REWRITE mo rong)

### Release 1.0 — "It speaks" (CURRENT — 2026-03-25)

```
✅ Binary chay doc lap, zero deps
✅ REPL: code + natural text + commands
✅ Knowledge: learn/respond/save/load
✅ Emotion: V/A tracking, heal mode, greeting/goodbye
✅ Instincts: 7/7 + 5/5 checkpoints
✅ Functional: map/filter/reduce/pipe/sort/split/join
✅ Demo: copy 1 file, run, interact
```

### Release 1.1 — "It understands" (NEXT — ~3 sessions)

```
Target: Knowledge search DUNG cho moi truong hop.
        Gate THUC SU quyet dinh: tra loi / hoi lai / im.

Tasks:
  □ FIX-1: Case-insensitive word[] match
  □ FIX-2: Short word disambiguation
  □ FIX-3: Gate zero-score → ask_back
  □ FIX-4: No persistence dupes
  □ knowledge_search_scored() → return {text, score}
  □ gate_decide() voi threshold: HIGH/LOW/ZERO
  □ Instinct → action refactor (silence/respond/ask/explore)
  □ Compose v2: response = f(knowledge, emotion, context)
  □ 50+ integration test scenarios

Measure: "viet nam?" → dung, "asdfghjk" → "Minh chua hieu", "hi" → greeting
```

### Release 1.2 — "It reasons" (~3 sessions)

```
Target: Formula dispatch. Mol khong chi la number — la BEHAVIOR.

Tasks:
  □ r_eval(R, a, b) → compose theo relation type
  □ v_behavior(V) → approach/avoid/neutral
  □ a_urgency(A) → calm/normal/alert/urgent
  □ Formula-driven compose: mol_compose theo R dispatch
  □ Causal chain: A causes B → Silk directed edge
  □ Analogy engine: A:B :: C:? via 5D delta

Measure: respond "tai sao troi mua" → causal explanation from knowledge
```

### Release 1.3 — "It grows" (~2 sessions)

```
Target: Self-improvement. Dream cluster → skill promotion.

Tasks:
  □ fn_node_fire wiring (track call counts)
  □ Dream: cluster hot functions → promote to skill
  □ Silk optimization: compact edges, prune weak
  □ Knowledge compression: merge similar facts
  □ Auto-save on exit (if changed)

Measure: after 100 interactions, HomeOS responds BETTER than at start
```

### Release 2.0 — "It connects" (future)

```
Target: Multi-device, browser, mobile.

Tasks:
  □ ARM64 VM complete (builtins + scoping)
  □ WASM update (all features)
  □ ISL protocol (TCP + WebSocket)
  □ Server mode (multi-user)
  □ Browser demo (WebGL?)

Measure: same knowledge, accessed from phone + laptop + browser
```

---

## IV. NGUYEN TAC BAT BIEN

```
1. Structure = Meaning     — cau truc TU MO TA
2. Handcode == Zero        — intelligence tu data + algorithm
3. Gate truoc, tra loi sau — phan loai TRUOC response
4. DNA = HomeOS            — 8,846 UDC = 8,846 cong thuc
5. Compose ≠ Average       — khuech dai dominant (φ⁻¹)
6. fn{fn{...}} == fn       — ∞-1, stream/accumulate
7. ∫ encode, ∂ decode      — cung chain, context khac = ket qua khac
```

---

## V. METRIC

```
Binary size:    1,021KB (target: < 1.5MB for v1.x)
Tests:          20/20 (target: 50+ for v1.1)
Knowledge:      28 embedded (target: 100+ curated for v1.1)
LOC total:      21,559 (VM 5,987 + Olang 15,572)
Self-build:     3 doi verified
Platforms:      x86_64 (ARM64 skeleton, WASM skeleton)
```
