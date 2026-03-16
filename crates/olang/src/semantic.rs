//! # semantic — Ngữ nghĩa của Olang
//!
//! Định nghĩa ý nghĩa của mỗi thao tác, kiểm tra tính hợp lệ,
//! và hạ AST xuống OlangProgram (IR).
//!
//! ## Quy tắc ngữ nghĩa
//!
//! | Construct            | Ngữ nghĩa                                         | Kiểu         |
//! |----------------------|----------------------------------------------------|--------------|
//! | `ident`              | Tra alias trong registry → MolecularChain          | Chain        |
//! | `a ∘ b`              | LCA(a, b) → chain cha chung                       | Chain        |
//! | `a ∈ b`              | Tạo Silk edge (a Member b)                         | Chain (= b)  |
//! | `a REL ?`            | Query: tìm nodes có relation REL với a             | Chain        |
//! | `let x = expr`       | Gán chain vào biến cục bộ `x`                      | ()           |
//! | `emit expr`          | Xuất chain ra caller                               | ()           |
//! | `if cond { ... }`    | Thực thi block nếu cond chain non-empty            | ()           |
//! | `loop N { ... }`     | Lặp block N lần (N ≤ 65536)                       | ()           |
//! | `fn f(a,b) { ... }`  | Định nghĩa hàm (inline khi gọi)                   | ()           |
//! | `f(x, y)`            | Gọi hàm, bind args vào params                     | Chain        |
//! | `dream`              | Trigger Dream cycle (STM → cluster → QR)           | ()           |
//! | `stats`              | Xuất system statistics                             | ()           |
//!
//! ## Scope Rules
//!
//! - Variables defined by `let` are local to current block.
//! - Function params are local to function body.
//! - Undefined identifiers → assumed registry alias (LOAD opcode).
//! - Functions must be defined before use.
//!
//! ## Type System
//!
//! Olang có 1 kiểu duy nhất: **MolecularChain**.
//! - Mọi expression evaluate → Chain.
//! - Empty chain = falsy (cho `if`).
//! - Non-empty chain = truthy.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::alphabet::RelOp;
use crate::ir::{OlangProgram, Op};
use crate::syntax::{Expr, Stmt};

// ─────────────────────────────────────────────────────────────────────────────
// SemError — lỗi ngữ nghĩa
// ─────────────────────────────────────────────────────────────────────────────

/// Lỗi ngữ nghĩa.
#[derive(Debug, Clone, PartialEq)]
pub struct SemError {
    /// Mô tả lỗi
    pub message: String,
}

