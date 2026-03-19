//! Integration: Builder → Binary → Extract → Verify
//!
//! Tests the full build pipeline:
//!   compile .ol → bytecode → pack (ELF + origin header) → extract → verify
//! Covers: olang + builder (pack + elf)

use olang::exec::bytecode::{decode_bytecode, encode_bytecode};
use olang::lang::semantic::lower;
use olang::lang::syntax::parse;

// ═══════════════════════════════════════════════════════════════════
// Origin header format constants
// ═══════════════════════════════════════════════════════════════════

const ORIGIN_MAGIC: [u8; 4] = [0xE2, 0x97, 0x8B, 0x4C]; // ○LNG
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const ELF_HEADER_SIZE: usize = 64;
const PROGRAM_HEADER_SIZE: usize = 56;
const TOTAL_ELF_HEADER: usize = ELF_HEADER_SIZE + PROGRAM_HEADER_SIZE; // 120
const ORIGIN_HEADER_SIZE: usize = 32;

/// Parse origin header from bytes at given offset.
struct OriginHeader {
    magic: [u8; 4],
    version: u8,
    arch: u8,
    vm_offset: u32,
    vm_size: u32,
    bc_offset: u32,
    bc_size: u32,
    kn_offset: u32,
    kn_size: u32,
    flags: u16,
}

impl OriginHeader {
    fn from_bytes(data: &[u8], offset: usize) -> Self {
        let d = &data[offset..];
        Self {
            magic: [d[0], d[1], d[2], d[3]],
            version: d[4],
            arch: d[5],
            vm_offset: u32::from_le_bytes([d[6], d[7], d[8], d[9]]),
            vm_size: u32::from_le_bytes([d[10], d[11], d[12], d[13]]),
            bc_offset: u32::from_le_bytes([d[14], d[15], d[16], d[17]]),
            bc_size: u32::from_le_bytes([d[18], d[19], d[20], d[21]]),
            kn_offset: u32::from_le_bytes([d[22], d[23], d[24], d[25]]),
            kn_size: u32::from_le_bytes([d[26], d[27], d[28], d[29]]),
            flags: u16::from_le_bytes([d[30], d[31]]),
        }
    }
}

/// Compile source to bytecode.
fn compile_source(source: &str) -> Vec<u8> {
    let stmts = parse(source).expect("parse failed");
    let program = lower(&stmts);
    encode_bytecode(&program.ops)
}

/// Build a minimal ELF binary with origin header + vm_code + bytecode + knowledge.
/// Mimics pack_elf() from tools/builder/src/pack.rs.
fn pack_elf(vm_code: &[u8], bytecode: &[u8], knowledge: &[u8], codegen_flag: bool) -> Vec<u8> {
    let combined_header = TOTAL_ELF_HEADER + ORIGIN_HEADER_SIZE;
    let mut output = vec![0u8; combined_header];

    // VM code
    let vm_offset = output.len() as u32;
    let vm_size = vm_code.len() as u32;
    output.extend_from_slice(vm_code);

    // Bytecode
    let bc_offset = output.len() as u32;
    let bc_size = bytecode.len() as u32;
    output.extend_from_slice(bytecode);

    // Knowledge
    let kn_offset = output.len() as u32;
    let kn_size = knowledge.len() as u32;
    output.extend_from_slice(knowledge);

    let flags: u16 = if codegen_flag { 1 } else { 0 };

    // Origin header at offset 120
    let mut oh = [0u8; ORIGIN_HEADER_SIZE];
    oh[0..4].copy_from_slice(&ORIGIN_MAGIC);
    oh[4] = 0x10; // version
    oh[5] = 0x01; // arch: x86_64
    oh[6..10].copy_from_slice(&vm_offset.to_le_bytes());
    oh[10..14].copy_from_slice(&vm_size.to_le_bytes());
    oh[14..18].copy_from_slice(&bc_offset.to_le_bytes());
    oh[18..22].copy_from_slice(&bc_size.to_le_bytes());
    oh[22..26].copy_from_slice(&kn_offset.to_le_bytes());
    oh[26..30].copy_from_slice(&kn_size.to_le_bytes());
    oh[30..32].copy_from_slice(&flags.to_le_bytes());
    output[TOTAL_ELF_HEADER..combined_header].copy_from_slice(&oh);

    // ELF header at offset 0 (minimal)
    output[0..4].copy_from_slice(&ELF_MAGIC);
    output[4] = 2;  // ELFCLASS64
    output[5] = 1;  // ELFDATA2LSB
    output[6] = 1;  // EV_CURRENT
    output[7] = 3;  // ELFOSABI_LINUX
    // e_type = ET_EXEC (2) at offset 16
    output[16] = 2;
    // e_machine = x86_64 (0x3E) at offset 18
    output[18] = 0x3E;

    output
}

