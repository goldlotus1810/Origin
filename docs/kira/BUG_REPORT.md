# BUG REPORT — Origin / Olang

> **Inspector: Kira**
> **Cập nhật: 2026-03-23**
> **Branch tested: main @ `4fdf79a`**

---

## Tổng quan phiên kiểm tra

- **Build**: `make build` — OK (843KB ELF64 static binary)
- **16 Rust warnings** (unused imports/functions/constants trong `crates/olang`) — dead code legacy, không ảnh hưởng runtime
- **Binary**: `origin_new.olang` đang tracked trong git dù đã có trong `.gitignore` (add trước khi gitignore tồn tại)

---

## BUG #1 — Dict literal gây SEGFAULT (Critical)

### Triệu chứng

```
Input:  let d = { name: "Kira", age: 3 }; emit d.name
Output: Parse error: expected '=' got ':'
        ... (nhiều parse errors)
        Segmentation fault (exit 139)
```

### Phân tích

1. **Parser không hỗ trợ dict literal syntax `{ key: value }`** — Parser hiện tại thấy `{` và expect block statement, không phải dict. Nó expect `identifier = expr` (let statement) thay vì `identifier: expr` (dict entry).

2. **Parse error KHÔNG được recover an toàn** — Sau khi parser fail, chương trình tiếp tục chạy với AST không hợp lệ → VM truy cập memory sai → segfault. Đây là lỗi nghiêm trọng hơn cả thiếu feature.

### Root cause (dự đoán)

- `stdlib/bootstrap/parser.ol` — function `parse_stmt` hoặc `parse_expr` gặp `{` → parse thành block → fail khi thấy `:` thay vì `=`
- `stdlib/repl.ol` — `repl_eval` không check parse error trước khi gọi `analyze` → `generate` → `__eval_bytecode` trên AST rỗng/corrupt

### Cách debug

```bash
# Reproduce
echo 'let d = { name: "Kira", age: 3 }; emit d.name' | ./origin_new.olang

# Debug với GDB
echo 'let d = { name: "Kira", age: 3 }' | gdb -batch \
  -ex "break .eval_bc_run" \
  -ex "run" \
  --args ./origin_new.olang

# Kiểm tra parser output
# Thêm emit debug vào parser.ol trước khi gọi analyze
```

### Fix đề xuất

**Ưu tiên 1 — Error recovery (chống crash):**
- Trong `repl.ol`: sau `parse(tokens)`, kiểm tra AST có valid không. Nếu có parse error → emit error message → return, KHÔNG gọi tiếp analyze/generate/eval.

**Ưu tiên 2 — Dict literal syntax:**
- Trong `parser.ol`: khi gặp `{` trong expression context, phân biệt block vs dict literal
- Heuristic: `{ identifier :` → dict literal, `{ identifier =` hoặc `{ if/while/...` → block
- Hoặc dùng syntax khác: `dict(name: "Kira", age: 3)` để tránh ambiguity với block

---

## TEST SUITE — Kết quả đầy đủ

### PASS (20/21 tests)

| # | Test | Input | Expected | Actual |
|---|------|-------|----------|--------|
| 1 | Emit number | `emit 42` | 42 | 42 |
| 2 | Fibonacci | `fib(20)` | 6765 | 6765 |
| 3 | Factorial | `fact(10)` | 3628800 | 3628800 |
| 4 | String emit | `emit "hello world"` | hello world | hello world |
| 5 | Array literal | `emit [5,2,8,1,9]` | [5, 2, 8, 1, 9] | [5, 2, 8, 1, 9] |
| 6 | Bubble sort | 5 elements | [1, 2, 5, 8, 9] | [1, 2, 5, 8, 9] |
| 7 | String concat | `"Age: " + 25` | Age: 25 | Age: 25 |
| 8 | Else-if | 3 branches | "c" | "c" |
| 9 | Unary minus | `emit -5` | -5 | -5 |
| 10 | Nested calls | `add(add(1,2),add(3,4))` | 10 | 10 |
| 11 | Higher-order fn | `map([1,2,3], double)` | [2, 4, 6] | [2, 4, 6] |
| 12 | String len | `len("hello")` | 5 | 5 |
| 13 | char_at | `char_at("hello", 0)` | h | h |
| 14 | Precedence | `1 + 2 * 3` | 7 | 7 |
| 15 | Division | `10 / 3` | 3.333... | 3.333333 |
| 16 | Empty array | `emit []` | [] | [] |
| 17 | Fn in condition | `if foo() == 0` | "zero" | "zero" |
| 18 | For-in loop | `for x in [1,2,3]` | 1,2,3 | 1,2,3 |
| 19 | Range | `for i in range(5)` | 0..4 | 0..4 |
| 20 | Break | `break` at i==5 | 0..4 | 0..4 |

### FAIL (0/21 tests)

_All 21 tests pass as of 2026-03-24._

| # | Test | Input | Expected | Actual | Status |
|---|------|-------|----------|--------|--------|
| 21 | Dict literal | `{ name: "Kira" }` | dict object | dict object | FIXED ✅ |

---

## Ghi chú cho phiên sau

- [x] Fix BUG #1 — error recovery + dict literal syntax (2026-03-24)
- [ ] Kiểm tra `origin_new.olang` tracked trong git — cân nhắc `git rm --cached` để untrack
- [ ] Dọn 16 Rust warnings (optional, low priority)
- [ ] Test thêm: recursion depth limit, large arrays (>256 elements), string edge cases
- [ ] Kiểm tra `make check-all` khi Rust tests sẵn sàng

---

## Changelog

```
2026-03-23  Phiên đầu tiên. 20/21 tests pass. BUG #1: dict literal segfault.
2026-03-24  BUG #1 FIXED. 21/21 tests pass. Dict literal + parse error recovery.
```
