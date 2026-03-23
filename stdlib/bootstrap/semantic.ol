// ── Olang Bootstrap Semantic Analyzer ─────────────────────────────
// Self-hosting preparation: semantic analyzer written in Olang.
// Reads AST → validates → emits IR opcodes (OlangProgram).
//
// Phase 4 / A9 — compiler self-hosting foundation.
// Depends on: stdlib/bootstrap/lexer.ol, parser.ol
//
// Reference: crates/olang/src/lang/semantic.rs (Rust implementation)
// This only needs to compile lexer.ol + parser.ol, not 100% of Rust version.

use olang.bootstrap.lexer;
use olang.bootstrap.parser;

// Explicit save stack for recursive compile_expr (ASM VM has no scoping)
let _ce_stack = [];
let _if_stack = [];

// ── IR Opcode representation ────────────────────────────────────
// We represent opcodes as structs with an "op" tag string + args.
// The Rust VM will interpret these when we bridge.

// Op represented as 3-element array: [tag, name, value]

// Explicit save stack for recursive compile_expr (ASM VM has no scoping)

fn make_op(_tag, _name, _value) {
    return [_tag, _name, _value];
}

fn make_op_num(_tag, _value) {
    return [_tag, "", _value];
}

fn make_op_name(_tag, _name) {
    return [_tag, _name, 0];
}

fn make_op_simple(_tag) {
    return [_tag, "", 0];
}

// ── Scope tracking ──────────────────────────────────────────────

type VarEntry {
    name: Str,
    slot: Num,
}

type FnEntry {
    name: Str,
    param_count: Num,
    body_pc: Num,
    params: Vec[Str],
}

type SemanticState {
    ops: Vec[Op],
    locals: Vec[Str],
    fns: Vec[FnEntry],
    fn_bodies: Vec[FnEntry],
    errors: Vec[Str],
    call_id: Num,
    use_call_closure: Num,
    compiled_fns: Vec[FnEntry],
    types: Vec[Str],
    unions: Vec[Str],
}

fn new_state() {
    return SemanticState {
        ops: [],
        locals: [],
        fns: [],
        fn_bodies: [],
        errors: [],
        call_id: 0,
        use_call_closure: 0,
        compiled_fns: [],
        types: [],
        unions: [],
    };
}

fn emit_op(state, _op) {
    push(state.ops, _op);
}

fn current_pos(state) {
    return len(state.ops);
}

fn patch_jump(state, pos, target) {
    let _pj_old = state.ops[pos];
    set_at(state.ops, pos, [_pj_old[0], _pj_old[1], target]);
}

fn is_local(state, name) {
    let i = len(state.locals) - 1;
    while i >= 0 {
        if state.locals[i] == name {
            return true;
        };
        let i = i - 1;
    };
    return false;
}

fn push_local(state, name) {
    push(state.locals, name);
}

fn save_locals(state) {
    return len(state.locals);
}

fn restore_locals(state, saved) {
    while len(state.locals) > saved {
        pop(state.locals);
    };
}

fn declare_fn(state, name, param_count, params) {
    push(state.fns, FnEntry {
        name: name, param_count: param_count,
        body_pc: 0, params: params,
    });
}

fn lookup_fn(state, name) {
    let i = len(state.fns) - 1;
    while i >= 0 {
        if state.fns[i].name == name {
            return state.fns[i];
        };
        let i = i - 1;
    };
    // Also check compiled_fns
    let i = len(state.compiled_fns) - 1;
    while i >= 0 {
        if state.compiled_fns[i].name == name {
            return state.compiled_fns[i];
        };
        let i = i - 1;
    };
    return false;
}

fn add_error(state, msg) {
    push(state.errors, msg);
}

fn next_call_id(state) {
    let id = state.call_id;
    state.call_id = state.call_id + 1;
    return id;
}

// ── Pass 1: Collect function definitions ────────────────────────

