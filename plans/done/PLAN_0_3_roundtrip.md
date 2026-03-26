# PLAN 0.3 — Round-trip: lexer.ol parse chính nó

**Phụ thuộc:** PLAN_0_1 + PLAN_0_2 phải xong
**Mục tiêu:** `lexer.ol` source → `tokenize()` → `parse()` → AST. Chứng minh bootstrap compiler hiểu chính nó.
**Yêu cầu:** Biết Rust. KHÔNG cần biết Olang.

---

## Tại sao quan trọng?

Đây là **proof of self-awareness**: compiler viết bằng Olang phải hiểu syntax của chính Olang.

```
lexer.ol (197 LOC text)
    ↓ read source as string
    ↓ tokenize(source)    ← lexer.ol parse chính nó
    ↓ parse(tokens)       ← parser.ol parse chính nó
    ↓ verify AST
    ✓ lexer.ol tự hiểu cấu trúc của mình
```

---

## Việc cần làm

### Task 1: lexer.ol tokenize chính nó

```rust
#[test]
fn test_lexer_ol_self_tokenize() {
    let source = std::fs::read_to_string("../../stdlib/bootstrap/lexer.ol").unwrap();

    // Chạy tokenize(source) trên VM
    // lexer.ol source = 197 dòng Olang

    let tokens = run_tokenize(&source);

    // Verify: phải có tokens (không crash, không empty)
    assert!(tokens.len() > 100, "lexer.ol should produce many tokens");

    // Verify keyword tokens: union, type, let, fn, if, else, while, return, pub
    let keywords: Vec<_> = tokens.iter()
        .filter(|t| matches!(t.kind, TokenKind::Keyword { .. }))
        .collect();

    // lexer.ol chứa: 1 union, 1 type, 1 let (KEYWORDS), 6 fn, nhiều if/else/while/return
    assert!(keywords.len() >= 10, "lexer.ol has many keywords");
}
```

### Task 2: parser.ol parse tokens từ lexer.ol

```rust
#[test]
fn test_lexer_ol_self_parse() {
    let source = std::fs::read_to_string("../../stdlib/bootstrap/lexer.ol").unwrap();

    // tokenize → parse
    let ast = run_parse(&run_tokenize(&source));

    // Verify cấu trúc AST của lexer.ol:
    //
    // lexer.ol chứa:
    //   1 union TokenKind { ... }     → 1 UnionDef
    //   1 type Token { ... }          → 1 TypeDef
    //   1 let KEYWORDS = [...]        → 1 LetStmt
    //   6 fn (is_keyword, is_alpha, is_digit, is_whitespace, tokenize, +1)
    //                                 → 6 FnDef

    let fn_count = ast.iter()
        .filter(|s| matches!(s, Stmt::FnDef { .. }))
        .count();

    let union_count = ast.iter()
        .filter(|s| matches!(s, Stmt::UnionDef { .. }))
        .count();

    assert_eq!(union_count, 1, "lexer.ol has 1 union (TokenKind)");
    assert_eq!(fn_count, 6, "lexer.ol has 6 functions");
}
```

### Task 3: parser.ol parse chính nó (399 LOC)

```rust
#[test]
fn test_parser_ol_self_parse() {
    let source = std::fs::read_to_string("../../stdlib/bootstrap/parser.ol").unwrap();

    let ast = run_parse(&run_tokenize(&source));

    // parser.ol chứa:
    //   use olang.bootstrap.lexer;    → 1 UseStmt
    //   2 union (Expr, Stmt)          → 2 UnionDef
    //   2 type (Field, Variant)       → 2 TypeDef
    //   1 type Parser { ... }         → 1 TypeDef (total 3)
    //   N fn (parse, parse_stmt, parse_expr, ...)

    let union_count = count_unions(&ast);
    let type_count = count_types(&ast);

    assert_eq!(union_count, 2, "parser.ol has 2 unions (Expr, Stmt)");
    assert!(type_count >= 3, "parser.ol has ≥3 type defs");
}
```

---

## Rào cản

### 1. lexer.ol chứa `pub fn` — visibility modifier
**Triệu chứng:** Tokenizer hoặc parser không hiểu `pub` trước `fn`
**Giải pháp:** Đảm bảo `pub` nằm trong KEYWORDS table và parser xử lý `pub fn` → FnDef với visibility Public

### 2. Self-referential: source chứa string literals có escape
**Triệu chứng:** `"\n"`, `"\\"`, `"\""` trong source bị tokenize sai
**Giải pháp:** lexer.ol dòng 139 xử lý escape, nhưng cần verify VM string handling

---

## Definition of Done

- [ ] `tokenize(lexer_source)` không crash, sản xuất >100 tokens
- [ ] `parse(tokenize(lexer_source))` → AST với 1 union, 1 type, 1 let, 6 fn
- [ ] `parse(tokenize(parser_source))` → AST với 2 union, ≥3 type
- [ ] Không có token nào bị Unknown/Error

## Ước tính

- Nếu PLAN_0_1 + 0_2 đã xong: **2-4 giờ** (chủ yếu viết test + fix edge cases)

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 0.3*
