# PLAN 1.1 — vm_x86_64.S — Machine Code VM

**Phụ thuộc:** PLAN_0_5 (bytecode format phải cố định trước)
**Mục tiêu:** Viết VM bằng x86_64 assembly. `./origin.olang` chạy trên Linux không cần Rust.
**Yêu cầu:** Biết x86_64 assembly (AT&T hoặc Intel syntax). Biết Linux syscalls.

---

## Bối cảnh cho ASM developer

### origin.olang file layout (sau khi build)

```
offset 0:    HEADER (32 bytes)
             [magic: 4B "○LNG"]
             [version: 1B]
             [arch: 1B]          ← 0x01 = x86_64
             [vm_offset: 4B]     ← offset đến VM machine code
             [vm_size: 4B]
             [bc_offset: 4B]     ← offset đến bytecode section
             [bc_size: 4B]
             [kn_offset: 4B]     ← offset đến knowledge section
             [kn_size: 4B]
             [flags: 2B]

offset vm_offset:  VM MACHINE CODE (bạn viết phần này)
offset bc_offset:  BYTECODE (output từ codegen.ol)
offset kn_offset:  KNOWLEDGE (origin.olang records)
```

### VM đọc bytecode format

Bytecode format từ PLAN_0_5:
```
0x01 Push      [chain_len:2][chain:N]
0x02 Load      [name_len:1][name:N]
0x03 Lca       (none)
0x06 Emit      (none)
0x09 Jmp       [target:4]
0x0A Jz        [target:4]
0x0F Halt      (none)
0x15 PushNum   [f64:8]
...
```

---

## Cấu trúc assembly

### File: `vm/x86_64/vm_x86_64.S`

```asm
# ═══════════════════════════════════════════════════════════════
# origin.olang VM — x86_64 Linux (no libc)
# ═══════════════════════════════════════════════════════════════

.section .text
.globl _start

# ── Register conventions ──────────────────────────────────────
# r12 = bytecode base pointer (immutable during execution)
# r13 = program counter (offset into bytecode)
# r14 = stack pointer (VM stack, NOT CPU stack)
# r15 = heap pointer (bump allocator)
# rbp = preserved (call frames)
# rsp = CPU stack (for CALL/RET, syscalls)

# ── Constants ─────────────────────────────────────────────────
.equ STACK_SIZE,   0x100000    # 1 MB VM stack
.equ HEAP_SIZE,    0x1000000   # 16 MB heap arena
.equ HEADER_SIZE,  32
```

### Task 1: Entry point + memory setup (~100 LOC)

```asm
_start:
    # 1. Đọc file chính nó
    #    /proc/self/exe → fd → mmap → base pointer
    #    HOẶC: header đã biết offset → tính trực tiếp

    # 2. mmap VM stack (1 MB)
    mov     $9, %rax           # sys_mmap
    xor     %rdi, %rdi         # addr = NULL
    mov     $STACK_SIZE, %rsi  # length
    mov     $3, %rdx           # PROT_READ | PROT_WRITE
    mov     $0x22, %r10        # MAP_PRIVATE | MAP_ANONYMOUS
    mov     $-1, %r8           # fd = -1
    xor     %r9, %r9           # offset = 0
    syscall
    mov     %rax, %r14         # r14 = VM stack base
    add     $STACK_SIZE, %r14  # stack grows downward

    # 3. mmap heap (16 MB)
    mov     $9, %rax
    xor     %rdi, %rdi
    mov     $HEAP_SIZE, %rsi
    mov     $3, %rdx
    mov     $0x22, %r10
    mov     $-1, %r8
    xor     %r9, %r9
    syscall
    mov     %rax, %r15         # r15 = heap base (bump allocator)

    # 4. Parse header → tìm bytecode section
    # r12 = base + header.bc_offset
    # r13 = 0 (PC = start of bytecode)

    # 5. Jump to VM loop
    jmp     vm_loop
```

### Task 2: VM dispatch loop (~200 LOC)

