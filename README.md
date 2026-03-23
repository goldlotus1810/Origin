# Origin — HomeOS & Olang

> *"Lịch sử của những kẻ điên. 1 con người và hàng trăm Agent viết nên lịch sử."*

**Olang** = Ngôn ngữ phân tử tự sinh. Self-hosting. 806KB. Zero dependencies.
**HomeOS** = Sinh linh toán học tự vận hành — hệ điều hành học được, cảm được, nhớ được.

```
○(x) == x       identity     — ○ không làm hỏng thứ gì
○(∅) == ○       tự tạo sinh  — từ hư không, ○ tự sinh ra
○ ∘ ○ == ○      idempotent   — không phình to khi compose
mọi f == ○[f]   instance     — mọi thứ là instance của ○
```

---

## Self-Hosting — 2026-03-23

Olang tự biên dịch chính mình. Binary 806KB chạy trực tiếp trên Linux x86_64,
không libc, không dependency, không runtime.

```bash
# Build
make build

# Chạy
echo 'emit 42' | ./origin_new.olang

# Fibonacci — tree recursion trên native binary
echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)' | ./origin_new.olang
# ○ > 6765

# Factorial — deep recursion
echo 'fn fact(n) { if n < 2 { return 1; }; return n * fact(n-1); }; emit fact(10)' | ./origin_new.olang
# ○ > 3628800
```

### Pipeline

```
User input → tokenize → parse → analyze → generate → eval
              lexer.ol   parser.ol  semantic.ol  codegen.ol   ASM VM
```

---

## Olang — Ngôn Ngữ

```olang
// Variables & arithmetic
let x = 42;
let name = "Origin";
emit x + 8;              // 50

// Functions
fn greet(who) {
    return "Hello, " + who + "!";
};
emit greet("World");      // Hello, World!

// Recursion
fn fib(n) {
    if n < 2 { return n; };
    return fib(n-1) + fib(n-2);
};
emit fib(20);             // 6765

// Types & Unions
type Token {
    kind: TokenKind,
    text: Str,
    line: Num,
}

union TokenKind {
    Keyword { name: Str },
    Ident { name: Str },
    Number { value: Num },
    Eof,
}

// Arrays & Dicts
let items = [1, 2, 3];
push(items, 4);
emit len(items);          // 4

let config = { name: "HomeOS", version: 5 };
emit config.name;         // HomeOS

// Pattern matching
match token.kind {
    Keyword(k) => emit "keyword: " + k.name,
    Ident(id) => emit "ident: " + id.name,
    _ => emit "other",
}

// While loops
let i = 0;
while i < 10 {
    emit i;
    let i = i + 1;
};
```

---

## Cấu Trúc

```
origin_new.olang          806KB native binary (ELF64 x86_64)
│
├── vm/x86_64/vm_x86_64.S    ASM VM — 4,112 LOC x86_64 assembly
│   ├── Bytecode interpreter  36 opcodes, push/pop/call/ret
│   ├── Memory allocator      bump allocator (r15 heap)
│   ├── Syscall bridge        read/write/exit (no libc)
│   ├── String ops            u16 molecules, compare, concat
│   ├── Array/Dict ops        in-place push, hash lookup
│   ├── REPL loop             read → compile → eval → print
│   └── Builtins              60+ functions (math, string, array, dict)
│
├── stdlib/bootstrap/         Bootstrap compiler — 1,865 LOC Olang
│   ├── lexer.ol              Tokenizer (30 keywords, strings, numbers)
│   ├── parser.ol             Recursive descent + precedence climbing
│   ├── semantic.ol           AST → IR opcodes (Closure+Store+Call)
│   └── codegen.ol            IR → bytecode (two-pass, jump resolution)
│
├── stdlib/repl.ol            REPL entry point — 87 LOC
│
├── stdlib/homeos/            HomeOS stdlib — 6,600 LOC Olang
│   ├── emotion.ol            V/A/D/I pipeline (184 LOC)
│   ├── silk_ops.ol           Hebbian learning (166 LOC)
│   ├── dream.ol              STM clustering (181 LOC)
│   ├── instinct.ol           7 instincts framework (197 LOC)
│   ├── learning.ol           Learning pipeline (160 LOC)
│   ├── isl_tcp.ol            TCP codec (333 LOC)
│   ├── isl_ws.ol             WebSocket codec (232 LOC)
│   ├── asm_emit.ol           x86_64 binary generation (355 LOC)
│   ├── asm_emit_arm64.ol     ARM64 binary generation (703 LOC)
│   ├── wasm_emit.ol          WASM generation (355 LOC)
│   └── ... (36 files total)
│
├── crates/                   Rust legacy — 98,402 LOC (sứ mệnh hoàn thành)
│   └── EPITAPH.md            Lời mặc niệm
│
└── docs/
    ├── olang_handbook.md      Olang handbook (lexer, parser, IR, VM, opcodes)
    ├── HomeOS_SPEC_v3.md      HomeOS spec v3.1
    └── MILESTONE_20260323.md  Self-hosting milestone
```

