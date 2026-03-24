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
origin_new.olang = ~928KB native binary (ELF64, no libc, no deps)

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

// String interpolation
let name = "World";
emit $"Hello {name}!";       // Hello World!
emit $"x = {x}, y = {y}";   // works with any expression

// Array comprehension
emit [x * 2 for x in [1,2,3]];           // [2, 4, 6]
emit [x for x in items if x > 3];        // filter

// Try/catch
try { __throw("error"); } catch { emit "caught"; };

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
len(a)  push(a, val)  pop(a)  a[i]  set_at(a, i, val)
__array_new(count)  __array_get(a, i)  __array_push(a, val)
__array_pop(a)  __array_range(n) → [0..n-1]
// NOTE: a[i] is desugared to __array_get(a, i) by the parser

// Dict
__dict_new(field_count)  __dict_get(dict, key)  __dict_set(dict, key, val)
struct_tag(dict, tag) → new dict with tag
__match_enum(dict, tag) → bool
__enum_field(dict, idx) → value
__enum_unit(tag) → unit variant
__dict_keys(dict) → array of keys

// Comparison (all return f64: 1.0 or 0.0)
__eq(a,b)  __cmp_ne(a,b)  __cmp_lt(a,b)  __cmp_gt(a,b)  __cmp_le(a,b)  __cmp_ge(a,b)

// Logic
__logic_not(x) → !x

// Bitwise
__bit_or(a,b)  __bit_and(a,b)  __bit_xor(a,b)

// I/O
emit expr           // print to stdout
__eval_bytecode(bc) // execute compiled bytecode

// Math
__floor(x)  __ceil(x)

// Bitwise
__bit_or(a,b)  __bit_and(a,b)  __bit_xor(a,b)  __bit_shl(a,n)  __bit_shr(a,n)

// Crypto
__sha256(str) → 64-char hex string (FIPS 180-4)

// Error handling
__throw(msg) → unwind to nearest try/catch

// File I/O
__file_read(path) → string contents (u16 molecules)
__file_write(path, content) → write string to file
__file_read_bytes(path) → raw byte buffer
__bytes_new(size) → zeroed byte buffer
__bytes_get(buf, offset) → byte as f64
__bytes_set(buf, offset, value) → write byte
__bytes_write(path, buf, size) → write raw bytes to file
__bytes_len(buf) → buffer size
__heap_save() → checkpoint index    __heap_restore(idx) → reset heap

// Type
__type_of(x) → "number"/"string"/"array"/"dict"/"closure"

// Conversion
__f64_to_le_bytes(n) → array of 8 bytes
__to_string(n) → string    __to_number(s) → number
__char_code(ch) → codepoint number
```

---

## ASM VM — Khi cần sửa (vm/x86_64/vm_x86_64.S)

### Bytecode opcodes (bc_format=1)

```
0x01 Push(str)       0x0B Dup             0x19 PushMol(5 bytes)
0x02 Load(name)      0x0C Pop             0x1A TryBegin(catch_offset)
0x06 Emit            0x0D Swap            0x1B CatchEnd
0x07 Call(name)      0x0F Halt            0x1C StoreUpdate(name)
0x08 Ret             0x13 Store(name)     0x24 CallClosure
0x09 Jmp(offset)     0x14 LoadLocal(name) 0x25 Closure(body_len)
0x0A Jz(offset)      0x15 PushNum(f64)
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

④ ARRAY_INIT_CAP = 16384 — Empty `[]` pre-allocates 16384 slots (256KB).
    ArrayLit `[1,2,3]` does NOT pre-allocate. _g_output uses 16384 slots (16KB bytecode).

⑤ Missing builtins → .call_skip → stack leak:
    Nếu thêm function mới cần builtin chưa có → PHẢI implement trong ASM.

⑥ op_call: KHÔNG switch r12 cho nested eval closures.
    body_pc tương đối theo buffer hiện tại, không phải boot_bc_base.

⑦ Direct bytecode emission — Semantic emits bytes trực tiếp vào _g_output.
    Jump targets resolved bằng backpatch (set_at + patch_jump).
    KHÔNG dùng two-pass hay IR ops buffer.
