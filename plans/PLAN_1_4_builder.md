# PLAN 1.4 — Builder tool (Rust — lần cuối dùng Rust cho tool mới)

**Phụ thuộc:** PLAN_0_6 (bytecode format) + PLAN_1_1 (vm_x86_64.o)
**Mục tiêu:** Pack VM code + bytecode + knowledge → 1 file origin.olang tự chạy.
**Yêu cầu:** Biết Rust. Hiểu ELF format cơ bản.

---

## Bối cảnh

Builder = tool Rust cuối cùng. Nó lấy 3 thành phần → ghép thành 1 executable:

```
INPUT:
  vm_x86_64.o           ← machine code VM (từ PLAN_1_1)
  *.ol bytecode          ← compiled Olang (từ PLAN_0_5)
  origin.olang           ← knowledge data (existing)

OUTPUT:
  origin.olang           ← ELF executable, tự chạy
  (chmod +x → ./origin.olang)
```

---

## Crate mới: `tools/builder/`

### File structure

```
tools/builder/
├── Cargo.toml
└── src/
    ├── main.rs          ← CLI entry: parse args, orchestrate
    ├── elf.rs           ← Generate ELF headers
    ├── pack.rs          ← Pack sections into origin.olang format
    └── compile.rs       ← Compile .ol files → bytecode
```

### Cargo.toml

```toml
[package]
name = "builder"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "builder"
path = "src/main.rs"

[dependencies]
olang = { path = "../../crates/olang" }
```

---

## Việc cần làm

### Task 1: CLI (main.rs, ~100 LOC)

```rust
fn main() {
    let args = parse_args();
    // args:
    //   --vm vm/x86_64/vm_x86_64.o     machine code object
    //   --stdlib stdlib/                  .ol files to compile
    //   --knowledge origin.olang          existing knowledge
    //   --output origin.olang             output file
    //   --arch x86_64|arm64|wasm         target architecture

    // 1. Compile all .ol → bytecode
    let bytecode = compile::compile_all(&args.stdlib_path)?;

    // 2. Read VM object code
    let vm_code = extract_text_section(&args.vm_path)?;

    // 3. Read existing knowledge
    let knowledge = std::fs::read(&args.knowledge_path)?;

    // 4. Pack everything
    let binary = pack::pack(PackConfig {
        arch: args.arch,
        vm_code: &vm_code,
        bytecode: &bytecode,
        knowledge: &knowledge,
    })?;

    // 5. Write output
    std::fs::write(&args.output, &binary)?;

    // 6. Make executable (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&args.output,
            std::fs::Permissions::from_mode(0o755))?;
    }

    println!("Built: {} ({} bytes)", args.output, binary.len());
}
```

### Task 2: ELF generator (elf.rs, ~200 LOC)

```rust
/// Generate minimal ELF64 executable header.
/// No libc, no dynamic linking, no sections table.
/// Just: ELF header + 1 program header + payload.

pub struct ElfConfig {
    pub entry_offset: u64,      // offset of _start in file
    pub load_addr: u64,         // virtual address (0x400000 standard)
    pub file_size: u64,
}

pub fn generate_elf_header(config: &ElfConfig) -> Vec<u8> {
    let mut header = Vec::with_capacity(120);

    // ELF magic
    header.extend_from_slice(&[0x7f, b'E', b'L', b'F']);

    // ELF64, little-endian, current version, Linux ABI
    header.extend_from_slice(&[2, 1, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0]);

    // Type: executable
    header.extend_from_slice(&2u16.to_le_bytes());

    // Machine: x86_64
    header.extend_from_slice(&0x3Eu16.to_le_bytes());

    // Version
    header.extend_from_slice(&1u32.to_le_bytes());

    // Entry point
    header.extend_from_slice(&config.entry_offset.to_le_bytes());

    // Program header offset (immediately after ELF header = 64)
    header.extend_from_slice(&64u64.to_le_bytes());

    // Section header offset (0 = none)
    header.extend_from_slice(&0u64.to_le_bytes());

    // Flags
    header.extend_from_slice(&0u32.to_le_bytes());

    // ELF header size = 64
    header.extend_from_slice(&64u16.to_le_bytes());

    // Program header entry size = 56
    header.extend_from_slice(&56u16.to_le_bytes());

    // Number of program headers = 1
    header.extend_from_slice(&1u16.to_le_bytes());

    // Section header entry size, count, string table index (all 0)
    header.extend_from_slice(&0u16.to_le_bytes());
    header.extend_from_slice(&0u16.to_le_bytes());
    header.extend_from_slice(&0u16.to_le_bytes());

    // Program header (LOAD: map entire file into memory)
    // Type = PT_LOAD (1)
    header.extend_from_slice(&1u32.to_le_bytes());
    // Flags = PF_R | PF_W | PF_X (7)
    header.extend_from_slice(&7u32.to_le_bytes());
    // Offset = 0 (load from start of file)
    header.extend_from_slice(&0u64.to_le_bytes());
    // Virtual address
    header.extend_from_slice(&config.load_addr.to_le_bytes());
    // Physical address (same)
    header.extend_from_slice(&config.load_addr.to_le_bytes());
    // File size
    header.extend_from_slice(&config.file_size.to_le_bytes());
    // Memory size (same as file)
    header.extend_from_slice(&config.file_size.to_le_bytes());
    // Alignment
    header.extend_from_slice(&0x1000u64.to_le_bytes());

    header // 120 bytes total (64 ELF + 56 program header)
}
```

