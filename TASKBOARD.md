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

### Tier 1 — DONE (session 2026-03-22)

| ID | Status | Summary |
|----|--------|---------|
| P2.0, P2.0b | DONE ✅ | VM builtins + 28 test failures fixed. PR #228 #229 #239. |
| VM.1-5,7,8 | DONE ✅ | VM optimization 10.73s→2.91s (3.7x). String no-alloc, keyword hash, CallBuiltin, scope cache, KnowTree sampling, Bellman path. |
| VM.6 | SKIP | SSO không cần — risk cao, performance đủ. |
| 8.1-8.3 | DONE ✅ | Parser: hex, ==, keywords as ident. |
| 11.2,11.3,11.5 | DONE ✅ | Server --eval + E2E tests 9/9 + Makefile. |
| 12.1-12.5 | DONE ✅ | Emotion pipeline + context + intent + response + language. |
| FE.1-8 | DONE ✅ | Formula Engine: R/V/A/T dispatch + T×S + pipeline wire. |
| TLC | DONE ✅ | 15 logic tests (6 patterns + 5 checkpoints). |
| AUTH | DONE ✅ | First-run auth: terms + Ed25519 master key. |
| UTF32→L1 | DONE ✅ | Import udc_utf32.json → origin.olang (41K chars + aliases). |
| Spec IX.I-K | DONE ✅ | KnowTree Sampling + Bellman Path + String Fingerprint. |

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
| P2.0b | Fix 37 remaining olang test failures | — | ~300 LOC | P2.0 | DONE ✅ | 28→6 failures fixed (string cmp, closure marker, enum split, to_number). PR #239. 6 còn lại = VM perf timeout. |
| VM.1 | String compare không allocate | PLAN_VM_OPT | ~60 LOC | — | DONE ✅ | char_at O(1) direct index, chain_cmp_bytes zero-alloc. PR #239. |
| VM.2 | Keyword hash builtin | PLAN_VM_OPT | ~40 LOC | — | DONE ✅ | __str_is_keyword O(1) matches!() builtin. PR #239. |
| VM.3 | Micro-opts (step batch + flags) | PLAN_VM_OPT | ~30 LOC | — | DONE ✅ | Step check mỗi 256 iterations. PR #239. |
| VM.4 | Scope variable cache | PLAN_VM_OPT | ~100 LOC | VM.1 | DONE ✅ | FNV-1a hash, 8 entries, auto-invalidate. PR #239. |
| VM.5 | Builtin dispatch table | PLAN_VM_OPT | ~200 LOC | VM.1 | DONE ✅ | Op::CallBuiltin(u8), 16 builtins inlined. PR #239. |
| VM.6 | Small-chain SSO | PLAN_VM_OPT | ~300 LOC | VM.1-5 | SKIP | Không cần sửa — risk cao, 189 callsites. Performance đủ với VM.1-5. |
| VM.7 | KnowTree sampling | PLAN_VM_OPT | ~150 LOC | FE.6 | DONE ✅ | eval_valence/arousal_from_table, Fib-sized sampling, 3-tier fallback. |
| VM.8 | Bellman path optimization | PLAN_VM_OPT | ~100 LOC | VM.7 | DONE ✅ | BellmanPathCache 55 entries, φ⁻¹ decay, Q-table eviction. 7/7 tests. |
| 8.1 | Parser: hex literals (0xFF) | PLAN_8 | ~80 LOC | — | DONE ✅ | Đã implement (session 2pN6F). |
| 8.2 | Parser: == trong match/struct | PLAN_8 | ~200 LOC | — | DONE ✅ | Đã implement (session 2pN6F). |
| 8.3 | Parser: keywords as ident + struct colon | PLAN_8 | ~100 LOC | — | DONE ✅ | Đã implement (session 2pN6F). |
| 12.1 | Wire walk_emotion() vào response | PLAN_12 | ~100 LOC | — | DONE ✅ | compose_response() wired (session 2pN6F). |
| 12.2 | Context recall trong response | PLAN_12 | ~80 LOC | 12.1 | DONE ✅ | ResponseContext + context-aware intent (session 2pN6F). |
| 12.3 | Intent estimation dùng context | PLAN_12 | ~120 LOC | 12.2 | DONE ✅ | Causality→skip AddClarify, repetition→EmpathizeFirst. |
| 12.4 | Response composer thay template | PLAN_12 | ~200 LOC | 12.3 | DONE ✅ | compose_response() thay render(). |
| 12.5 | Language detection + instinct wire | PLAN_12 | ~60 LOC | 12.4 | DONE ✅ | detect_language tiếng Việt không dấu. |
| 11.3 | Server --eval mode | PLAN_11 | ~80 LOC | — | DONE ✅ | --eval "expr" inline + stdin mode. PR claude/server-eval-mode. |
| 11.2 | Rust E2E test suite | PLAN_11 | ~300 LOC | 11.3 | DONE ✅ | t16_e2e_demo.rs: 9/9 pass. Đã có sẵn. |
| 11.5 | Makefile targets (demo/verify) | PLAN_11 | ~50 LOC | 11.2 | DONE ✅ | make demo + make verify đã có sẵn, verify 9/9 pass. |

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

