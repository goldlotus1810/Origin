//! # syntax — Cú pháp chính thức của Olang
//!
//! Định nghĩa ngữ pháp (grammar), AST, và recursive descent parser.
//!
//! ## Ngữ pháp Olang (BNF)
//!
//! ```text
//! program       = stmt*
//!
//! stmt          = let_stmt
//!               | emit_stmt
//!               | if_stmt
//!               | loop_stmt
//!               | fn_stmt
//!               | command_stmt
//!               | expr_stmt
//!
//! let_stmt      = 'let' IDENT '=' expr ';'
//! emit_stmt     = 'emit' expr ';'
//! if_stmt       = 'if' expr block ('else' block)?
//! loop_stmt     = 'loop' INT block
//! fn_stmt       = 'fn' IDENT '(' params? ')' block
//! command_stmt  = COMMAND ';'?
//! expr_stmt     = expr ';'
//!
//! block         = '{' stmt* '}'
//!
//! expr          = compose_expr
//! compose_expr  = rel_expr ('∘' rel_expr)*
//! rel_expr      = primary (REL_OP (primary | '?'))?
//! primary       = IDENT
//!               | INT
//!               | '(' expr ')'
//!               | IDENT '(' args? ')'    /* call */
//!
//! params        = IDENT (',' IDENT)*
//! args          = expr (',' expr)*
//!
//! REL_OP        = '∈' | '⊂' | '≡' | '⊥' | '→' | '≈' | '←' | '∪' | '∩' | '∂'
//! ```
//!
//! ## Backwards Compatibility
//!
//! Nếu input là single expression (không có `;`, `let`, `fn`, `if`, `loop`),
//! parser tự động wrap thành single-expr program.
//!
//! `fire ∘ water` → `[Stmt::Expr(Compose(fire, water))]`

extern crate alloc;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::alphabet::{Lexer, RelOp, Token};

// ─────────────────────────────────────────────────────────────────────────────
// AST — Abstract Syntax Tree
// ─────────────────────────────────────────────────────────────────────────────

/// Statement — đơn vị thực thi trong chương trình Olang.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `let name = expr;` — gán biến cục bộ
    Let {
        /// Tên biến
        name: String,
        /// Biểu thức giá trị
        value: Expr,
    },

    /// `emit expr;` — xuất chain ra caller
    Emit(Expr),

    /// `if cond { then } else { else_ }` — điều kiện
    If {
        /// Biểu thức điều kiện (non-empty = true)
        cond: Expr,
        /// Block thực thi khi true
        then_block: Vec<Stmt>,
        /// Block thực thi khi false (optional)
        else_block: Option<Vec<Stmt>>,
    },

    /// `loop N { body }` — lặp N lần
    Loop {
        /// Số lần lặp
        count: u32,
        /// Block lặp
        body: Vec<Stmt>,
    },

    /// `fn name(params) { body }` — định nghĩa hàm
    FnDef {
        /// Tên hàm
        name: String,
        /// Danh sách tham số
        params: Vec<String>,
        /// Thân hàm
        body: Vec<Stmt>,
    },

    /// Expression statement: `expr;`
    Expr(Expr),

    /// System command: `dream;` `stats;` etc.
    Command(String),
}

/// Expression — biểu thức trong Olang. Mọi expression evaluate → MolecularChain.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Identifier: tên biến, alias node, hoặc emoji
    Ident(String),

    /// Integer literal (dùng trong loop count)
    Int(u32),

    /// `a ∘ b` — Compose / LCA
    Compose(Box<Expr>, Box<Expr>),

    /// `a REL b` — tạo relation edge giữa a và b
    RelEdge {
        /// Bên trái
        lhs: Box<Expr>,
        /// Toán tử quan hệ
        op: RelOp,
        /// Bên phải
        rhs: Box<Expr>,
    },

    /// `a REL ?` — query: tìm nodes có relation với a
    RelQuery {
        /// Subject
        subject: Box<Expr>,
        /// Toán tử quan hệ
        op: RelOp,
    },

    /// `name(args)` — gọi hàm
    Call {
        /// Tên hàm
        name: String,
        /// Arguments
        args: Vec<Expr>,
    },

    /// `(expr)` — nhóm / parenthesized
    Group(Box<Expr>),
}

// ─────────────────────────────────────────────────────────────────────────────
// ParseError
// ─────────────────────────────────────────────────────────────────────────────

/// Lỗi cú pháp.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Mô tả lỗi
    pub message: String,
}

impl ParseError {
    fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser — recursive descent
// ─────────────────────────────────────────────────────────────────────────────

/// Recursive descent parser cho Olang.
pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    _src: &'a str,
}

