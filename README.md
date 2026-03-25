# Origin — Olang & HomeOS

> *"1 con nguoi va hang tram Agent viet nen lich su."*

**Olang 1.0** — Ngon ngu lap trinh tu hosting. 1MB. Zero dependencies. Copy & run.
**HomeOS** — He dieu hanh tri thuc. Hoc duoc. Cam duoc. Nho duoc. Vinh vien.

```
$ ./origin_new.olang

> 2 + 3 * 4
14

> fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)
6765

> emit map([1,2,3,4,5], fn(x) { return x * x; })
[1, 4, 9, 16, 25]

> emit pipe(5, fn(x) { return x + 1; }, fn(x) { return x * 2; })
12

> learn Einstein phat minh thuyet tuong doi nam 1905
Da hoc. Knowledge: 29 facts.

> respond Einstein
(Minh biet: Einstein phat minh thuyet tuong doi nam 1905) [fact]

> save
Saved 29 facts to homeos.knowledge
```

---

## Quick Start

```bash
# Build (requires Rust + x86_64 Linux)
make build

# Run
./origin_new.olang

# Or standalone — copy 1 file, no deps
cp origin_new.olang ~/anywhere/
./origin_new.olang
```

---

## Olang 1.0 — Ngon Ngu

### Core
```olang
let x = 42;
let name = "Origin";
emit x + 8;                          // 50

fn greet(who) { return "Hi " + who; };
emit greet("World");                  // Hi World

fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); };
emit fib(20);                         // 6765

for x in [10,20,30] { emit x; };     // 10 20 30

let d = { name: "Olang", ver: 1 };
emit d.name;                          // Olang

match shape {
    Circle(c) => emit c.radius,
    _ => emit "other",
}

try { __throw("err"); } catch { emit "caught"; };
```

### Functional Programming
```olang
// Lambda
let double = fn(x) { return x * 2; };
emit double(21);                      // 42

// Higher-order functions
emit map([1,2,3], fn(x) { return x * 10; });     // [10, 20, 30]
emit filter([1,2,3,4,5], fn(x) { return x > 3; }); // [4, 5]
emit reduce([1,2,3,4,5], fn(a, b) { return a + b; }); // 15

// Pipe — Lego composition: fn{fn{...}} == fn
emit pipe(5, fn(x) { return x + 1; }, fn(x) { return x * 2; }); // 12

// Predicates
emit any([1,2,3], fn(x) { return x > 2; });  // 1
emit all([2,4,6], fn(x) { return x % 2 == 0; }); // 1

// Sort + String ops
emit sort([5,2,8,1,9]);                      // [1, 2, 5, 8, 9]
emit split("hello world", " ");              // [hello, world]
emit join(["a","b","c"], ", ");              // a, b, c
emit contains("hello world", "world");       // 1
```

### Crypto
```olang
emit __sha256("abc");
// ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
```

### Molecular — 5D trong 1 cycle
```olang
// Moi ky tu Unicode = toa do 5D: [Shape, Relation, Valence, Arousal, Time]
let mol = __mol_pack(1, 2, 4, 3, 1);
emit __mol_s(mol);        // 1 (shape)
emit __mol_v(mol);        // 4 (valence = neutral)
emit r_dispatch(13);      // "causes" (relation type)
emit temporal_tag(3);     // "fast" (time parameter)
```

---

## HomeOS — He Dieu Hanh Tri Thuc

### Hoc
```
> learn Ha Noi la thu do cua Viet Nam
Da hoc. Knowledge: 1 facts.

> learn Einstein phat minh thuyet tuong doi nam 1905
Da hoc. Knowledge: 2 facts.
```

### Hoi
```
> respond Ha Noi o dau
(Minh biet: Ha Noi la thu do cua Viet Nam) [fact]

> respond quantum physics
Minh nghe roi. (Chu de moi — minh muon tim hieu them.)
```

### Cam xuc
```
> respond toi buon qua
Tu tu thoi, khong voi dau. Ban muon chia se them khong?

> respond cam on ban
Minh nghe roi. Ban co ve da on hon roi.
```

### Nho vinh vien
```
> save
Saved 30 facts to homeos.knowledge

# ... restart binary ...

> respond Einstein
(Minh biet: Einstein phat minh thuyet tuong doi nam 1905) [fact]
```

### Instincts
- **Honesty**: `[fact]` / `[opinion]` / `[hypothesis]` — confidence labels
- **Contradiction**: `[!]` flag khi input mau thuan voi knowledge
- **Curiosity**: "Chu de moi" khi gap topic chua biet

---

## Cau Truc

