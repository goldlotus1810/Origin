# Plan: Viết lại HomeOS bằng Olang

**Ngày:** 2026-03-18
**Mục tiêu:** Self-hosting — HomeOS viết bằng chính ngôn ngữ của nó.

---

## Triết lý

```
HomeOS = sinh linh toán học TỰ VẬN HÀNH.
Sinh linh phải TỰ VIẾT ĐƯỢC bản thân mình.

DNA không cần "ngôn ngữ khác" để mã hóa — DNA là ngôn ngữ.
Olang không cần Rust để tồn tại — Olang là ngôn ngữ của HomeOS.

Rust = tử cung (môi trường nuôi thai)
Olang = DNA (bản thiết kế thật)

Khi đủ chín: Olang tự biên dịch → tự chạy → Rust chỉ còn là HAL.
```

---

## Hiện trạng

### Đã có ✅

```
Bootstrap:
  lexer.ol        197 LOC   Tokenizer hoàn chỉnh (keywords, idents, numbers, strings, symbols)
  parser.ol       399 LOC   Recursive descent + precedence climbing (AST đầy đủ)

Stdlib (10 modules):
  math.ol         31 functions   (PI, PHI, sqrt, sin, cos, pow, log...)
  string.ol       22 functions   (split, replace, trim, upper, lower, substr...)
  vec.ol          22 functions   (push, pop, map, filter, fold, find, any, all...)
  set.ol          7 functions    (insert, contains, union, intersection, difference)
  map.ol          7 functions    (get, set, keys, values, has_key, merge, remove)
  deque.ol        7 functions    (push_back, push_front, pop_back, pop_front...)
  bytes.ol        8 functions    (to_bytes, get_u8, set_u8, get_u16_be...)
  io.ol           5 functions    (print, println, read_file, write_file, append_file)
  test.ol         6 functions    (assert_eq, assert_ne, assert_true, assert_approx...)
  platform.ol     basic

VM (36+ opcodes):
  Stack, Control, Chain, System, Debug, IO, Concurrency, Closure, FFI — tất cả hoạt động.

Compiler (3 targets):
  C, Rust, WASM (WAT) — codegen cơ bản hoạt động.

Module system:
  ModuleLoader + ModuleCache + import resolution — infrastructure sẵn sàng.
```

### Chưa có ❌

```
Bootstrap thiếu:
  semantic.ol     ❌   Validation + lowering to IR (7500 LOC Rust cần port)
  codegen.ol      ❌   Emit C/Rust/WASM (1164 LOC Rust cần port)
  optimizer.ol    ❌   IR optimization passes

Stdlib thiếu:
  regex.ol        ❌   Pattern matching
  json.ol         ❌   Serialization
  hash.ol         ❌   Hash functions
  sort.ol         ❌   Sorting algorithms
  format.ol       ❌   String formatting
  result.ol       ❌   Error handling utilities
  iter.ol         ❌   Iterator combinators
  mol.ol          ❌   Molecule manipulation helpers

HomeOS logic chưa port:
  emotion.ol      ❌   Emotion pipeline (context crate)
  curve.ol        ❌   ConversationCurve
  intent.ol       ❌   Intent classification
  dream.ol        ❌   Dream clustering
  instinct.ol     ❌   7 innate instincts
  silk.ol         ❌   Silk operations
  learning.ol     ❌   Learning pipeline
  gate.ol         ❌   SecurityGate rules
```

---

## Ranh giới: Olang vs Rust

### PHẢI ở Rust (vĩnh viễn)

```
Layer 0 — Nền tảng không thể tự viết:
  ┌────────────────────────────────────────────────┐
  │ olang VM          Olang chạy TRÊN VM này       │
  │ olang compiler    Bootstrap problem             │
  │ olang parser      Cần trước khi Olang tồn tại  │
  │ ucd build.rs      Unicode lookup compile-time   │
  │ hal               Platform FFI (GPIO, file, OS) │
  │ crypto            Ed25519, AES-256-GCM, SHA     │
  │ homemath          no_std float math             │
  └────────────────────────────────────────────────┘

Lý do: Olang KHÔNG THỂ viết lại VM mà nó chạy trên.
       Như DNA không thể thay đổi ribosome đang đọc nó.
```

