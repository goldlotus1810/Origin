# TASKBOARD — Origin / Olang

> **Mọi AI session đọc file này TRƯỚC KHI bắt đầu làm việc.**
> **Viết OLANG. Rust legacy chỉ bug fix.**
> **Chi tiết lịch sử:** [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Trạng thái: FULL STACK (2026-03-24)

```
origin_new.olang = ~871KB native binary (891,374 bytes)
  ✅ Bootstrap compiler: lexer + parser + semantic + codegen (2,883 LOC Olang)
  ✅ Intelligence layer: encode + analyze + intent + respond (OL.1-5)
  ✅ Crypto: SHA-256 FIPS 180-4 in ASM
  ✅ WASM: runs in browser (3KB)
  ✅ OL.8: REPL calls stdlib functions (boot/eval closure bridge)
  ✅ fib(20) = 6,765 | __sha256("abc") = ba7816bf...
  ✅ ASM VM x86_64 (5,031 LOC), no libc, zero dependencies
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

---

## Dependency Graph

```
            ┌────────────────────────────────┐
            │ TIER 1 — Intelligence Layer    │
            │ OL.1 Encoder                   │
            │ OL.2 Analysis                  │
            │ OL.3 Intent    → OL.5 Response │
            │ OL.4 Agents                    │
            └────────────┬───────────────────┘
                         │
            ┌────────────▼───────────────────┐
            │ TIER 2 — Language Features     │
            │ OL.6-10 (song song)            │
            └────────────┬───────────────────┘
                         │
            ┌────────────▼───────────────────┐
            │ TIER 3 — Platform              │
            │ OL.11 ARM64 │ OL.12 WASM       │
            │ OL.13 Crypto│ OL.14-15 Browser │
            └────────────┬───────────────────┘
                         │
            ┌────────────▼───────────────────┐
            │ TIER 4 — Cắt dây rốn           │
            │ CUT.1 → CUT.2 → CUT.3 → CUT.4 │
            └────────────────────────────────┘
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
| DC.46 | OPEN | `CLAUDE.md:391` — repl 304→322 LOC |
| DC.47 | OPEN ⚠️ | `CLAUDE.md:392` — homeos 40→**43 files**, 7,992→**8,910** LOC (+918!) |
| DC.48 | OPEN | `CLAUDE.md:37,378` — Binary ~901KB→~927KB (949KB) |
| DC.49 | OPEN | `CLAUDE.md:332` — Tests 14→**16** |
| DC.50 | OPEN | Phase 5 thiếu: alias, node, UDC decode, UTF-8, emo carry-over, stemming, digest |

### Spec v3 vs Code (architecture gap — INFO level)

| # | Spec Section | Status | Notes |
|---|-------------|--------|-------|
| SC.1 | SecurityGate 3-layer | ⚠️ Partial | Code: 2 keywords. Spec: Bloom + normalized + semantic |
| SC.2 | Fusion (multi-modality) | ❌ Not impl | text-only for now |
| SC.3 | 7 Instincts | ❌ Not impl | Honesty, Contradiction, Causality, etc. |
| SC.4 | Immune Selection N=3 | ❌ Not impl | single-branch inference |
| SC.5 | Homeostasis (Free Energy) | ❌ Not impl | no F tracking |
| SC.6 | DNA Repair (self_correct) | ❌ Not impl | no critique loop |
| SC.7 | KnowTree hierarchical | ❌ Flat array | spec says L0→L3 tree |
| SC.8 | UDC + P_weight encoding | ✅ Correct | block ranges + bit layout |
| SC.9 | Compose (amplify V) | ❌ **BUG-V** | `_amplify_v()` dùng TRUNG BÌNH — vi phạm Spec §1.6! |
| SC.10 | Compose S: Union | ⚠️ max() | Code dùng max() thay vì SDF union |
| SC.11 | Compose R: Compose | ⚠️ average | Code dùng (ra+rb)/2 — spec nói Compose |
| SC.12 | Hebbian Select | ⚠️ No decay | silk_co_activate OK, nhưng thiếu decay φ⁻¹ |
| SC.13 | Dream pipeline | ✅ Partial | dream_cycle scan STM, nhưng thiếu Maturity pipeline |
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
2026-03-24  Inspect #14: 8/8+test 16/16. DC.46-50 + SC.1-16 + DOC.1-4. Deep Spec v3 audit.
            BUG-V CRITICAL: _amplify_v() dùng TRUNG BÌNH — vi phạm Spec §1.6 + CHECK_TO_PASS §1!
            5 Checkpoints: 0/5 implement. SecurityGate: 2 keywords vs spec 3-layer. Hebbian: no decay.
            DOC.1: STORAGE_NOTE P_weight 5B vs Spec+code 2B. DOC.2: MILESTONE outdated.
```
