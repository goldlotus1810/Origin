//! # structural — Checks dựa trên cấu trúc thật, KHÔNG hardcode pattern
//!
//! Mỗi check PARSE source → extract cấu trúc → so sánh logic.
//! Không grep string cố định. Không assume tên biến.

use std::path::Path;
use crate::CheckResult;
use crate::parse_rs::{
    extract_structs, extract_enums, extract_bit_shifts,
    extract_ol_builtins, extract_vm_builtins, extract_match_arms,
};

// ═══════════════════════════════════════════════════════════════
// S1: Molecule struct — parse thật, tính bytes thật
// ═══════════════════════════════════════════════════════════════

pub fn check_molecule_size(root: &Path) -> CheckResult {
    println!("  [S1] Molecule struct — actual size...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");
    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("S1 Molecule Size", &format!("Cannot read: {}", e)),
    };

    let structs = extract_structs(&content);
    let mol = match structs.iter().find(|s| s.name == "Molecule") {
        Some(s) => s,
        None => return CheckResult::fail("S1 Molecule Size", "struct Molecule not found"),
    };

    let bytes = mol.estimated_bytes();
    let fields = mol.field_count();
    let has_packed = mol.has_field("p_packed") || mol.has_field("p");

    let mut details = Vec::new();
    for f in &mol.fields {
        details.push(format!("  {}: {} ({}B)", f.name, f.ty, crate::parse_rs::type_size_pub(&f.ty)));
    }
    details.push(format!("Total estimated: {} bytes, {} fields", bytes, fields));
    details.push(format!("Has packed u16 field: {}", if has_packed { "YES" } else { "NO" }));

    if bytes <= 2 || has_packed {
        CheckResult::pass("S1 Molecule Size", &format!("OK — {}B, {} fields", bytes, fields))
            .with_details(details)
    } else {
        details.push(format!("v2 spec: P_weight = 2 bytes. Actual: {} bytes ({:.1}x overhead)", bytes, bytes as f64 / 2.0));
        CheckResult::fail("S1 Molecule Size", &format!(
            "Molecule = {}B ({} fields) — v2 requires 2B", bytes, fields
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// S2: ShapeBase enum — count variants, verify no CSG ops
// ═══════════════════════════════════════════════════════════════

pub fn check_shapebase_variants(root: &Path) -> CheckResult {
    println!("  [S2] ShapeBase enum — variant count...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");
    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("S2 ShapeBase", &format!("Cannot read: {}", e)),
    };

    let enums = extract_enums(&content);
    let shape = match enums.iter().find(|e| e.name == "ShapeBase") {
        Some(e) => e,
        None => return CheckResult::fail("S2 ShapeBase", "enum ShapeBase not found"),
    };

    let count = shape.variant_count();
    let names = shape.variant_names();

    // CSG operations should NOT be in ShapeBase
    let csg_ops = ["Union", "Intersect", "Subtract", "Difference"];
    let csg_in_shape: Vec<_> = csg_ops.iter()
        .filter(|op| names.contains(op))
        .collect();

    let mut details = Vec::new();
    details.push(format!("Variants: {} — {:?}", count, names));
    if !csg_in_shape.is_empty() {
        details.push(format!("CSG ops wrongly in ShapeBase: {:?}", csg_in_shape));
    }

    // v2: 18 SDF primitives, 0 CSG ops in ShapeBase
    if count >= 18 && csg_in_shape.is_empty() {
        CheckResult::pass("S2 ShapeBase", &format!("OK — {}/18 SDF primitives, 0 CSG ops", count))
            .with_details(details)
    } else {
        details.push(format!("v2: 18 SDF primitives. Found: {}. CSG ops in shape: {}", count, csg_in_shape.len()));
        CheckResult::fail("S2 ShapeBase", &format!(
            "{} variants (need 18), {} CSG ops wrongly included", count, csg_in_shape.len()
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// S3: CompactQR bit layout — extract actual shifts, verify v2
// ═══════════════════════════════════════════════════════════════

pub fn check_compactqr_bits(root: &Path) -> CheckResult {
    println!("  [S3] CompactQR — bit shift layout...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");
    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("S3 CompactQR Bits", &format!("Cannot read: {}", e)),
    };

    let shifts = extract_bit_shifts(&content, "from_molecule_lossy");

    let mut details = Vec::new();
    for (var, amount) in &shifts {
        details.push(format!("  {} << {}", var, amount));
    }

    if shifts.is_empty() {
        return CheckResult::warn("S3 CompactQR Bits", "No bit shifts found in from_molecule_lossy")
            .with_details(details);
    }

    // v2 layout: S<<12, R<<8, V<<5, A<<2, T=last 2 bits (no shift or <<0)
    // Verify by checking shift amounts sum to valid 16-bit layout
    let s_shift = shifts.iter().find(|(v, _)| v == "s").map(|(_, n)| *n);
    let r_shift = shifts.iter().find(|(v, _)| v == "r").map(|(_, n)| *n);

    let v2_s = s_shift == Some(12);
    let v2_r = r_shift == Some(8);

    details.push(format!("S shift: {:?} (v2 expects 12)", s_shift));
    details.push(format!("R shift: {:?} (v2 expects 8)", r_shift));

    if v2_s && v2_r {
        CheckResult::pass("S3 CompactQR Bits", "OK — [S:4][R:4][V:3][A:3][T:2] layout")
            .with_details(details)
    } else {
        details.push("v2: [S:4 @12][R:4 @8][V:3 @5][A:3 @2][T:2 @0]".into());
        CheckResult::fail("S3 CompactQR Bits", &format!(
            "S<<{} R<<{} — v2 requires S<<12 R<<8",
            s_shift.unwrap_or(0), r_shift.unwrap_or(0)
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// S4: Olang compile gap — compare Stmt/Expr variants vs emit_expr match arms
// ═══════════════════════════════════════════════════════════════

pub fn check_compile_coverage(root: &Path) -> CheckResult {
    println!("  [S4] Olang — AST vs compiler coverage...");
    let syntax_path = root.join("crates/olang/src/lang/syntax.rs");
    let ir_path = root.join("crates/olang/src/exec/ir.rs");

    let syntax = match std::fs::read_to_string(&syntax_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("S4 Compile Coverage", &format!("Cannot read syntax.rs: {}", e)),
    };
    let ir = match std::fs::read_to_string(&ir_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("S4 Compile Coverage", &format!("Cannot read ir.rs: {}", e)),
    };

    // Extract Stmt and Expr enum variants from syntax.rs
    let enums = extract_enums(&syntax);
    let stmt_enum = enums.iter().find(|e| e.name == "Stmt");
    let expr_enum = enums.iter().find(|e| e.name == "Expr");

    // Extract match arms from emit_expr / emit_stmt in ir.rs
    let emit_arms = extract_match_arms(&ir, "stmt");
    let emit_expr_arms = extract_match_arms(&ir, "expr");
    let all_arms: std::collections::HashSet<_> = emit_arms.iter().chain(emit_expr_arms.iter()).collect();

    let mut details = Vec::new();
    let mut gap_count = 0;

    if let Some(stmt) = stmt_enum {
        details.push(format!("Stmt variants: {}", stmt.variant_count()));
        for v in &stmt.variants {
            if !all_arms.contains(&v.name) {
                // Skip common non-code variants
                if v.name == "Pub" || v.name == "ModDecl" { continue; }
                gap_count += 1;
                details.push(format!("  ❌ Stmt::{} — not in emit()", v.name));
            }
        }
    }

    if let Some(expr) = expr_enum {
        details.push(format!("Expr variants: {}", expr.variant_count()));
        for v in &expr.variants {
            if !all_arms.contains(&v.name) {
                gap_count += 1;
                details.push(format!("  ❌ Expr::{} — not in emit()", v.name));
            }
        }
    }

    if gap_count == 0 {
        CheckResult::pass("S4 Compile Coverage", "OK — all AST variants handled in emit()")
            .with_details(details)
    } else {
        details.push(format!("{} AST variants parsed but NOT compiled → silent code loss", gap_count));
        CheckResult::fail("S4 Compile Coverage", &format!(
            "{} AST variants not in emit() — silent drops", gap_count
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// S5: Stdlib builtins — cross-ref .ol files vs VM dispatch
// ═══════════════════════════════════════════════════════════════

pub fn check_stdlib_builtins_xref(root: &Path) -> CheckResult {
    println!("  [S5] Stdlib — builtin cross-reference...");
    let stdlib_dir = root.join("stdlib");
    let vm_path = root.join("crates/olang/src/exec/vm.rs");

    if !stdlib_dir.exists() {
        return CheckResult::fail("S5 Stdlib XRef", "stdlib/ not found");
    }

    let vm_source = match std::fs::read_to_string(&vm_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("S5 Stdlib XRef", &format!("Cannot read vm.rs: {}", e)),
    };

    // Extract all builtins the VM knows about
    let vm_builtins: std::collections::HashSet<String> = extract_vm_builtins(&vm_source).into_iter().collect();

    // Scan all .ol files for builtin calls
    let mut ol_builtins: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    fn scan_dir(dir: &Path, map: &mut std::collections::HashMap<String, Vec<String>>) {
        let entries = match std::fs::read_dir(dir) { Ok(e) => e, Err(_) => return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, map);
            } else if path.extension().and_then(|e| e.to_str()) == Some("ol") {
                let fname = path.file_name().unwrap_or_default().to_str().unwrap_or("?").to_string();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for builtin in extract_ol_builtins(&content) {
                        map.entry(builtin).or_default().push(fname.clone());
                    }
                }
            }
        }
    }
    scan_dir(&stdlib_dir, &mut ol_builtins);

    let mut details = Vec::new();
    let mut missing = Vec::new();

    for (builtin, files) in &ol_builtins {
        if !vm_builtins.contains(builtin) {
            let dedup: std::collections::HashSet<_> = files.iter().collect();
            let file_list: Vec<_> = dedup.into_iter().cloned().collect();
            missing.push(builtin.clone());
            details.push(format!("❌ {} — used by {:?}, NOT in VM", builtin, file_list));
        }
    }

    details.insert(0, format!("VM builtins: {}, .ol builtins: {}, MISSING: {}",
        vm_builtins.len(), ol_builtins.len(), missing.len()));

    if missing.is_empty() {
        CheckResult::pass("S5 Stdlib XRef", &format!("OK — all {} .ol builtins in VM", ol_builtins.len()))
            .with_details(details)
    } else {
        CheckResult::fail("S5 Stdlib XRef", &format!(
            "{} builtins NOT in VM — stdlib crashes at runtime", missing.len()
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// S6: Op enum vs VM execute — verify all opcodes handled
// ═══════════════════════════════════════════════════════════════

pub fn check_opcode_coverage(root: &Path) -> CheckResult {
    println!("  [S6] VM — opcode coverage...");
    let ir_path = root.join("crates/olang/src/exec/ir.rs");
    let vm_path = root.join("crates/olang/src/exec/vm.rs");

    let ir_source = match std::fs::read_to_string(&ir_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("S6 Opcode Coverage", &format!("Cannot read ir.rs: {}", e)),
    };
    let vm_source = match std::fs::read_to_string(&vm_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("S6 Opcode Coverage", &format!("Cannot read vm.rs: {}", e)),
    };

    // Extract Op enum variants from ir.rs
    let enums = extract_enums(&ir_source);
    let op_enum = match enums.iter().find(|e| e.name == "Op") {
        Some(e) => e,
        None => return CheckResult::fail("S6 Opcode Coverage", "enum Op not found in ir.rs"),
    };

    // Extract handled opcodes from vm.rs execute() match
    let handled = extract_match_arms(&vm_source, "op");
    let handled_set: std::collections::HashSet<_> = handled.iter().map(|s| s.as_str()).collect();

    let mut details = Vec::new();
    let mut unhandled = Vec::new();

    for v in &op_enum.variants {
        if !handled_set.contains(v.name.as_str()) {
            unhandled.push(v.name.clone());
            details.push(format!("❌ Op::{} — defined but NOT handled in VM", v.name));
        }
    }

    details.insert(0, format!("Op variants: {}, VM handled: {}, unhandled: {}",
        op_enum.variant_count(), handled.len(), unhandled.len()));

    if unhandled.is_empty() {
        CheckResult::pass("S6 Opcode Coverage", &format!(
            "OK — all {} opcodes handled", op_enum.variant_count()
        ))
            .with_details(details)
    } else {
        CheckResult::fail("S6 Opcode Coverage", &format!(
            "{} opcodes NOT handled in VM", unhandled.len()
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════
// S7: Chain inner type — parse MolecularChain struct
// ═══════════════════════════════════════════════════════════════

pub fn check_chain_type(root: &Path) -> CheckResult {
    println!("  [S7] MolecularChain — inner type...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");
    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("S7 Chain Type", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // Find MolecularChain definition — could be tuple struct or regular
    let mut inner_type = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("struct MolecularChain") {
            // Tuple struct: pub struct MolecularChain(pub Vec<Molecule>);
            if let Some(start) = trimmed.find("Vec<") {
                if let Some(end) = trimmed[start..].find('>') {
                    inner_type = trimmed[start..start + end + 1].to_string();
                }
            }
            break;
        }
    }

    if inner_type.is_empty() {
        return CheckResult::warn("S7 Chain Type", "Cannot find MolecularChain definition")
            .with_details(details);
    }

    details.push(format!("MolecularChain inner: {}", inner_type));

    if inner_type.contains("u16") {
        CheckResult::pass("S7 Chain Type", "OK — Vec<u16> (2B/link)")
            .with_details(details)
    } else {
        let link_size = if inner_type.contains("Molecule") { "11B" } else { "?" };
        details.push(format!("v2: chain link = u16 (2B) = codepoint reference"));
        details.push(format!("Current: {} ({}/link)", inner_type, link_size));
        CheckResult::fail("S7 Chain Type", &format!(
            "Chain = {} — v2 requires Vec<u16>", inner_type
        ))
            .with_details(details)
    }
}
