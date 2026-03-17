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

        Stmt::LetDestructure { names, value } => {
            validate_expr(value, scope, errors);
            for name in names {
                scope.define_local(name);
            }
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

        Stmt::ForEach { var, iter, body } => {
            validate_expr(iter, scope, errors);
            scope.enter();
            scope.locals.push(var.clone());
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

        Stmt::Use(_module) => {
            // Module imports are valid at any point
        }

        Stmt::FieldAssign { object, fields: _, value } => {
            validate_expr(value, scope, errors);
            if !scope.is_defined(object) {
                errors.push(SemError::new(&alloc::format!(
                    "Field assignment on undeclared variable '{}' (use 'let' to declare first)",
                    object
                )));
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

        Expr::Int(_) | Expr::Float(_) => {}

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

        Expr::Array(elements) => {
            for e in elements {
                validate_expr(e, scope, errors);
            }
        }

        Expr::Index { array, index } => {
            validate_expr(array, scope, errors);
            validate_expr(index, scope, errors);
        }

        Expr::Dict(fields) => {
            for (_key, value) in fields {
                validate_expr(value, scope, errors);
            }
        }

        Expr::FieldAccess { object, .. } => {
            validate_expr(object, scope, errors);
        }

        Expr::Pipe(left, right) => {
            validate_expr(left, scope, errors);
            validate_expr(right, scope, errors);
        }

        Expr::Lambda { params, body } => {
            scope.enter();
            for p in params {
                scope.define_local(p);
            }
            validate_expr(body, scope, errors);
            scope.exit();
        }

        Expr::IfExpr { cond, then_expr, else_expr } => {
            validate_expr(cond, scope, errors);
            validate_expr(then_expr, scope, errors);
            validate_expr(else_expr, scope, errors);
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
        Expr::Int(_) | Expr::Float(_) => ChainKind::Numeric,
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
        Expr::Array(_) => ChainKind::Unknown,
        Expr::Index { .. } => ChainKind::Unknown,
        Expr::Dict(_) => ChainKind::Unknown,
        Expr::FieldAccess { .. } => ChainKind::Unknown,
        Expr::Pipe(_, right) => infer_expr_kind(right),
        Expr::Lambda { .. } => ChainKind::Unknown,
        Expr::IfExpr { then_expr, .. } => infer_expr_kind(then_expr),
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
        Stmt::Let { .. } | Stmt::LetDestructure { .. }
        | Stmt::Emit(_) | Stmt::Command(_) | Stmt::CommandArg { .. } => {
            ChainKind::Void
        }
        Stmt::Expr(expr) => infer_expr_kind(expr),
        Stmt::If { .. } | Stmt::Loop { .. } | Stmt::FnDef { .. }
        | Stmt::Match { .. } | Stmt::TryCatch { .. } | Stmt::ForIn { .. }
        | Stmt::ForEach { .. }
        | Stmt::While { .. }
        | Stmt::Break | Stmt::Continue
        | Stmt::Assign { .. }
        | Stmt::FieldAssign { .. }
        | Stmt::Use(_) => ChainKind::Void,
        Stmt::Return(Some(expr)) => infer_expr_kind(expr),
        Stmt::Return(None) => ChainKind::Void,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Lowering — AST → OlangProgram
// ─────────────────────────────────────────────────────────────────────────────

/// Encode a string as a MolecularChain key (each byte → 1 molecule).
/// Used for dict field names — deterministic and comparable.
fn string_to_key_chain(s: &str) -> crate::molecular::MolecularChain {
    let mut mols = Vec::new();
    for b in s.bytes() {
        mols.push(crate::molecular::Molecule {
            shape: 0x02,       // marker: string key
            relation: 0x01,    // Member
            emotion: crate::molecular::EmotionDim { valence: b, arousal: 0 },
            time: 0x01,        // Static
        });
    }
    crate::molecular::MolecularChain(mols)
}

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
    /// Unique call site counter — prevents param name clashes when
    /// the same function is called multiple times in one expression
    call_id: u32,
    /// Depth of function inlining — when > 0, `return` should not emit Op::Ret
    /// but instead just leave the value on stack (since function is inlined)
    inline_depth: u32,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            prog: OlangProgram::new("olang"),
            locals: Vec::new(),
            fns: Vec::new(),
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
            call_id: 0,
            inline_depth: 0,
        }
    }

    fn next_call_id(&mut self) -> u32 {
        let id = self.call_id;
        self.call_id += 1;
        id
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

        Stmt::LetDestructure { names, value } => {
            // let { a, b } = dict → lower dict, then extract each field
            lower_expr(value, ctx);
            // Store dict as temporary
            ctx.emit(Op::Store("__destructure_tmp".into()));
            ctx.locals.push("__destructure_tmp".into());
            for name in names {
                ctx.emit(Op::LoadLocal("__destructure_tmp".into()));
                ctx.emit(Op::Push(string_to_key_chain(name)));
                ctx.emit(Op::Call("__dict_get".into()));
                ctx.emit(Op::Store(name.clone()));
                ctx.locals.push(name.clone());
            }
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

        Stmt::ForEach { var, iter, body } => {
            // for var in arr { body }
            // Strategy: store arr as local, get len, iterate with index counter
            // Stack: only idx on stack during loop; arr and len in locals
            lower_expr(iter, ctx);
            // Store arr as local
            ctx.emit(Op::Dup);                   // dup for len
            ctx.emit(Op::Call("__array_len".into()));
            ctx.emit(Op::Store("__foreach_len".into()));
            ctx.locals.push("__foreach_len".into());
            ctx.emit(Op::Store("__foreach_arr".into()));
            ctx.locals.push("__foreach_arr".into());
            // Push index counter = 0
            ctx.emit(Op::PushNum(0.0));
            ctx.emit(Op::Loop(1024));            // QT2: capped
            ctx.emit(Op::ScopeBegin);
            // Check idx >= len → break
            ctx.emit(Op::Dup);                   // [..., idx, idx]
            ctx.emit(Op::LoadLocal("__foreach_len".into()));
            ctx.emit(Op::Call("__cmp_ge".into())); // idx >= len → truthy
            let jz_pos = ctx.current_pos();
            ctx.emit(Op::Jz(0));                 // if falsy (idx < len) → continue
            ctx.emit(Op::Pop);                   // pop cmp result (truthy)
            let break_jmp = ctx.current_pos();
            ctx.emit(Op::Jmp(0));                // break out

            let cont_target = ctx.current_pos();
            ctx.patch_jump(jz_pos, cont_target);
            ctx.emit(Op::Pop);                   // pop cmp result (falsy)

            // Get element: arr[idx] → store as var
            ctx.emit(Op::Dup);                   // [..., idx, idx]
            ctx.emit(Op::LoadLocal("__foreach_arr".into())); // [..., idx, idx, arr]
            ctx.emit(Op::Swap);                  // [..., idx, arr, idx]
            ctx.emit(Op::Call("__array_get".into())); // [..., idx, elem]
            ctx.emit(Op::Store(var.clone()));
            ctx.locals.push(var.clone());

            // Body
            ctx.break_jumps.push(Vec::new());
            ctx.continue_jumps.push(Vec::new());
            let saved = ctx.locals.len();
            for s in body {
                lower_stmt(s, ctx);
            }
            ctx.locals.truncate(saved);

            // Patch continue → increment
            let inc_pos = ctx.current_pos();
            if let Some(conts) = ctx.continue_jumps.pop() {
                for cp in conts {
                    ctx.patch_jump(cp, inc_pos);
                }
            }

            // Increment index
            ctx.emit(Op::PushNum(1.0));
            ctx.emit(Op::Call("__hyp_add".into()));
            ctx.emit(Op::ScopeEnd);

            let end_pos = ctx.current_pos();
            ctx.patch_jump(break_jmp, end_pos);
            if let Some(breaks) = ctx.break_jumps.pop() {
                for bp in breaks {
                    ctx.patch_jump(bp, end_pos);
                }
            }
            ctx.emit(Op::Pop); // pop idx
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

        Stmt::FieldAssign { object, fields, value } => {
            // obj.field = value → load obj, push field key chain, lower value, __dict_set, store back
            // For nested: obj.a.b = value → load obj, get a, set b=value, set a=modified, store obj
            if fields.len() == 1 {
                // Simple: obj.field = value
                if ctx.is_local(object) {
                    ctx.emit(Op::LoadLocal(object.clone()));
                } else {
                    ctx.emit(Op::Load(object.clone()));
                }
                ctx.emit(Op::Push(string_to_key_chain(&fields[0])));
                lower_expr(value, ctx);
                ctx.emit(Op::Call("__dict_set".into()));
                ctx.emit(Op::StoreUpdate(object.clone()));
            } else {
                // Nested: obj.a.b.c = value
                // Strategy: load obj, navigate to parent dict, set last field, rebuild path
                // Store intermediate dicts as locals for reassembly
                let depth = fields.len();
                // Load root
                if ctx.is_local(object) {
                    ctx.emit(Op::LoadLocal(object.clone()));
                } else {
                    ctx.emit(Op::Load(object.clone()));
                }
                // Navigate down, storing intermediate dicts
                for (i, field) in fields.iter().enumerate().take(depth - 1) {
                    ctx.emit(Op::Dup); // keep copy for later reassembly
                    let local_name: String = alloc::format!("__nested_{}", i);
                    ctx.emit(Op::Store(local_name.clone()));
                    ctx.locals.push(local_name);
                    ctx.emit(Op::Push(string_to_key_chain(field)));
                    ctx.emit(Op::Call("__dict_get".into()));
                }
                // Set the last field on the innermost dict
                ctx.emit(Op::Push(string_to_key_chain(&fields[depth - 1])));
                lower_expr(value, ctx);
                ctx.emit(Op::Call("__dict_set".into()));
                // Rebuild path from inside out
                for i in (0..depth - 1).rev() {
                    let local_name: String = alloc::format!("__nested_{}", i);
                    ctx.emit(Op::LoadLocal(local_name));
                    ctx.emit(Op::Swap); // [parent, modified_child]
                    ctx.emit(Op::Push(string_to_key_chain(&fields[i])));
                    ctx.emit(Op::Swap); // [parent, key, modified_child]
                    ctx.emit(Op::Call("__dict_set".into()));
                }
                ctx.emit(Op::StoreUpdate(object.clone()));
            }
        }

        Stmt::Use(module) => {
            // Emit a Load for the module name — runtime can intercept and load the module
            ctx.emit(Op::Load(module.clone()));
            ctx.emit(Op::Call("__use_module".into()));
        }

        Stmt::Return(expr) => {
            if ctx.inline_depth > 0 {
                // Inside inlined function: just leave value on stack (no Ret)
                if let Some(e) = expr {
                    lower_expr(e, ctx);
                    // Value stays on stack as function's return value
                }
                // No Op::Ret — inlined functions continue after the call site
            } else {
                // Top-level return
                if let Some(e) = expr {
                    lower_expr(e, ctx);
                    ctx.emit(Op::Emit); // return value = emit it
                }
                ctx.emit(Op::Ret);
            }
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
            ctx.emit(Op::PushNum(*n as f64));
        }

        Expr::Float(f) => {
            ctx.emit(Op::PushNum(*f));
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
            // Built-in function mappings
            let builtin = match name.as_str() {
                "len" => Some("__array_len"),
                "push" => Some("__array_push"),
                "concat" => Some("__concat"),
                "head" => Some("__head"),
                "tail" => Some("__tail"),
                "get" => Some("__dict_get"),
                "set" => Some("__dict_set"),
                "keys" => Some("__dict_keys"),
                "str_len" => Some("__str_len"),
                "str_concat" => Some("__str_concat"),
                "to_string" => Some("__to_string"),
                "to_number" => Some("__to_number"),
                "print" => Some("__print"),
                "println" => Some("__println"),
                "abs" => Some("__hyp_abs"),
                "min" => Some("__hyp_min"),
                "max" => Some("__hyp_max"),
                "neg" => Some("__hyp_neg"),
                "mod" => Some("__hyp_mod"),
                "array_set" => Some("__array_set"),
                "slice" => Some("__array_slice"),
                "is_empty" => Some("__is_empty"),
                "eq" => Some("__eq"),
                // String builtins
                "str_split" => Some("__str_split"),
                "str_contains" => Some("__str_contains"),
                "str_replace" => Some("__str_replace"),
                "str_starts_with" => Some("__str_starts_with"),
                "str_ends_with" => Some("__str_ends_with"),
                "str_index_of" => Some("__str_index_of"),
                "str_trim" => Some("__str_trim"),
                "str_upper" => Some("__str_upper"),
                "str_lower" => Some("__str_lower"),
                "str_substr" => Some("__str_substr"),
                // Math builtins
                "floor" => Some("__hyp_floor"),
                "ceil" => Some("__hyp_ceil"),
                "round" => Some("__hyp_round"),
                "sqrt" => Some("__hyp_sqrt"),
                "pow" => Some("__hyp_pow"),
                "log" => Some("__hyp_log"),
                "sin" => Some("__hyp_sin"),
                "cos" => Some("__hyp_cos"),
                // Dict builtins
                "has_key" => Some("__dict_has_key"),
                "values" => Some("__dict_values"),
                "merge" => Some("__dict_merge"),
                "remove" => Some("__dict_remove"),
                // Array builtins
                "pop" => Some("__array_pop"),
                "reverse" => Some("__array_reverse"),
                "contains" => Some("__array_contains"),
                "join" => Some("__array_join"),
                "map" => Some("__array_map"),
                "filter" => Some("__array_filter"),
                // ISL builtins
                "isl_send" => Some("__isl_send"),
                "isl_broadcast" => Some("__isl_broadcast"),
                // Type/chain builtins
                "type_of" => Some("__type_of"),
                "chain_hash" => Some("__chain_hash"),
                "chain_len" => Some("__chain_len"),
                _ => None,
            };
            if let Some(builtin_name) = builtin {
                // For dict builtins, string args need key encoding
                if builtin_name == "__dict_get" || builtin_name == "__dict_set"
                   || builtin_name == "__dict_has_key" || builtin_name == "__dict_remove" {
                    // get(dict, "key") / set(dict, "key", value)
                    for arg in args {
                        // If arg is a string literal, encode as key chain
                        if let Expr::Str(s) = arg {
                            ctx.emit(Op::Push(string_to_key_chain(s)));
                        } else {
                            lower_expr(arg, ctx);
                        }
                    }
                } else {
                    for arg in args {
                        lower_expr(arg, ctx);
                    }
                }
                ctx.emit(Op::Call(builtin_name.into()));
            } else
            // Check if it's a user-defined function
            if let Some(fn_def) = ctx.lookup_fn(name) {
                // Inline the function with unique param names per call site
                // to prevent clashes when the same function is called multiple times
                let call_id = ctx.next_call_id();
                let saved = ctx.locals.len();

                // Generate unique param names: __p{call_id}_{param}
                let unique_params: Vec<String> = fn_def.params.iter()
                    .map(|p| alloc::format!("__p{}_{}", call_id, p))
                    .collect();

                // Evaluate args and store with unique names
                for (unique_name, arg) in unique_params.iter().zip(args.iter()) {
                    lower_expr(arg, ctx);
                    ctx.emit(Op::Store(unique_name.clone()));
                    ctx.locals.push(unique_name.clone());
                }

                // Also register original param names as aliases to unique names
                // so the function body can use them
                for (orig, unique_name) in fn_def.params.iter().zip(unique_params.iter()) {
                    ctx.emit(Op::LoadLocal(unique_name.clone()));
                    ctx.emit(Op::Store(orig.clone()));
                    ctx.locals.push(orig.clone());
                }

                // Lower function body (inside inline context)
                ctx.inline_depth += 1;

                // Lower all statements except the last one normally
                let body_len = fn_def.body.len();
                if body_len > 1 {
                    for s in &fn_def.body[..body_len - 1] {
                        lower_stmt(s, ctx);
                    }
                }

                // Handle last statement specially: if it's an expression,
                // don't Pop the result — it becomes the function's return value
                if let Some(last) = fn_def.body.last() {
                    match last {
                        Stmt::Expr(expr) => {
                            // Lower expression without Pop — value stays on stack
                            lower_expr(expr, ctx);
                        }
                        Stmt::Return(Some(expr)) => {
                            // Return value stays on stack
                            lower_expr(expr, ctx);
                        }
                        _ => {
                            // Other statements (emit, let, etc.) — lower normally
                            // and push empty as dummy return value
                            lower_stmt(last, ctx);
                            ctx.emit(Op::Push(crate::molecular::MolecularChain::empty()));
                        }
                    }
                } else {
                    // Empty function body — push empty
                    ctx.emit(Op::Push(crate::molecular::MolecularChain::empty()));
                }

                ctx.inline_depth -= 1;

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
                crate::alphabet::ArithOp::Mod => "__hyp_mod",
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
            // String literal → push as chain value (each byte → 1 molecule)
            ctx.emit(Op::Push(string_to_key_chain(s)));
        }

        Expr::Group(inner) => {
            lower_expr(inner, ctx);
        }

        Expr::Array(elements) => {
            // Push each element, then count, then call __array_new
            // Stack layout before call: [... elem0, elem1, ..., elemN-1, count]
            for e in elements {
                lower_expr(e, ctx);
            }
            ctx.emit(Op::PushNum(elements.len() as f64));
            ctx.emit(Op::Call("__array_new".into()));
        }

        Expr::Index { array, index } => {
            // Push array, push index, call __array_get
            lower_expr(array, ctx);
            lower_expr(index, ctx);
            ctx.emit(Op::Call("__array_get".into()));
        }

        Expr::Dict(fields) => {
            // Push key/value pairs alternating, then count, then call __dict_new
            // Stack: [key0, val0, key1, val1, ..., count]
            // Keys encoded as MolecularChain (each byte → 1 molecule)
            for (key, value) in fields {
                ctx.emit(Op::Push(string_to_key_chain(key)));
                lower_expr(value, ctx);
            }
            ctx.emit(Op::PushNum(fields.len() as f64)); // number of pairs
            ctx.emit(Op::Call("__dict_new".into()));
        }

        Expr::FieldAccess { object, field } => {
            // obj.field → load obj, push field key chain, __dict_get
            lower_expr(object, ctx);
            ctx.emit(Op::Push(string_to_key_chain(field)));
            ctx.emit(Op::Call("__dict_get".into()));
        }

        Expr::Pipe(left, right) => {
            // a |> f → evaluate a, then pass as first argument to f
            // If right is Call: inject left as first arg
            // If right is Ident: Call(ident, [left_result])
            lower_expr(left, ctx);
            match right.as_ref() {
                Expr::Call { name, args } => {
                    // left result is already on stack as first arg
                    for arg in args {
                        lower_expr(arg, ctx);
                    }
                    // Check builtins
                    let builtin = match name.as_str() {
                        "len" => Some("__array_len"),
                        "push" => Some("__array_push"),
                        "concat" => Some("__concat"),
                        "head" => Some("__head"),
                        "tail" => Some("__tail"),
                        "get" => Some("__dict_get"),
                        "set" => Some("__dict_set"),
                        "keys" => Some("__dict_keys"),
                        "values" => Some("__dict_values"),
                        "reverse" => Some("__array_reverse"),
                        "contains" => Some("__array_contains"),
                        "join" => Some("__array_join"),
                        "str_split" => Some("__str_split"),
                        "str_contains" => Some("__str_contains"),
                        "str_trim" => Some("__str_trim"),
                        "str_upper" => Some("__str_upper"),
                        "str_lower" => Some("__str_lower"),
                        _ => None,
                    };
                    ctx.emit(Op::Call(builtin.unwrap_or(name.as_str()).into()));
                }
                Expr::Ident(name) => {
                    // left already on stack, call function with it
                    let builtin = match name.as_str() {
                        "len" => Some("__array_len"),
                        "push" => Some("__array_push"),
                        "concat" => Some("__concat"),
                        "head" => Some("__head"),
                        "tail" => Some("__tail"),
                        "keys" => Some("__dict_keys"),
                        "values" => Some("__dict_values"),
                        "reverse" => Some("__array_reverse"),
                        "str_trim" => Some("__str_trim"),
                        "str_upper" => Some("__str_upper"),
                        "str_lower" => Some("__str_lower"),
                        _ => None,
                    };
                    if let Some(b) = builtin {
                        ctx.emit(Op::Call(b.into()));
                    } else if let Some(fn_def) = ctx.lookup_fn(name) {
                        // Inline: bind left (on stack) as first param with unique name
                        let call_id = ctx.next_call_id();
                        let saved = ctx.locals.len();
                        if let Some(first_param) = fn_def.params.first() {
                            let unique_name = alloc::format!("__p{}_{}", call_id, first_param);
                            ctx.emit(Op::Store(unique_name.clone()));
                            ctx.locals.push(unique_name.clone());
                            ctx.emit(Op::LoadLocal(unique_name));
                            ctx.emit(Op::Store(first_param.clone()));
                            ctx.locals.push(first_param.clone());
                        }
                        ctx.inline_depth += 1;
                        // Handle last statement as return value
                        let body_len = fn_def.body.len();
                        if body_len > 1 {
                            for s in &fn_def.body[..body_len - 1] {
                                lower_stmt(s, ctx);
                            }
                        }
                        if let Some(last) = fn_def.body.last() {
                            match last {
                                Stmt::Expr(expr) => lower_expr(expr, ctx),
                                Stmt::Return(Some(expr)) => lower_expr(expr, ctx),
                                _ => {
                                    lower_stmt(last, ctx);
                                    ctx.emit(Op::Push(crate::molecular::MolecularChain::empty()));
                                }
                            }
                        } else {
                            ctx.emit(Op::Push(crate::molecular::MolecularChain::empty()));
                        }
                        ctx.inline_depth -= 1;
                        ctx.locals.truncate(saved);
                    } else {
                        ctx.emit(Op::Call(name.clone()));
                    }
                }
                _ => {
                    // Fallback: evaluate right, treat as function call
                    lower_expr(right, ctx);
                    ctx.emit(Op::Call("__pipe_apply".into()));
                }
            }
        }

        Expr::Lambda { params, body } => {
            // Lambda: |x, y| expr
            // In stack-based VM without closures, we inline the lambda
            // The lambda value itself just pushes an empty marker;
            // actual inlining happens when lambda is used via pipe or call
            // For now: just lower the body (params should be bound by caller)
            // This handles the case where lambda appears in a pipe: a |> |x| x + 1
            // The pipe lowering already put 'a' on stack, we store it as param
            let saved = ctx.locals.len();
            for p in params {
                ctx.emit(Op::Store(p.clone()));
                ctx.locals.push(p.clone());
            }
            lower_expr(body, ctx);
            ctx.locals.truncate(saved);
        }

        Expr::IfExpr { cond, then_expr, else_expr } => {
            // if cond { then } else { else } as expression
            lower_expr(cond, ctx);
            let jz_pos = ctx.current_pos();
            ctx.emit(Op::Jz(0)); // if falsy → else branch
            ctx.emit(Op::Pop); // pop cond (truthy)
            lower_expr(then_expr, ctx);
            let jmp_pos = ctx.current_pos();
            ctx.emit(Op::Jmp(0)); // skip else
            let else_target = ctx.current_pos();
            ctx.patch_jump(jz_pos, else_target);
            ctx.emit(Op::Pop); // pop cond (falsy)
            lower_expr(else_expr, ctx);
            let end = ctx.current_pos();
            ctx.patch_jump(jmp_pos, end);
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
        // String literals are now pushed as chain values (not loaded as aliases)
        let has_push = prog.ops.iter().any(|op| matches!(op, Op::Push(_)));
        assert!(has_push, "String literal should produce Push(chain)");
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
        // Inlined function: return 42 leaves value on stack (no Op::Ret)
        let stmts = parse("fn foo() { return 42; } emit foo();").unwrap();
        let prog = lower(&stmts);
        let has_pushnum = prog.ops.iter().any(|op| matches!(op, Op::PushNum(n) if *n == 42.0));
        assert!(has_pushnum, "return 42 should produce PushNum(42)");
        assert!(prog.ops.contains(&Op::Emit), "should emit the return value");
    }

    #[test]
    fn lower_return_bare() {
        // Inlined function with bare return: no value left on stack
        let stmts = parse("fn foo() { return; } foo();").unwrap();
        let prog = lower(&stmts);
        // Bare return in inlined context pushes empty chain as dummy
        let has_push_empty = prog.ops.iter().any(|op| {
            matches!(op, Op::Push(c) if c.is_empty())
        });
        assert!(has_push_empty, "bare return in inline should push empty");
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

    // ── Array Tests ──────────────────────────────────────────────────────────

    #[test]
    fn parse_array_literal() {
        let stmts = parse("let arr = [1, 2, 3];").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Let { value: Expr::Array(elems), .. } = &stmts[0] {
            assert_eq!(elems.len(), 3);
        } else {
            panic!("Expected Let with Array, got {:?}", stmts[0]);
        }
    }

    #[test]
    fn parse_array_indexing() {
        let stmts = parse("emit arr[0];").unwrap();
        assert_eq!(stmts.len(), 1, "stmts: {:?}", stmts);
        match &stmts[0] {
            Stmt::Emit(Expr::Index { .. }) => { /* ok */ }
            other => panic!("Expected Emit with Index, got {:?}", other),
        }
    }

    #[test]
    fn parse_empty_array() {
        let stmts = parse("let arr = [];").unwrap();
        if let Stmt::Let { value: Expr::Array(elems), .. } = &stmts[0] {
            assert_eq!(elems.len(), 0);
        } else {
            panic!("Expected empty array");
        }
    }

    #[test]
    fn lower_array_ops() {
        let stmts = parse("let arr = [10, 20, 30]; emit arr[1];").unwrap();
        let prog = lower(&stmts);
        let op_names: Vec<_> = prog.ops.iter().map(|op| op.name()).collect();
        // Expected: PushNum(10), PushNum(20), PushNum(30), PushNum(3), CALL(__array_new), STORE(arr)
        // then LOAD_LOCAL(arr), PushNum(1), CALL(__array_get), EMIT, HALT
        assert!(op_names.contains(&"PUSH_NUM"), "ops: {:?}", op_names);
        assert!(op_names.contains(&"CALL"), "ops: {:?}", op_names);
        assert!(op_names.contains(&"STORE"), "ops: {:?}", op_names);
        assert!(op_names.contains(&"EMIT"), "ops: {:?}", op_names);
    }

    #[test]
    fn e2e_array_get() {
        let stmts = parse("let arr = [10, 20, 30]; emit arr[1];").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1, "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 20.0).abs() < f64::EPSILON, "arr[1]=20, got {}", v);
    }

    #[test]
    fn e2e_array_len() {
        let stmts = parse("let arr = [10, 20, 30]; emit len(arr);").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1, "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "len([10,20,30])=3, got {}", v);
    }

    // ── Dict / .field tests ───────────────────────────────────────────────

    #[test]
    fn parse_dict_literal() {
        let stmts = parse("let d = { name: 42, age: 10 };").unwrap();
        match &stmts[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "d");
                match value {
                    Expr::Dict(fields) => {
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].0, "name");
                        assert_eq!(fields[1].0, "age");
                    }
                    other => panic!("Expected Dict, got {:?}", other),
                }
            }
            other => panic!("Expected Let, got {:?}", other),
        }
    }

    #[test]
    fn parse_empty_dict() {
        // Empty {} is MolLiteral (backwards compat), not Dict
        let stmts = parse("let d = {};").unwrap();
        match &stmts[0] {
            Stmt::Let { value, .. } => {
                assert!(matches!(value, Expr::MolLiteral { .. }));
            }
            other => panic!("Expected Let, got {:?}", other),
        }
    }

    #[test]
    fn parse_field_access() {
        let stmts = parse("emit d.name;").unwrap();
        match &stmts[0] {
            Stmt::Emit(Expr::FieldAccess { object, field }) => {
                assert!(matches!(object.as_ref(), Expr::Ident(n) if n == "d"));
                assert_eq!(field, "name");
            }
            other => panic!("Expected Emit(FieldAccess), got {:?}", other),
        }
    }

    #[test]
    fn parse_chained_field_access() {
        let stmts = parse("emit a.b.c;").unwrap();
        match &stmts[0] {
            Stmt::Emit(Expr::FieldAccess { object, field }) => {
                assert_eq!(field, "c");
                match object.as_ref() {
                    Expr::FieldAccess { object: inner, field: f2 } => {
                        assert_eq!(f2, "b");
                        assert!(matches!(inner.as_ref(), Expr::Ident(n) if n == "a"));
                    }
                    other => panic!("Expected nested FieldAccess, got {:?}", other),
                }
            }
            other => panic!("Expected Emit(FieldAccess), got {:?}", other),
        }
    }

    #[test]
    fn parse_field_assign() {
        let stmts = parse("let d = { x: 1 }; d.x = 42;").unwrap();
        assert_eq!(stmts.len(), 2);
        match &stmts[1] {
            Stmt::FieldAssign { object, fields, value } => {
                assert_eq!(object, "d");
                assert_eq!(fields, &["x"]);
                assert!(matches!(value, Expr::Int(42)));
            }
            other => panic!("Expected FieldAssign, got {:?}", other),
        }
    }

    #[test]
    fn validate_field_assign_undeclared() {
        let stmts = parse("x.name = 42;").unwrap();
        let errors = validate(&stmts);
        assert!(
            errors.iter().any(|e| e.message.contains("undeclared")),
            "Should warn on undeclared field assign: {:?}", errors
        );
    }

    #[test]
    fn lower_dict_produces_dict_new() {
        let stmts = parse("let d = { x: 1, y: 2 };").unwrap();
        let prog = lower(&stmts);
        assert!(
            prog.ops.iter().any(|op| matches!(op, Op::Call(n) if n == "__dict_new")),
            "Should emit __dict_new call: {:?}", prog.ops
        );
        // Should push 2 as count (2 pairs)
        assert!(
            prog.ops.iter().any(|op| matches!(op, Op::PushNum(n) if (*n - 2.0).abs() < f64::EPSILON)),
            "Should push pair count 2: {:?}", prog.ops
        );
    }

    #[test]
    fn lower_field_access_produces_dict_get() {
        let stmts = parse("let d = { x: 1 }; emit d.x;").unwrap();
        let prog = lower(&stmts);
        assert!(
            prog.ops.iter().any(|op| matches!(op, Op::Call(n) if n == "__dict_get")),
            "Should emit __dict_get call: {:?}", prog.ops
        );
    }

    #[test]
    fn lower_field_assign_produces_dict_set() {
        let stmts = parse("let d = { x: 1 }; d.x = 42;").unwrap();
        let prog = lower(&stmts);
        assert!(
            prog.ops.iter().any(|op| matches!(op, Op::Call(n) if n == "__dict_set")),
            "Should emit __dict_set call: {:?}", prog.ops
        );
        assert!(
            prog.ops.iter().any(|op| matches!(op, Op::StoreUpdate(n) if n == "d")),
            "Should emit StoreUpdate to save dict back: {:?}", prog.ops
        );
    }

    #[test]
    fn e2e_dict_field_access() {
        // Create dict, access field, emit value
        let stmts = parse("let d = { x: 42, y: 10 }; emit d.x;").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs.is_empty(), "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 42.0).abs() < f64::EPSILON, "d.x should be 42, got {}", v);
    }

    #[test]
    fn e2e_dict_field_assign() {
        // Create dict, assign field, emit new value
        let stmts = parse("let d = { x: 1 }; d.x = 99; emit d.x;").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs.is_empty(), "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 99.0).abs() < f64::EPSILON, "d.x should be 99 after assign, got {}", v);
    }

    #[test]
    fn e2e_dict_multiple_fields() {
        let stmts = parse("let d = { a: 10, b: 20, c: 30 }; emit d.b;").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs.is_empty(), "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 20.0).abs() < f64::EPSILON, "d.b should be 20, got {}", v);
    }

    #[test]
    fn e2e_dict_add_new_field() {
        // Add field that didn't exist initially
        let stmts = parse("let d = { x: 1 }; d.y = 77; emit d.y;").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs.is_empty(), "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 77.0).abs() < f64::EPSILON, "d.y should be 77, got {}", v);
    }

    // ── Phase 2 Tests ───────────────────────────────────────────────────────

    #[test]
    fn parse_comments() {
        // Line comment
        let stmts = parse("let x = 1; // this is a comment\nemit x;").unwrap();
        assert_eq!(stmts.len(), 2);
        // Block comment
        let stmts2 = parse("let x = /* hidden */ 1; emit x;").unwrap();
        assert_eq!(stmts2.len(), 2);
    }

    #[test]
    fn parse_pipe_operator() {
        let stmts = parse("emit 5 |> to_string;").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Emit(Expr::Pipe(_, _)) => {}
            other => panic!("Expected Emit(Pipe), got {:?}", other),
        }
    }

    #[test]
    fn parse_lambda() {
        let stmts = parse("let f = |x| x + 1;").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Let { value: Expr::Lambda { params, .. }, .. } => {
                assert_eq!(params, &["x"]);
            }
            other => panic!("Expected Let with Lambda, got {:?}", other),
        }
    }

    #[test]
    fn parse_if_expr() {
        let stmts = parse("let x = if 1 { 10 } else { 20 };").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Let { value: Expr::IfExpr { .. }, .. } => {}
            other => panic!("Expected Let with IfExpr, got {:?}", other),
        }
    }

    #[test]
    fn e2e_if_expr() {
        let stmts = parse("let x = if 1 { 10 } else { 20 }; emit x;").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs.is_empty(), "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 10.0).abs() < f64::EPSILON, "truthy condition → 10, got {}", v);
    }

    #[test]
    fn parse_let_destructure() {
        let stmts = parse("let { a, b } = { a: 1, b: 2 };").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::LetDestructure { names, .. } => {
                assert_eq!(names, &["a", "b"]);
            }
            other => panic!("Expected LetDestructure, got {:?}", other),
        }
    }

    #[test]
    fn e2e_destructure() {
        let stmts = parse("let { x, y } = { x: 10, y: 20 }; emit x; emit y;").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 2, "Should have 2 outputs, got {:?}", outputs);
        let vx = outputs[0].to_number().unwrap();
        let vy = outputs[1].to_number().unwrap();
        assert!((vx - 10.0).abs() < f64::EPSILON, "x=10, got {}", vx);
        assert!((vy - 20.0).abs() < f64::EPSILON, "y=20, got {}", vy);
    }

    #[test]
    fn e2e_for_each() {
        let stmts = parse("let arr = [10, 20, 30]; let sum = 0; for x in arr { sum = sum + x; } emit sum;").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs.is_empty(), "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 60.0).abs() < f64::EPSILON, "sum=60, got {}", v);
    }

    #[test]
    fn parse_method_call() {
        // obj.method(args) desugars to Call { name: "method", args: [obj, ...] }
        let stmts = parse("emit arr.len();").unwrap();
        match &stmts[0] {
            Stmt::Emit(Expr::Call { name, args }) => {
                assert_eq!(name, "len");
                assert_eq!(args.len(), 1);
            }
            other => panic!("Expected Emit(Call), got {:?}", other),
        }
    }

    #[test]
    fn e2e_method_call_len() {
        let stmts = parse("let arr = [1, 2, 3]; emit arr.len();").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        let v = outputs[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "arr.len()=3, got {}", v);
    }

    #[test]
    fn parse_nested_field_assign() {
        let stmts = parse("let d = { a: { b: 1 } }; d.a.b = 42;").unwrap();
        match &stmts[1] {
            Stmt::FieldAssign { object, fields, .. } => {
                assert_eq!(object, "d");
                assert_eq!(fields, &["a", "b"]);
            }
            other => panic!("Expected FieldAssign, got {:?}", other),
        }
    }

    // ── Phase 5: String Builtins ────────────────────────────────────────────

    #[test]
    fn e2e_str_contains() {
        let stmts = parse("emit str_contains(\"hello world\", \"world\");").unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
    }

    #[test]
    fn e2e_str_index_of() {
        let stmts = parse("emit str_index_of(\"abcdef\", \"cd\");").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs.is_empty());
    }

    #[test]
    fn e2e_math_builtins() {
        // floor(3.7) = 3, ceil(3.2) = 4, round(3.5) = 4
        let stmts = parse("emit floor(3.7);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        let v = outputs[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "floor(3.7)=3, got {}", v);
    }

    #[test]
    fn e2e_ceil() {
        let stmts = parse("emit ceil(3.2);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 4.0).abs() < f64::EPSILON, "ceil(3.2)=4, got {}", v);
    }

    #[test]
    fn e2e_sqrt() {
        let stmts = parse("emit sqrt(25);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 5.0).abs() < 0.01, "sqrt(25)=5, got {}", v);
    }

    #[test]
    fn e2e_pow() {
        let stmts = parse("emit pow(2, 10);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 1024.0).abs() < f64::EPSILON, "pow(2,10)=1024, got {}", v);
    }

    #[test]
    fn e2e_array_reverse() {
        let stmts = parse("let arr = [1, 2, 3]; emit reverse(arr);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
    }

    #[test]
    fn e2e_array_contains() {
        let stmts = parse("let arr = [10, 20, 30]; emit contains(arr, 20);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        // 20 is in the array → should be truthy (1.0)
        assert!(!outputs[0].is_empty(), "contains should be truthy");
    }

    #[test]
    fn e2e_dict_has_key() {
        let stmts = parse("let d = { x: 10, y: 20 }; emit has_key(d, \"x\");").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs[0].is_empty(), "has_key should be truthy");
    }

    #[test]
    fn e2e_dict_values() {
        let stmts = parse("let d = { x: 10, y: 20 }; emit values(d);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
    }

    #[test]
    fn e2e_string_escape_sequences() {
        // Test that escape sequences are parsed correctly
        let stmts = parse("emit \"hello\\nworld\";").unwrap();
        match &stmts[0] {
            Stmt::Emit(Expr::Str(s)) => {
                assert!(s.contains('\n'), "Should contain newline, got: {:?}", s);
            }
            _ => panic!("Expected Emit(Str)"),
        }
    }

    #[test]
    fn e2e_string_escape_tab() {
        let stmts = parse("emit \"a\\tb\";").unwrap();
        match &stmts[0] {
            Stmt::Emit(Expr::Str(s)) => {
                assert!(s.contains('\t'), "Should contain tab, got: {:?}", s);
            }
            _ => panic!("Expected Emit(Str)"),
        }
    }

    #[test]
    fn e2e_string_escape_quote() {
        let stmts = parse("emit \"he said \\\"hi\\\"\";").unwrap();
        match &stmts[0] {
            Stmt::Emit(Expr::Str(s)) => {
                assert!(s.contains('"'), "Should contain quote, got: {:?}", s);
            }
            _ => panic!("Expected Emit(Str)"),
        }
    }

    #[test]
    fn e2e_method_call_push() {
        let stmts = parse("let arr = [1, 2]; emit arr.push(3);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
    }

    #[test]
    fn e2e_pipe_builtin() {
        let stmts = parse("let arr = [1, 2, 3]; emit arr |> len;").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "arr |> len = 3, got {}", v);
    }

    #[test]
    fn e2e_complex_program_fibonacci() {
        // Test a real program: compute fibonacci(7)
        let src = r#"
            let a = 0;
            let b = 1;
            for i in 0..7 {
                let temp = a + b;
                a = b;
                b = temp;
            }
            emit a;
        "#;
        let stmts = parse(src).unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        let v = outputs[0].to_number().unwrap();
        assert!((v - 13.0).abs() < f64::EPSILON, "fib(7)=13, got {}", v);
    }

    #[test]
    fn e2e_complex_program_array_ops() {
        // Build an array, push elements, get length, access element
        let src = r#"
            let arr = [10, 20, 30, 40, 50];
            emit arr.len();
            emit arr[2];
        "#;
        let stmts = parse(src).unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 2);
        let len = outputs[0].to_number().unwrap();
        let elem = outputs[1].to_number().unwrap();
        assert!((len - 5.0).abs() < f64::EPSILON, "len=5, got {}", len);
        assert!((elem - 30.0).abs() < f64::EPSILON, "arr[2]=30, got {}", elem);
    }

    #[test]
    fn e2e_complex_program_dict_ops() {
        // Create dict, access field, modify field
        let src = r#"
            let user = { name: "Leo", age: 5 };
            emit user.age;
            user.age = 6;
            emit user.age;
        "#;
        let stmts = parse(src).unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 2, "outputs: {:?}", outputs);
        let age1 = outputs[0].to_number().unwrap();
        let age2 = outputs[1].to_number().unwrap();
        assert!((age1 - 5.0).abs() < f64::EPSILON, "initial age=5, got {}", age1);
        assert!((age2 - 6.0).abs() < f64::EPSILON, "updated age=6, got {}", age2);
    }

    #[test]
    fn e2e_complex_program_conditional() {
        // if-else with nested conditions
        let src = r#"
            let x = 15;
            if x > 10 {
                if x > 20 {
                    emit 3;
                } else {
                    emit 2;
                }
            } else {
                emit 1;
            }
        "#;
        let stmts = parse(src).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        let v = outputs[0].to_number().unwrap();
        assert!((v - 2.0).abs() < f64::EPSILON, "x=15 (>10, <=20) → emit 2, got {}", v);
    }

    #[test]
    fn e2e_complex_math_expression() {
        // Test: complex numeric expression with chained operations
        let src = r#"
            let x = (3 + 4) * 2;
            let y = x - 1;
            emit y;
        "#;
        let stmts = parse(src).unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        let v = outputs[0].to_number().unwrap();
        assert!((v - 13.0).abs() < f64::EPSILON, "(3+4)*2-1=13, got {}", v);
    }

    #[test]
    fn e2e_if_expr_ternary() {
        let src = "let x = if 1 > 0 { 42 } else { 0 }; emit x;";
        let stmts = parse(src).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 42.0).abs() < f64::EPSILON, "ternary=42, got {}", v);
    }

    #[test]
    fn e2e_builtin_mapping_complete() {
        // Verify all new builtin names map correctly
        let builtins = [
            "str_split", "str_contains", "str_replace", "str_starts_with",
            "str_ends_with", "str_index_of", "str_trim", "str_upper",
            "str_lower", "str_substr", "floor", "ceil", "round", "sqrt",
            "pow", "log", "sin", "cos", "has_key", "values", "merge",
            "remove", "pop", "reverse", "contains", "join", "map",
            "filter", "isl_send", "isl_broadcast", "type_of",
            "chain_hash", "chain_len",
        ];
        for name in builtins {
            let src = alloc::format!("emit {}(1);", name);
            let stmts = parse(&src).unwrap();
            let prog = lower(&stmts);
            // Check that the builtin was mapped (should have a Call with __ prefix)
            let has_builtin = prog.ops.iter().any(|op| {
                matches!(op, Op::Call(ref n) if n.starts_with("__"))
            });
            assert!(has_builtin, "Builtin '{}' should map to __* call", name);
        }
    }

    // ── Phase 6: Real Usable Programs ───────────────────────────────────────

    #[test]
    fn e2e_string_value_as_chain() {
        // String literals are now actual chain values (not registry lookups)
        let stmts = parse(r#"emit "hello";"#).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let text = crate::vm::chain_to_string(&outputs[0]);
        assert_eq!(text, Some("hello".into()));
    }

    #[test]
    fn e2e_string_concat_works() {
        let stmts = parse(r#"let a = "hello"; let b = " world"; emit concat(a, b);"#).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let text = crate::vm::chain_to_string(&outputs[0]);
        assert_eq!(text, Some("hello world".into()));
    }

    #[test]
    fn e2e_string_len() {
        let stmts = parse(r#"emit str_len("test");"#).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let n = outputs[0].to_number().unwrap();
        assert!((n - 4.0).abs() < f64::EPSILON, "str_len('test')=4, got {}", n);
    }

    #[test]
    fn e2e_function_returns_value() {
        // fn with return expression — should leave value on stack
        let stmts = parse("fn square(x) { return x * x; } emit square(7);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1, "should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 49.0).abs() < f64::EPSILON, "square(7)=49, got {}", v);
    }

    #[test]
    fn e2e_function_implicit_return() {
        // Last expression in function body = implicit return value
        let stmts = parse("fn add(a, b) { a + b } emit add(3, 4);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1, "should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 7.0).abs() < f64::EPSILON, "add(3,4)=7, got {}", v);
    }

    #[test]
    fn e2e_multiple_function_calls_no_clash() {
        // Same function called twice in one expression — param names should not clash
        let stmts = parse("fn double(n) { return n * 2; } emit double(3) + double(4);").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1, "should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 14.0).abs() < f64::EPSILON, "double(3)+double(4)=14, got {}", v);
    }

    #[test]
    fn e2e_function_call_as_statement() {
        // Calling function as statement (not in emit) should not crash
        let stmts = parse("fn greet() { emit 42; } greet();").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let v = outputs[0].to_number().unwrap();
        assert!((v - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn e2e_real_program_fizzbuzz() {
        // FizzBuzz — a real program users would actually write
        let src = r#"
            let i = 1;
            while i < 16 {
                if mod(i, 15) == 0 {
                    emit "FizzBuzz";
                } else {
                    if mod(i, 3) == 0 {
                        emit "Fizz";
                    } else {
                        if mod(i, 5) == 0 {
                            emit "Buzz";
                        } else {
                            emit i;
                        }
                    }
                }
                i = i + 1;
            }
        "#;
        let stmts = parse(src).unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        // FizzBuzz 1-15: 1,2,Fizz,4,Buzz,Fizz,7,8,Fizz,Buzz,11,Fizz,13,14,FizzBuzz = 15 outputs
        assert_eq!(outputs.len(), 15, "FizzBuzz should produce 15 outputs, got {}", outputs.len());
        // Check specific values
        let v1 = outputs[0].to_number().unwrap();
        assert!((v1 - 1.0).abs() < f64::EPSILON, "first output should be 1");
        let fizz = crate::vm::chain_to_string(&outputs[2]);
        assert_eq!(fizz, Some("Fizz".into()), "3rd output should be Fizz");
        let buzz = crate::vm::chain_to_string(&outputs[4]);
        assert_eq!(buzz, Some("Buzz".into()), "5th output should be Buzz");
        let fizzbuzz = crate::vm::chain_to_string(&outputs[14]);
        assert_eq!(fizzbuzz, Some("FizzBuzz".into()), "15th output should be FizzBuzz");
    }

    #[test]
    fn e2e_real_program_sum_array() {
        // Sum all elements of an array — common real-world pattern
        let src = r#"
            let arr = [10, 20, 30, 40, 50];
            let total = 0;
            for i in 0..5 {
                total = total + arr[i];
            }
            emit total;
        "#;
        let stmts = parse(src).unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1);
        let total = outputs.last().unwrap().to_number().unwrap();
        assert!((total - 150.0).abs() < f64::EPSILON, "sum should be 150, got {}", total);
    }

    #[test]
    fn e2e_real_program_dict_lookup() {
        // Dictionary field access — real-world usage
        let src = r#"
            let user = {name: "Leo", age: 3, level: 1};
            emit user.name;
            emit user.age;
        "#;
        let stmts = parse(src).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 2, "should have 2 outputs, got {}", outputs.len());
        let name = crate::vm::chain_to_string(&outputs[0]);
        assert_eq!(name, Some("Leo".into()));
        let age = outputs[1].to_number().unwrap();
        assert!((age - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn e2e_chain_to_string_roundtrip() {
        // Verify string→chain→string roundtrip
        let original = "Hello, World! 123";
        let chain = crate::vm::string_to_chain(original);
        let recovered = crate::vm::chain_to_string(&chain);
        assert_eq!(recovered, Some(original.into()));
    }

    #[test]
    fn e2e_format_chain_display_number() {
        let chain = crate::molecular::MolecularChain::from_number(42.0);
        let display = crate::vm::format_chain_display(&chain);
        assert_eq!(display, "42");
    }

    #[test]
    fn e2e_format_chain_display_string() {
        let chain = crate::vm::string_to_chain("hello");
        let display = crate::vm::format_chain_display(&chain);
        assert_eq!(display, "hello");
    }

    #[test]
    fn e2e_format_chain_display_empty() {
        let chain = crate::molecular::MolecularChain::empty();
        let display = crate::vm::format_chain_display(&chain);
        assert_eq!(display, "(empty)");
    }

    #[test]
    fn e2e_emit_number_directly() {
        // Direct emit of a number — the simplest real program
        let stmts = parse("emit 42;").unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let v = outputs[0].to_number().unwrap();
        assert!((v - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn e2e_real_program_nested_functions() {
        // Functions calling other functions — real-world pattern
        let src = r#"
            fn square(x) { return x * x; }
            fn sum_of_squares(a, b) { return square(a) + square(b); }
            emit sum_of_squares(3, 4);
        "#;
        let stmts = parse(src).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1);
        let v = outputs[0].to_number().unwrap();
        assert!((v - 25.0).abs() < f64::EPSILON, "3^2 + 4^2 = 25, got {}", v);
    }

    #[test]
    fn e2e_real_program_string_processing() {
        // String processing — real-world pattern
        let src = r#"
            let greeting = "hello world";
            emit str_upper(greeting);
            emit str_contains(greeting, "world");
        "#;
        let stmts = parse(src).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 2, "should have 2 outputs, got {}", outputs.len());
        let upper = crate::vm::chain_to_string(&outputs[0]);
        assert_eq!(upper, Some("HELLO WORLD".into()));
        // str_contains returns non-empty chain (truthy) when found
        assert!(!outputs[1].is_empty(), "str_contains should return truthy for match");
    }

    #[test]
    fn e2e_real_program_for_each() {
        // for-each iteration over array
        let src = r#"
            let arr = [10, 20, 30];
            let sum = 0;
            for x in arr {
                sum = sum + x;
            }
            emit sum;
        "#;
        let stmts = parse(src).unwrap();
        let errors = validate(&stmts);
        assert!(errors.is_empty(), "validation errors: {:?}", errors);
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1);
        let sum = outputs.last().unwrap().to_number().unwrap();
        assert!((sum - 60.0).abs() < f64::EPSILON, "sum should be 60, got {}", sum);
    }

    #[test]
    fn e2e_real_program_try_catch() {
        // Error handling — real-world pattern
        let src = r#"
            try {
                emit 1 / 0;
            } catch {
                emit 999;
            }
        "#;
        let stmts = parse(src).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        // Division by zero should be caught
        let outputs = result.outputs();
        assert!(outputs.len() >= 1, "catch should produce output");
        let v = outputs.last().unwrap().to_number().unwrap();
        assert!((v - 999.0).abs() < f64::EPSILON, "catch block should emit 999");
    }

    #[test]
    fn e2e_real_program_match() {
        // Pattern matching with wildcard — real-world pattern
        let src = r#"
            let x = 42;
            match x {
                _ => { emit x; }
            }
        "#;
        let stmts = parse(src).unwrap();
        let prog = lower(&stmts);
        let vm = crate::vm::OlangVM::new();
        let result = vm.execute(&prog);
        assert!(!result.has_error(), "VM errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(outputs.len() >= 1, "wildcard match should produce output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 42.0).abs() < f64::EPSILON);
    }
}
