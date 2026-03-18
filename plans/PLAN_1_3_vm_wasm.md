# PLAN 1.3 — vm_wasm.wat: VM cho WebAssembly (~1500 LOC WAT)

**Phụ thuộc:** PLAN_1_1 phải xong (vm_x86_64.S hoạt động, bytecode format ổn định)
**Mục tiêu:** origin.olang chạy trong browser, Node.js, Cloudflare Workers — mọi nơi có WASM runtime.
**Yêu cầu:** Hiểu WebAssembly spec (MVP + bulk-memory), WAT text format, JS interop.

---

## Bối cảnh

### Tại sao WASM?

```
WASM = "write once, run anywhere" thực sự:
  - Browser (Chrome, Firefox, Safari, Edge)
  - Node.js / Deno / Bun (server-side)
  - Cloudflare Workers / Fastly Compute (edge)
  - Wasmer / Wasmtime (standalone runtime)
  - iOS WKWebView / Android WebView

HomeOS trong browser = không cần cài đặt.
Mở tab → HomeOS chạy ngay → đóng tab → dữ liệu local.
```

### Khác biệt chính so với native ASM

```
                    Native (x86/ARM)         WASM
────────────────────────────────────────────────────
Memory              mmap/brk (OS)            linear memory (grow)
Syscalls            direct (int/svc)         import từ host (JS)
File I/O            read/write/open          KHÔNG CÓ → host bridge
Entry point         _start                   (func (export "main"))
Registers           hardware (16-31)         virtual stack + locals
Float               SSE/NEON                 f64.add/f64.mul native
Strings             raw bytes                raw bytes (linear mem)
Self-reference      /proc/self/exe           KHÔNG → host load data
Crypto              AES-NI/ARMv8-CE          KHÔNG → software only
Threading           clone/pthread            KHÔNG (MVP) → SharedMem later
```

### Mối quan hệ với crates/wasm/ hiện tại

```
HIỆN TẠI (crates/wasm/):
  Rust code → wasm-bindgen → HomeOSWasm class
  = TOÀN BỘ Rust runtime compile sang WASM
  = ~2-5 MB WASM binary
  = CẦN Rust toolchain để build

SAU PLAN 1.3 (vm_wasm.wat):
  WAT text → wat2wasm → vm.wasm (~40-80 KB)
  + bytecode section (compiled Olang)
  + knowledge section
  = origin.olang.wasm (~200 KB total)
  = KHÔNG CẦN Rust

Lộ trình chuyển đổi:
  Phase 1: vm_wasm.wat chạy song song với crates/wasm/
  Phase 2: dần thay thế → crates/wasm/ deprecated
  Phase 3: crates/wasm/ xóa hoàn toàn

Bridge (crates/wasm/src/bridge.rs) = GIỮ LẠI logic:
  - BridgeMsg protocol (0x01-0xFF)
  - WebSocket ↔ ISL mapping
  - EventStream concept
  → Rewrite bằng Olang hoặc WAT helper functions
```

---

## Kiến trúc WASM VM

### Memory Layout (Linear Memory)

```
WASM linear memory = 1 flat byte array, grows in 64KB pages.

Offset       Size        Purpose
──────────────────────────────────────────
0x0000       32 B        Header cache (parsed from host)
0x0020       4 KB        VM stack (256 entries × 16 bytes)
0x1020       4 KB        Scope/locals area (256 scopes × 16 entries)
0x2020       4 KB        Loop stack (64 frames × 16 bytes)
0x3020       4 KB        Try stack (64 entries × 8 bytes)
0x4020       4 KB        Call stack (256 return addresses × 16 bytes)
0x5020       4 KB        Output buffer (emit results)
0x6020       4 KB        String buffer (temp conversions)
0x7020       4 KB        Event buffer (VmEvents for host)
0x8020       16 KB       Builtin name table (hash → index)
0xC020       ...         Heap arena (bump allocator, grows up)

Bytecode + Knowledge = host loads vào memory riêng hoặc
  copy vào linear memory offset configurable.

Mỗi VM stack entry = 16 bytes:
  [chain_hash: 8B (i64)][chain_ptr: 4B (i32)][chain_len: 4B (i32)]
  (WASM dùng i32 cho pointers — 4GB address space)
```

### Host Imports (thay syscalls)

```wat
;; Host cung cấp qua JavaScript/runtime:

;; I/O
(import "env" "host_write"
  (func $host_write (param i32 i32) (result i32)))
  ;; (buf_ptr, buf_len) → bytes_written
  ;; Dùng cho: Emit opcode, debug output

(import "env" "host_read"
  (func $host_read (param i32 i32) (result i32)))
  ;; (buf_ptr, buf_max) → bytes_read
  ;; Dùng cho: REPL input

;; Data loading (thay /proc/self/exe)
(import "env" "host_load_bytecode"
  (func $host_load_bytecode (param i32 i32) (result i32)))
  ;; (dest_ptr, max_len) → actual_len
  ;; Host copy bytecode section vào linear memory

(import "env" "host_load_knowledge"
  (func $host_load_knowledge (param i32 i32) (result i32)))
  ;; (dest_ptr, max_len) → actual_len

;; Persistence (thay file I/O)
(import "env" "host_persist"
  (func $host_persist (param i32 i32) (result i32)))
  ;; (data_ptr, data_len) → 0=ok, -1=error
  ;; Host saves to IndexedDB / localStorage / HTTP POST

(import "env" "host_load_persist"
  (func $host_load_persist (param i32 i32) (result i32)))
  ;; (dest_ptr, max_len) → actual_len
  ;; Host loads from IndexedDB / localStorage

;; Time
(import "env" "host_time_ns"
  (func $host_time_ns (result i64)))
  ;; Date.now() × 1_000_000 (ms → ns)

;; Events (output to host)
(import "env" "host_emit_event"
  (func $host_emit_event (param i32 i32 i32)))
  ;; (event_type, data_ptr, data_len)
  ;; Host receives: emotion updates, dream results, silk updates
  ;; Maps to BridgeMsg protocol (bridge.rs)

;; ISL bridge
(import "env" "host_isl_send"
  (func $host_isl_send (param i32 i32) (result i32)))
  ;; (frame_ptr, frame_len) → 0=ok
  ;; Send ISLFrame to host → WebSocket → other agents

(import "env" "host_isl_recv"
  (func $host_isl_recv (param i32 i32) (result i32)))
  ;; (dest_ptr, max_len) → actual_len (0 = no message)
  ;; Non-blocking receive from host ISL queue

;; Debug
(import "env" "host_log"
  (func $host_log (param i32 i32)))
  ;; (msg_ptr, msg_len) → console.log in browser
```

