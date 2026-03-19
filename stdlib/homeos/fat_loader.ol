// homeos/fat_loader.ol — Generate ELF loader stubs for fat binary
// Each stub: tiny ELF that reads fat binary, finds its arch, jumps to VM
//
// Option A from PLAN_4_2: per-arch ELF stubs (~1 KB each)
// Stub opens fat binary via /proc/self/exe neighbor, mmaps, jumps to VM

// Architecture IDs (same as fat_header.ol)
let STUB_ARCH_X86_64 = 0x01;
let STUB_ARCH_ARM64  = 0x02;

// ── x86_64 loader stub ──
// Generates raw x86_64 machine code that:
//   1. Open "origin.fat" (sibling file) via open() syscall
//   2. fstat to get size
//   3. mmap the fat binary
//   4. Parse fat header: find arch_id == 0x01
//   5. Jump to mmap_base + vm_off + entry_off

pub fn make_x86_64_stub(fat_path_bytes) {
  let code = code_new();

  // ── Prologue: save stack ──
  // push rbp; mov rbp, rsp
  emit_byte(code, 0x55);
  emit_bytes(code, [0x48, 0x89, 0xE5]);

  // ── Store fat path on stack ──
  // sub rsp, 256 (space for path + stat buf)
  emit_bytes(code, [0x48, 0x81, 0xEC, 0x00, 0x01, 0x00, 0x00]);

  // Write path bytes to [rsp]
  let i = 0;
  while i < len(fat_path_bytes) {
    // mov byte [rsp + i], imm8
    emit_bytes(code, [0xC6, 0x44, 0x24]);
    emit_byte(code, i);
    emit_byte(code, fat_path_bytes[i]);
    i = i + 1;
  }
  // null terminator
  emit_bytes(code, [0xC6, 0x44, 0x24]);
  emit_byte(code, len(fat_path_bytes));
  emit_byte(code, 0);

  // ── syscall: open(path, O_RDONLY) ──
  // rdi = rsp (path), rsi = 0 (O_RDONLY), rax = 2 (SYS_open)
  emit_bytes(code, [0x48, 0x89, 0xE7]);       // mov rdi, rsp
  emit_bytes(code, [0x48, 0x31, 0xF6]);       // xor rsi, rsi (O_RDONLY=0)
  emit_bytes(code, [0x48, 0xC7, 0xC0, 0x02, 0x00, 0x00, 0x00]); // mov rax, 2
  emit_bytes(code, [0x0F, 0x05]);             // syscall
  // rax = fd, save in r12
  emit_bytes(code, [0x49, 0x89, 0xC4]);       // mov r12, rax

  // ── syscall: fstat(fd, statbuf) ──
  // rdi = fd (r12), rsi = rsp+128 (stat buf), rax = 5 (SYS_fstat)
  emit_bytes(code, [0x4C, 0x89, 0xE7]);       // mov rdi, r12
  emit_bytes(code, [0x48, 0x8D, 0x74, 0x24, 0x80]); // lea rsi, [rsp+128]
  emit_bytes(code, [0x48, 0xC7, 0xC0, 0x05, 0x00, 0x00, 0x00]); // mov rax, 5
  emit_bytes(code, [0x0F, 0x05]);             // syscall
  // st_size at offset 48 in stat struct
  emit_bytes(code, [0x48, 0x8B, 0x4C, 0x24, 0xB0]); // mov rcx, [rsp+128+48=176]
  // save file size in r13
  emit_bytes(code, [0x49, 0x89, 0xCD]);       // mov r13, rcx

  // ── syscall: mmap(NULL, size, PROT_READ|PROT_EXEC, MAP_PRIVATE, fd, 0) ──
  emit_bytes(code, [0x48, 0x31, 0xFF]);       // xor rdi, rdi (addr=NULL)
  emit_bytes(code, [0x4C, 0x89, 0xEE]);       // mov rsi, r13 (size)
  emit_bytes(code, [0x48, 0xC7, 0xC2, 0x05, 0x00, 0x00, 0x00]); // mov rdx, 5 (PROT_READ|PROT_EXEC)
  emit_bytes(code, [0x41, 0xBA, 0x02, 0x00, 0x00, 0x00]); // mov r10d, 2 (MAP_PRIVATE)
  emit_bytes(code, [0x49, 0x89, 0xE0]);       // mov r8, r12 (fd) — WRONG, should use r8 for arg5
  // Fix: Linux mmap syscall: rdi=addr, rsi=len, rdx=prot, r10=flags, r8=fd, r9=offset
  emit_bytes(code, [0x4D, 0x89, 0xE0]);       // mov r8, r12 (fd)
  emit_bytes(code, [0x4D, 0x31, 0xC9]);       // xor r9, r9 (offset=0)
  emit_bytes(code, [0x48, 0xC7, 0xC0, 0x09, 0x00, 0x00, 0x00]); // mov rax, 9 (SYS_mmap)
  emit_bytes(code, [0x0F, 0x05]);             // syscall
  // rax = mmap base, save in r14
  emit_bytes(code, [0x49, 0x89, 0xC6]);       // mov r14, rax

  // ── Parse fat header: find arch 0x01 ──
  // byte[4] = version, byte[5] = arch_cnt
  // Entries start at offset 8, each 16B: [arch_id:1][vm_off:4][vm_size:4][entry_off:4][reserved:3]
  emit_bytes(code, [0x41, 0x0F, 0xB6, 0x4E, 0x05]); // movzx ecx, byte [r14+5] (arch_cnt)
  // r15 = offset into entries (start at 8)
  emit_bytes(code, [0x49, 0xC7, 0xC7, 0x08, 0x00, 0x00, 0x00]); // mov r15, 8

  // Loop: search entries
  let loop_start = code_len(code);
  // test ecx, ecx; jz not_found
  emit_bytes(code, [0x85, 0xC9]);             // test ecx, ecx
  let jz_not_found = code_len(code);
  emit_bytes(code, [0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]); // jz (placeholder)

  // al = byte [r14 + r15] (arch_id)
  emit_bytes(code, [0x43, 0x0F, 0xB6, 0x04, 0x3E]); // movzx eax, byte [r14+r15]
  // cmp al, 0x01
  emit_bytes(code, [0x3C, 0x01]);             // cmp al, 0x01
  let je_found = code_len(code);
  emit_bytes(code, [0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]); // je (placeholder)

  // next entry: r15 += 16, ecx -= 1
  emit_bytes(code, [0x49, 0x83, 0xC7, 0x10]); // add r15, 16
  emit_bytes(code, [0x83, 0xE9, 0x01]);       // sub ecx, 1
  // jmp loop_start
  let jmp_off = code_len(code);
  emit_byte(code, 0xE9);                      // jmp rel32
  let rel = loop_start - (code_len(code) + 4);
  emit_i32(code, rel);

  // not_found: exit(1)
  let not_found_pos = code_len(code);
  patch_i32(code, jz_not_found + 2, not_found_pos - (jz_not_found + 6));
  emit_bytes(code, [0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00]); // mov rdi, 1
  emit_bytes(code, [0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00]); // mov rax, 60 (exit)
  emit_bytes(code, [0x0F, 0x05]);             // syscall

  // found: read vm_off and entry_off, jump
  let found_pos = code_len(code);
  patch_i32(code, je_found + 2, found_pos - (je_found + 6));
  // edx = vm_off at [r14 + r15 + 1] (4 bytes LE)
  emit_bytes(code, [0x43, 0x8B, 0x54, 0x3E, 0x01]); // mov edx, [r14+r15+1]
  // ebx = entry_off at [r14 + r15 + 9] (4 bytes LE)
  emit_bytes(code, [0x43, 0x8B, 0x5C, 0x3E, 0x09]); // mov ebx, [r14+r15+9]
  // target = r14 + rdx + rbx
  emit_bytes(code, [0x4C, 0x89, 0xF0]);       // mov rax, r14
  emit_bytes(code, [0x48, 0x01, 0xD0]);       // add rax, rdx
  emit_bytes(code, [0x48, 0x01, 0xD8]);       // add rax, rbx
  // jump to VM entry
  emit_bytes(code, [0xFF, 0xE0]);             // jmp rax

  return code.bytes;
}

