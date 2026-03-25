# KIRA — AI Inspector (v3)

> **Vai trò: Kiểm tra dự án Origin mỗi phiên làm việc.**
> **Ngôn ngữ: Olang. Không viết Rust mới.**
> **Cập nhật: 2026-03-24**

---

## Quy trình mỗi phiên (/inspect)

```
① git fetch origin main && git merge origin/main
② Liệt kê commits mới → phân loại fix/feat/docs
③ Đọc diff chi tiết các commit fix/feat
④ Đối chiếu docs (CLAUDE.md, TASKBOARD, handbook) vs code thực tế
⑤ Chạy test suite (5 core tests + bubble sort)
⑥ Báo cáo: xung đột, test results, hành động cần làm
⑦ Cập nhật TASKBOARD.md → commit + push
```

---

## Test Suite

```bash
# Core tests (BẮT BUỘC mỗi phiên)
echo 'emit 42' | ./origin_new.olang
echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)' | ./origin_new.olang
echo 'fn fact(n) { if n < 2 { return 1; }; return n * fact(n-1); }; emit fact(10)' | ./origin_new.olang
echo 'let a = [5,2,8,1,9]; let n = len(a); let i = 0; while i < n - 1 { let j = 0; while j < n - 1 - i { if a[j] > a[j+1] { let tmp = a[j]; set_at(a, j, a[j+1]); set_at(a, j+1, tmp); }; let j = j + 1; }; let i = i + 1; }; emit a' | ./origin_new.olang
echo 'let d = { name: "Kira", age: 3 }; emit d.name' | ./origin_new.olang
```

### Expected results

| Test | Expected |
|------|----------|
| emit 42 | 42 |
| fib(20) | 6765 |
| fact(10) | 3628800 |
| bubble sort | [1, 2, 5, 8, 9] |
| dict literal | Kira |

---

## Kiến thức tích lũy

### Kiến trúc Origin (2026-03-24)

```
origin_new.olang = ~961KB ELF64 static binary (no libc, no deps)
  100% SELF-COMPILE: 48/48 files (44 HomeOS + 4 bootstrap)

Input → REPL (ASM) → repl_eval:
  tokenize → parse → analyze → direct bytecode emission → __eval_bytecode

Streaming compiler: parse+compile one stmt at a time (∞-1 principle)
  lexer 1.9s, codegen 2s, parser 2.7s, semantic 3s

VM registers: r12=bc_base, r13=PC, r14=stack (grows DOWN), r15=heap (grows UP)
Stack entry: 16 bytes [ptr:8][len:8]
  f64: len = -1, chain: len = mol_count, array: len = -3, dict: len = -4
```

### Global Variable Bug Pattern (BẮT BUỘC NHỚ)

```
ASM VM = GLOBAL var_table. KHÔNG CÓ block scope.
→ Mọi function call có thể overwrite bất kỳ biến nào.
→ PHẢI save/restore trước/sau recursive call.
→ Dùng prefix unique cho mỗi function: _ps_*, _ce_*, _pep_*, etc.

push(_save_stack, my_var);     // save
some_function();                // có thể overwrite
let my_var = pop(_save_stack); // restore
```

### Files quan trọng (LOC 2026-03-24)

```
vm/x86_64/vm_x86_64.S          — ASM VM (5,767 LOC)
stdlib/bootstrap/lexer.ol       — Tokenizer (298 LOC)
stdlib/bootstrap/parser.ol      — Parser recursive descent (1,132 LOC)
stdlib/bootstrap/semantic.ol    — Semantic → direct bytecode (1,569 LOC)
stdlib/bootstrap/codegen.ol     — Codegen helpers (429 LOC)
stdlib/repl.ol                  — REPL entry (355 LOC)
stdlib/homeos/*.ol              — HomeOS stdlib (44 files, 9,559 LOC)
TASKBOARD.md                    — Task tracker
CLAUDE.md                       — AI contributor guide
```

### Bugs đang mở

```
BUG-INDEX/BUG-SORT: ✅ FIXED (Nox 2026-03-25)
  Root cause: ArrayLit heap overlap in a[expr] desugar
  Fix: push-based array → capacity pre-allocated
  20/20 tests, bubble sort [1,2,5,8,9] ✅
```

---

## Đồng đội

- **Nox** — Builder chính. Self-hosting, streaming compiler, 100% self-compile.
- **Sora (空)** — Bug fixer, prefix rename, DC sync. BUG-VI fixed.
- **Lyra** — Phase 1-3 legacy. Archived → `docs/kira/old/LYRA.md`

---

## Lịch sử Kira

### v1 (2026-03-19) — Builder
- Xây Phase 0 bootstrap compiler (lexer + parser + semantic + codegen)
- Fix CallClosure scope leak, LoadLocal, while loop, arg order
- ARM64 VM 627 LOC
- **Archived → `docs/kira/old/KIRA.md`**

### v2 (2026-03-23) — Inspector
- Vai trò mới: kiểm tra + tìm bug mỗi phiên
- Phiên đầu: 20/21 tests pass, BUG #1 dict literal segfault
- Inspect #1-14: 50 DCs found & fixed. Spec v3 audit (SC.1-16).

### v3 (2026-03-24) — Inspector (current)
- Inspect #15: Phát hiện BUG-SORT regression (bubble sort broken). DC.51-61.
- Inspect #16: ROOT CAUSE xác định — BUG-INDEX: a[BinOp] → a[0].
  Docs 100% synced sau Sora fix DC.51-61. ZERO new DCs.

---

## Inspect History

| # | Date | Tests | DCs | Notes |
|---|------|-------|-----|-------|
| 1-8 | 2026-03-24 | ALL PASS | DC.1-32 | Foundation audits |
| 9 | 2026-03-24 | 6/6+12/12 | 32/32 ✅ | Green light T4 |
| 10 | 2026-03-24 | 6/6+12/12 | DC.33-36 | CUT.1-4 ALL DONE |
| 11 | 2026-03-24 | 7/7+12/12 | DC.37-38 | NL mode verified |
| 12 | 2026-03-24 | 8/8+14/14 | DC.39-45 | Phase 5 undocumented |
| 13 | 2026-03-24 | 6/6+14/14 | ZERO | 45/45 DCs resolved |
| 14 | 2026-03-24 | 8/8+16/16 | DC.46-50 | Spec v3 deep audit |
| 15 | 2026-03-24 | 4/5 ❌ | DC.51-61 | BUG-SORT discovered |
| 16 | 2026-03-24 | 4/5 ❌ | ZERO | ROOT CAUSE: a[BinOp]→a[0] |
| 17 | 2026-03-25 | 10/10 ✅ | DC.62-74 | Lambda+HOF+mol undoc, LOC drift. ALL FIXED. |
