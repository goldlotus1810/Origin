# TASKBOARD — Origin / Olang

> **Mọi AI session đọc file này TRƯỚC KHI bắt đầu làm việc.**
> **Viết OLANG. Rust legacy chỉ bug fix.**
> **Chi tiết lịch sử:** [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Trạng thái: SELF-HOSTING (2026-03-23)

```
origin_new.olang = 806KB native binary
  ✅ Bootstrap compiler: lexer.ol + parser.ol + semantic.ol + codegen.ol
  ✅ fib(20) = 6,765 | fact(10) = 3,628,800
  ✅ 27/27 REPL tests pass
  ✅ ASM VM x86_64, no libc, zero dependencies
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
| OL.1 | Encoder: text → molecule (.ol) | ~300 LOC | FREE | Port agents/encoder. Word → 5D coordinate. |
| OL.2 | Analysis: sentence fusion (.ol) | ~400 LOC | FREE | Port context/analysis. Emotion composition. |
| OL.3 | Intent inference engine (.ol) | ~300 LOC | FREE | Port intent estimation. Context-aware. |
| OL.4 | Agents: flesh out stubs (.ol) | ~500 LOC | FREE | chief.ol, leo.ol, worker.ol, gate.ol — real logic. |
| OL.5 | Response composer (.ol) | ~200 LOC | FREE | Emotion-aware, context-aware response generation. |

### Tier 2 — Mở rộng ngôn ngữ

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| OL.6 | For-in loops + range() | ~120 LOC | DONE ✅ | `for x in items { }`, `for i in range(n) { }`. PR #311. |
| OL.7 | Smart string concat | ~100 LOC | DONE ✅ | `"Age: " + 25` → auto-convert. `__to_string` builtin. PR #312. |
| OL.7b | String interpolation `"hello {name}"` | ~200 LOC | FREE | Blocked: lexer self-compile conflict with `{` in source. Needs f-string prefix. |
| OL.7c | Else-if chains | ~20 LOC | DONE ✅ | `else if` syntax. Parser var save/restore. PR #317. |
| OL.7d | Pretty-print arrays | ~80 LOC | DONE ✅ | `emit [1,2,3]` instead of `[array 3]`. PR #318. |
| OL.7e | Variable assignment fix | ~5 LOC | DONE ✅ | `let b = b + a` now works. LetStmt name save. PR #320. |
| OL.7f | FieldAssign fix + audit | ~10 LOC | DONE ✅ | Full 18-site match binding audit. PR #321. |
| OL.8 | Import/module system | ~300 LOC | FREE | `use module.func` — resolve at compile time. |
| OL.9 | Error handling | ~200 LOC | FREE | `try { ... } catch { ... }` — parser + VM. |
| OL.10 | Array/Dict comprehension | ~150 LOC | FREE | `[x * 2 for x in items]` — sugar. |

### Tier 3 — Platform

| ID | Task | Effort | Status | Notes |
|----|------|--------|--------|-------|
| OL.11 | ARM64 ASM VM | ~2000 LOC | FREE | vm/aarch64/vm_aarch64.S. asm_emit_arm64.ol có sẵn. |
| OL.12 | WASM target | ~1000 LOC | FREE | Compile to WASM. wasm_emit.ol có sẵn. |
| OL.13 | Crypto in ASM | ~500 LOC | FREE | SHA-256, AES-256 in x86_64 assembly. |
| OL.14 | Browser E2E | ~500 LOC | FREE | origin.html + WASM binary. |
| OL.15 | Mobile (Android/iOS) | ~1000 LOC | FREE | ARM64 native + WASM iOS. |

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

## Docs Conflicts — Cần fix (phát hiện 2026-03-23)

> **Docs lỗi thời so với code thực tế. AI session mới đọc CLAUDE.md sẽ hiểu sai.**

| # | Mức độ | File | Xung đột |
|---|--------|------|----------|
| DC.1 | **NGHIÊM TRỌNG** | `olang_handbook.md:1443` | `WhileStmt { cond, body }` → thực tế có 5 field: `cond, body, cond_start, cond_end, tokens` |
| DC.2 | **NGHIÊM TRỌNG** | `CLAUDE.md:239` | `ARRAY_INIT_CAP = 256` → thực tế = **4096** (commit `4ea4d55`) |
| DC.3 | **NGHIÊM TRỌNG** | `CLAUDE.md:53` | `r14 = VM stack (grows DOWN)` → LYRA.md ghi `grows upward`. Cần kiểm ASM. |
| DC.4 | TRUNG BÌNH | `CLAUDE.md:47` | `generate(state.ops)` → thực tế direct bytecode emission vào `_g_output` |
| DC.5 | TRUNG BÌNH | `CLAUDE.md:205-212` | Bảng save/restore thiếu 4 vị trí mới: WhileStmt, FieldAssign, Call args loop |
| DC.6 | TRUNG BÌNH | `CLAUDE.md:248` | `Two-pass codegen` → thực tế direct emission + backpatch |
| DC.7 | NHẸ | `CLAUDE.md:136` | `a[i]` liệt kê như syntax gốc → thực tế desugar thành `__array_get(a, i)` |
| DC.8 | NHẸ | `CLAUDE.md:257` | Binary size `806KB` → có thể đã thay đổi sau 64MB heap + fixes |

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
```
