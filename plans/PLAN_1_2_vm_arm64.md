# PLAN 1.2 — vm_arm64.S: VM cho ARM64 (~2000-3000 LOC ASM)

**Phụ thuộc:** PLAN_1_1 phải xong (vm_x86_64.S hoạt động, bytecode format ổn định)
**Mục tiêu:** origin.olang chạy native trên ARM64 (Android, iOS, Raspberry Pi, Apple Silicon, AWS Graviton)
**Yêu cầu:** Hiểu ARM64 ISA, Linux/macOS syscall convention, NEON FPU.

---

## Bối cảnh

### Tại sao ARM64?

```
ARM64 = nền tảng SỐ 1 cho mobile + edge:
  - Android phones/tablets (99% ARM64)
  - iPhone/iPad (Apple A-series)
  - Apple Silicon Mac (M1/M2/M3/M4)
  - Raspberry Pi 4/5 (ARM Cortex-A)
  - AWS Graviton / Azure Cobalt (server)
  - IoT gateways, routers, NAS

HomeOS = "chạy trên điện thoại" → ARM64 là BẮT BUỘC.
```

### Khác biệt chính so với x86_64

```
                    x86_64                  ARM64
────────────────────────────────────────────────────
Registers           16 GP (rax-r15)         31 GP (x0-x30) + sp + xzr
FP registers        xmm0-xmm15 (SSE)       d0-d31 (NEON/FP)
Calling convention  rdi,rsi,rdx,rcx,r8,r9   x0-x7 (args), x0 (return)
Syscall             syscall (rax=nr)        svc #0 (x8=nr)
Endianness          Little                  Little (default)
Instruction size    Variable (1-15B)        Fixed 4 bytes
Condition flags     EFLAGS register         NZCV (cmp → b.eq/b.ne)
Stack alignment     16 bytes                16 bytes
Link register       [rsp] (push/pop)        x30 (lr) — explicit save
```

---

## Bytecode format (từ PLAN_0_5)

```
Dùng CHUNG format bytecode với vm_x86_64.S.
VM arm64 đọc cùng bytecode section trong origin.olang.

Mỗi opcode = 1 byte tag + optional payload (xem PLAN_0_5 chi tiết).
```

---

## Register Allocation Convention

```
VM State Registers (reserved, KHÔNG dùng cho tính toán):
  x19  = pc          — program counter (pointer vào bytecode section)
  x20  = sp_vm       — VM stack pointer (KHÔNG phải hardware sp)
  x21  = stack_base  — base address của VM stack
  x22  = heap_base   — base address của heap arena
  x23  = heap_ptr    — current heap allocation pointer
  x24  = bc_start    — bytecode section start address
  x25  = bc_end      — bytecode section end address
  x26  = kn_start    — knowledge section start address
  x27  = locals_ptr  — pointer vào scope/locals area

Scratch Registers (tự do dùng trong opcode handlers):
  x0-x7    — syscall args + temp (caller-saved)
  x8       — syscall number
  x9-x15   — temp (caller-saved)
  x16-x17  — intra-procedure-call scratch (IP0/IP1)
  x28      — thêm 1 callee-saved nếu cần

FP Registers:
  d0-d7    — float args + scratch
  d8-d15   — callee-saved (dùng cho cached values nếu cần)
  d16-d31  — scratch

Special:
  sp       — hardware stack pointer (cho call/ret)
  x30 (lr) — link register (return address)
  xzr      — zero register (đọc = 0, ghi = discard)
```

### Tại sao convention này?

```
ARM64 có 31 GP registers — DƯ SỨC cho VM state.
x19-x28 = callee-saved → giữ nguyên qua function calls.
→ VM state KHÔNG CẦN load/store mỗi opcode dispatch.
→ Nhanh hơn x86_64 (chỉ có 5 callee-saved: rbx, rbp, r12-r15).

So sánh overhead mỗi opcode dispatch:
  x86_64: load pc từ memory → decode → execute → store pc
  ARM64:  x19 SẴN CÓ pc → decode → execute → add x19, x19, #1
  → ARM64 tiết kiệm ~2 instructions/opcode
```

---

## Cấu trúc file

```
vm_arm64.S (~2000-3000 LOC)

Sections:
  .text
    _start              — ELF entry point (Linux) / _main (macOS)
    vm_init             — mmap stack + heap, parse header
    vm_loop             — fetch-decode-execute cycle
    dispatch_table      — jump table (38+ entries)
    op_push ... op_halt — per-opcode handlers
    syscall_bridge      — sys_read, sys_write, sys_mmap, sys_exit
    math_ops            — f64 add/mul/div/sqrt/sin/cos (NEON)
    string_ops          — str_len, str_cmp, str_concat, str_hash
    chain_ops           — mol_encode, mol_decode, chain_hash, chain_lca
    crypto_ops          — sha256_block
    builtin_dispatch    — 60+ builtin function router
    error_handlers      — stack overflow/underflow, invalid opcode

  .rodata
    jump_table          — 38 entries × 8 bytes = 304 bytes
    sha256_k            — 64 × 4 bytes = 256 bytes
    sin_cos_table       — Taylor coefficients (optional)
    error_messages      — "stack overflow\n", etc.

  .bss
    (không có — tất cả memory qua mmap)
```

