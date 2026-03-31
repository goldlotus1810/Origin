# PLAN CUT.4 — Self-Build: origin.olang builds itself

> **Goal**: `build` REPL command → reads .ol files → compiles → packs → writes new binary
> **Result**: No Rust dependency. origin.olang = self-sufficient.
> **Status**: Planning

---

## Binary Format (Wrap Mode)

```
[VM ELF binary]              ← copy from current binary (bytes 0 to header_offset)
[Origin Header 32 bytes]     ← magic + offsets + sizes
[Bytecode blob]              ← all compiled .ol files concatenated
[Knowledge blob]             ← optional (previous binary)
[8-byte LE header_offset]    ← trailer pointing back to header
```

### Origin Header (32 bytes)

```
Bytes 0-3:   Magic "○LNG" (0xE2 0x97 0x8B 0x4C)
Byte 4:      Version 0x10 (self-exec)
Byte 5:      Arch (0x01 = x86_64)
Bytes 6-9:   vm_offset (u32 LE) — 0 in wrap mode
Bytes 10-13: vm_size (u32 LE) — 0 in wrap mode
Bytes 14-17: bc_offset (u32 LE) — header_offset + 32
Bytes 18-21: bc_size (u32 LE) — total bytecode length
Bytes 22-25: kn_offset (u32 LE) — after bytecode
Bytes 26-29: kn_size (u32 LE) — 0 if no knowledge
Bytes 30-31: flags (u16 LE) — bit 0 = codegen format
```

---

## Steps

### Step 1: Raw byte builtins (~30 LOC ASM)

Need builtins for binary data manipulation:

```
__bytes_new(size) → raw byte buffer on heap
__bytes_set(buf, offset, byte_value) → write 1 byte
__bytes_get(buf, offset) → read 1 byte
__bytes_write(path, buf, size) → write raw bytes to file
__bytes_copy(dst, dst_off, src, src_off, len) → memcpy
__file_read_bytes(path) → raw byte buffer + size
```

Current `__file_write` encodes as u16 molecules. Need RAW byte write for binary output.

### Step 2: Batch compile (`compile_all`) (~50 LOC Olang)

```olang
fn compile_all() {
    // Order: bootstrap first, then stdlib, then homeos
    let files = [
        "stdlib/bootstrap/lexer.ol",
        "stdlib/bootstrap/parser.ol",
        "stdlib/bootstrap/semantic.ol",
        "stdlib/bootstrap/codegen.ol",
        // ... stdlib root ...
        // ... stdlib/homeos/ ...
    ];
    let all_bytecode = __bytes_new(1048576);  // 1MB buffer
    let total_pos = 0;
    for f in files {
        // Reset compiler state
        _prefill_output();
        let _g_pos = 0;
        // Compile
        let src = __file_read(f);
        let tokens = tokenize(src);
        let ast = parse(tokens);
        let state = analyze(ast);
        // Copy bytecode to all_bytecode buffer
        let i = 0;
        while i < _g_pos {
            __bytes_set(all_bytecode, total_pos + i, _g_output[i]);
            i = i + 1;
        };
        total_pos = total_pos + _g_pos;
    };
    // Append Halt
    __bytes_set(all_bytecode, total_pos, 0x0F);
    total_pos = total_pos + 1;
    return { buf: all_bytecode, size: total_pos };
}
```

### Step 3: Extract VM from current binary (~20 LOC Olang)

```olang
fn extract_vm() {
    // Read current binary
    let self_bytes = __file_read_bytes("/proc/self/exe");
    // Read last 8 bytes = header_offset
    let header_offset = __bytes_read_u64_le(self_bytes, self_bytes.size - 8);
    // VM = bytes 0 to header_offset
    return { buf: self_bytes.buf, size: header_offset };
}
```

### Step 4: Pack binary (~40 LOC Olang)

