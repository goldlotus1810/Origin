/// Compile .ol source files → bytecode using the Olang pipeline.
///
/// Pipeline: source → parse → lower (semantic) → encode_bytecode

use std::path::Path;

#[derive(Debug)]
pub enum CompileError {
    Io(std::io::Error),
    Parse(String),
}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        CompileError::Io(e)
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Io(e) => write!(f, "IO error: {}", e),
            CompileError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

/// Compile a single .ol file to bytecode.
pub fn compile_file(path: &Path) -> Result<Vec<u8>, CompileError> {
    let source = std::fs::read_to_string(path)?;
    compile_source(&source)
}

/// Compile source string to bytecode.
pub fn compile_source(source: &str) -> Result<Vec<u8>, CompileError> {
    // Parse
    let stmts = olang::lang::syntax::parse(source)
        .map_err(|e| CompileError::Parse(format!("{:?}", e)))?;

    eprintln!("      parse: {} stmts", stmts.len());

    // Lower to IR
    let program = olang::lang::semantic::lower(&stmts);
    eprintln!("      lower: {} ops", program.ops.len());

    // Encode to bytecode
    let bc = olang::exec::bytecode::encode_bytecode(&program.ops);
    eprintln!("      encode: {} bytes", bc.len());
    Ok(bc)
}

/// Compile all .ol files in a directory (and bootstrap/ subdirectory).
pub fn compile_all(stdlib_path: &str) -> Result<Vec<u8>, CompileError> {
    let mut all_bytecode = Vec::new();

    // Compile bootstrap/ first (if exists)
    let bootstrap_dir = Path::new(stdlib_path).join("bootstrap");
    if bootstrap_dir.is_dir() {
        compile_dir(&bootstrap_dir, &mut all_bytecode)?;
    }

    // Compile stdlib root .ol files
    compile_dir(Path::new(stdlib_path), &mut all_bytecode)?;

    // Compile homeos/ subdirectory (HomeOS subsystem modules)
    let homeos_dir = Path::new(stdlib_path).join("homeos");
    if homeos_dir.is_dir() {
        compile_dir(&homeos_dir, &mut all_bytecode)?;
    }

    // B7: Append a single Halt (0x0F) at the end of all bytecode.
    // Individual files' Halts are stripped during compilation so
    // execution flows through all stdlib files before stopping.
    all_bytecode.push(0x0F);

    Ok(all_bytecode)
}

fn compile_dir(dir: &Path, output: &mut Vec<u8>) -> Result<(), CompileError> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map_or(false, |ext| ext == "ol")
        })
        .collect();

    // Sort for deterministic output
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        eprintln!("  Compiling: {}", path.display());
        match compile_file(&path) {
            Ok(mut bytecode) => {
                if bytecode.len() <= 1 {
                    eprintln!("    ⚠ {} EMPTY (parse failed?)", path.display());
                }
                // B7: Strip trailing Halt (0x0F) from each file's bytecode
                // so execution continues to the next file. The final Halt
                // is appended by compile_all() after all files.
                while bytecode.last() == Some(&0x0F) {
                    bytecode.pop();
                }
                // Relocate jump targets: add the current output offset
                // because bytecode was compiled with targets relative to 0.
                let base_offset = output.len() as u32;
                if base_offset > 0 {
                    relocate_jumps(&mut bytecode, base_offset);
                }
                output.extend_from_slice(&bytecode);
            }
            Err(e) => {
                eprintln!("  Warning: skipping {} ({})", path.display(), e);
            }
        }
    }

    Ok(())
}

