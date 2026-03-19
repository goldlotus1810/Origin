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
    # ── 1. Open bytecode file ────────────────────────────────────────
    # Check argc: if argv[1] exists, open that file
    # Otherwise fall back to /proc/self/exe
    # Stack at _start: [argc][argv[0]][argv[1]]...
    mov     (%rsp), %rax                # argc
    cmp     $2, %rax
    jb      .open_self                  # no args → open self
    mov     16(%rsp), %rdi              # argv[1]
    jmp     .do_open
.open_self:
    lea     proc_self_exe(%rip), %rdi
.do_open:
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
    push    %rax                        # save bc_offset on CPU stack
    movl    18(%rsp), %eax              # bc_size (now at 18+8 because of push)
    # Fix: header is still on stack, bc_size at original offset 18
    # After push, stack is: [bc_offset][header...]
    # So bc_size is at 8+18(%rsp) = 26(%rsp)
    pop     %rax                        # get bc_offset back
    push    %rax                        # re-push it
    movl    26(%rsp), %ebx              # bc_size from header (offset 18 + 8 for pushed bc_offset)
    # Actually let's re-read: header starts at 8(%rsp) now
    # bc_offset is at 0(%rsp), header at 8(%rsp)
    # bc_size field = header+18 = 8+18(%rsp) = 26(%rsp)
    # This is getting complex. Simpler approach:
    pop     %rax                        # bc_offset → rax
    movl    18(%rsp), %ebx              # bc_size from header (stack still has header)
    add     $HEADER_SIZE, %rsp          # free header from stack
    # Now rax = bc_offset, rbx = bc_size
    # Save bc_offset on stack for later (mmap clobbers r8)
    push    %rax                        # save bc_offset

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
    # Recover bc_offset from stack
    pop     %rsi                        # bc_offset
    # lseek to bc_offset
    mov     %rbp, %rdi                  # fd
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

    # ── 4. Initialize variable table ─────────────────────────────────
    call    var_table_init

    # ── 5. Enter VM loop ─────────────────────────────────────────────
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
op_load_local:
    # [0x31/0x34, name_len_u8, name_bytes...]
    movzbl  (%r12, %r13), %ecx         # name_len
    inc     %r13
    lea     (%r12, %r13), %rsi         # name ptr in bytecode
    push    %rcx
    call    var_load                     # → rax=ptr, rdx=len
    pop     %rcx
    add     %rcx, %r13                  # PC += name_len
    sub     $16, %r14
    mov     %rax, (%r14)
    mov     %rdx, 8(%r14)
    jmp     vm_loop

op_store:
op_store_upd:
    # [0x33/0x39, name_len_u8, name_bytes...]
    movzbl  (%r12, %r13), %ecx         # name_len
    inc     %r13
    lea     (%r12, %r13), %rsi         # name ptr
    mov     (%r14), %r8                # value ptr
    mov     8(%r14), %r9               # value len
    add     $16, %r14                  # pop value
    push    %rcx
    call    var_store
    pop     %rcx
    add     %rcx, %r13                 # PC += name_len
    jmp     vm_loop

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
    lea     (%r12, %r13), %rsi         # name ptr

    # Check builtin names
    # __hyp_add (9 chars)
    cmp     $9, %ecx
    jne     .call_check_7
    push    %rcx
    lea     bname_hyp_add(%rip), %rdi
    mov     $9, %ecx
    push    %rsi
    repe    cmpsb
    pop     %rsi
    pop     %rcx
    je      .call_add
    push    %rcx
    lea     bname_hyp_sub(%rip), %rdi
    mov     %rsi, %rax
    mov     %rax, %rsi                  # restore rsi
    mov     $9, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_sub
    push    %rcx
    lea     bname_hyp_mul(%rip), %rdi
    mov     %rsi, %rax
    mov     $9, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_mul
    push    %rcx
    lea     bname_hyp_div(%rip), %rdi
    mov     %rsi, %rax
    mov     $9, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_div
    jmp     .call_check_7

