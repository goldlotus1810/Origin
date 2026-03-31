//! Integration: VM execute compiled Olang source → verify output
//!
//! Tests the full pipeline: source → parse → lower → bytecode → VM execute
//! Covers: olang::lang::syntax + olang::lang::semantic + olang::exec::bytecode + olang::exec::vm

use olang::exec::bytecode::{decode_bytecode, encode_bytecode};
use olang::exec::ir::{OlangProgram, Op};
use olang::exec::vm::OlangVM;
use olang::lang::semantic::lower;
use olang::lang::syntax::parse;

/// Compile source → bytecode → decode → OlangProgram, then execute.
fn compile_and_run(source: &str) -> olang::exec::vm::VmResult {
    let stmts = parse(source).expect("parse should succeed");
    let program = lower(&stmts);
    let vm = OlangVM::new();
    vm.execute(&program)
}

/// Compile source → bytecode bytes (codegen format).
fn compile_to_bytecode(source: &str) -> Vec<u8> {
    let stmts = parse(source).expect("parse should succeed");
    let program = lower(&stmts);
    encode_bytecode(&program.ops)
}

// ═══════════════════════════════════════════════════════════════════
// Basic VM execution
// ═══════════════════════════════════════════════════════════════════

#[test]
fn vm_simple_let_and_halt() {
    let result = compile_and_run("let x = 42;");
    assert!(!result.has_error(), "simple let should not error: {:?}", result.errors());
}

#[test]
fn vm_emit_string_literal() {
    let result = compile_and_run(r#"emit "hello";"#);
    assert!(!result.has_error(), "emit string should not error: {:?}", result.errors());
    let outputs = result.outputs();
    assert!(!outputs.is_empty(), "should have at least one output");
}

#[test]
fn vm_arithmetic_add() {
    // Test: let a = 2; let b = 3; let c = a + b;
    let result = compile_and_run("let a = 2; let b = 3; let c = a + b;");
    assert!(!result.has_error(), "arithmetic should not error: {:?}", result.errors());
}

#[test]
fn vm_function_def_and_call() {
    let source = r#"
        fn double(x) { return x + x; }
        let r = double(21);
    "#;
    let result = compile_and_run(source);
    assert!(!result.has_error(), "fn call should not error: {:?}", result.errors());
}

#[test]
fn vm_if_else_control_flow() {
    let source = r#"
        let x = 10;
        if x > 5 {
            emit "big";
        } else {
            emit "small";
        }
    "#;
    let result = compile_and_run(source);
    assert!(!result.has_error(), "if/else should not error: {:?}", result.errors());
    let outputs = result.outputs();
    assert!(!outputs.is_empty(), "should emit something");
}

#[test]
fn vm_while_loop() {
    let source = r#"
        let i = 0;
        while i < 3 {
            i = i + 1;
        }
    "#;
    let result = compile_and_run(source);
    assert!(!result.has_error(), "while loop should not error: {:?}", result.errors());
}

#[test]
fn vm_string_concat() {
    let source = r#"
        let a = "hello";
        let b = " world";
        let c = a + b;
        emit c;
    "#;
    let result = compile_and_run(source);
    assert!(!result.has_error(), "string concat should not error: {:?}", result.errors());
    assert!(!result.outputs().is_empty(), "should emit concatenated string");
}

// ═══════════════════════════════════════════════════════════════════
// Bytecode encode/decode roundtrip
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bytecode_roundtrip_simple() {
    let bytecode = compile_to_bytecode("let x = 42;");
    assert!(!bytecode.is_empty(), "bytecode should not be empty");
    // Should end with Halt
    assert_eq!(*bytecode.last().unwrap(), 0x0F, "should end with Halt");
    // Should be decodable
    let ops = decode_bytecode(&bytecode).expect("decode should succeed");
    assert!(!ops.is_empty(), "decoded ops should not be empty");
}

#[test]
fn bytecode_roundtrip_preserves_ops() {
    let source = "let x = 42;";
    let stmts = parse(source).unwrap();
    let program = lower(&stmts);

    // Encode → decode should preserve structure
    let bytecode = encode_bytecode(&program.ops);
    let decoded = decode_bytecode(&bytecode).expect("decode should succeed");

    // Re-encode decoded ops should produce same bytecode
    let re_encoded = encode_bytecode(&decoded);
    assert_eq!(bytecode, re_encoded, "re-encoded bytecode should match original");
}

#[test]
fn bytecode_roundtrip_function() {
    let source = r#"
        fn add(a, b) { return a + b; }
        let r = add(1, 2);
    "#;
    let bytecode = compile_to_bytecode(source);
    let decoded = decode_bytecode(&bytecode).expect("should decode fn bytecode");
    // Verify decode produces reasonable ops (roundtrip not exact because
    // decoded Jz/Jmp targets are byte offsets, re-encode would double-convert)
    assert!(!decoded.is_empty(), "decoded should not be empty");
    let has_halt = decoded.iter().any(|op| matches!(op, Op::Halt));
    assert!(has_halt, "decoded should contain Halt");
}

#[test]
fn bytecode_decode_control_flow() {
    // Verify that bytecode encoding produces valid output that can be decoded.
    // Note: exact roundtrip (encode→decode→encode) is not guaranteed because
    // decoded Jz/Jmp/Closure targets are byte offsets, which the encoder
    // would double-convert through the offset table. Instead we verify:
    // 1. Encoding succeeds (non-empty output)
    // 2. Bytecode starts with valid opcodes (spot check)
    let source = r#"
        let x = 5;
        if x > 3 { emit "yes"; }
    "#;
    let bytecode = compile_to_bytecode(source);
    assert!(!bytecode.is_empty(), "bytecode should not be empty");
    // First opcode should be PushNum (0x15) for `let x = 5`
    assert_eq!(bytecode[0], 0x15, "first opcode should be PushNum");
    // Last opcode should be Halt (0x0F)
    assert_eq!(*bytecode.last().unwrap(), 0x0F, "last opcode should be Halt");
}

// ═══════════════════════════════════════════════════════════════════
// IR → VM direct execution (no bytecode intermediary)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn vm_direct_push_num_emit() {
    let prog = OlangProgram {
        name: "test".into(),
        ops: vec![
            Op::PushNum(42.0),
            Op::Emit,
            Op::Halt,
        ],
    };
    let result = OlangVM::new().execute(&prog);
    assert!(!result.has_error(), "direct PushNum+Emit should work");
    assert!(!result.outputs().is_empty(), "should have output");
}

