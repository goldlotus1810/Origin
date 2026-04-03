# OLANG LANGUAGE SPECIFICATION v2.0

> Tài liệu gốc. Mọi implementation PHẢI tuân theo spec này.
> Ngày: 2026-04-03. Tác giả: Lupin + Nox.

---

## 1. TỔNG QUAN

Olang là ngôn ngữ lập trình được thiết kế cho AI tự sửa và tự cải tiến.

**Nguyên tắc thiết kế:**
- Mọi giá trị là MolecularChain hoặc số hoặc chuỗi hoặc mảng hoặc dict
- Confidence (0.0→1.0) thay thế boolean cứng
- Hàm có thể đọc source code của chính nó
- State sống sót qua restart
- UTF-8 native, tiếng Việt first-class

**Nguồn gốc:**
- Syntax: Rust-inspired (let, fn, match, struct, enum, trait, impl)
- Molecular: Từ Origin Rust codebase (MolecularChain, P_weight, LCA, Silk)
- 7 primitives mới: confidence, derivative, send/receive, vec/mat, persist, fn_source, eval
- Compilation target: Origin VM bytecode (primary), C (bootstrap)

---

## 2. LEXICAL GRAMMAR

### 2.1 Character Set

Source code là UTF-8. Mọi identifier có thể chứa Unicode letters.

```
letter     = Unicode Letter category (Lu, Ll, Lt, Lm, Lo)
digit      = '0'..'9'
hex_digit  = digit | 'a'..'f' | 'A'..'F'
ident_start = letter | '_'
ident_char  = letter | digit | '_'
```

### 2.2 Whitespace và Comments

```
whitespace = ' ' | '\t' | '\r' | '\n'
line_comment = '//' ... '\n'
block_comment = '/*' ... '*/'
```

### 2.3 Keywords (32)

```
let     fn      if      else    match   
while   for     in      loop    break   
continue return  emit    try     catch   
throw   struct  enum    trait   impl    
self    pub     mut     use     from    
spawn   channel select  timeout
import  as      true    false
```

### 2.4 Token Types

```rust
enum Token {
    // Literals
    Ident(String),          // biến, tên hàm
    Int(i64),               // 42, 0xFF
    Float(f64),             // 3.14, 1e-5
    Str(String),            // "hello"
    FStr(Vec<FStrPart>),    // f"hello {name}"
    Bool(bool),             // true, false
    
    // Molecular literals
    MolLiteral {            // mol{ S=1 R=2 V=128 A=128 T=3 }
        s: u8, r: u8, v: u8, a: u8, t: u8
    },
    
    // Operators — Arithmetic
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    
    // Operators — Comparison
    Eq,         // ==
    Ne,         // !=
    Lt,         // <
    Gt,         // >
    Le,         // <=
    Ge,         // >=
    
    // Operators — Logic
    And,        // &&
    Or,         // ||
    Not,        // !
    
    // Operators — Bitwise
    BitAnd,     // &
    BitOr,      // |
    BitXor,     // ^
    BitNot,     // ~
    Shl,        // <<
    Shr,        // >>
    
    // Operators — Assignment
    Assign,     // =
    PlusAssign, // +=
    MinusAssign,// -=
    StarAssign, // *=
    SlashAssign,// /=
    
    // Operators — Special
    Pipe,       // |>  (pipe forward)
    DblQuestion,// ??  (unwrap or default)
    Question,   // ?   (try propagate)
    Arrow,      // ->  (return type annotation)
    FatArrow,   // =>  (match arm)
    DotDot,     // ..  (range)
    ColonColon, // ::  (path separator)
    
    // Operators — Molecular/Relation (18 loại, dùng Unicode)
    Rel(RelOp),
    // ∈ (member), ⊂ (subset), ≡ (equiv), ⊥ (ortho)
    // ∘ (compose/LCA), → (causes), ≈ (similar), ← (derived)
    // ∪ (contains), ∩ (intersects), ∖ (setminus), ↔ (bidir)
    // ⟶ (flows), ⟳ (repeats), ↑ (resolves), ⚡ (trigger)
    // ∥ (parallel), ∂ (context/derivative)
    
    // Delimiters
    LParen, RParen,     // ( )
    LBrace, RBrace,     // { }
    LBracket, RBracket, // [ ]
    Semi,               // ;
    Comma,              // ,
    Dot,                // .
    Colon,              // :
    
    // Special
    Eof,
}
```

### 2.5 Relation Operators (18)

Mỗi RelOp map 1-1 từ Rust gốc:

| Operator | Unicode | Ký hiệu | Ý nghĩa |
|----------|---------|----------|----------|
| Member | U+2208 | ∈ | a thuộc b |
| Subset | U+2282 | ⊂ | a là tập con của b |
| Equiv | U+2261 | ≡ | a tương đương b |
| Ortho | U+22A5 | ⊥ | a trực giao b |
| Compose | U+2218 | ∘ | LCA(a, b) |
| Causes | U+2192 | → | a gây ra b |
| Similar | U+2248 | ≈ | a tương tự b |
| Derived | U+2190 | ← | a bắt nguồn từ b |
| Contains | U+222A | ∪ | a chứa b |
| Intersects | U+2229 | ∩ | a giao b |
| SetMinus | U+2216 | ∖ | a trừ b |
| Bidir | U+2194 | ↔ | a liên kết 2 chiều b |
| Flows | U+27F6 | ⟶ | a chảy tới b |
| Repeats | U+27F3 | ⟳ | a lặp lại |
| Resolves | U+2191 | ↑ | a giải quyết |
| Trigger | U+26A1 | ⚡ | a kích hoạt b |
| Parallel | U+2225 | ∥ | a song song b |
| Context | U+2202 | ∂ | ngữ cảnh / đạo hàm |

Cũng có thể viết bằng ASCII: `member`, `subset`, `equiv`, `compose`, `causes`, v.v.

### 2.6 Number Literals

```
int_literal   = digit+ | '0x' hex_digit+ | '0b' ('0'|'1')+
float_literal = digit+ '.' digit+ ('e' [+-]? digit+)?
```

### 2.7 String Literals

```
string     = '"' (char | escape)* '"'
fstring    = 'f"' (char | escape | '{' expr '}')* '"'
escape     = '\\' ('n' | 't' | 'r' | '\\' | '"' | '0' | 'x' hex hex | 'u' '{' hex+ '}')
```

Unicode escape: `"\u{1F600}"` = 😀

---

## 3. SYNTAX — STATEMENTS

### 3.1 Program

```
program = stmt*
stmt = let_stmt | fn_def | if_stmt | while_stmt | for_stmt 
     | loop_stmt | match_stmt | try_stmt | emit_stmt 
     | return_stmt | break_stmt | continue_stmt | throw_stmt
     | struct_def | enum_def | trait_def | impl_block
     | use_stmt | import_stmt | assign_stmt | spawn_stmt
     | select_stmt | expr_stmt
```

### 3.2 Variable Declaration

```
let_stmt = 'let' ['mut'] IDENT [':' type] '=' expr ';'
         | 'let' '{' IDENT (',' IDENT)* '}' '=' expr ';'
```

```olang
let x = 42;
let mut count = 0;
let name: string = "Nox";
let { a, b } = get_pair();
```

### 3.3 Function Definition

```
fn_def = ['pub'] 'fn' IDENT ['[' type_params ']'] '(' params ')' ['->' type] block
params = (param (',' param)*)?
param = IDENT [':' type] ['=' expr]
type_params = IDENT (':' IDENT)? (',' IDENT (':' IDENT)?)*
```

```olang
fn add(a, b) { return a + b; }
fn greet(name: string) -> string { return f"Hello {name}"; }
fn map[T](arr: [T], f: fn(T) -> T) -> [T] { ... }
pub fn process(data, confidence: float = 0.5) { ... }
```

### 3.4 Control Flow

```olang
// If/else
if condition {
    body;
} else if other {
    body;
} else {
    body;
}

// While
while condition {
    body;
}

// For range
for i in 0..10 {
    emit i;
}

// For each
for item in collection {
    emit item;
}

// Loop N times
loop 100 {
    body;
}

// Match
match value {
    0 => { emit "zero"; }
    1..10 => { emit "small"; }
    x if x > 100 => { emit "big"; }
    _ => { emit "other"; }
}

// Match enum
match result {
    Ok(v) => { emit v; }
    Err(e) => { throw e; }
}

// Match molecular
match mol {
    mol{ V > 5 } => { emit "positive"; }
    mol{ A > 6, V < 2 } => { emit "agitated negative"; }
    _ => { emit "neutral"; }
}

// Break, continue
while true {
    if done { break; }
    if skip { continue; }
}
```

### 3.5 Error Handling

```olang
try {
    let data = file_read("path.txt");
    process(data);
} catch e {
    emit f"Error: {e}";
}

throw "something went wrong";
```

### 3.6 Struct, Enum, Trait, Impl

```olang
struct Point {
    x: float,
    y: float,
}

struct Node[T] {
    value: T,
    children: [Node[T]],
}

enum Result[T] {
    Ok(T),
    Err(string),
}

enum Color {
    Red,
    Green,
    Blue,
    Custom(int, int, int),
}

trait Encodable {
    fn encode(self) -> mol;
    fn decode(m: mol) -> Self;
}

impl Encodable for Point {
    fn encode(self) -> mol {
        return mol_pack(self.x, self.y, 0, 0, 0);
    }
    fn decode(m: mol) -> Point {
        let { s, r, v, a, t } = mol_unpack(m);
        return Point { x: s, y: r };
    }
}

impl Point {
    fn distance(self, other: Point) -> float {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        return sqrt(dx * dx + dy * dy);
    }
}
```

### 3.7 Module System

```olang
// Import file
import "math.ol";
import "stdlib/json.ol" as json;

// Use specific items
use json.{parse, stringify};
use math.{sin, cos, sqrt};

// Public exports
pub fn my_api() { ... }
pub struct MyType { ... }
```

### 3.8 Concurrency

```olang
// Spawn task
spawn {
    let result = heavy_compute();
    send(ch, result);
};

// Channel
let ch = channel();
send(ch, "hello");
let msg = receive(ch);

// Select (multi-channel wait)
select {
    msg from ch1 => { process(msg); }
    msg from ch2 => { handle(msg); }
    timeout 5000 => { emit "timeout"; }
}
```

### 3.9 Emit (Output)

```olang
emit "Hello World";          // output chuỗi
emit 42;                     // output số
emit f"x = {x}, y = {y}";   // output interpolated string
```

---

## 4. SYNTAX — EXPRESSIONS

### 4.1 Expression Precedence (thấp → cao)

| Level | Operators | Associativity |
|-------|-----------|---------------|
| 1 | `\|\|` | left |
| 2 | `&&` | left |
| 3 | `== != < > <= >=` | left |
| 4 | `\|` `^` | left |
| 5 | `&` | left |
| 6 | `<< >>` | left |
| 7 | `+ -` | left |
| 8 | `* / %` | left |
| 9 | `! ~ -` (unary) | right |
| 10 | `.` `[]` `()` | left |
| 11 | `\|>` | left |
| 12 | `??` | left |

### 4.2 Expression Types

```olang
// Literals
42                          // int
3.14                        // float
"hello"                     // string
f"x = {x}"                 // fstring
true                        // bool
[1, 2, 3]                  // array
{ "key": "value" }         // dict
mol{ S=1 R=2 V=5 A=3 T=1 } // molecular literal
(1, "two", 3.0)            // tuple

// Arithmetic
a + b; a - b; a * b; a / b; a % b;

// Comparison (trả về confidence 0.0..1.0, KHÔNG phải bool)
a == b;  // 1.0 nếu bằng, 0.0 nếu khác
a < b;   // 1.0 nếu đúng, 0.0 nếu sai

// Logic (trên confidence values)
a && b;  // min(a, b)
a || b;  // max(a, b)  
!a;      // 1.0 - a

// Pipe
data |> parse |> transform |> emit;
// tương đương: emit(transform(parse(data)));

// Unwrap
let val = maybe_null ?? "default";

// Try propagate
let data = file_read("path")?;  // return Err nếu fail

// Lambda/Closure
let add = |a, b| a + b;
let inc = |x| { let y = x + 1; return y; };
arr |> map(|x| x * 2) |> filter(|x| x > 5);

// Method call
point.distance(other);
arr.push(42);
str.len();

// Field access
point.x;
node.children[0];

// Index
arr[0];
dict["key"];
str[5..10];  // slice

// Struct literal  
Point { x: 1.0, y: 2.0 };

// Enum variant
Color::Red;
Result::Ok(42);
Result::Err("fail");

// Conditional expression
let v = if x > 0 { "pos" } else { "neg" };

// Molecular operations
let m = encode("hello");       // text → molecular chain
let dist = mol_dist(a, b);    // 5D Manhattan distance
let merged = a ∘ b;           // LCA compose
a → b;                        // create causal edge
a ∈ b;                        // member relation
let results = a → ?;          // query: what does a cause?
```

---

## 5. TYPE SYSTEM

### 5.1 Primitive Types

| Type | Size | Mô tả |
|------|------|--------|
| `int` | 64-bit | Signed integer (i64) |
| `float` | 64-bit | IEEE 754 double (f64) |
| `bool` | → confidence | true=1.0, false=0.0 |
| `string` | UTF-8 | Interned, immutable |
| `mol` | 16-bit | P_weight = [S:4][R:4][V:3][A:3][T:2] |
| `chain` | variable | MolecularChain = [mol, mol, ...] |
| `bytes` | variable | Raw byte array |
| `nil` | 0 | Absence of value |

