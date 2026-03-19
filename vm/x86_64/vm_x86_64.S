# ═══════════════════════════════════════════════════════════════════════════
# origin.olang VM — x86_64 Linux (no libc)
# Author: Lyra (session 2pN6F)
# ═══════════════════════════════════════════════════════════════════════════
#
# Register conventions:
#   r12 = bytecode base pointer (immutable during execution)
#   r13 = program counter (offset into bytecode)
#   r14 = VM stack pointer (NOT CPU stack)
#   r15 = heap pointer (bump allocator)
#   rbx = bytecode size (bounds checking)
#   rbp = preserved (call frame base)
#   rsp = CPU stack (for CALL/RET, syscalls)
#
# VM stack layout: each entry = 16 bytes
#   [ptr:8][len:8] — points to chain/string data on heap
#   For f64: ptr stores raw f64 bits, len = 0xFFFFFFFF_FFFFFFFF (marker)
#
# Bytecode format: see crates/olang/src/exec/ir.rs
# ═══════════════════════════════════════════════════════════════════════════

.section .text
.globl _start

# ── Constants ────────────────────────────────────────────────────────────
.equ STACK_SIZE,    0x100000        # 1 MB VM stack
.equ HEAP_SIZE,     0x1000000       # 16 MB heap arena
.equ HEADER_SIZE,   32
.equ F64_MARKER,    -1              # len = 0xFFFFFFFFFFFFFFFF means f64

# ── Syscall numbers ─────────────────────────────────────────────────────
.equ SYS_READ,      0
.equ SYS_WRITE,     1
.equ SYS_OPEN,      2
.equ SYS_CLOSE,     3
.equ SYS_MMAP,      9
.equ SYS_MUNMAP,    11
.equ SYS_EXIT,      60

# ── File open flags ─────────────────────────────────────────────────────
.equ O_RDONLY,       0
.equ PROT_RW,       3               # PROT_READ | PROT_WRITE
.equ MAP_PRIV_ANON, 0x22            # MAP_PRIVATE | MAP_ANONYMOUS

# ═══════════════════════════════════════════════════════════════════════════
# Task 1: Entry point + memory setup
# ═══════════════════════════════════════════════════════════════════════════

