# Plan: Olang thay thế Rust viết HomeOS

> **Mục tiêu:** Nâng Olang từ DSL → general-purpose language đủ sức viết toàn bộ HomeOS.
> **Nguyên tắc:** Không phá vỡ kiến trúc hiện tại. Mở rộng từ nền tảng đã có.
> **Phân công:** AI-A (ngôn ngữ core) · AI-B (runtime & ecosystem)

---

## Hiện trạng Olang

```
✅ Đã có                          ❌ Chưa có
─────────────────────────────────────────────────────
36 opcodes (stack VM)             Module system
if/else/while/for/match/try       Custom types (struct/enum)
fn + Call/Ret (depth 256)         Trait/Interface
let/Store/Load + Scope            Generics
Array, Dict, String (as Chain)    Iterator/Lazy eval
Arithmetic + Math builtins        Option/Result types
3 compile targets (C/Rust/WASM)   Closures / Higher-order fn
DeviceIO + FileIO + FFI           Async/Channel concurrency
SpawnBegin/End (cooperative)      Private/Public scope
LeoAI self-programming            Standard library
18 RelOps                         Byte-level serialization
Molecular literal { S R V A T }   Pattern destructuring
```

---

## Rust features HomeOS dùng → Olang cần thay thế

| Rust Feature | Dùng ở đâu | Olang cần |
|---|---|---|
| `struct` + methods | Mọi crate | `type` keyword + `impl` block |
| `enum` + variants | GateVerdict, ProcessResult, LeoState, MsgType | `union` keyword + match |
| `trait` + `impl Trait` | Skill, Iterator, From/Into | `trait` keyword |
| `Vec<T>`, `BTreeMap<K,V>` | Silk, Memory, Registry | Generic collections |
| `Option<T>`, `Result<T,E>` | Mọi nơi | Builtin Option/Result |
| `impl` methods (`&self`, `&mut self`) | Mọi nơi | Method receivers |
| Closures (`.map()`, `.filter()`) | Walk, Dream, Learning | Lambda syntax |
| `match` destructuring | Gate, ISL, VM | Pattern destructuring |
| `#[repr(u8)]` byte packing | ISL, Edge, Compact | Wire format syntax |
| `const fn` | Hebbian (PHI) | Compile-time eval |
| `pub/pub(crate)` | Mọi nơi | Visibility modifiers |
| Slice `[0..8]` | Serialization | Slice operations |
| `alloc::format!()` | Leo, Runtime | String interpolation |

---

## 4 Phase — Thứ tự bắt buộc

### Phase 1: Type System Foundation (tuần 1-2)

**Tại sao trước:** Mọi thứ phía sau đều cần type. Không có type → không có module, trait, collection.

#### AI-A: Core Types (syntax → semantic → IR → VM)

**A1. Struct/Record type**
```
// Syntax mới
type Vec3 {
  x: Num,
  y: Num,
  z: Num,
}

// Desugar → MolecularChain có metadata
// VM: StructNew(name), FieldGet(name), FieldSet(name)
```

Files sửa:
- `lang/syntax.rs` — parse `type Name { field: Type, ... }`
- `lang/semantic.rs` — validate type declarations, field access
- `exec/ir.rs` — thêm Op: `StructNew`, `FieldGet`, `FieldSet`, `StructDef`
- `exec/vm.rs` — implement struct operations trên stack

Test: 20+ tests cho struct create, field access, nested struct, pass to function

**A2. Enum/Union type**
```
union ProcessResult {
  Ok { chain: Chain, emotion: Emotion },
  Blocked { reason: Str },
  Crisis { message: Str },
  Empty,
}

// match đã có → mở rộng match destructuring
match result {
  Ok { chain, emotion } => { ... },
  Crisis { message } => { ... },
  _ => { ... },
}
```

Files sửa:
- `lang/syntax.rs` — parse `union Name { Variant { fields }, ... }`
- `lang/semantic.rs` — validate variant construction, match exhaustiveness
- `exec/ir.rs` — thêm Op: `UnionNew(variant)`, `UnionTag`, `UnionField`
- `exec/vm.rs` — implement union trên stack (tag byte + fields)