/// Adjust jump targets (Jmp, Jz, TryBegin) by adding a base offset.
/// This is needed when multiple files' bytecodes are concatenated,
/// since each file's internal jumps were encoded relative to offset 0.
fn relocate_jumps(bytecode: &mut [u8], base: u32) {
    let mut pc = 0;
    while pc < bytecode.len() {
        let tag = bytecode[pc];
        pc += 1;
        match tag {
            0x01 => { // Push [mol_count:2 u16][molecules: mol_count × 2 bytes]
                if pc + 2 > bytecode.len() { break; }
                let mol_count = u16::from_le_bytes([bytecode[pc], bytecode[pc+1]]) as usize;
                pc += 2 + mol_count * 2; // each molecule = 2 bytes
            }
            0x02 | 0x07 | 0x13 | 0x14 | 0x1C => { // Load/Call/Store/LoadLocal/StoreUpdate [len:1][name:N]
                if pc >= bytecode.len() { break; }
                let len = bytecode[pc] as usize;
                pc += 1 + len;
            }
            0x09 | 0x0A | 0x1A => { // Jmp/Jz/TryBegin [target:4] — RELOCATE
                if pc + 4 > bytecode.len() { break; }
                let target = u32::from_le_bytes([bytecode[pc], bytecode[pc+1], bytecode[pc+2], bytecode[pc+3]]);
                let relocated = target + base;
                bytecode[pc..pc+4].copy_from_slice(&relocated.to_le_bytes());
                pc += 4;
            }
            0x0E => { pc += 4; } // Loop [count:4] — NOT a jump target
            0x15 => { pc += 8; } // PushNum [f64:8]
            0x19 => { pc += 2; } // PushMol [u16: 2 bytes]
            0x04 | 0x05 => { pc += 1; } // Edge/Query [rel:1]
            0x25 => { // Closure [param:1][body_len:4] — body_len is byte count, NOT relocated
                pc += 1 + 4;
            }
            0x24 => { // CallClosure [name_len:1][name:N][arity:1]
                if pc >= bytecode.len() { break; }
                let len = bytecode[pc] as usize;
                pc += 1 + len + 1;
            }
            0x23 => { // Ffi [name_len:1][name:N][arity:1]
                if pc >= bytecode.len() { break; }
                let len = bytecode[pc] as usize;
                pc += 1 + len + 1;
            }
            0x3A => { pc += 1; } // CallBuiltin [id:1]
            _ => { } // Single-byte ops (0x03, 0x06, 0x08, 0x0B-0x0D, 0x0F-0x12, 0x16-0x18, 0x1B, etc.)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_ol_jumps_land_on_opcodes() {
        // Verify that all Jmp/Jz targets in semantic.ol's bytecode
        // land on valid opcode positions (not in the middle of data).
        let path = std::path::Path::new("stdlib/bootstrap/semantic.ol");
        if !path.exists() { return; } // skip if not in workspace root
        let bytecode = compile_file(path).unwrap();

        // Build a set of valid opcode positions
        let mut valid_positions = std::collections::HashSet::new();
        let mut pc = 0;
        while pc < bytecode.len() {
            valid_positions.insert(pc);
            let tag = bytecode[pc];
            pc += 1;
            match tag {
                0x01 => {
                    if pc + 2 > bytecode.len() { break; }
                    let len = u16::from_le_bytes([bytecode[pc], bytecode[pc+1]]) as usize;
                    pc += 2 + len;
                }
                0x02 | 0x07 | 0x13 | 0x14 | 0x1C => {
                    if pc >= bytecode.len() { break; }
                    let len = bytecode[pc] as usize;
                    pc += 1 + len;
                }
                0x09 | 0x0A | 0x1A | 0x0E => { pc += 4; }
                0x15 => { pc += 8; }
                0x19 => { pc += 5; }
                0x04 | 0x05 => { pc += 1; }
                0x25 => { pc += 1 + 4; }
                0x26 | 0x27 => {
                    if pc >= bytecode.len() { break; }
                    let len = bytecode[pc] as usize;
                    pc += 1 + len + 1;
                }
                _ => {} // single byte
            }
        }
        // Also add bytecode.len() as valid (jump past end = ok for Halt)
        valid_positions.insert(bytecode.len());

        // Now check all Jmp/Jz/TryBegin targets
        pc = 0;
        let mut bad = 0;
        while pc < bytecode.len() {
            let tag = bytecode[pc];
            pc += 1;
            match tag {
                0x09 | 0x0A | 0x1A => {
                    if pc + 4 > bytecode.len() { break; }
                    let target = u32::from_le_bytes([bytecode[pc], bytecode[pc+1], bytecode[pc+2], bytecode[pc+3]]) as usize;
                    if !valid_positions.contains(&target) && target < bytecode.len() {
                        let tag_name = match tag { 0x09 => "Jmp", 0x0A => "Jz", _ => "TryBegin" };
                        eprintln!("BAD {tag_name} at byte {}: target {target} is NOT a valid opcode position", pc - 1);
                        bad += 1;
                        if bad > 5 { break; }
                    }
                    pc += 4;
                }
                0x01 => {
                    if pc + 2 > bytecode.len() { break; }
                    let len = u16::from_le_bytes([bytecode[pc], bytecode[pc+1]]) as usize;
                    pc += 2 + len;
                }
                0x02 | 0x07 | 0x13 | 0x14 | 0x1C => {
                    if pc >= bytecode.len() { break; }
                    let len = bytecode[pc] as usize;
                    pc += 1 + len;
                }
                0x0E => { pc += 4; }
                0x15 => { pc += 8; }
                0x19 => { pc += 5; }
                0x04 | 0x05 => { pc += 1; }
                0x25 => { pc += 1 + 4; }
                0x26 | 0x27 => {
                    if pc >= bytecode.len() { break; }
                    let len = bytecode[pc] as usize;
                    pc += 1 + len + 1;
                }
                _ => {}
            }
        }
        assert_eq!(bad, 0, "{bad} jump targets land on non-opcode positions");
    }

    #[test]
    fn test_compile_simple() {
        let bytecode = compile_source("let x = 42;").unwrap();
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_compile_empty() {
        let bytecode = compile_source("").unwrap();
        // Empty source should produce at least a Halt
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn b7_halt_stripping() {
        // Verify that compiled bytecode ends with Halt (0x0F)
        let bytecode = compile_source("let x = 42;").unwrap();
        assert_eq!(*bytecode.last().unwrap(), 0x0F, "Bytecode should end with Halt");

        // After stripping: should not end with Halt
        let mut stripped = bytecode.clone();
        while stripped.last() == Some(&0x0F) {
            stripped.pop();
        }
        assert_ne!(*stripped.last().unwrap(), 0x0F, "Stripped bytecode should not end with Halt");
    }

    #[test]
    fn b7_concatenated_files_single_halt() {
        // Two files compiled together should only have ONE Halt at the end
        let a = compile_source("let x = 1;").unwrap();
        let b = compile_source("let y = 2;").unwrap();

        // Simulate compile_all: strip halts, concatenate, add final halt
        let mut combined = Vec::new();
        for mut bc in [a, b] {
            while bc.last() == Some(&0x0F) {
                bc.pop();
            }
            combined.extend_from_slice(&bc);
        }
        combined.push(0x0F); // final halt

        // Count Halts in combined bytecode
        let halt_count = combined.iter().filter(|&&b| b == 0x0F).count();
        assert_eq!(halt_count, 1, "Should have exactly 1 Halt, got {}", halt_count);
    }
}
