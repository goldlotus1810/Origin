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
    println!("[1/14] Compose — no simple average for Valence...");
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
    println!("[2/14] Self-correct — rollback guard...");
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
    println!("[3/14] Quality weights — Σ = 1.0...");
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
    println!("[4/14] Entropy — ε_floor for Σc...");
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
    println!("[5/14] HNSW insert — deterministic tie-breaking...");
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
    println!("[6/14] SecurityGate — 3-layer detection...");
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
    println!("[7/14] Pipeline — 5 checkpoints...");
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
    println!("[8/14] Invariant — Molecule not handwritten (QT④)...");
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
    println!("[9/14] Invariant — Append-only (QT⑧⑨⑩)...");
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
    println!("[10/14] Invariant — Agent tiers (QT⑮)...");
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
    println!("[11/14] Invariant — L0 does not import L1 (QT⑭)...");
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
    println!("[12/14] Invariant — Skill stateless (QT⑲-㉓)...");
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
    println!("[13/14] Invariant — Worker sends chain, not raw data...");
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
    println!("[14/14] Data — json/udc_utf32.json integrity...");

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
