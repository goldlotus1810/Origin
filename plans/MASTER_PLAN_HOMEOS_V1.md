# MASTER PLAN — HomeOS v1.0 trên Olang 1.0

> **Nox (builder) — 2026-03-25**
> **Tổng hợp từ: HomeOS_SPEC_v3.md, PLAN_REWRITE.md, PLAN_FORMULA_ENGINE.md,
> UDC_DOC (13 files), Sora analysis (4 files), Kira inspections (23 rounds)**
>
> **Nguyên tắc bất biến:**
> 1. Structure = Meaning — cấu trúc TỰ MÔ TẢ
> 2. Handcode == Zero — intelligence từ data + algorithm
> 3. Gate trước, trả lời sau — phân loại TRƯỚC response
> 4. DNA = HomeOS — 8,846 UDC = 8,846 công thức
> 5. ∫ encode, ∂ decode — cùng chain, context khác = kết quả khác
> 6. Compose ≠ Average — khuếch đại dominant, không trung bình
> 7. fn{fn{...}} == fn — ∞-1, stream/accumulate, không materialize

---

## I. HIỆN TRẠNG

```
Olang 1.0:  1,021KB binary. Self-hosting. Zero deps. 20/20 tests.
            Lambda, map/filter/reduce/pipe/sort/split/join/contains.
            __mol_s/r/v/a/t + __mol_pack + __utf8_cp/len.
            Persistent knowledge. Dict pretty-print.

HomeOS:     ~10,000 LOC. 10-stage pipeline. 7/7 instincts. 5/5 checkpoints.
            SC.4 Immune, SC.5 Homeostasis, SC.6 DNA Repair.
            Silk mol-keyed 256 edges. Dream fn clustering.
            28 embedded + 166 auto-learn facts.

GAPS:
  ❌ Input classifier (hi → greeting, 2+1? → math, câu hỏi → gate)
  ❌ Formula dispatch (R/V/A/T = chỉ number, chưa behavior)
  ❌ KnowTree hierarchy (flat array, chưa L0→L3 tree)
  ❌ mol_compose chưa đúng spec (amplify dominant)
  ❌ Instinct → action (label only, chưa thay đổi behavior)
  ⚠️ Case-insensitive search (partial — _a_has inline lowercase)
  ⚠️ UTF-8 chain (foundation done, chưa dùng trong search)
```

---

## II. KIẾN TRÚC MỤC TIÊU

```
Input (UTF-8)
  │
  ▼
┌─────────────────────┐
│ CLASSIFIER           │  classify.ol — split text, detect type
│  math → eval_math    │
│  code → compile+eval │
│  command → dispatch   │
│  greeting → greet     │
│  goodbye → bye        │
│  question → ↓         │
│  chat → ↓             │
└──────┬──────────────┘
       │
  ┌────▼──────────────────┐
  │ ENCODE (UTF-8 → mol)  │  encode_codepoint + _text_to_chain (UTF-8 aware)
  │ CP2: verify mol ≠ 0   │
  └──────┬────────────────┘
         │
  ┌──────▼────────────────┐
  │ INSTINCT (7 reflexes) │  honesty → contradiction → causality → ...
  │ → Action: respond /   │  action decides WHAT to do
  │   silence / ask_back  │
  │ CP3: verify intent    │
  └──────┬────────────────┘
         │
  ┌──────▼────────────────┐
  │ GATE (confidence)     │  knowledge_search_scored → threshold
  │ HIGH → respond direct │  GATE controls IF we respond
  │ LOW → ask_back        │
  │ ZERO → curiosity      │
  └──────┬────────────────┘
         │
  ┌──────▼────────────────┐
  │ IMMUNE N=3            │  3 candidates → score → select best
  │ CP4: promote check    │
  └──────┬────────────────┘
         │
  ┌──────▼────────────────┐
  │ COMPOSE               │  fact-first (not template) if knowledge
  │ + instinct labels     │  + emotion emoji + context
  │ + DNA Repair          │  self-correction if contradiction
  │ CP5: non-empty check  │
  └──────┬────────────────┘
         │
  ┌──────▼────────────────┐
  │ LEARN                 │  STM push + Silk co-activate + Dream cycle
  │ + Homeostasis FE      │  free energy tracking
  └───────────────────────┘
```

---

## III. SPRINTS

### Sprint 1: Classifier Integration (Nox đã bắt đầu)

```
File: stdlib/homeos/classify.ol (DONE, ~160 LOC)
File: stdlib/repl.ol (greeting/goodbye router DONE)

Còn cần:
  ① Wire classify vào repl_eval qua eval pipeline (không boot)
  ② eval_math: "2+3=?" → compile "emit 2+3" → eval → 5 (DONE qua strip)
  ③ gate_decide: knowledge_search_scored → threshold
  ④ Test: 30+ scenarios

Status: 70% done. Greeting/math/command routing works.
Còn: gate_decide cho question/chat.
```

