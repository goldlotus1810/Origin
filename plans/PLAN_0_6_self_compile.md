# PLAN 0.6 — Self-compile test

**Phụ thuộc:** PLAN_0_1 → 0_5 tất cả phải xong
**Mục tiêu:** Chứng minh Olang compiler tự compile chính nó. Cắt dây rốn bước 1.
**Yêu cầu:** Biết Rust. Hiểu concept bootstrapping compiler.

---

## Bài test quyết định

```
Compiler A (Rust):
  lexer.ol → Rust parser → Rust semantic → bytecode A

Compiler B (Olang):
  lexer.ol → lexer.ol(tokenize) → parser.ol(parse) → semantic.ol(analyze) → codegen.ol(generate) → bytecode B

Assert: bytecode A == bytecode B
```

Nếu test pass → Olang compiler viết bằng Olang produce output GIỐNG HỆT Rust compiler.
→ Rust compiler KHÔNG CẦN NỮA cho Olang code.

---

## Việc cần làm

### Task 1: Rust reference compiler

Tạo Rust function compile lexer.ol → bytecode bằng pipeline Rust hiện tại:

```rust
// crates/olang/tests/self_compile.rs

fn rust_compile(source: &str) -> Vec<u8> {
    // 1. Rust parser
    let ast = olang::syntax::Parser::new(source).parse_program().unwrap();
    // 2. Rust semantic
    let program = olang::semantic::compile_program(&ast).unwrap();
    // 3. Rust bytecode encoder (bytecode.rs từ PLAN_0_5)
    olang::bytecode::encode(&program.ops)
}
```

### Task 2: Olang bootstrap compiler

Tạo wrapper chạy toàn bộ Olang pipeline trên VM:

```rust
fn olang_compile(source: &str) -> Vec<u8> {
    // Load bootstrap modules
    let mut loader = ModuleLoader::new();
    loader.add_search_path("../../stdlib");

    // 1. tokenize (lexer.ol)
    let tokens = vm_call(&loader, "olang.bootstrap.lexer", "tokenize", &[source]);

    // 2. parse (parser.ol)
    let ast = vm_call(&loader, "olang.bootstrap.parser", "parse", &[tokens]);

    // 3. analyze (semantic.ol)
    let program = vm_call(&loader, "olang.bootstrap.semantic", "analyze", &[ast]);

    // 4. generate (codegen.ol)
    let bytecode = vm_call(&loader, "olang.bootstrap.codegen", "generate", &[program]);

    bytecode
}
```

### Task 3: Self-compile assertion

```rust
#[test]
fn test_self_compile_lexer() {
    let source = std::fs::read_to_string("../../stdlib/bootstrap/lexer.ol").unwrap();

    let bytecode_a = rust_compile(&source);
    let bytecode_b = olang_compile(&source);

    assert_eq!(bytecode_a, bytecode_b,
        "Olang compiler must produce identical bytecode as Rust compiler");
}

#[test]
fn test_self_compile_parser() {
    let source = std::fs::read_to_string("../../stdlib/bootstrap/parser.ol").unwrap();

    let bytecode_a = rust_compile(&source);
    let bytecode_b = olang_compile(&source);

    assert_eq!(bytecode_a, bytecode_b);
}

#[test]
fn test_self_compile_semantic() {
    // semantic.ol compile chính nó
    let source = std::fs::read_to_string("../../stdlib/bootstrap/semantic.ol").unwrap();

    let bytecode_a = rust_compile(&source);
    let bytecode_b = olang_compile(&source);

    assert_eq!(bytecode_a, bytecode_b,
        "semantic.ol must be able to compile itself identically");
}
```

### Task 4: Fixed-point test

```rust
#[test]
fn test_self_compile_fixed_point() {
    // Olang compiler compile semantic.ol lần 1 → bytecode_v1
    // Dùng bytecode_v1 để compile semantic.ol lần 2 → bytecode_v2
    // Assert: bytecode_v1 == bytecode_v2 (fixed point)

    let source = std::fs::read_to_string("../../stdlib/bootstrap/semantic.ol").unwrap();

    let v1 = olang_compile(&source);

    // Load v1 bytecode vào VM, dùng nó compile lại
    let v2 = olang_compile_with_bytecode(&v1, &source);

    assert_eq!(v1, v2, "Bootstrap compiler must reach fixed point");
}
```

---

## Tại sao bytecode có thể KHÁC

| Nguyên nhân | Giải pháp |
|-------------|-----------|
| Rust compiler optimize khác Olang | Cả hai phải dùng CÙNG lowering rules |
| String encoding khác | Chuẩn hóa: UTF-8, little-endian |
| Jump targets đánh số khác | Dùng absolute offsets, không dùng labels |
| Variable slot allocation khác | Dùng cùng scope-walk algorithm |

**Mẹo:** Nếu bytecode khác, diff 2 bản → tìm opcode đầu tiên khác → debug từ đó.

---

## Khi nào coi là PASS

Chỉ cần **1 trong 2** điều kiện:

**A. Byte-identical:** `bytecode_a == bytecode_b` → Hoàn hảo.

**B. Semantically equivalent:** Bytecode khác nhưng chạy ra CÙNG KẾT QUẢ:
```rust
let result_a = vm.execute(&decode(bytecode_a));
let result_b = vm.execute(&decode(bytecode_b));
assert_eq!(result_a, result_b);
```
→ Chấp nhận được, nhưng cần ghi nhận diff và chuẩn hóa dần.

---

## Definition of Done

- [ ] `rust_compile(lexer.ol) == olang_compile(lexer.ol)`
- [ ] `rust_compile(parser.ol) == olang_compile(parser.ol)`
- [ ] `olang_compile(semantic.ol)` không crash (compiler compile chính nó)
- [ ] Fixed-point: v1 == v2 (compile 2 lần ra cùng bytecode)

## Ước tính: 3-5 ngày (chủ yếu debug diff)

## Deliverable

Khi test pass:
> **Olang compiler viết bằng Olang, chạy trên Rust VM, tự compile chính nó.**
> **Cắt dây rốn bước 1: logic compiler KHÔNG CẦN Rust nữa.**

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 0.6*
