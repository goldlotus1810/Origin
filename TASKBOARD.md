# TASKBOARD — Bảng phân việc cho AI sessions

> **Mọi AI session đọc file này TRƯỚC KHI bắt đầu làm việc.**
> File này là nguồn sự thật duy nhất (single source of truth) về ai đang làm gì.
> **Chi tiết đầy đủ (debug/kiểm tra lỗi):** [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Quy trình phối hợp

```
KHI BẮT ĐẦU SESSION MỚI:
  1. git pull origin main          ← lấy TASKBOARD mới nhất
  2. Đọc TASKBOARD.md              ← xem task nào FREE, task nào CLAIMED
  3. Chọn task FREE                ← ưu tiên theo dependency graph
  4. Cập nhật TASKBOARD.md         ← đổi status → CLAIMED, ghi branch + ngày
  5. git commit + push             ← commit NGAY để session khác thấy
  6. Bắt đầu code

KHI HOÀN THÀNH:
  1. Tải cập nhật main.            ← cập nhật thay đổi mới nhất.
  2. Cập nhật TASKBOARD.md         ← đổi status → DONE, ghi notes
  2. git commit + push

KHI BỊ BLOCKED:
  1. Cập nhật TASKBOARD.md         ← đổi status → BLOCKED, ghi lý do
  2. git commit + push
  3. Chuyển sang task khác (nếu có)

⚠️ KHÔNG BAO GIỜ:
  ❌ Bắt đầu task đã CLAIMED bởi session khác
  ❌ Đổi status task của session khác
  ❌ Xóa dòng — chỉ thêm hoặc cập nhật status của mình
```

---

## ALL DONE ✅

Phase 0-11 | Task 12 | Phase 14.1-14.3 | Phase 15 (6/6) | Phase 16 (4/4) | V2 Migration T1-T14, T16 | INTG
→ Chi tiết: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Recently DONE (Phase 14)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 14.2 | Alias table tách riêng (T15) | DONE ✅ | 33,054 entries, 6B/entry, ~198KB. |
| 14.3 | Silk vertical parent_map persistence | DONE ✅ | RT_PARENT 0x0C (25B/record). |

---

## FREE Tasks — Ưu tiên cao → thấp

### Tier 1 — Unblocked, làm ngay được

| ID | Task | Plan | Effort | Depends | Status | Notes |
|----|------|------|--------|---------|--------|-------|
| P2.0 | Fix VM builtin test failures | PLAN_PHASE2 | ~200 LOC | — | DONE ✅ | VM 90/90 pass. Heap refs + string encoding bypass quantization. PR #228 #229. |
| P2.0b | Fix 37 remaining olang test failures | — | ~300 LOC | P2.0 | CLAIMED | Closure dispatch, bytes builtins, self-compile, iter. Session lupin-pc 2026-03-22. |
| 8.1 | Parser: hex literals (0xFF) | PLAN_8 | ~80 LOC | — | DONE ✅ | Đã implement (session 2pN6F). |
| 8.2 | Parser: == trong match/struct | PLAN_8 | ~200 LOC | — | DONE ✅ | Đã implement (session 2pN6F). |
| 8.3 | Parser: keywords as ident + struct colon | PLAN_8 | ~100 LOC | — | DONE ✅ | Đã implement (session 2pN6F). |
| 12.1 | Wire walk_emotion() vào response | PLAN_12 | ~100 LOC | — | DONE ✅ | compose_response() wired (session 2pN6F). |
| 12.2 | Context recall trong response | PLAN_12 | ~80 LOC | 12.1 | DONE ✅ | ResponseContext + context-aware intent (session 2pN6F). |
| 12.3 | Intent estimation dùng context | PLAN_12 | ~120 LOC | 12.2 | DONE ✅ | Causality→skip AddClarify, repetition→EmpathizeFirst. |
| 12.4 | Response composer thay template | PLAN_12 | ~200 LOC | 12.3 | DONE ✅ | compose_response() thay render(). |
| 12.5 | Language detection + instinct wire | PLAN_12 | ~60 LOC | 12.4 | DONE ✅ | detect_language tiếng Việt không dấu. |
| 11.3 | Server --eval mode | PLAN_11 | ~80 LOC | — | FREE | stdin → process → output → exit. |
| 11.2 | Rust E2E test suite | PLAN_11 | ~300 LOC | 11.3 | FREE | t16_e2e_demo.rs. |
| 11.5 | Makefile targets (demo/verify) | PLAN_11 | ~50 LOC | 11.2 | FREE | make demo, make verify. |

### Tier 1 — Chi tiết kỹ thuật (để CLI tự thực hiện)

#### P2.0 — Fix 135 VM builtin test failures
```
File:    crates/olang/src/exec/vm.rs
Chạy:    cargo test -p olang -- vm::tests 2>&1 | grep FAILED
Lỗi:     array_new_and_get, array_len, array_contains, array_reverse,
         dict_new_and_get, dict_set, dict_has_key,
         str_contains, str_index_of, str_replace, str_split,
         file_read_write_roundtrip, list_files_in_directory,
         builder_compile_write_roundtrip
Nguyên nhân: VM builtins (push/pop/len/get/set/contains/reverse/keys/values
         index_of/replace/split/substr) chưa implement hoặc bị lỗi.
Cách fix: Tìm match arm cho từng builtin fn name trong vm.rs,
         implement logic đúng cho Array/Dict/String operations.
DoD:     cargo test -p olang -- vm::tests → 0 FAILED (hoặc giảm tối đa)
```

#### 8.1 — Parser: hex literals (0xFF)
```
File:    crates/olang/src/exec/compiler.rs (hoặc lexer module)
Vấn đề:  "0xFF" tokenize thành Int(0) + Ident("xFF") thay vì Int(255)
Cách fix: Trong lex_number(), sau khi gặp '0', check next char:
         'x'|'X' → parse hex digits (0-9, a-f, A-F) → u64
         'b'|'B' → parse binary digits
         'o'|'O' → parse octal digits
Test:    cargo test -p olang -- hex
         Tạo test: assert lex("0xFF") == [Int(255)]
         Tạo test: assert lex("0b1010") == [Int(10)]
DoD:     13 .ol files that failed on hex → now parse OK
```

#### 8.2 — Parser: == trong match/struct
```
File:    crates/olang/src/exec/compiler.rs (parser module)
Vấn đề:  9 .ol files fail khi dùng == trong:
         - match patterns: case x == 0 =>
         - field comparisons: if a.type == "crisis"
         - struct equality: if result == expected
Cách fix: Parser cần handle == as BinOp(Eq) trong đúng context.
         Check parse_expr() và parse_match_arm().
Test:    cargo test -p olang -- parse
DoD:     9 .ol files that failed on == → now parse OK
```

#### 8.3 — Parser: keywords as ident + struct colon
```
File:    crates/olang/src/exec/compiler.rs
Vấn đề:  (1) "learn" in intent.ol is both keyword and identifier
         (2) struct literal { from: x, to: y } — colon syntax
Cách fix: (1) Contextual keywords — only treat as keyword in statement position
         (2) Parse { ident: expr, ... } as struct/dict literal
Test:    Parse intent.ol + silk_ops.ol thành công
DoD:     Tổng 30/54 → 54/54 .ol files parse OK (hoặc gần đó)
```

#### 12.1 — Wire walk_emotion() vào response
```
File:    crates/context/src/emotion.rs (hoặc tương đương)
         crates/runtime/src/core/origin.rs (pipeline)
Vấn đề:  walk_emotion() hiện return None → response không có emotion context
Cách fix: Implement walk_emotion(graph, words):
         1. Cho mỗi word, tìm node trong SilkGraph
         2. Walk Silk edges, collect emotion tags
         3. Amplify (KHÔNG average!) — cortisol+adrenaline = mạnh hơn
         4. Return composite EmotionState
         Wire vào T2 (sentence_affect) trong pipeline
DoD:     walk_emotion("buồn vì mất việc") → V < -0.7 (amplified)
```

#### 12.2 — Context recall trong response
```
File:    crates/context/src/ (new struct hoặc existing)
Vấn đề:  STM recall tính nhưng KHÔNG dùng trong response
Cách fix: Tạo ResponseContext { topics, repetition_count,
         related_concepts, causality, contradiction, novelty }
         Populate từ STM entries trước khi render response
DoD:     ResponseContext available trong response pipeline
```

#### 12.3 — Intent estimation dùng context
```
File:    crates/context/src/intent.rs
Vấn đề:  90% input → AddClarify (keyword-only, quá thô)
Cách fix: Dùng ResponseContext:
         - lặp topic 3+ lần → Heal intent
         - đã nêu nguyên nhân → không hỏi "tìm hiểu gì"
         - emotion V < -0.8 → prioritize Comfort over Clarify
DoD:     "Tôi buồn vì mất việc" (lần 3) → Heal, không AddClarify
```

#### 12.4 — Response composer thay template
```
File:    crates/agents/src/encoder/ hoặc response module
Vấn đề:  Template cứng, chỉ nhìn valence number
Cách fix: Tạo compose_response(emotion, context, intent, language):
         Part 1: Acknowledgment (dựa trên emotion)
         Part 2: Context-specific (dựa trên recall + topic)
         Part 3: Follow-up (dựa trên intent)
         Part 4: Topic reflection (dựa trên instincts)
DoD:     Response thay đổi theo context, không lặp template
```

#### 12.5 — Language detection + instinct wire
```
File:    crates/agents/src/encoder/ + crates/context/
Vấn đề:  "xin chào" → tiếng Anh response (sai)
         7 instincts chạy nhưng không đến response
Cách fix: (1) Detect: chứa dấu/từ Việt → vi, else en
         (2) Wire: honesty/causality/contradiction flags → ResponseContext
DoD:     "xin chào" → tiếng Việt response
```

#### 11.3 — Server --eval mode
```
File:    tools/server/src/main.rs
Vấn đề:  Server chỉ có REPL interactive mode
Cách fix: Thêm --eval flag: đọc stdin, process, output, exit
         cargo run -p server -- --eval "2 + 3" → "5"
         cargo run -p server -- --eval < input.txt → output
DoD:     cargo run -p server -- --eval "xin chào" → response
```

#### 11.2 — Rust E2E test suite
```
File:    tools/intg/src/ (hoặc tests/)
Cách:    Tạo t16_e2e_demo.rs — 10 test cases gọi server --eval:
         1. Arithmetic: "2 + 3" → "5"
         2. Variable: "let x = 42; x" → "42"
         3. Function: "fn f(x) { x * 2 } f(5)" → "10"
         4. String: "\"hello\" + \" world\"" → "hello world"
         5. Crisis: "tự tử" → crisis response (gate blocks)
         6-10. Thêm theo PLAN_11
DoD:     cargo test -p intg -- e2e → 10/10 PASS
```

#### 11.5 — Makefile targets
```
File:    Makefile
Cách:    Thêm targets:
         make demo    → chạy 10 demo scenarios, in kết quả
         make verify  → cargo test + clippy + demo + smoke-binary
DoD:     make verify → ALL PASS
```

---

### Tier 2 — Cần Tier 1 xong trước

| ID | Task | Plan | Effort | Depends | Status | Notes |
|----|------|------|--------|---------|--------|-------|
| P2.2 | Emotion pipeline (.ol) | PLAN_PHASE2 | ~200 LOC | P2.0 | BLOCKED | emotion/curve/intent .ol hoàn thiện. |
| P2.3 | Knowledge layer (.ol) | PLAN_PHASE2 | ~150 LOC | P2.0 | BLOCKED | silk_ops/dream/instinct/learning .ol. |
| P2.4 | Agent behavior (.ol) | PLAN_PHASE2 | ~300 LOC | P2.0 | BLOCKED | response/leo/chief/worker .ol. |
| P2.5 | E2E integration test | PLAN_PHASE2 | ~50 LOC | P2.2-4 | BLOCKED | 5 test cases end-to-end. |
| 9 | Native REPL | PLAN_9 | ~600 LOC | 8.1-8.3 | BLOCKED | ./origin → REPL. Cần parser fix trước. |
| 10 | Browser E2E | PLAN_10 | ~500 LOC | 9 | BLOCKED | origin.html, WASM compile+execute. |
| 11.1 | Demo script (10 scenarios) | PLAN_11 | ~300 LOC | 8, 9 | BLOCKED | Cần parser + REPL. |
| 11.4 | Native binary --eval | PLAN_11 | ~50 LOC | 9 | BLOCKED | Cần native REPL. |

### Tier 3 — Lớn, cần kế hoạch riêng

| ID | Task | Plan | Effort | Status | Notes |
|----|------|------|--------|--------|-------|
| PW | P_weight migration 5B→2B | PLAN_PWEIGHT | LỚN | DONE ✅ | T1-T12 V2 Migration hoàn thành. |
| V2 | V2 Migration BIG BANG | PLAN_V2 | RẤT LỚN | DONE ✅ | T1-T16 ALL DONE. |
| UDC | UDC Rebuild (59 blocks) | PLAN_UDC | Nhiều sessions | DONE ✅ | 8,846 entries, build pipeline v3.1, KT31 format. |
| TLC | Test Logic Check (6 patterns) | PLAN_TEST_LOGIC | Trung bình | FREE | 6 test files cần viết theo CHECK_TO_PASS_LOGIC_HANDBOOK.md. |
| AUTH | First-run Auth setup | PLAN_AUTH_first_run | ~200 LOC | FREE | Terms screen + Master Key (Ed25519) + Biometric option. Độc lập. |
| **FE** | **Formula Engine — Giá trị = Công thức** | **PLAN_FORMULA_ENGINE** | **~1250 LOC** | **FREE** | **CRITICAL: 3/5 chiều (R,V,A) không dùng công thức. T chỉ 2 bit tĩnh. Cần: formula dispatch + T spline accumulation.** |
| FE.1 | R dispatch (16 relation → operations) | PLAN_FORMULA_ENGINE | ~200 LOC | FREE | Category theory, algebraic structures → compose rules |
| FE.2 | V dispatch (8 levels → ValenceState) | PLAN_FORMULA_ENGINE | ~100 LOC | FREE | Potential energy → approach/avoidance behavior |
| FE.3 | A dispatch (8 levels → ArousalState) | PLAN_FORMULA_ENGINE | ~100 LOC | FREE | Damped oscillator → energy regime behavior |
| FE.4 | T SplineKnot accumulation | PLAN_FORMULA_ENGINE | ~300 LOC | FREE | Mỗi observe → append knot (duration, freq, amp, phase) |
| FE.5 | T Spline interpolation | PLAN_FORMULA_ENGINE | ~200 LOC | FE.4 | History → curve → temporal behavior |
| FE.6 | Wire formula engine vào pipeline | PLAN_FORMULA_ENGINE | ~200 LOC | FE.1-5 | encode → eval → store |
| FE.7 | Test: đọc P → reconstruct formula | PLAN_FORMULA_ENGINE | ~150 LOC | FE.6 | Verify giá trị tự mô tả |
| 7.2 | Mobile (Android + iOS) | PLAN_7_2 | 2-3 tuần | FREE | ARM64 native + WASM iOS. |

### Tier 4 — Cắt dây rốn (Rust → 0%)

| ID | Task | Plan | Effort | Status | Notes |
|----|------|------|--------|--------|-------|
| CUT.1 | Migrate runtime Rust → Olang | PLAN_REWRITE GD2 | RẤT LỚN | FREE | emotion/silk/agents crate → .ol files chạy trên ASM VM. Hiện .ol stubs có, cần wire vào ASM VM thay Rust. |
| CUT.2 | Migrate tools Rust → Olang | PLAN_REWRITE GD3 | LỚN | FREE | builder/server/bench/seeder → .ol. builder.ol đã có. |
| CUT.3 | Migrate tests Rust → Olang | — | LỚN | FREE | 1190 Rust tests → Olang test framework. |
| CUT.4 | Remove Rust dependency | PLAN_REWRITE | — | BLOCKED | Chỉ khi CUT.1-3 xong. origin.olang = 1 file tự đủ. |

---

## Dependency Graph

```
                    ┌─────────────────────────────────────────┐
                    │         TIER 1 — Làm ngay               │
                    │                                         │
  P2.0 DONE ✅ ────┤  8.1-8.3 DONE ✅   12.1→12.5 DONE ✅   │
  P2.0b CLAIMED    │  11.3→11.2→11.5 (E2E Server) FREE      │
                    └─────────────────────────────────────────┘
       │                    │
       ▼                    ▼
  ┌────────────┐    ┌───────────────┐
  │ TIER 2     │    │ TIER 2        │
  │ P2.2 Emot  │    │ 9 REPL        │
  │ P2.3 Know  │    │   ↓           │
  │ P2.4 Agent │    │ 10 Browser    │
  │   ↓        │    │ 11.1 Demo     │
  │ P2.5 E2E   │    │ 11.4 Native   │
  └────────────┘    └───────────────┘

  TIER 3 (song song):
    TLC (Test Logic) | AUTH (First-run) | 7.2 (Mobile)

  TIER 4 (cắt dây rốn — sequential):
    CUT.1 (Runtime) → CUT.2 (Tools) → CUT.3 (Tests) → CUT.4 (Remove Rust)
```

---

## Log thay đổi

```
2026-03-18  Tạo TASKBOARD. B1-B3, AUTH, Phase 0.1-0.2 DONE.
2026-03-19  Phase 0-9 ALL DONE. INTG ALL DONE. origin.olang 1.35MB ELF.
2026-03-21  V2 Migration T1-T12 DONE. Spec v3 audit → Phase 14-16 thêm.
2026-03-21  Session 2pN6F: Task 12 + Phase 15 (6) + Phase 16 (4) + T13/T14/T16 DONE.
2026-03-21  Chỉ còn 14.2 (Alias) + 14.3 (Silk vertical) FREE.
2026-03-21  14.2 (Alias table) DONE. 33K entries tách riêng khỏi KnowTree. Dọn plans/done/.
2026-03-21  14.3 (Silk vertical parent_map persistence) DONE.
2026-03-21  TẤT CẢ TASKS DONE. Bắt đầu lên kế hoạch Giai đoạn 2.
2026-03-21  Thêm 30+ tasks từ Plans còn lại. 3 tiers ưu tiên. Dependency graph.
```
→ Full log: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)
