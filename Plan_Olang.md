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

## Trạng thái hiện tại (2026-03-18)

```
Phase   Task                               Status      Ghi chú
──────────────────────────────────────────────────────────────────────────
  1     A1. Struct/Record type             ✅ DONE     Dict-backed, __struct_def/__struct_tag
        A2. Enum/Union type                ✅ DONE     __enum_def/__enum_unit/__enum_payload
        A3. Option/Result builtins         ⚠️ PARTIAL  ?? có, ? propagation chưa, builtin type chưa
        B1. Method blocks (impl)           ✅ DONE     __StructName_methodName mangling
        B2. Visibility modifiers           ✅ DONE     pub keyword
──────────────────────────────────────────────────────────────────────────
  2     A4. Trait system                   ✅ DONE     Registry-based dispatch, default methods
        A5. Generics                       ✅ DONE     Type erasure, trait bounds validation
        B3. Module system                  ✅ DONE     use/mod syntax, VmEvent::UseModule
        B4. Closures + lambdas             ✅ DONE     Closure/CallClosure opcodes
──────────────────────────────────────────────────────────────────────────
  3     A6. Iterator protocol              ❌ CHƯA     Không có Iterator trait, không lazy eval
        A7. Collections stdlib             ⚠️ PARTIAL  Array 16 builtins, Dict 8, thiếu Set/Deque
        B5. String nâng cấp               ✅ DONE     f-string, matches, chars, repeat, pad
        B6. Byte ops + Serialization       ✅ DONE     Bytes get/set, pack/unpack, bitwise
        B7. Math stdlib                    ✅ DONE     tan/atan/exp/ln/clamp/fib/PI/PHI
──────────────────────────────────────────────────────────────────────────
  4     A8. Channel concurrency            ✅ DONE     channel/send/recv + select + spawn
        A9. Compiler self-hosting          ✅ DONE     bootstrap/lexer.ol + parser.ol
        B8. IO + Platform stdlib           ✅ DONE     platform_arch/os/memory + stdlib files
        B9. Migration scaffolding          ✅ DONE     tools/migrate/ Rust→Olang skeleton
        B10. Test framework                ✅ DONE     assert_eq/ne/true + panic + stdlib/test.ol
```

### Còn thiếu để Olang hoàn thiện

```
CRITICAL (chặn migration):
  ① A3 hoàn thiện: ? error propagation + builtin Option/Result + methods
  ② A6 Iterator:   Iterator trait + .next() + lazy chaining + collect()
  ③ A7 hoàn thiện: Set + Deque builtins + stdlib .ol files
  ④ Module resolve: use/mod parse xong nhưng CHƯA resolve path → load file
  ⑤ String slice:  str[start..end] syntax (có str_substr nhưng chưa có [..])
  ⑥ Compiler:      Closure + Channel compile → C/Rust/WASM (hiện chỉ stubs)

SHOULD-HAVE (usability):
  ⑦ Stdlib .ol:    math.ol, string.ol, bytes.ol (builtins có, .ol chưa)
  ⑧ Error type:    Typed errors, not just string messages
  ⑨ Array chain:   arr.filter(...).map(...) — method chaining trên array
```

---

## Phase 5: Completion & Polish (tuần 9-10)

**Mục tiêu:** Hoàn thiện tất cả gaps còn lại, Olang sẵn sàng cho migration.

#### AI-A: Error propagation + Iterator + Module resolution

**A10. Error propagation `?` operator**
```
// ? trên Result: nếu Err → return Err ngay, nếu Ok → unwrap
fn read_config(path) -> Result {
  let data = file_read(path)?;       // return Err nếu fail
  let parsed = parse_json(data)?;    // return Err nếu fail
  Ok(parsed)
}

// ? trên Option: nếu None → return None ngay
fn first_name(user) -> Option {
  let profile = user.profile?;
  let name = profile.first_name?;
  Some(name)
}
```

Files sửa:
- `lang/syntax.rs` — parse postfix `?` operator (Expr::TryPropagate)
- `lang/semantic.rs` — lower ? → check tag → Jz to early return
- `exec/ir.rs` — Op::TryUnwrap (check enum tag, jump if Err/None)
- `exec/vm.rs` — unwrap Ok/Some payload hoặc early return Err/None

Test: 10+ tests cho ? trên Result, ? trên Option, chained ?, nested fn