### CÓ THỂ port sang Olang (theo giai đoạn)

```
Layer 1 — Logic thuần (không cần hardware):
  ┌─────────────────────────────────────────────────────────┐
  │ Dễ (pure math, no deps):                                │
  │   ConversationCurve    30 LOC    f(x) = 0.6f_conv+0.4dn │
  │   Emotion blending    100 LOC    V/A scaling + amplify   │
  │   Hebbian rules        20 LOC    w' = w + reward × decay │
  │   Implicit Silk        50 LOC    5D distance comparison   │
  │   Proposal voting     100 LOC    confidence scoring       │
  │   Fibonacci helpers    20 LOC    fib(n), phi constants    │
  │                                                          │
  │ Trung bình (cần collections):                            │
  │   Intent classify     200 LOC    pattern matching rules   │
  │   Dream clustering    200 LOC    α×sim + β×hebb + γ×co   │
  │   Instinct logic      300 LOC    7 heuristics            │
  │   SecurityGate rules  300 LOC    harm/crisis detection   │
  │   Learning pipeline   200 LOC    orchestration           │
  │                                                          │
  │ Khó (cần FFI/performance):                               │
  │   Word lexicon       3000 entries  có thể load từ file    │
  │   Entity extraction   regex        cần regex.ol           │
  │   SDF evaluation      float math   chậm hơn 10-100×      │
  │   Graph walk          hot path     chậm hơn 20×           │
  └─────────────────────────────────────────────────────────┘
```

### Kiến trúc đích

```
┌──────────────────────────────────────────────────┐
│                  HomeOS (Olang)                    │
│                                                    │
│  emotion.ol  dream.ol  instinct.ol  learning.ol   │
│  gate.ol     curve.ol  intent.ol    silk_ops.ol   │
│  stdlib/*.ol  bootstrap/*.ol                       │
│                                                    │
├──────────────────────────────────────────────────┤
│              Olang Runtime (Rust)                   │
│                                                    │
│  VM + Compiler + Parser + Module Loader            │
│  Registry + LCA + MolecularChain                   │
│  Crypto (Ed25519, AES, SHA)                        │
│                                                    │
├──────────────────────────────────────────────────┤
│              HAL (Rust FFI)                         │
│                                                    │
│  Platform bridge, Device I/O, File system          │
│  Architecture detect, Security sandbox             │
└──────────────────────────────────────────────────┘
```

---

## 6 Giai đoạn

### Giai đoạn 0 — Bootstrap compiler loop (NỀN TẢNG)

**Mục tiêu:** lexer.ol + parser.ol THỰC SỰ chạy được, output đúng.