#### VM.1 — String compare không allocate (5-10x speedup)
```
File:    crates/olang/src/exec/vm.rs
Vấn đề:  char_at() gọi chain_to_string() → allocate String mỗi lần
         200 dòng × 25 chars = 5000 allocations chỉ cho tokenize
Cách fix:
  1. __str_char_at: truy cập trực tiếp source.0[i] → O(1), zero alloc
     TRƯỚC: chain_to_string(&s).chars().nth(i)  // allocate String
     SAU:   s.0[i] & 0xFF → str_byte_mol(byte)  // zero alloc
  2. __str_substr: slice source.0[start..end].to_vec() thay vì decode+re-encode
  3. __cmp_lt/gt/le/ge cho strings: compare u16 slices trực tiếp
     fn chain_cmp_order(a, b) → Ordering  // zero alloc
  4. is_string_chain_fast: check first molecule only (uniform encoding)
     TRƯỚC: chain.0.iter().all(|&b| b & 0xFF00 == 0x2100)  // O(N)
     SAU:   chain.0.first().map_or(false, |&b| b & 0xFF00 == 0x2100)  // O(1)
DoD:     roundtrip_lexer_ol_self_tokenize < 3s (hiện 10s+)
         cargo test -p olang -- vm::tests → 0 FAILED
```

#### VM.2 — Keyword hash builtin (1.5x speedup)
```
File:    crates/olang/src/exec/vm.rs + stdlib/bootstrap/lexer.ol
Vấn đề:  is_keyword() loop 28 entries × full string compare
         500 identifiers × 28 = 14,000 comparisons mỗi tokenize
Cách fix:
  1. Thêm builtin "__str_is_keyword" trong vm.rs:
     match &bytes[..] { b"let"|b"fn"|b"if"|... → true, _ → false }
  2. Sửa lexer.ol: thay is_keyword(text) bằng __is_keyword(text) hoặc
     lowering intercept tự động
DoD:     is_keyword() = O(1) thay O(28)
```

#### VM.3 — Micro-optimizations (1.2x speedup)
```
File:    crates/olang/src/exec/vm.rs
Cách fix:
  1. Step batch: if steps & 0xFF == 0 && steps >= max → check mỗi 256 steps
  2. EarlyReturn flag: bool thay vì events.iter().any(|e| matches!(EarlyReturn))
  3. Pop optimization: stack.data.last().cloned() cho Dup
DoD:     Không regression, overall 1.2x faster
```

#### VM.4 — Scope variable cache (2-4x speedup)
```
File:    crates/olang/src/exec/vm.rs
Vấn đề:  LoadLocal("pos") scan 3-4 scopes × 10-15 vars = 45 string cmps
Cách fix:
  struct ScopeCache { entries: [(u32, usize, usize); 8] }  // Fib(6)=8
  LoadLocal: hash(name) → cache hit? → O(1) : linear scan → update cache
  Store/ScopeEnd: invalidate cache entries
DoD:     Tokenize inner loop 2-4x faster
         Không regression
```

