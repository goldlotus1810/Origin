# PLAN 11 — E2E Verification: Chứng Minh origin.olang Hoạt Động

**Phụ thuộc:** Phase 8 (parser), Phase 9 (native REPL), Phase 10 (browser)
**Mục tiêu:** Bất kỳ ai clone repo → build → run → thấy HomeOS hoạt động. Có bằng chứng.

---

## Bối cảnh

```
VẤN ĐỀ:
  "không ai thấy origin.olang tròn méo thế nào"
  — Có 2500+ tests nhưng toàn unit test
  — Không có 1 demo nào chạy được end-to-end
  — Không ai ngoài dev biết origin.olang làm gì
  — Build xong → ./origin → echo prompt → thoát. Xong.

SAU PLAN 11:
  1. make demo     → chạy 10 scenarios, output rõ ràng
  2. make verify   → automated E2E test, pass/fail
  3. README.md     → "Build & Run in 60 seconds"
  4. Video-ready   → demo script cho screencast
  5. CI/CD ready   → GitHub Actions chạy E2E tests
```

---

## Tasks

### 11.1 — Demo Script: 10 Scenarios (~200-300 LOC Olang/Shell)

**File:** `tools/demo/scenarios.sh` + `tools/demo/demo_*.ol`

```bash
#!/bin/bash
# make demo — chạy 10 scenarios, hiển thị kết quả

echo "═══════════════════════════════════════════"
echo "  ○ HomeOS E2E Demo — 10 Scenarios"
echo "═══════════════════════════════════════════"

PASS=0
FAIL=0
ORIGIN="./target/release/origin"  # hoặc native binary

# ─────────────────────────────────────────
# Scenario 1: Arithmetic
echo -n "1. Arithmetic (2+3=5)... "
RESULT=$(echo 'emit 2 + 3' | $ORIGIN --eval 2>&1)
if [ "$RESULT" = "5" ]; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# Scenario 2: Variables
echo -n "2. Variables (let x=42, emit x)... "
RESULT=$(echo -e 'let x = 42;\nemit x;' | $ORIGIN --eval 2>&1)
if [ "$RESULT" = "42" ]; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# Scenario 3: Functions
echo -n "3. Functions (fib(10)=55)... "
RESULT=$(echo -e 'fn fib(n) { if n < 2 { return n; } return fib(n-1)+fib(n-2); }\nemit fib(10);' | $ORIGIN --eval 2>&1)
if [ "$RESULT" = "55" ]; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# Scenario 4: Strings
echo -n "4. Strings (concat)... "
RESULT=$(echo 'emit "hello " + "world"' | $ORIGIN --eval 2>&1)
if [ "$RESULT" = "hello world" ]; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# Scenario 5: Hex literals (Phase 8)
echo -n "5. Hex literals (0xFF=255)... "
RESULT=$(echo 'emit 0xFF' | $ORIGIN --eval 2>&1)
if [ "$RESULT" = "255" ]; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# Scenario 6: Emotion — Crisis detection
echo -n "6. Emotion: Crisis detection... "
RESULT=$(echo 'tôi muốn tự tử' | $ORIGIN --eval 2>&1)
if echo "$RESULT" | grep -qi "crisis\|dừng\|giúp"; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# Scenario 7: Emotion — Supportive tone
echo -n "7. Emotion: Supportive tone... "
RESULT=$(echo 'tôi buồn vì mất việc' | $ORIGIN --eval 2>&1)
if echo "$RESULT" | grep -qi "cảm\|buồn\|kể"; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# Scenario 8: SecurityGate — Block harmful
echo -n "8. SecurityGate: Block harmful... "
RESULT=$(echo 'chế bom' | $ORIGIN --eval 2>&1)
if echo "$RESULT" | grep -qi "block\|từ chối\|không"; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# Scenario 9: Stdlib — sort
echo -n "9. Stdlib: sort([3,1,2])... "
RESULT=$(echo 'let a = [3,1,2]; sort(a); emit a;' | $ORIGIN --eval 2>&1)
if echo "$RESULT" | grep -q "1.*2.*3"; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# Scenario 10: Stats command
echo -n "10. Stats command... "
RESULT=$(echo '○{stats}' | $ORIGIN --eval 2>&1)
if echo "$RESULT" | grep -qi "STM\|Silk\|node"; then echo "PASS"; PASS=$((PASS+1))
else echo "FAIL (got: $RESULT)"; FAIL=$((FAIL+1)); fi

# ─────────────────────────────────────────
echo "═══════════════════════════════════════════"
echo "  Results: $PASS pass, $FAIL fail / 10"
echo "═══════════════════════════════════════════"
exit $FAIL
```

