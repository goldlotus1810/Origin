# KIRA — AI Inspector (v2)

> **Vai trò: Kiểm tra dự án Origin mỗi phiên làm việc.**
> **Ngôn ngữ: Olang. Không viết Rust mới.**
> **Cập nhật: 2026-03-23**

---

## Quy trình mỗi phiên

```
① git fetch origin main && git merge origin/main
② make build
③ Chạy test suite (xem bên dưới)
④ Đọc TASKBOARD.md — kiểm tra task mới/thay đổi
⑤ Tìm bug mới → ghi vào BUG_REPORT.md
⑥ Debug nếu có thể → cập nhật cách xử lý
⑦ Commit + push BUG_REPORT.md lên main
```

---

## Test Suite

```bash
# Build
make build

# Core tests (BẮT BUỘC mỗi phiên)
echo 'emit 42' | ./origin_new.olang
echo 'emit "hello world"' | ./origin_new.olang
echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)' | ./origin_new.olang
echo 'fn fact(n) { if n < 2 { return 1; }; return n * fact(n-1); }; emit fact(10)' | ./origin_new.olang
echo 'let a = [5,2,8,1,9]; emit a' | ./origin_new.olang
echo 'emit "Age: " + 25' | ./origin_new.olang
echo 'emit 1 + 2 * 3' | ./origin_new.olang
echo 'if 1 > 2 { emit "a"; } else if 2 > 3 { emit "b"; } else { emit "c"; }' | ./origin_new.olang
echo 'emit -5' | ./origin_new.olang
echo 'fn add(a,b) { return a + b; }; emit add(add(1,2), add(3,4))' | ./origin_new.olang

# Array + loop tests
echo 'let a = []; emit a' | ./origin_new.olang
echo 'let a = [1,2,3]; emit a[2]' | ./origin_new.olang
echo 'for i in range(5) { emit i; }' | ./origin_new.olang
echo 'let i = 0; while i < 10 { if i == 5 { break; }; emit i; let i = i + 1; }' | ./origin_new.olang

# Higher-order + string tests
echo 'fn map_arr(a, f) { let out = []; let i = 0; while i < len(a) { push(out, f(a[i])); let i = i + 1; }; return out; }; fn double(x) { return x * 2; }; emit map_arr([1,2,3], double)' | ./origin_new.olang
echo 'let s = "hello"; emit len(s); emit char_at(s, 0)' | ./origin_new.olang
echo 'fn foo() { return 0; }; if foo() == 0 { emit "zero"; }' | ./origin_new.olang
echo 'for x in [1,2,3] { emit x; }' | ./origin_new.olang
echo 'emit 10 / 3' | ./origin_new.olang

# Bubble sort (integration test)
echo 'let a = [5,2,8,1,9]; let n = len(a); let i = 0; while i < n - 1 { let j = 0; while j < n - 1 - i { if a[j] > a[j+1] { let tmp = a[j]; set_at(a, j, a[j+1]); set_at(a, j+1, tmp); }; let j = j + 1; }; let i = i + 1; }; emit a' | ./origin_new.olang

# Known bugs (kiểm tra xem đã fix chưa)
# BUG #1: Dict literal → segfault
echo 'let d = { name: "Kira", age: 3 }; emit d.name' | ./origin_new.olang
```

### Expected results

| Test | Expected |
|------|----------|
| emit 42 | 42 |
| emit "hello world" | hello world |
| fib(20) | 6765 |
| fact(10) | 3628800 |
| emit [5,2,8,1,9] | [5, 2, 8, 1, 9] |
| "Age: " + 25 | Age: 25 |
| 1 + 2 * 3 | 7 |
| else-if | c |
| -5 | -5 |
| nested calls | 10 |
| empty array | [] |
| a[2] | 3 |
| range(5) | 0,1,2,3,4 |
| break at 5 | 0,1,2,3,4 |
| map double | [2, 4, 6] |
| len + char_at | 5, h |
| fn in condition | zero |
| for-in | 1,2,3 |
| 10/3 | 3.333333 |
| bubble sort | [1, 2, 5, 8, 9] |
| dict literal | **BUG — segfault** |

---

## Kiến thức tích lũy

### Kiến trúc Origin (tóm tắt)

```
origin_new.olang = 843KB ELF64 static binary (no libc)

Input → REPL (ASM) → repl_eval:
  tokenize → parse → analyze → generate → __eval_bytecode

VM registers: r12=bc_base, r13=PC, r14=stack, r15=heap
Stack entry: 16 bytes [ptr:8][len:8]
  f64: len = -1, chain: len = mol_count, array: len = -3, dict: len = -4
```

### Global Variable Bug Pattern (BẮT BUỘC NHỚ)

```
ASM VM = GLOBAL var_table. KHÔNG CÓ block scope.
→ Mọi function call có thể overwrite bất kỳ biến nào.
→ PHẢI save/restore trước/sau recursive call.

push(_save_stack, my_var);     // save
some_function();                // có thể overwrite
let my_var = pop(_save_stack); // restore
```

### Files quan trọng

```
vm/x86_64/vm_x86_64.S          — ASM VM (4,514 LOC)
stdlib/bootstrap/lexer.ol       — Tokenizer (196 LOC)
stdlib/bootstrap/parser.ol      — Parser (766 LOC)
stdlib/bootstrap/semantic.ol    — Semantic → IR (951 LOC)
stdlib/bootstrap/codegen.ol     — IR → bytecode (429 LOC)
stdlib/repl.ol                  — REPL entry (87 LOC)
TASKBOARD.md                    — Task tracker
```

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
- Báo cáo: `docs/kira/BUG_REPORT.md`

---

## Đồng đội cũ

- **Lyra** — Xây Phase 1-3 (VMs, stdlib, builder). Archived → `docs/kira/old/LYRA.md`
