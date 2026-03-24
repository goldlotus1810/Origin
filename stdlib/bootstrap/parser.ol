// ── Olang Bootstrap Parser ────────────────────────────────────────
// Self-hosting preparation: parser written in Olang.
// Reads token list → produces AST nodes.
//
// Phase 4 / A9 — compiler self-hosting foundation.
// Depends on: stdlib/bootstrap/lexer.ol

use olang.bootstrap.lexer;

// ── AST Node types ───────────────────────────────────────────────

union Expr {
    Ident { name: Str },
    NumLit { value: Num },
    StrLit { value: Str },
    BoolLit { value: Num },
    BinOp { op: Str, lhs: Expr, rhs: Expr },
    UnaryNot { expr: Expr },
    Call { callee: Expr, args: Vec[Expr] },
    FieldAccess { object: Expr, field: Str },
    Index { object: Expr, index: Expr },
    ArrayLit { items: Vec[Expr] },
    PathExpr { base: Str, member: Str },
    StructLit { path: Str, fields: Vec[FieldInit] },
    IfExpr { cond: Expr, then_expr: Expr, else_expr: Expr },
    MolLiteral { packed: Num },
    MatchExpr { subject: Expr, arms: Vec[MatchArm] },
    DictLit { fields: Vec[FieldInit] },
    ArrayComp { var: Str, depth: Num },
}

union Stmt {
    LetStmt { name: Str, value: Expr },
    ExprStmt { expr: Expr },
    FnDef { name: Str, params: Vec[Str], body: Vec[Stmt] },
    IfStmt { cond: Expr, then_block: Vec[Stmt], else_block: Vec[Stmt] },
    WhileStmt { cond: Expr, body: Vec[Stmt] },
    ForStmt { var: Str, iter: Expr, body: Vec[Stmt] },
    ReturnStmt { value: Expr },
    EmitStmt { expr: Expr },
    TypeDef { name: Str, fields: Vec[Field] },
    UnionDef { name: Str, variants: Vec[Variant] },
    BreakStmt,
    ContinueStmt,
    UseStmt { path: Str },
    MatchStmt { subject: Expr, arms: Vec[MatchArm] },
    FieldAssign { object: Str, field: Str, value: Expr },
    TryCatch { try_block: Vec[Stmt], catch_block: Vec[Stmt] },
}

type Field {
    name: Str,
    type_name: Str,
}

type Variant {
    name: Str,
    fields: Vec[Field],
}

type FieldInit {
    name: Str,
    value: Expr,
}

type MatchArm {
    pattern: Str,
    bindings: Vec[Str],
    body: Vec[Stmt],
}

// ── Parse error flag (global, checked by repl.ol) ───────────────
let _g_parse_error = 0;

// ── ArrayComp globals (depth-indexed — arrays unsafe due to heap overlap) ──
let _g_comp_depth = 0;
// expr token ranges
let __g_ce0s = 0; let __g_ce0e = 0;
let __g_ce1s = 0; let __g_ce1e = 0;
let __g_ce2s = 0; let __g_ce2e = 0;
let __g_ce3s = 0; let __g_ce3e = 0;
// iter token ranges
let __g_ci0s = 0; let __g_ci0e = 0;
let __g_ci1s = 0; let __g_ci1e = 0;
let __g_ci2s = 0; let __g_ci2e = 0;
let __g_ci3s = 0; let __g_ci3e = 0;
// filter token ranges (-1 = no filter)
let __g_cf0s = -1; let __g_cf0e = -1;
let __g_cf1s = -1; let __g_cf1e = -1;
let __g_cf2s = -1; let __g_cf2e = -1;
let __g_cf3s = -1; let __g_cf3e = -1;
// var names
let __g_cv0 = ""; let __g_cv1 = ""; let __g_cv2 = ""; let __g_cv3 = "";
// tokens ref
let __g_comp_tokens = "";

// ── Parser state ─────────────────────────────────────────────────

type Parser {
    tokens: Vec[Token],
    pos: Num,
}

fn new_parser(tokens) {
    return Parser { tokens: tokens, pos: 0 };
}

fn peek(p) {
    if p.pos < len(p.tokens) {
        return p.tokens[p.pos];
    };
    return Token { kind: TokenKind::Eof, text: "", line: 0, col: 0 };
}