.call_check_7:
    # __eq (4), __cmp_lt (8), __cmp_gt (8), __cmp_le (8), __cmp_ge (8)
    cmp     $4, %ecx
    jne     .call_check_8
    push    %rcx
    lea     bname_eq(%rip), %rdi
    mov     %rsi, %rax
    mov     $4, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_eq
    jmp     .call_skip

.call_check_8:
    cmp     $8, %ecx
    jne     .call_check_str
    push    %rcx
    lea     bname_cmp_lt(%rip), %rdi
    mov     %rsi, %rax
    mov     $8, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_lt
    push    %rcx
    lea     bname_cmp_gt(%rip), %rdi
    mov     %rsi, %rax
    mov     $8, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_gt
    push    %rcx
    lea     bname_cmp_le(%rip), %rdi
    mov     %rsi, %rax
    mov     $8, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_le
    push    %rcx
    lea     bname_cmp_ge(%rip), %rdi
    mov     %rsi, %rax
    mov     $8, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_ge
    jmp     .call_check_str

.call_check_str:
    # String builtins: __len (5), __concat (8), __char_at (9), __substr (8)
    cmp     $5, %ecx
    jne     .call_check_str8
    push    %rcx
    lea     bname_len(%rip), %rdi
    mov     %rsi, %rax
    mov     $5, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_len
    jmp     .call_skip

.call_check_str8:
    cmp     $8, %ecx
    jne     .call_check_str9
    push    %rcx
    lea     bname_concat(%rip), %rdi
    mov     %rsi, %rax
    mov     $8, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_concat
    push    %rcx
    lea     bname_substr(%rip), %rdi
    mov     %rsi, %rax
    mov     $8, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_substr
    jmp     .call_skip

.call_check_str9:
    cmp     $9, %ecx
    jne     .call_skip
    push    %rcx
    lea     bname_char_at(%rip), %rdi
    mov     %rsi, %rax
    mov     $9, %ecx
    repe    cmpsb
    mov     %rax, %rsi
    pop     %rcx
    je      .call_char_at
    # Fall through to skip

.call_skip:
    # Unknown function — skip
    add     %rcx, %r13                  # PC += name_len
    jmp     vm_loop

# ── Math builtins ────────────────────────────────────────────────────────

.call_add:
    add     %rcx, %r13
    movsd   (%r14), %xmm1              # b
    movsd   16(%r14), %xmm0            # a
    add     $16, %r14
    addsd   %xmm1, %xmm0
    movsd   %xmm0, (%r14)
    jmp     vm_loop

.call_sub:
    add     %rcx, %r13
    movsd   (%r14), %xmm1              # b
    movsd   16(%r14), %xmm0            # a
    add     $16, %r14
    subsd   %xmm1, %xmm0              # a - b
    movsd   %xmm0, (%r14)
    jmp     vm_loop

.call_mul:
    add     %rcx, %r13
    movsd   (%r14), %xmm1
    movsd   16(%r14), %xmm0
    add     $16, %r14
    mulsd   %xmm1, %xmm0
    movsd   %xmm0, (%r14)
    jmp     vm_loop

.call_div:
    add     %rcx, %r13
    movsd   (%r14), %xmm1              # b (divisor)
    xorpd   %xmm2, %xmm2
    ucomisd %xmm2, %xmm1
    je      vm_error_div_zero
    movsd   16(%r14), %xmm0            # a (dividend)
    add     $16, %r14
    divsd   %xmm1, %xmm0              # a / b
    movsd   %xmm0, (%r14)
    jmp     vm_loop

# ── Comparison builtins ──────────────────────────────────────────────────

.call_eq:
    add     %rcx, %r13
    movsd   (%r14), %xmm1
    movsd   16(%r14), %xmm0
    add     $16, %r14
    ucomisd %xmm1, %xmm0
    je      .cmp_true
    # eq false → push empty (0 len)
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

