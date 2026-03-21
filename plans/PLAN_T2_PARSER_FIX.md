# PLAN T2 — Parser Fix: Chi Tiet Trien Khai 8.1, 8.2, 8.3

**Nguon:** PLAN_8_PARSER_UPGRADE.md
**Ngay tao:** 2026-03-22
**Muc tieu:** Xac nhan 54/54 .ol files parse OK. Fix bat ky loi parse nao con sot.

---

## Trang Thai Hien Tai (QUAN TRONG — DOC TRUOC)

```
PHAT HIEN: Phan lon cong viec PLAN 8 DA DUOC THUC HIEN.

✅ 8.1 Hex literals — DA IMPLEMENT
   File: crates/olang/src/lang/alphabet.rs:888-907
   lex_number() DA ho tro 0x/0X prefix, parse hex digits, tra Token::Int(value)
   Tests DA CO: lex_hex_literal_0xff, lex_hex_literal_0x00, lex_hex_literal_0x4f4c,
                lex_hex_uppercase_prefix, lex_hex_in_let_statement

✅ KNOWN_PARSE_FAILURES DA RONG
   File: tools/intg/tests/t13_stdlib_compile_audit.rs:56-58
   const KNOWN_PARSE_FAILURES: &[&str] = &[];
   Comment: "All 54 .ol files now parse successfully!"

✅ expect_ident() DA MO RONG
   File: crates/olang/src/lang/syntax.rs:2314-2336
   Chap nhan: Token::Command(s), Token::From, Token::Spawn, Token::Match,
              Token::Select, Token::Mut, Token::SelfKw, Token::Fn, Token::In,
              Token::Enum, Token::Struct

✅ is_dict_key() DA CO
   File: crates/olang/src/lang/syntax.rs:2133-2141
   Chap nhan keywords lam dict key

✅ CmpOp::Eq (==) DA PARSE DUNG
   File: crates/olang/src/lang/syntax.rs:1649-1671
   Token::Truth → CmpOp::Eq trong parse_compare_expr()
```

---

## VIEC CAN LAM — Xac nhan va tang cuong

Du phan lon da implement, can XAC NHAN + TANG CUONG de chac chan. 3 nhiem vu:

---

### T2.1 — Xac nhan Hex Literal Hoan Chinh (~10-20 LOC)

**Trang thai:** DA IMPLEMENT — can xac nhan edge cases

**File:** `crates/olang/src/lang/alphabet.rs`

**Code hien tai (dong 878-943):**
```rust
fn lex_number(&mut self) -> Token {
    let mut n: u32 = 0;
    // First digit
    if let Some(&(_, c)) = self.chars.peek() {
        if let Some(d) = c.to_digit(10) {
            n = d;
            self.chars.next();
        }
    }
    // Check for hex prefix: 0x or 0X
    if n == 0 {
        if let Some(&(_, c)) = self.chars.peek() {
            if c == 'x' || c == 'X' {
                self.chars.next(); // consume 'x'
                let mut hex_n: u32 = 0;
                let mut has_digits = false;
                while let Some(&(_, c)) = self.chars.peek() {
                    if let Some(d) = c.to_digit(16) {
                        hex_n = hex_n.saturating_mul(16).saturating_add(d);
                        has_digits = true;
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                if has_digits {
                    return Token::Int(hex_n);
                }
                return Token::Int(0);
            }
        }
    }
    // ... decimal + float ...
}
```

**Van de tiem an:**
1. `u32` overflow — `0xFFFFFFFF` = 4,294,967,295 (u32::MAX) nhung `0x100000000` se bi saturate
2. Khong ho tro binary (`0b1010`) va octal (`0o777`)
3. `0x` khong co digits → tra `Int(0)` thay vi bao loi

**De xuat fix (NEU CAN):**
```rust
// Them binary + octal support (OPTIONAL — chi lam neu co .ol file can)
if n == 0 {
    if let Some(&(_, c)) = self.chars.peek() {
        match c {
            'x' | 'X' => { /* da co */ }
            'b' | 'B' => {
                self.chars.next();
                let mut bin_n: u32 = 0;
                let mut has_digits = false;
                while let Some(&(_, c)) = self.chars.peek() {
                    if let Some(d) = c.to_digit(2) {
                        bin_n = bin_n.saturating_mul(2).saturating_add(d);
                        has_digits = true;
                        self.chars.next();
                    } else { break; }
                }
                if has_digits { return Token::Int(bin_n); }
                return Token::Int(0);
            }
            'o' | 'O' => { /* tuong tu voi to_digit(8) */ }
            _ => {}
        }
    }
}
```