_start:
    # ── 1. Read /proc/self/exe path to find ourselves ────────────────
    # Open /proc/self/exe
    lea     proc_self_exe(%rip), %rdi
    mov     $O_RDONLY, %esi
    xor     %edx, %edx
    mov     $SYS_OPEN, %eax
    syscall
    test    %rax, %rax
    js      err_open                    # negative = error
    mov     %rax, %rbp                  # rbp = fd (temporarily)

    # Read header (32 bytes) onto CPU stack
    sub     $HEADER_SIZE, %rsp
    mov     %rbp, %rdi                  # fd
    mov     %rsp, %rsi                  # buf = stack
    mov     $HEADER_SIZE, %edx          # count
    mov     $SYS_READ, %eax
    syscall
    cmp     $HEADER_SIZE, %rax
    jne     err_header

    # Parse header fields
    # offset  0: magic "○LNG" (4 bytes) — skip validation for now
    # offset  4: version (1 byte)
    # offset  5: arch (1 byte) — should be 0x01 for x86_64
    # offset  6: vm_offset (4 bytes LE)
    # offset 10: vm_size (4 bytes LE)
    # offset 14: bc_offset (4 bytes LE)
    # offset 18: bc_size (4 bytes LE)
    # offset 22: kn_offset (4 bytes LE)
    # offset 26: kn_size (4 bytes LE)
    # offset 30: flags (2 bytes LE)

    movl    14(%rsp), %eax              # bc_offset
    mov     %rax, %r8                   # r8 = bc_offset (saved)
    movl    18(%rsp), %eax              # bc_size
    mov     %rax, %rbx                  # rbx = bc_size (bounds)
    add     $HEADER_SIZE, %rsp          # restore stack

    # ── 2. mmap the bytecode section ─────────────────────────────────
    # We'll read the whole bytecode into heap memory
    # First, allocate VM stack
    mov     $SYS_MMAP, %eax
    xor     %edi, %edi                  # addr = NULL
    mov     $STACK_SIZE, %esi           # length = 1 MB
    mov     $PROT_RW, %edx             # PROT_READ | PROT_WRITE
    mov     $MAP_PRIV_ANON, %r10d      # MAP_PRIVATE | MAP_ANONYMOUS
    mov     $-1, %r8d                   # fd = -1
    xor     %r9d, %r9d                  # offset = 0
    syscall
    test    %rax, %rax
    js      err_mmap
    mov     %rax, %r14                  # r14 = VM stack base
    add     $STACK_SIZE, %r14           # stack grows downward

    # Allocate heap
    mov     $SYS_MMAP, %eax
    xor     %edi, %edi
    mov     $HEAP_SIZE, %esi            # 16 MB
    mov     $PROT_RW, %edx
    mov     $MAP_PRIV_ANON, %r10d
    mov     $-1, %r8d
    xor     %r9d, %r9d
    syscall
    test    %rax, %rax
    js      err_mmap
    mov     %rax, %r15                  # r15 = heap base (bump allocator)

    # ── 3. Read bytecode from file into heap ─────────────────────────
    # lseek to bc_offset
    mov     %rbp, %rdi                  # fd
    mov     %r8, %rsi                   # offset = bc_offset
    xor     %edx, %edx                  # SEEK_SET = 0
    mov     $8, %eax                    # sys_lseek
    syscall

    # Read bytecode into start of heap
    mov     %rbp, %rdi                  # fd
    mov     %r15, %rsi                  # buf = heap base
    mov     %rbx, %rdx                  # count = bc_size
    mov     $SYS_READ, %eax
    syscall
    cmp     %rbx, %rax
    jne     err_read_bc

    # Setup VM registers
    mov     %r15, %r12                  # r12 = bytecode base
    xor     %r13d, %r13d               # r13 = PC = 0
    add     %rbx, %r15                  # bump heap past bytecode

    # Close file
    mov     %rbp, %rdi
    mov     $SYS_CLOSE, %eax
    syscall

    # ── 4. Enter VM loop ─────────────────────────────────────────────
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Task 2: VM dispatch loop
# ═══════════════════════════════════════════════════════════════════════════

vm_loop:
    # Bounds check
    cmp     %rbx, %r13
    jae     op_halt                     # PC >= bc_size → done

    # Fetch opcode
    movzbl  (%r12, %r13), %eax         # al = bytecode[PC]
    inc     %r13                        # PC++

    # Dispatch via jump table
    # Opcodes are sparse (0x00-0x09, 0x0A-0x0F, 0x10-0x14,
    #                      0x20-0x22, 0x30-0x39, 0x40-0x42,
    #                      0x50-0x53, 0x60-0x61, 0x70-0x71, 0x80-0x83)
    # Use cascading comparisons for sparse ranges

    # Range 0x00-0x0F: basic ops
    cmp     $0x10, %al
    jb      dispatch_basic

    # Range 0x10-0x14: relation ops
    cmp     $0x15, %al
    jb      dispatch_relation

    # Range 0x20-0x22: control flow
    cmp     $0x20, %al
    jb      op_unknown
    cmp     $0x23, %al
    jb      dispatch_control

    # Range 0x30-0x39: data ops
    cmp     $0x30, %al
    jb      op_unknown
    cmp     $0x3A, %al
    jb      dispatch_data

    # Range 0x40-0x42: device ops (stub)
    cmp     $0x40, %al
    jb      op_unknown
    cmp     $0x43, %al
    jb      op_nop                      # device ops = nop in standalone VM

    # Range 0x50-0x53: file ops
    cmp     $0x50, %al
    jb      op_unknown
    cmp     $0x54, %al
    jb      dispatch_file

    # Everything else
    jmp     op_unknown