```olang
fn pack_binary(vm, bytecode) {
    let output_size = vm.size + 32 + bytecode.size + 8;
    let output = __bytes_new(output_size);
    let pos = 0;
    // Copy VM ELF
    __bytes_copy(output, 0, vm.buf, 0, vm.size);
    pos = vm.size;
    // Write Origin header
    let header_offset = pos;
    let bc_offset = header_offset + 32;
    __bytes_set(output, pos, 0xE2);     // ○
    __bytes_set(output, pos+1, 0x97);
    __bytes_set(output, pos+2, 0x8B);
    __bytes_set(output, pos+3, 0x4C);   // LNG
    __bytes_set(output, pos+4, 0x10);   // version
    __bytes_set(output, pos+5, 0x01);   // x86_64
    // bc_offset as u32 LE
    __bytes_set(output, pos+14, bc_offset % 256);
    __bytes_set(output, pos+15, __floor(bc_offset / 256) % 256);
    __bytes_set(output, pos+16, __floor(bc_offset / 65536) % 256);
    __bytes_set(output, pos+17, __floor(bc_offset / 16777216) % 256);
    // bc_size as u32 LE
    __bytes_set(output, pos+18, bytecode.size % 256);
    __bytes_set(output, pos+19, __floor(bytecode.size / 256) % 256);
    __bytes_set(output, pos+20, __floor(bytecode.size / 65536) % 256);
    __bytes_set(output, pos+21, __floor(bytecode.size / 16777216) % 256);
    // flags
    __bytes_set(output, pos+30, 1);     // codegen format
    pos = pos + 32;
    // Copy bytecode
    __bytes_copy(output, pos, bytecode.buf, 0, bytecode.size);
    pos = pos + bytecode.size;
    // Trailer: header_offset as u64 LE
    __bytes_set(output, pos, header_offset % 256);
    // ... (8 bytes LE)
    pos = pos + 8;
    return { buf: output, size: pos };
}
```

### Step 5: `build` REPL command (~10 LOC)

```olang
if src == "build" {
    let vm = extract_vm();
    let bc = compile_all();
    let binary = pack_binary(vm, bc);
    __bytes_write("origin_new.olang", binary.buf, binary.size);
    return $"Built: {binary.size} bytes";
}
```

---

## Dependencies

| What | Status | Needed for |
|------|--------|-----------|
| `__file_read` | ✅ DONE | Read .ol source files |
| `__file_write` | ✅ DONE (u16 strings) | Not enough — need raw bytes |
| `__bytes_new/set/get` | ❌ NEW | Raw byte buffer manipulation |
| `__bytes_write` | ❌ NEW | Write raw binary to file |
| `__file_read_bytes` | ❌ NEW | Read current binary as raw bytes |
| `compile` command | ✅ DONE | Single-file compile in boot context |
| `tokenize/parse/analyze` | ✅ DONE | Bootstrap compiler pipeline |
| `_prefill_output` | ✅ DONE | Reset compiler state per file |

---

## Effort Estimate

| Component | LOC | Language |
|-----------|-----|---------|
| Raw byte builtins | ~80 | ASM (vm_x86_64.S) |
| Batch compile | ~60 | Olang (repl.ol or encoder.ol) |
| VM extraction | ~20 | Olang |
| Binary packing | ~50 | Olang |
| Build command | ~10 | Olang (repl.ol) |
| **Total** | **~220 LOC** | ASM + Olang |

---

## Risk

| Risk | Severity | Mitigation |
|------|----------|------------|
| Heap exhaustion (63 files × compile) | HIGH | Reset heap between files. Or increase heap. |
| _g_output shared between compiles | MEDIUM | Call _prefill_output() between files |
| Parser errors for some .ol files | MEDIUM | Skip files with errors, compile what works |
| ELF not executable after pack | LOW | Copy VM binary verbatim, only append |
| Binary too large | LOW | Current = 902KB. Olang bytecode ~320KB. |

---

## Verification

```
1. build                        → writes origin_new.olang
2. chmod +x origin_new.olang
3. echo 'emit 42' | ./origin_new.olang   → 42
4. echo 'test' | ./origin_new.olang       → ALL PASS: 12/12
5. echo 'respond hello' | ./origin_new.olang → Minh nghe roi.
6. echo 'build' | ./origin_new.olang      → builds ANOTHER binary (self-host proof)
```

Step 6 = **origin.olang builds origin.olang which builds origin.olang**. Full circle.

---

*Khi CUT.4 xong: xóa `tools/builder/`, xóa `crates/`, xóa `Cargo.toml`. Chỉ còn:*
- `vm/x86_64/vm_x86_64.S` (ASM VM)
- `stdlib/**/*.ol` (Olang source)
- `origin_new.olang` (self-built binary)
- *Đồ điên.*