Test: 15+ tests cho union create, match destructure, nested union

**A3. Option/Result builtins**
```
// Option = union builtin
let x: Option = Some(42);
let y: Option = None;

// Result = union builtin
let r: Result = Ok(chain);
let e: Result = Err("failed");

// Unwrap operators
let v = x?;          // propagate None/Err
let v = x ?? 0;      // default value
```

Files sửa:
- `lang/semantic.rs` — recognize Option/Result as builtin unions
- `exec/vm.rs` — `?` operator, `??` operator
- `exec/ir.rs` — Op: `Unwrap`, `UnwrapOr`

Test: 10+ tests

#### AI-B: Method System + Visibility

**B1. Method blocks (impl)**
```
impl Vec3 {
  fn new(x, y, z) {
    Self { x: x, y: y, z: z }
  }

  fn length(self) {
    sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
  }

  fn scale(mut self, factor) {
    self.x = self.x * factor;
    self.y = self.y * factor;
    self.z = self.z * factor;
  }
}

let v = Vec3::new(1, 2, 3);
let len = v.length();
```

Files sửa:
- `lang/syntax.rs` — parse `impl TypeName { fn ... }`, `self`/`mut self` receiver
- `lang/semantic.rs` — bind methods to types, resolve `Self`, validate receivers
- `exec/ir.rs` — Op: `MethodCall(type, name)`, `LoadSelf`, `StoreSelf`
- `exec/vm.rs` — method dispatch: lookup type → method table → Call

Test: 15+ tests cho method call, self mutation, chaining

**B2. Visibility modifiers**
```
type Gateway {
  pub address: Str,      // public
  secret_key: Str,       // private (default)
}

pub fn connect(gw) { ... }    // public function
fn internal_check() { ... }   // private (default)
```

Files sửa:
- `lang/syntax.rs` — parse `pub` keyword trước field/fn/type
- `lang/semantic.rs` — enforce visibility rules khi access

Test: 8+ tests

---

### Phase 2: Abstraction & Composition (tuần 3-4)

**Tại sao sau Phase 1:** Trait cần type, Module cần visibility, Generics cần type.

#### AI-A: Trait System + Generics

**A4. Trait (interface)**
```
trait Skill {
  fn name(self) -> Str;
  fn execute(self, ctx) -> SkillResult;
}

impl Skill for ClusterSkill {
  fn name(self) { "cluster" }
  fn execute(self, ctx) {
    // ...
  }
}

// Trait as parameter type
fn run_skill(s: Skill, ctx) {
  let result = s.execute(ctx);
}
```

Files sửa:
- `lang/syntax.rs` — parse `trait Name { fn ... }`, `impl Trait for Type`
- `lang/semantic.rs` — trait registry, conformance check, method resolution
- `exec/ir.rs` — Op: `TraitCall(trait, method)`, vtable lookup
- `exec/vm.rs` — dynamic dispatch: object → trait vtable → method

Test: 20+ tests cho trait define, impl, polymorphism, multiple traits

**A5. Generics (type parameters)**
```
type Container[T] {
  items: Array[T],
  count: Num,
}

fn first[T](c: Container[T]) -> Option[T] {
  if c.count > 0 {
    Some(c.items[0])
  } else {
    None
  }
}
```

Files sửa:
- `lang/syntax.rs` — parse `Name[T]`, `fn name[T](...)`
- `lang/semantic.rs` — type parameter binding, monomorphization check
- `exec/vm.rs` — runtime: generics erased (all Chain), type tag for safety

Test: 15+ tests

#### AI-B: Module System + Closures

**B3. Module system**
```
// file: silk/graph.ol
mod silk.graph;

pub type SilkGraph { ... }
pub fn co_activate(g, a, b, w) { ... }

// file: agents/learning.ol
use silk.graph;
use silk.graph.SilkGraph;         // import type
use silk.graph.{ co_activate };   // import functions

fn learn(graph: SilkGraph) {
  co_activate(graph, a, b, 0.8);
}
```

