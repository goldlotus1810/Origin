# PLAN 3.1 — asm_emit.ol: Olang emit Machine Code (~500 LOC)

**Phụ thuộc:** Phase 2 DONE (stdlib + HomeOS logic bằng Olang)
**Mục tiêu:** Olang tự emit x86_64 machine code → không cần external assembler
**Tham chiếu:** `vm/x86_64/vm_x86_64.S` (reference implementation)

---

## Bối cảnh

```
HIỆN TẠI:
  vm_x86_64.S → as (GNU assembler) → vm_x86_64.o → ld → binary
  CẦN: as + ld (external tools)

SAU PLAN 3.1:
  asm_emit.ol chạy trên origin.olang → emit bytes trực tiếp
  KHÔNG CẦN: as, ld, hay bất kỳ tool nào
```

---

## Cấu trúc

### Instruction Encoding Table

```
// x86_64 instruction → byte sequence
// Chỉ cần ~30 instructions cho VM

fn emit_mov_reg_imm64(reg, val) { ... }     // REX.W + B8+rd + imm64
fn emit_mov_reg_reg(dst, src) { ... }       // REX.W + 89 /r
fn emit_mov_reg_mem(dst, base, off) { ... } // REX.W + 8B /r + disp
fn emit_mov_mem_reg(base, off, src) { ... } // REX.W + 89 /r + disp
fn emit_push_reg(reg) { ... }               // 50+rd
fn emit_pop_reg(reg) { ... }                // 58+rd
fn emit_add_reg_reg(dst, src) { ... }       // REX.W + 01 /r
fn emit_sub_reg_reg(dst, src) { ... }       // REX.W + 29 /r
fn emit_xor_reg_reg(dst, src) { ... }       // REX.W + 31 /r
fn emit_cmp_reg_reg(a, b) { ... }           // REX.W + 39 /r
fn emit_cmp_reg_imm(reg, val) { ... }       // REX.W + 81 /7 + imm32
fn emit_jmp_rel32(offset) { ... }           // E9 + rel32
fn emit_je_rel32(offset) { ... }            // 0F 84 + rel32
fn emit_jne_rel32(offset) { ... }           // 0F 85 + rel32
fn emit_call_rel32(offset) { ... }          // E8 + rel32
fn emit_ret() { ... }                       // C3
fn emit_syscall() { ... }                   // 0F 05
fn emit_nop() { ... }                       // 90

// SSE2 (f64)
fn emit_movsd_xmm_mem(xmm, base, off) { ... }
fn emit_addsd(dst, src) { ... }
fn emit_subsd(dst, src) { ... }
fn emit_mulsd(dst, src) { ... }
fn emit_divsd(dst, src) { ... }
```

### Register Encoding

```
// x86_64 register → 4-bit code
fn reg_code(name) {
  if name == "rax" { return 0; }
  if name == "rcx" { return 1; }
  if name == "rdx" { return 2; }
  if name == "rbx" { return 3; }
  if name == "rsp" { return 4; }
  if name == "rbp" { return 5; }
  if name == "rsi" { return 6; }
  if name == "rdi" { return 7; }
  if name == "r8"  { return 8; }
  // ... r9-r15
}
```

---

## Test Plan

```
// Test 1: emit nop → verify byte = 0x90
let code = emit_nop();
assert(code[0] == 0x90);

// Test 2: emit mov rax, 42 → verify encoding
let code = emit_mov_reg_imm64("rax", 42);
assert(code[0] == 0x48);  // REX.W
assert(code[1] == 0xB8);  // B8 + rax(0)
// imm64 = 42 LE

// Test 3: emit syscall → verify 0F 05
let code = emit_syscall();
assert(code[0] == 0x0F);
assert(code[1] == 0x05);

// Test 4: assemble mini program → run
// mov rax, 60 (sys_exit); xor rdi, rdi; syscall
// → should exit 0
```

---

## Definition of Done

- [ ] 30+ instruction emitters
- [ ] Register encoding table
- [ ] ModR/M byte generation
- [ ] REX prefix handling
- [ ] SSE2 f64 instructions
- [ ] Test: emit + execute minimal program

## Ước tính: 2-3 ngày
