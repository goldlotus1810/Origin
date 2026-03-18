# PLAN 0.4 — Viết semantic.ol (~800 LOC)

**Phụ thuộc:** PLAN_0_3 phải xong (lexer + parser chạy, self-parse OK)
**Mục tiêu:** Viết semantic analyzer bằng Olang — validate AST + lower xuống IR opcodes.
**Yêu cầu:** Biết Rust. Hiểu compiler theory cơ bản (scope, type check, IR lowering).

---

## Bối cảnh

### Vị trí trong pipeline

```
Source → lexer.ol → tokens → parser.ol → AST → semantic.ol → OlangProgram(Vec<Op>)
                                                  ^^^^^^^^^^^
                                                  PLAN NÀY
```

### Rust reference: semantic.rs đã có

File `crates/olang/src/lang/semantic.rs` (~288K) là Rust implementation. semantic.ol phải làm ĐÚNG NHỮNG GÌ semantic.rs làm, nhưng viết bằng Olang.

**KHÔNG cần port 100%.** Chỉ cần đủ để compile lexer.ol + parser.ol.

### Op enum (target output)

semantic.ol phải emit các Op này (từ `ir.rs:44-170`):

```
Stack:    Push, Load, Dup, Pop, Swap, PushNum, PushMol, Store, StoreUpdate, LoadLocal
Control:  Jmp, Jz, Loop, Call, Ret, ScopeBegin, ScopeEnd, TryBegin, CatchEnd, Halt, Nop
Chain:    Lca, Edge, Query, Emit, Fuse
```

---

## Việc cần làm

### Task 1: Scope tracking (~200 LOC)

```olang
// semantic.ol

type Scope {
    parent: Num,           // index vào scopes array (-1 = root)
    vars: Vec[VarEntry],   // biến đã khai báo
    fns: Vec[FnEntry],     // hàm đã khai báo
}

type VarEntry {
    name: Str,
    slot: Num,             // local variable slot index
}

type FnEntry {
    name: Str,
    param_count: Num,
    label: Num,            // jump target trong output ops
}

type SemanticState {
    scopes: Vec[Scope],
    current_scope: Num,
    ops: Vec[Op],          // output: IR opcodes
    errors: Vec[Str],
    next_label: Num,
}

fn push_scope(state) { ... }
fn pop_scope(state) { ... }
fn declare_var(state, name) → slot { ... }
fn resolve_var(state, name) → slot or error { ... }
fn declare_fn(state, name, param_count) { ... }
fn resolve_fn(state, name) → FnEntry or error { ... }
```

**Rust reference:** `semantic.rs` — tìm `struct Scope`, `push_scope()`, `pop_scope()`

### Task 2: Statement compilation (~300 LOC)

```olang
fn compile_stmt(state, stmt) {
    match stmt {
        Stmt::LetStmt { name, value } => {
            compile_expr(state, value);     // push value lên stack
            let slot = declare_var(state, name);
            emit(state, Op::Store(name));
        },

        Stmt::FnDef { name, params, body } => {
            let fn_label = next_label(state);
            declare_fn(state, name, len(params));
            emit(state, Op::Jmp(0));        // skip fn body (patch later)
            let body_start = len(state.ops);
            emit(state, Op::ScopeBegin);
            // Declare params as locals
            for param in params {
                declare_var(state, param);
                emit(state, Op::Store(param));
            };
            for s in body {
                compile_stmt(state, s);
            };
            emit(state, Op::ScopeEnd);
            emit(state, Op::Ret);
            // Patch Jmp to skip body
            patch_jmp(state, body_start - 1, len(state.ops));
        },

        Stmt::ReturnStmt { value } => {
            compile_expr(state, value);
            emit(state, Op::Ret);
        },

        Stmt::EmitStmt { expr } => {
            compile_expr(state, expr);
            emit(state, Op::Emit);
        },

        Stmt::IfStmt { cond, then_block, else_block } => {
            compile_expr(state, cond);
            let jz_pos = len(state.ops);
            emit(state, Op::Jz(0));         // patch later
            for s in then_block { compile_stmt(state, s); };
            if len(else_block) > 0 {
                let jmp_pos = len(state.ops);
                emit(state, Op::Jmp(0));    // skip else
                patch_jmp(state, jz_pos, len(state.ops));
                for s in else_block { compile_stmt(state, s); };
                patch_jmp(state, jmp_pos, len(state.ops));
            } else {
                patch_jmp(state, jz_pos, len(state.ops));
            };
        },

        Stmt::WhileStmt { cond, body } => {
            let loop_start = len(state.ops);
            compile_expr(state, cond);
            let jz_pos = len(state.ops);
            emit(state, Op::Jz(0));
            for s in body { compile_stmt(state, s); };
            emit(state, Op::Jmp(loop_start));
            patch_jmp(state, jz_pos, len(state.ops));
        },

        // TypeDef, UnionDef → đăng ký type metadata, không emit ops
        Stmt::TypeDef { name, fields } => {
            register_type(state, name, fields);
        },
        Stmt::UnionDef { name, variants } => {
            register_union(state, name, variants);
        },
    };
}
```

