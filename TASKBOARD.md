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
| CUT.1 | Migrate Rust runtime → Olang | LỚN | FREE | emotion/silk/agents → .ol chạy trên ASM VM. |
| CUT.2 | Migrate Rust tools → Olang | LỚN | FREE | builder/server → .ol. builder.ol có sẵn. |
| CUT.3 | Olang test framework | LỚN | FREE | 2,348 Rust tests → Olang tests. |
| CUT.4 | Remove Rust dependency | — | BLOCKED | Khi CUT.1-3 xong. origin.olang = 1 file tự đủ. |

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

### Docs Conflicts — Mới (phát hiện 2026-03-24 inspect #6)

| # | Mức độ | File | Xung đột |
|---|--------|------|----------|
| DC.28 | TRUNG BÌNH | `CLAUDE.md:37,347` | Binary `~891KB` → thực tế ~877KB (897,628 bytes). Sai đơn vị |
| DC.29 | NHẸ | `CLAUDE.md:200,205` | Opcode 0x09 Jmp duplicate entry |
| DC.30 | NHẸ | `CLAUDE.md:373` | HomeOS stdlib 7,832 LOC → thực tế 7,701 LOC |
| DC.31 | **NGHIÊM TRỌNG** | `CLAUDE.md:290` | ARRAY_INIT_CAP = 4096 → thực tế **8192** (BLOCK.2 đã đổi) |
| DC.32 | TRUNG BÌNH | `CLAUDE.md:369-372` | LOC drift: parser 952→974, semantic 1244→1301, repl 131→160 |

### Pre-T4 Blockers — ALL DONE ✅

| # | Status | Fix |
|---|--------|-----|
| BLOCK.1 | **DONE** ✅ | Match patterns: numbers, strings, wildcards. Pre-emit Jmp pattern. |
| BLOCK.2 | **DONE** ✅ | _g_output 4096→8192, ARRAY_INIT_CAP 4096→8192. 8KB bytecode. |
| BLOCK.3 | **DONE** ✅ | `test` REPL command, 12/12 inline tests. assert_eq framework. |

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
```
