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
.equ HEAP_SIZE,     0x40000000      # 1 GB heap arena
.equ HEADER_SIZE,   32
.equ ELF_HEADER_SIZE, 120           # ELF64 header + 1 program header
.equ F64_MARKER,    -1              # len = 0xFFFFFFFFFFFFFFFF means f64
.equ ARRAY_MARKER,  -3              # len = 0xFFFFFFFFFFFFFFFD means array
.equ DICT_MARKER,   -4              # len = 0xFFFFFFFFFFFFFFFC means dict
# Note: CLOSURE_MARKER = -2 is defined near cg_closure

# Array heap layout: [count:8][elem_0_ptr:8][elem_0_len:8]...
# When array is relocated by push: old location gets [FORWARD_MARKER:8][new_base:8]
# Dict heap layout:  [count:8][entry: key_ptr:8 key_len:8 val_ptr:8 val_len:8]...
.equ ARRAY_INIT_CAP, 512
.equ ARRAY_FORWARD_MARKER, -2      # sentinel: array relocated

# ── Syscall numbers ─────────────────────────────────────────────────────
.equ SYS_READ,      0
.equ SYS_WRITE,     1
.equ SYS_OPEN,      2
.equ SYS_CLOSE,     3
.equ SYS_MMAP,      9
.equ SYS_MUNMAP,    11
.equ SYS_NANOSLEEP, 35
.equ SYS_CLOCK_GETTIME, 228
.equ SYS_EXIT,      60
.equ CLOCK_MONOTONIC, 1

# ── File open flags ─────────────────────────────────────────────────────
.equ O_RDONLY,       0
.equ PROT_RW,       3               # PROT_READ | PROT_WRITE
.equ MAP_PRIV_ANON, 0x22            # MAP_PRIVATE | MAP_ANONYMOUS

# ═══════════════════════════════════════════════════════════════════════════
# Task 1: Entry point + memory setup
# ═══════════════════════════════════════════════════════════════════════════

_start:
    # ── 1. Open bytecode file ────────────────────────────────────────
    # Check argc: if argv[1] exists, check for --eval or open that file
    # Otherwise fall back to /proc/self/exe
    # Stack at _start: [argc][argv[0]][argv[1]]...
    mov     (%rsp), %rax                # argc
    cmp     $2, %rax
    jb      .open_self                  # no args → open self

    # Check if argv[1] == "--eval" (bytes: 2d 2d 65 76 61 6c 00)
    mov     16(%rsp), %rdi              # argv[1]
    # Compare first 4 bytes: "--ev" = 0x76652d2d (little-endian)
    cmpl    $0x76652d2d, (%rdi)         # "--ev"
    jne     .not_eval
    # Compare next 2 bytes: "al" = 0x6c61
    movzwl  4(%rdi), %eax
    cmp     $0x6c61, %eax               # "al" little-endian
    jne     .not_eval
    # Check null terminator
    cmpb    $0, 6(%rdi)
    jne     .not_eval

    # --eval mode detected: set flag, use argv[2] or /proc/self/exe
    movb    $1, eval_mode(%rip)
    cmpq    $3, (%rsp)                  # argc >= 3? (argv[2] = file)
    jb      .open_self
    mov     24(%rsp), %rdi              # argv[2]
    jmp     .do_open

.not_eval:
    mov     16(%rsp), %rdi              # argv[1] = filename
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

    # Check magic: if ELF (7f 45 4c 46), find origin header via trailer
    cmpl    $0x464c457f, (%rsp)         # "\x7fELF" in little-endian
    jne     .header_ok                  # not ELF → already origin header

    # ELF detected → read 8-byte trailer from end of file
    # lseek(fd, -8, SEEK_END) to find header_offset
    mov     %rbp, %rdi                  # fd
    mov     $-8, %rsi                   # offset = -8
    mov     $2, %edx                    # SEEK_END = 2
    mov     $8, %eax                    # sys_lseek
    syscall

    # Read 8 bytes (header_offset u64 LE) into stack buffer
    mov     %rbp, %rdi                  # fd
    mov     %rsp, %rsi                  # reuse stack buf
    mov     $8, %edx                    # count = 8
    mov     $SYS_READ, %eax
    syscall

    # Get header_offset from buffer
    mov     (%rsp), %rsi               # header_offset (u64 LE)

    # lseek(fd, header_offset, SEEK_SET)
    mov     %rbp, %rdi                  # fd
    xor     %edx, %edx                  # SEEK_SET = 0
    mov     $8, %eax                    # sys_lseek
    syscall

    # Re-read 32 bytes origin header
    mov     %rbp, %rdi                  # fd
    mov     %rsp, %rsi                  # buf = stack (overwrite)
    mov     $HEADER_SIZE, %edx          # count
    mov     $SYS_READ, %eax
    syscall
    cmp     $HEADER_SIZE, %rax
    jne     err_header

.header_ok:
    # Parse header fields
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
    # Read flags (offset 30 in header) before freeing
    movzwl  30(%rsp), %ecx             # flags (u16 LE)
    lea     bc_format(%rip), %rdx
    test    $1, %ecx                    # bit 0 = codegen format?
    jz      .fmt_irrs
    movb    $1, (%rdx)
    jmp     .fmt_done
.fmt_irrs:
    movb    $0, (%rdx)
.fmt_done:
    add     $HEADER_SIZE, %rsp          # free header from stack

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
    # Save stack top for debug
    mov     %r14, (%rax)

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
    # Save boot bytecode base + size for nested calls
    lea     boot_bc_base(%rip), %rax
    mov     %r12, (%rax)
    lea     boot_bc_size(%rip), %rax
    mov     %rbx, (%rax)

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

    # Check format flag (set during header parse)
    lea     bc_format(%rip), %rcx
    cmpb    $1, (%rcx)
    je      dispatch_codegen

    # ── ir.rs format dispatch ──
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

    # 0x3A: CallBuiltin(u8) — skip builtin_id byte, treat as no-op
    # ASM VM delegates to boot bytecode functions instead.
    # Format: [0x3A][builtin_id:1] = 2 bytes
    cmp     $0x3A, %al
    jne     .Lnot_call_builtin
    inc     %r13                        # skip builtin_id byte
    jmp     vm_loop
.Lnot_call_builtin:

    # Range 0x40-0x42: device ops (stub)
    cmp     $0x40, %al
    jb      op_unknown
    cmp     $0x43, %al
    jb      op_nop

    # Range 0x50-0x53: file ops
    cmp     $0x50, %al
    jb      op_unknown
    cmp     $0x54, %al
    jb      dispatch_file

    # Everything else
    jmp     op_unknown

# ── codegen.ol format dispatch (tags 0x01-0x24) ─────────────────────────
dispatch_codegen:
    # 0x3A = CallBuiltin(id) — dispatch to builtin by ID
    cmp     $0x3A, %al
    jne     .cg_not_builtin
    movzbl  (%r12, %r13), %eax         # read builtin ID
    inc     %r13                        # skip ID byte
    # Dispatch by builtin ID (matches BID_* constants from ir.rs)
    cmp     $6, %al                     # BID_HYP_ADD
    je      .call_add
    cmp     $7, %al                     # BID_HYP_SUB
    je      .call_sub
    cmp     $8, %al                     # BID_HYP_MUL
    je      .call_mul
    cmp     $9, %al                     # BID_HYP_DIV
    je      .call_div
    cmp     $0, %al                     # BID_EQ
    je      .bid_eq
    cmp     $1, %al                     # BID_CMP_LT
    je      .bid_lt
    cmp     $2, %al                     # BID_CMP_GT
    je      .bid_gt
    cmp     $3, %al                     # BID_CMP_LE
    je      .bid_le
    cmp     $4, %al                     # BID_CMP_GE
    je      .bid_ge
    cmp     $5, %al                     # BID_CMP_NE
    je      .bid_ne
    cmp     $10, %al                    # BID_LOGIC_NOT
    je      .bid_logic_not
    cmp     $11, %al                    # BID_ASSERT_TRUTH
    je      .bid_eq
    # All others: pop 2 args, push empty result
    # (most builtins are binary ops: pop 2, push 1)
    add     $16, %r14                   # pop arg 2
    # Replace top (arg 1) with empty
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

.bid_eq:
    xor     %ecx, %ecx
    jmp     .call_eq
.bid_lt:
    xor     %ecx, %ecx
    jmp     .call_lt
.bid_gt:
    xor     %ecx, %ecx
    jmp     .call_gt
.bid_le:
    xor     %ecx, %ecx
    jmp     .call_le
.bid_ge:
    xor     %ecx, %ecx
    jmp     .call_ge
.bid_ne:
    xor     %ecx, %ecx
    jmp     .call_ne
.bid_logic_not:
    # Invert: empty → 1.0, non-empty → empty
    mov     8(%r14), %rax
    test    %rax, %rax
    jz      .bid_not_was_empty
    # Was non-empty → make empty
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop
.bid_not_was_empty:
    # Was empty → make 1.0
    mov     $0x3FF0000000000000, %rax   # 1.0 as f64
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_is_keyword:
    # __str_is_keyword: pop string, check against keyword list
    # For now: push false (not a keyword) — stub
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

.cg_not_builtin:
    lea     codegen_table(%rip), %rcx
    movzbl  %al, %eax
    cmp     $0x26, %al
    jae     op_unknown
    movslq  (%rcx, %rax, 4), %rdx
    add     %rcx, %rdx
    jmp     *%rdx

.section .rodata
.align 4
codegen_table:
    .long op_unknown    - codegen_table # 0x00 (unused)
    .long cg_push       - codegen_table # 0x01 Push (name string, u16 len)
    .long op_load       - codegen_table # 0x02 Load
    .long op_lca        - codegen_table # 0x03 Lca
    .long op_edge       - codegen_table # 0x04 Edge
    .long op_query      - codegen_table # 0x05 Query
    .long op_emit       - codegen_table # 0x06 Emit
    .long op_call       - codegen_table # 0x07 Call
    .long op_ret        - codegen_table # 0x08 Ret
    .long op_jmp        - codegen_table # 0x09 Jmp
    .long op_jz         - codegen_table # 0x0A Jz
    .long op_dup        - codegen_table # 0x0B Dup
    .long op_pop        - codegen_table # 0x0C Pop
    .long op_swap       - codegen_table # 0x0D Swap
    .long op_loop       - codegen_table # 0x0E Loop
    .long op_halt       - codegen_table # 0x0F Halt
    .long op_nop        - codegen_table # 0x10 Dream
    .long op_nop        - codegen_table # 0x11 Stats
    .long op_nop        - codegen_table # 0x12 Nop
    .long op_store      - codegen_table # 0x13 Store
    .long op_load_local - codegen_table # 0x14 LoadLocal
    .long op_push_num   - codegen_table # 0x15 PushNum
    .long op_nop        - codegen_table # 0x16 Fuse
    .long op_nop        - codegen_table # 0x17 ScopeBegin
    .long op_nop        - codegen_table # 0x18 ScopeEnd
    .long op_push_mol   - codegen_table # 0x19 PushMol
    .long op_try_begin  - codegen_table # 0x1A TryBegin
    .long op_catch_end  - codegen_table # 0x1B CatchEnd
    .long op_store_upd  - codegen_table # 0x1C StoreUpdate
    .long op_nop        - codegen_table # 0x1D Trace
    .long op_nop        - codegen_table # 0x1E Inspect
    .long op_nop        - codegen_table # 0x1F Assert
    .long op_nop        - codegen_table # 0x20 TypeOf
    .long op_nop        - codegen_table # 0x21 Why
    .long op_nop        - codegen_table # 0x22 Explain
    .long op_nop        - codegen_table # 0x23 Ffi (stub)
    .long cg_call_closure - codegen_table # 0x24 CallClosure
    .long cg_closure      - codegen_table # 0x25 Closure
.section .text

# codegen Push: [chain_len:2 u16 LE][chain_bytes:N] → push raw bytes as chain
cg_push:
    # Read u16 molecule count
    movzwl  (%r12, %r13), %ecx         # u16 LE mol_count
    add     $2, %r13

    # Empty chain (mol_count == 0): just push empty onto stack
    test    %ecx, %ecx
    jnz     .cg_push_nonempty
    sub     $16, %r14
    movq    $0, (%r14)                  # ptr = 0
    movq    $0, 8(%r14)                 # len = 0
    jmp     vm_loop

.cg_push_nonempty:
    # Byte count = mol_count * 2 (each molecule = u16)
    mov     %ecx, %edx
    shl     $1, %edx                    # byte_count = mol_count * 2

    # Copy molecule bytes to heap
    mov     %r15, %rdi                  # dest = heap
    lea     (%r12, %r13), %rsi         # src = bytecode
    push    %rcx
    push    %rdx
    call    memcpy_small
    pop     %rdx
    pop     %rcx

    # Push (ptr, mol_count) onto VM stack
    sub     $16, %r14
    mov     %r15, (%r14)                # ptr to molecule data
    mov     %rcx, 8(%r14)               # mol_count (NOT byte count)
    add     %rdx, %r15                  # bump heap by byte_count
    add     %rdx, %r13                  # PC += byte_count
    jmp     vm_loop

# codegen Closure: [param_count:1][body_len:4]
# Creates a closure marker on VM stack, jumps over body.
# Marker: [ptr=body_pc as f64 bits][len=0xFFFFFFFFFFFFFFFE] (closure tag)
.equ CLOSURE_MARKER, -2

cg_closure:
    movzbl  (%r12, %r13), %ecx         # param_count
    inc     %r13
    movl    (%r12, %r13), %eax          # body_len (u32 LE)
    add     $4, %r13

    # Push closure marker: body_pc in ptr, param_count in len high bits
    # Encode: ptr = body_pc (current PC = start of body), len = CLOSURE_MARKER
    sub     $16, %r14
    # Store body_pc: if in boot context → relative PC, if in eval → absolute address
    # This allows op_call to distinguish boot vs eval closures
    lea     boot_bc_base(%rip), %rdx
    cmp     (%rdx), %r12
    je      .cg_closure_boot
    # Eval context: store absolute address (r12 + r13) with high bit set as flag
    lea     (%r12, %r13), %rdx
    bts     $63, %rdx                   # set bit 63 as "eval closure" flag
    mov     %rdx, (%r14)
    jmp     .cg_closure_done
.cg_closure_boot:
    mov     %r13, (%r14)                # boot: relative body_pc
.cg_closure_done:
    # Store closure marker with param_count encoded
    movq    $CLOSURE_MARKER, 8(%r14)    # len = closure tag
    # Save param_count in the byte after the marker
    # Actually, encode param_count in the closure marker itself:
    # We use a trick: len = 0xFFFFFFFFFFFFFFFE | (param_count << 48)
    # Simpler: just store param_count alongside. Use top byte of len.
    shl     $48, %rcx                   # param_count << 48
    orq     $CLOSURE_MARKER, %rcx       # combine with marker
    mov     %rcx, 8(%r14)              # store combined

    # Jump over body
    add     %eax, %r13d                 # PC += body_len
    jmp     vm_loop

# codegen CallClosure: [name_len:1][name:N][arity:1]
# Pops closure marker + args from stack, jumps to body PC.
# Uses CPU stack for return address.
cg_call_closure:
    movzbl  (%r12, %r13), %ecx         # name_len
    inc     %r13
    # Hash the name to find the closure in var table
    lea     (%r12, %r13), %rdi         # name ptr
    mov     %ecx, %edx                  # name len
    push    %rcx
    call    hash_name                   # → rax = name hash
    pop     %rcx
    add     %rcx, %r13                  # PC += name_len
    movzbl  (%r12, %r13), %ecx         # arity
    inc     %r13                        # PC past arity byte

    # Look up variable by hash → get closure marker
    push    %rcx                        # save arity
    call    var_load_hash               # rax=ptr, rdx=len
    pop     %rcx                        # restore arity

    # Check if it's a closure marker (len has CLOSURE_MARKER bits)
    mov     %rdx, %rsi
    and     $0xFFFF, %esi               # low 16 bits
    cmp     $CLOSURE_MARKER, %si        # == 0xFFFE?
    jne     .cc_not_closure

    # It's a closure — rax = body_pc, rdx has param_count in high bits
    # Extract body_pc
    mov     %rax, %rdi                  # body_pc

    # Save return context: PC + bytecode base
    push    %r12                        # save current r12
    push    %r13                        # save return PC

    # Check bit 63: eval closure (absolute address) vs boot closure (relative PC)
    bt      $63, %rdi
    jc      .cc_eval_closure

    # Boot closure: switch r12 to boot bytecode base, PC = relative body_pc
    lea     boot_bc_base(%rip), %rax
    mov     (%rax), %r12
    mov     %edi, %r13d                 # PC = body_pc (relative)
    jmp     vm_loop

.cc_eval_closure:
    # Eval closure: body_pc is absolute address with bit 63 set
    btr     $63, %rdi                   # clear bit 63 → absolute address
    # Use saved eval_bc_base as r12, compute body offset for r13
    lea     eval_bc_base(%rip), %rax
    mov     (%rax), %r12               # r12 = eval buffer base
    sub     %r12, %rdi                 # offset = absolute - base
    mov     %edi, %r13d               # PC = body offset within eval buffer
    jmp     vm_loop

.cc_not_closure:
    # Not a closure — push empty (like function not found)
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

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
    # Hash the name for lookup
    lea     (%r12, %r13), %rdi         # name ptr
    mov     %ecx, %edx                  # name len
    sub     $8, %rsp                    # align stack to 16 bytes
    push    %rcx                        # save name_len (stack now 16-aligned for call)
    call    hash_name                   # → rax = hash
    pop     %rcx
    add     $8, %rsp                    # restore alignment
    add     %rcx, %r13                  # PC += name_len
    # Search var table by hash
    call    var_load_hash               # rax=hash → rax=ptr, rdx=len
    sub     $16, %r14
    mov     %rax, (%r14)
    mov     %rdx, 8(%r14)
    jmp     vm_loop

