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

    /// `let { a, b } = expr;` — destructure dict/record into variables
    LetDestructure { names: Vec<String>, value: Expr },

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

    /// `fn name[T: Trait](params) { body }` hoặc `name ≔ (params) { body }`
    FnDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        /// Generic type parameters: `fn name[T, U](...)`
        type_params: Vec<String>,
        /// Trait bounds: `fn name[T: Skill, U: Iterator](...)`
        trait_bounds: Vec<(String, String)>,
    },

    /// Expression statement
    Expr(Expr),

    /// System command: `dream;` `stats;` `learn "text";`
    Command(String),

    /// Command with argument: `learn "text"`, `seed L0`
    CommandArg { name: String, arg: String },

    /// `match expr { pattern => { body }, _ => { body } }`
    Match {
        subject: Expr,
        arms: Vec<MatchArm>,
    },

    /// `try { body } catch { handler }`
    TryCatch {
        try_block: Vec<Stmt>,
        catch_block: Vec<Stmt>,
    },

    /// `for var in start..end { body }` — range iteration
    ForIn {
        var: String,
        start: u32,
        end: u32,
        body: Vec<Stmt>,
    },

    /// `for var in expr { body }` — iterate over array/collection
    ForEach {
        var: String,
        iter: Expr,
        body: Vec<Stmt>,
    },

    /// `while cond { body }`
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },

    /// `break;` — exit innermost loop
    Break,

    /// `continue;` — skip to next iteration
    Continue,

    /// `name = expr;` — reassign existing variable
    Assign { name: String, value: Expr },

    /// `return expr;` — return value from function
    Return(Option<Expr>),

    /// `use "module";` or `use module.path;` or `use module.path.{a, b};`
    Use { module: String, imports: Vec<String> },

    /// `mod module.path;` — module declaration
    ModDecl(String),

    /// `obj.field = expr;` or `obj.a.b.c = expr;` — assign to field of dict/record
    FieldAssign { object: String, fields: Vec<String>, value: Expr },

    /// `struct Name[T] { field1, field2, ... }` — struct definition (with optional generics)
    StructDef { name: String, type_params: Vec<String>, fields: Vec<StructField> },

    /// `enum Name[T] { Variant1, Variant2(Type), ... }` — enum definition (with optional generics)
    EnumDef { name: String, type_params: Vec<String>, variants: Vec<EnumVariant> },

    /// `impl Name { fn method(self, ...) { ... } ... }` — impl block
    ImplBlock { target: String, methods: Vec<Stmt> },

    /// `trait Name[T] { fn method(self); ... }` — trait definition (with optional generics)
    TraitDef { name: String, type_params: Vec<String>, methods: Vec<TraitMethod> },

    /// `impl TraitName for StructName { ... }` — trait implementation
    ImplTrait { trait_name: String, target: String, methods: Vec<Stmt> },

    /// `spawn { body }` — concurrent task
    Spawn { body: Vec<Stmt> },
}

/// Struct field with optional type annotation and visibility.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct StructField {
    pub name: String,
    pub type_name: Option<String>,
    /// `pub` field — accessible from outside the defining module
    pub is_pub: bool,
}

/// Enum variant — unit, tuple, or struct variant.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct EnumVariant {
    pub name: String,
    /// Payload types (empty = unit variant)
    pub fields: Vec<String>,
}

/// Trait method signature with optional default body.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<String>,
    /// Optional default implementation body.
    /// If present, implementors can omit this method.
    pub default_body: Option<Vec<Stmt>>,
}

/// Match arm — pattern + body.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct MatchArm {
    /// Pattern: Ident (type name), MolLiteral, or Wildcard ("_")
    pub pattern: MatchPattern,
    /// Body statements
    pub body: Vec<Stmt>,
}

/// Match pattern.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum MatchPattern {
    /// Match by type name: SDF, MATH, EMOTICON, MUSICAL, Mixed, Empty
    TypeName(String),
    /// Match by molecular literal: { S=1 R=6 }
    MolLiteral {
        shape: Option<u32>,
        relation: Option<u32>,
        valence: Option<u32>,
        arousal: Option<u32>,
        time: Option<u32>,
    },
    /// Wildcard: `_` — matches anything (default arm)
    Wildcard,
    /// Enum variant: `Color::Red` or `Option::Some(x)`
    EnumPattern { enum_name: String, variant: String, bindings: Vec<String> },
}

/// Comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    Le,
    /// `>=`
    Ge,
    /// `!=`
    Ne,
}