Files sửa:
- `lang/syntax.rs` — parse `mod`, `use`, `pub` at module level
- `lang/semantic.rs` — module resolution, import validation, circular dependency detection
- `exec/ir.rs` — Op: `Import(module, symbol)`
- `exec/vm.rs` — module loader: path → parse → compile → cache → link

Mới tạo:
- `exec/module.rs` — ModuleLoader, ModuleCache, dependency graph

Test: 20+ tests cho import, re-export, circular detection, pub/private across modules

**B4. Closures + Higher-order functions**
```
let double = |x| { x * 2 };
let result = double(21);  // 42

// Dùng với array methods
let scores = [3, 1, 4, 1, 5];
let high = scores.filter(|s| { s > 3 });
let doubled = scores.map(|s| { s * 2 });
let sum = scores.fold(0, |acc, s| { acc + s });
```

Files sửa:
- `lang/syntax.rs` — parse `|params| { body }` lambda syntax
- `lang/semantic.rs` — closure capture analysis (by value, Olang = all Chain)
- `exec/ir.rs` — Op: `Closure(param_count, body_offset)`, `CallClosure`
- `exec/vm.rs` — closure = captured env + function body

Test: 15+ tests

---

### Phase 3: Collections & Iteration (tuần 5-6)

**Tại sao sau Phase 2:** Collections cần generics + trait. Iterator cần trait + closure.

#### AI-A: Iterator Protocol + Stdlib Collections

**A6. Iterator trait + chaining**
```
trait Iterator[T] {
  fn next(mut self) -> Option[T];
}

// Builtin methods trên Iterator:
// .map(f), .filter(f), .fold(init, f), .collect()
// .enumerate(), .zip(other), .take(n), .skip(n)
// .any(f), .all(f), .find(f), .count()
// .min(), .max(), .sum()
// .min_by(f), .max_by(f), .sort_by(f)

let names = entries
  .filter(|e| { e.kind == "Knowledge" })
  .map(|e| { e.name })
  .collect();
```

Files sửa:
- Tạo `stdlib/iterator.ol` — Iterator trait + default methods
- `exec/vm.rs` — lazy evaluation: Iterator trên stack = (source, transform chain)
- `lang/semantic.rs` — `.method()` chaining resolution

Test: 25+ tests

**A7. Collections stdlib**
```
// Vec[T] — dynamic array (đã có Array, nâng cấp)
let v = Vec::new();
v.push(item);
v.len();
v.get(i);           // → Option[T]
v.remove(i);
v.retain(|x| { x > 0 });

// Map[K, V] — sorted map
let m = Map::new();
m.set(key, value);
m.get(key);          // → Option[V]
m.has(key);
m.keys();            // → Iterator[K]
m.values();          // → Iterator[V]

// Set[T] — unique values
let s = Set::new();
s.insert(item);
s.contains(item);
s.union(other);
s.intersection(other);

// Deque[T] — double-ended queue
let q = Deque::new();
q.push_back(item);
q.push_front(item);
q.pop_front();       // → Option[T]
```

Files sửa:
- Tạo `stdlib/vec.ol`, `stdlib/map.ol`, `stdlib/set.ol`, `stdlib/deque.ol`
- `exec/vm.rs` — native backing cho Vec/Map/Set/Deque (performance)
- Nâng cấp Array builtins hiện tại → Vec

Test: 30+ tests

#### AI-B: String + Byte + Math stdlib

**B5. String nâng cấp**
```
// String interpolation
let msg = f"Hello {name}, you have {count} items";

// Slice
let sub = text[2..5];

// Regex-lite (pattern matching, không full regex)
let matched = text.matches("fire*");

// StringBuilder cho performance
let sb = StringBuilder::new();
sb.append("hello");
sb.append(f" {name}");
let result = sb.build();
```

Files sửa:
- `lang/syntax.rs` — parse `f"..."` interpolation, `[start..end]` slice
- `exec/vm.rs` — string slice, pattern match, StringBuilder
- Tạo `stdlib/string.ol`