impl SemError {
    fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scope — quản lý biến và hàm
// ─────────────────────────────────────────────────────────────────────────────

/// Function definition cho scope tracking.
#[derive(Debug, Clone)]
struct FnInfo {
    name: String,
    param_count: usize,
}

/// Scope: theo dõi biến cục bộ và hàm.
struct Scope {
    /// Stack of local variable names (pushed on enter, popped on exit)
    locals: Vec<String>,
    /// Defined functions
    fns: Vec<FnInfo>,
    /// Stack frames: mỗi frame lưu số locals tại thời điểm enter
    frames: Vec<usize>,
}

impl Scope {
    fn new() -> Self {
        Self {
            locals: Vec::new(),
            fns: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn enter(&mut self) {
        self.frames.push(self.locals.len());
    }

    fn exit(&mut self) {
        if let Some(frame_start) = self.frames.pop() {
            self.locals.truncate(frame_start);
        }
    }

    fn define_local(&mut self, name: &str) {
        self.locals.push(name.to_string());
    }

    fn _is_local(&self, name: &str) -> bool {
        self.locals.iter().any(|n| n == name)
    }

    fn define_fn(&mut self, name: &str, param_count: usize) {
        self.fns.push(FnInfo {
            name: name.to_string(),
            param_count,
        });
    }

    fn lookup_fn(&self, name: &str) -> Option<&FnInfo> {
        self.fns.iter().rev().find(|f| f.name == name)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Validator — kiểm tra tính hợp lệ
// ─────────────────────────────────────────────────────────────────────────────

/// Validate chương trình AST, trả về danh sách lỗi.
pub fn validate(stmts: &[Stmt]) -> Vec<SemError> {
    let mut errors = Vec::new();
    let mut scope = Scope::new();
    scope.enter();

    // First pass: collect function definitions
    for stmt in stmts {
        if let Stmt::FnDef { name, params, .. } = stmt {
            scope.define_fn(name, params.len());
        }
    }

    // Second pass: validate
    for stmt in stmts {
        validate_stmt(stmt, &mut scope, &mut errors);
    }

    scope.exit();
    errors
}

fn validate_stmt(stmt: &Stmt, scope: &mut Scope, errors: &mut Vec<SemError>) {
    match stmt {
        Stmt::Let { name, value } => {
            validate_expr(value, scope, errors);
            scope.define_local(name);
        }

        Stmt::Emit(expr) => {
            validate_expr(expr, scope, errors);
        }

        Stmt::If {
            cond,
            then_block,
            else_block,
        } => {
            validate_expr(cond, scope, errors);
            scope.enter();
            for s in then_block {
                validate_stmt(s, scope, errors);
            }
            scope.exit();
            if let Some(else_stmts) = else_block {
                scope.enter();
                for s in else_stmts {
                    validate_stmt(s, scope, errors);
                }
                scope.exit();
            }
        }

        Stmt::Loop { count, body } => {
            if *count > 65536 {
                errors.push(SemError::new(&alloc::format!(
                    "Loop count {} exceeds max 65536",
                    count
                )));
            }
            scope.enter();
            for s in body {
                validate_stmt(s, scope, errors);
            }
            scope.exit();
        }

        Stmt::FnDef { name, params, body } => {
            // Check no duplicate params
            for (i, p) in params.iter().enumerate() {
                if params[..i].contains(p) {
                    errors.push(SemError::new(&alloc::format!(
                        "Duplicate parameter '{p}' in function '{name}'"
                    )));
                }
            }
            scope.enter();
            for p in params {
                scope.define_local(p);
            }
            for s in body {
                validate_stmt(s, scope, errors);
            }
            scope.exit();
        }

        Stmt::Expr(expr) => {
            validate_expr(expr, scope, errors);
        }

        Stmt::Command(_) => {
            // Commands are always valid
        }

        Stmt::CommandArg { .. } => {
            // Commands with args are always valid
        }
    }
}

fn validate_expr(expr: &Expr, scope: &mut Scope, errors: &mut Vec<SemError>) {
    match expr {
        Expr::Ident(_) => {
            // Identifiers: could be local var or registry alias.
            // We don't error on undefined — registry lookup happens at runtime.
        }

        Expr::Int(_) => {}

        Expr::Compose(a, b) => {
            validate_expr(a, scope, errors);
            validate_expr(b, scope, errors);
        }

        Expr::RelEdge { lhs, op, rhs } => {
            validate_expr(lhs, scope, errors);
            validate_expr(rhs, scope, errors);
            // Check: extended ops (Context, Contains, Intersects) → no direct byte mapping
            if op.to_rel_byte().is_none() {
                // Not an error — handled at lowering level.
                // Context queries use a different compilation strategy.
            }
        }

        Expr::RelQuery { subject, .. } => {
            validate_expr(subject, scope, errors);
        }

        Expr::Call { name, args } => {
            if let Some(fn_info) = scope.lookup_fn(name) {
                if args.len() != fn_info.param_count {
                    errors.push(SemError::new(&alloc::format!(
                        "Function '{}' expects {} args, got {}",
                        name, fn_info.param_count, args.len()
                    )));
                }
            }
            // Note: undefined function is not an error — could be a registry alias + call
            for arg in args {
                validate_expr(arg, scope, errors);
            }
        }

        Expr::Chain { head, steps } => {
            validate_expr(head, scope, errors);
            for (_, step_expr) in steps {
                validate_expr(step_expr, scope, errors);
            }
        }

        Expr::Arith { lhs, rhs, .. } => {
            validate_expr(lhs, scope, errors);
            validate_expr(rhs, scope, errors);
        }

        Expr::Str(_) => {
            // String literals are always valid
        }

        Expr::Group(inner) => {
            validate_expr(inner, scope, errors);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Lowering — AST → OlangProgram
// ─────────────────────────────────────────────────────────────────────────────

/// Lower (hạ) AST → OlangProgram (IR opcodes).
///
/// Semantic rules applied during lowering:
/// - `ident` → LoadLocal (nếu biến cục bộ) hoặc Load (registry alias)
/// - `a ∘ b` → lower(a), lower(b), LCA
/// - `a REL b` → lower(a), lower(b), EDGE(rel)
/// - `a REL ?` → lower(a), QUERY(rel)
/// - `let x = e` → lower(e), Store(x)
/// - `emit e` → lower(e), EMIT
/// - `if c { t } else { e }` → lower(c), JZ(else), lower(t), JMP(end), lower(e)
/// - `loop N { b }` → unroll N times: lower(b) × N
/// - `dream` → DREAM
/// - `stats` → STATS
pub fn lower(stmts: &[Stmt]) -> OlangProgram {
    let mut ctx = LowerCtx::new();

    // First pass: collect function definitions
    for stmt in stmts {
        if let Stmt::FnDef { name, params, body } = stmt {
            ctx.fns.push(FnDef {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
            });
        }
    }

    // Second pass: lower statements
    for stmt in stmts {
        lower_stmt(stmt, &mut ctx);
    }

    ctx.prog.push_op(Op::Halt);
    ctx.prog
}

#[derive(Clone)]
struct FnDef {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

struct LowerCtx {
    prog: OlangProgram,
    /// Local variable scope stack
    locals: Vec<String>,
    /// Function definitions
    fns: Vec<FnDef>,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            prog: OlangProgram::new("olang"),
            locals: Vec::new(),
            fns: Vec::new(),
        }
    }

    fn is_local(&self, name: &str) -> bool {
        self.locals.iter().any(|n| n == name)
    }

    fn lookup_fn(&self, name: &str) -> Option<FnDef> {
        self.fns.iter().rev().find(|f| f.name == name).cloned()
    }

    fn emit(&mut self, op: Op) {
        self.prog.push_op(op);
    }

    fn current_pos(&self) -> usize {
        self.prog.ops.len()
    }

    /// Patch a JMP/JZ target retroactively.
    fn patch_jump(&mut self, pos: usize, target: usize) {
        match &mut self.prog.ops[pos] {
            Op::Jmp(ref mut t) => *t = target,
            Op::Jz(ref mut t) => *t = target,
            _ => {}
        }
    }
}

fn lower_stmt(stmt: &Stmt, ctx: &mut LowerCtx) {
    match stmt {
        Stmt::Let { name, value } => {
            lower_expr(value, ctx);
            ctx.emit(Op::Store(name.clone()));
            ctx.locals.push(name.clone());
        }

        Stmt::Emit(expr) => {
            lower_expr(expr, ctx);
            ctx.emit(Op::Emit);
        }

        Stmt::If {
            cond,
            then_block,
            else_block,
        } => {
            // lower cond
            lower_expr(cond, ctx);
            // JZ → else (or end)
            let jz_pos = ctx.current_pos();
            ctx.emit(Op::Jz(0)); // placeholder
            ctx.emit(Op::Pop); // pop cond from stack

            // then block
            let saved = ctx.locals.len();
            for s in then_block {
                lower_stmt(s, ctx);
            }
            ctx.locals.truncate(saved);

            if let Some(else_stmts) = else_block {
                // JMP → end
                let jmp_pos = ctx.current_pos();
                ctx.emit(Op::Jmp(0)); // placeholder

                // else target
                let else_target = ctx.current_pos();
                ctx.patch_jump(jz_pos, else_target);
                ctx.emit(Op::Pop); // pop cond

                let saved2 = ctx.locals.len();
                for s in else_stmts {
                    lower_stmt(s, ctx);
                }
                ctx.locals.truncate(saved2);

                // end target
                let end_target = ctx.current_pos();
                ctx.patch_jump(jmp_pos, end_target);
            } else {
                // no else: JZ jumps to end
                let end_target = ctx.current_pos();
                ctx.patch_jump(jz_pos, end_target);
                ctx.emit(Op::Pop); // pop cond
            }
        }

        Stmt::Loop { count, body } => {
            // Unroll: repeat body N times
            for _ in 0..*count {
                let saved = ctx.locals.len();
                for s in body {
                    lower_stmt(s, ctx);
                }
                ctx.locals.truncate(saved);
            }
        }

        Stmt::FnDef { .. } => {
            // Function definitions are collected in first pass, not emitted inline
        }

        Stmt::Expr(expr) => {
            lower_expr(expr, ctx);
            // Discard result (expression statement)
            ctx.emit(Op::Pop);
        }

        Stmt::Command(cmd) => match cmd.as_str() {
            "dream" => ctx.emit(Op::Dream),
            "stats" => ctx.emit(Op::Stats),
            _ => ctx.emit(Op::Nop),
        },

        Stmt::CommandArg { name, arg } => {
            // Commands with arguments: push the arg as a Load, then dispatch
            ctx.emit(Op::Load(arg.clone()));
            match name.as_str() {
                "learn" => ctx.emit(Op::Call("learn".into())),
                "seed" => ctx.emit(Op::Call("seed".into())),
                _ => ctx.emit(Op::Call(name.clone())),
            }
        }
    }
}

fn lower_expr(expr: &Expr, ctx: &mut LowerCtx) {
    match expr {
        Expr::Ident(name) => {
            if name == "?" {
                // Wildcard — push empty chain
                ctx.emit(Op::Push(crate::molecular::MolecularChain::empty()));
            } else if ctx.is_local(name) {
                ctx.emit(Op::LoadLocal(name.clone()));
            } else {
                ctx.emit(Op::Load(name.clone()));
            }
        }

        Expr::Int(_) => {
            // Push empty chain as placeholder (ints used mainly for loop count)
            ctx.emit(Op::Push(crate::molecular::MolecularChain::empty()));
        }

        Expr::Compose(a, b) => {
            lower_expr(a, ctx);
            lower_expr(b, ctx);
            ctx.emit(Op::Lca);
        }

        Expr::RelEdge { lhs, op, rhs } => {
            lower_expr(lhs, ctx);
            lower_expr(rhs, ctx);
            if let Some(rel_byte) = op.to_rel_byte() {
                ctx.emit(Op::Edge(rel_byte));
            } else {
                // Extended ops: Context, Contains, Intersects
                // Map to semantic equivalents:
                // Context(∂) → load both, LCA (context = shared ancestor)
                // Contains(∪) → Edge with Member (0x01) reversed
                // Intersects(∩) → Edge with Equiv (0x03)
                match op {
                    RelOp::Context => ctx.emit(Op::Lca),
                    RelOp::Contains => ctx.emit(Op::Edge(0x01)),
                    RelOp::Intersects => ctx.emit(Op::Edge(0x03)),
                    _ => ctx.emit(Op::Nop),
                }
            }
        }

        Expr::RelQuery { subject, op } => {
            lower_expr(subject, ctx);
            if let Some(rel_byte) = op.to_rel_byte() {
                ctx.emit(Op::Query(rel_byte));
            } else {
                // Extended query: use closest core relation
                match op {
                    RelOp::Context => ctx.emit(Op::Query(0x07)), // Similar
                    RelOp::Contains => ctx.emit(Op::Query(0x01)), // Member
                    RelOp::Intersects => ctx.emit(Op::Query(0x03)), // Equiv
                    _ => ctx.emit(Op::Nop),
                }
            }
        }

        Expr::Call { name, args } => {
            // Check if it's a user-defined function
            if let Some(fn_def) = ctx.lookup_fn(name) {
                // Inline the function: bind args to params
                let saved = ctx.locals.len();

                // Evaluate args and store as locals
                for (param, arg) in fn_def.params.iter().zip(args.iter()) {
                    lower_expr(arg, ctx);
                    ctx.emit(Op::Store(param.clone()));
                    ctx.locals.push(param.clone());
                }

                // Lower function body
                for s in &fn_def.body {
                    lower_stmt(s, ctx);
                }

                ctx.locals.truncate(saved);
            } else {
                // Unknown function → CALL (let runtime handle it)
                for arg in args {
                    lower_expr(arg, ctx);
                }
                ctx.emit(Op::Call(name.clone()));
            }
        }

        Expr::Chain { head, steps } => {
            // Multi-hop chain query: A → ? → B
            // Lower each step as sequential QUERY or EDGE operations
            lower_expr(head, ctx);
            for (op, step_expr) in steps {
                let is_wildcard = matches!(step_expr, Expr::Ident(s) if s == "?");
                if is_wildcard {
                    // Wildcard step: query for nodes with this relation
                    if let Some(rel_byte) = op.to_rel_byte() {
                        ctx.emit(Op::Query(rel_byte));
                    } else {
                        ctx.emit(Op::Query(0x07)); // fallback: Similar
                    }
                } else {
                    // Concrete step: create edge
                    lower_expr(step_expr, ctx);
                    if let Some(rel_byte) = op.to_rel_byte() {
                        ctx.emit(Op::Edge(rel_byte));
                    } else {
                        ctx.emit(Op::Lca); // fallback for extended ops
                    }
                }
            }
        }

        Expr::Arith { lhs, op, rhs } => {
            // Arithmetic: lower both sides, then emit CALL to builtin
            lower_expr(lhs, ctx);
            lower_expr(rhs, ctx);
            let fn_name = match op {
                crate::alphabet::ArithOp::Add => "__add",
                crate::alphabet::ArithOp::Sub => "__sub",
                crate::alphabet::ArithOp::Mul => "__mul",
                crate::alphabet::ArithOp::Div => "__div",
            };
            ctx.emit(Op::Call(fn_name.into()));
        }

        Expr::Str(s) => {
            // String literal → Load as registry alias
            ctx.emit(Op::Load(s.clone()));
        }

        Expr::Group(inner) => {
            lower_expr(inner, ctx);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parse;

    // ── Validation ──────────────────────────────────────────────────────────

    #[test]
    fn validate_simple_program() {
        let stmts = parse("fire ∘ water").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "Simple compose should be valid");
    }

    #[test]
    fn validate_let_and_use() {
        let stmts = parse("let x = fire; emit x;").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_fn_arity_mismatch() {
        let stmts = parse("fn blend(a, b) { emit a ∘ b; } emit blend(fire);").unwrap();
        let errors = validate(&stmts);
        assert!(
            errors.iter().any(|e| e.message.contains("expects 2")),
            "Should detect arity mismatch: {:?}",
            errors
        );
    }

    #[test]
    fn validate_duplicate_params() {
        let stmts = parse("fn bad(x, x) { emit x; }").unwrap();
        let errors = validate(&stmts);
        assert!(
            errors.iter().any(|e| e.message.contains("Duplicate")),
            "Should detect duplicate params: {:?}",
            errors
        );
    }

    #[test]
    fn validate_loop_too_large() {
        let stmts = parse("loop 99999 { emit fire; }").unwrap();
        let errors = validate(&stmts);
        assert!(
            errors.iter().any(|e| e.message.contains("exceeds")),
            "Should detect loop too large: {:?}",
            errors
        );
    }

    #[test]
    fn validate_nested_scope() {
        let stmts = parse("if fire { let x = water; emit x; }").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "Nested scope should be valid");
    }

    // ── Lowering ────────────────────────────────────────────────────────────

    #[test]
    fn lower_simple_ident() {
        let stmts = parse("fire").unwrap();
        let prog = lower(&stmts);
        // Single expr → LOAD "fire", POP (discard), HALT
        assert!(
            prog.ops.contains(&Op::Load("fire".into())),
            "Should emit LOAD for registry alias"
        );
    }

    #[test]
    fn lower_compose() {
        let stmts = parse("fire ∘ water").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Load("fire".into())));
        assert!(prog.ops.contains(&Op::Load("water".into())));
        assert!(prog.ops.contains(&Op::Lca));
    }

    #[test]
    fn lower_relation_edge() {
        let stmts = parse("fire → water").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Edge(0x06))); // Causes = 0x06
    }

    #[test]
    fn lower_relation_query() {
        let stmts = parse("fire ∈ ?").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Query(0x01))); // Member = 0x01
    }

    #[test]
    fn lower_let_binding() {
        let stmts = parse("let steam = fire ∘ water; emit steam;").unwrap();
        let prog = lower(&stmts);
        assert!(
            prog.ops.contains(&Op::Store("steam".into())),
            "Should emit STORE for let binding"
        );
        assert!(
            prog.ops.contains(&Op::LoadLocal("steam".into())),
            "Should emit LOAD_LOCAL for local variable"
        );
        assert!(prog.ops.contains(&Op::Emit), "Should emit EMIT");
    }

    #[test]
    fn lower_emit() {
        let stmts = parse("emit fire;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Load("fire".into())));
        assert!(prog.ops.contains(&Op::Emit));
    }

    #[test]
    fn lower_if_basic() {
        let stmts = parse("if fire { emit water; }").unwrap();
        let prog = lower(&stmts);
        // Should have: LOAD fire, JZ(target), POP, LOAD water, EMIT, ..., POP, HALT
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jz(_))));
    }

