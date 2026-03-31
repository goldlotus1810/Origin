// homeos/asm_emit.ol — x86_64 machine code emitter
// Emit raw instruction bytes → no external assembler needed.
//
// x86_64 encoding: [prefixes][REX][opcode][ModR/M][SIB][disp][imm]
// REX prefix: 0x40 + W(3)R(2)X(1)B(0)
//   W=1 → 64-bit operand size

// ── Register codes ──
let RAX = 0; let RCX = 1; let RDX = 2; let RBX = 3;
let RSP = 4; let RBP = 5; let RSI = 6; let RDI = 7;
let R8  = 8; let R9  = 9; let R10 = 10; let R11 = 11;
let R12 = 12; let R13 = 13; let R14 = 14; let R15 = 15;

// XMM registers (for SSE2 f64)
let XMM0 = 0; let XMM1 = 1; let XMM2 = 2; let XMM3 = 3;

// ── Code buffer ──

pub fn code_new() {
  return { bytes: [], labels: {}, fixups: [] };
}

pub fn code_len(code) { return len(code.bytes); }

fn asm_asm_emit_byte(code, b) { push(code.bytes, b); }

fn asm_asm_emit_u16_le(code, val) {
  asm_emit_byte(code, val % 256);
  asm_emit_byte(code, (val / 256) % 256);
}

fn asm_asm_emit_u32_le(code, val) {
  asm_emit_byte(code, val % 256);
  asm_emit_byte(code, (val / 256) % 256);
  asm_emit_byte(code, (val / 65536) % 256);
  asm_emit_byte(code, (val / 16777216) % 256);
}

fn emit_u64_le(code, val) {
  asm_emit_u32_le(code, val % 4294967296);
  asm_emit_u32_le(code, val / 4294967296);
}

// ── REX prefix ──

fn rex_w(reg, rm) {
  // REX.W = 1 (64-bit), R from reg, B from rm
  let r_ext = 0;
  let b_ext = 0;
  if reg >= 8 { r_ext = 4; }
  if rm >= 8 { b_ext = 1; }
  return 0x48 + r_ext + b_ext;
}

fn needs_rex(reg, rm) {
  return reg >= 8 || rm >= 8;
}

// ── ModR/M byte ──

fn modrm(mod_val, reg, rm) {
  return ((mod_val % 4) * 64) + ((reg % 8) * 8) + (rm % 8);
}

// ═══════════════════════════════════════════════════════════
// INSTRUCTIONS
// ═══════════════════════════════════════════════════════════

// ── MOV ──

pub fn emit_mov_reg_imm64(code, reg, imm) {
  // REX.W + B8+rd + imm64
  asm_emit_byte(code, rex_w(0, reg));
  asm_emit_byte(code, 0xB8 + (reg % 8));
  emit_u64_le(code, imm);
}

pub fn emit_mov_reg_imm32(code, reg, imm) {
  // For 32-bit immediates (zero-extended)
  if reg >= 8 {
    asm_emit_byte(code, 0x41);  // REX.B
  }
  asm_emit_byte(code, 0xB8 + (reg % 8));
  asm_emit_u32_le(code, imm);
}

pub fn emit_mov_reg_reg(code, dst, src) {
  // REX.W + 89 /r (MOV r/m64, r64)
  asm_emit_byte(code, rex_w(src, dst));
  asm_emit_byte(code, 0x89);
  asm_emit_byte(code, modrm(3, src, dst));
}

pub fn emit_mov_reg_mem(code, dst, base, disp) {
  // REX.W + 8B /r + disp (MOV r64, [base+disp])
  asm_emit_byte(code, rex_w(dst, base));
  asm_emit_byte(code, 0x8B);
  if disp == 0 && (base % 8) != 5 {
    asm_emit_byte(code, modrm(0, dst, base));
  } else {
    if disp >= -128 && disp <= 127 {
      asm_emit_byte(code, modrm(1, dst, base));
      asm_emit_byte(code, disp % 256);
    } else {
      asm_emit_byte(code, modrm(2, dst, base));
      asm_emit_u32_le(code, disp);
    }
  }
}