---

## Việc cần làm

### Task 1: Module Skeleton + Memory Init (~100 LOC)

```wat
(module
  ;; === IMPORTS ===
  (import "env" "host_write" (func $host_write (param i32 i32) (result i32)))
  (import "env" "host_read" (func $host_read (param i32 i32) (result i32)))
  (import "env" "host_load_bytecode" (func $host_load_bytecode (param i32 i32) (result i32)))
  (import "env" "host_load_knowledge" (func $host_load_knowledge (param i32 i32) (result i32)))
  (import "env" "host_persist" (func $host_persist (param i32 i32) (result i32)))
  (import "env" "host_time_ns" (func $host_time_ns (result i64)))
  (import "env" "host_emit_event" (func $host_emit_event (param i32 i32 i32)))
  (import "env" "host_isl_send" (func $host_isl_send (param i32 i32) (result i32)))
  (import "env" "host_isl_recv" (func $host_isl_recv (param i32 i32) (result i32)))
  (import "env" "host_log" (func $host_log (param i32 i32)))

  ;; === MEMORY ===
  (memory (export "memory") 16)  ;; 16 pages = 1 MB initial
  ;; Grows as needed via memory.grow

  ;; === GLOBALS (VM state) ===
  (global $pc       (mut i32) (i32.const 0))     ;; program counter (offset into bytecode)
  (global $sp_vm    (mut i32) (i32.const 0x0020)) ;; VM stack pointer
  (global $heap_ptr (mut i32) (i32.const 0xC020)) ;; heap bump pointer
  (global $steps    (mut i32) (i32.const 0))      ;; step counter
  (global $scope_depth (mut i32) (i32.const 0))   ;; current scope depth
  (global $loop_top (mut i32) (i32.const 0))      ;; loop stack top pointer
  (global $try_top  (mut i32) (i32.const 0))      ;; try stack top pointer
  (global $call_top (mut i32) (i32.const 0))      ;; call stack top pointer
  (global $bc_start (mut i32) (i32.const 0))      ;; bytecode section start in memory
  (global $bc_end   (mut i32) (i32.const 0))      ;; bytecode section end
  (global $output_ptr (mut i32) (i32.const 0x5020)) ;; output buffer cursor
  (global $halted   (mut i32) (i32.const 0))      ;; halt flag

  ;; Constants
  (global $STACK_BASE i32 (i32.const 0x0020))
  (global $STACK_MAX  i32 (i32.const 256))        ;; max entries
  (global $STEPS_MAX  i32 (i32.const 65536))      ;; QT2: ∞-1
  (global $LOOP_MAX   i32 (i32.const 1024))       ;; max iterations

  ;; === INIT ===
  (func $init (export "init")
    ;; Load bytecode from host into linear memory
    (global.set $bc_start (i32.const 0x10000))  ;; 64KB offset
    (call $host_load_bytecode
      (global.get $bc_start)
      (i32.const 0x80000))  ;; max 512KB bytecode
    (global.get $bc_start)
    (i32.add)
    (global.set $bc_end)

    ;; Reset VM state
    (global.set $pc (i32.const 0))
    (global.set $sp_vm (global.get $STACK_BASE))
    (global.set $steps (i32.const 0))
    (global.set $halted (i32.const 0))
    (global.set $scope_depth (i32.const 0))
    (global.set $loop_top (i32.const 0x2020))
    (global.set $try_top (i32.const 0x3020))
    (global.set $call_top (i32.const 0x4020))
    (global.set $heap_ptr (i32.const 0xC020))
    (global.set $output_ptr (i32.const 0x5020))
  )
)
```

### Task 2: VM Loop + Dispatch (~200 LOC)

