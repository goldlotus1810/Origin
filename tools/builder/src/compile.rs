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

    // Lower to IR
    let program = olang::lang::semantic::lower(&stmts);

    // Encode to bytecode
    Ok(olang::exec::bytecode::encode_bytecode(&program.ops))
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
                // B7: Strip trailing Halt (0x0F) from each file's bytecode
                // so execution continues to the next file. The final Halt
                // is appended by compile_all() after all files.
                while bytecode.last() == Some(&0x0F) {
                    bytecode.pop();
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

#[cfg(test)]
mod tests {
    use super::*;

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
