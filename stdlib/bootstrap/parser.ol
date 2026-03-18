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
    BinOp { op: Str, lhs: Expr, rhs: Expr },
    Call { callee: Expr, args: Vec[Expr] },
    FieldAccess { object: Expr, field: Str },
    MolLiteral { s: Num, r: Num, v: Num, a: Num, t: Num },
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
}

type Field {
    name: Str,
    type_name: Str,
}

type Variant {
    name: Str,
    fields: Vec[Field],
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
        TokenKind::Ident { name } => {
            advance(p);
            // Check for function call: name(...)
            if is_symbol_tok(peek(p), "(") {
                advance(p); // consume (
                let args = [];
                while !is_symbol_tok(peek(p), ")") && !is_eof(peek(p)) {
                    push(args, parse_expr(p));
                    if is_symbol_tok(peek(p), ",") {
                        advance(p);
                    };
                };
                expect_symbol(p, ")");
                return Expr::Call {
                    callee: Expr::Ident { name: name },
                    args: args,
                };
            };
            // Check for field access: name.field
            if is_symbol_tok(peek(p), ".") {
                advance(p); // consume .
                let field = expect_ident(p);
                return Expr::FieldAccess {
                    object: Expr::Ident { name: name },
                    field: field,
                };
            };
            return Expr::Ident { name: name };
        },
        TokenKind::Symbol { ch } => {
            // Parenthesized expression
            if ch == "(" {
                advance(p);
                let expr = parse_expr(p);
                expect_symbol(p, ")");
                return expr;
            };
            // Molecular literal: { S=1 R=2 V=128 A=128 T=3 }
            if ch == "{" {
                advance(p);
                let s = 0;
                let r = 0;
                let v = 128;
                let a = 128;
                let t = 3;
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
                return Expr::MolLiteral { s: s, r: r, v: v, a: a, t: t };
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
    let lhs = parse_primary(p);

    while is_binop(peek(p)) {
        let op_tok = peek(p);
        match op_tok.kind {
            TokenKind::Symbol { ch } => {
                let prec = precedence(ch);
                if prec < min_prec {
                    break;
                };
                advance(p); // consume operator
                let rhs = parse_expr_prec(p, prec + 1);
                let lhs = Expr::BinOp { op: ch, lhs: lhs, rhs: rhs };
            },
            _ => { break; },
        };
    };

    return lhs;
}

pub fn parse_expr(p) {
    return parse_expr_prec(p, 1);
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
        return Stmt::IfStmt { cond: cond, then_block: then_block, else_block: else_block };
    };

    // while cond { ... }
    if is_keyword_tok(tok, "while") {
        advance(p);
        let cond = parse_expr(p);
        let body = parse_block(p);
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

    // type Name { fields }
    if is_keyword_tok(tok, "type") {
        advance(p);
        let name = expect_ident(p);
        expect_symbol(p, "{");
        let fields = [];
        while !is_symbol_tok(peek(p), "}") && !is_eof(peek(p)) {
            let fname = expect_ident(p);
            expect_symbol(p, ":");
            let tname = expect_ident(p);
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