fn advance(p) {
    let tok = peek(p);
    p.pos = p.pos + 1;
    return tok;
}

fn expect_symbol(p, sym) {
    let tok = advance(p);
    match tok.kind {
        TokenKind::Symbol { ch } => {
            if ch != sym {
                emit "Parse error: expected '" + sym + "' got '" + ch + "'";
                let _g_parse_error = 1;
            };
        },
        _ => {
            emit "Parse error: expected symbol '" + sym + "'";
            let _g_parse_error = 1;
        },
    };
    return tok;
}

fn expect_ident(p) {
    let tok = advance(p);
    match tok.kind {
        TokenKind::Ident { name } => {
            return name;
        },
        _ => {
            emit "Parse error: expected identifier";
            let _g_parse_error = 1;
            return "";
        },
    };
}

fn is_keyword_tok(tok, kw) {
    match tok.kind {
        TokenKind::Keyword { name } => {
            return name == kw;
        },
        _ => {
            return false;
        },
    };
}

fn is_symbol_tok(tok, sym) {
    match tok.kind {
        TokenKind::Symbol { ch } => {
            return ch == sym;
        },
        _ => {
            return false;
        },
    };
}

fn is_eof(tok) {
    match tok.kind {
        TokenKind::Eof => { return true; },
        _ => { return false; },
    };
}

fn is_ident_tok(tok) {
    match tok.kind {
        TokenKind::Ident { name } => { return true; },
        _ => { return false; },
    };
}

// ── Type annotation helper (skip type annotations) ──────────────

fn skip_type_annotation(p) {
    // Skip a type annotation like "Str", "Num", "Vec[Expr]", "Vec[Vec[Str]]"
    let tok = advance(p); // consume type name
    // Handle Vec[...] or other parameterized types
    if is_symbol_tok(peek(p), "[") {
        advance(p); // consume [
        let depth = 1;
        while depth > 0 && !is_eof(peek(p)) {
            if is_symbol_tok(peek(p), "[") {
                let depth = depth + 1;
            };
            if is_symbol_tok(peek(p), "]") {
                let depth = depth - 1;
            };
            advance(p);
        };
    };
}

// ── Expression parsing (precedence climbing) ─────────────────────

