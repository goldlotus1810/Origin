# TASKBOARD — Origin / Olang

> **Mọi AI session đọc file này TRƯỚC KHI bắt đầu làm việc.**
> **Viết OLANG. Rust legacy chỉ bug fix.**
> **Chi tiết lịch sử:** [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Trạng thái: FULL STACK (2026-03-25)

```
origin_new.olang = ~963KB native binary (985,435 bytes)
  ✅ Bootstrap compiler: lexer + parser + semantic + codegen (3,453 LOC Olang)
  ✅ Intelligence layer: 10-stage pipeline (alias→emoji→UDC→node→DN/QR→decode→output)
  ✅ Crypto: SHA-256 FIPS 180-4 in ASM
  ✅ WASM: runs in browser (3KB)
  ✅ OL.8: REPL calls stdlib functions (boot/eval closure bridge)
  ✅ fib(20) = 6,765 | __sha256("abc") = ba7816bf... | 20/20 tests
  ✅ ASM VM x86_64 (5,767 LOC), no libc, zero dependencies
  ✅ HomeOS: 44 files, 9,591 LOC Olang (alias, node, UDC decode, UTF-8, emoji)
  ✅ Streaming compiler: ALL 4 bootstrap files compile (0 segfaults)
     lexer 1.9s, codegen 2s, parser 2.7s, semantic 3s
  ✅ Lambda + map/filter/reduce/any/all/pipe (inline compiler builtins)
  ✅ T5: Layer 1 + ND.2/4 + LG.1/2/5 (auto-register, pipe, self-describe)
  ✅ Spec v3: SC.1,7,9-13 done (7/16)
```

---

## Hoàn thành ✅

### Rust Era (đã nén lại — xem `crates/EPITAPH.md`)

Phase 0-16, V2 Migration, UDC Rebuild, Formula Engine, Auth, VM Optimization 3.7x,
Parser upgrade, E2E tests, Logic check — TẤT CẢ DONE.
→ Chi tiết: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

### Self-hosting Era

| ID | Status | Summary |
|----|--------|---------|
| ASM.1-5 | DONE ✅ | Native binary boots. ASM VM x86_64, no libc. 806KB ELF. |
| REPL.1 | DONE ✅ | REPL loop: read → tokenize → parse → analyze → generate → eval. |
| REPL.2 | DONE ✅ | Bootstrap compiler: lexer.ol + parser.ol + semantic.ol + codegen.ol. |
| REPL.3 | DONE ✅ | Full language: arithmetic, strings, variables, if-else, while, functions. |
| REPL.4 | DONE ✅ | Deep recursion: fact(10)=3628800. VM scoping (snapshot/restore). |
| REPL.5 | DONE ✅ | Tree recursion: fib(20)=6765. BinOp rhs save + parser result save. |
| REPL.6 | DONE ✅ | 30+ ASM VM bugs fixed. 27/27 REPL tests. Clean output. |
| REPL.7 | DONE ✅ | Namespace collision fixes (45+ fn renames). ARRAY_INIT_CAP=256. |

---

## FREE Tasks — Olang Era

### Tier 1 — Port intelligence layer sang Olang

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| OL.1 | Encoder: text → molecule (.ol) | ~180 LOC | DONE ✅ | `encode <text>` REPL command. Block-range UCD mapper. LCA compose. Emotion. |
| OL.2 | Analysis: sentence fusion (.ol) | ~120 LOC | DONE ✅ | fusion.ol + pipeline in encoder.ol. Context detect + emotion compose. |
| OL.3 | Intent inference engine (.ol) | ~80 LOC | DONE ✅ | 6 intent types: chat/heal/learn/technical/command + tone selection. |
| OL.4 | Agents: dispatch pipeline (.ol) | ~60 LOC | DONE ✅ | gate→encode→analyze→leo dispatch. Crisis detection. `respond` command. |
| OL.5 | Response composer (.ol) | ~40 LOC | DONE ✅ | compose_reply: ack + follow-up by intent/tone. 5 tone modes. |

### Tier 2 — Mở rộng ngôn ngữ

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| OL.6 | For-in loops + range() | ~120 LOC | DONE ✅ | `for x in items { }`, `for i in range(n) { }`. PR #311. |
| OL.7 | Smart string concat | ~100 LOC | DONE ✅ | `"Age: " + 25` → auto-convert. `__to_string` builtin. PR #312. |
| OL.7b | String interpolation `$"hello {name}"` | ~50 LOC | DONE ✅ | Lexer desugars `$"...{expr}..."` to `"..." + __to_string(expr) + "..."`. |
| OL.7c | Else-if chains | ~20 LOC | DONE ✅ | `else if` syntax. Parser var save/restore. PR #317. |
| OL.7d | Pretty-print arrays | ~80 LOC | DONE ✅ | `emit [1,2,3]` instead of `[array 3]`. PR #318. |
| OL.7e | Variable assignment fix | ~5 LOC | DONE ✅ | `let b = b + a` now works. LetStmt name save. PR #320. |
| OL.7f | FieldAssign fix + audit | ~10 LOC | DONE ✅ | Full 18-site match binding audit. PR #321. |
| OL.8 | Import/module system | ~37 LOC ASM | DONE ✅ | Boot/eval closure bridge. REPL calls stdlib functions. Bit 63 tag. |
| OL.9 | Error handling | ~200 LOC | DONE ✅ | `try { ... } catch { ... }` + `__throw(msg)`. VM try_stack + parser + semantic. |
| OL.10 | Array comprehension | ~150 LOC | DONE ✅ | `[x * 2 for x in items if cond]`. Depth-indexed globals + manual token emit. |

### Tier 3 — Platform

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| OL.11 | ARM64 ASM VM | ~2000 LOC | WIP | 1,229 LOC. Boots bare. Closures added. Needs builtins+scoping for stdlib. |
| OL.12 | WASM target | ~1000 LOC | DONE ✅ | `emit 42` + `emit 1+2` works. 3KB binary. Node.js test harness. |
| OL.13 | Crypto in ASM | ~250 LOC | DONE ✅ (SHA-256) | `__sha256(str)` → 64-char hex. FIPS 180-4. 3/3 vectors pass. |
| OL.14 | Browser E2E | ~80 LOC | DONE ✅ | origin.html REPL. Dark theme. emit + arithmetic. |
| OL.15 | Mobile (Android/iOS) | ~1000 LOC | BLOCKED | Needs OL.11 ARM64 complete. |

### Tier 4 — Cắt dây rốn (hoàn toàn)

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| CUT.1 | Migrate Rust runtime → Olang | — | DONE ✅ | Rust crates = dead code. All runtime logic in Olang stdlib. |
| CUT.2 | Migrate Rust builder → Olang | LỚN | DONE ✅ | `build` command: copies VM + bytecode → new ELF. Self-build works! |
| CUT.3 | Olang test framework | — | DONE ✅ | `test` command, 12/12 tests. BLOCK.3 resolved. |
| CUT.4 | Self-build (no Rust) | — | DONE ✅ | `build` → 381KB binary. fib(20)=6765. Recursive self-build verified. |

### Tier 5 — UDC-native Olang (chuẩn hóa: mọi thứ = node = 16 bits)

> **Triết lý:** Thay vì nhớ hàng ngàn fn với hàng ngàn lệnh,
> chỉ cần nhớ 8,846 UDC/công thức và ghép chúng lại như Lego.
> 1 node(fn) == { x = val:UDC[ID]; ... } — 16 bits cho 1 hàm/lệnh.
> fn{fn{...}} == fn, n=∞-1 → không bao giờ thiếu bộ nhớ.
>
> **Nguyên tắc ∞-1:** Không materialize tất cả. Stream, accumulate, reference.
> **Nguyên tắc Structure=Meaning:** Syntax = toán. Không thêm keyword — cấu trúc tự nói.
> **Nguyên tắc T×S:** Mọi giá trị = tọa độ 5D. P_weight 2B = hình+quan hệ+cảm xúc+năng lượng+thời gian.

#### Phase 5A — Stabilize (ngôn ngữ ổn định)

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| ST.1 | BUG-INDEX/BUG-SORT fix | — | DONE ✅ | a[expr] heap overlap. ArrayLit→push. |
| ST.2 | map/filter/reduce | ~120 LOC | DONE ✅ | Inline compiler builtins. Lambda expressions + closure calls. |
| ST.2b | Lambda expressions | ~60 LOC | DONE ✅ | `fn(x) { body }` parsed + compiled. Higher-order functions work. |
| ST.2c | Cross-boundary closures | ~20 LOC ASM | DONE ✅ | eval_bc_base global. Boot↔eval closure calls fixed. |
| ST.3 | any/all + min/max/sum (boot) | ~80 LOC | DONE ✅ | Inline any/all compiler builtins. min_val/max_val/sum from iter.ol. |
| ST.4 | Regression test suite 20/20 | ~30 LOC | DONE ✅ | +4: idx_binop, idx_var_add, sort_first, sort_last. |

#### Phase 5A+ — Fix Intelligence (T5 Layer 1)

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| KN.1 | _mol_distance 5D | +5 LOC | DONE ✅ | S+R+V+A+T Manhattan, max 47 |
| KN.2 | _mol_similarity max 47 | +0 LOC | DONE ✅ | Was 14, now 47 |
| KN.3 | _text_to_chain all chars | +10 LOC | DONE ✅ | Was only first 2 chars per word |
| KN.4 | knowledge_search additive | +1 LOC | DONE ✅ | keyword×5 + mol_sim (was max) |
| IN.1 | Inline honesty instinct | +25 LOC | DONE ✅ | Confidence: [fact/opinion/hypothesis] |
| IN.2 | Curiosity for unknowns | +5 LOC | DONE ✅ | "chủ đề mới" when novelty > 7 |
| UT.1 | r_dispatch(r) | +20 LOC | DONE ✅ | 16 relation types → behavior tag |
| UT.2 | temporal_tag(t) | +8 LOC | DONE ✅ | 4 time levels → description |

#### Phase 5B — Node-native (1 LOC = 1 node = 16 bits)

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| ND.1 | Molecule literal | — | SKIP | `mol_new(s,r,v,a,t)` đã đủ. `__mol_pack` ASM nhanh hơn. |
| ND.2 | ASM mol extract builtins | ~90 LOC ASM | DONE ✅ | `__mol_s/r/v/a/t` bit extract + `__mol_pack` — 100x nhanh hơn __floor |
| ND.3 | Chain = array of u16 | ~30 LOC | DEFER | Current array works. Optimize khi memory tight. |
| ND.4 | fn_node metadata registry | ~70 LOC | DONE ✅ | register/fire/link/hot. fn has mol + fire_count + links. |
| ND.5 | Knowledge dual storage | — | KEEP | Giữ text+chain+mol+words (Sora). Chain cải thiện qua _text_to_chain fix. |
| ND.6 | Variable = node | — | DEFER | 20x performance hit. Giữ flat var_table. |

#### Phase 5C — Formula dispatch (giá trị = công thức = hình dạng)

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| FE.1 | R dispatch table | ~20 LOC | DONE ✅ | r_dispatch(r) → behavior tag. In encoder.ol. Foundation ready. |
| FE.2 | V/A physics | ~80 LOC | DEFER | Chỉ cần khi render 3D. V/A integers đủ cho emotion. |
| FE.3 | T spline parameters | ~10 LOC | DONE ✅ | temporal_tag(t). Sin/cos defer đến rendering. |
| FE.4 | S×T rendering | ~100 LOC | TODO | 18 SDF × T params = vô hạn hình dạng. f(p)=\|p\|-r, r=T.amplitude |
| FE.5 | 42 UDC encode formulas | ~200 LOC | TODO | F₀(cp)=[f_S,f_R,f_V,f_A,f_T]. Xem docs/UDC_DOC/UDC_formulas.md |

#### Phase 5D — Lego composition (fn = chain of nodes)

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| LG.1 | fn auto-register as node | ~15 LOC | DONE ✅ | Compiler emits fn_node_register after FnDef. Auto-tracked. |
| LG.2 | pipe() — Lego operator | ~15 LOC | DONE ✅ | `pipe(x, f1, f2, fn)` → fn(...f2(f1(x))). fn{fn{...}}==fn. |
| LG.3 | Silk = implicit từ chain order | ~40 LOC | TODO | Hybrid: implicit intra-chain + explicit inter-chain |
| LG.4 | Dream = cluster fn → skill | ~60 LOC | TODO | fn_node_hot(min) → cluster → promote to skill |
| LG.5 | fn self-describe | ~30 LOC | DONE ✅ | fn_node_describe: lazy mol + 5D metadata (V/A/R/T). |
| LG.5 | Self-describe: fn biết mình là gì | ~40 LOC | TODO | fn.mol → cảm xúc, fn.fire → hot function, fn.links → related fns |

---

## Dependency Graph

```
            ┌────────────────────────────────┐
            │ TIER 1 — Intelligence Layer    │  ✅ DONE
            └────────────┬───────────────────┘
            ┌────────────▼───────────────────┐
            │ TIER 2 — Language Features     │  ✅ DONE
            └────────────┬───────────────────┘
            ┌────────────▼───────────────────┐
            │ TIER 3 — Platform              │  ⚠️ ARM64 WIP
            └────────────┬───────────────────┘
            ┌────────────▼───────────────────┐
            │ TIER 4 — Cắt dây rốn           │  ✅ DONE
            └────────────┬───────────────────┘
                         │
    ═════════════════════╪══════════════════════ ← HIỆN TẠI
                         │
            ┌────────────▼───────────────────┐
            │ TIER 5A — Stabilize            │
            │ ST.1-4: bugs, stdlib, tests    │
            └────────────┬───────────────────┘
                         │
            ┌────────────▼───────────────────┐
            │ TIER 5B — Node-native          │
            │ ND.1-6: mol literals, chain,   │
            │         node type, knowledge   │
            └────────────┬───────────────────┘
                         │
            ┌────────────▼───────────────────┐
            │ TIER 5C — Formula dispatch     │
            │ FE.1-5: R/V/A/T/S eval,       │
            │         42 UDC formulas        │
            └────────────┬───────────────────┘
                         │
            ┌────────────▼───────────────────┐
            │ TIER 5D — Lego composition     │
            │ LG.1-5: fn=chain, compose,     │
            │         Silk, Dream, self-desc │
            └────────────────────────────────┘

  Sau 5D: HomeOS viết trên Olang = ghép Lego.
  Mỗi LOC = 1 node (16 bits). Mỗi fn = chain of nodes.
  Không nhớ hàng ngàn lệnh — chỉ nhớ 8,846 UDC + compose.
```
```

---

## Docs Conflicts — DONE ✅ (fixed 2026-03-24)

| # | Status | File | Fix |
|---|--------|------|-----|
| DC.1 | DONE ✅ | `olang_handbook.md` | WhileStmt → 5 fields (cond, body, cond_start, cond_end, tokens) |
| DC.2 | DONE ✅ | `CLAUDE.md` | ARRAY_INIT_CAP = 4096, ArrayLit no pre-allocate |
| DC.3 | DONE ✅ | verified | r14 grows DOWN confirmed in ASM (CLAUDE.md was correct) |
| DC.4 | DONE ✅ | `CLAUDE.md` | `generate(state.ops)` → direct emission vào `_g_output` |
| DC.5 | DONE ✅ | `CLAUDE.md` | Save/restore table: +7 sites (Call args, LetStmt, ElseIf, WhileStmt×2, FieldAssign, Parser while) |
| DC.6 | DONE ✅ | `CLAUDE.md` | Two-pass → direct emission + backpatch |
| DC.7 | DONE ✅ | `CLAUDE.md` | `a[i]` noted as desugar to `__array_get(a, i)` |
| DC.8 | DONE ✅ | `CLAUDE.md` | Binary size 806KB → ~824KB |

### Docs Conflicts — DC.9-DC.20 DONE ✅ (fixed 2026-03-24 Nox)

| # | Status | Fix |
|---|--------|-----|
| DC.9 | DONE ✅ | LOC counts updated: lexer 258, parser 952, semantic 1244, codegen 429, repl 117, VM 5031 |
| DC.10 | DONE ✅ | Expr 6→17 variants, Stmt 8→17 variants — all AST nodes in handbook |
| DC.11 | DONE ✅ | HomeOS stdlib 40 files, 7,304 LOC |
| DC.12 | DONE ✅ | "Chưa port" → "Port status" — OL.1-5 marked DONE |
| DC.13 | DONE ✅ | Binary ~824KB → ~861KB |
| DC.14 | DONE ✅ | Header updated |
| DC.15 | DONE ✅ | MolLiteral { packed: Num } |
| DC.16 | DONE ✅ | Added __dict_new, __array_new, __throw, __floor, __ceil, __sha256 |
| DC.17 | DONE ✅ | Dict + interpolation + comprehension + try/catch examples |
| DC.18 | DONE ✅ | __sha256 documented |
| DC.19 | DONE ✅ | $"hello {name}" documented |
| DC.20 | DONE ✅ | Binary size updated |

### Docs Conflicts — DC.21-DC.27 DONE ✅ (fixed 2026-03-24 Nox, pre-T4)

| # | Status | Fix |
|---|--------|-----|
| DC.21 | DONE ✅ | LOC updated: repl 131, homeos 7,832 |
| DC.22 | DONE ✅ | Binary ~891KB |
| DC.23 | DONE ✅ | Opcodes: 18 entries (was 13). Added TryBegin/CatchEnd/StoreUpdate/PushMol/CallClosure |
| DC.24 | DONE ✅ | Builtins: ~54 documented. Added __cmp_ne, bit ops, logic_not, array_pop/range, type_of, etc |
| DC.25 | DONE ✅ | REPL commands section: encode, respond, learn, memory, help |
| DC.26 | DONE ✅ | Memory systems section: STM, Silk, Dream, Knowledge documented |
| DC.27 | DEFERRED | PLAN_REWRITE.md — will update at T4 start |

### Docs Conflicts — DC.28-DC.32 DONE ✅ (fixed 2026-03-24 Nox)

| # | Status | Fix |
|---|--------|-----|
| DC.28 | DONE ✅ | Binary ~877KB (897,628 bytes) |
| DC.29 | DONE ✅ | Opcode table reorganized, no duplicates |
| DC.30 | DONE ✅ | HomeOS stdlib 7,701 LOC |
| DC.31 | DONE ✅ | ARRAY_INIT_CAP = 16384 documented |
| DC.32 | DONE ✅ | LOC: parser 974, semantic 1301, repl 160 |

### Pre-T4 Blockers — ALL DONE ✅

| # | Status | Fix |
|---|--------|-----|
| BLOCK.1 | **DONE** ✅ | Match patterns: numbers, strings, wildcards. Pre-emit Jmp pattern. |
| BLOCK.2 | **DONE** ✅ | _g_output 4096→8192, ARRAY_INIT_CAP 4096→8192. 8KB bytecode. |
| BLOCK.3 | **DONE** ✅ | `test` REPL command, 12/12 inline tests. assert_eq framework. |

### Docs Conflicts — Mới (phát hiện 2026-03-24 inspect #10)

| # | Mức độ | File | Xung đột |
|---|--------|------|----------|
| DC.33 | DONE ✅ | VM 5,471 LOC |
| DC.34 | DONE ✅ | repl 243 LOC, homeos 7,838 LOC |
| DC.35 | DONE ✅ | Binary ~890KB |
| DC.36 | DONE ✅ | File I/O + bytes + heap builtins documented |
| DC.37 | DONE ✅ | All REPL commands documented |
| DC.38 | DONE ✅ | Natural language mode documented |
| DC.39 | DONE ✅ | repl 304 LOC updated |
| DC.40 | DONE ✅ | lexer 259, parser 975, semantic 1315 updated |
| DC.41 | DONE ✅ | homeos 7,992 LOC updated |
| DC.42 | DONE ✅ | Binary ~901KB updated |
| DC.43 | DONE ✅ | Tests 14 updated |
| DC.44 | DONE ✅ | Phase 5 section added to CLAUDE.md |
| DC.45 | DONE ✅ | Auto-learn documented in Phase 5 section |
| DC.46 | DONE ✅ | repl 322 LOC updated |
| DC.47 | DONE ✅ | homeos 43 files, 8,910 LOC updated |
| DC.48 | DONE ✅ | Binary ~928KB (950,469 bytes) updated |
| DC.49 | DONE ✅ | Tests 16 updated |
| DC.50 | DONE ✅ | Phase 5 fully documented: alias, node, UDC decode, UTF-8, emoji, stemming, digest, DN/QR |

### Docs Conflicts — DC.51-DC.61 (phát hiện 2026-03-24 inspect #15)

| # | Mức độ | File | Xung đột |
|---|--------|------|----------|
| DC.51 | DONE ✅ | Binary ~943KB (965,292B) updated |
| DC.52 | DONE ✅ | lexer 298 LOC updated |
| DC.53 | DONE ✅ | parser 1,136 LOC updated |
| DC.54 | DONE ✅ | semantic 1,336 LOC updated |
| DC.55 | DONE ✅ | repl 343 LOC updated |
| DC.56 | DONE ✅ | VM 5,634 LOC (already correct) |
| DC.57 | DONE ✅ | HomeOS 44 files, 9,416 LOC updated |
| DC.58 | DONE ✅ | Bootstrap 3,542 LOC (lexer+parser+semantic+codegen) |
| DC.59 | DONE ✅ | Hex 0xFF documented in cú pháp |
| DC.60 | DONE ✅ | ^ (XOR) in precedence table |
| DC.61 | DONE ✅ | bare return; documented |

### Docs Conflicts — DC.62-DC.74 (phát hiện 2026-03-25 inspect #17)

| # | Mức độ | File | Xung đột | Status |
|---|--------|------|----------|--------|
| DC.62 | DONE ✅ | CLAUDE.md | parser.ol 1,136 → 1,132 LOC |
| DC.63 | DONE ✅ | CLAUDE.md | semantic.ol 1,336 → 1,569 LOC (+233) |
| DC.64 | DONE ✅ | CLAUDE.md | repl.ol 343 → 355 LOC |
| DC.65 | DONE ✅ | CLAUDE.md | VM 5,634 → 5,767 LOC (+133) |
| DC.66 | DONE ✅ | CLAUDE.md | HomeOS 9,416 → 9,559 LOC |
| DC.67 | DONE ✅ | CLAUDE.md | Binary ~943KB → ~961KB |
| DC.68 | DONE ✅ | CLAUDE.md | Lambda syntax + AST added |
| DC.69 | DONE ✅ | CLAUDE.md | map/filter/reduce/any/all documented |
| DC.70 | DONE ✅ | CLAUDE.md | __mol_s/r/v/a/t + __mol_pack documented |
| DC.71 | DONE ✅ | CLAUDE.md | T5 ND.4 fn_node API noted in Phase 5 |
| DC.72 | DONE ✅ | CLAUDE.md | eval_bc_base cross-boundary mechanism |
| DC.73 | DONE ✅ | TASKBOARD | Header stats updated |
| DC.74 | DONE ✅ | CLAUDE.md | Tests 16→20 updated |

### Docs Conflicts — DC.75-DC.79 (phát hiện 2026-03-25 inspect #18)

| # | Mức độ | File | Xung đột | Status |
|---|--------|------|----------|--------|
| DC.75 | DONE ✅ | CLAUDE.md | semantic.ol 1,569 → 1,594 LOC |
| DC.76 | DONE ✅ | CLAUDE.md | HomeOS 9,559 → 9,591 LOC |
| DC.77 | DONE ✅ | CLAUDE.md | Binary ~961KB → ~963KB |
| DC.78 | DONE ✅ | CLAUDE.md | pipe() builtin documented |
| DC.79 | DONE ✅ | CLAUDE.md | LG.1/LG.2/LG.5 in Phase 5 section |

### BUG-INDEX / BUG-SORT — FIXED ✅ (2026-03-25 Nox)

```
Root cause: parser.ol desugar a[expr] dùng ArrayLit [obj, idx] (qua __array_new).
  __array_new(2) tạo array KHÔNG có capacity dự phòng.
  Heap overlap khiến element[1] (index expr) bị mất → luôn trả a[0].
  BUG-SORT là hệ quả: bubble sort dùng a[j+1] → luôn so sánh/swap với a[0].
Fix: đổi sang push-based array (giống handler "(" call args).
  push() dùng capacity zone đã pre-allocate → an toàn.
Verify: [5,2,8,1,9] → [1,2,5,8,9] ✅, 16/16 tests, fib(20)=6765.
```

### Spec v3 vs Code (architecture gap — INFO level)

| # | Spec Section | Status | Notes |
|---|-------------|--------|-------|
| SC.1 | SecurityGate 3-layer | ✅ Done | 12 patterns (VI+EN), alias-normalized, inline matching |
| SC.2 | Fusion (multi-modality) | ❌ Not impl | text-only for now |
| SC.3 | 7 Instincts | ❌ Not impl | Honesty, Contradiction, Causality, etc. |
| SC.4 | Immune Selection N=3 | ❌ Not impl | single-branch inference |
| SC.5 | Homeostasis (Free Energy) | ❌ Not impl | no F tracking |
| SC.6 | DNA Repair (self_correct) | ❌ Not impl | no critique loop |
| SC.7 | KnowTree hierarchical | ✅ UDC chain | dual search: molecule similarity + keyword |
| SC.8 | UDC + P_weight encoding | ✅ Correct | block ranges + bit layout |
| SC.9 | Compose (amplify V) | ✅ Done | amplify_v: base + sign × boost (Spec §1.6) |
| SC.10 | Compose S: Union | ✅ Done | _union_s: SDF union (max complexity) |
| SC.11 | Compose R: Compose | ✅ Done | _compose_r: same→keep, diff→max |
| SC.12 | Hebbian Select | ✅ Done | silk_decay φ⁻¹ every 3 turns, prune < 0.01 |
| SC.13 | Dream pipeline | ✅ Done | strengthen dominant + decay weak (consolidation) |
| SC.14 | MolecularChain | ✅ Correct | u16 molecules |
| SC.15 | 10-stage pipeline | ✅ NEW | alias→UDC→node→DN/QR→decode→emoji→output |
| SC.16 | 5 Checkpoints | ❌ 0/5 | Pipeline không có checkpoint nào |

### Docs vs Docs conflicts

| # | Mức độ | Files | Xung đột |
|---|--------|-------|----------|
| DOC.1 | NGHIÊM TRỌNG | STORAGE_NOTE vs Spec v3 | P_weight: 5 bytes vs 2 bytes (u16) |
| DOC.2 | TRUNG BÌNH | MILESTONE vs reality | ARRAY_CAP 4096→16384, heap 64→256MB, binary 806→949KB |
| DOC.3 | TRUNG BÌNH | Handbook vs CLAUDE.md | Opcodes thiếu, binary "806KB" outdated |
| DOC.4 | NHẸ | CHECK_TO_PASS | check-logic tool (Rust) = dead code |

### Architecture Gap: "Mọi thứ = Node" (Spec v3 §II, §III)

```
CRITICAL GAP: Code hiện tại chỉ tạo node cho user input text.
Theo Spec v3, MỌI THỨ phải là node:

  fn = node { dn, mol, body: chain_of_nodes }
  skill = node { children: [node(fn), node(fn)...] }
  code = chain of instruction nodes
  variable = node { dn, value, mol }

Hệ quả:
  ① Fn có cảm xúc (mol) — heal() V=6, delete() V=2
  ② Fn có links — add↔subtract, parse↔tokenize
  ③ Fn có fire_count — hot function = promote QR
  ④ Fn có maturity — new=Evaluating, stable=Mature
  ⑤ Skill = composite node — Dream cluster fn → skill
  ⑥ Code = self-describing chain — inspect, compose, splice
  ⑦ Gene = data, data = gene — giống DNA

Hiện tại: fn = VM closure (bytecode blob), var = flat hash entry.
Target:   fn = node trong KnowTree, skill = tree of fn nodes.

KEY INSIGHT (từ spec): KHÔNG BAO GIỜ thiếu context/dung lượng vì:
  - Mọi thứ = u16 links (2 bytes) — 1 sách = 3.4KB
  - Text tự tách: đoạn → câu → từ → UDC index (2B mỗi từ)
  - Công thức toán = ĐÃ NẰM TRONG UDC (8,846 SDF, 18KB) — chỉ gọi index
  - Fn = chain of UDC instructions, gọi = traverse chain
  - Silk = implicit từ THỨ TỰ trong chain (0 bytes)
  - Context vô hạn = chỉ cần thêm links (2B mỗi cái)

VI PHẠM hiện tại:
  - Knowledge lưu nguyên string (10KB/fact) thay vì UDC chain (vài B)
  - learn_file không tách đoạn→câu→từ
  - STM max 32, Knowledge max 512 = giới hạn nhân tạo
  - Fn = opaque bytecode blob, không phải inspectable chain
  - Silk = explicit bigrams thay vì implicit chain order
```

---

## Log

```
2026-03-18  Tạo TASKBOARD. Rust era bắt đầu.
2026-03-19  Phase 0-9 ALL DONE. origin.olang 1.35MB.
2026-03-21  V2 Migration + Phase 14-16 DONE. Tier 1 planned.
2026-03-22  VM optimization 3.7x. Native binary boots. 806KB.
2026-03-23  SELF-HOSTING. fib(20)=6765. 30+ bugs fixed. 27/27 tests.
2026-03-23  Rust archived. Olang era begins.
2026-03-23  OL.6 for-in loops + range(). OL.7 smart string concat. PR #311 #312.
2026-03-23  OL.7c else-if. OL.7d pretty-print arrays. OL.7e var assign fix. OL.7f audit. PR #317-321.
2026-03-23  Nested if-else fixed (parser var save). Variable accumulation in loops fixed.
2026-03-23  break/continue for loops. Unary minus. FieldAssign audit. PR #323-324.
2026-03-23  a[i] desugar, nested Call fix, nested while fix (re-parse). PR #330-335.
2026-03-23  BUBBLE SORT on native binary: [5,2,8,1,9] → [1,2,5,8,9]. 64MB heap.
2026-03-24  Heap optimize. map/filter/reduce. Primes+sort+sum in 1 program. PR #337.
2026-03-24  28 PRs in 1 session. Olang = functional programming language. 844KB.
2026-03-24  Kira inspector: 8 docs conflicts found (DC.1-DC.8). CLAUDE.md + handbook lỗi thời.
2026-03-24  DC.1-DC.8 ALL FIXED. CLAUDE.md + handbook synced with code.
2026-03-24  BUG #1 FIXED: dict literal { key: value } + parse error recovery. 21/21 Kira tests pass.
2026-03-24  Inspect #2: 5/5 tests PASS. DC.1-DC.8 confirmed FIXED. 3 new minor conflicts (DC.9-DC.11).
2026-03-24  OL.9 DONE: try/catch + __throw(msg). VM try_stack, nested try, unhandled error exit.
2026-03-24  BUG-1 FIXED: nested for-in works! [11,21,12,22] + 3×3=9 results.
2026-03-24  BUG-2 FIXED: bare assignment (x = x + 1). Match binding save before parse_expr.
2026-03-24  BUG-5 FIXED: while accumulator (s = s + i → 3). Consequence of BUG-2 fix.
2026-03-24  BUG-3 PARTIAL: type/union semicolons fixed. Match on union still segfaults (heap overlap).
2026-03-24  BUG-4 NOT REPRODUCED: string concat in fn works on current binary.
2026-03-24  Inspect #3: 7/7 tests PASS (incl. comprehension + try/catch). 9 new conflicts DC.9-DC.17 (4 NGHIÊM TRỌNG).
2026-03-24  OL.7b DONE: string interpolation $"hello {name}". OL.13 DONE: SHA-256 in ASM (~250 LOC).
2026-03-24  Inspect #4: 8/8 tests PASS (incl. interpolation + SHA-256). DC.18-DC.20 new. Binary 881KB.
2026-03-24  OL.1-5 DONE: Full intelligence layer (encode→analyze→intent→respond).
2026-03-24  OL.8 DONE: Boot/eval closure bridge. REPL calls stdlib functions.
2026-03-24  OL.12 DONE: WASM VM works (emit 42, emit 1+2). OL.14 DONE: Browser demo.
2026-03-24  OL.11 WIP: ARM64 boots bare via QEMU. Needs builtins for stdlib.
2026-03-24  DC.9-DC.20 ALL FIXED. CLAUDE.md + handbook fully synced.
2026-03-24  STM + Silk + Dream + Knowledge learning. Nó nhớ. Nó biết sách. 891KB.
2026-03-24  Inspect #5 (pre-T4): 13/13 tests PASS. 7 new conflicts DC.21-DC.27. 3 blockers identified.
2026-03-24  DC.21-DC.26 FIXED by Nox. DC.27 DEFERRED.
2026-03-24  Inspect #6: 10/10 PASS. DC.28-DC.30 new (minor: KB unit error, opcode dup, LOC drift).
2026-03-24  BLOCK.1 DONE: match patterns (num/str/wildcard). BLOCK.2 DONE: 8KB output. BLOCK.3 DONE: test 12/12.
2026-03-24  Inspect #7: 9/9 PASS + test 12/12. All blockers DONE. DC.31-DC.32 new. READY FOR T4!
2026-03-24  ARRAY_INIT_CAP 8192→16384 (16KB bytecode). DC.31 updated.
2026-03-24  Inspect #8: 7/7 PASS + test 12/12. DC.31 now 4096→16384 (4x). No new issues.
2026-03-24  DC.28-DC.32 ALL FIXED by Nox. 32/32 docs conflicts resolved. ZERO open conflicts.
2026-03-24  Inspect #9: 6/6 PASS + test 12/12. ALL 32 DCs verified. Docs 100% synced. GREEN LIGHT T4.
2026-03-24  T4: File I/O builtins (__file_read/write). Self-compile pipeline. Comparison fix (f64 0.0).
2026-03-24  CUT.4 DONE: SELF-BUILD WORKS! build → 381KB binary. fib(20)=6765. Recursive verified.
2026-03-24  Inspect #10: 6/6+test 12/12+self-build. CUT.1-4 ALL DONE. DC.33-DC.36 new. Binary 909KB.
2026-03-24  feat: learn_file command, natural language mode (auto-detect code vs text), 911KB.
2026-03-24  Inspect #11: 7/7+test 12/12+NL mode. DC.37-DC.38 new (REPL commands, NL mode). repl 243 LOC.
2026-03-24  Phase 5: word affect 72w, personality templates, context window. Training data 661 entries.
2026-03-24  REPL fix: << >> && in boot code crashed Rust compiler. Tests 12→14. Auto-learn 166 facts.
2026-03-24  Inspect #12: 8/8+test 14/14. DC.33-38 FIXED. DC.39-DC.45 new (LOC drift, Phase 5 undoc). 923KB.
2026-03-24  DC.39-DC.45 ALL FIXED. CLAUDE.md fully synced: LOC, binary, tests, Phase 5.
2026-03-24  Inspect #13: 6/6+test 14/14. ZERO CONFLICTS. 45/45 DCs resolved. Docs 100% synced. 923KB.
2026-03-24  P5.2: emotion carry-over + Vietnamese stemming + digest + UTF-8 decoder.
2026-03-24  P5.3: full pipeline (alias→UDC→node→DN/QR→decode→emoji). 3 new files. 949KB.
2026-03-24  Inspect #14: DC.46-50 found (LOC drift, Phase 5 undoc). Spec v3 gap analysis. BUGFIX_ARCHITECTURE.md.
2026-03-24  BUG FIX: nested struct/dict/enum save-restore (5 sites). lexer.ol compiles! 1.0s.
2026-03-24  DC.46-DC.50 ALL FIXED. CLAUDE.md + TASKBOARD synced. 950KB. 16/16 tests.
2026-03-24  Inspect #14: 8/8+test 16/16. DC.46-50 + SC.1-16 + DOC.1-4. Deep Spec v3 audit.
2026-03-24  SC.9 FIXED: amplify_v Spec §1.6. SC.7: UDC chain knowledge. SC.10-11: Union/Compose.
2026-03-24  SC.1: SecurityGate 12 patterns. SC.12: Silk decay φ⁻¹. SC.13: Dream consolidation.
2026-03-24  __array_with_cap builtin: explicit capacity, fix token corruption from relocation.
2026-03-24  STREAMING COMPILER: parse+compile one stmt at a time. ALL 4 bootstrap files compile!
            lexer 1.9s, codegen 2s, parser 2.7s, semantic 3s. ZERO segfaults. 957KB.
2026-03-24  Kira: __sleep(ms) + __time() + __write_raw(). terminal.ol 284 LOC (ANSI animations). PR #404.
2026-03-24  Nox: 100% SELF-COMPILE (48/48). Hex literals, match-as-var, lambda skip, keyword dict fields.
            Parser 988→1136 LOC. Lexer 262→298 LOC. Binary 957KB→964KB.
2026-03-24  Inspect #15: 4/5 PASS. BUG-SORT REGRESSION (bubble sort broken). DC.51-61. Binary 964,642B.
2026-03-24  Sora: BUG-VI fixed (7 fn prefix rename). DC.51-61 ALL FIXED. Binary 965,292B.
2026-03-24  Inspect #16: 4/5 PASS. ROOT CAUSE: a[expr] BinOp → a[0]. Docs 100% synced. ZERO new DC.
2026-03-25  Nox: BUG-INDEX/BUG-SORT FIXED — ArrayLit→push in parser.ol. [1,2,5,8,9] ✅. 16/16 tests.
2026-03-25  Nox: Lambda expressions + closure calls. fn(x){body} parsed+compiled. Higher-order functions.
2026-03-25  Nox: Inline map/filter/reduce compiler builtins. Functional pipeline works.
2026-03-25  Nox: any/all builtins. Test suite 16→20. Phase 5A COMPLETE.
2026-03-25  Nox: T5 Layer 1 — BUG-KNOWLEDGE fixed (5D mol, all-chars chain, additive keyword×5).
            Knowledge retrieval correct: Einstein→Einstein, Vietnam→Vietnam.
            Instinct: [fact/opinion/hypothesis] labels. Curiosity. r_dispatch + temporal_tag.
2026-03-25  Nox: T5 ND.2 mol ASM builtins (__mol_s/r/v/a/t + __mol_pack). 100x faster extract.
            ND.4 fn_node metadata registry (register/fire/link/hot). Phase 5B core DONE.
2026-03-25  Kira: Inspect #17 — 10/10 PASS (5 core + 4 new HOF + test 20/20).
            DC.62-74 found+fixed (LOC drift, lambda/HOF/mol undocumented). 961KB.
2026-03-25  Nox: T5 LG.1 fn auto-register + LG.5 fn self-describe (lazy mol + 5D metadata).
2026-03-25  Nox: T5 LG.2 pipe() Lego operator. fn{fn{...}}==fn. Data flows through node chain.
2026-03-25  Kira: Inspect #18 — 7/7 PASS (incl. pipe). DC.75-79 found+fixed. 963KB.
```
