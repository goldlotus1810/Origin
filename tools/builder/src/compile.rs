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
            Ok(bytecode) => output.extend_from_slice(&bytecode),
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
}
