# Origin — Hướng dẫn cho AI Contributors

> **Giao tiếp với user bằng TIẾNG VIỆT. User là người Việt.**
> **Viết OLANG, không viết Rust mới (trừ bug fix legacy).**
> **Đọc `TASKBOARD.md` → claim task → rồi mới code.**

---

## Quy tắc làm việc (BẮT BUỘC mọi session)

```
① TIẾNG VIỆT — Mọi giao tiếp với user PHẢI bằng tiếng Việt.
  Code + commit message: tiếng Anh OK. Giải thích, báo cáo: TIẾNG VIỆT.

② OBSERVABLE — Liệt kê việc TRƯỚC KHI bắt đầu.
  Mỗi bước cập nhật status. KHÔNG làm im lặng rồi dump kết quả.

③ GIT DISCIPLINE — Mỗi session:
  a. git fetch origin main && git merge origin/main  ← TRƯỚC KHI code
  b. Làm xong → commit + push NGAY
  c. Cập nhật TASKBOARD.md nếu có thay đổi task
  d. KHÔNG push nếu chưa test: make build && echo 'emit 42' | ./origin_new.olang

④ VIẾT OLANG — Rust legacy KHÔNG được mở rộng.
  ✅ Viết .ol files mới trong stdlib/
  ✅ Sửa bug Rust nếu cần (crates/, tools/)
  ✅ Sửa ASM VM (vm/x86_64/vm_x86_64.S)
  ❌ Viết feature mới bằng Rust
  ❌ Thêm crate/dependency mới
```

---

## Kiến trúc hiện tại (Self-hosting)

```
origin_new.olang = ~824KB native binary (ELF64, no libc, no deps)

User input
  ↓
REPL loop (ASM)
  ↓
repl_eval(input)                    ← stdlib/repl.ol
  ├── tokenize(src)                 ← stdlib/bootstrap/lexer.ol
  ├── parse(tokens)                 ← stdlib/bootstrap/parser.ol
  ├── analyze(ast)                  ← stdlib/bootstrap/semantic.ol
  ├── emit bytecode → _g_output     ← direct emission + backpatch (NOT two-pass)
  └── __eval_bytecode(bc)           ← ASM VM executes compiled bytecode

VM registers:
  r12 = bytecode base
  r13 = PC (program counter)
  r14 = VM stack (grows DOWN, 16 bytes/entry: [ptr:8][len:8])
  r15 = heap (bump allocator, grows UP)
```

### VM Stack Entry Types

```
f64:     ptr = bits,     len = F64_MARKER (-1)
chain:   ptr = heap_ptr, len = mol_count
array:   ptr = heap_ptr, len = ARRAY_MARKER (-3)
dict:    ptr = heap_ptr, len = DICT_MARKER (-4)
closure: ptr = body_pc,  len = CLOSURE_MARKER (-2)
```

---

## Olang — Cách viết code

### Cú pháp cơ bản

```olang
// Variables
let x = 42;
let name = "hello";

// Functions
fn add(a, b) {
    return a + b;
};

// If-else
if x > 0 {
    emit "positive";
} else {
    emit "negative";
};

// While
let i = 0;
while i < 10 {
    emit i;
    let i = i + 1;
};

// Arrays
let items = [1, 2, 3];
push(items, 4);
emit items[0];           // 1
emit len(items);          // 4

// Dicts (dict literal syntax)
let config = { name: "HomeOS", version: 5 };
emit config.name;         // "HomeOS"
emit config.version;      // 5

// Types & Unions
type Point { x: Num, y: Num }
union Shape {
    Circle { radius: Num },
    Rect { w: Num, h: Num },
}

// Pattern matching
match shape {
    Circle(c) => emit c.radius,
    Rect(r) => emit r.w * r.h,
}
```

### Builtins (ASM VM)

