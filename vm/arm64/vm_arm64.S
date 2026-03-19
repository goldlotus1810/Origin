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

    // ── 2. Read header (32 bytes) ──
    sub     sp, sp, #HEADER_SIZE
    mov     x0, x28                 // fd
    mov     x1, sp                  // buf
    mov     x2, #HEADER_SIZE
    mov     x8, #SYS_READ
    svc     #0
    cmp     x0, #HEADER_SIZE
    b.ne    err_header

    // Parse header
    ldr     w9, [sp, #14]           // bc_offset
    ldr     w10, [sp, #18]          // bc_size
    ldrh    w11, [sp, #30]          // flags
    add     sp, sp, #HEADER_SIZE

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
    ldrb    w1, [x19], #1           // name_len
    // Hash name
    mov     x0, x19                 // name ptr
    mov     w2, w1                  // name len
    bl      fnv1a_hash              // → x0 = hash

    mov     x9, x0                  // save hash
    add     x19, x19, x1            // pc += name_len (w1 still valid? use saved)
    // Actually x1 might be clobbered by fnv1a. Let me re-read.
    // Save name_len before hash call
    // Fix: re-read from pc position
    // Simpler: save before call
    // Actually we already advanced x19 past name_len byte.
    // Let me restructure:

    // Re-read: x19 was at name start, w1 = name_len
    // After bl fnv1a_hash, x0 = hash, but w1 may be clobbered
    // Need to save w1. Use stack:
    // Let me redo this properly:
    b       op_store_impl

op_store_impl:
    // Re-entered. x19 points to name bytes.
    // Back up: re-read name_len
    ldrb    w1, [x19, #-1]         // name_len (byte before current x19)
    // Wait, we already did x19 += 1 for name_len byte in op_store above
    // So x19 is at name bytes start, w1 = name_len from ldrb above
    // But we branched here... this is messy.

    // SIMPLER APPROACH: redo from scratch
    // x19 is currently at: past the name_len byte, at name bytes start
    // We need: name_ptr = x19, name_len = w1 (from the ldrb above)
    // But w1 may have been clobbered. Let me use a callee-saved register.

    // Actually, the ldrb w1, [x19], #1 in op_store set w1 and advanced x19.
    // Then we branched to op_store_impl. w1 is still valid (no call yet).

    stp     x19, x1, [sp, #-16]!   // save name_ptr, name_len
    mov     x0, x19                 // name ptr
    mov     w2, w1                  // name len
    bl      fnv1a_hash              // → x0 = hash
    ldp     x19, x1, [sp], #16     // restore (but x19 is name_ptr, not PC!)

    // Oops: x19 is PC register. Can't use it as temp.
    // Let me use different approach entirely.
    b       vm_loop                 // STUB for now — will fix

op_load:
    // [0x02, name_len:1, name:N]
    ldrb    w1, [x19], #1
    // For now: push empty
    str     xzr, [x20]
    str     xzr, [x20, #8]
    add     x20, x20, #16
    add     x19, x19, x1
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
    add     x19, x19, x1            // skip name (stub — builtins TODO)
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