### Task 3: Packer (pack.rs, ~150 LOC)

```rust
pub struct PackConfig<'a> {
    pub arch: Arch,
    pub vm_code: &'a [u8],
    pub bytecode: &'a [u8],
    pub knowledge: &'a [u8],
}

pub fn pack(config: PackConfig) -> Vec<u8> {
    let mut output = Vec::new();

    // 1. Reserve space for ELF header (120 bytes) + origin header (32 bytes)
    let header_size = 120 + 32;
    output.resize(header_size, 0);

    // 2. Append VM code
    let vm_offset = output.len() as u32;
    let vm_size = config.vm_code.len() as u32;
    output.extend_from_slice(config.vm_code);

    // 3. Append bytecode
    let bc_offset = output.len() as u32;
    let bc_size = config.bytecode.len() as u32;
    output.extend_from_slice(config.bytecode);

    // 4. Append knowledge
    let kn_offset = output.len() as u32;
    let kn_size = config.knowledge.len() as u32;
    output.extend_from_slice(config.knowledge);

    // 5. Write origin header at offset 120
    let origin_header = build_origin_header(
        vm_offset, vm_size, bc_offset, bc_size, kn_offset, kn_size,
    );
    output[120..152].copy_from_slice(&origin_header);

    // 6. Write ELF header at offset 0
    let elf_header = generate_elf_header(&ElfConfig {
        entry_offset: (120 + 32 + vm_offset) as u64, // _start in VM code
        load_addr: 0x400000,
        file_size: output.len() as u64,
    });
    output[..120].copy_from_slice(&elf_header);

    output
}

fn build_origin_header(
    vm_offset: u32, vm_size: u32,
    bc_offset: u32, bc_size: u32,
    kn_offset: u32, kn_size: u32,
) -> [u8; 32] {
    let mut h = [0u8; 32];
    // Magic: ○LNG
    h[0..4].copy_from_slice(&[0xE2, 0x97, 0x8B, 0x4C]);
    // Version: 0x10 (self-exec)
    h[4] = 0x10;
    // Arch
    h[5] = 0x01; // x86_64
    // Offsets
    h[6..10].copy_from_slice(&vm_offset.to_le_bytes());
    h[10..14].copy_from_slice(&vm_size.to_le_bytes());
    h[14..18].copy_from_slice(&bc_offset.to_le_bytes());
    h[18..22].copy_from_slice(&bc_size.to_le_bytes());
    h[22..26].copy_from_slice(&kn_offset.to_le_bytes());
    h[26..30].copy_from_slice(&kn_size.to_le_bytes());
    // Flags: 0
    h[30..32].copy_from_slice(&0u16.to_le_bytes());
    h
}
```

### Task 4: Compile .ol files (compile.rs, ~100 LOC)

```rust
use olang::syntax::Parser;
use olang::semantic::compile_program;
use olang::bytecode::encode;

pub fn compile_all(stdlib_path: &str) -> Result<Vec<u8>, CompileError> {
    let mut all_bytecode = Vec::new();

    // Compile bootstrap
    for entry in std::fs::read_dir(format!("{}/bootstrap", stdlib_path))? {
        let path = entry?.path();
        if path.extension() == Some("ol".as_ref()) {
            let source = std::fs::read_to_string(&path)?;
            let ast = Parser::new(&source).parse_program()?;
            let program = compile_program(&ast)?;
            let bytecode = encode(&program.ops);
            all_bytecode.extend_from_slice(&bytecode);
        }
    }

    // Compile stdlib
    for entry in std::fs::read_dir(stdlib_path)? {
        let path = entry?.path();
        if path.extension() == Some("ol".as_ref()) {
            let source = std::fs::read_to_string(&path)?;
            let ast = Parser::new(&source).parse_program()?;
            let program = compile_program(&ast)?;
            let bytecode = encode(&program.ops);
            all_bytecode.extend_from_slice(&bytecode);
        }
    }

    Ok(all_bytecode)
}
```

---

## Usage

```bash
# 1. Assemble VM
as -o vm_x86_64.o vm/x86_64/vm_x86_64.S

# 2. Build origin.olang
cargo run -p builder -- \
    --vm vm/x86_64/vm_x86_64.o \
    --stdlib stdlib/ \
    --knowledge origin.olang \
    --output origin_new.olang \
    --arch x86_64

# 3. Test
./origin_new.olang
# → REPL starts, no Rust runtime needed
```

---

## Test plan

```rust
#[test]
fn test_builder_creates_valid_elf() {
    let binary = pack(/* minimal config */);
    // Check ELF magic
    assert_eq!(&binary[0..4], &[0x7f, b'E', b'L', b'F']);
    // Check origin magic
    assert_eq!(&binary[120..124], &[0xE2, 0x97, 0x8B, 0x4C]);
}

#[test]
fn test_builder_roundtrip() {
    // Build → read back → verify sections
    let binary = pack(/* config */);
    let header = parse_origin_header(&binary[120..152]);
    assert!(header.vm_size > 0);
    assert!(header.bc_size > 0);
}
```

## Definition of Done

- [ ] `cargo build -p builder` compiles
- [ ] `builder --vm ... --output out.olang` produces file
- [ ] Output file is valid ELF (readelf -h works)
- [ ] Output file contains origin header at offset 120
- [ ] `chmod +x out.olang && ./out.olang` → VM starts (even if minimal)

## Ước tính: 1 tuần

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 1.4*
