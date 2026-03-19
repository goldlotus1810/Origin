# PLAN 6.1 — Self-Update

**Phụ thuộc:** Phase 4 DONE (multi-arch build)
**Mục tiêu:** origin.olang tự cập nhật: ăn .ol mới → compile → append → tự biến đổi
**Tham chiếu:** `PLAN_REWRITE.md` (Giai đoạn 6.1)

---

## Bối cảnh

```
HIỆN TẠI:
  origin.olang = static binary, build 1 lần bởi builder.ol
  Muốn cập nhật → rebuild toàn bộ bằng builder.ol

SAU PLAN 6.1:
  origin.olang tự append thêm bytecode/knowledge vào chính nó
  "o install emotion_v2.ol" → compile → append → tự restart
  VM/bytecode sections = versioned, replaceable
  Knowledge section = append-only (QT⑨)
```

---

## Thiết kế

### Self-modify workflow

```
o install new_module.ol
  1. origin.olang đọc new_module.ol
  2. parse → detect program/data blocks
  3. Program blocks: compile → bytecode
  4. Data blocks: encode → knowledge records
  5. Append bytecode + knowledge vào CHÍNH MÌNH (origin.olang)
  6. Update section offsets trong header
  7. Restart VM → load section mới

o update existing_module.ol
  1. Giống install, nhưng:
  2. Thêm version tag [module_name:hash:version]
  3. Bytecode cũ KHÔNG xóa (append-only)
  4. Module loader ưu tiên version mới nhất
  5. Rollback: "o rollback module_name" → trỏ lại version cũ

o learn data.ol
  1. Parse → chỉ data blocks (không có ○{ })
  2. Encode → knowledge records
  3. Append vào knowledge section
  4. Update kn_size trong header
```

### Section versioning

```
Bytecode section layout (sau update):

  [module_a v1 bytecode]
  [module_b v1 bytecode]
  [module_a v2 bytecode]    ← install update
  [module_c v1 bytecode]    ← install mới

Module index (ở cuối bytecode section):
  [index_magic: 4B "MIDX"]
  [entry_count: 4B]
  [entries...]
    entry = [name_hash:8][offset:4][size:4][version:4] = 20 bytes

VM startup: đọc module index → load latest version mỗi module
```

### Self-append mechanism

```
origin.olang sửa chính nó:

1. Open self: fd = open("/proc/self/exe", O_RDWR)
   → Linux không cho phép sửa running executable!

Giải pháp: KHÔNG sửa in-place → copy + append + rename

  1. cp origin.olang origin.olang.new
  2. Append new sections vào origin.olang.new
  3. Update header trong origin.olang.new
  4. mv origin.olang.new origin.olang
  5. exec("origin.olang") → restart (optional)

Atomic update: rename() là atomic trên cùng filesystem.
Crash-safe: nếu crash giữa chừng, origin.olang cũ vẫn intact.
```

---

## Tasks

### 6.1.1 — install command (~200 LOC Olang)

```
install.ol:

pub fn install(source_path) {
  let src = file_read_string(source_path);
  let blocks = parse_blocks(src);  // separate program vs data

  let new_bc = [];
  let new_kn = [];

  for block in blocks {
    if block.type == "program" {
      let bc = compile_source(block.content);
      concat_bytes(new_bc, bc);
    }
    if block.type == "data" {
      let kn = encode_data(block.content);
      concat_bytes(new_kn, kn);
    }
  }

  // Read current origin.olang
  let self_path = __self_path();       // /proc/self/exe readlink
  let self_bytes = file_read_bytes(self_path);

  // Append
  let updated = append_sections(self_bytes, new_bc, new_kn);

  // Atomic write
  let tmp = self_path + ".new";
  file_write_bytes(tmp, updated);
  __rename(tmp, self_path);

  emit("Installed: " + source_path + "\n");
}
```

### 6.1.2 — Module index (~100 LOC)

```
module_index.ol:

pub fn build_index(modules) → bytes
  // modules = [{ name, offset, size, version }, ...]

pub fn parse_index(bc_bytes) → modules
  // Read from end of bytecode section

pub fn find_latest(index, name_hash) → { offset, size }
  // Return highest version for given module

pub fn add_module(index, name, offset, size) → updated_index
  // Append entry, increment version if name exists
```

### 6.1.3 — Header update (~50 LOC)

```
Sau append, cần update trong header:
  - bc_size += len(new_bytecode)
  - kn_size += len(new_knowledge)
  - Header vị trí cố định → seek + overwrite (trong .new file)
```

### 6.1.4 — VM builtins needed

```
__self_path()            → readlink("/proc/self/exe")
__rename(old, new)       → rename syscall
__file_append(path, data) → open(O_APPEND) + write
__exec(path, args)       → execve (optional, for restart)
```

---

## Rào cản

```
1. Linux: cannot modify running executable
   → Giải pháp: copy → modify → rename (atomic)
   → macOS: same restriction → same solution

2. Signature invalidation
   → Append thay đổi file → Ed25519 signature invalid
   → Giải pháp: re-sign sau mỗi update
   → Cần master key (password/biometric)
   → "o install" yêu cầu auth trước khi apply

3. Bytecode backward compatibility
   → Module compiled với compiler v1 → chạy trên VM v2?
   → Bytecode format stable (opcodes cố định)
   → Nếu VM thêm opcode mới → old bytecode không dùng → ok
   → Nếu VM đổi opcode semantics → version tag → reject old bytecode

4. Knowledge section grow → file quá lớn
   → origin.olang ban đầu ~616 KB
   → Knowledge grows ~33 bytes/concept
   → 1M concepts = ~33 MB → acceptable trên điện thoại
   → 10M concepts = ~330 MB → cần compaction (defer)
```

---

## Test Plan

```
Test 1: install hello.ol → verify output "Hello" works after install
Test 2: install → update (same module) → verify v2 loaded, v1 still exists
Test 3: rollback → verify v1 loaded again
Test 4: learn data.ol → verify knowledge section grew
Test 5: Crash simulation → rename không xảy ra → verify old file intact
Test 6: Module index → find_latest returns correct version
```

---

## Definition of Done

- [ ] `o install module.ol` → compile + append + atomic update
- [ ] `o update module.ol` → version increment + append
- [ ] `o learn data.ol` → encode + append knowledge
- [ ] Module index (versioned, latest-wins)
- [ ] Atomic self-update (copy + rename)
- [ ] Auth required for self-modification
- [ ] Test: install + update + rollback cycle

## Ước tính: 1-2 tuần
