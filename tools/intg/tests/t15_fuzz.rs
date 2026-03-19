//! Integration: Fuzz tests — random/malformed input → no panic
//!
//! Tests that the system handles invalid input gracefully:
//!   - Random strings → parse() returns Err, never panics
//!   - Random bytecode → VM returns error, never panics
//!   - Random ISL messages → decode returns None, never panics
//!   - Malformed inputs → no crash
//!
//! Uses a simple PRNG (xorshift64) for reproducible "random" data.
//!
//! Covers: olang::lang::syntax, olang::exec::vm, olang::exec::bytecode, isl

use isl::message::ISLMessage;
use olang::exec::bytecode::decode_bytecode;
use olang::exec::ir::{OlangProgram, Op};
use olang::exec::vm::OlangVM;
use olang::lang::syntax::parse;

/// Simple xorshift64 PRNG for reproducible tests.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self { Self(seed) }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn next_u8(&mut self) -> u8 { self.next() as u8 }

    fn gen_bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next_u8()).collect()
    }

    fn gen_string(&mut self, max_len: usize) -> String {
        let len = (self.next() as usize) % max_len + 1;
        let bytes: Vec<u8> = (0..len).map(|_| {
            // Printable ASCII + some control chars
            let b = self.next_u8();
            if b < 128 { b } else { b % 128 }
        }).collect();
        String::from_utf8_lossy(&bytes).to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Parser fuzz: random strings never panic
// ═══════════════════════════════════════════════════════════════════

#[test]
fn fuzz_parser_random_strings_no_panic() {
    let mut rng = Rng::new(0xDEADBEEF_CAFEBABE);

    for _ in 0..10_000 {
        let input = rng.gen_string(200);
        // parse may return Ok or Err — but must never panic
        let _ = parse(&input);
    }
}

#[test]
fn fuzz_parser_olang_like_strings_no_panic() {
    let mut rng = Rng::new(0x1234567890ABCDEF);

    let keywords = [
        "let", "fn", "if", "else", "while", "return", "emit",
        "match", "struct", "enum", "import", "from", "pub",
        "true", "false", "dream", "stats", "fuse", "trace",
        "typeof", "explain", "why", "assert", "learn", "seed",
    ];
    let ops = ["+", "-", "*", "/", "=", "==", "!=", "<", ">", "<=", ">=",
               "(", ")", "{", "}", "[", "]", ";", ",", ".", ":", "\""];

    for _ in 0..5_000 {
        // Build semi-valid looking Olang code
        let mut code = String::new();
        let tokens = (rng.next() as usize) % 20 + 1;
        for _ in 0..tokens {
            let choice = rng.next() % 4;
            match choice {
                0 => {
                    let kw = keywords[rng.next() as usize % keywords.len()];
                    code.push_str(kw);
                }
                1 => {
                    let op = ops[rng.next() as usize % ops.len()];
                    code.push_str(op);
                }
                2 => {
                    let n = rng.next() % 1000;
                    code.push_str(&n.to_string());
                }
                _ => {
                    let name_len = (rng.next() as usize) % 8 + 1;
                    let name: String = (0..name_len)
                        .map(|_| (b'a' + (rng.next_u8() % 26)) as char)
                        .collect();
                    code.push_str(&name);
                }
            }
            code.push(' ');
        }

        let _ = parse(&code);
    }
}

#[test]
fn fuzz_parser_adversarial_strings_no_panic() {
    let adversarial = [
        "",
        ";;;;;;;;",
        "(((((((((((((((((((((((",
        "))))))))))))))))))))))))",
        "{{{{{{{{{{{{{{{{{{{{",
        "}}}}}}}}}}}}}}}}}}}}",
        "let let let let let",
        "fn fn fn fn fn fn fn",
        "\"\"\"\"\"\"\"\"",
        "\"unclosed string",
        "/* unclosed comment",
        "// line comment\n// another\n",
        "0 1 2 3 4 5 6 7 8 9",
        "\0\0\0\0\0",
        "\x01\x02\x03\x04\x05",
        "∈ ⊂ ≡ ⊥ ∘ → ≈ ←",
        "🔥🔥🔥🔥🔥",
        &"a".repeat(10000),
        &"let x = ".repeat(1000),
        "fn()()()()()()()",
        "if if if if else else else",
        "return return return",
        "let = = = = = = ;",
        "emit emit emit emit;",
        "while while { while }",
    ];

    for input in &adversarial {
        let _ = parse(input);
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bytecode decoder fuzz: random bytes never panic
// ═══════════════════════════════════════════════════════════════════

#[test]
fn fuzz_bytecode_decode_random_bytes_no_panic() {
    let mut rng = Rng::new(0xFEEDFACE);

    for _ in 0..10_000 {
        let len = (rng.next() as usize) % 100 + 1;
        let bytes = rng.gen_bytes(len);
        // decode may return Ok or Err — but must never panic
        let _ = decode_bytecode(&bytes);
    }
}

#[test]
fn fuzz_bytecode_decode_opcode_like_bytes_no_panic() {
    let mut rng = Rng::new(0xBADC0DE);

    // Generate bytecode-like sequences (valid opcode tags with random payloads)
    for _ in 0..5_000 {
        let mut bytes = Vec::new();
        let ops = (rng.next() as usize) % 20 + 1;
        for _ in 0..ops {
            let tag = rng.next_u8() % 0x25; // Valid tag range
            bytes.push(tag);
            // Random payload bytes
            let payload_len = (rng.next() as usize) % 10;
            for _ in 0..payload_len {
                bytes.push(rng.next_u8());
            }
        }
        bytes.push(0x0F); // Halt at end

        let _ = decode_bytecode(&bytes);
    }
}

#[test]
fn fuzz_bytecode_decode_edge_cases_no_panic() {
    let edge_cases: Vec<Vec<u8>> = vec![
        vec![],                    // empty
        vec![0x0F],                // just Halt
        vec![0x00],                // Nop
        vec![0xFF],                // invalid tag
        vec![0x01],                // Push with no payload
        vec![0x01, 0x00],          // Push with partial len
        vec![0x01, 0x05, 0x00],    // Push with len=5 but only 1 byte after
        vec![0x15],                // PushNum with no f64
        vec![0x15, 0x00, 0x00, 0x00, 0x00], // PushNum partial
        vec![0x09, 0x00, 0x00],    // Jmp with partial target
        vec![0x0A, 0xFF, 0xFF, 0xFF, 0xFF], // Jz to out-of-bounds
        vec![0x02, 0x03, b'a', b'b', b'c', 0x0F], // Load "abc" + Halt
        vec![0x13, 0x01, b'x', 0x0F], // Store "x" + Halt (no value on stack)
        vec![0x19, 0x01, 0x02, 0x03, 0x04, 0x05, 0x0F], // PushMol + Halt
    ];

    for bytes in &edge_cases {
        let _ = decode_bytecode(bytes);
    }
}

// ═══════════════════════════════════════════════════════════════════
// VM fuzz: random programs don't crash (with step limit)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn fuzz_vm_random_ops_no_panic() {
    let mut rng = Rng::new(0xC0FFEE);

    for _ in 0..1_000 {
        let mut ops = Vec::new();
        let count = (rng.next() as usize) % 20 + 1;

        for _ in 0..count {
            let choice = rng.next() % 10;
            let op = match choice {
                0 => Op::PushNum(rng.next() as f64),
                1 => Op::Emit,
                2 => Op::Dup,
                3 => Op::Pop,
                4 => Op::Swap,
                5 => Op::Store(format!("v{}", rng.next() % 100)),
                6 => Op::Load(format!("v{}", rng.next() % 100)),
                7 => Op::Jz((rng.next() as usize) % 50),
                8 => Op::Jmp((rng.next() as usize) % 50),
                _ => Op::Nop,
            };
            ops.push(op);
        }
        ops.push(Op::Halt);

        let prog = OlangProgram {
            name: "fuzz".into(),
            ops,
        };

        let vm = OlangVM::with_max_steps(1000);
        let result = vm.execute(&prog);
        // May have errors (underflow, etc.) but should never panic
        let _ = result.has_error();
    }
}

#[test]
fn fuzz_vm_adversarial_programs_no_panic() {
    // Programs designed to stress edge cases
    let programs: Vec<Vec<Op>> = vec![
        // Deep stack: push many values
        {
            let mut ops: Vec<Op> = (0..500).map(|i| Op::PushNum(i as f64)).collect();
            ops.push(Op::Halt);
            ops
        },
        // Many pops on empty stack
        {
            let mut ops: Vec<Op> = (0..100).map(|_| Op::Pop).collect();
            ops.push(Op::Halt);
            ops
        },
        // Jump to self (infinite loop — step limit catches)
        vec![Op::Jmp(0)],
        // Jump past end
        vec![Op::Jmp(99999), Op::Halt],
        // Jz past end
        vec![Op::PushNum(0.0), Op::Jz(99999), Op::Halt],
        // Store without value
        vec![Op::Store("x".into()), Op::Halt],
        // Load nonexistent
        vec![Op::Load("nonexistent".into()), Op::Halt],
        // Emit empty stack
        vec![Op::Emit, Op::Halt],
        // Swap with 0 items
        vec![Op::Swap, Op::Halt],
        // Swap with 1 item
        vec![Op::PushNum(1.0), Op::Swap, Op::Halt],
        // Dup empty
        vec![Op::Dup, Op::Halt],
        // Many stores to same variable
        {
            let mut ops = Vec::new();
            for i in 0..500 {
                ops.push(Op::PushNum(i as f64));
                ops.push(Op::Store("x".into()));
            }
            ops.push(Op::Halt);
            ops
        },
        // Many different variables
        {
            let mut ops = Vec::new();
            for i in 0..200 {
                ops.push(Op::PushNum(i as f64));
                ops.push(Op::Store(format!("var_{}", i)));
            }
            ops.push(Op::Halt);
            ops
        },
        // Alternating push/pop
        {
            let mut ops = Vec::new();
            for _ in 0..500 {
                ops.push(Op::PushNum(42.0));
                ops.push(Op::Pop);
            }
            ops.push(Op::Halt);
            ops
        },
    ];

    for (i, ops) in programs.iter().enumerate() {
        let prog = OlangProgram {
            name: format!("adversarial_{}", i),
            ops: ops.clone(),
        };
        let vm = OlangVM::with_max_steps(10_000);
        let result = vm.execute(&prog);
        let _ = result.has_error();
    }
}

// ═══════════════════════════════════════════════════════════════════
// ISL fuzz: random bytes → decode never panics
// ═══════════════════════════════════════════════════════════════════

#[test]
fn fuzz_isl_decode_random_bytes_no_panic() {
    let mut rng = Rng::new(0x15100000);

    for _ in 0..10_000 {
        let bytes = rng.gen_bytes(12);
        // from_bytes may return None — but must never panic
        let _ = ISLMessage::from_bytes(&bytes);
    }
}

#[test]
fn fuzz_isl_decode_short_bytes_no_panic() {
    // Less than 12 bytes — should return None
    for len in 0..12 {
        let bytes: Vec<u8> = (0..len).map(|i| i as u8).collect();
        let result = ISLMessage::from_bytes(&bytes);
        assert!(result.is_none(), "ISL decode of {} bytes should be None", len);
    }
}

#[test]
fn fuzz_isl_decode_edge_cases_no_panic() {
    let cases: Vec<Vec<u8>> = vec![
        vec![0; 12],                                // all zeros
        vec![0xFF; 12],                             // all 0xFF
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0], // invalid msg_type
        vec![1, 2, 3, 4, 5, 6, 7, 8, 0x01, 0, 0, 0], // valid-ish
        vec![0; 100],                               // too long
    ];

    for bytes in &cases {
        let _ = ISLMessage::from_bytes(bytes);
    }
}
