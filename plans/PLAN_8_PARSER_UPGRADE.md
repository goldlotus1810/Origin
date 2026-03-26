# PLAN 8 — Parser Upgrade: Unlock 24 Failing .ol Files

**Phụ thuộc:** Phase 0 DONE (bootstrap compiler)
**Mục tiêu:** Parser hỗ trợ hex literals, match ==, struct colon syntax → 54/54 files parse OK

---

## Bối cảnh

```
HIỆN TẠI:
  54 .ol files trong stdlib/
  30 parse OK (55%)
  24 parse FAIL (45%) — 3 loại lỗi chính:

  Lỗi 1: HEX LITERALS (0xFF) — 13 files
    asm_emit.ol, asm_emit_arm64.ol, elf_emit.ol, reproduce.ol, wasm_emit.ol,
    isl_tcp.ol, isl_ws.ol, isl_ble.ol, isl_discovery.ol,
    builder.ol, fat_header.ol, fat_loader.ol, + có thể thêm

    Root cause: lex_number() chỉ hỗ trợ base-10
    "0xFF" → Token::Int(0) + Token::Ident("xFF") → parse error

  Lỗi 2: MATCH / COMPARISON (==) — 9 files
    benchmark.ol, dream.ol, dream_cache.ol, install.ol, jit.ol,
    module_index.ol, mol_pool.ol, optimize.ol, registry_cache.ol, silk_cache.ol

    Root cause: Token::Truth (==) ĐÃ ĐƯỢC tokenize đúng, nhưng
    một số context (match patterns, field comparison) chưa handle đúng.
    Cần investigate từng file cụ thể.

  Lỗi 3: KEYWORDS AS IDENTIFIERS + STRUCT COLON — 2 files
    intent.ol   — "learn" là Command keyword
    silk_ops.ol — lowercase struct literal với colon syntax

SAU PLAN 8:
  54/54 files parse OK
  Toàn bộ stdlib compile thành bytecode
  Unlocks: native REPL có thể chạy TẤT CẢ code
```

---

## Tasks

### 8.1 — Hex Literal Support (~50-80 LOC Rust)

**File:** `crates/olang/src/lang/alphabet.rs` — function `lex_number()` (line ~878)

```
HIỆN TẠI:
  lex_number() {
    c.to_digit(10) → chỉ base-10
    "0xFF" → Int(0) + Ident("xFF")
  }

SAU FIX:
  lex_number() {
    if first_digit == '0' && peek == 'x'|'X' {
      consume 'x'
      loop: c.to_digit(16) → accumulate hex value
      return Token::Int(hex_value)
    }
    // fallback: decimal as before
  }

  "0xFF" → Token::Int(255)
  "0x4F4C" → Token::Int(20300)
  "0x00" → Token::Int(0)
  "0xDEAD" → Token::Int(57005)

OPTIONAL (bonus):
  "0b1010" → Token::Int(10)  // binary
  "0o777"  → Token::Int(511) // octal
```

**Tests cần viết:**
```rust
#[test] fn lex_hex_literal_0xff()     // 0xFF → Int(255)
#[test] fn lex_hex_literal_0x00()     // 0x00 → Int(0)
#[test] fn lex_hex_literal_0x4f4c()   // 0x4F4C → Int(20300)
#[test] fn lex_hex_literal_uppercase() // 0XFF → Int(255)
#[test] fn lex_hex_in_let()           // let x = 0xFF; → LetStmt
#[test] fn lex_hex_in_comparison()    // if x == 0xFF { ... }
#[test] fn lex_hex_zero_prefix()      // 0 (plain zero) still works
#[test] fn lex_hex_no_digits()        // 0x → error or Int(0) + Ident("x")
```

**Impact:** Giải quyết 13/24 files (54% of failures)

**Rào cản:**
- `lex_number()` trả về `Token::Int(i64)` hay `Token::Int(u64)`? Hex values có thể lớn.
- Cần kiểm tra overflow: 0xFFFFFFFFFFFFFFFF > i64::MAX
- Parser dùng `Token::Int` ở đâu? Có cần change downstream?

---

### 8.2 — Investigate & Fix == in Context (~100-200 LOC Rust)

**TRƯỚC KHI CODE:** Chạy parser trên từng file failing → ghi lại CHÍNH XÁC lỗi gì.

```
Cần investigate:
  cargo test -p olang -- parse_benchmark  (nếu có)
  Hoặc viết quick test:
    let src = include_str!("../../stdlib/homeos/benchmark.ol");
    let result = parse(src);
    println!("{:?}", result.err());

Hypothesis:
  Token::Truth (==) ĐÃ tokenize đúng (B4 fix: alphabet.rs:715)
  Nhưng parser có thể fail ở:
    a) match arm pattern: match x { val == other => ... }
    b) Comparison trong while/if: while state.ops == expected { ... }
    c) Field access chain: obj.field == value
    d) Nested struct: { key: val == other }

  Mỗi case cần fix khác nhau.
```