    #[test]
    fn lower_if_else() {
        let stmts = parse("if fire { emit water; } else { emit earth; }").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jz(_))));
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jmp(_))));
    }

    #[test]
    fn lower_loop_unroll() {
        let stmts = parse("loop 3 { emit fire; }").unwrap();
        let prog = lower(&stmts);
        // Should have 3 × (LOAD fire, EMIT)
        let emit_count = prog.ops.iter().filter(|op| **op == Op::Emit).count();
        assert_eq!(emit_count, 3, "Loop 3 should produce 3 EMITs");
    }

    #[test]
    fn lower_command_dream() {
        let stmts = parse("dream").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Dream));
    }

    #[test]
    fn lower_command_stats() {
        let stmts = parse("emit fire; stats;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Stats));
    }

    #[test]
    fn lower_fn_inline() {
        let src = "fn blend(a, b) { emit a ∘ b; } blend(fire, water);";
        let stmts = parse(src).unwrap();
        let prog = lower(&stmts);
        // Function is inlined: LOAD fire, STORE a, LOAD water, STORE b, ...
        assert!(
            prog.ops.contains(&Op::Store("a".into())),
            "Fn params should be stored"
        );
        assert!(prog.ops.contains(&Op::Lca), "Fn body should have LCA");
    }

    #[test]
    fn lower_context_query_uses_lca() {
        let stmts = parse("bank ∂ finance").unwrap();
        let prog = lower(&stmts);
        // Context(∂) → LCA (shared ancestor = context)
        assert!(prog.ops.contains(&Op::Lca));
    }

    #[test]
    fn lower_ends_with_halt() {
        let stmts = parse("fire").unwrap();
        let prog = lower(&stmts);
        assert_eq!(
            *prog.ops.last().unwrap(),
            Op::Halt,
            "Program must end with HALT"
        );
    }

    // ── New constructs ──────────────────────────────────────────────────────

    #[test]
    fn lower_chain_query() {
        let stmts = parse("🌞 → ? → 🌵").unwrap();
        let prog = lower(&stmts);
        // Should have: LOAD 🌞, QUERY(Causes), LOAD 🌵, EDGE(Causes), POP, HALT
        assert!(prog.ops.contains(&Op::Load("🌞".into())));
        assert!(prog.ops.contains(&Op::Query(0x06))); // Causes query for wildcard
        assert!(prog.ops.contains(&Op::Load("🌵".into())));
    }

    #[test]
    fn lower_arithmetic() {
        let stmts = parse("1 + 2").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Call("__add".into())));
    }

    #[test]
    fn lower_string_literal() {
        let stmts = parse("emit \"hello\";").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Load("hello".into())));
        assert!(prog.ops.contains(&Op::Emit));
    }

    #[test]
    fn lower_command_arg() {
        let stmts = parse("learn \"tôi buồn\";").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Load("tôi buồn".into())));
        assert!(prog.ops.contains(&Op::Call("learn".into())));
    }

    #[test]
    fn lower_symbol_define() {
        let stmts = parse("steam ≔ fire ∘ water;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Store("steam".into())));
        assert!(prog.ops.contains(&Op::Lca));
    }

    #[test]
    fn lower_symbol_implies() {
        let stmts = parse("fire ⇒ { ○ water; }").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jz(_))));
        assert!(prog.ops.contains(&Op::Emit));
    }

    #[test]
    fn lower_cycle_loop() {
        let stmts = parse("↻ 2 { ○ fire; }").unwrap();
        let prog = lower(&stmts);
        let emit_count = prog.ops.iter().filter(|op| **op == Op::Emit).count();
        assert_eq!(emit_count, 2, "↻ 2 should produce 2 EMITs");
    }

    #[test]
    fn validate_chain_query() {
        let stmts = parse("🌞 → ? → 🌵").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_arithmetic() {
        let stmts = parse("1 + 2").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty());
    }
}