### 5.2 Compound Types

```
array  = '[' type ']'          // [int], [string], [mol]
dict   = '{' type ':' type '}' // {string: int}
tuple  = '(' type ',' ... ')'  // (int, string, float)
fn     = 'fn(' types ')' '->' type  // fn(int, int) -> int
option = type '?'              // int? = int | nil
```

### 5.3 User-Defined Types

```olang
struct Point { x: float, y: float }
enum Option[T] { Some(T), None }
trait Display { fn show(self) -> string; }
```

### 5.4 Confidence Type

**bool KHÔNG TỒN TẠI trong Olang.** Thay vào đó:

```olang
// Comparison trả về float 0.0..1.0
let sure = (x == y);           // 1.0 hoặc 0.0 cho exact match
let maybe = confidence(0.7);   // tạo confidence value

// Logic operations trên confidence
let both = sure && maybe;      // min(1.0, 0.7) = 0.7
let either = sure || maybe;    // max(1.0, 0.7) = 1.0
let not_sure = !sure;          // 1.0 - 1.0 = 0.0

// Bayesian update
let prior = 0.5;
let posterior = bayes_update(prior, evidence, likelihood);

// Threshold
if confidence > 0.8 {
    emit "khá chắc chắn";
} else if confidence > 0.35 {
    emit "có thể";
} else {
    // im lặng — BlackCurtain
}
```

### 5.5 Molecular Type

```olang
// P_weight encoding: u16
// [S:4][R:4][V:3][A:3][T:2] = 16 bits
// S = Shape (0-15), R = Relation (0-15)
// V = Valence (0-7), A = Arousal (0-7), T = Time (0-3)

let m = mol_pack(s, r, v, a, t);
let { s, r, v, a, t } = mol_unpack(m);
let dist = mol_dist(m1, m2);  // weighted Manhattan: |ΔS|+|ΔR|+2|ΔV|+2|ΔA|+4|ΔT|

// MolecularChain = sequence of mol values
let chain = encode("xin chào");  // text → chain of P_weights
let lca = chain_lca(c1, c2);    // Lowest Common Ancestor
let sim = chain_similarity(c1, c2); // 0.0..1.0
```

---

## 6. SEMANTICS

### 6.1 Scoping

Olang sử dụng **lexical scoping** với block scope:

```olang
let x = 1;          // scope: top-level
fn foo() {
    let x = 2;      // scope: foo, shadows outer x
    if true {
        let y = 3;  // scope: if block only
    }
    // y không tồn tại ở đây
}
```

Closures capture bằng reference:

```olang
let counter = 0;
let inc = || { counter = counter + 1; return counter; };
emit inc();  // 1
emit inc();  // 2
```

### 6.2 Mutability

Mặc định immutable. `mut` cho phép reassign:

```olang
let x = 1;
// x = 2;  // ERROR: x is immutable

let mut y = 1;
y = 2;  // OK
```

### 6.3 Error Handling Semantics

`try/catch` bắt mọi error trong block:

```olang
try {
    let x = risky_operation();
} catch e {
    // e là string mô tả error
    emit f"Error: {e}";
}
```

`throw` tạo error:
```olang
throw "invalid argument";
throw f"expected int, got {typeof(x)}";
```

`?` operator propagate error lên caller:
```olang
fn load_config() {
    let data = file_read("config.json")?;  // throw nếu fail
    let config = json_parse(data)?;         // throw nếu parse fail
    return config;
}
```

### 6.4 Pattern Matching

Match kiểm tra theo thứ tự, dừng ở arm đầu tiên match:

```olang
match value {
    // Literal
    42 => { ... }
    
    // Range
    0..10 => { ... }
    
    // Binding with guard
    x if x > 100 => { ... }
    
    // Enum destructure
    Ok(v) => { ... }
    Err(e) => { ... }
    
    // Struct destructure
    Point { x, y } => { ... }
    
    // Molecular constraint
    mol{ V > 5, A > 6 } => { ... }
    
    // Wildcard
    _ => { ... }
}
```

### 6.5 Pipe Operator

`|>` truyền kết quả bên trái làm argument đầu tiên bên phải:

```olang
"hello world" |> split(" ") |> map(uppercase) |> join(", ");
// tương đương: join(map(split("hello world", " "), uppercase), ", ")
```

---

## 7. BUILT-IN FUNCTIONS

### 7.1 Core

```
typeof(x) -> string           // "int", "float", "string", "array", "dict", "mol", "chain", "nil"
len(x) -> int                 // length of string/array/dict/chain
to_string(x) -> string        // convert anything to string
to_int(x) -> int              // parse string to int, or truncate float
to_float(x) -> float          // parse string to float, or promote int
```

### 7.2 String

```
str_len(s) -> int              // byte length
str_char_at(s, i) -> string    // UTF-8 character at byte index i
str_substr(s, start, len) -> string
str_concat(a, b) -> string     // a + b cũng hoạt động
str_split(s, delim) -> [string]
str_join(arr, delim) -> string
str_find(s, needle) -> int     // -1 nếu không tìm thấy
str_replace(s, old, new) -> string
str_upper(s) -> string
str_lower(s) -> string
str_trim(s) -> string
str_starts_with(s, prefix) -> confidence
str_ends_with(s, suffix) -> confidence
str_contains(s, needle) -> confidence
str_bytes(s) -> bytes          // UTF-8 bytes
str_from_bytes(b) -> string    // bytes → UTF-8 string
str_chars(s) -> [string]       // split into characters (codepoints)
str_codepoint_at(s, i) -> int  // Unicode codepoint at character index i
```

### 7.3 Array

```
array_new() -> []
array_push(arr, val) -> []     // trả về arr mới (immutable)
array_pop(arr) -> ([], val)    // trả về (arr mới, giá trị cuối)
array_get(arr, i) -> val
array_set(arr, i, val) -> []
array_len(arr) -> int
array_slice(arr, start, end) -> []
array_map(arr, f) -> []
array_filter(arr, f) -> []
array_reduce(arr, init, f) -> val
array_sort(arr) -> []
array_sort_by(arr, f) -> []
array_find(arr, f) -> val?
array_contains(arr, val) -> confidence
array_reverse(arr) -> []
array_flatten(arr) -> []
array_zip(a, b) -> [(val, val)]
array_enumerate(arr) -> [(int, val)]
```

### 7.4 Dict

```
dict_new() -> {}
dict_get(d, key) -> val?
dict_set(d, key, val) -> {}
dict_has(d, key) -> confidence
dict_keys(d) -> [string]
dict_values(d) -> [val]
dict_remove(d, key) -> {}
dict_len(d) -> int
dict_merge(a, b) -> {}
```

### 7.5 Math

```
sqrt(x), abs(x), floor(x), ceil(x), round(x)
sin(x), cos(x), tan(x), asin(x), acos(x), atan(x), atan2(y, x)
exp(x), log(x), log2(x), log10(x)
pow(base, exp), min(a, b), max(a, b), clamp(x, lo, hi)
PI, E, PHI  // constants: 3.14159..., 2.71828..., 1.61803...
random() -> float              // 0.0..1.0
random_int(lo, hi) -> int
```

### 7.6 Molecular / Brain

```
// Encode/Decode
encode(text) -> chain          // text → MolecularChain via 42 formulas
decode(chain) -> string        // chain → nearest text representation
encode_codepoint(cp) -> mol    // single Unicode codepoint → P_weight

// Molecular operations
mol_pack(s, r, v, a, t) -> mol
mol_unpack(m) -> {s, r, v, a, t}
mol_dist(a, b) -> int          // weighted Manhattan distance (0-70)
mol_compose(a, b) -> mol       // compose 2 molecules (S=union, R=zipf, V=amplify, A=max, T=dominant)

// Chain operations
chain_new() -> chain
chain_push(c, m) -> chain
chain_len(c) -> int
chain_get(c, i) -> mol
chain_lca(a, b) -> chain       // Lowest Common Ancestor
chain_similarity(a, b) -> float // 0.0..1.0
chain_lca_many(chains) -> chain // LCA of N chains

// KnowTree
kt_store(chain, emotion) -> int    // store, return node_id
kt_lookup(mol) -> chain?           // nearest match
kt_nearest(mol, k) -> [chain]     // k-nearest neighbors
kt_walk(start, hops) -> [chain]   // walk graph N hops
kt_learn(text) -> int             // learn text, return node_id

// Silk  
silk_fire(a, b, weight) -> ()      // strengthen edge
silk_decay(elapsed_ms) -> ()       // apply time decay
silk_walk(start, hops) -> [chain]  // walk learned edges
silk_weight(a, b) -> float         // get edge weight 0.0..1.0
silk_neighbors(node) -> [(chain, float)]  // neighbors + weights

// Instincts (trả về confidence)
instinct_honesty(chain) -> float      // 0.0..1.0
instinct_contradiction(a, b) -> float // 0.0..1.0
instinct_causality(a, b) -> float     // 0.0..1.0
instinct_abstraction(chains) -> float // 0.0..1.0
instinct_analogy(a, b, c) -> chain    // A:B :: C:?
instinct_curiosity(chain) -> float    // novelty score
instinct_reflection() -> float        // knowledge quality
```

### 7.7 I/O

```
// File
file_read(path) -> string?
file_write(path, data) -> confidence
file_append(path, data) -> confidence
file_exists(path) -> confidence
file_list(dir) -> [string]
file_remove(path) -> confidence
file_stat(path) -> {size: int, modified: int}

// Standard I/O
print(text) -> ()              // alias cho emit, nhưng không newline
println(text) -> ()            // emit + newline
readline() -> string           // đọc 1 dòng từ stdin
readlines() -> [string]        // đọc tất cả stdin

// Network
tcp_connect(host, port) -> connection?
tcp_listen(port) -> listener?
tcp_read(conn) -> bytes?
tcp_write(conn, data) -> confidence
udp_send(host, port, data) -> confidence
http_get(url) -> {status: int, body: string}?
http_post(url, body) -> {status: int, body: string}?

// Process
exec(cmd) -> {stdout: string, stderr: string, code: int}
env(name) -> string?
exit(code) -> !
```

### 7.8 Persistence (Olang v2 primitive)

```
persist_save(key, value) -> confidence     // save state to disk
persist_load(key) -> val?                  // load state
persist_delete(key) -> confidence
persist_keys() -> [string]
persist_exists(key) -> confidence
```

### 7.9 Self-Modification (Olang v2 primitive)

```
fn_source(f) -> string         // đọc source code của function f
eval(code) -> val?             // compile + execute string as code
compile(source) -> bytes?      // compile source → bytecode (không execute)
self_modify(file, old, new) -> confidence  // sửa source file
self_backup(file) -> confidence            // backup trước khi sửa
```

### 7.10 Confidence (Olang v2 primitive)

```
confidence(x) -> float            // clamp to 0.0..1.0
bayes_update(prior, evidence, likelihood) -> float
conf_and(a, b) -> float           // min(a, b)
conf_or(a, b) -> float            // max(a, b)  
conf_not(a) -> float              // 1.0 - a
conf_weighted(values, weights) -> float  // weighted average
conf_label(x) -> string           // "certain"/"likely"/"possible"/"unlikely"/"unknown"
```

---

## 8. BYTECODE FORMAT

### 8.1 Instruction Encoding

Olang sử dụng **register-based, 32-bit fixed-width** instructions:

```
+--------+--------+--------+--------+
| byte 3 | byte 2 | byte 1 | byte 0 |
+--------+--------+--------+--------+
|   B(8) |   C(8) |   A(8) |  OP(8) |
+--------+--------+--------+--------+

Hoặc format D (16-bit immediate):
+--------+---------+--------+--------+
|      D(16)       |   A(8) |  OP(8) |
+--------+---------+--------+--------+
```

- **OP**: opcode (0-255)
- **A**: destination register
- **B, C**: source registers hoặc constant index
- **D**: 16-bit unsigned immediate (dùng cho jumps, constants, offsets)

### 8.2 Opcode Table (64 opcodes)

**Data (0x00-0x0F):**

| Op | Mnemonic | Format | Mô tả |
|----|----------|--------|--------|
| 0x00 | NOP | - | no operation |
| 0x01 | LOAD_NIL | A | R[A] = nil |
| 0x02 | LOAD_TRUE | A | R[A] = 1.0 |
| 0x03 | LOAD_FALSE | A | R[A] = 0.0 |
| 0x04 | LOAD_INT | AD | R[A] = sign_extend(D) |
| 0x05 | LOAD_CONST | AD | R[A] = K[D] (constant pool) |
| 0x06 | LOAD_GLOBAL | AD | R[A] = G[K[D]] (global by name) |
| 0x07 | STORE_GLOBAL | AD | G[K[D]] = R[A] |
| 0x08 | MOVE | AB | R[A] = R[B] |
| 0x09 | LOAD_UPVAL | AB | R[A] = closure.upvals[B] |
| 0x0A | STORE_UPVAL | AB | closure.upvals[B] = R[A] |
| 0x0B | LOAD_FIELD | ABC | R[A] = R[B].fields[C] |
| 0x0C | STORE_FIELD | ABC | R[A].fields[B] = R[C] |
| 0x0D | LOAD_INDEX | ABC | R[A] = R[B][R[C]] |
| 0x0E | STORE_INDEX | ABC | R[A][R[B]] = R[C] |
| 0x0F | LOAD_MOL | AD | R[A] = mol(K[D]) (16-bit P_weight) |

**Arithmetic (0x10-0x1F):**