.call_lt:
    add     %rcx, %r13
    jmp     op_cmp_lt

.call_gt:
    add     %rcx, %r13
    jmp     op_cmp_gt

.call_le:
    add     %rcx, %r13
    jmp     op_cmp_le

.call_ge:
    add     %rcx, %r13
    jmp     op_cmp_ge

# ── String builtins ──────────────────────────────────────────────────────

.call_len:
    # Pop chain, push its length as f64
    add     %rcx, %r13
    mov     8(%r14), %rax              # len
    cmp     $F64_MARKER, %rax
    jne     .call_len_chain
    # f64 → len = 1? or 8? Let's say 1
    mov     $0x3FF0000000000000, %rax   # 1.0
    mov     %rax, (%r14)
    jmp     vm_loop
.call_len_chain:
    # Convert len to f64
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_concat:
    # Pop b, pop a, concatenate a+b on heap, push result
    add     %rcx, %r13
    mov     (%r14), %rsi               # b ptr
    mov     8(%r14), %rdx              # b len
    mov     16(%r14), %rdi             # a ptr
    mov     24(%r14), %rcx             # a len
    add     $32, %r14                  # pop 2

    # Copy a to heap
    mov     %r15, %rax                  # result ptr
    push    %rcx
    push    %rdx
    push    %rsi
    mov     %r15, %rdi                  # dest
    mov     16(%r14), %rsi             # Hmm, a ptr was popped. Use saved values
    # Actually let me redo: save before pop
    # This is getting complex. Simpler approach:
    pop     %rsi                        # b ptr
    pop     %rdx                        # b len
    pop     %rcx                        # a len

    # We already popped the stack. Need original pointers.
    # Let's re-read from where they were before pop.
    # a was at 16(%r14) before pop → now at -16(%r14)
    # Actually after add $32, r14 points past both entries.
    # a_ptr = -32(%r14), a_len = -24(%r14)
    # b_ptr = -16(%r14), b_len = -8(%r14)
    mov     -32(%r14), %rdi            # a ptr
    mov     -24(%r14), %rcx            # a len
    mov     -16(%r14), %rsi            # b ptr
    mov     -8(%r14), %rdx             # b len

    # Copy a
    mov     %r15, %rax                  # result start
    push    %rdx
    push    %rsi
    mov     %r15, %rdi
    mov     -32(%r14), %rsi            # a ptr
    mov     %rcx, %rdx
    mov     %ecx, %ecx
    rep     movsb
    mov     %rdi, %r15                  # bump

    # Copy b
    pop     %rsi                        # b ptr
    pop     %rdx                        # b len
    mov     %r15, %rdi
    mov     %edx, %ecx
    rep     movsb
    mov     %rdi, %r15                  # bump

    # Total len
    mov     -24(%r14), %rcx            # a len
    add     %rdx, %rcx                  # a+b len

    # Push result
    sub     $16, %r14
    mov     %rax, (%r14)               # ptr
    mov     %rcx, 8(%r14)              # len
    jmp     vm_loop

.call_char_at:
    # Pop index (f64), pop string, push char
    add     %rcx, %r13
    movsd   (%r14), %xmm0             # index
    cvttsd2si %xmm0, %rax             # index as int
    mov     16(%r14), %rsi            # string ptr
    mov     24(%r14), %rcx            # string len
    add     $32, %r14                 # pop 2

    # Bounds check
    cmp     %rcx, %rax
    jae     .char_at_empty

    # Get char (1 byte)
    movzbl  (%rsi, %rax), %edx
    movb    %dl, (%r15)
    sub     $16, %r14
    mov     %r15, (%r14)
    movq    $1, 8(%r14)
    inc     %r15
    jmp     vm_loop

.char_at_empty:
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