op_store:
op_store_upd:
    # [0x33/0x39, name_len_u8, name_bytes...]
    movzbl  (%r12, %r13), %ecx         # name_len
    inc     %r13
    lea     (%r12, %r13), %rdi         # name ptr
    mov     %ecx, %edx                  # name len
    # Save value before hash call
    mov     (%r14), %r8                # value ptr
    mov     8(%r14), %r9               # value len
    add     $16, %r14                  # pop value
    push    %rcx
    push    %r8
    push    %r9
    call    hash_name                   # → rax = hash
    pop     %r9
    pop     %r8
    pop     %rcx
    add     %rcx, %r13                 # PC += name_len
    call    var_store_hash              # rax=hash, r8=ptr, r9=len
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

    # Pop condition from stack
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
    # [0x32/0x07, name_len_u8, name_bytes...]
    movzbl  (%r12, %r13), %ecx         # name_len
    inc     %r13
    lea     (%r12, %r13), %rdi         # name ptr

    # Hash-based builtin dispatch (FNV-1a)
    mov     %ecx, %edx                  # name len for hash_name
    push    %rcx                        # save name_len
    call    hash_name                   # → rax = hash
    pop     %rcx

    # ── Math builtins ──
    movabs  $0x57252090B96BC074, %rdx   # __hyp_add
    cmp     %rdx, %rax
    je      .call_add
    movabs  $0xF28AFA91114A9D15, %rdx   # __hyp_sub
    cmp     %rdx, %rax
    je      .call_sub
    movabs  $0x7800C290CB86CA01, %rdx   # __hyp_mul
    cmp     %rdx, %rax
    je      .call_mul
    movabs  $0x3E279790AB8C5BA8, %rdx   # __hyp_div
    cmp     %rdx, %rax
    je      .call_div

    # ── f64_to_le_bytes ──
    movabs  $0x7DC336EFBBA1F85B, %rdx   # __f64_to_le_bytes
    cmp     %rdx, %rax
    je      .call_f64_to_le_bytes

    # ── Modulo ──
    movabs  $0x7822B690CBA39C03, %rdx   # __hyp_mod
    cmp     %rdx, %rax
    je      .call_mod

    # ── Comparison builtins ──
    movabs  $0x59C2494311519859, %rdx   # __eq
    cmp     %rdx, %rax
    je      .call_eq
    movabs  $0xE50D9329A4496776, %rdx   # __cmp_lt
    cmp     %rdx, %rax
    je      .call_lt
    movabs  $0xE525A529A45E17ED, %rdx   # __cmp_gt
    cmp     %rdx, %rax
    je      .call_gt
    movabs  $0xE50D8429A4494DF9, %rdx   # __cmp_le
    cmp     %rdx, %rax
    je      .call_le
    movabs  $0xE5259429A45DFB0A, %rdx   # __cmp_ge
    cmp     %rdx, %rax
    je      .call_ge
    movabs  $0xE5068029A443287F, %rdx   # __cmp_ne
    cmp     %rdx, %rax
    je      .call_ne

    # ── String builtins ──
    movabs  $0x231F02F69878E492, %rdx   # __len
    cmp     %rdx, %rax
    je      .call_len
    movabs  $0xA51ED5C2BDE7E73B, %rdx   # __concat
    cmp     %rdx, %rax
    je      .call_concat
    movabs  $0x9FC9B1482B01D5E9, %rdx   # __char_at
    cmp     %rdx, %rax
    je      .call_char_at
    movabs  $0x463483AB25CAB22D, %rdx   # __str_char_at (alias)
    cmp     %rdx, %rax
    je      .call_char_at
    movabs  $0xE386A6329E2424BE, %rdx   # __substr
    cmp     %rdx, %rax
    je      .call_substr
    movabs  $0xEA1BDD6D065BF46A, %rdx   # __str_substr (alias)
    cmp     %rdx, %rax
    je      .call_substr
    movabs  $0x304A3A896BAAE100, %rdx   # substr (short alias)
    cmp     %rdx, %rax
    je      .call_substr
    movabs  $0xFBE159BD1C6EBFA7, %rdx   # __str_trim
    cmp     %rdx, %rax
    je      .call_str_trim
    movabs  $0x0CA83B022A791B4F, %rdx   # __str_is_keyword
    cmp     %rdx, %rax
    je      .call_str_is_keyword
    movabs  $0x74AEDE408DC6787F, %rdx   # __str_slice
    cmp     %rdx, %rax
    je      .call_substr                 # str_slice = substr alias
    movabs  $0x53416FC92E4E354A, %rdx   # __str_bytes
    cmp     %rdx, %rax
    je      .call_str_bytes
    movabs  $0x3EA9B2DCA6EB46A2, %rdx   # __to_number
    cmp     %rdx, %rax
    je      .call_to_number
    movabs  $0xE1C2E76089AECDEF, %rdx   # to_num (short alias)
    cmp     %rdx, %rax
    je      .call_to_number
    movabs  $0x3F9867C32BC1B166, %rdx   # __to_string
    cmp     %rdx, %rax
    je      .call_to_string
    movabs  $0x07515EC1BC3E1099, %rdx   # __char_code
    cmp     %rdx, %rax
    je      .call_char_code
    # ── File I/O builtins ──
    movabs  $0x229C6ACCAE3B5DBE, %rdx   # __file_read
    cmp     %rdx, %rax
    je      .call_file_read
    movabs  $0xC8220FDA76231661, %rdx   # __file_write
    cmp     %rdx, %rax
    je      .call_file_write
    # ── Raw byte builtins (for self-build) ──
    movabs  $0xB2E2F645F237541B, %rdx   # __bytes_new
    cmp     %rdx, %rax
    je      .call_bytes_new
    movabs  $0x5871F4465043D679, %rdx   # __bytes_set
    cmp     %rdx, %rax
    je      .call_bytes_set
    movabs  $0xEF8C08461494EA05, %rdx   # __bytes_get
    cmp     %rdx, %rax
    je      .call_bytes_get
    movabs  $0x255314D8EC3C037E, %rdx   # __bytes_write
    cmp     %rdx, %rax
    je      .call_bytes_write_raw
    movabs  $0xA5845725C8B01606, %rdx   # __file_read_bytes
    cmp     %rdx, %rax
    je      .call_file_read_bytes
    movabs  $0xC4352F45FC06104A, %rdx   # __bytes_len
    cmp     %rdx, %rax
    je      .call_bytes_len
    movabs  $0xA3A423F5F0BEDF3B, %rdx   # __heap_save
    cmp     %rdx, %rax
    je      .call_heap_save
    movabs  $0xDACC10780A43732C, %rdx   # __heap_restore
    cmp     %rdx, %rax
    je      .call_heap_restore
    cmp     %rdx, %rax
    je      .call_file_write

    movabs  $0xE811320261AE416D, %rdx   # __type_of
    cmp     %rdx, %rax
    je      .call_type_of

    # ── Crypto builtins ──
    movabs  $0x656D6D860AA4E534, %rdx   # __sha256
    cmp     %rdx, %rax
    je      .call_sha256

    # ── Math builtins ──
    movabs  $0x185FC0C022E829AF, %rdx   # __floor
    cmp     %rdx, %rax
    je      .call_floor
    movabs  $0x9E3D368BE8B3606A, %rdx   # __ceil
    cmp     %rdx, %rax
    je      .call_ceil

    # ── Error handling builtins ──
    movabs  $0xDFF308A36F62AAF5, %rdx   # __throw
    cmp     %rdx, %rax
    je      .call_throw

    # ── Array builtins ──
    movabs  $0xF10E316E51D66791, %rdx   # __array_new
    cmp     %rdx, %rax
    je      .call_array_new
    movabs  $0xA4B7BF6E270F2A57, %rdx   # __array_get
    cmp     %rdx, %rax
    je      .call_array_get
    movabs  $0x5210E36E8946BE83, %rdx   # __array_set
    cmp     %rdx, %rax
    je      .call_array_set
    movabs  $0x90134D214B786D0F, %rdx   # __set_at (alias)
    cmp     %rdx, %rax
    je      .call_array_set
    movabs  $0xF8BB87C89678B27D, %rdx   # __array_push
    cmp     %rdx, %rax
    je      .call_array_push
    movabs  $0x9FC459281D29BE97, %rdx   # __push (alias)
    cmp     %rdx, %rax
    je      .call_array_push
    movabs  $0x476FB46E82B22CB0, %rdx   # __array_pop
    cmp     %rdx, %rax
    je      .call_array_pop
    movabs  $0x46A5ACF6ACD83186, %rdx   # __pop (alias)
    cmp     %rdx, %rax
    je      .call_array_pop
    movabs  $0xB56460DC788BF312, %rdx   # __array_range
    cmp     %rdx, %rax
    je      .call_array_range
    movabs  $0x4A94D43F636D9492, %rdx   # __array_with_cap
    cmp     %rdx, %rax
    je      .call_array_with_cap
    movabs  $0xDEE1C66E474DB2AC, %rdx   # __array_len
    cmp     %rdx, %rax
    je      .call_len                    # same as __len

    # ── Dict builtins ──
    movabs  $0x061BEB9B86B295B2, %rdx   # __dict_new
    cmp     %rdx, %rax
    je      .call_dict_new
    movabs  $0xC8B4C99B63B3BB90, %rdx   # __dict_get
    cmp     %rdx, %rax
    je      .call_dict_get
    movabs  $0x5FCEDD9B2804CF1C, %rdx   # __dict_set
    cmp     %rdx, %rax
    je      .call_dict_set
    movabs  $0x234A766F8747B620, %rdx   # __dict_keys
    cmp     %rdx, %rax
    je      .call_dict_keys

    # ── Bitwise builtins ──
    movabs  $0x31F6E47C4AE5A082, %rdx   # __bit_or
    cmp     %rdx, %rax
    je      .call_bit_or
    movabs  $0x7551B73312F1D4D4, %rdx   # __bit_and
    cmp     %rdx, %rax
    je      .call_bit_and
    movabs  $0x4E03A6338DB8ADC8, %rdx   # __bit_xor
    cmp     %rdx, %rax
    je      .call_bit_xor
    movabs  $0xECF7CD335641730C, %rdx   # __bit_shl
    cmp     %rdx, %rax
    je      .call_bit_shl
    movabs  $0xECF7E3335641986E, %rdx   # __bit_shr
    cmp     %rdx, %rax
    je      .call_bit_shr

    # ── Struct/enum builtins ──
    movabs  $0x46B6739DEFF894A5, %rdx   # __struct_tag
    cmp     %rdx, %rax
    je      .call_struct_tag
    movabs  $0x1D937C1FCA5461FD, %rdx   # __enum_unit
    cmp     %rdx, %rax
    je      .call_enum_unit
    movabs  $0x58F6E86D37FAFB4F, %rdx   # __enum_payload
    cmp     %rdx, %rax
    je      .call_enum_payload
    movabs  $0x6EFB31ABBB6698FC, %rdx   # __match_enum
    cmp     %rdx, %rax
    je      .call_match_enum
    movabs  $0x385DDAACD4BF03A1, %rdx   # __enum_field
    cmp     %rdx, %rax
    je      .call_enum_field
    movabs  $0x0275ACD27913EF29, %rdx   # __logic_not
    cmp     %rdx, %rax
    je      .call_logic_not

    # ── No-op builtins (pop correct arg count) ──
    movabs  $0xC7631FF0030A6033, %rdx   # __use_module (1 arg)
    cmp     %rdx, %rax
    je      .call_noop_pop1
    movabs  $0x49DF59EEA723E9FA, %rdx   # __use_module_selective (count+N+1)
    cmp     %rdx, %rax
    je      .call_use_module_selective
    movabs  $0xBA79129DA012260E, %rdx   # __struct_def (2 args)
    cmp     %rdx, %rax
    je      .call_noop_pop2
    movabs  $0xC3FD4316C34B889E, %rdx   # __enum_def (2 args)
    cmp     %rdx, %rax
    je      .call_noop_pop2

    # ── Timer/sleep builtins ──
    movabs  $0x9EEC1A86B3B21786, %rdx   # __sleep
    cmp     %rdx, %rax
    je      .call_sleep
    movabs  $0x0E6B6C4BB0C674BA, %rdx   # __time
    cmp     %rdx, %rax
    je      .call_time
    movabs  $0xC3ADA5D879284B3B, %rdx   # __write_raw
    cmp     %rdx, %rax
    je      .call_write_raw

    # ── Molecule extract builtins (T5 ND.2) ──
    movabs  $0x9AB0ED8E471CE883, %rdx   # __mol_s
    cmp     %rdx, %rax
    je      .call_mol_s
    movabs  $0x9AB0EC8E471CE6D0, %rdx   # __mol_r
    cmp     %rdx, %rax
    je      .call_mol_r
    movabs  $0x9AB0F08E471CED9C, %rdx   # __mol_v
    cmp     %rdx, %rax
    je      .call_mol_v
    movabs  $0x9AB0DF8E471CD0B9, %rdx   # __mol_a
    cmp     %rdx, %rax
    je      .call_mol_a
    movabs  $0x9AB0F28E471CF102, %rdx   # __mol_t
    cmp     %rdx, %rax
    je      .call_mol_t
    movabs  $0xABFC7C5900FA2DAB, %rdx   # __mol_pack
    cmp     %rdx, %rax
    je      .call_mol_pack

    # ── UTF-8 builtins ──
    movabs  $0x4878D01FD4BD5E1A, %rdx   # __utf8_cp(str, pos) → codepoint
    cmp     %rdx, %rax
    je      .call_utf8_cp
    movabs  $0xB7114D166502D5D0, %rdx   # __utf8_len(str, pos) → byte count (1-4)
    cmp     %rdx, %rax
    je      .call_utf8_len

    # ── Eval bytecode (nested execution) ──
    movabs  $0xB810EF14843FBB65, %rdx   # __eval_bytecode
    cmp     %rdx, %rax
    je      .call_eval_bytecode

    # ── Not a builtin: try var_table lookup ──
    # rax still has the hash from hash_name
    push    %rcx                        # save name_len
    call    var_load_hash               # rax=ptr, rdx=len
    pop     %rcx

    # Check if it's a closure
    mov     %rdx, %rsi
    and     $0xFFFF, %esi
    cmp     $CLOSURE_MARKER, %si
    jne     .call_skip

    # Found closure in var_table — call it
    add     %rcx, %r13                  # PC past name bytes
    push    %r12                        # save bytecode base (CPU stack)
    push    %r13                        # save return address (CPU stack)
    # Scope save: snapshot var_table to heap scope stack.
    # Only for nested eval closures (boot closures don't need scoping).
    mov     %rax, %rdi                  # save body_pc (full 64-bit, may have bit 63 flag)
    lea     boot_bc_base(%rip), %rsi
    cmp     (%rsi), %r12
    je      .closure_skip_scope

    # FAST scope save: just save var_table count (NO memcpy!)
    # depth_stack[depth] = current count → 1 store, 8 bytes
    lea     closure_depth(%rip), %rax
    mov     (%rax), %rcx               # current depth
    push    %rdi                        # save body_pc
    lea     var_table_base(%rip), %rsi
    mov     (%rsi), %rsi
    mov     (%rsi), %rdx               # current var_table count
    lea     scope_frame_ptrs(%rip), %rax
    mov     %rdx, (%rax, %rcx, 8)     # depth_stack[depth] = count
    lea     closure_depth(%rip), %rax
    incq    (%rax)                      # depth++
    pop     %rdi                        # restore body_pc

.closure_skip_scope:
    # body_pc in rdi (full 64-bit, saved at line 1051)
    # Check bit 63: if set → eval closure (absolute address)
    bt      $63, %rdi
    jc      .closure_is_eval

    # Boot closure: body_pc is relative to boot_bc_base
    lea     boot_bc_base(%rip), %rsi
    mov     (%rsi), %r12               # r12 = boot bytecode base
    mov     %edi, %r13d                # PC = relative body_pc
    jmp     vm_loop

.closure_is_eval:
    # Eval closure: body_pc is absolute address with bit 63 flag
    btr     $63, %rdi                  # clear flag → rdi = absolute address of body
    # Use saved eval_bc_base as r12, compute body offset for r13
    # Jump targets in body are relative to eval buffer base, not body start
    lea     eval_bc_base(%rip), %rax
    mov     (%rax), %r12               # r12 = eval buffer base
    sub     %r12, %rdi                 # offset = absolute_addr - base
    mov     %edi, %r13d                # PC = body offset within eval buffer
    jmp     vm_loop

.call_skip:
    
    add     %rcx, %r13
    jmp     vm_loop

# ── Math builtins ────────────────────────────────────────────────────────

.call_add:
    add     %rcx, %r13
    # Check if either operand is a string (not f64/array/dict/closure marker)
    mov     8(%r14), %rdx              # b len/marker
    mov     24(%r14), %rax             # a len/marker
    # If both are f64, do numeric add
    cmp     $F64_MARKER, %rdx
    jne     .call_add_concat
    cmp     $F64_MARKER, %rax
    jne     .call_add_concat
    # Both f64 — numeric add
    movsd   (%r14), %xmm1              # b
    movsd   16(%r14), %xmm0            # a
    add     $16, %r14
    addsd   %xmm1, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)       # ensure marker is set!
    jmp     vm_loop
.call_add_concat:
    # At least one operand is a string — convert both to string and concat
    # First, ensure both are strings: convert f64 if needed
    # Check b (top of stack)
    cmp     $F64_MARKER, %rdx
    jne     .add_check_a
    # b is f64 — convert to string in place
    movq    (%r14), %xmm0
    push    %rax
    call    f64_to_ascii               # → eax = len, output at r15
    mov     %r15, %rdi                 # ASCII source
    add     %rax, %r15
    mov     %r15, %rsi                 # u16 dest
    mov     %eax, %ecx
    push    %rcx
    # Encode ASCII → u16 molecules
.add_b_encode:
    test    %ecx, %ecx
    jz      .add_b_encode_done
    movzbl  (%rdi), %edx
    or      $0x2100, %edx
    movw    %dx, (%rsi)
    inc     %rdi
    add     $2, %rsi
    dec     %ecx
    jmp     .add_b_encode
.add_b_encode_done:
    pop     %rcx
    mov     %r15, (%r14)               # replace b ptr
    mov     %rcx, 8(%r14)              # replace b len
    mov     %rsi, %r15                 # bump heap
    pop     %rax
.add_check_a:
    # Check a (second on stack)
    cmp     $F64_MARKER, %rax
    jne     .add_do_concat
    # a is f64 — convert to string in place
    movq    16(%r14), %xmm0
    call    f64_to_ascii
    mov     %r15, %rdi
    add     %rax, %r15
    mov     %r15, %rsi
    mov     %eax, %ecx
    push    %rcx
.add_a_encode:
    test    %ecx, %ecx
    jz      .add_a_encode_done
    movzbl  (%rdi), %edx
    or      $0x2100, %edx
    movw    %dx, (%rsi)
    inc     %rdi
    add     $2, %rsi
    dec     %ecx
    jmp     .add_a_encode
.add_a_encode_done:
    pop     %rcx
    mov     %r15, 16(%r14)             # replace a ptr
    mov     %rcx, 24(%r14)             # replace a len
    mov     %rsi, %r15                 # bump heap
.add_do_concat:
    # Now both are strings on stack — do concat
    # Copy a molecules then b molecules to heap
    mov     (%r14), %rsi               # b ptr
    mov     8(%r14), %rdx              # b mol_count
    mov     16(%r14), %rdi             # a ptr
    mov     24(%r14), %rcx             # a mol_count
    add     $32, %r14                  # pop both

    # Copy a molecules (rcx * 2 bytes)
    mov     %r15, %rax                 # result start
    push    %rdx
    push    %rsi
    push    %rcx
    mov     %r15, %rdi                 # dest
    # rdi was a ptr, need to use it as source
    pop     %rcx                        # a mol_count
    push    %rcx
    mov     -32(%r14), %rsi            # re-read a ptr (before pop: was at 16+16=32 offset)
    # Simpler: use saved values
    pop     %rcx
    push    %rcx
    # a is at: original 16(%r14) → after add $32 → -16(%r14)
    mov     -16(%r14), %rsi            # a ptr
    shl     $1, %rcx                   # bytes = mol_count * 2
    rep     movsb
    mov     %rdi, %r15

    # Copy b molecules
    pop     %rcx                        # a mol_count (saved)
    pop     %rsi                        # b ptr
    pop     %rdx                        # b mol_count
    push    %rcx                        # save a count for total
    push    %rdx                        # save b count for total
    mov     %r15, %rdi
    mov     %rdx, %rcx
    shl     $1, %rcx                   # bytes = mol_count * 2
    rep     movsb
    mov     %rdi, %r15

    # Total mol_count
    pop     %rdx                        # b count
    pop     %rcx                        # a count
    add     %rdx, %rcx                  # total

    # Push result
    sub     $16, %r14
    mov     %rax, (%r14)               # ptr
    mov     %rcx, 8(%r14)              # mol_count
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
    je      .div_by_zero               # P0-C: return 0 instead of halt
    movsd   16(%r14), %xmm0            # a (dividend)
    add     $16, %r14
    divsd   %xmm1, %xmm0              # a / b
    movsd   %xmm0, (%r14)
    jmp     vm_loop
.div_by_zero:
    # Return 0 instead of killing REPL
    add     $16, %r14                   # pop divisor
    xorpd   %xmm0, %xmm0             # 0.0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_mod:
    add     %rcx, %r13
    movsd   (%r14), %xmm1              # b (divisor)
    movsd   16(%r14), %xmm0            # a (dividend)
    add     $16, %r14
    # a % b = a - floor(a/b) * b
    movapd  %xmm0, %xmm2              # save a
    divsd   %xmm1, %xmm0              # a / b
    roundsd $1, %xmm0, %xmm0          # floor(a/b) — $1 = round toward -inf
    mulsd   %xmm1, %xmm0              # floor(a/b) * b
    subsd   %xmm0, %xmm2              # a - floor(a/b)*b
    movsd   %xmm2, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# f64_to_le_bytes: pop f64, push array of 8 f64 byte values
.call_f64_to_le_bytes:
    add     %rcx, %r13
    # Pop f64 value
    mov     (%r14), %rax               # raw f64 bits
    add     $16, %r14                  # pop
    # Create array of 8 elements on heap
    mov     %r15, %rdi                 # array base
    movq    $8, (%r15)                 # count = 8
    add     $8, %r15                    # past count header
    # Extract each byte and store as f64
    mov     $8, %ecx
.f64_bytes_loop:
    movzbl  %al, %edx                  # low byte
    shr     $8, %rax                    # shift right 8 bits
    # Store byte value as f64
    push    %rax
    cvtsi2sd %edx, %xmm0
    movsd   %xmm0, (%r15)             # f64 ptr slot
    movq    $F64_MARKER, 8(%r15)       # len slot
    add     $16, %r15
    pop     %rax
    dec     %ecx
    jnz     .f64_bytes_loop
    # Push array
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

# ── Comparison builtins ──────────────────────────────────────────────────

.call_eq:
    add     %rcx, %r13
    # Check if both are f64
    mov     8(%r14), %rax              # b len
    mov     24(%r14), %rcx             # a len
    cmp     $F64_MARKER, %rax
    jne     .eq_chain
    cmp     $F64_MARKER, %rcx
    jne     .eq_chain
    # Both f64
    movsd   (%r14), %xmm1
    movsd   16(%r14), %xmm0
    add     $16, %r14
    ucomisd %xmm1, %xmm0
    je      .eq_true_chain
    jmp     .eq_false_chain

.eq_chain:
    # Compare chains by u16 molecules (each mol = 2 bytes)
    mov     8(%r14), %rax              # b mol_count
    mov     24(%r14), %rcx             # a mol_count
    cmp     %rax, %rcx
    jne     .eq_false                   # different mol_count → not equal
    # Same mol_count — compare mol_count × 2 bytes
    mov     (%r14), %rsi               # b ptr
    mov     16(%r14), %rdi             # a ptr
    add     $16, %r14                  # pop 1
    shl     $1, %rax                    # byte_count = mol_count × 2
    mov     %rax, %rcx                  # count in bytes
    test    %rcx, %rcx
    jz      .eq_true_chain              # both empty → equal
    repe    cmpsb
    je      .eq_true_chain
.eq_false_chain:
    # Push f64 0.0 (false) — NOT empty chain (breaks &&/|| short-circuit)
    xor     %eax, %eax
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop
.eq_true_chain:
    # Push 1.0 (true)
    mov     $0x3FF0000000000000, %rax
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.eq_false:
    add     $16, %r14                  # pop 1
    xor     %eax, %eax
    mov     %rax, (%r14)               # f64 0.0
    movq    $F64_MARKER, 8(%r14)       # mark as f64 (not empty chain)
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
    # Pop value, push its length as f64
    add     %rcx, %r13
    mov     8(%r14), %rax              # len/marker
    cmp     $F64_MARKER, %rax
    je      .call_len_f64
    cmp     $ARRAY_MARKER, %rax
    je      .call_len_array
    cmp     $DICT_MARKER, %rax
    je      .call_len_dict
    jmp     .call_len_chain
.call_len_f64:
    # f64 → len = 1
    mov     $0x3FF0000000000000, %rax   # 1.0
    mov     %rax, (%r14)
    jmp     vm_loop
.call_len_array:
    # Array: read count from heap (follow forward chain if relocated)
    mov     (%r14), %rdi               # array base ptr
.call_len_array_fwd:
    cmpq    $ARRAY_FORWARD_MARKER, (%rdi)
    jne     .call_len_array_direct
    mov     8(%rdi), %rdi              # follow forward
    jmp     .call_len_array_fwd