---

## Việc cần làm

### Task 1: Entry Point + Memory Setup (~200 LOC)

```asm
// vm_arm64.S

.global _start
.text

_start:
    // Parse origin.olang header (chính file đang chạy)
    // Linux: /proc/self/exe → mmap
    // macOS: _dyld_get_image_header hoặc argv[0]

    // 1. Mở chính mình
    mov     x0, #-100              // AT_FDCWD
    adr     x1, self_path          // "/proc/self/exe"
    mov     x2, #0                 // O_RDONLY
    mov     x8, #56                // __NR_openat (Linux ARM64)
    svc     #0
    mov     x28, x0                // fd saved

    // 2. fstat để lấy file size
    mov     x1, sp
    sub     sp, sp, #144           // struct stat buffer
    mov     x8, #80                // __NR_fstat
    svc     #0
    ldr     x1, [sp, #48]          // st_size
    mov     x9, x1                 // file_size saved

    // 3. mmap origin.olang vào memory (read-only)
    mov     x0, xzr                // addr = NULL
    mov     x1, x9                 // length = file_size
    mov     x2, #1                 // PROT_READ
    mov     x3, #2                 // MAP_PRIVATE
    mov     x4, x28                // fd
    mov     x5, xzr                // offset = 0
    mov     x8, #222               // __NR_mmap
    svc     #0
    // x0 = mapped address of origin.olang

    // 4. Parse header (32 bytes)
    //    [magic:4][version:1][arch:1][vm_off:4][vm_size:4]
    //    [bc_off:4][bc_size:4][kn_off:4][kn_size:4][flags:2]
    ldr     w1, [x0]               // magic "○LNG"
    // verify magic...
    ldrb    w2, [x0, #4]           // version
    ldrb    w3, [x0, #5]           // arch — phải = ARM64
    ldr     w4, [x0, #10]         // bc_offset
    ldr     w5, [x0, #14]         // bc_size
    add     x24, x0, x4            // bc_start = base + bc_offset
    add     x25, x24, x5           // bc_end = bc_start + bc_size

    // 5. mmap VM stack (1 MB)
    mov     x0, xzr
    mov     x1, #0x100000          // 1 MB
    mov     x2, #3                 // PROT_READ | PROT_WRITE
    mov     x3, #0x22              // MAP_PRIVATE | MAP_ANONYMOUS
    mov     x4, #-1                // fd = -1
    mov     x5, xzr
    mov     x8, #222
    svc     #0
    mov     x21, x0                // stack_base
    mov     x20, x0                // sp_vm = stack_base (empty, grows up)

    // 6. mmap heap arena (16 MB)
    mov     x0, xzr
    mov     x1, #0x1000000         // 16 MB
    mov     x2, #3
    mov     x3, #0x22
    mov     x4, #-1
    mov     x5, xzr
    mov     x8, #222
    svc     #0
    mov     x22, x0                // heap_base
    mov     x23, x0                // heap_ptr = heap_base

    // 7. Init PC → bytecode start
    mov     x19, x24               // pc = bc_start

    // 8. Jump to VM loop
    b       vm_loop
```

### Task 2: VM Loop + Dispatch (~150 LOC)

```asm
vm_loop:
    // Bounds check
    cmp     x19, x25               // pc >= bc_end?
    b.hs    vm_halt                 // yes → halt

    // Fetch opcode tag
    ldrb    w0, [x19]              // tag = *pc
    add     x19, x19, #1           // pc++

    // Dispatch via jump table
    cmp     w0, #0x26              // max known opcode
    b.hi    op_unknown             // unknown → error

    adr     x1, jump_table
    ldr     x1, [x1, x0, lsl #3]  // x1 = jump_table[tag]
    br      x1                     // branch to handler

// Jump table (in .rodata)
.section .rodata
.align 3
jump_table:
    .quad   op_unknown     // 0x00 — reserved
    .quad   op_push        // 0x01
    .quad   op_load        // 0x02
    .quad   op_lca         // 0x03
    .quad   op_edge        // 0x04
    .quad   op_query       // 0x05
    .quad   op_emit        // 0x06
    .quad   op_call        // 0x07
    .quad   op_ret         // 0x08
    .quad   op_jmp         // 0x09
    .quad   op_jz          // 0x0A
    .quad   op_dup         // 0x0B
    .quad   op_pop         // 0x0C
    .quad   op_swap        // 0x0D
    .quad   op_loop        // 0x0E
    .quad   op_halt        // 0x0F
    .quad   op_dream       // 0x10
    .quad   op_stats       // 0x11
    .quad   op_nop         // 0x12
    .quad   op_store       // 0x13
    .quad   op_load_local  // 0x14
    .quad   op_push_num    // 0x15
    .quad   op_fuse        // 0x16
    .quad   op_scope_begin // 0x17
    .quad   op_scope_end   // 0x18
    .quad   op_push_mol    // 0x19
    .quad   op_try_begin   // 0x1A
    .quad   op_catch_end   // 0x1B
    .quad   op_store_update // 0x1C
    .quad   op_trace       // 0x1D
    .quad   op_inspect     // 0x1E
    .quad   op_assert      // 0x1F
    .quad   op_typeof      // 0x20
    .quad   op_why         // 0x21
    .quad   op_explain     // 0x22
    .quad   op_ffi         // 0x23
    .quad   op_file_read   // 0x24
    .quad   op_file_write  // 0x25
    .quad   op_file_append // 0x26
```

