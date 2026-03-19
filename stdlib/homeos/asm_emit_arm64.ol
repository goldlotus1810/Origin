// homeos/asm_emit_arm64.ol — ARM64 (AArch64) machine code emitter
// Emit raw 32-bit fixed-width instructions → no external assembler needed.
//
// ARM64 encoding: all instructions are exactly 4 bytes (little-endian).
// Register naming: X0-X30 (64-bit), W0-W30 (32-bit), XZR/WZR = 31, SP = 31 (context-dependent)
//
// Reference: ARM Architecture Reference Manual (ARMv8-A)

// ── Register codes ──
let X0 = 0; let X1 = 1; let X2 = 2; let X3 = 3;
let X4 = 4; let X5 = 5; let X6 = 6; let X7 = 7;
let X8 = 8; let X9 = 9; let X10 = 10; let X11 = 11;
let X12 = 12; let X13 = 13; let X14 = 14; let X15 = 15;
let X16 = 16; let X17 = 17; let X18 = 18; let X19 = 19;
let X20 = 20; let X21 = 21; let X22 = 22; let X23 = 23;
let X24 = 24; let X25 = 25; let X26 = 26; let X27 = 27;
let X28 = 28; let X29 = 29; let X30 = 30;
let XZR = 31; let SP = 31;

// NEON/FP D registers (64-bit float)
let D0 = 0; let D1 = 1; let D2 = 2; let D3 = 3;
let D4 = 4; let D5 = 5; let D6 = 6; let D7 = 7;

// Condition codes for B.cond
let COND_EQ = 0;   // equal (Z=1)
let COND_NE = 1;   // not equal (Z=0)
let COND_HS = 2;   // unsigned >= (C=1)  (also CS)
let COND_LO = 3;   // unsigned < (C=0)   (also CC)
let COND_MI = 4;   // negative (N=1)
let COND_PL = 5;   // positive (N=0)
let COND_HI = 8;   // unsigned > (C=1 && Z=0)
let COND_LS = 9;   // unsigned <= (C=0 || Z=1)
let COND_GE = 10;  // signed >=
let COND_LT = 11;  // signed <
let COND_GT = 12;  // signed >
let COND_LE = 13;  // signed <=

// ── Code buffer ──

pub fn code_new() {
  return { bytes: [], labels: {}, fixups: [] };
}

pub fn code_len(code) { return len(code.bytes); }

fn emit_byte(code, b) { push(code.bytes, b % 256); }

fn emit_u32_le(code, val) {
  emit_byte(code, val % 256);
  emit_byte(code, (val / 256) % 256);
  emit_byte(code, (val / 65536) % 256);
  emit_byte(code, (val / 16777216) % 256);
}

// Emit a single 32-bit ARM64 instruction (little-endian)
fn emit_instr(code, instr) {
  emit_u32_le(code, instr);
}

// ═══════════════════════════════════════════════════════════
// DATA PROCESSING — IMMEDIATE
// ═══════════════════════════════════════════════════════════