.call_len_array_direct:
    mov     (%rdi), %rax               # count
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop
.call_len_dict:
    # Dict: read entry count from heap
    mov     (%r14), %rdi               # dict base ptr
    mov     (%rdi), %rax               # count
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop
.call_len_chain:
    # String: len is already in rax
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
    # Pop index (f64), pop string (molecule chain), push single char molecule
    add     %rcx, %r13
    movsd   (%r14), %xmm0             # index
    cvttsd2si %xmm0, %rax             # index as int
    mov     16(%r14), %rsi            # string ptr (u16 molecules)
    mov     24(%r14), %rcx            # string mol_count
    add     $32, %r14                 # pop 2

    # Bounds check (index < mol_count)
    cmp     %rcx, %rax
    jae     .char_at_empty

    # Get char molecule (u16 at rsi + rax*2)
    movzwl  (%rsi, %rax, 2), %edx     # read u16 molecule
    movw    %dx, (%r15)                # write u16 molecule
    sub     $16, %r14
    mov     %r15, (%r14)               # ptr to molecule
    movq    $1, 8(%r14)                # mol_count = 1
    add     $2, %r15                   # bump heap by 2 (u16)
    jmp     vm_loop

.char_at_empty:
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

.call_substr:
    # Pop end (f64), pop start (f64), pop string → push substring
    # substr(str, start, end) — end is exclusive index (not length)
    add     %rcx, %r13
    movsd   (%r14), %xmm0             # end index
    cvttsd2si %xmm0, %rdx
    movsd   16(%r14), %xmm0           # start index
    cvttsd2si %xmm0, %rax
    mov     32(%r14), %rsi            # string ptr
    mov     40(%r14), %rcx            # string mol_count
    add     $48, %r14                 # pop 3

    # Bounds
    cmp     %rcx, %rax
    jae     .char_at_empty
    cmp     %rcx, %rdx
    jbe     .substr_ok
    mov     %rcx, %rdx                 # clamp end to string length
.substr_ok:
    sub     %rax, %rdx                 # mol_count = end - start
    lea     (%rsi, %rax, 2), %rsi     # start ptr (×2 for u16 molecules)

    # Copy molecules to heap (rdx molecules × 2 bytes each)
    mov     %r15, %rax                 # save heap ptr
    mov     %r15, %rdi
    mov     %edx, %ecx
    shl     $1, %ecx                   # byte_count = mol_count × 2
    push    %rdx
    rep     movsb
    mov     %rdi, %r15
    pop     %rdx

    sub     $16, %r14
    mov     %rax, (%r14)               # ptr
    mov     %rdx, 8(%r14)              # mol_count
    jmp     vm_loop

# ── Not-equal builtin ───────────────────────────────────────────────────

.call_ne:
    add     %rcx, %r13
    # Check if both are f64
    mov     8(%r14), %rax              # b len
    mov     24(%r14), %rdx             # a len
    cmp     $F64_MARKER, %rax
    jne     .ne_chain
    cmp     $F64_MARKER, %rdx
    jne     .ne_chain
    # Both f64 — compare
    movsd   (%r14), %xmm1
    movsd   16(%r14), %xmm0
    add     $16, %r14
    ucomisd %xmm1, %xmm0
    jne     .ne_true
    # Equal → false
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop
.ne_true:
    mov     $0x3FF0000000000000, %rax   # 1.0
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop
.ne_chain:
    # Compare chains by content
    mov     8(%r14), %rax
    mov     24(%r14), %rdx
    cmp     %rax, %rdx
    jne     .ne_chain_diff
    # Same length — compare bytes
    mov     (%r14), %rsi
    mov     16(%r14), %rdi
    add     $16, %r14
    mov     %rax, %rcx
    test    %rcx, %rcx
    jz      .ne_chain_eq
    repe    cmpsb
    jne     .ne_true
.ne_chain_eq:
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop
.ne_chain_diff:
    add     $16, %r14
    mov     $0x3FF0000000000000, %rax
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# ── Type-of builtin ─────────────────────────────────────────────────────

.call_type_of:
    add     %rcx, %r13
    mov     8(%r14), %rax              # len/marker
    add     $16, %r14                  # pop value
    sub     $16, %r14                  # push result
    cmp     $F64_MARKER, %rax
    je      .typeof_number
    cmp     $ARRAY_MARKER, %rax
    je      .typeof_array
    cmp     $DICT_MARKER, %rax
    je      .typeof_dict
    cmp     $CLOSURE_MARKER, %rax
    je      .typeof_fn
    test    %rax, %rax
    jz      .typeof_nil
    # Default: string
    lea     str_string(%rip), %rax
    mov     %rax, (%r14)
    movq    $6, 8(%r14)
    jmp     vm_loop
.typeof_number:
    lea     str_number(%rip), %rax
    mov     %rax, (%r14)
    movq    $6, 8(%r14)
    jmp     vm_loop
.typeof_array:
    lea     str_array(%rip), %rax
    mov     %rax, (%r14)
    movq    $5, 8(%r14)
    jmp     vm_loop
.typeof_dict:
    lea     str_dict(%rip), %rax
    mov     %rax, (%r14)
    movq    $4, 8(%r14)
    jmp     vm_loop
.typeof_fn:
    lea     str_fn(%rip), %rax
    mov     %rax, (%r14)
    movq    $8, 8(%r14)
    jmp     vm_loop
.typeof_nil:
    lea     str_nil(%rip), %rax
    mov     %rax, (%r14)
    movq    $3, 8(%r14)
    jmp     vm_loop

# ── To-number builtin ───────────────────────────────────────────────────

.call_to_number:
    add     %rcx, %r13
    # Pop string, parse as decimal number
    # Strings are u16 molecules: each mol = [low_byte, 0x21] in LE.
    # Character value is in the LOW byte of each u16.
    # rcx = mol_count (number of characters)
    mov     (%r14), %rsi               # string ptr
    mov     8(%r14), %rcx              # mol_count (= char count)
    add     $16, %r14                  # pop

    # Simple decimal parser: sign + digits + optional '.' + digits
    xor     %eax, %eax                  # negative flag
    xorpd   %xmm0, %xmm0              # result = 0.0
    test    %rcx, %rcx
    jz      .to_num_done
    # Check minus sign (read low byte of first u16 molecule)
    movzbl  (%rsi), %edx
    cmp     $'-', %edx
    jne     .to_num_loop
    mov     $1, %eax                    # negative
    add     $2, %rsi                    # skip 1 molecule (2 bytes)
    dec     %rcx
.to_num_loop:
    test    %rcx, %rcx
    jz      .to_num_sign
    movzbl  (%rsi), %edx               # read low byte of u16 molecule
    cmp     $'.', %edx
    je      .to_num_frac
    sub     $'0', %edx
    cmp     $10, %edx
    jae     .to_num_sign
    # result = result * 10 + digit
    movsd   const_ten(%rip), %xmm1
    mulsd   %xmm1, %xmm0
    cvtsi2sd %edx, %xmm1
    addsd   %xmm1, %xmm0
    add     $2, %rsi                    # next u16 molecule
    dec     %rcx
    jmp     .to_num_loop
.to_num_frac:
    add     $2, %rsi                    # skip '.' molecule
    dec     %rcx
    movsd   const_one(%rip), %xmm2      # divisor = 1.0
.to_num_frac_loop:
    test    %rcx, %rcx
    jz      .to_num_sign
    movzbl  (%rsi), %edx               # low byte of u16 molecule
    sub     $'0', %edx
    cmp     $10, %edx
    jae     .to_num_sign
    movsd   const_ten(%rip), %xmm1
    mulsd   %xmm1, %xmm2               # divisor *= 10
    cvtsi2sd %edx, %xmm1               # digit
    divsd   %xmm2, %xmm1               # digit / divisor
    addsd   %xmm1, %xmm0               # result += digit/divisor
    add     $2, %rsi                    # next u16 molecule
    dec     %rcx
    jmp     .to_num_frac_loop
.to_num_sign:
    test    %eax, %eax
    jz      .to_num_done
    # Negate
    movsd   const_neg_one(%rip), %xmm1
    mulsd   %xmm1, %xmm0
.to_num_done:
    sub     $16, %r14
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# ── To-string builtin: convert any value to string ─────────────────────
# If already a string (u16 molecules), pass through.
# If f64, convert to decimal string (u16 molecules).
# Other types: return empty string.

.call_to_string:
    add     %rcx, %r13
    mov     (%r14), %rsi               # ptr
    mov     8(%r14), %rdx              # len/marker

    # If already a string chain (len > 0 and not a marker), pass through
    cmp     $F64_MARKER, %rdx
    je      .to_string_f64
    cmp     $ARRAY_MARKER, %rdx
    je      .to_string_other
    cmp     $DICT_MARKER, %rdx
    je      .to_string_other
    cmp     $CLOSURE_MARKER, %rdx
    je      .to_string_other
    # It's a string chain — pass through unchanged
    jmp     vm_loop

.to_string_f64:
    # Convert f64 to u16 molecule string
    add     $16, %r14                  # pop
    movq    %rsi, %xmm0               # rsi has f64 bits
    call    f64_to_ascii               # → eax = length, output at r15 (ASCII)
    # Now convert ASCII bytes at r15 to u16 molecules
    mov     %eax, %ecx                 # save ASCII length
    mov     %r15, %rsi                 # ASCII source
    add     %rax, %r15                 # bump past ASCII
    mov     %r15, %rdi                 # u16 molecule output
    push    %rcx
    xor     %edx, %edx
.to_str_encode:
    test    %ecx, %ecx
    jz      .to_str_encode_done
    movzbl  (%rsi), %edx
    or      $0x2100, %edx              # u16 molecule = 0x2100 | byte
    movw    %dx, (%rdi)
    add     $1, %rsi
    add     $2, %rdi
    dec     %ecx
    jmp     .to_str_encode
.to_str_encode_done:
    pop     %rcx                        # mol_count = ASCII length
    # Push string result
    sub     $16, %r14
    mov     %r15, (%r14)               # ptr to u16 molecules
    mov     %rcx, 8(%r14)              # mol_count
    mov     %rdi, %r15                  # bump heap past molecules
    jmp     vm_loop

.to_string_other:
    # Non-convertible types: return empty string
    mov     $0, (%r14)
    mov     $0, 8(%r14)
    jmp     vm_loop

# ── Char-code builtin: string → f64 byte value (NO heap allocation) ────

.call_char_code:
    add     %rcx, %r13
    # Pop string (u16 molecule chain)
    mov     (%r14), %rsi               # string ptr
    mov     8(%r14), %rcx              # mol_count
    # Read first molecule's low byte = ASCII value
    test    %rcx, %rcx
    jz      .char_code_zero
    movzbl  (%rsi), %eax               # low byte of first u16 molecule
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop
.char_code_zero:
    xorpd   %xmm0, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# ── String-trim builtin ─────────────────────────────────────────────────

.call_str_trim:
    add     %rcx, %r13
    mov     (%r14), %rsi               # string ptr (u16 molecules)
    mov     8(%r14), %rcx              # string mol_count
    add     $16, %r14                  # pop
    # Trim leading whitespace (u16 molecules, low byte = char value)
    mov     %rsi, %rdi                  # start (byte ptr)
    lea     (%rsi, %rcx, 2), %rsi      # end = ptr + mol_count × 2
.trim_lead:
    cmp     %rsi, %rdi
    jae     .trim_empty
    movzbl  (%rdi), %eax               # low byte of u16 LE molecule = char
    cmp     $32, %eax                   # space
    je      .trim_lead_next
    cmp     $9, %eax                    # tab
    je      .trim_lead_next
    cmp     $10, %eax                   # newline
    je      .trim_lead_next
    cmp     $13, %eax                   # CR
    je      .trim_lead_next
    jmp     .trim_trail
.trim_lead_next:
    add     $2, %rdi                    # next molecule (2 bytes per u16)
    jmp     .trim_lead
.trim_trail:
    # rdi = trimmed start (byte ptr), rsi = past end (byte ptr)
.trim_trail_loop:
    cmp     %rdi, %rsi
    jbe     .trim_done
    movzbl  -2(%rsi), %eax             # low byte of last u16 molecule
    cmp     $32, %eax
    je      .trim_trail_dec
    cmp     $9, %eax
    je      .trim_trail_dec
    cmp     $10, %eax
    je      .trim_trail_dec
    cmp     $13, %eax
    je      .trim_trail_dec
    jmp     .trim_done
.trim_trail_dec:
    sub     $2, %rsi                    # prev molecule (2 bytes per u16)
    jmp     .trim_trail_loop
.trim_done:
    mov     %rsi, %rcx
    sub     %rdi, %rcx                  # byte count of trimmed region
    # Copy to heap (rcx bytes = mol_count × 2)
    mov     %r15, %rax
    push    %rcx
    mov     %r15, %rsi
    xchg    %rsi, %rdi                  # rdi=dest=heap, rsi=src=trimmed
    mov     %ecx, %ecx
    rep     movsb
    mov     %rdi, %r15                  # bump heap
    pop     %rcx
    shr     $1, %rcx                    # mol_count = byte_count / 2
    sub     $16, %r14
    mov     %rax, (%r14)
    mov     %rcx, 8(%r14)
    jmp     vm_loop
.trim_empty:
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

# ── String-bytes builtin ────────────────────────────────────────────────

.call_str_bytes:
    add     %rcx, %r13
    # Pop string, create array of byte values
    mov     (%r14), %rsi               # string ptr
    mov     8(%r14), %rcx              # string len
    add     $16, %r14                  # pop

    # Create array on heap: [count:8][elem_0_ptr:8][elem_0_len:8]...
    mov     %r15, %rdi                  # array base
    mov     %rcx, (%r15)               # count = string length
    add     $8, %r15                    # past count header

    # Each u16 molecule → extract low byte → f64 on heap
    # Molecules are 2 bytes each (u16 LE): low byte = char value
    xor     %edx, %edx                 # molecule index
.str_bytes_loop:
    cmp     %rcx, %rdx
    jae     .str_bytes_done
    movzbl  (%rsi, %rdx, 2), %eax     # read low byte of u16 (stride 2)
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r15)              # ptr slot = f64 bits
    movq    $F64_MARKER, 8(%r15)       # len slot = F64_MARKER
    add     $16, %r15
    inc     %rdx
    jmp     .str_bytes_loop
.str_bytes_done:
    sub     $16, %r14
    mov     %rdi, (%r14)               # array base ptr
    movq    $ARRAY_MARKER, 8(%r14)     # marker
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Array builtins
# ═══════════════════════════════════════════════════════════════════════════

.call_array_new:
    # Stack: [elem0, elem1, ..., elemN-1, count]
    # count is f64 on top of stack
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rax              # count
    add     $16, %r14                   # pop count

    # Create array on heap
    mov     %r15, %rdi                  # array base
    mov     %rax, (%r15)               # store count
    add     $8, %r15                    # past count

    # Copy elements from stack (they are in reverse order on stack)
    # Stack has elem[N-1] on top, elem[0] deepest.
    # We want array[0] = elem0 (deepest), array[N-1] = elemN-1 (top).
    # Reverse: first calculate offset to deepest element, then copy forward.
    test    %rax, %rax
    jz      .array_new_done
    mov     %rax, %rdx                  # save count
    # Deepest element is at r14 + (count-1)*16
    lea     -16(%r14, %rax, 8), %rsi   # wrong offset, let me recalc
    # Actually: elem0 is at offset (count-1)*16 from current r14
    dec     %rax
    shl     $4, %rax                    # (count-1) * 16
    lea     (%r14, %rax), %rsi         # pointer to deepest element (elem0)
    mov     %rdx, %rcx                  # loop counter

.array_new_copy:
    # Copy from rsi (stack, deepest first) to heap
    mov     (%rsi), %rax               # elem ptr
    mov     8(%rsi), %r8               # elem len
    mov     %rax, (%r15)
    mov     %r8, 8(%r15)
    add     $16, %r15                   # heap forward
    sub     $16, %rsi                   # stack backward (toward top)
    dec     %rcx
    jnz     .array_new_copy

    # Pop all elements from stack
    shl     $4, %rdx                    # count * 16
    add     %rdx, %r14

.array_new_done:
    # Reserve capacity only for EMPTY arrays (used with push).
    # Non-empty arrays (ArrayLit [1,2,3]) are fixed-size — no capacity needed.
    mov     (%rdi), %rcx               # count
    test    %rcx, %rcx
    jnz     .array_new_push             # non-empty → skip capacity
    # Empty array: reserve ARRAY_INIT_CAP for future pushes
    lea     8(%rdi), %rax
    add     $ARRAY_INIT_CAP * 16, %rax
    cmp     %rax, %r15
    jae     .array_new_push
    mov     %rax, %r15
.array_new_push:
    # Push array
    sub     $16, %r14
    mov     %rdi, (%r14)               # array base ptr
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

.call_array_get:
    # Stack: [array, index] → pop both, push array[index]
    add     %rcx, %r13
    # Pop index
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rax              # index
    # Pop array
    mov     16(%r14), %rdi              # array base ptr
    add     $32, %r14                   # pop 2

    # Check array marker was valid
    test    %rdi, %rdi
    jz      .array_get_empty

    # Follow forward pointer chain if relocated
.array_get_fwd:
    cmpq    $ARRAY_FORWARD_MARKER, (%rdi)
    jne     .array_get_direct
    mov     8(%rdi), %rdi
    jmp     .array_get_fwd
.array_get_direct:
    # Read count
    mov     (%rdi), %rcx               # count
    cmp     %rcx, %rax
    jae     .array_get_empty            # out of bounds

    # Element at offset 8 + index*16
    shl     $4, %rax
    add     $8, %rax
    mov     (%rdi, %rax), %rdx         # elem ptr
    mov     8(%rdi, %rax), %rcx        # elem len

    sub     $16, %r14
    mov     %rdx, (%r14)
    mov     %rcx, 8(%r14)
    jmp     vm_loop

.array_get_empty:
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

.call_array_set:
    # Stack: [array, index, value] → set array[index] = value, push array
    add     %rcx, %r13
    # Pop value
    mov     (%r14), %r8                # val ptr
    mov     8(%r14), %r9               # val len
    # Pop index
    movsd   16(%r14), %xmm0
    cvttsd2si %xmm0, %rax              # index
    # Pop array
    mov     32(%r14), %rdi              # array base
    add     $48, %r14                   # pop 3

    test    %rdi, %rdi
    jz      .array_set_done

    # Follow forward pointer chain
.array_set_fwd:
    cmpq    $ARRAY_FORWARD_MARKER, (%rdi)
    jne     .array_set_direct
    mov     8(%rdi), %rdi
    jmp     .array_set_fwd
.array_set_direct:
    # Bounds check
    mov     (%rdi), %rcx               # count
    cmp     %rcx, %rax
    jae     .array_set_done

    # Write at offset 8 + index*16
    shl     $4, %rax
    add     $8, %rax
    mov     %r8, (%rdi, %rax)
    mov     %r9, 8(%rdi, %rax)

.array_set_done:
    # Push array back
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

.call_array_push:
    # Stack: [array, value] → append value to array, push array back
    add     %rcx, %r13
    # Pop value
    mov     (%r14), %r8                # val ptr
    mov     8(%r14), %r9               # val len
    # Pop array
    mov     16(%r14), %rdi              # array base
    mov     24(%r14), %rax             # should be ARRAY_MARKER
    add     $32, %r14                   # pop 2

    test    %rdi, %rdi
    jz      .array_push_create_new

    # Follow forward pointer chain if array was relocated
.array_push_fwd:
    cmpq    $ARRAY_FORWARD_MARKER, (%rdi)
    jne     .array_push_check
    mov     8(%rdi), %rdi
    jmp     .array_push_fwd
.array_push_check:
    # Check if write address is still within safe zone (< r15)
    mov     (%rdi), %rcx               # count
    mov     %rcx, %rax
    shl     $4, %rax
    add     $8, %rax                    # write offset = 8 + count*16
    lea     16(%rdi, %rax), %rdx       # end of write = base + offset + 16
    cmp     %r15, %rdx
    ja      .array_push_relocate       # write would go past heap top → relocate

    # Safe: write in-place
    mov     %r8, (%rdi, %rax)          # write val ptr
    mov     %r9, 8(%rdi, %rax)         # write val len
    incq    (%rdi)                      # count++
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

.array_push_relocate:
    # Capacity zone overwritten by other allocations.
    # Copy entire array to r15 (new location) + append new element.
    mov     %r15, %rsi                  # new base
    lea     1(%rcx), %rax              # new count
    mov     %rax, (%rsi)               # write new count
    # Copy old elements
    test    %rcx, %rcx
    jz      .push_reloc_append
    push    %rcx
    lea     8(%rdi), %r10              # src
    lea     8(%rsi), %r11              # dst
.push_reloc_copy:
    mov     (%r10), %rax
    mov     %rax, (%r11)
    mov     8(%r10), %rax
    mov     %rax, 8(%r11)
    add     $16, %r10
    add     $16, %r11
    dec     %rcx
    jnz     .push_reloc_copy
    pop     %rcx
    jmp     .push_reloc_write
.push_reloc_append:
    lea     8(%rsi), %r11              # dst for first element
.push_reloc_write:
    # Write new element
    mov     %r8, (%r11)
    mov     %r9, 8(%r11)
    add     $16, %r11
    # Bump r15 past new array + capacity
    # Use max(count*2, ARRAY_INIT_CAP) to avoid repeated relocations
    lea     8(%rsi), %rax
    mov     (%rsi), %rdx               # new count (already incremented)
    shl     $5, %rdx                   # count * 32 = count * 2 * 16 (double capacity)
    cmp     $ARRAY_INIT_CAP * 16, %rdx
    jae     .push_reloc_use_count
    mov     $ARRAY_INIT_CAP * 16, %edx # use ARRAY_INIT_CAP if larger
