# PLAN 5.1 — JIT Compilation

**Phụ thuộc:** Phase 4 DONE (multi-architecture)
**Mục tiêu:** VM detect hot loops → compile to native at runtime → near-native speed
**Tham chiếu:** `stdlib/homeos/asm_emit.ol`, `vm/x86_64/vm_x86_64.S`

---

## Bối cảnh

```
HIỆN TẠI:
  VM interpret từng opcode → fetch-decode-execute mỗi iteration
  Hot loop chạy 1000 lần = 1000 × dispatch overhead

SAU PLAN 5.1:
  VM detect: "loop body X chạy > threshold lần"
  → JIT compile loop body → native code (dùng asm_emit.ol)
  → Chạy native code trực tiếp → gần 0 dispatch overhead
  → Tự động, transparent với bytecode
```

---

## Thiết kế

### Profiling (counting)

```
Mỗi backward jump (loop header) có counter:

loop_counters: BTreeMap<u32, u32>   // pc → count
  Key = PC của loop header
  Value = số lần jump backward về PC này

Threshold: Fibonacci-based (QT⑰)
  Fib[10] = 55 → JIT compile sau 55 iterations
  Tại sao 55? Đủ warm-up, tránh compile code chạy 1-2 lần.
```

### JIT Pipeline

```
1. DETECT: backward jump + counter >= 55
     ↓
2. TRACE: record opcodes từ loop header → loop end
     ↓
3. COMPILE: opcodes → native code (asm_emit.ol)
     - PushNum → mov reg, imm64
     - Add → addsd xmm0, xmm1
     - Store → mov [var_table + hash*24], reg
     - Load → mov reg, [var_table + hash*24]
     - Jz → test + je
     - Emit → mov rax,1; syscall
     ↓
4. INSTALL: mmap RWX page → write native code → mprotect RX
     ↓
5. PATCH: loop header → jmp to native code
     ↓
6. EXECUTE: native code chạy trực tiếp, return khi loop ends
```

### Opcode → Native mapping (x86_64)

```
Bytecode          Native (x86_64)
─────────────────────────────────────────────────────
PushNum(val)      mov rax, val; push rax
Push(chain)       lea rax, [data+off]; push rax; push len
Load(name)        hash(name) → lookup var_table → push
Store(name)       pop → hash(name) → write var_table
Add               pop rbx; pop rax; add rax, rbx; push rax
  (f64)           movsd xmm1, [rsp]; add rsp,16;
                  movsd xmm0, [rsp]; addsd xmm0, xmm1;
                  movsd [rsp], xmm0
Sub/Mul/Div       tương tự Add
Jz(target)        pop rax; test rax,rax; je native_target
Jmp(target)       jmp native_target
Emit              call emit_helper (syscall write)
Loop(count)       dec rcx; jnz loop_top
```

### Guard + bailout

```
JIT code cần "bailout" khi gặp opcode không compile được:
  - Call (dynamic dispatch)
  - Dream/Stats/Fuse (complex side effects)
  - TryBegin/CatchEnd (exception handling)

Bailout:
  1. Save VM state (stack, pc, vars) vào shared memory
  2. Return to interpreter
  3. Interpreter tiếp tục từ bailout point

Guard:
  - Type guard: nếu var đổi type (chain → f64) → bailout
  - Range guard: nếu array index out of bounds → bailout
```

---

## Implementation

### 5.1.1 — jit.ol: JIT compiler (~400 LOC)

```
pub fn jit_compile(trace) → native_code
  // trace = Vec<Op> (recorded from interpreter)
  // native_code = bytes (machine code)

  let code = code_new();  // from asm_emit.ol

  // Prologue: save callee-saved registers
  emit_push_reg(code, RBP);
  emit_mov_reg_reg(code, RBP, RSP);
  emit_push_reg(code, R12);  // bytecode base
  emit_push_reg(code, R14);  // stack ptr
  emit_push_reg(code, R15);  // heap ptr

  // Load VM state from shared memory
  emit_mov_reg_mem(code, R14, RDI, 0);   // stack_ptr
  emit_mov_reg_mem(code, R15, RDI, 8);   // heap_ptr

  // Compile each op
  for op in trace {
    jit_emit_op(code, op);
  }

  // Epilogue: store VM state back
  emit_mov_mem_reg(code, RDI, 0, R14);
  emit_mov_mem_reg(code, RDI, 8, R15);
  emit_pop_reg(code, R15);
  emit_pop_reg(code, R14);
  emit_pop_reg(code, R12);
  emit_pop_reg(code, RBP);
  emit_ret(code);

  resolve_fixups(code);
  return code.bytes;
```