```
// Math
__eq(a, b)  __lt(a, b)  __gt(a, b)  __le(a, b)  __ge(a, b)
__add(a, b) __sub(a, b) __mul(a, b) __div(a, b) __hyp_mod(a, b)
__floor(x)  __ceil(x)   __sqrt(x)

// String (u16 molecules)
len(s)  char_at(s, i)  __substr(s, start, end)  // end EXCLUSIVE
__str_trim(s)  __to_number(s)  __to_string(n)
__str_bytes(s) → array of byte values
__str_is_keyword(s) → bool

// Array
len(a)  push(a, val)  a[i]  set_at(a, i, val)
// NOTE: a[i] is desugared to __array_get(a, i) by the parser

// Dict
struct_tag(dict, tag) → new dict with tag
__match_enum(dict, tag) → bool
__enum_field(dict, idx) → value

// I/O
emit expr           // print to stdout
__eval_bytecode(bc) // execute compiled bytecode

// Conversion
__f64_to_le_bytes(n) → array of 8 bytes
```

---

## ASM VM — Khi cần sửa (vm/x86_64/vm_x86_64.S)

### Bytecode opcodes (bc_format=1)

```
0x01 Push(str)       0x09 Jmp(offset)     0x13 Store(name)
0x02 Load(name)      0x0A Jz(offset)      0x14 LoadLocal(name)
0x06 Emit            0x0B Dup             0x15 PushNum(f64)
0x07 Call(name)      0x0C Pop             0x25 Closure(body_len)
0x08 Ret             0x0F Halt            0x0D Swap
```

### Scoping

```
Boot closures (r12 == boot_bc_base): NO scoping — flat var_table
Nested eval closures: FULL scoping via heap scope stack
  op_call: snapshot var_table → scope_frame_ptrs[depth]
  op_ret:  if depth > 0, restore var_table from snapshot
  4MB scope stack, 256 max depth
```

### Debug

```bash
echo 'emit 42' | gdb -batch -ex "break .eval_bc_run" -ex run --args ./origin_new.olang
```

---

## CRITICAL — Global Variable Pattern (BUG #1 SOURCE)

ASM VM dùng GLOBAL var_table. KHÔNG CÓ block scope.
`let x = 1` trong function A, function B cũng `let x = 2` → x bị overwrite.
Match bindings (`Expr::NumLit { value }`) cũng là global.

### Rule: SAVE trước recursive/nested call, RESTORE sau

```olang
// ĐÚNG:
push(_save_stack, my_var);     // save
some_function();                // có thể overwrite my_var
let my_var = pop(_save_stack); // restore

// SAI — BUG CHẮC CHẮN:
let my_var = something;
some_function();               // overwrite my_var!
use(my_var);                   // WRONG VALUE!
```

### Nơi PHẢI save/restore (đã fix, KHÔNG được bỏ):

| Vị trí | Biến cần save | Stack | File |
|--------|-------------|-------|------|
| BinOp compile | `rhs`, `op` | `_ce_stack` | semantic.ol |
| Call args compile | `_ce_saved_fname`, `_ce_saved_args`, `_ce_ai` | `_ce_stack` | semantic.ol |
| LetStmt compile | `_ls_name` | `_ce_stack` | semantic.ol |
| IfStmt compile | `_if_then`, `_if_else`, `_if_jz`, `_if_ti` | `_if_stack` | semantic.ol |
| ElseIf compile | `_if_else`, `_if_jmp`, `_if_ei` | `_if_stack` | semantic.ol |
| WhileStmt compile | `_wl_body`, `_wl_cond_start`, `_wl_cond_end`, `_wl_tokens` | `_ce_stack` | semantic.ol |
| WhileStmt body | `_wl_body`, `_wl_jz`, `_wl_start`, `_wl_bi` | `_ce_stack` | semantic.ol |
| FieldAssign compile | `_fa_obj` | `_ce_stack` | semantic.ol |
| Parser if-else | `_ps_cond`, `_ps_then` | `_pb_stack` | parser.ol |
| Parser call args | `_pp_result`, `_pp_call_args` | `_pb_stack` | parser.ol |
| Parser while | `_ps_wc_start`, `_ps_wc_end` | `_pb_stack` | parser.ol |
| parse_expr_prec | `_pep_lhs`, `ch`, `min_prec` | `_pep_stack` | parser.ol |
| parse_block | `_pb_stmts` | `_pb_stack` | parser.ol |

### Khi thêm code mới — Checklist:
1. Function gọi function khác? → Save locals trước, restore sau
2. Match arm gọi function? → Match bindings sẽ bị overwrite bởi inner match
3. Loop body gọi function? → Loop vars sẽ bị overwrite
4. Dùng prefix unique: `_ps_*` (parse_stmt), `_ce_*` (compile_expr), `_pep_*`, etc.