.push_reloc_use_count:
    add     %rdx, %rax                 # base + 8 + capacity
    cmp     %rax, %r11
    jae     .push_reloc_bump
    mov     %rax, %r11
.push_reloc_bump:
    mov     %r11, %r15
    # Write FORWARD POINTER at old location
    movq    $ARRAY_FORWARD_MARKER, (%rdi)   # mark old as forwarded
    mov     %rsi, 8(%rdi)                    # pointer to new location
    # Push NEW array
    sub     $16, %r14
    mov     %rsi, (%r14)
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

.array_push_create_new:
    # Create new array with 1 element + capacity
    mov     %r15, %rdi                  # array base
    movq    $1, (%r15)                  # count = 1
    mov     %r8, 8(%r15)               # elem ptr
    mov     %r9, 16(%r15)              # elem len
    # Reserve ARRAY_INIT_CAP slots
    lea     8(,%r15,1), %rax
    add     $ARRAY_INIT_CAP * 16 + 8, %r15  # bump heap past capacity
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

.call_array_pop:
    # Stack: [array] → pop last element, push it
    add     %rcx, %r13
    mov     (%r14), %rdi               # array base
    add     $16, %r14                   # pop array

    test    %rdi, %rdi
    jz      .array_pop_empty
    # Follow forward chain
.array_pop_fwd:
    cmpq    $ARRAY_FORWARD_MARKER, (%rdi)
    jne     .array_pop_direct
    mov     8(%rdi), %rdi
    jmp     .array_pop_fwd
.array_pop_direct:
    mov     (%rdi), %rcx               # count
    test    %rcx, %rcx
    jz      .array_pop_empty

    # Get last element
    dec     %rcx
    mov     %rcx, (%rdi)               # count--
    shl     $4, %rcx
    add     $8, %rcx
    mov     (%rdi, %rcx), %rax         # elem ptr
    mov     8(%rdi, %rcx), %rdx        # elem len
    sub     $16, %r14
    mov     %rax, (%r14)
    mov     %rdx, 8(%r14)
    jmp     vm_loop

.array_pop_empty:
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Array with capacity: __array_with_cap(n) → empty array with n-slot capacity
# Creates array with count=0, bumps heap by 8 + n*16
# ═══════════════════════════════════════════════════════════════════════════

.call_array_with_cap:
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2siq %xmm0, %rcx            # n = requested capacity
    add     $16, %r14                   # pop arg

    # Create empty array with explicit capacity
    mov     %r15, %rdi                  # array base
    movq    $0, (%r15)                  # count = 0 (empty)
    # Bump heap past capacity zone: base + 8 + n*16
    mov     %rcx, %rax
    shl     $4, %rax                    # n * 16
    add     $8, %rax                    # + header
    add     %rdi, %rax                  # base + 8 + n*16
    mov     %rax, %r15                  # bump heap

    # Push array
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Array range builtin: range(n) → [0, 1, 2, ..., n-1]
# ═══════════════════════════════════════════════════════════════════════════

.call_array_range:
    # Stack: [n] → array of [0, 1, ..., n-1]
    add     %rcx, %r13
    # Pop n (f64)
    movsd   (%r14), %xmm0
    add     $16, %r14
    cvttsd2siq %xmm0, %rcx            # n as integer

    # Allocate array on heap: count + n * 16 bytes (within ARRAY_INIT_CAP)
    mov     %r15, %rdi                  # array base
    mov     %rcx, (%r15)                # count = n
    add     $8, %r15                    # past count field

    # Fill array with f64 values 0, 1, 2, ..., n-1
    xor     %eax, %eax                  # i = 0
    test    %rcx, %rcx
    jz      .range_done
.range_fill:
    # Convert i to f64 and store as [ptr=f64_bits, len=F64_MARKER]
    cvtsi2sdq %rax, %xmm1
    movq    %xmm1, (%r15)              # ptr = f64 bits
    movq    $F64_MARKER, 8(%r15)       # len = F64_MARKER
    add     $16, %r15
    inc     %rax
    cmp     %rcx, %rax
    jb      .range_fill
.range_done:
    # Ensure heap is bumped to at least ARRAY_INIT_CAP
    # Calculate: base + 8 + ARRAY_INIT_CAP*16
    lea     8(%rdi), %rax
    add     $ARRAY_INIT_CAP * 16, %rax
    cmp     %rax, %r15
    jae     .range_push
    mov     %rax, %r15                  # bump heap to capacity
.range_push:
    # Push array onto stack
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Dict builtins
# ═══════════════════════════════════════════════════════════════════════════

.call_dict_new:
    # Stack: [key0, val0, key1, val1, ..., count]
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rax              # count (number of pairs)
    add     $16, %r14                   # pop count

    # Create dict on heap: [count:8][entries: key_ptr:8 key_len:8 val_ptr:8 val_len:8]
    mov     %r15, %rdi                  # dict base
    mov     %rax, (%r15)               # store count
    add     $8, %r15                    # past count header

    test    %rax, %rax
    jz      .dict_new_done
    mov     %rax, %rdx                  # save count

    # Pairs are on stack in reverse: top = last_val, then last_key, ...
    # Stack (top to bottom): valN-1, keyN-1, ..., val0, key0
    # We need dict entries in order: key0,val0, key1,val1, ...
    # Deepest pair (key0,val0) is at offset (count-1)*32 from r14
    dec     %rax
    shl     $5, %rax                    # (count-1) * 32
    lea     (%r14, %rax), %rsi         # ptr to deepest pair (key0)
    mov     %rdx, %rcx

.dict_new_copy:
    # Stack pair: val at (%rsi), key at 16(%rsi) (key pushed first = deeper)
    mov     16(%rsi), %rax             # key ptr (deeper on stack)
    mov     24(%rsi), %r8              # key len
    mov     (%rsi), %r9                # val ptr (top of pair)
    mov     8(%rsi), %r10              # val len
    mov     %rax, (%r15)               # store key ptr
    mov     %r8, 8(%r15)               # store key len
    mov     %r9, 16(%r15)              # store val ptr
    mov     %r10, 24(%r15)             # store val len
    add     $32, %r15                   # next entry
    sub     $32, %rsi                   # prev pair on stack
    dec     %rcx
    jnz     .dict_new_copy

    # Pop all pairs from stack
    mov     %rdx, %rcx
    shl     $5, %rcx                    # count * 32
    add     %rcx, %r14

.dict_new_done:
    sub     $16, %r14
    mov     %rdi, (%r14)               # dict base ptr
    movq    $DICT_MARKER, 8(%r14)
    jmp     vm_loop

.call_dict_get:
    # Stack: [dict, key] → pop both, push dict[key]
    add     %rcx, %r13
    # Check dict marker BEFORE popping: 24(%r14) = dict len field
    mov     24(%r14), %rax
    cmp     $DICT_MARKER, %rax
    jne     .dict_get_not_dict
    # Pop key
    mov     (%r14), %rsi               # key ptr
    mov     8(%r14), %rdx              # key len
    # Pop dict
    mov     16(%r14), %rdi              # dict base ptr
    add     $32, %r14                   # pop 2

    test    %rdi, %rdi
    jz      .dict_get_empty
    mov     (%rdi), %rcx               # entry count
    cmp     $10000, %rcx
    ja      .dict_get_empty
    test    %rcx, %rcx
    jz      .dict_get_empty
    lea     8(%rdi), %r8               # first entry
.dict_get_search:
    test    %rcx, %rcx
    jz      .dict_get_empty

    # Compare key: entry key at (%r8), len at 8(%r8)
    mov     8(%r8), %rax               # entry key len
    cmp     %rax, %rdx                  # same length?
    jne     .dict_get_next

    # Same mol_count — compare all bytes (mol_count × 2 for u16 molecules)
    push    %rcx
    push    %rsi
    push    %rdx
    push    %r8
    mov     (%r8), %rdi                # entry key ptr
    mov     %rdx, %rcx                 # mol_count
    shl     $1, %rcx                   # byte_count = mol_count × 2
    repe    cmpsb
    pop     %r8
    pop     %rdx
    pop     %rsi
    pop     %rcx
    je      .dict_get_found

.dict_get_next:
    add     $32, %r8                    # next entry
    dec     %rcx
    jmp     .dict_get_search

.dict_get_found:
    # Value is at 16(%r8) ptr, 24(%r8) len
    mov     16(%r8), %rax
    mov     24(%r8), %rdx
    sub     $16, %r14
    mov     %rax, (%r14)
    mov     %rdx, 8(%r14)
    jmp     vm_loop

.dict_get_not_dict:
    # Not a dict — pop both args, push empty
    add     $32, %r14
.dict_get_empty:
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

.call_dict_set:
    # Stack: [dict, key, value] → set dict[key] = value, push dict
    add     %rcx, %r13
    # Pop value
    mov     (%r14), %r9                # val ptr
    mov     8(%r14), %r10              # val len
    # Pop key
    mov     16(%r14), %rsi             # key ptr
    mov     24(%r14), %rdx             # key len
    # Pop dict — save dict base in rbp (callee-saved, preserved across cmpsb)
    mov     32(%r14), %rdi             # dict base
    push    %rbp
    mov     %rdi, %rbp                 # save dict base in rbp
    add     $48, %r14                  # pop 3

    test    %rdi, %rdi
    jz      .dict_set_push_rbp

    # Search for existing key
    mov     (%rdi), %rcx
    lea     8(%rdi), %r8
.dict_set_search:
    test    %rcx, %rcx
    jz      .dict_set_append

    mov     8(%r8), %rax
    cmp     %rax, %rdx
    jne     .dict_set_next

    push    %rcx
    push    %rsi
    push    %rdx
    push    %r8
    mov     (%r8), %rdi
    mov     %rdx, %rcx
    shl     $1, %rcx                   # byte_count = mol_count × 2
    repe    cmpsb
    pop     %r8
    pop     %rdx
    pop     %rsi
    pop     %rcx
    je      .dict_set_update

.dict_set_next:
    add     $32, %r8
    dec     %rcx
    jmp     .dict_set_search

.dict_set_update:
    # Found — update value in-place
    mov     %r9, 16(%r8)
    mov     %r10, 24(%r8)
    mov     %rbp, %rdi                 # restore dict base from rbp
    jmp     .dict_set_push_rbp

.dict_set_append:
    # Append new entry at end of dict (at r15, bump heap)
    mov     %rbp, %rdi                 # restore dict base from rbp
    mov     (%rdi), %rcx               # current count
    # Write new entry at r15 (heap top) — adjacent to dict
    mov     %rsi, (%r15)               # key ptr
    mov     %rdx, 8(%r15)              # key len
    mov     %r9, 16(%r15)              # val ptr
    mov     %r10, 24(%r15)             # val len
    add     $32, %r15                   # bump heap
    incq    (%rdi)                      # count++

.dict_set_push_rbp:
    pop     %rbp                        # restore rbp
.dict_set_push:
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $DICT_MARKER, 8(%r14)
    jmp     vm_loop

.call_dict_keys:
    # Stack: [dict] → push array of keys
    add     %rcx, %r13
    mov     (%r14), %rdi               # dict base
    add     $16, %r14                   # pop dict

    test    %rdi, %rdi
    jz      .dict_keys_empty

    # Create array of keys
    mov     (%rdi), %rcx               # dict entry count
    mov     %r15, %rsi                  # array base
    mov     %rcx, (%r15)               # array count
    add     $8, %r15                    # past count
    lea     8(%rdi), %r8               # first entry

    test    %rcx, %rcx
    jz      .dict_keys_done
.dict_keys_loop:
    # Copy key ptr/len to array
    mov     (%r8), %rax
    mov     8(%r8), %rdx
    mov     %rax, (%r15)
    mov     %rdx, 8(%r15)
    add     $16, %r15
    add     $32, %r8
    dec     %rcx
    jnz     .dict_keys_loop

.dict_keys_done:
    sub     $16, %r14
    mov     %rsi, (%r14)               # array base
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

.dict_keys_empty:
    # Empty array
    mov     %r15, %rdi
    movq    $0, (%r15)
    add     $8, %r15
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Bitwise builtins
# ═══════════════════════════════════════════════════════════════════════════

.call_bit_or:
    add     %rcx, %r13
    movsd   (%r14), %xmm1
    movsd   16(%r14), %xmm0
    add     $16, %r14
    cvttsd2si %xmm0, %rax
    cvttsd2si %xmm1, %rdx
    or      %rdx, %rax
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_bit_and:
    add     %rcx, %r13
    movsd   (%r14), %xmm1
    movsd   16(%r14), %xmm0
    add     $16, %r14
    cvttsd2si %xmm0, %rax
    cvttsd2si %xmm1, %rdx
    and     %rdx, %rax
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_bit_shl:
    add     %rcx, %r13
    movsd   (%r14), %xmm1              # shift amount
    movsd   16(%r14), %xmm0            # value
    add     $16, %r14
    cvttsd2si %xmm0, %rax
    cvttsd2si %xmm1, %rcx
    shl     %cl, %rax
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_bit_shr:
    add     %rcx, %r13
    movsd   (%r14), %xmm1
    movsd   16(%r14), %xmm0
    add     $16, %r14
    cvttsd2si %xmm0, %rax
    cvttsd2si %xmm1, %rcx
    shr     %cl, %rax
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_bit_xor:
    add     %rcx, %r13
    movsd   (%r14), %xmm1
    movsd   16(%r14), %xmm0
    add     $16, %r14
    cvttsd2si %xmm0, %rax
    cvttsd2si %xmm1, %rdx
    xor     %rdx, %rax
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# Struct tag: create NEW dict with tag entry prepended.
# Cannot write in-place because Push(tag_name) allocated between dict and r15.
# New dict layout: [count+1:8][tag_entry:32][old_entries:count*32]
.call_struct_tag:
    add     %rcx, %r13
    # Stack: [dict, tag_name]
    mov     (%r14), %rsi               # tag ptr
    mov     8(%r14), %rdx              # tag mol_count
    mov     16(%r14), %rdi             # old dict base ptr
    mov     (%rdi), %rcx               # old count

    # Allocate new dict at r15
    mov     %r15, %r8                   # new dict base = r15
    lea     1(%rcx), %rax              # new_count = count + 1
    mov     %rax, (%r15)               # write new count
    add     $8, %r15                    # past count header

    # Write tag entry first (key_ptr=0, key_len=0, val=tag)
    movq    $0, (%r15)                 # key_ptr = 0
    movq    $0, 8(%r15)                # key_len = 0
    mov     %rsi, 16(%r15)             # val_ptr = tag
    mov     %rdx, 24(%r15)             # val_len = tag mol_count
    add     $32, %r15

    # Copy old entries: count * 32 bytes from old_dict+8
    test    %rcx, %rcx
    jz      .struct_tag_done
    lea     8(%rdi), %rsi              # src = old entries
    mov     %r15, %rdi                 # dst = after tag entry
    mov     %rcx, %rax
    shl     $5, %rax                    # count * 32 bytes
    mov     %rax, %rcx                  # byte count for rep movsb
    rep     movsb
    add     %rax, %r15                  # bump heap

.struct_tag_done:
    # Pop tag, replace dict with new dict on stack
    add     $16, %r14                  # pop tag_name
    mov     %r8, (%r14)                # dict ptr = new dict
    # len stays DICT_MARKER (already there from dict_new)
    jmp     vm_loop

# Enum unit variant (no-op: tag is already on stack)
.call_enum_unit:
    add     %rcx, %r13
    # Stack: [tag] → leave tag as-is (tag IS the enum representation)
    jmp     vm_loop

# Enum payload variant: concatenate tag + separator + args into one chain
.call_enum_payload:
    add     %rcx, %r13
    # Stack (top to bottom): [count, argN-1, ..., arg0, tag]
    # Result: tag | 0x0000 | arg0 | 0x0000 | arg1 | ...

    # Pop count
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rax             # count = number of args
    add     $16, %r14                  # pop count

    push    %rax                       # save arg_count on CPU stack
    mov     %r15, %rbp                 # save result_base (rbp is free here)
    xor     %r8d, %r8d                 # total_mol_count = 0

    # Tag is deepest: at r14 + count*16
    mov     %rax, %rcx
    shl     $4, %rcx                   # count * 16
    mov     (%r14, %rcx), %rsi         # tag ptr
    mov     8(%r14, %rcx), %rcx        # tag mol_count
    add     %rcx, %r8                  # total += tag_mol_count

    # Copy tag to heap
    shl     $1, %rcx                   # byte_count
    mov     %r15, %rdi
    rep     movsb                      # copy (rcx becomes 0, rdi = r15 + bytes)
    mov     %rdi, %r15                 # bump heap

    # Append each arg with separator
    pop     %rax                       # restore arg_count
    test    %rax, %rax
    jz      .enum_payload_done
    # Args on stack: arg0 at r14+(count-1)*16, argN-1 at r14+0
    xor     %edx, %edx                 # i = 0
.enum_payload_loop:
    cmp     %rax, %rdx
    jae     .enum_payload_done

    # Separator: 0x0000
    movw    $0, (%r15)
    add     $2, %r15
    inc     %r8                        # total_mol_count++

    # arg[i] at r14 + (count-1-i)*16
    push    %rax
    push    %rdx
    mov     %rax, %rcx
    dec     %rcx
    sub     %rdx, %rcx                 # count-1-i
    shl     $4, %rcx
    mov     (%r14, %rcx), %rsi         # arg ptr
    mov     8(%r14, %rcx), %rcx        # arg mol_count
    add     %rcx, %r8                  # total += arg_mol_count
    shl     $1, %rcx                   # byte_count
    mov     %r15, %rdi
    rep     movsb
    mov     %rdi, %r15                 # bump heap
    pop     %rdx
    pop     %rax
    inc     %edx
    jmp     .enum_payload_loop

.enum_payload_done:
    # Pop all entries: count args + 1 tag
    inc     %rax                       # count + 1
    shl     $4, %rax                   # (count+1) * 16
    add     %rax, %r14

    # Push result chain
    sub     $16, %r14
    mov     %rbp, (%r14)              # result ptr (saved at start)
    mov     %r8, 8(%r14)              # total mol_count
    jmp     vm_loop

# Logic not: empty → 1.0, non-empty → empty
.call_logic_not:
    add     %rcx, %r13
    mov     8(%r14), %rax              # len/marker
    # Check if value is empty (ptr=0 and len=0) or f64 0.0
    cmp     $F64_MARKER, %rax
    je      .logic_not_f64
    # Chain: empty (len==0) → true, non-empty → false
    test    %rax, %rax
    jz      .logic_not_is_empty
    # Non-empty chain → push empty (false)
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop
.logic_not_f64:
    # f64: 0.0 → true, non-zero → false
    mov     (%r14), %rax               # f64 bits
    test    %rax, %rax
    jz      .logic_not_is_empty
    # Non-zero f64 → push empty (false)
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop
.logic_not_is_empty:
    # Empty/zero → push 1.0 (true)
    mov     $0x3FF0000000000000, %rax   # 1.0
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# Match enum: compare tag prefix of subject with expected tag
# Handles both chain format (tag|0x0000|payload) and dict format (DICT_MARKER).
.call_match_enum:
    add     %rcx, %r13
    # Stack: [subject, expected_tag]
    mov     (%r14), %rsi               # expected_tag ptr
    mov     8(%r14), %rdx              # expected_tag mol_count
    mov     16(%r14), %rdi             # subject ptr
    mov     24(%r14), %rcx             # subject mol_count/marker
    add     $16, %r14                  # pop 1 (keep result slot)

    # Check if subject is a dict (DICT_MARKER)
    cmp     $DICT_MARKER, %rcx
    je      .match_enum_dict

    # Chain format: extract tag = molecules before first 0x0000 separator
    xor     %r8d, %r8d                 # tag_mol_count = 0
    test    %rcx, %rcx
    jz      .match_enum_compare
.match_enum_scan:
    cmp     %rcx, %r8
    jae     .match_enum_compare
    movzwl  (%rdi, %r8, 2), %eax       # read u16 molecule
    test    %eax, %eax                  # 0x0000 separator?
    jz      .match_enum_compare
    inc     %r8
    jmp     .match_enum_scan

.match_enum_compare:
    # Compare tag_mol_count with expected mol_count
    cmp     %rdx, %r8
    jne     .match_enum_false
    # Compare bytes: tag (rdi) vs expected (rsi), mol_count*2 bytes
    mov     %r8, %rcx
    shl     $1, %rcx                   # byte_count
    push    %rsi
    push    %rdi
    repe    cmpsb
    pop     %rdi
    pop     %rsi
    je      .match_enum_true

# Dict format: scan entries for tag (key_ptr=0, key_len=0), compare value
.match_enum_dict:
    # rdi = dict base ptr, rsi = expected_tag ptr, rdx = expected_tag mol_count
    mov     (%rdi), %rcx               # dict entry count
    lea     8(%rdi), %r9               # first entry: dict_base + 8
    test    %rcx, %rcx
    jz      .match_enum_false
.match_enum_dict_scan:
    # Check if entry has key_len=0 (tag marker)
    cmpq    $0, 8(%r9)                 # key_len == 0?
    jne     .match_enum_dict_next
    cmpq    $0, (%r9)                  # key_ptr == 0?
    jne     .match_enum_dict_next
    # Found tag entry — compare value with expected
    mov     16(%r9), %rdi              # tag_val_ptr
    mov     24(%r9), %r8               # tag_val_mol_count
    jmp     .match_enum_compare
.match_enum_dict_next:
    add     $32, %r9                    # next entry
    dec     %rcx
    jnz     .match_enum_dict_scan
    jmp     .match_enum_false           # no tag found

