;; ═══════════════════════════════════════════════════════════════════════════
;; origin.olang VM — WASI variant
;; Author: Lyra (session 2pN6F)
;;
;; WASI = WebAssembly System Interface
;; Runs on: wasmtime, wasmer, wasm3, Node.js (--experimental-wasi), Deno
;;
;; Differences from vm_wasm.wat:
;;   - Imports from "wasi_snapshot_preview1" instead of "env"
;;   - Uses fd_write/fd_read for I/O
;;   - Exports _start instead of init+run
;;   - Bytecode must be embedded in data section (no host_load_bytecode)
;;
;; Usage:
;;   wasmtime vm_wasi.wasm
;;   wasmer run vm_wasi.wasm
;; ═══════════════════════════════════════════════════════════════════════════

(module
  ;; === WASI IMPORTS ===
  ;; fd_write(fd, iovs_ptr, iovs_len, nwritten_ptr) -> errno
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32)))
  ;; fd_read(fd, iovs_ptr, iovs_len, nread_ptr) -> errno
  (import "wasi_snapshot_preview1" "fd_read"
    (func $fd_read (param i32 i32 i32 i32) (result i32)))
  ;; proc_exit(code)
  (import "wasi_snapshot_preview1" "proc_exit"
    (func $proc_exit (param i32)))

  ;; === MEMORY ===
  (memory (export "memory") 16)  ;; 16 pages = 1 MB

  ;; === DATA SEGMENTS ===
  ;; Error messages at 0x100
  (data (i32.const 0x100) "stack overflow\n")
  (data (i32.const 0x110) "stack underflow\n")
  (data (i32.const 0x120) "division by zero\n")
  (data (i32.const 0x130) "unknown opcode\n")
  (data (i32.const 0x140) "step limit\n")
  (data (i32.const 0x150) "HomeOS WASM VM (WASI)\n")

  ;; WASI iovec scratch at 0x180 (8 bytes: ptr + len)
  ;; nwritten scratch at 0x190 (4 bytes)

  ;; Bytecode placeholder at 0x10000
  ;; Builder embeds actual bytecode here via data section patching
  ;; For standalone testing, embed a small program:
  ;;   0x15 = PushNum, followed by 8 bytes f64(42.0)
  ;;   0x06 = Emit
  ;;   0x0F = Halt
  (data (i32.const 0x10000) "\15\00\00\00\00\00\00\45\40\06\0f")

  ;; === GLOBALS ===
  (global $pc        (mut i32) (i32.const 0))
  (global $sp        (mut i32) (i32.const 0x1000))
  (global $heap      (mut i32) (i32.const 0xC000))
  (global $bc_start  (mut i32) (i32.const 0x10000))
  (global $bc_size   (mut i32) (i32.const 11))  ;; default test program size
  ;; NOTE: Builder must patch $bc_size to match actual embedded bytecode length.
  ;; The data section at 0x10000 and this global must be updated together.
  (global $steps     (mut i32) (i32.const 0))
  (global $halted    (mut i32) (i32.const 0))
  (global $var_count (mut i32) (i32.const 0))

  (global $SP_BASE   i32 (i32.const 0x1000))
  (global $SP_MAX    i32 (i32.const 0x4000))
  (global $VAR_BASE  i32 (i32.const 0x8000))
  (global $STEP_MAX  i32 (i32.const 1000000))

  ;; Temp storage for pop
  (global $tmp_hash (mut i64) (i64.const 0))
  (global $tmp_ptr  (mut i32) (i32.const 0))
  (global $tmp_len  (mut i32) (i32.const 0))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; WASI I/O WRAPPERS
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $wasi_write (param $ptr i32) (param $len i32) (result i32)
    ;; Write to stdout (fd=1) using WASI fd_write
    ;; iovec at 0x180: {ptr, len}
    (i32.store (i32.const 0x180) (local.get $ptr))
    (i32.store (i32.const 0x184) (local.get $len))
    (drop (call $fd_write
      (i32.const 1)      ;; fd = stdout
      (i32.const 0x180)  ;; iovs
      (i32.const 1)      ;; iovs_len
      (i32.const 0x190)));; nwritten
    (local.get $len))

  (func $wasi_write_stderr (param $ptr i32) (param $len i32)
    (i32.store (i32.const 0x180) (local.get $ptr))
    (i32.store (i32.const 0x184) (local.get $len))
    (drop (call $fd_write
      (i32.const 2)      ;; fd = stderr
      (i32.const 0x180)
      (i32.const 1)
      (i32.const 0x190))))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; FNV-1a HASH (64-bit) — same as vm_wasm.wat
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $fnv1a (param $ptr i32) (param $len i32) (result i64)
    (local $hash i64)
    (local $end i32)
    (local.set $hash (i64.const -3750763034362895579))
    (local.set $end (i32.add (local.get $ptr) (local.get $len)))
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u (local.get $ptr) (local.get $end)))
        (local.set $hash
          (i64.xor (local.get $hash)
            (i64.extend_i32_u (i32.load8_u (local.get $ptr)))))
        (local.set $hash
          (i64.mul (local.get $hash) (i64.const 1099511628211)))
        (local.set $ptr (i32.add (local.get $ptr) (i32.const 1)))
        (br $lp)))
    (local.get $hash))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; STACK OPS
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $vm_push (param $hash i64) (param $ptr i32) (param $len i32)
    (if (i32.ge_u (global.get $sp) (global.get $SP_MAX))
      (then (call $err_overflow) (return)))
    (i64.store (global.get $sp) (local.get $hash))
    (i32.store (i32.add (global.get $sp) (i32.const 8)) (local.get $ptr))
    (i32.store (i32.add (global.get $sp) (i32.const 12)) (local.get $len))
    (global.set $sp (i32.add (global.get $sp) (i32.const 16))))

  (func $vm_pop
    (if (i32.le_u (global.get $sp) (global.get $SP_BASE))
      (then (call $err_underflow) (return)))
    (global.set $sp (i32.sub (global.get $sp) (i32.const 16)))
    (global.set $tmp_hash (i64.load (global.get $sp)))
    (global.set $tmp_ptr (i32.load (i32.add (global.get $sp) (i32.const 8))))
    (global.set $tmp_len (i32.load (i32.add (global.get $sp) (i32.const 12)))))

  (func $vm_peek
    (if (i32.le_u (global.get $sp) (global.get $SP_BASE))
      (then (call $err_underflow) (return)))
    (global.set $tmp_hash (i64.load (i32.sub (global.get $sp) (i32.const 16))))
    (global.set $tmp_ptr (i32.load (i32.sub (global.get $sp) (i32.const 8))))
    (global.set $tmp_len (i32.load (i32.sub (global.get $sp) (i32.const 4)))))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; HEAP + BYTECODE READERS
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $heap_alloc (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap))
    (global.set $heap (i32.add (global.get $heap) (local.get $size)))
    (local.get $ptr))

  (func $heap_copy_from_bc (param $bc_off i32) (param $len i32) (result i32)
    (local $dst i32)
    (local.set $dst (call $heap_alloc (local.get $len)))
    (memory.copy (local.get $dst)
      (i32.add (global.get $bc_start) (local.get $bc_off))
      (local.get $len))
    (local.get $dst))

  (func $read_u8 (result i32)
    (local $v i32)
    (local.set $v (i32.load8_u
      (i32.add (global.get $bc_start) (global.get $pc))))
    (global.set $pc (i32.add (global.get $pc) (i32.const 1)))
    (local.get $v))

  (func $read_u16 (result i32)
    (local $v i32)
    (local.set $v (i32.load16_u
      (i32.add (global.get $bc_start) (global.get $pc))))
    (global.set $pc (i32.add (global.get $pc) (i32.const 2)))
    (local.get $v))

  (func $read_u32 (result i32)
    (local $v i32)
    (local.set $v (i32.load
      (i32.add (global.get $bc_start) (global.get $pc))))
    (global.set $pc (i32.add (global.get $pc) (i32.const 4)))
    (local.get $v))

  (func $read_f64 (result f64)
    (local $v f64)
    (local.set $v (f64.load
      (i32.add (global.get $bc_start) (global.get $pc))))
    (global.set $pc (i32.add (global.get $pc) (i32.const 8)))
    (local.get $v))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; VARIABLE TABLE
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $var_store (param $name_hash i64) (param $val_ptr i32) (param $val_len i32)
    (local $i i32) (local $addr i32)
    (local.set $i (i32.const 0))
    (block $found
      (loop $search
        (br_if $found (i32.ge_u (local.get $i) (global.get $var_count)))
        (local.set $addr
          (i32.add (global.get $VAR_BASE)
            (i32.mul (local.get $i) (i32.const 16))))
        (if (i64.eq (i64.load (local.get $addr)) (local.get $name_hash))
          (then
            (i32.store (i32.add (local.get $addr) (i32.const 8)) (local.get $val_ptr))
            (i32.store (i32.add (local.get $addr) (i32.const 12)) (local.get $val_len))
            (return)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $search)))
    (local.set $addr
      (i32.add (global.get $VAR_BASE)
        (i32.mul (global.get $var_count) (i32.const 16))))
    (i64.store (local.get $addr) (local.get $name_hash))
    (i32.store (i32.add (local.get $addr) (i32.const 8)) (local.get $val_ptr))
    (i32.store (i32.add (local.get $addr) (i32.const 12)) (local.get $val_len))
    (global.set $var_count (i32.add (global.get $var_count) (i32.const 1))))

  (func $var_load (param $name_hash i64)
    (local $i i32) (local $addr i32)
    (local.set $i (i32.const 0))
    (block $not_found
      (loop $search
        (br_if $not_found (i32.ge_u (local.get $i) (global.get $var_count)))
        (local.set $addr
          (i32.add (global.get $VAR_BASE)
            (i32.mul (local.get $i) (i32.const 16))))
        (if (i64.eq (i64.load (local.get $addr)) (local.get $name_hash))
          (then
            (call $vm_push
              (local.get $name_hash)
              (i32.load (i32.add (local.get $addr) (i32.const 8)))
              (i32.load (i32.add (local.get $addr) (i32.const 12))))
            (return)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $search)))
    (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0)))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; F64 HELPERS
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $push_f64 (param $val f64)
    (local $ptr i32)
    (local.set $ptr (call $heap_alloc (i32.const 8)))
    (f64.store (local.get $ptr) (local.get $val))
    (call $vm_push
      (call $fnv1a (local.get $ptr) (i32.const 8))
      (local.get $ptr)
      (i32.const 8)))

  (func $pop_f64 (result f64)
    (call $vm_pop)
    (if (result f64) (i32.eq (global.get $tmp_len) (i32.const 8))
      (then (f64.load (global.get $tmp_ptr)))
      (else (f64.const 0))))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; OPCODE HANDLERS
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $op_push
    (local $len i32) (local $ptr i32)
    (local.set $len (call $read_u16))
    (local.set $ptr (call $heap_copy_from_bc (global.get $pc) (local.get $len)))
    (global.set $pc (i32.add (global.get $pc) (local.get $len)))
    (call $vm_push
      (call $fnv1a (local.get $ptr) (local.get $len))
      (local.get $ptr) (local.get $len)))

  (func $op_load
    (local $n i32) (local $p i32)
    (local.set $n (call $read_u8))
    (local.set $p (i32.add (global.get $bc_start) (global.get $pc)))
    (global.set $pc (i32.add (global.get $pc) (local.get $n)))
    (call $var_load (call $fnv1a (local.get $p) (local.get $n))))

  (func $op_emit
    (call $vm_pop)
    (if (i32.gt_u (global.get $tmp_len) (i32.const 0))
      (then (drop (call $wasi_write (global.get $tmp_ptr) (global.get $tmp_len))))))

  (func $op_call
    (local $n i32) (local $p i32) (local $h i64)
    (local.set $n (call $read_u8))
    (local.set $p (i32.add (global.get $bc_start) (global.get $pc)))
    (global.set $pc (i32.add (global.get $pc) (local.get $n)))
    (local.set $h (call $fnv1a (local.get $p) (local.get $n)))
    (call $builtin_dispatch (local.get $h)))

  (func $op_jmp (global.set $pc (call $read_u32)))

  (func $op_jz
    (local $target i32)
    (local.set $target (call $read_u32))
    (call $vm_pop)
    (if (i32.eqz (global.get $tmp_len))
      (then (global.set $pc (local.get $target)) (return)))
    (if (i64.eqz (global.get $tmp_hash))
      (then (global.set $pc (local.get $target)) (return)))
    (if (i32.eq (global.get $tmp_len) (i32.const 8))
      (then
        (if (f64.eq (f64.load (global.get $tmp_ptr)) (f64.const 0))
          (then (global.set $pc (local.get $target)))))))

  (func $op_dup
    (call $vm_peek)
    (call $vm_push (global.get $tmp_hash) (global.get $tmp_ptr) (global.get $tmp_len)))

  (func $op_pop (call $vm_pop))

  (func $op_swap
    (local $h1 i64) (local $p1 i32) (local $l1 i32)
    (local $h2 i64) (local $p2 i32) (local $l2 i32)
    (call $vm_pop)
    (local.set $h1 (global.get $tmp_hash))
    (local.set $p1 (global.get $tmp_ptr))
    (local.set $l1 (global.get $tmp_len))
    (call $vm_pop)
    (local.set $h2 (global.get $tmp_hash))
    (local.set $p2 (global.get $tmp_ptr))
    (local.set $l2 (global.get $tmp_len))
    (call $vm_push (local.get $h1) (local.get $p1) (local.get $l1))
    (call $vm_push (local.get $h2) (local.get $p2) (local.get $l2)))

  (func $op_store
    (local $n i32) (local $p i32)
    (local.set $n (call $read_u8))
    (local.set $p (i32.add (global.get $bc_start) (global.get $pc)))
    (global.set $pc (i32.add (global.get $pc) (local.get $n)))
    (call $vm_pop)
    (call $var_store
      (call $fnv1a (local.get $p) (local.get $n))
      (global.get $tmp_ptr) (global.get $tmp_len)))

  (func $op_push_num (call $push_f64 (call $read_f64)))

  (func $op_push_mol
    (local $ptr i32)
    (local.set $ptr (call $heap_copy_from_bc (global.get $pc) (i32.const 5)))
    (global.set $pc (i32.add (global.get $pc) (i32.const 5)))
    (call $vm_push
      (call $fnv1a (local.get $ptr) (i32.const 5))
      (local.get $ptr) (i32.const 5)))

  (func $op_lca
    (local $p1 i32) (local $l1 i32) (local $p2 i32) (local $l2 i32)
    (local $dst i32) (local $i i32)
    (call $vm_pop)
    (local.set $p1 (global.get $tmp_ptr))
    (local.set $l1 (global.get $tmp_len))
    (call $vm_pop)
    (local.set $p2 (global.get $tmp_ptr))
    (local.set $l2 (global.get $tmp_len))
    (if (i32.or (i32.lt_u (local.get $l1) (i32.const 5))
                (i32.lt_u (local.get $l2) (i32.const 5)))
      (then (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0)) (return)))
    (local.set $dst (call $heap_alloc (i32.const 5)))
    (local.set $i (i32.const 0))
    (block $done
      (loop $dim
        (br_if $done (i32.ge_u (local.get $i) (i32.const 5)))
        (i32.store8 (i32.add (local.get $dst) (local.get $i))
          (i32.div_u
            (i32.add
              (i32.load8_u (i32.add (local.get $p1) (local.get $i)))
              (i32.load8_u (i32.add (local.get $p2) (local.get $i))))
            (i32.const 2)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $dim)))
    (call $vm_push
      (call $fnv1a (local.get $dst) (i32.const 5))
      (local.get $dst) (i32.const 5)))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; BUILTIN DISPATCH
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $builtin_dispatch (param $hash i64)
    (local $a f64) (local $b f64)

    ;; __hyp_add
    (if (i64.eq (local.get $hash) (i64.const 6279461061396250740))
      (then
        (local.set $b (call $pop_f64)) (local.set $a (call $pop_f64))
        (call $push_f64 (f64.add (local.get $a) (local.get $b))) (return)))
    ;; __hyp_sub
    (if (i64.eq (local.get $hash) (i64.const -969687268777616107))
      (then
        (local.set $b (call $pop_f64)) (local.set $a (call $pop_f64))
        (call $push_f64 (f64.sub (local.get $a) (local.get $b))) (return)))
    ;; __hyp_mul
    (if (i64.eq (local.get $hash) (i64.const 8647125211697039873))
      (then
        (local.set $b (call $pop_f64)) (local.set $a (call $pop_f64))
        (call $push_f64 (f64.mul (local.get $a) (local.get $b))) (return)))
    ;; __hyp_div
    (if (i64.eq (local.get $hash) (i64.const 4478715002052434856))
      (then
        (local.set $b (call $pop_f64))
        (if (f64.eq (local.get $b) (f64.const 0))
          (then (call $err_div_zero) (return)))
        (local.set $a (call $pop_f64))
        (call $push_f64 (f64.div (local.get $a) (local.get $b))) (return)))
    ;; __eq
    (if (i64.eq (local.get $hash) (i64.const 6467812567259650137))
      (then
        (local.set $b (call $pop_f64)) (local.set $a (call $pop_f64))
        (if (f64.eq (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))
    ;; __cmp_lt
    (if (i64.eq (local.get $hash) (i64.const -1941734057267599498))
      (then
        (local.set $b (call $pop_f64)) (local.set $a (call $pop_f64))
        (if (f64.lt (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))
    ;; __cmp_gt
    (if (i64.eq (local.get $hash) (i64.const -1934958866615887891))
      (then
        (local.set $b (call $pop_f64)) (local.set $a (call $pop_f64))
        (if (f64.gt (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))
    ;; __cmp_le
    (if (i64.eq (local.get $hash) (i64.const -1941750549942022663))
      (then
        (local.set $b (call $pop_f64)) (local.set $a (call $pop_f64))
        (if (f64.le (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))
    ;; __cmp_ge
    (if (i64.eq (local.get $hash) (i64.const -1934977558313567478))
      (then
        (local.set $b (call $pop_f64)) (local.set $a (call $pop_f64))
        (if (f64.ge (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))
    ;; __len
    (if (i64.eq (local.get $hash) (i64.const 2530744773748778130))
      (then
        (call $vm_pop)
        (call $push_f64 (f64.convert_i32_u (global.get $tmp_len)))
        (return)))
    ;; __concat
    (if (i64.eq (local.get $hash) (i64.const -6548561775763658949))
      (then (call $builtin_concat) (return)))
    ;; __char_at
    (if (i64.eq (local.get $hash) (i64.const -6932815227842865687))
      (then (call $builtin_char_at) (return)))
    ;; __substr
    (if (i64.eq (local.get $hash) (i64.const -2051769843888937794))
      (then (call $builtin_substr) (return)))
    ;; __push
    (if (i64.eq (local.get $hash) (i64.const -6934319497420882281))
      (then (call $builtin_push) (return)))
    ;; __pop
    (if (i64.eq (local.get $hash) (i64.const 5090665129273995654))
      (then (call $builtin_pop) (return)))
    ;; __cmp_ne
    (if (i64.eq (local.get $hash) (i64.const -1943725272825911169))
      (then
        (local.set $b (call $pop_f64)) (local.set $a (call $pop_f64))
        (if (f64.ne (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))
  )

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; STRING BUILTINS
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $builtin_concat
    (local $p1 i32) (local $l1 i32) (local $p2 i32) (local $l2 i32)
    (local $dst i32) (local $total i32)
    (call $vm_pop)
    (local.set $p2 (global.get $tmp_ptr))
    (local.set $l2 (global.get $tmp_len))
    (call $vm_pop)
    (local.set $p1 (global.get $tmp_ptr))
    (local.set $l1 (global.get $tmp_len))
    (local.set $total (i32.add (local.get $l1) (local.get $l2)))
    (local.set $dst (call $heap_alloc (local.get $total)))
    (if (i32.gt_u (local.get $l1) (i32.const 0))
      (then (memory.copy (local.get $dst) (local.get $p1) (local.get $l1))))
    (if (i32.gt_u (local.get $l2) (i32.const 0))
      (then (memory.copy
        (i32.add (local.get $dst) (local.get $l1))
        (local.get $p2) (local.get $l2))))
    (call $vm_push
      (call $fnv1a (local.get $dst) (local.get $total))
      (local.get $dst) (local.get $total)))

  (func $builtin_char_at
    (local $idx i32) (local $ptr i32) (local $slen i32) (local $dst i32)
    (call $vm_pop)
    (local.set $idx (i32.trunc_f64_s (f64.load (global.get $tmp_ptr))))
    (call $vm_pop)
    (local.set $ptr (global.get $tmp_ptr))
    (local.set $slen (global.get $tmp_len))
    (if (i32.or (i32.lt_s (local.get $idx) (i32.const 0))
                (i32.ge_u (local.get $idx) (local.get $slen)))
      (then (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0)) (return)))
    (local.set $dst (call $heap_alloc (i32.const 1)))
    (i32.store8 (local.get $dst)
      (i32.load8_u (i32.add (local.get $ptr) (local.get $idx))))
    (call $vm_push
      (call $fnv1a (local.get $dst) (i32.const 1))
      (local.get $dst) (i32.const 1)))

  (func $builtin_substr
    (local $slen_req i32) (local $start i32)
    (local $ptr i32) (local $chain_len i32)
    (local $dst i32) (local $actual_len i32)
    (call $vm_pop)
    (local.set $slen_req (i32.trunc_f64_s (f64.load (global.get $tmp_ptr))))
    (call $vm_pop)
    (local.set $start (i32.trunc_f64_s (f64.load (global.get $tmp_ptr))))
    (call $vm_pop)
    (local.set $ptr (global.get $tmp_ptr))
    (local.set $chain_len (global.get $tmp_len))
    (if (i32.lt_s (local.get $start) (i32.const 0))
      (then (local.set $start (i32.const 0))))
    (if (i32.ge_u (local.get $start) (local.get $chain_len))
      (then (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0)) (return)))
    (local.set $actual_len (local.get $slen_req))
    (if (i32.gt_u
          (i32.add (local.get $start) (local.get $actual_len))
          (local.get $chain_len))
      (then (local.set $actual_len
        (i32.sub (local.get $chain_len) (local.get $start)))))
    (local.set $dst (call $heap_alloc (local.get $actual_len)))
    (if (i32.gt_u (local.get $actual_len) (i32.const 0))
      (then (memory.copy (local.get $dst)
        (i32.add (local.get $ptr) (local.get $start))
        (local.get $actual_len))))
    (call $vm_push
      (call $fnv1a (local.get $dst) (local.get $actual_len))
      (local.get $dst) (local.get $actual_len)))

  ;; __push: push value onto array (stub — arrays not yet in VM)
  (func $builtin_push
    ;; For now: just keep value on stack (nop)
  )

  ;; __pop: pop value from array (stub)
  (func $builtin_pop
    ;; For now: push empty
    (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))
  )

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; ERROR HANDLERS
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $err_overflow
    (call $wasi_write_stderr (i32.const 0x100) (i32.const 15))
    (global.set $halted (i32.const 1)))
  (func $err_underflow
    (call $wasi_write_stderr (i32.const 0x110) (i32.const 16))
    (global.set $halted (i32.const 1)))
  (func $err_div_zero
    (call $wasi_write_stderr (i32.const 0x120) (i32.const 17))
    (global.set $halted (i32.const 1)))
  (func $err_unknown_op
    (call $wasi_write_stderr (i32.const 0x130) (i32.const 15))
    (global.set $halted (i32.const 1)))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; VM LOOP
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $vm_run (result i32)
    (local $tag i32)
    (block $exit
      (loop $lp
        (br_if $exit (global.get $halted))
        (br_if $exit (i32.ge_u (global.get $steps) (global.get $STEP_MAX)))
        (br_if $exit (i32.ge_u (global.get $pc) (global.get $bc_size)))
        (global.set $steps (i32.add (global.get $steps) (i32.const 1)))
        (local.set $tag (call $read_u8))

        (if (i32.eq (local.get $tag) (i32.const 0x01))
          (then (call $op_push))
        (else (if (i32.eq (local.get $tag) (i32.const 0x02))
          (then (call $op_load))
        (else (if (i32.eq (local.get $tag) (i32.const 0x03))
          (then (call $op_lca))
        (else (if (i32.eq (local.get $tag) (i32.const 0x04))
          (then (drop (call $read_u8)) (call $vm_pop) (call $vm_pop)
                (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0)))
        (else (if (i32.eq (local.get $tag) (i32.const 0x05))
          (then (drop (call $read_u8)) (call $vm_pop)
                (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0)))
        (else (if (i32.eq (local.get $tag) (i32.const 0x06))
          (then (call $op_emit))
        (else (if (i32.eq (local.get $tag) (i32.const 0x07))
          (then (call $op_call))
        (else (if (i32.eq (local.get $tag) (i32.const 0x09))
          (then (call $op_jmp))
        (else (if (i32.eq (local.get $tag) (i32.const 0x0A))
          (then (call $op_jz))
        (else (if (i32.eq (local.get $tag) (i32.const 0x0B))
          (then (call $op_dup))
        (else (if (i32.eq (local.get $tag) (i32.const 0x0C))
          (then (call $op_pop))
        (else (if (i32.eq (local.get $tag) (i32.const 0x0D))
          (then (call $op_swap))
        (else (if (i32.eq (local.get $tag) (i32.const 0x0E))
          (then (drop (call $read_u32)))
        (else (if (i32.eq (local.get $tag) (i32.const 0x0F))
          (then (global.set $halted (i32.const 1)))
        (else (if (i32.eq (local.get $tag) (i32.const 0x13))
          (then (call $op_store))
        (else (if (i32.eq (local.get $tag) (i32.const 0x14))
          (then (call $op_load))
        (else (if (i32.eq (local.get $tag) (i32.const 0x15))
          (then (call $op_push_num))
        (else (if (i32.eq (local.get $tag) (i32.const 0x19))
          (then (call $op_push_mol))
        (else (if (i32.eq (local.get $tag) (i32.const 0x1A))
          (then (drop (call $read_u32)))
        (else (if (i32.eq (local.get $tag) (i32.const 0x1C))
          (then (call $op_store))
        (else
          (if (i32.gt_u (local.get $tag) (i32.const 0x1C))
            (then (call $err_unknown_op)))
        ))))))))))))))))))))))))))))))))))))))))
        (br $lp)))

    (if (result i32) (global.get $halted)
      (then (i32.const 0))
      (else (i32.const 1))))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; _start — WASI entry point
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $_start (export "_start")
    ;; Print banner
    (drop (call $wasi_write (i32.const 0x150) (i32.const 22)))

    ;; Reset state
    (global.set $pc (i32.const 0))
    (global.set $sp (global.get $SP_BASE))
    (global.set $heap (i32.const 0xC000))
    (global.set $steps (i32.const 0))
    (global.set $halted (i32.const 0))
    (global.set $var_count (i32.const 0))

    ;; Bytecode already in data section at bc_start (0x10000)
    ;; bc_size set by global initializer (patched by builder)

    ;; Run VM
    (drop (call $vm_run))

    ;; Exit
    (call $proc_exit (i32.const 0)))
)