# ── Dispatch sub-tables ──────────────────────────────────────────────────

dispatch_basic:
    lea     basic_table(%rip), %rcx
    movzbl  %al, %eax
    movslq  (%rcx, %rax, 4), %rdx
    add     %rcx, %rdx
    jmp     *%rdx

.section .rodata
.align 4
basic_table:
    .long op_nop        - basic_table   # 0x00 NOP
    .long op_lca        - basic_table   # 0x01 LCA
    .long op_emit       - basic_table   # 0x02 EMIT
    .long op_ret        - basic_table   # 0x03 RET
    .long op_dup        - basic_table   # 0x04 DUP
    .long op_pop        - basic_table   # 0x05 POP
    .long op_swap       - basic_table   # 0x06 SWAP
    .long op_halt       - basic_table   # 0x07 HALT
    .long op_nop        - basic_table   # 0x08 DREAM (stub)
    .long op_nop        - basic_table   # 0x09 STATS (stub)
    .long op_nop        - basic_table   # 0x0A FUSE (stub)
    .long op_nop        - basic_table   # 0x0B TRACE (stub)
    .long op_nop        - basic_table   # 0x0C INSPECT (stub)
    .long op_nop        - basic_table   # 0x0D ASSERT (stub)
    .long op_nop        - basic_table   # 0x0E TYPEOF (stub)
    .long op_nop        - basic_table   # 0x0F WHY (stub)
.section .text

dispatch_relation:
    sub     $0x10, %al
    cmp     $0x00, %al
    je      op_edge
    cmp     $0x01, %al
    je      op_query
    jmp     op_nop                      # EXPLAIN, SCOPE_BEGIN, SCOPE_END = nop

dispatch_control:
    sub     $0x20, %al
    cmp     $0x00, %al
    je      op_loop
    cmp     $0x01, %al
    je      op_jmp
    cmp     $0x02, %al
    je      op_jz
    jmp     op_unknown

dispatch_data:
    sub     $0x30, %al
    lea     data_table(%rip), %rcx
    movzbl  %al, %eax
    cmp     $0x0A, %al
    jae     op_unknown
    movslq  (%rcx, %rax, 4), %rdx
    add     %rcx, %rdx
    jmp     *%rdx

.section .rodata
.align 4
data_table:
    .long op_push       - data_table    # 0x30 PUSH
    .long op_load       - data_table    # 0x31 LOAD
    .long op_call       - data_table    # 0x32 CALL
    .long op_store      - data_table    # 0x33 STORE
    .long op_load_local - data_table    # 0x34 LOAD_LOCAL
    .long op_push_num   - data_table    # 0x35 PUSH_NUM
    .long op_push_mol   - data_table    # 0x36 PUSH_MOL
    .long op_try_begin  - data_table    # 0x37 TRY_BEGIN
    .long op_catch_end  - data_table    # 0x38 CATCH_END
    .long op_store_upd  - data_table    # 0x39 STORE_UPDATE
.section .text

dispatch_file:
    sub     $0x50, %al
    cmp     $0x00, %al
    je      op_nop                      # FFI = nop in standalone
    cmp     $0x01, %al
    je      op_file_read
    cmp     $0x02, %al
    je      op_file_write
    cmp     $0x03, %al
    je      op_file_append
    jmp     op_unknown

# ═══════════════════════════════════════════════════════════════════════════
# Task 3: Stack operations
# ═══════════════════════════════════════════════════════════════════════════

