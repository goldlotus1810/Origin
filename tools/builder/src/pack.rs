/// Pack VM code + bytecode + knowledge → single origin.olang binary.
///
/// Two modes:
/// 1. ELF mode: vm_code is a flat binary, output is ELF with embedded bytecode
/// 2. Wrap mode: vm_code is a complete linked ELF, we append bytecode after it
///    and patch the origin header inside
///
/// Current implementation uses wrap mode for simplicity.

use crate::elf::{self, LOAD_ADDR, TOTAL_HEADER_SIZE};

/// Origin header magic: "○LNG" (UTF-8: E2 97 8B 4C)
const ORIGIN_MAGIC: [u8; 4] = [0xE2, 0x97, 0x8B, 0x4C];
const ORIGIN_HEADER_SIZE: usize = 32;

pub struct PackConfig<'a> {
    pub vm_code: &'a [u8],
    pub bytecode: &'a [u8],
    pub knowledge: &'a [u8],
    pub codegen_format: bool,
    pub is_linked_elf: bool,
}

pub fn pack(config: &PackConfig) -> Vec<u8> {
    if config.is_linked_elf {
        pack_wrap(config)
    } else {
        pack_elf(config)
    }
}

/// Wrap mode: take a linked ELF VM binary, create a new file that contains:
/// [VM ELF binary][origin header 32B][bytecode][knowledge][header_offset 8B]
/// The VM reads the last 8 bytes to find the origin header offset.
fn pack_wrap(config: &PackConfig) -> Vec<u8> {
    let mut output = Vec::new();

    // Start with the linked ELF VM binary
    output.extend_from_slice(config.vm_code);

    // Append origin header at current offset
    let header_offset = output.len() as u64;
    let bc_offset = (header_offset as usize + ORIGIN_HEADER_SIZE) as u32;
    let bc_size = config.bytecode.len() as u32;
    let kn_offset = bc_offset + bc_size;
    let kn_size = config.knowledge.len() as u32;
    let flags: u16 = if config.codegen_format { 1 } else { 0 };

    let origin_header = build_origin_header(
        0, 0, // vm fields unused in wrap mode
        bc_offset, bc_size,
        kn_offset, kn_size,
        flags,
    );
    output.extend_from_slice(&origin_header);
    output.extend_from_slice(config.bytecode);
    output.extend_from_slice(config.knowledge);

    // Trailer: 8-byte LE header_offset so VM can find the origin header
    output.extend_from_slice(&header_offset.to_le_bytes());

    output
}

/// ELF mode: vm_code is flat binary, generate fresh ELF around it.
fn pack_elf(config: &PackConfig) -> Vec<u8> {
    let combined_header = TOTAL_HEADER_SIZE + ORIGIN_HEADER_SIZE;
    let mut output = Vec::new();

    // Reserve ELF + origin headers
    output.resize(combined_header, 0);

    // VM code
    let vm_offset = output.len() as u32;
    let vm_size = config.vm_code.len() as u32;
    output.extend_from_slice(config.vm_code);

    // Bytecode
    let bc_offset = output.len() as u32;
    let bc_size = config.bytecode.len() as u32;
    output.extend_from_slice(config.bytecode);

    // Knowledge
    let kn_offset = output.len() as u32;
    let kn_size = config.knowledge.len() as u32;
    output.extend_from_slice(config.knowledge);

    // Origin header at offset 120
    let flags: u16 = if config.codegen_format { 1 } else { 0 };
    let origin_header = build_origin_header(
        vm_offset, vm_size,
        bc_offset, bc_size,
        kn_offset, kn_size,
        flags,
    );
    output[TOTAL_HEADER_SIZE..combined_header].copy_from_slice(&origin_header);

    // ELF header at offset 0
    let elf_header = elf::generate_elf64(&elf::ElfConfig {
        entry_vaddr: LOAD_ADDR + vm_offset as u64,
        file_size: output.len() as u64,
    });
    output[..TOTAL_HEADER_SIZE].copy_from_slice(&elf_header);

    output
}

fn build_origin_header(
    vm_offset: u32, vm_size: u32,
    bc_offset: u32, bc_size: u32,
    kn_offset: u32, kn_size: u32,
    flags: u16,
) -> [u8; ORIGIN_HEADER_SIZE] {
    let mut h = [0u8; ORIGIN_HEADER_SIZE];
    h[0..4].copy_from_slice(&ORIGIN_MAGIC);
    h[4] = 0x10; // version: self-exec
    h[5] = 0x01; // arch: x86_64
    h[6..10].copy_from_slice(&vm_offset.to_le_bytes());
    h[10..14].copy_from_slice(&vm_size.to_le_bytes());
    h[14..18].copy_from_slice(&bc_offset.to_le_bytes());
    h[18..22].copy_from_slice(&bc_size.to_le_bytes());
    h[22..26].copy_from_slice(&kn_offset.to_le_bytes());
    h[26..30].copy_from_slice(&kn_size.to_le_bytes());
    h[30..32].copy_from_slice(&flags.to_le_bytes());
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_elf_creates_valid_output() {
        let vm_code = b"fake VM code";
        let bytecode = b"fake bytecode";

        let binary = pack(&PackConfig {
            vm_code, bytecode, knowledge: b"",
            codegen_format: false, is_linked_elf: false,
        });

        // ELF magic
        assert_eq!(&binary[0..4], &[0x7f, b'E', b'L', b'F']);
        // Origin magic at 120
        assert_eq!(&binary[120..124], &ORIGIN_MAGIC);
    }

    #[test]
    fn test_pack_wrap_preserves_elf() {
        let fake_elf = b"\x7fELF_fake_linked_binary";
        let bytecode = b"test_bc";

        let binary = pack(&PackConfig {
            vm_code: fake_elf, bytecode, knowledge: b"",
            codegen_format: false, is_linked_elf: true,
        });

        // Starts with the original ELF
        assert_eq!(&binary[0..4], &[0x7f, b'E', b'L', b'F']);
        // Origin header follows the ELF
        let hdr_off = fake_elf.len();
        assert_eq!(&binary[hdr_off..hdr_off+4], &ORIGIN_MAGIC);
    }

    #[test]
    fn test_codegen_flag() {
        let binary = pack(&PackConfig {
            vm_code: b"vm", bytecode: b"bc", knowledge: b"",
            codegen_format: true, is_linked_elf: false,
        });
        // flags at origin header offset 120+30 = 150
        let flags = u16::from_le_bytes(binary[150..152].try_into().unwrap());
        assert_eq!(flags, 1);
    }
}
