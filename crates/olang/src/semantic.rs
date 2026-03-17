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
//! | `a + b`              | Hypothesis arithmetic (QT3: chưa chứng minh)       | Chain        |
//! | `a ⧺ b`              | Physical add (QT3: đã chứng minh) + FUSE           | Chain        |
//! | `a ⊖ b`              | Physical sub (QT3: đã chứng minh) + FUSE           | Chain        |
//! | `a == b`             | Truth assertion (QT3: sự thật chắc chắn)           | Chain        |
//! | `fuse`               | QT2: verify chain finite (∞-1 mới đúng)            | ()           |
//! | `dream`              | Trigger Dream cycle (STM → cluster → QR)           | ()           |
//! | `stats`              | Xuất system statistics                             | ()           |
//! | `trace`              | Toggle execution tracing                            | ()           |
//! | `inspect expr`       | Hiển thị cấu trúc chain (hash, molecules, bytes)   | Chain        |
//! | `assert expr`        | Kiểm tra chain non-empty, báo lỗi nếu empty        | Chain        |
//! | `typeof expr`        | Phân loại chain: SDF/MATH/EMOTICON/Mixed            | Chain        |
//! | `explain expr`       | Truy ngược nguồn gốc chain                         | Chain        |
//! | `why a b`            | Giải thích kết nối giữa 2 chains                   | Chain (LCA)  |
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