| Op | Mnemonic | Format | Mô tả |
|----|----------|--------|--------|
| 0x10 | ADD | ABC | R[A] = R[B] + R[C] |
| 0x11 | SUB | ABC | R[A] = R[B] - R[C] |
| 0x12 | MUL | ABC | R[A] = R[B] * R[C] |
| 0x13 | DIV | ABC | R[A] = R[B] / R[C] |
| 0x14 | MOD | ABC | R[A] = R[B] % R[C] |
| 0x15 | NEG | AB | R[A] = -R[B] |
| 0x16 | SHL | ABC | R[A] = R[B] << R[C] |
| 0x17 | SHR | ABC | R[A] = R[B] >> R[C] |
| 0x18 | BAND | ABC | R[A] = R[B] & R[C] |
| 0x19 | BOR | ABC | R[A] = R[B] \| R[C] |
| 0x1A | BXOR | ABC | R[A] = R[B] ^ R[C] |
| 0x1B | BNOT | AB | R[A] = ~R[B] |
| 0x1C | CONCAT | ABC | R[A] = str(R[B]) + str(R[C]) |

**Comparison (0x20-0x2F):**

| Op | Mnemonic | Format | Mô tả |
|----|----------|--------|--------|
| 0x20 | EQ | ABC | R[A] = (R[B] == R[C]) ? 1.0 : 0.0 |
| 0x21 | NE | ABC | R[A] = (R[B] != R[C]) ? 1.0 : 0.0 |
| 0x22 | LT | ABC | R[A] = (R[B] < R[C]) ? 1.0 : 0.0 |
| 0x23 | LE | ABC | R[A] = (R[B] <= R[C]) ? 1.0 : 0.0 |
| 0x24 | GT | ABC | R[A] = (R[B] > R[C]) ? 1.0 : 0.0 |
| 0x25 | GE | ABC | R[A] = (R[B] >= R[C]) ? 1.0 : 0.0 |
| 0x26 | CONF_AND | ABC | R[A] = min(R[B], R[C]) |
| 0x27 | CONF_OR | ABC | R[A] = max(R[B], R[C]) |
| 0x28 | CONF_NOT | AB | R[A] = 1.0 - R[B] |

**Control Flow (0x30-0x3F):**

| Op | Mnemonic | Format | Mô tả |
|----|----------|--------|--------|
| 0x30 | JMP | D | PC += signed(D) |
| 0x31 | JZ | AD | if R[A] == 0.0: PC += signed(D) |
| 0x32 | JNZ | AD | if R[A] != 0.0: PC += signed(D) |
| 0x33 | CALL | ABC | R[A..A+C-1] = R[B](R[B+1..B+C]) |
| 0x34 | RET | A | return R[A] |
| 0x35 | RET_NIL | - | return nil |
| 0x36 | CLOSURE | AD | R[A] = new_closure(proto[D]) |
| 0x37 | CALL_CLOSURE | ABC | R[A..A+C-1] = R[B](R[B+1..B+C]) |
| 0x38 | TRY_BEGIN | D | push error handler at PC+D |
| 0x39 | CATCH_END | - | pop error handler |
| 0x3A | THROW | A | throw R[A] as error |
| 0x3B | CLOSURE_CAP | AB | closure[A].upvals[next] = R[B] |
| 0x3C | LOOP_INIT | AD | R[A] = D (init loop counter) |
| 0x3D | LOOP_DEC | AD | R[A]--; if R[A] > 0: PC += signed(D) |

**Collection (0x40-0x4F):**

| Op | Mnemonic | Format | Mô tả |
|----|----------|--------|--------|
| 0x40 | NEW_ARRAY | AB | R[A] = new array, capacity R[B] |
| 0x41 | ARRAY_PUSH | AB | R[A].push(R[B]) |
| 0x42 | ARRAY_GET | ABC | R[A] = R[B][R[C]] |
| 0x43 | ARRAY_SET | ABC | R[A][R[B]] = R[C] |
| 0x44 | ARRAY_LEN | AB | R[A] = len(R[B]) |
| 0x45 | NEW_DICT | A | R[A] = new dict |
| 0x46 | DICT_GET | ABC | R[A] = R[B][R[C]] |
| 0x47 | DICT_SET | ABC | R[A][R[B]] = R[C] |
| 0x48 | NEW_STRUCT | AD | R[A] = new struct of type K[D] |
| 0x49 | SLICE | ABC | R[A] = R[B][R[C]..] (+ next instruction for end) |

**Molecular (0x50-0x5F):**

| Op | Mnemonic | Format | Mô tả |
|----|----------|--------|--------|
| 0x50 | MOL_PACK | A,BC | R[A] = pack(R[B..B+4]) (5 dims) |
| 0x51 | MOL_UNPACK | AB | R[A..A+4] = unpack(R[B]) |
| 0x52 | MOL_DIST | ABC | R[A] = dist(R[B], R[C]) |
| 0x53 | MOL_COMPOSE | ABC | R[A] = compose(R[B], R[C]) |
| 0x54 | MOL_ENCODE | AB | R[A] = encode_codepoint(R[B]) |
| 0x55 | MOL_DECODE | AB | R[A] = decode_mol(R[B]) |
| 0x56 | CHAIN_LCA | ABC | R[A] = lca(R[B], R[C]) |
| 0x57 | CHAIN_NEW | A | R[A] = new chain |
| 0x58 | CHAIN_PUSH | AB | R[A].push(R[B]) |
| 0x59 | CHAIN_LEN | AB | R[A] = len(R[B]) |
| 0x5A | CHAIN_GET | ABC | R[A] = R[B].mol_at(R[C]) |
| 0x5B | CHAIN_SIM | ABC | R[A] = similarity(R[B], R[C]) |

**Brain/Knowledge (0x60-0x6F):**

| Op | Mnemonic | Format | Mô tả |
|----|----------|--------|--------|
| 0x60 | KT_STORE | ABC | R[A] = kt_store(R[B], R[C]) |
| 0x61 | KT_LOOKUP | AB | R[A] = kt_lookup(R[B]) |
| 0x62 | KT_NEAREST | ABC | R[A] = kt_nearest(R[B], R[C]) |
| 0x63 | KT_WALK | ABC | R[A] = kt_walk(R[B], R[C]) |
| 0x64 | SILK_FIRE | ABC | silk_fire(R[A], R[B], R[C]) |
| 0x65 | SILK_DECAY | A | silk_decay(R[A]) |
| 0x66 | SILK_WALK | ABC | R[A] = silk_walk(R[B], R[C]) |
| 0x67 | SILK_WEIGHT | ABC | R[A] = silk_weight(R[B], R[C]) |
| 0x68 | DREAM | - | trigger dream cycle |
| 0x69 | INSTINCT | ABC | R[A] = instinct[R[B]](R[C]) |

**I/O (0x70-0x7F):**

| Op | Mnemonic | Format | Mô tả |
|----|----------|--------|--------|
| 0x70 | EMIT | A | output R[A] |
| 0x71 | READLINE | A | R[A] = read line from stdin |
| 0x72 | FILE_READ | AB | R[A] = file_read(R[B]) |
| 0x73 | FILE_WRITE | AB | file_write(R[A], R[B]) |
| 0x74 | FILE_APPEND | AB | file_append(R[A], R[B]) |
| 0x75 | EXEC | AB | R[A] = exec(R[B]) |
| 0x76 | TCP_CONNECT | ABC | R[A] = tcp_connect(R[B], R[C]) |
| 0x77 | TCP_READ | AB | R[A] = tcp_read(R[B]) |
| 0x78 | TCP_WRITE | AB | tcp_write(R[A], R[B]) |
| 0x79 | HTTP_GET | AB | R[A] = http_get(R[B]) |

**System (0xF0-0xFF):**

| Op | Mnemonic | Format | Mô tả |
|----|----------|--------|--------|
| 0xF0 | SPAWN | D | spawn task at PC+D |
| 0xF1 | CHAN_NEW | A | R[A] = new channel |
| 0xF2 | CHAN_SEND | AB | send R[B] to channel R[A] |
| 0xF3 | CHAN_RECV | AB | R[A] = receive from channel R[B] |
| 0xF4 | PERSIST_SAVE | AB | persist_save(R[A], R[B]) |
| 0xF5 | PERSIST_LOAD | AB | R[A] = persist_load(R[B]) |
| 0xF6 | FN_SOURCE | AB | R[A] = source_code(R[B]) |
| 0xF7 | EVAL | AB | R[A] = eval(R[B]) |
| 0xF8 | SELF_MODIFY | ABC | self_modify(R[A], R[B], R[C]) |
| 0xFE | HALT | - | stop execution |
| 0xFF | BUILTIN | AD | R[A] = builtin[D](args...) |

### 8.3 Binary File Format (.olang)

```
Header (32 bytes):
  magic:       [u8; 4]  = "OLAG"
  version:     u8       = 2
  arch:        u8       = 0 (portable bytecode)
  flags:       u16      = 0
  const_offset: u32     = offset to constant pool
  const_size:   u32     = size of constant pool
  code_offset:  u32     = offset to bytecode
  code_size:    u32     = size of bytecode (in instructions)
  proto_count:  u16     = number of function prototypes
  main_proto:   u16     = index of main/entry prototype
  reserved:     [u8; 4] = 0

Constant Pool:
  count: u32
  entries: [ConstEntry; count]
  
  ConstEntry:
    tag: u8
      0x01 = int (i64, 8 bytes)
      0x02 = float (f64, 8 bytes)
      0x03 = string (u32 len + UTF-8 bytes)
      0x04 = mol (u16 P_weight)
      0x05 = chain (u16 len + [u16; len] P_weights)

Function Prototype:
  name_idx:     u16     = index into constant pool (string)
  param_count:  u8
  register_count: u8    = max registers used
  upval_count:  u8
  code_offset:  u32     = offset into bytecode section
  code_len:     u32     = number of instructions
  line_info:    [u16]   = source line per instruction (debug)

Bytecode:
  instructions: [u32; code_size]
```

---

## 9. GRAMMAR SUMMARY (EBNF)

```ebnf
program      = { statement } ;
statement    = let_stmt | fn_def | if_stmt | while_stmt | for_stmt 
             | loop_stmt | match_stmt | try_stmt | emit_stmt
             | return_stmt | break_stmt | continue_stmt | throw_stmt
             | struct_def | enum_def | trait_def | impl_block
             | use_stmt | import_stmt | assign_stmt | spawn_stmt 
             | select_stmt | expr_stmt ;

let_stmt     = "let" ["mut"] IDENT [":" type] "=" expr ";" ;
fn_def       = ["pub"] "fn" IDENT ["[" type_params "]"] "(" [params] ")" ["->" type] block ;
if_stmt      = "if" expr block { "else" "if" expr block } [ "else" block ] ;
while_stmt   = "while" expr block ;
for_stmt     = "for" IDENT "in" expr ".." expr block 
             | "for" IDENT "in" expr block ;
loop_stmt    = "loop" expr block ;
match_stmt   = "match" expr "{" { match_arm } "}" ;
try_stmt     = "try" block "catch" [IDENT] block ;
emit_stmt    = "emit" expr ";" ;
return_stmt  = "return" [expr] ";" ;
break_stmt   = "break" ";" ;
continue_stmt = "continue" ";" ;
throw_stmt   = "throw" expr ";" ;
assign_stmt  = lvalue "=" expr ";" 
             | lvalue "+=" expr ";"
             | lvalue "-=" expr ";"
             | lvalue "*=" expr ";"
             | lvalue "/=" expr ";" ;
struct_def   = "struct" IDENT ["[" type_params "]"] "{" { field_def } "}" ;
enum_def     = "enum" IDENT ["[" type_params "]"] "{" { variant_def } "}" ;
trait_def    = "trait" IDENT ["[" type_params "]"] "{" { fn_sig } "}" ;
impl_block   = "impl" [IDENT "for"] IDENT "{" { fn_def } "}" ;
use_stmt     = "use" path ["." "{" ident_list "}"] ";" ;
import_stmt  = "import" STRING ["as" IDENT] ";" ;
spawn_stmt   = "spawn" block ";" ;
select_stmt  = "select" "{" { select_arm } "}" ;

block        = "{" { statement } "}" ;
match_arm    = pattern "=>" block ;
select_arm   = IDENT "from" expr "=>" block 
             | "timeout" expr "=>" block ;
             
expr         = unary { binop unary } ;
unary        = ["!" | "-" | "~"] postfix ;
postfix      = primary { "." IDENT | "[" expr "]" | "(" [args] ")" | "?" } ;
primary      = INT | FLOAT | STRING | FSTRING | "true" | "false" | "nil"
             | IDENT | "(" expr ")" | "[" [exprs] "]" | "{" [dict_entries] "}"
             | "mol{" mol_fields "}" | "|" [params] "|" (expr | block)
             | "if" expr block "else" block
             | IDENT "::" IDENT ["(" [args] ")"]
             | IDENT "{" [struct_fields] "}" ;

type         = "int" | "float" | "string" | "bool" | "mol" | "chain" | "bytes" | "nil"
             | "[" type "]" | "{" type ":" type "}" | "(" type {"," type} ")"
             | "fn" "(" [types] ")" "->" type | type "?" | IDENT ["[" types "]"] ;
```

---

## 10. RESERVED FOR FUTURE

Những feature CHƯA implement nhưng đã dành chỗ trong design:

- `async`/`await` — async I/O
- `macro` — compile-time code generation
- `unsafe` — raw memory access
- `where` — type constraints
- Generic associated types
- Operator overloading qua traits
- SIMD intrinsics (`vec` / `mat` as v2 primitives)
- Derivative operator `∂` as first-class (v2 primitive, cần thêm thiết kế)

