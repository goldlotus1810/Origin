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
    pub arch: Arch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86_64,
    Arm64,
}

impl Arch {
    pub fn byte(self) -> u8 {
        match self {
            Arch::X86_64 => 0x01,
            Arch::Arm64 => 0x02,
        }
    }

    pub fn e_machine(self) -> u16 {
        match self {
            Arch::X86_64 => 0x3E,
            Arch::Arm64 => 0xB7,
        }
    }
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

    let origin_header = build_origin_header_arch(
        0, 0, // vm fields unused in wrap mode
        bc_offset, bc_size,
        kn_offset, kn_size,
        flags,
        config.arch.byte(),
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
    let origin_header = build_origin_header_arch(
        vm_offset, vm_size,
        bc_offset, bc_size,
        kn_offset, kn_size,
        flags,
        config.arch.byte(),
    );
    output[TOTAL_HEADER_SIZE..combined_header].copy_from_slice(&origin_header);

    // ELF header at offset 0
    let elf_header = elf::generate_elf64(&elf::ElfConfig {
        entry_vaddr: LOAD_ADDR + vm_offset as u64,
        file_size: output.len() as u64,
        e_machine: config.arch.e_machine(),
    });
    output[..TOTAL_HEADER_SIZE].copy_from_slice(&elf_header);

    output
}

// ── Fat binary format (planned: multi-arch support) ──

#[allow(dead_code)]
/// Fat header magic: same ○LNG but version 0x20
const FAT_VERSION: u8 = 0x20;
#[allow(dead_code)]
const FAT_HEADER_SIZE: usize = 64;

/// Per-arch entry in fat header (16 bytes)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FatArchEntry {
    pub arch_id: u8,
    pub vm_offset: u32,
    pub vm_size: u32,
    pub entry_offset: u32,
}

/// Parsed fat header
#[allow(dead_code)]
#[derive(Debug)]
pub struct FatHeader {
    pub arch_count: u8,
    pub archs: Vec<FatArchEntry>,
    pub bc_offset: u32,
    pub bc_size: u32,
    pub kn_offset: u32,
    pub kn_size: u32,
}

/// Build a fat binary containing multiple arch VMs + shared bytecode + knowledge.
#[allow(dead_code)]
pub fn pack_fat(
    vms: &[(Arch, &[u8], u32)], // (arch, vm_code, entry_offset)
    bytecode: &[u8],
    knowledge: &[u8],
) -> Vec<u8> {
    let mut output = Vec::new();

    // Calculate offsets: [header 64B][VM 0][VM 1]...[bytecode][knowledge]
    let mut offset = FAT_HEADER_SIZE as u32;
    let mut entries = Vec::new();
    for (arch, vm, entry_off) in vms {
        entries.push(FatArchEntry {
            arch_id: arch.byte(),
            vm_offset: offset,
            vm_size: vm.len() as u32,
            entry_offset: *entry_off,
        });
        offset += vm.len() as u32;
    }
    let bc_offset = offset;
    let kn_offset = bc_offset + bytecode.len() as u32;

    // Build header
    let header = build_fat_header(&entries, bc_offset, bytecode.len() as u32, kn_offset, knowledge.len() as u32);
    output.extend_from_slice(&header);

    // VM sections
    for (_arch, vm, _entry) in vms {
        output.extend_from_slice(vm);
    }

    // Shared sections
    output.extend_from_slice(bytecode);
    output.extend_from_slice(knowledge);

    output
}

#[allow(dead_code)]
fn build_fat_header(
    archs: &[FatArchEntry],
    bc_offset: u32, bc_size: u32,
    kn_offset: u32, kn_size: u32,
) -> [u8; FAT_HEADER_SIZE] {
    let mut h = [0u8; FAT_HEADER_SIZE];
    h[0..4].copy_from_slice(&ORIGIN_MAGIC);
    h[4] = FAT_VERSION;
    h[5] = archs.len() as u8;
    // flags at 6..8 = 0

    let mut off = 8;
    for entry in archs {
        h[off] = entry.arch_id;
        h[off+1..off+5].copy_from_slice(&entry.vm_offset.to_le_bytes());
        h[off+5..off+9].copy_from_slice(&entry.vm_size.to_le_bytes());
        h[off+9..off+13].copy_from_slice(&entry.entry_offset.to_le_bytes());
        // reserved 3 bytes = 0
        off += 16;
    }

    // Shared section offsets (after per-arch entries)
    h[off..off+4].copy_from_slice(&bc_offset.to_le_bytes());
    h[off+4..off+8].copy_from_slice(&bc_size.to_le_bytes());
    h[off+8..off+12].copy_from_slice(&kn_offset.to_le_bytes());
    h[off+12..off+16].copy_from_slice(&kn_size.to_le_bytes());

    h
}