.match_enum_false:
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop
.match_enum_true:
    mov     $0x3FF0000000000000, %rax   # 1.0
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# Enum field: extract nth payload field from enum variant chain
# Handles both chain format (tag|0x0000|field0|0x0000|...) and dict format.
.call_enum_field:
    add     %rcx, %r13
    # Stack: [enum_chain, index]
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rax             # index
    mov     16(%r14), %rsi             # enum chain/dict ptr
    mov     24(%r14), %rcx             # enum chain mol_count / marker
    add     $16, %r14                  # pop 1 (keep result slot)

    # Check if subject is a dict (DICT_MARKER)
    cmp     $DICT_MARKER, %rcx
    je      .enum_field_dict

    # Chain format: skip past tag (before first 0x0000 separator)
    xor     %edx, %edx                 # position
.enum_field_skip_tag:
    cmp     %rcx, %rdx
    jae     .enum_field_empty
    movzwl  (%rsi, %rdx, 2), %edi
    inc     %rdx
    test    %edi, %edi                  # 0x0000 separator?
    jnz     .enum_field_skip_tag
    # rdx now points past the first separator

    # Skip 'index' more fields (each field ends with 0x0000 or end of chain)
    xor     %r8d, %r8d                 # current_field = 0
.enum_field_find:
    cmp     %rax, %r8
    jae     .enum_field_extract
    # Skip to next separator
.enum_field_skip:
    cmp     %rcx, %rdx
    jae     .enum_field_empty
    movzwl  (%rsi, %rdx, 2), %edi
    inc     %rdx
    test    %edi, %edi
    jnz     .enum_field_skip
    inc     %r8
    jmp     .enum_field_find

.enum_field_extract:
    # Extract field: from current rdx to next separator or end
    mov     %rdx, %r8                  # field_start
.enum_field_count:
    cmp     %rcx, %rdx
    jae     .enum_field_copy
    movzwl  (%rsi, %rdx, 2), %edi
    test    %edi, %edi
    jz      .enum_field_copy
    inc     %rdx
    jmp     .enum_field_count

.enum_field_copy:
    # Field molecules from r8 to rdx
    mov     %rdx, %r9
    sub     %r8, %r9                   # field_mol_count
    # Copy to heap
    lea     (%rsi, %r8, 2), %rsi       # source ptr
    mov     %r15, %rdi                 # dest
    mov     %r15, %rax                 # save result ptr
    mov     %r9, %rcx
    shl     $1, %rcx                   # byte_count
    rep     movsb
    mov     %rdi, %r15                 # bump heap
    mov     %rax, (%r14)               # result ptr
    mov     %r9, 8(%r14)               # result mol_count
    jmp     vm_loop

# Dict format: get nth non-tag entry's value
.enum_field_dict:
    # rsi = dict base, rax = desired index
    mov     (%rsi), %rcx               # entry count
    lea     8(%rsi), %r9               # first entry ptr
    xor     %r8d, %r8d                 # current non-tag index
    test    %rcx, %rcx
    jz      .enum_field_empty
.enum_field_dict_scan:
    # Skip tag entries (key_ptr=0, key_len=0)
    cmpq    $0, 8(%r9)                 # key_len == 0?
    jne     .enum_field_dict_check
    cmpq    $0, (%r9)                  # key_ptr == 0?
    je      .enum_field_dict_skip      # tag entry → skip
.enum_field_dict_check:
    # Non-tag entry — check if this is the desired index
    cmp     %rax, %r8
    je      .enum_field_dict_found
    inc     %r8
.enum_field_dict_skip:
    add     $32, %r9
    dec     %rcx
    jnz     .enum_field_dict_scan
    jmp     .enum_field_empty
.enum_field_dict_found:
    # Return this entry's value
    mov     16(%r9), %rax              # val_ptr
    mov     24(%r9), %rdx              # val_len
    mov     %rax, (%r14)
    mov     %rdx, 8(%r14)
    jmp     vm_loop

.enum_field_empty:
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

# String is-keyword: pop string, check if it's an Olang keyword
.call_str_is_keyword:
    add     %rcx, %r13
    # Pop string, hash its low bytes, compare with known keyword hashes
    mov     (%r14), %rsi               # string ptr
    mov     8(%r14), %rcx              # mol_count
    # Hash the low bytes (char values) using FNV-1a
    movabs  $0xcbf29ce484222325, %rax
    movabs  $0x100000001b3, %rdx
    test    %rcx, %rcx
    jz      .kw_check
.kw_hash_loop:
    movzbl  (%rsi), %edi               # low byte of molecule
    xor     %rdi, %rax
    imul    %rdx, %rax
    add     $2, %rsi                   # next molecule
    dec     %rcx
    jnz     .kw_hash_loop
.kw_check:
    # Compare hash against known keywords
    # Pre-computed FNV-1a hashes of keyword byte values
    lea     kw_hashes(%rip), %rsi
    mov     kw_count(%rip), %ecx
.kw_scan:
    test    %ecx, %ecx
    jz      .kw_false
    cmp     (%rsi), %rax
    je      .kw_true
    add     $8, %rsi
    dec     %ecx
    jmp     .kw_scan
.kw_true:
    mov     $0x3FF0000000000000, %rax   # 1.0
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop
.kw_false:
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

# No-op builtins: pop args and discard
.call_noop_pop1:
    add     %rcx, %r13
    add     $16, %r14                  # pop 1
    jmp     vm_loop

.call_noop_pop2:
    add     %rcx, %r13
    add     $32, %r14                  # pop 2
    jmp     vm_loop

# __use_module_selective: pop count, then count imports, then module name
.call_use_module_selective:
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rax             # count
    add     $16, %r14                  # pop count
    inc     %rax                       # +1 for module name
    shl     $4, %rax                   # (count+1) * 16
    add     %rax, %r14                 # pop all imports + module name
    jmp     vm_loop

# Eval bytecode (nested execution)
# ═══════════════════════════════════════════════════════════════════════════

.call_eval_bytecode:
    add     %rcx, %r13
    # Pop bytecode array from stack
    mov     (%r14), %rdi               # array base (or raw ptr)
    mov     8(%r14), %rax              # marker
    add     $16, %r14

    cmp     $ARRAY_MARKER, %rax
    jne     .eval_bc_raw

    # Follow forward pointer chain if array was relocated
.eval_bc_fwd:
    cmpq    $ARRAY_FORWARD_MARKER, (%rdi)
    jne     .eval_bc_array
    mov     8(%rdi), %rdi
    jmp     .eval_bc_fwd
.eval_bc_array:
    # Array of bytes: convert to contiguous bytecode buffer
    # Array: [count:8][elem_0_ptr:8][elem_0_len:8]...
    # Each element is a f64 representing a byte value
    mov     (%rdi), %rcx               # byte count
    lea     8(%rdi), %rsi              # first element
    mov     %r15, %r8                   # bytecode buffer start
    xor     %edx, %edx
.eval_bc_copy:
    cmp     %rcx, %rdx
    jae     .eval_bc_run
    movsd   (%rsi), %xmm0
    cvttsd2si %xmm0, %rax
    movb    %al, (%r15, %rdx)
    add     $16, %rsi                   # next array element
    inc     %rdx
    jmp     .eval_bc_copy

.eval_bc_raw:
    # Raw bytes (ptr, len)
    mov     %rdi, %r8                   # buffer ptr
    mov     %rax, %rcx                  # length
    test    %rcx, %rcx
    jz      .eval_bc_empty

.eval_bc_run:
    # Save current execution context on CPU stack
    push    %r12                        # save bytecode base
    push    %r13                        # save PC
    push    %rbx                        # save bytecode size
    movabs  $0xDEADBEEFCAFE0001, %rax
    push    %rax                        # nested flag sentinel

    # Set up nested execution
    mov     %r8, %r12                   # new bytecode base
    xor     %r13d, %r13d               # PC = 0
    mov     $0x100000, %rbx             # 1MB bounds — op_ret accepts boot PCs
    add     %rcx, %r15                  # bump heap past bytecode buffer
    # Save eval buffer base for cross-boundary closure calls
    lea     eval_bc_base(%rip), %rax
    mov     %r8, (%rax)

    jmp     vm_loop                     # execute!
    # When this hits op_halt, it will check the nested flag and return here

.eval_bc_empty:
    # Push empty result
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════

op_ret:
    # Return from closure call — pop return address from CPU stack.
    pop     %rax                        # saved PC
    pop     %r12                        # saved bytecode base
    mov     %eax, %r13d                 # PC = return address
    # Scope restore: ONLY if returning TO eval context (not boot→boot)
    # Check: is restored r12 the eval buffer? If r12 == boot_bc_base → boot context → skip
    lea     boot_bc_base(%rip), %rdi
    cmp     (%rdi), %r12
    je      vm_loop                     # returning to boot → no scope restore needed
    # FAST scope restore: just truncate var_table count (NO memcpy!)
    lea     closure_depth(%rip), %rsi
    cmpq    $0, (%rsi)
    je      vm_loop
    decq    (%rsi)                      # depth--
    mov     (%rsi), %rcx               # new depth
    # Restore count from depth_stack[depth]
    lea     scope_frame_ptrs(%rip), %rax
    mov     (%rax, %rcx, 8), %rdx      # saved_count
    lea     var_table_base(%rip), %rdi
    mov     (%rdi), %rdi
    mov     %rdx, (%rdi)               # truncate: var_table.count = saved_count
    jmp     vm_loop

# ═══════════════════════════════════════════════════════════════════════════
# SHA-256 (FIPS 180-4) — __sha256(string) → hex string (64 chars)
# ═══════════════════════════════════════════════════════════════════════════
.call_sha256:
    add     %rcx, %r13              # skip name bytes
    # Pop string: ptr=chain ptr, len=mol_count (u16 molecules)
    mov     (%r14), %rsi            # string ptr (u16 molecules)
    mov     8(%r14), %rdx           # string len (mol count)
    add     $16, %r14               # pop

    # Decode u16 string → raw bytes in sha_input buffer
    lea     sha_input(%rip), %rdi
    xor     %rcx, %rcx              # byte count
    test    %rdx, %rdx
    jz      .sha_empty_input
    mov     %rdx, %r8               # save mol count
.sha_decode_loop:
    cmp     %r8, %rcx
    jge     .sha_decode_done
    movzbl  (%rsi, %rcx, 2), %eax   # low byte of u16 mol
    mov     %al, (%rdi, %rcx)
    inc     %rcx
    jmp     .sha_decode_loop
.sha_decode_done:
.sha_empty_input:
    # rcx = input byte count, rdi = sha_input
    mov     %rcx, %r8               # r8 = msg_len

    # ── SHA-256 core ──
    # Initialize hash state H[0..7]
    lea     sha_state(%rip), %rbx
    movl    $0x6a09e667, 0(%rbx)
    movl    $0xbb67ae85, 4(%rbx)
    movl    $0x3c6ef372, 8(%rbx)
    movl    $0xa54ff53a, 12(%rbx)
    movl    $0x510e527f, 16(%rbx)
    movl    $0x9b05688c, 20(%rbx)
    movl    $0x1f83d9ab, 24(%rbx)
    movl    $0x5be0cd19, 28(%rbx)

    # Padding: append 0x80, zeros, then 64-bit big-endian length
    lea     sha_input(%rip), %rdi
    mov     %r8, %rcx               # msg_len
    movb    $0x80, (%rdi, %rcx)     # append 0x80
    inc     %rcx
    # Pad with zeros until (len % 64) == 56
.sha_pad_loop:
    mov     %rcx, %rax
    and     $63, %rax
    cmp     $56, %rax
    je      .sha_pad_done
    movb    $0, (%rdi, %rcx)
    inc     %rcx
    jmp     .sha_pad_loop
.sha_pad_done:
    # Append bit length (msg_len * 8) as big-endian u64
    mov     %r8, %rax
    shl     $3, %rax                 # bit length
    bswap   %rax
    mov     %rax, (%rdi, %rcx)
    add     $8, %rcx
    # rcx = total padded length (multiple of 64)

    # Process each 64-byte block
    lea     sha_input(%rip), %rdi
    xor     %r9, %r9                 # block offset
.sha_block_loop:
    cmp     %rcx, %r9
    jge     .sha_done

    # Save rcx/r9/rdi BEFORE W extension (W extend clobbers ecx)
    lea     sha_save_regs(%rip), %rax
    mov     %rcx, 24(%rax)            # padded length
    mov     %r9,  16(%rax)            # block offset
    mov     %rdi, 32(%rax)            # input pointer

    # Prepare message schedule W[0..63]
    lea     sha_w(%rip), %r10
    # W[0..15] = block words (big-endian)
    xor     %r11, %r11
.sha_w_init:
    cmp     $16, %r11
    jge     .sha_w_extend
    lea     (%rdi, %r9), %rax
    mov     (%rax, %r11, 4), %eax
    bswap   %eax
    mov     %eax, (%r10, %r11, 4)
    inc     %r11
    jmp     .sha_w_init

.sha_w_extend:
    cmp     $64, %r11
    jge     .sha_compress
    # W[i] = σ1(W[i-2]) + W[i-7] + σ0(W[i-15]) + W[i-16]
    # σ0(x) = ROTR(7,x) ^ ROTR(18,x) ^ SHR(3,x)
    # σ1(x) = ROTR(17,x) ^ ROTR(19,x) ^ SHR(10,x)
    lea     -2(%r11), %rax
    mov     (%r10, %rax, 4), %edx   # W[i-2]
    mov     %edx, %eax
    ror     $17, %eax
    mov     %edx, %ecx
    ror     $19, %ecx
    xor     %ecx, %eax
    mov     %edx, %ecx
    shr     $10, %ecx
    xor     %ecx, %eax               # σ1(W[i-2])
    push    %rax                      # save σ1

    lea     -15(%r11), %rax
    mov     (%r10, %rax, 4), %edx   # W[i-15]
    mov     %edx, %eax
    ror     $7, %eax
    mov     %edx, %ecx
    ror     $18, %ecx
    xor     %ecx, %eax
    mov     %edx, %ecx
    shr     $3, %ecx
    xor     %ecx, %eax               # σ0(W[i-15])

    pop     %rdx                      # σ1
    add     %edx, %eax               # σ1 + σ0
    lea     -7(%r11), %edx
    add     (%r10, %rdx, 4), %eax   # + W[i-7]
    lea     -16(%r11), %edx
    add     (%r10, %rdx, 4), %eax   # + W[i-16]
    mov     %eax, (%r10, %r11, 4)
    inc     %r11
    jmp     .sha_w_extend

.sha_compress:
    # Save remaining VM registers (rcx/r9/rdi already saved at block_loop)
    lea     sha_save_regs(%rip), %rax
    mov     %r12, 0(%rax)
    mov     %rbp, 8(%rax)
    # r9, rcx, rdi already saved before W extension
    mov     %rbx, 40(%rax)
    mov     %r14, 48(%rax)
    mov     %r15, 56(%rax)

    # Initialize working vars from state → sha_abcdefgh
    lea     sha_state(%rip), %rbx
    lea     sha_abcdefgh(%rip), %rbp
    mov     0(%rbx), %eax; mov %eax, 0(%rbp)
    mov     4(%rbx), %eax; mov %eax, 4(%rbp)
    mov     8(%rbx), %eax; mov %eax, 8(%rbp)
    mov     12(%rbx), %eax; mov %eax, 12(%rbp)
    mov     16(%rbx), %eax; mov %eax, 16(%rbp)
    mov     20(%rbx), %eax; mov %eax, 20(%rbp)
    mov     24(%rbx), %eax; mov %eax, 24(%rbp)
    mov     28(%rbx), %eax; mov %eax, 28(%rbp)

    # 64 compression rounds
    lea     sha_k(%rip), %r10
    lea     sha_w(%rip), %r11
    xor     %r12d, %r12d
.sha_round:
    cmp     $64, %r12d
    jge     .sha_round_done

    # Σ1(e) = ROTR(6,e) ^ ROTR(11,e) ^ ROTR(25,e)
    mov     16(%rbp), %eax           # e
    mov     %eax, %edx
    ror     $6, %edx
    mov     %eax, %ecx
    ror     $11, %ecx
    xor     %ecx, %edx
    mov     %eax, %ecx
    ror     $25, %ecx
    xor     %ecx, %edx               # Σ1(e)

    # Ch(e,f,g) = (e & f) ^ (~e & g)
    mov     16(%rbp), %eax           # e
    mov     20(%rbp), %ecx           # f
    and     %eax, %ecx               # e & f
    not     %eax
    and     24(%rbp), %eax           # ~e & g
    xor     %eax, %ecx               # Ch

    # T1 = h + Σ1(e) + Ch(e,f,g) + K[i] + W[i]
    mov     28(%rbp), %eax           # h
    add     %edx, %eax               # + Σ1
    add     %ecx, %eax               # + Ch
    add     (%r10, %r12, 4), %eax   # + K[i]
    add     (%r11, %r12, 4), %eax   # + W[i]
    mov     %eax, %esi               # T1

    # Σ0(a) = ROTR(2,a) ^ ROTR(13,a) ^ ROTR(22,a)
    mov     0(%rbp), %eax            # a
    mov     %eax, %edx
    ror     $2, %edx
    mov     %eax, %ecx
    ror     $13, %ecx
    xor     %ecx, %edx
    mov     %eax, %ecx
    ror     $22, %ecx
    xor     %ecx, %edx               # Σ0(a)

    # Maj(a,b,c) = (a & b) ^ (a & c) ^ (b & c)
    mov     0(%rbp), %eax            # a
    mov     4(%rbp), %ecx            # b
    mov     %eax, %edi
    and     %ecx, %edi               # a & b
    and     8(%rbp), %eax            # a & c
    xor     %eax, %edi
    and     8(%rbp), %ecx            # b & c
    xor     %ecx, %edi               # Maj

    # T2 = Σ0(a) + Maj(a,b,c)
    add     %edi, %edx               # T2

    # Shift: h=g, g=f, f=e, e=d+T1, d=c, c=b, b=a, a=T1+T2
    mov     24(%rbp), %eax; mov %eax, 28(%rbp)  # h = g
    mov     20(%rbp), %eax; mov %eax, 24(%rbp)  # g = f
    mov     16(%rbp), %eax; mov %eax, 20(%rbp)  # f = e
    mov     12(%rbp), %eax
    add     %esi, %eax
    mov     %eax, 16(%rbp)                       # e = d + T1
    mov     8(%rbp), %eax; mov %eax, 12(%rbp)   # d = c
    mov     4(%rbp), %eax; mov %eax, 8(%rbp)    # c = b
    mov     0(%rbp), %eax; mov %eax, 4(%rbp)    # b = a
    lea     (%rsi, %rdx), %eax
    mov     %eax, 0(%rbp)                        # a = T1 + T2

    inc     %r12d
    jmp     .sha_round

.sha_round_done:
    # Add compressed values to state BEFORE restoring rbp
    # (rbp still points to sha_abcdefgh from compression)
    lea     sha_state(%rip), %rbx
    mov     0(%rbp), %eax;  add %eax, 0(%rbx)
    mov     4(%rbp), %eax;  add %eax, 4(%rbx)
    mov     8(%rbp), %eax;  add %eax, 8(%rbx)
    mov     12(%rbp), %eax; add %eax, 12(%rbx)
    mov     16(%rbp), %eax; add %eax, 16(%rbx)
    mov     20(%rbp), %eax; add %eax, 20(%rbx)
    mov     24(%rbp), %eax; add %eax, 24(%rbx)
    mov     28(%rbp), %eax; add %eax, 28(%rbx)

    # NOW restore ALL VM registers from BSS
    lea     sha_save_regs(%rip), %rax
    mov     0(%rax), %r12
    mov     8(%rax), %rbp
    mov     16(%rax), %r9
    mov     24(%rax), %rcx
    mov     32(%rax), %rdi
    mov     40(%rax), %rbx
    mov     48(%rax), %r14
    mov     56(%rax), %r15

    add     $64, %r9
    jmp     .sha_block_loop

.sha_done:
    # Convert state H[0..7] to hex string on heap (u16 molecules)
    lea     sha_state(%rip), %rbx
    mov     %r15, %rdi               # output string base on heap
    movq    $0, %rcx                 # output mol count
    # 8 words × 8 hex chars = 64 chars
    xor     %r8d, %r8d               # word index
.sha_hex_loop:
    cmp     $8, %r8d
    jge     .sha_hex_done
    mov     (%rbx, %r8, 4), %eax
    # State words stored as native u32, already correct order — NO bswap needed
    # Emit 8 hex nibbles
    mov     $8, %r9d                  # nibble count
.sha_nibble:
    test    %r9d, %r9d
    jz      .sha_next_word
    dec     %r9d
    mov     %eax, %edx
    shr     $28, %edx                # top nibble
    shl     $4, %eax                 # shift left
    # Convert nibble to ASCII hex
    cmp     $10, %edx
    jl      .sha_digit
    add     $87, %edx                # 'a' - 10 = 87
    jmp     .sha_emit_hex
.sha_digit:
    add     $48, %edx                # '0' = 48
.sha_emit_hex:
    # Emit as u16 molecule: 0x2100 | byte
    or      $0x2100, %edx
    movw    %dx, (%r15)
    add     $2, %r15
    inc     %rcx
    jmp     .sha_nibble
.sha_next_word:
    inc     %r8d
    jmp     .sha_hex_loop

.sha_hex_done:
    # Push result string onto VM stack
    sub     $16, %r14
    mov     %rdi, (%r14)             # string ptr
    mov     %rcx, 8(%r14)            # mol count (64)
    jmp     vm_loop

.call_file_read:
    add     %rcx, %r13
    jmp     op_file_read

.call_file_write:
    add     %rcx, %r13
    jmp     op_file_write

# ── Raw byte buffer builtins ────────────────────────────────────────
# Bytes buffers: stored as [size:8][raw_bytes:N] on heap
# VM stack entry: ptr = heap_base, len = BYTES_MARKER (-5)
.equ BYTES_MARKER, -5