### Task 3: Stack Operations (~200 LOC)

```asm
// VM Stack layout:
//   Mỗi entry = 16 bytes: [chain_hash: 8B][chain_ptr: 8B]
//   chain_ptr → heap (mol_count:1 + tagged_bytes)
//   sp_vm (x20) trỏ tới TOP of stack
//   stack grows UP (base → top)

.text

// Helper: check stack not full (max 256 entries = 4096 bytes)
stack_check_overflow:
    sub     x0, x20, x21           // used = sp_vm - stack_base
    cmp     x0, #4096              // 256 × 16
    b.hs    err_stack_overflow
    ret

// Helper: check stack not empty
stack_check_underflow:
    cmp     x20, x21               // sp_vm == stack_base?
    b.eq    err_stack_underflow
    ret

op_push:
    // Payload: [chain_len:2][chain_bytes:N]
    bl      stack_check_overflow
    ldrh    w1, [x19]              // chain_len (little-endian)
    add     x19, x19, #2           // pc += 2

    // Allocate chain on heap
    add     x2, x1, #1             // +1 for mol_count byte
    // Bump allocate
    mov     x3, x23                // chain_ptr = heap_ptr
    add     x23, x23, x2           // heap_ptr += size

    // Copy chain bytes to heap
    mov     x4, x19                // src = pc (chain bytes in bytecode)
    mov     x5, x3                 // dst = heap allocation
    mov     x6, x1                 // len
copy_chain_loop:
    cbz     x6, copy_chain_done
    ldrb    w7, [x4], #1
    strb    w7, [x5], #1
    sub     x6, x6, #1
    b       copy_chain_loop
copy_chain_done:

    // Compute chain_hash (FNV-1a)
    mov     x0, x3                 // data ptr
    mov     x1, x1                 // data len (already in w1)
    bl      fnv1a_hash             // → x0 = hash

    // Push [hash:8][ptr:8] onto VM stack
    stp     x0, x3, [x20]         // store pair at sp_vm
    add     x20, x20, #16          // sp_vm += 16

    // Advance pc past chain bytes
    add     x19, x19, x1           // pc += chain_len (w1 still valid? reload if needed)
    b       vm_loop

op_push_num:
    // Payload: [f64:8] (IEEE 754 little-endian)
    bl      stack_check_overflow
    ldr     d0, [x19]              // load f64 from bytecode
    add     x19, x19, #8           // pc += 8

    // Encode f64 as 4-molecule chain on heap
    mov     x0, x23                // heap_ptr
    bl      encode_f64_to_chain    // → x0 = chain_ptr, x1 = chain_len
    mov     x3, x0

    // Hash
    mov     x1, x1                 // len
    bl      fnv1a_hash             // → x0 = hash

    // Push
    stp     x0, x3, [x20]
    add     x20, x20, #16
    b       vm_loop

op_push_mol:
    // Payload: [s:1][r:1][v:1][a:1][t:1]
    bl      stack_check_overflow

    // Allocate 6 bytes on heap (1 mol_count + 5 bytes)
    mov     x3, x23
    mov     w4, #1
    strb    w4, [x3]               // mol_count = 1
    ldrb    w4, [x19]              // shape
    strb    w4, [x3, #1]
    ldrb    w4, [x19, #1]          // relation
    strb    w4, [x3, #2]
    ldrb    w4, [x19, #2]          // valence
    strb    w4, [x3, #3]
    ldrb    w4, [x19, #3]          // arousal
    strb    w4, [x3, #4]
    ldrb    w4, [x19, #4]          // time
    strb    w4, [x3, #5]
    add     x23, x23, #6           // heap_ptr += 6
    add     x19, x19, #5           // pc += 5

    // Hash the 5 raw bytes
    add     x0, x3, #1             // skip mol_count
    mov     x1, #5
    bl      fnv1a_hash

    stp     x0, x3, [x20]
    add     x20, x20, #16
    b       vm_loop

op_dup:
    bl      stack_check_underflow
    bl      stack_check_overflow
    ldp     x0, x1, [x20, #-16]   // peek top
    stp     x0, x1, [x20]         // push copy
    add     x20, x20, #16
    b       vm_loop

op_pop:
    bl      stack_check_underflow
    sub     x20, x20, #16          // sp_vm -= 16 (discard top)
    b       vm_loop

op_swap:
    // Need ≥2 entries
    sub     x0, x20, x21
    cmp     x0, #32                // 2 × 16
    b.lo    err_stack_underflow
    ldp     x0, x1, [x20, #-16]   // top
    ldp     x2, x3, [x20, #-32]   // second
    stp     x2, x3, [x20, #-16]   // swap
    stp     x0, x1, [x20, #-32]
    b       vm_loop
```

