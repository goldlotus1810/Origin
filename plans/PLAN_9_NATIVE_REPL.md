# PLAN 9 — Native REPL: Make ./origin Interactive

**Phụ thuộc:** Phase 8 (parser upgrade), Phase 0 (bootstrap compiler), Phase 1 (VM)
**Mục tiêu:** Chạy `./origin` → REPL thật, compile + execute user input, hiển thị kết quả

---

## Bối cảnh

```
HIỆN TẠI:
  vm_x86_64.S có REPL skeleton:
    1. Print greeting "origin.olang v0.1 ready"
    2. Read stdin (256 bytes max)
    3. Check "exit"/"quit"
    4. ECHO INPUT BACK (placeholder)
    5. Loop

  Bootstrap compiler ĐÃ CÓ trong bytecode:
    tokenize(source) → tokens        (lexer.ol)
    parse(tokens) → ast              (parser.ol)
    analyze(ast) → semantic_state    (semantic.ol)
    generate(ops) → bytecode_bytes   (codegen.ol)

  NHƯNG:
    ❌ REPL không gọi được compiler — chỉ echo
    ❌ Bootstrap functions không registered trong var_table
    ❌ Thiếu builtins: push, set_at (compiler cần)
    ❌ Không có bytecode layering (nested execution)
    ❌ Error handling là stub (TryBegin/CatchEnd)

SAU PLAN 9:
  $ ./origin
  ○ HomeOS v0.05 · 54 modules · 246 atoms
  ○ > let x = 42
  ○ > emit x + 1
  43
  ○ > emit "hello " + "world"
  hello world
  ○ > fn fib(n) { if n < 2 { return n; } return fib(n-1) + fib(n-2); }
  ○ > emit fib(10)
  55
  ○ > exit
```

---

## Kiến trúc REPL

```
┌─────────────────────────────────────────────────┐
│ vm_x86_64.S                                     │
│                                                  │
│  _start → load binary → vm_loop (boot bytecode) │
│    ↓                                             │
│  op_halt → REPL mode                            │
│    ↓                                             │
│  .repl_loop:                                    │
│    read stdin → line                             │
│    ↓                                             │
│  .repl_compile:                                 │
│    CALL "tokenize"(line) → tokens               │
│    CALL "parse"(tokens)  → ast                  │
│    CALL "analyze"(ast)   → state                │
│    CALL "generate"(state.ops) → bytecode        │
│    ↓                                             │
│  .repl_execute:                                 │
│    save r12,r13 (old bytecode context)           │
│    set r12 = new_bytecode, r13 = 0              │
│    jmp vm_loop                                   │
│    ↓                                             │
│  nested op_halt → restore r12,r13               │
│    jmp .repl_loop                               │
│                                                  │
└─────────────────────────────────────────────────┘

Hai execution contexts:
  Context 0: Boot bytecode (pre-compiled stdlib)
    r12 = boot_bytecode_ptr
    r13 = boot_pc → runs to Halt → enters REPL

  Context 1: REPL bytecode (dynamically generated)
    r12 = heap_bytecode_ptr (from REPL compilation)
    r13 = 0 → runs to Halt → returns to REPL

State persistence:
  var_table, Silk graph, STM → survive between REPL iterations
  Mỗi "let x = 42" → x persists cho câu tiếp theo
```

---

## Tasks

### 9.1 — Bootstrap Module Registration (~200 LOC ASM)

**File:** `vm/x86_64/vm_x86_64.S`

```
HIỆN TẠI:
  vm_loop dispatches opcodes sequentially
  Boot bytecode = ALL .ol files concatenated
  Functions defined via FnDef → stored in var_table by name
  BUT: after boot Halt, var_table has all stdlib functions

THE GOOD NEWS:
  Khi boot bytecode chạy (all 54 .ol files), mọi pub fn sẽ tự
  register vào var_table qua FnDef opcode.

  tokenize() từ lexer.ol    → var_table["tokenize"] = body_pc
  parse() từ parser.ol      → var_table["parse"] = body_pc
  analyze() từ semantic.ol  → var_table["analyze"] = body_pc
  generate() từ codegen.ol  → var_table["generate"] = body_pc

  ĐÃ SẴN SÀNG — không cần đăng ký thêm!

CẦN LÀM:
  1. Verify: sau boot, var_table chứa "tokenize", "parse", "analyze", "generate"
  2. Debug: thêm op_trace_var_table() để dump var_table contents (dev mode)
  3. Fix: nếu function names bị mangle (module prefix etc.)
```

**Test:**
```
Sau boot bytecode chạy xong (Halt), check var_table:
  var_load_hash(hash("tokenize")) → body_pc != 0
  var_load_hash(hash("parse"))    → body_pc != 0
  var_load_hash(hash("analyze"))  → body_pc != 0
  var_load_hash(hash("generate")) → body_pc != 0
```

---

### 9.2 — Missing VM Builtins (~150-200 LOC ASM)

