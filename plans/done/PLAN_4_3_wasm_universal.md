# PLAN 4.3 — WASM Universal

**Phụ thuộc:** Phase 3 DONE
**Mục tiêu:** origin.olang.wasm = chạy mọi nơi có browser/WASI runtime
**Tham chiếu:** `vm/wasm/vm_wasm.wat`, `vm/wasm/host.js`

---

## Bối cảnh

```
HIỆN TẠI:
  vm_wasm.wat (655 LOC) = standalone WASM VM, 3KB .wasm
  host.js = Node.js test harness
  5/5 tests pass (hello, math, vars, loop, cmp)
  NHƯNG: chưa pack bytecode vào .wasm, chưa có browser UI

SAU PLAN 4.3:
  origin.olang.wasm = vm_wasm.wasm + bytecode (data section)
  origin.html = 1 HTML file, embed WASM, chạy trong browser
  WASM universal: browser, Node.js, Deno, Cloudflare Workers, wasmtime
```

---

## Tasks

### 4.3.1 — WASM bytecode embedding (~100 LOC WAT)

Bytecode embed trực tiếp vào WASM data section:

```wat
;; Data section — bytecode embedded at compile time
(data (i32.const 0x100000) "\xe2\x97\x8b\x4c...")  ;; bytecode bytes

;; Or: memory import từ host
(import "env" "bytecode_ptr" (global $bc_ptr i32))
(import "env" "bytecode_len" (global $bc_len i32))
```

**Hai approach:**

**Option A: Static embed (preferred for distribution)**
- builder.ol pack bytecode vào data section của .wasm
- .wasm file tự chứa mọi thứ
- Không cần fetch riêng

**Option B: Dynamic load (preferred for development)**
- host.js đọc bytecode file → pass vào WASM memory
- Linh hoạt hơn, dễ test
- Đã có cơ sở trong host.js hiện tại

### 4.3.2 — wasm_emit.ol (~200 LOC)

Olang emit WASM binary format (thay vì WAT text):

```
WASM binary format:
  [magic]     4B   \x00\x61\x73\x6D
  [version]   4B   \x01\x00\x00\x00
  [sections]  ...

Sections cần emit:
  Type section     (1)  — function signatures
  Import section   (2)  — env.read, env.write, env.mmap
  Function section (3)  — function indices
  Memory section   (5)  — memory declaration
  Export section    (7)  — _start, memory
  Code section     (10) — function bodies
  Data section     (11) — embedded bytecode
```

**Giải pháp đơn giản:** emit WAT text → dùng external `wat2wasm`
**Giải pháp tự đủ:** emit WASM binary trực tiếp (phức tạp hơn, nhưng 0 dependency)

Đề xuất: bắt đầu với WAT text (dùng công cụ có sẵn), sau đó viết binary emitter.

### 4.3.3 — Browser host (~150 LOC HTML/JS)

```html
<!-- origin.html — embedded trong origin.olang -->
<!DOCTYPE html>
<html>
<body>
  <div id="output"></div>
  <input id="input" placeholder="○ >" />
  <script>
    // Load WASM
    const wasm = await WebAssembly.instantiateStreaming(
      fetch('origin.olang.wasm'),
      {
        env: {
          write: (ptr, len) => { /* append to #output */ },
          read:  (ptr, len) => { /* read from #input */ },
          mmap:  (len) => { /* grow WASM memory */ }
        }
      }
    );
    wasm.instance.exports._start();
  </script>
</body>
</html>
```

Features:
- Terminal emulator (ANSI escape → CSS)
- ConversationCurve color mapping (tone → text color)
- UTF-8 full support
- No framework, no npm, no build step
- Có thể embed HTML vào data section của .wasm

### 4.3.4 — WASI support (~50 LOC WAT)

```
WASI = WebAssembly System Interface
  fd_read, fd_write, proc_exit
  Cho phép chạy WASM ngoài browser: wasmtime, wasmer, Node.js

Thay đổi:
  Import "wasi_snapshot_preview1" thay vì "env"
  _start export (convention)
  Detect: nếu có WASI → dùng WASI, nếu không → dùng env
```

---

## Rào cản

```
1. WASM binary format phức tạp (LEB128 encoding, section layout)
   → Giải pháp: emit WAT text trước, binary emitter sau
   → LEB128 = ~20 LOC helper functions

2. WASM memory model khác native (linear memory, no mmap)
   → Đã giải quyết: vm_wasm.wat dùng memory.grow

3. Browser security (CORS, service worker)
   → Giải pháp: serve từ origin.olang HTTP server (Phase 6)
   → Dev mode: python3 -m http.server

4. ⚠️ [THỰC TẾ] WASM VM cũng cần ELF/header detection giống native VM
   → Trong browser: bytecode loaded qua JS, không đọc file → OK
   → Trong WASI: cần đọc file → cần origin header parsing logic
   → vm_wasm.wat chưa có origin header parser

5. ⚠️ [THỰC TẾ] Bytecode format: PHẢI dùng codegen format (PLAN_0_5)
   → vm_wasm.wat dispatch table phải match bytecode.rs opcodes (0x01-0x24)
   → KHÔNG dùng ir.rs format (0x00-0x83)
   → Builder flag: --codegen BẮT BUỘC khi build cho WASM
```

---

## Test Plan

```
Test 1: Pack bytecode "hello" → .wasm → wasmtime → output "Hello"
Test 2: Pack stdlib bytecode → .wasm → run sort.ol test
Test 3: origin.html in browser → REPL works
Test 4: WASI mode → wasmtime origin.olang.wasm → "Hello from WASM"
```

---

## Definition of Done

- [ ] Bytecode embedded trong WASM (static hoặc dynamic)
- [ ] Browser host HTML/JS chạy REPL cơ bản
- [ ] WASI support (wasmtime / wasmer)
- [ ] builder.ol: --arch wasm → tạo origin.olang.wasm
- [ ] Test: browser REPL hiển thị output

## Ước tính: 3-5 ngày