op_push:
    # [0x30, chain_len_u8, chain_bytes...]
    movzbl  (%r12, %r13), %ecx         # ecx = chain_len
    inc     %r13                        # PC past len byte

    # Copy chain bytes to heap
    mov     %r15, %rdi                  # dest = heap_ptr
    lea     (%r12, %r13), %rsi         # src = bytecode + PC
    mov     %ecx, %edx                  # count
    push    %rcx                        # save len
    call    memcpy_small
    pop     %rcx

    # Push (ptr, len) onto VM stack
    sub     $16, %r14
    mov     %r15, (%r14)                # ptr
    mov     %rcx, 8(%r14)               # len
    add     %rcx, %r15                  # bump heap
    add     %rcx, %r13                  # PC += chain_len
    jmp     vm_loop

op_push_num:
    # [0x35, f64_le_bytes[0-7]]
    sub     $16, %r14
    mov     (%r12, %r13), %rax
    mov     %rax, (%r14)                # store f64 bits
    movq    $F64_MARKER, 8(%r14)        # len = marker (this is f64)
    add     $8, %r13                    # PC += 8
    jmp     vm_loop

op_push_mol:
    # [0x36, S, R, V, A, T] — 5 bytes
    mov     %r15, %rdi                  # dest = heap
    lea     (%r12, %r13), %rsi         # src
    mov     $5, %edx
    call    memcpy_small

    sub     $16, %r14
    mov     %r15, (%r14)                # ptr to 5 bytes on heap
    movq    $5, 8(%r14)                 # len = 5
    add     $5, %r15                    # bump heap
    add     $5, %r13                    # PC += 5
    jmp     vm_loop

op_dup:
    mov     (%r14), %rax
    mov     8(%r14), %rcx
    sub     $16, %r14
    mov     %rax, (%r14)
    mov     %rcx, 8(%r14)
    jmp     vm_loop

op_pop:
    add     $16, %r14
    jmp     vm_loop

op_swap:
    mov     (%r14), %rax
    mov     8(%r14), %rcx
    mov     16(%r14), %rdx
    mov     24(%r14), %rsi
    mov     %rdx, (%r14)
    mov     %rsi, 8(%r14)
    mov     %rax, 16(%r14)
    mov     %rcx, 24(%r14)
    jmp     vm_loop

op_load:
    # [0x31, name_len_u8, name_bytes...]
    # Variable lookup — for standalone VM, push empty
    movzbl  (%r12, %r13), %ecx         # name_len
    inc     %r13
    add     %rcx, %r13                  # skip name bytes
    # Push empty (null, 0)
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

op_load_local:
    # Same as load for now
    jmp     op_load

op_store:
    # [0x33, name_len_u8, name_bytes...]
    # Pop value and store — for standalone VM, just skip name + pop
    movzbl  (%r12, %r13), %ecx
    inc     %r13
    add     %rcx, %r13                  # skip name
    add     $16, %r14                   # pop
    jmp     vm_loop

op_store_upd:
    # Same as store for now
    jmp     op_store

# ═══════════════════════════════════════════════════════════════════════════
# Task 4: Control flow
# ═══════════════════════════════════════════════════════════════════════════

op_jmp:
    # [0x21, target_u32_le]
    movl    (%r12, %r13), %eax          # target
    mov     %eax, %r13d                 # PC = target
    jmp     vm_loop

op_jz:
    # [0x22, target_u32_le]
    movl    (%r12, %r13), %eax          # target
    add     $4, %r13                    # PC past target bytes

    # Check top of stack: is it zero/empty?
    mov     8(%r14), %rcx              # len
    mov     (%r14), %rdx               # ptr/value
    add     $16, %r14                  # pop

    # f64 check: if marker, compare f64 bits to 0.0
    cmp     $F64_MARKER, %rcx
    je      .jz_check_f64
    # chain check: len == 0 means empty/false
    test    %rcx, %rcx
    jz      .jz_take
    jmp     vm_loop
.jz_check_f64:
    test    %rdx, %rdx                  # f64 bits == 0?
    jz      .jz_take
    jmp     vm_loop
