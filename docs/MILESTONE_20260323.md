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