/// Parse a fat binary header.
#[allow(dead_code)]
pub fn parse_fat_header(data: &[u8]) -> Option<FatHeader> {
    if data.len() < FAT_HEADER_SIZE { return None; }
    if &data[0..4] != ORIGIN_MAGIC { return None; }
    if data[4] != FAT_VERSION { return None; }

    let arch_count = data[5];
    let mut archs = Vec::new();
    let mut off = 8;
    for _ in 0..arch_count {
        archs.push(FatArchEntry {
            arch_id: data[off],
            vm_offset: u32::from_le_bytes(data[off+1..off+5].try_into().ok()?),
            vm_size: u32::from_le_bytes(data[off+5..off+9].try_into().ok()?),
            entry_offset: u32::from_le_bytes(data[off+9..off+13].try_into().ok()?),
        });
        off += 16;
    }

    let bc_offset = u32::from_le_bytes(data[off..off+4].try_into().ok()?);
    let bc_size = u32::from_le_bytes(data[off+4..off+8].try_into().ok()?);
    let kn_offset = u32::from_le_bytes(data[off+8..off+12].try_into().ok()?);
    let kn_size = u32::from_le_bytes(data[off+12..off+16].try_into().ok()?);

    Some(FatHeader {
        arch_count,
        archs,
        bc_offset,
        bc_size,
        kn_offset,
        kn_size,
    })
}

fn build_origin_header_arch(
    vm_offset: u32, vm_size: u32,
    bc_offset: u32, bc_size: u32,
    kn_offset: u32, kn_size: u32,
    flags: u16,
    arch_byte: u8,
) -> [u8; ORIGIN_HEADER_SIZE] {
    let mut h = [0u8; ORIGIN_HEADER_SIZE];
    h[0..4].copy_from_slice(&ORIGIN_MAGIC);
    h[4] = 0x10; // version: self-exec
    h[5] = arch_byte; // arch: 0x01=x86_64, 0x02=arm64
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
            arch: Arch::X86_64,
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
            arch: Arch::X86_64,
        });

        // Starts with the original ELF
        assert_eq!(&binary[0..4], &[0x7f, b'E', b'L', b'F']);
        // Origin header follows the ELF
        let hdr_off = fake_elf.len();
        assert_eq!(&binary[hdr_off..hdr_off+4], &ORIGIN_MAGIC);
    }

    #[test]
    fn test_fat_binary_roundtrip() {
        let vm_x86 = b"x86_64 VM code here";
        let vm_arm = b"ARM64 VM code";
        let bytecode = b"shared bytecode";
        let knowledge = b"shared knowledge";

        let fat = pack_fat(
            &[(Arch::X86_64, vm_x86.as_slice(), 0), (Arch::Arm64, vm_arm.as_slice(), 0)],
            bytecode, knowledge,
        );

        // Parse header
        let hdr = parse_fat_header(&fat).expect("should parse fat header");
        assert_eq!(hdr.arch_count, 2);
        assert_eq!(hdr.archs[0].arch_id, 0x01); // x86_64
        assert_eq!(hdr.archs[1].arch_id, 0x02); // arm64

        // Verify VM extraction
        let x86_start = hdr.archs[0].vm_offset as usize;
        let x86_end = x86_start + hdr.archs[0].vm_size as usize;
        assert_eq!(&fat[x86_start..x86_end], vm_x86);

        let arm_start = hdr.archs[1].vm_offset as usize;
        let arm_end = arm_start + hdr.archs[1].vm_size as usize;
        assert_eq!(&fat[arm_start..arm_end], vm_arm);

        // Verify shared sections
        let bc_start = hdr.bc_offset as usize;
        assert_eq!(&fat[bc_start..bc_start + hdr.bc_size as usize], bytecode);
        let kn_start = hdr.kn_offset as usize;
        assert_eq!(&fat[kn_start..kn_start + hdr.kn_size as usize], knowledge);
    }

    #[test]
    fn test_fat_header_magic_and_version() {
        let fat = pack_fat(
            &[(Arch::X86_64, b"vm", 42)],
            b"bc", b"kn",
        );
        assert_eq!(&fat[0..4], &ORIGIN_MAGIC);
        assert_eq!(fat[4], FAT_VERSION); // 0x20
        assert_eq!(fat[5], 1);           // 1 arch
    }

    #[test]
    fn test_fat_entry_offset_preserved() {
        let fat = pack_fat(
            &[(Arch::X86_64, b"vm_code", 123)],
            b"", b"",
        );
        let hdr = parse_fat_header(&fat).unwrap();
        assert_eq!(hdr.archs[0].entry_offset, 123);
    }

    #[test]
    fn test_fat_shared_bytecode_offset() {
        let vm = b"0123456789"; // 10 bytes
        let fat = pack_fat(
            &[(Arch::X86_64, vm.as_slice(), 0)],
            b"BYTECODE", b"",
        );
        let hdr = parse_fat_header(&fat).unwrap();
        // bytecode should start at 64 (header) + 10 (vm) = 74
        assert_eq!(hdr.bc_offset, 74);
        assert_eq!(hdr.bc_size, 8);
    }

    #[test]
    fn test_codegen_flag() {
        let binary = pack(&PackConfig {
            vm_code: b"vm", bytecode: b"bc", knowledge: b"",
            codegen_format: true, is_linked_elf: false,
            arch: Arch::X86_64,
        });
        // flags at origin header offset 120+30 = 150
        let flags = u16::from_le_bytes(binary[150..152].try_into().unwrap());
        assert_eq!(flags, 1);
    }
}
