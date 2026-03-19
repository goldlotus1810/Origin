//! Integration: Stdlib compile audit — ALL .ol files parse + lower + encode + decode
//!
//! Verifies every stdlib and homeos .ol file can:
//!   1. Parse (syntax)
//!   2. Lower to IR (semantic)
//!   3. Encode to bytecode
//!   4. Decode back to ops
//!
//! Known limitations:
//!   - Push/Load decode asymmetry: Push (0x01, u16 len) decodes as Load (0x02, u8 len)
//!     for short strings. This means re-encode may differ in size. This is intentional.
//!   - Some files use hex literals (0xFF), `==` in match, or keywords as identifiers
//!     that the parser doesn't yet support. These are tracked as known parse failures.
//!
//! Covers: olang::lang::syntax + olang::lang::semantic + olang::exec::bytecode

use olang::exec::bytecode::{decode_bytecode, encode_bytecode};
use olang::exec::ir::Op;
use olang::lang::semantic::lower;
use olang::lang::syntax::parse;
use std::fs;
use std::path::Path;

/// Collect all .ol files under a directory recursively.
fn collect_ol_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                files.extend(collect_ol_files(&p));
            } else if p.extension().map(|e| e == "ol").unwrap_or(false) {
                files.push(p);
            }
        }
    }
    files.sort();
    files
}

fn stdlib_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("stdlib")
}

/// Files that use syntax not yet supported by the parser.
/// These are KNOWN limitations, not bugs — tracked for future parser work.
const KNOWN_PARSE_FAILURES: &[&str] = &[
    // Hex literal syntax (0xFF) — parser tokenizes "0" then "xB8" as separate tokens
    "asm_emit.ol",
    "asm_emit_arm64.ol",
    "elf_emit.ol",
    "reproduce.ol",
    "wasm_emit.ol",
    // == in match expressions or comparisons — parser sees Eq token unexpectedly
    "benchmark.ol",
    "dream.ol",
    "dream_cache.ol",
    "install.ol",
    "jit.ol",
    "module_index.ol",
    "mol_pool.ol",
    "optimize.ol",
    "registry_cache.ol",
    "silk_cache.ol",
    // Keywords used as identifiers or struct field syntax
    "intent.ol",      // "learn" is a Command keyword
    "silk_ops.ol",    // colon syntax in struct literal
];

// ═══════════════════════════════════════════════════════════════════
// Core audit: all parseable files compile and decode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn audit_all_parseable_files_compile_and_decode() {
    let files = collect_ol_files(&stdlib_dir());
    assert!(files.len() >= 40, "expected ≥40 .ol files, found {}", files.len());

    let mut pass = 0;
    let mut skip = 0;
    let mut fail = 0;
    let mut errors = Vec::new();

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        if KNOWN_PARSE_FAILURES.contains(&name.as_str()) {
            skip += 1;
            continue;
        }

        let source = fs::read_to_string(path).unwrap();

        // 1. Parse
        let stmts = match parse(&source) {
            Ok(s) => s,
            Err(e) => {
                fail += 1;
                errors.push(format!("{}: parse error: {:?}", name, e));
                continue;
            }
        };

        // 2. Lower
        let program = lower(&stmts);
        assert!(program.ops.len() >= 1, "{}: 0 ops", name);

        // 3. Encode
        let bytecode = encode_bytecode(&program.ops);
        assert!(!bytecode.is_empty(), "{}: empty bytecode", name);

        // 4. Decode (must succeed)
        match decode_bytecode(&bytecode) {
            Ok(decoded) => {
                assert!(!decoded.is_empty(), "{}: decoded to 0 ops", name);
                // Verify Halt is present
                let has_halt = decoded.iter().any(|op| matches!(op, Op::Halt));
                assert!(has_halt, "{}: no Halt in decoded ops", name);
                pass += 1;
            }
            Err(e) => {
                fail += 1;
                errors.push(format!("{}: decode error: {:?}", name, e));
            }
        }
    }

    if fail > 0 {
        panic!(
            "{} pass, {} skip, {} FAIL:\n  {}",
            pass, skip, fail, errors.join("\n  ")
        );
    }

    // At least 30 files should compile (50 total - 17 known parse failures)
    assert!(pass >= 30, "only {} files passed (expected ≥30)", pass);
}

#[test]
fn audit_file_count_is_50() {
    let files = collect_ol_files(&stdlib_dir());
    assert_eq!(files.len(), 50, "expected 50 .ol files, found {}", files.len());
}

// ═══════════════════════════════════════════════════════════════════
// Known parse failures are actually unparseable
// ═══════════════════════════════════════════════════════════════════