.call_bytes_new:
    # __bytes_new(size) → byte buffer
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rax          # size
    add     $16, %r14               # pop size
    # Allocate on heap: [size:8][zeroed bytes]
    mov     %r15, %rdi              # buffer base
    mov     %rax, (%r15)            # store size
    add     $8, %r15                # past size header
    # Zero the buffer
    push    %rdi                    # save buf base
    mov     %rax, %rcx              # count = size
    push    %rcx                    # save size
    mov     %r15, %rdi              # dest = data start
    xor     %eax, %eax              # AL = 0 (rep stosb stores AL)
    rep     stosb
    pop     %rax                    # restore size
    pop     %rdi                    # restore buf base
    add     %rax, %r15              # bump heap past buffer
    # Push result
    sub     $16, %r14
    mov     %rdi, (%r14)            # ptr = buffer base
    movq    $BYTES_MARKER, 8(%r14)  # marker
    jmp     vm_loop

.call_bytes_set:
    # __bytes_set(buf, offset, value) → buf
    add     %rcx, %r13
    # Pop value (f64)
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rax          # byte value
    # Pop offset (f64)
    movsd   16(%r14), %xmm1
    cvttsd2si %xmm1, %rdx          # offset
    # Pop buf
    mov     32(%r14), %rdi          # buf ptr
    add     $48, %r14               # pop 3
    # Write byte at buf + 8 + offset
    mov     %al, 8(%rdi, %rdx)
    # Push buf back
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $BYTES_MARKER, 8(%r14)
    jmp     vm_loop

.call_bytes_get:
    # __bytes_get(buf, offset) → f64 byte value
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rdx          # offset
    mov     16(%r14), %rdi          # buf ptr
    add     $16, %r14               # pop offset (keep buf slot for result)
    movzbl  8(%rdi, %rdx), %eax    # read byte
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r14)          # overwrite buf with result
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_bytes_write_raw:
    # __bytes_write(path, buf, size) → 0
    add     %rcx, %r13
    # Pop size
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %r9           # size
    # Pop buf
    mov     16(%r14), %r8           # buf ptr
    # Pop path (string)
    mov     32(%r14), %rsi          # path ptr (u16 mols)
    mov     40(%r14), %rdx          # path len
    add     $48, %r14               # pop 3
    # Decode path → ASCII
    lea     sha_input(%rip), %rdi
    xor     %rcx, %rcx
.bw_decode_path:
    cmp     %rdx, %rcx
    jge     .bw_open
    movzbl  (%rsi, %rcx, 2), %eax
    mov     %al, (%rdi, %rcx)
    inc     %rcx
    jmp     .bw_decode_path
.bw_open:
    movb    $0, (%rdi, %rcx)
    lea     sha_input(%rip), %rdi
    mov     $0x241, %esi            # O_WRONLY|O_CREAT|O_TRUNC
    mov     $0755, %edx             # permissions (executable)
    mov     $SYS_OPEN, %eax
    syscall
    test    %rax, %rax
    js      .bw_fail
    # Write raw bytes from buf+8
    mov     %rax, %rdi              # fd
    lea     8(%r8), %rsi            # data = buf + 8 (skip size header)
    mov     %r9, %rdx              # size
    mov     $SYS_WRITE, %eax
    syscall
    # Close
    mov     $SYS_CLOSE, %eax
    syscall
.bw_fail:
    # Push 0 result
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_file_read_bytes:
    # __file_read_bytes(path) → bytes buffer (raw, not u16 encoded)
    add     %rcx, %r13
    mov     (%r14), %rsi            # path ptr (u16 mols)
    mov     8(%r14), %rdx           # path len
    add     $16, %r14               # pop
    # Decode path
    lea     sha_input(%rip), %rdi
    xor     %rcx, %rcx
.frb_decode:
    cmp     %rdx, %rcx
    jge     .frb_open
    movzbl  (%rsi, %rcx, 2), %eax
    mov     %al, (%rdi, %rcx)
    inc     %rcx
    jmp     .frb_decode
.frb_open:
    movb    $0, (%rdi, %rcx)
    lea     sha_input(%rip), %rdi
    xor     %esi, %esi              # O_RDONLY
    xor     %edx, %edx
    mov     $SYS_OPEN, %eax
    syscall
    test    %rax, %rax
    js      .frb_empty
    # Read into heap as bytes buffer [size:8][data:N] — loop for large files
    mov     %rax, %r8               # fd
    mov     %r15, %rdi              # buf base
    add     $8, %r15                # skip size header
    xor     %r9d, %r9d              # r9 = total bytes read
.frb_read_loop:
    mov     %r8, %rdi               # fd
    lea     (%r15, %r9), %rsi       # buf = data_start + total_read
    mov     $0x100000, %edx         # read 1MB chunks
    mov     $SYS_READ, %eax
    syscall
    test    %rax, %rax
    jle     .frb_read_done
    add     %rax, %r9
    cmp     $0x100000, %rax
    je      .frb_read_loop
.frb_read_done:
    mov     %r8, %rdi
    mov     $SYS_CLOSE, %eax
    syscall
    mov     %r9, %rcx               # total bytes_read
    test    %rcx, %rcx
    jle     .frb_empty
    # Set size header
    mov     %r15, %rdi
    sub     $8, %rdi                # buf base
    mov     %rcx, (%rdi)            # size
    add     %rcx, %r15              # bump heap past data
    # Push
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $BYTES_MARKER, 8(%r14)
    jmp     vm_loop
.frb_empty:
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

.call_heap_save:
    # __heap_save() → stores r15 in BSS, returns index as f64
    add     %rcx, %r13
    lea     heap_checkpoints(%rip), %rax
    lea     heap_cp_count(%rip), %rdx
    mov     (%rdx), %rcx               # current count
    mov     %r15, (%rax, %rcx, 8)      # store r15 at checkpoints[count]
    inc     %rcx
    mov     %rcx, (%rdx)               # count++
    dec     %rcx                        # return index (before increment)
    sub     $16, %r14
    cvtsi2sd %rcx, %xmm0              # index as f64 (small number, safe)
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_heap_restore:
    # __heap_restore(index) → resets r15 from saved checkpoint
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %rax              # index
    add     $16, %r14                   # pop
    lea     heap_checkpoints(%rip), %rdx
    mov     (%rdx, %rax, 8), %r15      # restore r15
    # Push dummy result
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_bytes_len:
    # __bytes_len(buf) → f64 size
    add     %rcx, %r13
    mov     (%r14), %rdi            # buf ptr
    mov     (%rdi), %rax            # size from header
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_floor:
    # __floor(x) → floor(x)
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    roundsd $1, %xmm0, %xmm0          # round toward -inf
    movsd   %xmm0, (%r14)
    jmp     vm_loop

.call_ceil:
    # __ceil(x) → ceil(x)
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    roundsd $2, %xmm0, %xmm0          # round toward +inf
    movsd   %xmm0, (%r14)
    jmp     vm_loop

# ── Molecule extract builtins (T5 ND.2) ────────────────────────────────
# P_weight = [S:4][R:4][V:3][A:3][T:2] = 16 bits packed as f64
# __mol_s(mol) → (mol / 4096) % 16     (bits 12-15)
# __mol_r(mol) → (mol / 256) % 16      (bits 8-11)
# __mol_v(mol) → (mol / 32) % 8        (bits 5-7)
# __mol_a(mol) → (mol / 4) % 8         (bits 2-4)
# __mol_t(mol) → mol % 4               (bits 0-1)
# __mol_pack(s,r,v,a,t) → s*4096 + r*256 + v*32 + a*4 + t

.call_mol_s:
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %eax
    shr     $12, %eax
    and     $0x0F, %eax
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_mol_r:
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %eax
    shr     $8, %eax
    and     $0x0F, %eax
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_mol_v:
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %eax
    shr     $5, %eax
    and     $0x07, %eax
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_mol_a:
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %eax
    shr     $2, %eax
    and     $0x07, %eax
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_mol_t:
    add     %rcx, %r13
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %eax
    and     $0x03, %eax
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_mol_pack:
    # Stack: [s, r, v, a, t] → s*4096 + r*256 + v*32 + a*4 + t
    add     %rcx, %r13
    # Pop t
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %eax              # t
    add     $16, %r14
    # Pop a
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %edx              # a
    shl     $2, %edx                    # a*4
    add     %edx, %eax
    add     $16, %r14
    # Pop v
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %edx              # v
    shl     $5, %edx                    # v*32
    add     %edx, %eax
    add     $16, %r14
    # Pop r
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %edx              # r
    shl     $8, %edx                    # r*256
    add     %edx, %eax
    add     $16, %r14
    # Pop s
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %edx              # s
    shl     $12, %edx                   # s*4096
    add     %edx, %eax
    # Push result as f64
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# ── UTF-8 decoding builtins ──────────────────────────────────────────────
# __utf8_cp(str, pos) → Unicode codepoint at byte position pos
# __utf8_len(str, pos) → number of bytes in UTF-8 sequence (1-4)
# Strings are u16 molecules: each byte = 0x2100 | byte_value

.call_utf8_cp:
    add     %rcx, %r13
    # Pop pos (f64)
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %edx              # pos
    add     $16, %r14
    # Pop str (chain ptr + mol_count)
    mov     (%r14), %rsi               # str ptr (u16 molecules)
    # Read byte at pos: molecule = 0x2100 | byte, so byte = mol & 0xFF
    movzwl  (%rsi, %rdx, 2), %eax     # read u16 molecule at pos
    and     $0xFF, %eax                # extract byte value
    # Check UTF-8 lead byte
    cmp     $0x80, %eax
    jb      .utf8_cp_1byte             # 0xxxxxxx → 1 byte (ASCII)
    cmp     $0xE0, %eax
    jb      .utf8_cp_2byte             # 110xxxxx → 2 bytes
    cmp     $0xF0, %eax
    jb      .utf8_cp_3byte             # 1110xxxx → 3 bytes
    # 4 byte: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
    and     $0x07, %eax                # keep low 3 bits
    shl     $18, %eax
    inc     %edx
    movzwl  (%rsi, %rdx, 2), %ecx
    and     $0x3F, %ecx
    shl     $12, %ecx
    or      %ecx, %eax
    inc     %edx
    movzwl  (%rsi, %rdx, 2), %ecx
    and     $0x3F, %ecx
    shl     $6, %ecx
    or      %ecx, %eax
    inc     %edx
    movzwl  (%rsi, %rdx, 2), %ecx
    and     $0x3F, %ecx
    or      %ecx, %eax
    jmp     .utf8_cp_done
.utf8_cp_3byte:
    and     $0x0F, %eax                # keep low 4 bits
    shl     $12, %eax
    inc     %edx
    movzwl  (%rsi, %rdx, 2), %ecx
    and     $0x3F, %ecx
    shl     $6, %ecx
    or      %ecx, %eax
    inc     %edx
    movzwl  (%rsi, %rdx, 2), %ecx
    and     $0x3F, %ecx
    or      %ecx, %eax
    jmp     .utf8_cp_done
.utf8_cp_2byte:
    and     $0x1F, %eax                # keep low 5 bits
    shl     $6, %eax
    inc     %edx
    movzwl  (%rsi, %rdx, 2), %ecx
    and     $0x3F, %ecx
    or      %ecx, %eax
    jmp     .utf8_cp_done
.utf8_cp_1byte:
    # eax already has the byte value (ASCII)
.utf8_cp_done:
    # Push codepoint as f64
    cvtsi2sd %eax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_utf8_len:
    add     %rcx, %r13
    # Pop pos (f64)
    movsd   (%r14), %xmm0
    cvttsd2si %xmm0, %edx              # pos
    add     $16, %r14
    # Pop str
    mov     (%r14), %rsi               # str ptr
    movzwl  (%rsi, %rdx, 2), %eax     # read u16 molecule at pos
    and     $0xFF, %eax                # byte value
    # Return byte count based on lead byte
    mov     $1, %ecx                    # default 1
    cmp     $0x80, %eax
    jb      .utf8_len_done
    mov     $2, %ecx
    cmp     $0xE0, %eax
    jb      .utf8_len_done
    mov     $3, %ecx
    cmp     $0xF0, %eax
    jb      .utf8_len_done
    mov     $4, %ecx
.utf8_len_done:
    cvtsi2sd %ecx, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# ── __sleep(ms) — nanosleep syscall ──────────────────────────────────────
.call_sleep:
    add     %rcx, %r13                  # skip name_len
    # Pop ms (f64) from VM stack
    movsd   (%r14), %xmm0
    add     $16, %r14                   # pop
    # Convert ms → seconds + nanoseconds
    cvttsd2si %xmm0, %rax              # ms as integer
    # rax = total ms, compute sec = ms / 1000, nsec = (ms % 1000) * 1_000_000
    xor     %edx, %edx
    mov     $1000, %rcx
    div     %rcx                        # rax=sec, rdx=remainder_ms
    imul    $1000000, %rdx, %rdx        # remainder → nanoseconds
    # Build timespec on CPU stack (16 bytes aligned)
    sub     $16, %rsp
    mov     %rax, (%rsp)               # tv_sec
    mov     %rdx, 8(%rsp)             # tv_nsec
    # syscall nanosleep(timespec*, NULL)
    mov     %rsp, %rdi                  # rdi = &timespec
    xor     %esi, %esi                  # rsi = NULL (remaining)
    mov     $SYS_NANOSLEEP, %eax
    syscall
    add     $16, %rsp                   # restore stack
    # Push 0.0 (success)
    sub     $16, %r14
    xorpd   %xmm0, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# ── __time() — clock_gettime CLOCK_MONOTONIC → ms ──────────────────────
.call_time:
    add     %rcx, %r13                  # skip name_len
    # Build timespec on CPU stack
    sub     $16, %rsp
    mov     $CLOCK_MONOTONIC, %edi      # clock_id
    mov     %rsp, %rsi                  # &timespec
    mov     $SYS_CLOCK_GETTIME, %eax
    syscall
    # Convert to milliseconds: sec*1000 + nsec/1_000_000
    mov     (%rsp), %rax               # tv_sec
    imul    $1000, %rax, %rax           # sec → ms
    mov     8(%rsp), %rdx             # tv_nsec
    mov     $1000000, %rcx
    xor     %r8d, %r8d                  # clear for div
    push    %rax                        # save sec_ms
    mov     %rdx, %rax
    xor     %edx, %edx
    div     %rcx                        # rax = nsec/1000000
    pop     %rdx                        # rdx = sec_ms
    add     %rdx, %rax                  # total ms
    add     $16, %rsp                   # restore stack
    # Push result as f64
    sub     $16, %r14
    cvtsi2sd %rax, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

# ── __write_raw(str) — write raw bytes to stdout (for ANSI escapes) ────
.call_write_raw:
    add     %rcx, %r13                  # skip name_len
    # Pop string from VM stack
    mov     (%r14), %rsi               # ptr (u16 molecules)
    mov     8(%r14), %rdx              # len (mol count)
    add     $16, %r14                   # pop
    # Decode u16 molecules → raw bytes in temp buffer on CPU stack
    # Each molecule = 0x2100 | byte, stride 2 bytes
    # Max reasonable: 256 bytes for ANSI sequences
    mov     %rdx, %rcx                 # mol_count
    sub     $256, %rsp                  # temp buffer
    mov     %rsp, %rdi                  # dest ptr
    xor     %r8d, %r8d                  # byte count
.wr_decode:
    test    %rcx, %rcx
    jz      .wr_write
    movzwl  (%rsi), %eax               # read u16 molecule
    and     $0xFF, %eax                 # extract byte (mask off 0x2100)
    movb    %al, (%rdi, %r8)
    inc     %r8
    add     $2, %rsi                    # next molecule
    dec     %rcx
    jmp     .wr_decode
.wr_write:
    # SYS_WRITE(fd=1, buf=rsp, len=r8)
    mov     $1, %edi                    # stdout
    mov     %rsp, %rsi                  # buffer
    mov     %r8, %rdx                   # byte count
    mov     $SYS_WRITE, %eax
    syscall
    add     $256, %rsp                  # restore stack
    # Push 0.0
    sub     $16, %r14
    xorpd   %xmm0, %xmm0
    movsd   %xmm0, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.call_throw:
    # __throw(msg) — unwind to nearest try/catch
    # Stack: [msg_value] → store in __error var, jump to catch_PC
    add     %rcx, %r13                  # skip name_len
    # Pop error message (keep on stack for catch block to access)
    mov     (%r14), %r8                # msg ptr
    mov     8(%r14), %r9               # msg len
    add     $16, %r14                   # pop

    # Store error message in __error variable (hash of "__error")
    movabs  $0x07515EC1BC3E1099, %rdx   # temp: store as "__error" var
    # Actually, use the var_table to store error. For now, just jump to catch.

    # Check try_depth
    lea     try_depth(%rip), %rsi
    cmpq    $0, (%rsi)
    je      .throw_no_handler

    # Pop try frame
    decq    (%rsi)                      # depth--
    mov     (%rsi), %rdx               # new depth
    lea     try_stack(%rip), %rdi
    shl     $4, %rdx                    # depth * 16
    mov     (%rdi, %rdx), %rax         # catch_PC
    mov     8(%rdi, %rdx), %r14        # restore stack pointer

    # Push error message onto restored stack (catch block can access it)
    sub     $16, %r14
    mov     %r8, (%r14)
    mov     %r9, 8(%r14)

    # Jump to catch_PC
    mov     %rax, %r13                  # set PC = catch_PC
    jmp     vm_loop

.throw_no_handler:
    # No try block — print error and halt
    # Print "Error: " prefix
    lea     .err_prefix(%rip), %rsi
    mov     $7, %rdx
    mov     $1, %rdi
    mov     $1, %rax
    syscall
    # Print error message
    mov     %r8, %rsi
    mov     %r9, %rdx
    cmp     $F64_MARKER, %rdx
    je      .throw_halt                 # skip if f64 (can't print number easily)
    test    %rdx, %rdx
    jz      .throw_halt
    # Check if string (0x21xx pattern)
    movzwl  (%rsi), %eax
    and     $0xFF00, %eax
    cmp     $0x2100, %eax
    jne     .throw_halt
    # Decode u16 string
    mov     %r9, %rcx                   # mol count
    lea     throw_buf(%rip), %rdi
    xor     %rdx, %rdx
.throw_decode:
    cmp     %rcx, %rdx
    jge     .throw_write
    movzbl  (%rsi, %rdx, 2), %eax
    mov     %al, (%rdi, %rdx)
    inc     %rdx
    jmp     .throw_decode
.throw_write:
    lea     throw_buf(%rip), %rsi
    # rdx = length
    mov     $1, %rdi
    mov     $1, %rax
    syscall
.throw_halt:
    # Print newline and halt
    lea     str_newline(%rip), %rsi
    mov     $1, %rdx
    mov     $1, %rdi
    mov     $1, %rax
    syscall
    mov     $60, %rax
    mov     $1, %rdi
    syscall

.section .rodata
.err_prefix:
    .ascii  "Error: "
.align 4
sha_k:
    .long 0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5
    .long 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5
    .long 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3
    .long 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174
    .long 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc
    .long 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da
    .long 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7
    .long 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967
    .long 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13
    .long 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85
    .long 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3
    .long 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070
    .long 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5
    .long 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3
    .long 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208
    .long 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
.section .text

op_try_begin:
    # [0x1A, catch_offset_u32_le]
    # Push (catch_PC, saved_r14) onto try_stack
    # r13 already points past opcode byte
    movzbl  (%r12, %r13), %eax         # read catch_offset byte 0
    movzbl  1(%r12, %r13), %edx
    shl     $8, %edx
    or      %edx, %eax
    movzbl  2(%r12, %r13), %edx
    shl     $16, %edx
    or      %edx, %eax
    movzbl  3(%r12, %r13), %edx
    shl     $24, %edx
    or      %edx, %eax                  # eax = catch_offset (absolute PC)
    add     $4, %r13

    # Push onto try_stack
    lea     try_depth(%rip), %rsi
    mov     (%rsi), %rdx               # depth
    lea     try_stack(%rip), %rdi
    shl     $4, %rdx                    # depth * 16
    lea     (%rdi, %rdx), %rdi         # try_stack[depth]
    mov     %rax, (%rdi)               # catch_PC
    mov     %r14, 8(%rdi)              # saved stack pointer
    lea     try_depth(%rip), %rsi
    incq    (%rsi)                      # depth++
    jmp     vm_loop

op_catch_end:
    # Pop try_stack (catch block finished)
    lea     try_depth(%rip), %rsi
    cmpq    $0, (%rsi)
    je      vm_loop                     # no try frame → skip
    decq    (%rsi)                      # depth--
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

    # Array/dict: print type indicator
    cmp     $ARRAY_MARKER, %rdx
    je      .emit_array
    cmp     $DICT_MARKER, %rdx
    je      .emit_dict

    # If len == 0, skip
    test    %rdx, %rdx
    jz      vm_loop

    # Check if this is a string chain (molecules with 0x21xx pattern)
    # First molecule top byte = 0x21 → string encoded
    movzwl  (%rsi), %eax
    and     $0xFF00, %eax
    cmp     $0x2100, %eax
    jne     .emit_raw

    # String chain: decode molecules to ASCII bytes on heap, then print
    push    %rsi
    push    %rdx
    mov     %r15, %rdi                  # decode buffer = heap
    mov     %rdx, %rcx                  # mol_count
.emit_decode_loop:
    test    %rcx, %rcx
    jz      .emit_decode_done
    movzwl  (%rsi), %eax               # read u16 molecule
    movb    %al, (%rdi)                 # store lower byte (ASCII value)
    add     $2, %rsi                    # next molecule
    inc     %rdi                        # next output byte
    dec     %rcx
    jmp     .emit_decode_loop
.emit_decode_done:
    pop     %rdx                        # mol_count = string length
    pop     %rsi
    mov     %r15, %rsi                  # buf = decoded bytes
    mov     $1, %edi                    # stdout
    mov     $SYS_WRITE, %eax
    syscall
    jmp     .emit_newline