### Task 3: Expression compilation (~200 LOC)

```olang
fn compile_expr(state, expr) {
    match expr {
        Expr::NumLit { value } => {
            emit(state, Op::PushNum(value));
        },
        Expr::StrLit { value } => {
            emit(state, Op::Push(encode_string(value)));
        },
        Expr::Ident { name } => {
            // Tìm biến local → LoadLocal, không thì Load (global alias)
            if is_local(state, name) {
                emit(state, Op::LoadLocal(name));
            } else {
                emit(state, Op::Load(name));
            };
        },
        Expr::BinOp { op, lhs, rhs } => {
            compile_expr(state, lhs);
            compile_expr(state, rhs);
            emit(state, op_for_binop(op));   // +, -, *, /, ==, !=, <, >, &&, ||
        },
        Expr::Call { callee, args } => {
            // Push args
            for arg in args {
                compile_expr(state, arg);
            };
            // Resolve function
            let fn_entry = resolve_fn(state, callee.name);
            emit(state, Op::Call(callee.name));
        },
        Expr::FieldAccess { object, field } => {
            compile_expr(state, object);
            emit(state, Op::Ffi("field_get", 2));  // runtime FFI
        },
        Expr::MolLiteral { s, r, v, a, t } => {
            emit(state, Op::PushMol(s, r, v, a, t));
        },
    };
}
```

### Task 4: Validation (~100 LOC)

```olang
fn validate(state) {
    // 1. Undeclared variables → error
    // 2. Function call arity mismatch → error
    // 3. Return outside function → error
    // 4. Break/continue outside loop → error
}
```

### Task 5: Entry point

```olang
pub fn analyze(ast) {
    let state = new_semantic_state();
    for stmt in ast {
        compile_stmt(state, stmt);
    };
    validate(state);
    if len(state.errors) > 0 {
        return { ok: false, errors: state.errors };
    };
    return { ok: true, program: state.ops };
}
```

---

## Test (viết bằng Rust)

```rust
// crates/olang/tests/bootstrap_semantic.rs

#[test]
fn test_semantic_ol_let() {
    // "let x = 42;" → [PushNum(42), Store("x")]
    let ast = parse(tokenize("let x = 42;"));
    let program = run_analyze(ast);
    assert_eq!(program.ops, vec![Op::PushNum(42.0), Op::Store("x".into())]);
}

#[test]
fn test_semantic_ol_fn_def() {
    // "fn f(x) { return x; }" → [Jmp(?), ScopeBegin, Store("x"), LoadLocal("x"), Ret, ScopeEnd]
    let ast = parse(tokenize("fn f(x) { return x; }"));
    let program = run_analyze(ast);
    assert!(program.ops.contains(&Op::ScopeBegin));
    assert!(program.ops.contains(&Op::Ret));
}

#[test]
fn test_semantic_ol_undeclared_var() {
    // "emit y;" where y is not declared → error
    let ast = parse(tokenize("emit y;"));
    let result = run_analyze(ast);
    assert!(!result.ok);
    assert!(result.errors[0].contains("undeclared"));
}

#[test]
fn test_semantic_ol_compile_lexer() {
    // THE BIG TEST: semantic.ol compile lexer.ol thành OlangProgram
    let lexer_source = std::fs::read_to_string("../../stdlib/bootstrap/lexer.ol").unwrap();
    let ast = parse(tokenize(&lexer_source));
    let program = run_analyze(ast);
    assert!(program.ok, "semantic.ol must compile lexer.ol: {:?}", program.errors);
    assert!(program.ops.len() > 50, "lexer.ol should produce many ops");
}
```

---

## Cách viết

1. Tạo file `stdlib/bootstrap/semantic.ol`
2. Viết từng phần (scope → stmt → expr → validate)
3. Test mỗi phần trên Rust VM trước khi viết phần tiếp
4. **Tham khảo** `semantic.rs` cho logic, nhưng **viết bằng Olang syntax**

## Rào cản

| Rào cản | Giải pháp |
|---------|-----------|
| Olang chưa hỗ trợ `match` trên union | Dùng if/else chain thay thế |
| `Vec[Op]` cần dynamic append | Dùng built-in `push()` từ FFI |
| `patch_jmp` cần random access array | Cần `set_at(array, index, value)` built-in |

## Definition of Done

- [ ] `stdlib/bootstrap/semantic.ol` tồn tại (~800 LOC)
- [ ] semantic.ol chạy trên Rust VM
- [ ] `analyze(parse(tokenize("let x = 42;")))` → đúng ops
- [ ] `analyze(parse(tokenize(lexer_source)))` → OlangProgram OK
- [ ] Undeclared variable → error

## Ước tính: 3-5 ngày

---

*Tham chiếu: PLAN_REWRITE.md § Giai đoạn 0.4*