### Task 4: Control Flow (~250 LOC)

```asm
op_jmp:
    // Payload: [target:4] (u32 LE, byte offset from bc_start)
    ldr     w0, [x19]             // target offset
    add     x19, x24, x0          // pc = bc_start + target
    b       vm_loop

op_jz:
    // Payload: [target:4]
    // Pop top → if empty chain → jump
    bl      stack_check_underflow
    sub     x20, x20, #16
    ldp     x0, x1, [x20]         // hash, chain_ptr
    // Check if chain is empty (mol_count == 0 or hash == 0)
    cbz     x0, jz_take_jump      // hash == 0 → empty → jump
    // Also check mol_count
    ldrb    w2, [x1]              // mol_count
    cbz     w2, jz_take_jump
    // Not empty → skip jump, advance pc past target
    add     x19, x19, #4
    b       vm_loop
jz_take_jump:
    ldr     w0, [x19]
    add     x19, x24, x0          // pc = bc_start + target
    b       vm_loop

op_loop:
    // Payload: [count:4]
    ldr     w0, [x19]             // iteration count
    add     x19, x19, #4

    // Cap at 1024 (QT2: ∞-1)
    cmp     w0, #1024
    csel    w0, w0, w27, lo       // w27 pre-loaded with 1024

    // Push loop frame: [return_pc][remaining]
    // Loop stack separate from value stack — use heap area
    // (Implementation detail: dedicate a fixed region of heap for loop stack)
    // ... store loop context ...
    b       vm_loop

op_call:
    // Payload: [name_len:1][name:N]
    ldrb    w0, [x19]             // name_len
    add     x19, x19, #1
    // x19 now points to name bytes

    // Check builtin table first
    mov     x1, x19                // name_ptr
    bl      builtin_lookup         // → x0 = handler address, or 0
    add     x19, x19, x0           // advance pc past name (w0 reloaded)
    cbz     x0, call_external      // not builtin → external call
    br      x0                     // jump to builtin handler

call_external:
    // Emit LookupAlias event (write to output buffer)
    // ... implementation depends on event system ...
    b       vm_loop

op_ret:
    // Pop scope, return to caller
    // Restore pc from call stack
    // ... implementation matches scope management ...
    b       vm_loop

op_halt:
vm_halt:
    // Exit cleanly
    mov     x0, #0                 // exit code 0
    mov     x8, #93                // __NR_exit (Linux ARM64)
    svc     #0

op_nop:
    b       vm_loop

op_scope_begin:
    // Push new scope frame onto locals area
    // ... increment scope depth, save locals_ptr ...
    b       vm_loop

op_scope_end:
    // Pop scope frame, restore locals_ptr
    // Check loop stack — if in loop, jump back
    // ... decrement scope depth ...
    b       vm_loop
```

### Task 5: Syscall Bridge (~100 LOC)

```asm
// Linux ARM64 syscall convention:
//   x8 = syscall number
//   x0-x5 = arguments
//   svc #0
//   x0 = return value (negative = -errno)

// Linux ARM64 syscall numbers (khác x86_64!):
//   read    = 63
//   write   = 64
//   openat  = 56
//   close   = 57
//   fstat   = 80
//   mmap    = 222
//   munmap  = 215
//   exit    = 93

sys_write:
    // x0 = fd, x1 = buf, x2 = count
    mov     x8, #64                // __NR_write
    svc     #0
    ret

sys_read:
    // x0 = fd, x1 = buf, x2 = count
    mov     x8, #63                // __NR_read
    svc     #0
    ret

sys_openat:
    // x0 = dirfd, x1 = pathname, x2 = flags, x3 = mode
    mov     x8, #56
    svc     #0
    ret

sys_close:
    // x0 = fd
    mov     x8, #57
    svc     #0
    ret

sys_mmap:
    // x0=addr, x1=len, x2=prot, x3=flags, x4=fd, x5=offset
    mov     x8, #222
    svc     #0
    ret

sys_exit:
    mov     x8, #93
    svc     #0
    // never returns

op_emit:
    // Pop chain → write to stdout
    bl      stack_check_underflow
    sub     x20, x20, #16
    ldp     x0, x1, [x20]         // hash, chain_ptr
    // Serialize chain to text (or raw bytes)
    // Write to stdout (fd=1)
    mov     x0, #1                 // fd = stdout
    // x1 = buf (chain bytes), x2 = len
    bl      sys_write
    b       vm_loop
```