---

---

## 11. EFFECT SYSTEM (từ Rust semantic.rs Phase 6F)

Mỗi function có 1 trong 3 effect kinds:

```c
enum EffectKind {
    EFFECT_PURE   = 0,  // no side effects — pure computation
    EFFECT_EMITS  = 1,  // has output effects (emit statements)
    EFFECT_CAUSES = 2,  // has causal/state effects (Relation::Causes)
};
```

**Suy luận effect tự động:**
- Function chứa `emit` → EFFECT_EMITS
- Function chứa relation `→` (Causes) → EFFECT_CAUSES
- Còn lại → EFFECT_PURE

```olang
fn add(a, b) { return a + b; }           // PURE
fn greet(name) { emit f"Hello {name}"; }  // EMITS
fn learn(text) { kt_learn(text); }         // CAUSES (modifies KnowTree)
```

**Tại sao cần:** Compiler có thể optimize PURE functions (memoize, reorder). CAUSES functions phải chạy đúng thứ tự. Pipeline checkpoint kiểm tra CAUSES functions có pass SecurityGate không.

---

## 12. VALUE SEMANTICS (từ Rust semantic.rs Phase 6D)

Olang suy luận value semantics từ **Time dimension** của molecular literal:

```c
enum ValueSemantics {
    VAL_COPY  = 0,  // default — standard clone
    VAL_COW   = 1,  // T=Static → Copy-on-Write (immutable sharing, copy on mutation)
    VAL_MOVE  = 2,  // T=Fast/Instant → Move (ownership transfer, use-after-move = error)
    VAL_SHARE = 3,  // T=Medium/Slow → Share (reference-counted, multiple readers)
};
```

```olang
let a = encode("📦");   // 📦 T=Static → CoW
let b = a;               // b shares a's data (no copy)
b.modify();              // NOW copy happens (Copy-on-Write)

let msg = encode("⚡");  // ⚡ T=Instant → Move
send(ch, msg);           // msg ownership transferred
// emit msg;             // ERROR: use-after-move

let data = encode("♩");  // ♩ T=Medium → Share
let ref1 = data;         // ref-counted share
let ref2 = data;         // still valid, ref_count=3
```

**Khi không có molecular info → default = Copy (an toàn nhất).**

---

## 13. 7 CORE PRIMITIVES (từ Olang v2 Design)

### 13.1 confidence (đã trong spec — section 5.4)

```olang
let sure = confidence(0.95);
let maybe = bayes_update(0.5, evidence, likelihood);
if sure > 0.8 { emit "chắc chắn"; }
```

### 13.2 derivative (∂) — first-class operator

Numerical differentiation — central difference. Không cần symbolic CAS.

```olang
// Concept: ∂ tính đạo hàm số của function
let f = |x| x * x + 2 * x;
let df = ∂(f);           // df = |x| 2*x + 2
emit df(3);              // 8

// Ứng dụng trong brain: ConversationCurve
let v_curve = |t| valence_at(t);
let v_prime = ∂(v_curve);     // V'(t) = tốc độ thay đổi cảm xúc
let v_double = ∂(v_prime);    // V''(t) = gia tốc cảm xúc

// Implementation: numerical differentiation
// ∂(f)(x) = (f(x + h) - f(x - h)) / (2h), h = 1e-7
```

**Thực hiện:** Central difference. Không cần symbolic differentiation.

```c
// VM opcode: DERIV (0xFA)
// R[A] = numerical_derivative(R[B], R[C])
// R[B] = closure (function), R[C] = x value
double numerical_derivative(VM* vm, Value closure, double x) {
    double h = 1e-7;
    Value xph = val_float(x + h);
    Value xmh = val_float(x - h);
    double fxph = as_float(call_closure(vm, closure, &xph, 1));
    double fxmh = as_float(call_closure(vm, closure, &xmh, 1));
    return (fxph - fxmh) / (2.0 * h);
}
```

### 13.3 send/receive — async messaging

```olang
let ch = channel();
spawn { send(ch, expensive_compute()); };
let result = receive(ch);  // blocks until message arrives

// Multi-channel
select {
    msg from ch1 => { process(msg); }
    msg from ch2 => { handle(msg); }
    timeout 5000 => { emit "timeout"; }
}
```

**Thực hiện:** [CHƯA THIẾT KẾ] — concurrency opcodes chưa có trong VM. Cần thiết kế green thread model trước.

### 13.4 vec/mat — vector/matrix as basic types

vec = OlArray of float. mat = OlArray of vec. Không cần type riêng.

```olang
let v = vec(1.0, 2.0, 3.0);       // 3D vector
let m = mat([1,0], [0,1]);         // 2x2 identity matrix
let dot = v * v;                    // dot product = 14.0
let scaled = v * 2.0;              // element-wise = vec(2,4,6)

// Ứng dụng: Shadow Vector operations
let sv1 = shadow_vector(node1);
let sv2 = shadow_vector(node2);
let similarity = cosine(sv1, sv2);
```

**Thực hiện:** vec = OlArray of float, mat = OlArray of vec. Overload `*` cho dot product. Cần thêm opcodes: VEC_DOT, VEC_SCALE, VEC_ADD, MAT_MUL. 

vec = OlArray of float (§2.3 OlArray). Shadow Vector = float[8] (MOL §13).
SIMD: DEFER — benchmark trước không SIMD, chỉ thêm khi bottleneck.

### 13.5 persist — state survives restart

```olang
// Save
persist_save("boot_count", boot_count + 1);
persist_save("last_session", timestamp());

// Load (returns nil if not found)
let count = persist_load("boot_count") ?? 0;
emit f"Boot #{count}";

// Automatic persist for marked variables
let mut config = persist_load("config") ?? default_config();
// ... modify config ...
persist_save("config", config);  // explicit save
```

**Thực hiện:** → xem SPEC_ORIGIN_VM §8 Persistence (key-value binary format).

### 13.6 fn_source — functions read own source

```olang
fn hello() {
    emit "Hello World";
}

let src = fn_source(hello);
emit src;  // "fn hello() {\n    emit \"Hello World\";\n}"

// Self-modification: read → modify → eval
let new_src = str_replace(src, "Hello", "Xin chào");
eval(new_src);  // defines new version of hello
hello();  // "Xin chào World"
```

**Thực hiện:** OlProto stores source_offset + source_len vào binary file. fn_source() reads back from file.

```c
// OlProto extension
typedef struct {
    // ... existing fields ...
    uint32_t source_offset;  // offset in .olang file
    uint32_t source_len;     // length in bytes
} OlProto;

// VM opcode: FN_SOURCE (0xF6)
Value vm_fn_source(VM* vm, Value closure) {
    OlProto* proto = as_closure(closure)->proto;
    if (proto->source_len == 0) return val_nil();
    // Read source from file
    // ... (implementation depends on file format)
    return val_string(strtab_intern(&vm->strings, source_bytes, proto->source_len));
}
```

### 13.7 eval — compile and execute string as code

```olang
let code = "emit 1 + 2;";
eval(code);  // outputs: 3

// Dynamic function definition
eval("fn square(x) { return x * x; }");
emit square(5);  // 25

// Self-modification with backup
let src = fn_source(my_func);
let new_src = optimize(src);
self_backup("my_func.ol");
eval(new_src);
```

**Thực hiện:** eval() = lexer + parser + codegen + execute trong VM.

```c
// VM opcode: EVAL (0xF7)
Value vm_eval(VM* vm, OlStr* code) {
    // 1. Lex
    TokenList tokens = lex(code->data, code->byte_len);
    // 2. Parse
    ASTNode* ast = parse(&tokens);
    // 3. Codegen
    OlProto* proto = codegen(ast);
    // 4. Execute
    vm_execute(vm, proto);
    // 5. Return last value
    return vm->regs[0];
}
```

**[CẦN COMPILER] — eval cần compiler hoạt động bên trong VM. Đây là circular dependency: VM cần compiler, compiler cần VM. Giải pháp: compiler viết bằng Olang, self-hosting.**

---

## 14. COMPILER PIPELINE

### 14.1 Overview

```
Source (.ol) → Lexer → Tokens → Parser → AST → Semantic Analysis → IR → Codegen → Bytecode (.olang)
```

### 14.2 Lexer (clox-style on-demand scanner)

**Architecture:** clox pattern (Crafting Interpreters). Scanner on-demand — không tạo token list.
Parser gọi `scanToken()` khi cần. 1 token lookahead. ~250 LOC C.

Nguồn: craftinginterpreters.com/scanning-on-demand.html, Wren wren_compiler.c

```c
/* On-demand scanner: không buffer tokens, parser pull khi cần.
 * Giống clox (256 LOC), Wren, Lua — tất cả single-pass.
 * Token trỏ vào source string (zero-copy).
 */
typedef struct {
    int      type;      // token type (§2.4 enum)
    const char* start;  // pointer into source
    int      length;    // byte length
    int      line;
} Token;

typedef struct {
    const char* source;
    const char* current;  // next unscanned byte
    int         line;
} Scanner;

void scanner_init(Scanner* s, const char* source) {
    s->source = source;
    s->current = source;
    s->line = 1;
}

// Scan 1 token — gọi bởi parser khi cần
Token scan_token(Scanner* s) {
    skip_whitespace_and_comments(s);

    Token t = { .start = s->current, .line = s->line };
    if (*s->current == '\0') { t.type = TK_EOF; return t; }

    char c = *s->current++;

    // Identifiers + keywords
    if (is_alpha(c) || c == '_') {
        while (is_alpha(*s->current) || is_digit(*s->current) || *s->current == '_')
            s->current++;
        t.length = (int)(s->current - t.start);
        t.type = identifier_type(t.start, t.length);  // keyword trie check
        return t;
    }

    // Numbers
    if (is_digit(c)) {
        if (c == '0' && (*s->current == 'x' || *s->current == 'X')) {
            s->current++;  // hex
            while (is_hex(*s->current)) s->current++;
        } else {
            while (is_digit(*s->current)) s->current++;
            if (*s->current == '.' && is_digit(s->current[1])) {
                s->current++;
                while (is_digit(*s->current)) s->current++;
            }
        }
        t.length = (int)(s->current - t.start);
        t.type = TK_NUMBER;
        return t;
    }

    // Strings
    if (c == '"') {
        while (*s->current != '"' && *s->current != '\0') {
            if (*s->current == '\\') s->current++;  // skip escape
            if (*s->current == '\n') s->line++;
            s->current++;
        }
        if (*s->current == '"') s->current++;
        t.length = (int)(s->current - t.start);
        t.type = TK_STRING;
        return t;
    }

    // Operators (switch on first char, peek for 2-char ops)
    switch (c) {
        case '(': t.type = TK_LPAREN; break;
        case ')': t.type = TK_RPAREN; break;
        case '{': t.type = TK_LBRACE; break;
        case '}': t.type = TK_RBRACE; break;
        case '[': t.type = TK_LBRACKET; break;
        case ']': t.type = TK_RBRACKET; break;
        case ',': t.type = TK_COMMA; break;
        case '.': t.type = (*s->current == '.') ? (s->current++, TK_DOTDOT) : TK_DOT; break;
        case ';': t.type = TK_SEMI; break;
        case ':': t.type = (*s->current == ':') ? (s->current++, TK_COLONCOLON) : TK_COLON; break;
        case '+': t.type = (*s->current == '=') ? (s->current++, TK_PLUSEQ) : TK_PLUS; break;
        case '-': t.type = (*s->current == '>') ? (s->current++, TK_ARROW) : 
                           (*s->current == '=') ? (s->current++, TK_MINUSEQ) : TK_MINUS; break;
        case '*': t.type = (*s->current == '=') ? (s->current++, TK_STAREQ) : TK_STAR; break;
        case '/': t.type = (*s->current == '=') ? (s->current++, TK_SLASHEQ) : TK_SLASH; break;
        case '%': t.type = TK_PERCENT; break;
        case '!': t.type = (*s->current == '=') ? (s->current++, TK_NE) : TK_NOT; break;
        case '=': t.type = (*s->current == '=') ? (s->current++, TK_EQEQ) :
                           (*s->current == '>') ? (s->current++, TK_FATARROW) : TK_EQ; break;
        case '<': t.type = (*s->current == '=') ? (s->current++, TK_LE) :
                           (*s->current == '<') ? (s->current++, TK_SHL) : TK_LT; break;
        case '>': t.type = (*s->current == '=') ? (s->current++, TK_GE) :
                           (*s->current == '>') ? (s->current++, TK_SHR) : TK_GT; break;
        case '&': t.type = (*s->current == '&') ? (s->current++, TK_AND) : TK_BITAND; break;
        case '|': t.type = (*s->current == '|') ? (s->current++, TK_OR) :
                           (*s->current == '>') ? (s->current++, TK_PIPE) : TK_BITOR; break;
        case '^': t.type = TK_BITXOR; break;
        case '~': t.type = TK_BITNOT; break;
        case '?': t.type = (*s->current == '?') ? (s->current++, TK_DBLQUESTION) : TK_QUESTION; break;
        default:  t.type = TK_ERROR; break;
    }
    t.length = (int)(s->current - t.start);
    return t;
}

// Keyword detection — clox trie pattern (O(1), no hash table)
// switch on first char, then memcmp suffix
static int identifier_type(const char* start, int len) {
    switch (start[0]) {
        case 'b': if (len==5 && !memcmp(start+1,"reak",4)) return TK_BREAK; break;
        case 'c': if (len==5 && !memcmp(start+1,"atch",4)) return TK_CATCH;
                  if (len==8 && !memcmp(start+1,"ontinue",7)) return TK_CONTINUE; break;
        case 'e': if (len==4 && !memcmp(start+1,"mit",3)) return TK_EMIT;
                  if (len==4 && !memcmp(start+1,"lse",3)) return TK_ELSE; break;
        case 'f': if (len==2 && start[1]=='n') return TK_FN;
                  if (len==3 && !memcmp(start+1,"or",2)) return TK_FOR;
                  if (len==5 && !memcmp(start+1,"alse",4)) return TK_FALSE; break;
        case 'i': if (len==2 && start[1]=='f') return TK_IF;
                  if (len==2 && start[1]=='n') return TK_IN;
                  if (len==6 && !memcmp(start+1,"mport",5)) return TK_IMPORT; break;
        case 'l': if (len==3 && !memcmp(start+1,"et",2)) return TK_LET; break;
        case 'm': if (len==5 && !memcmp(start+1,"atch",4)) return TK_MATCH; break;
        case 'r': if (len==6 && !memcmp(start+1,"eturn",5)) return TK_RETURN; break;
        case 's': if (len==6 && !memcmp(start+1,"truct",5)) return TK_STRUCT; break;
        case 't': if (len==4 && !memcmp(start+1,"rue",3)) return TK_TRUE;
                  if (len==3 && !memcmp(start+1,"ry",2)) return TK_TRY;
                  if (len==5 && !memcmp(start+1,"hrow",4)) return TK_THROW; break;
        case 'w': if (len==5 && !memcmp(start+1,"hile",4)) return TK_WHILE; break;
    }
    return TK_IDENT;
}
```