**Tests can them:**
```rust
#[test] fn lex_hex_large_value()     // 0xFFFFFFFF → Int(4294967295)
#[test] fn lex_hex_overflow_saturate() // 0x100000000 → Int(u32::MAX)
#[test] fn lex_plain_zero()          // 0 → Int(0) (khong phai hex)
#[test] fn lex_zero_followed_by_dot() // 0.5 → Float(0.5) (khong phai hex)
```

**Lenh kiem tra:**
```bash
cargo test -p olang -- lex_hex
cargo test -p olang -- lex_number
```

**Uoc tinh LOC:** 10-20 (them tests, khong can sua code chinh)
**Rui ro:** THAP — code da hoat dong, chi them tests
**Dinh nghia hoan thanh:**
- [ ] `cargo test -p olang -- lex_hex` → 6+ tests PASS
- [ ] `0xFFFFFFFF` tokenize dung (hoac saturate co bao)
- [ ] `0` + `0.5` khong bi nham la hex

---

### T2.2 — Xac nhan == Trong Moi Context (~20-50 LOC)

**Trang thai:** DA IMPLEMENT — can kiem tra tung file cu fail

**File chinh:** `crates/olang/src/lang/syntax.rs`

**Code hien tai (dong 1649-1671):**
```rust
fn parse_compare_expr(&mut self) -> Result<Expr, ParseError> {
    let left = self.parse_arith_expr()?;
    let cmp_op = match self.peek() {
        Token::Lt => Some(CmpOp::Lt),
        Token::Gt => Some(CmpOp::Gt),
        Token::Le => Some(CmpOp::Le),
        Token::Ge => Some(CmpOp::Ge),
        Token::Ne => Some(CmpOp::Ne),
        Token::Truth => Some(CmpOp::Eq),  // ← == da duoc handle
        _ => None,
    };
    if let Some(op) = cmp_op {
        self.advance();
        let right = self.parse_arith_expr()?;
        Ok(Expr::Compare { lhs: Box::new(left), op, rhs: Box::new(right) })
    } else {
        Ok(left)
    }
}
```

**parse_match_pattern() (dong 845-901) — CHUA ho tro == trong pattern:**
```rust
fn parse_match_pattern(&mut self) -> Result<MatchPattern, ParseError> {
    match self.peek() {
        Token::Ident(s) if s == "_" => { ... Wildcard }
        Token::Circle => { ... MolConstraintPattern }
        Token::LBrace => { ... MolLiteral }
        Token::Ident(_) => { ... TypeName hoac EnumPattern }
        _ => Err("Expected match pattern ...")  // ← == se fail o day
    }
}
```

**Van de tiem an:**
Match pattern chua ho tro expression patterns:
```
match x {
    0 => { ... }        // ✅ Int literal pattern — chua co!
    val == 0 => { ... } // ❌ Comparison trong pattern — chua co!
    "hello" => { ... }  // ❌ String literal pattern — chua co!
}
```

**De xuat fix:**
```rust
fn parse_match_pattern(&mut self) -> Result<MatchPattern, ParseError> {
    match self.peek() {
        Token::Ident(s) if s == "_" => { ... }
        Token::Circle => { ... }
        Token::LBrace => { ... }
        // THEM: Int literal patterns
        Token::Int(n) => {
            let n = *n;  // copy value truoc khi advance
            self.advance();
            Ok(MatchPattern::IntLiteral(n))
        }
        // THEM: String literal patterns
        Token::Str(s) => {
            let s = s.clone();
            self.advance();
            Ok(MatchPattern::StringLiteral(s))
        }
        Token::Ident(_) => { ... }
        _ => Err("Expected match pattern ...")
    }
}
```