### Task 6: Math Operations — NEON FPU (~200 LOC)

```asm
// ARM64 dùng NEON/FP registers (d0-d31) cho floating point
// KHÔNG cần x87 hay SSE — hardware FP native

f64_add:
    // Pop 2 f64 từ VM stack, decode, add, encode, push
    // ... decode chain → f64 (xem encode_f64_to_chain) ...
    fadd    d0, d0, d1             // d0 = a + b
    bl      encode_f64_to_chain
    // push result
    b       vm_loop

f64_sub:
    fsub    d0, d0, d1
    // ...

f64_mul:
    fmul    d0, d0, d1
    // ...

f64_div:
    // Check division by zero
    fcmp    d1, #0.0
    b.eq    err_division_by_zero
    fdiv    d0, d0, d1
    // ...

f64_sqrt:
    fsqrt   d0, d0
    // ...

// Sin/Cos — dùng hardware nếu có, Taylor nếu không
// ARM64 KHÔNG có fsin/fcos instruction → Taylor series
// Hoặc: dùng polynomial approximation (Cody-Waite range reduction)
f64_sin:
    // Range reduction: x → [-π, π]
    // Minimax polynomial (7 terms, đủ cho double precision)
    // sin(x) ≈ x - x³/6 + x⁵/120 - x⁷/5040 + ...
    // ... ~30 instructions ...
    ret

f64_cos:
    // cos(x) = sin(x + π/2)
    // ... hoặc polynomial riêng ...
    ret
```

### Task 7: String Operations (~150 LOC)

```asm
// Strings in HomeOS = molecule chains (1 byte per molecule)
// Shape=0x02, Relation=0x01, Valence=byte_value

str_len:
    // Input: chain_ptr → count molecules with shape=0x02
    ldrb    w0, [x1]              // mol_count
    // Return count as PushNum
    ret

str_cmp:
    // Compare 2 string chains byte-by-byte
    // Return: 0 (equal), -1 (less), 1 (greater)
    // ... loop comparing valence bytes ...
    ret

str_concat:
    // Allocate new chain = chain_a + chain_b
    // Copy molecules from both chains
    // Recompute hash
    ret

str_hash:
    // FNV-1a of valence bytes only (text content)
    ret
```

### Task 8: Chain Operations (~300 LOC)

```asm
// FNV-1a Hash (64-bit)
// Constants: offset=0xcbf29ce484222325, prime=0x100000001b3

fnv1a_hash:
    // x0 = data ptr, x1 = data len → x0 = hash
    ldr     x2, =0xcbf29ce484222325   // FNV offset basis
    ldr     x3, =0x100000001b3         // FNV prime
fnv1a_loop:
    cbz     x1, fnv1a_done
    ldrb    w4, [x0], #1              // byte = *data++
    eor     x2, x2, x4                // hash ^= byte
    mul     x2, x2, x3                // hash *= prime
    sub     x1, x1, #1
    b       fnv1a_loop
fnv1a_done:
    mov     x0, x2                     // return hash
    ret

// chain_lca — Lowest Common Ancestor
// Hot path! ~30 instructions cho simple case
chain_lca:
    // Input: 2 chain_ptrs trên stack
    // Output: 1 LCA chain
    //
    // Algorithm (simplified for ASM):
    //   1. min_len = min(len_a, len_b)
    //   2. Per molecule position:
    //      result[i] = weighted_avg(a[i], b[i]) per dimension
    //   3. Hash result chain
    //
    // Full LCA (mode detection, variance) = too complex for ASM
    // → Giữ core weighted average, defer full LCA to bytecode
    ret

// mol_encode: raw 5 bytes → tagged format
mol_encode:
    // Check defaults, build presence mask
    // ... compact encoding ...
    ret

// mol_decode: tagged bytes → 5 raw bytes
mol_decode:
    // Read presence mask, fill defaults
    ret
```

### Task 9: SHA-256 (~300 LOC)