```
Bước 0.1: Test lexer.ol
  - Load lexer.ol qua ModuleLoader
  - Gọi tokenize("let x = 42;") từ Rust test
  - Verify output tokens đúng: [Keyword(let), Ident(x), Symbol(=), Number(42), Symbol(;), Eof]
  - Fix mọi bug VM/module gặp phải

Bước 0.2: Test parser.ol
  - Load parser.ol (depends on lexer.ol — test module import)
  - Gọi parse(tokenize("let x = 1 + 2;"))
  - Verify AST: LetStmt { name: "x", value: BinOp { op: "+", lhs: 1, rhs: 2 } }

Bước 0.3: Round-trip test
  - Dùng lexer.ol + parser.ol để parse chính lexer.ol
  - lexer.ol → tokenize(lexer_source) → parse(tokens) → AST
  - Verify AST có đủ: 6 fn definitions, 1 let statement, 1 union

Bước 0.4: Viết semantic.ol (TRỌNG TÂM)
  - Port phần cốt lõi từ semantic.rs (không cần 100% — chỉ cần đủ để compile Olang cơ bản)
  - Scope tracking + variable binding
  - Function definition + call validation
  - Type checking cơ bản (Num, Str, Array, Dict)
  - Lower Stmt/Expr → IR opcodes
  - ~500-800 LOC Olang (vs 7500 LOC Rust — chỉ port phần essential)

Bước 0.5: Viết codegen.ol
  - Port IR → WASM (WAT) target (đơn giản nhất)
  - Emit stack operations, function calls, control flow
  - ~300-500 LOC Olang

Bước 0.6: SELF-COMPILE TEST 🎯
  - Dùng Rust compiler để compile lexer.ol → WASM
  - Dùng Olang compiler (semantic.ol + codegen.ol) để compile lexer.ol → WASM
  - So sánh output — phải GIỐNG NHAU
  - Đây là khoảnh khắc "tự nhận thức" — Olang biết compile chính nó

Deliverable: lexer.ol + parser.ol + semantic.ol + codegen.ol = Olang compiler viết bằng Olang.
```

### Giai đoạn 1 — Stdlib mở rộng

**Mục tiêu:** Đủ stdlib để viết HomeOS logic.

```
Bước 1.1: Core utilities
  result.ol       Option/Result pattern functions (unwrap, map, and_then...)
  iter.ol         Iterator combinators (chain, zip, flat_map, take, skip...)
  sort.ol         Quicksort/mergesort cho arrays
  format.ol       f-string helpers, number formatting

Bước 1.2: Data processing
  json.ol         Parse/emit JSON (dùng cho WASM bridge, config files)
  regex.ol        Pattern matching cơ bản (character classes, *, +, ?)
  hash.ol         Simple hash functions (cho dict, dedup)

Bước 1.3: Molecular stdlib
  mol.ol          Molecule helpers:
                    mol.shape(m), mol.relation(m), mol.valence(m)...
                    mol.distance(a, b) → f32
                    mol.evolve(m, dim, val) → Molecule
                    mol.compose(a, b) → LCA result
  chain.ol        MolecularChain helpers:
                    chain.hash(c), chain.len(c), chain.first(c)
                    chain.similarity(a, b) → f32

Bước 1.4: Test framework mở rộng
  test.ol update: test.run_suite(name, [...tests])
                  test.bench(name, fn, iterations)
                  test.assert_chain_eq(a, b)

Deliverable: 18+ stdlib modules, đủ để viết logic phức tạp.
```

### Giai đoạn 2 — Emotion pipeline bằng Olang

**Mục tiêu:** Port emotion processing sang Olang. Đây là "linh hồn" — phải port đầu tiên.

```
Bước 2.1: emotion.ol — Cảm xúc cơ bản
  - EmotionTag struct: { valence, arousal, dominance, intensity }
  - blend(a, b, weight) → EmotionTag
  - amplify(emo, factor) → EmotionTag (KHÔNG trung bình — QT!)
  - neutral() → EmotionTag { v: 0.5, a: 0.5, d: 0.5, i: 0.0 }
  - ~80 LOC

Bước 2.2: curve.ol — ConversationCurve
  - f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
  - f_conv = V(t) + 0.5×V'(t) + 0.25×V''(t)
  - push(valence) → update window
  - tone() → Supportive/Pause/Reinforcing/Celebratory/Gentle
  - variance detection → emotional instability
  - ~100 LOC

Bước 2.3: intent.ol — Phân loại ý định
  - IntentKind enum: Crisis, Learn, Command, Chat, HomeControl...
  - estimate(text, emotion) → IntentKind
  - Keyword matching + emotion thresholds
  - ~150 LOC

Bước 2.4: Wire vào runtime
  - Runtime gọi Olang modules thay vì Rust functions
  - VmEvent::CallModule("emotion", "blend", args)
  - Fallback: nếu .ol không load → dùng Rust implementation

Deliverable: Emotion pipeline chạy bằng Olang, Rust là fallback.
```