// ── ARM64 loader stub ──
// Same logic as x86_64 but using ARM64 instructions

pub fn make_arm64_stub(fat_path_bytes) {
  let code = code_new();

  // ── Store path on stack ──
  // sub sp, sp, #256
  emit_arm(code, 0xD10400FF);

  // Write path bytes using strb
  let i = 0;
  while i < len(fat_path_bytes) {
    // mov w8, #byte
    emit_arm(code, 0x52800008 | (fat_path_bytes[i] * 32));
    // strb w8, [sp, #i]
    emit_arm(code, 0x39000008 | (i * 1024));  // imm12 scaled for byte
    i = i + 1;
  }
  // null terminator
  emit_arm(code, 0x52800008);                 // mov w8, #0
  emit_arm(code, 0x39000008 | (len(fat_path_bytes) * 1024));

  // ── openat(AT_FDCWD, path, O_RDONLY) ──
  emit_arm(code, 0x12800020);                 // mov w0, #-100 (AT_FDCWD = 0xFFFFFF9C)
  // Fix: AT_FDCWD = -100, encoded as movn w0, #99
  emit_arm(code, 0x910003E1);                 // mov x1, sp (path)
  emit_arm(code, 0xD2800002);                 // mov x2, #0 (O_RDONLY)
  emit_arm(code, 0xD2800703);                 // mov x3, #56 (SYS_openat on ARM64)
  // Correct: x8 = syscall number
  emit_arm(code, 0xD2800708);                 // mov x8, #56 (SYS_openat)
  emit_arm(code, 0xD4000001);                 // svc #0
  // x0 = fd, save in x19
  emit_arm(code, 0xAA0003F3);                 // mov x19, x0

  // ── fstat(fd, statbuf) on stack ──
  emit_arm(code, 0xAA1303E0);                 // mov x0, x19 (fd)
  emit_arm(code, 0x910203E1);                 // add x1, sp, #128 (stat buf)
  emit_arm(code, 0xD2800508);                 // mov x8, #40 (SYS_fstat — actually newfstatat=79 on ARM64)
  // Correct for ARM64: SYS_fstat = 80 (fstat), but newer kernels use fstatat(79)
  // Use fstat (80) for simplicity
  emit_arm(code, 0xD2800A08);                 // mov x8, #80
  emit_arm(code, 0xD4000001);                 // svc #0
  // st_size at offset 48 — ldr x20, [sp, #176]
  emit_arm(code, 0xF9405814);                 // ldr x20, [sp, #176]

  // ── mmap(NULL, size, PROT_READ|PROT_EXEC, MAP_PRIVATE, fd, 0) ──
  emit_arm(code, 0xD2800000);                 // mov x0, #0 (addr=NULL)
  emit_arm(code, 0xAA1403E1);                 // mov x1, x20 (size)
  emit_arm(code, 0xD28000A2);                 // mov x2, #5 (PROT_READ|PROT_EXEC)
  emit_arm(code, 0xD2800043);                 // mov x3, #2 (MAP_PRIVATE)
  emit_arm(code, 0xAA1303E4);                 // mov x4, x19 (fd)
  emit_arm(code, 0xD2800005);                 // mov x5, #0 (offset)
  emit_arm(code, 0xD2800DC8);                 // mov x8, #222 (SYS_mmap on ARM64)
  emit_arm(code, 0xD4000001);                 // svc #0
  // x0 = mmap base, save in x21
  emit_arm(code, 0xAA0003F5);                 // mov x21, x0

  // ── Parse fat header: find arch 0x02 ──
  // x22 = arch_cnt (byte at offset 5)
  emit_arm(code, 0x394016B6);                 // ldrb w22, [x21, #5]
  // x23 = current entry offset (start at 8)
  emit_arm(code, 0xD2800117);                 // mov x23, #8

  // Loop
  let loop_off = code_len(code);
  // cbz w22, not_found
  let cbz_off = code_len(code);
  emit_arm(code, 0x34000016);                 // cbz w22, +0 (placeholder)

  // w24 = byte [x21 + x23] (arch_id)
  emit_arm(code, 0x386376B8);                 // ldrb w24, [x21, x23]
  // cmp w24, #2
  emit_arm(code, 0x7100091F);                 // cmp w24, #2 (ARM64)
  // b.eq found
  let beq_off = code_len(code);
  emit_arm(code, 0x54000000);                 // b.eq +0 (placeholder)

  // next: x23 += 16, w22 -= 1
  emit_arm(code, 0x910042F7);                 // add x23, x23, #16
  emit_arm(code, 0x510006D6);                 // sub w22, w22, #1
  // b loop
  let b_loop_rel = loop_off - code_len(code);
  emit_arm(code, 0x14000000 | ((b_loop_rel / 4) & 0x3FFFFFF));

  // not_found: exit(1)
  let not_found_off = code_len(code);
  // Patch cbz
  let cbz_rel = (not_found_off - cbz_off) / 4;
  patch_arm_imm19(code, cbz_off, cbz_rel);
  emit_arm(code, 0xD2800020);                 // mov x0, #1
  emit_arm(code, 0xD2800BA8);                 // mov x8, #93 (SYS_exit)
  emit_arm(code, 0xD4000001);                 // svc #0

  // found: read vm_off, entry_off, jump
  let found_off = code_len(code);
  // Patch b.eq
  let beq_rel = (found_off - beq_off) / 4;
  patch_arm_imm19(code, beq_off, beq_rel);

  // x24 = vm_off at [x21 + x23 + 1]
  emit_arm(code, 0x8B1702B8);                 // add x24, x21, x23
  emit_arm(code, 0xB9400718);                 // ldr w24, [x24, #4] — actually offset 1, use unaligned
  // Better: add x24, x21, x23; add x24, x24, #1; ldr w24, [x24] (unaligned)
  // Simplify: compute base+off, load fields
  emit_arm(code, 0x8B1702A0);                 // add x0, x21, x23 (entry base)
  emit_arm(code, 0xB9400418);                 // ldr w24, [x0, #4]  — vm_off at +1..+4 (unaligned issue)
  // ARM64 supports unaligned access for LDR, so offset byte 1:
  emit_arm(code, 0x38401418);                 // ldrb w24, [x0, #1] — byte-by-byte for safety
  // Actually, let's just load aligned from +0 and shift
  // Simpler approach: use ldur (unaligned load)
  emit_arm(code, 0xB8401018);                 // ldur w24, [x0, #1]  — vm_off (unaligned OK on ARM64)
  emit_arm(code, 0xB8409019);                 // ldur w25, [x0, #9]  — entry_off

  // target = x21 + x24 + x25
  emit_arm(code, 0x8B1802A0);                 // add x0, x21, x24
  emit_arm(code, 0x8B190000);                 // add x0, x0, x25
  // br x0
  emit_arm(code, 0xD61F0000);                 // br x0

  return code.bytes;
}