fn collect_fns(state, stmts) {
    let i = 0;
    while i < len(stmts) {
        let stmt = stmts[i];
        match stmt {
            Stmt::FnDef { name, params, body } => {
                declare_fn(state, name, len(params), params);
                push(state.fn_bodies, FnEntry {
                    name: name, param_count: len(params),
                    body_pc: 0, params: params,
                });
            },
            Stmt::TypeDef { name, fields } => {
                push(state.types, name);
            },
            Stmt::UnionDef { name, variants } => {
                push(state.unions, name);
            },
            _ => {},
        };
        let i = i + 1;
    };
}

// ── Pass 1.5: Pre-compile function bodies (CallClosure mode) ───

fn precompile_fns(state, stmts) {
    let i = 0;
    while i < len(stmts) {
        let stmt = stmts[i];
        match stmt {
            Stmt::FnDef { name, params, body } => {
                let body_start = current_pos(state);
                // Store params (they arrive on stack from CallClosure)
                let pi = len(params) - 1;
                while pi >= 0 {
                    emit_op(state, make_op_name("Store", params[pi]));
                    push_local(state, params[pi]);
                    let pi = pi - 1;
                };
                // Compile function body
                let saved = save_locals(state);
                let bi = 0;
                while bi < len(body) {
                    compile_stmt(state, body[bi]);
                    let bi = bi + 1;
                };
                // Default return (empty)
                emit_op(state, make_op_name("Push", ""));
                emit_op(state, make_op_simple("Ret"));
                restore_locals(state, saved);
                // Register compiled function
                push(state.compiled_fns, FnEntry {
                    name: name, param_count: len(params),
                    body_pc: body_start, params: params,
                });
            },
            _ => {},
        };
        let i = i + 1;
    };
}

// ── Expression compilation ──────────────────────────────────────

