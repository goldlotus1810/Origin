# Olang — Self-Hosting Programming Language

**1MB binary. Zero dependencies. Copy one file and run.**

Olang is a programming language that compiles itself. The entire compiler, runtime, standard library, and an AI knowledge engine (HomeOS) are packed into a single 1MB native binary. No installation. No package manager. No runtime.

```
$ chmod +x origin_new.olang
$ ./origin_new.olang

> 2 + 3
5

> fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }
> emit fib(20)
6765

> emit map([1,2,3,4,5], fn(x) { return x * x; })
[1, 4, 9, 16, 25]

> emit sort([5,2,8,1,9])
[1, 2, 5, 8, 9]

> hi
Xin chao! Minh la HomeOS. Ban muon lam gi hom nay?

> learn Vietnam is a country in Southeast Asia with capital Hanoi
Da hoc. Knowledge: 29 facts.

> respond Where is Vietnam?
(Minh biet: Vietnam is a country in Southeast Asia with capital Hanoi) [fact]

> save
Saved 29 facts to homeos.knowledge
```

---

## Install

### Pre-built binary (Linux x86_64)

```bash
# Download the binary (or copy from someone who has it)
cp origin_new.olang ~/bin/olang
chmod +x ~/bin/olang
~/bin/olang
```

That's it. One file. No dependencies. No libc. No runtime.

### Build from source

Requires: Linux x86_64, Rust toolchain, GNU as + ld

```bash
git clone https://github.com/goldlotus1810/Origin.git
cd Origin
make build
./origin_new.olang
```

### Verify

```bash
# Run test suite (20 tests)
echo 'test' | ./origin_new.olang
# Expected: ALL PASS: 20/20

# Fibonacci
echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)' | ./origin_new.olang
# Expected: 6765

# SHA-256
echo 'emit __sha256("abc")' | ./origin_new.olang
# Expected: ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
```

---

## Language Features

### Variables & Functions
```olang
let x = 42;
let name = "Olang";
emit x + 8;                              // 50

fn add(a, b) { return a + b; };
emit add(3, 4);                           // 7

fn fib(n) {
    if n < 2 { return n; };
    return fib(n-1) + fib(n-2);
};
emit fib(20);                             // 6765
```

### Control Flow
```olang
if x > 0 { emit "positive"; } else { emit "negative"; };

let i = 0;
while i < 5 { emit i; i = i + 1; };

for x in [10, 20, 30] { emit x; };

match token.kind {
    Keyword(k) => emit k.name,
    Number(n)  => emit n.value,
    _          => emit "other",
};

try { __throw("oops"); } catch { emit "caught"; };
```

### Data Structures
```olang
// Arrays
let arr = [1, 2, 3, 4, 5];
emit arr[0];                              // 1
emit len(arr);                            // 5
push(arr, 6);
emit sort([5, 2, 8, 1, 9]);              // [1, 2, 5, 8, 9]

// Dicts (pretty-printed)
let user = { name: "Lupin", age: 30 };
emit user;                                // {name: Lupin, age: 30}
emit user.name;                           // Lupin

// Strings
emit split("hello world foo", " ");      // [hello, world, foo]
emit join(["a", "b", "c"], ", ");        // a, b, c
emit contains("hello world", "world");   // 1
```

### Functional Programming
```olang
// Lambda (anonymous functions)
let double = fn(x) { return x * 2; };
emit double(21);                          // 42

// Higher-order functions
emit map([1,2,3], fn(x) { return x * 10; });         // [10, 20, 30]
emit filter([1,2,3,4,5], fn(x) { return x > 3; });   // [4, 5]
emit reduce([1,2,3,4,5], fn(a, b) { return a + b; }); // 15
emit any([1,2,3], fn(x) { return x > 2; });           // 1
emit all([2,4,6], fn(x) { return x % 2 == 0; });      // 1

// Pipe — compose functions like Lego blocks
emit pipe(5, fn(x) { return x + 1; }, fn(x) { return x * 2; });  // 12

// Chaining
let words = split("cherry apple banana", " ");
emit join(sort(words), " ");              // apple banana cherry
```

### Crypto
```olang
emit __sha256("hello");
// 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
```

---

## HomeOS — AI Knowledge Engine

Built into the binary. Learns facts, answers questions, tracks emotions, remembers across restarts.

### Teach it
```
> learn The Earth orbits the Sun in 365 days
Da hoc. Knowledge: 29 facts.

> learn Python was created by Guido van Rossum in 1991
Da hoc. Knowledge: 30 facts.
```

### Ask it
```
> respond Earth orbit
(Minh biet: The Earth orbits the Sun in 365 days) [fact]

> respond Who created Python?
(Minh biet: Python was created by Guido van Rossum in 1991) [fact]
```