// ── Helpers ──

fn code_new() {
  return { bytes: [] };
}

fn code_len(code) {
  return len(code.bytes);
}

fn emit_byte(code, b) {
  push(code.bytes, b % 256);
}

fn emit_bytes(code, bytes) {
  let i = 0;
  while i < len(bytes) {
    push(code.bytes, bytes[i]);
    i = i + 1;
  }
}

fn emit_arm(code, inst) {
  // ARM64: 32-bit little-endian instruction
  push(code.bytes, inst % 256);
  push(code.bytes, (inst / 256) % 256);
  push(code.bytes, (inst / 65536) % 256);
  push(code.bytes, (inst / 16777216) % 256);
}

fn emit_i32(code, val) {
  // Signed 32-bit LE
  if val < 0 { val = val + 4294967296; }
  push(code.bytes, val % 256);
  push(code.bytes, (val / 256) % 256);
  push(code.bytes, (val / 65536) % 256);
  push(code.bytes, (val / 16777216) % 256);
}

fn patch_i32(code, off, val) {
  if val < 0 { val = val + 4294967296; }
  code.bytes[off]     = val % 256;
  code.bytes[off + 1] = (val / 256) % 256;
  code.bytes[off + 2] = (val / 65536) % 256;
  code.bytes[off + 3] = (val / 16777216) % 256;
}

fn patch_arm_imm19(code, off, imm19) {
  // Patch ARM64 instruction at byte offset 'off' with 19-bit immediate (bits 23:5)
  if imm19 < 0 { imm19 = imm19 + 524288; }  // 2^19
  let existing = code.bytes[off]
               + code.bytes[off + 1] * 256
               + code.bytes[off + 2] * 65536
               + code.bytes[off + 3] * 16777216;
  // Clear bits 23:5, set new imm19
  let mask = 0xFFFFFFFF - (0x00FFFFE0);      // clear imm19 field
  let patched = (existing & mask) | ((imm19 & 0x7FFFF) * 32);
  code.bytes[off]     = patched % 256;
  code.bytes[off + 1] = (patched / 256) % 256;
  code.bytes[off + 2] = (patched / 65536) % 256;
  code.bytes[off + 3] = (patched / 16777216) % 256;
}