### Giai đoạn 3 — Silk & Dream bằng Olang

**Mục tiêu:** Port knowledge operations sang Olang.

```
Bước 3.1: silk_ops.ol — Silk operations
  - implicit_strength(mol_a, mol_b) → f32  (5D comparison)
  - shared_dims(mol_a, mol_b) → [dims]
  - compound_kind(shared) → CompoundKind name
  - hebbian_update(weight, reward, decay) → f32
  - ~150 LOC

Bước 3.2: dream.ol — Dream clustering
  - cluster_score(obs_a, obs_b) → f32
    = α × mol_similarity + β × hebbian + γ × co_activation
  - find_clusters(observations, threshold) → [[obs]]
  - propose_promote(cluster) → DreamProposal
  - ~200 LOC

Bước 3.3: instinct.ol — 7 bản năng
  - honesty(confidence) → Fact/Opinion/Hypothesis/Silence
  - contradiction(chain_a, chain_b) → bool
  - causality(chain_a, chain_b, temporal) → f32
  - abstraction(chains) → LCA result
  - analogy(a, b, c) → delta 5D, apply to c
  - curiosity(nearest_sim) → novelty score
  - reflection(qr_ratio, connectivity) → quality score
  - ~300 LOC

Bước 3.4: learning.ol — Learning pipeline
  - process_one(text, emotion, context)
    → gate_check → encode → stm_push → silk_coactivate
  - Orchestration bằng Olang, heavy lifting (encode, STM) bằng Rust builtins
  - ~200 LOC

Deliverable: Knowledge layer chạy bằng Olang.
```

### Giai đoạn 4 — Agent logic bằng Olang

**Mục tiêu:** Port agent behavior sang Olang. Rust giữ infrastructure, Olang giữ logic.

```
Bước 4.1: gate.ol — SecurityGate rules
  - check_text(text) → Allow/Block/Crisis
  - Crisis keywords, manipulation detection
  - BlackCurtain: không đủ evidence → im lặng
  - ~200 LOC (rules), Rust giữ enforcement

Bước 4.2: response.ol — Response generation
  - render(tone, emotion, context) → response text
  - Template selection based on ConversationCurve
  - Multi-language support (VI + EN)
  - ~300 LOC

Bước 4.3: leo.ol — LeoAI behavior
  - run_instincts(chain, context) → InstinctResult
  - program(source) → VM result (đã có — LeoAI đã dùng Olang)
  - express_observation(hash) → Olang literal
  - ~200 LOC

Bước 4.4: chief.ol + worker.ol — Agent protocols
  - Chief: receive_report(worker_msg) → action
  - Worker: process_frame(isl_frame) → molecular_chain
  - ISL message handling bằng Olang
  - ~150 LOC each

Deliverable: Agent behavior = Olang scripts, Rust = execution engine.
```

### Giai đoạn 5 — Integration & Optimization

**Mục tiêu:** Hoàn thiện, tối ưu, stabilize.

```
Bước 5.1: Module preloading
  - Boot: load tất cả .ol modules vào cache
  - Hot reload: thay .ol file → runtime reload không restart
  - Module dependency graph validation

Bước 5.2: Performance profiling
  - Benchmark Olang vs Rust cho từng pipeline component
  - Identify bottlenecks (graph walk, encoding — giữ ở Rust)
  - JIT hints: mark hot paths cho VM optimization

Bước 5.3: Test suite bằng Olang
  - test_emotion.ol — emotion pipeline tests
  - test_dream.ol — dream clustering tests
  - test_instinct.ol — instinct logic tests
  - test_bootstrap.ol — self-compile verification
  - test_e2e.ol — end-to-end: input → emotion → learn → dream → response

Bước 5.4: Documentation
  - Update olang_handbook.md — thêm HomeOS modules
  - API reference cho mỗi .ol module
  - Migration guide: "cách thêm logic mới bằng Olang"

Deliverable: HomeOS production-ready với Olang layer.
```