pub fn emit_mov_mem_reg(code, base, disp, src) {
  // REX.W + 89 /r + disp (MOV [base+disp], r64)
  asm_emit_byte(code, rex_w(src, base));
  asm_emit_byte(code, 0x89);
  if disp == 0 && (base % 8) != 5 {
    asm_emit_byte(code, modrm(0, src, base));
  } else {
    if disp >= -128 && disp <= 127 {
      asm_emit_byte(code, modrm(1, src, base));
      asm_emit_byte(code, disp % 256);
    } else {
      asm_emit_byte(code, modrm(2, src, base));
      asm_emit_u32_le(code, disp);
    }
  }
}

// ── PUSH/POP ──

pub fn emit_push(code, reg) {
  if reg >= 8 { asm_emit_byte(code, 0x41); }
  asm_emit_byte(code, 0x50 + (reg % 8));
}

pub fn emit_pop(code, reg) {
  if reg >= 8 { asm_emit_byte(code, 0x41); }
  asm_emit_byte(code, 0x58 + (reg % 8));
}

// ── ALU ──

pub fn emit_add_reg_reg(code, dst, src) {
  asm_emit_byte(code, rex_w(src, dst));
  asm_emit_byte(code, 0x01);
  asm_emit_byte(code, modrm(3, src, dst));
}

pub fn emit_sub_reg_reg(code, dst, src) {
  asm_emit_byte(code, rex_w(src, dst));
  asm_emit_byte(code, 0x29);
  asm_emit_byte(code, modrm(3, src, dst));
}

pub fn emit_xor_reg_reg(code, dst, src) {
  asm_emit_byte(code, rex_w(src, dst));
  asm_emit_byte(code, 0x31);
  asm_emit_byte(code, modrm(3, src, dst));
}

pub fn emit_cmp_reg_reg(code, a, b) {
  asm_emit_byte(code, rex_w(b, a));
  asm_emit_byte(code, 0x39);
  asm_emit_byte(code, modrm(3, b, a));
}

pub fn emit_add_reg_imm32(code, reg, imm) {
  asm_emit_byte(code, rex_w(0, reg));
  asm_emit_byte(code, 0x81);
  asm_emit_byte(code, modrm(3, 0, reg));  // /0 = ADD
  asm_emit_u32_le(code, imm);
}

pub fn emit_sub_reg_imm32(code, reg, imm) {
  asm_emit_byte(code, rex_w(0, reg));
  asm_emit_byte(code, 0x81);
  asm_emit_byte(code, modrm(3, 5, reg));  // /5 = SUB
  asm_emit_u32_le(code, imm);
}

pub fn emit_inc(code, reg) {
  asm_emit_byte(code, rex_w(0, reg));
  asm_emit_byte(code, 0xFF);
  asm_emit_byte(code, modrm(3, 0, reg));  // /0 = INC
}

pub fn emit_dec(code, reg) {
  asm_emit_byte(code, rex_w(0, reg));
  asm_emit_byte(code, 0xFF);
  asm_emit_byte(code, modrm(3, 1, reg));  // /1 = DEC
}

// ── JUMPS ──

pub fn emit_jmp_rel32(code, offset) {
  asm_emit_byte(code, 0xE9);
  asm_emit_u32_le(code, offset);
}

pub fn emit_je_rel32(code, offset) {
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x84);
  asm_emit_u32_le(code, offset);
}

pub fn emit_jne_rel32(code, offset) {
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x85);
  asm_emit_u32_le(code, offset);
}

pub fn emit_jb_rel32(code, offset) {
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x82);
  asm_emit_u32_le(code, offset);
}

pub fn emit_jae_rel32(code, offset) {
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x83);
  asm_emit_u32_le(code, offset);
}

// ── CALL/RET ──

pub fn emit_call_rel32(code, offset) {
  asm_emit_byte(code, 0xE8);
  asm_emit_u32_le(code, offset);
}