fn compile_expr(state, expr) {
    match expr {
        Expr::NumLit { value } => {
            let _ce_numval = value;
            emit_op(state, make_op_num("PushNum", _ce_numval));
        },
        Expr::StrLit { value } => {
            emit_op(state, make_op_name("Push", value));
        },
        Expr::BoolLit { value } => {
            if value == 1 {
                emit_op(state, make_op_num("PushNum", 1));
            } else {
                emit_op(state, make_op_name("Push", ""));
            };
        },
        Expr::Ident { name } => {
            if name == "true" {
                emit_op(state, make_op_num("PushNum", 1));
            } else {
                if name == "false" {
                    emit_op(state, make_op_name("Push", ""));
                } else {
                    if is_local(state, name) {
                        emit_op(state, make_op_name("LoadLocal", name));
                    } else {
                        emit_op(state, make_op_name("Load", name));
                    };
                };
            };
        },
        Expr::BinOp { op, lhs, rhs } => {
            // Short-circuit for && and ||
            if op == "&&" {
                compile_expr(state, lhs);
                let jz_pos = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                emit_op(state, make_op_simple("Pop"));
                compile_expr(state, rhs);
                patch_jump(state, jz_pos, current_pos(state));
            } else {
                if op == "||" {
                    compile_expr(state, lhs);
                    let jz_pos = current_pos(state);
                    emit_op(state, make_op_num("Jz", 0));
                    let jmp_pos = current_pos(state);
                    emit_op(state, make_op_num("Jmp", 0));
                    patch_jump(state, jz_pos, current_pos(state));
                    emit_op(state, make_op_simple("Pop"));
                    compile_expr(state, rhs);
                    patch_jump(state, jmp_pos, current_pos(state));
                } else {
                    push(_ce_stack, op);
                    push(_ce_stack, rhs);
                    compile_expr(state, lhs);
                    let _bo_rhs = pop(_ce_stack);
                    compile_expr(state, _bo_rhs);
                    let _binop = pop(_ce_stack);
                    if _binop == "+" { emit_op(state, make_op_name("Call", "__hyp_add")); };
                    if _binop == "-" { emit_op(state, make_op_name("Call", "__hyp_sub")); };
                    if _binop == "*" { emit_op(state, make_op_name("Call", "__hyp_mul")); };
                    if _binop == "/" { emit_op(state, make_op_name("Call", "__hyp_div")); };
                    if _binop == "%" { emit_op(state, make_op_name("Call", "__hyp_mod")); };
                    if _binop == "==" { emit_op(state, make_op_name("Call", "__eq")); };
                    if _binop == "!=" { emit_op(state, make_op_name("Call", "__cmp_ne")); };
                    if _binop == "<" { emit_op(state, make_op_name("Call", "__cmp_lt")); };
                    if _binop == ">" { emit_op(state, make_op_name("Call", "__cmp_gt")); };
                    if _binop == "<=" { emit_op(state, make_op_name("Call", "__cmp_le")); };
                    if _binop == ">=" { emit_op(state, make_op_name("Call", "__cmp_ge")); };
                };
            };
        },
        Expr::UnaryNot { expr } => {
            compile_expr(state, expr);
            emit_op(state, make_op_name("Call", "__logic_not"));
        },
        Expr::Call { callee, args } => {
            // Check if it's a known builtin
            let _ce_fname = "";
            match callee {
                Expr::Ident { name } => { let _ce_fname = name; },
                _ => {},
            };
            // Compile args
            let ai = 0;
            while ai < len(args) {
                compile_expr(state, args[ai]);
                let ai = ai + 1;
            };
            // Dispatch: builtin, user-defined, or unknown
            if _ce_fname == "len" {
                emit_op(state, make_op_name("Call", "__array_len"));
            } else {
                if _ce_fname == "push" {
                    emit_op(state, make_op_name("Call", "__array_push"));
                } else {
                    if _ce_fname == "pop" {
                        emit_op(state, make_op_name("Call", "__array_pop"));
                    } else {
                        if _ce_fname == "char_at" {
                            emit_op(state, make_op_name("Call", "__str_char_at"));
                        } else {
                            if _ce_fname == "substr" {
                                emit_op(state, make_op_name("Call", "__str_substr"));
                            } else {
                                if _ce_fname == "to_num" {
                                    emit_op(state, make_op_name("Call", "__to_number"));
                                } else {
                                    if _ce_fname == "set_at" {
                                        emit_op(state, make_op_name("Call", "__array_set"));
                                    } else {
                                        if _ce_fname == "range" {
                                            emit_op(state, make_op_name("Call", "__array_range"));
                                        } else {
                                            // User-defined or unknown function
                                            emit_op(state, make_op_name("Call", _ce_fname));
                                        };
                                    };
                                };
                            };
                        };
                    };
                };
            };
        },
        Expr::FieldAccess { object, field } => {
            compile_expr(state, object);
            emit_op(state, make_op_name("Push", field));
            emit_op(state, make_op_name("Call", "__dict_get"));
        },
        Expr::Index { object, index } => {
            compile_expr(state, object);
            compile_expr(state, index);
            emit_op(state, make_op_name("Call", "__array_get"));
        },
        Expr::ArrayLit { items } => {
            let ai = 0;
            while ai < len(items) {
                compile_expr(state, items[ai]);
                let ai = ai + 1;
            };
            emit_op(state, make_op_num("PushNum", len(items)));
            emit_op(state, make_op_name("Call", "__array_new"));
        },
        Expr::PathExpr { base, member } => {
            // Enum variant without fields (unit variant): Base::Member
            let tag = base + "::" + member;
            emit_op(state, make_op_name("Push", tag));
            emit_op(state, make_op_name("Call", "__enum_unit"));
        },
        Expr::StructLit { path, fields } => {
            // Struct or enum variant with fields
            let fi = 0;
            while fi < len(fields) {
                let f = fields[fi];
                emit_op(state, make_op_name("Push", f.name));
                compile_expr(state, f.value);
                let fi = fi + 1;
            };
            emit_op(state, make_op_num("PushNum", len(fields)));
            emit_op(state, make_op_name("Call", "__dict_new"));
            emit_op(state, make_op_name("Push", path));
            emit_op(state, make_op_name("Call", "__struct_tag"));
        },
        Expr::IfExpr { cond, then_expr, else_expr } => {
            compile_expr(state, cond);
            let jz_pos = current_pos(state);
            emit_op(state, make_op_num("Jz", 0));
            emit_op(state, make_op_simple("Pop"));
            compile_expr(state, then_expr);
            let jmp_pos = current_pos(state);
            emit_op(state, make_op_num("Jmp", 0));
            patch_jump(state, jz_pos, current_pos(state));
            emit_op(state, make_op_simple("Pop"));
            compile_expr(state, else_expr);
            patch_jump(state, jmp_pos, current_pos(state));
        },
        Expr::MolLiteral { packed } => {
            // packed u16 [S:4][R:4][V:3][A:3][T:2] — already packed by parser
            let op = Op { tag: "PushMol", name: "", value: packed };
            emit_op(state, op);
        },
        Expr::MatchExpr { subject, arms } => {
            // Simplified match: store subject, test each arm
            compile_expr(state, subject);
            let subj_name = "__match_subj";
            emit_op(state, make_op_name("Store", subj_name));
            push_local(state, subj_name);
            let end_jumps = [];
            let ai = 0;
            while ai < len(arms) {
                let arm = arms[ai];
                if arm.pattern != "_" {
                    // Load subject, check type/tag
                    emit_op(state, make_op_name("LoadLocal", subj_name));
                    emit_op(state, make_op_name("Call", "__type_of"));
                    emit_op(state, make_op_name("Push", arm.pattern));
                    emit_op(state, make_op_name("Call", "__eq"));
                    let jz_pos = current_pos(state);
                    emit_op(state, make_op_num("Jz", 0));
                    emit_op(state, make_op_simple("Pop"));
                    // Extract bindings
                    let bi = 0;
                    while bi < len(arm.bindings) {
                        emit_op(state, make_op_name("LoadLocal", subj_name));
                        emit_op(state, make_op_name("Push", arm.bindings[bi]));
                        emit_op(state, make_op_name("Call", "__dict_get"));
                        emit_op(state, make_op_name("Store", arm.bindings[bi]));
                        push_local(state, arm.bindings[bi]);
                        let bi = bi + 1;
                    };
                    // Compile arm body
                    let si = 0;
                    while si < len(arm.body) {
                        compile_stmt(state, arm.body[si]);
                        let si = si + 1;
                    };
                    push(end_jumps, current_pos(state));
                    emit_op(state, make_op_num("Jmp", 0));
                    patch_jump(state, jz_pos, current_pos(state));
                    emit_op(state, make_op_simple("Pop"));
                } else {
                    // Wildcard: always matches
                    let si = 0;
                    while si < len(arm.body) {
                        compile_stmt(state, arm.body[si]);
                        let si = si + 1;
                    };
                    push(end_jumps, current_pos(state));
                    emit_op(state, make_op_num("Jmp", 0));
                };
                let ai = ai + 1;
            };
            // Patch all end jumps
            let ei = 0;
            while ei < len(end_jumps) {
                patch_jump(state, end_jumps[ei], current_pos(state));
                let ei = ei + 1;
            };
        },
        _ => {
            add_error(state, "Unknown expression type");
        },
    };
}