---

## Thêm patterns quan trọng

```
① RENAME all locals — Dùng prefix unique cho mỗi function:
    parse_stmt → _ps_*, parse_expr_prec → _pep_*, compile_expr → _ce_*
    LÝ DO: ASM VM có global var_table, không có block scope.

② SAVE recursive variables — Trước mỗi recursive call:
    push(_ce_stack, rhs);       // save
    compile_expr(lhs);          // recursive call
    let rhs = pop(_ce_stack);   // restore
    LÝ DO: Recursive call sẽ overwrite global vars.

③ Strings = u16 molecules — Mỗi char = 0x2100 | byte:
    Stride 2 cho mọi string builtin.
    codegen emit_str_u16: encode từng byte → 0x2100 | byte.

④ ARRAY_INIT_CAP = 4096 — Empty `[]` pre-allocates 4096 slots (64KB).
    ArrayLit `[1,2,3]` does NOT pre-allocate. Heap overlap risk with `[]`.

⑤ Missing builtins → .call_skip → stack leak:
    Nếu thêm function mới cần builtin chưa có → PHẢI implement trong ASM.

⑥ op_call: KHÔNG switch r12 cho nested eval closures.
    body_pc tương đối theo buffer hiện tại, không phải boot_bc_base.

⑦ Direct bytecode emission — Semantic emits bytes trực tiếp vào _g_output.
    Jump targets resolved bằng backpatch (set_at + patch_jump).
    KHÔNG dùng two-pass hay IR ops buffer.
```

---

## Build & Test

```bash
# Build native binary
make build                    # → origin_new.olang (~824KB)

# Test
echo 'emit 42' | ./origin_new.olang
echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)' | ./origin_new.olang

# Rust legacy tests (nếu sửa crates/)
cargo test --workspace
cargo clippy --workspace

# Full verify
make check-all
```

---

## Files quan trọng

| File | Vai trò |
|------|---------|
| `vm/x86_64/vm_x86_64.S` | ASM VM — trái tim (4,112 LOC) |
| `stdlib/bootstrap/lexer.ol` | Tokenizer (196 LOC) |
| `stdlib/bootstrap/parser.ol` | Parser recursive descent (718 LOC) |
| `stdlib/bootstrap/semantic.ol` | Semantic → IR opcodes (649 LOC) |
| `stdlib/bootstrap/codegen.ol` | IR → bytecode (302 LOC) |
| `stdlib/repl.ol` | REPL entry point (87 LOC) |
| `stdlib/homeos/*.ol` | HomeOS stdlib (36 files, 6,600 LOC) |
| `docs/olang_handbook.md` | Olang handbook |
| `docs/HomeOS_SPEC_v3.md` | HomeOS spec v3.1 |
| `TASKBOARD.md` | Task tracker |

---

## Chưa port sang Olang (cần làm)

| Rust module | LOC | Ưu tiên | Olang tương đương |
|-------------|-----|---------|-------------------|
| agents/encoder | 1,030 | CAO | text → molecule encoding |
| context/analysis | 2,108 | CAO | fusion, inference engines |
| olang/crypto | 2,736 | TRUNG BÌNH | SHA-256, AES, Ed25519 |
| runtime/core | 7,512 | TRUNG BÌNH | full pipeline orchestration |
| vsdf/dynamics | 5,125 | THẤP | physics simulation |
| wasm/lib | 15,270 | THẤP | WASM runtime |
| hal/detect | 3,500 | THẤP | platform detection |

---

## Tài liệu

| File | Nội dung |
|------|---------|
| `docs/olang_handbook.md` | Olang đầy đủ: lexer · parser · IR · VM · opcodes |
| `docs/HomeOS_SPEC_v3.md` | HomeOS spec v3.1 |
| `docs/MILESTONE_20260323.md` | Self-hosting milestone |
| `PLAN_REWRITE.md` | Lộ trình Rust → Olang (7 giai đoạn) |
| `crates/EPITAPH.md` | Lời mặc niệm cho Rust legacy |
| `old/MEMORIAL.md` | Tài liệu lịch sử |
