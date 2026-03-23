# Olang Handbook — So tay Ngon ngu Olang

> **Phien ban:** 0.07 | **Cap nhat:** 2026-03-23
>
> Olang = ngon ngu lap trinh + suy luan + sang tao cua HomeOS.
> Moi thu la MolecularChain. Moi phep toan la bien doi chain.

---

## Muc luc

1. [Khai niem co ban](#1-khai-niem-co-ban)
2. [Bien va kieu du lieu](#2-bien-va-kieu-du-lieu)
3. [Toan tu](#3-toan-tu)
4. [Dieu khien luong](#4-dieu-khien-luong)
5. [Ham](#5-ham)
6. [Struct va Enum](#6-struct-va-enum)
7. [Trait va Generics](#7-trait-va-generics)
8. [Collections](#8-collections)
9. [Iterator va Closure](#9-iterator-va-closure)
10. [Xu ly loi](#10-xu-ly-loi)
11. [Dong thoi (Concurrency)](#11-dong-thoi)
12. [Module va Import](#12-module-va-import)
13. [Molecular — Linh hon Olang](#13-molecular)
14. [Relations — 18 lien ket](#14-relations)
15. [Suy luan va Tri thuc](#15-suy-luan)
16. [IO va He thong](#16-io-va-he-thong)
17. [Debug va Kiem tra](#17-debug-va-kiem-tra)
18. [Standard Library](#18-standard-library)
19. [Bootstrap — Tu viet Olang bang Olang](#19-bootstrap)
20. [Anti-patterns](#20-anti-patterns)
21. [Tham chieu nhanh](#21-tham-chieu-nhanh)

---

## 1. Khai niem co ban

### Olang la gi?

Olang la ngon ngu cua HomeOS — mot he dieu hanh chay tren dien thoai, dung
2 bytes (Molecule = u16) de bieu dien bat ky khai niem nao. Olang khong lam
viec voi text hay so thong thuong — no lam viec voi **MolecularChain**: chuoi
cac phan tu 5 chieu nen thanh 16 bits.

### Moi thu la MolecularChain

```
Molecule = [S:4][R:4][V:3][A:3][T:2] = 16 bits = 2 bytes (u16)
MolecularChain = Vec<u16> = chuoi nhieu molecules (moi link 2 bytes)

Khi ban viet:
  fire           -> tra Registry -> MolecularChain
  42             -> ma hoa thanh 4-molecule chain
  "xin chao"    -> di qua NLP pipeline -> chain
  { S=1 R=6 }   -> tao truc tiep 1 molecule
```

### Chay Olang

```bash
# REPL
cargo run -p server
> ○{emit "Hello, HomeOS!"}

# Trong runtime
○{fire ∘ water}
○{1 + 2}
○{dream}
```

Moi bieu thuc Olang nam trong `○{...}` khi goi tu runtime.
Khi viet file `.ol`, khong can `○{}`.

---

## 2. Bien va kieu du lieu

### Khai bao bien

```olang
let x = 42;                    // Bat bien (immutable)
let mut y = 0;                 // Kha bien (mutable)
y = y + 1;                     // OK — y la mut
// x = 10;                     // LOI — x la immutable
```

### Huy cau truc (Destructuring)

```olang
let { a, b } = get_pair();    // Trich 2 truong tu dict/tuple
```

### Kieu du lieu co ban

| Kieu | Vi du | Ghi chu |
|------|-------|---------|
| Num (so) | `42`, `3.14` | Tat ca so la f64 ben trong |
| Str (chuoi) | `"hello"` | Chuoi ky tu |
| Bool (logic) | Chain rong = false, khac = true | Khong co `true`/`false` literal trong VM |
| Array | `[1, 2, 3]` | Mang dong |
| Dict | `{ name: "Leo", age: 3 }` | Tu dien key-value |
| Tuple | `(a, b, c)` | Bo gia tri |
| Set | `Set()` | Tap hop (qua stdlib) |
| Deque | `Deque()` | Hang doi 2 dau (qua stdlib) |
| MolecularChain | `fire`, `{ S=1 R=6 }` | Kieu nen tang — moi thu deu la chain |
| Option | `Option::Some(x)`, `Option::None` | Gia tri co the vang mat |
| Result | `Result::Ok(x)`, `Result::Err(e)` | Ket qua co the loi |

### F-string (noi suy chuoi)

```olang
let name = "Leo";
let age = 3;
emit f"Xin chao, {name}! Ban {age} tuoi.";
// -> "Xin chao, Leo! Ban 3 tuoi."
```

### Ep kieu

```olang
let s = to_string(42);        // Num -> Str
let n = to_num("42");         // Str -> Num
```

---

## 3. Toan tu

### Toan hoc (QT3 Axiom)

Olang phan biet 3 cap do chan ly:

```olang
// Gia thuyet (hypothesis) — chua chung minh
1 + 2                          // __hyp_add
10 - 3                         // __hyp_sub
4 * 5                          // __hyp_mul (hoac 4 x 5)
10 / 2                         // __hyp_div (hoac 10 / 2)
7 % 3                          // __hyp_mod

// Vat ly (physical) — da chung minh
mass_a ⧺ mass_b                // __phys_add + FUSE
total ⊖ used                   // __phys_sub + FUSE

// Chan ly (truth) — chac chan
fire == flame                  // __assert_truth
```

### So sanh

```olang
a < b                          // Nho hon
a > b                          // Lon hon
a <= b                         // Nho hon hoac bang
a >= b                         // Lon hon hoac bang
a != b                         // Khac nhau
a == b                         // Bang nhau (truth assertion)
```

### Logic

```olang
a && b                         // VA (ca hai non-empty)
a || b                         // HOAC (mot trong hai non-empty)
!a                             // PHU DINH
```

### Bitwise

```olang
a & b                          // AND
a ^ b                          // XOR
~a                             // NOT
a << 2                         // Dich trai
a >> 1                         // Dich phai
```

### Pipe (ong dan)

```olang
fire |> typeof |> emit;
// Tuong duong: emit(typeof(fire))
// Du lieu chay tu trai sang phai
```

### Unwrap

```olang
let val = maybe_none() ?? 0;   // Neu None/Err -> dung 0
let val = might_fail()?;       // Neu Err -> return Err som
```

---

## 4. Dieu khien luong

### If / Else

```olang
if x > 10 {
    emit "lon";
} else {
    emit "nho";
}

// Unicode syntax:
x > 10 ⇒ {
    ○ "lon";
} ⊥ {
    ○ "nho";
}

// If la bieu thuc — tra ve gia tri:
let label = if x > 10 { "lon" } else { "nho" };
```

### Loop (lap co dinh)

```olang
loop 5 {
    emit "chao";
}

// Unicode:
↻ 5 {
    ○ "chao";
}
```

> Gioi han: toi da 1024 lan lap moi lenh `loop` (QT2 — chong vo han).

### While (lap co dieu kien)

```olang
let mut i = 0;
while i < 10 {
    emit i;
    i = i + 1;
}
```

### For (lap tren range hoac collection)

```olang
// Range
for i in 0..10 {
    emit i;
}

// Collection
let items = [1, 2, 3, 4, 5];
for x in items {
    emit x;
}
```

### Break va Continue

```olang
let mut i = 0;
while i < 100 {
    if i == 50 { break; }
    if i % 2 == 0 {
        i = i + 1;
        continue;
    }
    emit i;
    i = i + 1;
}
```

### Match (doi mau)

```olang
match value {
    Color::Red => { emit "do"; },
    Color::Blue => { emit "xanh"; },
    _ => { emit "khac"; },
}

// Match voi molecular pattern:
match node {
    { S=1 R=6 } => { emit "fire-like"; },
    ○{ V>0x80, A>0x80 } => { emit "positive + excited"; },
    _ => { emit "khong biet"; },
}
```

---

## 5. Ham

### Dinh nghia ham

```olang
fn greet(name) {
    emit f"Xin chao, {name}!";
}

greet("Leo");

// Unicode syntax:
greet ≔ (name) {
    ○ f"Xin chao, {name}!";
};
```

### Ham co generics

```olang
fn swap[T](a: T, b: T) {
    return (b, a);
}
```

### Ham co trait bounds

```olang
fn sort[T: Comparable](items: [T]) {
    // chi chap nhan T implement Comparable
}
```

### Ham co molecular constraints

```olang
fn process_hot(x: ○{ V>0xC0, A>0x80 }) {
    // chi chap nhan molecule co Valence > 0xC0 va Arousal > 0x80
    emit f"Nong va manh: {x}";
}
```

### Return

```olang
fn add(a, b) {
    return a + b;
}

let sum = add(1, 2);  // sum = 3
```

---

## 6. Struct va Enum

### Struct (cau truc du lieu)

```olang
struct Point {
    x: Num,
    y: Num,
}

// Tao instance:
let p = Point { x: 10, y: 20 };

// Truy cap truong:
emit p.x;           // 10
emit p.y;           // 20

// Gan lai (neu mut):
let mut p2 = Point { x: 0, y: 0 };
p2.x = 5;
```

### Struct voi generics

```olang
struct Pair[T, U] {
    first: T,
    second: U,
}

let pair = Pair[Num, Str] { first: 42, second: "hello" };
```

### Struct voi pub

```olang
pub struct Config {
    pub name: Str,
    pub version: Num,
    internal_id: Num,       // private
}
```

### Enum (kieu liet ke)

```olang
enum Color {
    Red,
    Green,
    Blue,
    Custom(Num, Num, Num),  // variant co du lieu
}

let c = Color::Red;
let custom = Color::Custom(255, 128, 0);

match c {
    Color::Red => { emit "do"; },
    Color::Green => { emit "xanh la"; },
    Color::Blue => { emit "xanh duong"; },
    Color::Custom(r, g, b) => { emit f"RGB({r},{g},{b})"; },
}
```

### Enum voi generics

```olang
enum Option[T] {
    Some(T),
    None,
}

enum Result[T, E] {
    Ok(T),
    Err(E),
}

let maybe = Option::Some(42);
let nothing = Option::None;

match maybe {
    Option::Some(val) => { emit val; },
    Option::None => { emit "khong co gi"; },
}
```

### Union (tuong tu enum — dung trong bootstrap)

```olang
union TokenKind {
    Keyword { name: Str },
    Ident { name: Str },
    Number { value: Num },
    StringLit { value: Str },
    Symbol { ch: Str },
    Eof,
}
```

`union` va `enum` tuong duong ve y nghia. `union` la ten goc,
`enum` la ten quen thuoc tu Rust.

---

## 7. Trait va Generics

### Trait (giao dien)

```olang
trait Drawable {
    fn draw(self);
    fn area(self);
    fn name(self) {
        // default implementation
        return "unknown";
    }
}
```

### Impl (hien thuc)

```olang
impl Point {
    fn distance(self) {
        return sqrt(self.x * self.x + self.y * self.y);
    }
}

let p = Point { x: 3, y: 4 };
emit p.distance();  // 5.0
```

### Impl trait cho struct

```olang
impl Drawable for Circle {
    fn draw(self) {
        emit f"Ve hinh tron ban kinh {self.r}";
    }
    fn area(self) {
        return 3.14159 * self.r * self.r;
    }
}

let c = Circle { r: 5 };
c.draw();
emit c.area();
```

### Method receivers

```olang
impl Vec3 {
    fn length(self) { ... }           // by value (tieu thu self)
    fn normalize(&self) { ... }       // immutable borrow
    fn set_x(&mut self, x) { ... }    // mutable borrow
    fn reset(mut self) { ... }        // mutable by value
}
```

### Generics voi trait bounds

```olang
fn print_all[T: Drawable](items: [T]) {
    for item in items {
        item.draw();
    }
}

trait Iterator[T] {
    fn next(self);
}

impl[T] Iterator[T] for List[T] {
    fn next(self) { ... }
}
```

---

## 8. Collections

### Array (mang)

```olang
let arr = [1, 2, 3, 4, 5];

// Truy cap
emit arr[0];                   // 1
emit arr.len();                // 5

// Thay doi
let mut arr = [1, 2, 3];
arr.push(4);                   // [1, 2, 3, 4]
let last = arr.pop();          // last=4, arr=[1,2,3]

// Slice
let sub = arr[1..3];           // [2, 3]
let from = arr[2..];           // [3]
let to = arr[..2];             // [1, 2]

// Tim kiem
emit arr.contains(2);          // true
emit arr.reverse();            // [3, 2, 1]
emit arr.join(", ");           // "1, 2, 3"
```

### Dict (tu dien)

```olang
let config = {
    name: "HomeOS",
    version: 5,
    debug: false,
};

// Truy cap
emit config.name;              // "HomeOS"
emit config.get("version");   // 5

// Cap nhat
config.set("debug", true);

// Duyet
let ks = config.keys();       // ["name", "version", "debug"]
let vs = config.values();     // ["HomeOS", 5, true]

// Kiem tra
emit config.has_key("name");   // true

// Gop
let extra = { port: 8080 };
let merged = config.merge(extra);
```

### Set (tap hop)

```olang
use set;

let s = Set();
s.insert(1);
s.insert(2);
s.insert(2);                  // Khong them trung lap
emit s.len();                  // 2
emit s.contains(1);           // true

let a = Set();  a.insert(1);  a.insert(2);  a.insert(3);
let b = Set();  b.insert(2);  b.insert(3);  b.insert(4);
emit a.union(b);              // {1, 2, 3, 4}
emit a.intersection(b);       // {2, 3}
emit a.difference(b);         // {1}
```

### Deque (hang doi 2 dau)

```olang
use deque;

let q = Deque();
q.push_back(1);
q.push_back(2);
q.push_front(0);
emit q.pop_front();           // 0
emit q.pop_back();            // 2
emit q.peek_front();          // 1
emit q.len();                 // 1
```

---

## 9. Iterator va Closure

### Closure (ham vo danh)

```olang
let double = |x| { x * 2 };
emit double(21);               // 42

let add = |a, b| { a + b };
emit add(3, 4);                // 7
```

### Array voi closure

```olang
let scores = [3, 1, 4, 1, 5, 9, 2, 6];

// Map — bien doi moi phan tu
let doubled = scores.map(|s| { s * 2 });
// [6, 2, 8, 2, 10, 18, 4, 12]

// Filter — loc theo dieu kien
let high = scores.filter(|s| { s > 3 });
// [4, 5, 9, 6]

// Fold — gop thanh 1 gia tri
let sum = scores.fold(0, |acc, s| { acc + s });
// 31

// Any / All — kiem tra dieu kien
emit scores.any(|s| { s > 8 });   // true (9 > 8)
emit scores.all(|s| { s > 0 });   // true

// Find — tim phan tu dau tien thoa man
let first_big = scores.find(|s| { s > 5 });
// 9

// Count — dem so phan tu thoa man
let count = scores.count(|s| { s > 3 });
// 4

// Enumerate — them index
let indexed = scores.enumerate();
// [(0, 3), (1, 1), (2, 4), ...]
```

### Iterator (luoi — lazy evaluation)

```olang
let result = [1, 2, 3, 4, 5]
    .iter()
    .map(|x| { x * 2 })
    .filter(|x| { x > 4 })
    .collect();
// [6, 8, 10]

// Cac phep toan iterator:
.iter()          // Tao iterator tu array
.map(f)          // Bien doi (lazy)
.filter(f)       // Loc (lazy)
.take(n)         // Lay n phan tu dau
.skip(n)         // Bo qua n phan tu dau
.chain(other)    // Noi 2 iterators
.zip(other)      // Ghep cap
.flat_map(f)     // Map + flatten
.sum()           // Tong
.min()           // Nho nhat
.max()           // Lon nhat
.collect()       // Thu thap thanh array (trigger evaluation)
```

> **Quan trong:** `.map()` va `.filter()` tren iterator la **lazy** — chi tinh
> khi goi `.collect()`. Tren array truc tiep (khong co `.iter()`) la **eager**.

---

## 10. Xu ly loi

### Try / Catch

```olang
try {
    let result = risky_operation();
    emit result;
} catch {
    emit "Da xay ra loi!";
}
```

### Option va Result

```olang
fn find_user(id) {
    if id == 0 {
        return Option::None;
    }
    return Option::Some({ name: "Leo", id: id });
}

let user = find_user(1);
match user {
    Option::Some(u) => { emit f"Tim thay: {u.name}"; },
    Option::None => { emit "Khong tim thay"; },
}
```

### Toan tu `?` (try propagation)

```olang
fn process() {
    let data = read_file("config.olang")?;  // Neu Err -> return Err som
    let parsed = parse(data)?;               // Neu Err -> return Err som
    return Result::Ok(parsed);
}
```

### Toan tu `??` (unwrap voi gia tri mac dinh)

```olang
let name = get_name() ?? "Anonymous";
let port = get_config("port") ?? 8080;
```

---

## 11. Dong thoi (Concurrency)

### Spawn (tao tac vu dong thoi)

```olang
spawn {
    loop 3 {
        emit "background task";
    }
}
emit "main continues";
```

> `spawn` la cooperative (khong preemptive) — VM xu ly tuan tu nhung
> Runtime co the phan phoi.

### Channel (kenh giao tiep)

```olang
let ch = channel();

spawn {
    ch.send("hello from spawn");
}

let msg = ch.recv();
emit msg;  // "hello from spawn"
```

### Select (cho nhieu kenh)

```olang
let ch1 = channel();
let ch2 = channel();

select {
    msg from ch1 => {
        emit f"Tu ch1: {msg}";
    },
    msg from ch2 => {
        emit f"Tu ch2: {msg}";
    },
    timeout 1000 => {
        emit "Het thoi gian cho!";
    },
}
```

---

## 12. Module va Import

### Khai bao module

```olang
// File: stdlib/math.ol
mod math;

pub fn sqrt(x) { sqrt(x) }
pub fn sin(x) { sin(x) }
// fn internal() { ... }  // private — khong export
```

### Import module

```olang
// Import toan bo module
use math;
emit math.sqrt(16);

// Import cu the
use math.{ sqrt, sin };
emit sqrt(16);

// Import tu duong dan
use olang.bootstrap.lexer;
let tokens = lexer.tokenize(source);
```

### Pub (cong khai)

```olang
pub fn api_function() { ... }     // Nhin thay tu ben ngoai
fn internal_helper() { ... }       // Chi nhin thay trong module
pub struct Config { ... }          // Struct cong khai
```

---

## 13. Molecular — Linh hon Olang

### Molecule = 5 CONG THUC, khong phai 5 gia tri

```
Molecule KHONG luu gia tri tinh. Molecule luu CONG THUC.

Moi chieu = (index: f(x)) = tham chieu den 1 cong thuc goc L0:

  Shape    = (index: f_s(inputs...))   ← cong thuc hinh dang
  Relation = (index: f_r(inputs...))   ← cong thuc quan he
  Valence  = (index: f_v(inputs...))   ← cong thuc cam xuc
  Arousal  = (index: f_a(inputs...))   ← cong thuc cuong do
  Time     = (index: f_t(inputs...))   ← cong thuc thoi gian

Vong doi cong thuc:
  Chua co input  → TIEM NANG   (cong thuc chua evaluate)
  Co input       → GIA TRI     (the vao cong thuc → gia tri cu the)
  Du gia tri     → CHIN        (thay cong thuc bang hang so → promote QR)

8,846 L0 node = 8,846 CONG THUC GOC = 8,846 KENH SILK
Moi concept = TO HOP cac cong thuc L0:
  "Insulin" = compose(f_protein, f_signal, f_regulate)
            = [ref_L0_1] [ref_L0_2] [ref_L0_3] [op]
            = CONG THUC, khong phai gia tri
```

### 5 chieu — 8 ho co ban moi chieu

```
Shape    (4 bits, 0-15):   Sphere, Plane, Box, Cone, Torus, Cylinder, Capsule,
                           Ellipsoid, Helix, Fractal, Wave, Lattice, Point,
                           Line, Ring, Cross  (VSDF maps to 18 SDF primitives)
Relation (4 bits, 0-15):  Member, Subset, Equiv, Ortho, Compose, Causes, Similar,
                           Derived (+ 8 reserved)
Valence  (3 bits, 0-7):   0=tieu cuc, 4=trung tinh, 7=tich cuc
Arousal  (3 bits, 0-7):   0=binh tinh, 7=kich dong
Time     (2 bits, 0-3):   Static, Slow, Fast, Instant

2 node cung (index: f(x)) tren chieu nao = Silk tren chieu do.
Silk KHONG CAN LUU — chi can SO SANH index.
```

### Molecular Literal

```olang
// Tao molecule truc tiep tu 5 chieu (v2: 2 bytes/mol):
let fire_mol = { S=1 R=6 V=6 A=5 T=3 };

// Gia tri mac dinh: S=1(Sphere) R=1(Member) V=4(trung tinh) A=4 T=2(Fast)
let default_mol = { S=1 };     // chi dinh Shape, con lai la mac dinh

// v2 layout: [S:4][R:4][V:3][A:3][T:2] = 16 bits = 1 u16
// Toan bo molecule nen vao 2 bytes — tiet kiem 60% so voi v1 (5B)
```

### Compose (LCA) — To hop cong thuc

```olang
let parent = fire ∘ water;
// Tim Lowest Common Ancestor — "to tien chung" trong cay tri thuc
// fire va water deu la "elements" -> parent ≈ elements

// LCA = lay cong thuc A va cong thuc B → sinh cong thuc C moi
// C chua co gia tri — CHO du lieu thuc
```

### Evolution — Thay 1 BIEN trong cong thuc → loai moi

```olang
// Molecule.evolve(dim, new_value) -> loai moi
// Giu nguyen 4 cong thuc, chi THAY 1 cong thuc
// → chain_hash moi → concept moi

// Vi du: lua -> cac bien the
// fire.evolve(Valence, 0x40)   -> "lua nhe"    (thay f_v → cong thuc moi)
// fire.evolve(Time, Instant)   -> "chay no"     (thay f_t → cuc nhanh)
// fire.evolve(Shape, Line)     -> "tia lua"     (thay f_s → hinh dang moi)
```

### Maturity — Vong doi cong thuc

```olang
// Moi node di qua 3 giai doan:
//   Formula    → moi tao, chua co input that (5 cong thuc tiem nang)
//   Evaluating → fire_count >= fib(depth), dang tich luy evidence
//   Mature     → weight >= 0.854 && fire_count >= fib(depth), san sang QR

// Dream = danh gia cong thuc nao da "chin"
//   STM day cong thuc chua evaluate
//   Dream di qua → the gia tri vao → node chin → promote QR
//   Node chua du data → giu cong thuc → cho them input
```

### Molecular Constraints (rang buoc chieu)

```olang
// Ham chi chap nhan molecule thoa man dieu kien:
fn process_hot(x: ○{ V>0xC0 }) {
    emit "nong!";
}

fn process_calm(x: ○{ A<0x40, T=1 }) {
    emit "binh tinh va tinh";
}

// Trong match:
match mol {
    ○{ V>0xC0, A>0xC0 } => { emit "nong va manh"; },
    ○{ V<0x40 } => { emit "tieu cuc"; },
    _ => { emit "binh thuong"; },
}

// Cac toan tu rang buoc: =  >  <  >=  <=  *  (wildcard)
// Cac chieu: S (Shape), R (Relation), V (Valence), A (Arousal), T (Time)
```

### Ownership tu Time dimension

```olang
// Time dimension quyet dinh ngu nghia so huu:
let cow_val = { T=1 };        // Static  -> Copy-on-Write (chia se, copy khi sua)
let shared  = { T=3 };        // Medium  -> Share (chia se tham chieu)
let moved   = { T=4 };        // Fast    -> Move (chuyen quyen so huu)
let instant = { T=5 };        // Instant -> Move (su dung 1 lan)

// let mut Bat buoc cho bien doi:
let mut x = { S=1 V=200 };
x.V = 100;                    // OK — mut
```

---

## 14. Relations — 18 lien ket

### Tao edge (quan he giua 2 nodes)

```olang
fire ∈ elements;               // fire la thanh vien cua elements
fire ⊂ energy;                 // fire la tap con cua energy
fire ≡ flame;                  // fire dong nhat voi flame
fire → heat;                   // fire gay ra heat
fire ≈ sun;                    // fire tuong tu sun
fire ← combustion;             // fire duoc dan xuat tu combustion
```

### Query (truy van)

```olang
fire ∈ ?;                      // fire thuoc nhom nao?
fire → ?;                      // fire gay ra cai gi?
? → fire;                      // cai gi gay ra fire?
? ⊥ fire;                      // cai gi doi lap fire?
```

### Chain query (truy van nhieu buoc)

```olang
🌞 → ? → 🌵;                  // mat troi -> [gi do] -> xuong rong
fire → ? → ? → water;         // 3 buoc tu fire den water
```

### Context query (ngu canh)

```olang
bank ∂ finance;                // "bank" trong ngu canh "finance" -> ngan hang
bank ∂ river;                  // "bank" trong ngu canh "river" -> bo song
```

### Bang day du 18 RelOps

| Ky hieu | Ten         | Byte | Y nghia                     |
|---------|-------------|------|-----------------------------|
| `∈`     | Member      | 0x01 | A la thanh vien cua B       |
| `⊂`     | Subset      | 0x02 | A la tap con cua B          |
| `≡`     | Equiv       | 0x03 | A dong nhat B               |
| `⊥`     | Orthogonal  | 0x04 | A doc lap / xung dot B      |
| `∘`     | Compose     | 0x05 | LCA(A, B)                   |
| `→`     | Causes      | 0x06 | A gay ra B                  |
| `≈`     | Similar     | 0x07 | A tuong tu B                |
| `←`     | DerivedFrom | 0x08 | A dan xuat tu B             |
| `∪`     | Contains    | 0x09 | A chua B (khong gian)       |
| `∩`     | Intersects  | 0x0A | A giao B                    |
| `∖`     | SetMinus    | 0x0B | A tru B                     |
| `↔`     | Bidir       | 0x0C | A <-> B hai chieu           |
| `⟶`     | Flows       | 0x0D | A chay den B (thoi gian)    |
| `⟳`     | Repeats     | 0x0E | A lap lai chu ky B          |
| `↑`     | Resolves    | 0x0F | A giai quyet tai B          |
| `⚡`    | Trigger     | 0x10 | A kich hoat B               |
| `∥`     | Parallel    | 0x11 | A song song B               |
| `∂`     | Context     | —    | A trong ngu canh B          |

---

## 15. Suy luan va Tri thuc

### Hoc tu van ban

```olang
learn "Lua chay nong, nuoc lam mat";
learn "Lua va nuoc doi lap nhau";
// -> Emotion pipeline phan tich -> Encode -> STM -> Silk
```

### Nhan qua (Causality)

```olang
🌞 → growth;
growth → 🌵;
🌞 → ? → 🌵;                  // Hoi: qua gi ma mat troi giup xuong rong?
```

### Tuong tu (Analogy)

```olang
// A:B :: C:? -> delta 5D tu A->B, ap len C
let cause_heat = fire ∘ heat;
let analogy = ice ∘ cold;
cause_heat ∘ analogy;          // Cung loai: quan he nhan qua
```

### Truu tuong (Abstraction)

```olang
fire ⊂ energy;
water ⊂ matter;
energy ∘ matter;               // -> "physical_world" (truu tuong hon)
```

### Mau thuan (Contradiction)

```olang
hot ⊥ cold;                   // Hai dieu xung dot
```

### Dream — Hop nhat tri thuc

```olang
learn "toi yeu nhac";
learn "nhac lam toi vui";
learn "vui la cam xuc tot";
dream;                         // STM -> cluster -> promote QR (ky uc dai han)
```

### LeoAI tu lap trinh

```olang
// LeoAI co the tu sinh va chay Olang:
// program("emit fire ∘ water;")           -> parse -> compile -> VM -> hoc
// program_compose("fire", "water")        -> sinh + chay "emit fire ∘ water;"
// program_verify("fire", hash)            -> kiem chung truth
// program_experiment(hash, "V", 100)      -> thay 1 chieu, hoc ket qua
```

---

## 16. IO va He thong

### Console

```olang
use io;

io.print("khong xuong dong");
io.println("co xuong dong");
```

### File (append-only theo QT9)

```olang
use io;

let data = io.read_file("config.olang");
io.append_file("log.olang", "dong moi\n");
// io.write_file() co san nhung uu tien dung append_file (QT9)
```

### Device IO (HAL)

```olang
// Doc cam bien
let temp = device_read("sensor_temp");

// Dieu khien thiet bi
device_write("light_living", 255);

// Liet ke thiet bi
device_list();
```

### ISL (giao tiep giua agents)

```olang
// Gui message qua ISL
isl_send(address, payload);
isl_broadcast(payload);
```

### Lenh he thong

| Lenh | Mo ta |
|------|-------|
| `dream` | Trigger Dream cycle (STM -> cluster -> QR) |
| `stats` | Thong ke he thong |
| `health` | Health check |
| `status` | Status report |
| `seed L0` | Seed 35 L0 nodes tu UCD |
| `shutdown` | Tat he thong |
| `reboot` | Khoi dong lai |
| `compile module` | Bien dich module |
| `ram` | Hien thi RAM |

### Lenh toan hoc

| Lenh | Vi du | Mo ta |
|------|-------|-------|
| `solve` | `solve "2x+3=7"` | Giai phuong trinh |
| `derive` | `derive "x^2+3x"` | Tinh dao ham |
| `integrate` | `integrate "2x"` | Tinh tich phan |
| `simplify` | `simplify "2x+3x"` | Rut gon |
| `fib` | `fib 10` | So Fibonacci thu n |
| `const` | `const PI` | Hang so toan hoc |

---

## 17. Debug va Kiem tra

### Trace

```olang
trace;                         // Bat trace mode
emit fire ∘ water;             // Moi buoc in: [trace pc=N op=OP stack=S]
trace;                         // Tat trace mode
```

### Inspect

```olang
inspect fire;
// [inspect hash=0x... molecules=1 bytes=5 empty=false]
```

### Assert

```olang
assert fire;                   // OK neu fire ton tai (non-empty)
assert unknown_thing;          // [ASSERT FAILED: chain is empty]
```

### TypeOf

```olang
typeof fire;
// [typeof 0xABCD = SDF]

typeof fire ∘ water;
// [typeof 0x1234 = Mixed(SDF+EMOTICON)]

// Cac loai: SDF, MATH, EMOTICON, Mixed, Empty, Numeric
```

### Explain

```olang
explain fire;
// [explain origin of 0x...]
// Truy nguon goc: tu UCD nao? Ai tao?
```

### Why

```olang
why fire, heat;
// Tinh LCA(fire, heat) -> giai thich quan he
// [why_connection from=0x... to=0x...]
```

### Test framework (stdlib/test.ol)

```olang
use test;

fn test_math() {
    test.assert_eq(1 + 1, 2);
    test.assert_ne(1, 2);
    test.assert_true(5 > 3);
    test.assert_approx(3.14, 3.14159, 0.01);
}

fn test_string() {
    let s = "hello";
    test.assert_eq(s.str_len(), 5);
    test.assert_contains(s, "ell");
}
```

---

## 18. Standard Library

### math.ol

```olang
use math;

math.PI                        // 3.14159265358979323846
math.PHI                       // 1.61803398874989484820 (ti le vang)
math.E                         // 2.71828182845904523536

math.abs(-5)                   // 5
math.sqrt(16)                  // 4
math.pow(2, 10)                // 1024
math.log(math.E)               // 1.0
math.sin(0)                    // 0
math.cos(0)                    // 1
math.min(3, 7)                 // 3
math.max(3, 7)                 // 7
math.floor(3.7)                // 3
math.ceil(3.2)                 // 4
math.round(3.5)                // 4
```

### vec.ol

```olang
use vec;

let v = vec.new();             // []
vec.push(v, 1);                // [1]
vec.push(v, 2);                // [1, 2]
emit vec.len(v);               // 2
emit vec.get(v, 0);            // 1
vec.set(v, 0, 10);             // [10, 2]
emit vec.contains(v, 10);     // true
emit vec.reverse(v);           // [2, 10]
emit vec.join(v, "-");         // "10-2"
let doubled = vec.map(v, |x| { x * 2 });
let big = vec.filter(v, |x| { x > 5 });
let sum = vec.fold(v, 0, |acc, x| { acc + x });
```

### string.ol

```olang
use string;

let s = "Hello, World!";
string.len(s)                  // 13
string.upper(s)                // "HELLO, WORLD!"
string.lower(s)                // "hello, world!"
string.trim("  hi  ")         // "hi"
string.split("a,b,c", ",")    // ["a", "b", "c"]
string.contains(s, "World")   // true
string.replace(s, "World", "HomeOS")  // "Hello, HomeOS!"
string.starts_with(s, "Hello")  // true
string.index_of(s, "World")   // 7
string.substr(s, 0, 5)        // "Hello"
string.concat("a", "b")       // "ab"
string.chars("abc")           // ["a", "b", "c"]
string.repeat("ha", 3)        // "hahaha"
string.pad_left("42", 5, "0")   // "00042"
```

### map.ol

```olang
use map;

let m = { name: "Leo" };
map.get(m, "name")             // "Leo"
map.set(m, "age", 3)
map.has_key(m, "name")         // true
map.keys(m)                    // ["name", "age"]
map.values(m)                  // ["Leo", 3]
map.remove(m, "age")
map.merge(m, { version: 1 })
```

### set.ol

```olang
use set;

let s = set.new();
set.insert(s, "a");
set.insert(s, "b");
set.contains(s, "a")          // true
set.len(s)                     // 2
set.union(s1, s2)
set.intersection(s1, s2)
set.difference(s1, s2)
set.to_array(s)                // ["a", "b"]
```

### deque.ol

```olang
use deque;

let q = deque.new();
deque.push_back(q, 1);
deque.push_front(q, 0);
deque.pop_front(q)             // 0
deque.pop_back(q)              // 1
deque.peek_front(q)
deque.peek_back(q)
deque.len(q)
```

### bytes.ol

```olang
use bytes;

let b = bytes.to_bytes(value);
bytes.byte_len(b)
bytes.get_u8(b, 0)
bytes.set_u8(b, 0, 255)
bytes.get_u16_be(b, 0)
bytes.set_u16_be(b, 0, 1024)
bytes.get_u32_be(b, 0)
bytes.set_u32_be(b, 0, 65536)
bytes.from_bytes(b)
```

### io.ol

```olang
use io;

io.print("khong newline");
io.println("co newline");
let data = io.read_file("path");
io.append_file("path", data);
io.write_file("path", data);
```

### test.ol

```olang
use test;

test.assert_eq(a, b)          // a == b
test.assert_ne(a, b)          // a != b
test.assert_true(val)          // val la truthy
test.assert_false(val)         // val la falsy
test.assert_contains(hay, needle)  // hay chua needle
test.assert_approx(a, b, eps) // |a-b| < eps
```

---

## 19. Bootstrap — Tu viet Olang bang Olang

HomeOS dang tren hanh trinh tu-hosting: viet trinh bien dich Olang bang chinh
Olang. Hai file bootstrap cho thay kha nang nay:

### Lexer (stdlib/bootstrap/lexer.ol)

```olang
// Dinh nghia token bang union:
union TokenKind {
    Keyword { name: Str },
    Ident { name: Str },
    Number { value: Num },
    StringLit { value: Str },
    Symbol { ch: Str },
    Eof,
}

type Token {
    kind: TokenKind,
    text: Str,
    line: Num,
    col: Num,
}

// Tokenize:
pub fn tokenize(source) {
    let tokens = [];
    let pos = 0;
    while pos < len(source) {
        let ch = char_at(source, pos);
        if is_whitespace(ch) { ... continue; }
        if is_alpha(ch) { ... }     // -> Keyword hoac Ident
        if is_digit(ch) { ... }     // -> Number
        if ch == "\"" { ... }       // -> StringLit
        // ...
        push(tokens, Token { kind: ..., text: ..., line: ..., col: ... });
    }
    return tokens;
}
```

### Parser (stdlib/bootstrap/parser.ol)

```olang
use olang.bootstrap.lexer;

// AST nodes:
union Expr {
    Ident { name: Str },
    NumLit { value: Num },
    BinOp { op: Str, lhs: Expr, rhs: Expr },
    Call { callee: Expr, args: Vec[Expr] },
    FieldAccess { object: Expr, field: Str },
    MolLiteral { s: Num, r: Num, v: Num, a: Num, t: Num },
}

union Stmt {
    LetStmt { name: Str, value: Expr },
    FnDef { name: Str, params: Vec[Str], body: Vec[Stmt] },
    IfStmt { cond: Expr, then_block: Vec[Stmt], else_block: Vec[Stmt] },
    WhileStmt { cond: Expr, body: Vec[Stmt] },
    ReturnStmt { value: Expr },
    EmitStmt { expr: Expr },
    TypeDef { name: Str, fields: Vec[Field] },
    UnionDef { name: Str, variants: Vec[Variant] },
}

// Recursive descent parser voi precedence climbing:
pub fn parse_expr(p) { return parse_expr_prec(p, 1); }
pub fn parse_stmt(p) { ... }
pub fn parse(tokens) {
    let p = new_parser(tokens);
    let program = [];
    while !is_eof(peek(p)) {
        push(program, parse_stmt(p));
    }
    return program;
}
```

### Semantic Analyzer (stdlib/bootstrap/semantic.ol)

```olang
// Bien AST thanh IR opcodes:
pub fn analyze(ast) {
    let state = new_state();
    collect_fns(state, ast);
    let _si = 0;
    while _si < len(ast) {
        compile_stmt(state, ast[_si]);
        let _si = _si + 1;
    };
    emit_op(state, make_op_simple("Halt"));
    return state;
}
```

### Code Generator (stdlib/bootstrap/codegen.ol)

```olang
// Bien IR ops thanh bytecode nhi phan:
pub fn generate(ops) {
    // Pass 1: do kich thuoc thuc te cua moi op
    // Pass 2: encode voi jump targets da resolve
    let _gout = [];
    // ... encode_op cho moi op ...
    return _gout;
}
```

### REPL Pipeline (stdlib/repl.ol)

```olang
pub fn repl_eval(input) {
    let src = __str_trim(input);
    let tokens = tokenize(src);       // lexer.ol
    let ast = parse(tokens);          // parser.ol
    let state = analyze(ast);         // semantic.ol
    let bc = generate(state.ops);     // codegen.ol
    return __eval_bytecode(bc);       // ASM VM nested eval
}
```

### Native Binary — 806KB, zero dependencies

Olang chay tren native binary (x86_64, no libc):
- ASM VM: `vm/x86_64/vm_x86_64.S` (~4000 LOC assembly)
- Bootstrap compiler: 4 file Olang tu viet chinh no
- Full features: arithmetic, strings, variables, if-else, while, functions
- Deep recursion: `fact(10) = 3,628,800`
- Tree recursion: `fib(20) = 6,765`
- VM var_table scoping: snapshot/restore per closure call
- 27/27 REPL tests pass

```
$ echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(10)' | ./origin_new.olang
○ HomeOS v0.05
○ Type code or text · exit to quit
○ > 55
○ > bye
```

---

## 20. Anti-patterns

### KHONG BAO GIO lam

```olang
// SAI: Trung binh cam xuc
let avg = (buon + vui) / 2;
// DUNG: Amplify qua Silk
buon ∘ vui;  // LCA tu nhien, Silk edges amplify

// SAI: Hardcode chain hash
let x = 0x01020304;
// DUNG: Tu registry/UCD
fire;  // registry lookup -> encode_codepoint

// SAI: Skip SecurityGate
// Gate LUON chay truoc moi thu

// SAI: Delete du lieu
// Append-only: KHONG BAO GIO xoa (QT9)

// SAI: Worker gui raw data
// Worker gui molecular chain, KHONG gui raw bytes

// SAI: Skill giu state
// Skill stateless — state nam trong Agent

// SAI: Vong lap vo han
while true { ... }
// DUNG: Loop co gioi han (QT2)
loop 100 { ... }
```

### QT Axioms can nho

| QT | Nguyen ly | Ap dung |
|----|-----------|---------|
| QT1 | ○ la nguon goc | `emit` / `○` xuat chain |
| QT2 | ∞ la sai, ∞-1 moi dung | `FUSE` kiem tra huu han, loop max 1024 |
| QT3 | +/- gia thuyet, ⧺/⊖ vat ly, == chan ly | 3 cap do toan hoc |
| QT9 | Append-only | Khong xoa, khong ghi de |

---

## 21. Tham chieu nhanh

### Keywords va Unicode

| Keyword  | Unicode | Y nghia          |
|----------|---------|------------------|
| `let`    | `≔`     | Gan bien         |
| `fn`     | —       | Dinh nghia ham   |
| `if`     | `⇒`     | Dieu kien        |
| `else`   | `⊥`     | Nhanh else       |
| `loop`   | `↻`     | Lap              |
| `emit`   | `○`     | Xuat             |
| `struct` | `type`  | Cau truc         |
| `enum`   | `union` | Liet ke          |
| `trait`  | —       | Giao dien        |
| `impl`   | —       | Hien thuc        |
| `match`  | —       | Doi mau          |
| `spawn`  | —       | Dong thoi        |
| `select` | —       | Cho nhieu kenh   |
| `use`    | —       | Import module    |
| `pub`    | —       | Cong khai        |
| `mut`    | —       | Kha bien         |
| `return` | —       | Tra ve           |
| `break`  | —       | Thoat loop       |
| `continue` | —     | Lap tiep         |
| `try`    | —       | Thu loi          |
| `catch`  | —       | Bat loi          |
| `for`    | —       | Duyet            |
| `while`  | —       | Lap dieu kien    |
| `self`   | —       | Tu tham chieu    |

### Do uu tien toan tu (thap -> cao)

```
1.  |>              Pipe
2.  ??              Unwrap mac dinh
3.  ||              Logic OR
4.  &&              Logic AND
5.  == != < > <= >= So sanh
6.  ∈ ⊂ ≡ → ≈ ...  Relations
7.  ∘               Compose (LCA)
8.  + - * / %       Toan hoc
9.  << >> & ^       Bitwise
10. ~ !             Prefix (NOT)
11. () [] .         Call, Index, Field
```

### VM Opcodes (50+)

**Stack:** Push, PushNum, PushMol, Dup, Pop, Swap
**Bien:** Store, StoreUpdate, Load, LoadLocal
**Chain:** Lca, Edge, Query, Emit, Fuse
**Control:** Jmp, Jz, Loop, Call, Ret, Halt, Nop
**Scope:** ScopeBegin, ScopeEnd
**Loi:** TryBegin, CatchEnd
**Debug:** Trace, Inspect, Assert, TypeOf, Why, Explain
**System:** Dream, Stats
**IO:** DeviceRead, DeviceWrite, DeviceList, FileRead, FileWrite, FileAppend
**Concurrency:** SpawnBegin, SpawnEnd, ChanNew, ChanSend, ChanRecv, Select
**Closure:** Closure, CallClosure
**FFI:** Ffi

### Compile targets (hien tai)

```
Olang source → tokenize → parse → analyze → generate → bytecode
                lexer.ol   parser.ol  semantic.ol  codegen.ol
                                                       ↓
                                                  ASM VM (x86_64)
                                                  origin_new.olang
                                                  806KB, no libc
```

### Bytecode opcodes (codegen format, bc_format=1)

```
0x01 Push(str)       0x09 Jmp(offset)     0x13 Store(name)
0x02 Load(name)      0x0A Jz(offset)      0x14 LoadLocal(name)
0x06 Emit            0x0B Dup             0x15 PushNum(f64)
0x07 Call(name)      0x0C Pop             0x25 Closure(body_len)
0x08 Ret             0x0F Halt            0x0D Swap

Strings: u16 molecules (each byte → 0x2100 | byte_value)
Numbers: f64 little-endian (8 bytes)
```

---

## 22. HomeOS Stdlib — Code thuc te

### Emotion pipeline (stdlib/homeos/emotion.ol)

```olang
pub fn emotion_new(v, a, d, i) {
    return { v: v, a: a, d: d, i: i };
}

// AMPLIFY — KHONG trung binh, amplify qua Silk weight
// factor = 1 + w × phi^-1 (Golden Ratio boost)
pub fn amplify(emo, silk_weight) {
    let factor = 1.0 + silk_weight * 0.618;
    return emotion_new(
        clamp(emo.v * factor, -1.0, 1.0),
        clamp(emo.a * factor, 0.0, 1.0),
        emo.d,
        clamp(emo.i * factor, 0.0, 1.0)
    );
}

// Compose 2 emotions — AMPLIFY, NOT average
pub fn compose(a, b, silk_weight) {
    let base_v = (a.v + b.v) / 2.0;
    let boost = abs(a.v - base_v) * silk_weight * 0.5;
    // ... amplification logic
}
```

### 7 Instincts (stdlib/homeos/instinct.ol)

```olang
pub fn run_instincts(observation, knowledge) {
    let result = { action: "process", confidence: 0.0 };

    // ① Honesty — confidence < 0.4 → im lang (BlackCurtain)
    result.confidence = assess_confidence(observation, knowledge);
    if result.confidence < 0.40 {
        result.action = "silence";
        return result;
    }

    // ② Contradiction detection
    // ③ Causality — temporal + co-activation
    // ④ Abstraction — N chains → LCA → categorical
    // ⑤ Analogy — A:B :: C:? → delta 5D
    // ⑥ Curiosity — 1 - nearest_similarity
    // ⑦ Reflection — knowledge quality
    return result;
}
```

### ISL TCP codec (stdlib/homeos/isl_tcp.ol)

```olang
pub fn isl_connect(host, port) {
    let socket = __tcp_connect(host, port);
    return { socket: socket, buffer: [], state: "connected" };
}

pub fn isl_send(conn, msg) {
    let frame = encode_isl_frame(msg);
    __tcp_write(conn.socket, frame);
}

fn encode_isl_frame(msg) {
    let out = [];
    push(out, msg.from);        // ISL address (4 bytes)
    push(out, msg.to);          // ISL address (4 bytes)
    push(out, msg.msg_type);    // 1 byte
    // ... payload encoding
    return out;
}
```

---

## Vi du tong hop

### Chuong trinh hoc va suy luan

```olang
// 1. Xay dung tri thuc
fire ∈ elements;
water ∈ elements;
fire → heat;
water → cool;
fire ⊥ water;

// 2. Hoc tu van ban
learn "Lua chay nong, nuoc lam mat";
learn "Lua va nuoc doi lap nhau";

// 3. Kiem tra
inspect fire;
typeof fire;

// 4. Suy luan
fire → ?;                      // fire gay ra gi? -> heat
? ⊥ fire;                      // cai gi doi lap? -> water
let common = fire ∘ water;     // LCA -> "elements"

// 5. Xac nhan
assert fire;
fire == flame;

// 6. Hop nhat
dream;
stats;
```

### Chuong trinh xu ly du lieu

```olang
use math;
use string;

fn process_scores(scores) {
    let total = scores.fold(0, |acc, s| { acc + s });
    let avg = total / scores.len();
    let high = scores.filter(|s| { s > avg });
    let low = scores.filter(|s| { s <= avg });

    emit f"Tong: {total}";
    emit f"Trung binh: {avg}";
    emit f"Tren TB: {high.len()} mon";
    emit f"Duoi TB: {low.len()} mon";

    return { total: total, average: avg, high: high, low: low };
}

let result = process_scores([8, 6, 9, 7, 5, 10, 3]);
```

### Chuong trinh struct + trait

```olang
struct Circle {
    x: Num,
    y: Num,
    r: Num,
}

struct Rect {
    x: Num,
    y: Num,
    w: Num,
    h: Num,
}

trait Shape {
    fn area(self);
    fn describe(self);
}

impl Shape for Circle {
    fn area(self) {
        return 3.14159 * self.r * self.r;
    }
    fn describe(self) {
        emit f"Hinh tron tai ({self.x},{self.y}) r={self.r}";
    }
}

impl Shape for Rect {
    fn area(self) {
        return self.w * self.h;
    }
    fn describe(self) {
        emit f"Hinh chu nhat tai ({self.x},{self.y}) {self.w}x{self.h}";
    }
}

let shapes = [
    Circle { x: 0, y: 0, r: 5 },
    Rect { x: 1, y: 1, w: 10, h: 20 },
];

for s in shapes {
    s.describe();
    emit f"  Dien tich: {s.area()}";
}
```

### Chuong trinh concurrent

```olang
let results = channel();

spawn {
    let sum = [1, 2, 3, 4, 5].fold(0, |a, x| { a + x });
    results.send(f"Tong: {sum}");
}

spawn {
    let product = [1, 2, 3, 4, 5].fold(1, |a, x| { a * x });
    results.send(f"Tich: {product}");
}

// Nhan 2 ket qua
emit results.recv();
emit results.recv();
```

### Tho — Cam xuc amplify qua Silk

```olang
fn poem(lines) {
    for line in lines {
        learn line;
    }
    // Moi dong -> learn -> STM -> Silk co-activation
    // Cam xuc tu dong amplify (KHONG trung binh)
    dream;  // Hop nhat thanh ky uc dai han
}

poem([
    "Mat troi len",
    "Hoa no ven duong",
    "Gio thoang huong thom",
    "Long ta nhe nhang",
]);
```

---

*Olang Handbook v0.05 — HomeOS 2026-03-18*