```asm
// SHA-256 for chain integrity + QR verification
// ARM64 có crypto extensions (ARMv8.0-A optional, ARMv8.2+ mandatory)
// Detect via HWCAP → dùng sha256h/sha256h2 nếu có, software fallback nếu không

sha256_block:
    // Input: x0 = state[8] ptr, x1 = data ptr, x2 = block count

    // Check ARMv8-CE availability
    // mrs x3, ID_AA64ISAR0_EL1 → check SHA2 field
    // Nếu có:
    //   sha256h  q0, q1, v2.4s    // hardware acceleration
    //   sha256h2 q0, q1, v2.4s
    //   sha256su0 v0.4s, v1.4s
    //   sha256su1 v0.4s, v1.4s, v2.4s
    // Nếu không:
    //   Software fallback (~200 instructions)

sha256_software:
    // Standard SHA-256 compression function
    // 64 rounds, K constants from .rodata
    // ... ~200 instructions ...
    ret

.section .rodata
sha256_k:
    .word 0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5
    .word 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5
    // ... 60 more words ...
```

### Task 10: Builtin Dispatch (~400 LOC)

```asm
// 60+ builtins routed by name hash
// Pre-compute FNV-1a of builtin names → lookup table

builtin_lookup:
    // x0 = name_len, x1 = name_ptr → x0 = handler address (0 if not found)
    // Hash the name
    mov     x0, x1
    // x1 already has len... need to reload
    bl      fnv1a_hash             // x0 = name hash

    // Binary search or hash table lookup
    adr     x1, builtin_table
    // ... search for matching hash ...
    // Return handler address in x0
    ret

.section .rodata
.align 3
builtin_table:
    // [hash:8][handler_addr:8] pairs, sorted by hash
    // Pre-computed at assembly time
    .quad   0x..., __hyp_add
    .quad   0x..., __hyp_sub
    .quad   0x..., __hyp_mul
    // ... 60+ entries ...
```

### Task 11: Error Handlers (~50 LOC)

```asm
err_stack_overflow:
    adr     x1, msg_stack_overflow
    mov     x2, #16
    mov     x0, #2                 // stderr
    bl      sys_write
    mov     x0, #1                 // exit code 1
    bl      sys_exit

err_stack_underflow:
    adr     x1, msg_stack_underflow
    mov     x2, #17
    mov     x0, #2
    bl      sys_write
    mov     x0, #1
    bl      sys_exit

err_division_by_zero:
    adr     x1, msg_div_zero
    mov     x2, #18
    mov     x0, #2
    bl      sys_write
    mov     x0, #1
    bl      sys_exit

op_unknown:
    adr     x1, msg_unknown_op
    mov     x2, #15
    mov     x0, #2
    bl      sys_write
    mov     x0, #1
    bl      sys_exit

.section .rodata
msg_stack_overflow:  .asciz "stack overflow\n"
msg_stack_underflow: .asciz "stack underflow\n"
msg_div_zero:        .asciz "division by zero\n"
msg_unknown_op:      .asciz "unknown opcode\n"
```

---

## Platform Variants

### Linux ARM64 (chính)

```
Syscall:  svc #0, x8=number
Binary:   ELF64 aarch64
Entry:    _start
Self-ref: /proc/self/exe

Syscall numbers (Linux ARM64):
  read=63, write=64, openat=56, close=57,
  fstat=80, mmap=222, munmap=215, exit=93,
  getpid=172, clock_gettime=113
```

### macOS ARM64 (Apple Silicon)

```
Syscall:  svc #0x80, x16=number (KHÁC Linux!)
Binary:   Mach-O arm64
Entry:    _main
Self-ref: _NSGetExecutablePath() hoặc argv[0]

Syscall numbers (macOS ARM64):
  read=3, write=4, open=5, close=6,
  mmap=197, munmap=73, exit=1

⚠ Khác biệt lớn:
  - Syscall register: x16 (không phải x8)
  - Syscall numbers khác hoàn toàn
  - Instruction: svc #0x80 (không phải svc #0)
  - Mach-O header thay vì ELF

→ Giải pháp: #ifdef hoặc 2 file riêng (vm_arm64_linux.S, vm_arm64_macos.S)
→ Hoặc: runtime detect qua header magic + conditional branch
```

### Android (Linux ARM64 + Bionic)

```
Cùng syscall convention với Linux ARM64.
Khác biệt:
  - /proc/self/exe có thể bị restricted
  - SELinux policy ảnh hưởng mmap executable
  - Cần dlopen() pattern thay vì bare ELF

→ Phase sau: wrapper qua JNI, không cần thay đổi VM core
```

---

## Tối ưu ARM64-specific

### 1. Jump table với TBZ/TBNZ

```asm
// ARM64 có test-bit-and-branch — nhanh cho boolean checks
tbnz    w0, #0, label     // branch if bit 0 set
tbz     w0, #7, label     // branch if bit 7 clear
```

### 2. Load/Store Pair (LDP/STP)

```asm
// ARM64 có paired load/store — 2 registers cùng lúc
ldp     x0, x1, [x20, #-16]   // load 16 bytes in 1 instruction
stp     x0, x1, [x20]         // store 16 bytes in 1 instruction
// → Stack push/pop = 1 instruction thay vì 2
```

### 3. Conditional Select (CSEL)