### 14.3 Parser (Pratt table + recursive descent)

**Architecture:** clox/Wren pattern. Pratt table cho expressions, recursive descent cho statements.
Single-pass — parse trực tiếp sang bytecode qua codegen (§14.5). Không tạo AST riêng.

Nguồn: craftinginterpreters.com/compiling-expressions.html, Wren wren_compiler.c

```c
/* Pratt Parser core — 15 dòng logic (Crafting Interpreters)
 * Mỗi token type có: prefix function, infix function, precedence level
 * Driver: parsePrecedence(minPrec) — gọi prefix, rồi loop infix
 */

typedef void (*ParseFn)(Compiler* c, int canAssign);

typedef struct {
    ParseFn prefix;
    ParseFn infix;
    int     precedence;
} ParseRule;

// 14 precedence levels (thấp → cao)
enum {
    PREC_NONE = 0,
    PREC_ASSIGNMENT,    // =
    PREC_PIPE,          // |>
    PREC_OR,            // ||
    PREC_AND,           // &&
    PREC_BITOR,         // |
    PREC_BITXOR,        // ^
    PREC_BITAND,        // &
    PREC_EQUALITY,      // == !=
    PREC_COMPARISON,    // < > <= >=
    PREC_SHIFT,         // << >>
    PREC_TERM,          // + -
    PREC_FACTOR,        // * / %
    PREC_UNARY,         // ! - ~
    PREC_CALL,          // . () []
    PREC_PRIMARY,
};

// Parse rule table — indexed by token type
static ParseRule rules[] = {
    [TK_LPAREN]    = { grouping,   call,      PREC_CALL },
    [TK_LBRACKET]  = { array_lit,  subscript, PREC_CALL },
    [TK_LBRACE]    = { dict_lit,   NULL,      PREC_NONE },
    [TK_MINUS]     = { unary,      binary,    PREC_TERM },
    [TK_PLUS]      = { NULL,       binary,    PREC_TERM },
    [TK_STAR]      = { NULL,       binary,    PREC_FACTOR },
    [TK_SLASH]     = { NULL,       binary,    PREC_FACTOR },
    [TK_PERCENT]   = { NULL,       binary,    PREC_FACTOR },
    [TK_NOT]       = { unary,      NULL,      PREC_NONE },
    [TK_EQEQ]     = { NULL,       binary,    PREC_EQUALITY },
    [TK_NE]        = { NULL,       binary,    PREC_EQUALITY },
    [TK_LT]        = { NULL,       binary,    PREC_COMPARISON },
    [TK_GT]        = { NULL,       binary,    PREC_COMPARISON },
    [TK_LE]        = { NULL,       binary,    PREC_COMPARISON },
    [TK_GE]        = { NULL,       binary,    PREC_COMPARISON },
    [TK_AND]       = { NULL,       and_,      PREC_AND },
    [TK_OR]        = { NULL,       or_,       PREC_OR },
    [TK_PIPE]      = { NULL,       pipe,      PREC_PIPE },
    [TK_NUMBER]    = { number,     NULL,      PREC_NONE },
    [TK_STRING]    = { string,     NULL,      PREC_NONE },
    [TK_IDENT]     = { variable,   NULL,      PREC_NONE },
    [TK_TRUE]      = { literal,    NULL,      PREC_NONE },
    [TK_FALSE]     = { literal,    NULL,      PREC_NONE },
    [TK_DOT]       = { NULL,       dot,       PREC_CALL },
};

// Pratt driver — THE core engine
void parse_precedence(Compiler* c, int min_prec) {
    advance(c);  // consume token → c->previous
    ParseFn prefix = rules[c->previous.type].prefix;
    if (!prefix) { error(c, "Expect expression."); return; }

    int can_assign = (min_prec <= PREC_ASSIGNMENT);
    prefix(c, can_assign);

    while (min_prec <= rules[c->current.type].precedence) {
        advance(c);
        ParseFn infix = rules[c->previous.type].infix;
        infix(c, can_assign);
    }
}

// Statement dispatcher — recursive descent
void compile_statement(Compiler* c) {
    if (match(c, TK_LET))     let_declaration(c);
    else if (match(c, TK_FN))  fn_declaration(c);
    else if (match(c, TK_IF))  if_statement(c);
    else if (match(c, TK_WHILE)) while_statement(c);
    else if (match(c, TK_FOR))  for_statement(c);
    else if (match(c, TK_MATCH)) match_statement(c);
    else if (match(c, TK_TRY))  try_statement(c);
    else if (match(c, TK_RETURN)) return_statement(c);
    else if (match(c, TK_EMIT))  emit_statement(c);
    else if (match(c, TK_THROW)) throw_statement(c);
    else if (match(c, TK_BREAK)) break_statement(c);
    else if (match(c, TK_CONTINUE)) continue_statement(c);
    else if (match(c, TK_IMPORT)) import_statement(c);
    else if (match(c, TK_STRUCT)) struct_declaration(c);
    else if (match(c, TK_LBRACE)) block(c);
    else expression_statement(c);
    // → mỗi hàm trên gọi parse_precedence() cho expressions
    // → rồi gọi codegen (§14.5) để emit bytecode
}

// Compiler state — wraps scanner + codegen
typedef struct Compiler {
    Scanner  scanner;
    Token    previous;
    Token    current;
    Codegen  codegen;     // §14.5
    int      had_error;
    int      panic_mode;  // suppress cascade errors
    struct Compiler* enclosing;  // for nested functions
} Compiler;

void advance(Compiler* c) {
    c->previous = c->current;
    c->current = scan_token(&c->scanner);
}

int match(Compiler* c, int type) {
    if (c->current.type != type) return 0;
    advance(c);
    return 1;
}

void error(Compiler* c, const char* msg) {
    if (c->panic_mode) return;  // suppress cascade
    c->panic_mode = 1;
    c->had_error = 1;
    fprintf(stderr, "[line %d] Error: %s\n", c->previous.line, msg);
}

// Top-level compile
OlProto* compile(const char* source) {
    Compiler c;
    scanner_init(&c.scanner, source);
    cg_init(&c.codegen);
    c.had_error = 0;
    c.panic_mode = 0;
    c.enclosing = NULL;
    advance(&c);  // prime first token

    while (c.current.type != TK_EOF) {
        compile_statement(&c);
    }

    cg_emit(&c.codegen, ENCODE_AD(OP_HALT, 0, 0));
    return c.had_error ? NULL : codegen_build(&c.codegen);
}
```

**Input:** UTF-8 source code
**Output:** OlProto (bytecode + constant pool) — xem §14.5 codegen

```c
// AST node types (map 1-1 từ section 3 + 4)
enum ASTType {
    AST_LET, AST_FN_DEF, AST_IF, AST_WHILE, AST_FOR_RANGE, AST_FOR_EACH,
    AST_LOOP, AST_MATCH, AST_TRY, AST_EMIT, AST_RETURN, AST_BREAK,
    AST_CONTINUE, AST_THROW, AST_STRUCT, AST_ENUM, AST_TRAIT, AST_IMPL,
    AST_USE, AST_IMPORT, AST_ASSIGN, AST_SPAWN, AST_SELECT, AST_EXPR,
    // Expressions
    AST_IDENT, AST_INT, AST_FLOAT, AST_STRING, AST_BOOL, AST_NIL,
    AST_ARRAY, AST_DICT, AST_MOL_LITERAL, AST_TUPLE,
    AST_BINARY, AST_UNARY, AST_CALL, AST_INDEX, AST_FIELD,
    AST_LAMBDA, AST_PIPE, AST_UNWRAP_OR, AST_TRY_PROP,
    AST_IF_EXPR, AST_STRUCT_LITERAL, AST_ENUM_VARIANT,
    AST_METHOD_CALL, AST_SLICE, AST_FSTRING,
};

typedef struct ASTNode {
    enum ASTType type;
    union {
        struct { OlStr* name; struct ASTNode* value; int mutable; } let_stmt;
        struct { OlStr* name; OlStr** params; int param_count; struct ASTNode** body; int body_count; } fn_def;
        struct { struct ASTNode* cond; struct ASTNode** then_body; int then_count; struct ASTNode** else_body; int else_count; } if_stmt;
        struct { struct ASTNode* left; int op; struct ASTNode* right; } binary;
        struct { OlStr* name; struct ASTNode** args; int arg_count; } call;
        // ... etc for each AST type
    };
    uint32_t line;
} ASTNode;

// Recursive descent parser
ASTNode* parse(TokenList* tokens);
```

**Thuật toán:** Recursive descent. Xem section 9 (EBNF grammar) cho production rules.

### 14.4 Semantic Analysis

**Input:** AST
**Output:** Validated AST + scope info + effect info

Kiểm tra (từ Rust semantic.rs):
1. **Scope validation:** biến phải được define trước khi dùng
2. **Mutability check:** immutable biến không thể reassign
3. **Move check:** moved biến không thể dùng lại (Phase 6D)
4. **Effect inference:** function effect = Pure/Emits/Causes (Phase 6F)
5. **Trait conformance:** impl phải có đủ method signatures
6. **Parameter constraints:** molecular constraints validated (Phase 6B)
7. **Loop limit:** loop count ≤ 65536

### 14.5 Codegen

**Input:** Validated AST
**Output:** OlProto (bytecode + constant pool)

```c
// Codegen state
typedef struct {
    uint32_t* code;
    uint32_t  code_len;
    uint32_t  code_capacity;
    Value*    constants;
    uint32_t  const_count;
    uint32_t  const_capacity;
    uint8_t   next_reg;      // next available register
    uint8_t   max_reg;       // max register used
} Codegen;

// Compile expression → register
uint8_t compile_expr(Codegen* cg, ASTNode* expr);

// Compile statement
void compile_stmt(Codegen* cg, ASTNode* stmt);

// Build final prototype
OlProto* codegen_build(Codegen* cg);
```

**Register allocation:** Linear scan. Mỗi expression kết quả vào next_reg++. Khi block exit, reg reset.

**Instruction encode macros** (defined in src/value.h — VM §4.3):
```c
#define ENCODE_ABC(op,a,b,c) ((uint32_t)(op)|((uint32_t)(a)<<8)|((uint32_t)(c)<<16)|((uint32_t)(b)<<24))
#define ENCODE_AD(op,a,d)    ((uint32_t)(op)|((uint32_t)(a)<<8)|((uint32_t)(d)<<16))
```

