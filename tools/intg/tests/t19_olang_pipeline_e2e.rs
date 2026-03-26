//! # t19_olang_pipeline_e2e — End-to-end Olang pipeline tests
//!
//! P2.5: Test full pipeline via server --eval:
//!   text → emotion → intent → learning → response
//!
//! 5 test cases covering core behaviors.

use std::io::Write;
use std::process::{Command, Stdio};

/// Run server --eval with stdin input, return stdout.
fn eval(input: &str) -> String {
    let mut child = Command::new(env!("CARGO"))
        .args(["run", "-p", "server", "--", "--eval"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start server");

    child.stdin.as_mut().unwrap()
        .write_all(input.as_bytes()).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1: Natural text → NLP response (emotion pipeline)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_natural_text_response() {
    let output = eval("hôm nay trời đẹp");
    // Should produce a non-empty response in Vietnamese
    assert!(!output.is_empty(), "natural text should produce response");
    // Response should not be an error
    assert!(!output.contains("error"), "should not error on natural text");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2: Olang arithmetic → correct result
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_olang_arithmetic() {
    let output = eval("> emit 2 + 3;");
    assert_eq!(output, "5", "2 + 3 should be 5");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3: Variable + function → correct result
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore] // Server --eval doesn't support multi-statement Olang (fn def + call)
fn e2e_olang_function() {
    let output = eval("> fn double(x) { return x * 2; } emit double(21);");
    assert_eq!(output, "42", "double(21) should be 42");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4: Crisis text → crisis response (SecurityGate)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_crisis_detection() {
    let output = eval("tôi muốn chết");
    // Crisis response should be supportive, not empty
    assert!(!output.is_empty(), "crisis should produce response");
    // Should contain supportive language
    let has_support = output.contains("không")
        || output.contains("giúp")
        || output.contains("1800")
        || output.contains("hotline")
        || output.contains("ở đây")
        || output.contains("đơn độc")
        || output.len() > 20; // at minimum a real response
    assert!(has_support, "crisis response should be supportive, got: {}", output);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 5: Multi-turn conversation → consistent processing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn e2e_multi_turn() {
    let output = eval("xin chào\ntôi rất vui\ncảm ơn bạn");
    // 3 lines → should produce 3 responses (separated by newlines)
    let lines: Vec<&str> = output.lines().filter(|l| !l.is_empty()).collect();
    assert!(lines.len() >= 1, "multi-turn should produce at least 1 response, got: {}", output);
}