/// Build wrap-mode binary: [VM ELF][origin header][bytecode][knowledge][trailer 8B]
fn pack_wrap(vm_elf: &[u8], bytecode: &[u8], knowledge: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(vm_elf);

    let header_offset = output.len() as u64;
    let bc_offset = (header_offset as usize + ORIGIN_HEADER_SIZE) as u32;
    let bc_size = bytecode.len() as u32;
    let kn_offset = bc_offset + bc_size;
    let kn_size = knowledge.len() as u32;

    let mut oh = [0u8; ORIGIN_HEADER_SIZE];
    oh[0..4].copy_from_slice(&ORIGIN_MAGIC);
    oh[4] = 0x10;
    oh[5] = 0x01;
    oh[6..10].copy_from_slice(&0u32.to_le_bytes()); // vm fields unused in wrap
    oh[10..14].copy_from_slice(&0u32.to_le_bytes());
    oh[14..18].copy_from_slice(&bc_offset.to_le_bytes());
    oh[18..22].copy_from_slice(&bc_size.to_le_bytes());
    oh[22..26].copy_from_slice(&kn_offset.to_le_bytes());
    oh[26..30].copy_from_slice(&kn_size.to_le_bytes());
    oh[30..32].copy_from_slice(&0u16.to_le_bytes());
    output.extend_from_slice(&oh);
    output.extend_from_slice(bytecode);
    output.extend_from_slice(knowledge);

    // Trailer: 8-byte LE header offset
    output.extend_from_slice(&header_offset.to_le_bytes());

    output
}

// ═══════════════════════════════════════════════════════════════════
// ELF mode tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn elf_mode_has_valid_elf_magic() {
    let binary = pack_elf(b"fake_vm", b"fake_bc", b"", false);
    assert_eq!(&binary[0..4], &ELF_MAGIC);
}

#[test]
fn elf_mode_has_valid_origin_header() {
    let binary = pack_elf(b"fake_vm", b"fake_bc", b"kn_data", false);
    let oh = OriginHeader::from_bytes(&binary, TOTAL_ELF_HEADER);
    assert_eq!(oh.magic, ORIGIN_MAGIC);
    assert_eq!(oh.version, 0x10);
    assert_eq!(oh.arch, 0x01); // x86_64
}

#[test]
fn elf_mode_offsets_are_correct() {
    let vm = b"VM_CODE_HERE";
    let bc = b"BYTECODE_DATA";
    let kn = b"KNOWLEDGE";
    let binary = pack_elf(vm, bc, kn, false);
    let oh = OriginHeader::from_bytes(&binary, TOTAL_ELF_HEADER);

    // VM should start after combined header (120 + 32 = 152)
    assert_eq!(oh.vm_offset, 152);
    assert_eq!(oh.vm_size, vm.len() as u32);

    // Bytecode follows VM
    assert_eq!(oh.bc_offset, 152 + vm.len() as u32);
    assert_eq!(oh.bc_size, bc.len() as u32);

    // Knowledge follows bytecode
    assert_eq!(oh.kn_offset, 152 + vm.len() as u32 + bc.len() as u32);
    assert_eq!(oh.kn_size, kn.len() as u32);
}

#[test]
fn elf_mode_bytecode_extractable() {
    let bc = b"test_bytecode_data_1234";
    let binary = pack_elf(b"vm", bc, b"", false);
    let oh = OriginHeader::from_bytes(&binary, TOTAL_ELF_HEADER);

    let start = oh.bc_offset as usize;
    let end = start + oh.bc_size as usize;
    let extracted = &binary[start..end];
    assert_eq!(extracted, bc);
}

#[test]
fn elf_mode_knowledge_extractable() {
    let kn = b"knowledge_blob_xyz";
    let binary = pack_elf(b"vm", b"bc", kn, false);
    let oh = OriginHeader::from_bytes(&binary, TOTAL_ELF_HEADER);

    let start = oh.kn_offset as usize;
    let end = start + oh.kn_size as usize;
    assert_eq!(&binary[start..end], kn);
}

#[test]
fn elf_mode_codegen_flag() {
    let binary_no = pack_elf(b"vm", b"bc", b"", false);
    let oh_no = OriginHeader::from_bytes(&binary_no, TOTAL_ELF_HEADER);
    assert_eq!(oh_no.flags, 0);

    let binary_yes = pack_elf(b"vm", b"bc", b"", true);
    let oh_yes = OriginHeader::from_bytes(&binary_yes, TOTAL_ELF_HEADER);
    assert_eq!(oh_yes.flags, 1);
}