```asm
// Branchless conditionals — tránh branch misprediction
cmp     w0, #1024
mov     w1, #1024
csel    w0, w0, w1, lo     // w0 = (w0 < 1024) ? w0 : 1024
```

### 4. Hardware Crypto (nếu có)

```asm
// ARMv8 Crypto Extensions — 10-50× nhanh hơn software
// Detect via: mrs x0, ID_AA64ISAR0_EL1
sha256h   q0, q1, v2.4s    // SHA-256 hash update (4 rounds)
sha256su0 v0.4s, v1.4s     // schedule update part 1
aese      v0.16b, v1.16b   // AES single round encrypt
aesd      v0.16b, v1.16b   // AES single round decrypt
```

### 5. NEON SIMD (parallel chain ops)

```asm
// Process 4 molecules simultaneously (5 bytes each → pad to 8)
ld4     {v0.8b, v1.8b, v2.8b, v3.8b}, [x0]
// v0 = shapes, v1 = relations, v2 = valences, v3 = arousals
// → Parallel LCA computation trên 4 molecules
```

---

## Rào cản

| Rào cản | Giải pháp |
|---------|-----------|
| Linux vs macOS syscall khác hoàn toàn | Phase 1: Linux only. macOS = phase sau hoặc file riêng |
| Không có fsin/fcos instruction | Taylor polynomial (~30 instructions), hoặc lookup table |
| Crypto extensions optional | Runtime detect via HWCAP, software fallback |
| Full LCA quá phức tạp cho ASM | Core weighted avg trong ASM, full LCA defer to bytecode |
| 60+ builtins = nhiều code | Hash-based dispatch table, mỗi builtin ~10-20 instructions |
| Android SELinux restrictions | JNI wrapper, không ảnh hưởng VM core |
| Testing trên ARM64 hardware | QEMU user-mode emulation cho CI, real hardware cho perf test |

---

## Test Plan

### Test 1: Hello World

```
Bytecode: PushNum(0) → Emit → Halt
Expected: write "0" to stdout, exit 0
Run: qemu-aarch64 ./origin.olang.arm64
```

### Test 2: Math

```
Bytecode: PushNum(3) → PushNum(5) → Call("__hyp_add") → Emit → Halt
Expected: "8"
```

### Test 3: Jump

```
Bytecode: Jmp(offset_to_halt) → Emit(should skip) → Halt
Expected: no output, clean exit
```

### Test 4: Stack Overflow

```
Bytecode: 257× Push → should error
Expected: "stack overflow" to stderr, exit 1
```

### Test 5: Chain Operations

```
Bytecode: Push(chain_a) → Push(chain_b) → Lca → Emit → Halt
Expected: LCA result
```

### Test 6: Self-read

```
origin.olang ARM64 binary reads its own header, verifies magic bytes.
Tests: mmap self, parse header, correct section offsets.
```

### Cross-validation

```
Cùng bytecode chạy trên vm_x86_64 và vm_arm64:
  → Output phải GIỐNG NHAU
  → Hash results phải GIỐNG NHAU
  → FNV-1a deterministic trên cả 2 architectures
```

---

## Definition of Done

- [ ] `vm_arm64.S` tồn tại (~2000-3000 LOC)
- [ ] Entry point: mmap self, parse header, init stack+heap
- [ ] VM loop: fetch-decode-dispatch 38 opcodes
- [ ] Stack operations: push/pop/dup/swap (16-byte entries)
- [ ] Control flow: jmp/jz/loop/call/ret
- [ ] Math: f64 add/sub/mul/div/sqrt via NEON
- [ ] String: len/cmp/concat/hash
- [ ] Chain: fnv1a_hash, chain_lca (simplified)
- [ ] SHA-256: software + optional ARMv8-CE hardware path
- [ ] Syscall bridge: read/write/mmap/exit (Linux ARM64)
- [ ] 60+ builtins via hash dispatch table
- [ ] Error handling: overflow/underflow/div-zero/unknown-op
- [ ] Chạy trên QEMU aarch64: "Hello from ARM64 VM"
- [ ] Cross-validation: cùng bytecode → cùng output với x86_64
- [ ] Limits: max_steps=65536, max_stack=256, max_loop=1024

## Ước tính: 3-4 ngày (sau khi vm_x86_64.S ổn định)

---

## Phase 5+ Note: GPU/NPU Offload khi scale 100B nodes

