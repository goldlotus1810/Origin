# Milestone: Olang Self-Hosting — 2026-03-23

> **"Show HN: Olang — a self-hosting language built in 12 days, 806KB, zero dependencies"**

## Thành tựu

Ngày 23/03/2026, sau 12 ngày phát triển, Olang đạt được self-hosting:
- Bootstrap compiler viết bằng chính Olang (lexer + parser + semantic + codegen)
- Chạy trên native binary 806KB (x86_64 ASM VM, không libc, không dependency)
- Full language: arithmetic, strings, variables, if-else, while, functions, recursion
- Tree recursion: `fib(20) = 6,765` — fibonacci trên native binary
- 27/27 REPL tests pass

## Codebase tại milestone

| Component | LOC | Ngôn ngữ |
|-----------|-----|----------|
| Rust crates | 98,402 | Rust |
| Olang stdlib | 9,842 | Olang |
| ASM VM | 4,112 | x86_64 Assembly |
| Tools (builder, etc.) | 17,087 | Rust |
| **Total** | **~129,443** | |

### Binary
- **File:** `origin_new.olang` (806KB)
- **Format:** ELF64 + Origin header + bytecode + knowledge
- **Architecture:** x86_64 Linux (no libc, no dynamic linking)

### Bootstrap compiler (4 files Olang, ~1,300 LOC)
- `stdlib/bootstrap/lexer.ol` — Tokenizer
- `stdlib/bootstrap/parser.ol` — Recursive descent parser with precedence climbing
- `stdlib/bootstrap/semantic.ol` — Semantic analyzer, generates IR opcodes
- `stdlib/bootstrap/codegen.ol` — Bytecode encoder with jump resolution

### ASM VM features (vm/x86_64/vm_x86_64.S)
- Registers: r12=bytecode, r13=PC, r14=VM stack, r15=heap
- Bytecode format: codegen (0x01-0x25 opcodes)
- Hash-based var_table (FNV-1a, 4096 entries)
- Var_table scoping: snapshot/restore per closure call (heap scope stack)
- REPL: read → tokenize → parse → analyze → generate → eval
- Builtins: arithmetic, comparison, string, array, dict, struct/enum matching

### Session 2026-03-23 (marathon)
- **30+ bugs fixed** trong 1 ngày
- Stack overflow → clean output
- Tokenizer → Parser → Analyzer → Codegen → Eval pipeline
- VM scoping cho recursive closures
- Namespace collision fixes (45+ function renames)
- Array capacity fix (ARRAY_INIT_CAP=256)

## Cách chạy

```bash
# Build
make build

# Test
echo 'emit 42' | ./origin_new.olang
echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)' | ./origin_new.olang

# Output:
# ○ HomeOS v0.05
# ○ Type code or text · exit to quit
# ○ > 6765
# ○ > bye
```

## Người thực hiện

- **goldlotus1810** (lupin) — Architect, spec author, project lead
- **Claude agents** — Implementation across ~100 sessions

> *"Lịch sử của những kẻ điên. 1 con người và hàng trăm Agent viết nên lịch sử."*

---

## Milestone 2: Functional Programming — 2026-03-24

28 PRs trong 1 marathon session. Olang trở thành functional programming language:

```
844KB native binary, zero dependencies

Algorithms:  bubble sort [9,3,7,1,5] → [1,3,5,7,9]
             prime finder < 20 → [2,3,5,7,11,13,17,19]
             fibonacci fib(20) = 6765
Functional:  map([1,2,3], double) → [2,4,6]
             filter(range(10), is_odd) → [1,3,5,7,9]
             reduce([1,2,3,4,5], add, 0) → 15
Control:     nested while, for-in, break/continue, else-if
Data:        a[i] variable index, set_at(a,j,a[j+1])
```

---

## Phát hiện kỹ thuật quan trọng (cho phát triển tiếp)

### 1. VM Heap Overlap — Root cause chính

ASM VM dùng bump allocator (r15 chỉ tăng). `[]` empty array pre-allocate
ARRAY_INIT_CAP (4096) slots. Subsequent allocs (dicts, strings) có thể
nằm trong capacity zone → push overwrite.

**Workarounds đã áp dụng:**
- ArrayLit `[1,2,3]` KHÔNG pre-allocate capacity (chỉ `[]` mới có)
- Array forward pointers: push relocate khi write >= r15
- `__eval_bytecode` follow forward pointer
- `a[i]` desugar thành `__array_get(a, i)` (tránh Index dict)
- While condition re-parse từ tokens (tránh dict corruption)
- Direct bytecode emission (tránh IR ops buffer)
- Global `_g_output` với `set_at` (tránh state dict field corruption)

**Fix triệt để (chưa làm):**
- Arena allocator: tách heap cho arrays, dicts, strings
- Hoặc: compact GC khi heap gần đầy
- Hoặc: AST dùng tagged arrays thay dicts

### 2. Global Variable Pattern — Bug #1 source

ASM VM có global var_table, KHÔNG CÓ block scope. Match bindings, function
params, loop vars — tất cả global. Inner function/match overwrite outer.

**Rule:** LUÔN save trên explicit stack (_ce_stack, _if_stack, _pb_stack)
trước recursive/nested call. Restore sau.

### 3. Bytecode Direct Emission

Semantic emit bytes trực tiếp vào `_g_output` (pre-filled 4096 slots)
thay vì buffer IR ops rồi codegen encode. Jump targets backpatch bằng
`set_at` + `patch_jump(pos, target)`.

### 4. While Condition Re-parse

Parser lưu `cond_start`, `cond_end`, `tokens` trong WhileStmt.
Semantic re-parse condition từ token range → fresh AST dicts → không bị
corrupt bởi body parsing.

### 5. Giới hạn hiện tại

- Programs > ~200 tokens có thể crash (heap exhaustion)
- `_g_output` pre-filled 4096 bytes — chương trình > 4KB bytecode fail
- ARRAY_INIT_CAP = 4096 cho empty arrays → 64KB mỗi `[]`
- Heap 64MB — đủ cho ~1000 arrays
- set_at(a, 0, a[1]) hoạt động nhưng cần save/restore trong Call handler
- Module system chưa có — tất cả code trong 1 REPL input