**A11. Builtin Option/Result types + methods**
```
// Builtin — không cần user define
let x = Some(42);
let y = None;
let r = Ok("data");
let e = Err("failed");

// Methods
x.is_some()       // → true
x.is_none()       // → false
x.unwrap()        // → 42 (panic nếu None)
x.unwrap_or(0)    // → 42 (hoặc 0 nếu None)
x.map(|v| { v * 2 })  // → Some(84)
r.is_ok()         // → true
r.is_err()        // → false
r.map_err(|e| { f"wrapped: {e}" })
```

Files sửa:
- `lang/semantic.rs` — register Some/None/Ok/Err as builtin constructors
- `exec/vm.rs` — builtins: __opt_is_some, __opt_is_none, __opt_unwrap,
                  __opt_map, __res_is_ok, __res_is_err, __res_map_err

Test: 15+ tests

**A12. Iterator protocol + lazy chaining**
```
trait Iterator {
  fn next(mut self) -> Option;
}

// Array auto-implements Iterator
let result = [1, 2, 3, 4, 5]
  .iter()
  .filter(|x| { x > 2 })
  .map(|x| { x * 10 })
  .collect();
// → [30, 40, 50]

// Custom iterator
type Range { current: Num, end: Num }
impl Iterator for Range {
  fn next(mut self) -> Option {
    if self.current < self.end {
      let v = self.current;
      self.current = self.current + 1;
      Some(v)
    } else {
      None
    }
  }
}

// Iterator methods (default implementations)
// .filter(f), .map(f), .fold(init, f), .collect()
// .enumerate(), .zip(other), .take(n), .skip(n)
// .any(f), .all(f), .find(f), .count()
// .sum(), .min(), .max()
// .chain(other), .flat_map(f)
```

Files sửa:
- `exec/vm.rs` — Iterator builtins: __iter_new (wrap array), __iter_next,
                  __iter_filter, __iter_map (lazy transform chain),
                  __iter_collect (eagerly consume → array)
- `lang/semantic.rs` — .iter() method dispatch, iterator chaining resolution
- Tạo `stdlib/iterator.ol` — Iterator trait + default methods

Test: 20+ tests cho iter/filter/map/collect, custom iterator, chaining, zip, enumerate

**A13. Module resolution (import thật)**
```
// file: silk/graph.ol
mod silk.graph;
pub type SilkGraph { ... }
pub fn co_activate(g, a, b, w) { ... }

// file: main.ol
use silk.graph;                    // → load silk/graph.ol
use silk.graph.{ SilkGraph };     // → import specific symbol
use silk.graph.co_activate;       // → import function

let g = SilkGraph::new();
co_activate(g, a, b, 0.8);
```

Files sửa:
- `exec/module.rs` — ModuleResolver: path → parse → compile → cache
                      Circular dependency detection (topological sort)
                      Symbol table per module, pub/private enforcement
- `lang/semantic.rs` — lower Stmt::Use → Op::Import with resolution
- `exec/vm.rs` — handle VmEvent::UseModule → load + execute + merge scope

Test: 15+ tests cho resolve path, circular detect, pub/private cross-module,
      selective import, re-export

#### AI-B: Collections hoàn thiện + Stdlib modules + Compiler backends

**B11. Set + Deque builtins**
```
// Set — unique values, hash-based
let s = Set::new();
s.insert(42);
s.insert(42);          // no-op, already exists
s.contains(42);        // → true
s.len();               // → 1
s.remove(42);
s.union(other_set);
s.intersection(other_set);
s.difference(other_set);
s.to_array();          // → convert to array

// Deque — double-ended queue
let q = Deque::new();
q.push_back(1);
q.push_front(0);
q.pop_front();         // → Some(0)
q.pop_back();          // → Some(1)
q.len();
q.peek_front();
q.peek_back();
```

Files sửa:
- `exec/vm.rs` — builtins: __set_new, __set_insert, __set_contains, __set_remove,
                  __set_len, __set_union, __set_intersection, __set_difference,
                  __set_to_array,
                  __deque_new, __deque_push_back, __deque_push_front,
                  __deque_pop_front, __deque_pop_back, __deque_len,
                  __deque_peek_front, __deque_peek_back
- `lang/semantic.rs` — method dispatch cho Set/Deque

Test: 20+ tests

**B12. String slice syntax + array method chaining**
```
// String slice — [start..end]
let s = "hello world";
let sub = s[0..5];     // → "hello"
let rest = s[6..];     // → "world"
let head = s[..5];     // → "hello"

// Array method chaining (method syntax trên array results)
let result = [1, 2, 3, 4, 5]
  .filter(|x| { x > 2 })
  .map(|x| { x * 10 });
// → [30, 40, 50]
```