.jz_take:
    mov     %eax, %r13d                 # PC = target
    jmp     vm_loop

op_loop:
    # [0x20, n_u32_le] — loop n times (simplified: just read and skip)
    movl    (%r12, %r13), %eax
    add     $4, %r13
    # Stub: treat as nop (full loop needs loop counter in scope)
    jmp     vm_loop

op_call:
    # [0x32, name_len_u8, name_bytes...]
    movzbl  (%r12, %r13), %ecx         # name_len
    inc     %r13
    add     %rcx, %r13                  # skip name
    # Stub: no function table in standalone VM yet
    jmp     vm_loop

op_ret:
    # Return from call — pop return address from CPU stack
    # If no call was made, this is a no-op
    jmp     vm_loop

op_try_begin:
    # [0x37, target_u32_le]
    add     $4, %r13                    # skip catch target
    jmp     vm_loop

op_catch_end:
    # [0x38]
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Task 5: Syscall bridge + Emit
# ═══════════════════════════════════════════════════════════════════════════

op_emit:
    # Pop top → write to stdout
    mov     (%r14), %rsi               # buf = chain ptr
    mov     8(%r14), %rdx              # len
    add     $16, %r14                  # pop

    # If f64 marker, format number first
    cmp     $F64_MARKER, %rdx
    je      .emit_f64

    # If len == 0, skip
    test    %rdx, %rdx
    jz      vm_loop

    # Write chain bytes to stdout
    mov     $1, %edi                   # fd = stdout
    mov     $SYS_WRITE, %eax
    syscall
    jmp     vm_loop

.emit_f64:
    # Convert f64 to decimal string and print
    # For now, just print the raw 8 bytes (placeholder)
    # TODO: proper f64 → ASCII conversion
    lea     f64_placeholder(%rip), %rsi
    mov     $7, %edx                   # "[f64]\n" len
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     vm_loop

# ── Syscall wrappers ─────────────────────────────────────────────────────

sys_write:
    # rdi = fd, rsi = buf, rdx = len
    mov     $SYS_WRITE, %eax
    syscall
    ret

sys_read:
    # rdi = fd, rsi = buf, rdx = len
    mov     $SYS_READ, %eax
    syscall
    ret

sys_open:
    # rdi = path, rsi = flags, rdx = mode
    mov     $SYS_OPEN, %eax
    syscall
    ret

sys_close:
    mov     $SYS_CLOSE, %eax
    syscall
    ret

# ═══════════════════════════════════════════════════════════════════════════
# Task 6: LCA (5D weighted average)
# ═══════════════════════════════════════════════════════════════════════════

op_lca:
    # Pop 2 chains, compute per-dimension average of first molecule
    # Chain A: (%r14), Chain B: 16(%r14)
    mov     (%r14), %rsi               # chain A ptr
    mov     8(%r14), %rax              # chain A len
    mov     16(%r14), %rdi             # chain B ptr
    mov     24(%r14), %rcx             # chain B len
    add     $32, %r14                  # pop 2

    # Need at least 5 bytes each (one molecule)
    cmp     $5, %rax
    jb      .lca_empty
    cmp     $5, %rcx
    jb      .lca_empty

    # Allocate 5 bytes on heap for result
    mov     %r15, %rax                  # result ptr

    # Dim 0: Shape — (A[0] + B[0]) / 2
    movzbl  (%rsi), %ecx
    movzbl  (%rdi), %edx
    add     %edx, %ecx
    shr     $1, %ecx
    movb    %cl, (%r15)

    # Dim 1: Relation
    movzbl  1(%rsi), %ecx
    movzbl  1(%rdi), %edx
    add     %edx, %ecx
    shr     $1, %ecx
    movb    %cl, 1(%r15)

    # Dim 2: Valence
    movzbl  2(%rsi), %ecx
    movzbl  2(%rdi), %edx
    add     %edx, %ecx
    shr     $1, %ecx
    movb    %cl, 2(%r15)

    # Dim 3: Arousal
    movzbl  3(%rsi), %ecx
    movzbl  3(%rdi), %edx
    add     %edx, %ecx
    shr     $1, %ecx
    movb    %cl, 3(%r15)

    # Dim 4: Time
    movzbl  4(%rsi), %ecx
    movzbl  4(%rdi), %edx
    add     %edx, %ecx
    shr     $1, %ecx
    movb    %cl, 4(%r15)

    # Push result
    sub     $16, %r14
    mov     %rax, (%r14)                # ptr
    movq    $5, 8(%r14)                 # len = 5
    add     $5, %r15                    # bump heap
    jmp     vm_loop