**File:** `vm/x86_64/vm_x86_64.S`

Bootstrap compiler cần những builtins này:

```
HIỆN CÓ:                    THIẾU:
  __len(array)     ✅          __push(array, value)    ❌
  __concat(a, b)   ✅          __pop(array)            ❌
  __char_at(s, i)  ✅          __set_at(array, i, val) ❌
  __substr(s,a,b)  ✅          __str_bytes(s)          ❌
  __hyp_add/sub    ✅          __f64_to_le_bytes(n)    ❌
  __eq, __cmp_*    ✅          __type_of(val)          ❌
                               __to_number(s)          ❌
                               __keys(obj)             ❌

Mỗi builtin: 15-30 LOC ASM

__push(array, value):
  Pop value, pop array_ptr
  Load array.len from header
  Write value at array.data[len]
  Increment array.len
  Push array_ptr back

__pop(array):
  Pop array_ptr
  Load array.len
  Decrement len
  Load value at array.data[len-1]
  Push value

__set_at(array, index, value):
  Pop value, pop index, pop array_ptr
  Write value at array.data[index]
  Push array_ptr back

__str_bytes(string):
  Pop string (ptr, len)
  Allocate array on heap
  Copy each byte as integer element
  Push array_ptr

__to_number(string):
  Pop string (ptr, len)
  Parse decimal/hex/float → f64
  Push f64

__type_of(value):
  Pop value
  Check value tag (f64, string, array, dict)
  Push type name as string
```

**Rào cản:** VM memory layout cho arrays/dicts
- Hiện tại VM dùng f64 stack → arrays/dicts cần ptr representation
- Cần quy ước: array = [tag:8][len:8][cap:8][data:N*8]
- Dict = [tag:8][count:8][entries: [hash:8][key_ptr:8][val:8]*N]

**CRITICAL:** Phải align memory layout giữa:
- vm_x86_64.S (native execution)
- Olang compiler output (bytecode format)
- Rust VM (reference implementation)

Nếu layouts khác nhau → bootstrap compiler sẽ produce wrong bytecode.

---

### 9.3 — REPL Compile Pipeline (~100-150 LOC ASM)

**File:** `vm/x86_64/vm_x86_64.S` — replace `.repl_echo` section

```asm
.repl_compile:
    # r15 = input buffer ptr, rcx = input length
    # Step 1: Push input string onto VM stack
    # String representation: push (ptr, len) as tagged value

    # Step 2: Call tokenize (already in var_table from boot)
    # Equivalent to: tokens = tokenize(input_str)
    # → Construct Call opcode sequence on heap
    # → Set r12 = call_sequence, r13 = 0
    # → jmp vm_loop → returns tokens on stack

    # Step 3: Call parse(tokens)
    # → tokens already on stack from Step 2

    # Step 4: Call analyze(ast)
    # → ast already on stack from Step 3

    # Step 5: Extract .ops from SemanticState
    # → Field access on struct result

    # Step 6: Call generate(ops)
    # → Returns bytecode bytes array

    # Step 7: Execute generated bytecode
    jmp .repl_execute

.repl_execute:
    # Save current execution context
    push %r12           # save boot bytecode ptr
    push %r13           # save boot pc (after Halt)

    # Set up new execution context
    # r12 = generated bytecode (from heap)
    # r13 = 0 (start of new bytecode)
    mov %rax, %r12      # new bytecode ptr (from generate() result)
    xor %r13d, %r13d    # PC = 0

    jmp vm_loop         # execute!
    # When nested Halt reached → needs to return here

.repl_halt_return:
    # Restore boot context
    pop %r13
    pop %r12
    jmp .repl_loop      # back to prompt
```

**ALTERNATIVE — Simpler Approach:**

Thay vì gọi 4 functions riêng lẻ, tạo 1 "repl_entry" function trong Olang:

```olang
// repl.ol — REPL entry point (NEW FILE)
pub fn repl_eval(input) {
  let tokens = tokenize(input);
  if len(tokens) == 0 { return ""; }

  let ast = parse(tokens);
  if ast.error { return "Parse error: " + ast.error; }

  let state = analyze(ast);
  if len(state.errors) > 0 { return "Error: " + state.errors[0]; }

  let bytecode = generate(state.ops);
  return __eval_bytecode(bytecode);  // NEW VM builtin
}
```

Rồi VM chỉ cần: `CALL "repl_eval"(input_string)` — 1 lần gọi duy nhất.

**Builtin mới:**
```
__eval_bytecode(bytes):
  Save r12, r13
  Copy bytes to heap as bytecode buffer
  Set r12 = buffer, r13 = 0
  Run vm_loop (nested)
  On Halt: restore r12, r13
  Return output from vm_emit_buffer
```

---

### 9.4 — Bytecode Layering (~50-80 LOC ASM)