pub fn emit_ret(code) {
  asm_emit_byte(code, 0xC3);
}

// ── SYSCALL ──

pub fn emit_syscall(code) {
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x05);
}

// ── NOP ──

pub fn emit_nop(code) {
  asm_emit_byte(code, 0x90);
}

// ── SSE2 (f64) ──

pub fn emit_movsd_load(code, xmm, base, disp) {
  // F2 REX 0F 10 /r — MOVSD xmm, [mem]
  asm_emit_byte(code, 0xF2);
  if base >= 8 || xmm >= 8 {
    asm_emit_byte(code, rex_w(xmm, base) - 8);  // REX without W
  }
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x10);
  if disp == 0 && (base % 8) != 5 {
    asm_emit_byte(code, modrm(0, xmm, base));
  } else {
    asm_emit_byte(code, modrm(1, xmm, base));
    asm_emit_byte(code, disp % 256);
  }
}

pub fn emit_movsd_store(code, base, disp, xmm) {
  // F2 REX 0F 11 /r — MOVSD [mem], xmm
  asm_emit_byte(code, 0xF2);
  if base >= 8 || xmm >= 8 {
    asm_emit_byte(code, rex_w(xmm, base) - 8);
  }
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x11);
  if disp == 0 && (base % 8) != 5 {
    asm_emit_byte(code, modrm(0, xmm, base));
  } else {
    asm_emit_byte(code, modrm(1, xmm, base));
    asm_emit_byte(code, disp % 256);
  }
}

pub fn emit_addsd(code, dst, src) {
  asm_emit_byte(code, 0xF2);
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x58);
  asm_emit_byte(code, modrm(3, dst, src));
}

pub fn emit_subsd(code, dst, src) {
  asm_emit_byte(code, 0xF2);
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x5C);
  asm_emit_byte(code, modrm(3, dst, src));
}

pub fn emit_mulsd(code, dst, src) {
  asm_emit_byte(code, 0xF2);
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x59);
  asm_emit_byte(code, modrm(3, dst, src));
}

pub fn emit_divsd(code, dst, src) {
  asm_emit_byte(code, 0xF2);
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x5E);
  asm_emit_byte(code, modrm(3, dst, src));
}

// ── Labels & Fixups ──

pub fn label(code, name) {
  code.labels[name] = code_len(code);
}

pub fn emit_jmp_label(code, name) {
  // Emit jmp with fixup
  asm_emit_byte(code, 0xE9);
  push(code.fixups, { pos: code_len(code), label: name, size: 4 });
  asm_emit_u32_le(code, 0);  // placeholder
}

pub fn emit_je_label(code, name) {
  asm_emit_byte(code, 0x0F);
  asm_emit_byte(code, 0x84);
  push(code.fixups, { pos: code_len(code), label: name, size: 4 });
  asm_emit_u32_le(code, 0);
}

pub fn resolve_fixups(code) {
  let i = 0;
  while i < len(code.fixups) {
    let f = code.fixups[i];
    let target = code.labels[f.label];
    if target != 0 {
      // rel32 = target - (fixup_pos + 4)
      let rel = target - (f.pos + 4);
      // Patch bytes
      code.bytes[f.pos] = rel % 256;
      code.bytes[f.pos + 1] = (rel / 256) % 256;
      code.bytes[f.pos + 2] = (rel / 65536) % 256;
      code.bytes[f.pos + 3] = (rel / 16777216) % 256;
    }
    i = i + 1;
  }
}

// ── Convenience: emit common patterns ──

pub fn emit_mov_rax_imm(code, val) { emit_mov_reg_imm64(code, RAX, val); }
pub fn emit_xor_rdi_rdi(code) { emit_xor_reg_reg(code, RDI, RDI); }
pub fn emit_exit_0(code) {
  emit_mov_reg_imm32(code, RAX, 60);  // sys_exit
  emit_xor_reg_reg(code, RDI, RDI);    // status = 0
  emit_syscall(code);
}
