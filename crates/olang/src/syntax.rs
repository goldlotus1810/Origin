//! # syntax — Cú pháp chính thức của Olang
//!
//! ## Ngữ pháp Olang (BNF)
//!
//! ```text
//! program       = stmt*
//!
//! stmt          = let_stmt | emit_stmt | if_stmt | loop_stmt
//!               | fn_stmt  | command_stmt | expr_stmt
//!
//! ── Keyword style ─────────────────────────────────────────
//! let_stmt      = 'let' IDENT '=' expr ';'
//! emit_stmt     = ('emit' | '○') expr ';'
//! if_stmt       = 'if' expr block ('else' block)?
//! loop_stmt     = ('loop' | '↻') INT block
//! fn_stmt       = 'fn' IDENT '(' params? ')' block
//! command_stmt  = COMMAND (STR)? ';'?
//!
//! ── Symbol style (tương đương) ────────────────────────────
//! define_stmt   = expr '≔' expr ';'             → Let
//! fn_def_sym    = expr '≔' '(' params ')' block → FnDef
//! implies_stmt  = expr '⇒' block ('⊥' block)?  → If
//! cycle_stmt    = '↻' INT block                 → Loop
//! circle_stmt   = '○' expr ';'                  → Emit
//!
//! ── Expressions ───────────────────────────────────────────
//! expr          = rel_chain
//! rel_chain     = compose (REL_OP (compose | '?'))*
//! compose       = arith ('∘' arith)*
//! arith         = primary (ARITH_OP primary)*
//! primary       = IDENT | INT | STR
//!               | '(' expr ')'
//!               | IDENT '(' args? ')'
//!               | '?' (wildcard)
//!
//! REL_OP        = ∈ ⊂ ≡ ⊥ → ≈ ← ∪ ∩ ∂ ∖ ↔ ⟳ ⚡ ∥
//! ARITH_OP      = + - × ÷
//! ```
//!
//! ## Chain Queries
//!
//! ```text
//! ○{🌞 → ? → 🌵}     → tìm X sao cho 🌞→X và X→🌵
//! ○{? ∈ L3_Thermo}    → tất cả nodes trong nhóm
//! ```

extern crate alloc;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::alphabet::{ArithOp, Lexer, PhysOp, RelOp, Token};

// ─────────────────────────────────────────────────────────────────────────────
// AST — Abstract Syntax Tree
// ─────────────────────────────────────────────────────────────────────────────

/// Statement — đơn vị thực thi.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Stmt {
    /// `let name = expr;` hoặc `name ≔ expr;`
    Let { name: String, value: Expr },

    /// `emit expr;` hoặc `○ expr;`
    Emit(Expr),

    /// `if cond { then } else { else_ }` hoặc `cond ⇒ { then } ⊥ { else_ }`
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },

    /// `loop N { body }` hoặc `↻ N { body }`
    Loop { count: u32, body: Vec<Stmt> },

    /// `fn name(params) { body }` hoặc `name ≔ (params) { body }`
    FnDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    /// Expression statement
    Expr(Expr),

    /// System command: `dream;` `stats;` `learn "text";`
    Command(String),

    /// Command with argument: `learn "text"`, `seed L0`
    CommandArg { name: String, arg: String },
}