```
HIỆN TẠI (Phase 1): CPU-only là ĐÚNG.
  - 5400 L0 nodes → mọi thứ sequential, microseconds
  - Dream cluster vài trăm observations → trivial
  - Silk walk vài nghìn edges → instant

TƯƠNG LAI (100B nodes = 3.3 TB knowledge):
  - Dream clustering O(N²) trên 1M+ observations → KHÔNG THỂ CPU-only
  - Silk graph traversal 100B edges → BFS/DFS = minutes
  - Batch LCA: cluster 1M nodes → 1M × 5D weighted avg
  - Batch FNV-1a: import 10M chains/sec pipeline
  - Similarity search: nearest neighbor trong 100B 5D points

COMPUTE LANDSCAPE trên ARM64:
  ┌─────────────────────────────────────────────────────────────┐
  │ Unit         Capability              Access                 │
  ├─────────────────────────────────────────────────────────────┤
  │ CPU (A78+)   General, sequential     Direct ASM (hiện tại)  │
  │ GPU (Mali/   Parallel f32/f16,       Vulkan Compute shader  │
  │      Adreno) 1000+ cores             OpenCL (deprecated)    │
  │ NPU (Hexagon Tensor ops, int8/int4   NNAPI / QNN / vendor   │
  │      /APU)   matrix multiply         SDK (fragmented)       │
  │ Apple GPU    Unified memory,         Metal Compute shader   │
  │              f32/f16, 128+ cores                            │
  │ Apple ANE    Neural Engine, int8     CoreML only (opaque)    │
  └─────────────────────────────────────────────────────────────┘

WORKLOAD → ACCELERATOR MAPPING:
  Workload                    Best fit     Why
  ──────────────────────────────────────────────────────────────
  Opcode dispatch             CPU          Sequential, branchy
  FNV-1a (single)             CPU          Sequential, few ops
  FNV-1a (batch 10M)          GPU          Embarrassingly parallel
  LCA (single)                CPU          5D avg, trivial
  LCA (batch 1M)              GPU          1M independent 5D avgs
  Dream clustering            GPU          Pairwise distance matrix
  Silk BFS/DFS                CPU+GPU      Graph traversal hybrid
  Similarity search (KNN)     GPU/NPU      5D nearest neighbor
  SHA-256 (batch verify)      GPU          Parallel blocks
  Molecule comparison (5D)    NPU          Low-precision vector ops
  ──────────────────────────────────────────────────────────────

THIẾT KẾ DỰ KIẾN (Phase 5+):
  1. VM thêm opcode: GpuSubmit(kernel_id, data_ptr, data_len)
     → Không thay đổi bytecode format — chỉ thêm tag mới
     → Kernel = pre-compiled compute shader, embed trong origin.olang

  2. Kernel types (ít, tập trung):
     KERNEL_BATCH_HASH      — N chains → N hashes (GPU)
     KERNEL_BATCH_LCA       — N×M chains → N LCA results (GPU)
     KERNEL_DISTANCE_MATRIX — N observations → N×N similarity (GPU)
     KERNEL_KNN_SEARCH      — query 5D → top-K nearest (GPU/NPU)
     KERNEL_SILK_BFS        — start node → reachable set (GPU)

  3. Memory model:
     CPU (origin.olang) ←→ GPU (device memory)
     ARM64: Unified Memory (Apple/Qualcomm) → zero-copy
     x86_64: PCIe transfer → explicit copy

  4. Fallback chain: NPU → GPU → SIMD (NEON) → scalar CPU
     Runtime detect capabilities → chọn path nhanh nhất
     HAL đã có: has_simd, has_crypto → thêm has_gpu, has_npu

  5. Nguyên tắc:
     ① GPU/NPU = ACCELERATOR, không phải REQUIREMENT
       origin.olang PHẢI chạy được trên CPU-only (phone cũ, RPi)
     ② Kết quả GPU == kết quả CPU (deterministic, bitwise)
       → f32 trên GPU vs f64 trên CPU = KHÔNG CHẤP NHẬN
       → Phải dùng f64 hoặc fixed-point đảm bảo consistency
     ③ Không vendor lock-in: Vulkan (cross-platform) > Metal/CUDA
       → Apple: Metal (bắt buộc) + Vulkan (MoltenVK fallback)
     ④ Kernel code embed trong origin.olang (self-contained)
       → Không download shader runtime
       → SPIR-V binary embed trong bytecode section

CON SỐ ƯỚC TÍNH (100B nodes):
  Operation              CPU (1 core)    GPU (Mali-G78)    Speedup
  ─────────────────────────────────────────────────────────────────
  1M FNV-1a hashes       50ms            0.5ms             100×
  1M LCA (5D avg)        200ms           2ms               100×
  Distance matrix 10K    500ms           5ms               100×
  KNN search (K=10)      1s              10ms              100×
  Silk BFS (depth=5)     2s              200ms             10×
  Dream cluster (1M obs) 30min           30s               60×
  ─────────────────────────────────────────────────────────────────

⚠️ KHÔNG IMPLEMENT BÂY GIỜ.
   Phase 1: CPU-only, correct, simple.
   Phase 5: Profile real workloads → thêm GPU kernel cho bottleneck.
   Thiết kế VM sao cho THÊM opcode = backward compatible.
```

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 1.2*
*Phụ thuộc: PLAN_1_1 (vm_x86_64.S), PLAN_0_5 (bytecode format)*
