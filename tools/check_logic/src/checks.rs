//! # Logic checks — 6 bug patterns + 5 checkpoints + 23 invariants + data
//!
//! Mỗi check trả về CheckResult { Pass | Fail | Warn }.
//! Ref: docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md
//! Ref: old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md

use std::path::Path;
use crate::{CheckResult, scan_rs_files, grep_pattern, grep_pattern_ci};

// ═══════════════════════════════════════════════════════════════════
// Bug Pattern #1: compose() KHÔNG ĐƯỢC dùng simple average cho V
// ═══════════════════════════════════════════════════════════════════

pub fn check_compose_no_average(root: &Path) -> CheckResult {
    println!("[1/37] Compose — no simple average for Valence...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    // Tìm pattern nguy hiểm: (Va + Vb) / 2  hoặc  / 2.0 trong context compose
    // Chỉ check trong crates/context và crates/silk (pipeline code)
    let pipeline_files: Vec<_> = files.iter()
        .filter(|(p, _)| {
            let ps = p.to_str().unwrap_or("");
            (ps.contains("context") || ps.contains("silk") || ps.contains("agents"))
                && !ps.contains("test")
        })
        .cloned()
        .collect();

    let mut violations = Vec::new();

    // Pattern: simple average of valence/emotion
    for (path, content) in &pipeline_files {
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }
            // Detect: (x + y) / 2  or  / 2.0  in emotion/valence context
            let has_div2 = trimmed.contains("/ 2.0") || trimmed.contains("/ 2 ");
            let in_emotion_ctx = trimmed.contains("valence")
                || trimmed.contains("emotion")
                || trimmed.contains("affect")
                || trimmed.contains(".v ")
                || trimmed.contains(".v)");

            if has_div2 && in_emotion_ctx {
                let rel = path.strip_prefix(root).unwrap_or(path);
                violations.push(format!(
                    "{}:{} — simple average in emotion path: {}",
                    rel.display(), i + 1, trimmed
                ));
            }
        }
    }

    // Verify amplify exists
    let amplify_hits = grep_pattern(&pipeline_files, "amplify");
    let blend_hits = grep_pattern(&pipeline_files, "blend_composite");

    if !violations.is_empty() {
        CheckResult::fail("BP#1 Compose", &format!("{} violation(s) — simple average found", violations.len()))
            .with_details(violations)
    } else if amplify_hits.is_empty() && blend_hits.is_empty() {
        CheckResult::warn("BP#1 Compose", "No amplify/blend_composite found — verify compose logic manually")
    } else {
        CheckResult::pass("BP#1 Compose", &format!(
            "OK — {} amplify refs, {} blend refs, 0 simple-average violations",
            amplify_hits.len(), blend_hits.len()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bug Pattern #2: self-correct PHẢI có rollback guard
// ═══════════════════════════════════════════════════════════════════

pub fn check_self_correct_rollback(root: &Path) -> CheckResult {
    println!("[2/37] Self-correct — rollback guard...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let rollback_hits = grep_pattern_ci(&files, "rollback");
    let backup_hits = grep_pattern_ci(&files, "backup");
    let _self_correct_hits = grep_pattern_ci(&files, "self_correct");
    let quality_phi = grep_pattern(&files, "0.618");

    // fire_count + maturity = implicit self-correct path
    let fire_count_hits = grep_pattern(&files, "fire_count");
    let maturity_hits = grep_pattern_ci(&files, "maturity");

    let has_explicit_rollback = !rollback_hits.is_empty() || !backup_hits.is_empty();
    let has_implicit_selfcorrect = !fire_count_hits.is_empty() && !maturity_hits.is_empty();

    if has_explicit_rollback && !quality_phi.is_empty() {
        CheckResult::pass("BP#2 Self-correct", &format!(
            "OK — rollback refs: {}, φ⁻¹ threshold: {}, fire_count: {}",
            rollback_hits.len() + backup_hits.len(),
            quality_phi.len(),
            fire_count_hits.len()
        ))
    } else if has_implicit_selfcorrect {
        CheckResult::warn("BP#2 Self-correct", &format!(
            "Implicit via fire_count ({} refs) + maturity ({} refs) — no explicit rollback guard yet",
            fire_count_hits.len(), maturity_hits.len()
        ))
    } else {
        CheckResult::fail("BP#2 Self-correct", "No rollback guard found — quality corrections may worsen output")
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bug Pattern #3: quality weights Σ = 1.0
// ═══════════════════════════════════════════════════════════════════

pub fn check_quality_weights(root: &Path) -> CheckResult {
    println!("[3/37] Quality weights — Σ = 1.0...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let quality_hits = grep_pattern_ci(&files, "quality");
    let _weight_sum_hits = grep_pattern(&files, "w1 + w2");

    // Check: quality computation exists
    let quality_formula = grep_pattern(&files, "valid_score");
    let quality_entropy = grep_pattern(&files, "entropy_score");

    if !quality_formula.is_empty() && !quality_entropy.is_empty() {
        CheckResult::pass("BP#3 Quality weights", &format!(
            "OK — quality formula found ({} valid refs, {} entropy refs)",
            quality_formula.len(), quality_entropy.len()
        ))
    } else if !quality_hits.is_empty() {
        CheckResult::warn("BP#3 Quality weights", &format!(
            "{} quality refs found — verify Σwᵢ = 1.0 manually",
            quality_hits.len()
        ))
    } else {
        CheckResult::warn("BP#3 Quality weights", "No explicit quality scoring found — needed for self-correct")
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bug Pattern #4: entropy ε_floor = 0.01
// ═══════════════════════════════════════════════════════════════════

pub fn check_entropy_floor(root: &Path) -> CheckResult {
    println!("[4/37] Entropy — ε_floor for Σc...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let floor_hits = grep_pattern(&files, "floor");

    // Check for Shannon entropy computation
    let shannon_hits = grep_pattern(&files, "log2");
    let ln_hits = grep_pattern(&files, ".ln(");

    // Check aesthetic floor (known existing)
    let aesthetic_floor = grep_pattern(&files, "aesthetic");

    let has_entropy_floor = floor_hits.iter().any(|(p, _, l)| {
        let ps = p.to_str().unwrap_or("");
        (ps.contains("context") || ps.contains("agents")) && l.contains("floor")
    });

    if has_entropy_floor && (!shannon_hits.is_empty() || !ln_hits.is_empty()) {
        CheckResult::pass("BP#4 Entropy ε_floor", &format!(
            "OK — entropy floor found, {} log/ln refs",
            shannon_hits.len() + ln_hits.len()
        ))
    } else if !aesthetic_floor.is_empty() {
        CheckResult::warn("BP#4 Entropy ε_floor", &format!(
            "Aesthetic floor exists ({} refs) — need general ε_floor = 0.01 for entropy Σc",
            aesthetic_floor.len()
        ))
    } else {
        CheckResult::fail("BP#4 Entropy ε_floor", "No entropy floor found — Σc near 0 may cause H explosion")
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bug Pattern #5: HNSW insert deterministic tie-breaking
// ═══════════════════════════════════════════════════════════════════

pub fn check_hnsw_tiebreak(root: &Path) -> CheckResult {
    println!("[5/37] HNSW insert — deterministic tie-breaking...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let hnsw_hits = grep_pattern_ci(&files, "hnsw");
    let tiebreak_hits = grep_pattern_ci(&files, "tie");
    let _insert_hits = grep_pattern(&files, "fn insert");

    // Check for deterministic ordering in nearest-neighbor search
    let nearest_hits = grep_pattern_ci(&files, "nearest");
    let _search_hits = grep_pattern(&files, "fn search");

    if !tiebreak_hits.is_empty() {
        CheckResult::pass("BP#5 HNSW tie-break", &format!(
            "OK — tie-breaking logic found ({} refs)",
            tiebreak_hits.len()
        ))
    } else if !hnsw_hits.is_empty() || !nearest_hits.is_empty() {
        CheckResult::warn("BP#5 HNSW tie-break", &format!(
            "HNSW/nearest refs exist ({}/{}) but no explicit tie-breaking — add R-priority + index fallback",
            hnsw_hits.len(), nearest_hits.len()
        ))
    } else {
        CheckResult::warn("BP#5 HNSW tie-break", "No HNSW insert found yet — needed for L5+ dynamic knowledge")
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bug Pattern #6: SecurityGate ≥ 3 layers
// ═══════════════════════════════════════════════════════════════════

pub fn check_security_gate_3layer(root: &Path) -> CheckResult {
    println!("[6/37] SecurityGate — 3-layer detection...");
    let gate_path = root.join("crates/agents/src/pipeline/gate.rs");

    if !gate_path.exists() {
        return CheckResult::fail("BP#6 SecurityGate", "gate.rs not found!");
    }

    let content = match std::fs::read_to_string(&gate_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("BP#6 SecurityGate", &format!("Cannot read gate.rs: {}", e)),
    };

    // Check for 3 detection layers
    let has_crisis = content.contains("is_crisis") || content.contains("Crisis");
    let has_harmful = content.contains("is_harmful") || content.contains("Harmful");
    let has_normalized = content.contains("normalize") || content.contains("is_manipulation");
    let has_blackcurtain = content.contains("BlackCurtain") || content.contains("Unknown");

    let layer_count = [has_crisis, has_harmful, has_normalized].iter().filter(|&&x| x).count();

    let mut details = Vec::new();
    details.push(format!("Layer 1 (exact/crisis): {}", if has_crisis { "✅" } else { "❌" }));
    details.push(format!("Layer 2 (harmful/normalized): {}", if has_harmful { "✅" } else { "❌" }));
    details.push(format!("Layer 3 (manipulation/semantic): {}", if has_normalized { "✅" } else { "❌" }));
    details.push(format!("BlackCurtain: {}", if has_blackcurtain { "✅" } else { "❌" }));

    if layer_count >= 3 {
        CheckResult::pass("BP#6 SecurityGate", &format!("OK — {}/3 layers + BlackCurtain", layer_count))
            .with_details(details)
    } else if layer_count >= 2 {
        CheckResult::warn("BP#6 SecurityGate", &format!("Only {}/3 layers — need normalized text check", layer_count))
            .with_details(details)
    } else {
        CheckResult::fail("BP#6 SecurityGate", &format!("Only {}/3 layers — insufficient protection", layer_count))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Pipeline: 5 Checkpoints
// ═══════════════════════════════════════════════════════════════════

pub fn check_pipeline_checkpoints(root: &Path) -> CheckResult {
    println!("[7/37] Pipeline — 5 checkpoints...");
    let runtime_dir = root.join("crates/runtime");
    let agents_dir = root.join("crates/agents");
    let memory_dir = root.join("crates/memory");

    let rt_files = scan_rs_files(&runtime_dir);
    let ag_files = scan_rs_files(&agents_dir);
    let mem_files = scan_rs_files(&memory_dir);

    let all_files: Vec<_> = rt_files.iter().chain(ag_files.iter()).chain(mem_files.iter()).cloned().collect();

    // CP1: SecurityGate runs FIRST
    let gate_check = grep_pattern(&all_files, "check_text");
    let gate_first = grep_pattern(&all_files, "gate");

    // CP2: append-only (file write before RAM)
    let write_first = grep_pattern(&all_files, "append_node");
    let _registry_after = grep_pattern(&all_files, "registry");

    // CP3: fire_count + Fibonacci
    let fire_check = grep_pattern(&all_files, "fire_count");
    let fib_check = grep_pattern(&all_files, "fib");

    // CP4: LCA variance
    let variance_check = grep_pattern_ci(&all_files, "variance");

    // CP5: response validation
    let response_check = grep_pattern_ci(&all_files, "response");
    let tone_check = grep_pattern_ci(&all_files, "tone");

    let mut details = Vec::new();
    let mut cp_count = 0;

    if !gate_check.is_empty() || !gate_first.is_empty() {
        cp_count += 1;
        details.push(format!("CP1 GATE: ✅ ({} refs)", gate_check.len() + gate_first.len()));
    } else {
        details.push("CP1 GATE: ❌ SecurityGate.check_text() not found in pipeline".into());
    }

    if !write_first.is_empty() {
        cp_count += 1;
        details.push(format!("CP2 ENCODE/QT8: ✅ append_node ({} refs)", write_first.len()));
    } else {
        details.push("CP2 ENCODE/QT8: ❌ append-only write not found".into());
    }

    if !fire_check.is_empty() && !fib_check.is_empty() {
        cp_count += 1;
        details.push(format!("CP3 PROMOTE: ✅ fire_count ({}) + fib ({})", fire_check.len(), fib_check.len()));
    } else {
        details.push(format!("CP3 PROMOTE: ❌ fire_count={}, fib={}", fire_check.len(), fib_check.len()));
    }

    if !variance_check.is_empty() {
        cp_count += 1;
        details.push(format!("CP4 VARIANCE: ✅ ({} refs)", variance_check.len()));
    } else {
        details.push("CP4 VARIANCE: ❌ LCA variance check not found".into());
    }

    if !response_check.is_empty() && !tone_check.is_empty() {
        cp_count += 1;
        details.push(format!("CP5 RESPONSE: ✅ response ({}) + tone ({})", response_check.len(), tone_check.len()));
    } else {
        details.push(format!("CP5 RESPONSE: ⚠️ response={}, tone={}", response_check.len(), tone_check.len()));
    }

    if cp_count >= 5 {
        CheckResult::pass("Checkpoints", &format!("OK — {}/5 checkpoints found", cp_count))
            .with_details(details)
    } else if cp_count >= 3 {
        CheckResult::warn("Checkpoints", &format!("{}/5 checkpoints — need {} more", cp_count, 5 - cp_count))
            .with_details(details)
    } else {
        CheckResult::fail("Checkpoints", &format!("Only {}/5 checkpoints", cp_count))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Invariant: Molecule only from encode_codepoint / LCA
// ═══════════════════════════════════════════════════════════════════

pub fn check_molecule_not_handwritten(root: &Path) -> CheckResult {
    println!("[8/37] Invariant — Molecule not handwritten (QT④)...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    // Find Molecule struct constructions outside of encoder/lca
    let mut violations = Vec::new();

    for (path, content) in &files {
        let ps = path.to_str().unwrap_or("");
        // Skip: encoder.rs, lca.rs, test files, vm (PushMol allowed), vsdf (FFR allowed)
        if ps.contains("encoder") || ps.contains("lca")
            || ps.contains("test") || ps.contains("vm.rs")
            || ps.contains("vsdf")
        {
            continue;
        }
        // Skip test helper functions (fn test_mol is OK in test context)
        let in_test_mod = content.contains("#[cfg(test)]");
        if in_test_mod {
            continue;
        }

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            // Detect: Molecule { shape: ..., relation: ... }  — handwritten construction
            if trimmed.contains("Molecule {")
                && (trimmed.contains("shape:") || trimmed.contains("relation:"))
                && !trimmed.contains("//")
            {
                let rel = path.strip_prefix(root).unwrap_or(path);
                violations.push(format!("{}:{} — {}", rel.display(), i + 1, trimmed));
            }
        }
    }

    // Verify encode_codepoint exists
    let encode_hits = grep_pattern(&files, "encode_codepoint");

    if !violations.is_empty() {
        CheckResult::fail("QT④ Molecule", &format!("{} handwritten Molecule(s) found", violations.len()))
            .with_details(violations)
    } else {
        CheckResult::pass("QT④ Molecule", &format!(
            "OK — 0 handwritten, {} encode_codepoint refs",
            encode_hits.len()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Invariant: Append-only — NO delete, NO overwrite (QT⑧⑨⑩)
// ═══════════════════════════════════════════════════════════════════

pub fn check_append_only(root: &Path) -> CheckResult {
    println!("[9/37] Invariant — Append-only (QT⑧⑨⑩)...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let mut violations = Vec::new();

    for (path, content) in &files {
        let ps = path.to_str().unwrap_or("");
        if ps.contains("test") || ps.contains("migrate") {
            continue;
        }
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            // Detect dangerous patterns in storage code
            if ps.contains("storage") || ps.contains("registry") || ps.contains("writer") {
                if trimmed.contains(".remove(") && !trimmed.contains("// allowed") {
                    let rel = path.strip_prefix(root).unwrap_or(path);
                    violations.push(format!("{}:{} — remove(): {}", rel.display(), i + 1, trimmed));
                }
                if trimmed.contains("seek(SeekFrom::Start(0))") || trimmed.contains("set_len(0)") {
                    let rel = path.strip_prefix(root).unwrap_or(path);
                    violations.push(format!("{}:{} — overwrite: {}", rel.display(), i + 1, trimmed));
                }
            }
        }
    }

    // Verify append exists
    let append_hits = grep_pattern(&files, "append");

    if !violations.is_empty() {
        CheckResult::fail("QT⑧⑨⑩ Append-only", &format!("{} delete/overwrite found", violations.len()))
            .with_details(violations)
    } else {
        CheckResult::pass("QT⑧⑨⑩ Append-only", &format!("OK — 0 violations, {} append refs", append_hits.len()))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Invariant: Agent tiers (QT⑮)
// AAM↔Chief ✅  Chief↔Chief ✅  Chief↔Worker ✅
// AAM↔Worker ❌  Worker↔Worker ❌
// ═══════════════════════════════════════════════════════════════════

pub fn check_agent_tiers(root: &Path) -> CheckResult {
    println!("[10/37] Invariant — Agent tiers (QT⑮)...");
    let agents_dir = root.join("crates/agents");
    let files = scan_rs_files(&agents_dir);

    let mut violations = Vec::new();

    // Check worker files don't communicate with AAM directly
    for (path, content) in &files {
        let ps = path.to_str().unwrap_or("");
        if ps.contains("test") {
            continue;
        }

        if ps.contains("worker") {
            // Workers should NOT reference AAM directly
            for (i, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if (trimmed.contains("aam") || trimmed.contains("Aam") || trimmed.contains("AAM"))
                    && !trimmed.contains("aam_decision")  // receiving is OK (via Chief)
                    && !trimmed.contains("// via chief")
                {
                    // Only flag direct sends
                    if trimmed.contains("send_to_aam") || trimmed.contains("aam.send") {
                        let rel = path.strip_prefix(root).unwrap_or(path);
                        violations.push(format!("{}:{} — Worker↔AAM direct: {}", rel.display(), i + 1, trimmed));
                    }
                }
            }
        }
    }

    // Verify hierarchy exists
    let chief_hits = grep_pattern(&files, "Chief");
    let worker_hits = grep_pattern(&files, "Worker");

    if !violations.is_empty() {
        CheckResult::fail("QT⑮ Agent tiers", &format!("{} tier violation(s)", violations.len()))
            .with_details(violations)
    } else {
        CheckResult::pass("QT⑮ Agent tiers", &format!(
            "OK — Chiefs: {} refs, Workers: {} refs, 0 tier violations",
            chief_hits.len(), worker_hits.len()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Invariant: L0 không import L1 (QT⑭)
// ═══════════════════════════════════════════════════════════════════

pub fn check_l0_no_import_l1(root: &Path) -> CheckResult {
    println!("[11/37] Invariant — L0 does not import L1 (QT⑭)...");
    // L0 crates: ucd, olang
    // L1 crates: silk, context, agents, memory, runtime
    let l0_crates = ["ucd", "olang"];
    let l1_crates = ["silk", "context", "agents", "memory", "runtime"];

    let mut violations = Vec::new();

    for l0 in &l0_crates {
        let cargo_path = root.join(format!("crates/{}/Cargo.toml", l0));
        if !cargo_path.exists() {
            continue;
        }
        let content = match std::fs::read_to_string(&cargo_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for l1 in &l1_crates {
            // Check if L0 crate depends on L1 crate
            if content.contains(&format!("{} ", l1))
                || content.contains(&format!("{}=", l1))
                || content.contains(&format!("{} =", l1))
            {
                violations.push(format!("crates/{}/Cargo.toml imports L1 crate '{}'", l0, l1));
            }
        }
    }

    if !violations.is_empty() {
        CheckResult::fail("QT⑭ L0→L1", &format!("{} L0 imports L1", violations.len()))
            .with_details(violations)
    } else {
        CheckResult::pass("QT⑭ L0→L1", "OK — L0 (ucd, olang) does not import L1")
    }
}

// ═══════════════════════════════════════════════════════════════════
// Invariant: Skill stateless (QT⑲⑳㉑㉒㉓)
// ═══════════════════════════════════════════════════════════════════

pub fn check_skill_stateless(root: &Path) -> CheckResult {
    println!("[12/37] Invariant — Skill stateless (QT⑲-㉓)...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let mut violations = Vec::new();

    for (path, content) in &files {
        let ps = path.to_str().unwrap_or("");
        if !ps.contains("skill") || ps.contains("test") {
            continue;
        }
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            // Skill holding Agent reference
            if trimmed.contains("agent:") && trimmed.contains("&") && trimmed.contains("Agent") {
                let rel = path.strip_prefix(root).unwrap_or(path);
                violations.push(format!("{}:{} — Skill holds Agent ref: {}", rel.display(), i + 1, trimmed));
            }
            // Skill with HashMap/Vec state field (struct with cache)
            if trimmed.contains("cache:") && trimmed.contains("HashMap") {
                let rel = path.strip_prefix(root).unwrap_or(path);
                violations.push(format!("{}:{} — Skill has cache state: {}", rel.display(), i + 1, trimmed));
            }
        }
    }

    // Check that skills use ExecContext
    let exec_ctx_hits = grep_pattern(&files, "ExecContext");

    if !violations.is_empty() {
        CheckResult::fail("QT⑲-㉓ Skill stateless", &format!("{} stateful Skill(s)", violations.len()))
            .with_details(violations)
    } else {
        CheckResult::pass("QT⑲-㉓ Skill stateless", &format!(
            "OK — 0 stateful skills, {} ExecContext refs",
            exec_ctx_hits.len()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Invariant: Worker sends chain, not raw data (QT⑮ extension)
// ═══════════════════════════════════════════════════════════════════

pub fn check_worker_sends_chain(root: &Path) -> CheckResult {
    println!("[13/37] Invariant — Worker sends chain, not raw data...");
    let agents_dir = root.join("crates/agents");
    let files = scan_rs_files(&agents_dir);

    let mut violations = Vec::new();

    for (path, content) in &files {
        let ps = path.to_str().unwrap_or("");
        if !ps.contains("worker") || ps.contains("test") {
            continue;
        }
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            // Workers should not send raw bytes/images/audio
            if (trimmed.contains("send(") || trimmed.contains("chief."))
                && (trimmed.contains("raw_") || trimmed.contains("bytes")
                    || trimmed.contains("image_data") || trimmed.contains("audio_data"))
            {
                let rel = path.strip_prefix(root).unwrap_or(path);
                violations.push(format!("{}:{} — raw data send: {}", rel.display(), i + 1, trimmed));
            }
        }
    }

    // Verify ISLFrame usage
    let isl_hits = grep_pattern(&files, "ISLFrame");
    let chain_hits = grep_pattern(&files, "chain");

    if !violations.is_empty() {
        CheckResult::fail("Worker→chain", &format!("{} raw data send(s)", violations.len()))
            .with_details(violations)
    } else {
        CheckResult::pass("Worker→chain", &format!(
            "OK — 0 raw sends, {} ISLFrame refs, {} chain refs",
            isl_hits.len(), chain_hits.len()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Data Integrity: json/udc_utf32.json
// ═══════════════════════════════════════════════════════════════════

pub fn check_udc_utf32_data(root: &Path) -> CheckResult {
    println!("[14/37] Data — json/udc_utf32.json integrity...");

    let json_path = root.join("json/udc_utf32_compact.json");
    let bin_path = root.join("json/udc_p_table.bin");

    if !json_path.exists() {
        return CheckResult::fail("UDC Data", "json/udc_utf32_compact.json not found — run tools/build_udc/step1-6");
    }
    if !bin_path.exists() {
        return CheckResult::fail("UDC Data", "json/udc_p_table.bin not found");
    }

    // Parse JSON (lightweight check — just verify structure)
    let content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("UDC Data", &format!("Cannot read JSON: {}", e)),
    };

    let mut details = Vec::new();

    // Check key fields exist
    let has_protocol = content.contains("UTF32-SDF-INTEGRATOR");
    let has_planes = content.contains("\"planes\"");
    let has_p_layout = content.contains("[S:4][R:4][V:3][A:3][T:2]");
    let has_aliases = content.contains("\"aliases\"");
    let _has_no_toplevel_name = !content.contains("\"name\":\"FIRE\"");

    // Check specific emoji anchors
    let has_fire = content.contains("\"1F525\"");
    let has_smile = content.contains("\"1F60A\"");
    let has_heart = content.contains("\"2764\"");

    // Check name = codepoint (no top-level "name" field with English name)
    // In the new format, "name" should only appear inside aliases.en.name
    let name_in_aliases = content.contains("\"aliases\":{\"en\":{\"name\":");

    details.push(format!("Protocol UTF32-SDF-INTEGRATOR: {}", if has_protocol { "✅" } else { "❌" }));
    details.push(format!("Planes structure: {}", if has_planes { "✅" } else { "❌" }));
    details.push(format!("P layout [S:4][R:4][V:3][A:3][T:2]: {}", if has_p_layout { "✅" } else { "❌" }));
    details.push(format!("Aliases (en/vi): {}", if has_aliases { "✅" } else { "❌" }));
    details.push(format!("EN name in aliases (not top-level): {}", if name_in_aliases { "✅" } else { "❌" }));
    details.push(format!("Anchor 🔥 1F525: {}", if has_fire { "✅" } else { "❌" }));
    details.push(format!("Anchor 😊 1F60A: {}", if has_smile { "✅" } else { "❌" }));
    details.push(format!("Anchor ❤ 2764: {}", if has_heart { "✅" } else { "❌" }));

    // Binary table check
    let bin_data = match std::fs::read(&bin_path) {
        Ok(d) => d,
        Err(e) => return CheckResult::fail("UDC Data", &format!("Cannot read binary: {}", e)),
    };
    let bin_count = if bin_data.len() >= 4 {
        u32::from_le_bytes([bin_data[0], bin_data[1], bin_data[2], bin_data[3]])
    } else {
        0
    };
    details.push(format!("Binary table entries: {} (expected ~41338)", bin_count));
    let bin_ok = bin_count > 40000 && bin_count < 50000;

    let json_size_mb = content.len() as f64 / 1024.0 / 1024.0;
    details.push(format!("JSON size: {:.1} MB", json_size_mb));

    let all_ok = has_protocol && has_planes && has_p_layout && has_aliases
        && name_in_aliases && has_fire && has_smile && has_heart && bin_ok;

    if all_ok {
        CheckResult::pass("UDC Data", &format!("OK — {} entries, {:.1} MB", bin_count, json_size_mb))
            .with_details(details)
    } else {
        CheckResult::fail("UDC Data", "Data integrity issues found")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// DEEP CHECK: P_weight — Molecule struct phải dùng packed u16
// v2 spec: [S:4][R:4][V:3][A:3][T:2] = 16 bits = 2 bytes
// ═══════════════════════════════════════════════════════════════════

pub fn check_pweight_molecule_struct(root: &Path) -> CheckResult {
    println!("[15/37] DEEP — Molecule struct P_weight layout...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");

    if !mol_path.exists() {
        return CheckResult::fail("P_weight Molecule", "molecular.rs not found");
    }

    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("P_weight Molecule", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // Check 1: Molecule struct still uses 5 separate u8 fields?
    let has_shape_u8 = content.contains("pub shape: u8");
    let has_relation_u8 = content.contains("pub relation: u8");
    let has_time_u8 = content.contains("pub time: u8");

    // Check 2: to_bytes returns [u8; 5]?
    let has_5byte_serialize = content.contains("[u8; 5]");

    // Check 3: Has packed u16 p_packed field?
    let has_p_packed = content.contains("p_packed: u16") || content.contains("p: u16");

    // Check 4: chain_hash uses 5 bytes?
    let has_5byte_hash = content.contains("chain_hash(&self.to_bytes())");

    if has_p_packed && !has_5byte_serialize {
        details.push("Molecule has packed u16 P_weight: ✅".into());
        details.push("No [u8;5] serialization: ✅".into());
        CheckResult::pass("P_weight Molecule", "OK — Molecule uses packed u16 (v2)")
            .with_details(details)
    } else {
        if has_shape_u8 && has_relation_u8 && has_time_u8 {
            details.push("Molecule uses 5 × u8 fields (shape, relation, V, A, time): ❌ LEGACY".into());
        }
        if has_5byte_serialize {
            details.push("to_bytes() → [u8; 5]: ❌ should be u16".into());
        }
        if has_5byte_hash {
            details.push("chain_hash uses fnv1a([u8;5]): ❌ should use u16".into());
        }
        if !has_p_packed {
            details.push("No p_packed: u16 field: ❌ need packed P_weight".into());
        }
        details.push("Ref: plans/PLAN_PWEIGHT_MIGRATION.md".into());
        CheckResult::fail("P_weight Molecule", "Molecule still uses 5B layout — v2 requires 2B packed u16")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// DEEP CHECK: CompactQR bit layout phải = [S:4][R:4][V:3][A:3][T:2]
// Code hiện tại: [S:3][R:3][T:3][V:4][A:3] — SAI
// ═══════════════════════════════════════════════════════════════════

pub fn check_pweight_compactqr_layout(root: &Path) -> CheckResult {
    println!("[16/37] DEEP — CompactQR bit layout vs v2...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");

    if !mol_path.exists() {
        return CheckResult::fail("P_weight CompactQR", "molecular.rs not found");
    }

    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("P_weight CompactQR", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // v2 layout: (s << 12) | (r << 8) | (v << 5) | (a << 2) | t
    let has_v2_layout = content.contains("s << 12")
        && content.contains("r << 8")
        && content.contains("v << 5")
        && content.contains("a << 2");

    // Current wrong layout: (s << 13) | (r << 10) | (t << 7) | (v << 3) | a
    let has_wrong_layout = content.contains("s << 13")
        || content.contains("r << 10")
        || content.contains("t << 7");

    if has_v2_layout && !has_wrong_layout {
        details.push("Bit layout: [S:4][R:4][V:3][A:3][T:2] ✅".into());
        CheckResult::pass("P_weight CompactQR", "OK — bit layout matches v2 spec")
            .with_details(details)
    } else if has_wrong_layout {
        details.push("Current: [S:3][R:3][T:3][V:4][A:3] — WRONG ❌".into());
        details.push("Expected: [S:4][R:4][V:3][A:3][T:2] — v2 spec".into());
        details.push("s << 13 → should be s << 12".into());
        details.push("r << 10 → should be r << 8".into());
        details.push("t << 7 → T should be last 2 bits, not middle".into());
        details.push("Ref: plans/PLAN_PWEIGHT_MIGRATION.md Phase 1".into());
        CheckResult::fail("P_weight CompactQR", "Bit layout WRONG — [S:3][R:3][T:3][V:4][A:3] vs v2 [S:4][R:4][V:3][A:3][T:2]")
            .with_details(details)
    } else {
        details.push("Cannot determine bit layout — verify manually".into());
        CheckResult::warn("P_weight CompactQR", "Cannot detect bit layout pattern")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// DEEP CHECK: UCD build.rs sinh UcdEntry — phải có packed u16
// ═══════════════════════════════════════════════════════════════════

pub fn check_pweight_ucd_build(root: &Path) -> CheckResult {
    println!("[17/37] DEEP — UCD build.rs P_weight format...");
    let build_path = root.join("crates/ucd/build.rs");

    if !build_path.exists() {
        return CheckResult::fail("P_weight UCD", "build.rs not found");
    }

    let content = match std::fs::read_to_string(&build_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("P_weight UCD", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // Check if build.rs generates u16 packed P
    let has_u16_p = content.contains("p_packed: u16") || content.contains("p: u16");

    // Check if it still uses 5 separate fields
    let has_5_fields = content.contains("shape: u8")
        && content.contains("relation: u8")
        && content.contains("valence: u8")
        && content.contains("arousal: u8")
        && content.contains("time: u8");

    // Check if it reads from udc_p_table.bin
    let reads_p_table = content.contains("udc_p_table.bin") || content.contains("p_table");

    // Check chain_hash uses 5 bytes
    let hash_5b = content.contains("fn chain_hash(shape: u8, relation: u8, valence: u8, arousal: u8, time: u8)");

    if has_u16_p && !has_5_fields {
        details.push("UcdEntry has packed u16 P: ✅".into());
        CheckResult::pass("P_weight UCD", "OK — build.rs generates packed u16")
            .with_details(details)
    } else {
        if has_5_fields {
            details.push("UcdEntry still uses 5 × u8 (shape, relation, V, A, T): ❌".into());
        }
        if !reads_p_table {
            details.push("Does not read udc_p_table.bin: ❌ (should use pre-packed P)".into());
        }
        if hash_5b {
            details.push("chain_hash(shape, relation, valence, arousal, time) uses 5B: ❌".into());
        }
        if !has_u16_p {
            details.push("No u16 packed P field: ❌".into());
        }
        details.push("Ref: plans/PLAN_PWEIGHT_MIGRATION.md Phase 2-3".into());
        CheckResult::fail("P_weight UCD", "build.rs still generates 5B UcdEntry — v2 requires packed u16")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// DEEP CHECK: KnowTree size = 65,536 × 2B = 128 KB (v2)
// Current: 65,536 × 5B = 320 KB
// ═══════════════════════════════════════════════════════════════════

pub fn check_pweight_knowtree_size(root: &Path) -> CheckResult {
    println!("[18/37] DEEP — KnowTree node size...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let mut details = Vec::new();

    // Check KnowTree stores Molecule (5B+metadata) or u16 (2B)
    let knowtree_molecule = grep_pattern(&files, "Vec<Molecule>");
    let knowtree_u16 = grep_pattern(&files, "Vec<u16>");

    // Check FormulaTable size constant
    let _formula_65536 = grep_pattern(&files, "65_536");
    let _formula_64k = grep_pattern(&files, "65536");

    // Check Molecule in FormulaTable (it holds Vec<Molecule>)
    let formula_table_mol: Vec<_> = knowtree_molecule.iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            ps.contains("molecular") && l.contains("FormulaTable")
                || l.contains("formula") || l.contains("table")
        })
        .collect();

    if !formula_table_mol.is_empty() {
        details.push("FormulaTable stores Vec<Molecule> (5B+ each): ❌".into());
        details.push("v2 spec: KnowTree node = 2B (P_weight packed u16)".into());
        details.push("Current: 65,536 × ~11B = ~704 KB (Molecule is 11 bytes in struct)".into());
        details.push("Expected: 65,536 × 2B = 128 KB".into());
        CheckResult::fail("P_weight KnowTree", "FormulaTable uses Molecule (5B+) — v2 requires u16 (2B) per node")
            .with_details(details)
    } else if !knowtree_u16.is_empty() {
        details.push("KnowTree stores u16 entries: ✅".into());
        CheckResult::pass("P_weight KnowTree", "OK — KnowTree uses u16 (2B per node)")
            .with_details(details)
    } else {
        details.push("Cannot determine KnowTree node type".into());
        CheckResult::warn("P_weight KnowTree", "Cannot verify KnowTree node size — check manually")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// WIRING CHECK: Dream → AAM → QR Promotion chain
// v2 spec: Dream sinh proposal → AAM review → approve → QR promote
// ═══════════════════════════════════════════════════════════════════

pub fn check_wiring_dream_aam(root: &Path) -> CheckResult {
    println!("[19/37] WIRING — Dream → AAM → QR promotion...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let mut details = Vec::new();

    // Check 1: Dream::run() exists and is called
    let dream_run = grep_pattern(&files, "run_dream");
    let dream_exists = !dream_run.is_empty();

    // Check 2: AAM::review() is called from somewhere (not just defined)
    let aam_review_def = grep_pattern(&files, "fn review");
    let aam_review_call: Vec<_> = grep_pattern(&files, ".review(")
        .into_iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            !ps.contains("test") && !l.trim().starts_with("//") && !l.contains("fn review")
        })
        .collect();

    // Check 3: Proposals are submitted to AAM
    let submit_proposal = grep_pattern(&files, "submit_proposal");
    let proposal_to_aam = grep_pattern(&files, "aam.review");

    // Check 4: QR promotion after AAM approval
    let from_approved = grep_pattern(&files, "from_approved");
    let _promote_qr = grep_pattern(&files, "promote");

    details.push(format!("Dream::run() called: {}", if dream_exists { "✅" } else { "❌" }));
    details.push(format!("AAM::review() defined: {} refs", aam_review_def.len()));
    details.push(format!("AAM::review() CALLED: {} refs", aam_review_call.len()));
    details.push(format!("submit_proposal → AAM: {} refs", submit_proposal.len() + proposal_to_aam.len()));
    details.push(format!("QRProposal::from_approved(): {} refs", from_approved.len()));

    let chain_complete = dream_exists
        && !aam_review_call.is_empty()
        && (!submit_proposal.is_empty() || !proposal_to_aam.is_empty());

    if chain_complete {
        CheckResult::pass("WIRING Dream→AAM", "OK — Dream → AAM → QR promotion chain complete")
            .with_details(details)
    } else {
        details.push("Dream sinh proposals nhưng KHÔNG submit vào AAM".into());
        details.push("AAM::review() KHÔNG được gọi → QR KHÔNG promote".into());
        details.push("→ KnowTree KHÔNG grow dài hạn".into());
        CheckResult::fail("WIRING Dream→AAM", "Dream→AAM→QR chain BROKEN — long-term learning disconnected")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// WIRING CHECK: EpistemicFirewall wired into response rendering
// v2 spec: Response phải qua epistemic level (Fact/Opinion/Hypothesis/Unknown)
// ═══════════════════════════════════════════════════════════════════

pub fn check_wiring_epistemic(root: &Path) -> CheckResult {
    println!("[20/37] WIRING — EpistemicFirewall in response...");
    let agents_dir = root.join("crates/agents");
    let runtime_dir = root.join("crates/runtime");

    let ag_files = scan_rs_files(&agents_dir);
    let rt_files = scan_rs_files(&runtime_dir);
    let all: Vec<_> = ag_files.iter().chain(rt_files.iter()).cloned().collect();

    let mut details = Vec::new();

    // Check: EpistemicFirewall::wrap() called outside test
    let wrap_calls: Vec<_> = grep_pattern(&all, "wrap(")
        .into_iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            !ps.contains("test")
                && l.contains("pistemic") || l.contains("firewall")
                || l.contains("Firewall")
        })
        .collect();

    // Check: EpistemicFirewall::should_answer() called outside test
    let should_answer: Vec<_> = grep_pattern(&all, "should_answer")
        .into_iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            !ps.contains("test") && !l.contains("fn should_answer")
        })
        .collect();

    // Check: epistemic level in response rendering
    let epistemic_render = grep_pattern(&all, "epistemic");

    details.push(format!("EpistemicFirewall::wrap() called: {} refs", wrap_calls.len()));
    details.push(format!("EpistemicFirewall::should_answer() called: {} refs", should_answer.len()));
    details.push(format!("Epistemic refs in pipeline: {} total", epistemic_render.len()));

    if !wrap_calls.is_empty() && !should_answer.is_empty() {
        CheckResult::pass("WIRING Epistemic", "OK — EpistemicFirewall wired into response")
            .with_details(details)
    } else {
        details.push("EpistemicFirewall defined but NOT called from pipeline".into());
        details.push("Response không có epistemic level (Fact/Opinion/Unknown)".into());
        CheckResult::fail("WIRING Epistemic", "EpistemicFirewall NOT wired — response lacks epistemic grading")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// WIRING CHECK: sentence_affect_unified() thay vì sentence_affect()
// v2 spec: Emotion pipeline phải dùng unified (implicit 5D + Hebbian)
// ═══════════════════════════════════════════════════════════════════

pub fn check_wiring_unified_affect(root: &Path) -> CheckResult {
    println!("[21/37] WIRING — sentence_affect_unified() usage...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let mut details = Vec::new();

    // Check: sentence_affect_unified exists
    let unified_def = grep_pattern(&files, "fn sentence_affect_unified");

    // Check: sentence_affect_unified called from runtime/agents (not test)
    let unified_calls: Vec<_> = grep_pattern(&files, "sentence_affect_unified")
        .into_iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            !ps.contains("test") && !l.contains("fn sentence_affect_unified")
        })
        .collect();

    // Check: old sentence_affect still used
    let old_calls: Vec<_> = grep_pattern(&files, "sentence_affect(")
        .into_iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            (ps.contains("runtime") || ps.contains("agents"))
                && !ps.contains("test")
                && !l.contains("fn sentence_affect")
                && !l.contains("unified")
        })
        .collect();

    details.push(format!("sentence_affect_unified() defined: {}", if !unified_def.is_empty() { "✅" } else { "❌" }));
    details.push(format!("sentence_affect_unified() called: {} refs", unified_calls.len()));
    details.push(format!("OLD sentence_affect() still called: {} refs", old_calls.len()));
    for (p, line, text) in &old_calls {
        let rel = p.strip_prefix(root).unwrap_or(p);
        details.push(format!("  OLD: {}:{} — {}", rel.display(), line, text));
    }

    if !unified_calls.is_empty() && old_calls.is_empty() {
        CheckResult::pass("WIRING Unified Affect", "OK — using sentence_affect_unified()")
            .with_details(details)
    } else if !unified_def.is_empty() && unified_calls.is_empty() {
        details.push("sentence_affect_unified() EXISTS but NEVER CALLED".into());
        details.push("Pipeline still uses OLD sentence_affect() (Hebbian-only, no implicit 5D)".into());
        CheckResult::fail("WIRING Unified Affect", "sentence_affect_unified() defined but NOT wired — pipeline uses OLD version")
            .with_details(details)
    } else {
        CheckResult::warn("WIRING Unified Affect", "sentence_affect_unified() not found — may not be implemented yet")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// WIRING CHECK: Word selection pipeline (target_affect → select_words)
// v2 spec: Response rendering should use emotion-aware word selection
// ═══════════════════════════════════════════════════════════════════

pub fn check_wiring_word_selection(root: &Path) -> CheckResult {
    println!("[22/37] WIRING — Word selection pipeline...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let mut details = Vec::new();

    // Check: target_affect / select_words called from outside context/
    let target_calls: Vec<_> = grep_pattern(&files, "target_affect")
        .into_iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            !ps.contains("test") && !l.contains("fn target_affect")
                && (ps.contains("runtime") || ps.contains("agents"))
        })
        .collect();

    let select_calls: Vec<_> = grep_pattern(&files, "select_words")
        .into_iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            !ps.contains("test") && !l.contains("fn select_words")
                && (ps.contains("runtime") || ps.contains("agents"))
        })
        .collect();

    let affect_comp: Vec<_> = grep_pattern(&files, "affect_components")
        .into_iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            !ps.contains("test") && !l.contains("fn affect_components")
                && (ps.contains("runtime") || ps.contains("agents"))
        })
        .collect();

    details.push(format!("target_affect() called from runtime/agents: {} refs", target_calls.len()));
    details.push(format!("select_words() called from runtime/agents: {} refs", select_calls.len()));
    details.push(format!("affect_components() called from runtime/agents: {} refs", affect_comp.len()));

    let any_wired = !target_calls.is_empty() || !select_calls.is_empty() || !affect_comp.is_empty();

    if any_wired {
        CheckResult::pass("WIRING Word Select", "OK — emotion-aware word selection wired")
            .with_details(details)
    } else {
        details.push("Word selection pipeline defined in context/ but NEVER called from runtime/agents".into());
        details.push("Response rendering does NOT use emotion-aware word selection".into());
        CheckResult::fail("WIRING Word Select", "Word selection pipeline NOT wired — response ignores emotion-aware words")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// AUDIT #2: ShapeBase = 8, v2 = 18 SDF primitives
// Union/Intersect/Subtract = CSG ops, not shapes
// ═══════════════════════════════════════════════════════════════════

pub fn check_shapebase_18sdf(root: &Path) -> CheckResult {
    println!("[23/37] AUDIT — ShapeBase 8 vs v2 18 SDF...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");

    if !mol_path.exists() {
        return CheckResult::fail("ShapeBase 18 SDF", "molecular.rs not found");
    }

    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("ShapeBase 18 SDF", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // v2 requires 18 SDF primitives
    let v2_shapes = [
        "Sphere", "Box", "Capsule", "Plane", "Torus", "Ellipsoid",
        "Cone", "Cylinder", "Octahedron", "Pyramid", "HexPrism",
        "Prism", "RoundBox", "Link", "Revolve", "Extrude",
        "CutSphere", "DeathStar",
    ];

    let mut found = 0;
    let mut missing = Vec::new();
    for shape in &v2_shapes {
        if content.contains(shape) {
            found += 1;
        } else {
            missing.push(*shape);
        }
    }

    // Check CSG ops wrongly in ShapeBase
    let has_union_shape = content.contains("Union") && content.contains("ShapeBase");
    let has_intersect_shape = content.contains("Intersect") && content.contains("ShapeBase");
    let has_subtract_shape = content.contains("Subtract") && content.contains("ShapeBase");
    let csg_in_shape = has_union_shape || has_intersect_shape || has_subtract_shape;

    details.push(format!("SDF primitives found: {}/18", found));
    if !missing.is_empty() {
        details.push(format!("Missing: {}", missing.join(", ")));
    }
    if csg_in_shape {
        details.push("CSG ops (Union/Intersect/Subtract) in ShapeBase: ❌ should be separate".into());
    }

    if found >= 18 && !csg_in_shape {
        CheckResult::pass("ShapeBase 18 SDF", &format!("OK — {}/18 SDF primitives", found))
            .with_details(details)
    } else {
        CheckResult::fail("ShapeBase 18 SDF", &format!(
            "Only {}/18 SDF primitives{}", found,
            if csg_in_shape { " + CSG ops wrongly in ShapeBase" } else { "" }
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// AUDIT #3: KnowTree = array 65,536 × 2B, not hash-based
// ═══════════════════════════════════════════════════════════════════

pub fn check_knowtree_array(root: &Path) -> CheckResult {
    println!("[24/37] AUDIT — KnowTree = array, not hash...");
    let olang_dir = root.join("crates/olang");
    let files = scan_rs_files(&olang_dir);

    let mut details = Vec::new();

    // Check: KnowTree uses array index (codepoint → P_weight) vs hash-based
    let hash_lookup = grep_pattern(&files, "chain_hash");
    let compact_node = grep_pattern(&files, "CompactNode");
    let slim_node = grep_pattern(&files, "SlimNode");
    let array_index = grep_pattern(&files, "[codepoint]");  // array index syntax

    // Check: size is 65,536 × 2B = 128KB?
    let tiered_store = grep_pattern(&files, "TieredStore");

    let is_hash_based = !hash_lookup.is_empty() && (!compact_node.is_empty() || !slim_node.is_empty());
    let is_array_based = !array_index.is_empty() && tiered_store.is_empty();

    details.push(format!("Hash-based lookup (chain_hash): {} refs", hash_lookup.len()));
    details.push(format!("CompactNode (hash 8B + mol + meta): {} refs", compact_node.len()));
    details.push(format!("SlimNode: {} refs", slim_node.len()));
    details.push(format!("TieredStore: {} refs", tiered_store.len()));

    if is_array_based {
        CheckResult::pass("KnowTree Array", "OK — array-based O(1) lookup by codepoint")
            .with_details(details)
    } else if is_hash_based {
        details.push("v2 spec: KnowTree[codepoint] → P_weight, O(1) array lookup".into());
        details.push("Current: chain_hash → CompactNode → Molecule, O(log n) hash lookup".into());
        details.push("Expected: array 65,536 × 2B = 128 KB".into());
        CheckResult::fail("KnowTree Array", "KnowTree is hash-based — v2 requires array 65,536 × 2B")
            .with_details(details)
    } else {
        CheckResult::warn("KnowTree Array", "Cannot determine KnowTree type")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// AUDIT #4: MolecularChain = Vec<Molecule>, v2 = Vec<u16>
// ═══════════════════════════════════════════════════════════════════

pub fn check_chain_u16(root: &Path) -> CheckResult {
    println!("[25/37] AUDIT — MolecularChain = Vec<u16> vs Vec<Molecule>...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");

    if !mol_path.exists() {
        return CheckResult::fail("Chain Vec<u16>", "molecular.rs not found");
    }

    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("Chain Vec<u16>", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // Check: MolecularChain wraps Vec<Molecule> (11B each) or Vec<u16> (2B each)?
    let has_vec_mol = content.contains("Vec<Molecule>") && content.contains("MolecularChain");
    let has_vec_u16 = content.contains("Vec<u16>") && content.contains("MolecularChain");

    if has_vec_u16 && !has_vec_mol {
        details.push("MolecularChain = Vec<u16>: ✅ (2B/link)".into());
        CheckResult::pass("Chain Vec<u16>", "OK — chain links are u16 codepoint references")
            .with_details(details)
    } else if has_vec_mol {
        details.push("MolecularChain = Vec<Molecule>: ❌ (11B/link)".into());
        details.push("v2 spec: chain link = u16 codepoint → KnowTree[cp]".into());
        details.push("Current: chain link = full Molecule embedded (5.5x overhead)".into());
        details.push("DNA analogy: chain = sequence of REFERENCES, not VALUES".into());
        CheckResult::fail("Chain Vec<u16>", "MolecularChain wraps Vec<Molecule> (11B/link) — v2 requires Vec<u16> (2B/link)")
            .with_details(details)
    } else {
        CheckResult::warn("Chain Vec<u16>", "Cannot determine chain type")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// AUDIT #5: LCA compose rules — amplify/Union/max/dominant
// Code uses weighted average for ALL dimensions
// ═══════════════════════════════════════════════════════════════════

pub fn check_lca_compose_rules(root: &Path) -> CheckResult {
    println!("[26/37] AUDIT — LCA compose rules vs v2...");
    let lca_path = root.join("crates/olang/src/mol/lca.rs");

    if !lca_path.exists() {
        return CheckResult::fail("LCA Compose", "lca.rs not found");
    }

    let content = match std::fs::read_to_string(&lca_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("LCA Compose", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // v2 rules:
    //   S = Union(A.S, B.S)
    //   R = Compose (fixed value)
    //   V = amplify(A.V, B.V, w_AB) — NOT average
    //   A = max(A.A, B.A)
    //   T = dominant(A.T, B.T)

    let has_wavg = content.contains("mode_or_wavg");
    let has_union = content.contains("Union") || content.contains("union");
    let has_amplify = content.contains("amplify");
    let has_max_arousal = content.contains("max") && content.contains("arousal");
    let has_dominant = content.contains("dominant");

    // Check each dimension
    let s_ok = has_union;
    let r_ok = content.contains("Compose") || content.contains("compose");
    let v_ok = has_amplify && !has_wavg;  // V must NOT use wavg
    let a_ok = has_max_arousal;
    let t_ok = has_dominant;

    details.push(format!("S = Union(): {}", if s_ok { "✅" } else { "❌ uses mode_or_wavg" }));
    details.push(format!("R = Compose: {}", if r_ok { "✅" } else { "❌ uses mode_or_wavg" }));
    details.push(format!("V = amplify(): {}", if v_ok { "✅" } else { "❌ uses mode_or_wavg (CRITICAL)" }));
    details.push(format!("A = max(): {}", if a_ok { "✅" } else { "❌ uses mode_or_wavg" }));
    details.push(format!("T = dominant(): {}", if t_ok { "✅" } else { "❌ uses mode_or_wavg" }));

    if has_wavg {
        details.push("mode_or_wavg() found — used for ALL dimensions: ❌".into());
        details.push("v2: each dimension has DIFFERENT compose rule".into());
    }

    let ok_count = [s_ok, r_ok, v_ok, a_ok, t_ok].iter().filter(|&&x| x).count();

    if ok_count == 5 {
        CheckResult::pass("LCA Compose", "OK — all 5 dimensions use correct compose rules")
            .with_details(details)
    } else {
        CheckResult::fail("LCA Compose", &format!(
            "LCA uses mode_or_wavg for ALL dims — {}/5 correct (v2: Union/Compose/amplify/max/dominant)",
            ok_count
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// AUDIT #7: UCD blocks — 29 ranges vs v3 59 blocks
// ═══════════════════════════════════════════════════════════════════

pub fn check_ucd_block_count(root: &Path) -> CheckResult {
    println!("[27/37] AUDIT — UCD blocks 29 vs v3 59...");
    let build_path = root.join("crates/ucd/build.rs");

    if !build_path.exists() {
        return CheckResult::fail("UCD Blocks", "build.rs not found");
    }

    let content = match std::fs::read_to_string(&build_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("UCD Blocks", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // Count Unicode range definitions (0x????..0x???? or ..=)
    let _range_count = content.matches("0x").count() / 2;  // approximate pairs

    // Check for specific missing blocks
    let has_braille = content.contains("2800") || content.contains("Braille");
    let has_ornamental = content.contains("1F650") || content.contains("Ornamental");
    let has_mahjong = content.contains("1F000") || content.contains("Mahjong");
    let has_domino = content.contains("1F030") || content.contains("Domino");
    let has_playing = content.contains("1F0A0") || content.contains("Playing");
    let has_znamenny = content.contains("1CF00") || content.contains("Znamenny");
    let has_byzantine = content.contains("1D000") || content.contains("Byzantine");

    // Check L0 anchor count — v3 says 8,846
    let table_size_match = content.contains("8846") || content.contains("8_846");

    details.push(format!("Braille Patterns: {}", if has_braille { "✅" } else { "❌ missing" }));
    details.push(format!("Ornamental Dingbats: {}", if has_ornamental { "✅" } else { "❌ missing" }));
    details.push(format!("Mahjong Tiles: {}", if has_mahjong { "✅" } else { "❌ missing" }));
    details.push(format!("Domino Tiles: {}", if has_domino { "✅" } else { "❌ missing" }));
    details.push(format!("Playing Cards: {}", if has_playing { "✅" } else { "❌ missing" }));
    details.push(format!("Znamenny Musical: {}", if has_znamenny { "✅" } else { "❌ missing" }));
    details.push(format!("Byzantine Musical: {}", if has_byzantine { "✅" } else { "❌ missing" }));
    details.push(format!("L0 anchor count = 8,846: {}", if table_size_match { "✅" } else { "❌" }));

    let missing_count = [has_braille, has_ornamental, has_mahjong, has_domino,
                         has_playing, has_znamenny, has_byzantine]
        .iter().filter(|&&x| !x).count();

    if missing_count == 0 && table_size_match {
        CheckResult::pass("UCD Blocks", "OK — all 59 blocks covered, 8,846 anchors")
            .with_details(details)
    } else {
        details.push(format!("{} key blocks missing — v3 requires 59 blocks / 8,846 L0 anchors", missing_count));
        CheckResult::fail("UCD Blocks", &format!(
            "{} key blocks missing, L0 anchors ≠ 8,846 — v3 requires 59 blocks",
            missing_count
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// OLANG #1: IR/Compiler — features parsed but NOT compiled
// ═══════════════════════════════════════════════════════════════════

pub fn check_olang_compile_gap(root: &Path) -> CheckResult {
    println!("[28/37] OLANG — Parsed-but-not-compiled features...");
    let ir_path = root.join("crates/olang/src/exec/ir.rs");

    if !ir_path.exists() {
        return CheckResult::fail("OLANG Compile Gap", "ir.rs not found");
    }

    let content = match std::fs::read_to_string(&ir_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("OLANG Compile Gap", &format!("Cannot read: {}", e)),
    };

    let syntax_path = root.join("crates/olang/src/lang/syntax.rs");
    let syntax = std::fs::read_to_string(&syntax_path).unwrap_or_default();

    let mut details = Vec::new();
    let mut gap_count = 0;

    let checks = [
        ("Break", "Break", "break statement"),
        ("Continue", "Continue", "continue statement"),
        ("StructDef", "StructDef", "struct definition"),
        ("EnumDef", "EnumDef", "enum definition"),
        ("TraitDef", "TraitDef", "trait definition"),
        ("ImplBlock", "ImplBlock", "impl block"),
        ("TryPropagate", "TryPropagate", "? try propagation"),
        ("UnwrapOr", "UnwrapOr", "?? unwrap"),
        ("FStr", "FStr", "f-string interpolation"),
        ("Tuple", "Tuple", "tuple expression"),
        ("Slice", "Slice", "array slicing"),
        ("FieldAssign", "FieldAssign", "struct field assignment"),
        ("IndexAssign", "IndexAssign", "array index assignment"),
        ("MethodCall", "MethodCall", "method call dispatch"),
    ];

    for (name, ast_pattern, desc) in &checks {
        let in_syntax = syntax.contains(ast_pattern);
        let in_ir = content.contains(&format!("{}(", name))
            || content.contains(&format!("{} {{", name))
            || content.contains(&format!("{} =>", name));

        if in_syntax && !in_ir {
            gap_count += 1;
            details.push(format!("❌ {} — parsed but NOT compiled: {}", name, desc));
        }
    }

    if gap_count == 0 {
        CheckResult::pass("OLANG Compile Gap", "OK — all parsed features compiled")
            .with_details(details)
    } else {
        details.push(format!("{} features parse OK then SILENTLY DROP — no error, no code", gap_count));
        CheckResult::fail("OLANG Compile Gap", &format!(
            "{} features parsed but NOT compiled — silent code loss", gap_count
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// OLANG #2: Stdlib missing builtins
// ═══════════════════════════════════════════════════════════════════

pub fn check_olang_stdlib_builtins(root: &Path) -> CheckResult {
    println!("[29/37] OLANG — Stdlib builtin availability...");
    let stdlib_dir = root.join("stdlib");
    let vm_path = root.join("crates/olang/src/exec/vm.rs");

    if !stdlib_dir.exists() {
        return CheckResult::fail("OLANG Stdlib", "stdlib/ not found");
    }
    if !vm_path.exists() {
        return CheckResult::fail("OLANG Stdlib", "vm.rs not found");
    }

    let vm_content = match std::fs::read_to_string(&vm_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("OLANG Stdlib", &format!("Cannot read vm.rs: {}", e)),
    };

    fn scan_ol_builtins(dir: &std::path::Path, results: &mut Vec<(String, String)>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_ol_builtins(&path, results);
            } else if path.extension().and_then(|e| e.to_str()) == Some("ol") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let fname = path.file_name().unwrap_or_default().to_str().unwrap_or("?").to_string();
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("//") { continue; }
                        for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_') {
                            if word.starts_with("__") && word.len() > 3 {
                                results.push((fname.clone(), word.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    let mut ol_builtins: Vec<(String, String)> = Vec::new();
    scan_ol_builtins(&stdlib_dir, &mut ol_builtins);

    let mut unique: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (file, name) in &ol_builtins {
        unique.entry(name.clone()).or_default().push(file.clone());
    }

    let mut details = Vec::new();
    let mut missing = 0;
    let mut found = 0;

    for (builtin, files) in &unique {
        let in_vm = vm_content.contains(&format!("\"{}\"", builtin)) || vm_content.contains(builtin);
        if in_vm {
            found += 1;
        } else {
            missing += 1;
            let dedup: std::collections::HashSet<_> = files.iter().collect();
            let file_list: Vec<_> = dedup.into_iter().collect();
            details.push(format!("❌ {} — used by {:?} but NOT in VM", builtin, file_list));
        }
    }

    details.insert(0, format!("Builtins in VM: {}, MISSING: {}", found, missing));

    if missing == 0 {
        CheckResult::pass("OLANG Stdlib", &format!("OK — all {} builtins in VM", found))
            .with_details(details)
    } else {
        CheckResult::fail("OLANG Stdlib", &format!(
            "{} builtins NOT in VM — stdlib will crash at runtime", missing
        ))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// OLANG #3: Handbook says 5B Molecule, v2 says 2B
// ═══════════════════════════════════════════════════════════════════

pub fn check_olang_handbook_vs_v2(root: &Path) -> CheckResult {
    println!("[30/37] OLANG — Handbook 5B vs v2 2B conflict...");

    let handbook = root.join("docs/olang_handbook.md");
    let handbook_content = std::fs::read_to_string(&handbook).unwrap_or_default();

    let mut details = Vec::new();

    let handbook_5b = handbook_content.contains("5 bytes");
    let handbook_2b = handbook_content.contains("2 bytes");

    if handbook_5b && !handbook_2b {
        details.push("Handbook: Molecule = 5 bytes [S][R][V][A][T]".into());
        details.push("v2 spec: P_weight = 2 bytes packed u16".into());
        details.push("Handbook NOT updated for v2 — will mislead contributors".into());
        CheckResult::fail("OLANG Handbook", "Handbook says 5B Molecule — conflicts with v2 (2B)")
            .with_details(details)
    } else if handbook_2b {
        CheckResult::pass("OLANG Handbook", "OK — handbook aligned with v2")
            .with_details(details)
    } else {
        CheckResult::warn("OLANG Handbook", "Cannot determine — check manually")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// OLANG #4: PushMol opcode = 5 params, v2 = 2B
// ═══════════════════════════════════════════════════════════════════

pub fn check_olang_pushmol(root: &Path) -> CheckResult {
    println!("[31/37] OLANG — PushMol opcode size...");
    let ir_path = root.join("crates/olang/src/exec/ir.rs");

    if !ir_path.exists() {
        return CheckResult::fail("OLANG PushMol", "ir.rs not found");
    }

    let content = match std::fs::read_to_string(&ir_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("OLANG PushMol", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();
    let has_5param = content.contains("PushMol(u8, u8, u8, u8, u8)");

    if has_5param {
        details.push("PushMol(u8, u8, u8, u8, u8) = 5 params: ❌".into());
        details.push("v2: should be PushMol(u16) = 1 packed param".into());
        details.push("Bytecode: [0x19][S][R][V][A][T]=6B → [0x19][P_hi][P_lo]=3B".into());
        CheckResult::fail("OLANG PushMol", "PushMol = 5 params — v2 requires packed u16")
            .with_details(details)
    } else {
        CheckResult::pass("OLANG PushMol", "OK — PushMol uses packed format")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// OLANG #5: Bootstrap compiler files exist
// ═══════════════════════════════════════════════════════════════════

pub fn check_olang_bootstrap(root: &Path) -> CheckResult {
    println!("[32/37] OLANG — Bootstrap compiler...");
    let bootstrap_dir = root.join("stdlib/bootstrap");

    let mut details = Vec::new();
    let expected = ["lexer.ol", "parser.ol", "semantic.ol", "codegen.ol"];
    let mut found = 0;

    for file in &expected {
        let path = bootstrap_dir.join(file);
        if path.exists() {
            found += 1;
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            details.push(format!("✅ {} — {} bytes", file, size));
        } else {
            details.push(format!("❌ {} — NOT FOUND", file));
        }
    }

    if found == 4 {
        CheckResult::pass("OLANG Bootstrap", &format!("{}/4 bootstrap files", found))
            .with_details(details)
    } else {
        CheckResult::fail("OLANG Bootstrap", &format!("Only {}/4 bootstrap files", found))
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// L0 CASCADE #1: Valence hardcode vs udc.json (E0.3)
// build.rs uses name heuristics, not json/udc.json data
// ═══════════════════════════════════════════════════════════════════

pub fn check_l0_valence_source(root: &Path) -> CheckResult {
    println!("[33/37] L0 — Valence source: heuristic vs udc.json...");
    let build_path = root.join("crates/ucd/build.rs");

    if !build_path.exists() {
        return CheckResult::fail("L0 Valence", "build.rs not found");
    }

    let content = match std::fs::read_to_string(&build_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("L0 Valence", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // Check: does build.rs read json/udc.json or json/udc_utf32_compact.json?
    let reads_json = content.contains("udc.json")
        || content.contains("udc_utf32")
        || content.contains("udc_p_table");

    // Check: does it use name-based heuristics?
    let has_heuristic = content.contains("contains(\"FIRE\")")
        || content.contains("contains(\"HEART\")")
        || content.contains("contains(\"SKULL\")");

    // E0.4: duplicate condition bug
    let has_piano_bug = content.contains("contains(\"PIANO\") && name.contains(\"PIANO\")");

    // E0.5: collision perturbation
    let has_perturb = content.contains("perturb") || content.contains("perturbation")
        || (content.contains("collision") && content.contains("valence"));

    details.push(format!("Reads json/udc.json data: {}", if reads_json { "✅" } else { "❌ NO — uses heuristic" }));
    details.push(format!("Name-based heuristic (FIRE/HEART/SKULL): {}", if has_heuristic { "❌ YES" } else { "✅ NO" }));
    if has_piano_bug {
        details.push("E0.4 BUG: PIANO && PIANO = always true (should check 2 occurrences)".into());
    }
    if has_perturb {
        details.push("E0.5: Collision perturb V/A ±1..±127 → breaks L0 anchor semantics".into());
    }

    if reads_json && !has_heuristic {
        CheckResult::pass("L0 Valence", "OK — reads udc.json, no name heuristics")
            .with_details(details)
    } else {
        details.push("v2: P_weight L0 từ tài liệu — json/udc.json có 323K dòng data".into());
        details.push("build.rs KHÔNG đọc json → dùng name matching → sai cho ngàn codepoints".into());
        CheckResult::fail("L0 Valence", "build.rs uses name heuristics, NOT json/udc.json data")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// L0 CASCADE #2: L0 seed = 35 nodes, v3 = 8,846 (E5.2 + E1.3)
// 44% of anchor points fallback to Sphere/neutral → meaningless
// ═══════════════════════════════════════════════════════════════════

pub fn check_l0_seed_count(root: &Path) -> CheckResult {
    println!("[34/37] L0 — Seed count: 35 vs 8,846...");
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);

    let mut details = Vec::new();

    // Check KnowTree seed count
    let seed_35 = grep_pattern(&files, "35 seeded");
    let seed_8846 = grep_pattern(&files, "8846");
    let seed_9k = grep_pattern(&files, "8_846");

    // Check UCD_TABLE size
    let _table_refs = grep_pattern(&files, "UCD_TABLE");

    // Check fallback defaults (E1.3)
    let fallback_sphere = grep_pattern(&files, "unwrap_or(0x01)");
    let fallback_neutral = grep_pattern(&files, "unwrap_or(0x80)");

    details.push(format!("'35 seeded' refs: {}", seed_35.len()));
    details.push(format!("'8846' or '8_846' refs: {}", seed_8846.len() + seed_9k.len()));
    details.push(format!("Fallback to Sphere (0x01): {} refs", fallback_sphere.len()));
    details.push(format!("Fallback to neutral (0x80): {} refs", fallback_neutral.len()));

    let has_correct_seed = !seed_8846.is_empty() || !seed_9k.is_empty();

    if has_correct_seed && seed_35.is_empty() {
        CheckResult::pass("L0 Seed Count", "OK — 8,846 L0 anchors")
            .with_details(details)
    } else {
        details.push("v3: 59 blocks × ~150 avg = 8,846 L0 anchor points".into());
        details.push("Current: 35 seeds → 8,811 missing → fallback Sphere/neutral".into());
        details.push("→ 44% L0 anchors = Sphere + neutral = MEANINGLESS".into());
        details.push("→ ALL distance comparisons using missing anchors = WRONG".into());
        CheckResult::fail("L0 Seed Count", "Only 35 L0 seeds — v3 requires 8,846 (4,184 codepoints missing)")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// L0 CASCADE #3: similarity() only uses 2/5 dims (E3.3)
// v2: strength = Σ match_d × precision_d (all 5 dims equal)
// ═══════════════════════════════════════════════════════════════════

pub fn check_l0_similarity_dims(root: &Path) -> CheckResult {
    println!("[35/37] L0 — similarity() dimensions...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");

    if !mol_path.exists() {
        return CheckResult::fail("L0 Similarity", "molecular.rs not found");
    }

    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("L0 Similarity", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    // Check: similarity_full weights
    let has_unequal_weights = content.contains("0.3") && content.contains("0.2") && content.contains("0.5");
    let _has_equal_weights = content.contains("0.2") && content.contains("/ 5");

    // Check: similarity() uses all 5 dims or just 2
    let _has_shape_only = content.contains("shape_base ==") || content.contains("shape_base ==");
    let _has_all_5 = content.contains("valence") && content.contains("arousal") && content.contains("time");

    if has_unequal_weights {
        details.push("similarity_full() weights: 0.3×shape + 0.2×relation + 0.5×emotion".into());
        details.push("v2 spec: strength = Σ match_d × precision_d (5 dims EQUAL weight)".into());
        details.push("Current weights favor emotion (0.5) over shape (0.3) — DIFFERENT from v2".into());
    }

    if has_unequal_weights {
        CheckResult::fail("L0 Similarity", "similarity() uses unequal dim weights — v2 requires equal 5D")
            .with_details(details)
    } else {
        CheckResult::pass("L0 Similarity", "OK — similarity uses equal 5D weights")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// L0 CASCADE #4: Molecule::raw() is public — enables QT④ bypass
// ═══════════════════════════════════════════════════════════════════

pub fn check_l0_mol_raw_public(root: &Path) -> CheckResult {
    println!("[36/37] L0 — Molecule::raw() visibility...");
    let mol_path = root.join("crates/olang/src/mol/molecular.rs");

    if !mol_path.exists() {
        return CheckResult::fail("L0 Mol::raw", "molecular.rs not found");
    }

    let content = match std::fs::read_to_string(&mol_path) {
        Ok(c) => c,
        Err(e) => return CheckResult::fail("L0 Mol::raw", &format!("Cannot read: {}", e)),
    };

    let mut details = Vec::new();

    let has_pub_raw = content.contains("pub fn raw(");

    // Count callers of Molecule::raw outside encoder/lca/test
    let crates = root.join("crates");
    let files = scan_rs_files(&crates);
    let raw_calls: Vec<_> = grep_pattern(&files, "Molecule::raw(")
        .into_iter()
        .filter(|(p, _, l)| {
            let ps = p.to_str().unwrap_or("");
            !ps.contains("encoder") && !ps.contains("lca")
                && !ps.contains("test") && !ps.contains("molecular.rs")
                && !l.trim().starts_with("//")
        })
        .collect();

    details.push(format!("Molecule::raw() is pub: {}", if has_pub_raw { "❌ YES" } else { "✅ NO (pub(crate) or private)" }));
    details.push(format!("raw() called outside encoder/lca/test: {} refs", raw_calls.len()));
    for (p, line, text) in &raw_calls {
        let rel = p.strip_prefix(root).unwrap_or(p);
        details.push(format!("  {}:{} — {}", rel.display(), line, text));
    }

    if has_pub_raw && !raw_calls.is_empty() {
        details.push("QT④: Molecule từ encode_codepoint() — KHÔNG viết tay".into());
        details.push("raw() is pub → any code can bypass encode_codepoint()".into());
        CheckResult::fail("L0 Mol::raw", &format!(
            "Molecule::raw() is pub + {} external callers — QT④ bypassable",
            raw_calls.len()
        ))
            .with_details(details)
    } else if has_pub_raw {
        CheckResult::warn("L0 Mol::raw", "Molecule::raw() is pub but 0 external callers — should be pub(crate)")
            .with_details(details)
    } else {
        CheckResult::pass("L0 Mol::raw", "OK — raw() is not public")
            .with_details(details)
    }
}

// ═══════════════════════════════════════════════════════════════════
// L0 CASCADE #5: PLAN_REWRITE alignment — Rust vs Olang target
// ═══════════════════════════════════════════════════════════════════

pub fn check_rewrite_alignment(root: &Path) -> CheckResult {
    println!("[37/37] REWRITE — Rust code vs Olang target...");

    let mut details = Vec::new();

    // Check: how much Olang bytecode exists vs Rust code
    let stdlib_dir = root.join("stdlib");
    let crates_dir = root.join("crates");

    let mut ol_files = 0;
    let mut ol_bytes: u64 = 0;
    fn count_ol(dir: &std::path::Path, files: &mut i32, bytes: &mut u64) {
        let entries = match std::fs::read_dir(dir) { Ok(e) => e, Err(_) => return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { count_ol(&path, files, bytes); }
            else if path.extension().and_then(|e| e.to_str()) == Some("ol") {
                *files += 1;
                *bytes += std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    count_ol(&stdlib_dir, &mut ol_files, &mut ol_bytes);

    let mut rs_files = 0;
    let mut rs_bytes: u64 = 0;
    fn count_rs(dir: &std::path::Path, files: &mut i32, bytes: &mut u64) {
        let entries = match std::fs::read_dir(dir) { Ok(e) => e, Err(_) => return };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_str().unwrap_or("");
            if name == "target" || name.starts_with('.') { continue; }
            if path.is_dir() { count_rs(&path, files, bytes); }
            else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                *files += 1;
                *bytes += std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    count_rs(&crates_dir, &mut rs_files, &mut rs_bytes);

    let ol_kb = ol_bytes / 1024;
    let rs_kb = rs_bytes / 1024;
    let ratio = if ol_bytes > 0 { rs_bytes as f64 / ol_bytes as f64 } else { 0.0 };

    details.push(format!("Olang (.ol): {} files, {} KB", ol_files, ol_kb));
    details.push(format!("Rust (.rs):  {} files, {} KB", rs_files, rs_kb));
    details.push(format!("Ratio Rust/Olang: {:.1}x", ratio));
    details.push("PLAN_REWRITE: Rust = tử cung, Olang = sinh linh".into());
    details.push(format!("Migration progress: {:.1}% Olang", ol_bytes as f64 / (ol_bytes + rs_bytes) as f64 * 100.0));

    // This is informational, not pass/fail
    CheckResult::warn("REWRITE Progress", &format!(
        "Rust {:.0}x larger than Olang — migration {:.1}%",
        ratio,
        ol_bytes as f64 / (ol_bytes + rs_bytes) as f64 * 100.0
    ))
        .with_details(details)
}
