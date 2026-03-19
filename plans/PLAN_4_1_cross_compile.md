# PLAN 4.1 — Cross-Compile: x86_64 → ARM64

**Phụ thuộc:** Phase 3 DONE (builder.ol tự build origin.olang)
**Mục tiêu:** origin.olang (x86_64) sinh ra origin.olang (ARM64)
**Tham chiếu:** `vm/arm64/vm_arm64.S`, `stdlib/homeos/asm_emit.ol`

---

## Bối cảnh

```
HIỆN TẠI:
  asm_emit.ol chỉ emit x86_64 instructions
  builder.ol chỉ tạo x86_64 ELF
  ARM64 VM có sẵn (vm_arm64.S, 627 LOC) nhưng cần external assembler

SAU PLAN 4.1:
  origin.olang (x86_64) chạy asm_emit_arm64.ol → emit ARM64 machine code
  elf_emit.ol mở rộng → hỗ trợ ARM64 ELF (e_machine = 0xB7)
  builder.ol chấp nhận --arch arm64 → tạo origin.olang cho ARM64
  1 máy x86_64 → build cho TẤT CẢ architecture
```

---

## Tasks

### 4.1.1 — asm_emit_arm64.ol (~400 LOC)

ARM64 instruction encoding đơn giản hơn x86_64 (fixed-width 32-bit).

```
// ARM64 instruction format: 32 bits, fixed-width
// Thanh ghi: W0-W30 (32-bit), X0-X30 (64-bit), SP, XZR

fn emit_arm64_instr(code, instr32) { ... }  // push 4 bytes LE

// Data processing — immediate
fn emit_movz(code, rd, imm16, shift) { ... }      // MOVZ Xd, #imm16, LSL #shift
fn emit_movk(code, rd, imm16, shift) { ... }      // MOVK Xd, #imm16, LSL #shift
fn emit_mov_imm64(code, rd, val) { ... }           // MOVZ+MOVK×3 → load 64-bit

// Arithmetic
fn emit_add_reg(code, rd, rn, rm) { ... }          // ADD Xd, Xn, Xm
fn emit_sub_reg(code, rd, rn, rm) { ... }          // SUB Xd, Xn, Xm
fn emit_add_imm(code, rd, rn, imm12) { ... }       // ADD Xd, Xn, #imm12
fn emit_sub_imm(code, rd, rn, imm12) { ... }       // SUB Xd, Xn, #imm12
fn emit_cmp_reg(code, rn, rm) { ... }              // SUBS XZR, Xn, Xm
fn emit_cmp_imm(code, rn, imm12) { ... }           // SUBS XZR, Xn, #imm12

// Logic
fn emit_and_reg(code, rd, rn, rm) { ... }
fn emit_orr_reg(code, rd, rn, rm) { ... }
fn emit_eor_reg(code, rd, rn, rm) { ... }

// Memory
fn emit_ldr(code, rt, rn, offset) { ... }          // LDR Xt, [Xn, #offset]
fn emit_str(code, rt, rn, offset) { ... }          // STR Xt, [Xn, #offset]
fn emit_ldrb(code, wt, xn, offset) { ... }         // LDRB Wt, [Xn, #offset]
fn emit_stp(code, rt, rt2, rn, offset) { ... }     // STP Xt, Xt2, [Xn, #offset]!
fn emit_ldp(code, rt, rt2, rn, offset) { ... }     // LDP Xt, Xt2, [Xn, #offset]!

// Branch
fn emit_b(code, offset) { ... }                    // B #offset (26-bit signed)
fn emit_bl(code, offset) { ... }                   // BL #offset (function call)
fn emit_b_cond(code, cond, offset) { ... }         // B.cond #offset (19-bit)
fn emit_cbz(code, rt, offset) { ... }              // CBZ Xt, #offset
fn emit_cbnz(code, rt, offset) { ... }             // CBNZ Xt, #offset
fn emit_ret(code) { ... }                          // RET X30

// System
fn emit_svc(code, imm16) { ... }                   // SVC #0 (syscall)
fn emit_nop(code) { ... }                          // NOP

// NEON/FP (f64)
fn emit_fmov_d(code, dd, dn) { ... }               // FMOV Dd, Dn
fn emit_fadd_d(code, dd, dn, dm) { ... }            // FADD Dd, Dn, Dm
fn emit_fsub_d(code, dd, dn, dm) { ... }            // FSUB Dd, Dn, Dm
fn emit_fmul_d(code, dd, dn, dm) { ... }            // FMUL Dd, Dn, Dm
fn emit_fdiv_d(code, dd, dn, dm) { ... }            // FDIV Dd, Dn, Dm
fn emit_ldr_d(code, dt, xn, offset) { ... }         // LDR Dt, [Xn, #offset]
fn emit_str_d(code, dt, xn, offset) { ... }         // STR Dt, [Xn, #offset]

// Label + fixup (giống asm_emit.ol x86_64)
fn label(code, name) { ... }
fn fixup_branch(code, name) { ... }
fn resolve_fixups(code) { ... }
```

