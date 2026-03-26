# PLAN 3.2 — elf_emit.ol: Olang tạo ELF binary (~200 LOC)

**Phụ thuộc:** PLAN_3_1 (asm_emit.ol)
**Mục tiêu:** Olang tạo ELF64 executable từ machine code bytes
**Tham chiếu:** `tools/builder/src/elf.rs` (Rust reference)

---

## Cấu trúc

```
fn make_elf(code_bytes, entry_offset) {
  let buf = [];

  // ELF header (64 bytes)
  append_elf_header(buf, entry_offset, total_size);

  // Program header (56 bytes)
  append_program_header(buf, total_size);

  // Code
  append_bytes(buf, code_bytes);

  return buf;
}

fn append_elf_header(buf, entry_vaddr, file_size) {
  // Magic
  push_bytes(buf, [0x7F, 0x45, 0x4C, 0x46]);
  // Class=64, LE, version, Linux ABI
  push_bytes(buf, [2, 1, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0]);
  // e_type = ET_EXEC
  push_u16_le(buf, 2);
  // e_machine = x86_64
  push_u16_le(buf, 0x3E);
  // e_version
  push_u32_le(buf, 1);
  // e_entry
  push_u64_le(buf, entry_vaddr);
  // e_phoff = 64
  push_u64_le(buf, 64);
  // e_shoff = 0
  push_u64_le(buf, 0);
  // e_flags = 0
  push_u32_le(buf, 0);
  // e_ehsize = 64
  push_u16_le(buf, 64);
  // e_phentsize = 56
  push_u16_le(buf, 56);
  // e_phnum = 1
  push_u16_le(buf, 1);
  // e_shentsize, e_shnum, e_shstrndx = 0
  push_u16_le(buf, 0);
  push_u16_le(buf, 0);
  push_u16_le(buf, 0);
}

fn append_program_header(buf, file_size) {
  // PT_LOAD
  push_u32_le(buf, 1);
  // PF_R|PF_W|PF_X
  push_u32_le(buf, 7);
  // p_offset = 0
  push_u64_le(buf, 0);
  // p_vaddr = 0x400000
  push_u64_le(buf, 0x400000);
  // p_paddr
  push_u64_le(buf, 0x400000);
  // p_filesz
  push_u64_le(buf, file_size);
  // p_memsz
  push_u64_le(buf, file_size);
  // p_align
  push_u64_le(buf, 0x1000);
}
```

---

## Definition of Done

- [ ] `elf_emit.ol` generates valid ELF64 header
- [ ] `readelf -h` validates output
- [ ] Combined with asm_emit.ol: generate runnable binary

## Ước tính: 0.5 ngày
