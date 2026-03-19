/// Minimal ELF64 executable generator.
/// No libc, no dynamic linking, no section headers.
/// Just: ELF header (64 bytes) + 1 program header (56 bytes) = 120 bytes.

pub const ELF_HEADER_SIZE: usize = 64;
pub const PROGRAM_HEADER_SIZE: usize = 56;
pub const TOTAL_HEADER_SIZE: usize = ELF_HEADER_SIZE + PROGRAM_HEADER_SIZE;

/// Standard load address for Linux x86_64 static executables.
pub const LOAD_ADDR: u64 = 0x400000;

pub struct ElfConfig {
    /// Virtual address of _start entry point.
    pub entry_vaddr: u64,
    /// Total file size.
    pub file_size: u64,
    /// ELF e_machine value (0x3E for x86_64, 0xB7 for ARM64).
    pub e_machine: u16,
}

/// Generate a complete ELF64 header (64 + 56 = 120 bytes).
pub fn generate_elf64(config: &ElfConfig) -> [u8; TOTAL_HEADER_SIZE] {
    let mut buf = [0u8; TOTAL_HEADER_SIZE];
    let mut pos = 0;

    // ── ELF Header (64 bytes) ──────────────────────────────────────

    // e_ident: magic + class + data + version + OS/ABI + padding
    buf[0..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']); // magic
    buf[4] = 2;  // ELFCLASS64
    buf[5] = 1;  // ELFDATA2LSB (little endian)
    buf[6] = 1;  // EV_CURRENT
    buf[7] = 3;  // ELFOSABI_LINUX
    // bytes 8-15: padding (already 0)
    pos = 16;

    // e_type: ET_EXEC (2)
    write_u16(&mut buf, &mut pos, 2);
    // e_machine: EM_X86_64 (0x3E) or EM_AARCH64 (0xB7)
    write_u16(&mut buf, &mut pos, config.e_machine);
    // e_version: EV_CURRENT (1)
    write_u32(&mut buf, &mut pos, 1);
    // e_entry: entry point virtual address
    write_u64(&mut buf, &mut pos, config.entry_vaddr);
    // e_phoff: program header offset = 64 (right after ELF header)
    write_u64(&mut buf, &mut pos, ELF_HEADER_SIZE as u64);
    // e_shoff: section header offset = 0 (none)
    write_u64(&mut buf, &mut pos, 0);
    // e_flags: 0
    write_u32(&mut buf, &mut pos, 0);
    // e_ehsize: ELF header size = 64
    write_u16(&mut buf, &mut pos, ELF_HEADER_SIZE as u16);
    // e_phentsize: program header entry size = 56
    write_u16(&mut buf, &mut pos, PROGRAM_HEADER_SIZE as u16);
    // e_phnum: 1 program header
    write_u16(&mut buf, &mut pos, 1);
    // e_shentsize, e_shnum, e_shstrndx: all 0
    write_u16(&mut buf, &mut pos, 0);
    write_u16(&mut buf, &mut pos, 0);
    write_u16(&mut buf, &mut pos, 0);

    assert_eq!(pos, ELF_HEADER_SIZE);

    // ── Program Header (56 bytes) ──────────────────────────────────

    // p_type: PT_LOAD (1)
    write_u32(&mut buf, &mut pos, 1);
    // p_flags: PF_R | PF_W | PF_X (7) — read+write+execute
    write_u32(&mut buf, &mut pos, 7);
    // p_offset: 0 (load entire file)
    write_u64(&mut buf, &mut pos, 0);
    // p_vaddr: virtual address
    write_u64(&mut buf, &mut pos, LOAD_ADDR);
    // p_paddr: physical address (same)
    write_u64(&mut buf, &mut pos, LOAD_ADDR);
    // p_filesz: file size
    write_u64(&mut buf, &mut pos, config.file_size);
    // p_memsz: memory size (same as file)
    write_u64(&mut buf, &mut pos, config.file_size);
    // p_align: page alignment
    write_u64(&mut buf, &mut pos, 0x1000);

    assert_eq!(pos, TOTAL_HEADER_SIZE);

    buf
}

fn write_u16(buf: &mut [u8], pos: &mut usize, val: u16) {
    buf[*pos..*pos + 2].copy_from_slice(&val.to_le_bytes());
    *pos += 2;
}

fn write_u32(buf: &mut [u8], pos: &mut usize, val: u32) {
    buf[*pos..*pos + 4].copy_from_slice(&val.to_le_bytes());
    *pos += 4;
}

fn write_u64(buf: &mut [u8], pos: &mut usize, val: u64) {
    buf[*pos..*pos + 8].copy_from_slice(&val.to_le_bytes());
    *pos += 8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_header_size() {
        let header = generate_elf64(&ElfConfig {
            entry_vaddr: LOAD_ADDR + 152,
            file_size: 1024,
            e_machine: 0x3E,
        });
        assert_eq!(header.len(), 120);
        // ELF magic
        assert_eq!(&header[0..4], &[0x7f, b'E', b'L', b'F']);
        // Class = 64-bit
        assert_eq!(header[4], 2);
        // Machine = x86_64
        assert_eq!(header[18], 0x3E);
        assert_eq!(header[19], 0x00);
    }

    #[test]
    fn test_elf_entry_point() {
        let entry = LOAD_ADDR + 200;
        let header = generate_elf64(&ElfConfig {
            entry_vaddr: entry,
            file_size: 4096,
            e_machine: 0x3E,
        });
        let stored = u64::from_le_bytes(header[24..32].try_into().unwrap());
        assert_eq!(stored, entry);
    }

    #[test]
    fn test_elf_arm64() {
        let header = generate_elf64(&ElfConfig {
            entry_vaddr: LOAD_ADDR + 152,
            file_size: 1024,
            e_machine: 0xB7,
        });
        // Machine = ARM64
        assert_eq!(header[18], 0xB7);
        assert_eq!(header[19], 0x00);
    }
}