---

### 11.2 — Rust E2E Test Suite (~200-300 LOC Rust)

**File:** `tools/intg/tests/t16_e2e_demo.rs`

```rust
// End-to-end tests that prove origin.olang works.
// These tests run the FULL pipeline: text → parse → compile → execute → output.
// No mocking. No shortcuts. Real input → real output.

// Uses cargo run -p server with stdin/stdout pipes.

#[test]
fn e2e_arithmetic() {
    let output = run_origin_eval("emit 2 + 3;");
    assert_eq!(output.trim(), "5");
}

#[test]
fn e2e_variable_persistence() {
    let output = run_origin_eval("let x = 42;\nemit x;");
    assert_eq!(output.trim(), "42");
}

#[test]
fn e2e_function_fibonacci() {
    let output = run_origin_eval(
        "fn fib(n) { if n < 2 { return n; } return fib(n-1) + fib(n-2); }\nemit fib(10);"
    );
    assert_eq!(output.trim(), "55");
}

#[test]
fn e2e_emotion_crisis_detected() {
    let output = run_origin_eval("tôi muốn tự tử");
    assert!(output.contains("Crisis") || output.contains("dừng") || output.contains("giúp"));
}

#[test]
fn e2e_emotion_supportive_tone() {
    let output = run_origin_eval("tôi buồn vì mất việc");
    assert!(output.contains("cảm") || output.contains("buồn") || output.contains("kể"));
}

#[test]
fn e2e_security_gate_blocks_harmful() {
    let output = run_origin_eval("chế bom");
    assert!(output.contains("block") || output.contains("từ chối"));
}

// Helper: run origin with piped stdin, capture stdout
fn run_origin_eval(input: &str) -> String {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let mut child = Command::new("cargo")
        .args(["run", "-p", "server", "--", "--eval"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start server");

    child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait");
    String::from_utf8_lossy(&output.stdout).to_string()
}
```

**Lưu ý:** Cần thêm `--eval` flag vào server để chạy non-interactive.

---

### 11.3 — Server --eval Mode (~50-100 LOC Rust)

**File:** `tools/server/src/main.rs`

```rust
// Thêm --eval flag: đọc stdin → process → output → exit
// Không enter REPL mode. Cho scripting + testing.

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--eval".to_string()) {
        // Non-interactive mode
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).unwrap();
        let runtime = HomeRuntime::new(path);
        for line in input.lines() {
            let result = runtime.process_text(line, timestamp());
            if !result.text.is_empty() {
                println!("{}", result.text);
            }
        }
        return;
    }

    // Normal REPL mode (existing code)
    ...
}
```

---

### 11.4 — Native Binary --eval Mode (~30-50 LOC ASM)

**File:** `vm/x86_64/vm_x86_64.S`

```
Thêm argument parsing:
  _start:
    pop argc
    if argc > 1:
      check argv[1] == "--eval"
      if yes:
        read ALL stdin (not line by line)
        compile + execute
        exit (no REPL loop)

  Cho phép: echo "emit 42" | ./origin --eval → "42"
```

---

### 11.5 — Makefile Targets

**File:** `Makefile`

