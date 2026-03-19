;; ═══════════════════════════════════════════════════════════════════════════
;; origin.olang VM — WebAssembly (WAT text format)
;; Author: Lyra (session 2pN6F)
;;
;; Bytecode format: codegen.ol tags (0x01-0x24)
;; Memory: linear, 16 pages (1 MB) initial
;; Stack entry: 16 bytes [hash:8 (i64)][ptr:4 (i32)][len:4 (i32)]
;; ═══════════════════════════════════════════════════════════════════════════

(module
  ;; === IMPORTS (host provides these) ===
  (import "env" "host_write"
    (func $host_write (param i32 i32) (result i32)))
  (import "env" "host_read"
    (func $host_read (param i32 i32) (result i32)))
  (import "env" "host_load_bytecode"
    (func $host_load_bytecode (param i32 i32) (result i32)))
  (import "env" "host_log"
    (func $host_log (param i32 i32)))
  (import "env" "host_emit_event"
    (func $host_emit_event (param i32 i32 i32)))

  ;; === MEMORY ===
  (memory (export "memory") 16)  ;; 16 pages = 1 MB initial

  ;; === DATA SEGMENTS (error messages) ===
  (data (i32.const 0x100) "stack overflow\n")
  (data (i32.const 0x110) "stack underflow\n")
  (data (i32.const 0x120) "division by zero\n")
  (data (i32.const 0x130) "unknown opcode\n")
  (data (i32.const 0x140) "step limit\n")

  ;; Builtin names for hash lookup
  (data (i32.const 0x200) "__hyp_add\00__hyp_sub\00__hyp_mul\00__hyp_div\00__eq\00__cmp_lt\00__cmp_gt\00__cmp_le\00__cmp_ge\00__len\00__concat\00__char_at\00__substr\00__cmp_ne\00")

  ;; === GLOBALS ===
  (global $pc        (mut i32) (i32.const 0))
  (global $sp        (mut i32) (i32.const 0x1000))  ;; VM stack base
  (global $heap      (mut i32) (i32.const 0xC000))  ;; heap bump ptr
  (global $bc_start  (mut i32) (i32.const 0))
  (global $bc_size   (mut i32) (i32.const 0))
  (global $steps     (mut i32) (i32.const 0))
  (global $halted    (mut i32) (i32.const 0))

  ;; Variable table: starts at 0x8000, max 256 entries
  ;; Each: [name_hash:8][val_ptr:4][val_len:4] = 16 bytes
  (global $var_count (mut i32) (i32.const 0))

  ;; Constants
  (global $SP_BASE   i32 (i32.const 0x1000))
  (global $SP_MAX    i32 (i32.const 0x4000))  ;; 12KB stack = 768 entries
  (global $VAR_BASE  i32 (i32.const 0x8000))
  (global $STEP_MAX  i32 (i32.const 1000000))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; INIT
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $init (export "init")
    ;; Load bytecode from host at offset 0x10000
    (global.set $bc_start (i32.const 0x10000))
    (global.set $bc_size
      (call $host_load_bytecode
        (i32.const 0x10000)
        (i32.const 0x80000)))  ;; max 512KB

    ;; Reset state
    (global.set $pc (i32.const 0))
    (global.set $sp (global.get $SP_BASE))
    (global.set $heap (i32.const 0xC000))
    (global.set $steps (i32.const 0))
    (global.set $halted (i32.const 0))
    (global.set $var_count (i32.const 0))
  )

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; FNV-1a HASH (64-bit)
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $fnv1a (param $ptr i32) (param $len i32) (result i64)
    (local $hash i64)
    (local $end i32)
    (local.set $hash (i64.const -3750763034362895579))  ;; 0xcbf29ce484222325
    (local.set $end (i32.add (local.get $ptr) (local.get $len)))
    (block $done
      (loop $lp
        (br_if $done (i32.ge_u (local.get $ptr) (local.get $end)))
        (local.set $hash
          (i64.xor (local.get $hash)
            (i64.extend_i32_u (i32.load8_u (local.get $ptr)))))
        (local.set $hash
          (i64.mul (local.get $hash) (i64.const 1099511628211)))  ;; 0x100000001b3
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

  (func $vm_pop_hash (result i64)
    (if (result i64) (i32.le_u (global.get $sp) (global.get $SP_BASE))
      (then (call $err_underflow) (i64.const 0))
      (else
        (global.set $sp (i32.sub (global.get $sp) (i32.const 16)))
        (i64.load (global.get $sp)))))

  (func $vm_pop_ptr (result i32)
    (i32.load (i32.add (global.get $sp) (i32.const 8))))

  (func $vm_pop_len (result i32)
    (i32.load (i32.add (global.get $sp) (i32.const 12))))

  ;; Pop all 3 fields, store in temp globals
  (global $tmp_hash (mut i64) (i64.const 0))
  (global $tmp_ptr  (mut i32) (i32.const 0))
  (global $tmp_len  (mut i32) (i32.const 0))

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
  ;; HEAP ALLOC + COPY
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $heap_alloc (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap))
    (global.set $heap (i32.add (global.get $heap) (local.get $size)))
    (local.get $ptr))

  (func $heap_copy_from_bc (param $bc_off i32) (param $len i32) (result i32)
    ;; Copy from bytecode to heap, return heap ptr
    (local $dst i32)
    (local $src i32)
    (local.set $dst (call $heap_alloc (local.get $len)))
    (local.set $src (i32.add (global.get $bc_start) (local.get $bc_off)))
    (memory.copy (local.get $dst) (local.get $src) (local.get $len))
    (local.get $dst))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; BYTECODE READERS
  ;; ═══════════════════════════════════════════════════════════════════════

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
  ;; VARIABLE TABLE (hash-based lookup)
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $var_store (param $name_hash i64) (param $val_ptr i32) (param $val_len i32)
    (local $i i32)
    (local $addr i32)
    ;; Search for existing
    (local.set $i (i32.const 0))
    (block $found
      (loop $search
        (br_if $found (i32.ge_u (local.get $i) (global.get $var_count)))
        (local.set $addr
          (i32.add (global.get $VAR_BASE)
            (i32.mul (local.get $i) (i32.const 16))))
        (if (i64.eq (i64.load (local.get $addr)) (local.get $name_hash))
          (then
            ;; Update existing
            (i32.store (i32.add (local.get $addr) (i32.const 8)) (local.get $val_ptr))
            (i32.store (i32.add (local.get $addr) (i32.const 12)) (local.get $val_len))
            (return)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $search)))
    ;; New entry
    (local.set $addr
      (i32.add (global.get $VAR_BASE)
        (i32.mul (global.get $var_count) (i32.const 16))))
    (i64.store (local.get $addr) (local.get $name_hash))
    (i32.store (i32.add (local.get $addr) (i32.const 8)) (local.get $val_ptr))
    (i32.store (i32.add (local.get $addr) (i32.const 12)) (local.get $val_len))
    (global.set $var_count (i32.add (global.get $var_count) (i32.const 1))))

  (func $var_load (param $name_hash i64)
    ;; Push found value or empty
    (local $i i32)
    (local $addr i32)
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
    ;; Not found → push empty
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

  ;; 0x01: Push [chain_len:2 u16][chain_bytes:N]
  (func $op_push
    (local $len i32)
    (local $ptr i32)
    (local.set $len (call $read_u16))
    (local.set $ptr (call $heap_copy_from_bc (global.get $pc) (local.get $len)))
    (global.set $pc (i32.add (global.get $pc) (local.get $len)))
    (call $vm_push
      (call $fnv1a (local.get $ptr) (local.get $len))
      (local.get $ptr)
      (local.get $len)))

  ;; 0x02: Load [name_len:1][name:N]
  (func $op_load
    (local $name_len i32)
    (local $name_ptr i32)
    (local.set $name_len (call $read_u8))
    (local.set $name_ptr (i32.add (global.get $bc_start) (global.get $pc)))
    (global.set $pc (i32.add (global.get $pc) (local.get $name_len)))
    (call $var_load (call $fnv1a (local.get $name_ptr) (local.get $name_len))))

  ;; 0x06: Emit
  (func $op_emit
    (call $vm_pop)
    (if (i32.gt_u (global.get $tmp_len) (i32.const 0))
      (then
        (drop (call $host_write (global.get $tmp_ptr) (global.get $tmp_len))))))

  ;; 0x07: Call [name_len:1][name:N]
  (func $op_call
    (local $name_len i32)
    (local $name_ptr i32)
    (local $name_hash i64)
    (local.set $name_len (call $read_u8))
    (local.set $name_ptr (i32.add (global.get $bc_start) (global.get $pc)))
    (global.set $pc (i32.add (global.get $pc) (local.get $name_len)))
    (local.set $name_hash (call $fnv1a (local.get $name_ptr) (local.get $name_len)))
    (call $builtin_dispatch (local.get $name_hash)))

  ;; 0x09: Jmp [target:4]
  (func $op_jmp
    (global.set $pc (call $read_u32)))

  ;; 0x0A: Jz [target:4]
  (func $op_jz
    (local $target i32)
    (local.set $target (call $read_u32))
    (call $vm_pop)
    ;; Empty = hash==0 or len==0 → take jump
    (if (i32.or
          (i64.eqz (global.get $tmp_hash))
          (i32.eqz (global.get $tmp_len)))
      (then (global.set $pc (local.get $target)))))

  ;; 0x0B: Dup
  (func $op_dup
    (call $vm_peek)
    (call $vm_push (global.get $tmp_hash) (global.get $tmp_ptr) (global.get $tmp_len)))

  ;; 0x0C: Pop
  (func $op_pop
    (call $vm_pop))

  ;; 0x0D: Swap
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

  ;; 0x0E: Loop [count:4]
  (func $op_loop
    (drop (call $read_u32)))  ;; stub: skip count

  ;; 0x13: Store [name_len:1][name:N]
  (func $op_store
    (local $name_len i32)
    (local $name_ptr i32)
    (local.set $name_len (call $read_u8))
    (local.set $name_ptr (i32.add (global.get $bc_start) (global.get $pc)))
    (global.set $pc (i32.add (global.get $pc) (local.get $name_len)))
    (call $vm_pop)
    (call $var_store
      (call $fnv1a (local.get $name_ptr) (local.get $name_len))
      (global.get $tmp_ptr)
      (global.get $tmp_len)))

  ;; 0x14: LoadLocal — same as Load
  (func $op_load_local
    (call $op_load))

  ;; 0x15: PushNum [f64:8]
  (func $op_push_num
    (call $push_f64 (call $read_f64)))

  ;; 0x19: PushMol [S:1][R:1][V:1][A:1][T:1]
  (func $op_push_mol
    (local $ptr i32)
    (local.set $ptr (call $heap_copy_from_bc (global.get $pc) (i32.const 5)))
    (global.set $pc (i32.add (global.get $pc) (i32.const 5)))
    (call $vm_push
      (call $fnv1a (local.get $ptr) (i32.const 5))
      (local.get $ptr)
      (i32.const 5)))

  ;; 0x1C: StoreUpdate — same as Store
  (func $op_store_update
    (call $op_store))

  ;; 0x03: LCA
  (func $op_lca
    (local $p1 i32) (local $l1 i32)
    (local $p2 i32) (local $l2 i32)
    (local $dst i32)
    (local $i i32)
    (call $vm_pop)
    (local.set $p1 (global.get $tmp_ptr))
    (local.set $l1 (global.get $tmp_len))
    (call $vm_pop)
    (local.set $p2 (global.get $tmp_ptr))
    (local.set $l2 (global.get $tmp_len))
    ;; Need >= 5 bytes each
    (if (i32.or (i32.lt_u (local.get $l1) (i32.const 5))
                (i32.lt_u (local.get $l2) (i32.const 5)))
      (then
        (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))
        (return)))
    ;; Average each of 5 dimensions
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
      (local.get $dst)
      (i32.const 5)))

  ;; 0x04: Edge [rel:1]
  (func $op_edge
    (drop (call $read_u8))
    (call $vm_pop) (call $vm_pop)  ;; pop 2, push empty
    (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0)))

  ;; 0x05: Query [rel:1]
  (func $op_query
    (drop (call $read_u8))
    (call $vm_pop)  ;; pop 1, push empty
    (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0)))

  ;; 0x1A: TryBegin [target:4]
  (func $op_try_begin
    (drop (call $read_u32)))  ;; stub

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; BUILTIN DISPATCH (by name hash)
  ;; ═══════════════════════════════════════════════════════════════════════

  ;; Pre-computed FNV-1a hashes for builtin names
  ;; (computed at compile time, must match fnv1a runtime)

  (func $builtin_dispatch (param $hash i64)
    (local $a f64) (local $b f64)

    ;; __hyp_add
    (if (i64.eq (local.get $hash) (i64.const -4394791828366498724))
      (then
        (local.set $b (call $pop_f64))
        (local.set $a (call $pop_f64))
        (call $push_f64 (f64.add (local.get $a) (local.get $b)))
        (return)))

    ;; __hyp_sub
    (if (i64.eq (local.get $hash) (i64.const -4394791828332907669))
      (then
        (local.set $b (call $pop_f64))
        (local.set $a (call $pop_f64))
        (call $push_f64 (f64.sub (local.get $a) (local.get $b)))
        (return)))

    ;; __hyp_mul
    (if (i64.eq (local.get $hash) (i64.const -4394791828349721234))
      (then
        (local.set $b (call $pop_f64))
        (local.set $a (call $pop_f64))
        (call $push_f64 (f64.mul (local.get $a) (local.get $b)))
        (return)))

    ;; __hyp_div
    (if (i64.eq (local.get $hash) (i64.const -4394791828377378555))
      (then
        (local.set $b (call $pop_f64))
        (if (f64.eq (local.get $b) (f64.const 0))
          (then (call $err_div_zero) (return)))
        (local.set $a (call $pop_f64))
        (call $push_f64 (f64.div (local.get $a) (local.get $b)))
        (return)))

    ;; __eq
    (if (i64.eq (local.get $hash) (i64.const 6293835889444872024))
      (then
        (call $vm_pop)
        (local.set $b (f64.load (global.get $tmp_ptr)))
        (call $vm_pop)
        (local.set $a (f64.load (global.get $tmp_ptr)))
        (if (f64.eq (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))

    ;; __cmp_lt
    (if (i64.eq (local.get $hash) (i64.const -3807292011824697569))
      (then
        (local.set $b (call $pop_f64))
        (local.set $a (call $pop_f64))
        (if (f64.lt (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))

    ;; __cmp_gt
    (if (i64.eq (local.get $hash) (i64.const -3807292011824734525))
      (then
        (local.set $b (call $pop_f64))
        (local.set $a (call $pop_f64))
        (if (f64.gt (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))

    ;; __cmp_le
    (if (i64.eq (local.get $hash) (i64.const -3807292011824700656))
      (then
        (local.set $b (call $pop_f64))
        (local.set $a (call $pop_f64))
        (if (f64.le (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))

    ;; __cmp_ge
    (if (i64.eq (local.get $hash) (i64.const -3807292011824737612))
      (then
        (local.set $b (call $pop_f64))
        (local.set $a (call $pop_f64))
        (if (f64.ge (local.get $a) (local.get $b))
          (then (call $push_f64 (f64.const 1)))
          (else (call $vm_push (i64.const 0) (i32.const 0) (i32.const 0))))
        (return)))

    ;; __len
    (if (i64.eq (local.get $hash) (i64.const 6578919763498122553))
      (then
        (call $vm_pop)
        (call $push_f64 (f64.convert_i32_u (global.get $tmp_len)))
        (return)))

    ;; Unknown builtin — ignore
  )

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; ERROR HANDLERS
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $err_overflow
    (drop (call $host_write (i32.const 0x100) (i32.const 15)))
    (global.set $halted (i32.const 1)))

  (func $err_underflow
    (drop (call $host_write (i32.const 0x110) (i32.const 16)))
    (global.set $halted (i32.const 1)))

  (func $err_div_zero
    (drop (call $host_write (i32.const 0x120) (i32.const 17)))
    (global.set $halted (i32.const 1)))

  (func $err_unknown_op
    (drop (call $host_write (i32.const 0x130) (i32.const 15)))
    (global.set $halted (i32.const 1)))

  ;; ═══════════════════════════════════════════════════════════════════════
  ;; VM LOOP (main dispatch)
  ;; ═══════════════════════════════════════════════════════════════════════

  (func $run (export "run") (result i32)
    (local $tag i32)

    (block $exit
      (loop $lp
        ;; Check halt
        (br_if $exit (global.get $halted))
        ;; Check step limit
        (br_if $exit (i32.ge_u (global.get $steps) (global.get $STEP_MAX)))
        ;; Check bounds
        (br_if $exit (i32.ge_u (global.get $pc) (global.get $bc_size)))

        ;; Step++
        (global.set $steps (i32.add (global.get $steps) (i32.const 1)))

        ;; Fetch tag
        (local.set $tag (call $read_u8))

        ;; Dispatch via if-chain (simpler, guaranteed correct)
        (if (i32.eq (local.get $tag) (i32.const 0x01))
          (then (call $op_push))
        (else (if (i32.eq (local.get $tag) (i32.const 0x02))
          (then (call $op_load))
        (else (if (i32.eq (local.get $tag) (i32.const 0x03))
          (then (call $op_lca))
        (else (if (i32.eq (local.get $tag) (i32.const 0x04))
          (then (call $op_edge))
        (else (if (i32.eq (local.get $tag) (i32.const 0x05))
          (then (call $op_query))
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
          (then (call $op_loop))
        (else (if (i32.eq (local.get $tag) (i32.const 0x0F))
          (then (global.set $halted (i32.const 1)))
        (else (if (i32.eq (local.get $tag) (i32.const 0x13))
          (then (call $op_store))
        (else (if (i32.eq (local.get $tag) (i32.const 0x14))
          (then (call $op_load_local))
        (else (if (i32.eq (local.get $tag) (i32.const 0x15))
          (then (call $op_push_num))
        (else (if (i32.eq (local.get $tag) (i32.const 0x19))
          (then (call $op_push_mol))
        (else (if (i32.eq (local.get $tag) (i32.const 0x1A))
          (then (call $op_try_begin))
        (else (if (i32.eq (local.get $tag) (i32.const 0x1C))
          (then (call $op_store_update))
        (else
          ;; 0x00,0x08,0x10-0x12,0x16-0x18,0x1B = nop/stub
          ;; unknown = error (if > 0x1C)
          (if (i32.gt_u (local.get $tag) (i32.const 0x1C))
            (then (call $err_unknown_op)))
        ))))))))))))))))))))))))))))))))))))))))
        (br $lp)
      ) ;; end loop $lp
    ) ;; end block $exit

    ;; Return: 0=normal halt, 1=step limit, 2=error
    (if (result i32) (global.get $halted)
      (then (i32.const 0))
      (else (i32.const 1))))
)