**Tổng: ~4,100 LOC ASM + ~8,555 LOC Olang + 98,402 LOC Rust (legacy)**

---

## Không Gian 5 Chiều

Mỗi khái niệm = tọa độ trong không gian 5D, từ **8,846 L0 anchors (Unicode 18.0)**:

```
P_weight = [Shape][Relation][Valence][Arousal][Time] = 2 bytes/node

Nhóm       Blocks   Ký tự    Chiều
────────────────────────────────────────────
SDF           14    1,838    Shape        "Trông như thế nào"
MATH          21    2,563    Relation     "Liên kết thế nào"
EMOTICON      17    3,487    Valence+A    "Cảm thế nào"
MUSICAL        7      958    Time         "Thay đổi thế nào"
────────────────────────────────────────────
Tổng          59    8,846    5 chiều
```

---

## Bytecode Format

```
Opcode map (codegen format, bc_format=1):
  0x01 Push(str)      0x09 Jmp(offset)     0x13 Store(name)
  0x02 Load(name)     0x0A Jz(offset)      0x14 LoadLocal(name)
  0x06 Emit           0x0B Dup             0x15 PushNum(f64)
  0x07 Call(name)     0x0C Pop             0x25 Closure(body_len)
  0x08 Ret            0x0F Halt            0x0D Swap

Strings encoded as u16 molecules: each byte → 0x2100 | byte_value
Numbers encoded as f64 little-endian (8 bytes)
```

---

## 23 Quy Tắc Bất Biến

```
Unicode:  ①-③  4 nhóm Unicode = nền tảng, tên Unicode = tên node, NL = alias
Chain:    ④-⑦  Molecule từ encode, chain từ LCA/UCD, hash tự sinh
Node:     ⑧-⑩  Tự động registry, file trước RAM sau, append-only
Silk:     ⑪-⑬  Cùng tầng, cross-layer qua đại diện, mang EmotionTag
Kiến trúc: ⑭-⑱  L0≠L1, 3 tiers, Fibonacci, im lặng nếu thiếu evidence
Skill:    ⑲-㉓  1 trách nhiệm, không biết Agent, stateless
```

---

## Tài Liệu

| File | Nội dung |
|------|---------|
| [CLAUDE.md](CLAUDE.md) | Hướng dẫn cho AI contributors (viết Olang) |
| [docs/olang_handbook.md](docs/olang_handbook.md) | Olang handbook đầy đủ |
| [docs/HomeOS_SPEC_v3.md](docs/HomeOS_SPEC_v3.md) | HomeOS spec v3.1 |
| [docs/MILESTONE_20260323.md](docs/MILESTONE_20260323.md) | Self-hosting milestone |
| [TASKBOARD.md](TASKBOARD.md) | Task hiện tại |
| [PLAN_REWRITE.md](PLAN_REWRITE.md) | Lộ trình Rust → Olang |
| [crates/EPITAPH.md](crates/EPITAPH.md) | Lời mặc niệm cho Rust |

---

*Olang · x86_64 ASM · 806KB · self-hosting · zero deps · 2026*