Test: 15+ tests

**B6. Byte operations + Serialization**
```
// Byte array
let buf = Bytes::new(12);
buf.set_u8(0, 0x01);
buf.set_u32_be(4, address);
let val = buf.get_u16_le(8);

// Pack/Unpack (cho ISL, Wire format)
let frame = pack("4B 4B 1B 3B", from, to, msg_type, payload);
let (from, to, msg_type, payload) = unpack("4B 4B 1B 3B", frame);

// Bitwise operations (đã có cmp_and/or/xor, nâng cấp)
let mask = flags & 0xFF;
let shifted = value << 3;
```

Files sửa:
- Tạo `stdlib/bytes.ol` — Bytes type
- `exec/vm.rs` — byte get/set, pack/unpack, bit shift
- `exec/ir.rs` — Op: `BitShiftL`, `BitShiftR`, `BitAnd`, `BitOr`, `BitXor`

Test: 20+ tests

**B7. Math stdlib**
```
mod math;

pub const PI = 3.14159265358979;
pub const PHI = 1.61803398874989;    // golden ratio
pub const PHI_INV = 0.61803398874989;

pub fn sqrt(x) { __hyp_sqrt(x) }
pub fn sin(x)  { __hyp_sin(x) }
pub fn cos(x)  { __hyp_cos(x) }
pub fn pow(base, exp) { __hyp_pow(base, exp) }
pub fn clamp(x, lo, hi) { min(max(x, lo), hi) }

// Fibonacci sequence (xuyên suốt HomeOS)
pub fn fib(n) {
  if n <= 1 { n }
  else {
    let a = 0;
    let b = 1;
    loop n - 1 {
      let tmp = b;
      b = a + b;
      a = tmp;
    }
    b
  }
}
```

Files: tạo `stdlib/math.ol`

Test: 10+ tests

---

### Phase 4: Concurrency & Self-Hosting (tuần 7-8)

**Tại sao cuối:** Async cần module + trait + closure. Self-hosting cần mọi thứ trước đó.

#### AI-A: Channel Concurrency + Compile Pipeline

**A8. Channel-based concurrency (phù hợp ISL)**
```
// Channel = typed pipe giữa 2 spawn
let (tx, rx) = channel();

spawn {
  let data = read_sensor();
  tx.send(data);
}

spawn {
  let data = rx.recv();  // block until data
  process(data);
}

// Select (multi-channel wait)
select {
  msg from rx1 => { handle_sensor(msg) },
  msg from rx2 => { handle_command(msg) },
  timeout 1000 => { idle() },
}
```

Files sửa:
- `exec/ir.rs` — Op: `ChanNew`, `ChanSend`, `ChanRecv`, `Select`
- `exec/vm.rs` — channel queue, spawn scheduler, select mechanism
- `lang/syntax.rs` — parse `channel()`, `select { ... }`
- `lang/semantic.rs` — validate channel usage

Test: 20+ tests cho send/recv, select, timeout, deadlock detection

**A9. Compiler self-hosting preparation**
```
// Olang compiler viết bằng Olang
// Phase này: viết lexer + parser bằng Olang

mod olang.bootstrap.lexer;

type Token {
  kind: TokenKind,
  text: Str,
  line: Num,
  col: Num,
}

union TokenKind {
  Keyword { name: Str },
  Ident { name: Str },
  Number { value: Num },
  String { value: Str },
  Symbol { ch: Str },
  Eof,
}

pub fn tokenize(source: Str) -> Vec[Token] {
  // ...
}
```

Files: tạo `stdlib/bootstrap/lexer.ol`, `stdlib/bootstrap/parser.ol`

Test: lexer tokenize Olang source → tokens, parser tokens → AST

#### AI-B: Stdlib hoàn chỉnh + Migration tools