.emit_raw:
    # Raw chain bytes — write directly
    mov     $1, %edi                   # fd = stdout
    mov     $SYS_WRITE, %eax
    syscall

.emit_newline:
    # Print newline after emit
    lea     str_newline(%rip), %rsi
    mov     $1, %edx
    mov     $1, %edi
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
    # Print newline
    lea     str_newline(%rip), %rsi
    mov     $1, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     vm_loop

.emit_array:
    # Print "[elem0, elem1, ...]" with actual contents
    # rsi = array base ptr
    test    %rsi, %rsi
    jz      .emit_array_empty
    # Follow forward pointer chain
.emit_array_fwd:
    cmpq    $ARRAY_FORWARD_MARKER, (%rsi)
    jne     .emit_array_start
    mov     8(%rsi), %rsi
    jmp     .emit_array_fwd
.emit_array_start:
    push    %rbp                        # save rbp (callee-saved, used as elem ptr)
    push    %rsi                        # save array base
    # Print "["
    lea     str_lbracket(%rip), %rsi
    mov     $1, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rsi                        # restore array base
    mov     (%rsi), %rcx               # count
    test    %rcx, %rcx
    jz      .emit_array_close
    lea     8(%rsi), %rbp              # first element ptr
    xor     %eax, %eax                  # index = 0
    push    %rcx                        # save count
.emit_array_elem:
    push    %rax                        # save index
    push    %rbp                        # save element ptr
    # Check element type
    mov     8(%rbp), %rdx              # elem len/marker
    cmp     $F64_MARKER, %rdx
    je      .emit_array_f64
    # String element: decode u16 molecules
    mov     (%rbp), %rsi               # string ptr
    test    %rdx, %rdx
    jz      .emit_array_elem_done
    # Decode molecules to ASCII on heap
    push    %rdx
    mov     %r15, %rdi
    mov     %rdx, %rcx
.emit_arr_str_decode:
    test    %rcx, %rcx
    jz      .emit_arr_str_write
    movzwl  (%rsi), %eax
    movb    %al, (%rdi)
    add     $2, %rsi
    inc     %rdi
    dec     %rcx
    jmp     .emit_arr_str_decode
.emit_arr_str_write:
    pop     %rdx                        # mol_count = char count
    mov     %r15, %rsi
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     .emit_array_elem_done
.emit_array_f64:
    movq    (%rbp), %xmm0             # f64 bits
    call    f64_to_ascii               # → eax = len, output at r15
    mov     %r15, %rsi
    mov     %eax, %edx
    mov     $1, %edi
    push    %rax
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rax
    add     %rax, %r15
.emit_array_elem_done:
    pop     %rbp                        # restore element ptr
    pop     %rax                        # restore index
    inc     %rax
    add     $16, %rbp                   # next element
    # Check if more elements
    mov     (%rsp), %rcx              # peek count from stack
    cmp     %rcx, %rax
    jae     .emit_array_close_pop
    # Print ", "
    push    %rax
    push    %rbp
    lea     str_comma_space(%rip), %rsi
    mov     $2, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rbp
    pop     %rax
    jmp     .emit_array_elem
.emit_array_close_pop:
    pop     %rcx                        # pop saved count
.emit_array_close:
    lea     str_rbracket_nl(%rip), %rsi
    mov     $2, %edx                    # "]\n"
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rbp                        # restore rbp
    jmp     vm_loop
.emit_array_empty:
    lea     str_empty_array(%rip), %rsi
    mov     $4, %edx                    # "[]\n"
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     vm_loop

.emit_dict:
    # Pretty-print: {key1: val1, key2: val2}
    test    %rsi, %rsi
    jz      .emit_dict_empty
    push    %rbp
    mov     %rsi, %rbp                  # rbp = dict base
    # Print "{"
    lea     str_lbrace(%rip), %rsi
    mov     $1, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    mov     (%rbp), %rcx               # count
    test    %rcx, %rcx
    jz      .emit_dict_close
    lea     8(%rbp), %rbp              # first entry
    xor     %eax, %eax                  # index = 0
    push    %rcx                        # save count
.emit_dict_entry:
    push    %rax                        # save index
    push    %rbp                        # save entry ptr
    # Comma separator (if not first)
    test    %eax, %eax
    jz      .emit_dict_nocomma
    lea     str_comma_sp(%rip), %rsi
    mov     $2, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
.emit_dict_nocomma:
    pop     %rbp                        # restore entry ptr
    pop     %rax                        # restore index
    push    %rax
    push    %rbp
    # Print key (string at entry+0, len at entry+8)
    mov     (%rbp), %rsi               # key ptr
    mov     8(%rbp), %rdx              # key len
    test    %rdx, %rdx
    jz      .emit_dict_key_done
    # Decode u16 molecules to ASCII
    push    %rdx
    mov     %r15, %rdi
    mov     %rdx, %rcx
.emit_dk_loop:
    test    %rcx, %rcx
    jz      .emit_dk_done
    movzwl  (%rsi), %eax
    movb    %al, (%rdi)
    add     $2, %rsi
    inc     %rdi
    dec     %rcx
    jmp     .emit_dk_loop
.emit_dk_done:
    pop     %rdx
    mov     %r15, %rsi
    mov     $1, %edi
    push    %rdx
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rdx
.emit_dict_key_done:
    # Print ": "
    lea     str_colon_sp(%rip), %rsi
    mov     $2, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    # Print value (entry+16=val_ptr, entry+24=val_len)
    pop     %rbp
    pop     %rax
    push    %rax
    push    %rbp
    mov     24(%rbp), %rdx             # val len/marker
    cmp     $F64_MARKER, %rdx
    je      .emit_dict_val_f64
    # String value: decode u16
    mov     16(%rbp), %rsi             # val ptr
    test    %rdx, %rdx
    jz      .emit_dict_val_done
    push    %rdx
    mov     %r15, %rdi
    mov     %rdx, %rcx
.emit_dv_loop:
    test    %rcx, %rcx
    jz      .emit_dv_done
    movzwl  (%rsi), %eax
    movb    %al, (%rdi)
    add     $2, %rsi
    inc     %rdi
    dec     %rcx
    jmp     .emit_dv_loop
.emit_dv_done:
    pop     %rdx
    mov     %r15, %rsi
    mov     $1, %edi
    push    %rdx
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rdx
    jmp     .emit_dict_val_done
.emit_dict_val_f64:
    mov     16(%rbp), %rax
    movq    %rax, %xmm0
    call    f64_to_ascii
    mov     %r15, %rsi
    mov     %eax, %edx
    mov     $1, %edi
    push    %rax
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rax
.emit_dict_val_done:
    pop     %rbp                        # restore entry ptr
    pop     %rax                        # restore index
    add     $32, %rbp                   # next entry (32 bytes each)
    inc     %eax
    mov     (%rsp), %rcx              # peek count
    cmp     %ecx, %eax
    jb      .emit_dict_entry
    add     $8, %rsp                    # pop count
.emit_dict_close:
    pop     %rbp
    lea     str_brace_nl(%rip), %rsi
    mov     $2, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     vm_loop
.emit_dict_empty:
    lea     str_empty_dict(%rip), %rsi
    mov     $4, %edx
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
    # Pop filename string, read file contents, push as string
    # Stack: [filename_ptr, filename_len] → [content_ptr, content_len]
    mov     (%r14), %rsi            # filename ptr (u16 molecules)
    mov     8(%r14), %rdx           # filename len (mol count)
    add     $16, %r14               # pop

    # Decode u16 filename → ASCII bytes in temp buffer
    lea     sha_input(%rip), %rdi   # reuse sha_input as temp (1024 bytes)
    xor     %rcx, %rcx
    test    %rdx, %rdx
    jz      .fread_empty
.fread_decode:
    cmp     %rdx, %rcx
    jge     .fread_open
    movzbl  (%rsi, %rcx, 2), %eax
    mov     %al, (%rdi, %rcx)
    inc     %rcx
    jmp     .fread_decode

.fread_open:
    movb    $0, (%rdi, %rcx)        # null-terminate
    # Open file
    lea     sha_input(%rip), %rdi   # filename
    xor     %esi, %esi              # O_RDONLY
    xor     %edx, %edx              # mode 0
    mov     $SYS_OPEN, %eax
    syscall
    test    %rax, %rax
    js      .fread_empty             # open failed → push empty

    # Read file into heap — loop until EOF (supports files > 1MB)
    mov     %rax, %r8               # save fd
    xor     %r9d, %r9d              # r9 = total bytes read
.fread_loop:
    mov     %r8, %rdi               # fd
    lea     (%r15, %r9), %rsi       # buf = heap + total_read
    mov     $0x100000, %edx         # read 1MB chunks
    mov     $SYS_READ, %eax
    syscall
    test    %rax, %rax
    jle     .fread_done             # EOF or error
    add     %rax, %r9               # total += bytes_read
    cmp     $0x100000, %rax         # got full chunk?
    je      .fread_loop             # yes → more data
.fread_done:
    # Close file
    mov     %r8, %rdi
    mov     $SYS_CLOSE, %eax
    syscall

    mov     %r9, %rcx               # total bytes_read
    test    %rcx, %rcx
    jle     .fread_empty

    # Encode bytes → u16 molecules on heap (after raw bytes)
    mov     %r15, %r8               # raw bytes base
    add     %rcx, %r15              # bump past raw bytes
    mov     %r15, %rdi              # u16 output base
    xor     %rdx, %rdx
.fread_encode:
    cmp     %rcx, %rdx
    jge     .fread_push
    movzbl  (%r8, %rdx), %eax
    or      $0x2100, %eax
    movw    %ax, (%r15)
    add     $2, %r15
    inc     %rdx
    jmp     .fread_encode

.fread_push:
    # Push string (ptr=u16 base, len=mol count)
    sub     $16, %r14
    mov     %rdi, (%r14)            # u16 base
    mov     %rcx, 8(%r14)           # mol count = byte count
    jmp     vm_loop

.fread_empty:
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop

op_file_write:
op_file_append:
    # Pop content, pop filename, write to file
    # Stack: [filename, content]
    mov     (%r14), %r8             # content ptr
    mov     8(%r14), %r9            # content len
    mov     16(%r14), %rsi          # filename ptr
    mov     24(%r14), %rdx          # filename len
    add     $32, %r14               # pop both

    # Decode filename → ASCII
    lea     sha_input(%rip), %rdi
    xor     %rcx, %rcx
.fwrite_fname:
    cmp     %rdx, %rcx
    jge     .fwrite_open
    movzbl  (%rsi, %rcx, 2), %eax
    mov     %al, (%rdi, %rcx)
    inc     %rcx
    jmp     .fwrite_fname
.fwrite_open:
    movb    $0, (%rdi, %rcx)
    # Open/create file
    lea     sha_input(%rip), %rdi
    mov     $0x241, %esi            # O_WRONLY|O_CREAT|O_TRUNC
    mov     $0644, %edx             # permissions
    mov     $SYS_OPEN, %eax
    syscall
    test    %rax, %rax
    js      vm_loop                  # open failed

    # Decode content u16 → bytes, write
    push    %rax                    # save fd
    lea     sha_input(%rip), %rdi   # reuse buffer for decoded content
    xor     %rcx, %rcx
.fwrite_decode:
    cmp     %r9, %rcx
    jge     .fwrite_do
    movzbl  (%r8, %rcx, 2), %eax
    mov     %al, (%rdi, %rcx)
    inc     %rcx
    jmp     .fwrite_decode
.fwrite_do:
    pop     %rdi                    # fd
    push    %rcx                    # save len
    lea     sha_input(%rip), %rsi   # buf
    pop     %rdx                    # len
    push    %rdi                    # save fd
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rdi                    # fd
    mov     $SYS_CLOSE, %eax
    syscall
    # Push dummy result (builtin must leave value on stack for ExprStmt Pop)
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
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
    movsd   16(%r14), %xmm0            # B (deeper)
    movsd   (%r14), %xmm1              # A (top)
    add     $16, %r14                   # pop 1, result overwrites remaining
    ucomisd %xmm1, %xmm0
    je      .cmp_true
    jmp     .cmp_false

op_cmp_lt:
    movb    $0, %al                     # op_id = 0 (LT)
    jmp     .cmp_dispatch
op_cmp_gt:
    movb    $1, %al                     # op_id = 1 (GT)
    jmp     .cmp_dispatch
op_cmp_le:
    movb    $2, %al                     # op_id = 2 (LE)
    jmp     .cmp_dispatch
op_cmp_ge:
    movb    $3, %al                     # op_id = 3 (GE)

.cmp_dispatch:
    # Check if both are f64
    push    %rax                        # save op_id
    mov     8(%r14), %rax              # b len
    mov     24(%r14), %rcx             # a len
    cmp     $F64_MARKER, %rax
    jne     .cmp_chain
    cmp     $F64_MARKER, %rcx
    jne     .cmp_chain

    # Both f64 — compare as floating point
    movsd   16(%r14), %xmm0           # a
    movsd   (%r14), %xmm1             # b
    add     $16, %r14                  # pop 1
    pop     %rax                       # restore op_id
    # Dispatch op_id FIRST, then ucomisd (to avoid clobbering flags)
    test    %al, %al
    jz      .cmp_f64_lt
    cmp     $1, %al
    je      .cmp_f64_gt
    cmp     $2, %al
    je      .cmp_f64_le
    # op_id == 3: GE
    ucomisd %xmm1, %xmm0
    jae     .cmp_true
    jmp     .cmp_false
.cmp_f64_lt:
    ucomisd %xmm1, %xmm0
    jb      .cmp_true
    jmp     .cmp_false
.cmp_f64_gt:
    ucomisd %xmm1, %xmm0
    ja      .cmp_true
    jmp     .cmp_false
.cmp_f64_le:
    ucomisd %xmm1, %xmm0
    jbe     .cmp_true
    jmp     .cmp_false

.cmp_chain:
    # At least one is a chain — compare by low bytes of molecules
    # a: 16(%r14)=ptr, 24(%r14)=mol_count
    # b: (%r14)=ptr, 8(%r14)=mol_count
    mov     (%r14), %rsi               # b ptr
    mov     8(%r14), %rdx              # b mol_count (might be F64_MARKER)
    mov     16(%r14), %rdi             # a ptr
    mov     24(%r14), %rcx             # a mol_count (might be F64_MARKER)
    add     $16, %r14                  # pop 1

    # If either has F64_MARKER, treat as len=0 for comparison (type mismatch)
    cmp     $F64_MARKER, %rdx
    jne     .cmp_chain_b_ok
    xor     %edx, %edx                 # b len = 0
.cmp_chain_b_ok:
    cmp     $F64_MARKER, %rcx
    jne     .cmp_chain_a_ok
    xor     %ecx, %ecx                 # a len = 0
.cmp_chain_a_ok:
    # Compare molecule by molecule (low byte = char value)
    # Use min(a_len, b_len) for loop count
    mov     %rcx, %r8                  # save a_len
    mov     %rdx, %r9                  # save b_len
    cmp     %rdx, %rcx
    jbe     .cmp_chain_use_a
    mov     %rdx, %rcx                 # min_len = b_len
.cmp_chain_use_a:
    # rcx = min_len, rdi = a_ptr, rsi = b_ptr
    test    %rcx, %rcx
    jz      .cmp_chain_len             # both empty prefix → compare lengths
.cmp_chain_loop:
    movzbl  (%rdi), %eax               # a low byte (char value)
    movzbl  (%rsi), %edx               # b low byte (char value)
    cmp     %edx, %eax
    jb      .cmp_chain_a_less
    ja      .cmp_chain_a_greater
    add     $2, %rdi                   # next molecule (u16)
    add     $2, %rsi
    dec     %rcx
    jnz     .cmp_chain_loop

.cmp_chain_len:
    # All compared bytes equal — compare lengths
    cmp     %r9, %r8                   # a_len vs b_len
    jb      .cmp_chain_a_less
    ja      .cmp_chain_a_greater
    # Equal: a_len == b_len and all bytes match
    pop     %rax                       # op_id
    cmp     $2, %al                    # LE or GE → true for equal
    jae     .cmp_true
    jmp     .cmp_false                 # LT or GT → false for equal

.cmp_chain_a_less:
    pop     %rax                       # op_id
    test    %al, %al                   # LT (0)?
    jz      .cmp_true
    cmp     $2, %al                    # LE (2)?
    je      .cmp_true
    jmp     .cmp_false                 # GT (1) or GE (3) → false

.cmp_chain_a_greater:
    pop     %rax                       # op_id
    cmp     $1, %al                    # GT (1)?
    je      .cmp_true
    cmp     $3, %al                    # GE (3)?
    je      .cmp_true
    jmp     .cmp_false                 # LT (0) or LE (2) → false

.cmp_true:
    # Rust VM returns non-empty chain (1.0) for true
    mov     $0x3FF0000000000000, %rax   # 1.0 in f64
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.cmp_false:
    # Return f64 0.0 for false (NOT empty chain — breaks &&/|| logic)
    xor     %eax, %eax
    mov     %rax, (%r14)               # f64 bits = 0 (0.0)
    movq    $F64_MARKER, 8(%r14)       # mark as f64
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

.equ VAR_TABLE_MAX, 1024

# Initialize variable table — called once from _start
# Uses r15 (heap bump), stores table base in var_table_base
var_table_init:
    mov     %r15, %rax
    lea     var_table_base(%rip), %rcx
    mov     %rax, (%rcx)               # save table base address
    movq    $0, (%r15)                  # count = 0
    # Reserve 4096 entries × 24 bytes + 8 header = 98312 bytes
    add     $98312, %r15
    # Initialize var shadow stack (for closure scoping)
    mov     %r15, %rax
    lea     var_shadow_base(%rip), %rcx
    mov     %rax, (%rcx)
    lea     var_shadow_top(%rip), %rcx
    mov     %rax, (%rcx)
    lea     closure_depth(%rip), %rcx
    movq    $0, (%rcx)
    # Reserve 4MB for shadow stack (deep recursion snapshots)
    add     $0x400000, %r15
    ret

# ── Hash-based variable table ──
# Table layout: [count:8][entries...]
# Entry: [name_hash:8][val_ptr:8][val_len:8] = 24 bytes
# Simple, no string comparison needed.

# hash_name: rdi=name ptr, edx=name len → rax=hash
hash_name:
    # FNV-1a
    movabs  $0xcbf29ce484222325, %rax
    movabs  $0x100000001b3, %rcx
    add     %rdi, %rdx                  # end ptr
.hn_loop:
    cmp     %rdx, %rdi
    jae     .hn_done
    movzbl  (%rdi), %esi
    xor     %rsi, %rax
    imul    %rcx, %rax
    inc     %rdi
    jmp     .hn_loop
.hn_done:
    ret

# var_store_hash: rax=hash, r8=val_ptr, r9=val_len
# FRAME-AWARE: only search within current frame (depth_stack[depth-1]..count)
# This ensures inner function "n" SHADOWS outer "n" instead of overwriting
var_store_hash:
    push    %rbx
    lea     var_table_base(%rip), %rbx
    mov     (%rbx), %rbx               # table base
    mov     (%rbx), %rcx               # total count

    # Get frame_start: if depth > 0, start from depth_stack[depth-1]
    push    %rax                        # save hash
    lea     closure_depth(%rip), %rdi
    mov     (%rdi), %rdi               # depth
    xor     %r10d, %r10d               # frame_start = 0 (default: search all)
    test    %rdi, %rdi
    jz      .vsh_have_start
    dec     %rdi
    lea     scope_frame_ptrs(%rip), %r10
    mov     (%r10, %rdi, 8), %r10d    # frame_start = depth_stack[depth-1]
.vsh_have_start:
    pop     %rax                        # restore hash

    # entries_in_frame = count - frame_start
    mov     %rcx, %rdi                 # total count
    sub     %r10, %rdi                  # entries to search
    test    %rdi, %rdi
    jle     .vsh_new                    # no entries in frame → append

    # Reverse search from LAST entry to frame_start
    lea     -1(%rcx), %rdx
    imul    $24, %rdx
    lea     8(%rbx, %rdx), %rdi        # last entry ptr
    mov     %rcx, %rdx
    sub     %r10, %rdx                  # loop count = entries in frame

.vsh_search:
    cmp     (%rdi), %rax
    jne     .vsh_next
    # Found in current frame — update
    mov     %r8, 8(%rdi)
    mov     %r9, 16(%rdi)
    pop     %rbx
    ret
.vsh_next:
    sub     $24, %rdi
    dec     %rdx
    jnz     .vsh_search
    # Not found in frame → append as shadow

.vsh_new:
    # Append new entry at end
    lea     var_table_base(%rip), %rbx
    mov     (%rbx), %rbx
    mov     (%rbx), %rcx
    imul    $24, %rcx, %rdx
    lea     8(%rbx, %rdx), %rdi
    mov     %rax, (%rdi)
    mov     %r8, 8(%rdi)
    mov     %r9, 16(%rdi)
    incq    (%rbx)                     # count++
    pop     %rbx
    ret

# var_load_hash: rax=hash → rax=val_ptr, rdx=val_len
# REVERSE search: start from LAST entry (locals found first, O(1) for recent vars)
var_load_hash:
    push    %rbx
    lea     var_table_base(%rip), %rbx
    mov     (%rbx), %rbx
    mov     (%rbx), %rcx
    test    %rcx, %rcx
    jz      .vlh_not_found

    # Start from last entry
    lea     -1(%rcx), %rdx
    imul    $24, %rdx
    lea     8(%rbx, %rdx), %rdi

.vlh_search:
    cmp     (%rdi), %rax
    jne     .vlh_next
    mov     8(%rdi), %rax
    mov     16(%rdi), %rdx
    pop     %rbx
    ret
.vlh_next:
    sub     $24, %rdi
    dec     %rcx
    jnz     .vlh_search