```wat
  ;; === VM LOOP ===
  (func $vm_run (export "run") (result i32)
    ;; Returns: 0=ok, 1=error, 2=halted normally
    (local $tag i32)
    (local $abs_pc i32)

    (block $exit
      (loop $loop
        ;; Check halt flag
        (br_if $exit (global.get $halted))

        ;; Check step limit (QT2)
        (br_if $exit
          (i32.ge_u (global.get $steps) (global.get $STEPS_MAX)))

        ;; Check bounds: pc < bc_size
        (br_if $exit
          (i32.ge_u (global.get $pc)
            (i32.sub (global.get $bc_end) (global.get $bc_start))))

        ;; Increment step counter
        (global.set $steps
          (i32.add (global.get $steps) (i32.const 1)))

        ;; Fetch opcode: absolute address = bc_start + pc
        (local.set $abs_pc
          (i32.add (global.get $bc_start) (global.get $pc)))
        (local.set $tag
          (i32.load8_u (local.get $abs_pc)))

        ;; Advance PC past tag
        (global.set $pc
          (i32.add (global.get $pc) (i32.const 1)))

        ;; Dispatch via br_table (WASM's jump table)
        (block $op_unknown
        (block $op_file_append  ;; 0x26
        (block $op_file_write   ;; 0x25
        (block $op_file_read    ;; 0x24
        (block $op_ffi          ;; 0x23
        (block $op_explain      ;; 0x22
        (block $op_why          ;; 0x21
        (block $op_typeof       ;; 0x20
        (block $op_assert       ;; 0x1F
        (block $op_inspect      ;; 0x1E
        (block $op_trace        ;; 0x1D
        (block $op_store_update ;; 0x1C
        (block $op_catch_end    ;; 0x1B
        (block $op_try_begin    ;; 0x1A
        (block $op_push_mol     ;; 0x19
        (block $op_scope_end    ;; 0x18
        (block $op_scope_begin  ;; 0x17
        (block $op_fuse         ;; 0x16
        (block $op_push_num     ;; 0x15
        (block $op_load_local   ;; 0x14
        (block $op_store        ;; 0x13
        (block $op_nop          ;; 0x12
        (block $op_stats        ;; 0x11
        (block $op_dream        ;; 0x10
        (block $op_halt         ;; 0x0F
        (block $op_loop         ;; 0x0E
        (block $op_swap         ;; 0x0D
        (block $op_pop          ;; 0x0C
        (block $op_dup          ;; 0x0B
        (block $op_jz           ;; 0x0A
        (block $op_jmp          ;; 0x09
        (block $op_ret          ;; 0x08
        (block $op_call         ;; 0x07
        (block $op_emit         ;; 0x06
        (block $op_query        ;; 0x05
        (block $op_edge         ;; 0x04
        (block $op_lca          ;; 0x03
        (block $op_load         ;; 0x02
        (block $op_push         ;; 0x01
        (block $op_reserved     ;; 0x00
          (br_table
            $op_reserved     ;; 0x00
            $op_push         ;; 0x01
            $op_load         ;; 0x02
            $op_lca          ;; 0x03
            $op_edge         ;; 0x04
            $op_query        ;; 0x05
            $op_emit         ;; 0x06
            $op_call         ;; 0x07
            $op_ret          ;; 0x08
            $op_jmp          ;; 0x09
            $op_jz           ;; 0x0A
            $op_dup          ;; 0x0B
            $op_pop          ;; 0x0C
            $op_swap         ;; 0x0D
            $op_loop         ;; 0x0E
            $op_halt         ;; 0x0F
            $op_dream        ;; 0x10
            $op_stats        ;; 0x11
            $op_nop          ;; 0x12
            $op_store        ;; 0x13
            $op_load_local   ;; 0x14
            $op_push_num     ;; 0x15
            $op_fuse         ;; 0x16
            $op_scope_begin  ;; 0x17
            $op_scope_end    ;; 0x18
            $op_push_mol     ;; 0x19
            $op_try_begin    ;; 0x1A
            $op_catch_end    ;; 0x1B
            $op_store_update ;; 0x1C
            $op_trace        ;; 0x1D
            $op_inspect      ;; 0x1E
            $op_assert       ;; 0x1F
            $op_typeof       ;; 0x20
            $op_why          ;; 0x21
            $op_explain      ;; 0x22
            $op_ffi          ;; 0x23
            $op_file_read    ;; 0x24
            $op_file_write   ;; 0x25
            $op_file_append  ;; 0x26
            $op_unknown      ;; default
            (local.get $tag)
          )
        ) ;; $op_reserved
        (br $loop)  ;; nop for reserved

        ) ;; $op_push
        (call $handle_push)
        (br $loop)

        ) ;; $op_load
        (call $handle_load)
        (br $loop)

        ) ;; $op_lca
        (call $handle_lca)
        (br $loop)

        ;; ... (mỗi opcode tương tự) ...

        ) ;; $op_halt
        (global.set $halted (i32.const 1))
        (br $loop)

        ) ;; ... remaining opcodes ...

        ) ;; $op_unknown
        (call $handle_error_unknown_opcode)
        (global.set $halted (i32.const 1))

        ;; End of all blocks — fall through to loop or exit
        (br $loop)
      ) ;; loop
    ) ;; block $exit

    ;; Return status
    (if (result i32) (global.get $halted)
      (then (i32.const 0))    ;; normal halt
      (else (i32.const 2))    ;; step limit exceeded
    )
  )
```

### Task 3: Stack Operations (~150 LOC)