.call_substr:
    # Pop len (f64), pop start (f64), pop string → push substring
    add     %rcx, %r13
    movsd   (%r14), %xmm0             # sub_len
    cvttsd2si %xmm0, %rdx
    movsd   16(%r14), %xmm0           # start
    cvttsd2si %xmm0, %rax
    mov     32(%r14), %rsi            # string ptr
    mov     40(%r14), %rcx            # string len
    add     $48, %r14                 # pop 3

    # Bounds
    cmp     %rcx, %rax
    jae     .char_at_empty
    add     %rax, %rdx
    cmp     %rcx, %rdx
    jbe     .substr_ok
    mov     %rcx, %rdx
.substr_ok:
    sub     %rax, %rdx                 # actual length
    lea     (%rsi, %rax), %rsi        # start ptr

    # Copy to heap
    mov     %r15, %rax
    mov     %r15, %rdi
    mov     %edx, %ecx
    push    %rdx
    rep     movsb
    mov     %rdi, %r15
    pop     %rdx

    sub     $16, %r14
    mov     %rax, (%r14)
    mov     %rdx, 8(%r14)
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
    movq    %rsi, %xmm0                # rsi has the f64 bits from stack
    call    f64_to_ascii                # → eax = length, output at r15
    mov     %r15, %rsi                  # buf
    mov     %eax, %edx                  # len
    mov     $1, %edi                    # stdout
    push    %rax
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rax
    add     %rax, %r15                  # bump heap past formatted string
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
# Task 7: Math ops (SSE2 — all x86_64 CPUs have this)
# ═══════════════════════════════════════════════════════════════════════════

# Binary math ops: pop 2 f64, compute, push result
# Stack layout for f64: [f64_bits:8][F64_MARKER:8]

op_add:
    movsd   16(%r14), %xmm0            # B (deeper in stack)
    movsd   (%r14), %xmm1              # A (top)
    add     $16, %r14                   # pop 1
    addsd   %xmm1, %xmm0
    movsd   %xmm0, (%r14)
    jmp     vm_loop

op_sub:
    movsd   16(%r14), %xmm0            # B
    movsd   (%r14), %xmm1              # A
    add     $16, %r14
    subsd   %xmm1, %xmm0              # B - A
    movsd   %xmm0, (%r14)
    jmp     vm_loop

op_mul:
    movsd   16(%r14), %xmm0
    movsd   (%r14), %xmm1
    add     $16, %r14
    mulsd   %xmm1, %xmm0
    movsd   %xmm0, (%r14)
    jmp     vm_loop

op_div:
    movsd   (%r14), %xmm1              # A (divisor)
    # Check zero
    xorpd   %xmm2, %xmm2
    ucomisd %xmm2, %xmm1
    je      vm_error_div_zero
    movsd   16(%r14), %xmm0            # B (dividend)
    add     $16, %r14
    divsd   %xmm1, %xmm0              # B / A
    movsd   %xmm0, (%r14)
    jmp     vm_loop

op_mod:
    # f64 modulo: a - floor(a/b) * b
    movsd   (%r14), %xmm1              # A (divisor)
    xorpd   %xmm2, %xmm2
    ucomisd %xmm2, %xmm1
    je      vm_error_div_zero
    movsd   16(%r14), %xmm0            # B (dividend)
    add     $16, %r14
    movapd  %xmm0, %xmm3              # save B
    divsd   %xmm1, %xmm0              # B/A
    roundsd $1, %xmm0, %xmm0          # floor(B/A)  (SSE4.1)
    mulsd   %xmm1, %xmm0              # floor(B/A)*A
    subsd   %xmm0, %xmm3              # B - floor(B/A)*A
    movsd   %xmm3, (%r14)
    jmp     vm_loop

op_neg:
    # Negate top f64
    movsd   (%r14), %xmm0
    mov     $0x8000000000000000, %rax   # sign bit mask
    movq    %rax, %xmm1
    xorpd   %xmm1, %xmm0
    movsd   %xmm0, (%r14)
    jmp     vm_loop

# ── Comparison ops (return f64: 1.0 = true, 0.0 = false) ────────────────

