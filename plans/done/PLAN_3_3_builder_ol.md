# PLAN 3.3 — builder.ol: Self-sufficient Builder (~300 LOC)

**Phụ thuộc:** PLAN_3_1 (asm_emit.ol), PLAN_3_2 (elf_emit.ol)
**Mục tiêu:** Olang tự build origin.olang → KHÔNG CẦN Rust builder nữa
**Tham chiếu:** `tools/builder/src/` (Rust reference)

---

## Cấu trúc

```
fn build(config) {
  // 1. Compile all .ol → bytecode
  let bytecode = compile_all(config.stdlib_path);

  // 2. Emit VM machine code
  let vm_code = emit_vm(config.arch);

  // 3. Build origin header
  let header = make_origin_header(vm_code, bytecode, config.knowledge);

  // 4. Wrap in ELF
  let elf = make_elf(concat(header, vm_code, bytecode, config.knowledge),
                     120 + len(header));  // entry = after ELF + origin headers

  // 5. Write file
  file_write(config.output, elf);

  // 6. chmod +x
  emit("Built: " + config.output);
}
```

---

## PLAN 3.4 — Full Self-Build Test

```
Quy trình:
  1. origin.olang v1 (built by Rust builder) chạy builder.ol
  2. builder.ol → tạo origin.olang v2
  3. origin.olang v2 chạy builder.ol → tạo origin.olang v3
  4. Assert: v2 == v3 (fixed point)

Nếu v2 == v3 → compiler deterministic → self-hosting thành công
Nếu v2 != v3 → debug non-determinism (timestamps, random, etc.)
```

---

## Definition of Done

- [ ] `builder.ol` compiles
- [ ] Reads .ol files, compiles to bytecode
- [ ] Packs VM + bytecode + knowledge → origin.olang
- [ ] Output is valid ELF (`readelf -h` passes)
- [ ] Output runs (`./origin_v2.olang` starts)
- [ ] **CRITICAL:** v2 == v3 (self-build fixed point)

## Ước tính: 2-3 ngày

---

*Sau PLAN 3.4: Rust = 0%. origin.olang hoàn toàn tự đủ.*