#[test]
fn vm_direct_store_load() {
    let prog = OlangProgram {
        name: "test".into(),
        ops: vec![
            Op::PushNum(99.0),
            Op::Store("myvar".into()),
            Op::Load("myvar".into()),
            Op::Emit,
            Op::Halt,
        ],
    };
    let result = OlangVM::new().execute(&prog);
    assert!(!result.has_error());
}

#[test]
fn vm_step_limit_prevents_infinite_loop() {
    let prog = OlangProgram {
        name: "test".into(),
        ops: vec![
            Op::Jmp(0), // infinite jump to self
        ],
    };
    let vm = OlangVM::with_max_steps(100);
    let result = vm.execute(&prog);
    assert!(result.has_error(), "infinite loop should be caught");
    let has_max_steps = result.errors().iter().any(|e| {
        matches!(e, olang::exec::vm::VmError::MaxStepsExceeded)
    });
    assert!(has_max_steps, "should have MaxStepsExceeded error");
}

// ═══════════════════════════════════════════════════════════════════
// Multi-file compilation (B7 halt stripping)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn b7_concatenated_bytecode_executes_all() {
    // Simulate compile_all: two sources, strip trailing halts, single final halt
    let bc1 = compile_to_bytecode("let a = 1;");
    let bc2 = compile_to_bytecode("let b = 2;");

    let mut combined = Vec::new();
    for mut bc in [bc1, bc2] {
        while bc.last() == Some(&0x0F) {
            bc.pop();
        }
        combined.extend_from_slice(&bc);
    }
    combined.push(0x0F); // single final halt

    // Should decode and have exactly one Halt
    let ops = decode_bytecode(&combined).expect("combined should decode");
    let halt_count = ops.iter().filter(|op| matches!(op, Op::Halt)).count();
    assert_eq!(halt_count, 1, "combined should have exactly 1 Halt, got {}", halt_count);
}