// ── Statement compilation ───────────────────────────────────────

fn compile_stmt(state, stmt) {
    match stmt {
        Stmt::LetStmt { name, value } => {
            compile_expr(state, value);
            if is_local(state, name) {
                emit_op(state, make_op_name("StoreUpdate", name));
            } else {
                emit_op(state, make_op_name("Store", name));
                push_local(state, name);
            };
        },
        Stmt::FnDef { name, params, body } => {
            // Save name/params before body compilation (body may overwrite "name")
            let _fn_name = name;
            let _fn_params = params;
            let _fn_body = body;
            let _fn_pcnt = len(_fn_params);
            // Emit Closure(param_count, body_len) + body + Store(name).
            let _fn_closure_pos = current_pos(state);
            emit_op(state, make_op("Closure", 0, _fn_pcnt));
            // Store params (reversed for stack order)
            let _fn_saved = save_locals(state);
            let _fn_pi = _fn_pcnt - 1;
            while _fn_pi >= 0 {
                emit_op(state, make_op_name("Store", _fn_params[_fn_pi]));
                push_local(state, _fn_params[_fn_pi]);
                let _fn_pi = _fn_pi - 1;
            };
            // Compile body
            let _fn_bi = 0;
            while _fn_bi < len(_fn_body) {
                compile_stmt(state, _fn_body[_fn_bi]);
                let _fn_bi = _fn_bi + 1;
            };
            // Default return
            emit_op(state, make_op_name("Push", ""));
            emit_op(state, make_op_simple("Ret"));
            restore_locals(state, _fn_saved);
            // Patch Closure body_len
            let _fn_body_len = current_pos(state) - _fn_closure_pos - 1;
            set_at(state.ops, _fn_closure_pos, make_op(
                "Closure", _fn_body_len, _fn_pcnt
            ));
            // Store closure in var_table
            emit_op(state, make_op_name("Store", _fn_name));
        },
        Stmt::ReturnStmt { value } => {
            compile_expr(state, value);
            emit_op(state, make_op_simple("Ret"));
        },
        Stmt::EmitStmt { expr } => {
            compile_expr(state, expr);
            emit_op(state, make_op_simple("Emit"));
        },
        Stmt::IfStmt { cond, then_block, else_block } => {
            // Save blocks on _if_stack (separate from _ce_stack to avoid interleave)
            push(_if_stack, then_block);
            push(_if_stack, else_block);
            compile_expr(state, cond);
            let _if_jz = current_pos(state);
            emit_op(state, make_op_num("Jz", 0));
            // Restore after compile_expr
            let _if_else = pop(_if_stack);
            let _if_then = pop(_if_stack);
            // Then block
            let _if_ti = 0;
            while _if_ti < len(_if_then) {
                push(_if_stack, _if_then);
                push(_if_stack, _if_else);
                push(_if_stack, _if_jz);
                push(_if_stack, _if_ti);
                compile_stmt(state, _if_then[_if_ti]);
                let _if_ti = pop(_if_stack);
                let _if_jz = pop(_if_stack);
                let _if_else = pop(_if_stack);
                let _if_then = pop(_if_stack);
                let _if_ti = _if_ti + 1;
            };
            if len(_if_else) > 0 {
                let _if_jmp = current_pos(state);
                emit_op(state, make_op_num("Jmp", 0));
                patch_jump(state, _if_jz, current_pos(state));
                let _if_ei = 0;
                while _if_ei < len(_if_else) {
                    push(_if_stack, _if_else);
                    push(_if_stack, _if_jmp);
                    push(_if_stack, _if_ei);
                    compile_stmt(state, _if_else[_if_ei]);
                    let _if_ei = pop(_if_stack);
                    let _if_jmp = pop(_if_stack);
                    let _if_else = pop(_if_stack);
                    let _if_ei = _if_ei + 1;
                };
                patch_jump(state, _if_jmp, current_pos(state));
            } else {
                patch_jump(state, _if_jz, current_pos(state));
            };
        },
        Stmt::WhileStmt { cond, body } => {
            emit_op(state, make_op_simple("ScopeBegin"));
            let loop_start = current_pos(state);
            compile_expr(state, cond);
            let jz_pos = current_pos(state);
            emit_op(state, make_op_num("Jz", 0));
            // Jz pops condition. No extra Pop.
            let bi = 0;
            while bi < len(body) {
                compile_stmt(state, body[bi]);
                let bi = bi + 1;
            };
            emit_op(state, make_op_simple("ScopeEnd"));
            emit_op(state, make_op_num("Jmp", loop_start));
            patch_jump(state, jz_pos, current_pos(state));
        },
        Stmt::ForStmt { var, iter, body } => {
            // Lower for-in to while loop:
            //   let __for_arr = iter;
            //   let __for_len = len(__for_arr);
            //   let __for_idx = 0;
            //   while __for_idx < __for_len {
            //       let var = __for_arr[__for_idx];
            //       body...
            //       __for_idx = __for_idx + 1;
            //   }

            // Evaluate and store iterator
            compile_expr(state, iter);
            emit_op(state, make_op_name("Store", "__for_arr"));

            // Store length: len(__for_arr)
            emit_op(state, make_op_name("Load", "__for_arr"));
            emit_op(state, make_op_name("Call", "__array_len"));
            emit_op(state, make_op_name("Store", "__for_len"));

            // Initialize index = 0
            emit_op(state, make_op_num("PushNum", 0));
            emit_op(state, make_op_name("Store", "__for_idx"));

            // Loop start: __for_idx < __for_len
            let _fl_start = current_pos(state);
            emit_op(state, make_op_name("Load", "__for_idx"));
            emit_op(state, make_op_name("Load", "__for_len"));
            emit_op(state, make_op_name("Call", "__cmp_lt"));
            let _fl_jz = current_pos(state);
            emit_op(state, make_op_num("Jz", 0));

            // let var = __for_arr[__for_idx]
            emit_op(state, make_op_name("Load", "__for_arr"));
            emit_op(state, make_op_name("Load", "__for_idx"));
            emit_op(state, make_op_name("Call", "__array_get"));
            emit_op(state, make_op_name("Store", var));

            // Compile body
            let _fl_bi = 0;
            while _fl_bi < len(body) {
                compile_stmt(state, body[_fl_bi]);
                let _fl_bi = _fl_bi + 1;
            };

            // Increment: __for_idx = __for_idx + 1
            emit_op(state, make_op_name("Load", "__for_idx"));
            emit_op(state, make_op_num("PushNum", 1));
            emit_op(state, make_op_name("Call", "__hyp_add"));
            emit_op(state, make_op_name("Store", "__for_idx"));

            // Jump back to loop start
            emit_op(state, make_op_num("Jmp", _fl_start));
            patch_jump(state, _fl_jz, current_pos(state));
        },
        Stmt::BreakStmt => {
            // Simplified: emit Jmp(0) — would need break_jumps tracking
            // For bootstrap, break inside while is uncommon
            emit_op(state, make_op_num("Jmp", 0));
        },
        Stmt::ContinueStmt => {
            // Simplified: emit Jmp(0)
            emit_op(state, make_op_num("Jmp", 0));
        },
        Stmt::TypeDef { name, fields } => {
            // Type metadata — no opcodes needed for bootstrap
        },
        Stmt::UnionDef { name, variants } => {
            // Union metadata — no opcodes needed for bootstrap
        },
        Stmt::UseStmt { path } => {
            // Module import — no opcodes needed for bootstrap
        },
        Stmt::FieldAssign { object, field, value } => {
            // obj.field = value → load obj, set field, store back
            if is_local(state, object) {
                emit_op(state, make_op_name("LoadLocal", object));
            } else {
                emit_op(state, make_op_name("Load", object));
            };
            emit_op(state, make_op_name("Push", field));
            compile_expr(state, value);
            emit_op(state, make_op_name("Call", "__dict_set"));
            emit_op(state, make_op_name("StoreUpdate", object));
        },
        Stmt::MatchStmt { subject, arms } => {
            // Match as statement: compile subject match expression, discard result
            compile_expr(state, subject);
            emit_op(state, make_op_simple("Pop"));
        },
        Stmt::ExprStmt { expr } => {
            compile_expr(state, expr);
            emit_op(state, make_op_simple("Pop"));
        },
        _ => {
            add_error(state, "Unknown statement type");
        },
    };
}

// ── Validation ──────────────────────────────────────────────────

fn validate(state) {
    // Basic validation — more can be added later
    // For bootstrap, we mainly need the compilation to succeed
}

// ── Entry point ─────────────────────────────────────────────────

pub fn analyze(ast) {
    let state = new_state();

    // Pass 1: Collect function definitions
    collect_fns(state, ast);

    // Decide compilation strategy
    if len(state.fns) > 10 {
        state.use_call_closure = 1;
        let skip_pos = current_pos(state);
        emit_op(state, make_op_num("Jmp", 0));
        precompile_fns(state, ast);
        patch_jump(state, skip_pos, current_pos(state));
    };

    // Pass 2: Compile all statements
    let _si = 0;
    while _si < len(ast) {
        compile_stmt(state, ast[_si]);
        let _si = _si + 1;
    };

    // End program
    emit_op(state, make_op_simple("Halt"));

    // Validate
    validate(state);

    return state;
}