fn parse_primary(p) {
    let tok = peek(p);

    match tok.kind {
        TokenKind::Number { value } => {
            advance(p);
            return Expr::NumLit { value: value };
        },
        TokenKind::StringLit { value } => {
            advance(p);
            return Expr::StrLit { value: value };
        },
        TokenKind::Keyword { name } => {
            // true/false literals
            if name == "true" {
                advance(p);
                return Expr::BoolLit { value: 1 };
            };
            if name == "false" {
                advance(p);
                return Expr::BoolLit { value: 0 };
            };
            // if-expression: if cond { expr } else { expr }
            if name == "if" {
                advance(p);
                let cond = parse_expr(p);
                expect_symbol(p, "{");
                let then_val = parse_expr(p);
                expect_symbol(p, "}");
                let else_val = Expr::NumLit { value: 0 };
                if is_keyword_tok(peek(p), "else") {
                    advance(p);
                    expect_symbol(p, "{");
                    let else_val = parse_expr(p);
                    expect_symbol(p, "}");
                };
                return Expr::IfExpr { cond: cond, then_expr: then_val, else_expr: else_val };
            };
            // match expression
            if name == "match" {
                return parse_match_expr(p);
            };
            // Other keywords used as identifiers (fallthrough to emit error)
            emit "Parse error: unexpected keyword '" + name + "'";
            let _g_parse_error = 1;
            advance(p);
            return Expr::NumLit { value: 0 };
        },
        TokenKind::Ident { name } => {
            advance(p);
            let _pp_result = Expr::Ident { name: name };

            // Path expression: Name::Member or Name::Member { fields }
            if is_symbol_tok(peek(p), "::") {
                advance(p); // consume ::
                let member = expect_ident(p);
                // Struct constructor: Name::Member { field: value, ... }
                if is_symbol_tok(peek(p), "{") {
                    advance(p); // consume {
                    let fields = [];
                    while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
                        let fname = expect_ident(p);
                        expect_symbol(p, ":");
                        let fvalue = parse_expr(p);
                        push(fields, FieldInit { name: fname, value: fvalue });
                        if is_symbol_tok(peek(p), ",") { advance(p); };
                    };
                    expect_symbol(p, "}");
                    let _pp_result = Expr::StructLit { path: name + "::" + member, fields: fields };
                } else {
                    let _pp_result = Expr::PathExpr { base: name, member: member };
                };
            };

            // Struct literal: Name { field: value, ... }
            // Only when Name starts with uppercase (heuristic)
            if is_symbol_tok(peek(p), "{") {
                let first_ch = char_at(name, 0);
                if first_ch >= "A" && first_ch <= "Z" {
                    advance(p); // consume {
                    let fields = [];
                    while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
                        let fname = expect_ident(p);
                        expect_symbol(p, ":");
                        let fvalue = parse_expr(p);
                        push(fields, FieldInit { name: fname, value: fvalue });
                        if is_symbol_tok(peek(p), ",") { advance(p); };
                    };
                    expect_symbol(p, "}");
                    let _pp_result = Expr::StructLit { path: name, fields: fields };
                };
            };

            // Postfix chain: .field, [index], (args) — can repeat
            // Use is_postfix helper to check loop condition without mutable flag
            while is_symbol_tok(peek(p), ".") || is_symbol_tok(peek(p), "[") || is_symbol_tok(peek(p), "(") {
                if is_symbol_tok(peek(p), ".") {
                    advance(p);
                    let field = expect_ident(p);
                    let _pp_result = Expr::FieldAccess { object: _pp_result, field: field };
                };
                if is_symbol_tok(peek(p), "[") {
                    advance(p);
                    // Parse index expression
                    push(_pb_stack, _pp_result);
                    let _pp_idx_expr = parse_expr(p);
                    let _pp_saved_obj = pop(_pb_stack);
                    expect_symbol(p, "]");
                    // Desugar a[i] → __array_get(a, i) — avoids Index dict corruption
                    let _pp_result = Expr::Call {
                        callee: Expr::Ident { name: "__array_get" },
                        args: [_pp_saved_obj, _pp_idx_expr],
                    };
                };
                if is_symbol_tok(peek(p), "(") {
                    advance(p);
                    let _pp_call_args = [];
                    // Save _pp_result before recursive parse_expr (may overwrite)
                    push(_pb_stack, _pp_result);
                    while !is_symbol_tok(peek(p), ")") && !is_eof(peek(p)) {
                        push(_pb_stack, _pp_call_args);
                        let _pp_arg = parse_expr(p);
                        let _pp_call_args = pop(_pb_stack);
                        push(_pp_call_args, _pp_arg);
                        if is_symbol_tok(peek(p), ",") { advance(p); };
                    };
                    expect_symbol(p, ")");
                    let _pp_saved = pop(_pb_stack);
                    let _pp_result = Expr::Call { callee: _pp_saved, args: _pp_call_args };
                };
            };

            return _pp_result;
        },
        TokenKind::Symbol { ch } => {
            // Unary NOT: !expr
            if ch == "!" {
                advance(p);
                let expr = parse_primary(p);
                return Expr::UnaryNot { expr: expr };
            };
            // Unary minus: -expr → 0 - expr
            if ch == "-" {
                advance(p);
                let _ps_neg = parse_primary(p);
                return Expr::BinOp { op: "-", lhs: Expr::NumLit { value: 0 }, rhs: _ps_neg };
            };
            // Parenthesized expression
            if ch == "(" {
                advance(p);
                let expr = parse_expr(p);
                expect_symbol(p, ")");
                return expr;
            };
            // Array literal or comprehension: [expr, ...] or [expr for x in iter]
            if ch == "[" {
                advance(p);
                // Empty array
                if is_symbol_tok(peek(p), "]") {
                    advance(p);
                    return Expr::ArrayLit { items: [] };
                };
                // Save expr range: increment depth FIRST to prevent inner [ ] from
                // overwriting our depth slot. Inner arrays use depth+1 slot (harmless).
                let _pa_my_depth = _g_comp_depth;
                let _g_comp_depth = _g_comp_depth + 1;
                if _pa_my_depth == 0 { let __g_ce0s = p.pos; };
                if _pa_my_depth == 1 { let __g_ce1s = p.pos; };
                if _pa_my_depth == 2 { let __g_ce2s = p.pos; };
                if _pa_my_depth == 3 { let __g_ce3s = p.pos; };
                let _pa_first = parse_expr(p);
                if _pa_my_depth == 0 { let __g_ce0e = p.pos; };
                if _pa_my_depth == 1 { let __g_ce1e = p.pos; };
                if _pa_my_depth == 2 { let __g_ce2e = p.pos; };
                if _pa_my_depth == 3 { let __g_ce3e = p.pos; };
                // Check for comprehension
                if is_keyword_tok(peek(p), "for") {
                    advance(p);
                    let _pa_cv = peek(p).text;
                    advance(p);
                    advance(p);  // consume "in"
                    let _pa_iter_start = p.pos;
                    let _pa_iter_expr = parse_expr(p);
                    let _pa_iter_end = p.pos;
                    let _pa_filt_start = -1;
                    let _pa_filt_end = -1;
                    if is_keyword_tok(peek(p), "if") {
                        advance(p);
                        let _pa_filt_start = p.pos;
                        let _pa_filt_expr = parse_expr(p);
                        let _pa_filt_end = p.pos;
                    };
                    expect_symbol(p, "]");
                    // Save iter + filter + var to depth-indexed globals
                    // (expr range already saved above, before inner parse)
                    let _pa_d = _g_comp_depth - 1;
                    if _pa_d == 0 {
                        let __g_ci0s = _pa_iter_start; let __g_ci0e = _pa_iter_end;
                        let __g_cf0s = _pa_filt_start; let __g_cf0e = _pa_filt_end;
                        let __g_cv0 = _pa_cv;
                    };
                    if _pa_d == 1 {
                        let __g_ci1s = _pa_iter_start; let __g_ci1e = _pa_iter_end;
                        let __g_cf1s = _pa_filt_start; let __g_cf1e = _pa_filt_end;
                        let __g_cv1 = _pa_cv;
                    };
                    if _pa_d == 2 {
                        let __g_ci2s = _pa_iter_start; let __g_ci2e = _pa_iter_end;
                        let __g_cf2s = _pa_filt_start; let __g_cf2e = _pa_filt_end;
                        let __g_cv2 = _pa_cv;
                    };
                    if _pa_d == 3 {
                        let __g_ci3s = _pa_iter_start; let __g_ci3e = _pa_iter_end;
                        let __g_cf3s = _pa_filt_start; let __g_cf3e = _pa_filt_end;
                        let __g_cv3 = _pa_cv;
                    };
                    let __g_comp_tokens = p.tokens;
                    // depth already incremented above
                    return Expr::ArrayComp { var: _pa_cv, depth: _pa_d };
                };
                // Regular array literal — restore depth (was incremented speculatively)
                let _g_comp_depth = _g_comp_depth - 1;
                let items = [_pa_first];
                while is_symbol_tok(peek(p), ",") {
                    advance(p);
                    if !is_symbol_tok(peek(p), "]") {
                        push(items, parse_expr(p));
                    };
                };
                expect_symbol(p, "]");
                return Expr::ArrayLit { items: items };
            };
            // Molecular literal: { S=1 R=2 V=128 A=128 T=3 }
            // Also handles dict literal attempt: { key: val } → error + skip
            if ch == "{" {
                advance(p);
                // Lookahead: if next is ident followed by ":", it's a dict literal (unsupported)
                // Skip to matching "}" to prevent cascading errors
                let _mol_peek = peek(p);
                let _mol_is_dict = 0;
                match _mol_peek.kind {
                    TokenKind::Ident { name } => {
                        // Check if token after ident is ":"
                        if p.pos + 1 < len(p.tokens) {
                            let _mol_peek2 = p.tokens[p.pos + 1];
                            match _mol_peek2.kind {
                                TokenKind::Symbol { ch } => {
                                    if ch == ":" {
                                        let _mol_is_dict = 1;
                                    };
                                },
                                _ => {},
                            };
                        };
                    },
                    _ => {},
                };
                if _mol_is_dict == 1 {
                    // Parse dict literal: { key: value, key2: value2 }
                    let _dl_fields = [];
                    while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) && _g_parse_error == 0 {
                        let _dl_fname = expect_ident(p);
                        expect_symbol(p, ":");
                        let _dl_fval = parse_expr(p);
                        push(_dl_fields, FieldInit { name: _dl_fname, value: _dl_fval });
                        if is_symbol_tok(peek(p), ",") { advance(p); };
                    };
                    expect_symbol(p, "}");
                    return Expr::DictLit { fields: _dl_fields };
                };
                let s = 0;
                let r = 0;
                let v = 0;
                let a = 0;
                let t = 0;
                while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) && _g_parse_error == 0 {
                    let dim = expect_ident(p);
                    expect_symbol(p, "=");
                    let val_tok = advance(p);
                    match val_tok.kind {
                        TokenKind::Number { value } => {
                            if dim == "S" { let s = value; };
                            if dim == "R" { let r = value; };
                            if dim == "V" { let v = value; };
                            if dim == "A" { let a = value; };
                            if dim == "T" { let t = value; };
                        },
                        _ => {},
                    };
                };
                expect_symbol(p, "}");
                // Pack into u16: [S:4][R:4][V:3][A:3][T:2]
                let packed = mol_new(s, r, v, a, t);
                return Expr::MolLiteral { packed: packed };
            };
            emit "Parse error: unexpected symbol '" + ch + "'";
            let _g_parse_error = 1;
            advance(p);
            return Expr::NumLit { value: 0 };
        },
        _ => {
            emit "Parse error: unexpected token";
            let _g_parse_error = 1;
            advance(p);
            return Expr::NumLit { value: 0 };
        },
    };
}