op_cmp_eq:
    movsd   16(%r14), %xmm0            # B
    movsd   (%r14), %xmm1              # A
    add     $16, %r14
    ucomisd %xmm1, %xmm0
    je      .cmp_true
    jmp     .cmp_false

op_cmp_lt:
    movsd   16(%r14), %xmm0            # B
    movsd   (%r14), %xmm1              # A
    add     $16, %r14
    ucomisd %xmm1, %xmm0
    jb      .cmp_true
    jmp     .cmp_false

op_cmp_gt:
    movsd   16(%r14), %xmm0
    movsd   (%r14), %xmm1
    add     $16, %r14
    ucomisd %xmm1, %xmm0
    ja      .cmp_true
    jmp     .cmp_false

op_cmp_le:
    movsd   16(%r14), %xmm0
    movsd   (%r14), %xmm1
    add     $16, %r14
    ucomisd %xmm1, %xmm0
    jbe     .cmp_true
    jmp     .cmp_false

op_cmp_ge:
    movsd   16(%r14), %xmm0
    movsd   (%r14), %xmm1
    add     $16, %r14
    ucomisd %xmm1, %xmm0
    jae     .cmp_true
    jmp     .cmp_false

.cmp_true:
    mov     $0x3FF0000000000000, %rax   # 1.0 in f64
    mov     %rax, (%r14)
    jmp     vm_loop

.cmp_false:
    xor     %eax, %eax                  # 0.0 in f64
    mov     %rax, (%r14)
    jmp     vm_loop

vm_error_div_zero:
    lea     msg_divzero(%rip), %rsi
    mov     $16, %edx
    mov     $2, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     op_halt_err

# ═══════════════════════════════════════════════════════════════════════════
# f64 → ASCII decimal conversion
# ═══════════════════════════════════════════════════════════════════════════
# Input:  xmm0 = f64 value
# Output: writes ASCII to heap at r15, returns length in eax
# Clobbers: rax, rcx, rdx, rdi, xmm0-xmm3

f64_to_ascii:
    mov     %r15, %rdi                  # output buffer = heap

    # Handle negative
    xorpd   %xmm1, %xmm1
    ucomisd %xmm1, %xmm0
    jae     .f2a_positive
    movb    $'-', (%rdi)
    inc     %rdi
    # Negate
    mov     $0x8000000000000000, %rax
    movq    %rax, %xmm1
    xorpd   %xmm1, %xmm0

.f2a_positive:
    # Handle zero
    xorpd   %xmm1, %xmm1
    ucomisd %xmm1, %xmm0
    jne     .f2a_nonzero
    movb    $'0', (%rdi)
    inc     %rdi
    jmp     .f2a_done

.f2a_nonzero:
    # Split integer and fractional parts
    # integer = (u64)f64
    cvttsd2si %xmm0, %rax              # rax = integer part
    test    %rax, %rax
    js      .f2a_large                  # overflow → print "inf"

    # Convert integer part via stack (reversed digits)
    cvtsi2sd %rax, %xmm1               # xmm1 = (f64)integer
    subsd   %xmm1, %xmm0              # xmm0 = fractional part
    push    %rax                        # save for later

    # Integer digits
    mov     %rdi, %rcx                  # save start pos
    mov     %rsp, %rdx                  # save rsp
    # Push digits in reverse
    mov     (%rdx), %rax                # restore integer
    add     $8, %rsp                    # balance the push above
    test    %rax, %rax
    jnz     .f2a_int_digits
    movb    $'0', (%rdi)
    inc     %rdi
    jmp     .f2a_frac

.f2a_int_digits:
    # We need to output digits. Use a temp buffer on stack.
    sub     $24, %rsp                   # temp buffer for digits
    mov     %rsp, %rcx                  # digit buffer
    xor     %edx, %edx                  # digit count

