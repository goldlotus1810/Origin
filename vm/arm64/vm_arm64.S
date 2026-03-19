// ═══════════════════════════════════════════════════════════════════════════
// origin.olang VM — ARM64 Linux (no libc)
// Author: Lyra (session 2pN6F)
//
// Register conventions:
//   x19 = program counter (absolute pointer into bytecode)
//   x20 = VM stack pointer (grows up, 16 bytes per entry)
//   x21 = VM stack base
//   x22 = heap base
//   x23 = heap bump pointer
//   x24 = bytecode section start
//   x25 = bytecode section end (bc_start + bc_size)
//   x26 = variable table base
//   x27 = variable count
//   x28 = file descriptor (during init)
//
// VM stack entry: [ptr:8][len:8] = 16 bytes
// For f64: ptr stores raw f64 bits reinterpreted as u64, len = 0xFFFFFFFFFFFFFFFF
//
// Bytecode: codegen.ol format (tags 0x01-0x24)
// ═══════════════════════════════════════════════════════════════════════════

.text
.global _start

// ── Constants ──
.equ STACK_SIZE,   0x100000     // 1 MB VM stack
.equ HEAP_SIZE,    0x1000000    // 16 MB heap
.equ HEADER_SIZE,  32
.equ F64_MARKER,   -1

// ── Linux ARM64 syscall numbers ──
.equ SYS_READ,     63
.equ SYS_WRITE,    64
.equ SYS_OPENAT,   56
.equ SYS_CLOSE,    57
.equ SYS_LSEEK,    62
.equ SYS_MMAP,     222
.equ SYS_EXIT,     93
.equ SYS_FSTAT,    80

.equ AT_FDCWD,     -100
.equ O_RDONLY,     0

// ═══════════════════════════════════════════════════════════════════════════
// Entry point
// ═══════════════════════════════════════════════════════════════════════════