```wat
  ;; Stack entry = 16 bytes: [hash:8 (i64)][ptr:4 (i32)][len:4 (i32)]
  ;; sp_vm points to next free slot (grows up)

  (func $stack_depth (result i32)
    (i32.div_u
      (i32.sub (global.get $sp_vm) (global.get $STACK_BASE))
      (i32.const 16))
  )

  (func $stack_check_overflow (result i32)
    ;; Returns 1 if overflow (depth >= 256)
    (i32.ge_u (call $stack_depth) (global.get $STACK_MAX))
  )

  (func $stack_check_underflow (result i32)
    ;; Returns 1 if empty
    (i32.le_u (global.get $sp_vm) (global.get $STACK_BASE))
  )

  (func $vm_push (param $hash i64) (param $ptr i32) (param $len i32)
    (if (call $stack_check_overflow)
      (then (call $handle_error_overflow) (return)))
    ;; Store entry
    (i64.store (global.get $sp_vm) (local.get $hash))
    (i32.store (i32.add (global.get $sp_vm) (i32.const 8)) (local.get $ptr))
    (i32.store (i32.add (global.get $sp_vm) (i32.const 12)) (local.get $len))
    ;; Advance sp
    (global.set $sp_vm (i32.add (global.get $sp_vm) (i32.const 16)))
  )

  (func $vm_pop (result i64 i32 i32)
    ;; Returns (hash, ptr, len)
    (if (call $stack_check_underflow)
      (then
        (call $handle_error_underflow)
        (return (i64.const 0) (i32.const 0) (i32.const 0))))
    ;; Decrement sp
    (global.set $sp_vm (i32.sub (global.get $sp_vm) (i32.const 16)))
    ;; Load entry
    (i64.load (global.get $sp_vm))
    (i32.load (i32.add (global.get $sp_vm) (i32.const 8)))
    (i32.load (i32.add (global.get $sp_vm) (i32.const 12)))
  )

  (func $vm_peek (result i64 i32 i32)
    ;; Returns top without popping
    (if (call $stack_check_underflow)
      (then
        (call $handle_error_underflow)
        (return (i64.const 0) (i32.const 0) (i32.const 0))))
    (i64.load (i32.sub (global.get $sp_vm) (i32.const 16)))
    (i32.load (i32.sub (global.get $sp_vm) (i32.const 8)))
    (i32.load (i32.sub (global.get $sp_vm) (i32.const 4)))
  )

  (func $handle_push
    (local $chain_len i32)
    (local $abs_pc i32)
    (local $heap_start i32)
    (local $hash i64)

    ;; Read chain_len (2 bytes LE)
    (local.set $abs_pc
      (i32.add (global.get $bc_start) (global.get $pc)))
    (local.set $chain_len
      (i32.load16_u (local.get $abs_pc)))
    (global.set $pc
      (i32.add (global.get $pc) (i32.const 2)))

    ;; Bump allocate on heap
    (local.set $heap_start (global.get $heap_ptr))
    (global.set $heap_ptr
      (i32.add (global.get $heap_ptr) (local.get $chain_len)))

    ;; Copy chain bytes from bytecode to heap
    ;; (memory.copy is bulk-memory proposal, widely supported)
    (local.set $abs_pc
      (i32.add (global.get $bc_start) (global.get $pc)))
    (memory.copy
      (local.get $heap_start)     ;; dest
      (local.get $abs_pc)         ;; src
      (local.get $chain_len))     ;; len

    ;; Advance pc past chain
    (global.set $pc
      (i32.add (global.get $pc) (local.get $chain_len)))

    ;; Hash chain bytes
    (local.set $hash
      (call $fnv1a (local.get $heap_start) (local.get $chain_len)))

    ;; Push onto VM stack
    (call $vm_push
      (local.get $hash)
      (local.get $heap_start)
      (local.get $chain_len))
  )

  (func $handle_dup
    (local $hash i64)
    (local $ptr i32)
    (local $len i32)
    (call $vm_peek)
    (local.set $len)
    (local.set $ptr)
    (local.set $hash)
    (call $vm_push (local.get $hash) (local.get $ptr) (local.get $len))
  )

  (func $handle_pop
    (call $vm_pop)
    (drop) (drop) (drop)  ;; discard all 3 values
  )

  (func $handle_swap
    (local $h1 i64) (local $p1 i32) (local $l1 i32)
    (local $h2 i64) (local $p2 i32) (local $l2 i32)
    ;; Pop top 2
    (call $vm_pop)
    (local.set $l1) (local.set $p1) (local.set $h1)
    (call $vm_pop)
    (local.set $l2) (local.set $p2) (local.set $h2)
    ;; Push back in reverse order
    (call $vm_push (local.get $h1) (local.get $p1) (local.get $l1))
    (call $vm_push (local.get $h2) (local.get $p2) (local.get $l2))
  )
```

### Task 4: Control Flow (~150 LOC)

```wat
  (func $handle_jmp
    (local $abs_pc i32)
    ;; Read target offset (4 bytes LE)
    (local.set $abs_pc
      (i32.add (global.get $bc_start) (global.get $pc)))
    (global.set $pc
      (i32.load (local.get $abs_pc)))  ;; pc = target (byte offset from bc_start)
  )

  (func $handle_jz
    (local $abs_pc i32)
    (local $hash i64)
    (local $ptr i32)
    (local $len i32)
    (local $target i32)

    ;; Read target
    (local.set $abs_pc
      (i32.add (global.get $bc_start) (global.get $pc)))
    (local.set $target
      (i32.load (local.get $abs_pc)))

    ;; Pop top
    (call $vm_pop)
    (local.set $len) (local.set $ptr) (local.set $hash)

    ;; Check if empty (hash == 0 or len == 0)
    (if (i32.or
          (i64.eqz (local.get $hash))
          (i32.eqz (local.get $len)))
      (then
        ;; Empty → take jump
        (global.set $pc (local.get $target)))
      (else
        ;; Not empty → skip target bytes
        (global.set $pc
          (i32.add (global.get $pc) (i32.const 4)))))
  )

  (func $handle_loop
    (local $abs_pc i32)
    (local $count i32)
    ;; Read count (4 bytes LE)
    (local.set $abs_pc
      (i32.add (global.get $bc_start) (global.get $pc)))
    (local.set $count
      (i32.load (local.get $abs_pc)))
    (global.set $pc
      (i32.add (global.get $pc) (i32.const 4)))

    ;; Cap at LOOP_MAX (QT2)
    (if (i32.gt_u (local.get $count) (global.get $LOOP_MAX))
      (then (local.set $count (global.get $LOOP_MAX))))

    ;; Push loop frame onto loop stack
    ;; [return_pc:4][remaining:4] = 8 bytes
    (i32.store (global.get $loop_top) (global.get $pc))  ;; return PC
    (i32.store
      (i32.add (global.get $loop_top) (i32.const 4))
      (local.get $count))
    (global.set $loop_top
      (i32.add (global.get $loop_top) (i32.const 8)))
  )

  (func $handle_call
    (local $abs_pc i32)
    (local $name_len i32)
    (local $name_ptr i32)
    (local $builtin_idx i32)

    ;; Read name_len (1 byte)
    (local.set $abs_pc
      (i32.add (global.get $bc_start) (global.get $pc)))
    (local.set $name_len
      (i32.load8_u (local.get $abs_pc)))
    (global.set $pc
      (i32.add (global.get $pc) (i32.const 1)))

    ;; Name pointer (absolute in memory)
    (local.set $name_ptr
      (i32.add (global.get $bc_start) (global.get $pc)))

    ;; Advance PC past name
    (global.set $pc
      (i32.add (global.get $pc) (local.get $name_len)))

    ;; Lookup builtin
    (local.set $builtin_idx
      (call $builtin_lookup (local.get $name_ptr) (local.get $name_len)))

    ;; Dispatch
    (if (i32.ge_s (local.get $builtin_idx) (i32.const 0))
      (then
        ;; Known builtin → dispatch
        (call $builtin_dispatch (local.get $builtin_idx)))
      (else
        ;; Unknown → emit LookupAlias event to host
        (call $host_emit_event
          (i32.const 1)  ;; event type: LookupAlias
          (local.get $name_ptr)
          (local.get $name_len))))
  )

  (func $handle_ret
    ;; Pop call frame, restore PC
    (if (i32.gt_u (global.get $call_top) (i32.const 0x4020))
      (then
        (global.set $call_top
          (i32.sub (global.get $call_top) (i32.const 4)))
        (global.set $pc
          (i32.load (global.get $call_top))))
      (else
        ;; No call frame → halt
        (global.set $halted (i32.const 1))))
  )
```

