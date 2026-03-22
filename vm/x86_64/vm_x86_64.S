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
.equ ELF_HEADER_SIZE, 120           # ELF64 header + 1 program header
.equ F64_MARKER,    -1              # len = 0xFFFFFFFFFFFFFFFF means f64
.equ ARRAY_MARKER,  -3              # len = 0xFFFFFFFFFFFFFFFD means array
.equ DICT_MARKER,   -4              # len = 0xFFFFFFFFFFFFFFFC means dict
# Note: CLOSURE_MARKER = -2 is defined near cg_closure

# Array heap layout: [count:8][elem_0_ptr:8][elem_0_len:8]...
# Dict heap layout:  [count:8][entry: key_ptr:8 key_len:8 val_ptr:8 val_len:8]...
# Max initial capacity: 16 elements (grows on demand)
.equ ARRAY_INIT_CAP, 16

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
    # 0x3A = CallBuiltin: skip builtin_id byte (1 byte)
    cmp     $0x3A, %al
    jne     .cg_not_builtin
    inc     %r13                        # skip builtin_id
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
    # Store body_pc as raw u64 (NOT f64, just integer PC offset)
    mov     %r13, (%r14)                # body_pc = current PC (start of body)
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

    # Save return address on CPU stack
    push    %r13                        # save current PC (return point)

    # Jump to body
    mov     %edi, %r13d                 # PC = body_pc
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
    # Store by hash
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
    movabs  $0xE386A6329E2424BE, %rdx   # __substr
    cmp     %rdx, %rax
    je      .call_substr
    movabs  $0xFBE159BD1C6EBFA7, %rdx   # __str_trim
    cmp     %rdx, %rax
    je      .call_str_trim
    movabs  $0x74AEDE408DC6787F, %rdx   # __str_slice
    cmp     %rdx, %rax
    je      .call_substr                 # str_slice = substr alias
    movabs  $0x53416FC92E4E354A, %rdx   # __str_bytes
    cmp     %rdx, %rax
    je      .call_str_bytes
    movabs  $0x3EA9B2DCA6EB46A2, %rdx   # __to_number
    cmp     %rdx, %rax
    je      .call_to_number
    movabs  $0xE811320261AE416D, %rdx   # __type_of
    cmp     %rdx, %rax
    je      .call_type_of

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
    push    %r13                        # save return address
    mov     %eax, %r13d                 # PC = body_pc
    jmp     vm_loop

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
    # Compare chains byte-by-byte
    mov     8(%r14), %rax              # b len
    mov     24(%r14), %rcx             # a len
    cmp     %rax, %rcx
    jne     .eq_false                   # different lengths → not equal
    # Same length — compare bytes
    mov     (%r14), %rsi               # b ptr
    mov     16(%r14), %rdi             # a ptr
    add     $16, %r14                  # pop 1
    mov     %rax, %rcx                  # count
    test    %rcx, %rcx
    jz      .eq_true_chain              # both empty → equal
    repe    cmpsb
    je      .eq_true_chain
.eq_false_chain:
    # Push empty (false)
    xor     %eax, %eax
    mov     %rax, (%r14)
    mov     %rax, 8(%r14)
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
    # Array: read count from heap
    mov     (%r14), %rdi               # array base ptr
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
    mov     (%r14), %rsi               # string ptr
    mov     8(%r14), %rcx              # string len
    add     $16, %r14                  # pop

    # Simple decimal parser: sign + digits + optional '.' + digits
    xor     %eax, %eax                  # negative flag
    xorpd   %xmm0, %xmm0              # result = 0.0
    test    %rcx, %rcx
    jz      .to_num_done
    # Check minus sign
    cmpb    $'-', (%rsi)
    jne     .to_num_loop
    mov     $1, %eax                    # negative
    inc     %rsi
    dec     %rcx
.to_num_loop:
    test    %rcx, %rcx
    jz      .to_num_sign
    movzbl  (%rsi), %edx
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
    inc     %rsi
    dec     %rcx
    jmp     .to_num_loop
.to_num_frac:
    inc     %rsi
    dec     %rcx
    movsd   const_one(%rip), %xmm2      # divisor = 1.0
.to_num_frac_loop:
    test    %rcx, %rcx
    jz      .to_num_sign
    movzbl  (%rsi), %edx
    sub     $'0', %edx
    cmp     $10, %edx
    jae     .to_num_sign
    movsd   const_ten(%rip), %xmm1
    mulsd   %xmm1, %xmm2               # divisor *= 10
    cvtsi2sd %edx, %xmm1               # digit
    divsd   %xmm2, %xmm1               # digit / divisor
    addsd   %xmm1, %xmm0               # result += digit/divisor
    inc     %rsi
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