**Encoding reference (fixed-width 32-bit):**
```
ADD  Xd, Xn, Xm   → 10001011 000 Rm   000000 Rn    Rd     (0x8B000000 + regs)
SUB  Xd, Xn, Xm   → 11001011 000 Rm   000000 Rn    Rd     (0xCB000000 + regs)
LDR  Xt, [Xn, #o]  → 11111001 01 imm12       Rn     Rt     (0xF9400000 + ...)
STR  Xt, [Xn, #o]  → 11111001 00 imm12       Rn     Rt     (0xF9000000 + ...)
B    #off           → 000101   imm26                        (0x14000000 + off/4)
BL   #off           → 100101   imm26                        (0x94000000 + off/4)
SVC  #0             → 11010100 000 1 imm16 000 01           (0xD4000001)
```

### 4.1.2 — elf_emit.ol mở rộng

Thêm hỗ trợ ARM64 vào `elf_emit.ol`:

```
Thay đổi trong make_elf():
  - Nhận thêm param `arch` ("x86_64" hoặc "arm64")
  - e_machine: 0x3E (x86_64) hoặc 0xB7 (ARM64 = EM_AARCH64)
  - p_align: 0x1000 (cả hai)
  - Entry point: tính giống nhau

Thay đổi trong make_origin_header():
  - Arch byte: 0x01 (x86_64) hoặc 0x02 (arm64)
```

Ước tính: ~15 LOC thay đổi.

### 4.1.3 — builder.ol mở rộng

```
Thay đổi:
  - default_config() thêm field `arch: "x86_64"`
  - build(config): truyền config.arch cho make_elf() và make_origin_header()
  - compile_all(): bytecode KHÔNG đổi (arch-independent)
  - VM selection: config.vm_path thay đổi theo arch:
    arch == "x86_64" → "vm/x86_64/vm_x86_64.bin"
    arch == "arm64"  → "vm/arm64/vm_arm64.bin"
```

### 4.1.4 — ARM64 op_call fix

Hiện tại `vm/arm64/vm_arm64.S:449` op_call là stub. Cần implement:
- Parse function name từ bytecode
- Hash name (FNV-1a)
- Dispatch built-in functions (len, push, pop, char_at, substr, emit, etc.)
- Tham chiếu: x86_64 `cg_call` implementation

---

## Rào cản

```
1. ARM64 VM chưa test runtime (QEMU unavailable)
   → Giải pháp: test trên máy ARM64 thật hoặc CI with QEMU
   → Fallback: cross-compile + manual verify trên Raspberry Pi / M1 Mac

2. ARM64 op_call là stub
   → PHẢI fix trước khi cross-compile có ý nghĩa
   → Ước tính: ~200 LOC ASM (tham chiếu x86_64 cg_call)

3. ARM64 syscall numbers khác Linux x86_64
   → Linux ARM64: read=63, write=64, mmap=222, exit=93
   → Đã đúng trong vm_arm64.S (SYS_READ=63, SYS_WRITE=64, SYS_EXIT=93)
```

---

## Test Plan

```
Test 1: asm_emit_arm64.ol — emit NOP → verify 4 bytes = 0xD503201F
Test 2: emit MOV X0, #42 → verify encoding
Test 3: emit SVC #0 → verify 0xD4000001
Test 4: build ARM64 origin.olang → readelf -h → verify EM_AARCH64
Test 5: (nếu có QEMU/ARM64 máy thật) chạy origin.olang ARM64 → "Hello"
```

---

## Definition of Done

- [ ] asm_emit_arm64.ol: 30+ ARM64 instruction emitters
- [ ] elf_emit.ol: hỗ trợ ARM64 ELF (e_machine = 0xB7)
- [ ] builder.ol: --arch arm64 tạo binary đúng
- [ ] ARM64 op_call: implemented (không còn stub)
- [ ] Test: tạo ARM64 ELF, readelf xác nhận valid

## Ước tính: 3-5 ngày
