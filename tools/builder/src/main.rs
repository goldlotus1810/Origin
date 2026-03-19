//! # builder — Pack VM + bytecode + knowledge → origin.olang executable
//!
//! Usage:
//!   builder --vm vm/x86_64/vm_x86_64.bin \
//!           --stdlib stdlib/ \
//!           --knowledge origin.olang \
//!           --output origin_new.olang
//!
//! The output file is a valid ELF64 executable (Linux x86_64, no libc).

mod elf;
mod pack;
mod compile;

use std::process;

struct Args {
    vm_path: String,
    stdlib_path: Option<String>,
    bytecode_path: Option<String>,
    knowledge_path: Option<String>,
    output: String,
    codegen_format: bool,
    wrap_mode: bool,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut vm_path = String::new();
    let mut stdlib_path = None;
    let mut bytecode_path = None;
    let mut knowledge_path = None;
    let mut output = String::from("origin_new.olang");
    #[allow(unused_assignments)]
    let mut codegen_format = false;
    let mut wrap_mode = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--vm" => { i += 1; vm_path = args[i].clone(); }
            "--stdlib" => { i += 1; stdlib_path = Some(args[i].clone()); }
            "--bytecode" => { i += 1; bytecode_path = Some(args[i].clone()); }
            "--knowledge" | "--kn" => { i += 1; knowledge_path = Some(args[i].clone()); }
            "--output" | "-o" => { i += 1; output = args[i].clone(); }
            "--codegen" => { codegen_format = true; }
            "--wrap" => { wrap_mode = true; }
            "--help" | "-h" => {
                eprintln!("Usage: builder --vm <vm.bin> [--stdlib <dir>] [--bytecode <file>] [--knowledge <file>] [-o <output>] [--codegen] [--wrap]");
                process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                process::exit(1);
            }
        }
        i += 1;
    }

    if vm_path.is_empty() {
        eprintln!("Error: --vm is required");
        process::exit(1);
    }

    Args { vm_path, stdlib_path, bytecode_path, knowledge_path, output, codegen_format, wrap_mode }
}

/// Check if an ELF is a relocatable object file (ET_REL=1) vs executable (ET_EXEC=2).
fn is_object_file(data: &[u8]) -> bool {
    if data.len() < 18 { return false; }
    let e_type = u16::from_le_bytes([data[16], data[17]]);
    e_type == 1 // ET_REL
}

/// Extract .text section from an ELF object file or use raw data.
fn extract_vm_code_from(data: &[u8]) -> Vec<u8> {
    if data.len() > 4 && &data[0..4] == b"\x7fELF" {
        extract_elf_text(data).unwrap_or_else(|| {
            eprintln!("Warning: could not find .text in ELF, using entire file");
            data.to_vec()
        })
    } else {
        data.to_vec()
    }
}

/// Simple ELF64 .text section extractor.
fn extract_elf_text(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 64 { return None; }

    // ELF64 header fields
    let shoff = u64::from_le_bytes(data[40..48].try_into().ok()?) as usize;
    let shentsize = u16::from_le_bytes(data[58..60].try_into().ok()?) as usize;
    let shnum = u16::from_le_bytes(data[60..62].try_into().ok()?) as usize;
    let shstrndx = u16::from_le_bytes(data[62..64].try_into().ok()?) as usize;

    if shoff == 0 || shnum == 0 || shentsize < 64 { return None; }

    // Find string table section
    let strtab_off = {
        let sh = shoff + shstrndx * shentsize;
        if sh + shentsize > data.len() { return None; }
        u64::from_le_bytes(data[sh+24..sh+32].try_into().ok()?) as usize
    };

    // Search for .text section
    for i in 0..shnum {
        let sh = shoff + i * shentsize;
        if sh + shentsize > data.len() { continue; }

        let name_off = u32::from_le_bytes(data[sh..sh+4].try_into().ok()?) as usize;
        let sh_type = u32::from_le_bytes(data[sh+4..sh+8].try_into().ok()?);
        let offset = u64::from_le_bytes(data[sh+24..sh+32].try_into().ok()?) as usize;
        let size = u64::from_le_bytes(data[sh+32..sh+40].try_into().ok()?) as usize;

        // SHT_PROGBITS = 1
        if sh_type != 1 { continue; }

        // Check name
        let name_start = strtab_off + name_off;
        if name_start + 5 <= data.len() {
            if &data[name_start..name_start+5] == b".text" {
                if offset + size <= data.len() {
                    return Some(data[offset..offset+size].to_vec());
                }
            }
        }
    }

    None
}

fn main() {
    let args = parse_args();

    eprintln!("Builder — origin.olang packer");
    eprintln!("  VM: {}", args.vm_path);

    // 1. Read VM code
    let vm_raw = std::fs::read(&args.vm_path).unwrap_or_else(|e| {
        eprintln!("Error reading VM file: {}", e);
        process::exit(1);
    });
    let is_elf = vm_raw.len() > 4 && &vm_raw[0..4] == b"\x7fELF";
    // For wrap mode or linked ELF: keep entire binary
    // For ELF .o files: extract .text section
    let vm_code = if args.wrap_mode || (is_elf && !is_object_file(&vm_raw)) {
        vm_raw.clone()
    } else {
        extract_vm_code_from(&vm_raw)
    };
    eprintln!("  VM code: {} bytes ({})", vm_code.len(),
        if is_elf && !is_object_file(&vm_raw) { "linked ELF" } else { "extracted .text" });

    // 2. Get bytecode (compile .ol or read pre-compiled)
    let bytecode = if let Some(ref bc_path) = args.bytecode_path {
        eprintln!("  Bytecode: {} (pre-compiled)", bc_path);
        std::fs::read(bc_path).unwrap_or_else(|e| {
            eprintln!("Error reading bytecode: {}", e);
            process::exit(1);
        })
    } else if let Some(ref stdlib) = args.stdlib_path {
        eprintln!("  Compiling stdlib: {}", stdlib);
        compile::compile_all(stdlib).unwrap_or_else(|e| {
            eprintln!("Error compiling: {}", e);
            process::exit(1);
        })
    } else {
        eprintln!("  No bytecode (empty)");
        Vec::new()
    };
    eprintln!("  Bytecode: {} bytes", bytecode.len());

    // 3. Read knowledge
    let knowledge = if let Some(ref kn_path) = args.knowledge_path {
        std::fs::read(kn_path).unwrap_or_else(|e| {
            eprintln!("Warning: could not read knowledge file: {}", e);
            Vec::new()
        })
    } else {
        Vec::new()
    };
    eprintln!("  Knowledge: {} bytes", knowledge.len());

    // 4. Pack
    let is_linked_elf = args.wrap_mode ||
        (is_elf && !is_object_file(&vm_raw));

    let binary = pack::pack(&pack::PackConfig {
        vm_code: &vm_code,
        bytecode: &bytecode,
        knowledge: &knowledge,
        codegen_format: args.codegen_format,
        is_linked_elf,
        arch: pack::Arch::X86_64,
    });

    // 5. Write output
    std::fs::write(&args.output, &binary).unwrap_or_else(|e| {
        eprintln!("Error writing output: {}", e);
        process::exit(1);
    });

    // 6. Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(
            &args.output,
            std::fs::Permissions::from_mode(0o755),
        );
    }

    eprintln!("  Output: {} ({} bytes)", args.output, binary.len());
    eprintln!("Done!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_raw_binary() {
        // Non-ELF file → treated as raw
        let data = vec![0x90, 0xC3]; // nop; ret
        // Since extract_elf_text checks ELF magic, non-ELF returns None
        assert!(extract_elf_text(&data).is_none());
    }
}