# ── String-trim builtin ─────────────────────────────────────────────────

.call_str_trim:
    add     %rcx, %r13
    mov     (%r14), %rsi               # string ptr
    mov     8(%r14), %rcx              # string len
    add     $16, %r14                  # pop
    # Trim leading spaces
    mov     %rsi, %rdi                  # start
    add     %rcx, %rsi                  # end
.trim_lead:
    cmp     %rsi, %rdi
    jae     .trim_empty
    movzbl  (%rdi), %eax
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
    inc     %rdi
    jmp     .trim_lead
.trim_trail:
    # rdi = trimmed start, rsi = past end
.trim_trail_loop:
    cmp     %rdi, %rsi
    jbe     .trim_done
    movzbl  -1(%rsi), %eax
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
    dec     %rsi
    jmp     .trim_trail_loop
.trim_done:
    mov     %rsi, %rcx
    sub     %rdi, %rcx                  # new length
    # Copy to heap
    mov     %r15, %rax
    push    %rcx
    mov     %r15, %rsi
    xchg    %rsi, %rdi                  # rdi=dest=heap, rsi=src=trimmed
    mov     %ecx, %ecx
    rep     movsb
    mov     %rdi, %r15                  # bump
    pop     %rcx
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

    # Each byte → f64 on heap
    xor     %edx, %edx
.str_bytes_loop:
    cmp     %rcx, %rdx
    jae     .str_bytes_done
    movzbl  (%rsi, %rdx), %eax
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
    # Stack: [array, value] → append value to array, push array
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

    # Read count, write element at end
    mov     (%rdi), %rcx               # count
    # Element position: 8 + count*16
    mov     %rcx, %rax
    shl     $4, %rax
    add     $8, %rax
    mov     %r8, (%rdi, %rax)          # write val ptr
    mov     %r9, 8(%rdi, %rax)         # write val len
    incq    (%rdi)                      # count++

    # Push array back
    sub     $16, %r14
    mov     %rdi, (%r14)
    movq    $ARRAY_MARKER, 8(%r14)
    jmp     vm_loop

.array_push_create_new:
    # Create new array with 1 element
    mov     %r15, %rdi                  # array base
    movq    $1, (%r15)                  # count = 1
    mov     %r8, 8(%r15)               # elem ptr
    mov     %r9, 16(%r15)              # elem len
    add     $24, %r15                   # bump heap
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
    # key at (%rsi), val at 16(%rsi)
    mov     (%rsi), %rax               # key ptr
    mov     8(%rsi), %r8               # key len
    mov     16(%rsi), %r9              # val ptr
    mov     24(%rsi), %r10             # val len
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
    # Pop key
    mov     (%r14), %rsi               # key ptr
    mov     8(%r14), %rdx              # key len
    # Pop dict
    mov     16(%r14), %rdi              # dict base
    add     $32, %r14                   # pop 2

    test    %rdi, %rdi
    jz      .dict_get_empty

    # Search entries
    mov     (%rdi), %rcx               # entry count
    lea     8(%rdi), %r8               # first entry
.dict_get_search:
    test    %rcx, %rcx
    jz      .dict_get_empty

    # Compare key: entry key at (%r8), len at 8(%r8)
    mov     8(%r8), %rax               # entry key len
    cmp     %rax, %rdx                  # same length?
    jne     .dict_get_next

    # Same length — compare bytes
    push    %rcx
    push    %rsi
    push    %rdx
    push    %r8
    mov     (%r8), %rdi                # entry key ptr
    mov     %rdx, %rcx                 # byte count
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
    # Pop dict
    mov     32(%r14), %rdi             # dict base
    add     $48, %r14                  # pop 3

    test    %rdi, %rdi
    jz      .dict_set_push

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
    # Found — update value
    mov     %r9, 16(%r8)
    mov     %r10, 24(%r8)
    # Reload dict base (rdi may have been clobbered by cmpsb)
    mov     32(%r14), %rdi             # Actually we already popped...
    # rdi was saved before loop, let's restore from dict base
    jmp     .dict_set_push