/// Expression — mọi expression evaluate → MolecularChain.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Expr {
    /// Identifier: node alias, biến, emoji
    Ident(String),

    /// Integer literal
    Int(u32),

    /// Float literal
    Float(f64),

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

    /// `a < b`, `a > b`, `a <= b`, `a >= b` — comparison (returns 1.0 or 0.0)
    Compare {
        lhs: Box<Expr>,
        op: CmpOp,
        rhs: Box<Expr>,
    },

    /// `a && b` — logical and (both non-empty)
    LogicAnd(Box<Expr>, Box<Expr>),

    /// `a || b` — logical or (either non-empty)
    LogicOr(Box<Expr>, Box<Expr>),

    /// `!a` — logical not (empty → non-empty, non-empty → empty)
    LogicNot(Box<Expr>),

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

    /// Array literal: `[a, b, c]`
    Array(Vec<Expr>),

    /// Array indexing: `arr[idx]`
    Index { array: Box<Expr>, index: Box<Expr> },

    /// Dict literal: `{ key: value, key2: value2 }`
    Dict(Vec<(String, Expr)>),

    /// Field access: `obj.field`
    FieldAccess { object: Box<Expr>, field: String },

    /// Pipe: `a |> f` — pass left as argument to right
    Pipe(Box<Expr>, Box<Expr>),

    /// Lambda: `|x, y| expr`
    Lambda { params: Vec<String>, body: Box<Expr> },

    /// Conditional expression: `if cond { a } else { b }`
    IfExpr { cond: Box<Expr>, then_expr: Box<Expr>, else_expr: Box<Expr> },

    /// Struct instantiation: `Point { x: 1, y: 2 }`
    StructLiteral { name: String, fields: Vec<(String, Expr)> },

    /// Enum variant: `Color::Red` or `Option::Some(value)`
    EnumVariantExpr { enum_name: String, variant: String, args: Vec<Expr> },

    /// Method call: `obj.method(args)` — dispatched via impl table
    MethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },

    /// Self reference: `self`
    SelfRef,

    /// `channel()` — create a new channel for spawn communication
    ChannelCreate,

    /// `expr ?? default` — unwrap Option/Result with default value
    /// If expr is empty (None/Err), return default; otherwise return expr.
    UnwrapOr { value: Box<Expr>, default: Box<Expr> },

    /// Tuple literal: `(a, b, c)` — used for multiple return values
    Tuple(Vec<Expr>),
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

    /// Extract root ident and field path from a nested FieldAccess expression.
    /// `a.b.c` → Some(("a", ["b", "c"]))
    fn extract_field_path(expr: &Expr) -> Option<(String, Vec<String>)> {
        match expr {
            Expr::FieldAccess { object, field } => {
                match object.as_ref() {
                    Expr::Ident(root) => Some((root.clone(), alloc::vec![field.clone()])),
                    _ => {
                        let (root, mut path) = Self::extract_field_path(object)?;
                        path.push(field.clone());
                        Some((root, path))
                    }
                }
            }
            _ => None,
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
            Token::Match => self.parse_match(),
            Token::Try => self.parse_try_catch(),
            Token::For => self.parse_for_in(),
            Token::While => self.parse_while(),
            Token::Break => {
                self.advance();
                if self.check(&Token::Semi) { self.advance(); }
                Ok(Stmt::Break)
            }
            Token::Continue => {
                self.advance();
                if self.check(&Token::Semi) { self.advance(); }
                Ok(Stmt::Continue)
            }
            Token::Return => {
                self.advance();
                // return; or return expr;
                if self.check(&Token::Semi) || self.check(&Token::RBrace) || self.at_eof() {
                    if self.check(&Token::Semi) { self.advance(); }
                    Ok(Stmt::Return(None))
                } else {
                    let expr = self.parse_expr()?;
                    if self.check(&Token::Semi) { self.advance(); }
                    Ok(Stmt::Return(Some(expr)))
                }
            }
            Token::Use => {
                self.advance();
                // use "module"; or use module.path; or use module.path.{a, b};
                let mut path = match self.peek() {
                    Token::Str(s) => {
                        let s = s.clone();
                        self.advance();
                        s
                    }
                    Token::Ident(s) => {
                        let s = s.clone();
                        self.advance();
                        s
                    }
                    _ => return Err(ParseError::new("Expected module name after 'use'")),
                };
                // Parse dot-separated path: use silk.graph
                while self.check(&Token::Dot) {
                    self.advance();
                    // Check for selective import: use silk.graph.{a, b}
                    if self.check(&Token::LBrace) {
                        self.advance();
                        let mut imports = Vec::new();
                        if !self.check(&Token::RBrace) {
                            imports.push(self.expect_ident()?);
                            while self.check(&Token::Comma) {
                                self.advance();
                                imports.push(self.expect_ident()?);
                            }
                        }
                        self.expect(&Token::RBrace)?;
                        if self.check(&Token::Semi) { self.advance(); }
                        return Ok(Stmt::Use { module: path, imports });
                    }
                    let seg = self.expect_ident()?;
                    path = alloc::format!("{path}.{seg}");
                }
                if self.check(&Token::Semi) { self.advance(); }
                Ok(Stmt::Use { module: path, imports: Vec::new() })
            }
            Token::ModKw => {
                self.advance();
                // mod module.path;
                let mut path = self.expect_ident()?;
                while self.check(&Token::Dot) {
                    self.advance();
                    let seg = self.expect_ident()?;
                    path = alloc::format!("{path}.{seg}");
                }
                if self.check(&Token::Semi) { self.advance(); }
                Ok(Stmt::ModDecl(path))
            }
            Token::Struct => self.parse_struct_def(),
            Token::Enum => self.parse_enum_def(),
            Token::Impl => self.parse_impl(),
            Token::Trait => self.parse_trait_def(),
            Token::Pub => {
                self.advance(); // consume 'pub'
                // pub struct / pub fn / pub enum / pub trait — skip pub, parse inner
                self.parse_stmt()
            }
            Token::Spawn => self.parse_spawn(),
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
        // Check for destructuring: let { a, b } = expr;
        if self.check(&Token::LBrace) {
            self.advance(); // consume {
            let mut names = Vec::new();
            if !self.check(&Token::RBrace) {
                names.push(self.expect_ident()?);
                while self.check(&Token::Comma) {
                    self.advance();
                    if self.check(&Token::RBrace) { break; } // trailing comma
                    names.push(self.expect_ident()?);
                }
            }
            self.expect(&Token::RBrace)?;
            self.expect(&Token::Eq)?;
            let value = self.parse_expr()?;
            self.expect(&Token::Semi)?;
            return Ok(Stmt::LetDestructure { names, value });
        }
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
            if self.check(&Token::If) {
                // else if → nested if as single-stmt else block
                let nested_if = self.parse_if()?;
                Some(alloc::vec![nested_if])
            } else {
                Some(self.parse_block()?)
            }
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
        // Parse optional generic type params: fn name[T, U: Trait](...)
        let (type_params, trait_bounds) = self.parse_generic_params()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;
        let body = self.parse_block()?;
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::FnDef { name, params, body, type_params, trait_bounds })
    }

    /// `match expr { pattern => { body }, ... }`
    fn parse_match(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'match'
        let subject = self.parse_expr()?;
        self.expect(&Token::LBrace)?;

        let mut arms = Vec::new();
        while !self.check(&Token::RBrace) && !self.at_eof() {
            let pattern = self.parse_match_pattern()?;
            self.expect(&Token::FatArrow)?;
            let body = self.parse_block()?;
            // Optional comma or semicolon separator
            if self.check(&Token::Comma) || self.check(&Token::Semi) {
                self.advance();
            }
            arms.push(MatchArm { pattern, body });
        }
        self.expect(&Token::RBrace)?;
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::Match { subject, arms })
    }

    /// Parse match pattern: TypeName, MolLiteral, or Wildcard
    fn parse_match_pattern(&mut self) -> Result<MatchPattern, ParseError> {
        match self.peek() {
            // _ → wildcard
            Token::Ident(s) if s == "_" => {
                self.advance();
                Ok(MatchPattern::Wildcard)
            }
            // { S=1 R=2 ... } → molecular literal pattern
            Token::LBrace => {
                // Parse as mol literal using existing parse_mol_literal
                let expr = self.try_parse_mol_literal()?;
                match expr {
                    Expr::MolLiteral { shape, relation, valence, arousal, time } => {
                        Ok(MatchPattern::MolLiteral { shape, relation, valence, arousal, time })
                    }
                    _ => Err(ParseError::new("Expected molecular literal pattern")),
                }
            }
            // Ident → type name OR enum variant pattern (Name::Variant or Name::Variant(x))
            Token::Ident(_) => {
                let name = self.expect_ident()?;
                // Check for :: (enum variant pattern)
                if self.check(&Token::ColonColon) {
                    self.advance();
                    let variant = self.expect_ident()?;
                    // Optional payload bindings: Name::Variant(x, y)
                    let mut bindings = Vec::new();
                    if self.check(&Token::LParen) {
                        self.advance();
                        while !self.check(&Token::RParen) && !self.at_eof() {
                            bindings.push(self.expect_ident()?);
                            if self.check(&Token::Comma) { self.advance(); }
                        }
                        self.expect(&Token::RParen)?;
                    }
                    Ok(MatchPattern::EnumPattern { enum_name: name, variant, bindings })
                } else {
                    Ok(MatchPattern::TypeName(name))
                }
            }
            _ => Err(ParseError::new("Expected match pattern (type name, { mol }, or _)")),
        }
    }

    /// `try { body } catch { handler }`
    fn parse_try_catch(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'try'
        let try_block = self.parse_block()?;
        self.expect(&Token::Catch)?;
        let catch_block = self.parse_block()?;
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::TryCatch {
            try_block,
            catch_block,
        })
    }

    /// `for` IDENT `in` (INT `..` INT | expr) `{` stmts `}`
    fn parse_for_in(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'for'
        let var = self.expect_ident()?;
        self.expect(&Token::In)?;

        // Try range: INT .. INT
        if let Token::Int(start) = self.peek() {
            let saved = self.pos;
            self.advance(); // consume start
            if self.check(&Token::DotDot) {
                self.advance(); // consume ..
                let end = self.expect_int()?;
                let body = self.parse_block()?;
                if self.check(&Token::Semi) { self.advance(); }
                return Ok(Stmt::ForIn { var, start, end, body });
            }
            // Not a range — backtrack and parse as expression
            self.pos = saved;
        }

        // For-each: for var in expr { body }
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        if self.check(&Token::Semi) { self.advance(); }
        Ok(Stmt::ForEach { var, iter, body })
    }

    /// `while` expr `{` stmts `}`
    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'while'
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        if self.check(&Token::Semi) {
            self.advance();
        }
        Ok(Stmt::While { cond, body })
    }

    // ── Type system: struct, enum, impl, trait ──────────────────────────────

    /// Parse optional type parameters: `[T, U, ...]`. Returns empty vec if none.
    fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
        if !self.check(&Token::LBracket) {
            return Ok(Vec::new());
        }
        self.advance(); // consume '['
        let mut params = Vec::new();
        while !self.check(&Token::RBracket) && !self.at_eof() {
            let name = self.expect_ident()?;
            // Skip optional trait bound (handled by parse_generic_params)
            if self.check(&Token::Colon) {
                self.advance();
                let _bound = self.expect_ident()?;
            }
            params.push(name);
            if self.check(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RBracket)?;
        Ok(params)
    }

    /// Parse generic type parameters with optional trait bounds: `[T, U: Trait]`
    /// Returns (type_params, trait_bounds) where trait_bounds is vec of (param, bound).
    #[allow(clippy::type_complexity)]
    fn parse_generic_params(&mut self) -> Result<(Vec<String>, Vec<(String, String)>), ParseError> {
        if !self.check(&Token::LBracket) {
            return Ok((Vec::new(), Vec::new()));
        }
        self.advance(); // consume '['
        let mut params = Vec::new();
        let mut bounds = Vec::new();
        while !self.check(&Token::RBracket) && !self.at_eof() {
            let name = self.expect_ident()?;
            // Optional trait bound: T: Trait
            if self.check(&Token::Colon) {
                self.advance();
                let bound = self.expect_ident()?;
                bounds.push((name.clone(), bound));
            }
            params.push(name);
            if self.check(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RBracket)?;
        Ok((params, bounds))
    }

    /// `struct Name[T] { field1: Type, field2, ... }`
    fn parse_struct_def(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'struct'
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.at_eof() {
            let is_pub = if self.check(&Token::Pub) {
                self.advance();
                true
            } else {
                false
            };
            let field_name = self.expect_ident()?;
            let type_name = if self.check(&Token::Colon) {
                self.advance();
                Some(self.expect_ident()?)
            } else {
                None
            };
            fields.push(StructField { name: field_name, type_name, is_pub });
            if self.check(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RBrace)?;
        if self.check(&Token::Semi) { self.advance(); }
        Ok(Stmt::StructDef { name, type_params, fields })
    }

    /// `enum Name { Variant1, Variant2(Type), ... }`
    fn parse_enum_def(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'enum'
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(&Token::LBrace)?;
        let mut variants = Vec::new();
        while !self.check(&Token::RBrace) && !self.at_eof() {
            let variant_name = self.expect_ident()?;
            let mut fields = Vec::new();
            if self.check(&Token::LParen) {
                self.advance();
                while !self.check(&Token::RParen) && !self.at_eof() {
                    fields.push(self.expect_ident()?);
                    if self.check(&Token::Comma) { self.advance(); }
                }
                self.expect(&Token::RParen)?;
            }
            variants.push(EnumVariant { name: variant_name, fields });
            if self.check(&Token::Comma) { self.advance(); }
        }
        self.expect(&Token::RBrace)?;
        if self.check(&Token::Semi) { self.advance(); }
        Ok(Stmt::EnumDef { name, type_params, variants })
    }

    /// `impl Name { ... }` or `impl Trait for Name { ... }`
    fn parse_impl(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'impl'
        let first = self.expect_ident()?;
        // Check for `impl Trait for Type { ... }`
        if self.check(&Token::For) {
            self.advance();
            let target = self.expect_ident()?;
            self.expect(&Token::LBrace)?;
            let mut methods = Vec::new();
            while !self.check(&Token::RBrace) && !self.at_eof() {
                if self.check(&Token::Pub) { self.advance(); }
                methods.push(self.parse_fn()?);
            }
            self.expect(&Token::RBrace)?;
            if self.check(&Token::Semi) { self.advance(); }
            return Ok(Stmt::ImplTrait {
                trait_name: first,
                target,
                methods,
            });
        }
        // Plain impl
        self.expect(&Token::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.at_eof() {
            if self.check(&Token::Pub) { self.advance(); }
            methods.push(self.parse_fn()?);
        }
        self.expect(&Token::RBrace)?;
        if self.check(&Token::Semi) { self.advance(); }
        Ok(Stmt::ImplBlock { target: first, methods })
    }

    /// `trait Name { fn method(self); fn default_method(self) { body } ... }`
    fn parse_trait_def(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'trait'
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(&Token::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.at_eof() {
            if self.check(&Token::Pub) { self.advance(); }
            self.expect(&Token::Fn)?;
            let method_name = self.expect_ident()?;
            self.expect(&Token::LParen)?;
            let mut params = Vec::new();
            while !self.check(&Token::RParen) && !self.at_eof() {
                if self.check(&Token::SelfKw) {
                    self.advance();
                    params.push("self".into());
                } else {
                    params.push(self.expect_ident()?);
                }
                if self.check(&Token::Comma) { self.advance(); }
            }
            self.expect(&Token::RParen)?;
            // Check for default body: `{ ... }` or just `;`
            let default_body = if self.check(&Token::LBrace) {
                Some(self.parse_block()?)
            } else {
                if self.check(&Token::Semi) { self.advance(); }
                None
            };
            methods.push(TraitMethod { name: method_name, params, default_body });
        }
        self.expect(&Token::RBrace)?;
        if self.check(&Token::Semi) { self.advance(); }
        Ok(Stmt::TraitDef { name, type_params, methods })
    }

    /// `spawn { body }`
    fn parse_spawn(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'spawn'
        self.expect(&Token::LBrace)?;
        let mut body = Vec::new();
        while !self.check(&Token::RBrace) && !self.at_eof() {
            body.push(self.parse_stmt()?);
        }
        self.expect(&Token::RBrace)?;
        if self.check(&Token::Semi) { self.advance(); }
        Ok(Stmt::Spawn { body })
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

    /// Parse expression, then check for ≔ or ⇒ or = suffix
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

        // ident = expr → reassignment (not `let`, not `==`)
        // obj.field = expr → field assignment (supports a.b.c = value)
        if self.check(&Token::Eq) {
            // Check if it's a (nested) field assign: extract root + path
            if let Some((root, fields)) = Self::extract_field_path(&expr) {
                self.advance(); // consume =
                let value = self.parse_expr()?;
                self.expect(&Token::Semi)?;
                return Ok(Stmt::FieldAssign { object: root, fields, value });
            }
            let is_assign = matches!(&expr, Expr::Ident(_));
            if is_assign {
                if let Expr::Ident(name) = expr {
                    self.advance(); // consume =
                    let value = self.parse_expr()?;
                    self.expect(&Token::Semi)?;
                    return Ok(Stmt::Assign { name, value });
                }
                unreachable!();
            }
            // Not assignable, fall through to expression stmt
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
                        return Ok(Stmt::FnDef { name, params, body, type_params: Vec::new(), trait_bounds: Vec::new() });
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
            // Accept `self` as first parameter
            if self.check(&Token::SelfKw) {
                self.advance();
                params.push("self".into());
                if self.check(&Token::Comma) { self.advance(); }
            }
            while !self.check(&Token::RParen) && !self.at_eof() {
                params.push(self.expect_ident()?);
                if self.check(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        Ok(params)
    }

    // ── Expression parsing (precedence: primary → arith → compose → rel) ──

    /// expr = pipe_chain
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_pipe_chain()
    }

    /// unwrap_chain = rel_chain ('??' rel_chain)?
    fn parse_unwrap_chain(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_rel_chain()?;
        if self.check(&Token::DoubleQuestion) {
            self.advance();
            let default = self.parse_rel_chain()?;
            return Ok(Expr::UnwrapOr {
                value: Box::new(left),
                default: Box::new(default),
            });
        }
        Ok(left)
    }

    /// pipe_chain = unwrap_chain ('|>' unwrap_chain)*
    fn parse_pipe_chain(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unwrap_chain()?;
        while self.check(&Token::PipeArrow) {
            self.advance();
            let right = self.parse_unwrap_chain()?;
            // a |> f  →  f(a)  — pipe left as first arg to right (must be Call)
            left = Expr::Pipe(Box::new(left), Box::new(right));
        }
        Ok(left)
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
                let (op, rhs) = match steps.into_iter().next() {
                    Some(pair) => pair,
                    None => return Err(ParseError::new("Expected relation step")),
                };
                if rhs == Expr::Ident(String::from("?")) {
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
        let left = self.parse_logic_or()?;
        if self.check(&Token::Truth) {
            self.advance();
            let right = self.parse_logic_or()?;
            return Ok(Expr::Truth {
                lhs: Box::new(left),
                rhs: Box::new(right),
            });
        }
        Ok(left)
    }

    /// logic_or = logic_and ('||' logic_and)*
    fn parse_logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_logic_and()?;
        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_logic_and()?;
            left = Expr::LogicOr(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// logic_and = compose ('&&' compose)*
    fn parse_logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_compose_expr()?;
        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_compose_expr()?;
            left = Expr::LogicAnd(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// compose = compare ('∘' compare)*
    fn parse_compose_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_compare_expr()?;

        while self.check_rel(RelOp::Compose) {
            self.advance();
            let right = self.parse_compare_expr()?;
            left = Expr::Compose(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// compare = arith (('<' | '>' | '<=' | '>=' | '!=') arith)?
    /// Note: '==' is handled at truth_expr level (QT3: sự thật chắc chắn)
    fn parse_compare_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_arith_expr()?;
        let cmp_op = match self.peek() {
            Token::Lt => Some(CmpOp::Lt),
            Token::Gt => Some(CmpOp::Gt),
            Token::Le => Some(CmpOp::Le),
            Token::Ge => Some(CmpOp::Ge),
            Token::Ne => Some(CmpOp::Ne),
            _ => None,
        };
        if let Some(op) = cmp_op {
            self.advance();
            let right = self.parse_arith_expr()?;
            Ok(Expr::Compare {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
            })
        } else {
            Ok(left)
        }
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
        let mut expr = match self.peek() {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Check for :: (enum variant or associated function)
                if self.check(&Token::ColonColon) {
                    self.advance(); // consume ::
                    let variant = self.expect_ident()?;
                    // Check for payload: Name::Variant(args)
                    if self.check(&Token::LParen) {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(&Token::RParen)?;
                        Expr::EnumVariantExpr { enum_name: name, variant, args }
                    } else {
                        Expr::EnumVariantExpr { enum_name: name, variant, args: Vec::new() }
                    }
                }
                // Check for struct literal: Name { field: value, ... }
                // Distinguish from block: Name followed by { ident : (not =)
                else if self.check(&Token::LBrace) && self.is_struct_literal(&name) {
                    self.advance(); // consume {
                    let mut fields = Vec::new();
                    while !self.check(&Token::RBrace) && !self.at_eof() {
                        let field_name = self.expect_ident()?;
                        self.expect(&Token::Colon)?;
                        let value = self.parse_expr()?;
                        fields.push((field_name, value));
                        if self.check(&Token::Comma) { self.advance(); }
                    }
                    self.expect(&Token::RBrace)?;
                    Expr::StructLiteral { name, fields }
                }
                // Check for function call: name(args)
                else if self.check(&Token::LParen) {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    Expr::Call { name, args }
                }
                // Check for array indexing: name[idx]
                else if self.check(&Token::LBracket) {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    Expr::Index {
                        array: Box::new(Expr::Ident(name)),
                        index: Box::new(index),
                    }
                } else {
                    Expr::Ident(name)
                }
            }

            // self keyword
            Token::SelfKw => {
                self.advance();
                Expr::SelfRef
            }

            // channel() — create a new channel
            Token::Channel => {
                self.advance();
                self.expect(&Token::LParen)?;
                self.expect(&Token::RParen)?;
                Expr::ChannelCreate
            }

            Token::Int(val) => {
                self.advance();
                Expr::Int(val)
            }

            Token::Float(val) => {
                let v = val;
                self.advance();
                Expr::Float(v)
            }

            Token::Str(s) => {
                let s = s.clone();
                self.advance();
                Expr::Str(s)
            }

            Token::LParen => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Expr::Group(Box::new(inner))
            }

            Token::Wild => {
                self.advance();
                Expr::Ident("?".to_string())
            }

            // { ... } — Dict literal or Molecular literal
            Token::LBrace => {
                if self.is_dict_literal() {
                    self.parse_dict_literal()?
                } else {
                    self.try_parse_mol_literal()?
                }
            }

            // Array literal: [a, b, c]
            Token::LBracket => {
                self.advance(); // consume [
                let mut elements = Vec::new();
                if !self.check(&Token::RBracket) {
                    elements.push(self.parse_expr()?);
                    while self.check(&Token::Comma) {
                        self.advance();
                        if self.check(&Token::RBracket) { break; } // trailing comma
                        elements.push(self.parse_expr()?);
                    }
                }
                self.expect(&Token::RBracket)?;
                Expr::Array(elements)
            }

            // Lambda: |x, y| expr
            Token::Pipe => {
                self.advance(); // consume |
                let mut params = Vec::new();
                if !self.check(&Token::Pipe) {
                    params.push(self.expect_ident()?);
                    while self.check(&Token::Comma) {
                        self.advance();
                        params.push(self.expect_ident()?);
                    }
                }
                self.expect(&Token::Pipe)?;
                let body = self.parse_expr()?;
                Expr::Lambda { params, body: Box::new(body) }
            }

            // If-expression: if cond { then } else { else }
            Token::If => {
                self.advance(); // consume if
                let cond = self.parse_expr()?;
                self.expect(&Token::LBrace)?;
                let then_expr = self.parse_expr()?;
                self.expect(&Token::RBrace)?;
                self.expect(&Token::Else)?;
                self.expect(&Token::LBrace)?;
                let else_expr = self.parse_expr()?;
                self.expect(&Token::RBrace)?;
                Expr::IfExpr {
                    cond: Box::new(cond),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                }
            }

            // Logical not: !expr
            Token::Not => {
                self.advance();
                let inner = self.parse_primary()?;
                Expr::LogicNot(Box::new(inner))
            }

            other => return Err(ParseError::new(&alloc::format!(
                "Unexpected token: {:?}",
                other
            ))),
        };

        // Postfix: .field access or .method(args) (can chain: a.b.c, a.method())
        while self.check(&Token::Dot) {
            self.advance(); // consume .
            let field = self.expect_ident()?;
            // Check for method call: obj.method(args)
            if self.check(&Token::LParen) {
                self.advance(); // consume (
                let mut args = Vec::new();
                if !self.check(&Token::RParen) {
                    args.push(self.parse_expr()?);
                    while self.check(&Token::Comma) {
                        self.advance();
                        args.push(self.parse_expr()?);
                    }
                }
                self.expect(&Token::RParen)?;
                expr = Expr::MethodCall {
                    object: Box::new(expr),
                    method: field,
                    args,
                };
            } else {
                expr = Expr::FieldAccess {
                    object: Box::new(expr),
                    field,
                };
            }
        }

        Ok(expr)
    }

    /// Check if `Name {` is a struct literal (capitalized name + { ident : ).
    /// Struct names must start with uppercase letter.
    fn is_struct_literal(&self, name: &str) -> bool {
        // Name must start with uppercase
        if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            return false;
        }
        // Look ahead: { ident :
        if self.pos + 2 < self.tokens.len() {
            matches!(&self.tokens[self.pos + 1], Token::Ident(_))
                && matches!(&self.tokens[self.pos + 2], Token::Colon)
        } else {
            false
        }
    }

    /// Look ahead to determine if `{` starts a Dict literal (`{ key: value }`)
    /// vs a MolLiteral (`{ S=1 R=2 }`).
    fn is_dict_literal(&self) -> bool {
        // pos is at LBrace. Check tokens[pos+1] = Ident, tokens[pos+2] = Colon
        if self.pos + 2 < self.tokens.len() {
            matches!(&self.tokens[self.pos + 1], Token::Ident(_))
                && matches!(&self.tokens[self.pos + 2], Token::Colon)
        } else {
            false
        }
    }

    /// Parse dict literal: `{ key: value, key2: value2 }`
    fn parse_dict_literal(&mut self) -> Result<Expr, ParseError> {
        self.advance(); // consume {
        let mut fields = Vec::new();
        if !self.check(&Token::RBrace) {
            loop {
                let key = self.expect_ident()?;
                self.expect(&Token::Colon)?;
                let value = self.parse_expr()?;
                fields.push((key, value));
                if self.check(&Token::Comma) {
                    self.advance();
                    if self.check(&Token::RBrace) { break; } // trailing comma
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(Expr::Dict(fields))
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
                    | Token::Struct
                    | Token::Enum
                    | Token::Impl
                    | Token::Trait
                    | Token::Spawn
                    | Token::Pub
                    | Token::ModKw
                    | Token::Use
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
                type_params: vec![],
                trait_bounds: vec![],
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

    // ── Match expression ────────────────────────────────────────────────

    #[test]
    fn parse_match_basic() {
        let stmts = parse("match fire { SDF => { emit water; } _ => { stats; } }").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Match { subject, arms } => {
                assert_eq!(*subject, Expr::Ident("fire".into()));
                assert_eq!(arms.len(), 2);
                assert_eq!(arms[0].pattern, MatchPattern::TypeName("SDF".into()));
                assert_eq!(arms[1].pattern, MatchPattern::Wildcard);
            }
            _ => panic!("Expected Match statement"),
        }
    }

    #[test]
    fn parse_match_multiple_arms() {
        let stmts = parse("match fire { SDF => { stats; } MATH => { dream; } EMOTICON => { fuse; } _ => { trace; } }").unwrap();
        match &stmts[0] {
            Stmt::Match { arms, .. } => {
                assert_eq!(arms.len(), 4);
                assert_eq!(arms[0].pattern, MatchPattern::TypeName("SDF".into()));
                assert_eq!(arms[1].pattern, MatchPattern::TypeName("MATH".into()));
                assert_eq!(arms[2].pattern, MatchPattern::TypeName("EMOTICON".into()));
                assert_eq!(arms[3].pattern, MatchPattern::Wildcard);
            }
            _ => panic!("Expected Match"),
        }
    }

    #[test]
    fn parse_match_no_wildcard() {
        let stmts = parse("match fire { SDF => { stats; } }").unwrap();
        match &stmts[0] {
            Stmt::Match { arms, .. } => {
                assert_eq!(arms.len(), 1);
            }
            _ => panic!("Expected Match"),
        }
    }

    // ── Try/Catch ───────────────────────────────────────────────────────

    #[test]
    fn parse_try_catch_basic() {
        let stmts = parse("try { emit fire; } catch { stats; }").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::TryCatch { try_block, catch_block } => {
                assert!(!try_block.is_empty());
                assert!(!catch_block.is_empty());
            }
            _ => panic!("Expected TryCatch"),
        }
    }

    #[test]
    fn parse_try_catch_nested() {
        let stmts = parse("try { if fire { emit water; } } catch { dream; }").unwrap();
        match &stmts[0] {
            Stmt::TryCatch { try_block, .. } => {
                assert!(matches!(&try_block[0], Stmt::If { .. }));
            }
            _ => panic!("Expected TryCatch"),
        }
    }

    // ── For-In ────────────────────────────────────────────────────────

    #[test]
    fn parse_for_in_basic() {
        let stmts = parse("for i in 0..5 { emit fire; }").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::ForIn { var, start, end, body } => {
                assert_eq!(var, "i");
                assert_eq!(*start, 0);
                assert_eq!(*end, 5);
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected ForIn"),
        }
    }

    #[test]
    fn parse_for_in_with_nested_body() {
        let stmts = parse("for x in 1..10 { if fire { emit water; } }").unwrap();
        match &stmts[0] {
            Stmt::ForIn { var, start, end, body } => {
                assert_eq!(var, "x");
                assert_eq!(*start, 1);
                assert_eq!(*end, 10);
                assert!(matches!(&body[0], Stmt::If { .. }));
            }
            _ => panic!("Expected ForIn"),
        }
    }

    #[test]
    fn lex_for_in_tokens() {
        use crate::alphabet::{Lexer, Token};
        let tokens = Lexer::tokenize_all("for i in 0..5 { }");
        assert_eq!(tokens[0], Token::For);
        assert_eq!(tokens[1], Token::Ident("i".into()));
        assert_eq!(tokens[2], Token::In);
        assert_eq!(tokens[3], Token::Int(0));
        assert_eq!(tokens[4], Token::DotDot);
        assert_eq!(tokens[5], Token::Int(5));
    }

    #[test]
    fn parse_while_basic() {
        let stmts = parse("while x < 10 { emit x; }").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::While { cond, body } => {
                // cond should be Compare(x < 10)
                match cond {
                    Expr::Compare { op, .. } => assert_eq!(*op, CmpOp::Lt),
                    _ => panic!("Expected Compare, got {:?}", cond),
                }
                assert!(!body.is_empty());
            }
            _ => panic!("Expected While, got {:?}", stmts[0]),
        }
    }

    #[test]
    fn parse_compare_lt() {
        let stmts = parse("emit x < 10;").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Emit(expr) => match expr {
                Expr::Compare { op, .. } => assert_eq!(*op, CmpOp::Lt),
                _ => panic!("Expected Compare, got {:?}", expr),
            },
            _ => panic!("Expected Emit"),
        }
    }

    #[test]
    fn parse_compare_ge() {
        let stmts = parse("emit x >= 5;").unwrap();
        match &stmts[0] {
            Stmt::Emit(expr) => match expr {
                Expr::Compare { op, .. } => assert_eq!(*op, CmpOp::Ge),
                _ => panic!("Expected Compare, got {:?}", expr),
            },
            _ => panic!("Expected Emit"),
        }
    }

    #[test]
    fn lex_comparison_tokens() {
        use crate::alphabet::{Lexer, Token};
        let tokens = Lexer::tokenize_all("x < 10");
        assert!(tokens.contains(&Token::Lt));
        let tokens2 = Lexer::tokenize_all("x >= 5");
        assert!(tokens2.contains(&Token::Ge));
        let tokens3 = Lexer::tokenize_all("x <= 3");
        assert!(tokens3.contains(&Token::Le));
    }

    #[test]
    fn parse_ne() {
        let stmts = parse("emit x != 5;").unwrap();
        match &stmts[0] {
            Stmt::Emit(expr) => match expr {
                Expr::Compare { op, .. } => assert_eq!(*op, CmpOp::Ne),
                _ => panic!("Expected Compare, got {:?}", expr),
            },
            _ => panic!("Expected Emit"),
        }
    }

    #[test]
    fn lex_ne_and_not() {
        use crate::alphabet::{Lexer, Token};
        let tokens = Lexer::tokenize_all("x != 5");
        assert!(tokens.contains(&Token::Ne));
        let tokens2 = Lexer::tokenize_all("!x");
        assert!(tokens2.contains(&Token::Not));
    }

    #[test]
    fn lex_and_or() {
        use crate::alphabet::{Lexer, Token};
        let tokens = Lexer::tokenize_all("a && b");
        assert!(tokens.contains(&Token::And));
        let tokens2 = Lexer::tokenize_all("a || b");
        assert!(tokens2.contains(&Token::Or));
    }

    #[test]
    fn parse_logic_not() {
        let stmts = parse("emit !x;").unwrap();
        match &stmts[0] {
            Stmt::Emit(expr) => match expr {
                Expr::LogicNot(_) => {} // ok
                _ => panic!("Expected LogicNot, got {:?}", expr),
            },
            _ => panic!("Expected Emit"),
        }
    }

    #[test]
    fn parse_logic_and() {
        let stmts = parse("emit a && b;").unwrap();
        match &stmts[0] {
            Stmt::Emit(expr) => match expr {
                Expr::LogicAnd(_, _) => {} // ok
                _ => panic!("Expected LogicAnd, got {:?}", expr),
            },
            _ => panic!("Expected Emit"),
        }
    }

    #[test]
    fn parse_logic_or() {
        let stmts = parse("emit a || b;").unwrap();
        match &stmts[0] {
            Stmt::Emit(expr) => match expr {
                Expr::LogicOr(_, _) => {} // ok
                _ => panic!("Expected LogicOr, got {:?}", expr),
            },
            _ => panic!("Expected Emit"),
        }
    }

    #[test]
    fn parse_break_stmt() {
        let stmts = parse("while x < 10 { break; }").unwrap();
        match &stmts[0] {
            Stmt::While { body, .. } => {
                assert!(matches!(&body[0], Stmt::Break));
            }
            _ => panic!("Expected While"),
        }
    }

    #[test]
    fn parse_continue_stmt() {
        let stmts = parse("for i in 0..5 { continue; }").unwrap();
        match &stmts[0] {
            Stmt::ForIn { body, .. } => {
                assert!(matches!(&body[0], Stmt::Continue));
            }
            _ => panic!("Expected ForIn"),
        }
    }

    // ── Type system: struct, enum, impl, trait ──────────────────────────────

    #[test]
    fn parse_struct_basic() {
        let stmts = parse("struct Point { x, y }").unwrap();
        match &stmts[0] {
            Stmt::StructDef { name, type_params, fields } => {
                assert_eq!(name, "Point");
                assert!(type_params.is_empty());
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "x");
                assert_eq!(fields[1].name, "y");
            }
            _ => panic!("Expected StructDef"),
        }
    }

    #[test]
    fn parse_struct_with_type_params() {
        let stmts = parse("struct Pair[T, U] { first: T, second: U }").unwrap();
        match &stmts[0] {
            Stmt::StructDef { name, type_params, fields } => {
                assert_eq!(name, "Pair");
                assert_eq!(type_params, &["T", "U"]);
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "first");
                assert_eq!(fields[0].type_name.as_deref(), Some("T"));
                assert_eq!(fields[1].name, "second");
                assert_eq!(fields[1].type_name.as_deref(), Some("U"));
            }
            _ => panic!("Expected StructDef"),
        }
    }

    #[test]
    fn parse_enum_basic() {
        let stmts = parse("enum Color { Red, Green, Blue }").unwrap();
        match &stmts[0] {
            Stmt::EnumDef { name, type_params, variants } => {
                assert_eq!(name, "Color");
                assert!(type_params.is_empty());
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].name, "Red");
                assert!(variants[0].fields.is_empty());
            }
            _ => panic!("Expected EnumDef"),
        }
    }

    #[test]
    fn parse_enum_with_type_params() {
        let stmts = parse("enum Option[T] { Some(T), None }").unwrap();
        match &stmts[0] {
            Stmt::EnumDef { name, type_params, variants } => {
                assert_eq!(name, "Option");
                assert_eq!(type_params, &["T"]);
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Some");
                assert_eq!(variants[0].fields, vec!["T"]);
                assert_eq!(variants[1].name, "None");
                assert!(variants[1].fields.is_empty());
            }
            _ => panic!("Expected EnumDef"),
        }
    }

    #[test]
    fn parse_impl_block() {
        let src = r#"
            impl Point {
                fn new(x, y) { emit x; }
            }
        "#;
        let stmts = parse(src).unwrap();
        match &stmts[0] {
            Stmt::ImplBlock { target, methods } => {
                assert_eq!(target, "Point");
                assert_eq!(methods.len(), 1);
            }
            _ => panic!("Expected ImplBlock"),
        }
    }

    #[test]
    fn parse_trait_def() {
        let src = r#"
            trait Drawable {
                fn draw(self);
                fn area(self, scale);
            }
        "#;
        let stmts = parse(src).unwrap();
        match &stmts[0] {
            Stmt::TraitDef { name, type_params, methods } => {
                assert_eq!(name, "Drawable");
                assert!(type_params.is_empty());
                assert_eq!(methods.len(), 2);
                assert_eq!(methods[0].name, "draw");
                assert_eq!(methods[0].params, vec!["self"]);
                assert_eq!(methods[1].name, "area");
                assert_eq!(methods[1].params, vec!["self", "scale"]);
            }
            _ => panic!("Expected TraitDef"),
        }
    }

    #[test]
    fn parse_trait_with_type_params() {
        let src = r#"
            trait Container[T] {
                fn get(self);
                fn put(self, item);
            }
        "#;
        let stmts = parse(src).unwrap();
        match &stmts[0] {
            Stmt::TraitDef { name, type_params, methods } => {
                assert_eq!(name, "Container");
                assert_eq!(type_params, &["T"]);
                assert_eq!(methods.len(), 2);
            }
            _ => panic!("Expected TraitDef"),
        }
    }

    #[test]
    fn parse_impl_trait_for_type() {
        let src = r#"
            impl Drawable for Circle {
                fn draw(self) { emit self; }
            }
        "#;
        let stmts = parse(src).unwrap();
        match &stmts[0] {
            Stmt::ImplTrait { trait_name, target, methods } => {
                assert_eq!(trait_name, "Drawable");
                assert_eq!(target, "Circle");
                assert_eq!(methods.len(), 1);
            }
            _ => panic!("Expected ImplTrait"),
        }
    }

    #[test]
    fn parse_unwrap_or() {
        let stmts = parse("x ?? 0").unwrap();
        match &stmts[0] {
            Stmt::Expr(Expr::UnwrapOr { value, default }) => {
                assert!(matches!(value.as_ref(), Expr::Ident(n) if n == "x"));
                assert!(matches!(default.as_ref(), Expr::Int(0)));
            }
            _ => panic!("Expected UnwrapOr, got {:?}", stmts[0]),
        }
    }

    #[test]
    fn parse_method_call() {
        let stmts = parse("obj.method(arg1, arg2)").unwrap();
        match &stmts[0] {
            Stmt::Expr(Expr::MethodCall { object, method, args }) => {
                assert!(matches!(object.as_ref(), Expr::Ident(n) if n == "obj"));
                assert_eq!(method, "method");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected MethodCall"),
        }
    }

    #[test]
    fn parse_struct_literal() {
        let stmts = parse("Point { x: 1, y: 2 }").unwrap();
        match &stmts[0] {
            Stmt::Expr(Expr::StructLiteral { name, fields }) => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected StructLiteral, got {:?}", stmts[0]),
        }
    }
}