```
Hỗ trợ nested execution:
  Boot bytecode → CALL repl_eval → nested bytecode → Halt → return

Implementation:
  Execution stack (separate from data stack):
    [exec_context:0] = { r12=boot_bc, r13=boot_pc, r14=boot_sp }
    [exec_context:1] = { r12=repl_bc, r13=repl_pc, r14=repl_sp }

  On nested Halt:
    if exec_stack_depth > 0:
      pop exec_context → restore r12, r13, r14
      continue vm_loop
    else:
      enter REPL mode (current behavior)

  Max depth: 2-3 levels (boot → REPL → eval)
```

---

### 9.5 — REPL Greeting & UX (~50-100 LOC ASM/Olang)

```
HIỆN TẠI: "origin.olang v0.1 ready\n"

SAU:
  ○ HomeOS v0.05
  ○ 54 modules · 246 atoms · 187 silk edges
  ○ Type text to chat · ○{help} for commands · Ctrl+C to exit
  ○ > _

  Features:
    - History: ↑↓ arrow keys (ring buffer, last 50 lines)
    - Error display: parse/compile errors in red (ANSI \033[31m)
    - Output: emit results in normal color
    - Prompt: ○ > (2 UTF-8 bytes + space + cursor)
    - Multiline: trailing { or \ → continue on next line
```

---

### 9.6 — Natural Text Mode (Emotion Pipeline)

```
REPL phải handle 2 loại input:

  1. Olang code: "let x = 42" hoặc "○{stats}"
     → compile → execute → show output

  2. Natural text: "tôi buồn vì mất việc"
     → emotion pipeline → response

Detection (giống runtime/parser.rs):
  - Starts with "let"/"fn"/"if"/"while"/"emit"/"match" → Olang code
  - Starts with "○{" → Olang command
  - Otherwise → natural text → emotion pipeline

Natural text pipeline (trong bytecode, từ Phase 2 .ol files):
  gate_check(text) → allow/crisis/block
  text_to_mol(text) → molecular chain
  run_instincts(observation, knowledge)
  stm_push(chain, emotion, timestamp)
  co_activate_text(words, silk_graph)
  walk_emotion(silk_graph, words) → amplified emotion
  curve_push(emotion) → curve_tone() → tone
  render(tone, content) → response string
  emit response

Entry function: process_text(input) — cần viết trong Olang
```

---

## DoD (Definition of Done)

```
✅ ./origin chạy → hiển thị greeting với module count
✅ Gõ "let x = 42" → x lưu trong var_table
✅ Gõ "emit x + 1" → hiển thị "43"
✅ Gõ "emit fib(10)" (sau khi define fib) → hiển thị "55"
✅ Gõ "tôi buồn vì mất việc" → emotion-aware response
✅ Gõ "exit" → clean exit
✅ Parse/compile errors → hiển thị error message (không crash)
✅ Variables persist giữa REPL lines
✅ cargo test --workspace pass (không break existing)
```

---

## Effort Estimate

```
9.1 Module registration:    verify + debug, 1-2h
9.2 Missing builtins:       150-200 LOC ASM, 3-5h
9.3 REPL compile pipeline:  100-150 LOC ASM/Olang, 3-5h
9.4 Bytecode layering:      50-80 LOC ASM, 2-3h
9.5 REPL UX:                50-100 LOC, 1-2h
9.6 Natural text mode:      100-200 LOC Olang, 2-4h

TỔNG: ~500-700 LOC, 12-20h
```

---

## Rào cản & Mitigation

```
Rào cản                              Mitigation
───────────────────────────────────────────────────────────
Memory layout mismatch               → Document + test layout giữa ASM/Rust/Olang
  (array/dict representation)          Viết layout_test.ol để verify

Bootstrap compiler chưa handle       → Phase 8 PHẢI xong trước (hex literals etc.)
  tất cả syntax                        REPL chỉ cần handle basic syntax đầu tiên

Nested execution complexity           → Start simple: __eval_bytecode builtin
                                       Không cần full context switch lúc đầu
                                       Nested 1 level đủ cho REPL

Error handling (TryBegin stub)        → REPL wrap trong try/catch:
                                       parse error → print error, không crash
                                       runtime error → print error, continue
                                       Nếu TryBegin quá phức tạp → manual check

String encoding mismatch              → VM string = (ptr, len) trên stack
  (UTF-8 vs byte array)                 Olang string = byte array
                                       Cần adapter hoặc unify

var_table overflow (256 limit)        → 54 files × ~5 pub fn = ~270 entries
                                       Cần tăng limit lên 512 hoặc 1024
                                       Hoặc: dùng module prefix + hash table
```

---

## Dependencies

```
MUST HAVE FIRST:
  ✅ Phase 8 (parser upgrade) — compiler phải parse tất cả syntax
  ✅ Phase 0 (bootstrap compiler) — tokenize/parse/analyze/generate
  ✅ Phase 1.1 (vm_x86_64.S) — REPL skeleton

NICE TO HAVE:
  Phase 2 (emotion pipeline .ol) — cho natural text mode
  Phase 7.4 (ISL transport) — cho network mode
```
