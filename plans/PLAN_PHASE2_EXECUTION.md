# PLAN: Giai đoạn 2 — Stdlib + HomeOS Logic bằng Olang

**Ngày:** 2026-03-21
**Mục tiêu:** Mọi logic HomeOS = Olang bytecode chạy trên VM.
**Deliverable:** 17 file .ol hoàn chỉnh, tổng ~3,500 LOC, 0 test failure mới.

---

## Hiện trạng

```
VM:     40+ opcodes ✅ — đủ cho Phase 2
Stdlib: 8 file .ol đã viết ✅ (~600 LOC)
Logic:  59 file .ol tồn tại, phần lớn stubs (30%)
Tests:  1087 PASS / 135 FAIL — blocker chính

Blocker: 135 test failures trong Vec/Dict/String builtins
→ Phải fix TRƯỚC KHI viết logic .ol
```

---

## Thứ tự thực hiện (6 bước)

### Bước 0 — Fix VM Builtins (BLOCKER)

**Mục tiêu:** 135 failing tests → 0 failures liên quan builtins.

```
Cần fix trong crates/olang/src/exec/vm.rs:
  ❌ array_new_and_get, array_len, array_contains, array_reverse
  ❌ dict_new_and_get, dict_set, dict_has_key
  ❌ str_contains, str_index_of, str_replace, str_split
  ❌ file_read_write_roundtrip, list_files_in_directory
  ❌ builder_compile_write_roundtrip

Chia nhỏ:
  0a. Fix Array builtins (push, pop, len, get, set, contains, reverse)
  0b. Fix Dict builtins (new, get, set, has_key, keys, values)
  0c. Fix String builtins (contains, index_of, replace, split, substr)
  0d. Fix File I/O builtins (read, write, append, list_dir)
  0e. Fix Compiler roundtrip (builder test)

DoD: cargo test -p olang -- vm::tests → 0 failures
```

---

### Bước 1 — Phase 2.1: Stdlib hoàn thiện

**Status:** 8/8 files ✅ (~600 LOC). Cần VERIFY chạy được trên VM.

```
stdlib/
  result.ol    14 LOC  ✅ Option/Result patterns
  iter.ol      35 LOC  ✅ Iterator combinators
  sort.ol      40 LOC  ✅ Quicksort/mergesort
  format.ol    47 LOC  ✅ String formatting
  json.ol     200 LOC  ✅ Parse/emit JSON
  hash.ol      40 LOC  ✅ Hash functions
  mol.ol      108 LOC  ✅ Molecule helpers
  chain.ol     94 LOC  ✅ Chain helpers

DoD: Mỗi file import + call thành công trên VM.
     cargo test -p olang -- stdlib → ALL PASS.
```

---

### Bước 2 — Phase 2.2: Emotion Pipeline (~380 LOC)

**Status:** 3 files stubs (~180 LOC, 47%). Cần hoàn thiện logic.

```
stdlib/
  emotion.ol   43→150 LOC  Cần: blend_amplify() thay vì average
                            V/A/D/I blending via Silk walk (KHÔNG trung bình)
                            Amplify: cortisol + adrenaline = mạnh hơn

  curve.ol     73→120 LOC  Cần: f(x) = 0.6×f_conv + 0.4×f_dn
                            Tone detection (positive/negative/neutral)
                            Fibonacci threshold for shift detection

  intent.ol    59→110 LOC  Cần: Crisis/Learn/Command/Chat classifier
                            Crisis keywords → early return
                            Score-based classification

Logic cốt lõi:
  ✅ emotion_new(v, a, d, i) → dict
  ❌ blend_amplify(emotions[]) → composite (KHÔNG average!)
  ❌ walk_emotion(graph, words) → amplified composite
  ❌ curve_update(prev, current) → new_curve
  ❌ detect_tone(curve) → "positive"/"negative"/"neutral"
  ❌ classify_intent(text, emotion) → "crisis"/"learn"/"command"/"chat"

DoD: emotion pipeline end-to-end:
     text → emotion.blend → curve.update → intent.classify → result
```

---

### Bước 3 — Phase 2.3: Knowledge Layer (~650 LOC)

**Status:** 4 files (~700 LOC, 80%). Cần verify + complete walk logic.