### Task 5: FNV-1a Hash (~40 LOC)

```wat
  (func $fnv1a (param $ptr i32) (param $len i32) (result i64)
    (local $hash i64)
    (local $end i32)
    (local $byte i64)

    ;; FNV-1a offset basis
    (local.set $hash (i64.const 0xcbf29ce484222325))
    (local.set $end (i32.add (local.get $ptr) (local.get $len)))

    (block $done
      (loop $byte_loop
        ;; Check if done
        (br_if $done
          (i32.ge_u (local.get $ptr) (local.get $end)))

        ;; hash ^= byte
        (local.set $byte
          (i64.extend_i32_u (i32.load8_u (local.get $ptr))))
        (local.set $hash
          (i64.xor (local.get $hash) (local.get $byte)))

        ;; hash *= FNV prime (0x100000001b3)
        (local.set $hash
          (i64.mul (local.get $hash) (i64.const 0x100000001b3)))

        ;; ptr++
        (local.set $ptr
          (i32.add (local.get $ptr) (i32.const 1)))
        (br $byte_loop)
      )
    )
    (local.get $hash)
  )
```

### Task 6: Math Builtins (~150 LOC)

```wat
  ;; WASM f64 operations = NATIVE — không cần Taylor series!
  ;; f64.add, f64.sub, f64.mul, f64.div, f64.sqrt, f64.abs,
  ;; f64.ceil, f64.floor, f64.nearest, f64.min, f64.max
  ;; → Tất cả là WASM instructions, NHANH VÀ CHÍNH XÁC

  (func $math_add
    ;; Pop 2 numbers, decode, add, encode, push
    (local $a f64) (local $b f64)
    (local $h1 i64) (local $p1 i32) (local $l1 i32)
    (local $h2 i64) (local $p2 i32) (local $l2 i32)

    (call $vm_pop)
    (local.set $l1) (local.set $p1) (local.set $h1)
    (call $vm_pop)
    (local.set $l2) (local.set $p2) (local.set $h2)

    ;; Decode chains to f64
    (local.set $b (call $chain_to_f64 (local.get $p1) (local.get $l1)))
    (local.set $a (call $chain_to_f64 (local.get $p2) (local.get $l2)))

    ;; Add (WASM native instruction!)
    (call $push_f64 (f64.add (local.get $a) (local.get $b)))
  )

  (func $math_sub
    ;; Same pattern, f64.sub
    ;; ... (analogous to math_add) ...
  )

  (func $math_mul
    ;; f64.mul
  )

  (func $math_div
    (local $a f64) (local $b f64)
    ;; ... pop & decode ...
    ;; Division by zero check
    (if (f64.eq (local.get $b) (f64.const 0))
      (then (call $handle_error_div_zero) (return)))
    (call $push_f64 (f64.div (local.get $a) (local.get $b)))
  )

  (func $math_sqrt
    ;; f64.sqrt — WASM native!
  )

  ;; Sin/Cos — WASM KHÔNG CÓ native instruction
  ;; → Taylor series hoặc import từ host (Math.sin)
  ;; Option A: import
  ;;   (import "env" "math_sin" (func $host_sin (param f64) (result f64)))
  ;; Option B: Taylor (tự chứa, không phụ thuộc host)

  (func $math_sin (param $x f64) (result f64)
    ;; Taylor: sin(x) = x - x³/3! + x⁵/5! - x⁷/7! + x⁹/9! - x¹¹/11!
    ;; Range reduction trước: x mod 2π → [-π, π]
    (local $x2 f64) (local $term f64) (local $sum f64)
    ;; ... ~20 instructions ...
    (local.get $sum)
  )

  (func $math_cos (param $x f64) (result f64)
    ;; cos(x) = 1 - x²/2! + x⁴/4! - x⁶/6! + x⁸/8!
    ;; ... ~20 instructions ...
    (local.get $sum)
  )

  ;; Helper: encode f64 → chain on heap, push to VM stack
  (func $push_f64 (param $val f64)
    (local $ptr i32)
    (local $hash i64)
    ;; Allocate 8 bytes on heap for f64 encoding
    ;; (simplified — full impl encodes as 4-molecule chain)
    (local.set $ptr (global.get $heap_ptr))
    (f64.store (local.get $ptr) (local.get $val))
    (global.set $heap_ptr
      (i32.add (global.get $heap_ptr) (i32.const 8)))
    ;; Hash
    (local.set $hash
      (call $fnv1a (local.get $ptr) (i32.const 8)))
    ;; Push
    (call $vm_push (local.get $hash) (local.get $ptr) (i32.const 8))
  )

  ;; Helper: decode chain → f64
  (func $chain_to_f64 (param $ptr i32) (param $len i32) (result f64)
    ;; Read 4 molecules, extract valence bytes, reconstruct f64
    ;; (simplified — read raw f64 from heap)
    (f64.load (local.get $ptr))
  )
```

### Task 7: Emit + I/O Bridge (~100 LOC)