**B8. IO + Platform stdlib**
```
mod io;

// File operations (append-only per QT9)
pub fn read_file(path: Str) -> Result[Str] { ... }
pub fn append_file(path: Str, data: Bytes) -> Result[Num] { ... }

// Console
pub fn print(msg: Str) { ... }
pub fn println(msg: Str) { ... }

mod platform;

// Platform detection (thay thế hal crate)
pub fn arch() -> Str { ... }        // "x86_64", "aarch64", "riscv64"
pub fn os() -> Str { ... }          // "linux", "macos", "windows", "bare"
pub fn memory_total() -> Num { ... }
```

Files: tạo `stdlib/io.ol`, `stdlib/platform.ol`

Test: 10+ tests

**B9. Migration scaffolding**
```
// Tool: chuyển Rust crate → Olang module
// Không auto-convert, mà tạo skeleton + type stubs

// Input: crates/silk/src/graph.rs
// Output: silk/graph.ol (skeleton)

mod silk.graph;

type SilkGraph {
  nodes: Vec[u64],
  edges: Vec[SilkEdge],
}

type SilkEdge {
  from: Num,
  to: Num,
  kind: EdgeKind,
  weight: Num,
  emotion: EmotionTag,
}

impl SilkGraph {
  fn new() -> SilkGraph { ... }           // TODO: migrate
  fn add_edge(mut self, e: SilkEdge) { ... }  // TODO: migrate
  fn co_activate(mut self, a, b, w) { ... }    // TODO: migrate
}
```

Files: tạo `tools/migrate/` — Rust AST → Olang skeleton generator

Test: generate skeleton cho 1 crate, verify syntax valid

**B10. Test framework bằng Olang**
```
mod test;

pub fn assert_eq(a, b) {
  if a != b {
    panic(f"assert_eq failed: {a} != {b}");
  }
}

pub fn assert_ok(result) {
  match result {
    Ok { .. } => { },
    Err { msg } => { panic(f"expected Ok, got Err: {msg}") },
  }
}

// Test runner
pub fn run_tests(tests: Vec[TestCase]) {
  let passed = 0;
  let failed = 0;
  for t in tests {
    try {
      t.run();
      passed = passed + 1;
    } catch {
      failed = failed + 1;
      println(f"FAIL: {t.name}");
    }
  }
  println(f"{passed} passed, {failed} failed");
}
```

Files: tạo `stdlib/test.ol`

Test: self-test (test framework tests itself)

---

## Phân công tổng hợp

```
Phase   AI-A (Language Core)              AI-B (Runtime & Ecosystem)
──────────────────────────────────────────────────────────────────────
  1     A1. Struct/Record type            B1. Method blocks (impl)
        A2. Enum/Union type               B2. Visibility modifiers
        A3. Option/Result builtins
──────────────────────────────────────────────────────────────────────
  2     A4. Trait system                  B3. Module system
        A5. Generics                      B4. Closures + lambdas
──────────────────────────────────────────────────────────────────────
  3     A6. Iterator protocol             B5. String nâng cấp
        A7. Collections stdlib            B6. Byte ops + Serialization
                                          B7. Math stdlib
──────────────────────────────────────────────────────────────────────
  4     A8. Channel concurrency           B8. IO + Platform stdlib
        A9. Compiler self-hosting         B9. Migration scaffolding
                                          B10. Test framework
```

### Dependencies (thứ tự trong phase)

```
Phase 1: A1 → A2 → A3 (sequential, enum cần struct)
          B1 depends on A1 (methods cần struct)
          B2 independent

Phase 2: A4 depends on A1+A2 (trait cần type)
          A5 depends on A4 (generics cần trait bounds)
          B3 depends on B2 (module cần visibility)
          B4 independent

Phase 3: A6 depends on A4+B4 (Iterator = trait + closure)
          A7 depends on A5+A6 (Vec[T] = generic + iterator)
          B5, B6, B7 independent

Phase 4: A8 depends on B4 (channel callback = closure)
          A9 depends on ALL previous
          B8, B9, B10 independent
```

---

## Files sẽ sửa (tổng hợp)

