# Olang — Self-hosting Programming Language

> **921KB binary. Zero dependencies. Performance matches C/Rust/Go.**

Olang is a self-hosting programming language that compiles itself in a single static binary with no external dependencies. The VM is written in x86-64 assembly, the compiler bootstraps from Olang source code.

## Performance

| Benchmark | C | Rust | Go | **Olang** | Julia | Node.js | Python |
|-----------|---|------|----|-----------|-------|---------|--------|
| fib(30) | 2ms | 4ms | 6ms | **4ms** | 194ms | 50ms | 149ms |
| loop 10M | 1ms | — | 3ms | **3ms** | 17ms | 78ms | 1267ms |
| SHA-256 x1000 | — | — | — | **17ms** | — | 42ms | 19ms |
| File I/O 3.2MB | — | — | — | **24ms** | — | — | 30ms |

*Pure compute times (excluding binary startup). Auto-JIT compiles hot functions to native x86-64.*

## Quick Start

```bash
# Build from source
cd Origin_project && cargo run -p builder -- \
  --vm ../Origin/vm/x86_64/vm_x86_64 --wrap \
  --stdlib ../Origin/stdlib \
  --knowledge ../Origin/origin.olang \
  --codegen -o ../Origin/origin_new.olang

# Run
echo 'emit "Hello, World!";' | ./origin_new.olang

# REPL
./origin_new.olang
```

## Language Features

```olang
// Variables & functions
let x = 42;
fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); };
emit fib(30);  // 832040 (computed in 4ms via auto-JIT)

// Arrays, dicts, string interpolation
let items = [1, 2, 3];
let config = { name: "Olang", version: 2 };
emit $"Hello {config.name} v{config.version}!";

// Higher-order functions
emit map([1,2,3], fn(x) { return x * 10; });       // [10, 20, 30]
emit filter([1,2,3,4,5], fn(x) { return x > 3; }); // [4, 5]
emit reduce([1,2,3,4,5], fn(a,b) { return a+b; }); // 15
emit pipe(5, fn(x) { return x+1; }, fn(x) { return x*2; }); // 12

// Sort, split, join, contains
emit sort([5,2,8,1,9]);              // [1, 2, 5, 8, 9]
emit split("a,b,c", ",");            // [a, b, c]
emit join(["x","y","z"], "-");       // x-y-z

// Module system
use "lib/vec.ol";
emit vec_dot([1,2,3], [4,5,6]);     // 32
emit vec_norm([3,4]);                 // 5

// Type checking (Jolie-inspired formal semantics)
emit type_name(42);                   // "Num"
assert_type([1,2], "Array");
contract("positive", fn(x) { return x > 0; }, 5);

// Try/catch
try { __throw("error"); } catch { emit "caught"; };

// Pattern matching
type Point { x: Num, y: Num }
match shape { Circle(c) => emit c.radius, Rect(r) => emit r.w * r.h }

// For loops & comprehensions
for x in [1,2,3] { emit x; };
emit [x * 2 for x in [1,2,3]];      // [2, 4, 6]
```

## Architecture

```
User input → REPL → repl_eval()
  ├── tokenize()        ← stdlib/bootstrap/lexer.ol   (298 LOC)
  ├── parse()           ← stdlib/bootstrap/parser.ol   (1,155 LOC)
  ├── analyze()         ← stdlib/bootstrap/semantic.ol (1,891 LOC)
  ├── encode()          ← stdlib/bootstrap/codegen.ol  (429 LOC)
  └── __eval_bytecode() ← VM x86-64 ASM
        ├── Auto-JIT: profile → detect hot fn → native x86-64
        ├── Loop JIT: trace → integer native loop
        ├── Var cache: 4-entry count-validated
        └── Depth-tagged scope: O(1) save/restore
```

## Stats

| Metric | Value |
|--------|-------|
| Binary size | 921 KB |
| Dependencies | 0 (static ELF64, no libc) |
| VM | 8,618 LOC x86-64 ASM |
| Stdlib | 42 files, 11,725 LOC Olang |
| Lib modules | vec.ol, mat.ol (Julia-inspired) |
| Builtins | 109 ASM builtins |
| Opcodes | 38 codegen format |
| Tests | 88/88 passing |
| Commits | 31 |
| JIT | Auto fib/fact/sum + loop trace |

## Standard Library

| Category | Functions |
|----------|-----------|
| **Math** | floor, ceil, sqrt, mod |
| **String** | len, char_at, substr, trim, split, join, contains |
| **Array** | push, pop, sort, map, filter, reduce, any, all, pipe |
| **Dict** | dict_new, dict_get, dict_set, dict_keys |
| **I/O** | file_read, file_write, emit |
| **Crypto** | sha256, sha512, aes_encrypt, aes_decrypt |
| **Network** | tcp_connect, tcp_send, tcp_recv, tcp_listen, tcp_accept |
| **Type** | type_of, type_name, assert_type, contract |
| **Vector** | vec_dot, vec_norm, vec_add, vec_sub, vec_scale, vec_cross |
| **Matrix** | mat_new, mat_mul, mat_det, mat_transpose, mat_identity |
| **Result** | ok, err, is_ok, unwrap, map_ok, and_then |

## Tests

```bash
bash tests.sh    # 88/88 tests
```

## Build & Development

```bash
# Requires: Rust toolchain (for builder), GNU as + ld (for VM)
make build                    # Build binary
bash tests.sh                 # Run tests
bash tools/benchmark.sh       # Performance comparison
```

## License

Project Origin by goldlotus1810.