```

---

## REPL Commands

```
emit <expr>              Evaluate and print expression
let x = 42               Define variable
fn f(x) { ... }          Define function
encode <text>            Show molecular encoding + intent + tone + context
respond <text>           Full agent response (with STM memory)
learn <text>             Teach HomeOS a fact
learn_file <path>        Read file and learn each line as fact
compile <path>           Compile .ol file → show bytecode size
build                    Self-build: compile + pack → origin_built.olang
test                     Run 16 inline tests
memory                   Show STM turns + Silk edges + Knowledge facts
help                     Show available commands
exit / quit              Exit REPL
<natural text>           Auto-detect: non-code → agent_respond (Vietnamese OK)
```

## Memory Systems (STM + Silk + Dream + Knowledge)

```
STM (Short-Term Memory):
  Global array, max 32 turns. Each: { input, intent, tone, turn }
  stm_push(), stm_last_input(), stm_count(), stm_find_related()

Silk (Hebbian Learning):
  Co-activate word bigrams on each input. Max 64 edges.
  silk_learn_from_text(), silk_find_related(), silk_count()

Dream (Consolidation):
  Runs every 5 turns. Scans STM for repeated intent patterns.
  dream_cycle()

Knowledge Store:
  Learned facts from `learn` command. Max 512 entries.
  knowledge_learn(), knowledge_search(), knowledge_count()
  Retrieval: split query → match keywords → best scoring fact
```

## Phase 5 — Intelligence Layer (P5)

```
10-Stage Pipeline: input → alias → emoji → UDC encode → node → Learning → DN/QR
                   ← UDC decode ← emoji ← alias ← output

Alias System:      31 Vietnamese slang mappings (ko→khong, dc→duoc, bn→ban)
                   10 emoji shortcodes (:)→😊, :(→😢, <3→❤)
UTF-8 Decoder:     utf8_decode() — 1-4 byte sequences → full Unicode codepoint
Emoji Emotion:     25+ emoji with fine-grained V/A (😊=V7/A6, 😭=V1/A6, 😡=V1/A7)
                   text_emotion_unicode() — scan text for emoji → extract V/A from molecule
Word Affect:       72 entries (Vietnamese + English), Vietnamese stemming (negator/intensifier)
                   text_emotion_v2() — word + emoji fusion (70% emoji / 30% word)
Emotion Carry:     EMA 60/40 across turns, streak detection (3+ same → bias tone)
Personality:       14 template globals, set_personality("formal"|"casual"|"english")
Context Window:    STM max 32 turns, auto-digest when >16 (compress + evict)
DN/QR Nodes:       SHA-256 addressed nodes, dedup, fire counting, bidirectional linking
                   qr_search() — keyword matching weighted by fire count
UDC Decoder:       molecule → mood label (Russell's circumplex), emoji_for_emotion()
Auto-Learn:        _boot_learn() loads training data on first REPL call
Training Data:     docs/training/ — 6 files, 661 entries auto-loaded at boot
Self-Compile:      lexer.ol compiles in 1.0s (was hanging — nested dict/struct fix)
```

---

## Build & Test

```bash
# Build native binary
make build                    # → origin_new.olang (~928KB)

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
| `vm/x86_64/vm_x86_64.S` | ASM VM — trái tim (5,471 LOC) |
| `stdlib/bootstrap/lexer.ol` | Tokenizer (259 LOC) |
| `stdlib/bootstrap/parser.ol` | Parser recursive descent (988 LOC) |
| `stdlib/bootstrap/semantic.ol` | Semantic → direct bytecode emission (1,337 LOC) |
| `stdlib/bootstrap/codegen.ol` | Codegen helpers (429 LOC) |
| `stdlib/repl.ol` | REPL entry point (322 LOC) |
| `stdlib/homeos/*.ol` | HomeOS stdlib (43 files, 8,910 LOC) |
| `docs/olang_handbook.md` | Olang handbook |
| `docs/HomeOS_SPEC_v3.md` | HomeOS spec v3.1 |
| `TASKBOARD.md` | Task tracker |

---

## Port status (Rust → Olang)

| Rust module | Status | Olang files |
|-------------|--------|-------------|
| agents/encoder | ✅ DONE (OL.1) | `encoder.ol` — text→molecule, block-range UCD |
| context/analysis | ✅ DONE (OL.2-3) | `encoder.ol` — fusion, intent, context detect |
| agents/pipeline | ✅ DONE (OL.4-5) | `encoder.ol` — agent dispatch, response composer |
| olang/crypto | ✅ SHA-256 (OL.13) | `vm_x86_64.S` — `__sha256()` FIPS 180-4 |
| runtime/core | PARTIAL | `repl.ol` — REPL + `encode`/`respond` commands |
| vsdf/dynamics | THẤP | chưa port |
| wasm/lib | ✅ WASM VM (OL.12) | `vm_wasm.wat` — runs in browser |
| hal/detect | THẤP | chưa port |

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