```wat
  (func $handle_emit
    (local $hash i64) (local $ptr i32) (local $len i32)
    ;; Pop chain
    (call $vm_pop)
    (local.set $len) (local.set $ptr) (local.set $hash)

    ;; Write chain bytes to host
    (drop (call $host_write (local.get $ptr) (local.get $len)))

    ;; Also emit event for host tracking
    (call $host_emit_event
      (i32.const 0)           ;; event type: Output
      (local.get $ptr)
      (local.get $len))
  )

  (func $handle_dream
    ;; Emit TriggerDream event → host handles Dream cycle
    (call $host_emit_event
      (i32.const 7)           ;; event type: Dream (maps to BridgeMsg 0x12)
      (i32.const 0)
      (i32.const 0))
  )

  (func $handle_stats
    ;; Collect stats, write to output buffer
    ;; stack_depth, steps, heap_used, scope_depth
    ;; Format as simple text or binary
    (call $host_emit_event
      (i32.const 8)           ;; event type: Stats
      (i32.const 0)
      (i32.const 0))
  )

  (func $handle_file_read
    ;; Pop path chain → emit FileReadRequest event to host
    (local $hash i64) (local $ptr i32) (local $len i32)
    (call $vm_pop)
    (local.set $len) (local.set $ptr) (local.set $hash)
    (call $host_emit_event
      (i32.const 10)          ;; event type: FileRead
      (local.get $ptr)
      (local.get $len))
  )

  (func $handle_file_write
    ;; Pop data + path → emit FileWriteRequest (append-only, QT9)
    ;; ... pop 2 chains ...
    (call $host_emit_event
      (i32.const 11)          ;; event type: FileWrite
      (i32.const 0)           ;; host reads from shared memory
      (i32.const 0))
  )

  (func $handle_file_append
    ;; Similar to file_write
  )
```

### Task 8: Builtin Dispatch (~200 LOC)

```wat
  ;; Builtin lookup by name hash
  ;; Pre-compute FNV-1a of known builtin names → sorted table in .rodata area

  (func $builtin_lookup (param $name_ptr i32) (param $name_len i32) (result i32)
    ;; Returns builtin index (0-63) or -1 if not found
    (local $hash i64)
    (local.set $hash (call $fnv1a (local.get $name_ptr) (local.get $name_len)))

    ;; Linear scan of builtin table (small N=60, linear OK)
    ;; Table stored at BUILTIN_TABLE offset in memory
    ;; Format: [hash:8][index:4][padding:4] × N entries
    ;; ... search loop ...

    (i32.const -1)  ;; not found
  )

  (func $builtin_dispatch (param $idx i32)
    ;; Switch on builtin index
    ;; Group: math (0-15), comparison (16-23), string (24-39),
    ;;        array (40-55), dict (56-63), etc.

    (block $unknown
    (block $b63
    ;; ... (nested blocks for br_table) ...
    (block $b0
      (br_table $b0 $b1 $b2 ;; ... $b63 $unknown
        (local.get $idx))
    ) ;; $b0 = __hyp_add
    (call $math_add) (return)
    ) ;; $b1 = __hyp_sub
    (call $math_sub) (return)
    ;; ... etc for all builtins ...
    ) ;; $unknown
  )
```

### Task 9: SHA-256 Software (~150 LOC)

```wat
  ;; SHA-256 (software only — WASM has no crypto instructions)
  ;; Used for chain integrity verification

  ;; K constants stored in data segment
  (data (i32.const 0x8020)
    "\98\2f\8a\42\91\44\37\71"  ;; K[0..1] (big-endian)
    ;; ... 62 more 32-bit constants ...
  )

  (func $sha256_block (param $state_ptr i32) (param $data_ptr i32)
    ;; Standard SHA-256 compression: 64 rounds
    ;; Working variables: a-h in locals
    (local $a i32) (local $b i32) (local $c i32) (local $d i32)
    (local $e i32) (local $f i32) (local $g i32) (local $h_var i32)
    (local $t1 i32) (local $t2 i32)
    (local $i i32) (local $w_ptr i32)

    ;; Load initial state
    (local.set $a (i32.load (local.get $state_ptr)))
    ;; ... load b-h ...

    ;; 64 rounds
    ;; ... ~100 instructions ...
  )
```

### Task 10: Error Handlers (~50 LOC)

```wat
  ;; Error messages stored in data segment
  (data (i32.const 0x100) "stack overflow\n")    ;; 15 bytes
  (data (i32.const 0x110) "stack underflow\n")   ;; 16 bytes
  (data (i32.const 0x120) "division by zero\n")  ;; 17 bytes
  (data (i32.const 0x130) "unknown opcode\n")    ;; 15 bytes
  (data (i32.const 0x140) "step limit exceeded\n") ;; 20 bytes

  (func $handle_error_overflow
    (drop (call $host_write (i32.const 0x100) (i32.const 15)))
    (global.set $halted (i32.const 1))
  )

  (func $handle_error_underflow
    (drop (call $host_write (i32.const 0x110) (i32.const 16)))
    (global.set $halted (i32.const 1))
  )

  (func $handle_error_div_zero
    (drop (call $host_write (i32.const 0x120) (i32.const 17)))
    (global.set $halted (i32.const 1))
  )

  (func $handle_error_unknown_opcode
    (drop (call $host_write (i32.const 0x130) (i32.const 15)))
    (global.set $halted (i32.const 1))
  )
```

---

## JavaScript Host Implementation

### Minimal Host (~100 LOC JS)