.vlh_not_found:
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
    # al still has the opcode byte
    push    %rax
    movzbl  %al, %eax
    # Convert to 2-char hex string on stack
    mov     %eax, %ecx
    shr     $4, %ecx
    and     $0xF, %ecx
    lea     hex_chars(%rip), %rdx
    movzbl  (%rdx, %rcx), %ecx
    and     $0xF, %eax
    movzbl  (%rdx, %rax), %eax
    # Write "0xNN\n"
    sub     $8, %rsp
    movb    $0x30, (%rsp)           # '0'
    movb    $0x78, 1(%rsp)          # 'x'
    movb    %cl, 2(%rsp)            # high nibble
    movb    %al, 3(%rsp)            # low nibble
    movb    $0x0A, 4(%rsp)          # newline
    mov     %rsp, %rsi
    mov     $5, %edx
    mov     $2, %edi                # stderr
    mov     $1, %eax                # SYS_WRITE
    syscall
    add     $8, %rsp
    pop     %rax

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
    # Check if this is a nested halt (from __eval_bytecode or REPL trampoline)
    # Sentinels: 0xDEAD_BEEF_CAFE_0001 = eval_bytecode, 0002 = REPL trampoline
    .equ NESTED_SENTINEL, 0xDEADBEEFCAFE0001
    .equ REPL_SENTINEL,   0xDEADBEEFCAFE0002
    mov     (%rsp), %rax               # PEEK (not pop)

    movabs  $NESTED_SENTINEL, %rdx
    cmp     %rdx, %rax
    je      .halt_nested                # eval_bytecode → restore + vm_loop

    movabs  $REPL_SENTINEL, %rdx
    cmp     %rdx, %rax
    je      .halt_repl                  # REPL trampoline → restore + repl_loop

    jmp     .halt_boot                  # not nested → boot completion

.halt_nested:
    # __eval_bytecode: pop sentinel + saved context, continue caller bytecode
    add     $8, %rsp                    # skip sentinel
    jmp     .halt_nested_restore

.halt_repl:
    # REPL trampoline: pop sentinel + saved context, return to REPL loop
    add     $8, %rsp                    # skip sentinel
    pop     %rbx                        # restore bytecode size
    pop     %r13                        # restore PC
    pop     %r12                        # restore bytecode base
    jmp     .repl_loop                  # back to REPL prompt

.halt_boot:
    # Not nested — this is the boot completion.

    # Check eval_mode: if set, process stdin and exit (no REPL)
    cmpb    $0, eval_mode(%rip)
    jne     .eval_stdin

    # Print greeting
    lea     msg_greeting(%rip), %rsi
    movzbl  msg_greeting_len(%rip), %edx
    mov     $1, %edi                    # stdout
    mov     $SYS_WRITE, %eax
    syscall

    # ── REPL loop: read line → compile → execute ─────────────────
.repl_loop:
    # Print prompt: "⦿ "
    lea     msg_prompt(%rip), %rsi
    mov     $4, %edx                    # "⦿ " (⦿ is 3 bytes UTF-8 + space)
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall

    # Read line from stdin (byte-by-byte until newline, for pipe compatibility)
    mov     %r15, %rsi                  # buf = heap ptr
    xor     %ecx, %ecx                  # bytes read = 0
.repl_read_byte:
    lea     (%rsi, %rcx), %rdx         # dest = buf + offset
    push    %rcx
    push    %rsi
    mov     %rdx, %rsi                  # buf for read
    mov     $1, %edx                    # read 1 byte
    xor     %edi, %edi                  # fd = stdin
    mov     $SYS_READ, %eax
    syscall
    pop     %rsi
    pop     %rcx
    test    %rax, %rax
    jle     .repl_read_done             # EOF or error
    cmpb    $0x0A, (%rsi, %rcx)        # newline?
    je      .repl_read_newline
    inc     %rcx
    cmp     $4095, %rcx                 # max length
    jl      .repl_read_byte
.repl_read_newline:
    inc     %rcx                        # include newline in count
.repl_read_done:
    mov     %rcx, %rax                  # rax = bytes read (compatible with old code)

    # Check for EOF or error
    test    %rax, %rax
    jle     .repl_exit                  # EOF → exit

    # Save input length and buffer pointer
    mov     %rax, %rcx                  # input length
    mov     %r15, %rdi                  # input buffer

    # Check for "exit\n" or "quit\n"
    cmp     $5, %rcx
    jne     .repl_check_help
    cmpl    $0x74697865, (%rdi)         # "exit" little-endian
    jne     .repl_check_quit
    cmpb    $0x0A, 4(%rdi)
    je      .repl_exit
.repl_check_quit:
    cmpl    $0x74697571, (%rdi)         # "quit" little-endian
    jne     .repl_check_help
    cmpb    $0x0A, 4(%rdi)
    je      .repl_exit

.repl_check_help:
    # Check for "help\n" (5 bytes)
    cmp     $5, %rcx
    jne     .repl_try_eval
    cmpl    $0x706C6568, (%rdi)         # "help" little-endian
    jne     .repl_try_eval
    cmpb    $0x0A, 4(%rdi)
    jne     .repl_try_eval
    # Print help message
    lea     msg_help(%rip), %rsi
    mov     $msg_help_len, %edx
    mov     $1, %edi                    # stdout
    mov     $SYS_WRITE, %eax
    syscall
    jmp     .repl_loop

.repl_try_eval:
    # Construct trampoline bytecode on heap that:
    #   1. Pushes input string
    #   2. Calls repl_eval(input)
    #   3. Emits result
    #   4. Halts (nested)
    #
    # Format (codegen bytecode):
    #   [0x01][len:2 LE][input bytes]   — Push string
    #   [0x24][0x09]["repl_eval"][0x01] — CallClosure "repl_eval" arity=1
    #   [0x06]                          — Emit
    #   [0x0F]                          — Halt

    # Bump heap past input data (so trampoline doesn't overwrite input)
    add     %rcx, %r15                  # r15 += input_len
    mov     %r15, %r8                   # trampoline base (after input)
    # Check repl_eval exists first
    push    %rcx                        # save input len
    push    %rdi                        # save input ptr
    lea     bname_repl_eval(%rip), %rdi
    mov     $9, %edx
    call    hash_name
    call    var_load_hash
    mov     %rdx, %rsi
    and     $0xFFFF, %esi
    pop     %rdi                        # restore input ptr
    pop     %rcx                        # restore input len
    cmp     $CLOSURE_MARKER, %si
    jne     .repl_fallback_echo

    # Build trampoline bytecode
    # Push opcode with binary molecule encoding:
    # [0x01][mol_count:u16][str_byte_mol(b0):u16][str_byte_mol(b1):u16]...
    # str_byte_mol(b) = 0x2100 | b  (shape=2, rel=1, lower 8 bits = byte value)
    movb    $0x01, (%r15)               # Push opcode
    movw    %cx, 1(%r15)                # mol_count = string length (1 mol per byte)
    add     $3, %r15

    # Encode each input byte as string molecule (u16 LE)
    push    %rcx
    push    %rdi
.repl_encode_loop:
    test    %ecx, %ecx
    jz      .repl_encode_done
    movzbl  (%rdi), %eax                # read input byte
    or      $0x2100, %eax               # str_byte_mol: shape=2, rel=1
    movw    %ax, (%r15)                 # write u16 LE molecule
    add     $2, %r15                    # 2 bytes per molecule
    inc     %rdi
    dec     %ecx
    jmp     .repl_encode_loop
.repl_encode_done:
    pop     %rdi
    pop     %rcx

    # CallClosure "repl_eval" arity=1
    movb    $0x24, (%r15)               # CallClosure opcode
    movb    $9, 1(%r15)                 # name len
    # Copy "repl_eval"
    push    %rcx
    lea     bname_repl_eval(%rip), %rsi
    lea     2(%r15), %rdi
    mov     $9, %ecx
    rep     movsb
    pop     %rcx
    movb    $1, 11(%r15)                # arity = 1
    add     $12, %r15

    # Emit
    movb    $0x06, (%r15)
    inc     %r15

    # Halt (nested)
    movb    $0x0F, (%r15)
    inc     %r15

    # Calculate trampoline length
    mov     %r15, %rcx
    sub     %r8, %rcx                   # trampoline bytecode length

    # Execute as nested bytecode: save context + run
    push    %r12                        # save boot bytecode base
    push    %r13                        # save boot PC
    push    %rbx                        # save boot bytecode size
    movabs  $0xDEADBEEFCAFE0002, %rax
    push    %rax                        # REPL sentinel (0002 = return to repl_loop)

    # Use boot bytecode base as r12 (not trampoline)
    # Trampoline is at %r8 — save it, set r12 = boot base
    mov     %r8, %r12                   # bytecode base = trampoline (for Push/CallClosure)
    xor     %r13d, %r13d               # PC = 0
    # Set rbx to large value so op_ret accepts boot bytecode PCs
    # (trampoline CallClosure jumps into boot bytecode bodies where
    # Ret pops saved PCs that are >> trampoline size)
    mov     $0x100000, %rbx             # 1MB — larger than any bytecode
    jmp     vm_loop                     # execute trampoline!
    # Flow: Push input → CallClosure repl_eval → Halt(REPL sentinel)
    #       → restore → jmp .repl_loop

.repl_fallback_echo:
    # repl_eval not found — echo input back
    mov     %rdi, %rsi                  # input ptr
    mov     %rcx, %rdx                  # input len
    mov     $1, %edi                    # stdout
    mov     $SYS_WRITE, %eax
    syscall
    jmp     .repl_loop

.halt_nested_restore:
    # Returning from nested __eval_bytecode execution
    # CPU stack now has: [saved_rbx][saved_r13][saved_r12]
    pop     %rbx                        # restore bytecode size
    pop     %r13                        # restore PC
    pop     %r12                        # restore bytecode base
    # Push empty return value (eval_bytecode consumed its arg but needs to
    # return a value for the caller's stack balance)
    sub     $16, %r14
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
    jmp     vm_loop                     # continue outer execution

# ── --eval mode: read ALL stdin → process each line → exit ─────────
.eval_stdin:
    # Pop the value that op_halt pushed back (prevents stack leak)
    pop     %rax

    # Check if this is a re-entry (buffer already read)
    cmpq    $0, eval_buf(%rip)
    jne     .eval_next_line             # re-entry → continue from saved offset

    # First entry: allocate 64KB buffer and read all stdin
    mov     %r15, %rax
    mov     %rax, eval_buf(%rip)        # save buffer base in BSS
    add     $0x10000, %r15              # reserve 64KB

    # Read all stdin into buffer
    mov     %rax, %rsi                  # buf
    mov     $0x10000, %edx             # max 64KB
    xor     %edi, %edi                  # fd = stdin (0)
    mov     $SYS_READ, %eax
    syscall
    test    %rax, %rax
    jle     .eval_exit                  # EOF or error → exit

    mov     %rax, eval_total(%rip)      # save total length
    movq    $0, eval_offset(%rip)       # start at offset 0

.eval_next_line:
    mov     eval_offset(%rip), %rcx     # current offset
    mov     eval_total(%rip), %r9       # total length
    mov     eval_buf(%rip), %r8         # buffer base
    cmp     %r9, %rcx
    jge     .eval_exit                  # processed all input

    # Find end of current line (scan for \n or end of input)
    mov     %rcx, %rdi                  # save line start offset
.eval_scan:
    cmp     %r9, %rcx
    jge     .eval_got_line              # end of input = end of line
    cmpb    $0x0A, (%r8,%rcx)          # newline?
    je      .eval_got_line
    inc     %rcx
    jmp     .eval_scan

.eval_got_line:
    # rdi = line start offset, rcx = line end offset (at \n or end)
    # Save next offset (past \n) for after execution
    mov     %rcx, %rax
    cmp     %r9, %rcx
    jge     .eval_no_skip_nl
    inc     %rax                        # skip past \n
.eval_no_skip_nl:
    mov     %rax, eval_offset(%rip)     # save offset for next iteration

    mov     %rcx, %rax
    sub     %rdi, %rax                  # rax = line length
    test    %rax, %rax
    jz      .eval_next_line             # skip empty lines

    # Set up line pointer and length
    lea     (%r8,%rdi), %rdi            # rdi = line start ptr
    mov     %rax, %rcx                  # rcx = line length

    # Strip trailing \r if present
    cmpb    $0x0D, -1(%rdi,%rcx)
    jne     .eval_no_cr
    dec     %rcx
    test    %rcx, %rcx
    jz      .eval_next_line             # empty after CR strip
.eval_no_cr:

    # Check if repl_eval exists
    push    %rcx                        # save line len
    push    %rdi                        # save line ptr
    lea     bname_repl_eval(%rip), %rdi
    mov     $9, %edx
    call    hash_name
    call    var_load_hash
    mov     %rdx, %rsi
    and     $0xFFFF, %esi
    pop     %rdi                        # restore line ptr
    pop     %rcx                        # restore line len
    cmp     $CLOSURE_MARKER, %si
    jne     .eval_fallback_echo

    # Build trampoline: Push string → CallClosure repl_eval → Emit → Halt
    mov     %r15, %r8                   # trampoline base
    # Push with binary molecule encoding (same as REPL trampoline)
    movb    $0x01, (%r15)               # Push opcode
    movw    %cx, 1(%r15)                # mol_count = string length
    add     $3, %r15

    # Encode each byte as string molecule (0x2100 | byte)
    push    %rcx
    push    %rdi
.eval_encode_loop:
    test    %ecx, %ecx
    jz      .eval_encode_done
    movzbl  (%rdi), %eax
    or      $0x2100, %eax
    movw    %ax, (%r15)
    add     $2, %r15
    inc     %rdi
    dec     %ecx
    jmp     .eval_encode_loop
.eval_encode_done:
    pop     %rdi
    pop     %rcx

    # CallClosure "repl_eval" arity=1
    movb    $0x24, (%r15)
    movb    $9, 1(%r15)
    push    %rcx
    lea     bname_repl_eval(%rip), %rsi
    lea     2(%r15), %rdi
    mov     $9, %ecx
    rep     movsb
    pop     %rcx
    movb    $1, 11(%r15)                # arity = 1
    add     $12, %r15

    # Emit + Halt
    movb    $0x06, (%r15)
    inc     %r15
    movb    $0x0F, (%r15)
    inc     %r15

    # Calculate trampoline size
    mov     %r15, %rcx
    sub     %r8, %rcx

    # Execute as nested bytecode
    push    %r12
    push    %r13
    push    %rbx
    movabs  $0xDEADBEEFCAFE0001, %rax
    push    %rax                        # nested flag sentinel

    mov     %r8, %r12                   # bytecode base = trampoline
    xor     %r13d, %r13d               # PC = 0
    mov     $0x100000, %rbx             # 1MB bounds — op_ret accepts boot PCs
    jmp     vm_loop
    # Flow: trampoline → Halt(nested) → .halt_nested → restore r12/r13/rbx
    #       → vm_loop → boot bytecode Halt → op_halt → not nested
    #       → eval_mode check → .eval_stdin → .eval_next_line (reads from BSS)

.eval_fallback_echo:
    # repl_eval not found — echo input
    mov     %rdi, %rsi                  # line ptr
    mov     %rcx, %rdx                  # line len
    mov     $1, %edi                    # stdout
    mov     $SYS_WRITE, %eax
    syscall
    # Write newline
    lea     str_newline(%rip), %rsi
    mov     $1, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     .eval_next_line

.eval_exit:
    # Exit silently (no greeting, no goodbye)
    mov     $SYS_EXIT, %eax
    xor     %edi, %edi                  # exit code 0
    syscall

.repl_exit:
    # Print goodbye
    lea     msg_bye(%rip), %rsi
    mov     $4, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
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

hex_chars:
    .ascii "0123456789ABCDEF"
msg_pc:
    .ascii "PC= "
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

msg_greeting:
    .ascii "\xe2\xa6\xbf HomeOS v0.05\n"
    .ascii "\xe2\x97\x8b Type code or text \xc2\xb7 exit to quit\n"
msg_greeting_len:
    .byte 55                            # exact byte count of greeting

msg_prompt:
    .ascii "\xe2\xa6\xbf "               # "⦿ " (⦿ U+29BF = 3 bytes UTF-8 + space)

msg_help:
    .ascii "Code: let fn emit if while for match lambda\n"
    .ascii "HOF:  map filter reduce pipe any all\n"
    .ascii "AI:   learn <fact>  respond <text>  memory  fns\n"
    .ascii "Sys:  test  build  save  load  compile <path>  exit\n"
.equ msg_help_len, . - msg_help

# ── Keyword hash table for __str_is_keyword ────────────────────────────
.align 4
kw_count: .long 28
.align 8
kw_hashes:
    .quad 0x127284191DCC577A  # let
    .quad 0x08985A07B541D4F5  # fn
    .quad 0x08B73007B55C3E26  # if
    .quad 0x7F2B6C605332DD30  # else
    .quad 0xCDECE4AD70D57DEB  # loop
    .quad 0xCE87A3885811296E  # while
    .quad 0xDCB27818FED9DA90  # for
    .quad 0x08B73807B55C4BBE  # in
    .quad 0xC5C7B983377CAD5F  # return
    .quad 0x93B7591DEBC7CE38  # break
    .quad 0xD2BFD5ACCD1966A4  # continue
    .quad 0x86D29860573F5FCA  # emit
    .quad 0xA79439EF7BFA9C2D  # type
    .quad 0x4B2A584E9A909034  # union
    .quad 0xEE517EC576C9B637  # impl
    .quad 0xDEE78BA6C508610B  # trait
    .quad 0xC3BFE3A4FE4C13F6  # match
    .quad 0x570AC119447423EE  # try
    .quad 0xC1B2E33B13EC076C  # catch
    .quad 0x4328F78AB20E1F98  # spawn
    .quad 0xBD76100BD58C9C0D  # select
    .quad 0x60827E549DE65488  # timeout
    .quad 0x7F845078D7A5C0B5  # from
    .quad 0x4C52B0193DCCE634  # use
    .quad 0x0808591917670EE3  # mod
    .quad 0x77F362195699B624  # pub
    .quad 0x5B5C98EF514DBFA5  # true
    .quad 0xB5FAE2C14238B978  # false

    .ascii "!"

msg_bye:
    .ascii "bye\n"

str_newline:
    .ascii "\n"

# ── Type name strings ───────────────────────────────────────────────────
str_number:     .ascii "number"
str_string:     .ascii "string"
str_array:      .ascii "array"
str_dict:       .ascii "dict"
str_fn:         .ascii "function"
str_nil:        .ascii "nil"

# ── Float constants ─────────────────────────────────────────────────────
.align 8
const_ten:      .double 10.0
const_one:      .double 1.0
const_neg_one:  .double -1.0

# ── Emit format strings ─────────────────────────────────────────────────
str_array_open:  .ascii "[array "
str_bracket_nl:  .ascii "]\n"
str_empty_array: .ascii "[]\n\0"
str_lbracket:    .ascii "["
str_rbracket_nl: .ascii "]\n"
str_comma_space: .ascii ", "
str_dict_open:   .ascii "{dict "
str_brace_nl:    .ascii "}\n"
str_empty_dict:  .ascii "{}\n\0"
str_lbrace:      .ascii "{"
str_colon_sp:    .ascii ": "
str_comma_sp:    .ascii ", "

# ── REPL function name ──────────────────────────────────────────────────
bname_repl_eval: .ascii "repl_eval"

# ── BSS (writable, zero-initialized) ────────────────────────────────────
.section .bss
var_table_base:
    .space 8                            # pointer to variable table on heap
bc_format:
    .space 1                            # 0 = ir.rs format, 1 = codegen.ol format
    .align 8
boot_bc_base:
    .space 8                            # boot bytecode base address (for nested calls)
boot_bc_size:
    .space 8                            # boot bytecode size
eval_bc_base:
    .space 8                            # eval bytecode base (for cross-boundary closure calls)
eval_mode:
    .space 1                            # 0 = REPL mode, 1 = --eval mode (stdin)
    .align 8
    .space 8                            # saved initial r14 for debug
jz_did_pop:
    .space 1                            # 1 = Jz already popped, next Pop is no-op
    .align 8
# Var shadow stack: saves [hash, old_ptr, old_len] per modified var in closure.
# Marker entry: hash=0 separates call frames.
var_shadow_base:
    .space 8                            # pointer to shadow stack on heap
var_shadow_top:
    .space 8                            # current top of shadow stack
closure_depth:
    .space 8                            # nesting depth (0 = not in closure)
_save_rbx:
    .space 8                            # temp save for rbx
scope_frame_ptrs:
    .space 2048                         # 256 frame pointers × 8 bytes (max recursion depth)
    .align 8
try_stack:
    .space 256                          # 16 try frames × 16 bytes (catch_PC:8 + saved_r14:8)
try_depth:
    .space 8                            # current try nesting depth (0 = no try)
throw_buf:
    .space 256                          # temp buffer for error message decoding
heap_checkpoints:
    .space 128                          # 16 checkpoint slots × 8 bytes
heap_cp_count:
    .space 8
sha_input:
    .space 1024                         # SHA-256 input buffer (max ~900 bytes after padding)
sha_state:
    .space 32                           # H[0..7] (8 × 4 bytes)
sha_w:
    .space 256                          # W[0..63] message schedule (64 × 4 bytes)
sha_abcdefgh:
    .space 32                           # working variables a..h (8 × 4 bytes)
sha_save_regs:
    .space 64                           # saved VM registers during SHA-256 (8 × 8 bytes)
sha_round_idx:
    .space 8                            # round counter (avoid clobbering r12)
eval_buf:
    .space 8                            # pointer to stdin buffer on heap
eval_total:
    .space 8                            # total bytes read from stdin
eval_offset:
    .space 8                            # current processing offset in buffer
