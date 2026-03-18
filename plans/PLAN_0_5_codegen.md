# PLAN 0.5 — Viết codegen.ol (~400 LOC)

**Phụ thuộc:** PLAN_0_4 phải xong (semantic.ol chạy được)
**Mục tiêu:** Emit binary bytecode từ IR opcodes — thay vì C/Rust/WASM text.
**Yêu cầu:** Biết Rust. Hiểu binary encoding cơ bản.

---

## Bối cảnh

### Hiện tại (Rust compiler)

`crates/olang/src/exec/compiler.rs` emit TEXT:
- Target::C → C source code (string)
- Target::Rust → Rust source code (string)
- Target::WASM → WAT text format (string)

### Cần thêm: Target::Bytecode

codegen.ol emit BINARY opcodes mà VM đọc trực tiếp, không qua text.

```
semantic.ol output: Vec[Op]   (abstract IR)
    ↓ codegen.ol
Binary bytecode: Vec[u8]      (compact, VM-ready)
```

---

## Bytecode format

```
Mỗi opcode = 1 byte tag + optional payload

Tag    Op              Payload
────────────────────────────────────────────────
0x01   Push            [chain_len:2][chain_bytes:N]
0x02   Load            [name_len:1][name:N]
0x03   Lca             (none)
0x04   Edge            [rel:1]
0x05   Query           [rel:1]
0x06   Emit            (none)
0x07   Call            [name_len:1][name:N]
0x08   Ret             (none)
0x09   Jmp             [target:4]
0x0A   Jz              [target:4]
0x0B   Dup             (none)
0x0C   Pop             (none)
0x0D   Swap            (none)
0x0E   Loop            [count:4]
0x0F   Halt            (none)
0x10   Dream           (none)
0x11   Stats           (none)
0x12   Nop             (none)
0x13   Store           [name_len:1][name:N]
0x14   LoadLocal       [name_len:1][name:N]
0x15   PushNum         [f64:8]          (IEEE 754)
0x16   Fuse            (none)
0x17   ScopeBegin      (none)
0x18   ScopeEnd        (none)
0x19   PushMol         [s:1][r:1][v:1][a:1][t:1]
0x1A   TryBegin        [catch_pc:4]
0x1B   CatchEnd        (none)
0x1C   StoreUpdate     [name_len:1][name:N]
0x1D   Trace           (none)
0x1E   Inspect         (none)
0x1F   Assert          (none)
0x20   TypeOf          (none)
0x21   Why             (none)
0x22   Explain         (none)
0x23   Ffi             [name_len:1][name:N][arity:1]
0x24   FileRead        (none)
0x25   FileWrite       (none)
0x26   FileAppend      (none)
```

---

## Việc cần làm

### Task 1: Bytecode encoder (~200 LOC)

```olang
// codegen.ol

fn encode_op(op) {
    match op {
        Op::Push(chain) => {
            let bytes = chain_to_bytes(chain);
            return concat([0x01], u16_le(len(bytes)), bytes);
        },
        Op::Load(name) => {
            return concat([0x02], [len(name)], str_to_bytes(name));
        },
        Op::Lca => { return [0x03]; },
        Op::PushNum(n) => {
            return concat([0x15], f64_le(n));
        },
        Op::Jmp(target) => {
            return concat([0x09], u32_le(target));
        },
        Op::Jz(target) => {
            return concat([0x0A], u32_le(target));
        },
        // ... etc cho tất cả 36+ opcodes
    };
}

pub fn generate(ops) {
    let output = [];
    for op in ops {
        let bytes = encode_op(op);
        push_all(output, bytes);
    };
    return output;
}
```

### Task 2: Bytecode decoder (cho VM — Rust side, ~200 LOC)

**File:** `crates/olang/src/exec/bytecode.rs` (MỚI)

```rust
/// Decode binary bytecode → Vec<Op>
pub fn decode_bytecode(bytes: &[u8]) -> Result<Vec<Op>, DecodeError> {
    let mut ops = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let tag = bytes[pos];
        pos += 1;
        match tag {
            0x01 => { // Push
                let len = u16::from_le_bytes([bytes[pos], bytes[pos+1]]) as usize;
                pos += 2;
                let chain = MolecularChain::from_bytes(&bytes[pos..pos+len])?;
                pos += len;
                ops.push(Op::Push(chain));
            },
            0x02 => { // Load
                let name_len = bytes[pos] as usize;
                pos += 1;
                let name = String::from_utf8(bytes[pos..pos+name_len].to_vec())?;
                pos += name_len;
                ops.push(Op::Load(name));
            },
            0x03 => ops.push(Op::Lca),
            0x15 => { // PushNum
                let n = f64::from_le_bytes(bytes[pos..pos+8].try_into()?);
                pos += 8;
                ops.push(Op::PushNum(n));
            },
            // ... etc
            _ => return Err(DecodeError::UnknownOpcode(tag)),
        }
    }
    Ok(ops)
}
```

### Task 3: Round-trip test

```rust
#[test]
fn test_bytecode_roundtrip() {
    let ops = vec![
        Op::PushNum(42.0),
        Op::Store("x".into()),
        Op::LoadLocal("x".into()),
        Op::Emit,
        Op::Halt,
    ];

    // Encode (dùng codegen.ol trên VM)
    let bytes = run_codegen_ol(&ops);

    // Decode (dùng Rust decoder)
    let decoded = decode_bytecode(&bytes).unwrap();

    assert_eq!(ops, decoded);
}
```

---

## Rào cản

| Rào cản | Giải pháp |
|---------|-----------|
| codegen.ol cần byte-level operations | Cần `bytes.ol` stdlib — ĐÃ CÓ trong stdlib/ |
| Endianness | Luôn dùng little-endian (x86 native) |
| Chain serialization | Dùng `to_tagged_bytes()` từ molecular.rs |

## Definition of Done

- [ ] `stdlib/bootstrap/codegen.ol` tồn tại (~400 LOC)
- [ ] `crates/olang/src/exec/bytecode.rs` — Rust decoder
- [ ] Round-trip: encode → decode → same ops
- [ ] codegen.ol(semantic.ol(parse(tokenize("let x = 42;")))) → valid bytecode
- [ ] VM đọc bytecode trực tiếp (thay vì Vec<Op>)

## Ước tính: 2-3 ngày

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 0.5*