```javascript
// host.js — JavaScript side of the WASM bridge

class HomeOSHost {
  constructor() {
    this.events = [];
    this.bytecode = null;
    this.knowledge = null;
    this.persistence = {};  // key → Uint8Array
  }

  async load(wasmUrl, bytecodeUrl) {
    // Load bytecode
    const bcResp = await fetch(bytecodeUrl);
    this.bytecode = new Uint8Array(await bcResp.arrayBuffer());

    // Load WASM with imports
    const importObject = {
      env: {
        host_write: (ptr, len) => {
          const bytes = new Uint8Array(this.memory.buffer, ptr, len);
          const text = new TextDecoder().decode(bytes);
          this.onOutput?.(text);
          return len;
        },

        host_read: (ptr, maxLen) => {
          // Non-blocking: return 0 if no input pending
          if (!this.pendingInput) return 0;
          const bytes = new TextEncoder().encode(this.pendingInput);
          const copyLen = Math.min(bytes.length, maxLen);
          new Uint8Array(this.memory.buffer, ptr, copyLen).set(bytes.slice(0, copyLen));
          this.pendingInput = null;
          return copyLen;
        },

        host_load_bytecode: (ptr, maxLen) => {
          if (!this.bytecode) return 0;
          const copyLen = Math.min(this.bytecode.length, maxLen);
          new Uint8Array(this.memory.buffer, ptr, copyLen).set(this.bytecode.slice(0, copyLen));
          return copyLen;
        },

        host_load_knowledge: (ptr, maxLen) => {
          if (!this.knowledge) return 0;
          const copyLen = Math.min(this.knowledge.length, maxLen);
          new Uint8Array(this.memory.buffer, ptr, copyLen).set(this.knowledge.slice(0, copyLen));
          return copyLen;
        },

        host_persist: (ptr, len) => {
          const data = new Uint8Array(this.memory.buffer, ptr, len).slice();
          // IndexedDB or localStorage
          try {
            localStorage.setItem('homeos_data', btoa(String.fromCharCode(...data)));
            return 0;
          } catch { return -1; }
        },

        host_time_ns: () => {
          return BigInt(Date.now()) * 1_000_000n;
        },

        host_emit_event: (type, ptr, len) => {
          const data = len > 0
            ? new Uint8Array(this.memory.buffer, ptr, len).slice()
            : null;
          this.events.push({ type, data, ts: Date.now() });
          this.onEvent?.({ type, data });
        },

        host_isl_send: (ptr, len) => {
          const frame = new Uint8Array(this.memory.buffer, ptr, len).slice();
          this.onISLSend?.(frame);
          return 0;
        },

        host_isl_recv: (ptr, maxLen) => {
          if (!this.islQueue?.length) return 0;
          const frame = this.islQueue.shift();
          const copyLen = Math.min(frame.length, maxLen);
          new Uint8Array(this.memory.buffer, ptr, copyLen).set(frame.slice(0, copyLen));
          return copyLen;
        },

        host_log: (ptr, len) => {
          const msg = new TextDecoder().decode(
            new Uint8Array(this.memory.buffer, ptr, len));
          console.log('[HomeOS]', msg);
        },
      },
    };

    const { instance } = await WebAssembly.instantiateStreaming(
      fetch(wasmUrl), importObject);

    this.instance = instance;
    this.memory = instance.exports.memory;

    // Init VM
    instance.exports.init();
  }

  run() {
    return this.instance.exports.run();
  }

  sendInput(text) {
    this.pendingInput = text;
  }

  drainEvents() {
    const events = this.events;
    this.events = [];
    return events;
  }
}

// Usage:
// const host = new HomeOSHost();
// host.onOutput = (text) => document.getElementById('output').textContent += text;
// await host.load('vm.wasm', 'bytecode.bin');
// host.run();
```

### WebSocket Bridge (~50 LOC JS)

```javascript
// bridge.js — WebSocket ↔ ISL bridge (browser side)

class HomeOSBridge {
  constructor(host, wsUrl) {
    this.host = host;
    this.ws = new WebSocket(wsUrl);

    // ISL frames from WASM → WebSocket
    host.onISLSend = (frame) => {
      if (this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(frame);
      }
    };

    // WebSocket → ISL queue for WASM
    this.ws.onmessage = (event) => {
      const frame = new Uint8Array(event.data);
      if (!host.islQueue) host.islQueue = [];
      host.islQueue.push(frame);
    };
  }
}
```

---

## Mapping: BridgeMsg (bridge.rs) → WASM Events

```
Existing bridge.rs protocol → reuse in WASM host:

BridgeMsg Type    Code    WASM event_type    Direction
─────────────────────────────────────────────────────────
TextInput         0x01    host_read           JS → WASM
OlangInput        0x02    host_read           JS → WASM
Response          0x10    host_emit_event(0)  WASM → JS
EmotionUpdate     0x11    host_emit_event(2)  WASM → JS
DreamResult       0x12    host_emit_event(7)  WASM → JS
SilkUpdate        0x13    host_emit_event(3)  WASM → JS
SceneUpdate       0x14    host_emit_event(4)  WASM → JS
Stats             0x20    host_emit_event(8)  WASM → JS
Ping/Pong         0xFE/FF (JS handles directly)

Event type mapping:
  0 = Output (Emit opcode)
  1 = LookupAlias (Load opcode, unknown name)
  2 = EmotionUpdate
  3 = SilkUpdate
  4 = SceneUpdate
  5 = CreateEdge
  6 = QueryRelation
  7 = TriggerDream
  8 = RequestStats
  9 = TraceStep
  10 = FileReadRequest
  11 = FileWriteRequest
```

---

## Lợi thế WASM so với native ASM

```
1. f64 operations = NATIVE WASM instructions
   → f64.add, f64.sub, f64.mul, f64.div, f64.sqrt, f64.abs
   → f64.ceil, f64.floor, f64.nearest, f64.min, f64.max
   → KHÔNG CẦN Taylor series cho basic math (trừ sin/cos)
   → Chính xác IEEE 754 guaranteed

2. Memory safety = WASM sandbox
   → Bounds checking automatic
   → Không buffer overflow
   → Không arbitrary code execution

3. br_table = native jump table
   → Opcode dispatch = 1 instruction
   → Tương đương jump table trong x86/ARM

4. memory.copy = bulk memory operation
   → Copy chain bytes nhanh (WASM bulk-memory proposal)
   → Không cần byte-by-byte loop

5. Portable
   → Cùng .wasm file chạy mọi nơi
   → Không cần compile per-platform
```

---

## Hạn chế WASM và giải pháp