_start:
    // Save sp for stack frame
    mov     x29, sp

    // ── 1. Open file (argv[1] or /proc/self/exe) ──
    ldr     x0, [sp]                // argc
    cmp     x0, #2
    b.lt    .open_self
    ldr     x1, [sp, #16]           // argv[1]
    b       .do_open
.open_self:
    adr     x1, proc_self_exe
.do_open:
    mov     x0, #AT_FDCWD
    mov     x2, #O_RDONLY
    mov     x3, #0
    mov     x8, #SYS_OPENAT
    svc     #0
    cmp     x0, #0
    b.lt    err_open
    mov     x28, x0                 // fd

    // ── 2. Read first 32 bytes ──
    sub     sp, sp, #48             // 48 bytes aligned (32 header + 16 scratch)
    mov     x0, x28                 // fd
    mov     x1, sp                  // buf
    mov     x2, #HEADER_SIZE
    mov     x8, #SYS_READ
    svc     #0
    cmp     x0, #HEADER_SIZE
    b.ne    err_header

    // Check if ELF magic: 7f 45 4c 46
    ldr     w0, [sp]
    movz    w1, #0x457f             // "\x7fE" LE
    movk    w1, #0x464c, lsl #16   // "LF" LE
    cmp     w0, w1
    b.ne    .header_ok

    // ── ELF detected → read 8-byte trailer from end of file ──
    // lseek(fd, -8, SEEK_END)
    mov     x0, x28                 // fd
    mov     x1, #-8                 // offset = -8
    mov     x2, #2                  // SEEK_END
    mov     x8, #SYS_LSEEK
    svc     #0

    // Read 8 bytes (header_offset u64 LE)
    mov     x0, x28                 // fd
    mov     x1, sp                  // buf (reuse)
    mov     x2, #8
    mov     x8, #SYS_READ
    svc     #0

    // lseek(fd, header_offset, SEEK_SET)
    ldr     x1, [sp]               // header_offset
    mov     x0, x28                 // fd
    mov     x2, #0                  // SEEK_SET
    mov     x8, #SYS_LSEEK
    svc     #0

    // Re-read 32 bytes origin header
    mov     x0, x28                 // fd
    mov     x1, sp                  // buf
    mov     x2, #HEADER_SIZE
    mov     x8, #SYS_READ
    svc     #0
    cmp     x0, #HEADER_SIZE
    b.ne    err_header

.header_ok:
    // Parse origin header fields
    // offset 14: bc_offset (4 bytes LE)
    // offset 18: bc_size (4 bytes LE)
    // offset 30: flags (2 bytes LE)
    ldr     w9, [sp, #14]           // bc_offset
    ldr     w10, [sp, #18]          // bc_size
    ldrh    w11, [sp, #30]          // flags
    add     sp, sp, #48             // free stack

    // ── 3. mmap VM stack ──
    mov     x0, xzr
    mov     x1, #STACK_SIZE
    mov     x2, #3                  // PROT_READ|PROT_WRITE
    mov     x3, #0x22               // MAP_PRIVATE|MAP_ANONYMOUS
    mov     x4, #-1
    mov     x5, xzr
    mov     x8, #SYS_MMAP
    svc     #0
    cmp     x0, #0
    b.lt    err_mmap
    mov     x21, x0                 // stack_base
    mov     x20, x0                 // sp_vm = base (grows up)

    // ── 4. mmap heap ──
    mov     x0, xzr
    mov     x1, #HEAP_SIZE
    mov     x2, #3
    mov     x3, #0x22
    mov     x4, #-1
    mov     x5, xzr
    mov     x8, #SYS_MMAP
    svc     #0
    cmp     x0, #0
    b.lt    err_mmap
    mov     x22, x0                 // heap_base
    mov     x23, x0                 // heap_ptr

    // ── 5. Read bytecode into heap ──
    // lseek to bc_offset
    mov     x0, x28                 // fd
    mov     x1, x9                  // offset = bc_offset (w9 → x9)
    mov     x2, #0                  // SEEK_SET
    mov     x8, #SYS_LSEEK
    svc     #0

    // Read bytecode
    mov     x0, x28
    mov     x1, x23                 // buf = heap
    mov     x2, x10                 // count = bc_size
    mov     x8, #SYS_READ
    svc     #0
    cmp     x0, x10
    b.ne    err_read_bc

    // Setup VM registers
    mov     x24, x23                // bc_start = heap
    add     x25, x24, x10           // bc_end = bc_start + bc_size
    mov     x19, x24                // pc = bc_start
    add     x23, x23, x10           // bump heap past bytecode

    // Close file
    mov     x0, x28
    mov     x8, #SYS_CLOSE
    svc     #0

    // Init variable table
    mov     x26, x23                // var_table = heap
    mov     x27, #0                 // var_count = 0
    add     x23, x23, #4096         // reserve 4KB for vars

    // ── 6. Enter VM loop ──
    b       vm_loop

// ═══════════════════════════════════════════════════════════════════════════
// VM dispatch loop
// ═══════════════════════════════════════════════════════════════════════════

vm_loop:
    // Bounds check
    cmp     x19, x25
    b.hs    op_halt

    // Fetch tag
    ldrb    w0, [x19], #1           // w0 = *pc++

    // Dispatch via if-chain (codegen format 0x01-0x1C)
    cmp     w0, #0x01
    b.eq    op_push
    cmp     w0, #0x02
    b.eq    op_load
    cmp     w0, #0x03
    b.eq    op_lca
    cmp     w0, #0x04
    b.eq    op_edge
    cmp     w0, #0x05
    b.eq    op_query
    cmp     w0, #0x06
    b.eq    op_emit
    cmp     w0, #0x07
    b.eq    op_call
    cmp     w0, #0x08
    b.eq    vm_loop                 // Ret stub
    cmp     w0, #0x09
    b.eq    op_jmp
    cmp     w0, #0x0A
    b.eq    op_jz
    cmp     w0, #0x0B
    b.eq    op_dup
    cmp     w0, #0x0C
    b.eq    op_pop
    cmp     w0, #0x0D
    b.eq    op_swap
    cmp     w0, #0x0E
    b.eq    op_loop
    cmp     w0, #0x0F
    b.eq    op_halt
    cmp     w0, #0x13
    b.eq    op_store
    cmp     w0, #0x14
    b.eq    op_load                 // LoadLocal = Load
    cmp     w0, #0x15
    b.eq    op_push_num
    cmp     w0, #0x19
    b.eq    op_push_mol
    cmp     w0, #0x1A
    b.eq    op_try_begin
    cmp     w0, #0x1C
    b.eq    op_store                // StoreUpdate = Store
    // 0x10-0x12, 0x16-0x18, 0x1B = nop
    cmp     w0, #0x1C
    b.ls    vm_loop                 // known range, nop
    b       op_unknown

// ═══════════════════════════════════════════════════════════════════════════
// Stack operations
// ═══════════════════════════════════════════════════════════════════════════

op_push:
    // [0x01, chain_len:2 LE, chain_bytes:N]
    ldrh    w1, [x19]               // chain_len (u16 LE)
    add     x19, x19, #2

    // Copy to heap
    mov     x2, x23                 // dest = heap_ptr
    mov     x3, x19                 // src = pc
    mov     w4, w1                  // count
    bl      memcpy_small

    // Push (ptr, len)
    str     x23, [x20]              // ptr
    str     x1, [x20, #8]           // len
    add     x20, x20, #16
    add     x23, x23, x1            // bump heap
    add     x19, x19, x1            // pc += chain_len
    b       vm_loop

op_push_num:
    // [0x15, f64:8 LE]
    ldr     x0, [x19]               // raw f64 bits
    add     x19, x19, #8

    // Store on heap
    str     x0, [x23]
    str     x23, [x20]              // ptr
    mov     x1, #8
    str     x1, [x20, #8]           // len = 8
    add     x20, x20, #16
    add     x23, x23, #8
    b       vm_loop

op_push_mol:
    // [0x19, S:1, R:1, V:1, A:1, T:1]
    mov     x2, x23                 // dest
    mov     x3, x19                 // src
    mov     w4, #5
    bl      memcpy_small
    str     x23, [x20]
    mov     x1, #5
    str     x1, [x20, #8]
    add     x20, x20, #16
    add     x23, x23, #5
    add     x19, x19, #5
    b       vm_loop

op_dup:
    ldr     x0, [x20, #-16]        // peek ptr
    ldr     x1, [x20, #-8]         // peek len
    str     x0, [x20]
    str     x1, [x20, #8]
    add     x20, x20, #16
    b       vm_loop

op_pop:
    sub     x20, x20, #16
    b       vm_loop

op_swap:
    ldr     x0, [x20, #-16]        // top ptr
    ldr     x1, [x20, #-8]         // top len
    ldr     x2, [x20, #-32]        // second ptr
    ldr     x3, [x20, #-24]        // second len
    str     x2, [x20, #-16]
    str     x3, [x20, #-8]
    str     x0, [x20, #-32]
    str     x1, [x20, #-24]
    b       vm_loop

// ═══════════════════════════════════════════════════════════════════════════
// Variable store/load (hash-based)
// ═══════════════════════════════════════════════════════════════════════════

op_store:
    // [0x13, name_len:1, name:N]
    ldrb    w1, [x19], #1           // name_len, x19 now at name bytes

    // Save name_len and value before hash call
    sub     x20, x20, #16           // pop value
    ldr     x9, [x20]               // val_ptr
    ldr     x10, [x20, #8]          // val_len

    // Hash the name
    stp     x9, x10, [sp, #-16]!   // save val ptr/len
    str     x1, [sp, #-8]!         // save name_len
    mov     x0, x19                 // name ptr
    mov     w2, w1                  // name len
    bl      fnv1a_hash              // → x0 = hash
    ldr     x1, [sp], #8           // restore name_len
    ldp     x9, x10, [sp], #16     // restore val ptr/len

    add     x19, x19, x1           // pc += name_len

    // Store: search var table by hash, update or insert
    // x0=hash, x9=val_ptr, x10=val_len
    // Var table at x26, count at x27
    // Entry: [hash:8][ptr:8][len:8] = 24 bytes
    mov     x2, x26                 // table base
    mov     w3, w27                 // count
    cbz     w3, .vs_new

    mov     w4, #0                  // index
.vs_search:
    ldr     x5, [x2]               // entry hash
    cmp     x5, x0
    b.ne    .vs_skip
    // Found — update
    str     x9, [x2, #8]
    str     x10, [x2, #16]
    b       vm_loop
.vs_skip:
    add     x2, x2, #24
    add     w4, w4, #1
    cmp     w4, w3
    b.lt    .vs_search

.vs_new:
    // Append new entry
    mov     x2, x26
    mov     w3, w27
    mov     w4, #24
    mul     w3, w3, w4
    add     x2, x2, x3             // ptr to new slot
    str     x0, [x2]               // hash
    str     x9, [x2, #8]           // val_ptr
    str     x10, [x2, #16]         // val_len
    add     w27, w27, #1            // count++
    b       vm_loop

op_load:
    // [0x02, name_len:1, name:N]
    ldrb    w1, [x19], #1           // name_len

    // Hash the name
    str     x1, [sp, #-8]!
    mov     x0, x19                 // name ptr
    mov     w2, w1
    bl      fnv1a_hash              // → x0 = hash
    ldr     x1, [sp], #8

    add     x19, x19, x1           // pc += name_len

    // Search var table by hash
    mov     x2, x26                 // table base
    mov     w3, w27                 // count
    cbz     w3, .vl_not_found

    mov     w4, #0
.vl_search:
    ldr     x5, [x2]
    cmp     x5, x0
    b.ne    .vl_skip
    // Found
    ldr     x6, [x2, #8]           // val_ptr
    ldr     x7, [x2, #16]          // val_len
    str     x6, [x20]
    str     x7, [x20, #8]
    add     x20, x20, #16
    b       vm_loop
.vl_skip:
    add     x2, x2, #24
    add     w4, w4, #1
    cmp     w4, w3
    b.lt    .vl_search

.vl_not_found:
    str     xzr, [x20]
    str     xzr, [x20, #8]
    add     x20, x20, #16
    b       vm_loop

// ═══════════════════════════════════════════════════════════════════════════
// Control flow
// ═══════════════════════════════════════════════════════════════════════════

op_jmp:
    // [0x09, target:4 LE]
    ldr     w0, [x19]               // target offset
    add     x19, x24, x0            // pc = bc_start + target
    b       vm_loop

op_jz:
    // [0x0A, target:4 LE]
    ldr     w0, [x19]
    add     x19, x19, #4

    // Pop and check
    sub     x20, x20, #16
    ldr     x1, [x20]               // ptr
    ldr     x2, [x20, #8]           // len

    // Empty check (len == 0)
    cbz     x2, .jz_take

    // f64 zero check (len == 8)
    cmp     x2, #8
    b.ne    vm_loop
    ldr     x3, [x1]                // f64 bits
    cbz     x3, .jz_take            // 0.0 = all zeros
    b       vm_loop

.jz_take:
    add     x19, x24, x0            // pc = bc_start + target
    b       vm_loop

op_loop:
    // [0x0E, count:4]
    add     x19, x19, #4            // skip count (stub)
    b       vm_loop

op_try_begin:
    add     x19, x19, #4            // skip target
    b       vm_loop

// ═══════════════════════════════════════════════════════════════════════════
// Emit + Call
// ═══════════════════════════════════════════════════════════════════════════

op_emit:
    sub     x20, x20, #16
    ldr     x1, [x20]               // ptr
    ldr     x2, [x20, #8]           // len
    cbz     x2, vm_loop

    // Write to stdout
    mov     x0, #1                  // fd = stdout
    mov     x8, #SYS_WRITE
    svc     #0
    b       vm_loop

op_call:
    // [0x07, name_len:1, name:N]
    ldrb    w1, [x19], #1           // name_len

    // Hash the function name (FNV-1a)
    stp     x29, x30, [sp, #-16]!
    str     x1, [sp, #-8]!         // save name_len
    mov     x0, x19                 // name ptr
    mov     w2, w1                  // name len
    bl      fnv1a_hash              // → x0 = hash
    ldr     x1, [sp], #8           // restore name_len
    ldp     x29, x30, [sp], #16

    add     x19, x19, x1           // pc += name_len

    // ── Dispatch builtins by hash ──
    // Pre-computed FNV-1a hashes (verified by rustc):
    //   __hyp_add = 0x57252090b96bc074
    //   __hyp_sub = 0xf28afa91114a9d15
    //   __hyp_mul = 0x7800c290cb86ca01
    //   __hyp_div = 0x3e279790ab8c5ba8
    //   __eq      = 0x59c2494311519859
    //   __cmp_lt  = 0xe50d9329a4496776
    //   __cmp_gt  = 0xe525a529a45e17ed
    //   __cmp_le  = 0xe50d8429a4494df9
    //   __cmp_ge  = 0xe5259429a45dfb0a
    //   __len     = 0x231f02f69878e492
    //   __concat  = 0xa51ed5c2bde7e73b
    //   __char_at = 0x9fc9b1482b01d5e9
    //   __substr  = 0xe386a6329e2424be
    //   __push    = 0x9fc459281d29be97
    //   __pop     = 0x46a5acf6acd83186

    // Check math builtins (most common)
    movz    x2, #0xc074
    movk    x2, #0xb96b, lsl #16
    movk    x2, #0x2090, lsl #32
    movk    x2, #0x5725, lsl #48   // __hyp_add
    cmp     x0, x2
    b.eq    .call_add

    movz    x2, #0x9d15
    movk    x2, #0x114a, lsl #16
    movk    x2, #0xfa91, lsl #32
    movk    x2, #0xf28a, lsl #48   // __hyp_sub
    cmp     x0, x2
    b.eq    .call_sub

    movz    x2, #0xca01
    movk    x2, #0xcb86, lsl #16
    movk    x2, #0xc290, lsl #32
    movk    x2, #0x7800, lsl #48   // __hyp_mul
    cmp     x0, x2
    b.eq    .call_mul

    movz    x2, #0x5ba8
    movk    x2, #0xab8c, lsl #16
    movk    x2, #0x9790, lsl #32
    movk    x2, #0x3e27, lsl #48   // __hyp_div
    cmp     x0, x2
    b.eq    .call_div

    // Comparison builtins
    movz    x2, #0x9859
    movk    x2, #0x1151, lsl #16
    movk    x2, #0x4943, lsl #32
    movk    x2, #0x59c2, lsl #48   // __eq
    cmp     x0, x2
    b.eq    .call_eq

    movz    x2, #0x6776
    movk    x2, #0xa449, lsl #16
    movk    x2, #0x9329, lsl #32
    movk    x2, #0xe50d, lsl #48   // __cmp_lt
    cmp     x0, x2
    b.eq    .call_lt

    movz    x2, #0x17ed
    movk    x2, #0xa45e, lsl #16
    movk    x2, #0xa529, lsl #32
    movk    x2, #0xe525, lsl #48   // __cmp_gt
    cmp     x0, x2
    b.eq    .call_gt

    movz    x2, #0x4df9
    movk    x2, #0xa449, lsl #16
    movk    x2, #0x8429, lsl #32
    movk    x2, #0xe50d, lsl #48   // __cmp_le
    cmp     x0, x2
    b.eq    .call_le

    movz    x2, #0xfb0a
    movk    x2, #0xa45d, lsl #16
    movk    x2, #0x9429, lsl #32
    movk    x2, #0xe525, lsl #48   // __cmp_ge
    cmp     x0, x2
    b.eq    .call_ge

    // String builtins
    movz    x2, #0xe492
    movk    x2, #0x9878, lsl #16
    movk    x2, #0x02f6, lsl #32
    movk    x2, #0x231f, lsl #48   // __len
    cmp     x0, x2
    b.eq    .call_len

    movz    x2, #0xe73b
    movk    x2, #0xbde7, lsl #16
    movk    x2, #0xd5c2, lsl #32
    movk    x2, #0xa51e, lsl #48   // __concat
    cmp     x0, x2
    b.eq    .call_concat

    movz    x2, #0xd5e9
    movk    x2, #0x2b01, lsl #16
    movk    x2, #0xb148, lsl #32
    movk    x2, #0x9fc9, lsl #48   // __char_at
    cmp     x0, x2
    b.eq    .call_char_at

    movz    x2, #0x24be
    movk    x2, #0x9e24, lsl #16
    movk    x2, #0xa632, lsl #32
    movk    x2, #0xe386, lsl #48   // __substr
    cmp     x0, x2
    b.eq    .call_substr

    movz    x2, #0xbe97
    movk    x2, #0x1d29, lsl #16
    movk    x2, #0x5928, lsl #32
    movk    x2, #0x9fc4, lsl #48   // __push
    cmp     x0, x2
    b.eq    .call_push

    movz    x2, #0x3186
    movk    x2, #0xacd8, lsl #16
    movk    x2, #0xacf6, lsl #32
    movk    x2, #0x46a5, lsl #48   // __pop
    cmp     x0, x2
    b.eq    .call_pop

    // Unknown builtin — skip (no-op)
    b       vm_loop

// ── Math builtins (f64) ──────────────────────────────────────────────

.call_add:
    // Pop 2 f64, push a + b
    ldr     x0, [x20, #-16]        // b (raw f64 bits)
    ldr     x1, [x20, #-32]        // a
    fmov    d1, x0
    fmov    d0, x1
    fadd    d0, d0, d1
    fmov    x0, d0
    sub     x20, x20, #16           // pop one
    str     x0, [x20, #-16]         // overwrite a with result
    b       vm_loop

.call_sub:
    ldr     x0, [x20, #-16]        // b
    ldr     x1, [x20, #-32]        // a
    fmov    d1, x0
    fmov    d0, x1
    fsub    d0, d0, d1              // a - b
    fmov    x0, d0
    sub     x20, x20, #16
    str     x0, [x20, #-16]
    b       vm_loop

.call_mul:
    ldr     x0, [x20, #-16]        // b
    ldr     x1, [x20, #-32]        // a
    fmov    d1, x0
    fmov    d0, x1
    fmul    d0, d0, d1
    fmov    x0, d0
    sub     x20, x20, #16
    str     x0, [x20, #-16]
    b       vm_loop

.call_div:
    ldr     x0, [x20, #-16]        // b (divisor)
    fmov    d1, x0
    fcmp    d1, #0.0
    b.eq    .call_div_zero
    ldr     x1, [x20, #-32]        // a
    fmov    d0, x1
    fdiv    d0, d0, d1              // a / b
    fmov    x0, d0
    sub     x20, x20, #16
    str     x0, [x20, #-16]
    b       vm_loop
.call_div_zero:
    // Push 0.0 on divide by zero
    sub     x20, x20, #16
    str     xzr, [x20, #-16]
    b       vm_loop

// ── Comparison builtins ──────────────────────────────────────────────

.call_eq:
    // Pop 2, push 1.0 if equal, empty if not
    ldr     x0, [x20, #-16]        // b ptr/bits
    ldr     x1, [x20, #-8]         // b len
    ldr     x2, [x20, #-32]        // a ptr/bits
    ldr     x3, [x20, #-24]        // a len
    sub     x20, x20, #16           // pop one

    // Both f64? (len == F64_MARKER)
    cmp     x1, #F64_MARKER
    b.ne    .eq_chain
    cmp     x3, #F64_MARKER
    b.ne    .eq_false

    // f64 compare
    fmov    d0, x2
    fmov    d1, x0
    fcmp    d0, d1
    b.ne    .eq_false
    // Equal → push 1.0
    movz    x0, #0x0000
    movk    x0, #0x0000, lsl #16
    movk    x0, #0xf03f, lsl #32   // 1.0 = 0x3FF0000000000000
    movk    x0, #0x3ff0, lsl #48
    str     x0, [x20, #-16]
    mov     x1, #F64_MARKER
    str     x1, [x20, #-8]
    b       vm_loop

.eq_chain:
    // Chain compare: same length + same bytes
    cmp     x1, x3
    b.ne    .eq_false
    // Compare bytes
    mov     x4, #0                  // index
.eq_cmp_loop:
    cmp     x4, x1
    b.hs    .eq_true
    ldrb    w5, [x2, x4]
    ldrb    w6, [x0, x4]
    cmp     w5, w6
    b.ne    .eq_false
    add     x4, x4, #1
    b       .eq_cmp_loop
.eq_true:
    movz    x0, #0x0000
    movk    x0, #0x0000, lsl #16
    movk    x0, #0xf03f, lsl #32
    movk    x0, #0x3ff0, lsl #48   // 1.0
    str     x0, [x20, #-16]
    mov     x1, #F64_MARKER
    str     x1, [x20, #-8]
    b       vm_loop
.eq_false:
    str     xzr, [x20, #-16]       // empty
    str     xzr, [x20, #-8]
    b       vm_loop

.call_lt:
    ldr     x0, [x20, #-16]        // b
    ldr     x1, [x20, #-32]        // a
    fmov    d1, x0
    fmov    d0, x1
    sub     x20, x20, #16
    fcmp    d0, d1
    b.lt    .cmp_true
    b       .cmp_false

.call_gt:
    ldr     x0, [x20, #-16]
    ldr     x1, [x20, #-32]
    fmov    d1, x0
    fmov    d0, x1
    sub     x20, x20, #16
    fcmp    d0, d1
    b.gt    .cmp_true
    b       .cmp_false

.call_le:
    ldr     x0, [x20, #-16]
    ldr     x1, [x20, #-32]
    fmov    d1, x0
    fmov    d0, x1
    sub     x20, x20, #16
    fcmp    d0, d1
    b.le    .cmp_true
    b       .cmp_false

.call_ge:
    ldr     x0, [x20, #-16]
    ldr     x1, [x20, #-32]
    fmov    d1, x0
    fmov    d0, x1
    sub     x20, x20, #16
    fcmp    d0, d1
    b.ge    .cmp_true
    b       .cmp_false

.cmp_true:
    movz    x0, #0x0000
    movk    x0, #0x0000, lsl #16
    movk    x0, #0xf03f, lsl #32
    movk    x0, #0x3ff0, lsl #48   // 1.0
    str     x0, [x20, #-16]
    mov     x1, #F64_MARKER
    str     x1, [x20, #-8]
    b       vm_loop

.cmp_false:
    str     xzr, [x20, #-16]
    str     xzr, [x20, #-8]
    b       vm_loop

// ── String builtins ──────────────────────────────────────────────────

.call_len:
    // Pop 1 value, push its length as f64
    sub     x20, x20, #16
    ldr     x1, [x20, #8]          // len
    // Convert len to f64
    scvtf   d0, x1
    fmov    x0, d0
    str     x0, [x20]              // push f64 bits
    mov     x1, #F64_MARKER
    str     x1, [x20, #8]
    add     x20, x20, #16
    b       vm_loop

.call_concat:
    // Pop 2 strings, push concatenation
    sub     x20, x20, #16
    ldr     x4, [x20]              // b ptr
    ldr     x5, [x20, #8]          // b len
    sub     x20, x20, #16
    ldr     x6, [x20]              // a ptr
    ldr     x7, [x20, #8]          // a len

    // Copy a to heap
    mov     x2, x23                 // dest start
    mov     x3, x6                  // src = a
    mov     w4, w7                  // count = a len
    stp     x5, x4, [sp, #-16]!   // save b len, b ptr (reuse x4)
    // Actually save properly
    ldp     x5, x4, [sp], #16
    // Simpler: copy a bytes
    mov     x8, x23                 // result start
    mov     x9, #0                  // index
.cc_a:
    cmp     x9, x7
    b.hs    .cc_b_start
    ldrb    w10, [x6, x9]
    strb    w10, [x23]
    add     x23, x23, #1
    add     x9, x9, #1
    b       .cc_a
.cc_b_start:
    // Copy b to heap after a
    ldr     x4, [x20, #16]         // b ptr (was popped earlier, re-read from old position)
    ldr     x5, [x20, #24]         // b len
    mov     x9, #0
.cc_b:
    cmp     x9, x5
    b.hs    .cc_done
    ldrb    w10, [x4, x9]
    strb    w10, [x23]
    add     x23, x23, #1
    add     x9, x9, #1
    b       .cc_b
.cc_done:
    // Push result
    str     x8, [x20]              // ptr = start of concat
    add     x0, x7, x5             // total len
    str     x0, [x20, #8]
    add     x20, x20, #16
    b       vm_loop

.call_char_at:
    // Pop index (f64), pop string, push char
    sub     x20, x20, #16
    ldr     x0, [x20]              // index (f64 bits)
    fmov    d0, x0
    fcvtzs  x0, d0                  // index as int
    sub     x20, x20, #16
    ldr     x1, [x20]              // str ptr
    ldr     x2, [x20, #8]          // str len

    // Bounds check
    cmp     x0, x2
    b.hs    .ca_empty
    // Get char
    ldrb    w3, [x1, x0]
    strb    w3, [x23]
    str     x23, [x20]
    mov     x1, #1
    str     x1, [x20, #8]
    add     x20, x20, #16
    add     x23, x23, #1
    b       vm_loop
.ca_empty:
    str     xzr, [x20]
    str     xzr, [x20, #8]
    add     x20, x20, #16
    b       vm_loop

.call_substr:
    // Pop len (f64), pop start (f64), pop string → push substring
    sub     x20, x20, #16
    ldr     x0, [x20]              // sub_len (f64 bits)
    fmov    d0, x0
    fcvtzs  x3, d0                  // sub_len as int

    sub     x20, x20, #16
    ldr     x0, [x20]              // start (f64 bits)
    fmov    d0, x0
    fcvtzs  x2, d0                  // start as int

    sub     x20, x20, #16
    ldr     x0, [x20]              // str ptr
    ldr     x1, [x20, #8]          // str len

    // Clamp
    cmp     x2, x1
    b.hs    .ss_empty
    add     x4, x2, x3             // start + sub_len
    cmp     x4, x1
    b.ls    .ss_ok
    sub     x3, x1, x2             // clamp to remaining
.ss_ok:
    // Copy to heap
    add     x5, x0, x2             // src = str + start
    mov     x6, x23                 // dest
    mov     x7, #0
.ss_copy:
    cmp     x7, x3
    b.hs    .ss_done
    ldrb    w8, [x5, x7]
    strb    w8, [x23]
    add     x23, x23, #1
    add     x7, x7, #1
    b       .ss_copy
.ss_done:
    str     x6, [x20]
    str     x3, [x20, #8]
    add     x20, x20, #16
    b       vm_loop
.ss_empty:
    str     xzr, [x20]
    str     xzr, [x20, #8]
    add     x20, x20, #16
    b       vm_loop

.call_push:
    // Pop value, pop array-like → append (simplified: just skip for now)
    // In VM context: arrays are heap regions, push extends
    b       vm_loop

.call_pop:
    // Pop from array-like → push popped value
    b       vm_loop

// ═══════════════════════════════════════════════════════════════════════════
// LCA + Edge + Query
// ═══════════════════════════════════════════════════════════════════════════

op_lca:
    // Pop 2, average 5 dimensions, push result
    sub     x20, x20, #16
    ldr     x0, [x20]               // chain A ptr
    ldr     x1, [x20, #8]           // chain A len
    sub     x20, x20, #16
    ldr     x2, [x20]               // chain B ptr
    ldr     x3, [x20, #8]           // chain B len

    // Need >= 5 bytes each
    cmp     x1, #5
    b.lo    .lca_empty
    cmp     x3, #5
    b.lo    .lca_empty

    // Average each dim
    mov     x4, x23                 // result ptr
    mov     w5, #0                  // dim counter
.lca_dim:
    ldrb    w6, [x0, x5]
    ldrb    w7, [x2, x5]
    add     w6, w6, w7
    lsr     w6, w6, #1
    strb    w6, [x23, x5]
    add     w5, w5, #1
    cmp     w5, #5
    b.lo    .lca_dim

    str     x4, [x20]
    mov     x1, #5
    str     x1, [x20, #8]
    add     x20, x20, #16
    add     x23, x23, #5
    b       vm_loop

.lca_empty:
    str     xzr, [x20]
    str     xzr, [x20, #8]
    add     x20, x20, #16
    b       vm_loop

op_edge:
    ldrb    w0, [x19], #1           // relation byte
    sub     x20, x20, #16           // pop 2, push empty
    str     xzr, [x20, #-16]
    str     xzr, [x20, #-8]
    b       vm_loop

op_query:
    ldrb    w0, [x19], #1
    str     xzr, [x20, #-16]       // overwrite top with empty
    str     xzr, [x20, #-8]
    b       vm_loop

// ═══════════════════════════════════════════════════════════════════════════
// Utilities
// ═══════════════════════════════════════════════════════════════════════════

memcpy_small:
    // x2=dest, x3=src, w4=count
    cbz     w4, .mc_done
.mc_loop:
    ldrb    w5, [x3], #1
    strb    w5, [x2], #1
    subs    w4, w4, #1
    b.ne    .mc_loop
.mc_done:
    ret

fnv1a_hash:
    // x0=data ptr, w2=len → x0=hash
    // FNV-1a offset basis = 0xcbf29ce484222325
    movz    x3, #0x2325
    movk    x3, #0x4222, lsl #16
    movk    x3, #0x9ce4, lsl #32
    movk    x3, #0xcbf2, lsl #48

    movz    x4, #0x01b3              // FNV prime
    movk    x4, #0x0000, lsl #16
    movk    x4, #0x0001, lsl #32
    movk    x4, #0x0100, lsl #48

    add     x5, x0, x2              // end ptr
.fnv_loop:
    cmp     x0, x5
    b.hs    .fnv_done
    ldrb    w6, [x0], #1
    eor     x3, x3, x6
    mul     x3, x3, x4
    b       .fnv_loop
.fnv_done:
    mov     x0, x3
    ret

// ═══════════════════════════════════════════════════════════════════════════
// Error handlers
// ═══════════════════════════════════════════════════════════════════════════

op_unknown:
    adr     x1, msg_unknown
    mov     x2, #15
    mov     x0, #2                  // stderr
    mov     x8, #SYS_WRITE
    svc     #0
    b       op_halt_err

op_nop:
    b       vm_loop

err_open:
    adr     x1, msg_open
    mov     x2, #20
    b       .err_print

err_header:
    adr     x1, msg_header
    mov     x2, #17
    b       .err_print

err_mmap:
    adr     x1, msg_mmap
    mov     x2, #13
    b       .err_print

err_read_bc:
    adr     x1, msg_readbc
    mov     x2, #20
    b       .err_print

.err_print:
    mov     x0, #2
    mov     x8, #SYS_WRITE
    svc     #0
    b       op_halt_err

// ═══════════════════════════════════════════════════════════════════════════
// Exit
// ═══════════════════════════════════════════════════════════════════════════

op_halt:
    mov     x0, #0
    mov     x8, #SYS_EXIT
    svc     #0

op_halt_err:
    mov     x0, #1
    mov     x8, #SYS_EXIT
    svc     #0

// ═══════════════════════════════════════════════════════════════════════════
// Read-only data
// ═══════════════════════════════════════════════════════════════════════════

.section .rodata

proc_self_exe:
    .asciz "/proc/self/exe"

msg_unknown:
    .ascii "Unknown opcode!\n"

msg_open:
    .ascii "Cannot open binary!\n"

msg_header:
    .ascii "Bad header read!\n"

msg_mmap:
    .ascii "mmap failed!\n"

msg_readbc:
    .ascii "Bytecode read fail!\n"