fn precedence(op) {
    if op == "||" { return 1; };
    if op == "&&" { return 2; };
    if op == "==" || op == "!=" { return 3; };
    if op == "<" || op == ">" || op == "<=" || op == ">=" { return 4; };
    if op == "+" || op == "-" { return 5; };
    if op == "*" || op == "/" || op == "%" { return 6; };
    return 0;
}

fn is_binop(tok) {
    match tok.kind {
        TokenKind::Symbol { ch } => {
            return precedence(ch) > 0;
        },
        _ => { return false; },
    };
}

// Explicit save stack for recursive parse_expr_prec (ASM VM has no scoping)
let _pep_stack = [];

fn parse_expr_prec(p, min_prec) {
    let _pep_lhs = parse_primary(p);

    while is_binop(peek(p)) {
        let _pep_tok = peek(p);
        match _pep_tok.kind {
            TokenKind::Symbol { ch } => {
                let _pep_prec = precedence(ch);
                if _pep_prec < min_prec {
                    break;
                };
                advance(p); // consume operator
                // Save ALL state before recursive call
                push(_pep_stack, _pep_lhs);
                push(_pep_stack, ch);
                push(_pep_stack, min_prec);
                let _pep_rhs = parse_expr_prec(p, _pep_prec + 1);
                // Restore after recursion
                let min_prec = pop(_pep_stack);
                let _pep_op = pop(_pep_stack);
                let _pep_saved = pop(_pep_stack);
                let _pep_lhs = Expr::BinOp { op: _pep_op, lhs: _pep_saved, rhs: _pep_rhs };
            },
            _ => { break; },
        };
    };

    return _pep_lhs;
}