### Sửa (existing)
```
crates/olang/src/lang/syntax.rs      — ~500 lines thêm (parse type, union, impl, trait, mod, use, lambda, select)
crates/olang/src/lang/semantic.rs    — ~400 lines thêm (validate types, traits, modules, generics)
crates/olang/src/lang/alphabet.rs    — ~50 lines thêm (new keywords: type, union, impl, trait, mod, use, pub, mut)
crates/olang/src/exec/ir.rs          — ~150 lines thêm (new opcodes)
crates/olang/src/exec/vm.rs          — ~600 lines thêm (struct/union/trait/channel/iterator ops)
crates/olang/src/exec/compiler.rs    — ~200 lines thêm (compile new opcodes → C/Rust/WASM)
```

### Tạo mới
```
crates/olang/src/exec/module.rs      — ModuleLoader, ModuleCache (~300 lines)
stdlib/math.ol                       — math constants + functions
stdlib/string.ol                     — string utilities
stdlib/bytes.ol                      — byte manipulation
stdlib/io.ol                         — file + console
stdlib/platform.ol                   — platform detection
stdlib/test.ol                       — test framework
stdlib/iterator.ol                   — Iterator trait
stdlib/vec.ol                        — Vec[T]
stdlib/map.ol                        — Map[K,V]
stdlib/set.ol                        — Set[T]
stdlib/deque.ol                      — Deque[T]
stdlib/bootstrap/lexer.ol            — self-hosting lexer
stdlib/bootstrap/parser.ol           — self-hosting parser
tools/migrate/                       — Rust→Olang skeleton generator
```

---

## Validation per phase

```
Phase 1 done khi:
  ✅ type Vec3 { x: Num } — create, field access, nested
  ✅ union Result { Ok{v}, Err{msg} } — create, match destructure
  ✅ impl Vec3 { fn length(self) {...} } — method call v.length()
  ✅ pub/private — field access denied across module
  ✅ Option/Result + ? operator
  ✅ cargo test --workspace passes (existing 1786+ tests)

Phase 2 done khi:
  ✅ trait Skill { fn execute(self, ctx) } — define, impl, dynamic dispatch
  ✅ type Container[T] { items: Vec[T] } — generic instantiation
  ✅ mod silk.graph; use silk.graph.SilkGraph; — module import/export
  ✅ let f = |x| { x * 2 }; f(21) == 42 — closure works
  ✅ cargo test --workspace passes

Phase 3 done khi:
  ✅ [1,2,3].filter(|x|{x>1}).map(|x|{x*2}).collect() == [4,6]
  ✅ Vec, Map, Set, Deque operations
  ✅ f"hello {name}" string interpolation
  ✅ Bytes pack/unpack cho ISL frame
  ✅ math.fib(11) == 89
  ✅ cargo test --workspace passes

Phase 4 done khi:
  ✅ channel send/recv giữa 2 spawn
  ✅ select multi-channel
  ✅ Olang lexer viết bằng Olang — tokenize chính nó
  ✅ Migration tool sinh skeleton cho silk crate
  ✅ Test framework tự test chính nó
  ✅ cargo test --workspace passes
```

---

## Sau Phase 4: Migration Path

```
Thứ tự migrate Rust → Olang (dựa trên dependency graph):

1. ucd         → giữ Rust (build.rs đọc UnicodeData.txt, compile-time)
2. olang/mol   → giữ Rust (core engine, performance critical)
3. olang/exec  → giữ Rust (VM, compiler — tự host dần)
4. isl         → migrate đầu tiên (12-byte protocol, đơn giản)
5. silk        → migrate (graph + hebbian, dùng Olang collections)
6. context     → migrate (emotion pipeline, dùng Olang trait)
7. agents      → migrate (LeoAI, skills, dùng Olang module)
8. memory      → migrate (STM, dream, dùng Olang collections)
9. hal         → migrate (platform detect, dùng Olang FFI)
10. vsdf       → migrate (SDF + render, dùng Olang math)
11. runtime    → migrate cuối (entry point, depends on all)

Kết quả: HomeOS kernel (ucd + olang/mol + olang/exec) = Rust
          HomeOS userspace (mọi thứ khác) = Olang
          → Giống Linux: kernel C, userspace bất kỳ
```