```c
/* ── CODEGEN — ĐẦY ĐỦ ────────────────────────────────────────────
 *
 * Compile AST → OlProto bytecode.
 * Register-based: mỗi expression result → next_reg.
 * Tham khảo: Lua 5.x codegen (lcode.c), Origin compiler.ol (1711 LOC).
 *
 * Principles:
 * 1. Expression → trả về register chứa result
 * 2. Statement → không trả về register (side effects only)
 * 3. Scope = {name→register} mapping, push/pop khi enter/exit block
 * 4. Jump patching: emit JMP/JZ với offset=0, patch sau khi biết target
 * 5. Nested functions: compile thành OlProto riêng, tham chiếu qua constant pool
 */

/* ── Scope management ──────────────────────────────────────────── */

typedef struct {
    OlStr*  name;
    uint8_t reg;
    int     depth;      // scope depth (0 = top-level)
} Local;

#define MAX_LOCALS 256

typedef struct {
    uint32_t* code;
    uint32_t  code_len;
    uint32_t  code_cap;

    Value*    constants;
    uint32_t  const_count;
    uint32_t  const_cap;

    Local     locals[MAX_LOCALS];
    int       local_count;
    int       scope_depth;

    uint8_t   next_reg;
    uint8_t   max_reg;

    // Break/continue patch lists
    uint32_t  break_patches[64];
    int       break_count;
    uint32_t  continue_target;    // PC of loop start

    // Nested function protos
    OlProto*  sub_protos[256];
    int       sub_proto_count;

    // Parent codegen (for closures)
    struct Codegen* parent;
} Codegen;

// §14.5.2 SAFETY PATTERNS — Bounds checks + malloc guards
// Rule: mọi array có fixed size PHẢI check trước khi push.
//       mọi malloc/realloc PHẢI check NULL.

#define CG_CHECK_BREAK(cg) do { \
    if ((cg)->break_count >= 64) { \
        vm_error(NULL, "Too many break statements (max 64)"); \
        return; \
    } \
} while(0)

#define CG_CHECK_PROTO(cg) do { \
    if ((cg)->sub_proto_count >= 256) { \
        vm_error(NULL, "Too many nested functions (max 256)"); \
        return 0; \
    } \
} while(0)

#define CG_CHECK_LOCAL(cg) do { \
    if ((cg)->local_count >= MAX_LOCALS) { \
        vm_error(NULL, "Too many local variables (max %d)", MAX_LOCALS); \
        return; \
    } \
} while(0)

// Malloc guard: sử dụng cho mọi allocation
#define ALLOC_CHECK(ptr) do { \
    if (!(ptr)) { vm_error(NULL, "Out of memory"); return; } \
} while(0)

#define ALLOC_CHECK_NULL(ptr) do { \
    if (!(ptr)) { vm_error(NULL, "Out of memory"); return NULL; } \
} while(0)

// Áp dụng:
// - cg_emit() gọi CG_CHECK_CODE_CAP trước write
// - break statement gọi CG_CHECK_BREAK trước push
// - compile_function gọi CG_CHECK_PROTO trước push
// - cg_define_local gọi CG_CHECK_LOCAL trước push
// - malloc/realloc kết quả luôn qua ALLOC_CHECK

void cg_init(Codegen* cg) {
    cg->code_cap = 1024;
    cg->code = malloc(cg->code_cap * sizeof(uint32_t));
    cg->code_len = 0;
    cg->const_cap = 256;
    cg->constants = malloc(cg->const_cap * sizeof(Value));
    cg->const_count = 0;
    cg->local_count = 0;
    cg->scope_depth = 0;
    cg->next_reg = 0;
    cg->max_reg = 0;
    cg->break_count = 0;
    cg->continue_target = 0;
    cg->sub_proto_count = 0;
    cg->parent = NULL;
}

// Emit 1 instruction, return its PC index
uint32_t cg_emit(Codegen* cg, uint32_t inst) {
    if (cg->code_len >= cg->code_cap) {
        cg->code_cap *= 2;
        cg->code = realloc(cg->code, cg->code_cap * sizeof(uint32_t));
    }
    uint32_t pc = cg->code_len;
    cg->code[cg->code_len++] = inst;
    return pc;
}

// Add constant, return index
uint32_t cg_add_const(Codegen* cg, Value v) {
    // Check duplicate (optional optimization)
    for (uint32_t i = 0; i < cg->const_count; i++) {
        if (cg->constants[i] == v) return i;
    }
    if (cg->const_count >= cg->const_cap) {
        cg->const_cap *= 2;
        cg->constants = realloc(cg->constants, cg->const_cap * sizeof(Value));
    }
    cg->constants[cg->const_count] = v;
    return cg->const_count++;
}

// Allocate register
uint8_t cg_reg(Codegen* cg) {
    uint8_t r = cg->next_reg++;
    if (cg->next_reg > cg->max_reg) cg->max_reg = cg->next_reg;
    return r;
}

// Free registers back to mark
void cg_free_regs(Codegen* cg, uint8_t mark) {
    cg->next_reg = mark;
}

/* ── Scope ─────────────────────────────────────────────────────── */

void cg_begin_scope(Codegen* cg) {
    cg->scope_depth++;
}

void cg_end_scope(Codegen* cg) {
    // Pop locals in this scope
    while (cg->local_count > 0 &&
           cg->locals[cg->local_count - 1].depth >= cg->scope_depth) {
        cg->local_count--;
        // Register freed implicitly (but we don't reuse in this simple scheme)
    }
    cg->scope_depth--;
}

// Declare local variable → assign register
uint8_t cg_declare_local(Codegen* cg, OlStr* name) {
    uint8_t reg = cg_reg(cg);
    cg->locals[cg->local_count++] = (Local){
        .name = name, .reg = reg, .depth = cg->scope_depth
    };
    return reg;
}

// Resolve local → register, or -1 if not found
int cg_resolve_local(Codegen* cg, OlStr* name) {
    for (int i = cg->local_count - 1; i >= 0; i--) {
        if (cg->locals[i].name == name) return cg->locals[i].reg;
    }
    return -1;  // not a local → try global or upvalue
}

// Resolve upvalue (search parent scopes)
int cg_resolve_upvalue(Codegen* cg, OlStr* name) {
    if (!cg->parent) return -1;
    int local = cg_resolve_local((Codegen*)cg->parent, name);
    if (local >= 0) {
        // Found in parent → capture as upvalue
        // Return upvalue index (to be used with OP_LOAD_UPVAL)
        return local;  // simplified — full version tracks upvalue list
    }
    return -1;
}

// §14.5.1 FREE VARIABLE ANALYSIS — Full upvalue tracking
// Replaces simplified cg_resolve_upvalue above khi implement.
//
// Upvalue = variable từ parent scope captured bởi closure.
// Track list: mỗi function có upvals[] — index → parent register hoặc parent upvalue.
//
// typedef struct {
//     uint8_t index;       // register index in parent (if is_local=1)
//                          // or upvalue index in parent (if is_local=0)
//     uint8_t is_local;    // 1 = parent's local register, 0 = parent's upvalue
// } UpvalueRef;
//
// Thêm vào Codegen struct:
//     UpvalueRef upvals[256];
//     int upval_count;
//
// int cg_resolve_upvalue_full(Codegen* cg, OlStr* name) {
//     if (!cg->parent) return -1;
//
//     // Case 1: variable is a local in immediate parent
//     int local = cg_resolve_local((Codegen*)cg->parent, name);
//     if (local >= 0) {
//         // Mark parent's local as captured (prevents register reuse)
//         ((Codegen*)cg->parent)->locals[local].is_captured = 1;
//         return cg_add_upvalue(cg, (uint8_t)local, 1);
//     }
//
//     // Case 2: variable is already an upvalue in parent (recursive)
//     int upval = cg_resolve_upvalue_full((Codegen*)cg->parent, name);
//     if (upval >= 0) {
//         return cg_add_upvalue(cg, (uint8_t)upval, 0);
//     }
//
//     return -1;  // not found in any enclosing scope → global
// }
//
// int cg_add_upvalue(Codegen* cg, uint8_t index, uint8_t is_local) {
//     // Check if already captured
//     for (int i = 0; i < cg->upval_count; i++) {
//         if (cg->upvals[i].index == index && cg->upvals[i].is_local == is_local)
//             return i;
//     }
//     if (cg->upval_count >= 256) return -1;  // too many upvalues
//     cg->upvals[cg->upval_count] = (UpvalueRef){index, is_local};
//     return cg->upval_count++;
// }
//
// Emit in compile_function (thay [CHƯA] block):
//     for (int i = 0; i < sub_cg.upval_count; i++) {
//         cg_emit(cg, ENCODE_ABC(OP_CLOSURE_CAP,
//             sub_cg.upvals[i].index,
//             sub_cg.upvals[i].is_local, 0));
//     }
//
// Pattern from: clox (Crafting Interpreters, Nystrom) + Lua 5.3 upvalue model.

/* ── Patch jumps ───────────────────────────────────────────────── */

// Emit JZ/JMP with placeholder offset, return PC for patching
uint32_t cg_emit_jump(Codegen* cg, uint8_t op, uint8_t reg) {
    return cg_emit(cg, ENCODE_AD(op, reg, 0));  // offset=0, patch later
}

// Patch jump at 'pc' to jump to current code position
void cg_patch_jump(Codegen* cg, uint32_t pc) {
    int16_t offset = (int16_t)(cg->code_len - pc);
    cg->code[pc] = (cg->code[pc] & 0xFFFF) | ((uint32_t)(uint16_t)offset << 16);
}

/* ── compile_expr: AST expression → register ──────────────────── */

uint8_t compile_expr(Codegen* cg, ASTNode* node) {
    uint8_t save = cg->next_reg;

    switch (node->type) {

    case AST_INT: {
        uint8_t r = cg_reg(cg);
        int32_t val = node->int_val;
        if (val >= -32768 && val <= 32767) {
            cg_emit(cg, ENCODE_AD(OP_LOAD_INT, r, (uint16_t)(int16_t)val));
        } else {
            uint32_t k = cg_add_const(cg, val_int(val));
            cg_emit(cg, ENCODE_AD(OP_LOAD_CONST, r, k));
        }
        return r;
    }

    case AST_FLOAT: {
        uint8_t r = cg_reg(cg);
        uint32_t k = cg_add_const(cg, val_float(node->float_val));
        cg_emit(cg, ENCODE_AD(OP_LOAD_CONST, r, k));
        return r;
    }

    case AST_STRING: {
        uint8_t r = cg_reg(cg);
        uint32_t k = cg_add_const(cg, val_string(node->str_val));
        cg_emit(cg, ENCODE_AD(OP_LOAD_CONST, r, k));
        return r;
    }

    case AST_BOOL: {
        uint8_t r = cg_reg(cg);
        cg_emit(cg, ENCODE_AD(node->bool_val ? OP_LOAD_TRUE : OP_LOAD_FALSE, r, 0));
        return r;
    }

    case AST_NIL: {
        uint8_t r = cg_reg(cg);
        cg_emit(cg, ENCODE_AD(OP_LOAD_NIL, r, 0));
        return r;
    }

    case AST_IDENT: {
        OlStr* name = node->ident_name;
        int local = cg_resolve_local(cg, name);
        if (local >= 0) {
            return (uint8_t)local;  // already in a register
        }
        int upval = cg_resolve_upvalue(cg, name);
        if (upval >= 0) {
            uint8_t r = cg_reg(cg);
            cg_emit(cg, ENCODE_ABC(OP_LOAD_UPVAL, r, (uint8_t)upval, 0));
            return r;
        }
        // Global
        uint8_t r = cg_reg(cg);
        uint32_t k = cg_add_const(cg, val_string(name));
        cg_emit(cg, ENCODE_AD(OP_LOAD_GLOBAL, r, k));
        return r;
    }

    case AST_BINARY: {
        uint8_t left  = compile_expr(cg, node->binary.left);
        uint8_t right = compile_expr(cg, node->binary.right);
        uint8_t out   = cg_reg(cg);

        uint8_t op;
        switch (node->binary.op) {
            case '+':  op = OP_ADD;    break;
            case '-':  op = OP_SUB;    break;
            case '*':  op = OP_MUL;    break;
            case '/':  op = OP_DIV;    break;
            case '%':  op = OP_MOD;    break;
            case TK_EQ:  op = OP_EQ;   break;
            case TK_NE:  op = OP_NE;   break;
            case '<':  op = OP_LT;    break;
            case TK_LE:  op = OP_LE;   break;
            case '>':  op = OP_GT;    break;
            case TK_GE:  op = OP_GE;   break;
            default: op = OP_NOP; break;
        }
        cg_emit(cg, ENCODE_ABC(op, out, left, right));
        cg_free_regs(cg, out + 1);
        return out;
    }

    case AST_UNARY: {
        uint8_t operand = compile_expr(cg, node->unary.operand);
        uint8_t out = cg_reg(cg);
        if (node->unary.op == '-') {
            cg_emit(cg, ENCODE_ABC(OP_NEG, out, operand, 0));
        } else if (node->unary.op == '!') {
            // !x → x == 0
            uint8_t zero = cg_reg(cg);
            cg_emit(cg, ENCODE_AD(OP_LOAD_INT, zero, 0));
            cg_emit(cg, ENCODE_ABC(OP_EQ, out, operand, zero));
            cg_free_regs(cg, out + 1);
        }
        return out;
    }

    case AST_CALL: {
        // Layout: R[fn], R[fn+1]=arg0, R[fn+2]=arg1, ...
        // Then: CALL R[result], R[fn], argc
        OlStr* fn_name = node->call.name;
        int local = cg_resolve_local(cg, fn_name);

        uint8_t fn_reg;
        if (local >= 0) {
            fn_reg = (uint8_t)local;
        } else {
            fn_reg = cg_reg(cg);
            uint32_t k = cg_add_const(cg, val_string(fn_name));
            cg_emit(cg, ENCODE_AD(OP_LOAD_GLOBAL, fn_reg, k));
        }

        // Compile args into consecutive registers after fn_reg
        int argc = node->call.arg_count;
        for (int i = 0; i < argc; i++) {
            uint8_t arg_reg = cg_reg(cg);
            uint8_t src = compile_expr(cg, node->call.args[i]);
            if (src != arg_reg) {
                cg_emit(cg, ENCODE_ABC(OP_MOVE, arg_reg, src, 0));
            }
        }

        uint8_t result = fn_reg;  // reuse fn_reg for result (Lua convention)
        cg_emit(cg, ENCODE_ABC(OP_CALL, result, fn_reg, argc));
        cg_free_regs(cg, result + 1);
        return result;
    }

    case AST_ARRAY: {
        uint8_t arr = cg_reg(cg);
        int count = node->array.elem_count;
        cg_emit(cg, ENCODE_AD(OP_NEW_ARRAY, arr, count > 0 ? count : 8));

        for (int i = 0; i < count; i++) {
            uint8_t elem = compile_expr(cg, node->array.elems[i]);
            cg_emit(cg, ENCODE_ABC(OP_ARRAY_PUSH, arr, elem, 0));
            cg_free_regs(cg, arr + 1);
        }
        return arr;
    }

    case AST_INDEX: {
        uint8_t obj = compile_expr(cg, node->index.object);
        uint8_t idx = compile_expr(cg, node->index.index);
        uint8_t out = cg_reg(cg);
        cg_emit(cg, ENCODE_ABC(OP_ARRAY_GET, out, obj, idx));
        cg_free_regs(cg, out + 1);
        return out;
    }

    case AST_DICT: {
        uint8_t dict = cg_reg(cg);
        int count = node->dict.pair_count;
        cg_emit(cg, ENCODE_AD(OP_NEW_DICT, dict, count > 0 ? count * 2 : 8));

        for (int i = 0; i < count; i++) {
            uint8_t key = compile_expr(cg, node->dict.keys[i]);
            uint8_t val = compile_expr(cg, node->dict.values[i]);
            cg_emit(cg, ENCODE_ABC(OP_DICT_SET, dict, key, val));
            cg_free_regs(cg, dict + 1);
        }
        return dict;
    }

    case AST_FIELD: {
        // obj.field → DICT_GET R[out], R[obj], R[key]
        uint8_t obj = compile_expr(cg, node->field.object);
        uint8_t key = cg_reg(cg);
        uint32_t k = cg_add_const(cg, val_string(node->field.name));
        cg_emit(cg, ENCODE_AD(OP_LOAD_CONST, key, k));
        uint8_t out = cg_reg(cg);
        cg_emit(cg, ENCODE_ABC(OP_DICT_GET, out, obj, key));
        cg_free_regs(cg, out + 1);
        return out;
    }

    case AST_LAMBDA: {
        // Compile body as sub-proto
        Codegen sub;
        cg_init(&sub);
        sub.parent = (struct Codegen*)cg;

        // Declare parameters as locals in sub
        for (int i = 0; i < node->fn_def.param_count; i++) {
            cg_declare_local(&sub, node->fn_def.params[i]);
        }

        // Compile body
        for (int i = 0; i < node->fn_def.body_count; i++) {
            compile_stmt(&sub, node->fn_def.body[i]);
        }
        // Implicit return nil if no explicit return
        cg_emit(&sub, ENCODE_AD(OP_RET_NIL, 0, 0));

        OlProto* proto = codegen_build(&sub);
        proto->param_count = node->fn_def.param_count;

        // Store proto in parent's sub_proto list
        int proto_idx = cg->sub_proto_count;
        cg->sub_protos[cg->sub_proto_count++] = proto;

        // Emit CLOSURE
        uint8_t r = cg_reg(cg);
        uint32_t k = cg_add_const(cg, val_int(proto_idx));
        cg_emit(cg, ENCODE_AD(OP_CLOSURE, r, k));

        // Emit CLOSURE_CAP for each free variable
        // [CHƯA: free variable analysis — cần scan body cho unresolved idents]
        return r;
    }

    case AST_MOL_LITERAL: {
        uint8_t r = cg_reg(cg);
        uint16_t mol = mol_pack(node->mol.s, node->mol.r, node->mol.v,
                                node->mol.a, node->mol.t);
        cg_emit(cg, ENCODE_AD(OP_LOAD_MOL, r, mol));
        return r;
    }

    default:
        // Unknown expression → nil
        {
            uint8_t r = cg_reg(cg);
            cg_emit(cg, ENCODE_AD(OP_LOAD_NIL, r, 0));
            return r;
        }
    }
}

/* ── compile_stmt: AST statement → opcodes ─────────────────────── */

void compile_stmt(Codegen* cg, ASTNode* node) {

    switch (node->type) {

    case AST_LET: {
        // let x = expr
        uint8_t val_reg = compile_expr(cg, node->let_stmt.value);
        if (cg->scope_depth > 0) {
            // Local variable
            uint8_t local = cg_declare_local(cg, node->let_stmt.name);
            if (local != val_reg) {
                cg_emit(cg, ENCODE_ABC(OP_MOVE, local, val_reg, 0));
            }
        } else {
            // Global variable
            uint32_t k = cg_add_const(cg, val_string(node->let_stmt.name));
            cg_emit(cg, ENCODE_AD(OP_STORE_GLOBAL, val_reg, k));
        }
        break;
    }

    case AST_ASSIGN: {
        // x = expr (reassign)
        uint8_t val_reg = compile_expr(cg, node->assign.value);
        int local = cg_resolve_local(cg, node->assign.name);
        if (local >= 0) {
            if (local != val_reg) {
                cg_emit(cg, ENCODE_ABC(OP_MOVE, (uint8_t)local, val_reg, 0));
            }
        } else {
            uint32_t k = cg_add_const(cg, val_string(node->assign.name));
            cg_emit(cg, ENCODE_AD(OP_STORE_GLOBAL, val_reg, k));
        }
        break;
    }

    case AST_EMIT: {
        uint8_t r = compile_expr(cg, node->emit.value);
        cg_emit(cg, ENCODE_AD(OP_EMIT, r, 0));
        break;
    }

    case AST_RETURN: {
        if (node->return_stmt.value) {
            uint8_t r = compile_expr(cg, node->return_stmt.value);
            cg_emit(cg, ENCODE_AD(OP_RET, r, 0));
        } else {
            cg_emit(cg, ENCODE_AD(OP_RET_NIL, 0, 0));
        }
        break;
    }

    case AST_IF: {
        // if cond { then } else { else_body }
        uint8_t cond = compile_expr(cg, node->if_stmt.cond);
        uint32_t jz_pc = cg_emit_jump(cg, OP_JZ, cond);

        cg_begin_scope(cg);
        for (int i = 0; i < node->if_stmt.then_count; i++)
            compile_stmt(cg, node->if_stmt.then_body[i]);
        cg_end_scope(cg);

        if (node->if_stmt.else_count > 0) {
            uint32_t jmp_pc = cg_emit_jump(cg, OP_JMP, 0);
            cg_patch_jump(cg, jz_pc);

            cg_begin_scope(cg);
            for (int i = 0; i < node->if_stmt.else_count; i++)
                compile_stmt(cg, node->if_stmt.else_body[i]);
            cg_end_scope(cg);

            cg_patch_jump(cg, jmp_pc);
        } else {
            cg_patch_jump(cg, jz_pc);
        }
        break;
    }

    case AST_WHILE: {
        // while cond { body }
        uint32_t loop_start = cg->code_len;
        cg->continue_target = loop_start;
        int old_break_count = cg->break_count;

        uint8_t cond = compile_expr(cg, node->while_stmt.cond);
        uint32_t exit_jz = cg_emit_jump(cg, OP_JZ, cond);

        cg_begin_scope(cg);
        for (int i = 0; i < node->while_stmt.body_count; i++)
            compile_stmt(cg, node->while_stmt.body[i]);
        cg_end_scope(cg);

        // Jump back to loop start
        int16_t back = (int16_t)(loop_start - cg->code_len);
        cg_emit(cg, ENCODE_AD(OP_JMP, 0, (uint16_t)back));

        // Patch exit
        cg_patch_jump(cg, exit_jz);

        // Patch breaks
        for (int i = old_break_count; i < cg->break_count; i++)
            cg_patch_jump(cg, cg->break_patches[i]);
        cg->break_count = old_break_count;
        break;
    }

    case AST_FOR_EACH: {
        // for x in arr { body }
        // Desugar: let __arr = arr; let __i = 0; let __len = len(__arr);
        //          while __i < __len { let x = __arr[__i]; body; __i = __i + 1 }
        uint8_t arr_reg = compile_expr(cg, node->for_each.iterable);
        uint8_t i_reg = cg_reg(cg);
        uint8_t len_reg = cg_reg(cg);

        cg_emit(cg, ENCODE_AD(OP_LOAD_INT, i_reg, 0));
        cg_emit(cg, ENCODE_ABC(OP_ARRAY_LEN, len_reg, arr_reg, 0));

        uint32_t loop_start = cg->code_len;
        cg->continue_target = loop_start;
        int old_break_count = cg->break_count;

        // __i < __len
        uint8_t cond = cg_reg(cg);
        cg_emit(cg, ENCODE_ABC(OP_LT, cond, i_reg, len_reg));
        uint32_t exit_jz = cg_emit_jump(cg, OP_JZ, cond);

        cg_begin_scope(cg);

        // let x = __arr[__i]
        uint8_t elem = cg_declare_local(cg, node->for_each.var_name);
        cg_emit(cg, ENCODE_ABC(OP_ARRAY_GET, elem, arr_reg, i_reg));

        for (int i = 0; i < node->for_each.body_count; i++)
            compile_stmt(cg, node->for_each.body[i]);

        cg_end_scope(cg);

        // __i = __i + 1
        uint8_t one = cg_reg(cg);
        cg_emit(cg, ENCODE_AD(OP_LOAD_INT, one, 1));
        cg_emit(cg, ENCODE_ABC(OP_ADD, i_reg, i_reg, one));
        cg_free_regs(cg, one);

        int16_t back = (int16_t)(loop_start - cg->code_len);
        cg_emit(cg, ENCODE_AD(OP_JMP, 0, (uint16_t)back));

        cg_patch_jump(cg, exit_jz);

        for (int i = old_break_count; i < cg->break_count; i++)
            cg_patch_jump(cg, cg->break_patches[i]);
        cg->break_count = old_break_count;
        break;
    }

    case AST_BREAK: {
        cg->break_patches[cg->break_count++] = cg_emit_jump(cg, OP_JMP, 0);
        break;
    }

    case AST_CONTINUE: {
        int16_t back = (int16_t)(cg->continue_target - cg->code_len);
        cg_emit(cg, ENCODE_AD(OP_JMP, 0, (uint16_t)back));
        break;
    }

    case AST_TRY: {
        // try { body } catch(e) { handler }
        uint32_t try_pc = cg_emit(cg, ENCODE_AD(OP_TRY_BEGIN, 0, 0));  // patch later

        cg_begin_scope(cg);
        for (int i = 0; i < node->try_stmt.body_count; i++)
            compile_stmt(cg, node->try_stmt.body[i]);
        cg_end_scope(cg);

        cg_emit(cg, ENCODE_AD(OP_CATCH_END, 0, 0));
        uint32_t jmp_end = cg_emit_jump(cg, OP_JMP, 0);

        // Patch TRY_BEGIN to point to catch
        cg_patch_jump(cg, try_pc);

        cg_begin_scope(cg);
        // Error value is in R[0] (set by THROW handler in VM)
        if (node->try_stmt.catch_var) {
            uint8_t err_reg = cg_declare_local(cg, node->try_stmt.catch_var);
            cg_emit(cg, ENCODE_ABC(OP_MOVE, err_reg, 0, 0));
        }
        for (int i = 0; i < node->try_stmt.catch_count; i++)
            compile_stmt(cg, node->try_stmt.catch_body[i]);
        cg_end_scope(cg);

        cg_patch_jump(cg, jmp_end);
        break;
    }

    case AST_THROW: {
        uint8_t r = compile_expr(cg, node->throw_stmt.value);
        cg_emit(cg, ENCODE_AD(OP_THROW, r, 0));
        break;
    }

    case AST_FN_DEF: {
        // fn name(params) { body } → compile body as sub-proto, store as global
        Codegen sub;
        cg_init(&sub);
        sub.parent = (struct Codegen*)cg;

        for (int i = 0; i < node->fn_def.param_count; i++)
            cg_declare_local(&sub, node->fn_def.params[i]);

        for (int i = 0; i < node->fn_def.body_count; i++)
            compile_stmt(&sub, node->fn_def.body[i]);

        cg_emit(&sub, ENCODE_AD(OP_RET_NIL, 0, 0));

        OlProto* proto = codegen_build(&sub);
        proto->param_count = node->fn_def.param_count;
        proto->name = node->fn_def.name;

        int proto_idx = cg->sub_proto_count;
        cg->sub_protos[cg->sub_proto_count++] = proto;

        // Emit CLOSURE into register, then store as global
        uint8_t r = cg_reg(cg);
        uint32_t k = cg_add_const(cg, val_int(proto_idx));
        cg_emit(cg, ENCODE_AD(OP_CLOSURE, r, k));

        uint32_t name_k = cg_add_const(cg, val_string(node->fn_def.name));
        cg_emit(cg, ENCODE_AD(OP_STORE_GLOBAL, r, name_k));
        break;
    }

    case AST_MATCH: {
        // match expr { pat1 => body1; pat2 => body2; }
        // Desugar → if/else chain
        uint8_t val = compile_expr(cg, node->match.expr);
        uint32_t end_patches[64];
        int patch_count = 0;

        for (int i = 0; i < node->match.arm_count; i++) {
            // Compare val == pattern[i]
            uint8_t pat = compile_expr(cg, node->match.patterns[i]);
            uint8_t cmp = cg_reg(cg);
            cg_emit(cg, ENCODE_ABC(OP_EQ, cmp, val, pat));
            uint32_t skip = cg_emit_jump(cg, OP_JZ, cmp);
            cg_free_regs(cg, cmp);

            // Arm body
            cg_begin_scope(cg);
            for (int j = 0; j < node->match.arm_bodies[i].count; j++)
                compile_stmt(cg, node->match.arm_bodies[i].stmts[j]);
            cg_end_scope(cg);

            end_patches[patch_count++] = cg_emit_jump(cg, OP_JMP, 0);
            cg_patch_jump(cg, skip);
        }

        // Patch all end jumps
        for (int i = 0; i < patch_count; i++)
            cg_patch_jump(cg, end_patches[i]);
        break;
    }

    case AST_IMPORT: {
        // import "file.ol" → resolved at compile time (source prepend)
        // No runtime opcode needed — compiler concatenates source files
        break;
    }

    case AST_EXPR: {
        // Standalone expression (e.g., function call as statement)
        compile_expr(cg, node->expr.value);
        break;
    }

    default:
        break;
    }
}

/* ── Build final OlProto ───────────────────────────────────────── */

OlProto* codegen_build(Codegen* cg) {
    OlProto* proto = malloc(sizeof(OlProto));
    proto->code = cg->code;
    proto->code_len = cg->code_len;
    proto->constants = cg->constants;
    proto->const_count = cg->const_count;
    proto->param_count = 0;      // set by caller
    proto->reg_count = cg->max_reg;
    proto->name = NULL;          // set by caller
    return proto;
}
```