**Approach:**
1. Parse từng file, ghi lại error message + line number
2. Group by error pattern
3. Fix từng pattern

**Likely fixes:**
```
Pattern A — match arm comparison:
  match_arm() chưa handle CmpOp trong pattern
  → Extend parse_pattern() to allow comparison expressions

Pattern B — comparison chain:
  a.field == b.field chưa parse vì field access + == chưa combine đúng
  → Check parse_comparison() precedence

Pattern C — struct field with ==:
  { key: expr == other } → parser stops at ==
  → parse_expr() trong struct literal context cần handle ==

Mỗi pattern: ~30-50 LOC fix.
```

**Impact:** Giải quyết 9/24 files (37% of failures)

---

### 8.3 — Keywords as Identifiers + Struct Colon (~50-100 LOC Rust)

**File:** `crates/olang/src/lang/syntax.rs`

```
intent.ol: "learn" là Command keyword
  → Khi parser gặp "learn" → Token::Command("learn") thay vì Token::Ident("learn")
  → Nhưng trong struct field context: { action: "learn" } → cần là string, OK
  → Hoặc: let learn = ... → cần là ident

  Fix: expect_ident() thêm Token::Command(s) → Ok(s.into())
  HOẶC: alphabet.rs chỉ tạo Token::Command khi ở statement-start position

silk_ops.ol: lowercase struct literal
  → is_struct_literal() yêu cầu uppercase first char
  → silk_ops.ol dùng: edge { from: hash_a, to: hash_b, weight: 0.8 }
  → "edge" là lowercase → parser coi { ... } là block, không phải struct

  Fix: Relax is_struct_literal() constraint
  HOẶC: Dùng lookahead — nếu { ident : expr, ... } → struct literal
  HOẶC: Dùng explicit constructor: Edge { from: ..., to: ... }

  Rào cản: Phân biệt struct literal vs block scope:
    foo { x: 1 }  ← struct literal
    foo { x = 1 } ← expression + block

  Lookahead giải pháp:
    Sau Ident + "{", peek ahead:
    - Nếu thấy Ident + ":" → struct literal
    - Nếu thấy Ident + "=" hoặc keyword → block
```

**Impact:** Giải quyết 2/24 files (8% of failures)

---

### 8.4 — Audit & Remove from KNOWN_PARSE_FAILURES

```
Sau khi fix 8.1 + 8.2 + 8.3:
  1. Chạy t13_stdlib_compile_audit.rs
  2. Files nào parse OK → xóa khỏi KNOWN_PARSE_FAILURES
  3. Files nào vẫn fail → investigate, document, hoặc fix code .ol
  4. Mục tiêu: KNOWN_PARSE_FAILURES = [] (empty)
```

---

## DoD (Definition of Done)

```
✅ lex_number() hỗ trợ 0x hex literals
✅ Mọi 24 failing files parse OK (hoặc có documented reason nếu vẫn fail)
✅ KNOWN_PARSE_FAILURES reduced to 0 (hoặc < 3 với documented reason)
✅ t13_stdlib_compile_audit: 54/54 files compile + decode
✅ Không break existing tests (cargo test --workspace pass)
✅ ≥ 10 new parser tests cho hex, ==, struct syntax
✅ 0 clippy warnings
```

---

## Effort Estimate

```
8.1 Hex literals:          50-80 LOC, 1-2h      ← HIGH priority, LOW risk
8.2 == in context:        100-200 LOC, 2-4h     ← HIGH priority, MEDIUM risk
8.3 Keywords + struct:     50-100 LOC, 1-2h      ← LOW priority, MEDIUM risk
8.4 Audit cleanup:         20 LOC, 30min         ← cleanup

TỔNG: ~250-400 LOC Rust, 4-8h
```

---

## Rào cản & Mitigation

```
Rào cản                              Mitigation
───────────────────────────────────────────────────────────
Parser regression                    → Run cargo test --workspace TRƯỚC + SAU mỗi fix
                                       t13_stdlib_compile_audit là safety net

== fix quá phức tạp                  → Investigate TỪNG file trước khi code
                                       Có thể chỉ cần thay đổi .ol source thay vì parser

Struct literal ambiguity             → Dùng uppercase convention nếu lookahead quá phức tạp
                                       silk_ops.ol có thể rename Edge { ... }

Hex overflow                         → Giới hạn hex ở i64::MAX (0x7FFFFFFFFFFFFFFF)
                                       Hoặc u64 + wrap
```
