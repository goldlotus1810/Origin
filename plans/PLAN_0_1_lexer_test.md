# PLAN 0.1 — Test lexer.ol trên Rust VM

**Mục tiêu:** Chứng minh `stdlib/bootstrap/lexer.ol` (197 LOC Olang) chạy được trên Rust VM hiện tại.

**Yêu cầu:** Biết Rust. KHÔNG cần biết Olang.

---

## Bối cảnh cho Rust developer

### Olang là gì?

Olang = ngôn ngữ lập trình riêng của HomeOS. Cú pháp giống Rust/TypeScript:
- `let x = 42;` — khai báo biến
- `fn foo(x) { return x + 1; }` — hàm
- `union TokenKind { Keyword { name: Str }, Eof }` — enum
- `type Token { kind: TokenKind, text: Str }` — struct

### Hệ thống hiện tại

```
lexer.ol (Olang source)
    ↓ parse bởi Rust
crates/olang/src/lang/syntax.rs     → Stmt/Expr AST
    ↓ compile bởi Rust
crates/olang/src/lang/semantic.rs   → OlangProgram (Vec<Op>)
    ↓ execute bởi Rust
crates/olang/src/exec/vm.rs         → VmEvent (output)
```

### File cần đọc trước khi code

| File | Dòng | Đọc gì |
|------|------|---------|
| `stdlib/bootstrap/lexer.ol` | 1-197 | Toàn bộ — tokenizer viết bằng Olang |
| `crates/olang/src/exec/module.rs` | 1-80 | ModuleLoader, CompiledModule, SymbolKind |
| `crates/olang/src/exec/vm.rs` | 541-600 | OlangVM struct, execute() entry |
| `crates/olang/src/exec/ir.rs` | 44-170 | Op enum — 36+ opcodes |
| `crates/olang/src/lang/semantic.rs` | 1-100 | compile_program() entry |

---

## Việc cần làm

### Task 1: Viết integration test load lexer.ol

**File tạo:** `crates/olang/tests/bootstrap_lexer.rs`

```rust
//! Integration test: load lexer.ol qua ModuleLoader → compile → run trên VM.

use olang::module::ModuleLoader;
use olang::vm::OlangVM;

#[test]
fn test_lexer_ol_loads_and_compiles() {
    // 1. Tạo ModuleLoader với search path = stdlib/bootstrap/
    let mut loader = ModuleLoader::new();
    loader.add_search_path("../../stdlib/bootstrap");

    // 2. Load lexer.ol → parse → compile → OlangProgram
    let module = loader.load("olang.bootstrap.lexer")
        .expect("lexer.ol should parse and compile");

    // 3. Verify exports: pub fn tokenize(source) phải tồn tại
    assert!(module.exports.iter().any(|s| s.name == "tokenize"),
        "lexer.ol must export tokenize()");
}
```

**Nếu ModuleLoader chưa hoạt động end-to-end**, thì dùng cách thủ công:

```rust
use std::fs;
use olang::syntax::Parser;
use olang::semantic::compile_program;
use olang::vm::OlangVM;

#[test]
fn test_lexer_ol_manual_pipeline() {
    // 1. Đọc source
    let source = fs::read_to_string("../../stdlib/bootstrap/lexer.ol")
        .expect("lexer.ol must exist");

    // 2. Parse → AST
    let ast = Parser::new(&source).parse_program()
        .expect("lexer.ol must parse");

    // 3. Compile AST → OlangProgram
    let program = compile_program(&ast)
        .expect("lexer.ol must compile");

    // 4. Verify: program có opcodes
    assert!(!program.ops.is_empty(), "compiled program must have ops");

    // 5. Print stats
    println!("lexer.ol compiled: {} ops", program.ops.len());
}
```

### Task 2: Test tokenize() chạy trên VM

```rust
#[test]
fn test_lexer_ol_tokenize_simple() {
    // Load + compile lexer.ol (dùng helper từ Task 1)
    let program = load_and_compile_lexer();

    // Tạo program gọi tokenize("let x = 42;")
    // Cần tạo wrapper program:
    //   PUSH "let x = 42;"     ← input
    //   CALL "tokenize"         ← gọi hàm từ lexer.ol
    //   EMIT                    ← output kết quả

    let vm = OlangVM::new();
    let events = vm.execute(&wrapper_program);

    // Verify: output phải chứa tokens
    // Token: Keyword("let"), Ident("x"), Symbol("="), Number(42), Symbol(";"), Eof
    let outputs: Vec<_> = events.iter()
        .filter_map(|e| match e {
            VmEvent::Output(chain) => Some(chain),
            _ => None,
        })
        .collect();

    assert!(!outputs.is_empty(), "tokenize must produce output");
}
```

### Task 3: Verify token count

```rust
#[test]
fn test_lexer_ol_token_count() {
    // tokenize("let x = 42;") phải ra 6 tokens:
    // Keyword("let"), Ident("x"), Symbol("="), Number(42), Symbol(";"), Eof

    // ... (setup)

    assert_eq!(token_count, 6, "let x = 42; → 6 tokens");
}
```

---

## Rào cản có thể gặp

### 1. ModuleLoader chưa wire end-to-end
**Triệu chứng:** `loader.load()` fail vì chưa implement
**Giải pháp:** Dùng cách thủ công (Task 1 alternative). Đọc file → parse → compile bằng tay.

### 2. Olang syntax chưa được Rust parser hỗ trợ hết
**Triệu chứng:** `Parser::parse_program()` fail trên `union`, `type`, hoặc cú pháp Olang
**Giải pháp:** Kiểm tra `syntax.rs` — xem `Stmt` enum có `UnionDef`, `TypeDef` không. Nếu thiếu, thêm vào.

### 3. VM chưa có built-in functions (len, char_at, substr, push, to_num)
**Triệu chứng:** VM emit `LookupAlias("len")` nhưng không ai inject
**Giải pháp:** Cần viết FFI bridge — map tên hàm → Rust implementation:
```rust
// Trong vm.rs hoặc file mới: builtins.rs
fn handle_ffi(name: &str, args: &[MolecularChain]) -> MolecularChain {
    match name {
        "len" => { /* string length */ },
        "char_at" => { /* string index */ },
        "substr" => { /* substring */ },
        "push" => { /* array push */ },
        "to_num" => { /* string → number */ },
        _ => MolecularChain::empty(),
    }
}
```
**Đây là rào cản LỚN NHẤT.** lexer.ol dùng 5 built-in functions. VM cần FFI bridge.

### 4. VM không hỗ trợ arrays/strings natively
**Triệu chứng:** `let tokens = [];` fail
**Giải pháp:** MolecularChain hiện chỉ chứa molecules. Cần extend hoặc dùng VmValue enum mới.

---

## Definition of Done

- [ ] `cargo test -p olang bootstrap_lexer` pass
- [ ] lexer.ol parse thành AST không lỗi
- [ ] lexer.ol compile thành OlangProgram không lỗi
- [ ] `tokenize("let x = 42;")` trên VM trả về 6 tokens
- [ ] `tokenize("fn f(x) { return x + 1; }")` trả về đúng tokens

## Ước tính

- Nếu ModuleLoader + FFI bridge đã có: **2-4 giờ**
- Nếu cần viết FFI bridge cho built-ins: **1-2 ngày**
- Nếu cần extend VM để hỗ trợ arrays: **2-3 ngày**

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 0.1*