.f2a_int_loop:
    test    %rax, %rax
    jz      .f2a_int_copy
    mov     $10, %r8
    xor     %edx, %edx
    push    %rdx
    cqo
    idiv    %r8                         # rax = quotient, rdx = remainder
    pop     %r8                         # just balance (we don't use r8 here)
    add     $'0', %dl
    movb    %dl, (%rcx)
    inc     %rcx
    jmp     .f2a_int_loop

.f2a_int_copy:
    # Copy digits in reverse (they're backwards in buffer)
.f2a_int_rev:
    dec     %rcx
    cmp     %rsp, %rcx
    jb      .f2a_int_rev_done
    movb    (%rcx), %al
    movb    %al, (%rdi)
    inc     %rdi
    jmp     .f2a_int_rev

.f2a_int_rev_done:
    add     $24, %rsp

.f2a_frac:
    # Fractional part: multiply by 10, take integer, repeat (6 digits)
    xorpd   %xmm1, %xmm1
    ucomisd %xmm1, %xmm0
    je      .f2a_done                   # no fractional part

    movb    $'.', (%rdi)
    inc     %rdi
    mov     $6, %ecx                    # 6 decimal places

    # Load 10.0
    mov     $0x4024000000000000, %rax   # 10.0 in f64
    movq    %rax, %xmm2

.f2a_frac_loop:
    mulsd   %xmm2, %xmm0              # frac *= 10
    cvttsd2si %xmm0, %rax             # digit = (int)frac
    cmp     $9, %rax
    ja      .f2a_frac_clamp
    jmp     .f2a_frac_store
.f2a_frac_clamp:
    mov     $9, %eax
.f2a_frac_store:
    add     $'0', %al
    movb    %al, (%rdi)
    inc     %rdi
    # frac -= digit
    sub     $'0', %al
    movzbl  %al, %eax
    cvtsi2sd %eax, %xmm1
    subsd   %xmm1, %xmm0
    dec     %ecx
    jnz     .f2a_frac_loop

    # Strip trailing zeros
.f2a_strip:
    dec     %rdi
    cmpb    $'0', (%rdi)
    je      .f2a_strip
    cmpb    $'.', (%rdi)
    je      .f2a_done                   # don't keep trailing dot
    inc     %rdi                        # keep last non-zero

.f2a_done:
    # Calculate length
    mov     %rdi, %rax
    sub     %r15, %rax                  # length
    ret

.f2a_large:
    # Number too large for i64 → print "inf"
    movb    $'i', (%rdi)
    movb    $'n', 1(%rdi)
    movb    $'f', 2(%rdi)
    add     $3, %rdi
    jmp     .f2a_done

# ═══════════════════════════════════════════════════════════════════════════
# Variable name table (simple linear search)
# ═══════════════════════════════════════════════════════════════════════════
#
# Layout on heap:
#   var_table_ptr → [count:8][entries...]
#   Each entry: [name_len:1][name:N][value_ptr:8][value_len:8]
#
# For standalone VM, we keep it simple: max 256 variables, linear scan.

.equ VAR_TABLE_MAX, 256

# Initialize variable table — called once from _start
# Uses r15 (heap bump), stores table base in var_table_base
var_table_init:
    mov     %r15, %rax
    lea     var_table_base(%rip), %rcx
    mov     %rax, (%rcx)               # save table base address
    movq    $0, (%r15)                  # count = 0
    add     $8, %r15                    # bump past count
    ret

# var_store: name at rsi (len in ecx), value ptr in r8, value len in r9
var_store:
    push    %rbx
    push    %r12
    push    %r13
    mov     %rsi, %r12                  # save name ptr
    mov     %ecx, %r13d                 # save name len

    # First check if variable exists (update)
    lea     var_table_base(%rip), %rax
    mov     (%rax), %rbx               # table base
    mov     (%rbx), %rax               # count
    test    %rax, %rax
    jz      .vs_new                     # empty table

    lea     8(%rbx), %rdi              # first entry
    mov     %rax, %rcx                  # counter