/// Expression — mọi expression evaluate → MolecularChain.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Expr {
    /// Identifier: node alias, biến, emoji
    Ident(String),

    /// Integer literal
    Int(u32),

    /// String literal: "text"
    Str(String),

    /// `a ∘ b` — Compose / LCA
    Compose(Box<Expr>, Box<Expr>),

    /// `a REL b` — relation edge
    RelEdge {
        lhs: Box<Expr>,
        op: RelOp,
        rhs: Box<Expr>,
    },

    /// `a REL ?` — relation query
    RelQuery { subject: Box<Expr>, op: RelOp },

    /// `A → ? → B` — chain query (multi-hop path finding)
    Chain {
        head: Box<Expr>,
        steps: Vec<(RelOp, Expr)>,
    },

    /// `1 + 2`, `3 × 4` — arithmetic GIẢ THUYẾT (QT3: chưa chứng minh)
    Arith {
        lhs: Box<Expr>,
        op: ArithOp,
        rhs: Box<Expr>,
    },

    /// `a ⧺ b`, `a ⊖ b` — vật lý ĐÃ CHỨNG MINH (QT3)
    PhysOp {
        lhs: Box<Expr>,
        op: PhysOp,
        rhs: Box<Expr>,
    },

    /// `a == b` — sự thật chắc chắn (QT3)
    Truth {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    /// `name(args)` — function call
    Call { name: String, args: Vec<Expr> },

    /// `(expr)` — grouping
    Group(Box<Expr>),

    /// Molecular literal: `{ S=1 R=2 V=128 A=128 T=3 }`
    ///
    /// Construct a 1-molecule chain from explicit dimension values.
    /// Unspecified dimensions use defaults:
    ///   S=1 (Sphere), R=1 (Member), V=128 (neutral), A=128 (moderate), T=3 (Medium)
    MolLiteral {
        shape: Option<u32>,
        relation: Option<u32>,
        valence: Option<u32>,
        arousal: Option<u32>,
        time: Option<u32>,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// ParseError
// ─────────────────────────────────────────────────────────────────────────────

/// Lỗi cú pháp.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
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
#[allow(missing_docs)]
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
    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        if self.tokens.is_empty() {
            return Ok(Vec::new());
        }

        // Single expression mode (backwards compatible)
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
            // Keyword style
            Token::Let => self.parse_let(),
            Token::Emit => self.parse_emit_kw(),
            Token::If => self.parse_if(),
            Token::Loop => self.parse_loop_kw(),
            Token::Fn => self.parse_fn(),
            Token::Command(_) => self.parse_command(),

            // Symbol style
            Token::Circle => self.parse_emit_sym(),   // ○ expr;
            Token::Cycle => self.parse_loop_sym(),    // ↻ N { }

            // Expression, then check for ≔ or ⇒ suffix
            _ => self.parse_expr_or_symbol_stmt(),
        }
    }

    /// `let name = expr;`
    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'let'
        let name = self.expect_ident()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(&Token::Semi)?;
        Ok(Stmt::Let { name, value })
    }

    /// `emit expr;`
    fn parse_emit_kw(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'emit'
        let expr = self.parse_expr()?;
        self.expect(&Token::Semi)?;
        Ok(Stmt::Emit(expr))
    }

    /// `○ expr;`
    fn parse_emit_sym(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume ○
        let expr = self.parse_expr()?;
        // Optional semicolon
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::Emit(expr))
    }

    /// `if expr { stmts } (else { stmts })?`
    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'if'
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.check(&Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
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
    fn parse_loop_kw(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'loop'
        let count = self.expect_int()?;
        let body = self.parse_block()?;
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::Loop { count, body })
    }

    /// `↻ N { stmts }`
    fn parse_loop_sym(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume ↻
        let count = self.expect_int()?;
        let body = self.parse_block()?;
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::Loop { count, body })
    }

    /// `fn name(params) { stmts }`
    fn parse_fn(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'fn'
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;
        let body = self.parse_block()?;
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::FnDef { name, params, body })
    }

    /// command (STR)? ';'?
    fn parse_command(&mut self) -> Result<Stmt, ParseError> {
        let cmd = match self.advance() {
            Token::Command(s) => s,
            _ => return Err(ParseError::new("Expected command")),
        };
        // Check for argument: learn "text" or seed L0
        match self.peek() {
            Token::Str(s) => {
                let arg = s.clone();
                self.advance();
                if self.check(&Token::Semi) {
                    self.advance();
                }
                Ok(Stmt::CommandArg {
                    name: cmd,
                    arg,
                })
            }
            Token::Ident(s) => {
                // e.g., "seed L0"
                let arg = s.clone();
                self.advance();
                if self.check(&Token::Semi) {
                    self.advance();
                }
                Ok(Stmt::CommandArg {
                    name: cmd,
                    arg,
                })
            }
            _ => {
                if self.check(&Token::Semi) {
                    self.advance();
                }
                Ok(Stmt::Command(cmd))
            }
        }
    }

    /// Parse expression, then check for ≔ or ⇒ suffix
    fn parse_expr_or_symbol_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;

        // ≔ → define (let or fn)
        if self.check(&Token::Define) {
            return self.finish_define(expr);
        }

        // ⇒ → implies (if/else)
        if self.check(&Token::Implies) {
            return self.finish_implies(expr);
        }

        // Regular expression statement
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::Expr(expr))
    }

    /// `name ≔ expr;` or `name ≔ (params) { body }`
    fn finish_define(&mut self, lhs: Expr) -> Result<Stmt, ParseError> {
        self.advance(); // consume ≔

        let name = match lhs {
            Expr::Ident(n) => n,
            _ => return Err(ParseError::new("Left side of ≔ must be an identifier")),
        };

        // Check for function definition: name ≔ (params) { body }
        if self.check(&Token::LParen) {
            // Could be fn def or grouped expr — peek ahead
            let saved_pos = self.pos;
            self.advance(); // consume (

            // Try to parse as param list
            if let Ok(params) = self.try_parse_params() {
                if self.check(&Token::RParen) {
                    self.advance(); // consume )
                    if self.check(&Token::LBrace) {
                        let body = self.parse_block()?;
                        if self.check(&Token::Semi) {
                            self.advance();
                        }
                        return Ok(Stmt::FnDef { name, params, body });
                    }
                }
            }

            // Not a fn def — backtrack and parse as regular expression
            self.pos = saved_pos;
        }

        // Regular define: name ≔ expr;
        let value = self.parse_expr()?;
        self.expect(&Token::Semi)?;
        Ok(Stmt::Let { name, value })
    }

    /// `cond ⇒ { then } (⊥ { else })?`
    fn finish_implies(&mut self, cond: Expr) -> Result<Stmt, ParseError> {
        self.advance(); // consume ⇒
        let then_block = self.parse_block()?;

        // ⊥ { else } — orthogonal = else
        let else_block = if self.check_rel(RelOp::Ortho) {
            self.advance(); // consume ⊥
            Some(self.parse_block()?)
        } else {
            None
        };

        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::If {
            cond,
            then_block,
            else_block,
        })
    }

    /// Try to parse params without consuming on failure.
    fn try_parse_params(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) {
            return Ok(params);
        }
        match self.peek() {
            Token::Ident(s) => {
                params.push(s.clone());
                self.advance();
            }
            _ => return Err(ParseError::new("Expected param name")),
        }
        while self.check(&Token::Comma) {
            self.advance();
            match self.peek() {
                Token::Ident(s) => {
                    params.push(s.clone());
                    self.advance();
                }
                _ => return Err(ParseError::new("Expected param name")),
            }
        }
        Ok(params)
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

    // ── Expression parsing (precedence: primary → arith → compose → rel) ──

    /// expr = rel_chain
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_rel_chain()
    }

    /// rel_chain = truth (REL_OP (truth | '?'))*
    ///
    /// Single hop → RelEdge or RelQuery.
    /// Multiple hops → Chain (multi-hop path finding).
    fn parse_rel_chain(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_truth_expr()?;

        let mut steps: Vec<(RelOp, Expr)> = Vec::new();

        while let Token::Rel(op) = self.peek() {
            if op == RelOp::Compose {
                break; // ∘ handled at compose level
            }
            // Don't consume ⊥ if it looks like an "else" (after ⇒ block)
            if op == RelOp::Ortho && !steps.is_empty() {
                // Could be else — let caller decide
                break;
            }
            self.advance();

            let rhs = if self.check(&Token::Wild) {
                self.advance();
                Expr::Ident("?".to_string())
            } else {
                self.parse_truth_expr()?
            };
            steps.push((op, rhs));
        }

        match steps.len() {
            0 => Ok(left),
            1 => {
                let (op, rhs) = steps.into_iter().next().unwrap();
                if rhs == Expr::Ident("?".to_string()) {
                    Ok(Expr::RelQuery {
                        subject: Box::new(left),
                        op,
                    })
                } else {
                    Ok(Expr::RelEdge {
                        lhs: Box::new(left),
                        op,
                        rhs: Box::new(rhs),
                    })
                }
            }
            _ => Ok(Expr::Chain {
                head: Box::new(left),
                steps,
            }),
        }
    }

    /// truth = compose ('==' compose)?    (QT3: sự thật chắc chắn)
    fn parse_truth_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_compose_expr()?;
        if self.check(&Token::Truth) {
            self.advance();
            let right = self.parse_compose_expr()?;
            return Ok(Expr::Truth {
                lhs: Box::new(left),
                rhs: Box::new(right),
            });
        }
        Ok(left)
    }

    /// compose = arith ('∘' arith)*
    fn parse_compose_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_arith_expr()?;

        while self.check_rel(RelOp::Compose) {
            self.advance();
            let right = self.parse_arith_expr()?;
            left = Expr::Compose(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// arith = primary ((ARITH_OP | PHYS_OP) primary)*
    fn parse_arith_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;

        loop {
            match self.peek() {
                Token::Arith(op) => {
                    self.advance();
                    let right = self.parse_primary()?;
                    left = Expr::Arith {
                        lhs: Box::new(left),
                        op,
                        rhs: Box::new(right),
                    };
                }
                Token::Phys(op) => {
                    self.advance();
                    let right = self.parse_primary()?;
                    left = Expr::PhysOp {
                        lhs: Box::new(left),
                        op,
                        rhs: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// primary = IDENT | IDENT '(' args ')' | INT | STR | '(' expr ')' | '?'
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Check for function call: name(args)
                if self.check(&Token::LParen) {
                    self.advance();
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

            Token::Str(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Str(s))
            }

            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::Group(Box::new(expr)))
            }

            Token::Wild => {
                self.advance();
                Ok(Expr::Ident("?".to_string()))
            }

            // Molecular literal: { S=1 R=2 V=128 A=128 T=3 }
            Token::LBrace => {
                self.try_parse_mol_literal()
            }

            other => Err(ParseError::new(&alloc::format!(
                "Unexpected token: {:?}",
                other
            ))),
        }
    }

    /// Parse molecular literal: `{ S=1 R=2 V=128 A=128 T=3 }`
    ///
    /// Dimensions: S(hape), R(elation), V(alence), A(rousal), T(ime)
    /// All optional — unspecified = default.
    fn try_parse_mol_literal(&mut self) -> Result<Expr, ParseError> {
        self.advance(); // consume {

        let mut shape = None;
        let mut relation = None;
        let mut valence = None;
        let mut arousal = None;
        let mut time = None;

        while !self.check(&Token::RBrace) && !self.at_eof() {
            let dim_name = self.expect_ident()?;
            self.expect(&Token::Eq)?;
            let val = self.expect_int()?;

            match dim_name.as_str() {
                "S" | "shape" => shape = Some(val),
                "R" | "relation" => relation = Some(val),
                "V" | "valence" => valence = Some(val),
                "A" | "arousal" => arousal = Some(val),
                "T" | "time" => time = Some(val),
                _ => return Err(ParseError::new(&alloc::format!(
                    "Unknown dimension '{}'. Use S, R, V, A, or T", dim_name
                ))),
            }

            // Optional comma separator
            if self.check(&Token::Comma) {
                self.advance();
            }
        }

        self.expect(&Token::RBrace)?;

        Ok(Expr::MolLiteral {
            shape,
            relation,
            valence,
            arousal,
            time,
        })
    }

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

    /// Single-expression mode: no statement indicators?
    fn is_single_expr(&self) -> bool {
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
                    | Token::Define   // ≔
                    | Token::Implies  // ⇒
                    | Token::Cycle    // ↻
                    | Token::Circle   // ○
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

    // ── Chain queries (NEW) ─────────────────────────────────────────────────

    #[test]
    fn parse_chain_query() {
        // ○{🌞 → ? → 🌵} = tìm X sao cho 🌞→X và X→🌵
        let stmts = parse("🌞 → ? → 🌵").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::Chain {
                head: Box::new(Expr::Ident("🌞".into())),
                steps: vec![
                    (RelOp::Causes, Expr::Ident("?".into())),
                    (RelOp::Causes, Expr::Ident("🌵".into())),
                ],
            })]
        );
    }

    #[test]
    fn parse_chain_mixed_ops() {
        // A ∈ ? → B = tìm X sao cho A∈X và X→B
        let stmts = parse("fire ∈ ? → water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::Chain {
                head: Box::new(Expr::Ident("fire".into())),
                steps: vec![
                    (RelOp::Member, Expr::Ident("?".into())),
                    (RelOp::Causes, Expr::Ident("water".into())),
                ],
            })]
        );
    }

    // ── Arithmetic (NEW) ────────────────────────────────────────────────────

    #[test]
    fn parse_arithmetic_add() {
        let stmts = parse("1 + 2").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::Arith {
                lhs: Box::new(Expr::Int(1)),
                op: ArithOp::Add,
                rhs: Box::new(Expr::Int(2)),
            })]
        );
    }

    #[test]
    fn parse_arithmetic_mul() {
        let stmts = parse("3 × 4").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::Arith {
                lhs: Box::new(Expr::Int(3)),
                op: ArithOp::Mul,
                rhs: Box::new(Expr::Int(4)),
            })]
        );
    }

    // ── Keyword style (existing, still works) ───────────────────────────────

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
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], Stmt::If { .. }));
    }

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

    #[test]
    fn parse_fn_def() {
        let stmts = parse("fn blend(a, b) { emit a ∘ b; }").unwrap();
        assert!(matches!(stmts[0], Stmt::FnDef { .. }));
    }

    #[test]
    fn parse_fn_call() {
        let stmts = parse("emit blend(fire, water);").unwrap();
        assert!(matches!(stmts[0], Stmt::Emit(Expr::Call { .. })));
    }

    // ── Symbol style (NEW) ──────────────────────────────────────────────────

    #[test]
    fn parse_define_symbol() {
        // steam ≔ fire ∘ water;
        let stmts = parse("steam ≔ fire ∘ water;").unwrap();
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

    #[test]
    fn parse_circle_emit() {
        // ○ fire;
        let stmts = parse("○ fire;").unwrap();
        assert_eq!(stmts, vec![Stmt::Emit(Expr::Ident("fire".into()))]);
    }

    #[test]
    fn parse_implies_if() {
        // fire ⇒ { ○ water; }
        let stmts = parse("fire ⇒ { ○ water; }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::If {
                cond: Expr::Ident("fire".into()),
                then_block: vec![Stmt::Emit(Expr::Ident("water".into()))],
                else_block: None,
            }]
        );
    }

    #[test]
    fn parse_implies_if_else() {
        // fire ⇒ { ○ fire; } ⊥ { ○ water; }
        let stmts = parse("fire ⇒ { ○ fire; } ⊥ { ○ water; }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::If {
                cond: Expr::Ident("fire".into()),
                then_block: vec![Stmt::Emit(Expr::Ident("fire".into()))],
                else_block: Some(vec![Stmt::Emit(Expr::Ident("water".into()))]),
            }]
        );
    }

    #[test]
    fn parse_cycle_loop() {
        // ↻ 3 { ○ fire; }
        let stmts = parse("↻ 3 { ○ fire; }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Loop {
                count: 3,
                body: vec![Stmt::Emit(Expr::Ident("fire".into()))],
            }]
        );
    }

    #[test]
    fn parse_define_fn() {
        // blend ≔ (a, b) { ○ a ∘ b; }
        let stmts = parse("blend ≔ (a, b) { ○ a ∘ b; }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::FnDef {
                name: "blend".into(),
                params: vec!["a".into(), "b".into()],
                body: vec![Stmt::Emit(Expr::Compose(
                    Box::new(Expr::Ident("a".into())),
                    Box::new(Expr::Ident("b".into())),
                ))],
            }]
        );
    }

    // ── Commands with args (NEW) ────────────────────────────────────────────

    #[test]
    fn parse_learn_command() {
        let stmts = parse("learn \"tôi buồn vì mất việc\";").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::CommandArg {
                name: "learn".into(),
                arg: "tôi buồn vì mất việc".into(),
            }]
        );
    }

    #[test]
    fn parse_seed_command() {
        let stmts = parse("seed L0;").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::CommandArg {
                name: "seed".into(),
                arg: "L0".into(),
            }]
        );
    }

    // ── New relation operators ──────────────────────────────────────────────

    #[test]
    fn parse_set_minus() {
        let stmts = parse("fire ∖ water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::RelEdge {
                lhs: Box::new(Expr::Ident("fire".into())),
                op: RelOp::SetMinus,
                rhs: Box::new(Expr::Ident("water".into())),
            })]
        );
    }

    #[test]
    fn parse_bidir() {
        let stmts = parse("fire ↔ water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::RelEdge {
                lhs: Box::new(Expr::Ident("fire".into())),
                op: RelOp::Bidir,
                rhs: Box::new(Expr::Ident("water".into())),
            })]
        );
    }

    #[test]
    fn parse_trigger() {
        let stmts = parse("🔥 ⚡ 💧").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::RelEdge {
                lhs: Box::new(Expr::Ident("🔥".into())),
                op: RelOp::Trigger,
                rhs: Box::new(Expr::Ident("💧".into())),
            })]
        );
    }

    #[test]
    fn parse_parallel() {
        let stmts = parse("fire ∥ water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::RelEdge {
                lhs: Box::new(Expr::Ident("fire".into())),
                op: RelOp::Parallel,
                rhs: Box::new(Expr::Ident("water".into())),
            })]
        );
    }

    // ── Grouping ────────────────────────────────────────────────────────────

    #[test]
    fn parse_grouped_expr() {
        let stmts = parse("(fire ∘ water) → earth").unwrap();
        assert!(matches!(stmts[0], Stmt::Expr(Expr::RelEdge { .. })));
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

    // ── Pure symbol program ─────────────────────────────────────────────────

    #[test]
    fn parse_pure_symbol_program() {
        // Entire program in pure math symbols, no English keywords
        let src = "steam ≔ 🔥 ∘ 💧; steam ∈ ? ⇒ { ○ steam; } ⊥ { ○ 💧; }";
        let stmts = parse(src).unwrap();
        assert_eq!(stmts.len(), 2);
        assert!(matches!(stmts[0], Stmt::Let { .. }));
        assert!(matches!(stmts[1], Stmt::If { .. }));
    }

    // ── Error cases ─────────────────────────────────────────────────────────

    #[test]
    fn parse_empty_is_ok() {
        let stmts = parse("").unwrap();
        assert!(stmts.is_empty());
    }

    #[test]
    fn parse_unclosed_paren() {
        assert!(parse("(fire").is_err());
    }

    #[test]
    fn parse_let_missing_semi() {
        assert!(parse("let x = fire").is_err());
    }

    // ── QT3: hypothesis vs physical vs truth ────────────────────────────────

    #[test]
    fn parse_physical_add() {
        let stmts = parse("fire ⧺ water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::PhysOp {
                lhs: Box::new(Expr::Ident("fire".into())),
                op: PhysOp::PhysAdd,
                rhs: Box::new(Expr::Ident("water".into())),
            })]
        );
    }

    #[test]
    fn parse_physical_sub() {
        let stmts = parse("fire ⊖ water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::PhysOp {
                lhs: Box::new(Expr::Ident("fire".into())),
                op: PhysOp::PhysSub,
                rhs: Box::new(Expr::Ident("water".into())),
            })]
        );
    }

    #[test]
    fn parse_truth_assertion() {
        let stmts = parse("fire == water").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::Truth {
                lhs: Box::new(Expr::Ident("fire".into())),
                rhs: Box::new(Expr::Ident("water".into())),
            })]
        );
    }

    #[test]
    fn parse_hypothesis_vs_physical() {
        // QT3: 1 + 2 = hypothesis, fire ⧺ water = proven
        let hyp = parse("1 + 2").unwrap();
        assert!(matches!(hyp[0], Stmt::Expr(Expr::Arith { .. })));

        let phys = parse("fire ⧺ water").unwrap();
        assert!(matches!(phys[0], Stmt::Expr(Expr::PhysOp { .. })));
    }

    // ── Molecular Literals ──────────────────────────────────────────────────

    #[test]
    fn parse_mol_literal_all_dims() {
        let stmts = parse("{ S=1 R=6 V=200 A=180 T=4 }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::MolLiteral {
                shape: Some(1),
                relation: Some(6),
                valence: Some(200),
                arousal: Some(180),
                time: Some(4),
            })]
        );
    }

    #[test]
    fn parse_mol_literal_partial() {
        let stmts = parse("{ S=1 R=2 T=3 }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::MolLiteral {
                shape: Some(1),
                relation: Some(2),
                valence: None,
                arousal: None,
                time: Some(3),
            })]
        );
    }

    #[test]
    fn parse_mol_literal_with_commas() {
        let stmts = parse("{ S=1, R=6, V=200, A=180, T=4 }").unwrap();
        assert!(matches!(stmts[0], Stmt::Expr(Expr::MolLiteral { .. })));
    }

    #[test]
    fn parse_mol_literal_truth_assertion() {
        // "lửa" == { S=1 R=6 T=4 }
        let stmts = parse("\"lửa\" == { S=1 R=6 T=4 }").unwrap();
        assert!(matches!(stmts[0], Stmt::Expr(Expr::Truth { .. })));
    }

    #[test]
    fn parse_mol_literal_long_names() {
        let stmts = parse("{ shape=1 relation=6 valence=200 arousal=180 time=4 }").unwrap();
        assert_eq!(
            stmts,
            vec![Stmt::Expr(Expr::MolLiteral {
                shape: Some(1),
                relation: Some(6),
                valence: Some(200),
                arousal: Some(180),
                time: Some(4),
            })]
        );
    }

    #[test]
    fn parse_mol_literal_in_let() {
        let stmts = parse("let fire = { S=1 R=6 V=200 A=180 T=4 };").unwrap();
        assert!(matches!(stmts[0], Stmt::Let { .. }));
    }

    #[test]
    fn parse_mol_literal_unknown_dim_errors() {
        let result = parse("{ X=1 }");
        assert!(result.is_err());
    }
}
