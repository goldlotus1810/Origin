// homeos/fat_header.ol — Fat binary format: multi-arch in 1 file
// Shared bytecode + knowledge, per-arch VM sections
//
// Layout:
//   [Fat Header 64B]
//   [VM Section 0: x86_64]
//   [VM Section 1: ARM64]
//   ...
//   [Bytecode: shared]
//   [Knowledge: shared]

// ── Architecture IDs ──

let FAT_ARCH_X86_64 = 0x01;
let FAT_ARCH_ARM64  = 0x02;
let FAT_ARCH_RISCV  = 0x03;
let FAT_ARCH_WASM   = 0x04;

// Fat format version (0x20 = v32, distinguishes from single-arch 0x10)
let FAT_VERSION = 0x20;

// Magic bytes: ○LNG
let FAT_MAGIC = [0xE2, 0x97, 0x8B, 0x4C];

// ── Per-arch entry (16 bytes) ──
//   [arch_id: 1B] [vm_off: 4B] [vm_size: 4B] [entry_off: 4B] [reserved: 3B]

fn make_arch_entry(arch_id, vm_off, vm_size, entry_off) {
  let buf = [];
  push(buf, arch_id);
  push_u32(buf, vm_off);
  push_u32(buf, vm_size);
  push_u32(buf, entry_off);
  // reserved 3 bytes
  push(buf, 0);
  push(buf, 0);
  push(buf, 0);
  return buf;
}

// ── Fat Header (64 bytes) ──
//   [magic: 4B] [version: 1B] [arch_cnt: 1B] [flags: 2B]
//   [per-arch entries: 16B × arch_cnt]
//   [bc_offset: 4B] [bc_size: 4B] [kn_offset: 4B] [kn_size: 4B]
//   [pad to 64B]

pub fn make_fat_header(archs, bc_off, bc_sz, kn_off, kn_sz) {
  let buf = [];
  let arch_cnt = len(archs);

  // Validate: 1-4 architectures
  if arch_cnt < 1 || arch_cnt > 4 {
    emit("ERROR: fat binary supports 1-4 architectures\n");
    return [];
  }

  // Magic
  push_bytes(buf, FAT_MAGIC);
  // Version
  push(buf, FAT_VERSION);
  // Architecture count
  push(buf, arch_cnt);
  // Flags (reserved)
  push_u16(buf, 0);

  // Per-arch entries (16B each, max 4 = 64B)
  let i = 0;
  while i < arch_cnt {
    let entry = archs[i];
    let entry_bytes = make_arch_entry(
      entry.arch_id, entry.vm_off, entry.vm_size, entry.entry_off
    );
    push_bytes(buf, entry_bytes);
    i = i + 1;
  }

  // Shared section offsets
  push_u32(buf, bc_off);
  push_u32(buf, bc_sz);
  push_u32(buf, kn_off);
  push_u32(buf, kn_sz);

  // Pad to 64 bytes
  while len(buf) < 64 {
    push(buf, 0);
  }

  return buf;
}

// ── Parse fat header from bytes ──

pub fn parse_fat_header(bytes) {
  // Validate magic
  if len(bytes) < 64 {
    return { valid: 0, error: "too short" };
  }
  if bytes[0] != 0xE2 || bytes[1] != 0x97 || bytes[2] != 0x8B || bytes[3] != 0x4C {
    return { valid: 0, error: "bad magic" };
  }

  let version = bytes[4];
  if version != FAT_VERSION {
    return { valid: 0, error: "not fat format" };
  }

  let arch_cnt = bytes[5];
  let flags = read_u16(bytes, 6);

  // Parse per-arch entries starting at offset 8
  let archs = [];
  let off = 8;
  let i = 0;
  while i < arch_cnt {
    let arch_id   = bytes[off];
    let vm_off    = read_u32(bytes, off + 1);
    let vm_size   = read_u32(bytes, off + 5);
    let entry_off = read_u32(bytes, off + 9);
    push(archs, {
      arch_id: arch_id,
      vm_off: vm_off,
      vm_size: vm_size,
      entry_off: entry_off
    });
    off = off + 16;
    i = i + 1;
  }

  // Shared section offsets (after per-arch entries)
  let bc_off  = read_u32(bytes, off);
  let bc_sz   = read_u32(bytes, off + 4);
  let kn_off  = read_u32(bytes, off + 8);
  let kn_sz   = read_u32(bytes, off + 12);

  return {
    valid: 1,
    version: version,
    arch_cnt: arch_cnt,
    flags: flags,
    archs: archs,
    bc_offset: bc_off,
    bc_size: bc_sz,
    kn_offset: kn_off,
    kn_size: kn_sz
  };
}

// ── Find arch entry by ID ──

pub fn find_arch(fat_hdr, arch_id) {
  let i = 0;
  while i < fat_hdr.arch_cnt {
    if fat_hdr.archs[i].arch_id == arch_id {
      return fat_hdr.archs[i];
    }
    i = i + 1;
  }
  return { arch_id: 0, vm_off: 0, vm_size: 0, entry_off: 0 };
}

// ── Extract VM bytes for a given arch from fat binary ──

pub fn extract_vm(fat_bytes, fat_hdr, arch_id) {
  let arch = find_arch(fat_hdr, arch_id);
  if arch.arch_id == 0 {
    return [];
  }
  let result = [];
  let i = 0;
  while i < arch.vm_size {
    push(result, fat_bytes[arch.vm_off + i]);
    i = i + 1;
  }
  return result;
}

// ── Extract shared bytecode from fat binary ──

pub fn extract_bytecode(fat_bytes, fat_hdr) {
  let result = [];
  let i = 0;
  while i < fat_hdr.bc_size {
    push(result, fat_bytes[fat_hdr.bc_offset + i]);
    i = i + 1;
  }
  return result;
}

// ── Extract shared knowledge from fat binary ──

pub fn extract_knowledge(fat_bytes, fat_hdr) {
  let result = [];
  let i = 0;
  while i < fat_hdr.kn_size {
    push(result, fat_bytes[fat_hdr.kn_offset + i]);
    i = i + 1;
  }
  return result;
}

// ── Byte helpers (read, little-endian) ──

fn read_u16(bytes, off) {
  return bytes[off] + bytes[off + 1] * 256;
}

fn read_u32(bytes, off) {
  return bytes[off]
       + bytes[off + 1] * 256
       + bytes[off + 2] * 65536
       + bytes[off + 3] * 16777216;
}

// ── Byte helpers (write, little-endian) — same as elf_emit.ol ──

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