impl<'a> Parser<'a> {
    /// Tạo parser mới từ source text.
    pub fn new(src: &'a str) -> Self {
        let tokens = Lexer::tokenize_all(src);
        Self {
            tokens,
            pos: 0,
            _src: src,
        }
    }

    /// Parse toàn bộ source → Vec<Stmt>.
    ///
    /// Nếu input là single expression → wrap thành `[Stmt::Expr(expr)]`.
    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        // Empty program
        if self.tokens.is_empty() {
            return Ok(Vec::new());
        }

        // Detect: single expression (no semicolons, no keywords at top level)?
        if self.is_single_expr() {
            let expr = self.parse_expr()?;
            return Ok(alloc::vec![Stmt::Expr(expr)]);
        }

        let mut stmts = Vec::new();
        while !self.at_eof() && !self.check(&Token::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // ── Statement parsing ───────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Let => self.parse_let(),
            Token::Emit => self.parse_emit(),
            Token::If => self.parse_if(),
            Token::Loop => self.parse_loop(),
            Token::Fn => self.parse_fn(),
            Token::Command(_) => self.parse_command(),
            _ => self.parse_expr_stmt(),
        }
    }

    /// `let name = expr;`
    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Let)?;
        let name = self.expect_ident()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(&Token::Semi)?;
        Ok(Stmt::Let { name, value })
    }

    /// `emit expr;`
    fn parse_emit(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Emit)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::Semi)?;
        Ok(Stmt::Emit(expr))
    }

    /// `if expr { stmts } (else { stmts })?`
    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::If)?;
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.check(&Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        // Optional trailing semicolon
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::If {
            cond,
            then_block,
            else_block,
        })
    }

    /// `loop N { stmts }`
    fn parse_loop(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Loop)?;
        let count = self.expect_int()?;
        let body = self.parse_block()?;
        // Optional trailing semicolon
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::Loop { count, body })
    }

    /// `fn name(params) { stmts }`
    fn parse_fn(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Fn)?;
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;
        let body = self.parse_block()?;
        // Optional trailing semicolon
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::FnDef { name, params, body })
    }

    /// command ';'?
    fn parse_command(&mut self) -> Result<Stmt, ParseError> {
        let cmd = match self.advance() {
            Token::Command(s) => s,
            _ => return Err(ParseError::new("Expected command")),
        };
        // Optional semicolon
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::Command(cmd))
    }

    /// expr ';'
    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;
        // Semicolon required for expr statements in multi-stmt context
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::Expr(expr))
    }

    /// `{ stmt* }`
    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&Token::RBrace) && !self.at_eof() {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    /// params: IDENT (',' IDENT)*
    fn parse_params(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            params.push(self.expect_ident()?);
            while self.check(&Token::Comma) {
                self.advance();
                params.push(self.expect_ident()?);
            }
        }
        Ok(params)
    }

    // ── Expression parsing ──────────────────────────────────────────────────

    /// expr = compose_expr
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_compose_expr()
    }

    /// compose_expr = rel_expr ('∘' rel_expr)*
    fn parse_compose_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_rel_expr()?;

        while self.check_rel(RelOp::Compose) {
            self.advance(); // consume ∘
            let right = self.parse_rel_expr()?;
            left = Expr::Compose(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// rel_expr = primary (REL_OP (primary | '?'))?
    fn parse_rel_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_primary()?;

        // Check for relation operator (not ∘, which is handled in compose_expr)
        if let Token::Rel(rel_op) = self.peek() {
            if rel_op != RelOp::Compose {
                self.advance(); // consume relation op
                let op = rel_op;

                if self.check(&Token::Wild) {
                    self.advance(); // consume ?
                    return Ok(Expr::RelQuery {
                        subject: Box::new(left),
                        op,
                    });
                }

                let right = self.parse_primary()?;
                return Ok(Expr::RelEdge {
                    lhs: Box::new(left),
                    op,
                    rhs: Box::new(right),
                });
            }
        }

        Ok(left)
    }

    /// primary = IDENT | IDENT '(' args ')' | INT | '(' expr ')'
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Check for function call: name(args)
                if self.check(&Token::LParen) {
                    self.advance(); // (
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    return Ok(Expr::Call { name, args });
                }

                Ok(Expr::Ident(name))
            }

            Token::Int(val) => {
                self.advance();
                Ok(Expr::Int(val))
            }

            Token::LParen => {
                self.advance(); // (
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::Group(Box::new(expr)))
            }

            Token::Wild => {
                // Standalone ? in some positions (e.g. `? → water`)
                self.advance();
                Ok(Expr::Ident("?".to_string()))
            }

            other => Err(ParseError::new(&alloc::format!(
                "Unexpected token: {:?}",
                other
            ))),
        }
    }

    /// args: expr (',' expr)*
    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if !self.check(&Token::RParen) {
            args.push(self.parse_expr()?);
            while self.check(&Token::Comma) {
                self.advance();
                args.push(self.parse_expr()?);
            }
        }
        Ok(args)
    }

    // ── Helpers ─────────────────────────────────────────────────────────────

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn check(&self, expected: &Token) -> bool {
        core::mem::discriminant(&self.peek()) == core::mem::discriminant(expected)
    }

    fn check_rel(&self, op: RelOp) -> bool {
        matches!(self.peek(), Token::Rel(o) if o == op)
    }

    fn at_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn expect(&mut self, expected: &Token) -> Result<Token, ParseError> {
        if self.check(expected) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(&alloc::format!(
                "Expected {:?}, got {:?}",
                expected,
                self.peek()
            )))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            other => Err(ParseError::new(&alloc::format!(
                "Expected identifier, got {:?}",
                other
            ))),
        }
    }

    fn expect_int(&mut self) -> Result<u32, ParseError> {
        match self.advance() {
            Token::Int(n) => Ok(n),
            other => Err(ParseError::new(&alloc::format!(
                "Expected integer, got {:?}",
                other
            ))),
        }
    }

    /// Detect: chỉ có 1 expression, không có statement keywords hoặc commands?
    fn is_single_expr(&self) -> bool {
        // If there are no semicolons, no statement keywords, no commands → single expr
        !self.tokens.iter().any(|t| {
            matches!(
                t,
                Token::Semi
                    | Token::Let
                    | Token::Fn
                    | Token::If
                    | Token::Loop
                    | Token::Emit
                    | Token::Command(_)
            )
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Convenience
// ─────────────────────────────────────────────────────────────────────────────

/// Parse source text → AST.
pub fn parse(src: &str) -> Result<Vec<Stmt>, ParseError> {
    Parser::new(src).parse_program()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    // ── Single expressions (backwards compatible) ───────────────────────────

    #[test]
    fn parse_simple_ident() {
        let stmts = parse("fire").unwrap();
        assert_eq!(stmts, vec![Stmt::Expr(Expr::Ident("fire".into()))]);
    }

    #[test]
    fn parse_emoji_ident() {
        let stmts = parse("🔥").unwrap();
        assert_eq!(stmts, vec![Stmt::Expr(Expr::Ident("🔥".into()))]);
    }

    #[test]
    fn parse_compose() {
        let stmts = parse("fire ∘ water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::Compose(
                Box::new(Expr::Ident("fire".into())),
                Box::new(Expr::Ident("water".into())),
            ))]
        );
    }

    #[test]
    fn parse_triple_compose() {
        let stmts = parse("fire ∘ water ∘ earth").unwrap();
        // Left-associative: (fire ∘ water) ∘ earth
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::Compose(
                Box::new(Expr::Compose(
                    Box::new(Expr::Ident("fire".into())),
                    Box::new(Expr::Ident("water".into())),
                )),
                Box::new(Expr::Ident("earth".into())),
            ))]
        );
    }

    #[test]
    fn parse_relation_query() {
        let stmts = parse("🔥 ∈ ?").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::RelQuery {
                subject: Box::new(Expr::Ident("🔥".into())),
                op: RelOp::Member,
            })]
        );
    }

    #[test]
    fn parse_relation_edge() {
        let stmts = parse("fire → water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::RelEdge {
                lhs: Box::new(Expr::Ident("fire".into())),
                op: RelOp::Causes,
                rhs: Box::new(Expr::Ident("water".into())),
            })]
        );
    }

    #[test]
    fn parse_context_query() {
        let stmts = parse("bank ∂ finance").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::RelEdge {
                lhs: Box::new(Expr::Ident("bank".into())),
                op: RelOp::Context,
                rhs: Box::new(Expr::Ident("finance".into())),
            })]
        );
    }

    #[test]
    fn parse_reverse_query() {
        let stmts = parse("? → water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::RelEdge {
                lhs: Box::new(Expr::Ident("?".into())),
                op: RelOp::Causes,
                rhs: Box::new(Expr::Ident("water".into())),
            })]
        );
    }

    #[test]
    fn parse_command_standalone() {
        let stmts = parse("dream").unwrap();
        assert_eq!(stmts, vec![Stmt::Command("dream".into())]);
    }

    // ── Let binding ─────────────────────────────────────────────────────────

    #[test]
    fn parse_let_binding() {
        let stmts = parse("let steam = fire ∘ water;").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Let {
                name: "steam".into(),
                value: Expr::Compose(
                    Box::new(Expr::Ident("fire".into())),
                    Box::new(Expr::Ident("water".into())),
                ),
            }]
        );
    }

    // ── If/else ─────────────────────────────────────────────────────────────

    #[test]
    fn parse_if_simple() {
        let stmts = parse("if fire { emit fire; }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::If {
                cond: Expr::Ident("fire".into()),
                then_block: vec![Stmt::Emit(Expr::Ident("fire".into()))],
                else_block: None,
            }]
        );
    }

    #[test]
    fn parse_if_else() {
        let stmts = parse("if fire { emit fire; } else { emit water; }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::If {
                cond: Expr::Ident("fire".into()),
                then_block: vec![Stmt::Emit(Expr::Ident("fire".into()))],
                else_block: Some(vec![Stmt::Emit(Expr::Ident("water".into()))]),
            }]
        );
    }

    // ── Loop ────────────────────────────────────────────────────────────────

    #[test]
    fn parse_loop_basic() {
        let stmts = parse("loop 3 { emit fire; }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Loop {
                count: 3,
                body: vec![Stmt::Emit(Expr::Ident("fire".into()))],
            }]
        );
    }

    // ── Fn definition ───────────────────────────────────────────────────────

    #[test]
    fn parse_fn_def() {
        let stmts = parse("fn blend(a, b) { emit a ∘ b; }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::FnDef {
                name: "blend".into(),
                params: alloc::vec!["a".into(), "b".into()],
                body: vec![Stmt::Emit(Expr::Compose(
                    Box::new(Expr::Ident("a".into())),
                    Box::new(Expr::Ident("b".into())),
                ))],
            }]
        );
    }

    #[test]
    fn parse_fn_no_params() {
        let stmts = parse("fn greet() { emit fire; }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::FnDef {
                name: "greet".into(),
                params: Vec::new(),
                body: vec![Stmt::Emit(Expr::Ident("fire".into()))],
            }]
        );
    }

    // ── Function call ───────────────────────────────────────────────────────

    #[test]
    fn parse_fn_call() {
        let stmts = parse("emit blend(fire, water);").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Emit(Expr::Call {
                name: "blend".into(),
                args: vec![
                    Expr::Ident("fire".into()),
                    Expr::Ident("water".into()),
                ],
            })]
        );
    }

    // ── Multi-statement programs ────────────────────────────────────────────

    #[test]
    fn parse_multi_stmt() {
        let src = "let x = fire; let y = water; emit x ∘ y;";
        let stmts = parse(src).unwrap();
        assert_eq!(stmts.len(), 3);
        assert!(matches!(stmts[0], Stmt::Let { .. }));
        assert!(matches!(stmts[1], Stmt::Let { .. }));
        assert!(matches!(stmts[2], Stmt::Emit(_)));
    }

    #[test]
    fn parse_command_in_program() {
        let stmts = parse("emit fire; dream;").unwrap();
        assert_eq!(stmts.len(), 2);
        assert!(matches!(stmts[0], Stmt::Emit(_)));
        assert!(matches!(stmts[1], Stmt::Command(ref s) if s == "dream"));
    }

    // ── Vietnamese ──────────────────────────────────────────────────────────

    #[test]
    fn parse_vietnamese_compose() {
        let stmts = parse("lửa ∘ nước").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::Compose(
                Box::new(Expr::Ident("lửa".into())),
                Box::new(Expr::Ident("nước".into())),
            ))]
        );
    }

    // ── Grouping ────────────────────────────────────────────────────────────

    #[test]
    fn parse_grouped_expr() {
        let stmts = parse("(fire ∘ water) → earth").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::RelEdge {
                lhs: Box::new(Expr::Group(Box::new(Expr::Compose(
                    Box::new(Expr::Ident("fire".into())),
                    Box::new(Expr::Ident("water".into())),
                )))),
                op: RelOp::Causes,
                rhs: Box::new(Expr::Ident("earth".into())),
            })]
        );
    }

    // ── Error cases ─────────────────────────────────────────────────────────

    #[test]
    fn parse_empty_is_ok() {
        let stmts = parse("").unwrap();
        assert!(stmts.is_empty());
    }

    #[test]
    fn parse_unclosed_paren() {
        let result = parse("(fire");
        assert!(result.is_err());
    }

    #[test]
    fn parse_let_missing_semi() {
        let result = parse("let x = fire");
        // Missing semicolon → error in multi-stmt mode
        // But since there's 'let', it's NOT single-expr mode
        assert!(result.is_err());
    }
}