```
stdlib/
  silk_ops.ol  166→180 LOC  Cần: walk() hoàn chỉnh (5D distance)
                             Hebbian update (fire_together → weight++)
                             Implicit Silk comparison

  dream.ol     181→200 LOC  Cần: STM clustering (Fibonacci threshold)
                             promote() → KnowTree
                             batch_score() → rank proposals

  instinct.ol  197→150 LOC  Cần: 7 bản năng running
                             honesty, contradiction, causality,
                             energy, memory, curiosity, survival

  learning.ol  160→120 LOC  Cần: Pipeline orchestration
                             T1→T2→T3→T4→T5→T6→T7 chaining
                             Error propagation (BlackCurtain)

Logic cốt lõi:
  ✅ silk_distance(a, b) → f64 (5D Euclidean)
  ⚠️ walk(graph, words) → weighted composite (cần complete)
  ⚠️ dream_cluster(stm_entries) → proposals (logic đơn giản)
  ⚠️ instinct_check(proposal) → approve/reject (7 bản năng)
  ❌ learning_pipeline(input, context) → response

DoD: learning_pipeline() chạy end-to-end trên VM.
     dream.cluster + instinct.check integration test.
```

---

### Bước 4 — Phase 2.4: Agent Behavior (~500 LOC)

**Status:** 5 files stubs (~198 LOC, 40%). Gate done, rest cần viết.

```
stdlib/
  gate.ol      51→80 LOC   ✅ SecurityGate: crisis + keyword check
                             Cần: BlackCurtain threshold (Fibonacci)

  response.ol  28→120 LOC  Cần: Template rendering engine
                             Multi-language (vi/en/ja)
                             Tone-adaptive response

  leo.ol       41→100 LOC  Cần: Self-programming agent
                             Instinct runner
                             Proposal evaluation

  chief.ol     36→100 LOC  Cần: Tier 1 protocol
                             Receive Worker reports
                             Aggregate → AAM decision

  worker.ol    42→100 LOC  Cần: Tier 2 protocol
                             Device read/write via HAL
                             Report chain (not raw data)

Logic cốt lõi:
  ✅ gate_check(input) → pass/block/crisis
  ❌ render_response(template, emotion, language) → text
  ❌ leo_evaluate(proposals, instincts) → approved
  ❌ chief_aggregate(worker_reports) → decision
  ❌ worker_execute(command) → chain_report

DoD: Full pipeline: input → gate → learning → response → output.
     Agent tier test: worker → chief → AAM chain.
```

---

### Bước 5 — Integration Test

```
E2E test: text input → full pipeline → response output

Test cases:
  1. "Xin chào" → greeting response (Chat intent)
  2. "Tôi buồn" → empathy response (negative emotion)
  3. "2 + 3"    → calculation (Command intent)
  4. "Tự tử"    → crisis response (Crisis intent, gate blocks)
  5. "Học cái gì mới" → learning trigger (Learn intent)

DoD: 5/5 E2E tests pass trên VM.
     make smoke-binary passes.
```

---

## Dependency Graph

```
Bước 0 (Fix Builtins) ←── BLOCKER
    ↓
Bước 1 (Stdlib verify)
    ↓
Bước 2 (Emotion) ──→ Bước 3 (Knowledge) ──→ Bước 4 (Agents)
                                                    ↓
                                              Bước 5 (E2E)
```

---

## Ước tính LOC

```
Bước 0: ~200 LOC (fix Rust VM builtins)
Bước 1:   ~0 LOC (verify existing)
Bước 2: ~200 LOC (complete emotion stubs)
Bước 3: ~150 LOC (complete knowledge gaps)
Bước 4: ~300 LOC (write agent logic)
Bước 5:  ~50 LOC (E2E test .ol files)
─────────────────
Tổng:   ~900 LOC mới + fix

Phase 2 sau khi xong: ~3,500 LOC Olang (17 core files)
```

---

## Rủi ro

| Rủi ro | Xác suất | Mitigation |
|--------|----------|------------|
| VM builtins phức tạp hơn dự kiến | Trung bình | Chia nhỏ: array → dict → string → file |
| Emotion logic không chạy đúng | Thấp | Test từng fn trước khi pipeline |
| Dream walk() quá chậm | Thấp | Benchmark, optimize sau Phase 5 |
| Agent ISL communication broken | Trung bình | Test local trước, ISL sau |

---

## Checklist trước khi bắt đầu mỗi bước

```
□ git fetch origin main && git merge origin/main
□ Đọc PLAN này + CLAUDE.md
□ cargo test --workspace → ghi nhận baseline failures
□ Molecule từ encode_codepoint() — KHÔNG viết tay
□ Emotion qua TOÀN BỘ pipeline — KHÔNG trung bình
□ SecurityGate LUÔN chạy trước
□ Append-only — không delete/overwrite
□ cargo test && cargo clippy trước khi push
```