pub fn parse_expr(p) {
    return parse_expr_prec(p, 1);
}

// ── Match expression parsing ────────────────────────────────────

fn parse_match_expr(p) {
    advance(p); // consume 'match'
    let subject = parse_expr(p);
    expect_symbol(p, "{");
    let arms = [];
    while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
        // Parse pattern: Name::Variant { bindings } or _ (wildcard)
        let pattern = "";
        let bindings = [];
        let ptok = peek(p);

        // Pattern: _ (wildcard), Name::Variant { bindings }, Name, number, string
        if is_ident_tok(ptok) {
            let pname = expect_ident(p);
            if pname == "_" {
                let pattern = "_";
            } else {
                let pattern = pname;
                if is_symbol_tok(peek(p), "::") {
                    advance(p);
                    let variant = expect_ident(p);
                    let pattern = pname + "::" + variant;
                };
                if is_symbol_tok(peek(p), "{") {
                    advance(p);
                    while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
                        push(bindings, expect_ident(p));
                        if is_symbol_tok(peek(p), ",") { advance(p); };
                    };
                    expect_symbol(p, "}");
                };
            };
        } else {
            // Number or string literal pattern
            match ptok.kind {
                TokenKind::Number { value } => {
                    advance(p);
                    let pattern = "__num:" + __to_string(value);
                },
                TokenKind::StringLit { value } => {
                    advance(p);
                    let pattern = "__str:" + value;
                },
                _ => {
                    advance(p);
                    let pattern = "_";
                },
            };
        };

        expect_symbol(p, "=>");
        // Save body token range for re-parse in semantic
        let _ma_body_start = p.pos;
        let body = [];
        if is_symbol_tok(peek(p), "{") {
            let body = parse_block(p);
        } else {
            push(body, Stmt::ExprStmt { expr: parse_expr(p) });
            if is_symbol_tok(peek(p), ";") { advance(p); };
        };
        let _ma_body_end = p.pos;
        // Save to depth-indexed globals (max 8 arms)
        let _ma_idx = len(arms);
        if _ma_idx == 0 { let __g_ma0_pat = pattern; let __g_ma0_bs = _ma_body_start; let __g_ma0_be = _ma_body_end; };
        if _ma_idx == 1 { let __g_ma1_pat = pattern; let __g_ma1_bs = _ma_body_start; let __g_ma1_be = _ma_body_end; };
        if _ma_idx == 2 { let __g_ma2_pat = pattern; let __g_ma2_bs = _ma_body_start; let __g_ma2_be = _ma_body_end; };
        if _ma_idx == 3 { let __g_ma3_pat = pattern; let __g_ma3_bs = _ma_body_start; let __g_ma3_be = _ma_body_end; };
        let __g_ma_tokens = p.tokens;
        push(arms, MatchArm { pattern: pattern, bindings: bindings, body: body });
        if is_symbol_tok(peek(p), ",") { advance(p); };
    };
    expect_symbol(p, "}");
    return Expr::MatchExpr { subject: subject, arms: arms };
}

