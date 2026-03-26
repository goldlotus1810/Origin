# PLAN 10 — Browser E2E: origin.html Hoạt Động Thật

**Phụ thuộc:** Phase 8 (parser), Phase 9 (REPL concept), Phase 1.3 (vm_wasm.wat), Phase 4.3 (wasm_emit.ol)
**Mục tiêu:** Mở origin.html trong browser → gõ text → thấy kết quả. Không cần backend.

---

## Bối cảnh

```
HIỆN TẠI:
  origin.html:
    ✅ UI: textarea input, output pane, drag & drop
    ✅ Auto-load vm_wasm.wasm từ cùng directory
    ❌ Input handler: "[input received — REPL compile not yet wired]"
    ❌ KHÔNG compile user input → KHÔNG execute → KHÔNG output

  vm_wasm.wat:
    ✅ 36 opcode dispatch
    ✅ 12 builtins (math, string, comparison)
    ✅ Bytecode embedded trong data section
    ❌ Không có REPL entry point
    ❌ Thiếu push/pop/set_at builtins (giống vm_x86_64.S)

  vm_wasi.wat:
    ✅ WASI I/O wrappers
    ❌ Bytecode size hardcoded (cần patching)
    ❌ Không có stdin REPL

SAU PLAN 10:
  origin.html mở trong browser:
    1. WASM VM loads, boots (executes stdlib bytecode)
    2. User gõ "let x = 42" → compile → execute → silent
    3. User gõ "emit x + 1" → compile → execute → "43" hiện trong output
    4. User gõ "tôi buồn" → emotion response hiện trong output
    5. Drag & drop .ol file → compile → execute → output
```

---

## Kiến trúc

```
┌─────────────────────────────────────────────────┐
│ origin.html (Browser)                            │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │ <textarea id="input">                     │   │
│  │   User types here                         │   │
│  └──────────────────────────────────────────┘   │
│           ↓ onKeyDown(Enter)                    │
│  ┌──────────────────────────────────────────┐   │
│  │ JS: repl_eval(input_text)                 │   │
│  │   1. Encode text as UTF-8 bytes           │   │
│  │   2. Copy to WASM linear memory           │   │
│  │   3. Call WASM export: eval(ptr, len)     │   │
│  │   4. Read output from WASM memory         │   │
│  │   5. Display in output pane               │   │
│  └──────────────────────────────────────────┘   │
│           ↓                                      │
│  ┌──────────────────────────────────────────┐   │
│  │ vm_wasm.wat (WebAssembly)                 │   │
│  │                                           │   │
│  │  (export "eval" (func $eval))             │   │
│  │    → Push string onto VM stack            │   │
│  │    → Call "repl_eval" (Olang function)    │   │
│  │    → Capture emit output                  │   │
│  │    → Return output ptr + len              │   │
│  │                                           │   │
│  │  (export "boot" (func $boot))             │   │
│  │    → Execute all boot bytecode            │   │
│  │    → Register stdlib functions            │   │
│  │    → Return module count                  │   │
│  │                                           │   │
│  │  (import "env" "host_write")              │   │
│  │    → JS captures output                   │   │
│  │    → Appends to output buffer             │   │
│  └──────────────────────────────────────────┘   │
│           ↓                                      │
│  ┌──────────────────────────────────────────┐   │
│  │ <pre id="output">                         │   │
│  │   43                                      │   │
│  │   hello world                             │   │
│  └──────────────────────────────────────────┘   │
│                                                  │
└─────────────────────────────────────────────────┘
```

---

## Tasks

### 10.1 — WASM VM: boot + eval exports (~100-150 LOC WAT)

**File:** `vm/wasm/vm_wasm.wat`

```wat
;; NEW: Boot function — executes all pre-loaded bytecode
(func $boot (export "boot") (result i32)
  ;; Set bytecode pointer and size from embedded data
  ;; Execute vm_loop until Halt
  ;; Return number of registered functions
  ...
)

;; NEW: Eval function — compile and execute one line
(func $eval (export "eval") (param $ptr i32) (param $len i32) (result i32)
  ;; 1. Push input string onto VM stack
  ;; 2. Call "repl_eval" (registered from repl.ol)
  ;; 3. Execute compiled bytecode (nested)
  ;; 4. Return output buffer ptr (output captured by host_write)
  ...
)

;; NEW: Get output — read captured emit output
(func $get_output (export "get_output") (result i32 i32)
  ;; Return (ptr, len) of output buffer
  ;; Reset buffer for next eval
  ...
)
```

**Missing builtins (same as Phase 9):**
```
__push, __pop, __set_at, __str_bytes, __to_number, __type_of
```

---

### 10.2 — origin.html: Wire Input → WASM → Output (~100-150 LOC JS)

**File:** `vm/wasm/origin.html`