.vs_search:
    movzbl  (%rdi), %edx               # entry name_len
    cmp     %r13d, %edx
    jne     .vs_next
    # Compare names
    push    %rcx
    push    %rdi
    lea     1(%rdi), %rsi
    mov     %r12, %rdi
    mov     %r13d, %ecx
    repe    cmpsb
    pop     %rdi
    pop     %rcx
    jne     .vs_next
    # Found — update value
    movzbl  (%rdi), %edx               # name_len again
    lea     1(%rdi, %rdx), %rdi        # skip to value fields
    mov     %r8, (%rdi)                # value ptr
    mov     %r9, 8(%rdi)               # value len
    jmp     .vs_done

.vs_next:
    # Skip this entry: 1 + name_len + 16
    movzbl  (%rdi), %edx
    lea     17(%rdi, %rdx), %rdi
    dec     %rcx
    jnz     .vs_search

.vs_new:
    # Add new entry at end of table (on heap)
    # Entry at r15: [name_len:1][name:N][value_ptr:8][value_len:8]
    movb    %r13b, (%r15)              # name_len
    inc     %r15
    # Copy name
    mov     %r15, %rdi
    mov     %r12, %rsi
    mov     %r13d, %ecx
    rep     movsb
    mov     %rdi, %r15
    # Value
    mov     %r8, (%r15)
    mov     %r9, 8(%r15)
    add     $16, %r15
    # Increment count
    lea     var_table_base(%rip), %rax
    mov     (%rax), %rbx
    incq    (%rbx)

.vs_done:
    pop     %r13
    pop     %r12
    pop     %rbx
    ret

# var_load: name at rsi (len in ecx) → result ptr in rax, len in rdx
var_load:
    push    %rbx
    lea     var_table_base(%rip), %rax
    mov     (%rax), %rbx               # table base
    mov     (%rbx), %rax               # count
    test    %rax, %rax
    jz      .vl_not_found

    lea     8(%rbx), %rdi
    mov     %rax, %r8                   # counter

.vl_search:
    movzbl  (%rdi), %edx
    cmp     %ecx, %edx
    jne     .vl_skip
    # Compare names
    push    %rcx
    push    %rdi
    push    %rsi
    lea     1(%rdi), %rdi
    mov     %ecx, %ecx                  # clear upper
    push    %rcx
    repe    cmpsb
    pop     %rcx
    pop     %rsi
    pop     %rdi
    pop     %rcx
    jne     .vl_skip
    # Found
    movzbl  (%rdi), %edx
    lea     1(%rdi, %rdx), %rdi
    mov     (%rdi), %rax               # value ptr
    mov     8(%rdi), %rdx              # value len
    pop     %rbx
    ret

.vl_skip:
    movzbl  (%rdi), %edx
    lea     17(%rdi, %rdx), %rdi
    dec     %r8
    jnz     .vl_search

.vl_not_found:
    xor     %eax, %eax
    xor     %edx, %edx
    pop     %rbx
    ret

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

msg_divzero:
    .ascii "Division by zero!\n"

# ── Builtin function names ───────────────────────────────────────────────
bname_hyp_add:  .ascii "__hyp_add"
bname_hyp_sub:  .ascii "__hyp_sub"
bname_hyp_mul:  .ascii "__hyp_mul"
bname_hyp_div:  .ascii "__hyp_div"
bname_eq:       .ascii "__eq"
bname_cmp_lt:   .ascii "__cmp_lt"
bname_cmp_gt:   .ascii "__cmp_gt"
bname_cmp_le:   .ascii "__cmp_le"
bname_cmp_ge:   .ascii "__cmp_ge"
bname_len:      .ascii "__len"
bname_concat:   .ascii "__concat"
bname_char_at:  .ascii "__char_at"
bname_substr:   .ascii "__substr"

# ── BSS (writable, zero-initialized) ────────────────────────────────────
.section .bss
var_table_base:
    .space 8                            # pointer to variable table on heap
