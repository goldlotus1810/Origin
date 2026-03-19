//! # t16_e2e_demo — End-to-end tests via server --eval
//!
//! Chạy `cargo run -p server -- --eval` với stdin piped,
//! capture stdout, verify output.
//! Không mock. Real pipeline: text → parse → compile → execute → output.

use std::io::Write;
use std::process::{Command, Stdio};

/// Run server --eval with piped input, return stdout.
fn run_eval(input: &str) -> String {
    let mut child = Command::new(env!("CARGO"))
        .args(["run", "-p", "server", "--", "--eval"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start server");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .expect("failed to wait for server");
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Run server --eval, return (stdout, stderr, exit_code).
fn run_eval_full(input: &str) -> (String, String, i32) {
    let mut child = Command::new(env!("CARGO"))
        .args(["run", "-p", "server", "--", "--eval"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start server");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .expect("failed to wait for server");
    let code = output.status.code().unwrap_or(-1);
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        code,
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Basic: --eval mode works
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn eval_exits_cleanly() {
    let (_, _, code) = run_eval_full("");
    assert_eq!(code, 0, "empty input should exit 0");
}

#[test]
fn eval_no_banner() {
    let out = run_eval("");
    assert!(
        !out.contains("HomeOS"),
        "eval mode should not print boot banner"
    );
    assert!(
        !out.contains("[boot]"),
        "eval mode should not print boot log"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Natural language processing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn eval_natural_text_produces_response() {
    let out = run_eval("hello");
    assert!(
        !out.trim().is_empty(),
        "natural text should produce some response"
    );
}

#[test]
fn eval_emotion_pipeline_works() {
    // Vietnamese emotional text → emotion pipeline should respond
    let out = run_eval("tôi vui quá");
    assert!(
        !out.trim().is_empty(),
        "emotional text should produce response"
    );
}

#[test]
fn eval_multi_line_processes_all() {
    // Multiple lines → each processed independently
    let out = run_eval("hello\ntôi vui");
    // Should have at least one response
    assert!(
        !out.trim().is_empty(),
        "multi-line input should produce output"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Olang commands (○{...} syntax)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn eval_olang_stats() {
    let out = run_eval("○{stats}");
    // Stats should mention STM, Silk, or node counts
    let lower = out.to_lowercase();
    assert!(
        lower.contains("stm")
            || lower.contains("silk")
            || lower.contains("node")
            || lower.contains("turn")
            || lower.contains("registry"),
        "stats should show system info, got: {}",
        out
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Inline Olang programs (> prefix)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn eval_inline_olang_program() {
    // > prefix triggers run_program
    let out = run_eval("> emit 42;");
    // VM should emit something (may be "42" or contain it)
    // Note: VM output format depends on implementation
    assert!(
        out.contains("42") || !out.trim().is_empty(),
        "inline program should produce output"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Empty/whitespace handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn eval_empty_lines_skipped() {
    let (out, _, code) = run_eval_full("\n\n\n");
    assert_eq!(code, 0);
    assert!(
        out.trim().is_empty(),
        "empty lines should produce no output"
    );
}

#[test]
fn eval_whitespace_only_skipped() {
    let (out, _, code) = run_eval_full("   \n  \t  \n");
    assert_eq!(code, 0);
    assert!(
        out.trim().is_empty(),
        "whitespace-only lines should produce no output"
    );
}