#### VM.5 — Builtin dispatch table (2-3x speedup)
```
File:    crates/olang/src/exec/ir.rs + vm.rs + semantic.rs
Vấn đề:  Op::Call("__eq") match 207+ string arms tuần tự
Cách fix:
  1. Thêm Op::CallBuiltin(u8) vào ir.rs
  2. BuiltinId enum: EQ=0, CMP_LT=1, CHAR_AT=2, ARRAY_GET=3, ...
  3. Lowering: detect __eq → emit CallBuiltin(BID_EQ)
  4. VM: BUILTIN_TABLE[id](...) → jump table O(1)
DoD:     Builtin calls O(1) thay O(100)
         cargo test -p olang → 0 regression
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

### Tier 2 — UNBLOCKED (Tier 1 DONE) → Chuẩn bị cắt Rust

| ID | Task | Plan | Effort | Depends | Status | Notes |
|----|------|------|--------|---------|--------|-------|
| P2.2 | Emotion pipeline (.ol) | PLAN_PHASE2 | ~200 LOC | P2.0 ✅ | DONE ✅ | emotion.ol 156 LOC + intent.ol 127 LOC + curve.ol 110 LOC. |
| P2.3 | Knowledge layer (.ol) | PLAN_PHASE2 | ~150 LOC | P2.0 ✅ | DONE ✅ | silk_ops 166 + dream 181 + instinct 197 + learning 160 LOC. Đã có sẵn. |
| P2.4 | Agent behavior (.ol) | PLAN_PHASE2 | ~300 LOC | P2.0 ✅ | DONE ✅ | response 100 + leo 41 + chief 36 + worker 42 + gate 51 LOC. |
| P2.5 | E2E integration test | PLAN_PHASE2 | ~50 LOC | P2.2-4 | BLOCKED | 5 test cases end-to-end. Chờ P2.2-4. |
| 9 | Native REPL | PLAN_9 | ~600 LOC | 8.1-8.3 ✅ | FREE | ./origin → REPL. Parser đã fix. |
| 10 | Browser E2E | PLAN_10 | ~500 LOC | 9 | BLOCKED | origin.html, WASM. Chờ REPL. |
| 11.1 | Demo script (10 scenarios) | PLAN_11 | ~300 LOC | 8 ✅, 9 | BLOCKED | Chờ REPL. |
| 11.4 | Native binary --eval | PLAN_11 | ~50 LOC | 9 | BLOCKED | Chờ REPL. |

### Tier 3 — Lớn, cần kế hoạch riêng

| ID | Task | Plan | Effort | Status | Notes |
|----|------|------|--------|--------|-------|
| PW | P_weight migration 5B→2B | PLAN_PWEIGHT | LỚN | DONE ✅ | T1-T12 V2 Migration hoàn thành. |
| V2 | V2 Migration BIG BANG | PLAN_V2 | RẤT LỚN | DONE ✅ | T1-T16 ALL DONE. |
| UDC | UDC Rebuild (59 blocks) | PLAN_UDC | Nhiều sessions | DONE ✅ | 8,846 entries, build pipeline v3.1, KT31 format. |
| TLC | Test Logic Check (6 patterns) | PLAN_TEST_LOGIC | Trung bình | DONE ✅ | t18_logic_check.rs: 15/15 pass. 6 patterns + 5 checkpoints verified. |
| AUTH | First-run Auth setup | PLAN_AUTH_first_run | ~200 LOC | DONE ✅ | Terms screen + Master Key (Ed25519) wired into server boot. PR claude/fe6. |
| **FE** | **Formula Engine — Giá trị = Công thức** | **PLAN_FORMULA_ENGINE** | **1547 LOC** | **DONE ✅** | **PR #237. formula.rs + spline.rs + parametric.rs** |
| FE.1 | R dispatch (16 relation → operations) | PLAN_FORMULA_ENGINE | 848 LOC | DONE ✅ | RelationOp, compose(), is_symmetric/transitive |
| FE.2 | V dispatch (8 levels → ValenceState) | PLAN_FORMULA_ENGINE | (in formula.rs) | DONE ✅ | Potential energy, approach_tendency() |
| FE.3 | A dispatch (8 levels → ArousalState) | PLAN_FORMULA_ENGINE | (in formula.rs) | DONE ✅ | Damped oscillator, urgency(), oscillation_freq() |
| FE.4 | T SplineKnot accumulation | PLAN_FORMULA_ENGINE | 411 LOC | DONE ✅ | SplineKnot 24B, observe_text/sensor, TimeHistory |
| FE.5 | T Spline interpolation | PLAN_FORMULA_ENGINE | (in spline.rs) | DONE ✅ | amplitude_at(), predict(), detect_periodicity() |
| FE.6 | Wire formula engine vào pipeline | PLAN_FORMULA_ENGINE | — | DONE ✅ | FormulaState wired into T6b2: urgency→arousal, approach→valence amplify. |
| FE.7 | Test: đọc P → reconstruct formula | PLAN_FORMULA_ENGINE | — | DONE ✅ | t17_formula_engine.rs: 9/9 pass. R/V/A/T roundtrip + urgency + approach. |
| **FE.8** | **T×S: T làm tham số cho SDF** | **PLAN_FORMULA_ENGINE** | **285 LOC** | **DONE ✅** | **ParametricSdf, sdf_union, snowman() ⛄** |
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