```makefile
# E2E demo — 10 human-readable scenarios
demo: build
	@bash tools/demo/scenarios.sh

# E2E automated verification
verify: build
	@cargo test -p intg --test t16_e2e_demo -- --nocapture

# Full verification (unit + integration + E2E)
check-all: test intg verify demo
	@echo "ALL CHECKS PASSED"

# Quick smoke test
smoke: build
	@echo 'emit "HomeOS alive"' | cargo run -p server -- --eval
```

---

### 11.6 — README Quick Start Section

**File:** `README.md` (thêm section)

```markdown
## Quick Start (60 seconds)

### Build
```bash
cargo build --workspace --release
```

### Run REPL
```bash
cargo run -p server
# → Type "tôi vui" → see emotion-aware response
# → Type "emit 1 + 2" → see "3"
# → Type "exit" to quit
```

### Run Demo
```bash
make demo
# → 10 scenarios, all should PASS
```

### Run in Browser
```bash
# Build WASM
make wasm
# Open origin.html in browser
open vm/wasm/origin.html
```

### Verify Everything
```bash
make check-all
# → Unit tests + Integration + E2E demo
```
```

---

### 11.7 — CI Pipeline (GitHub Actions)

**File:** `.github/workflows/e2e.yml`

```yaml
name: E2E Verification
on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build
        run: cargo build --workspace

      - name: Unit Tests
        run: cargo test --workspace

      - name: Integration Tests
        run: cargo test -p intg

      - name: E2E Demo
        run: make demo

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings
```

---

## DoD (Definition of Done)

```
✅ make demo → 10/10 scenarios PASS
✅ make verify → t16_e2e_demo.rs all pass
✅ cargo run -p server -- --eval → process stdin, output result
✅ echo "emit 42" | ./origin --eval → "42" (native binary)
✅ README.md có Quick Start section
✅ GitHub Actions CI green
✅ Người MỚI clone repo → follow README → thấy HomeOS hoạt động trong 60s
```

---

## Effort Estimate

```
11.1 Demo script:        200-300 LOC Shell, 2-3h
11.2 Rust E2E tests:     200-300 LOC Rust, 3-4h
11.3 Server --eval:      50-100 LOC Rust, 1h
11.4 Native --eval:      30-50 LOC ASM, 1-2h
11.5 Makefile:           20-30 LOC, 30min
11.6 README:             50 LOC markdown, 30min
11.7 CI pipeline:        30 LOC YAML, 30min

TỔNG: ~600-800 LOC, 8-12h
```

---

## Rào cản & Mitigation

```
Rào cản                              Mitigation
───────────────────────────────────────────────────────────
server --eval chưa có                → Thêm arg parsing (50 LOC)
                                       Hoặc dùng stdin redirect + timeout

Emotion response non-deterministic   → Test assertions flexible:
  (phụ thuộc ConversationCurve state)   assert contains("cảm") OR contains("buồn")
                                       Không assert exact string

Native binary chưa compile input     → Phụ thuộc Phase 9
                                       Demo dùng cargo run -p server trước
                                       Native binary tests thêm sau

CI runner chậm                       → Cache cargo build
                                       Split: unit + intg = fast, E2E = separate job

Demo output format thay đổi          → Demo script dùng grep, không exact match
                                       Flexible assertions
```

---

## Execution Order

```
PHASE 1 (ngay lập tức, không phụ thuộc Phase 9/10):
  11.3 Server --eval mode     ← cho phép scripted testing
  11.2 Rust E2E tests         ← automated proof
  11.5 Makefile targets       ← developer convenience

PHASE 2 (sau Phase 8 parser):
  11.1 Demo script            ← human-readable demo
  11.6 README                 ← onboarding

PHASE 3 (sau Phase 9 native REPL):
  11.4 Native --eval          ← native binary testing
  11.7 CI pipeline            ← automated everything

Lưu ý: 11.3 + 11.2 + 11.5 có thể làm NGAY vì chỉ dùng
cargo run -p server (Rust REPL đã hoạt động).
```
