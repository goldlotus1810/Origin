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
let _ce_stack = __array_with_cap(512);
let _if_stack = __array_with_cap(512);
let _break_patches = [];
let _continue_patches = [];
let _g_output = [];
let _g_pos = 0;
let _g_for_depth = 0;
let __g_for_vars = ["", "", "", "", "", "", "", ""];  // max 8 nesting levels
let _g_comp_depth = 0;

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

let _g_output_ready = 0;

fn _prefill_output() {
    // Only allocate ONCE — reuse on subsequent calls
    if _g_output_ready == 0 {
        // Use __array_range to allocate exact size in ONE shot (no relocation!)
        // Values [0..16383] will be overwritten by set_at during codegen
        let _g_output = __array_range(16384);
        let _g_output_ready = 1;
    };
    // NOTE: _g_pos NOT reset here — streaming compiler accumulates.
    // Caller must reset _g_pos explicitly when starting a new compilation.
}

fn new_state() {
    _prefill_output();
    // NOTE: _g_pos NOT reset here — streaming compiler accumulates across statements
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

// Direct bytecode emission — no IR buffer, no heap corruption
fn _emit_byte(state, _eb_val) {
    set_at(_g_output, _g_pos, _eb_val);
    let _g_pos = _g_pos + 1;
}

fn _emit_u32_le(state, _eu_val) {
    _emit_byte(state, _eu_val % 256);
    _emit_byte(state, (_eu_val / 256) % 256);
    _emit_byte(state, (_eu_val / 65536) % 256);
    _emit_byte(state, (_eu_val / 16777216) % 256);
}

fn _emit_f64_le(state, _ef_val) {
    let _ef_bytes = __f64_to_le_bytes(_ef_val);
    let _ef_i = 0;
    while _ef_i < 8 {
        _emit_byte(state, _ef_bytes[_ef_i]);
        let _ef_i = _ef_i + 1;
    };
}

fn _emit_str(state, _es_str) {
    let _es_bytes = __str_bytes(_es_str);
    let _es_len = len(_es_bytes);
    _emit_byte(state, _es_len);
    let _es_i = 0;
    while _es_i < _es_len {
        _emit_byte(state, _es_bytes[_es_i]);
        let _es_i = _es_i + 1;
    };
}

fn _emit_str_u16(state, _esu_str) {
    let _esu_bytes = __str_bytes(_esu_str);
    let _esu_len = len(_esu_bytes);
    _emit_byte(state, _esu_len % 256);
    _emit_byte(state, (_esu_len / 256) % 256);
    let _esu_i = 0;
    while _esu_i < _esu_len {
        _emit_byte(state, _esu_bytes[_esu_i]);
        _emit_byte(state, 33);
        let _esu_i = _esu_i + 1;
    };
}

// Direct emit functions that DON'T use temp arrays (avoid heap overlap)
fn emit_num(state, _en_val) {
    _emit_byte(state, 21);
    _emit_f64_le(state, _en_val);
}

fn emit_load(state, _el_name) {
    let _el_len = len(_el_name);
    _emit_byte(state, 2);
    _emit_byte(state, _el_len);
    let _el_i = 0;
    while _el_i < _el_len {
        _emit_byte(state, __char_code(char_at(_el_name, _el_i)));
        let _el_i = _el_i + 1;
    };
}

fn emit_store(state, _es_name) {
    let _es_len = len(_es_name);
    _emit_byte(state, 19);
    _emit_byte(state, _es_len);
    let _es_i = 0;
    while _es_i < _es_len {
        _emit_byte(state, __char_code(char_at(_es_name, _es_i)));
        let _es_i = _es_i + 1;
    };
}

fn emit_call(state, _ec_name) {
    let _ec_len = len(_ec_name);
    _emit_byte(state, 7);
    _emit_byte(state, _ec_len);
    let _ec_i = 0;
    while _ec_i < _ec_len {
        _emit_byte(state, __char_code(char_at(_ec_name, _ec_i)));
        let _ec_i = _ec_i + 1;
    };
}

fn emit_push_str(state, _eps_name) {
    _emit_byte(state, 1);
    _emit_str_u16(state, _eps_name);
}

fn emit_simple(state, _esm_opcode) {
    _emit_byte(state, _esm_opcode);
}

fn emit_jmp(state, _ej_target) {
    _emit_byte(state, 9);
    _emit_u32_le(state, _ej_target);
}

fn emit_jz(state, _ejz_target) {
    _emit_byte(state, 10);
    _emit_u32_le(state, _ejz_target);
}

fn emit_closure(state, _ecl_pcnt) {
    _emit_byte(state, 37);
    _emit_byte(state, _ecl_pcnt);
    _emit_u32_le(state, 0);
}

// Legacy emit_op — routes to direct functions
fn emit_op(state, _op) {
    let _eo_tag = _op[0];
    let _eo_name = _op[1];
    let _eo_val = _op[2];
    if _eo_tag == "PushNum" { emit_num(state, _eo_val); return; };
    if _eo_tag == "Emit" { emit_simple(state, 6); return; };
    if _eo_tag == "Halt" { emit_simple(state, 15); return; };
    if _eo_tag == "Ret" { emit_simple(state, 8); return; };
    if _eo_tag == "Pop" { emit_simple(state, 12); return; };
    if _eo_tag == "Dup" { emit_simple(state, 11); return; };
    if _eo_tag == "Swap" { emit_simple(state, 13); return; };
    if _eo_tag == "Push" { emit_push_str(state, _eo_name); return; };
    if _eo_tag == "Load" { emit_load(state, _eo_name); return; };
    if _eo_tag == "Store" { emit_store(state, _eo_name); return; };
    if _eo_tag == "LoadLocal" { emit_load(state, _eo_name); return; };
    if _eo_tag == "Call" { emit_call(state, _eo_name); return; };
    if _eo_tag == "Jmp" { emit_jmp(state, _eo_val); return; };
    if _eo_tag == "Jz" { emit_jz(state, _eo_val); return; };
    if _eo_tag == "Closure" { emit_closure(state, _eo_val); return; };
}

fn current_pos(state) {
    return _g_pos;
}

fn patch_jump(state, pos, target) {
    // Patch 4-byte LE u32 at pos+1 (after opcode byte)
    let _pj_pos = pos + 1;
    set_at(_g_output, _pj_pos, target % 256);
    set_at(_g_output, _pj_pos + 1, (target / 256) % 256);
    set_at(_g_output, _pj_pos + 2, (target / 65536) % 256);
    set_at(_g_output, _pj_pos + 3, (target / 16777216) % 256);
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
                emit_num(state, 1);
            } else {
                if name == "false" {
                    emit_push_str(state, "");
                } else {
                    emit_load(state, name);
                };
            };
        },
        Expr::BinOp { op, lhs, rhs } => {
            // Short-circuit for && and ||
            if op == "&&" {
                // Short-circuit: Dup → Jz(end) → Pop → rhs → end
                compile_expr(state, lhs);
                emit_op(state, make_op_simple("Dup"));
                let jz_pos = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                emit_op(state, make_op_simple("Pop"));
                compile_expr(state, rhs);
                patch_jump(state, jz_pos, current_pos(state));
            } else {
                if op == "||" {
                    // Short-circuit: Dup → Jz(false) → Jmp(end) → false: Pop → rhs → end
                    compile_expr(state, lhs);
                    emit_op(state, make_op_simple("Dup"));
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
                    if _binop == "<<" { emit_op(state, make_op_name("Call", "__bit_shl")); };
                    if _binop == ">>" { emit_op(state, make_op_name("Call", "__bit_shr")); };
                    if _binop == "|" { emit_op(state, make_op_name("Call", "__bit_or")); };
                    if _binop == "^" { emit_op(state, make_op_name("Call", "__bit_xor")); };
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
            // ── Inline higher-order builtins: map, filter, reduce ──
            if _ce_fname == "map" && len(args) == 2 {
                // map(arr, f) → inline loop: result=[], for i in arr { push(result, f(arr[i])) }
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__ma"));
                compile_expr(state, args[1]);
                emit_op(state, make_op_name("Store", "__mf"));
                // result = []
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Call", "__array_new"));
                emit_op(state, make_op_name("Store", "__mr"));
                // i = 0
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Store", "__mi"));
                // loop: if i >= len(arr) → exit
                let _mp_loop = current_pos(state);
                emit_op(state, make_op_name("Load", "__mi"));
                emit_op(state, make_op_name("Load", "__ma"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _mp_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // push(result, f(arr[i]))
                emit_op(state, make_op_name("Load", "__mr"));
                emit_op(state, make_op_name("Load", "__ma"));
                emit_op(state, make_op_name("Load", "__mi"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Call", "__mf"));
                emit_op(state, make_op_name("Call", "__array_push"));
                emit_op(state, make_op_name("Store", "__mr"));
                // i = i + 1
                emit_op(state, make_op_name("Load", "__mi"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__mi"));
                emit_jmp(state, _mp_loop);
                patch_jump(state, _mp_jz, current_pos(state));
                emit_op(state, make_op_name("Load", "__mr"));
                return;
            };
            if _ce_fname == "filter" && len(args) == 2 {
                // filter(arr, f) → inline loop: result=[], for x in arr { if f(x) push(result, x) }
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__fa"));
                compile_expr(state, args[1]);
                emit_op(state, make_op_name("Store", "__ff"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Call", "__array_new"));
                emit_op(state, make_op_name("Store", "__fr"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Store", "__fi"));
                let _fp_loop = current_pos(state);
                emit_op(state, make_op_name("Load", "__fi"));
                emit_op(state, make_op_name("Load", "__fa"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _fp_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // x = arr[i]
                emit_op(state, make_op_name("Load", "__fa"));
                emit_op(state, make_op_name("Load", "__fi"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Store", "__fx"));
                // if f(x) { push(result, x) }
                emit_op(state, make_op_name("Load", "__fx"));
                emit_op(state, make_op_name("Call", "__ff"));
                let _fp_skip = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                emit_op(state, make_op_name("Load", "__fr"));
                emit_op(state, make_op_name("Load", "__fx"));
                emit_op(state, make_op_name("Call", "__array_push"));
                emit_op(state, make_op_name("Store", "__fr"));
                patch_jump(state, _fp_skip, current_pos(state));
                // i++
                emit_op(state, make_op_name("Load", "__fi"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__fi"));
                emit_jmp(state, _fp_loop);
                patch_jump(state, _fp_jz, current_pos(state));
                emit_op(state, make_op_name("Load", "__fr"));
                return;
            };
            if _ce_fname == "reduce" && len(args) == 2 {
                // reduce(arr, f) → acc=arr[0], for i=1..len { acc=f(acc,arr[i]) }
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__ra"));
                compile_expr(state, args[1]);
                emit_op(state, make_op_name("Store", "__rf"));
                // acc = arr[0]
                emit_op(state, make_op_name("Load", "__ra"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Store", "__rc"));
                // i = 1
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Store", "__ri"));
                let _rp_loop = current_pos(state);
                emit_op(state, make_op_name("Load", "__ri"));
                emit_op(state, make_op_name("Load", "__ra"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _rp_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // acc = f(acc, arr[i])
                emit_op(state, make_op_name("Load", "__rc"));
                emit_op(state, make_op_name("Load", "__ra"));
                emit_op(state, make_op_name("Load", "__ri"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Call", "__rf"));
                emit_op(state, make_op_name("Store", "__rc"));
                // i++
                emit_op(state, make_op_name("Load", "__ri"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__ri"));
                emit_jmp(state, _rp_loop);
                patch_jump(state, _rp_jz, current_pos(state));
                emit_op(state, make_op_name("Load", "__rc"));
                return;
            };
            if _ce_fname == "reduce" && len(args) == 3 {
                // reduce(arr, f, init) → acc=init, for i=0..len { acc=f(acc,arr[i]) }
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__ra"));
                compile_expr(state, args[1]);
                emit_op(state, make_op_name("Store", "__rf"));
                compile_expr(state, args[2]);
                emit_op(state, make_op_name("Store", "__rc"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Store", "__ri"));
                let _rp3_loop = current_pos(state);
                emit_op(state, make_op_name("Load", "__ri"));
                emit_op(state, make_op_name("Load", "__ra"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _rp3_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                emit_op(state, make_op_name("Load", "__rc"));
                emit_op(state, make_op_name("Load", "__ra"));
                emit_op(state, make_op_name("Load", "__ri"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Call", "__rf"));
                emit_op(state, make_op_name("Store", "__rc"));
                emit_op(state, make_op_name("Load", "__ri"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__ri"));
                emit_jmp(state, _rp3_loop);
                patch_jump(state, _rp3_jz, current_pos(state));
                emit_op(state, make_op_name("Load", "__rc"));
                return;
            };
            if _ce_fname == "pipe" && len(args) >= 2 {
                // pipe(x, f1, f2, ..., fn) → fn(...f2(f1(x)))
                // The Lego operator: chain functions together
                // First arg = initial value, rest = functions to apply
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__pp_val"));
                let _pp_i = 1;
                while _pp_i < len(args) {
                    push(_ce_stack, _pp_i);
                    compile_expr(state, args[_pp_i]);
                    let _pp_i = pop(_ce_stack);
                    emit_op(state, make_op_name("Store", "__pp_fn"));
                    emit_op(state, make_op_name("Load", "__pp_val"));
                    emit_op(state, make_op_name("Call", "__pp_fn"));
                    emit_op(state, make_op_name("Store", "__pp_val"));
                    let _pp_i = _pp_i + 1;
                };
                emit_op(state, make_op_name("Load", "__pp_val"));
                return;
            };
            if _ce_fname == "any" && len(args) == 2 {
                // any(arr, f) → true if f(x) for some x in arr
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__ya"));
                compile_expr(state, args[1]);
                emit_op(state, make_op_name("Store", "__yf"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Store", "__yi"));
                let _yp_loop = current_pos(state);
                emit_op(state, make_op_name("Load", "__yi"));
                emit_op(state, make_op_name("Load", "__ya"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _yp_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                emit_op(state, make_op_name("Load", "__ya"));
                emit_op(state, make_op_name("Load", "__yi"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Call", "__yf"));
                let _yp_false = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // found true → return 1
                emit_op(state, make_op_num("PushNum", 1));
                let _yp_done = current_pos(state);
                emit_op(state, make_op_num("Jmp", 0));
                patch_jump(state, _yp_false, current_pos(state));
                // i++
                emit_op(state, make_op_name("Load", "__yi"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__yi"));
                emit_jmp(state, _yp_loop);
                // not found → return 0
                patch_jump(state, _yp_jz, current_pos(state));
                emit_op(state, make_op_num("PushNum", 0));
                patch_jump(state, _yp_done, current_pos(state));
                return;
            };
            if _ce_fname == "all" && len(args) == 2 {
                // all(arr, f) → true if f(x) for all x in arr
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__la"));
                compile_expr(state, args[1]);
                emit_op(state, make_op_name("Store", "__lf"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Store", "__li"));
                let _lp_loop = current_pos(state);
                emit_op(state, make_op_name("Load", "__li"));
                emit_op(state, make_op_name("Load", "__la"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _lp_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                emit_op(state, make_op_name("Load", "__la"));
                emit_op(state, make_op_name("Load", "__li"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Call", "__lf"));
                let _lp_true = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // still true → i++
                emit_op(state, make_op_name("Load", "__li"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__li"));
                emit_jmp(state, _lp_loop);
                // found false → return 0
                patch_jump(state, _lp_true, current_pos(state));
                emit_op(state, make_op_num("PushNum", 0));
                let _lp_done = current_pos(state);
                emit_op(state, make_op_num("Jmp", 0));
                // all passed → return 1
                patch_jump(state, _lp_jz, current_pos(state));
                emit_op(state, make_op_num("PushNum", 1));
                patch_jump(state, _lp_done, current_pos(state));
                return;
            };
            if _ce_fname == "join" && len(args) == 2 {
                // join(arr, sep) → concatenate array elements with separator
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__jn_a"));
                compile_expr(state, args[1]);
                emit_op(state, make_op_name("Store", "__jn_sep"));
                emit_op(state, make_op_name("Push", ""));
                emit_op(state, make_op_name("Store", "__jn_r"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Store", "__jn_i"));
                let _jn_loop = current_pos(state);
                emit_op(state, make_op_name("Load", "__jn_i"));
                emit_op(state, make_op_name("Load", "__jn_a"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _jn_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // if i > 0: r = r + sep
                emit_op(state, make_op_name("Load", "__jn_i"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Call", "__cmp_gt"));
                let _jn_nosep = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                emit_op(state, make_op_name("Load", "__jn_r"));
                emit_op(state, make_op_name("Load", "__jn_sep"));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__jn_r"));
                patch_jump(state, _jn_nosep, current_pos(state));
                // r = r + __to_string(arr[i])
                emit_op(state, make_op_name("Load", "__jn_r"));
                emit_op(state, make_op_name("Load", "__jn_a"));
                emit_op(state, make_op_name("Load", "__jn_i"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__jn_r"));
                // i++
                emit_op(state, make_op_name("Load", "__jn_i"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__jn_i"));
                emit_jmp(state, _jn_loop);
                patch_jump(state, _jn_jz, current_pos(state));
                emit_op(state, make_op_name("Load", "__jn_r"));
                return;
            };
            if _ce_fname == "contains" && len(args) == 2 {
                // contains(str, substr) → 1 if found, 0 if not
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__cn_s"));
                compile_expr(state, args[1]);
                emit_op(state, make_op_name("Store", "__cn_sub"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Store", "__cn_i"));
                // slen - sublen + 1 = max start
                emit_op(state, make_op_name("Load", "__cn_s"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Load", "__cn_sub"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__hyp_sub"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__cn_max"));
                let _cn_loop = current_pos(state);
                emit_op(state, make_op_name("Load", "__cn_i"));
                emit_op(state, make_op_name("Load", "__cn_max"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _cn_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // check substr match at position i
                emit_op(state, make_op_name("Load", "__cn_s"));
                emit_op(state, make_op_name("Load", "__cn_i"));
                emit_op(state, make_op_name("Load", "__cn_i"));
                emit_op(state, make_op_name("Load", "__cn_sub"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Call", "__str_substr"));
                emit_op(state, make_op_name("Load", "__cn_sub"));
                emit_op(state, make_op_name("Call", "__eq"));
                let _cn_nomatch = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // found!
                emit_op(state, make_op_num("PushNum", 1));
                let _cn_done = current_pos(state);
                emit_op(state, make_op_num("Jmp", 0));
                patch_jump(state, _cn_nomatch, current_pos(state));
                // i++
                emit_op(state, make_op_name("Load", "__cn_i"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__cn_i"));
                emit_jmp(state, _cn_loop);
                // not found
                patch_jump(state, _cn_jz, current_pos(state));
                emit_op(state, make_op_num("PushNum", 0));
                patch_jump(state, _cn_done, current_pos(state));
                return;
            };
            if _ce_fname == "sort" && len(args) == 1 {
                // sort(arr) → new sorted array (insertion sort, in-place on copy)
                // Copy arr → __sa, then insertion sort __sa
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__sa_src"));
                // result = []; copy all elements
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Call", "__array_new"));
                emit_op(state, make_op_name("Store", "__sa"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Store", "__sa_ci"));
                // copy loop
                let _sa_copy = current_pos(state);
                emit_op(state, make_op_name("Load", "__sa_ci"));
                emit_op(state, make_op_name("Load", "__sa_src"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _sa_copy_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                emit_op(state, make_op_name("Load", "__sa"));
                emit_op(state, make_op_name("Load", "__sa_src"));
                emit_op(state, make_op_name("Load", "__sa_ci"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Call", "__array_push"));
                emit_op(state, make_op_name("Store", "__sa"));
                emit_op(state, make_op_name("Load", "__sa_ci"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__sa_ci"));
                emit_jmp(state, _sa_copy);
                patch_jump(state, _sa_copy_jz, current_pos(state));
                // Insertion sort: i=1..len, key=a[i], j=i-1, while j>=0 && a[j]>key: a[j+1]=a[j], j--; a[j+1]=key
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Store", "__sa_i"));
                let _sa_outer = current_pos(state);
                emit_op(state, make_op_name("Load", "__sa_i"));
                emit_op(state, make_op_name("Load", "__sa"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _sa_outer_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // key = a[i]
                emit_op(state, make_op_name("Load", "__sa"));
                emit_op(state, make_op_name("Load", "__sa_i"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Store", "__sa_key"));
                // j = i - 1
                emit_op(state, make_op_name("Load", "__sa_i"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_sub"));
                emit_op(state, make_op_name("Store", "__sa_j"));
                // inner loop: while j >= 0 && a[j] > key
                let _sa_inner = current_pos(state);
                emit_op(state, make_op_name("Load", "__sa_j"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Call", "__cmp_ge"));
                let _sa_inner_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                emit_op(state, make_op_name("Load", "__sa"));
                emit_op(state, make_op_name("Load", "__sa_j"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Load", "__sa_key"));
                emit_op(state, make_op_name("Call", "__cmp_gt"));
                let _sa_noswap = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // a[j+1] = a[j]
                emit_op(state, make_op_name("Load", "__sa"));
                emit_op(state, make_op_name("Load", "__sa_j"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Load", "__sa"));
                emit_op(state, make_op_name("Load", "__sa_j"));
                emit_op(state, make_op_name("Call", "__array_get"));
                emit_op(state, make_op_name("Call", "__array_set"));
                emit_op(state, make_op_simple("Pop"));
                // j--
                emit_op(state, make_op_name("Load", "__sa_j"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_sub"));
                emit_op(state, make_op_name("Store", "__sa_j"));
                emit_jmp(state, _sa_inner);
                patch_jump(state, _sa_inner_jz, current_pos(state));
                patch_jump(state, _sa_noswap, current_pos(state));
                // a[j+1] = key
                emit_op(state, make_op_name("Load", "__sa"));
                emit_op(state, make_op_name("Load", "__sa_j"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Load", "__sa_key"));
                emit_op(state, make_op_name("Call", "__array_set"));
                emit_op(state, make_op_simple("Pop"));
                // i++
                emit_op(state, make_op_name("Load", "__sa_i"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__sa_i"));
                emit_jmp(state, _sa_outer);
                patch_jump(state, _sa_outer_jz, current_pos(state));
                emit_op(state, make_op_name("Load", "__sa"));
                return;
            };
            if _ce_fname == "split" && len(args) == 2 {
                // split(str, sep) → array of strings
                // Only supports single-char separator
                compile_expr(state, args[0]);
                emit_op(state, make_op_name("Store", "__sp_str"));
                compile_expr(state, args[1]);
                emit_op(state, make_op_name("Store", "__sp_sep"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Call", "__array_new"));
                emit_op(state, make_op_name("Store", "__sp_r"));
                emit_op(state, make_op_name("Push", ""));
                emit_op(state, make_op_name("Store", "__sp_cur"));
                emit_op(state, make_op_num("PushNum", 0));
                emit_op(state, make_op_name("Store", "__sp_i"));
                let _sp_loop = current_pos(state);
                emit_op(state, make_op_name("Load", "__sp_i"));
                emit_op(state, make_op_name("Load", "__sp_str"));
                emit_op(state, make_op_name("Call", "__array_len"));
                emit_op(state, make_op_name("Call", "__cmp_lt"));
                let _sp_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // ch = str[i]
                emit_op(state, make_op_name("Load", "__sp_str"));
                emit_op(state, make_op_name("Load", "__sp_i"));
                emit_op(state, make_op_name("Call", "__str_char_at"));
                emit_op(state, make_op_name("Store", "__sp_ch"));
                // if ch == sep → push cur, reset
                emit_op(state, make_op_name("Load", "__sp_ch"));
                emit_op(state, make_op_name("Load", "__sp_sep"));
                emit_op(state, make_op_name("Call", "__eq"));
                let _sp_nosplit = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                // split: push cur to result
                emit_op(state, make_op_name("Load", "__sp_r"));
                emit_op(state, make_op_name("Load", "__sp_cur"));
                emit_op(state, make_op_name("Call", "__array_push"));
                emit_op(state, make_op_name("Store", "__sp_r"));
                emit_op(state, make_op_name("Push", ""));
                emit_op(state, make_op_name("Store", "__sp_cur"));
                let _sp_cont = current_pos(state);
                emit_op(state, make_op_num("Jmp", 0));
                patch_jump(state, _sp_nosplit, current_pos(state));
                // no split: cur = cur + ch
                emit_op(state, make_op_name("Load", "__sp_cur"));
                emit_op(state, make_op_name("Load", "__sp_ch"));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__sp_cur"));
                patch_jump(state, _sp_cont, current_pos(state));
                // i++
                emit_op(state, make_op_name("Load", "__sp_i"));
                emit_op(state, make_op_num("PushNum", 1));
                emit_op(state, make_op_name("Call", "__hyp_add"));
                emit_op(state, make_op_name("Store", "__sp_i"));
                emit_jmp(state, _sp_loop);
                patch_jump(state, _sp_jz, current_pos(state));
                // push last segment
                emit_op(state, make_op_name("Load", "__sp_r"));
                emit_op(state, make_op_name("Load", "__sp_cur"));
                emit_op(state, make_op_name("Call", "__array_push"));
                emit_op(state, make_op_name("Store", "__sp_r"));
                emit_op(state, make_op_name("Load", "__sp_r"));
                return;
            };
            // enumerate/zip deferred — nested array creation in loops triggers heap overlap
            // Workaround: use for i in range(len(arr)) with arr[i], or manual loops
            // Save fname+args before compiling (inner Call overwrites them!)
            let _ce_saved_fname = _ce_fname;
            let _ce_saved_args = args;
            // Compile args
            let _ce_ai = 0;
            while _ce_ai < len(_ce_saved_args) {
                push(_ce_stack, _ce_saved_fname);
                push(_ce_stack, _ce_saved_args);
                push(_ce_stack, _ce_ai);
                compile_expr(state, _ce_saved_args[_ce_ai]);
                let _ce_ai = pop(_ce_stack);
                let _ce_saved_args = pop(_ce_stack);
                let _ce_saved_fname = pop(_ce_stack);
                let _ce_ai = _ce_ai + 1;
            };
            let _ce_fname = _ce_saved_fname;
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
            emit_call(state, "__array_get");
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
            // Save fields/fi before compile_expr (recursive may clobber)
            let fi = 0;
            let _sl_count = len(fields);
            while fi < _sl_count {
                let f = fields[fi];
                let _sl_fname = f.name;
                let _sl_fval = f.value;
                emit_op(state, make_op_name("Push", _sl_fname));
                push(_ce_stack, fields);
                push(_ce_stack, fi);
                push(_ce_stack, _sl_count);
                push(_ce_stack, path);
                compile_expr(state, _sl_fval);
                let path = pop(_ce_stack);
                let _sl_count = pop(_ce_stack);
                let fi = pop(_ce_stack);
                let fields = pop(_ce_stack);
                let fi = fi + 1;
            };
            emit_op(state, make_op_num("PushNum", _sl_count));
            emit_op(state, make_op_name("Call", "__dict_new"));
            emit_op(state, make_op_name("Push", path));
            emit_op(state, make_op_name("Call", "__struct_tag"));
        },
        Expr::DictLit { fields } => {
            // Dict literal: { key: value, ... } — no tag
            // Save fields before compile_expr (recursive may clobber _dl_i/_dl_f)
            let _dl_i = 0;
            let _dl_count = len(fields);
            while _dl_i < _dl_count {
                let _dl_f = fields[_dl_i];
                let _dl_fname = _dl_f.name;
                let _dl_fval = _dl_f.value;
                emit_op(state, make_op_name("Push", _dl_fname));
                push(_ce_stack, fields);
                push(_ce_stack, _dl_i);
                push(_ce_stack, _dl_count);
                compile_expr(state, _dl_fval);
                let _dl_count = pop(_ce_stack);
                let _dl_i = pop(_ce_stack);
                let fields = pop(_ce_stack);
                let _dl_i = _dl_i + 1;
            };
            emit_op(state, make_op_num("PushNum", _dl_count));
            emit_op(state, make_op_name("Call", "__dict_new"));
        },
        Expr::ArrayComp { var, depth } => {
            // ArrayComp: [expr for var in iter if filter]
            // [expr for var in iter if filter]
            // Read depth-indexed globals (set by parser)
            let _cc_var = "";
            let _cc_es = 0; let _cc_ee = 0;
            let _cc_is = 0; let _cc_ie = 0;
            let _cc_fs = -1; let _cc_fe = -1;
            if depth == 0 {
                let _cc_var = __g_cv0;
                let _cc_es = __g_ce0s; let _cc_ee = __g_ce0e;
                let _cc_is = __g_ci0s; let _cc_ie = __g_ci0e;
                let _cc_fs = __g_cf0s; let _cc_fe = __g_cf0e;
            };
            if depth == 1 {
                let _cc_var = __g_cv1;
                let _cc_es = __g_ce1s; let _cc_ee = __g_ce1e;
                let _cc_is = __g_ci1s; let _cc_ie = __g_ci1e;
                let _cc_fs = __g_cf1s; let _cc_fe = __g_cf1e;
            };
            if depth == 2 {
                let _cc_var = __g_cv2;
                let _cc_es = __g_ce2s; let _cc_ee = __g_ce2e;
                let _cc_is = __g_ci2s; let _cc_ie = __g_ci2e;
                let _cc_fs = __g_cf2s; let _cc_fe = __g_cf2e;
            };
            if depth == 3 {
                let _cc_var = __g_cv3;
                let _cc_es = __g_ce3s; let _cc_ee = __g_ce3e;
                let _cc_is = __g_ci3s; let _cc_ie = __g_ci3e;
                let _cc_fs = __g_cf3s; let _cc_fe = __g_cf3e;
            };

            // Unique runtime var names
            let _cc_d = __to_string(depth);
            let _cc_result = "__comp_" + _cc_d + "_r";
            let _cc_arr = "__comp_" + _cc_d + "_a";
            let _cc_len = "__comp_" + _cc_d + "_l";
            let _cc_idx = "__comp_" + _cc_d + "_i";

            // Empty result array
            emit_op(state, make_op_num("PushNum", 0));
            emit_op(state, make_op_name("Call", "__array_new"));
            emit_op(state, make_op_name("Store", _cc_result));

            // Compile iter (re-parse from tokens)
            let _cc_p = new_parser(__g_comp_tokens);
            _cc_p.pos = _cc_is;
            let _cc_iter_ast = parse_expr(_cc_p);
            compile_expr(state, _cc_iter_ast);
            emit_op(state, make_op_name("Store", _cc_arr));

            // len
            emit_op(state, make_op_name("Load", _cc_arr));
            emit_op(state, make_op_name("Call", "__array_len"));
            emit_op(state, make_op_name("Store", _cc_len));

            // idx = 0
            emit_op(state, make_op_num("PushNum", 0));
            emit_op(state, make_op_name("Store", _cc_idx));

            // Loop condition
            let _cc_loop = current_pos(state);
            emit_op(state, make_op_name("Load", _cc_idx));
            emit_op(state, make_op_name("Load", _cc_len));
            emit_op(state, make_op_name("Call", "__cmp_lt"));
            let _cc_exit_jz = current_pos(state);
            emit_op(state, make_op_num("Jz", 0));

            // var = arr[idx]
            emit_op(state, make_op_name("Load", _cc_arr));
            emit_op(state, make_op_name("Load", _cc_idx));
            emit_op(state, make_op_name("Call", "__array_get"));
            emit_op(state, make_op_name("Store", _cc_var));

            // Pre-emit increment (before body — same pattern as for-in fix)
            let _cc_body_jmp = current_pos(state);
            emit_jmp(state, 0);
            let _cc_inc = current_pos(state);
            emit_op(state, make_op_name("Load", _cc_idx));
            emit_op(state, make_op_num("PushNum", 1));
            emit_op(state, make_op_name("Call", "__hyp_add"));
            emit_op(state, make_op_name("Store", _cc_idx));
            emit_jmp(state, _cc_loop);
            let _cc_body_start = current_pos(state);
            patch_jump(state, _cc_body_jmp, _cc_body_start);

            // Save inc/exit_jz to depth-indexed globals
            if depth == 0 { let __g_cc_inc0 = _cc_inc; let __g_cc_jz0 = _cc_exit_jz; };
            if depth == 1 { let __g_cc_inc1 = _cc_inc; let __g_cc_jz1 = _cc_exit_jz; };
            if depth == 2 { let __g_cc_inc2 = _cc_inc; let __g_cc_jz2 = _cc_exit_jz; };
            if depth == 3 { let __g_cc_inc3 = _cc_inc; let __g_cc_jz3 = _cc_exit_jz; };

            // Optional filter (re-parse from tokens)
            if _cc_fs >= 0 {
                let _cc_fp = new_parser(__g_comp_tokens);
                _cc_fp.pos = _cc_fs;
                let _cc_filter_ast = parse_expr(_cc_fp);
                compile_expr(state, _cc_filter_ast);
                let _cc_filter_jz = current_pos(state);
                emit_op(state, make_op_num("Jz", 0));
                if depth == 0 { let __g_cc_fjz0 = _cc_filter_jz; };
                if depth == 1 { let __g_cc_fjz1 = _cc_filter_jz; };
                if depth == 2 { let __g_cc_fjz2 = _cc_filter_jz; };
                if depth == 3 { let __g_cc_fjz3 = _cc_filter_jz; };
            };

            // Emit expr bytecode MANUALLY from token info in globals
            // NO compile_expr, NO parse_expr — pure emit_op calls
            // Supports: single token (Load var / PushNum), binary op (lhs op rhs)
            emit_op(state, make_op_name("Load", _cc_result));
            // Read expr tokens from globals (saved by parser)
            let _cc_expr_ntoks = _cc_ee - _cc_es;
            // expr_ntoks determines emit strategy: 1=identity, 3=binop, 4=fn(arg)
            if _cc_expr_ntoks == 1 {
                // Single token: just Load var
                emit_op(state, make_op_name("Load", _cc_var));
            };
            if _cc_expr_ntoks == 3 {
                // lhs op rhs — read from token array directly
                // Token at _cc_es = lhs, _cc_es+1 = op, _cc_es+2 = rhs
                let _cc_t0 = __g_comp_tokens[_cc_es];
                let _cc_t1 = __g_comp_tokens[_cc_es + 1];
                let _cc_t2 = __g_comp_tokens[_cc_es + 2];
                // Emit lhs
                match _cc_t0.kind {
                    TokenKind::Ident { name } => { emit_op(state, make_op_name("Load", name)); },
                    TokenKind::Number { value } => { emit_op(state, make_op_num("PushNum", value)); },
                    _ => {},
                };
                // Emit rhs
                match _cc_t2.kind {
                    TokenKind::Ident { name } => { emit_op(state, make_op_name("Load", name)); },
                    TokenKind::Number { value } => { emit_op(state, make_op_num("PushNum", value)); },
                    _ => {},
                };
                // Emit op
                match _cc_t1.kind {
                    TokenKind::Symbol { ch } => {
                        if ch == "+" { emit_op(state, make_op_name("Call", "__hyp_add")); };
                        if ch == "-" { emit_op(state, make_op_name("Call", "__hyp_sub")); };
                        if ch == "*" { emit_op(state, make_op_name("Call", "__hyp_mul")); };
                        if ch == "/" { emit_op(state, make_op_name("Call", "__hyp_div")); };
                        if ch == "%" { emit_op(state, make_op_name("Call", "__hyp_mod")); };
                    },
                    _ => {},
                };
            };
            if _cc_expr_ntoks == 4 {
                // fn(arg) pattern: id ( arg )
                let _cc_t0 = __g_comp_tokens[_cc_es];
                let _cc_t2 = __g_comp_tokens[_cc_es + 2];
                match _cc_t2.kind {
                    TokenKind::Ident { name } => { emit_op(state, make_op_name("Load", name)); },
                    TokenKind::Number { value } => { emit_op(state, make_op_num("PushNum", value)); },
                    _ => {},
                };
                match _cc_t0.kind {
                    TokenKind::Ident { name } => { emit_op(state, make_op_name("Call", name)); },
                    _ => {},
                };
            };
            emit_op(state, make_op_name("Call", "__array_push"));
            emit_op(state, make_op_name("Store", _cc_result));

            // Patch filter skip
            if _cc_fs >= 0 {
                let _cc_fjz_val = -1;
                if depth == 0 { let _cc_fjz_val = __g_cc_fjz0; };
                if depth == 1 { let _cc_fjz_val = __g_cc_fjz1; };
                if depth == 2 { let _cc_fjz_val = __g_cc_fjz2; };
                if depth == 3 { let _cc_fjz_val = __g_cc_fjz3; };
                patch_jump(state, _cc_fjz_val, current_pos(state));
            };

            // Jump to increment (restore from globals)
            let _cc_inc_r = 0; let _cc_exit_jz_r = 0;
            if depth == 0 { let _cc_inc_r = __g_cc_inc0; let _cc_exit_jz_r = __g_cc_jz0; };
            if depth == 1 { let _cc_inc_r = __g_cc_inc1; let _cc_exit_jz_r = __g_cc_jz1; };
            if depth == 2 { let _cc_inc_r = __g_cc_inc2; let _cc_exit_jz_r = __g_cc_jz2; };
            if depth == 3 { let _cc_inc_r = __g_cc_inc3; let _cc_exit_jz_r = __g_cc_jz3; };
            emit_jmp(state, _cc_inc_r);

            // Exit
            patch_jump(state, _cc_exit_jz_r, current_pos(state));
            emit_op(state, make_op_name("Load", _cc_result));
            let _g_comp_depth = _g_comp_depth - 1;
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
            let __g_mej0 = -1; let __g_mej1 = -1; let __g_mej2 = -1; let __g_mej3 = -1;
            let ai = 0;
            let _m_num_arms = len(arms);
            while ai < _m_num_arms {
                // Read pattern + body token range from GLOBALS (dict fields corrupt)
                let _m_pattern = "";
                let _m_body_s = 0;
                let _m_body_e = 0;
                if ai == 0 { let _m_pattern = __g_ma0_pat; let _m_body_s = __g_ma0_bs; let _m_body_e = __g_ma0_be; };
                if ai == 1 { let _m_pattern = __g_ma1_pat; let _m_body_s = __g_ma1_bs; let _m_body_e = __g_ma1_be; };
                if ai == 2 { let _m_pattern = __g_ma2_pat; let _m_body_s = __g_ma2_bs; let _m_body_e = __g_ma2_be; };
                if ai == 3 { let _m_pattern = __g_ma3_pat; let _m_body_s = __g_ma3_bs; let _m_body_e = __g_ma3_be; };
                let _m_bindings = _m_bindings;
                let _m_body = _m_body;
                if _m_pattern != "_" {
                    // Load subject and compare
                    emit_op(state, make_op_name("LoadLocal", subj_name));
                    let _mp = _m_pattern;
                    let _mp_is_num = 0;
                    let _mp_is_str = 0;
                    if len(_mp) > 6 {
                        if char_at(_mp, 0) == "_" {
                            if char_at(_mp, 1) == "_" {
                                if char_at(_mp, 5) == ":" {
                                    if char_at(_mp, 2) == "n" { let _mp_is_num = 1; };
                                    if char_at(_mp, 2) == "s" { let _mp_is_str = 1; };
                                };
                            };
                        };
                    };
                    if _mp_is_num == 1 {
                        // Number pattern: compare subject == number
                        let _mp_numstr = __substr(_mp, 6, len(_mp));
                        emit_op(state, make_op_num("PushNum", __to_number(_mp_numstr)));
                        emit_op(state, make_op_name("Call", "__eq"));
                    } else {
                        if _mp_is_str == 1 {
                            // String pattern: compare subject == string
                            let _mp_strval = __substr(_mp, 6, len(_mp));
                            emit_push_str(state, _mp_strval);
                            emit_op(state, make_op_name("Call", "__eq"));
                        } else {
                            // Enum/struct pattern: compare type tag
                            emit_op(state, make_op_name("Push", _mp));
                            emit_op(state, make_op_name("Call", "__match_enum"));
                        };
                    };
                    let jz_pos = current_pos(state);
                    emit_op(state, make_op_num("Jz", 0));
                    // Pre-emit end-Jmp BEFORE body (body re-parse corrupts later bytes)
                    let _m_skip_jmp = current_pos(state);
                    emit_jmp(state, 0);              // skip → body_start
                    let _mej_pos = current_pos(state);
                    if ai == 0 { let __g_mej0 = _mej_pos; };
                    if ai == 1 { let __g_mej1 = _mej_pos; };
                    if ai == 2 { let __g_mej2 = _mej_pos; };
                    if ai == 3 { let __g_mej3 = _mej_pos; };
                    emit_jmp(state, 0);              // end-Jmp (patched later)
                    let _m_body_begin = current_pos(state);
                    patch_jump(state, _m_skip_jmp, _m_body_begin);
                    // Compile arm body via re-parse
                    let _m_bpos = _m_body_s + 1;
                    while _m_bpos < (_m_body_e - 1) {
                        let _m_bp2 = new_parser(__g_ma_tokens);
                        _m_bp2.pos = _m_bpos;
                        if is_symbol_tok(peek(_m_bp2), "}") { break; };
                        if is_eof(peek(_m_bp2)) { break; };
                        let _m_bstmt = parse_stmt(_m_bp2);
                        compile_stmt(state, _m_bstmt);
                        let _m_bpos = _m_bp2.pos;
                    };
                    // Jump to pre-emitted end-Jmp
                    emit_jmp(state, _mej_pos);
                    patch_jump(state, jz_pos, current_pos(state));
                } else {
                    // Wildcard: pre-emit end-Jmp then body
                    let _m_wskip = current_pos(state);
                    emit_jmp(state, 0);
                    let _mej_pos = current_pos(state);
                    if ai == 0 { let __g_mej0 = _mej_pos; };
                    if ai == 1 { let __g_mej1 = _mej_pos; };
                    if ai == 2 { let __g_mej2 = _mej_pos; };
                    if ai == 3 { let __g_mej3 = _mej_pos; };
                    emit_jmp(state, 0);
                    let _m_wbody_begin = current_pos(state);
                    patch_jump(state, _m_wskip, _m_wbody_begin);
                    let _m_wpos = _m_body_s + 1;
                    while _m_wpos < (_m_body_e - 1) {
                        let _m_wp2 = new_parser(__g_ma_tokens);
                        _m_wp2.pos = _m_wpos;
                        if is_symbol_tok(peek(_m_wp2), "}") { break; };
                        if is_eof(peek(_m_wp2)) { break; };
                        let _m_wstmt2 = parse_stmt(_m_wp2);
                        compile_stmt(state, _m_wstmt2);
                        let _m_wpos = _m_wp2.pos;
                    };
                    emit_jmp(state, _mej_pos);
                };
                let ai = ai + 1;
            };
            // Patch all end jumps BEFORE dummy result (so they execute PushNum)
            let _m_end = current_pos(state);
            // Push dummy result (match is expression, needs value on stack)
            emit_op(state, make_op_num("PushNum", 0));
            if __g_mej0 >= 0 { patch_jump(state, __g_mej0, _m_end); };
            if __g_mej1 >= 0 { patch_jump(state, __g_mej1, _m_end); };
            if __g_mej2 >= 0 { patch_jump(state, __g_mej2, _m_end); };
            if __g_mej3 >= 0 { patch_jump(state, __g_mej3, _m_end); };
        },
        Expr::Lambda { params, body } => {
            // Lambda expression: fn(params) { body } → emit Closure like FnDef but no Store
            let _lm_params = params;
            let _lm_body = body;
            let _lm_pcnt = len(_lm_params);
            let _lm_closure_pos = current_pos(state);
            emit_op(state, make_op_num("Closure", _lm_pcnt));
            // Store params (reversed for stack order)
            let _lm_saved = save_locals(state);
            let _lm_pi = _lm_pcnt - 1;
            while _lm_pi >= 0 {
                emit_op(state, make_op_name("Store", _lm_params[_lm_pi]));
                push_local(state, _lm_params[_lm_pi]);
                let _lm_pi = _lm_pi - 1;
            };
            // Compile body
            let _lm_bi = 0;
            while _lm_bi < len(_lm_body) {
                compile_stmt(state, _lm_body[_lm_bi]);
                let _lm_bi = _lm_bi + 1;
            };
            // Default return
            emit_op(state, make_op_name("Push", ""));
            emit_op(state, make_op_simple("Ret"));
            restore_locals(state, _lm_saved);
            // Patch body_len
            let _lm_body_len = current_pos(state) - _lm_closure_pos - 6;
            let _lm_bpos = _lm_closure_pos + 2;
            set_at(_g_output, _lm_bpos, _lm_body_len % 256);
            set_at(_g_output, _lm_bpos + 1, (_lm_body_len / 256) % 256);
            set_at(_g_output, _lm_bpos + 2, (_lm_body_len / 65536) % 256);
            set_at(_g_output, _lm_bpos + 3, (_lm_body_len / 16777216) % 256);
            // Closure value is now on stack (pushed by cg_closure opcode)
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
            let _ls_name = name;
            push(_ce_stack, _ls_name);
            compile_expr(state, value);
            let _ls_name = pop(_ce_stack);
            emit_op(state, make_op_name("Store", _ls_name));
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
            // Patch Closure body_len (in bytes)
            // Closure instruction = [0x25][param_count:1][body_len:4] = 6 bytes
            let _fn_body_len = current_pos(state) - _fn_closure_pos - 6;
            let _fn_bpos = _fn_closure_pos + 2;
            set_at(_g_output, _fn_bpos, _fn_body_len % 256);
            set_at(_g_output, _fn_bpos + 1, (_fn_body_len / 256) % 256);
            set_at(_g_output, _fn_bpos + 2, (_fn_body_len / 65536) % 256);
            set_at(_g_output, _fn_bpos + 3, (_fn_body_len / 16777216) % 256);
            // Store closure in var_table
            emit_op(state, make_op_name("Store", _fn_name));
            // T5 LG.1: Auto-register fn as node (name + param count)
            emit_op(state, make_op_name("Push", _fn_name));
            emit_op(state, make_op_num("PushNum", _fn_pcnt));
            emit_op(state, make_op_name("Call", "fn_node_register"));
            emit_op(state, make_op_simple("Pop"));
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
            // Save outer break/continue context
            let _wl_old_breaks = _break_patches;
            let _wl_old_conts = _continue_patches;
            let _break_patches = [];
            let _continue_patches = [];
            let _wl_body = body;
            let _wl_start = current_pos(state);
            // Re-parse condition from tokens (avoids dict corruption)
            // WhileStmt has cond_start, cond_end, tokens fields
            let _wl_cond_start = stmt.cond_start;
            let _wl_cond_end = stmt.cond_end;
            let _wl_tokens = stmt.tokens;
            push(_ce_stack, _wl_body);
            push(_ce_stack, _wl_cond_start);
            push(_ce_stack, _wl_cond_end);
            push(_ce_stack, _wl_tokens);
            // Fresh parse of condition from saved tokens
            let _wl_p = new_parser(_wl_tokens);
            _wl_p.pos = _wl_cond_start;
            let _wl_fresh_cond = parse_expr(_wl_p);
            compile_expr(state, _wl_fresh_cond);
            let _wl_tokens = pop(_ce_stack);
            let _wl_cond_end = pop(_ce_stack);
            let _wl_cond_start = pop(_ce_stack);
            let _wl_body = pop(_ce_stack);
            let _wl_jz = current_pos(state);
            emit_op(state, make_op_num("Jz", 0));
            let _wl_bi = 0;
            while _wl_bi < len(_wl_body) {
                push(_ce_stack, _wl_body);
                push(_ce_stack, _wl_jz);
                push(_ce_stack, _wl_start);
                push(_ce_stack, _wl_bi);
                compile_stmt(state, _wl_body[_wl_bi]);
                let _wl_bi = pop(_ce_stack);
                let _wl_start = pop(_ce_stack);
                let _wl_jz = pop(_ce_stack);
                let _wl_body = pop(_ce_stack);
                let _wl_bi = _wl_bi + 1;
            };
            // Patch continue → _wl_start
            let _wl_cp = 0;
            while _wl_cp < len(_continue_patches) {
                patch_jump(state, _continue_patches[_wl_cp], _wl_start);
                let _wl_cp = _wl_cp + 1;
            };
            emit_jmp(state, _wl_start);
            // Patch break → after loop
            let _wl_exit = current_pos(state);
            patch_jump(state, _wl_jz, _wl_exit);
            let _bp_i = 0;
            while _bp_i < len(_break_patches) {
                patch_jump(state, _break_patches[_bp_i], _wl_exit);
                let _bp_i = _bp_i + 1;
            };
            // Restore outer context
            let _break_patches = _wl_old_breaks;
            let _continue_patches = _wl_old_conts;
        },
        Stmt::ForStmt { var, iter, body } => {
            // Read ALL depth-indexed globals BEFORE incrementing _g_for_depth
            let _fl_var = "";
            let _fl_is = 0;
            let _fl_ie = 0;
            if _g_for_depth == 0 { let _fl_var = __g_fv0; let _fl_is = __g_fi0s; let _fl_ie = __g_fi0e; };
            if _g_for_depth == 1 { let _fl_var = __g_fv1; let _fl_is = __g_fi1s; let _fl_ie = __g_fi1e; };
            if _g_for_depth == 2 { let _fl_var = __g_fv2; let _fl_is = __g_fi2s; let _fl_ie = __g_fi2e; };
            if _g_for_depth == 3 { let _fl_var = __g_fv3; let _fl_is = __g_fi3s; let _fl_ie = __g_fi3e; };
            // Lower for-in to while loop with UNIQUE names per depth
            let _fl_d = __to_string(_g_for_depth);
            let _fl_arr = "__for_" + _fl_d + "_arr";
            let _fl_len = "__for_" + _fl_d + "_len";
            let _fl_idx = "__for_" + _fl_d + "_idx";
            let _g_for_depth = _g_for_depth + 1;

            // Save outer break/continue context
            let _fl_old_breaks = _break_patches;
            let _fl_old_conts = _continue_patches;
            let _break_patches = [];
            let _continue_patches = [];

            // Evaluate and store iterator (re-parse from tokens — dict field corrupt)
            let _fl_ip = new_parser(__g_fi_tokens);
            _fl_ip.pos = _fl_is;
            let _fl_iter_ast = parse_expr(_fl_ip);
            compile_expr(state, _fl_iter_ast);
            emit_op(state, make_op_name("Store", _fl_arr));

            // Store length
            emit_op(state, make_op_name("Load", _fl_arr));
            emit_op(state, make_op_name("Call", "__array_len"));
            emit_op(state, make_op_name("Store", _fl_len));

            // Initialize index = 0
            emit_op(state, make_op_num("PushNum", 0));
            emit_op(state, make_op_name("Store", _fl_idx));

            // Loop start
            let _fl_start = current_pos(state);
            let _continue_target = _fl_start;
            emit_op(state, make_op_name("Load", _fl_idx));
            emit_op(state, make_op_name("Load", _fl_len));
            emit_op(state, make_op_name("Call", "__cmp_lt"));
            let _fl_jz = current_pos(state);
            emit_op(state, make_op_num("Jz", 0));

            // let var = arr[idx]
            emit_op(state, make_op_name("Load", _fl_arr));
            emit_op(state, make_op_name("Load", _fl_idx));
            emit_op(state, make_op_name("Call", "__array_get"));
            emit_op(state, make_op_name("Store", _fl_var));

            // Emit increment + jump BEFORE body compilation
            // (so we don't need _fl_idx string after body — it may be corrupt)
            // Skip increment on first entry: Jmp → body_start
            let _fl_body_jmp = current_pos(state);
            emit_jmp(state, 0);              // placeholder → body_start

            // INCREMENT SECTION (jumped to from body end)
            let _fl_inc = current_pos(state);
            emit_op(state, make_op_name("Load", _fl_idx));
            emit_op(state, make_op_num("PushNum", 1));
            emit_op(state, make_op_name("Call", "__hyp_add"));
            emit_op(state, make_op_name("Store", _fl_idx));
            emit_jmp(state, _fl_start);     // jump back to condition check

            // BODY START (patched from body_jmp)
            let _fl_body_start = current_pos(state);
            patch_jump(state, _fl_body_jmp, _fl_body_start);

            // Save _fl_inc and _fl_jz to depth-indexed globals
            let _fl_my_depth = _g_for_depth - 1;
            if _fl_my_depth == 0 { let __g_fl_inc0 = _fl_inc; let __g_fl_jz0 = _fl_jz; };
            if _fl_my_depth == 1 { let __g_fl_inc1 = _fl_inc; let __g_fl_jz1 = _fl_jz; };
            if _fl_my_depth == 2 { let __g_fl_inc2 = _fl_inc; let __g_fl_jz2 = _fl_jz; };
            if _fl_my_depth == 3 { let __g_fl_inc3 = _fl_inc; let __g_fl_jz3 = _fl_jz; };

            // Compile body
            let _fl_bi = 0;
            while _fl_bi < len(body) {
                compile_stmt(state, body[_fl_bi]);
                let _fl_bi = _fl_bi + 1;
            };

            // Restore _fl_inc and _fl_jz (re-compute depth from _g_for_depth)
            let _fl_rd = _g_for_depth - 1;
            if _fl_rd == 0 { let _fl_inc = __g_fl_inc0; let _fl_jz = __g_fl_jz0; };
            if _fl_rd == 1 { let _fl_inc = __g_fl_inc1; let _fl_jz = __g_fl_jz1; };
            if _fl_rd == 2 { let _fl_inc = __g_fl_inc2; let _fl_jz = __g_fl_jz2; };
            if _fl_rd == 3 { let _fl_inc = __g_fl_inc3; let _fl_jz = __g_fl_jz3; };

            // Patch continue → increment section
            let _fl_cp = 0;
            while _fl_cp < len(_continue_patches) {
                patch_jump(state, _continue_patches[_fl_cp], _fl_inc);
                let _fl_cp = _fl_cp + 1;
            };

            // Jump to increment section
            emit_jmp(state, _fl_inc);

            // (removed duplicate Jmp — increment section already has Jmp LOOP_START)
            // Patch break + exit
            let _fl_exit = current_pos(state);
            patch_jump(state, _fl_jz, _fl_exit);
            let _fl_bp = 0;
            while _fl_bp < len(_break_patches) {
                patch_jump(state, _break_patches[_fl_bp], _fl_exit);
                let _fl_bp = _fl_bp + 1;
            };
            // Restore
            let _break_patches = _fl_old_breaks;
            let _continue_patches = _fl_old_conts;
            let _g_for_depth = _g_for_depth - 1;
        },
        Stmt::BreakStmt => {
            let _brk_pos = current_pos(state);
            emit_op(state, make_op_num("Jmp", 0));
            push(_break_patches, _brk_pos);
        },
        Stmt::ContinueStmt => {
            let _cont_pos = current_pos(state);
            emit_op(state, make_op_num("Jmp", 0));
            push(_continue_patches, _cont_pos);
        },
        Stmt::TypeDef { name, fields } => {
            // Type metadata — no opcodes needed for bootstrap
        },
        Stmt::UnionDef { name, variants } => {
            // Union metadata — no opcodes needed for bootstrap
        },
        Stmt::UseStmt { path } => {
            // Module import: UseStmt is handled at REPL level by inlining file contents
            // before compilation. By the time we reach semantic, `use` has been expanded.
            // If we somehow get here, it means use wasn't expanded → skip silently.
        },
        Stmt::FieldAssign { object, field, value } => {
            // obj.field = value → load obj, set field, store back
            let _fa_obj = object;
            if is_local(state, _fa_obj) {
                emit_op(state, make_op_name("LoadLocal", _fa_obj));
            } else {
                emit_op(state, make_op_name("Load", _fa_obj));
            };
            emit_op(state, make_op_name("Push", field));
            push(_ce_stack, _fa_obj);
            compile_expr(state, value);
            let _fa_obj = pop(_ce_stack);
            emit_op(state, make_op_name("Call", "__dict_set"));
            emit_op(state, make_op_name("Store", _fa_obj));
        },
        Stmt::MatchStmt { subject, arms } => {
            // Match as statement: compile subject match expression, discard result
            compile_expr(state, subject);
            emit_op(state, make_op_simple("Pop"));
        },
        Stmt::TryCatch { try_block, catch_block } => {
            // try { body } catch { handler }
            // → TryBegin(catch_pc) [try body] Jmp(end) [catch body] CatchEnd
            let _tc_try = try_block;
            let _tc_catch = catch_block;

            // TryBegin — patch later with catch_pc
            let _tc_try_pos = current_pos(state);
            _emit_byte(state, 26);      // 0x1A = TryBegin
            _emit_u32_le(state, 0);     // placeholder for catch_offset

            // Compile try block
            let _tc_ti = 0;
            while _tc_ti < len(_tc_try) {
                push(_ce_stack, _tc_try);
                push(_ce_stack, _tc_catch);
                push(_ce_stack, _tc_try_pos);
                push(_ce_stack, _tc_ti);
                compile_stmt(state, _tc_try[_tc_ti]);
                let _tc_ti = pop(_ce_stack);
                let _tc_try_pos = pop(_ce_stack);
                let _tc_catch = pop(_ce_stack);
                let _tc_try = pop(_ce_stack);
                let _tc_ti = _tc_ti + 1;
            };

            // Jmp past catch on success
            let _tc_jmp_pos = current_pos(state);
            emit_jmp(state, 0);         // placeholder

            // Patch TryBegin → catch_pc
            let _tc_catch_pc = current_pos(state);
            patch_jump(state, _tc_try_pos, _tc_catch_pc);

            // Compile catch block
            let _tc_ci = 0;
            while _tc_ci < len(_tc_catch) {
                push(_ce_stack, _tc_catch);
                push(_ce_stack, _tc_jmp_pos);
                push(_ce_stack, _tc_ci);
                compile_stmt(state, _tc_catch[_tc_ci]);
                let _tc_ci = pop(_ce_stack);
                let _tc_jmp_pos = pop(_ce_stack);
                let _tc_catch = pop(_ce_stack);
                let _tc_ci = _tc_ci + 1;
            };

            // CatchEnd
            _emit_byte(state, 27);      // 0x1B = CatchEnd

            // Patch Jmp → after catch
            patch_jump(state, _tc_jmp_pos, current_pos(state));
        },
        Stmt::ExprStmt { expr } => {
            compile_expr(state, expr);
            // P0-B: Auto-emit bare expressions (like Python/Node REPL)
            // "2+3" → prints 5 instead of silently discarding
            emit_op(state, make_op_simple("Emit"));
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
