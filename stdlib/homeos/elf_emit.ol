// homeos/elf_emit.ol — Generate ELF64 executable binary
// Minimal: ELF header (64B) + program header (56B) = 120B
// Supports: x86_64 (e_machine=0x3E) and ARM64 (e_machine=0xB7)

let LOAD_ADDR = 0x400000;

// Architecture constants
let ARCH_X86_64 = "x86_64";
let ARCH_ARM64  = "arm64";

pub fn make_elf(code_bytes, entry_offset) {
  return make_elf_arch(code_bytes, entry_offset, ARCH_X86_64);
}

pub fn make_elf_arch(code_bytes, entry_offset, arch) {
  let buf = [];
  let total_size = 120 + len(code_bytes);

  // e_machine value
  let e_machine = 0x3E;  // x86_64
  if arch == ARCH_ARM64 { e_machine = 0xB7; }  // EM_AARCH64

  // ── ELF Header (64 bytes) ──
  // e_ident: magic
  push_bytes(buf, [0x7F, 0x45, 0x4C, 0x46]);
  // class=64, LE, version, Linux ABI, padding
  push_bytes(buf, [2, 1, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0]);
  // e_type = ET_EXEC
  push_u16(buf, 2);
  // e_machine
  push_u16(buf, e_machine);
  // e_version
  push_u32(buf, 1);
  // e_entry
  push_u64(buf, LOAD_ADDR + 120 + entry_offset);
  // e_phoff = 64
  push_u64(buf, 64);
  // e_shoff = 0
  push_u64(buf, 0);
  // e_flags
  push_u32(buf, 0);
  // e_ehsize = 64
  push_u16(buf, 64);
  // e_phentsize = 56
  push_u16(buf, 56);
  // e_phnum = 1
  push_u16(buf, 1);
  // e_shentsize, e_shnum, e_shstrndx
  push_u16(buf, 0);
  push_u16(buf, 0);
  push_u16(buf, 0);

  // ── Program Header (56 bytes) ──
  // p_type = PT_LOAD
  push_u32(buf, 1);
  // p_flags = RWX
  push_u32(buf, 7);
  // p_offset = 0
  push_u64(buf, 0);
  // p_vaddr
  push_u64(buf, LOAD_ADDR);
  // p_paddr
  push_u64(buf, LOAD_ADDR);
  // p_filesz
  push_u64(buf, total_size);
  // p_memsz
  push_u64(buf, total_size);
  // p_align
  push_u64(buf, 0x1000);

  // ── Code ──
  let i = 0;
  while i < len(code_bytes) {
    push(buf, code_bytes[i]);
    i = i + 1;
  }

  return buf;
}

// ── Origin header (32 bytes) ──

pub fn make_origin_header(vm_off, vm_sz, bc_off, bc_sz, kn_off, kn_sz, flags) {
  return make_origin_header_arch(vm_off, vm_sz, bc_off, bc_sz, kn_off, kn_sz, flags, ARCH_X86_64);
}

pub fn make_origin_header_arch(vm_off, vm_sz, bc_off, bc_sz, kn_off, kn_sz, flags, arch) {
  let buf = [];
  // Magic: ○LNG
  push_bytes(buf, [0xE2, 0x97, 0x8B, 0x4C]);
  // Version
  push(buf, 0x10);
  // Arch: 0x01=x86_64, 0x02=arm64
  let arch_byte = 0x01;
  if arch == ARCH_ARM64 { arch_byte = 0x02; }
  push(buf, arch_byte);
  push_u32(buf, vm_off);
  push_u32(buf, vm_sz);
  push_u32(buf, bc_off);
  push_u32(buf, bc_sz);
  push_u32(buf, kn_off);
  push_u32(buf, kn_sz);
  push_u16(buf, flags);
  return buf;
}

// ── Byte helpers ──

fn push_bytes(buf, bytes) {
  let i = 0;
  while i < len(bytes) {
    push(buf, bytes[i]);
    i = i + 1;
  }
}

fn push_u16(buf, val) {
  push(buf, val % 256);
  push(buf, (val / 256) % 256);
}

fn push_u32(buf, val) {
  push(buf, val % 256);
  push(buf, (val / 256) % 256);
  push(buf, (val / 65536) % 256);
  push(buf, (val / 16777216) % 256);
}

fn push_u64(buf, val) {
  push_u32(buf, val % 4294967296);
  push_u32(buf, val / 4294967296);
}