.dict_set_append:
    # Append new entry at end of dict
    mov     (%rdi), %rcx               # current count
    # New entry position: 8 + count*32
    mov     %rcx, %rax
    shl     $5, %rax
    add     $8, %rax
    mov     %rsi, (%rdi, %rax)         # key ptr
    mov     %rdx, 8(%rdi, %rax)        # key len
    mov     %r9, 16(%rdi, %rax)        # val ptr
    mov     %r10, 24(%rdi, %rax)       # val len
    incq    (%rdi)                      # count++

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
    mov     %rcx, %rbx                  # bytecode size
    add     %rcx, %r15                  # bump heap past bytecode buffer

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
    # CPU stack top has the saved PC from cg_call_closure.
    # Check if we have a return address (rsp points to saved PC).
    # If the top of CPU stack looks like a valid PC (< bytecode size),
    # restore it. Otherwise treat as no-op.
    pop     %rax                        # potential saved PC
    cmp     %rbx, %rax                  # compare with bc_size
    jae     .ret_noop                   # not a valid PC → push back and continue
    mov     %eax, %r13d                 # PC = saved return address
    jmp     vm_loop
.ret_noop:
    push    %rax                        # restore stack (wasn't a return addr)
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
    # Print "[array N]" with element count
    # rsi = array base ptr
    test    %rsi, %rsi
    jz      .emit_array_empty
    mov     (%rsi), %rax               # count
    lea     str_array_open(%rip), %rsi
    mov     $7, %edx                    # "[array "
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    # Print count as number
    cvtsi2sd %rax, %xmm0
    call    f64_to_ascii
    mov     %r15, %rsi
    mov     %eax, %edx
    mov     $1, %edi
    push    %rax
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rax
    add     %rax, %r15
    lea     str_bracket_nl(%rip), %rsi
    mov     $2, %edx
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     vm_loop
.emit_array_empty:
    lea     str_empty_array(%rip), %rsi
    mov     $4, %edx                    # "[]\n"
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    jmp     vm_loop

.emit_dict:
    # Print "{dict N}"
    test    %rsi, %rsi
    jz      .emit_dict_empty
    mov     (%rsi), %rax
    lea     str_dict_open(%rip), %rsi
    mov     $6, %edx                    # "{dict "
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall
    cvtsi2sd %rax, %xmm0
    call    f64_to_ascii
    mov     %r15, %rsi
    mov     %eax, %edx
    mov     $1, %edi
    push    %rax
    mov     $SYS_WRITE, %eax
    syscall
    pop     %rax
    add     %rax, %r15
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
    movsd   16(%r14), %xmm0            # B (deeper)
    movsd   (%r14), %xmm1              # A (top)
    add     $16, %r14                   # pop 1, result overwrites remaining
    ucomisd %xmm1, %xmm0
    je      .cmp_true
    jmp     .cmp_false

op_cmp_lt:
    movsd   16(%r14), %xmm0
    movsd   (%r14), %xmm1
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
    # Rust VM returns non-empty chain (1.0) for true
    mov     $0x3FF0000000000000, %rax   # 1.0 in f64
    mov     %rax, (%r14)
    movq    $F64_MARKER, 8(%r14)
    jmp     vm_loop

.cmp_false:
    # Rust VM returns empty chain for false → Jz compatible
    xor     %eax, %eax
    mov     %rax, (%r14)               # ptr = 0
    mov     %rax, 8(%r14)              # len = 0 (empty = false)
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
    # Reserve 1024 entries × 24 bytes + 8 header = 24584 bytes
    add     $24584, %r15                # bump past reserved var table space
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
var_store_hash:
    push    %rbx
    lea     var_table_base(%rip), %rbx
    mov     (%rbx), %rbx               # table base
    mov     (%rbx), %rcx               # count
    test    %rcx, %rcx
    jz      .vsh_new

    # Search for existing hash
    lea     8(%rbx), %rdi              # first entry
.vsh_search:
    cmp     (%rdi), %rax               # compare hash
    jne     .vsh_next
    # Found — update
    mov     %r8, 8(%rdi)
    mov     %r9, 16(%rdi)
    pop     %rbx
    ret
.vsh_next:
    add     $24, %rdi
    dec     %rcx
    jnz     .vsh_search

.vsh_new:
    # New entry: append at table end
    lea     var_table_base(%rip), %rbx
    mov     (%rbx), %rbx
    mov     (%rbx), %rcx              # current count
    lea     8(%rbx), %rdi
    imul    $24, %rcx, %rcx
    add     %rcx, %rdi                 # ptr to new entry
    mov     %rax, (%rdi)               # hash
    mov     %r8, 8(%rdi)              # val_ptr
    mov     %r9, 16(%rdi)             # val_len
    lea     var_table_base(%rip), %rbx
    mov     (%rbx), %rbx
    incq    (%rbx)                     # count++
    pop     %rbx
    ret

# var_load_hash: rax=hash → rax=val_ptr, rdx=val_len
var_load_hash:
    push    %rbx
    lea     var_table_base(%rip), %rbx
    mov     (%rbx), %rbx
    mov     (%rbx), %rcx
    test    %rcx, %rcx
    jz      .vlh_not_found

    lea     8(%rbx), %rdi