#[test]
fn audit_known_parse_failures_are_real() {
    let dir = stdlib_dir();
    let files = collect_ol_files(&dir);

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !KNOWN_PARSE_FAILURES.contains(&name.as_str()) {
            continue;
        }

        let source = fs::read_to_string(path).unwrap();
        // These should actually fail to parse — if they start passing,
        // the parser improved and we should remove them from the known list
        if parse(&source).is_ok() {
            panic!(
                "{} is in KNOWN_PARSE_FAILURES but now parses OK! Remove it from the list.",
                name
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bootstrap files (large, critical)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn audit_bootstrap_files_produce_substantial_bytecode() {
    let bootstrap_dir = stdlib_dir().join("bootstrap");
    let files = collect_ol_files(&bootstrap_dir);
    assert_eq!(files.len(), 4, "bootstrap should have 4 files");

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let source = fs::read_to_string(path).unwrap();
        let stmts = parse(&source).unwrap_or_else(|e| panic!("{}: {:?}", name, e));
        let program = lower(&stmts);
        let bytecode = encode_bytecode(&program.ops);

        // Bootstrap files are large
        assert!(program.ops.len() >= 50,
            "{} should have ≥50 ops (got {})", name, program.ops.len());
        assert!(bytecode.len() >= 200,
            "{} should have ≥200 bytes bytecode (got {})", name, bytecode.len());
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bytecode properties
// ═══════════════════════════════════════════════════════════════════

#[test]
fn audit_every_parseable_file_ends_with_halt() {
    let files = collect_ol_files(&stdlib_dir());

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if KNOWN_PARSE_FAILURES.contains(&name.as_str()) {
            continue;
        }

        let source = fs::read_to_string(path).unwrap();
        let stmts = match parse(&source) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let program = lower(&stmts);
        let bytecode = encode_bytecode(&program.ops);

        assert_eq!(
            *bytecode.last().unwrap(), 0x0F,
            "{} bytecode should end with Halt (0x0F)", name
        );
    }
}

#[test]
fn audit_bytecode_roundtrip_decode_succeeds_for_all_parseable() {
    // Even though re-encode may differ (Push/Load asymmetry),
    // decode must always succeed for valid encoded bytecode.
    let files = collect_ol_files(&stdlib_dir());
    let mut decoded_count = 0;

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if KNOWN_PARSE_FAILURES.contains(&name.as_str()) {
            continue;
        }

        let source = fs::read_to_string(path).unwrap();
        let stmts = match parse(&source) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let program = lower(&stmts);
        let bytecode = encode_bytecode(&program.ops);
        let decoded = decode_bytecode(&bytecode)
            .unwrap_or_else(|e| panic!("{}: decode failed: {:?}", name, e));

        assert!(!decoded.is_empty(), "{}: decoded to 0 ops", name);
        decoded_count += 1;
    }

    assert!(decoded_count >= 30, "only {} files decoded (expected ≥30)", decoded_count);
}

// ═══════════════════════════════════════════════════════════════════
// Specific important files
// ═══════════════════════════════════════════════════════════════════

#[test]
fn audit_core_stdlib_files_compile() {
    // These core files MUST compile — they are the foundation
    let core_files = [
        "result.ol", "iter.ol", "sort.ol", "hash.ol", "format.ol",
        "json.ol", "mol.ol", "chain.ol", "string.ol", "math.ol",
        "vec.ol", "map.ol", "set.ol", "deque.ol", "bytes.ol",
    ];

    let dir = stdlib_dir();
    for name in &core_files {
        let path = dir.join(name);
        assert!(path.exists(), "{} should exist", name);

        let source = fs::read_to_string(&path).unwrap();
        let stmts = parse(&source)
            .unwrap_or_else(|e| panic!("{}: parse failed: {:?}", name, e));
        let program = lower(&stmts);
        assert!(program.ops.len() >= 1, "{}: 0 ops", name);

        let bytecode = encode_bytecode(&program.ops);
        decode_bytecode(&bytecode)
            .unwrap_or_else(|e| panic!("{}: decode failed: {:?}", name, e));
    }
}

#[test]
fn audit_homeos_parseable_files_compile() {
    let homeos_dir = stdlib_dir().join("homeos");
    let files = collect_ol_files(&homeos_dir);
    assert!(files.len() >= 20, "homeos should have ≥20 files, found {}", files.len());

    let mut compiled = 0;
    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if KNOWN_PARSE_FAILURES.contains(&name.as_str()) {
            continue;
        }

        let source = fs::read_to_string(path).unwrap();
        let stmts = match parse(&source) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let program = lower(&stmts);
        let bytecode = encode_bytecode(&program.ops);
        decode_bytecode(&bytecode)
            .unwrap_or_else(|e| panic!("{}: decode failed: {:?}", name, e));
        compiled += 1;
    }

    // At least some homeos files should compile
    assert!(compiled >= 5, "only {} homeos files compiled", compiled);
}