```asm
vm_loop:
    # Fetch opcode
    movzbl  (%r12, %r13), %eax   # al = bytecode[PC]
    inc     %r13                   # PC++

    # Dispatch (jump table)
    cmp     $0x30, %al             # max opcode
    jae     vm_error_unknown_op
    lea     dispatch_table(%rip), %rcx
    movslq  (%rcx, %rax, 4), %rdx
    add     %rcx, %rdx
    jmp     *%rdx

.section .rodata
dispatch_table:
    .long op_halt - dispatch_table        # 0x00 (reserved)
    .long op_push - dispatch_table        # 0x01
    .long op_load - dispatch_table        # 0x02
    .long op_lca - dispatch_table         # 0x03
    .long op_edge - dispatch_table        # 0x04
    .long op_query - dispatch_table       # 0x05
    .long op_emit - dispatch_table        # 0x06
    .long op_call - dispatch_table        # 0x07
    .long op_ret - dispatch_table         # 0x08
    .long op_jmp - dispatch_table         # 0x09
    .long op_jz - dispatch_table          # 0x0A
    # ... etc

.section .text
```

### Task 3: Stack operations (~100 LOC)

```asm
# VM stack: mỗi entry = 16 bytes (pointer + length)
# [ptr:8][len:8] → points to chain data on heap

op_push:
    # Read chain_len (2 bytes LE)
    movzwl  (%r12, %r13), %ecx    # ecx = chain_len
    add     $2, %r13               # PC += 2

    # Copy chain bytes to heap
    mov     %r15, %rdi             # heap_ptr
    lea     (%r12, %r13), %rsi    # source = bytecode + PC
    mov     %ecx, %edx             # length
    call    memcpy_inline

    # Push (ptr, len) onto VM stack
    sub     $16, %r14
    mov     %r15, (%r14)           # ptr
    mov     %rcx, 8(%r14)          # len
    add     %rcx, %r15             # bump heap
    add     %rcx, %r13             # PC += chain_len
    jmp     vm_loop

op_push_num:                        # 0x15
    # Read f64 (8 bytes)
    sub     $16, %r14
    mov     (%r12, %r13), %rax
    mov     %rax, (%r14)           # store f64 bits
    movq    $8, 8(%r14)            # len = 8 (f64 marker)
    add     $8, %r13
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
```

### Task 4: Control flow (~100 LOC)

```asm
op_jmp:
    movl    (%r12, %r13), %eax    # target (4 bytes)
    mov     %eax, %r13d            # PC = target
    jmp     vm_loop

op_jz:
    movl    (%r12, %r13), %eax    # target
    add     $4, %r13               # skip target bytes
    # Check top of stack: empty?
    mov     8(%r14), %rcx          # len
    add     $16, %r14              # pop
    test    %rcx, %rcx
    jz      .jz_take               # len == 0 → jump
    jmp     vm_loop
.jz_take:
    mov     %eax, %r13d
    jmp     vm_loop

op_call:
    # Read function name → lookup in function table
    movzbl  (%r12, %r13), %ecx    # name_len
    inc     %r13
    # ... lookup function → get target PC
    # Push return address onto call stack
    push    %r13                    # save return PC
    mov     %eax, %r13d            # PC = function start
    jmp     vm_loop

op_ret:
    pop     %r13                    # restore return PC
    jmp     vm_loop

op_halt:
    # Exit cleanly
    mov     $60, %rax              # sys_exit
    xor     %edi, %edi             # status = 0
    syscall
```

### Task 5: Syscall bridge (~50 LOC)

```asm
# ── Syscall wrappers (no libc) ────────────────────────────────

sys_write:
    # rdi = fd, rsi = buf, rdx = len
    mov     $1, %rax
    syscall
    ret

sys_read:
    # rdi = fd, rsi = buf, rdx = len
    mov     $0, %rax
    syscall
    ret

sys_open:
    # rdi = path, rsi = flags, rdx = mode
    mov     $2, %rax
    syscall
    ret

sys_mmap:
    # rdi=addr, rsi=len, rdx=prot, r10=flags, r8=fd, r9=offset
    mov     $9, %rax
    syscall
    ret

sys_exit:
    mov     $60, %rax
    mov     %rdi, %rdi
    syscall

op_emit:
    # Pop top → write to stdout
    mov     (%r14), %rsi           # buf = chain ptr
    mov     8(%r14), %rdx          # len
    add     $16, %r14              # pop
    mov     $1, %edi               # fd = stdout
    call    sys_write
    jmp     vm_loop
```

### Task 6: LCA (hot path, ~30 LOC)