// MOVZ Xd, #imm16, LSL #shift  (shift = 0, 16, 32, 48)
// Encoding: 1 10 100101 hw imm16 Rd
pub fn emit_movz(code, rd, imm16, shift) {
  let hw = shift / 16;
  let instr = 0xD2800000 + (hw * 0x200000) + ((imm16 % 65536) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// MOVK Xd, #imm16, LSL #shift  (keep other bits)
// Encoding: 1 11 100101 hw imm16 Rd
pub fn emit_movk(code, rd, imm16, shift) {
  let hw = shift / 16;
  let instr = 0xF2800000 + (hw * 0x200000) + ((imm16 % 65536) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// MOVZ Wd, #imm16 (32-bit variant)
// Encoding: 0 10 100101 hw imm16 Rd
pub fn emit_movz_w(code, rd, imm16, shift) {
  let hw = shift / 16;
  let instr = 0x52800000 + (hw * 0x200000) + ((imm16 % 65536) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// Load 64-bit immediate into Xd using MOVZ + up to 3 MOVK
pub fn emit_mov_imm64(code, rd, val) {
  let v0 = val % 65536;
  let v1 = (val / 65536) % 65536;
  let v2 = (val / 4294967296) % 65536;
  let v3 = (val / 281474976710656) % 65536;

  emit_movz(code, rd, v0, 0);
  if v1 != 0 { emit_movk(code, rd, v1, 16); }
  if v2 != 0 { emit_movk(code, rd, v2, 32); }
  if v3 != 0 { emit_movk(code, rd, v3, 48); }
}

// MOV Xd, Xn  (alias for ORR Xd, XZR, Xn)
pub fn emit_mov_reg(code, rd, rn) {
  // ORR Xd, XZR, Xn: 1 01 01010 00 0 Xn 000000 11111 Xd
  let instr = 0xAA0003E0 + ((rn % 32) * 65536) + (rd % 32);
  emit_instr(code, instr);
}

// MOV Wd, Wn (32-bit)
pub fn emit_mov_reg_w(code, rd, rn) {
  let instr = 0x2A0003E0 + ((rn % 32) * 65536) + (rd % 32);
  emit_instr(code, instr);
}

// ═══════════════════════════════════════════════════════════
// ARITHMETIC
// ═══════════════════════════════════════════════════════════

// ADD Xd, Xn, Xm
pub fn emit_add_reg(code, rd, rn, rm) {
  let instr = 0x8B000000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// SUB Xd, Xn, Xm
pub fn emit_sub_reg(code, rd, rn, rm) {
  let instr = 0xCB000000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// ADD Xd, Xn, #imm12
pub fn emit_add_imm(code, rd, rn, imm12) {
  let instr = 0x91000000 + ((imm12 % 4096) * 1024) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// SUB Xd, Xn, #imm12
pub fn emit_sub_imm(code, rd, rn, imm12) {
  let instr = 0xD1000000 + ((imm12 % 4096) * 1024) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// ADD Wd, Wn, Wm (32-bit)
pub fn emit_add_reg_w(code, rd, rn, rm) {
  let instr = 0x0B000000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// SUB Wd, Wn, Wm (32-bit)
pub fn emit_sub_reg_w(code, rd, rn, rm) {
  let instr = 0x4B000000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// ADD Wd, Wn, #imm12 (32-bit)
pub fn emit_add_imm_w(code, rd, rn, imm12) {
  let instr = 0x11000000 + ((imm12 % 4096) * 1024) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// SUBS Xd, Xn, Xm  (sets flags)
pub fn emit_subs_reg(code, rd, rn, rm) {
  let instr = 0xEB000000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// CMP Xn, Xm  (alias for SUBS XZR, Xn, Xm)
pub fn emit_cmp_reg(code, rn, rm) {
  emit_subs_reg(code, XZR, rn, rm);
}

// SUBS Xd, Xn, #imm12  (sets flags)
pub fn emit_subs_imm(code, rd, rn, imm12) {
  let instr = 0xF1000000 + ((imm12 % 4096) * 1024) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// CMP Xn, #imm12
pub fn emit_cmp_imm(code, rn, imm12) {
  emit_subs_imm(code, XZR, rn, imm12);
}

// SUBS Wd, Wn, #imm12 (32-bit, sets flags)
pub fn emit_subs_imm_w(code, rd, rn, imm12) {
  let instr = 0x71000000 + ((imm12 % 4096) * 1024) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// CMP Wn, #imm12 (32-bit)
pub fn emit_cmp_imm_w(code, rn, imm12) {
  emit_subs_imm_w(code, XZR, rn, imm12);
}

// MUL Xd, Xn, Xm  (alias for MADD Xd, Xn, Xm, XZR)
pub fn emit_mul(code, rd, rn, rm) {
  let instr = 0x9B007C00 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// MUL Wd, Wn, Wm (32-bit)
pub fn emit_mul_w(code, rd, rn, rm) {
  let instr = 0x1B007C00 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// UDIV Xd, Xn, Xm
pub fn emit_udiv(code, rd, rn, rm) {
  let instr = 0x9AC00800 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// SDIV Xd, Xn, Xm
pub fn emit_sdiv(code, rd, rn, rm) {
  let instr = 0x9AC00C00 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// LSL Xd, Xn, Xm  (alias for LSLV)
pub fn emit_lsl(code, rd, rn, rm) {
  let instr = 0x9AC02000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// LSR Xd, Xn, Xm  (alias for LSRV)
pub fn emit_lsr(code, rd, rn, rm) {
  let instr = 0x9AC02400 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// ═══════════════════════════════════════════════════════════
// LOGIC
// ═══════════════════════════════════════════════════════════

// AND Xd, Xn, Xm
pub fn emit_and_reg(code, rd, rn, rm) {
  let instr = 0x8A000000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// ORR Xd, Xn, Xm
pub fn emit_orr_reg(code, rd, rn, rm) {
  let instr = 0xAA000000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// EOR Xd, Xn, Xm
pub fn emit_eor_reg(code, rd, rn, rm) {
  let instr = 0xCA000000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rd % 32);
  emit_instr(code, instr);
}

// MVN Xd, Xm (NOT — alias for ORN Xd, XZR, Xm)
pub fn emit_mvn(code, rd, rm) {
  let instr = 0xAA200000 + ((rm % 32) * 65536) + (0x1F * 32) + (rd % 32);
  emit_instr(code, instr);
}

// TST Xn, Xm (alias for ANDS XZR, Xn, Xm)
pub fn emit_tst_reg(code, rn, rm) {
  let instr = 0xEA000000 + ((rm % 32) * 65536) + ((rn % 32) * 32) + XZR;
  emit_instr(code, instr);
}

// ═══════════════════════════════════════════════════════════
// MEMORY — Load/Store
// ═══════════════════════════════════════════════════════════

// LDR Xt, [Xn, #offset]  (offset must be 8-byte aligned, /8 encoded)
// Encoding: 11 111 00101 imm12 Rn Rt
pub fn emit_ldr(code, rt, rn, offset) {
  let imm12 = (offset / 8) % 4096;
  let instr = 0xF9400000 + (imm12 * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// STR Xt, [Xn, #offset]
pub fn emit_str(code, rt, rn, offset) {
  let imm12 = (offset / 8) % 4096;
  let instr = 0xF9000000 + (imm12 * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// LDR Wt, [Xn, #offset]  (32-bit, offset /4)
pub fn emit_ldr_w(code, rt, rn, offset) {
  let imm12 = (offset / 4) % 4096;
  let instr = 0xB9400000 + (imm12 * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// STR Wt, [Xn, #offset] (32-bit, offset /4)
pub fn emit_str_w(code, rt, rn, offset) {
  let imm12 = (offset / 4) % 4096;
  let instr = 0xB9000000 + (imm12 * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// LDRB Wt, [Xn, #offset]  (byte load, zero-extend)
pub fn emit_ldrb(code, rt, rn, offset) {
  let imm12 = offset % 4096;
  let instr = 0x39400000 + (imm12 * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// STRB Wt, [Xn, #offset]  (byte store)
pub fn emit_strb(code, rt, rn, offset) {
  let imm12 = offset % 4096;
  let instr = 0x39000000 + (imm12 * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// LDRH Wt, [Xn, #offset]  (halfword load, offset /2)
pub fn emit_ldrh(code, rt, rn, offset) {
  let imm12 = (offset / 2) % 4096;
  let instr = 0x79400000 + (imm12 * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// STRH Wt, [Xn, #offset] (halfword store, offset /2)
pub fn emit_strh(code, rt, rn, offset) {
  let imm12 = (offset / 2) % 4096;
  let instr = 0x79000000 + (imm12 * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// STP Xt, Xt2, [Xn, #offset]!  (pre-index store pair)
// Encoding: 10 101 00110 imm7 Rt2 Rn Rt
pub fn emit_stp_pre(code, rt, rt2, rn, offset) {
  let imm7 = ((offset / 8) + 128) % 128;  // signed 7-bit
  if offset < 0 { imm7 = ((offset / 8) + 128) % 128; }
  let instr = 0xA9800000 + ((imm7 % 128) * 32768) + ((rt2 % 32) * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// LDP Xt, Xt2, [Xn], #offset  (post-index load pair)
// Encoding: 10 101 00011 imm7 Rt2 Rn Rt
pub fn emit_ldp_post(code, rt, rt2, rn, offset) {
  let imm7 = ((offset / 8) + 128) % 128;
  let instr = 0xA8C00000 + ((imm7 % 128) * 32768) + ((rt2 % 32) * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// STP Xt, Xt2, [Xn, #offset]  (signed offset, no writeback)
pub fn emit_stp(code, rt, rt2, rn, offset) {
  let imm7 = ((offset / 8) + 128) % 128;
  let instr = 0xA9000000 + ((imm7 % 128) * 32768) + ((rt2 % 32) * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// LDP Xt, Xt2, [Xn, #offset]  (signed offset, no writeback)
pub fn emit_ldp(code, rt, rt2, rn, offset) {
  let imm7 = ((offset / 8) + 128) % 128;
  let instr = 0xA9400000 + ((imm7 % 128) * 32768) + ((rt2 % 32) * 1024) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// LDR Xt, [Xn, Xm]  (register offset)
pub fn emit_ldr_reg(code, rt, rn, rm) {
  // 11 111 00011 1 Rm 011 0 10 Rn Rt  (LSL #3)
  let instr = 0xF8606800 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// LDRB Wt, [Xn, Xm]  (register offset, byte)
pub fn emit_ldrb_reg(code, rt, rn, rm) {
  // 00 111 00011 1 Rm 011 0 10 Rn Rt
  let instr = 0x38606800 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// STRB Wt, [Xn, Xm] (register offset, byte)
pub fn emit_strb_reg(code, rt, rn, rm) {
  let instr = 0x38206800 + ((rm % 32) * 65536) + ((rn % 32) * 32) + (rt % 32);
  emit_instr(code, instr);
}

// ═══════════════════════════════════════════════════════════
// BRANCH
// ═══════════════════════════════════════════════════════════

// B #offset  (26-bit signed, PC-relative, offset in bytes, must be 4-aligned)
pub fn emit_b(code, offset) {
  let imm26 = ((offset / 4) + 67108864) % 67108864;  // signed 26-bit
  let instr = 0x14000000 + imm26;
  emit_instr(code, instr);
}

// BL #offset  (branch with link = function call)
pub fn emit_bl(code, offset) {
  let imm26 = ((offset / 4) + 67108864) % 67108864;
  let instr = 0x94000000 + imm26;
  emit_instr(code, instr);
}

// B.cond #offset  (19-bit signed, PC-relative)
pub fn emit_b_cond(code, cond, offset) {
  let imm19 = ((offset / 4) + 524288) % 524288;  // signed 19-bit
  let instr = 0x54000000 + (imm19 * 32) + (cond % 16);
  emit_instr(code, instr);
}

// CBZ Xt, #offset  (compare and branch if zero)
pub fn emit_cbz(code, rt, offset) {
  let imm19 = ((offset / 4) + 524288) % 524288;
  let instr = 0xB4000000 + (imm19 * 32) + (rt % 32);
  emit_instr(code, instr);
}

// CBNZ Xt, #offset  (compare and branch if not zero)
pub fn emit_cbnz(code, rt, offset) {
  let imm19 = ((offset / 4) + 524288) % 524288;
  let instr = 0xB5000000 + (imm19 * 32) + (rt % 32);
  emit_instr(code, instr);
}

// CBZ Wt, #offset (32-bit)
pub fn emit_cbz_w(code, rt, offset) {
  let imm19 = ((offset / 4) + 524288) % 524288;
  let instr = 0x34000000 + (imm19 * 32) + (rt % 32);
  emit_instr(code, instr);
}

// BR Xn  (indirect branch)
pub fn emit_br(code, rn) {
  let instr = 0xD61F0000 + ((rn % 32) * 32);
  emit_instr(code, instr);
}

// BLR Xn  (indirect call)
pub fn emit_blr(code, rn) {
  let instr = 0xD63F0000 + ((rn % 32) * 32);
  emit_instr(code, instr);
}

// RET {Xn}  (default X30)
pub fn emit_ret(code) {
  emit_instr(code, 0xD65F03C0);  // RET X30
}

pub fn emit_ret_reg(code, rn) {
  let instr = 0xD65F0000 + ((rn % 32) * 32);
  emit_instr(code, instr);
}

// ═══════════════════════════════════════════════════════════
// SYSTEM
// ═══════════════════════════════════════════════════════════

// SVC #imm16  (supervisor call / syscall)
pub fn emit_svc(code, imm16) {
  let instr = 0xD4000001 + ((imm16 % 65536) * 32);
  emit_instr(code, instr);
}

// NOP
pub fn emit_nop(code) {
  emit_instr(code, 0xD503201F);
}

// BRK #imm16  (breakpoint)
pub fn emit_brk(code, imm16) {
  let instr = 0xD4200000 + ((imm16 % 65536) * 32);
  emit_instr(code, instr);
}

// ═══════════════════════════════════════════════════════════
// ADDRESS GENERATION
// ═══════════════════════════════════════════════════════════

// ADR Xd, #offset  (PC-relative, 21-bit signed)
pub fn emit_adr(code, rd, offset) {
  let immlo = ((offset + 4) % 4);
  let immhi = (offset / 4) % 524288;
  let instr = 0x10000000 + (immlo * 536870912) + (immhi * 32) + (rd % 32);
  emit_instr(code, instr);
}

// ADRP Xd, #offset  (PC-relative page, 21-bit signed, 4KB granule)
pub fn emit_adrp(code, rd, offset) {
  let imm = (offset / 4096) % 2097152;
  let immlo = imm % 4;
  let immhi = imm / 4;
  let instr = 0x90000000 + (immlo * 536870912) + (immhi * 32) + (rd % 32);
  emit_instr(code, instr);
}

// ═══════════════════════════════════════════════════════════
// NEON/FP — f64 (double precision)
// ═══════════════════════════════════════════════════════════

// FMOV Dd, Dn
pub fn emit_fmov_d(code, dd, dn) {
  let instr = 0x1E604000 + ((dn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// FADD Dd, Dn, Dm
pub fn emit_fadd_d(code, dd, dn, dm) {
  let instr = 0x1E602800 + ((dm % 32) * 65536) + ((dn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// FSUB Dd, Dn, Dm
pub fn emit_fsub_d(code, dd, dn, dm) {
  let instr = 0x1E603800 + ((dm % 32) * 65536) + ((dn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// FMUL Dd, Dn, Dm
pub fn emit_fmul_d(code, dd, dn, dm) {
  let instr = 0x1E600800 + ((dm % 32) * 65536) + ((dn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// FDIV Dd, Dn, Dm
pub fn emit_fdiv_d(code, dd, dn, dm) {
  let instr = 0x1E601800 + ((dm % 32) * 65536) + ((dn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// FCMP Dn, Dm  (sets NZCV flags)
pub fn emit_fcmp_d(code, dn, dm) {
  let instr = 0x1E602000 + ((dm % 32) * 65536) + ((dn % 32) * 32);
  emit_instr(code, instr);
}

// FCMP Dn, #0.0
pub fn emit_fcmp_zero_d(code, dn) {
  let instr = 0x1E602008 + ((dn % 32) * 32);
  emit_instr(code, instr);
}

// SCVTF Dd, Xn  (signed int → f64)
pub fn emit_scvtf_d(code, dd, xn) {
  let instr = 0x9E620000 + ((xn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// FCVTZS Xd, Dn  (f64 → signed int, truncate)
pub fn emit_fcvtzs_d(code, xd, dn) {
  let instr = 0x9E780000 + ((dn % 32) * 32) + (xd % 32);
  emit_instr(code, instr);
}

// FMOV Xd, Dn  (copy bits, no conversion)
pub fn emit_fmov_to_gpr(code, xd, dn) {
  let instr = 0x9E660000 + ((dn % 32) * 32) + (xd % 32);
  emit_instr(code, instr);
}

// FMOV Dd, Xn  (copy bits, no conversion)
pub fn emit_fmov_from_gpr(code, dd, xn) {
  let instr = 0x9E670000 + ((xn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// LDR Dt, [Xn, #offset]  (FP load, offset /8)
pub fn emit_ldr_d(code, dt, xn, offset) {
  let imm12 = (offset / 8) % 4096;
  let instr = 0xFD400000 + (imm12 * 1024) + ((xn % 32) * 32) + (dt % 32);
  emit_instr(code, instr);
}

// STR Dt, [Xn, #offset]  (FP store, offset /8)
pub fn emit_str_d(code, dt, xn, offset) {
  let imm12 = (offset / 8) % 4096;
  let instr = 0xFD000000 + (imm12 * 1024) + ((xn % 32) * 32) + (dt % 32);
  emit_instr(code, instr);
}

// FNEG Dd, Dn
pub fn emit_fneg_d(code, dd, dn) {
  let instr = 0x1E614000 + ((dn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// FABS Dd, Dn
pub fn emit_fabs_d(code, dd, dn) {
  let instr = 0x1E60C000 + ((dn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// FSQRT Dd, Dn
pub fn emit_fsqrt_d(code, dd, dn) {
  let instr = 0x1E61C000 + ((dn % 32) * 32) + (dd % 32);
  emit_instr(code, instr);
}

// ═══════════════════════════════════════════════════════════
// LABELS & FIXUPS
// ═══════════════════════════════════════════════════════════

pub fn label(code, name) {
  code.labels[name] = code_len(code);
}

// B label (unconditional branch to label, fixup later)
pub fn emit_b_label(code, name) {
  push(code.fixups, { pos: code_len(code), label: name, kind: "b" });
  emit_instr(code, 0x14000000);  // placeholder
}

// BL label
pub fn emit_bl_label(code, name) {
  push(code.fixups, { pos: code_len(code), label: name, kind: "bl" });
  emit_instr(code, 0x94000000);
}

// B.cond label
pub fn emit_b_cond_label(code, cond, name) {
  push(code.fixups, { pos: code_len(code), label: name, kind: "bcond", cond: cond });
  emit_instr(code, 0x54000000 + (cond % 16));
}

// CBZ Xt, label
pub fn emit_cbz_label(code, rt, name) {
  push(code.fixups, { pos: code_len(code), label: name, kind: "cbz", rt: rt });
  emit_instr(code, 0xB4000000 + (rt % 32));
}

// CBNZ Xt, label
pub fn emit_cbnz_label(code, rt, name) {
  push(code.fixups, { pos: code_len(code), label: name, kind: "cbnz", rt: rt });
  emit_instr(code, 0xB5000000 + (rt % 32));
}

pub fn resolve_fixups(code) {
  let i = 0;
  while i < len(code.fixups) {
    let f = code.fixups[i];
    let target = code.labels[f.label];
    if target != 0 || f.label == code.labels[f.label] {
      let rel = target - f.pos;  // byte offset from instruction
      let rel4 = rel / 4;        // instruction offset

      if f.kind == "b" || f.kind == "bl" {
        // 26-bit immediate
        let imm26 = (rel4 + 67108864) % 67108864;
        let base = 0x14000000;
        if f.kind == "bl" { base = 0x94000000; }
        patch_instr(code, f.pos, base + imm26);
      }
      if f.kind == "bcond" {
        // 19-bit immediate
        let imm19 = (rel4 + 524288) % 524288;
        patch_instr(code, f.pos, 0x54000000 + (imm19 * 32) + (f.cond % 16));
      }
      if f.kind == "cbz" {
        let imm19 = (rel4 + 524288) % 524288;
        patch_instr(code, f.pos, 0xB4000000 + (imm19 * 32) + (f.rt % 32));
      }
      if f.kind == "cbnz" {
        let imm19 = (rel4 + 524288) % 524288;
        patch_instr(code, f.pos, 0xB5000000 + (imm19 * 32) + (f.rt % 32));
      }
    }
    i = i + 1;
  }
}

fn patch_instr(code, pos, instr) {
  code.bytes[pos] = instr % 256;
  code.bytes[pos + 1] = (instr / 256) % 256;
  code.bytes[pos + 2] = (instr / 65536) % 256;
  code.bytes[pos + 3] = (instr / 16777216) % 256;
}

// ═══════════════════════════════════════════════════════════
// CONVENIENCE
// ═══════════════════════════════════════════════════════════

// Linux ARM64 syscall: args in X0-X5, number in X8, SVC #0
pub fn emit_syscall(code) {
  emit_svc(code, 0);
}

pub fn emit_exit_0(code) {
  emit_movz(code, X0, 0, 0);     // X0 = 0 (status)
  emit_movz(code, X8, 93, 0);    // X8 = 93 (SYS_EXIT)
  emit_svc(code, 0);
}

pub fn emit_exit_1(code) {
  emit_movz(code, X0, 1, 0);
  emit_movz(code, X8, 93, 0);
  emit_svc(code, 0);
}

// Write string: X0=fd, X1=ptr, X2=len → SVC
pub fn emit_write(code, fd) {
  // Assumes X1=ptr, X2=len already set
  emit_movz(code, X0, fd, 0);
  emit_movz(code, X8, 64, 0);    // SYS_WRITE = 64 on ARM64
  emit_svc(code, 0);
}

// Save callee-saved pair to stack
pub fn emit_push_pair(code, r1, r2) {
  emit_stp_pre(code, r1, r2, SP, -16);
}

// Restore callee-saved pair from stack
pub fn emit_pop_pair(code, r1, r2) {
  emit_ldp_post(code, r1, r2, SP, 16);
}