.lca_empty:
    # Push empty
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Task 7: Edge + Query (relation ops)
# ═══════════════════════════════════════════════════════════════════════════

op_edge:
    # [0x10, relation_byte] — create edge between top 2 stack items
    movzbl  (%r12, %r13), %eax         # relation byte
    inc     %r13
    # Stub: pop 2, push empty (no graph in standalone VM)
    add     $32, %r14
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

op_query:
    # [0x11, relation_byte]
    movzbl  (%r12, %r13), %eax
    inc     %r13
    # Stub: pop 1, push empty
    add     $16, %r14
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# File I/O operations (standalone)
# ═══════════════════════════════════════════════════════════════════════════

op_file_read:
    # Pop filename from stack, read file, push contents
    # Stub: push empty
    add     $16, %r14                   # pop filename
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

op_file_write:
op_file_append:
    # Pop value, pop filename, write
    # Stub: pop 2
    add     $32, %r14
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Utilities
# ═══════════════════════════════════════════════════════════════════════════

# memcpy_small: rdi=dest, rsi=src, edx=count (< 256)
memcpy_small:
    mov     %edx, %ecx
    rep     movsb
    ret

# ═══════════════════════════════════════════════════════════════════════════
# Error handlers
# ═══════════════════════════════════════════════════════════════════════════

op_unknown:
    lea     msg_unknown(%rip), %rsi
    mov     $18, %edx
    mov     $2, %edi                    # stderr
    mov     $SYS_WRITE, %eax
    syscall
    jmp     op_halt_err

op_nop:
    jmp     vm_loop

err_open:
    lea     msg_open(%rip), %rsi
    mov     $22, %edx
    mov     $2, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     op_halt_err

err_header:
    lea     msg_header(%rip), %rsi
    mov     $20, %edx
    mov     $2, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     op_halt_err

err_mmap:
    lea     msg_mmap(%rip), %rsi
    mov     $13, %edx
    mov     $2, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     op_halt_err

err_read_bc:
    lea     msg_readbc(%rip), %rsi
    mov     $21, %edx
    mov     $2, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     op_halt_err

# ═══════════════════════════════════════════════════════════════════════════
# Exit
# ═══════════════════════════════════════════════════════════════════════════

op_halt:
    mov     $SYS_EXIT, %eax
    xor     %edi, %edi                  # exit(0)
    syscall

op_halt_err:
    mov     $SYS_EXIT, %eax
    mov     $1, %edi                    # exit(1)
    syscall

# ═══════════════════════════════════════════════════════════════════════════
# Read-only data
# ═══════════════════════════════════════════════════════════════════════════

.section .rodata

proc_self_exe:
    .asciz "/proc/self/exe"

msg_unknown:
    .ascii "Unknown opcode!\n"
    .byte 0, 0

msg_open:
    .ascii "Cannot open binary!\n"
    .byte 0, 0

msg_header:
    .ascii "Bad header read!\n"
    .byte 0, 0

msg_mmap:
    .ascii "mmap failed!\n"

msg_readbc:
    .ascii "Bytecode read fail!\n"
    .byte 0

f64_placeholder:
    .ascii "[f64]\n"
    .byte 0