```javascript
// NEW: REPL integration
let wasmInstance = null;
let outputBuffer = '';

async function initVM() {
  const imports = {
    env: {
      host_write(ptr, len) {
        // Capture VM output
        const bytes = new Uint8Array(wasmInstance.exports.memory.buffer, ptr, len);
        outputBuffer += new TextDecoder().decode(bytes);
      },
      host_read(ptr, len) { return 0; }, // Not needed in browser
      host_log(ptr, len) {
        const bytes = new Uint8Array(wasmInstance.exports.memory.buffer, ptr, len);
        console.log(new TextDecoder().decode(bytes));
      }
    }
  };

  const wasm = await WebAssembly.instantiateStreaming(fetch('vm_wasm.wasm'), imports);
  wasmInstance = wasm.instance;

  // Boot: execute stdlib bytecode
  const moduleCount = wasmInstance.exports.boot();
  appendOutput(`○ HomeOS WASM · ${moduleCount} modules loaded\n○ > `);
}

function evalInput(text) {
  outputBuffer = '';
  const encoder = new TextEncoder();
  const bytes = encoder.encode(text);

  // Copy to WASM memory
  const ptr = wasmInstance.exports.alloc(bytes.length);
  new Uint8Array(wasmInstance.exports.memory.buffer).set(bytes, ptr);

  // Call eval
  wasmInstance.exports.eval(ptr, bytes.length);

  // Display output
  if (outputBuffer) {
    appendOutput(outputBuffer + '\n');
  }
  appendOutput('○ > ');
}

document.getElementById('input').addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    const text = e.target.value.trim();
    if (text) {
      appendOutput(text + '\n');
      evalInput(text);
      e.target.value = '';
    }
  }
});
```

---

### 10.3 — origin.html UI Redesign (~100-150 LOC HTML/CSS)

```
HIỆN TẠI: Basic textarea + drag & drop zone
SAU:
  ┌─────────────────────────────────────────────────┐
  │  ○ HomeOS                          WASM · 54 mod │
  ├─────────────────────────────────────────────────┤
  │                                                  │
  │  ○ HomeOS WASM v0.05 · 54 modules · 246 atoms   │
  │                                                  │
  │  bạn: let x = 42                                │
  │  bạn: emit x + 1                               │
  │  ○: 43                                          │
  │                                                  │
  │  bạn: tôi buồn vì mất việc                     │
  │  ○: Cảm giác nặng nề — bạn muốn kể thêm?      │
  │                                                  │
  │                                                  │
  ├─────────────────────────────────────────────────┤
  │  ○ > [________________input________________]     │
  └─────────────────────────────────────────────────┘

  Features:
    - Chat-style output (scrollable, auto-scroll down)
    - Input at bottom (fixed position)
    - Monospace font (code-friendly)
    - Color coding: user input = gray, output = white, errors = red
    - Dark theme (default)
    - No external CSS/JS dependencies
    - Single HTML file (inline everything)
    - Responsive (mobile-friendly)
```

---

### 10.4 — Drag & Drop .ol Files

```
HIỆN TẠI: Drag & drop .wasm files
SAU: Drag & drop .ol source files → compile → execute

  flow:
    1. User drags emotion.ol onto browser
    2. JS reads file as text
    3. JS calls wasmInstance.exports.eval(text_ptr, text_len)
    4. WASM compiles .ol source → bytecode → execute
    5. Functions defined in .ol → available for subsequent REPL input
    6. Output from file execution → displayed in output pane

  Also supports:
    - .wasm binary → load as VM module
    - .bin bytecode → execute directly
    - .ol source → compile + execute
```

---

### 10.5 — WASI CLI Parity (~50-100 LOC WAT)

**File:** `vm/wasm/vm_wasi.wat`

```
HIỆN TẠI:
  Fixed bytecode embedded
  No stdin REPL

SAU:
  wasmtime vm_wasi.wasm
    → Boot (execute embedded bytecode)
    → REPL loop: read stdin → compile → execute → write stdout
    → Same experience as native ./origin

  Cần:
    - fd_read(0, ...) for stdin
    - fd_write(1, ...) for stdout
    - Same boot + eval flow as vm_wasm.wat
    - Bytecode size auto-patched by builder (bc_size field)
```

---

## DoD (Definition of Done)

```
✅ origin.html mở trong Chrome/Firefox/Safari → WASM loads OK
✅ Gõ "emit 1 + 2" → hiển thị "3"
✅ Gõ "let x = 42" rồi "emit x" → hiển thị "42"
✅ Gõ Vietnamese text → emotion response
✅ Drag & drop .ol file → compile + execute → output
✅ wasmtime vm_wasi.wasm → REPL works
✅ Không external dependencies (no npm, no CDN, no build step)
✅ Single origin.html file < 50KB (inline CSS/JS)
✅ Mobile responsive (iOS Safari, Android Chrome)
```

---

## Effort Estimate

```
10.1 WASM boot + eval:     100-150 LOC WAT, 3-5h
10.2 JS wire:              100-150 LOC JS, 2-3h
10.3 UI redesign:          100-150 LOC HTML/CSS, 2-3h
10.4 Drag & drop .ol:      50-80 LOC JS, 1-2h
10.5 WASI parity:          50-100 LOC WAT, 2-3h

TỔNG: ~400-600 LOC, 10-16h
```

---

## Rào cản & Mitigation

```
Rào cản                              Mitigation
───────────────────────────────────────────────────────────
WASM memory management               → Use linear memory with bump allocator
  (no GC in WASM)                      Pre-allocate 16MB, never free
                                       Reset allocator between REPL evals

UTF-8 encoding mismatch              → JS TextEncoder/TextDecoder
  (JS string vs WASM bytes)            Always pass bytes, never strings

WASM export function limits          → boot + eval + alloc + get_output = 4 exports
                                       Enough for REPL

Large bytecode embedding             → Split: boot VM in .wasm, bytecode in .bin
                                       origin.html loads both
                                       Or: single .wasm with bytecode in data section

Browser security (CORS)              → All inline in single HTML file
                                       No external fetches needed
                                       Or: simple python -m http.server for dev
```