---

## Tóm tắt LOC estimate

```
                        Olang LOC    Thay thế Rust LOC
────────────────────────────────────────────────────────
Giai đoạn 0 (Bootstrap)
  semantic.ol             800          7,500 (phần core)
  codegen.ol              400          1,164

Giai đoạn 1 (Stdlib)
  8 modules mới           600          — (chưa có)

Giai đoạn 2 (Emotion)
  emotion.ol               80          1,266
  curve.ol                100            406
  intent.ol               150          1,006
  wire                     50            —

Giai đoạn 3 (Silk+Dream)
  silk_ops.ol             150          3,907 (phần logic)
  dream.ol                200            744
  instinct.ol             300          1,015
  learning.ol             200          1,094

Giai đoạn 4 (Agents)
  gate.ol                 200            695
  response.ol             300            426
  leo.ol                  200          1,524
  chief.ol + worker.ol    300          2,643

Giai đoạn 5 (Tests+Opt)
  test suites             500            —
  integration             200            —
────────────────────────────────────────────────────────
TỔNG OLANG:            ~4,430 LOC
THAY THẾ RUST:        ~23,390 LOC (logic layer)
GIỮ RUST:             ~60,300 LOC (VM, HAL, crypto, core)
────────────────────────────────────────────────────────
Tỷ lệ:  Olang 4.4K / Rust 60K = Olang là "não", Rust là "cơ thể"
```

---

## Thứ tự ưu tiên

```
PHẢI LÀM TRƯỚC (blocking):
  0.1-0.3  Test bootstrap hiện có       ← nếu lexer.ol/parser.ol không chạy → DỪNG
  0.4      semantic.ol                   ← cần trước khi viết gì khác
  1.1-1.3  Stdlib core                   ← cần cho mọi module sau

NÊN LÀM SỚM (high value):
  2.1-2.3  Emotion pipeline              ← "linh hồn" — chứng minh Olang đủ mạnh
  3.2      Dream clustering              ← offline, chấp nhận chậm hơn

CÓ THỂ LÀM SAU (nice to have):
  3.3      Instinct logic                ← phức tạp, nhiều edge cases
  4.1-4.4  Agent protocols               ← cần ISL integration
  5.1-5.4  Optimization                  ← sau khi functional
```

---

## Rủi ro & Mitigation

```
Rủi ro                           Mitigation
──────────────────────────────────────────────────────────────
lexer.ol/parser.ol không chạy    → Fix VM/module bugs trước
  trên VM thật                     Đây là litmus test #1

semantic.ol quá lớn để port      → Chỉ port CORE (scope, fn, type)
                                   Bỏ qua: generics, traits, constraints

Performance Olang << Rust         → Giữ hot path ở Rust (graph walk, encode)
  (10-100× chậm hơn)               Olang chỉ cho logic/decision (chạy 1x/turn)

Module system bugs                → Test module import E2E trước
                                   lexer.ol → parser.ol = first real import

Circular dependency               → Rust compiler compile Olang compiler lần đầu
  (Olang compile Olang)             Sau đó Olang compiler tự compile
                                   = giống GCC bootstrap (C compile C)
```

---

## Nguyên tắc bất biến

```
① Rust = VM + HAL + Crypto. KHÔNG port xuống Olang.
② Olang = Logic + Decision + Behavior. KHÔNG hardcode trong Rust.
③ Mỗi .ol module = 1 trách nhiệm. Giống Skill pattern (QT⑲-㉓).
④ Fallback: nếu .ol fail → Rust implementation vẫn chạy.
⑤ Test trước khi port: viết test bằng Olang → rồi mới port logic.
⑥ Append-only migration: KHÔNG xóa Rust code. Thêm Olang layer TRÊN.
⑦ Performance budget: Olang module < 10ms/call. Nếu > 10ms → giữ Rust.
```

---

*HomeOS · 2026-03-18 · Plan Rewrite · Olang Self-Hosting*