### 5.1.2 — VM integration (~100 LOC ASM)

```asm
;; In vm_loop, at backward jump:
check_jit:
    ;; counter[pc]++
    ;; if counter[pc] >= 55:
    ;;   call jit_compile(trace)
    ;;   patch this jump → direct native call
    ;; else:
    ;;   continue interpreting

;; JIT entry:
jit_entry:
    ;; Load VM state struct address → rdi
    lea     rdi, [vm_state(%rip)]
    ;; Call JIT-compiled native code
    call    *%rax           ;; rax = native code pointer
    ;; VM state updated in-place
    jmp     vm_loop
```

### 5.1.3 — mmap RWX → RX (~30 LOC)

```
Syscall sequence:
  1. mmap(NULL, size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0)
     → get writable page
  2. memcpy(page, native_code, len)
  3. mprotect(page, size, PROT_READ|PROT_EXEC)
     → make executable, no longer writable (W^X)
```

---

## Rào cản

```
1. W^X enforcement (macOS, some Linux hardened)
   → Giải pháp: mmap RW → write → mprotect RX (never RWX simultaneously)
   → macOS: MAP_JIT flag + pthread_jit_write_protect_np()

2. JIT code correctness
   → Giải pháp: fall back to interpreter if any doubt
   → Conservative: only JIT simple numeric loops initially
   → Add more opcodes to JIT gradually

3. Code cache management
   → Simple: fixed-size arena (1 MB), evict LRU
   → Fibonacci-based eviction: keep most-fired, evict least-fired

4. ⚠️ [THỰC TẾ] Hai bytecode format → JIT phải hỗ trợ cả hai
   → ir.rs format (flags bit 0 = 0): opcodes 0x00-0x83
   → codegen format (flags bit 0 = 1): opcodes 0x01-0x24
   → JIT trace recording cần biết format đang chạy
   → Đề xuất: JIT chỉ hỗ trợ codegen format trước (simpler, denser)
   → VM đã có bc_format flag (vm_x86_64.S) → JIT đọc flag này

5. ⚠️ [THỰC TẾ] VM hiện tại exit ngay sau khi load bytecode nếu không có entry point
   → origin.olang hiện tại: load 811KB bytecode → exit 0 (không crash, nhưng không làm gì)
   → JIT cần bytecode CÓ hot loops → cần entry point function hoặc main()
   → Phụ thuộc: stdlib cần fn main() hoặc VM auto-execute first function
```

---

## Test Plan

```
Test 1: Simple loop — for i in 0..100: sum += i → verify sum == 4950
  Interpreter: ~500μs, JIT: ~10μs (50× faster)

Test 2: Nested loop — matrix multiply 10×10
  Verify correctness + measure speedup

Test 3: Bailout — loop with Call inside → should not JIT, stay interpreted

Test 4: Type guard — loop starts numeric, var becomes string → bailout

Test 5: Memory — JIT 100 loops → verify code cache doesn't leak
```

---

## Definition of Done

- [ ] Loop counter profiling (detect hot loops)
- [ ] Trace recording (capture opcodes in loop body)
- [ ] JIT compiler: PushNum, Add/Sub/Mul/Div, Load, Store, Jz, Jmp
- [ ] mmap + mprotect (W^X compliant)
- [ ] Bailout mechanism (fall back to interpreter)
- [ ] Test: JIT'd loop produces correct result
- [ ] Benchmark: measurable speedup on numeric loops

## Ước tính: 1-2 tuần
