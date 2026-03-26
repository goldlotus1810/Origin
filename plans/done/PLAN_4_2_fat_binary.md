# PLAN 4.2 — Fat Binary (optional)

**Phụ thuộc:** 4.1 (cross-compile hoạt động)
**Mục tiêu:** 1 origin.olang chứa VM cho NHIỀU architecture
**Tham chiếu:** macOS Universal Binary concept

---

## Bối cảnh

```
HIỆN TẠI (sau 4.1):
  origin_x86.olang = ELF x86_64 (VM x86 + bytecode + knowledge)
  origin_arm.olang = ELF ARM64  (VM ARM + bytecode + knowledge)
  → 2 files, bytecode + knowledge giống nhau → lãng phí

SAU PLAN 4.2:
  origin.olang = 1 file chứa:
    [Fat Header]
    [VM x86_64]
    [VM ARM64]
    [Bytecode]     ← chia sẻ (arch-independent)
    [Knowledge]    ← chia sẻ
  → Loader chọn VM section đúng theo runtime detect
```

---

## Cấu trúc Fat Header

```
Fat Origin Format:

┌──────────────────────────────────────────────────────┐
│ FAT HEADER (64 bytes)                                 │
│   [○LNG]      4B    magic                             │
│   [0x20]      1B    version (v32 = fat format)        │
│   [arch_cnt]  1B    số lượng architecture (2-4)       │
│   [flags]     2B    feature flags                     │
│   ┌── per-arch entry (16 bytes × arch_cnt) ──┐       │
│   │  [arch_id]   1B   0x01=x86, 0x02=arm64,  │       │
│   │                    0x03=riscv, 0x04=wasm  │       │
│   │  [vm_off]    4B   offset to VM section    │       │
│   │  [vm_size]   4B   size of VM code         │       │
│   │  [entry_off] 4B   entry point offset      │       │
│   │  [reserved]  3B                           │       │
│   └───────────────────────────────────────────┘       │
│   [bc_offset]  4B    shared bytecode offset           │
│   [bc_size]    4B    shared bytecode size              │
│   [kn_offset]  4B    shared knowledge offset           │
│   [kn_size]    4B    shared knowledge size             │
│   [pad to 64]                                         │
├──────────────────────────────────────────────────────┤
│ VM SECTION 0: x86_64 (~100 KB)                        │
├──────────────────────────────────────────────────────┤
│ VM SECTION 1: ARM64 (~80 KB)                          │
├──────────────────────────────────────────────────────┤
│ BYTECODE: shared (~500 KB)                            │
├──────────────────────────────────────────────────────┤
│ KNOWLEDGE: shared (grows)                             │
└──────────────────────────────────────────────────────┘
```

---

## Tasks

### 4.2.1 — fat_header.ol (~100 LOC)

```
pub fn make_fat_header(archs, bc_off, bc_sz, kn_off, kn_sz) → bytes
  // archs = [{ id, vm_off, vm_sz, entry_off }, ...]
  // Validate: arch_cnt 1-4
  // Serialize header

pub fn parse_fat_header(bytes) → FatHeader
  // Deserialize header
  // Used by loader at startup
```

### 4.2.2 — Loader stub (~50 LOC ASM per arch)

Mỗi architecture cần loader stub ở đầu file:
```
x86_64 loader (ELF entry point):
  1. Read fat header (first 64 bytes of self)
  2. Find arch_id == 0x01
  3. Jump to vm_off + entry_off

ARM64 loader (ELF entry point):
  1. Read fat header
  2. Find arch_id == 0x02
  3. Jump to vm_off + entry_off
```

Vấn đề: ELF chỉ hỗ trợ 1 e_machine → không thể là multi-arch ELF.

**Giải pháp:** Script wrapper hoặc platform-specific loader:
```
Option A: symlink-based (giống macOS lipo)
  origin.olang = fat binary (raw, không ELF header)
  o_x86   = ELF x86 stub → mmap origin.olang → jump
  o_arm64 = ELF ARM stub → mmap origin.olang → jump

Option B: arch detect script
  #!/bin/sh
  case $(uname -m) in
    x86_64) exec ./o_x86 "$@" ;;
    aarch64) exec ./o_arm "$@" ;;
  esac

Option C: Linux binfmt_misc
  Register origin.olang with custom handler
```

**Đề xuất: Option A** — mỗi arch có 1 ELF stub nhỏ (~1 KB) mmap fat binary.

### 4.2.3 — builder.ol mở rộng

```
build_fat(config) {
  // 1. Compile bytecode (shared)
  // 2. For each arch in config.archs:
  //      Read pre-assembled VM binary
  // 3. Calculate offsets
  // 4. Serialize fat header + VM sections + bytecode + knowledge
}
```

---

## Rào cản

```
1. ELF format = single architecture
   → Fat binary KHÔNG phải ELF → cần wrapper stubs
   → Hoặc: build separate ELFs per arch (simpler, recommended initially)

2. Fat binary tăng kích thước
   → 2 VMs (~180 KB) vs 1 VM (~100 KB) = chỉ tăng ~80 KB
   → Bytecode + knowledge chia sẻ → tiết kiệm đáng kể vs 2 separate files
```

---

## Definition of Done

- [ ] fat_header.ol: serialize/parse fat header
- [ ] Loader stubs cho x86_64 và ARM64
- [ ] builder.ol: build_fat() tạo fat binary
- [ ] Test: fat binary → extract correct VM per arch
- [ ] Test: loader stub mmap + jump thành công

## Ước tính: 2-3 ngày

## Ghi chú

Task này là **optional**. Nếu cross-compile (4.1) hoạt động tốt, có thể bỏ qua
fat binary và chỉ build separate binaries per architecture. Fat binary chủ yếu
hữu ích khi distribute cho end users trên nhiều platform.