.vlh_search:
    cmp     (%rdi), %rax
    jne     .vlh_next
    # Found
    mov     8(%rdi), %rax
    mov     16(%rdi), %rdx
    pop     %rbx
    ret
.vlh_next:
    add     $24, %rdi
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
    # Check if this is a nested halt (from __eval_bytecode)
    # Use unique sentinel 0xDEAD_BEEF_CAFE_0001 to avoid collision with
    # argc (1) or other stack values.
    .equ NESTED_SENTINEL, 0xDEADBEEFCAFE0001
    mov     (%rsp), %rax               # PEEK (not pop)
    movabs  $NESTED_SENTINEL, %rdx
    cmp     %rdx, %rax
    jne     .halt_boot                  # not nested → boot completion

    # Nested: pop the flag + saved context
    add     $8, %rsp                    # skip nested flag
    jmp     .halt_nested_restore

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
    # Print prompt: "○ > "
    lea     msg_prompt(%rip), %rsi
    mov     $6, %edx                    # "○ > " (○ is 3 bytes UTF-8 + space + > + space)
    mov     $1, %edi
    mov     $SYS_WRITE, %eax
    syscall

    # Read line from stdin into heap buffer
    mov     %r15, %rsi                  # buf = heap ptr
    mov     $4096, %edx                 # max 4KB input
    xor     %edi, %edi                  # fd = stdin
    mov     $SYS_READ, %eax
    syscall

    # Check for EOF or error
    test    %rax, %rax
    jle     .repl_exit                  # EOF → exit

    # Save input length and buffer pointer
    mov     %rax, %rcx                  # input length
    mov     %r15, %rdi                  # input buffer

    # Check for "exit\n" or "quit\n"
    cmp     $5, %rcx
    jne     .repl_try_eval
    cmpl    $0x74697865, (%rdi)         # "exit" little-endian
    jne     .repl_check_quit
    cmpb    $0x0A, 4(%rdi)
    je      .repl_exit
.repl_check_quit:
    cmpl    $0x74697571, (%rdi)         # "quit" little-endian
    jne     .repl_try_eval
    cmpb    $0x0A, 4(%rdi)
    je      .repl_exit

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

    mov     %r15, %r8                   # trampoline base
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
    movabs  $0xDEADBEEFCAFE0001, %rax
    push    %rax                        # nested flag sentinel

    mov     %r8, %r12                   # bytecode base = trampoline
    xor     %r13d, %r13d               # PC = 0
    mov     %rcx, %rbx                  # bytecode size
    jmp     vm_loop                     # execute trampoline!
    # Flow: Push input → CallClosure repl_eval → (run compiler) → Emit → Halt
    # Halt (nested) → .halt_nested → restore → vm_loop → op_halt → REPL loop

.repl_fallback_echo:
    # repl_eval not found — echo input back
    mov     %rdi, %rsi                  # input ptr
    mov     %rcx, %rdx                  # input len
    mov     $1, %edi                    # stdout
    mov     $SYS_WRITE, %eax
    syscall
    jmp     .repl_loop

.halt_nested:
.halt_nested_restore:
    # Returning from nested __eval_bytecode execution
    # CPU stack now has: [saved_rbx][saved_r13][saved_r12]
    pop     %rbx                        # restore bytecode size
    pop     %r13                        # restore PC
    pop     %r12                        # restore bytecode base
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
    mov     %rcx, %rbx                  # bytecode size
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
    .ascii "\xe2\x97\x8b HomeOS v0.05\n"
    .ascii "\xe2\x97\x8b Type code or text \xc2\xb7 exit to quit\n"
    .byte 0
msg_greeting_len:
    .byte 59                            # total length of greeting

msg_prompt:
    .ascii "\xe2\x97\x8b > "             # "○ > " (○ is 3 bytes UTF-8 + space + > + space)

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
str_dict_open:   .ascii "{dict "
str_brace_nl:    .ascii "}\n"
str_empty_dict:  .ascii "{}\n\0"

# ── REPL function name ──────────────────────────────────────────────────
bname_repl_eval: .ascii "repl_eval"

# ── BSS (writable, zero-initialized) ────────────────────────────────────
.section .bss
var_table_base:
    .space 8                            # pointer to variable table on heap
bc_format:
    .space 1                            # 0 = ir.rs format, 1 = codegen.ol format
eval_mode:
    .space 1                            # 0 = REPL mode, 1 = --eval mode (stdin)
    .align 8
eval_buf:
    .space 8                            # pointer to stdin buffer on heap
eval_total:
    .space 8                            # total bytes read from stdin
eval_offset:
    .space 8                            # current processing offset in buffer
