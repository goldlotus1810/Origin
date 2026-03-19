// stdlib/homeos/install.ol — Self-update: install/update/learn modules
// PLAN 6.1.1: origin.olang tự append bytecode/knowledge vào chính nó.
// Atomic update: copy → modify → rename (crash-safe).

// ── Install: compile .ol → append bytecode ─────────────────────────

pub fn install(source_path) {
  let src = __file_read(source_path);
  if len(src) == 0 {
    emit "Error: cannot read ";
    emit source_path;
    emit "\n";
    return false;
  }

  // Compile source to bytecode
  let bytecode = __compile(src);
  if len(bytecode) == 0 {
    emit "Error: compilation failed\n";
    return false;
  }

  // Compute module name hash from path
  let name_hash = path_to_hash(source_path);

  // Read current binary
  let self_path = __self_path();
  let self_bytes = __file_read_bytes(self_path);
  if len(self_bytes) == 0 {
    emit "Error: cannot read self\n";
    return false;
  }

  // Parse origin header (find bc/kn offsets)
  let header = parse_origin_header(self_bytes);
  if header.valid == false {
    emit "Error: invalid origin header\n";
    return false;
  }

  // Build module index entry
  let entry = {
    name_hash: name_hash,
    offset: header.bc_offset + header.bc_size,
    size: len(bytecode),
    version: 1
  };

  // Check existing module index for version bump
  let idx = parse_module_index(self_bytes, header);
  let existing = find_module(idx, name_hash);
  if existing.size > 0 {
    entry.version = existing.version + 1;
  }

  // Append bytecode (before knowledge section)
  // Layout: [VM][header][old_bc][NEW_BC][knowledge][trailer]
  let new_bytes = [];
  // Copy everything up to end of bytecode
  append_range(new_bytes, self_bytes, 0, header.bc_offset + header.bc_size);
  // Append new bytecode
  append_bytes(new_bytes, bytecode);
  // Copy knowledge section
  append_range(new_bytes, self_bytes, header.kn_offset, header.kn_offset + header.kn_size);
  // Update header: bc_size, kn_offset
  let new_bc_size = header.bc_size + len(bytecode);
  let new_kn_offset = header.bc_offset + new_bc_size;
  patch_header(new_bytes, header.header_offset, new_bc_size, new_kn_offset, header.kn_size);
  // Append trailer (8 bytes: header offset)
  append_u64(new_bytes, header.header_offset);

  // Atomic write: .new → rename
  let tmp_path = self_path + ".new";
  __file_write_bytes(tmp_path, new_bytes);
  __rename(tmp_path, self_path);

  emit "Installed: ";
  emit source_path;
  emit " (v";
  emit entry.version;
  emit ", ";
  emit len(bytecode);
  emit " bytes)\n";
  return true;
}

// ── Update: same as install but tracks version ─────────────────────

pub fn update(source_path) {
  // Update = install with version tracking
  return install(source_path);
}

// ── Learn: append data to knowledge section ─────────────────────────

pub fn learn_file(data_path) {
  let data = __file_read_bytes(data_path);
  if len(data) == 0 {
    emit "Error: cannot read ";
    emit data_path;
    emit "\n";
    return false;
  }

  let self_path = __self_path();
  let self_bytes = __file_read_bytes(self_path);
  let header = parse_origin_header(self_bytes);
  if header.valid == false { return false; }

  // Append to knowledge section
  let new_bytes = [];
  // Copy everything up to end of knowledge
  append_range(new_bytes, self_bytes, 0, header.kn_offset + header.kn_size);
  // Append new knowledge data
  append_bytes(new_bytes, data);
  // Update header: kn_size
  let new_kn_size = header.kn_size + len(data);
  patch_header(new_bytes, header.header_offset, header.bc_size,
               header.bc_offset + header.bc_size, new_kn_size);
  // Trailer
  append_u64(new_bytes, header.header_offset);

  // Atomic write
  let tmp_path = self_path + ".new";
  __file_write_bytes(tmp_path, new_bytes);
  __rename(tmp_path, self_path);

  emit "Learned: ";
  emit data_path;
  emit " (";
  emit len(data);
  emit " bytes)\n";
  return true;
}

// ── Header parsing helpers ──────────────────────────────────────────

fn parse_origin_header(bytes) {
  // Find origin header in binary
  // Wrap mode: last 8 bytes = header offset
  let file_len = len(bytes);
  if file_len < 40 { return { valid: false }; }

  let header_offset = 0;

  // Check if starts with ELF magic
  if bytes[0] == 127 && bytes[1] == 69 {
    // ELF: read trailer (last 8 bytes)
    header_offset = read_u64(bytes, file_len - 8);
  } else {
    // Direct origin header at 0
    header_offset = 0;
  }

  // Validate magic: ○LNG = [0xE2, 0x97, 0x8B, 0x4C]
  let off = header_offset;
  if bytes[off] != 226 || bytes[off + 1] != 151 ||
     bytes[off + 2] != 139 || bytes[off + 3] != 76 {
    return { valid: false };
  }

  return {
    valid: true,
    header_offset: header_offset,
    version: bytes[off + 4],
    arch: bytes[off + 5],
    vm_offset: read_u32(bytes, off + 6),
    vm_size: read_u32(bytes, off + 10),
    bc_offset: read_u32(bytes, off + 14),
    bc_size: read_u32(bytes, off + 18),
    kn_offset: read_u32(bytes, off + 22),
    kn_size: read_u32(bytes, off + 26),
    flags: read_u16(bytes, off + 30)
  };
}

fn patch_header(bytes, header_offset, bc_size, kn_offset, kn_size) {
  let off = header_offset;
  write_u32(bytes, off + 18, bc_size);
  write_u32(bytes, off + 22, kn_offset);
  write_u32(bytes, off + 26, kn_size);
}

fn path_to_hash(path) {
  // FNV-1a hash of path string
  let hash = 2166136261;
  let i = 0;
  while i < len(path) {
    let ch = char_at(path, i);
    hash = hash * 16777619;
    hash = hash + to_num(ch);
    i = i + 1;
  }
  return hash;
}

// ── Byte manipulation helpers ───────────────────────────────────────

fn read_u16(bytes, off) {
  return bytes[off] + bytes[off + 1] * 256;
}

fn read_u32(bytes, off) {
  return bytes[off] + bytes[off + 1] * 256 +
         bytes[off + 2] * 65536 + bytes[off + 3] * 16777216;
}

fn read_u64(bytes, off) {
  return read_u32(bytes, off) + read_u32(bytes, off + 4) * 4294967296;
}

fn write_u32(bytes, off, val) {
  bytes[off] = val - floor(val / 256) * 256;
  val = floor(val / 256);
  bytes[off + 1] = val - floor(val / 256) * 256;
  val = floor(val / 256);
  bytes[off + 2] = val - floor(val / 256) * 256;
  val = floor(val / 256);
  bytes[off + 3] = val;
}

fn append_range(dest, src, from, to) {
  let i = from;
  while i < to {
    push(dest, src[i]);
    i = i + 1;
  }
}

fn append_bytes(dest, src) {
  let i = 0;
  while i < len(src) {
    push(dest, src[i]);
    i = i + 1;
  }
}

fn append_u64(dest, val) {
  let i = 0;
  while i < 8 {
    push(dest, val - floor(val / 256) * 256);
    val = floor(val / 256);
    i = i + 1;
  }
}