Files sửa:
- `lang/syntax.rs` — parse `expr[start..end]`, `expr[start..]`, `expr[..end]`
                      (Expr::Slice { object, start, end })
- `lang/semantic.rs` — lower Slice → __str_slice hoặc __array_slice
- `exec/vm.rs` — __str_slice(str, start, end), enhanced __array_slice
- `lang/semantic.rs` — method chaining: .filter().map() trả array, cho phép
                        tiếp tục gọi method trên kết quả

Test: 15+ tests

**B13. Stdlib .ol modules**
```
Files tạo mới:
  stdlib/math.ol     — wrap math builtins: PI, PHI, sqrt, sin, cos, pow, etc.
  stdlib/string.ol   — wrap string builtins: split, contains, replace, trim, etc.
  stdlib/bytes.ol    — wrap byte builtins: Bytes::new, get/set, pack/unpack
  stdlib/vec.ol      — Vec type + methods (push, pop, map, filter, fold, etc.)
  stdlib/map.ol      — Map type + methods (get, set, keys, values, merge)
  stdlib/set.ol      — Set type + methods (insert, contains, union, intersection)
  stdlib/deque.ol    — Deque type + methods (push_back/front, pop_back/front)
  stdlib/option.ol   — Option type docs + helper functions
  stdlib/result.ol   — Result type docs + helper functions
```

Mỗi file: wrap builtins thành module có doc, export pub functions.

Test: mỗi module 5+ tests (import + sử dụng)

**B14. Compiler backends hoàn thiện**
```
Hiện tại Closure + Channel trong compiler.rs chỉ là stubs/comments.
Cần implement thực tế cho 3 targets:

C backend:
  - Closure → struct { env, fn_ptr }, capture by value
  - Channel → ring buffer + mutex (pthread)
  - Select → poll multiple channels

Rust backend:
  - Closure → Fn trait objects, capture by clone
  - Channel → std::sync::mpsc hoặc crossbeam
  - Select → futures::select! hoặc manual poll

WASM/WAT backend:
  - Closure → funcref + env table
  - Channel → SharedArrayBuffer + Atomics
  - Select → Promise.race pattern
```

Files sửa:
- `exec/compiler.rs` — replace stubs with real implementations cho Closure,
                        CallClosure, ChanNew, ChanSend, ChanRecv, Select,
                        SpawnBegin, SpawnEnd cho C/Rust/WASM

Test: compile → link → execute cho mỗi target

---

## Phân công tổng hợp

```
Phase   AI-A (Language Core)              AI-B (Runtime & Ecosystem)
──────────────────────────────────────────────────────────────────────
  1     A1. Struct/Record type       ✅   B1. Method blocks (impl)    ✅
        A2. Enum/Union type          ✅   B2. Visibility modifiers    ✅
        A3. Option/Result builtins   ⚠️
──────────────────────────────────────────────────────────────────────
  2     A4. Trait system             ✅   B3. Module system           ✅
        A5. Generics                 ✅   B4. Closures + lambdas      ✅
──────────────────────────────────────────────────────────────────────
  3     A6. Iterator protocol        ❌   B5. String nâng cấp         ✅
        A7. Collections stdlib       ⚠️   B6. Byte ops + Serialization✅
                                          B7. Math stdlib             ✅
──────────────────────────────────────────────────────────────────────
  4     A8. Channel concurrency      ✅   B8. IO + Platform stdlib    ✅
        A9. Compiler self-hosting    ✅   B9. Migration scaffolding   ✅
                                          B10. Test framework         ✅
──────────────────────────────────────────────────────────────────────
  5     A10. ? error propagation          B11. Set + Deque builtins
        A11. Builtin Option/Result        B12. String slice + method chain
        A12. Iterator protocol + lazy     B13. Stdlib .ol modules
        A13. Module resolution            B14. Compiler backends hoàn thiện
```

### Dependencies Phase 5

```
Phase 5:
  A10 independent (postfix ? operator)
  A11 depends on A10 (Option/Result methods cần ? để hữu dụng)
  A12 depends on A4+B4 (Iterator = trait + closure) — cả hai đã DONE
  A13 independent (module loader)

  B11 independent (Set + Deque = new VM builtins)
  B12 depends on B5 (string slice mở rộng từ string builtins)
  B13 depends on A13 (stdlib .ol cần module resolution để import)
  B14 independent (compiler codegen)

  Thứ tự đề xuất:
    AI-A: A10 → A11 → A12 (song song A13)
    AI-B: B11 (song song B12) → B13 (sau A13) → B14

  Có thể chạy song song:
    AI-A làm A10+A11+A12+A13
    AI-B làm B11+B12+B14 trước, B13 sau khi A13 xong
```

