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
}

union Stmt {
    LetStmt { name: Str, value: Expr },
    ExprStmt { expr: Expr },
    FnDef { name: Str, params: Vec[Str], body: Vec[Stmt] },
    IfStmt { cond: Expr, then_block: Vec[Stmt], else_block: Vec[Stmt] },
    WhileStmt { cond: Expr, body: Vec[Stmt] },
    ReturnStmt { value: Expr },
    EmitStmt { expr: Expr },
    TypeDef { name: Str, fields: Vec[Field] },
    UnionDef { name: Str, variants: Vec[Variant] },
    BreakStmt,
    ContinueStmt,
    UseStmt { path: Str },
    MatchStmt { subject: Expr, arms: Vec[MatchArm] },
    FieldAssign { object: Str, field: Str, value: Expr },
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
            };
        },
        _ => {
            emit "Parse error: expected symbol '" + sym + "'";
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
            advance(p);
            return Expr::NumLit { value: 0 };
        },
        TokenKind::Ident { name } => {
            advance(p);
            let result = Expr::Ident { name: name };

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
                    let result = Expr::StructLit { path: name + "::" + member, fields: fields };
                } else {
                    let result = Expr::PathExpr { base: name, member: member };
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
                    let result = Expr::StructLit { path: name, fields: fields };
                };
            };

            // Postfix chain: .field, [index], (args) — can repeat
            // Use is_postfix helper to check loop condition without mutable flag
            while is_symbol_tok(peek(p), ".") || is_symbol_tok(peek(p), "[") || is_symbol_tok(peek(p), "(") {
                if is_symbol_tok(peek(p), ".") {
                    advance(p);
                    let field = expect_ident(p);
                    let result = Expr::FieldAccess { object: result, field: field };
                };
                if is_symbol_tok(peek(p), "[") {
                    advance(p);
                    let index = parse_expr(p);
                    expect_symbol(p, "]");
                    let result = Expr::Index { object: result, index: index };
                };
                if is_symbol_tok(peek(p), "(") {
                    advance(p);
                    let args = [];
                    while !is_symbol_tok(peek(p), ")") && !is_eof(peek(p)) {
                        push(args, parse_expr(p));
                        if is_symbol_tok(peek(p), ",") { advance(p); };
                    };
                    expect_symbol(p, ")");
                    let result = Expr::Call { callee: result, args: args };
                };
            };

            return result;
        },
        TokenKind::Symbol { ch } => {
            // Unary NOT: !expr
            if ch == "!" {
                advance(p);
                let expr = parse_primary(p);
                return Expr::UnaryNot { expr: expr };
            };
            // Parenthesized expression
            if ch == "(" {
                advance(p);
                let expr = parse_expr(p);
                expect_symbol(p, ")");
                return expr;
            };
            // Array literal: [expr, expr, ...]
            if ch == "[" {
                advance(p);
                let items = [];
                while !is_symbol_tok(peek(p), "]") && !is_eof(peek(p)) {
                    push(items, parse_expr(p));
                    if is_symbol_tok(peek(p), ",") {
                        advance(p);
                    };
                };
                expect_symbol(p, "]");
                return Expr::ArrayLit { items: items };
            };
            // Molecular literal: { S=1 R=2 V=128 A=128 T=3 }
            if ch == "{" {
                advance(p);
                let s = 0;
                let r = 0;
                let v = 0;
                let a = 0;
                let t = 0;
                while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
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
            advance(p);
            return Expr::NumLit { value: 0 };
        },
        _ => {
            emit "Parse error: unexpected token";
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
                let _pep_op = ch;
                let _pep_saved = _pep_lhs;
                let _pep_rhs = parse_expr_prec(p, _pep_prec + 1);
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

        // Wildcard: _
        if is_ident_tok(ptok) {
            let pname = expect_ident(p);
            if pname == "_" {
                let pattern = "_";
            } else {
                // Could be Name::Variant { ... } or just Name
                let pattern = pname;
                if is_symbol_tok(peek(p), "::") {
                    advance(p); // consume ::
                    let variant = expect_ident(p);
                    let pattern = pname + "::" + variant;
                };
                // Parse bindings: { field1, field2 } or { field1 }
                if is_symbol_tok(peek(p), "{") {
                    advance(p); // consume {
                    while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
                        push(bindings, expect_ident(p));
                        if is_symbol_tok(peek(p), ",") { advance(p); };
                    };
                    expect_symbol(p, "}");
                };
            };
        };

        expect_symbol(p, "=>");
        let body = [];
        // Arm body: { stmts }
        if is_symbol_tok(peek(p), "{") {
            let body = parse_block(p);
        } else {
            // Single expression
            push(body, Stmt::ExprStmt { expr: parse_expr(p) });
            if is_symbol_tok(peek(p), ";") { advance(p); };
        };
        push(arms, MatchArm { pattern: pattern, bindings: bindings, body: body });
        if is_symbol_tok(peek(p), ",") { advance(p); };
    };
    expect_symbol(p, "}");
    return Expr::MatchExpr { subject: subject, arms: arms };
}