### Emotional intelligence
```
> toi buon qua
Tu tu thoi, khong voi dau. Ban muon chia se them khong?

> cam on ban
Ban co ve da on hon roi.

> hi
Xin chao! Minh la HomeOS. Ban muon lam gi hom nay?

> bye
Hen gap lai! Cam on ban da tro chuyen.
```

### Persistent memory
```
> save
Saved 30 facts to homeos.knowledge

# Restart the binary — knowledge auto-loads
> respond Earth
(Minh biet: The Earth orbits the Sun in 365 days) [fact]
```

### 7 Instincts
| # | Instinct | Behavior |
|---|----------|----------|
| 1 | **Honesty** | `[fact]` / `[opinion]` / `[hypothesis]` confidence labels |
| 2 | **Contradiction** | `[!]` flag + polite correction when input conflicts with knowledge |
| 3 | **Causality** | Detects "why/tai sao" → causal reasoning mode |
| 4 | **Abstraction** | `[khai niem quen thuoc]` for well-known concepts |
| 5 | **Analogy** | Detects "similar/giong" → comparison mode |
| 6 | **Curiosity** | "Chu de moi" for unknown topics |
| 7 | **Reflection** | Periodic self-assessment of conversation quality |

---

## REPL Commands

| Command | Description |
|---------|-------------|
| `let x = 42` | Define variable |
| `fn f(x) { ... }` | Define function |
| `emit expr` | Print expression (or just type expression) |
| `map/filter/reduce/pipe` | Functional programming |
| `sort/split/join/contains` | Array & string operations |
| `learn <fact>` | Teach HomeOS a fact |
| `respond <text>` | Ask HomeOS (knowledge + emotion) |
| `memory` | Show STM, Silk, Knowledge, Emotion stats |
| `fns` | List registered function nodes |
| `save` | Save knowledge to file (persistent) |
| `load` | Load knowledge from file |
| `test` | Run 20 built-in tests |
| `help` | Show all commands |
| `exit` | Quit |

---

## Technical Details

```
Binary:     1,021KB (1,021,393 bytes), ELF64 x86_64, statically linked
Runtime:    No libc, no allocator, no runtime. Pure syscalls (read/write/exit)
VM:         5,987 LOC x86_64 assembly, 36 opcodes, 70+ builtins
Compiler:   3,748 LOC Olang (lexer + parser + semantic + codegen)
HomeOS:     10,042 LOC Olang (45 files: intelligence, emotion, knowledge, networking)
Stdlib:     1,740 LOC Olang (sort, json, iter, format, mol, chain)
REPL:       451 LOC Olang
Tests:      20/20 built-in
Self-build: Compiles itself. 3 generations verified.
```

### How it works

```
User input → REPL loop (ASM)
  → tokenize (lexer.ol)
  → parse (parser.ol)
  → compile to bytecode (semantic.ol)
  → execute on ASM VM
  → print result

Natural text → classify (greeting/question/chat)
  → encode to 5D molecule (UDC)
  → search knowledge (keyword + molecular similarity)
  → apply 7 instincts
  → compose response (emotion-aware)
  → learn from interaction (STM + Silk + Dream)
```

---

## Team

| Agent | Role |
|-------|------|
| **goldlotus1810** (Lupin) | Creator, architect, direction |
| **Nox** | Builder: compiler, VM, language features, HomeOS pipeline |
| **Kira** | Inspector: 23 inspections, 112 docs conflicts fixed |
| **Sora** | Reviewer: architecture analysis, P0 blockers, release audits |
| **Lyra** | Documentation: handbook, spec v3 |
| **Kaze** | Binary format: self-build, ELF packer |
| **Lara** | Unicode: UCD database, 8,846 UDC anchor points |

## Timeline

```
2026-03-11  Project starts (Rust era)
2026-03-19  Phase 0-9 complete. 1.35MB binary.
2026-03-22  VM optimization 3.7x. Native binary boots.
2026-03-23  SELF-HOSTING. Olang compiles itself. 806KB. Rust archived.
2026-03-24  100% self-compile (48/48 files). Intelligence pipeline.
2026-03-25  OLANG 1.0. Full language + AI engine. 1,021KB.
```

**14 days. From zero to 1MB self-hosting language + AI. Zero dependencies.**

---

## License & Links

- Repository: [github.com/goldlotus1810/Origin](https://github.com/goldlotus1810/Origin)
- Spec: [docs/HomeOS_SPEC_v3.md](docs/HomeOS_SPEC_v3.md)
- Handbook: [docs/olang_handbook.md](docs/olang_handbook.md)
- Master Plan: [plans/MASTER_PLAN_HOMEOS_V1.md](plans/MASTER_PLAN_HOMEOS_V1.md)

---

*Olang 1.0 · 1,021KB · self-hosting · zero dependencies · 2026*
