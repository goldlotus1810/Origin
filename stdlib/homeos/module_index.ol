// stdlib/homeos/module_index.ol — Module versioning index
// PLAN 6.1.2: Track installed modules with version numbers.
// Index stored at end of bytecode section: [MIDX][count][entries...]
// Entry = [name_hash:8][offset:4][size:4][version:4] = 20 bytes

let INDEX_MAGIC = "MIDX";

pub fn index_new() {
  return { entries: [] };
}

pub fn index_add(idx, name_hash, offset, size) {
  // Add or update module entry (increment version if exists)
  let i = 0;
  let n = len(idx.entries);
  while i < n {
    if idx.entries[i].name_hash == name_hash {
      // Existing: bump version, update offset/size
      idx.entries[i].version = idx.entries[i].version + 1;
      idx.entries[i].offset = offset;
      idx.entries[i].size = size;
      return idx.entries[i].version;
    }
    i = i + 1;
  }
  // New module
  push(idx.entries, {
    name_hash: name_hash,
    offset: offset,
    size: size,
    version: 1
  });
  return 1;
}

pub fn index_find_latest(idx, name_hash) {
  // Return latest version entry for module
  let i = 0;
  let n = len(idx.entries);
  while i < n {
    if idx.entries[i].name_hash == name_hash {
      return idx.entries[i];
    }
    i = i + 1;
  }
  return { name_hash: 0, offset: 0, size: 0, version: 0 };
}

pub fn index_list(idx) {
  return idx.entries;
}

pub fn index_count(idx) {
  return len(idx.entries);
}

pub fn index_to_bytes(idx) {
  // Serialize index to bytes: [MIDX:4][count:4][entries...]
  let buf = [];
  // Magic
  push(buf, 77); push(buf, 73); push(buf, 68); push(buf, 88);  // "MIDX"
  // Count (4 bytes LE)
  let count = len(idx.entries);
  push_u32(buf, count);
  // Entries (20 bytes each)
  let i = 0;
  while i < count {
    let e = idx.entries[i];
    push_u64(buf, e.name_hash);
    push_u32(buf, e.offset);
    push_u32(buf, e.size);
    push_u32(buf, e.version);
    i = i + 1;
  }
  return buf;
}

pub fn index_from_bytes(bytes, start, length) {
  // Deserialize index from bytes
  let idx = index_new();
  if length < 8 { return idx; }
  // Check magic
  if bytes[start] != 77 || bytes[start + 1] != 73 ||
     bytes[start + 2] != 68 || bytes[start + 3] != 88 {
    return idx;  // no valid index
  }
  let count = read_u32(bytes, start + 4);
  let off = start + 8;
  let i = 0;
  while i < count && off + 20 <= start + length {
    push(idx.entries, {
      name_hash: read_u64(bytes, off),
      offset: read_u32(bytes, off + 8),
      size: read_u32(bytes, off + 12),
      version: read_u32(bytes, off + 16)
    });
    off = off + 20;
    i = i + 1;
  }
  return idx;
}

// ── Helpers ─────────────────────────────────────────────────────────

fn push_u32(buf, val) {
  let i = 0;
  while i < 4 {
    push(buf, val - floor(val / 256) * 256);
    val = floor(val / 256);
    i = i + 1;
  }
}

fn push_u64(buf, val) {
  let i = 0;
  while i < 8 {
    push(buf, val - floor(val / 256) * 256);
    val = floor(val / 256);
    i = i + 1;
  }
}

fn read_u32(bytes, off) {
  return bytes[off] + bytes[off + 1] * 256 +
         bytes[off + 2] * 65536 + bytes[off + 3] * 16777216;
}

fn read_u64(bytes, off) {
  return read_u32(bytes, off) + read_u32(bytes, off + 4) * 4294967296;
}