// ── Statement parsing ────────────────────────────────────────────

// Explicit stack for recursive parse_block (ASM VM has no scoping)
let _pb_stack = [];

fn parse_block(p) {
    expect_symbol(p, "{");
    let _pb_stmts = [];
    while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) && _g_parse_error == 0 {
        // Save before recursive parse_stmt (which may call parse_block)
        push(_pb_stack, _pb_stmts);
        let _pb_item = parse_stmt(p);
        let _pb_stmts = pop(_pb_stack);
        push(_pb_stmts, _pb_item);
    };
    expect_symbol(p, "}");
    return _pb_stmts;
}

pub fn parse_stmt(p) {
    let tok = peek(p);

    // use path;
    if is_keyword_tok(tok, "use") {
        advance(p);
        let path = expect_ident(p);
        // Consume dotted path: use a.b.c
        while is_symbol_tok(peek(p), ".") {
            advance(p);
            let segment = expect_ident(p);
            let path = path + "." + segment;
        };
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::UseStmt { path: path };
    };

    // let name = expr;
    if is_keyword_tok(tok, "let") {
        advance(p);
        let _ps_lname = expect_ident(p);
        expect_symbol(p, "=");
        let _ps_lval = parse_expr(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::LetStmt { name: _ps_lname, value: _ps_lval };
    };

    // fn name(params) { body }
    if is_keyword_tok(tok, "fn") || is_keyword_tok(tok, "pub") {
        if is_keyword_tok(tok, "pub") {
            advance(p); // skip pub
        };
        advance(p); // skip fn
        let _ps_fn_name = expect_ident(p);
        expect_symbol(p, "(");
        let _ps_fn_params = [];
        while !is_symbol_tok(peek(p), ")") && !is_eof(peek(p)) {
            push(_ps_fn_params, expect_ident(p));
            if is_symbol_tok(peek(p), ",") { advance(p); };
        };
        expect_symbol(p, ")");
        let _ps_fn_body = parse_block(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::FnDef { name: _ps_fn_name, params: _ps_fn_params, body: _ps_fn_body };
    };

    // if cond { ... } else { ... }
    if is_keyword_tok(tok, "if") {
        advance(p);
        let _ps_cond = parse_expr(p);
        let _ps_then = parse_block(p);
        if is_keyword_tok(peek(p), "else") {
            advance(p);
            // Save cond+then before parsing else (inner if-stmt overwrites _ps_*)
            push(_pb_stack, _ps_cond);
            push(_pb_stack, _ps_then);
            if is_keyword_tok(peek(p), "if") {
                // else if → parse as single-stmt else block
                let _ps_elif = parse_stmt(p);
                let _ps_then = pop(_pb_stack);
                let _ps_cond = pop(_pb_stack);
                return Stmt::IfStmt { cond: _ps_cond, then_block: _ps_then, else_block: [_ps_elif] };
            } else {
                let _ps_else_blk = parse_block(p);
                let _ps_then = pop(_pb_stack);
                let _ps_cond = pop(_pb_stack);
                if is_symbol_tok(peek(p), ";") { advance(p); };
                return Stmt::IfStmt { cond: _ps_cond, then_block: _ps_then, else_block: _ps_else_blk };
            };
        };
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::IfStmt { cond: _ps_cond, then_block: _ps_then, else_block: [] };
    };

    // while cond { ... }
    if is_keyword_tok(tok, "while") {
        advance(p);
        // Save token range for condition (re-parsed in semantic to avoid dict corruption)
        let _ps_wc_start = p.pos;
        let _ps_wcond = parse_expr(p);
        let _ps_wc_end = p.pos;
        push(_pb_stack, _ps_wc_start);
        push(_pb_stack, _ps_wc_end);
        let _ps_wbody = parse_block(p);
        let _ps_wc_end = pop(_pb_stack);
        let _ps_wc_start = pop(_pb_stack);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::WhileStmt { cond: _ps_wcond, body: _ps_wbody, cond_start: _ps_wc_start, cond_end: _ps_wc_end, tokens: p.tokens };
    };

    // for var in expr { ... }
    if is_keyword_tok(tok, "for") {
        advance(p);
        let _ps_fvar = peek(p).text;
        advance(p);
        // Save var name to individual globals (arrays unsafe — heap overlap)
        if _g_for_depth == 0 { let __g_fv0 = _ps_fvar; };
        if _g_for_depth == 1 { let __g_fv1 = _ps_fvar; };
        if _g_for_depth == 2 { let __g_fv2 = _ps_fvar; };
        if _g_for_depth == 3 { let __g_fv3 = _ps_fvar; };
        // expect "in" keyword
        advance(p);
        // Save iter token range (iter Expr gets corrupted by inner ForStmt dicts)
        let _ps_fi_start = p.pos;
        let _ps_fiter = parse_expr(p);
        let _ps_fi_end = p.pos;
        if _g_for_depth == 0 { let __g_fi0s = _ps_fi_start; let __g_fi0e = _ps_fi_end; };
        if _g_for_depth == 1 { let __g_fi1s = _ps_fi_start; let __g_fi1e = _ps_fi_end; };
        if _g_for_depth == 2 { let __g_fi2s = _ps_fi_start; let __g_fi2e = _ps_fi_end; };
        if _g_for_depth == 3 { let __g_fi3s = _ps_fi_start; let __g_fi3e = _ps_fi_end; };
        let __g_fi_tokens = p.tokens;
        let _g_for_depth = _g_for_depth + 1;
        let _ps_fbody = parse_block(p);
        let _g_for_depth = _g_for_depth - 1;
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::ForStmt { var: _ps_fvar, iter: _ps_fiter, body: _ps_fbody };
    };

    // return expr;
    if is_keyword_tok(tok, "return") {
        advance(p);
        let _ps_rval = parse_expr(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::ReturnStmt { value: _ps_rval };
    };

    // emit expr;
    if is_keyword_tok(tok, "emit") {
        advance(p);
        let _ps_expr = parse_expr(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::EmitStmt { expr: _ps_expr };
    };

    // try { ... } catch { ... }
    if is_keyword_tok(tok, "try") {
        advance(p);
        let _ps_try = parse_block(p);
        // expect "catch"
        if is_keyword_tok(peek(p), "catch") {
            advance(p);
        };
        let _ps_catch = parse_block(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::TryCatch { try_block: _ps_try, catch_block: _ps_catch };
    };

    // break;
    if is_keyword_tok(tok, "break") {
        advance(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::BreakStmt;
    };

    // continue;
    if is_keyword_tok(tok, "continue") {
        advance(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::ContinueStmt;
    };

    // match subject { arms }
    if is_keyword_tok(tok, "match") {
        let mexpr = parse_match_expr(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::MatchStmt { subject: mexpr, arms: [] };
    };

    // type Name { fields }
    if is_keyword_tok(tok, "type") {
        advance(p);
        let name = expect_ident(p);
        expect_symbol(p, "{");
        let fields = [];
        while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
            let fname = expect_ident(p);
            expect_symbol(p, ":");
            // Parse type annotation (may be complex like Vec[Expr])
            let tname = expect_ident(p);
            if is_symbol_tok(peek(p), "[") {
                // Skip generic params: Vec[Type]
                advance(p);
                let depth = 1;
                while depth > 0 && !is_eof(peek(p)) {
                    if is_symbol_tok(peek(p), "[") { let depth = depth + 1; };
                    if is_symbol_tok(peek(p), "]") { let depth = depth - 1; };
                    if depth > 0 { advance(p); };
                };
                expect_symbol(p, "]");
            };
            push(fields, Field { name: fname, type_name: tname });
            if is_symbol_tok(peek(p), ",") { advance(p); };
        };
        expect_symbol(p, "}");
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::TypeDef { name: name, fields: fields };
    };

    // union Name { variants }
    if is_keyword_tok(tok, "union") {
        advance(p);
        let name = expect_ident(p);
        expect_symbol(p, "{");
        let variants = [];
        while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
            let vname = expect_ident(p);
            let vfields = [];
            if is_symbol_tok(peek(p), "{") {
                advance(p);
                while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
                    let fname = expect_ident(p);
                    expect_symbol(p, ":");
                    let tname = expect_ident(p);
                    // Skip generic params
                    if is_symbol_tok(peek(p), "[") {
                        advance(p);
                        let depth = 1;
                        while depth > 0 && !is_eof(peek(p)) {
                            if is_symbol_tok(peek(p), "[") { let depth = depth + 1; };
                            if is_symbol_tok(peek(p), "]") { let depth = depth - 1; };
                            if depth > 0 { advance(p); };
                        };
                        expect_symbol(p, "]");
                    };
                    push(vfields, Field { name: fname, type_name: tname });
                    if is_symbol_tok(peek(p), ",") { advance(p); };
                };
                expect_symbol(p, "}");
            };
            push(variants, Variant { name: vname, fields: vfields });
            if is_symbol_tok(peek(p), ",") { advance(p); };
        };
        expect_symbol(p, "}");
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::UnionDef { name: name, variants: variants };
    };

    // Check for field assignment: name.field = expr;
    // Must check before expression statement since it starts with ident
    match tok.kind {
        TokenKind::Ident { name } => {
            // Look ahead for assignment patterns
            if p.pos + 1 < len(p.tokens) {
                let next1 = p.tokens[p.pos + 1];

                // Bare assignment: name = expr → desugar to LetStmt
                if is_symbol_tok(next1, "=") {
                    // Check it's not == (comparison)
                    if p.pos + 2 < len(p.tokens) {
                        let next2 = p.tokens[p.pos + 2];
                        if !is_symbol_tok(next2, "=") {
                            let _ps_aname = name;  // save BEFORE parse_expr (overwrites match binding)
                            advance(p); // consume name
                            advance(p); // consume =
                            let _ps_aval = parse_expr(p);
                            if is_symbol_tok(peek(p), ";") { advance(p); };
                            return Stmt::LetStmt { name: _ps_aname, value: _ps_aval };
                        };
                    };
                };

                // Field assignment: name.field = expr
                if is_symbol_tok(next1, ".") {
                    if p.pos + 2 < len(p.tokens) {
                        let next2 = p.tokens[p.pos + 2];
                        if is_ident_tok(next2) {
                            if p.pos + 3 < len(p.tokens) {
                                let next3 = p.tokens[p.pos + 3];
                                if is_symbol_tok(next3, "=") {
                                    advance(p); // consume name
                                    advance(p); // consume .
                                    let field = expect_ident(p);
                                    expect_symbol(p, "=");
                                    let value = parse_expr(p);
                                    if is_symbol_tok(peek(p), ";") { advance(p); };
                                    return Stmt::FieldAssign { object: name, field: field, value: value };
                                };
                            };
                        };
                    };
                };
            };
        },
        _ => {},
    };

    // Expression statement
    let expr = parse_expr(p);
    if is_symbol_tok(peek(p), ";") { advance(p); };
    return Stmt::ExprStmt { expr: expr };
}

// ── Top-level parse ──────────────────────────────────────────────

pub fn parse(tokens) {
    let _g_parse_error = 0;
    let _pp = new_parser(tokens);
    let _pp_program = [];
    while !is_eof(peek(_pp)) && _g_parse_error == 0 {
        push(_pp_program, parse_stmt(_pp));
    };
    return _pp_program;
}