// ── Statement parsing ────────────────────────────────────────────

fn parse_block(p) {
    expect_symbol(p, "{");
    let stmts = [];
    while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
        push(stmts, parse_stmt(p));
    };
    expect_symbol(p, "}");
    return stmts;
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
        let name = expect_ident(p);
        expect_symbol(p, "=");
        let value = parse_expr(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::LetStmt { name: name, value: value };
    };

    // fn name(params) { body }
    if is_keyword_tok(tok, "fn") || is_keyword_tok(tok, "pub") {
        if is_keyword_tok(tok, "pub") {
            advance(p); // skip pub
        };
        advance(p); // skip fn
        let name = expect_ident(p);
        expect_symbol(p, "(");
        let params = [];
        while !is_symbol_tok(peek(p), ")") && !is_eof(peek(p)) {
            push(params, expect_ident(p));
            if is_symbol_tok(peek(p), ",") { advance(p); };
        };
        expect_symbol(p, ")");
        let body = parse_block(p);
        return Stmt::FnDef { name: name, params: params, body: body };
    };

    // if cond { ... } else { ... }
    if is_keyword_tok(tok, "if") {
        advance(p);
        let cond = parse_expr(p);
        let then_block = parse_block(p);
        let else_block = [];
        if is_keyword_tok(peek(p), "else") {
            advance(p);
            let else_block = parse_block(p);
        };
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::IfStmt { cond: cond, then_block: then_block, else_block: else_block };
    };

    // while cond { ... }
    if is_keyword_tok(tok, "while") {
        advance(p);
        let cond = parse_expr(p);
        let body = parse_block(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::WhileStmt { cond: cond, body: body };
    };

    // return expr;
    if is_keyword_tok(tok, "return") {
        advance(p);
        let value = parse_expr(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::ReturnStmt { value: value };
    };

    // emit expr;
    if is_keyword_tok(tok, "emit") {
        advance(p);
        let expr = parse_expr(p);
        if is_symbol_tok(peek(p), ";") { advance(p); };
        return Stmt::EmitStmt { expr: expr };
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
        return Stmt::UnionDef { name: name, variants: variants };
    };

    // Check for field assignment: name.field = expr;
    // Must check before expression statement since it starts with ident
    match tok.kind {
        TokenKind::Ident { name } => {
            // Look ahead for name.field = expr pattern
            if p.pos + 2 < len(p.tokens) {
                let next1 = p.tokens[p.pos + 1];
                let next2 = p.tokens[p.pos + 2];
                if is_symbol_tok(next1, ".") && is_ident_tok(next2) {
                    // Check if it's assignment: name.field = expr
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
    let p = new_parser(tokens);
    let program = [];
    while !is_eof(peek(p)) {
        push(program, parse_stmt(p));
    };
    return program;
}