**Phu thuoc:** Can them variants vao `MatchPattern` enum:
```rust
// File: crates/olang/src/lang/syntax.rs (khoang dong 230-260)
pub enum MatchPattern {
    TypeName(String),
    EnumPattern { ... },
    MolLiteral { ... },
    MolConstraintPattern { ... },
    Wildcard,
    IntLiteral(u32),      // THEM
    StringLiteral(String), // THEM
    BoolLiteral(bool),    // THEM
}
```

**Cach kiem tra 9 files cu:**
```bash
# Chay t13 audit de xac nhan 54/54 parse OK
cargo test -p intg -- t13_stdlib_compile_audit

# Kiem tra cu the tung file
# (t13 da check — nhung co the viet test rieng)
cargo test -p olang -- parse_match
```

**Test can them:**
```rust
#[test]
fn parse_match_int_literal_pattern() {
    let stmts = parse("match x { 0 => { zero(); } _ => { other(); } }").unwrap();
    assert!(matches!(stmts[0], Stmt::Match { .. }));
}

#[test]
fn parse_match_string_literal_pattern() {
    let stmts = parse("match cmd { \"exit\" => { quit(); } _ => { run(); } }").unwrap();
    assert!(matches!(stmts[0], Stmt::Match { .. }));
}

#[test]
fn parse_comparison_in_if() {
    let stmts = parse("if x == 0 { zero(); }").unwrap();
    assert!(matches!(stmts[0], Stmt::If { .. }));
}

#[test]
fn parse_comparison_field_access() {
    let stmts = parse("if obj.field == \"value\" { ok(); }").unwrap();
    assert!(matches!(stmts[0], Stmt::If { .. }));
}
```

**Lenh kiem tra:**
```bash
cargo test -p olang -- parse_match
cargo test -p olang -- parse_compare
cargo test -p intg -- t13_stdlib_compile_audit
```

**Uoc tinh LOC:** 20-50 (them match pattern variants + tests)
**Rui ro:** TRUNG BINH — them MatchPattern variant can cap nhat semantic.rs + compiler
**Dinh nghia hoan thanh:**
- [ ] `cargo test -p intg -- t13_stdlib_compile_audit` → 54/54 parse OK
- [ ] Match voi int literal pattern hoat dong
- [ ] Match voi string literal pattern hoat dong
- [ ] `if a.field == b` parse thanh cong
- [ ] 4+ tests moi PASS

---

### T2.3 — Xac nhan Keywords as Identifiers + Struct Colon (~10-30 LOC)

**Trang thai:** PHAN LON DA IMPLEMENT — can xac nhan

**File chinh:** `crates/olang/src/lang/syntax.rs`

**expect_ident() DA MO RONG (dong 2314-2336):**
```rust
fn expect_ident(&mut self) -> Result<String, ParseError> {
    match self.advance() {
        Token::Ident(s) => Ok(s),
        Token::From => Ok("from".into()),
        Token::Enum => Ok("union".into()),
        Token::Struct => Ok("type".into()),
        Token::Fn => Ok("fn".into()),
        Token::In => Ok("in".into()),
        Token::Command(s) => Ok(s),        // ← "learn", "trace", etc. DA DUOC XU LY
        Token::Spawn => Ok("spawn".into()),
        Token::Match => Ok("match".into()),
        Token::Select => Ok("select".into()),
        Token::Mut => Ok("mut".into()),
        Token::SelfKw => Ok("self".into()),
        other => Err(...)
    }
}
```

**is_struct_literal() (dong 2118-2130):**
```rust
fn is_struct_literal(&self, name: &str) -> bool {
    // Name must start with uppercase  ← GIOI HAN: "edge" se fail
    if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return false;
    }
    // Look ahead: { ident :
    if self.pos + 2 < self.tokens.len() {
        matches!(&self.tokens[self.pos + 1], Token::Ident(_))
            && matches!(&self.tokens[self.pos + 2], Token::Colon)
    } else {
        false
    }
}
```

**Van de con lai:**
1. **intent.ol "learn":** DA FIX — `Token::Command(s) => Ok(s)` trong expect_ident()
2. **silk_ops.ol struct literal lowercase:** `edge { from: x, to: y }`
   - `is_struct_literal("edge")` tra `false` vi lowercase
   - DA DUOC FIX bang cach khac — check KNOWN_PARSE_FAILURES = []