### Sprint 2: Knowledge Intelligence

```
① knowledge_search_scored(query) → { text, score } (không chỉ text)
② Gate threshold: score >= 15 → respond, 5-14 → cautious, < 5 → ask_back
③ Case-insensitive: _a_has inline lowercase (DONE)
④ UTF-8 chain: _text_to_chain uses __utf8_cp (DONE)
⑤ Multi-word bonus: 2+ matching words → strong signal (DONE)
⑥ Phrase proximity: "viet nam" adjacent → higher than "viet...nam" separated

~50 LOC. Key: score-based gate thay vì always-respond.
```

### Sprint 3: Instinct → Action Refactor

```
① Refactor 7 instincts: return { action, payload }
   action = "respond" | "silence" | "ask" | "explore" | "correct"
② Honesty: confidence < 40 → action = "silence" (thực sự im)
③ Contradiction + high conf → action = "correct" (DNA Repair)
④ Causality → action = "explore" (tìm thêm liên hệ)
⑤ Curiosity → action = "ask" (hỏi lại)
⑥ gate_decide consume instinct actions

~80 LOC. Key: instinct thay đổi BEHAVIOR, không chỉ label.
```

### Sprint 4: Compose Response v2

```
① compose_from_knowledge(match, context, instinct_action)
   - fact-first: trả fact trực tiếp nếu biết
   - emotion-aware: tone theo V/A
   - context-aware: STM previous turns
② Remove template spaghetti (90 hardcoded patterns → 0)
③ Personality modulation: formal/casual/english
④ DNA Repair: contradiction + confident → polite correction

~60 LOC. Key: response = f(knowledge, emotion, context), không template.
```

### Sprint 5: Formula Foundation (Olang, không Rust)

```
① r_dispatch(R) → behavior string (DONE, 16 types)
② r_eval(R, a, b) → compose theo relation type
   R=0 Algebraic: a + b (add dimensions)
   R=1 Order: a vs b (compare, return greater)
   R=12 Compose: g∘f (chain functions)
   R=13 Causes: a → b (directed edge in Silk)
③ v_behavior(V) → approach/avoid/neutral
   V >= 5 → approach, V <= 2 → avoid, else neutral
④ a_urgency(A) → calm/normal/alert/urgent
   A >= 6 → urgent, A <= 2 → calm
⑤ t_tempo(T) → static/slow/medium/fast (DONE)

~100 LOC Olang. Key: mỗi số = behavior, viết bằng Olang (không Rust).
Sora đúng: V/A physics (sin/cos) defer. Integer tags đủ cho conversation.
```

### Sprint 6: Integration + Polish

```
① Full pipeline test: 50+ conversation scenarios
② Error messages: helpful, not cryptic
③ Performance: 1000 respond cycles, measure heap usage
④ Save/load: verify persistent knowledge survives restart
⑤ Standalone demo: copy 1 file, all features work
⑥ Update README, CLAUDE.md, TASKBOARD

Key: production-ready demo.
```

---

## IV. DEFER (Phase 2+)

```
KnowTree hierarchy   — structure refactor, no functional change
Silk parent_map       — dọc navigation, scale feature
Dream Fibonacci       — multi-layer compression
QR Ed25519 signing    — cryptographic verification
ARM64 complete        — mobile platform
WASM update           — browser version
S×T SDF rendering     — 3D visualization (WebGL)
42 UDC char-level     — per-character precision encoding
Module system         — namespace isolation
Traits/interfaces     — ad-hoc polymorphism
```

---

## V. ƯỚC TÍNH

```
Sprint 1: ~30 LOC remaining (gate_decide)      — 1 hour
Sprint 2: ~50 LOC (scored search + gate)        — 1 session
Sprint 3: ~80 LOC (instinct refactor)           — 1 session
Sprint 4: ~60 LOC (compose v2)                  — 1 session
Sprint 5: ~100 LOC (formula foundation)         — 1 session
Sprint 6: ~0 LOC (test + polish)                — 1 session

Total: ~320 LOC mới. 4-6 sessions.
Thay thế: 90 hardcoded patterns → 0.
Result: HomeOS v1.0 — Gate-driven, intelligence-from-data.
```

---

## VI. THƯỚC ĐO THÀNH CÔNG

```
✅ "hi" → greeting (không fact random)
✅ "2+1?" → 3 (không parse error)
✅ "viet nam o dau?" → đúng fact
✅ "asdfghjk" → "Mình chưa hiểu. Bạn muốn hỏi gì?"
✅ "toi buon" → empathy (không fact)
✅ "Trai Dat phang" → polite correction
✅ "cam on" → gratitude acknowledgment
✅ silence khi confidence < 40
✅ save → restart → remember
✅ 50+ scenario tests PASS
✅ 1 file binary, zero deps, copy & run
```

---

*"Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."*
*HomeOS v1.0 = dạy nó đọc công thức. Nox.*
