# PLAN 0.2 — Test parser.ol trên Rust VM

**Phụ thuộc:** PLAN_0_1 phải xong (lexer.ol chạy được)
**Mục tiêu:** `parser.ol` (399 LOC) import `lexer.ol`, parse tokens → AST, chạy trên Rust VM.
**Yêu cầu:** Biết Rust. KHÔNG cần biết Olang.

---

## Bối cảnh

`stdlib/bootstrap/parser.ol` là recursive descent parser viết bằng Olang:
- Import lexer: `use olang.bootstrap.lexer;`
- Input: token list (từ `tokenize()`)
- Output: AST nodes (`Vec[Stmt]`)
- Hỗ trợ: let, fn, if/else, while, return, emit, type, union, match

### AST types (định nghĩa trong parser.ol)

```
union Expr {
    Ident { name: Str },
    NumLit { value: Num },
    StrLit { value: Str },
    BinOp { op: Str, lhs: Expr, rhs: Expr },
    Call { callee: Expr, args: Vec[Expr] },
    FieldAccess { object: Expr, field: Str },
    MolLiteral { s: Num, r: Num, v: Num, a: Num, t: Num },
}

union Stmt {
    LetStmt, ExprStmt, FnDef, IfStmt, WhileStmt,
    ReturnStmt, EmitStmt, TypeDef, UnionDef,
}
```

### Chuỗi thực thi

```
"fn f(x) { return x + 1; }"
    ↓ tokenize() (lexer.ol — từ PLAN_0_1)
[Keyword("fn"), Ident("f"), Symbol("("), Ident("x"), Symbol(")"),
 Symbol("{"), Keyword("return"), Ident("x"), Symbol("+"), Number(1),
 Symbol(";"), Symbol("}")]
    ↓ parse() (parser.ol — PLAN này)
[FnDef { name: "f", params: ["x"], body: [ReturnStmt { ... }] }]
```

---

## File cần đọc

| File | Đọc gì |
|------|---------|
| `stdlib/bootstrap/parser.ol` | Toàn bộ 399 LOC — recursive descent parser |
| `stdlib/bootstrap/lexer.ol` | Đã đọc ở PLAN_0_1 — tokenizer |
| `crates/olang/src/exec/module.rs:374+` | ModuleLoader — cách resolve `use` |

---

## Việc cần làm

### Task 1: Module import hoạt động

parser.ol dòng 8: `use olang.bootstrap.lexer;`

Đây là test đầu tiên cho **module system**. ModuleLoader phải:
1. Resolve `olang.bootstrap.lexer` → `stdlib/bootstrap/lexer.ol`
2. Parse + compile lexer.ol
3. Inject exports (fn `tokenize`) vào scope của parser.ol
4. Parse + compile parser.ol

```rust
// crates/olang/tests/bootstrap_parser.rs

#[test]
fn test_parser_ol_imports_lexer() {
    let mut loader = ModuleLoader::new();
    loader.add_search_path("../../stdlib");

    // Load parser.ol — phải tự kéo lexer.ol theo
    let module = loader.load("olang.bootstrap.parser")
        .expect("parser.ol should load with lexer.ol dependency");

    // Verify exports
    assert!(module.exports.iter().any(|s| s.name == "parse"),
        "parser.ol must export parse()");
}
```

**Rào cản:** ModuleLoader có thể chưa wire `use` → load dependency tự động.
**Giải pháp:** Xem `module.rs` — nếu `load()` chưa resolve `use`, cần implement:
```rust
// Trong ModuleLoader::load():
// 1. Parse source → AST
// 2. Scan AST cho Stmt::Use { path: "olang.bootstrap.lexer" }
// 3. Recursively load dependency
// 4. Inject dependency exports vào current scope
// 5. Compile current module
```

### Task 2: parse() chạy trên VM

```rust
#[test]
fn test_parser_ol_parse_let() {
    // Setup: load lexer.ol + parser.ol

    // Tạo wrapper program:
    //   1. Gọi tokenize("let x = 42;") → tokens
    //   2. Gọi parse(tokens) → AST
    //   3. EMIT AST

    let vm = OlangVM::new();
    let events = vm.execute(&wrapper);

    // Verify: AST chứa 1 LetStmt { name: "x", value: NumLit(42) }
}

#[test]
fn test_parser_ol_parse_fn() {
    // tokenize("fn f(x) { return x + 1; }") → parse → AST
    // Verify: 1 FnDef { name: "f", params: ["x"], body: [ReturnStmt] }
}

#[test]
fn test_parser_ol_parse_if_else() {
    // tokenize("if x > 0 { emit x; } else { emit y; }") → parse → AST
    // Verify: 1 IfStmt with then_block and else_block
}
```

### Task 3: Verify nhiều statements

```rust
#[test]
fn test_parser_ol_parse_multi_stmt() {
    let source = r#"
        let x = 1;
        let y = 2;
        fn add(a, b) { return a + b; }
        emit add(x, y);
    "#;

    // tokenize → parse → verify 4 statements
    // [LetStmt, LetStmt, FnDef, EmitStmt]
}
```

---

## Rào cản có thể gặp

### 1. Module import chưa implement
**Mức độ:** BLOCKING — parser.ol KHÔNG chạy nếu không import được lexer.ol
**Giải pháp:** Implement `use` resolution trong ModuleLoader (xem Task 1)
**Ước tính:** 4-8 giờ nếu chưa có

### 2. Recursive Expr (BinOp chứa BinOp)
**Triệu chứng:** VM không hỗ trợ recursive data structures
**Giải pháp:** Olang `union` cần map sang VM representation. Có thể dùng:
- Heap-allocated chains (mỗi AST node = 1 chain_hash, body trong registry)
- Hoặc flatten AST thành linear list

### 3. parser.ol dùng `Vec[Expr]` — generic type
**Triệu chứng:** Rust compiler (semantic.rs) chưa hỗ trợ `Vec[T]` syntax
**Giải pháp:** Map `Vec[T]` → dynamic array trong VM (dùng MolecularChain list)

---

## Definition of Done

- [ ] `use olang.bootstrap.lexer;` resolve thành công
- [ ] parser.ol compile không lỗi
- [ ] `parse(tokenize("let x = 42;"))` → 1 LetStmt
- [ ] `parse(tokenize("fn f(x) { return x + 1; }"))` → 1 FnDef
- [ ] `parse(tokenize("if x > 0 { emit x; }"))` → 1 IfStmt

## Ước tính

- Nếu module system đã wire: **4-8 giờ**
- Nếu cần implement module import: **2-3 ngày**

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 0.2*