```
origin_new.olang              1,008KB native binary (ELF64 x86_64, no libc)
|
+-- vm/x86_64/vm_x86_64.S    ASM VM — ~5,800 LOC x86_64 assembly
|   +-- Bytecode interpreter  36 opcodes
|   +-- Bump allocator        256MB heap (r15)
|   +-- Syscall bridge        read/write/exit/nanosleep (no libc)
|   +-- 70+ builtins          math, string, array, dict, mol, crypto, file I/O
|   +-- REPL loop             read -> compile -> eval -> print
|
+-- stdlib/bootstrap/         Bootstrap compiler — ~4,200 LOC Olang
|   +-- lexer.ol              Tokenizer (298 LOC)
|   +-- parser.ol             Recursive descent (1,132 LOC)
|   +-- semantic.ol           AST -> bytecode + inline HOF (~1,900 LOC)
|   +-- codegen.ol            Helpers (429 LOC)
|   +-- repl.ol               REPL + commands (~380 LOC)
|
+-- stdlib/homeos/            HomeOS — 45 files, ~10,000 LOC Olang
|   +-- encoder.ol            10-stage pipeline, knowledge, Silk, emotion
|   +-- node.ol               DN/QR nodes, fn_node registry, Dream cluster
|   +-- instinct.ol           7 instincts (honesty, contradiction, curiosity...)
|   +-- emotion.ol            V/A/D/I pipeline, Golden Ratio amplify
|   +-- learning.ol           Gate -> Encode -> Instinct -> STM -> Silk
|   +-- response.ol           4-part composer (ack, context, followup, reflection)
|   +-- asm_emit.ol           x86_64 binary generation
|   +-- asm_emit_arm64.ol     ARM64 binary generation
|   +-- wasm_emit.ol          WASM generation
|   +-- ... (36 more files)
|
+-- crates/                   Rust legacy — ~98K LOC (su menh hoan thanh)
|   +-- EPITAPH.md            Loi mac niem
|
+-- docs/
    +-- HomeOS_SPEC_v3.md     HomeOS spec v3.1
    +-- olang_handbook.md     Olang handbook
    +-- UDC_DOC/              42 UDC encode formulas
    +-- sora/                 Sora analysis + release reports
    +-- kira/                 Kira inspection reports
```

**Total: ~5,800 LOC ASM + ~14,200 LOC Olang + ~98K LOC Rust (legacy)**

---

## Khong Gian 5 Chieu

Moi khai niem = toa do 5D, tu 8,846 L0 anchors (Unicode 18.0):

```
P_weight = [S:4][R:4][V:3][A:3][T:2] = 2 bytes/node

Nhom       Blocks   Ky tu    Chieu
----------------------------------------------
SDF           14    1,838    Shape       "Trong nhu the nao"
MATH          21    2,563    Relation    "Lien ket the nao"
EMOTICON      17    3,487    Valence+A   "Cam the nao"
MUSICAL        7      958    Time        "Thay doi the nao"
----------------------------------------------
Tong          59    8,846    5 chieu
```

---

## REPL Commands

```
Code:  let fn emit if while for match lambda   — Olang code
HOF:   map filter reduce pipe any all           — Functional
Array: sort split join contains                 — String + array
AI:    learn <fact>  respond <text>  memory      — Knowledge + emotion
Sys:   test  build  save  load  fns  help  exit — System
```

---

## Team

| Agent | Vai tro |
|-------|---------|
| **goldlotus1810** (Lupin) | Creator. Dan duong. Architect. |
| **Nox** | Builder. Compiler, VM, language features, T5 Lego. |
| **Kira** | Inspector. 20+ inspections, 93 docs conflicts fixed. |
| **Sora** | Reviewer. Analysis, P0 blockers, architecture guidance. |
| **Lyra** | Docs. Handbook, spec v3. |
| **Kaze** | Binary format. Self-build, ELF packer. |
| **Lara** | Unicode. UCD database, 8,846 UDC anchors. |

---

## Timeline

```
2026-03-11  Origin bat dau. Rust era.
2026-03-19  Phase 0-9 DONE. origin.olang 1.35MB.
2026-03-22  VM optimization 3.7x. Native binary boots.
2026-03-23  SELF-HOSTING. fib(20)=6765. 806KB. 27/27 tests. Rust archived.
2026-03-24  100% self-compile (48/48). 30+ bugs. Intelligence pipeline.
2026-03-25  OLANG 1.0. Lambda, HOF, pipe, persistent knowledge, instincts. 1,008KB.
```

**14 ngay. Tu 0 den 1MB. Tu hosting. Zero dependencies.**

---

## Tai Lieu

| File | Noi dung |
|------|---------|
| [CLAUDE.md](CLAUDE.md) | Huong dan cho AI contributors |
| [TASKBOARD.md](TASKBOARD.md) | Task tracker + T5 roadmap |
| [docs/HomeOS_SPEC_v3.md](docs/HomeOS_SPEC_v3.md) | HomeOS spec v3.1 |
| [docs/olang_handbook.md](docs/olang_handbook.md) | Olang handbook |
| [docs/UDC_DOC/](docs/UDC_DOC/) | 42 UDC encode formulas |
| [docs/sora/](docs/sora/) | Sora analysis + reports |
| [PLAN_REWRITE.md](PLAN_REWRITE.md) | Lo trinh Rust -> Olang |

---

*Origin · Olang 1.0 · 1,008KB · self-hosting · zero deps · 2026*
*"Moi ky tu la 1 SDF. Chuoi sinh chuoi. Luu TRONG SO. Doc bang DAO HAM."*