---

## Files sẽ sửa (tổng hợp)

### Đã sửa (Phase 1-4)
```
crates/olang/src/lang/syntax.rs      — parse type, union, impl, trait, mod, use, lambda, select, f-string, bitwise
crates/olang/src/lang/semantic.rs    — validate types, traits, modules, generics, string/byte/math builtins
crates/olang/src/lang/alphabet.rs    — keywords, f-string, bitwise tokens
crates/olang/src/exec/ir.rs          — struct/union/trait/channel/closure/select opcodes
crates/olang/src/exec/vm.rs          — struct/union/trait/channel/string/byte/math/platform/test builtins
crates/olang/src/exec/compiler.rs    — channel opcodes (stubs)
```

### Cần sửa (Phase 5)
```
crates/olang/src/lang/syntax.rs      — ~80 lines (postfix ?, slice [..])
crates/olang/src/lang/semantic.rs    — ~200 lines (?, Option/Result methods, iter dispatch, slice lower)
crates/olang/src/exec/ir.rs          — ~20 lines (TryUnwrap opcode)
crates/olang/src/exec/vm.rs          — ~400 lines (Option/Result/Iterator/Set/Deque builtins)
crates/olang/src/exec/compiler.rs    — ~300 lines (Closure/Channel real codegen)
crates/olang/src/exec/module.rs      — ~200 lines (ModuleResolver: path resolve, cache, circular detect)
```

### Đã tạo
```
stdlib/io.ol              ✅    stdlib/bootstrap/lexer.ol   ✅
stdlib/platform.ol        ✅    stdlib/bootstrap/parser.ol  ✅
stdlib/test.ol            ✅    tools/migrate/              ✅
```

### Cần tạo (Phase 5)
```
stdlib/math.ol            — math constants + functions
stdlib/string.ol          — string utilities
stdlib/bytes.ol           — byte manipulation
stdlib/vec.ol             — Vec type + methods
stdlib/map.ol             — Map type + methods
stdlib/set.ol             — Set type + methods
stdlib/deque.ol           — Deque type + methods
stdlib/option.ol          — Option helpers
stdlib/result.ol          — Result helpers
stdlib/iterator.ol        — Iterator trait + defaults
```

---

## Validation per phase

```
Phase 1 done khi:
  ✅ type Vec3 { x: Num } — create, field access, nested
  ✅ union Result { Ok{v}, Err{msg} } — create, match destructure
  ✅ impl Vec3 { fn length(self) {...} } — method call v.length()
  ✅ pub/private — field access denied across module
  ⚠️ Option/Result + ? operator — ?? có, ? chưa
  ✅ cargo test --workspace passes

Phase 2 done khi:
  ✅ trait Skill { fn execute(self, ctx) } — define, impl, dynamic dispatch
  ✅ type Container[T] { items: Vec[T] } — generic instantiation
  ✅ mod silk.graph; use silk.graph.SilkGraph; — syntax có, resolve chưa
  ✅ let f = |x| { x * 2 }; f(21) == 42 — closure works
  ✅ cargo test --workspace passes

Phase 3 done khi:
  ⚠️ [1,2,3].filter(|x|{x>1}).map(|x|{x*2}).collect() — eager ok, lazy chưa
  ⚠️ Vec, Map có — Set, Deque chưa
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

Phase 5 done khi:
  ▢ file_read(path)? — ? propagation hoạt động
  ▢ Some(42).map(|x| { x * 2 }) == Some(84)
  ▢ None.unwrap_or(0) == 0
  ▢ [1,2,3].iter().filter(|x|{x>1}).map(|x|{x*2}).collect() == [4,6] — lazy
  ▢ Set::new(); s.insert(42); s.contains(42) — Set hoạt động
  ▢ Deque::new(); q.push_back(1); q.pop_front() — Deque hoạt động
  ▢ "hello"[0..3] == "hel" — string slice
  ▢ use math; math.sqrt(144) — module resolve thật
  ▢ Closure compile → C/Rust/WASM (không chỉ stubs)
  ▢ cargo test --workspace passes
```

---

## Sau Phase 5: Migration Path

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