**Những gì CHƯA có trong codegen (ghi rõ):**
1. **Free variable analysis** cho closures — cần scan body, tìm idents không resolve được local/global → capture làm upvalue. ~100 LOC.
2. **Struct compilation** — AST_STRUCT → OBJ_STRUCT layout. Hiện tại dùng dict thay thế. ~50 LOC.
3. **Pipe operator** `|>` — desugar `a |> f` → `f(a)`. ~10 LOC.
4. **F-string** — desugar `f"hello {x}"` → concat chain. ~30 LOC.
5. **Concurrency opcodes** — spawn, channel, select. [CHƯA THIẾT KẾ]
6. **import resolution** — file read + source concat. Hiện tại = no-op. Cần ~50 LOC.

---

## 15. COMPILER ↔ VM OPCODE MAPPING

Mỗi AST node compile thành chuỗi opcodes cụ thể:

| AST | Opcodes | Note |
|-----|---------|------|
| `let x = expr` | compile(expr)→R[n]; STORE_GLOBAL/LOCAL | |
| `emit expr` | compile(expr)→R[n]; EMIT R[n] | |
| `a + b` | compile(a)→R[n]; compile(b)→R[n+1]; ADD R[n+2],R[n],R[n+1] | String: CONCAT |
| `a < b` | compile(a)→R[n]; compile(b)→R[n+1]; LT R[n+2],R[n],R[n+1] | Returns 1.0/0.0 |
| `if cond { A } else { B }` | compile(cond)→R[n]; JZ R[n],+skip; compile(A); JMP +skip2; compile(B) | |
| `while cond { body }` | label: compile(cond)→R[n]; JZ R[n],+exit; compile(body); JMP -back; exit: | |
| `for i in 0..10 { body }` | LOAD_INT R[n],0; LOAD_INT R[n+1],10; label: LT R[n+2],R[n],R[n+1]; JZ +exit; body; ADD R[n],R[n],1; JMP -back | |
| `fn f(a,b) { body }` | CLOSURE R[n],proto[D] | Proto stores compiled body |
| `f(x, y)` | compile(args); CALL R[result],R[fn],argc | |
| `try { A } catch { B }` | TRY_BEGIN +catch; compile(A); JMP +end; catch: compile(B); CATCH_END | |
| `throw e` | compile(e)→R[n]; THROW R[n] | |
| `mol_dist(a, b)` | compile(a); compile(b); MOL_DIST R[out],R[a],R[b] | |
| `encode(cp)` | compile(cp); MOL_ENCODE R[out],R[cp] | |
| `[1, 2, 3]` | NEW_ARRAY R[n],3; LOAD_INT R[t],1; ARRAY_PUSH R[n],R[t]; ... | |
| `arr[i]` | ARRAY_GET R[out],R[arr],R[i] | |
| `{k: v}` | NEW_DICT R[n]; compile(k); compile(v); DICT_SET R[n],R[k],R[v] | |
| `a \|> f` | compile(a)→R[n]; CALL R[result],R[f],1 (với R[f+1]=R[n]) | |
| `return expr` | compile(expr)→R[n]; RET R[n] | |
| `break` | JMP +break_target (patched later) | |
| `continue` | JMP +continue_target (patched later) | |
| `spawn { body }` | SPAWN +body_end; compile(body); SPAWN_END | DEFER — cần green thread model |
| `channel()` | CHAN_NEW R[n] | DEFER |
| `send(ch, v)` | CHAN_SEND R[ch],R[v] | DEFER |
| `receive(ch)` | CHAN_RECV R[out],R[ch] | DEFER |
| `persist_save(k,v)` | PERSIST_SAVE R[k],R[v] | DEFER — cần persistence layer |
| `fn_source(f)` | FN_SOURCE R[out],R[f] | OlProto.source_offset (VM §4.1) |
| `eval(code)` | EVAL R[out],R[code] | DEFER — cần compiler trong VM |
| `∂(f)(x)` | CALL derivative closure: (f(x+h)-f(x-h))/(2h), h=1e-7 | §13.2 central difference |