```
Hạn chế                           Giải pháp
─────────────────────────────────────────────────────────────────
Không có sin/cos instruction       Taylor series (~20 instructions)
                                   HOẶC import host Math.sin/Math.cos
Không có crypto instruction        SHA-256 software (~150 LOC WAT)
                                   (chậm hơn 10-50× so với hardware)
Không có file I/O                  host_persist → IndexedDB/localStorage
                                   host_load_persist → load on boot
Không có threads (MVP)             Cooperative scheduling qua host
                                   Web Workers cho parallel (Phase sau)
Không có /proc/self/exe            host_load_bytecode → host cung cấp
32-bit pointers (i32)              4 GB address space — đủ cho HomeOS
                                   (knowledge section rarely > 1 GB)
Startup cost (compile .wasm)       Streaming compilation (instantiateStreaming)
                                   Caching compiled module (IDBCache)
GC pressure (large linear memory)  Fixed-size arena, no GC needed
                                   Bump allocator, reset per-turn
```

---

## Rào cản

| Rào cản | Giải pháp |
|---------|-----------|
| WAT verbose (text format) | Viết WAT, compile wat2wasm. Hoặc dùng tool generate |
| Sin/Cos không native | Taylor 11 terms (double precision), hoặc import Math.sin |
| No threads | Single-threaded OK cho Phase 1. Web Workers cho Phase sau |
| IndexedDB async | Host bridge handles async, WASM sees sync API |
| Large bytecode copy | memory.copy (bulk-memory) — widely supported since 2020 |
| 60+ builtins = nhiều WAT code | Code generator tool: generate WAT từ builtin spec |
| Testing across browsers | CI: headless Chrome + Firefox + Node.js test runner |
| Persistence across sessions | IndexedDB for large data, localStorage for small config |

---

## Test Plan

### Test 1: Module loads

```javascript
const host = new HomeOSHost();
await host.load('vm.wasm', 'empty_bytecode.bin');
// No crash = pass
```

### Test 2: Hello World

```
Bytecode: Push("hello") → Emit → Halt
JS: host.onOutput = (text) => assert(text === "hello")
host.run();
```

### Test 3: Math

```
Bytecode: PushNum(3) → PushNum(5) → Call("__hyp_add") → Emit → Halt
Expected: onOutput receives "8"
```

### Test 4: FNV-1a consistency

```
Cùng input → WASM fnv1a() == Rust fnv1a() == x86_64 fnv1a() == ARM64 fnv1a()
Test vectors:
  fnv1a("") = 0xcbf29ce484222325
  fnv1a("hello") = expected_hash
  fnv1a([0x01, 0x01, 0x80, 0x80, 0x03]) = expected_mol_hash
```

### Test 5: Stack overflow

```
Bytecode: 257× Push → should error
Expected: onOutput receives "stack overflow"
```

### Test 6: Event bridge

```
Bytecode: PushMol(1,6,200,180,4) → Emit → Dream → Halt
Expected events:
  { type: 0, data: mol_bytes }  // Output
  { type: 7, data: null }       // TriggerDream
```

### Test 7: Persistence round-trip

```javascript
// Run 1: learn something
host.sendInput("learn test data");
host.run();
// Persist
const data = host.drainPendingWrites();
localStorage.setItem('homeos', data);

// Run 2: load persisted
const host2 = new HomeOSHost();
host2.knowledge = localStorage.getItem('homeos');
await host2.load('vm.wasm', 'bytecode.bin');
// Verify knowledge restored
```

### Cross-validation

```
Cùng bytecode chạy trên:
  1. vm_x86_64 (native Linux)
  2. vm_arm64 (QEMU hoặc real ARM)
  3. vm_wasm.wat (Node.js)
  4. crates/wasm/ (Rust WASM — hiện tại)

Tất cả phải cho cùng output + cùng hash values.
```

---

## So sánh: crates/wasm/ (hiện tại) vs vm_wasm.wat (mục tiêu)

```
                    crates/wasm/              vm_wasm.wat
─────────────────────────────────────────────────────────────
Source language      Rust + wasm_bindgen       WAT (hand-written)
Build tool           wasm-pack + cargo         wat2wasm (1 command)
Binary size          ~2-5 MB                   ~40-80 KB
Dependencies         Rust toolchain            wabt only (hoặc 0)
Runtime init         ~200ms (Rust init)        ~5ms (minimal init)
Full HomeOS logic    ✅ (compiled from Rust)    Bytecode (Olang)
Bridge protocol      bridge.rs (Rust)          host.js (JavaScript)
Self-contained       ❌ (needs wasm-pack)       ✅ (wat2wasm → done)
Auditability         ~1000 LOC Rust            ~1500 LOC WAT
```

---

## Definition of Done

- [ ] `vm_wasm.wat` tồn tại (~1500 LOC WAT)
- [ ] Module structure: imports, memory, globals, data segments
- [ ] VM loop: fetch-decode-dispatch 38 opcodes via br_table
- [ ] Stack operations: push/pop/dup/swap (16-byte entries)
- [ ] Control flow: jmp/jz/loop/call/ret
- [ ] Math: f64 native ops + Taylor sin/cos
- [ ] FNV-1a hash: consistent with Rust/x86/ARM implementations
- [ ] Emit: output to host via host_write
- [ ] Event bridge: host_emit_event maps to BridgeMsg protocol
- [ ] SHA-256 software implementation
- [ ] Builtin dispatch: 60+ functions via hash lookup + br_table
- [ ] Error handling: overflow/underflow/div-zero/unknown-op
- [ ] `host.js` (~100 LOC): JavaScript host implementation
- [ ] `bridge.js` (~50 LOC): WebSocket ↔ ISL bridge
- [ ] Compiles: `wat2wasm vm_wasm.wat -o vm.wasm`
- [ ] Runs in Node.js: "Hello from WASM VM"
- [ ] Runs in Chrome: same output
- [ ] Cross-validation: cùng bytecode → cùng output với x86_64/ARM64
- [ ] Persistence: IndexedDB round-trip works

## Ước tính: 2-3 ngày (sau khi vm_x86_64.S ổn định)

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 1.3*
*Phụ thuộc: PLAN_1_1 (vm_x86_64.S), PLAN_0_5 (bytecode format)*
*Liên quan: crates/wasm/src/bridge.rs (BridgeMsg protocol giữ lại)*