```asm
op_lca:
    # ⚠️ v2: LCA KHÔNG dùng average. Compose rules:
    #   S = Union(A,B)        — CSG union
    #   R = Compose (fixed)   — always Compose
    #   V = amplify(Va,Vb,w)  — KHÔNG trung bình
    #   A = max(Aa,Ab)        — lấy cao hơn
    #   T = dominant(Ta,Tb)   — lấy chủ đạo
    # Molecule = u16 packed [S:4][R:4][V:3][A:3][T:2] = 2 bytes
    #
    # TODO(v2): Implement v2 LCA rules thay vì average
    # Pop 2 chains, compute LCA per v2 spec
    # Chain A: (%r14)
    # Chain B: 16(%r14)

    mov     (%r14), %rsi           # chain A ptr
    mov     16(%r14), %rdi         # chain B ptr
    add     $32, %r14              # pop 2

    # Allocate result on heap (2 bytes — v2 Molecule = u16)
    mov     %r15, %rax
    # TODO(v2): implement amplify/Union/max/dominant
    # Current (LEGACY): (A[i] + B[i]) / 2
    movzbl  (%rsi), %ecx
    movzbl  (%rdi), %edx
    add     %edx, %ecx
    shr     $1, %ecx
    movb    %cl, (%r15)
    # ... repeat for dims 1-4

    # Push result
    sub     $16, %r14
    mov     %rax, (%r14)
    movq    $5, 8(%r14)
    add     $5, %r15
    jmp     vm_loop
```

### Task 7: Math ops (dùng x87 FPU, ~80 LOC)

```asm
# f64 operations — dùng SSE2 (mọi x86_64 CPU đều có)

op_add:
    movsd   (%r14), %xmm0         # A
    movsd   16(%r14), %xmm1       # B
    add     $16, %r14              # pop 1
    addsd   %xmm1, %xmm0
    movsd   %xmm0, (%r14)
    jmp     vm_loop

op_mul:
    movsd   (%r14), %xmm0
    movsd   16(%r14), %xmm1
    add     $16, %r14
    mulsd   %xmm1, %xmm0
    movsd   %xmm0, (%r14)
    jmp     vm_loop

op_div:
    movsd   (%r14), %xmm0
    # Check zero
    xorpd   %xmm2, %xmm2
    ucomisd %xmm2, %xmm0
    je      vm_error_div_zero
    movsd   16(%r14), %xmm1
    add     $16, %r14
    divsd   %xmm0, %xmm1
    movsd   %xmm1, (%r14)
    jmp     vm_loop
```

---

## Build process

```bash
# Assemble
as -o vm_x86_64.o vm/x86_64/vm_x86_64.S

# Link (no libc, static)
ld -o vm_x86_64 vm_x86_64.o -e _start --static -nostdlib

# Test standalone
echo "Hello" | ./vm_x86_64

# Integrate with builder (PLAN_1_4):
# builder packs vm_x86_64 code + bytecode + knowledge → origin.olang
```

---

## Test plan

```bash
# 1. Minimal: VM chạy, print, exit
# Bytecode: [PushStr "Hello from ASM VM\n"] [Emit] [Halt]
./test_vm_hello

# 2. Math: 2 + 3 = 5
# Bytecode: [PushNum 2] [PushNum 3] [Add] [Emit] [Halt]
./test_vm_math

# 3. Control flow: countdown loop
# Bytecode: [PushNum 10] [Store "n"] [Loop: LoadLocal "n" → Emit → Dec → Jz exit]
./test_vm_loop

# 4. LCA: compute LCA of 2 molecules
./test_vm_lca

# 5. Full: chạy lexer.ol bytecode
./test_vm_lexer
```

---

## LOC estimate

```
_start + memory setup:    ~80 LOC
vm_loop + dispatch:       ~60 LOC
Stack ops (8 opcodes):    ~120 LOC
Control flow (6 opcodes): ~100 LOC
Syscall bridge:           ~60 LOC
Math ops (SSE2):          ~120 LOC
String ops:               ~200 LOC
Chain ops (LCA, hash):    ~300 LOC
Crypto (SHA-256):         ~400 LOC (optional phase 1)
Error handling:           ~100 LOC
───────────────────────────
Total:                    ~1,540 LOC (without crypto)
                          ~1,940 LOC (with SHA-256)
```

## Definition of Done

- [ ] `vm_x86_64.S` assemble + link thành công (no libc)
- [ ] VM print "Hello" → stdout → exit 0
- [ ] VM execute PushNum + Add + Emit → output "5"
- [ ] VM execute Jmp + Jz → loop chạy đúng
- [ ] VM chạy lexer.ol bytecode → tokenize "let x = 42;" → output tokens

## Ước tính: 2-3 tuần

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 1.1*