---

## 16. REPL MODE — Interactive Olang

Cảm hứng từ Julia REPL. Olang cần chạy interactive, không chỉ compile→run.

### 16.1 REPL Architecture

```c
/* Origin REPL: compile + execute từng line/block.
 *
 * Giống Julia:
 *   - Gõ expression → thấy kết quả ngay
 *   - Gõ function → lưu vào scope
 *   - import → load file vào session
 *
 * Khác Julia:
 *   - Mỗi input cũng được ENCODE vào molecular chain (brain luôn học)
 *   - REPL state persist qua restart (origin.dat)
 *
 * Implementation: eval() opcode + embedded compiler
 *   Phase 1: compiler bằng C (embedded trong VM)
 *   Phase 2: compiler bằng Olang (self-hosting, eval gọi compiler.ol)
 */

void repl_loop(VM* vm) {
    Agent ag;
    agent_init(&ag, vm);

    char prompt[16] = "origin> ";
    char input[4096];

    while (1) {
        write(STDOUT_FILENO, prompt, strlen(prompt));

        // Read line
        if (!fgets(input, sizeof(input), stdin)) break;
        size_t len = strlen(input);
        if (len > 0 && input[len-1] == '\n') input[--len] = '\0';
        if (len == 0) continue;

        // Special commands
        if (strcmp(input, "exit") == 0 || strcmp(input, "quit") == 0) break;
        if (strcmp(input, "save") == 0) { origin_checkpoint(vm); continue; }
        if (strcmp(input, "status") == 0) {
            printf("KnowTree: %u nodes, %u QR\n",
                   vm->knowtree->node_count, vm->knowtree->qr_count);
            printf("Silk: %u edges\n", vm->silk->edge_count);
            printf("Interactions: %u\n", ag.interaction_count);
            continue;
        }

        // Try compile as expression/statement
        OlProto* proto = compile_string(vm, input, len);
        if (proto) {
            // Execute compiled code
            vm_execute(vm, proto);
            // Show last register value (if non-nil) — like Julia REPL
            Value result = vm->regs[0];
            if (!is_nil(result)) {
                vm_emit(vm, result);
            }
        }

        // Also feed to brain (learn from interaction)
        agent_tick(&ag, vm, input, len);
    }

    // Save on exit
    origin_checkpoint(vm);
}
```

### 16.2 Read/Parse Other Languages — "Ăn" Data

Olang đọc các format khác qua **parser viết bằng Olang** (không FFI):

```olang
// ═══ JSON (đã có — json.ol, 199 LOC) ═══
let config = json_parse(__file_read("config.json"))
emit config["name"]  // "Nox"

// ═══ CSV parser (~50 LOC Olang) ═══
fn csv_parse(text) {
    let rows = []
    let lines = __str_split(text, "\n")
    let headers = __str_split(lines[0], ",")
    for i in 1..len(lines) {
        let fields = __str_split(lines[i], ",")
        let row = {}
        for j in 0..len(headers) {
            row[__str_trim(headers[j])] = __str_trim(fields[j])
        }
        push(rows, row)
    }
    return rows
}

// ═══ Read Julia file → extract function signatures ═══
fn parse_julia_functions(source) {
    let funcs = []
    let lines = __str_split(source, "\n")
    for line in lines {
        if __str_starts_with(__str_trim(line), "function ") {
            let name = __str_split(__str_trim(line), " ")[1]
            let name = __str_split(name, "(")[0]
            push(funcs, {"name": name, "line": line})
        }
    }
    return funcs
}

// ═══ Read Python file → extract class/function names ═══
fn parse_python_classes(source) {
    let items = []
    let lines = __str_split(source, "\n")
    for line in lines {
        let trimmed = __str_trim(line)
        if __str_starts_with(trimmed, "class ") {
            let name = __str_split(trimmed, " ")[1]
            push(items, {"type": "class", "name": __str_split(name, "(")[0]})
        }
        if __str_starts_with(trimmed, "def ") {
            let name = __str_split(trimmed, " ")[1]
            push(items, {"type": "function", "name": __str_split(name, "(")[0]})
        }
    }
    return items
}

// ═══ Ăn: parse → encode → lưu vào KnowTree ═══
fn eat_file(path) {
    let content = __file_read(path)
    let ext = __str_split(path, ".")[len(__str_split(path, ".")) - 1]

    if ext == "json" {
        let data = json_parse(content)
        // Encode mỗi key-value thành molecular chain
        for key in __dict_keys(data) {
            let chain = encode(key + " " + __to_string(data[key]))
            kt_store(chain)
        }
    }
    if ext == "csv" {
        let rows = csv_parse(content)
        for row in rows {
            let text = ""
            for key in __dict_keys(row) {
                text = text + key + "=" + row[key] + " "
            }
            kt_store(encode(text))
        }
    }
    if ext == "jl" {
        let funcs = parse_julia_functions(content)
        for f in funcs {
            kt_store(encode("julia function " + f["name"]))
        }
    }
    if ext == "py" {
        let items = parse_python_classes(content)
        for item in items {
            kt_store(encode(item["type"] + " " + item["name"]))
        }
    }

    emit "Ate " + path + " (" + __to_string(len(content)) + " bytes)"
}
```

### 16.3 Julia Debug Tools (offline)

Julia đọc Olang state files → visualize, verify, profile:

```julia
# tools/debug_origin.jl — Julia debugger cho Origin
using Mmap, Printf

# Đọc origin.dat → inspect KnowTree
function inspect_knowtree(path="origin.dat")
    data = read(path)
    magic = String(data[1:4])
    @assert magic == "ORIG" "Not an Origin file"
    node_count = reinterpret(UInt32, data[7:10])[1]
    edge_count = reinterpret(UInt32, data[11:14])[1]
    println("KnowTree: $node_count nodes, Silk: $edge_count edges")

    # Parse nodes (32 bytes each)
    offset = 17  # after header
    for i in 1:min(node_count, 20)
        pw = reinterpret(UInt16, data[offset:offset+1])[1]
        fire = reinterpret(UInt16, data[offset+2:offset+3])[1]
        s, r, v, a, t = (pw>>12)&0xF, (pw>>8)&0xF, (pw>>5)&0x7, (pw>>2)&0x7, pw&0x3
        @printf("  Node %d: mol(S=%d R=%d V=%d A=%d T=%d) fire=%d\n", i, s, r, v, a, t, fire)
        offset += 32
    end
end

# Verify encode/decode round-trip
function verify_roundtrip(encode_table_path="data/encode_table.bin",
                           knn_path="data/mol_knn_table.bin")
    enc = Mmap.mmap(open(encode_table_path), Vector{UInt16}, (65536,))
    knn = Mmap.mmap(open(knn_path), Matrix{UInt16}, (16, 65536))

    # Test: encode 'A' (0x41) → P_weight → nearest neighbors
    pw = enc[0x42]  # 1-indexed
    println("'A' → P_weight = $(pw) = mol(S=$((pw>>12)&0xF))")
    println("  Nearest 5: ", [enc[knn[i, pw+1]+1] for i in 1:5])
end

# Profile: collision histogram
function collision_histogram(encode_table_path="data/encode_table.bin")
    enc = Mmap.mmap(open(encode_table_path), Vector{UInt16}, (65536,))
    buckets = Dict{UInt16, Int}()
    for pw in enc
        buckets[pw] = get(buckets, pw, 0) + 1
    end
    sizes = sort(collect(values(buckets)))
    println("Collision histogram:")
    println("  1 codepoint:  $(count(==(1), sizes)) buckets")
    println("  2-5:          $(count(x -> 2<=x<=5, sizes)) buckets")
    println("  6-20:         $(count(x -> 6<=x<=20, sizes)) buckets")
    println("  >20:          $(count(>(20), sizes)) buckets")
    println("  Max collision: $(maximum(sizes))")
end
```

---

## 17. OPEN QUESTIONS

| # | Câu hỏi | Status |
|---|---------|--------|
| Q1 | ∂(f) numerical hay symbolic? | DECIDED: numerical (central difference, §13.2) |
| Q2 | vec/mat SIMD? | DEFER — benchmark trước |
| Q3 | eval() compiler embedded | Phase 1: C compiler in VM. Phase 2: self-hosting |
| Q4 | Concurrency model | DEFER — green threads (BEAM style) |
| Q5 | Struct layout | DECIDED: fixed fields (compile-time offsets), dict = dynamic |

---

*Spec: 17 sections. Olang = ngôn ngữ cho AI tự sửa, interactive (REPL), đọc mọi format (JSON/CSV/Julia/Python), kết hợp với Julia offline tools.
Cross-reference: SPEC_ORIGIN_VM.md (§4 opcodes, §5 brain, §11 Julia), SPEC_MOLECULAR_ENGINE.md (§1-27).*