use crate::ir::{OlangProgram, Op};
use crate::syntax::{CmpOp, Expr, Stmt};

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

    fn is_defined(&self, name: &str) -> bool {
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

        Stmt::ForIn { var, start, end, body } => {
            if end <= start {
                errors.push(SemError::new(&alloc::format!(
                    "for-in range {}..{} is empty", start, end
                )));
            }
            scope.enter();
            scope.locals.push(var.clone());
            for s in body {
                validate_stmt(s, scope, errors);
            }
            scope.exit();
        }

        Stmt::While { cond: _, body } => {
            scope.enter();
            for s in body {
                validate_stmt(s, scope, errors);
            }
            scope.exit();
        }

        Stmt::Break | Stmt::Continue => {
            // Valid only inside loops — structural check, no scope impact
        }

        Stmt::Assign { name, value } => {
            validate_expr(value, scope, errors);
            // Warn if variable not previously defined (typo risk)
            if !scope.is_defined(name) {
                errors.push(SemError::new(&alloc::format!(
                    "Assignment to undeclared variable '{}' (use 'let' to declare first)",
                    name
                )));
            }
        }

        Stmt::Return(expr) => {
            if let Some(e) = expr {
                validate_expr(e, scope, errors);
            }
        }

        Stmt::TryCatch { try_block, catch_block } => {
            scope.enter();
            for s in try_block {
                validate_stmt(s, scope, errors);
            }
            scope.exit();
            scope.enter();
            for s in catch_block {
                validate_stmt(s, scope, errors);
            }
            scope.exit();
        }

        Stmt::Match { subject, arms } => {
            validate_expr(subject, scope, errors);
            let mut has_wildcard = false;
            for (i, arm) in arms.iter().enumerate() {
                if has_wildcard {
                    errors.push(SemError::new(&alloc::format!(
                        "Match arm {} is unreachable after wildcard '_'", i
                    )));
                }
                if arm.pattern == crate::syntax::MatchPattern::Wildcard {
                    has_wildcard = true;
                }
                scope.enter();
                for s in &arm.body {
                    validate_stmt(s, scope, errors);
                }
                scope.exit();
            }
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

        Expr::PhysOp { lhs, rhs, .. } => {
            validate_expr(lhs, scope, errors);
            validate_expr(rhs, scope, errors);
        }

        Expr::Truth { lhs, rhs } => {
            validate_expr(lhs, scope, errors);
            validate_expr(rhs, scope, errors);
        }

        Expr::Compare { lhs, rhs, .. } => {
            validate_expr(lhs, scope, errors);
            validate_expr(rhs, scope, errors);
        }

        Expr::LogicAnd(a, b) | Expr::LogicOr(a, b) => {
            validate_expr(a, scope, errors);
            validate_expr(b, scope, errors);
        }

        Expr::LogicNot(inner) => {
            validate_expr(inner, scope, errors);
        }

        Expr::Str(_) => {
            // String literals are always valid
        }

        Expr::Group(inner) => {
            validate_expr(inner, scope, errors);
        }

        Expr::MolLiteral { shape, relation, valence, arousal, time } => {
            // Validate dimension values are in valid byte range (0-255)
            for (name, val) in [("S", shape), ("R", relation), ("V", valence), ("A", arousal), ("T", time)] {
                if let Some(v) = val {
                    if *v > 255 {
                        errors.push(SemError::new(&alloc::format!(
                            "Dimension {name}={v} exceeds max 255"
                        )));
                    }
                    // Shape and Relation must be > 0 (no zero-byte)
                    if (name == "S" || name == "R" || name == "T") && *v == 0 {
                        errors.push(SemError::new(&alloc::format!(
                            "Dimension {name} must be > 0"
                        )));
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Type Inference — dự đoán kiểu chain từ expression
// ─────────────────────────────────────────────────────────────────────────────

/// Kiểu suy luận cho expression. Olang chỉ có 1 kiểu (MolecularChain),
/// nhưng type hints giúp phát hiện lỗi sớm và tối ưu codegen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChainKind {
    /// Chain chứa SDF primitives (geometric shapes)
    Sdf,
    /// Chain chứa Math/Relation ops
    Math,
    /// Chain chứa cảm xúc mạnh (extreme valence)
    Emoticon,
    /// Chain số (từ PushNum hoặc arithmetic)
    Numeric,
    /// Chưa biết (runtime mới xác định)
    Unknown,
    /// Void — statements không trả về chain (let, emit, command)
    Void,
}

/// Infer chain kind cho expression (best-effort, static analysis).
pub fn infer_expr_kind(expr: &Expr) -> ChainKind {
    match expr {
        Expr::Int(_) => ChainKind::Numeric,
        Expr::Ident(_) | Expr::Str(_) => ChainKind::Unknown,
        Expr::Compose(a, b) => {
            // LCA of two chains: if both same kind → same kind; otherwise Unknown
            let ka = infer_expr_kind(a);
            let kb = infer_expr_kind(b);
            if ka == kb { ka } else { ChainKind::Unknown }
        }
        Expr::Arith { .. } => ChainKind::Numeric,
        Expr::PhysOp { .. } => ChainKind::Numeric,
        Expr::Truth { .. } => ChainKind::Unknown,
        Expr::RelEdge { .. } | Expr::RelQuery { .. } | Expr::Chain { .. } => ChainKind::Unknown,
        Expr::Call { .. } => ChainKind::Unknown,
        Expr::Compare { .. } => ChainKind::Numeric,
        Expr::LogicAnd(..) | Expr::LogicOr(..) | Expr::LogicNot(..) => ChainKind::Unknown,
        Expr::Group(inner) => infer_expr_kind(inner),
        Expr::MolLiteral { shape, valence, .. } => {
            // Heuristic: check shape and valence to classify
            match shape {
                Some(s) if *s <= 4 => ChainKind::Sdf, // Sphere..Cone
                Some(s) if *s >= 5 => ChainKind::Math, // Torus..Subtract
                _ => match valence {
                    Some(v) if *v < 80 || *v > 176 => ChainKind::Emoticon,
                    _ => ChainKind::Unknown,
                },
            }
        }
    }
}

/// Infer kind cho statement (most statements → Void).
pub fn infer_stmt_kind(stmt: &Stmt) -> ChainKind {
    match stmt {
        Stmt::Let { .. } | Stmt::Emit(_) | Stmt::Command(_) | Stmt::CommandArg { .. } => {
            ChainKind::Void
        }
        Stmt::Expr(expr) => infer_expr_kind(expr),
        Stmt::If { .. } | Stmt::Loop { .. } | Stmt::FnDef { .. }
        | Stmt::Match { .. } | Stmt::TryCatch { .. } | Stmt::ForIn { .. }
        | Stmt::While { .. }
        | Stmt::Break | Stmt::Continue
        | Stmt::Assign { .. } => ChainKind::Void,
        Stmt::Return(Some(expr)) => infer_expr_kind(expr),
        Stmt::Return(None) => ChainKind::Void,
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
    /// Break targets: Jmp placeholders to patch past loop end
    break_jumps: Vec<Vec<usize>>,
    /// Continue targets: Jmp placeholders to patch to ScopeEnd
    continue_jumps: Vec<Vec<usize>>,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            prog: OlangProgram::new("olang"),
            locals: Vec::new(),
            fns: Vec::new(),
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
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

        Stmt::While { cond, body } => {
            // while cond { body }
            // Layout: Loop(1024) ScopeBegin [cond] Jz(end) Pop [body] ScopeEnd [end:] Pop
            // QT2: ∞-1 — capped at 1024 iterations
            ctx.emit(Op::Loop(1024));
            ctx.emit(Op::ScopeBegin);
            lower_expr(cond, ctx);
            let jz_pos = ctx.current_pos();
            ctx.emit(Op::Jz(0)); // placeholder — patched to end
            ctx.emit(Op::Pop); // pop cond result (true path continues)
            // Set up break/continue context (forward jump placeholders)
            ctx.break_jumps.push(Vec::new());
            ctx.continue_jumps.push(Vec::new());
            let saved = ctx.locals.len();
            for s in body {
                lower_stmt(s, ctx);
            }
            ctx.locals.truncate(saved);
            // Patch continue → ScopeEnd (triggers next iteration)
            let scope_end_pos = ctx.current_pos();
            if let Some(conts) = ctx.continue_jumps.pop() {
                for cp in conts {
                    ctx.patch_jump(cp, scope_end_pos);
                }
            }
            ctx.emit(Op::ScopeEnd); // triggers loop jump-back
            let end = ctx.current_pos();
            ctx.patch_jump(jz_pos, end);
            // Patch break → end (past loop)
            if let Some(breaks) = ctx.break_jumps.pop() {
                for bp in breaks {
                    ctx.patch_jump(bp, end);
                }
            }
            ctx.emit(Op::Pop); // pop cond result (false path, jumped here)
        }

        Stmt::ForIn { var, start, end, body } => {
            // for var in start..end { body }
            // Counter lives on stack; each iteration DUP into scoped var.
            // PushNum(start)          // counter on stack
            // Loop(N)
            //   ScopeBegin
            //   Dup, Store(var)       // body sees var in scope
            //   [body]
            //   PushNum(1), Call(__hyp_add) // increment counter on stack
            //   ScopeEnd              // pop scope, loop jump-back
            // Pop                     // discard counter after loop
            let count = end.saturating_sub(*start);
            ctx.emit(Op::PushNum(*start as f64));
            if count > 0 {
                ctx.emit(Op::Loop(count));
                ctx.emit(Op::ScopeBegin);
                ctx.emit(Op::Dup);
                ctx.emit(Op::Store(var.clone()));
                ctx.locals.push(var.clone());
                ctx.break_jumps.push(Vec::new());
                ctx.continue_jumps.push(Vec::new());
                let saved = ctx.locals.len();
                for s in body {
                    lower_stmt(s, ctx);
                }
                ctx.locals.truncate(saved);
                // Patch continue → increment (before ScopeEnd)
                let inc_pos = ctx.current_pos();
                if let Some(conts) = ctx.continue_jumps.pop() {
                    for cp in conts {
                        ctx.patch_jump(cp, inc_pos);
                    }
                }
                // Increment counter on stack
                ctx.emit(Op::PushNum(1.0));
                ctx.emit(Op::Call("__hyp_add".into()));
                ctx.emit(Op::ScopeEnd);
                let end_pos = ctx.current_pos();
                // Patch break → past loop
                if let Some(breaks) = ctx.break_jumps.pop() {
                    for bp in breaks {
                        ctx.patch_jump(bp, end_pos);
                    }
                }
            }
            ctx.emit(Op::Pop);
        }

        Stmt::Break => {
            // Emit Jmp(0) placeholder → patched to end of loop
            let pos = ctx.current_pos();
            ctx.emit(Op::Jmp(0));
            if let Some(breaks) = ctx.break_jumps.last_mut() {
                breaks.push(pos);
            }
        }

        Stmt::Assign { name, value } => {
            lower_expr(value, ctx);
            ctx.emit(Op::StoreUpdate(name.clone()));
        }

        Stmt::Return(expr) => {
            if let Some(e) = expr {
                lower_expr(e, ctx);
                ctx.emit(Op::Emit); // return value = emit it
            }
            ctx.emit(Op::Ret);
        }

        Stmt::Continue => {
            // Emit Jmp(0) placeholder → patched to ScopeEnd / increment
            let pos = ctx.current_pos();
            ctx.emit(Op::Jmp(0));
            if let Some(conts) = ctx.continue_jumps.last_mut() {
                conts.push(pos);
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
            "fuse" => ctx.emit(Op::Fuse),
            "trace" => ctx.emit(Op::Trace),
            _ => ctx.emit(Op::Nop),
        },

        Stmt::CommandArg { name, arg } => {
            // Commands with arguments: push the arg as a Load, then dispatch
            ctx.emit(Op::Load(arg.clone()));
            match name.as_str() {
                "learn" => ctx.emit(Op::Call("learn".into())),
                "seed" => ctx.emit(Op::Call("seed".into())),
                // Reasoning & debug commands with arguments
                "inspect" => ctx.emit(Op::Inspect),
                "assert" => ctx.emit(Op::Assert),
                "typeof" => ctx.emit(Op::TypeOf),
                "explain" => ctx.emit(Op::Explain),
                "why" => {
                    // why expects 2 chains: the arg is the second, need first from context
                    // For single-arg form: explain origin of this chain
                    ctx.emit(Op::Explain);
                }
                _ => ctx.emit(Op::Call(name.clone())),
            }
        }

        Stmt::TryCatch { try_block, catch_block } => {
            // try { body } catch { handler }
            // → TryBegin(catch_pc), [try_body], Jmp(end), [catch_body], CatchEnd
            let try_begin_pos = ctx.current_pos();
            ctx.emit(Op::TryBegin(0)); // placeholder for catch target

            // Try block
            let saved = ctx.locals.len();
            for s in try_block {
                lower_stmt(s, ctx);
            }
            ctx.locals.truncate(saved);

            // Jump past catch on success
            let jmp_pos = ctx.current_pos();
            ctx.emit(Op::Jmp(0)); // placeholder for end

            // Catch block starts here
            let catch_start = ctx.current_pos();
            ctx.patch_jump(try_begin_pos, catch_start);
            // Patch TryBegin target
            if let Op::TryBegin(ref mut t) = ctx.prog.ops[try_begin_pos] {
                *t = catch_start;
            }

            let saved2 = ctx.locals.len();
            for s in catch_block {
                lower_stmt(s, ctx);
            }
            ctx.locals.truncate(saved2);
            ctx.emit(Op::CatchEnd);

            // End: patch success jump
            let end = ctx.current_pos();
            ctx.patch_jump(jmp_pos, end);
        }

        Stmt::Match { subject, arms } => {
            // Compile match as chained if/else:
            //   evaluate subject → DUP + TypeOf for each arm → compare → execute body
            //
            // Strategy: subject on stack, then for each arm:
            //   DUP, TypeOf (pushes type string), compare with pattern,
            //   if match → execute body → jump to end
            //   else → next arm
            lower_expr(subject, ctx);

            let mut end_jumps: Vec<usize> = Vec::new();
            let mut wildcard_idx = None;

            for (i, arm) in arms.iter().enumerate() {
                match &arm.pattern {
                    crate::syntax::MatchPattern::Wildcard => {
                        wildcard_idx = Some(i);
                        break; // wildcard must be last
                    }
                    crate::syntax::MatchPattern::TypeName(name) => {
                        // DUP subject, TypeOf → compare with type name
                        ctx.emit(Op::Dup);
                        ctx.emit(Op::TypeOf);
                        // Load expected type name for comparison
                        ctx.emit(Op::Load(name.clone()));
                        // Call __match_type: pops type_result + expected, pushes match boolean
                        ctx.emit(Op::Call("__match_type".into()));
                        let jz_pos = ctx.current_pos();
                        ctx.emit(Op::Jz(0)); // skip body if no match
                        ctx.emit(Op::Pop); // pop match result

                        // Execute body
                        let saved = ctx.locals.len();
                        for s in &arm.body {
                            lower_stmt(s, ctx);
                        }
                        ctx.locals.truncate(saved);

                        // Jump to end
                        end_jumps.push(ctx.current_pos());
                        ctx.emit(Op::Jmp(0)); // placeholder

                        // Patch Jz → next arm
                        let next = ctx.current_pos();
                        ctx.patch_jump(jz_pos, next);
                        ctx.emit(Op::Pop); // pop match result on no-match path
                    }
                    crate::syntax::MatchPattern::MolLiteral { shape, relation, valence, arousal, time } => {
                        // DUP subject, push expected mol, compare
                        ctx.emit(Op::Dup);
                        let s = shape.unwrap_or(1) as u8;
                        let r = relation.unwrap_or(1) as u8;
                        let v = valence.unwrap_or(128) as u8;
                        let a = arousal.unwrap_or(128) as u8;
                        let t = time.unwrap_or(3) as u8;
                        ctx.emit(Op::PushMol(s, r, v, a, t));
                        ctx.emit(Op::Call("__match_mol".into()));
                        let jz_pos = ctx.current_pos();
                        ctx.emit(Op::Jz(0));
                        ctx.emit(Op::Pop);

                        let saved = ctx.locals.len();
                        for s_stmt in &arm.body {
                            lower_stmt(s_stmt, ctx);
                        }
                        ctx.locals.truncate(saved);

                        end_jumps.push(ctx.current_pos());
                        ctx.emit(Op::Jmp(0));

                        let next = ctx.current_pos();
                        ctx.patch_jump(jz_pos, next);
                        ctx.emit(Op::Pop);
                    }
                }
            }

            // Wildcard arm (default)
            if let Some(wi) = wildcard_idx {
                let saved = ctx.locals.len();
                for s in &arms[wi].body {
                    lower_stmt(s, ctx);
                }
                ctx.locals.truncate(saved);
            }

            // Pop the subject from stack
            ctx.emit(Op::Pop);

            // Patch all end jumps
            let end = ctx.current_pos();
            for jmp_pos in end_jumps {
                ctx.patch_jump(jmp_pos, end);
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

        Expr::Int(n) => {
            // Push numeric chain (Phase 1: Math Runtime)
            ctx.emit(Op::PushNum(*n as f64));
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
                // Only Context(∂) has no byte — semantic-only, use LCA
                ctx.emit(Op::Lca);
            }
        }

        Expr::RelQuery { subject, op } => {
            lower_expr(subject, ctx);
            if let Some(rel_byte) = op.to_rel_byte() {
                ctx.emit(Op::Query(rel_byte));
            } else {
                // Only Context(∂) has no byte — query Similar (0x07) as semantic proxy
                ctx.emit(Op::Query(0x07));
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
            // QT3: Arithmetic = hypothesis (chưa chứng minh)
            lower_expr(lhs, ctx);
            lower_expr(rhs, ctx);
            let fn_name = match op {
                crate::alphabet::ArithOp::Add => "__hyp_add",
                crate::alphabet::ArithOp::Sub => "__hyp_sub",
                crate::alphabet::ArithOp::Mul => "__hyp_mul",
                crate::alphabet::ArithOp::Div => "__hyp_div",
            };
            ctx.emit(Op::Call(fn_name.into()));
        }

        Expr::PhysOp { lhs, op, rhs } => {
            // QT3: Physical = proven (đã chứng minh)
            lower_expr(lhs, ctx);
            lower_expr(rhs, ctx);
            let fn_name = match op {
                crate::alphabet::PhysOp::PhysAdd => "__phys_add",
                crate::alphabet::PhysOp::PhysSub => "__phys_sub",
            };
            ctx.emit(Op::Call(fn_name.into()));
            // FUSE: verify result chain is finite (QT2: ∞-1)
            ctx.emit(Op::Fuse);
        }

        Expr::Truth { lhs, rhs } => {
            // QT3: == = proven truth
            lower_expr(lhs, ctx);
            lower_expr(rhs, ctx);
            ctx.emit(Op::Call("__assert_truth".into()));
        }

        Expr::Compare { lhs, op, rhs } => {
            // Compare: lhs < rhs → PushNum(lhs), PushNum(rhs), Call(__cmp_<op>)
            // Returns 1.0 (true) or 0.0 (false)
            lower_expr(lhs, ctx);
            lower_expr(rhs, ctx);
            let builtin = match op {
                CmpOp::Lt => "__cmp_lt",
                CmpOp::Gt => "__cmp_gt",
                CmpOp::Le => "__cmp_le",
                CmpOp::Ge => "__cmp_ge",
                CmpOp::Ne => "__cmp_ne",
            };
            ctx.emit(Op::Call(builtin.into()));
        }

        Expr::LogicAnd(a, b) => {
            // Short-circuit: if a is empty, result is empty; else result is b
            lower_expr(a, ctx);
            let jz_pos = ctx.current_pos();
            ctx.emit(Op::Jz(0)); // if a falsy → jump to end (leave empty on stack)
            ctx.emit(Op::Pop); // pop a (truthy)
            lower_expr(b, ctx);
            let end = ctx.current_pos();
            ctx.patch_jump(jz_pos, end);
        }

        Expr::LogicOr(a, b) => {
            // Short-circuit: if a is non-empty, result is a; else result is b
            lower_expr(a, ctx);
            // Jz: if a empty → eval b
            let jz_pos = ctx.current_pos();
            ctx.emit(Op::Jz(0));
            // a truthy: jump past b
            let jmp_pos = ctx.current_pos();
            ctx.emit(Op::Jmp(0));
            // a falsy: pop empty, eval b
            let false_branch = ctx.current_pos();
            ctx.patch_jump(jz_pos, false_branch);
            ctx.emit(Op::Pop);
            lower_expr(b, ctx);
            let end = ctx.current_pos();
            ctx.patch_jump(jmp_pos, end);
        }

        Expr::LogicNot(inner) => {
            // !expr: empty → non-empty (1.0), non-empty → empty
            lower_expr(inner, ctx);
            ctx.emit(Op::Call("__logic_not".into()));
        }

        Expr::Str(s) => {
            // String literal → Load as registry alias
            ctx.emit(Op::Load(s.clone()));
        }

        Expr::Group(inner) => {
            lower_expr(inner, ctx);
        }

        Expr::MolLiteral { shape, relation, valence, arousal, time } => {
            // Molecular literal → PushMol with defaults for unspecified dimensions
            let s = shape.unwrap_or(1) as u8;     // Sphere
            let r = relation.unwrap_or(1) as u8;   // Member
            let v = valence.unwrap_or(128) as u8;   // neutral
            let a = arousal.unwrap_or(128) as u8;   // moderate
            let t = time.unwrap_or(3) as u8;       // Medium
            ctx.emit(Op::PushMol(s, r, v, a, t));
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
        // QT3: +/- = hypothesis → __hyp_add
        let stmts = parse("1 + 2").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Call("__hyp_add".into())));
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

    // ── QT3: hypothesis vs physical vs truth ────────────────────────────────

    #[test]
    fn lower_hypothesis_arithmetic() {
        // QT3: +/- = hypothesis → __hyp_add
        let stmts = parse("1 + 2").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Call("__hyp_add".into())));
    }

    #[test]
    fn lower_physical_add() {
        // QT3: ⧺ = proven → __phys_add + FUSE
        let stmts = parse("fire ⧺ water").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Call("__phys_add".into())));
        assert!(prog.ops.contains(&Op::Fuse), "Physical ops must FUSE (QT2)");
    }

    #[test]
    fn lower_physical_sub() {
        // QT3: ⊖ = proven → __phys_sub + FUSE
        let stmts = parse("fire ⊖ water").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Call("__phys_sub".into())));
        assert!(prog.ops.contains(&Op::Fuse));
    }

    #[test]
    fn lower_truth_assertion() {
        // QT3: == = proven truth
        let stmts = parse("fire == water").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Call("__assert_truth".into())));
    }

    #[test]
    fn lower_fuse_command() {
        let stmts = parse("fuse").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Fuse));
    }

    #[test]
    fn validate_physical_ops() {
        let stmts = parse("fire ⧺ water").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_truth() {
        let stmts = parse("fire == water").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty());
    }

    // ── Reasoning & Debug primitives ────────────────────────────────────────

    #[test]
    fn lower_trace_command() {
        let stmts = parse("trace").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Trace));
    }

    #[test]
    fn lower_inspect_command() {
        let stmts = parse("inspect fire;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Load("fire".into())));
        assert!(prog.ops.contains(&Op::Inspect));
    }

    #[test]
    fn lower_assert_command() {
        let stmts = parse("assert fire;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Assert));
    }

    #[test]
    fn lower_typeof_command() {
        let stmts = parse("typeof fire;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::TypeOf));
    }

    #[test]
    fn lower_explain_command() {
        let stmts = parse("explain fire;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Explain));
    }

    // ── Phase 1: Math Runtime — numeric lowering ────────────────────────────

    #[test]
    fn lower_int_to_pushnum() {
        let stmts = parse("42").unwrap();
        let prog = lower(&stmts);
        assert!(
            prog.ops.iter().any(|op| matches!(op, Op::PushNum(n) if (*n - 42.0).abs() < f64::EPSILON)),
            "Int literal should lower to PushNum(42.0)"
        );
    }

    #[test]
    fn lower_addition_computes() {
        // "1 + 2" → PushNum(1), PushNum(2), Call(__hyp_add)
        let stmts = parse("1 + 2").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::PushNum(n) if (*n - 1.0).abs() < f64::EPSILON)));
        assert!(prog.ops.iter().any(|op| matches!(op, Op::PushNum(n) if (*n - 2.0).abs() < f64::EPSILON)));
        assert!(prog.ops.contains(&Op::Call("__hyp_add".into())));
    }

    // ── Molecular Literal — validation & lowering ──────────────────────────

    #[test]
    fn validate_mol_literal_valid() {
        let stmts = parse("{ S=1 R=6 V=200 A=180 T=4 }").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "Valid mol literal should pass: {:?}", errors);
    }

    #[test]
    fn validate_mol_literal_zero_shape_errors() {
        let stmts = parse("{ S=0 R=1 T=1 }").unwrap();
        let errors = validate(&stmts);
        assert!(
            errors.iter().any(|e| e.message.contains("S must be > 0")),
            "S=0 should error: {:?}", errors
        );
    }

    #[test]
    fn validate_mol_literal_exceeds_max() {
        let stmts = parse("{ S=999 }").unwrap();
        let errors = validate(&stmts);
        assert!(
            errors.iter().any(|e| e.message.contains("exceeds max 255")),
            "S=999 should error: {:?}", errors
        );
    }

    #[test]
    fn lower_mol_literal_all_dims() {
        let stmts = parse("{ S=1 R=6 V=200 A=180 T=4 }").unwrap();
        let prog = lower(&stmts);
        assert!(
            prog.ops.contains(&Op::PushMol(1, 6, 200, 180, 4)),
            "Should lower to PushMol(1,6,200,180,4): {:?}", prog.ops
        );
    }

    #[test]
    fn lower_mol_literal_defaults() {
        // Only S=5 specified → R=1(default), V=128, A=128, T=3
        let stmts = parse("{ S=5 }").unwrap();
        let prog = lower(&stmts);
        assert!(
            prog.ops.contains(&Op::PushMol(5, 1, 128, 128, 3)),
            "Unspecified dims should get defaults: {:?}", prog.ops
        );
    }

    #[test]
    fn lower_mol_literal_in_emit() {
        let stmts = parse("emit { S=2 R=3 V=100 A=50 T=1 };").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::PushMol(2, 3, 100, 50, 1)));
        assert!(prog.ops.contains(&Op::Emit));
    }

    // ── Type Inference ──────────────────────────────────────────────────

    #[test]
    fn infer_int_is_numeric() {
        let stmts = parse("42").unwrap();
        if let Stmt::Expr(ref e) = stmts[0] {
            assert_eq!(infer_expr_kind(e), ChainKind::Numeric);
        }
    }

    #[test]
    fn infer_arith_is_numeric() {
        let stmts = parse("1 + 2").unwrap();
        if let Stmt::Expr(ref e) = stmts[0] {
            assert_eq!(infer_expr_kind(e), ChainKind::Numeric);
        }
    }

    #[test]
    fn infer_ident_is_unknown() {
        let stmts = parse("fire").unwrap();
        if let Stmt::Expr(ref e) = stmts[0] {
            assert_eq!(infer_expr_kind(e), ChainKind::Unknown);
        }
    }

    #[test]
    fn infer_mol_literal_sdf() {
        let stmts = parse("{ S=1 }").unwrap();
        if let Stmt::Expr(ref e) = stmts[0] {
            assert_eq!(infer_expr_kind(e), ChainKind::Sdf);
        }
    }

    #[test]
    fn infer_mol_literal_emoticon() {
        let stmts = parse("{ V=200 A=200 }").unwrap();
        if let Stmt::Expr(ref e) = stmts[0] {
            assert_eq!(infer_expr_kind(e), ChainKind::Emoticon);
        }
    }

    #[test]
    fn infer_let_is_void() {
        let stmts = parse("let x = fire;").unwrap();
        assert_eq!(infer_stmt_kind(&stmts[0]), ChainKind::Void);
    }

    // ── Match ───────────────────────────────────────────────────────────

    #[test]
    fn validate_match_basic() {
        let stmts = parse("match fire { SDF => { emit water; } _ => { stats; } }").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "Basic match should be valid: {:?}", errors);
    }

    #[test]
    fn validate_match_unreachable_after_wildcard() {
        let stmts = parse("match fire { _ => { stats; } SDF => { dream; } }").unwrap();
        let errors = validate(&stmts);
        assert!(
            errors.iter().any(|e| e.message.contains("unreachable")),
            "Arms after wildcard should be unreachable: {:?}", errors
        );
    }

    #[test]
    fn lower_match_produces_typeof() {
        let stmts = parse("match fire { SDF => { emit water; } _ => { stats; } }").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::TypeOf), "Match should use TypeOf");
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jz(_))), "Match should have conditional jumps");
    }

    #[test]
    fn lower_match_wildcard_only() {
        let stmts = parse("match fire { _ => { stats; } }").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.contains(&Op::Stats), "Wildcard arm should emit Stats");
    }

    #[test]
    fn infer_match_is_void() {
        let stmts = parse("match fire { SDF => { stats; } }").unwrap();
        assert_eq!(infer_stmt_kind(&stmts[0]), ChainKind::Void);
    }

    // ── TryCatch ────────────────────────────────────────────────────────

    #[test]
    fn validate_try_catch_basic() {
        let stmts = parse("try { emit fire; } catch { stats; }").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "Basic try/catch should be valid: {:?}", errors);
    }

    #[test]
    fn lower_try_catch_has_opcodes() {
        let stmts = parse("try { emit fire; } catch { stats; }").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::TryBegin(_))));
        assert!(prog.ops.contains(&Op::CatchEnd));
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jmp(_))), "Should have Jmp to skip catch on success");
    }

    #[test]
    fn infer_try_catch_is_void() {
        let stmts = parse("try { emit fire; } catch { dream; }").unwrap();
        assert_eq!(infer_stmt_kind(&stmts[0]), ChainKind::Void);
    }

    #[test]
    fn validate_while_basic() {
        let stmts = parse("while x < 10 { emit x; }").unwrap();
        let warnings = validate(&stmts);
        // No errors expected
        assert!(!warnings.iter().any(|w| w.message.contains("Error")), "Unexpected errors: {:?}", warnings);
    }

    #[test]
    fn lower_while_has_loop_and_jz() {
        let stmts = parse("while x < 10 { emit x; }").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Loop(1024))), "Should have Loop(1024)");
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jz(_))), "Should have Jz for condition");
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Call(ref n) if n == "__cmp_lt")),
            "Should have __cmp_lt call");
    }

    #[test]
    fn infer_while_is_void() {
        let stmts = parse("while x < 10 { emit x; }").unwrap();
        assert_eq!(infer_stmt_kind(&stmts[0]), ChainKind::Void);
    }

    #[test]
    fn validate_compare_expr() {
        let stmts = parse("emit x >= 5;").unwrap();
        let warnings = validate(&stmts);
        assert!(!warnings.iter().any(|w| w.message.contains("Error")), "Unexpected errors: {:?}", warnings);
    }

    #[test]
    fn infer_compare_is_numeric() {
        let stmts = parse("emit x < 10;").unwrap();
        match &stmts[0] {
            Stmt::Emit(expr) => assert_eq!(infer_expr_kind(expr), ChainKind::Numeric),
            _ => panic!("Expected Emit"),
        }
    }

    #[test]
    fn lower_ne_has_cmp_ne() {
        let stmts = parse("emit x != 5;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Call(ref n) if n == "__cmp_ne")),
            "Should have __cmp_ne call");
    }

    #[test]
    fn lower_logic_not_has_call() {
        let stmts = parse("emit !x;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Call(ref n) if n == "__logic_not")),
            "Should have __logic_not call");
    }

    #[test]
    fn lower_logic_and_has_jz() {
        let stmts = parse("emit a && b;").unwrap();
        let prog = lower(&stmts);
        // && uses short-circuit: Jz to skip b if a is falsy
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jz(_))),
            "LogicAnd should use Jz for short-circuit");
    }

    #[test]
    fn lower_logic_or_has_jz_and_jmp() {
        let stmts = parse("emit a || b;").unwrap();
        let prog = lower(&stmts);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jz(_))));
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jmp(_))));
    }

    #[test]
    fn lower_break_has_jmp() {
        let stmts = parse("while x < 10 { break; }").unwrap();
        let prog = lower(&stmts);
        // break emits Jmp → patched to end of loop
        let jmp_count = prog.ops.iter().filter(|op| matches!(op, Op::Jmp(_))).count();
        assert!(jmp_count >= 1, "break should produce Jmp");
    }

    #[test]
    fn lower_continue_has_jmp() {
        let stmts = parse("for i in 0..5 { continue; }").unwrap();
        let prog = lower(&stmts);
        let jmp_count = prog.ops.iter().filter(|op| matches!(op, Op::Jmp(_))).count();
        assert!(jmp_count >= 1, "continue should produce Jmp");
    }

    // ── Variable Reassignment ────────────────────────────────────────────────

    #[test]
    fn validate_assign_undeclared() {
        let stmts = parse("x = 5;").unwrap();
        let errors = validate(&stmts);
        assert!(
            errors.iter().any(|e| e.message.contains("undeclared")),
            "Should warn about undeclared variable: {:?}",
            errors
        );
    }

    #[test]
    fn validate_assign_declared_ok() {
        let stmts = parse("let x = 1; x = 2;").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "Assign to declared var should be valid: {:?}", errors);
    }

    #[test]
    fn lower_assign_produces_store_update() {
        let stmts = parse("let x = 1; x = 2;").unwrap();
        let prog = lower(&stmts);
        let has_store_update = prog.ops.iter().any(|op| matches!(op, Op::StoreUpdate(_)));
        assert!(has_store_update, "Assign should produce StoreUpdate opcode");
    }

    // ── Return ───────────────────────────────────────────────────────────────

    #[test]
    fn lower_return_value() {
        let stmts = parse("fn foo() { return 42; } emit foo();").unwrap();
        let prog = lower(&stmts);
        let has_ret = prog.ops.iter().any(|op| matches!(op, Op::Ret));
        assert!(has_ret, "return should produce Ret opcode");
    }

    #[test]
    fn lower_return_bare() {
        // Functions are inlined at call site, so we need a call to produce Ret
        let stmts = parse("fn foo() { return; } emit foo();").unwrap();
        let prog = lower(&stmts);
        let has_ret = prog.ops.iter().any(|op| matches!(op, Op::Ret));
        assert!(has_ret, "bare return should produce Ret opcode");
    }

    // ── End-to-End (parse → lower → VM execute) ─────────────────────────────

    #[test]
    fn e2e_assign_in_while_loop() {
        // let x = 0; while x < 3 { emit x; x = x + 1; }
        let stmts = parse("let x = 0; while x < 3 { emit x; x = x + 1; }").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 3, "Should emit 3 values, got {}", outputs.len());
        let v0 = outputs[0].to_number().unwrap();
        let v1 = outputs[1].to_number().unwrap();
        let v2 = outputs[2].to_number().unwrap();
        assert!((v0 - 0.0).abs() < f64::EPSILON, "x=0");
        assert!((v1 - 1.0).abs() < f64::EPSILON, "x=1");
        assert!((v2 - 2.0).abs() < f64::EPSILON, "x=2");
    }

    #[test]
    fn e2e_return_from_function() {
        let stmts = parse("fn double(n) { return n + n; } emit double(21);").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1, "Should have output, got {}", outputs.len());
        let v = outputs[0].to_number().unwrap();
        assert!((v - 42.0).abs() < f64::EPSILON, "double(21)=42, got {}", v);
    }
}
