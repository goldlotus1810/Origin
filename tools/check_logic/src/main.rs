//! # check-logic — HomeOS v2 Spec Validator
//!
//! Công cụ kiểm tra toàn bộ mã nguồn + dữ liệu theo logic v2.
//! Mọi AI contributor PHẢI pass tool này trước khi push.
//!
//! Usage:
//!   cargo run -p check_logic
//!   make check-logic
//!
//! Dựa trên: docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md
//!           old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md

use std::fs;
use std::path::{Path, PathBuf};
use std::process;

mod checks;

fn main() {
    let root = find_project_root();
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║     check-logic — HomeOS v2 Spec Validator          ║");
    println!("║     docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md            ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();
    println!("Root: {}", root.display());
    println!();

    let results = vec![
        // ── 6 Bug Patterns ──
        checks::check_compose_no_average(&root),
        checks::check_self_correct_rollback(&root),
        checks::check_quality_weights(&root),
        checks::check_entropy_floor(&root),
        checks::check_hnsw_tiebreak(&root),
        checks::check_security_gate_3layer(&root),
        // ── 5 Checkpoints ──
        checks::check_pipeline_checkpoints(&root),
        // ── 23 Invariant Rules ──
        checks::check_molecule_not_handwritten(&root),
        checks::check_append_only(&root),
        checks::check_agent_tiers(&root),
        checks::check_l0_no_import_l1(&root),
        checks::check_skill_stateless(&root),
        checks::check_worker_sends_chain(&root),
        // ── Data Integrity ──
        checks::check_udc_utf32_data(&root),
    ];

    // ── Report ──
    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("                    RESULTS");
    println!("═══════════════════════════════════════════════════════");

    let mut pass = 0;
    let mut fail = 0;
    let mut warn = 0;

    for r in &results {
        let icon = match r.status {
            Status::Pass => { pass += 1; "✅" }
            Status::Fail => { fail += 1; "❌" }
            Status::Warn => { warn += 1; "⚠️ " }
        };
        println!("{} {}: {}", icon, r.name, r.summary);
        for detail in &r.details {
            println!("     {}", detail);
        }
    }

    println!();
    println!("───────────────────────────────────────────────────────");
    println!("  PASS: {}  |  WARN: {}  |  FAIL: {}", pass, warn, fail);
    println!("───────────────────────────────────────────────────────");

    if fail > 0 {
        println!();
        println!("❌ FAILED — Fix {} issue(s) before push.", fail);
        println!("   Ref: docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md");
        process::exit(1);
    } else if warn > 0 {
        println!();
        println!("⚠️  PASSED with {} warning(s).", warn);
        process::exit(0);
    } else {
        println!();
        println!("✅ ALL CHECKS PASSED");
        process::exit(0);
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Pass,
    Fail,
    Warn,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: Status,
    pub summary: String,
    pub details: Vec<String>,
}

impl CheckResult {
    fn pass(name: &str, summary: &str) -> Self {
        Self {
            name: name.to_string(),
            status: Status::Pass,
            summary: summary.to_string(),
            details: vec![],
        }
    }
    fn fail(name: &str, summary: &str) -> Self {
        Self {
            name: name.to_string(),
            status: Status::Fail,
            summary: summary.to_string(),
            details: vec![],
        }
    }
    fn warn(name: &str, summary: &str) -> Self {
        Self {
            name: name.to_string(),
            status: Status::Warn,
            summary: summary.to_string(),
            details: vec![],
        }
    }
    fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }
}

/// Walk up from CWD to find project root (has Cargo.toml with [workspace]).
fn find_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("cannot get cwd");
    loop {
        let cargo = dir.join("Cargo.toml");
        if cargo.exists() {
            if let Ok(content) = fs::read_to_string(&cargo) {
                if content.contains("[workspace]") {
                    return dir;
                }
            }
        }
        if !dir.pop() {
            eprintln!("ERROR: Cannot find workspace root (Cargo.toml with [workspace])");
            process::exit(2);
        }
    }
}

/// Scan all .rs files under a directory, return (path, content) pairs.
pub fn scan_rs_files(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }
    fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip target/ and hidden dirs
                let name = path.file_name().unwrap_or_default().to_str().unwrap_or("");
                if name == "target" || name.starts_with('.') {
                    continue;
                }
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    out.push((path, content));
                }
            }
        }
    }
    walk(dir, &mut results);
    results
}

/// Count pattern occurrences in source, return (count, locations).
pub fn grep_pattern(files: &[(PathBuf, String)], pattern: &str) -> Vec<(PathBuf, usize, String)> {
    let mut hits = Vec::new();
    for (path, content) in files {
        for (i, line) in content.lines().enumerate() {
            if line.contains(pattern) {
                hits.push((path.clone(), i + 1, line.trim().to_string()));
            }
        }
    }
    hits
}

/// Count regex-like pattern (simple substring, case-insensitive option).
pub fn grep_pattern_ci(files: &[(PathBuf, String)], pattern: &str) -> Vec<(PathBuf, usize, String)> {
    let pat_lower = pattern.to_lowercase();
    let mut hits = Vec::new();
    for (path, content) in files {
        for (i, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&pat_lower) {
                hits.push((path.clone(), i + 1, line.trim().to_string()));
            }
        }
    }
    hits
}