**De xuat fix (NEU silk_ops.ol van dung lowercase struct):**
```rust
fn is_struct_literal(&self, name: &str) -> bool {
    // Relax: cho phep lowercase NEU co lookahead { ident : pattern
    // (gioi han: khong nham voi block scope)
    if self.pos + 2 < self.tokens.len() {
        matches!(&self.tokens[self.pos + 1], Token::Ident(_))
            && matches!(&self.tokens[self.pos + 2], Token::Colon)
    } else {
        false
    }
    // LUU Y: Bo uppercase constraint co the gay nham:
    //   foo { x: 1 }  ← struct literal  (DUNG)
    //   foo { x = 1 } ← call + block    (KHAC — co '=' khong phai ':')
    // Vi lookahead check Ident + Colon, nen van an toan.
}
```

**Test can them:**
```rust
#[test]
fn parse_command_keyword_as_ident() {
    // "learn" la Token::Command nhung co the dung lam ident
    let stmts = parse("let learn = 42;").unwrap();
    assert!(!stmts.is_empty());
}

#[test]
fn parse_struct_literal_uppercase() {
    let stmts = parse("let p = Point { x: 1, y: 2 };").unwrap();
    assert!(!stmts.is_empty());
}
```

**Lenh kiem tra:**
```bash
cargo test -p olang -- parse_command_keyword
cargo test -p olang -- parse_struct_literal
cargo test -p intg -- t13_stdlib_compile_audit
```

**Uoc tinh LOC:** 10-30 (them tests, co the sua is_struct_literal)
**Rui ro:** THAP — phan lon da fix, chi xac nhan
**Dinh nghia hoan thanh:**
- [ ] `cargo test -p intg -- t13_stdlib_compile_audit` → 54/54 parse OK
- [ ] `let learn = 42` parse thanh cong
- [ ] Struct literal uppercase parse dung
- [ ] 0 clippy warnings

---

## Thu Tu Thuc Hien

```
T2.1 (Xac nhan hex)     ← Nhanh nhat, it rui ro, 15 phut
  ↓
T2.2 (Xac nhan ==)      ← Quan trong nhat, co the can them MatchPattern variants
  ↓
T2.3 (Keywords/struct)   ← Cuoi cung, it viec nhat
```

**Tong uoc tinh:** 40-100 LOC, 1-2h (phan lon la viet tests)

---

## Lenh Kiem Tra Toan Dien

```bash
# 1. Chay toan bo tests workspace
cargo test --workspace

# 2. Chay cu the parser tests
cargo test -p olang -- lex_hex
cargo test -p olang -- parse_match
cargo test -p olang -- parse_compare

# 3. Chay t13 audit — PHAI 54/54
cargo test -p intg -- t13_stdlib_compile_audit

# 4. Clippy
cargo clippy --workspace

# 5. Smoke binary
make smoke-binary
```

---

## Danh Gia Rui Ro

```
Rui ro                              Muc do    Giam thieu
──────────────────────────────────────────────────────────────
Them MatchPattern variant           TRUNG BINH  Cap nhat semantic.rs + compiler.rs cung luc
                                                  Chay cargo test --workspace sau moi thay doi

is_struct_literal() bo uppercase    THAP        Lookahead { ident : du de phan biet
                                                  struct literal vs block scope

u32 overflow cho hex lon            THAP        saturating_mul da co, chi them test
                                                  Neu can u64, doi Token::Int(u32) → Token::Int(u64)

Regression tests cu                 RAT THAP    t13 audit la safety net — 54/54 files
```

---

## Dinh Nghia Hoan Thanh Toan Bo T2

```
✅ cargo test -p intg -- t13_stdlib_compile_audit → 54/54 parse OK
✅ cargo test -p olang -- lex_hex → 6+ tests PASS
✅ cargo test -p olang -- parse_match → match voi int/string patterns
✅ cargo test -p olang -- parse_compare → == trong moi context
✅ cargo test --workspace → 0 FAILED
✅ cargo clippy --workspace → 0 warnings
✅ KNOWN_PARSE_FAILURES van = [] (khong them file moi vao)
```