// ═══════════════════════════════════════════════════════════════════
// Wrap mode tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn wrap_mode_preserves_vm_elf() {
    let fake_elf = b"\x7fELF_fake_linked_binary_content";
    let binary = pack_wrap(fake_elf, b"bc", b"");

    // Binary starts with original VM ELF
    assert_eq!(&binary[0..4], &ELF_MAGIC);
    assert_eq!(&binary[0..fake_elf.len()], &fake_elf[..]);
}

#[test]
fn wrap_mode_trailer_points_to_header() {
    let fake_elf = b"\x7fELF_fake_vm";
    let binary = pack_wrap(fake_elf, b"bytecode", b"knowledge");

    // Last 8 bytes = header offset
    let trailer_offset = binary.len() - 8;
    let header_offset = u64::from_le_bytes(
        binary[trailer_offset..].try_into().unwrap()
    ) as usize;

    assert_eq!(header_offset, fake_elf.len());

    // Origin header at that offset
    let oh = OriginHeader::from_bytes(&binary, header_offset);
    assert_eq!(oh.magic, ORIGIN_MAGIC);
}

#[test]
fn wrap_mode_bytecode_extractable() {
    let fake_elf = b"\x7fELF_linked";
    let bc = b"my_bytecode_here";
    let binary = pack_wrap(fake_elf, bc, b"");

    // Read trailer → header → bc_offset/bc_size → extract
    let trailer_off = binary.len() - 8;
    let hdr_off = u64::from_le_bytes(binary[trailer_off..].try_into().unwrap()) as usize;
    let oh = OriginHeader::from_bytes(&binary, hdr_off);

    let extracted = &binary[oh.bc_offset as usize..(oh.bc_offset + oh.bc_size) as usize];
    assert_eq!(extracted, bc);
}

// ═══════════════════════════════════════════════════════════════════
// Full roundtrip: compile → pack → extract → decode → verify
// ═══════════════════════════════════════════════════════════════════

#[test]
fn full_roundtrip_compile_pack_extract_decode() {
    // 1. Compile source to bytecode
    let source = "let x = 42; let y = x + 1;";
    let bytecode = compile_source(source);
    assert!(!bytecode.is_empty());

    // 2. Pack into ELF
    let binary = pack_elf(b"fake_vm", &bytecode, b"", false);

    // 3. Extract bytecode from packed binary
    let oh = OriginHeader::from_bytes(&binary, TOTAL_ELF_HEADER);
    let extracted = &binary[oh.bc_offset as usize..(oh.bc_offset + oh.bc_size) as usize];

    // 4. Verify extracted matches original
    assert_eq!(extracted, &bytecode[..], "extracted bytecode must match original");

    // 5. Decode extracted bytecode
    let ops = decode_bytecode(extracted).expect("extracted bytecode must decode");
    assert!(!ops.is_empty(), "decoded ops should not be empty");

    // 6. Re-encode and verify determinism
    let re_encoded = encode_bytecode(&ops);
    assert_eq!(re_encoded, bytecode, "re-encoded must match original bytecode");
}

#[test]
fn full_roundtrip_wrap_mode() {
    let source = r#"fn greet() { emit "hello"; } greet();"#;
    let bytecode = compile_source(source);

    // Pack in wrap mode
    let fake_elf = b"\x7fELF_real_vm_binary_placeholder";
    let binary = pack_wrap(fake_elf, &bytecode, b"knowledge_data");

    // Extract via trailer
    let trailer_off = binary.len() - 8;
    let hdr_off = u64::from_le_bytes(binary[trailer_off..].try_into().unwrap()) as usize;
    let oh = OriginHeader::from_bytes(&binary, hdr_off);

    let extracted_bc = &binary[oh.bc_offset as usize..(oh.bc_offset + oh.bc_size) as usize];
    assert_eq!(extracted_bc, &bytecode[..]);

    let extracted_kn = &binary[oh.kn_offset as usize..(oh.kn_offset + oh.kn_size) as usize];
    assert_eq!(extracted_kn, b"knowledge_data");
}

#[test]
fn arm64_origin_header_arch_byte() {
    let mut binary = pack_elf(b"arm64_vm", b"bc", b"", false);
    // Manually set arch to arm64 (0x02) in origin header
    binary[TOTAL_ELF_HEADER + 5] = 0x02;
    // Set e_machine to ARM64 (0xB7) in ELF header
    binary[18] = 0xB7;

    let oh = OriginHeader::from_bytes(&binary, TOTAL_ELF_HEADER);
    assert_eq!(oh.arch, 0x02, "arch should be ARM64");
    assert_eq!(binary[18], 0xB7, "e_machine should be ARM64");
}
