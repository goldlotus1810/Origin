//! # parser — ○{} Parser
//!
//! 2 mode tuyệt đối:
//!   text thường  → giao tiếp tự nhiên
//!   ○{...}       → parse và thực thi
//!
//! ○ = U+25CB WHITE CIRCLE = QT1 = nguồn gốc
//!
//! Cú pháp trong ○{}:
//!   ○{🔥}              → Query node
//!   ○{lửa}             → Alias lookup
//!   ○{🔥 ∈ ?}          → Relation query
//!   ○{🔥 ∘ 💧}         → Compose (LCA)
//!   ○{? → 💧}           → Reverse query
//!   ○{🔥 ≈ ?}          → Similarity query
//!   ○{bank ∂ finance}  → Context query
//!   ○{○{🔥} ∈ ?}       → Pipeline (nested)
//!   ○{dream}           → System command
//!   ○{stats}           → System command

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// OlangToken
// ─────────────────────────────────────────────────────────────────────────────

/// Token trong ○{} expression.
#[derive(Debug, Clone, PartialEq)]
pub enum OlangToken {
    /// Node identifier (emoji, word, codepoint)
    Node(String),
    /// Relation operator (∈ ⊂ ≡ ∘ → ≈ ∂ ...)
    Relation(RelationOp),
    /// Wildcard ?
    Wildcard,
    /// System command (dream, stats, seed, ...)
    Command(String),
    /// Nested ○{...}
    Nested(Vec<OlangToken>),
}

/// Relation operator trong ○{}.
/// 18 RelOps: 10 gốc + 8 mở rộng (Phase 11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationOp {
    Member,      // ∈ U+2208
    Subset,      // ⊂ U+2282
    Equiv,       // ≡ U+2261
    Compose,     // ∘ U+2218
    Causes,      // → U+2192
    Similar,     // ≈ U+2248
    DerivedFrom, // ← U+2190
    Context,     // ∂ U+2202 (dùng cho "bank ∂ finance")
    Contains,    // ∪ U+222A
    Intersects,  // ∩ U+2229
    // Phase 11: 8 RelOps mở rộng
    Orthogonal,  // ⊥ U+22A5 — vuông góc / độc lập
    SetMinus,    // ∖ U+2216 — loại trừ tập hợp
    Bidir,       // ↔ U+2194 — quan hệ hai chiều
    Flows,       // ⟶ U+27F6 — dòng chảy / pipeline
    Repeats,     // ⟳ U+27F3 — lặp lại / chu kỳ
    Resolves,    // ↑ U+2191 — giải quyết / nâng cấp
    Trigger,     // ⚡ U+26A1 — kích hoạt
    Parallel,    // ∥ U+2225 — song song / đồng thời
}

impl RelationOp {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '∈' => Some(Self::Member),
            '⊂' => Some(Self::Subset),
            '≡' => Some(Self::Equiv),
            '∘' => Some(Self::Compose),
            '→' => Some(Self::Causes),
            '≈' => Some(Self::Similar),
            '←' => Some(Self::DerivedFrom),
            '∂' => Some(Self::Context),
            '∪' => Some(Self::Contains),
            '∩' => Some(Self::Intersects),
            '⊥' => Some(Self::Orthogonal),
            '∖' => Some(Self::SetMinus),
            '↔' => Some(Self::Bidir),
            '⟶' => Some(Self::Flows),
            '⟳' => Some(Self::Repeats),
            '↑' => Some(Self::Resolves),
            '⚡' => Some(Self::Trigger),
            '∥' => Some(Self::Parallel),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Member => "∈",
            Self::Subset => "⊂",
            Self::Equiv => "≡",
            Self::Compose => "∘",
            Self::Causes => "→",
            Self::Similar => "≈",
            Self::DerivedFrom => "←",
            Self::Context => "∂",
            Self::Contains => "∪",
            Self::Intersects => "∩",
            Self::Orthogonal => "⊥",
            Self::SetMinus => "∖",
            Self::Bidir => "↔",
            Self::Flows => "⟶",
            Self::Repeats => "⟳",
            Self::Resolves => "↑",
            Self::Trigger => "⚡",
            Self::Parallel => "∥",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OlangExpr — parsed expression
// ─────────────────────────────────────────────────────────────────────────────

/// Arithmetic operator trong ○{}.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    /// + addition (QT3: giả thuyết)
    Add,
    /// - subtraction (QT3: giả thuyết)
    Sub,
    /// × multiplication (QT3: giả thuyết)
    Mul,
    /// ÷ division (QT3: giả thuyết)
    Div,
}

/// Một expression đã parse từ ○{...}.
#[derive(Debug, Clone, PartialEq)]
pub enum OlangExpr {
    /// ○{🔥} — query node
    Query(String),
    /// ○{🔥 ∈ ?} — relation query
    RelationQuery {
        subject: String,
        relation: RelationOp,
        /// None = wildcard ?
        object: Option<String>,
    },
    /// ○{🔥 ∘ 💧} — compose
    Compose { a: String, b: String },
    /// ○{bank ∂ finance} — context query
    ContextQuery { term: String, context: String },
    /// ○{dream} / ○{stats} / ○{seed L0}
    Command(String),
    /// ○{○{🔥} ∈ ?} — pipeline
    Pipeline(Vec<OlangExpr>),
    /// ○{1 + 2} — arithmetic (QT3: giả thuyết)
    Arithmetic {
        lhs: f64,
        op: ArithOp,
        rhs: f64,
    },
    /// ○{{ S=1 R=6 V=200 A=180 T=4 }} — molecular literal
    MolecularLiteral {
        shape: u8,
        relation: u8,
        valence: u8,
        arousal: u8,
        time: u8,
    },
    /// ○{let x = fire} — variable binding
    LetBinding {
        name: String,
        value: alloc::boxed::Box<OlangExpr>,
    },
    /// ○{if fire { stats } else { dream }} — conditional
    IfElse {
        condition: alloc::boxed::Box<OlangExpr>,
        then_body: Vec<OlangExpr>,
        else_body: Vec<OlangExpr>,
    },
    /// ○{loop 3 { emit fire }} — loop N times
    LoopBlock {
        count: u32,
        body: Vec<OlangExpr>,
    },
    /// ○{fn test { emit fire }} — function definition
    FnDef {
        name: String,
        body: Vec<OlangExpr>,
    },
    /// ○{spawn { loop 3 { stats } }} — concurrent execution (Go-style)
    Spawn {
        body: Vec<OlangExpr>,
    },
    /// ○{fire |> typeof |> emit} — pipe chain (Julia-style)
    Pipe(Vec<OlangExpr>),
    /// ○{use cluster} — import skill/module (Python-style)
    Use(String),
    /// ○{emit fire} — explicit emit (output to caller)
    Emit(alloc::boxed::Box<OlangExpr>),
    /// ○{return fire} — return value from function
    Return(alloc::boxed::Box<OlangExpr>),
    /// ○{match fire { SDF => { stats } _ => { dream } }} — pattern matching
    Match {
        subject: alloc::boxed::Box<OlangExpr>,
        arms: Vec<(String, Vec<OlangExpr>)>, // (pattern_name, body)
    },
    /// ○{try { risky } catch { fallback }} — error handling
    TryCatch {
        try_body: Vec<OlangExpr>,
        catch_body: Vec<OlangExpr>,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// ParseResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả parse một text input.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseResult {
    /// Text thông thường — giao tiếp tự nhiên
    Natural(String),
    /// ○{} expression đã parse
    OlangExpr(OlangExpr),
    /// Parse error
    Error(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// OlangParser
// ─────────────────────────────────────────────────────────────────────────────

/// Parser cho ○{} language.
pub struct OlangParser;

impl OlangParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse text → ParseResult.
    ///
    /// Nếu text bắt đầu với ○{ → parse OlangExpr.
    /// Nếu không → Natural text.
    pub fn parse(&self, input: &str) -> ParseResult {
        let trimmed = input.trim();

        // Detect ○{...}
        if let Some(inner) = extract_olang(trimmed) {
            match self.parse_expr(inner.trim()) {
                Ok(expr) => ParseResult::OlangExpr(expr),
                Err(e) => ParseResult::Error(e),
            }
        } else {
            ParseResult::Natural(trimmed.to_string())
        }
    }

    /// Parse expression bên trong ○{...}.
    fn parse_expr(&self, expr: &str) -> Result<OlangExpr, String> {
        let trimmed = expr.trim();

        // Empty
        if trimmed.is_empty() {
            return Err("Empty expression — use ○{help} for syntax guide".to_string());
        }

        // Unmatched braces
        let open_count = trimmed.chars().filter(|&c| c == '{').count();
        let close_count = trimmed.chars().filter(|&c| c == '}').count();
        if open_count != close_count {
            return Err(format!(
                "Unmatched braces: {} open, {} close",
                open_count, close_count
            ));
        }

        // System commands: dream, stats, seed, ...
        if is_command(trimmed) {
            return Ok(OlangExpr::Command(trimmed.to_string()));
        }

        // Nested pipeline: ○{○{...} ...}
        if trimmed.starts_with('○') {
            if let Some(inner) = extract_olang(trimmed) {
                let inner_expr = self.parse_expr(inner.trim())?;
                return Ok(OlangExpr::Pipeline(alloc::vec![inner_expr]));
            }
        }

        // ZWJ sequence: contains U+200D → ZwjSeq node
        if trimmed.contains('\u{200D}') {
            return Ok(OlangExpr::Query(trimmed.to_string()));
        }

        // Molecular literal: { S=1 R=6 V=200 A=180 T=4 }
        if let Some(mol) = try_parse_molecular_literal(trimmed) {
            return Ok(mol);
        }

        // Let binding: let x = <expr>
        if let Some(binding) = self.try_parse_let(trimmed) {
            return Ok(binding);
        }

        // If/else conditional: if <cond> { <then> } else { <else> }
        if let Some(if_expr) = self.try_parse_if(trimmed) {
            return Ok(if_expr);
        }

        // Loop: loop N { <body> }
        if let Some(loop_expr) = self.try_parse_loop(trimmed) {
            return Ok(loop_expr);
        }

        // Function definition: fn name { <body> }
        if let Some(fn_expr) = self.try_parse_fn(trimmed) {
            return Ok(fn_expr);
        }

        // Spawn (Go-style concurrency): spawn { <body> }
        if let Some(spawn_expr) = self.try_parse_spawn(trimmed) {
            return Ok(spawn_expr);
        }

        // Match (pattern matching): match <expr> { <arm> => { <body> } ... }
        if let Some(match_expr) = self.try_parse_match(trimmed) {
            return Ok(match_expr);
        }

        // Try/catch (error handling): try { body } catch { handler }
        if let Some(try_expr) = self.try_parse_try_catch(trimmed) {
            return Ok(try_expr);
        }

        // Use (Python-style import): use <skill>
        if let Some(use_expr) = try_parse_use(trimmed) {
            return Ok(use_expr);
        }

        // Emit: emit <expr>
        if let Some(emit_expr) = self.try_parse_emit(trimmed) {
            return Ok(emit_expr);
        }

        // Return: return <expr>
        if let Some(ret_expr) = self.try_parse_return(trimmed) {
            return Ok(ret_expr);
        }

        // Pipe (Julia-style): expr |> expr |> expr
        if trimmed.contains("|>") {
            if let Some(pipe_expr) = self.try_parse_pipe(trimmed) {
                return Ok(pipe_expr);
            }
        }

        // Arithmetic: detect numeric expressions like "1 + 2", "3.5 × 4", "10 - 3", "8 ÷ 2"
        if let Some(arith) = try_parse_arithmetic(trimmed) {
            return Ok(arith);
        }

        // + operator with non-numeric operands → Compose (e.g. 🔥 + 💧)
        if trimmed.contains('+') && !trimmed.chars().any(|c| RelationOp::from_char(c).is_some()) {
            return Ok(OlangExpr::Compose {
                a: trimmed.split('+').next().unwrap_or("").trim().to_string(),
                b: trimmed.split('+').nth(1).unwrap_or("").trim().to_string(),
            });
        }

        // Tokenize: split by whitespace, preserve Unicode operators
        let tokens = tokenize(trimmed);

        match tokens.as_slice() {
            // ○{node} — simple query
            [OlangToken::Node(name)] => Ok(OlangExpr::Query(name.clone())),

            // ○{node ∘ node} — compose
            [OlangToken::Node(a), OlangToken::Relation(RelationOp::Compose), OlangToken::Node(b)] => {
                Ok(OlangExpr::Compose {
                    a: a.clone(),
                    b: b.clone(),
                })
            }

            // ○{node ∂ context} — context query
            [OlangToken::Node(term), OlangToken::Relation(RelationOp::Context), OlangToken::Node(ctx)] => {
                Ok(OlangExpr::ContextQuery {
                    term: term.clone(),
                    context: ctx.clone(),
                })
            }

            // ○{node rel ?} — relation query with wildcard
            [OlangToken::Node(subj), OlangToken::Relation(rel), OlangToken::Wildcard] => {
                Ok(OlangExpr::RelationQuery {
                    subject: subj.clone(),
                    relation: *rel,
                    object: None,
                })
            }

            // ○{? rel node} — reverse query
            [OlangToken::Wildcard, OlangToken::Relation(rel), OlangToken::Node(obj)] => {
                Ok(OlangExpr::RelationQuery {
                    subject: "?".to_string(),
                    relation: *rel,
                    object: Some(obj.clone()),
                })
            }

            // ○{node rel node} — binary relation
            [OlangToken::Node(subj), OlangToken::Relation(rel), OlangToken::Node(obj)] => {
                Ok(OlangExpr::RelationQuery {
                    subject: subj.clone(),
                    relation: *rel,
                    object: Some(obj.clone()),
                })
            }

            // Unknown → try as query
            _ => Ok(OlangExpr::Query(trimmed.to_string())),
        }
    }

    /// Try to parse let binding: "let x = fire" or "let x = { S=1 R=6 }"
    fn try_parse_let(&self, s: &str) -> Option<OlangExpr> {
        let trimmed = s.trim();
        if !trimmed.starts_with("let ") {
            return None;
        }
        let rest = trimmed["let ".len()..].trim();
        let eq_pos = rest.find('=')?;
        let name = rest[..eq_pos].trim();
        let value_str = rest[eq_pos + 1..].trim();

        if name.is_empty() || value_str.is_empty() {
            return None;
        }

        // Parse the value expression recursively
        let value_expr = self.parse_expr(value_str).ok()?;

        Some(OlangExpr::LetBinding {
            name: name.to_string(),
            value: alloc::boxed::Box::new(value_expr),
        })
    }

    /// Try to parse if/else: "if fire { stats } else { dream }"
    fn try_parse_if(&self, s: &str) -> Option<OlangExpr> {
        let trimmed = s.trim();
        if !trimmed.starts_with("if ") {
            return None;
        }
        let rest = trimmed["if ".len()..].trim();

        // Find opening brace for then-body
        let then_open = rest.find('{')?;
        let cond_str = rest[..then_open].trim();
        if cond_str.is_empty() {
            return None;
        }

        // Find matching closing brace
        let then_close = find_matching_brace(rest, then_open)?;
        let then_str = rest[then_open + 1..then_close].trim();

        // Parse condition
        let condition = self.parse_expr(cond_str).ok()?;

        // Parse then body (semicolon-separated statements)
        let then_body = self.parse_block(then_str);

        // Check for else clause
        let after_then = rest[then_close + 1..].trim();
        let else_body = if after_then.starts_with("else") {
            let else_rest = after_then.strip_prefix("else").unwrap_or("").trim();
            let else_open = else_rest.find('{')?;
            let else_close = find_matching_brace(else_rest, else_open)?;
            let else_str = else_rest[else_open + 1..else_close].trim();
            self.parse_block(else_str)
        } else {
            Vec::new()
        };

        Some(OlangExpr::IfElse {
            condition: alloc::boxed::Box::new(condition),
            then_body,
            else_body,
        })
    }

    /// Try to parse loop: "loop 3 { emit fire }"
    fn try_parse_loop(&self, s: &str) -> Option<OlangExpr> {
        let trimmed = s.trim();
        if !trimmed.starts_with("loop ") {
            return None;
        }
        let rest = trimmed["loop ".len()..].trim();

        // Find opening brace
        let brace_open = rest.find('{')?;
        let count_str = rest[..brace_open].trim();
        let count: u32 = count_str.parse().ok()?;

        if count == 0 {
            return None;
        }

        // Find matching closing brace
        let brace_close = find_matching_brace(rest, brace_open)?;
        let body_str = rest[brace_open + 1..brace_close].trim();

        let body = self.parse_block(body_str);

        Some(OlangExpr::LoopBlock { count, body })
    }

    /// Try to parse function definition: "fn test { emit fire }"
    fn try_parse_fn(&self, s: &str) -> Option<OlangExpr> {
        let trimmed = s.trim();
        if !trimmed.starts_with("fn ") {
            return None;
        }
        let rest = trimmed["fn ".len()..].trim();

        // Find opening brace
        let brace_open = rest.find('{')?;
        let name = rest[..brace_open].trim();
        if name.is_empty() {
            return None;
        }

        // Find matching closing brace
        let brace_close = find_matching_brace(rest, brace_open)?;
        let body_str = rest[brace_open + 1..brace_close].trim();

        let body = self.parse_block(body_str);

        Some(OlangExpr::FnDef {
            name: name.to_string(),
            body,
        })
    }

    /// Try to parse spawn (Go-style concurrency): "spawn { stats; dream }"
    fn try_parse_spawn(&self, s: &str) -> Option<OlangExpr> {
        let trimmed = s.trim();
        if !trimmed.starts_with("spawn ") && !trimmed.starts_with("spawn{") {
            return None;
        }
        let rest = if let Some(r) = trimmed.strip_prefix("spawn ") {
            r.trim()
        } else if let Some(r) = trimmed.strip_prefix("spawn") {
            r
        } else {
            return None;
        };
        let brace_open = rest.find('{')?;
        let brace_close = find_matching_brace(rest, brace_open)?;
        let body_str = rest[brace_open + 1..brace_close].trim();
        let body = self.parse_block(body_str);
        Some(OlangExpr::Spawn { body })
    }

    /// Try to parse match: "match fire { SDF => { stats } _ => { dream } }"
    fn try_parse_match(&self, s: &str) -> Option<OlangExpr> {
        let trimmed = s.trim();
        if !trimmed.starts_with("match ") {
            return None;
        }
        let rest = trimmed.strip_prefix("match ")?.trim();

        // Find the outer { that starts the match arms
        let brace_open = rest.find('{')?;
        let subject_str = rest[..brace_open].trim();
        if subject_str.is_empty() {
            return None;
        }
        let subject = self.parse_expr(subject_str).ok()?;

        let brace_close = find_matching_brace(rest, brace_open)?;
        let arms_str = rest[brace_open + 1..brace_close].trim();

        // Parse arms: "pattern => { body }, pattern => { body }"
        let mut arms = Vec::new();
        let mut remaining = arms_str;
        while !remaining.is_empty() {
            remaining = remaining.trim();
            if remaining.is_empty() {
                break;
            }
            // Find =>
            let arrow = remaining.find("=>")?;
            let pattern = remaining[..arrow].trim().to_string();
            remaining = remaining[arrow + 2..].trim();

            // Parse body: { ... }
            let body_open = remaining.find('{')?;
            let body_close = find_matching_brace(remaining, body_open)?;
            let body_str = remaining[body_open + 1..body_close].trim();
            let body = self.parse_block(body_str);

            arms.push((pattern, body));
            remaining = remaining[body_close + 1..].trim();
            // Skip comma/semicolon separator
            if remaining.starts_with(',') || remaining.starts_with(';') {
                remaining = &remaining[1..];
            }
        }

        Some(OlangExpr::Match {
            subject: alloc::boxed::Box::new(subject),
            arms,
        })
    }

    /// Try to parse try/catch: "try { risky } catch { fallback }"
    fn try_parse_try_catch(&self, s: &str) -> Option<OlangExpr> {
        let trimmed = s.trim();
        if !trimmed.starts_with("try ") && !trimmed.starts_with("try{") {
            return None;
        }
        let rest = if let Some(r) = trimmed.strip_prefix("try ") {
            r.trim()
        } else if let Some(r) = trimmed.strip_prefix("try") {
            r
        } else {
            return None;
        };
        let try_open = rest.find('{')?;
        let try_close = find_matching_brace(rest, try_open)?;
        let try_str = rest[try_open + 1..try_close].trim();
        let try_body = self.parse_block(try_str);

        let after_try = rest[try_close + 1..].trim();
        let catch_rest = after_try.strip_prefix("catch")?.trim();
        let catch_open = catch_rest.find('{')?;
        let catch_close = find_matching_brace(catch_rest, catch_open)?;
        let catch_str = catch_rest[catch_open + 1..catch_close].trim();
        let catch_body = self.parse_block(catch_str);

        Some(OlangExpr::TryCatch {
            try_body,
            catch_body,
        })
    }

    /// Try to parse pipe (Julia-style): "fire |> typeof |> emit"
    fn try_parse_pipe(&self, s: &str) -> Option<OlangExpr> {
        let parts: Vec<&str> = s.split("|>").collect();
        if parts.len() < 2 {
            return None;
        }
        let exprs: Vec<OlangExpr> = parts
            .iter()
            .filter_map(|part| {
                let t = part.trim();
                if t.is_empty() {
                    None
                } else {
                    self.parse_expr(t).ok()
                }
            })
            .collect();
        if exprs.len() >= 2 {
            Some(OlangExpr::Pipe(exprs))
        } else {
            None
        }
    }

    /// Try to parse emit: "emit fire"
    fn try_parse_emit(&self, s: &str) -> Option<OlangExpr> {
        let trimmed = s.trim();
        if !trimmed.starts_with("emit ") {
            return None;
        }
        let arg = trimmed["emit ".len()..].trim();
        if arg.is_empty() {
            return None;
        }
        let expr = self.parse_expr(arg).ok()?;
        Some(OlangExpr::Emit(alloc::boxed::Box::new(expr)))
    }

    /// Try to parse return: "return fire"
    fn try_parse_return(&self, s: &str) -> Option<OlangExpr> {
        let trimmed = s.trim();
        if !trimmed.starts_with("return ") {
            return None;
        }
        let arg = trimmed["return ".len()..].trim();
        if arg.is_empty() {
            return None;
        }
        let expr = self.parse_expr(arg).ok()?;
        Some(OlangExpr::Return(alloc::boxed::Box::new(expr)))
    }

    /// Parse a block of semicolon-separated statements into Vec<OlangExpr>.
    fn parse_block(&self, block: &str) -> Vec<OlangExpr> {
        if block.is_empty() {
            return Vec::new();
        }
        block
            .split(';')
            .filter_map(|stmt| {
                let s = stmt.trim();
                if s.is_empty() {
                    None
                } else {
                    self.parse_expr(s).ok()
                }
            })
            .collect()
    }
}

/// Try to parse use/import: "use cluster" or "use similarity"
fn try_parse_use(s: &str) -> Option<OlangExpr> {
    let trimmed = s.trim();
    if !trimmed.starts_with("use ") {
        return None;
    }
    let module = trimmed["use ".len()..].trim();
    if module.is_empty() {
        return None;
    }
    Some(OlangExpr::Use(module.to_string()))
}

/// Find matching closing brace for opening brace at `open_pos`.
fn find_matching_brace(s: &str, open_pos: usize) -> Option<usize> {
    let mut depth = 0u32;
    for (i, c) in s[open_pos..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_pos + i);
                }
            }
            _ => {}
        }
    }
    None
}

impl Default for OlangParser {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Extract content bên trong ○{...}.
/// Returns None nếu không phải ○{...}.
pub fn extract_olang(s: &str) -> Option<&str> {
    // ○ = U+25CB = 3 bytes UTF-8: 0xE2 0x97 0x8B
    let mut chars = s.char_indices();
    let (_, first) = chars.next()?;
    if first != '○' {
        return None;
    }

    let (next_pos, next) = chars.next()?;
    if next != '{' {
        return None;
    }

    let start = next_pos + '{'.len_utf8();

    // Tìm closing } (handle nested)
    let mut depth = 1usize;
    let mut end = start;
    for (i, c) in s[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    } // Không đóng

    Some(&s[start..end])
}

/// Tokenize expression.
fn tokenize(expr: &str) -> Vec<OlangToken> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for c in expr.chars() {
        // Relation operator
        if let Some(op) = RelationOp::from_char(c) {
            // Flush current
            let word = current.trim().to_string();
            if !word.is_empty() {
                tokens.push(token_from_str(&word));
            }
            current.clear();
            tokens.push(OlangToken::Relation(op));
        } else if c == '?' {
            let word = current.trim().to_string();
            if !word.is_empty() {
                tokens.push(token_from_str(&word));
            }
            current.clear();
            tokens.push(OlangToken::Wildcard);
        } else if c.is_whitespace() {
            let word = current.trim().to_string();
            if !word.is_empty() {
                tokens.push(token_from_str(&word));
                current.clear();
            }
        } else {
            current.push(c);
        }
    }

    // Flush last
    let word = current.trim().to_string();
    if !word.is_empty() {
        tokens.push(token_from_str(&word));
    }

    tokens
}

fn token_from_str(s: &str) -> OlangToken {
    OlangToken::Node(s.to_string())
}

/// Try to parse molecular literal: { S=1 R=6 V=200 A=180 T=4 }
///
/// All 5 dimensions optional, defaults: S=1 R=1 V=128 A=128 T=3
fn try_parse_molecular_literal(s: &str) -> Option<OlangExpr> {
    let trimmed = s.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return None;
    }
    let inner = trimmed[1..trimmed.len() - 1].trim();
    if inner.is_empty() {
        return None;
    }

    // Defaults (from CLAUDE.md semantic: Sphere, Member, neutral, Medium)
    let mut shape: u8 = 1;    // Sphere
    let mut relation: u8 = 1; // Member
    let mut valence: u8 = 128; // Neutral
    let mut arousal: u8 = 128; // Neutral
    let mut time: u8 = 3;     // Medium

    let mut found_any = false;
    for part in inner.split_whitespace() {
        let kv: alloc::vec::Vec<&str> = part.splitn(2, '=').collect();
        if kv.len() != 2 {
            return None; // not a key=value pair
        }
        let key = kv[0].trim();
        let val: u8 = match kv[1].trim().parse() {
            Ok(v) => v,
            Err(_) => return None,
        };
        match key {
            "S" | "s" => shape = val,
            "R" | "r" => relation = val,
            "V" | "v" => valence = val,
            "A" | "a" => arousal = val,
            "T" | "t" => time = val,
            _ => return None, // unknown dimension
        }
        found_any = true;
    }

    if !found_any {
        return None;
    }

    Some(OlangExpr::MolecularLiteral { shape, relation, valence, arousal, time })
}

/// Try to parse arithmetic expression: "1 + 2", "3.14 × 2", "10 - 3", "8 ÷ 2"
/// Returns None if not a valid arithmetic expression (falls through to Compose/Query).
fn try_parse_arithmetic(s: &str) -> Option<OlangExpr> {
    // Find arithmetic operator (scan left to right, last +/- wins for precedence,
    // but we keep it simple: single operator between two numbers)
    let ops: &[(char, ArithOp)] = &[
        ('+', ArithOp::Add),
        ('-', ArithOp::Sub),
        ('×', ArithOp::Mul),
        ('÷', ArithOp::Div),
        ('*', ArithOp::Mul),
        ('/', ArithOp::Div),
    ];

    for &(op_char, ref op) in ops {
        // Split on operator — find it (skip if it's a negative sign at start)
        if let Some(pos) = find_arith_op(s, op_char) {
            let lhs_str = s[..pos].trim();
            let rhs_str = s[pos + op_char.len_utf8()..].trim();

            if let (Ok(lhs), Ok(rhs)) = (parse_number(lhs_str), parse_number(rhs_str)) {
                return Some(OlangExpr::Arithmetic {
                    lhs,
                    op: *op,
                    rhs,
                });
            }
        }
    }
    None
}

/// Find arithmetic operator position, skipping leading negative sign.
fn find_arith_op(s: &str, op: char) -> Option<usize> {
    let mut chars = s.char_indices().peekable();
    // Skip leading whitespace and optional leading minus (negative number)
    while let Some(&(_, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    // Skip leading minus (negative number, not subtraction)
    if let Some(&(_, '-')) = chars.peek() {
        chars.next();
        // Skip digits/dots after minus
        while let Some(&(_, c)) = chars.peek() {
            if c.is_ascii_digit() || c == '.' {
                chars.next();
            } else {
                break;
            }
        }
    } else {
        // Skip digits/dots of first number
        while let Some(&(_, c)) = chars.peek() {
            if c.is_ascii_digit() || c == '.' {
                chars.next();
            } else {
                break;
            }
        }
    }
    // Now look for the operator
    for (i, c) in chars {
        if c == op {
            return Some(i);
        }
    }
    None
}

/// Parse number string, supporting integers and decimals.
fn parse_number(s: &str) -> Result<f64, ()> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(());
    }
    // Only allow valid numeric characters
    let valid = trimmed.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-');
    if !valid {
        return Err(());
    }
    trimmed.parse::<f64>().map_err(|_| ())
}

fn is_command(s: &str) -> bool {
    matches!(
        s,
        "dream"
            | "stats"
            | "health"
            | "seed L0"
            | "seed"
            | "shutdown"
            | "reboot"
            | "status"
            | "help"
    ) || s.starts_with("compile ")
        || is_math_command(s)
        || is_constant_command(s)
        || is_leo_command(s)
}

/// Check if input is a LeoAI programming command.
fn is_leo_command(s: &str) -> bool {
    s.starts_with("leo ") || s.starts_with("program ") || s.starts_with("run ")
}

/// Check if input is a math command (prefix-based).
fn is_math_command(s: &str) -> bool {
    let prefixes = [
        "solve ", "giai ", "derive ", "derivative ", "dao-ham ", "d/dx ",
        "integrate ", "integral ", "tich-phan ", "simplify ", "eval ",
    ];
    prefixes.iter().any(|p| s.starts_with(p))
}

/// Check if input is a constant/fibonacci command.
fn is_constant_command(s: &str) -> bool {
    s.starts_with("const ") || s.starts_with("hang-so ") || s.starts_with("fib ") || s.starts_with("fibonacci ")
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> OlangParser {
        OlangParser::new()
    }

    // ── Natural text ─────────────────────────────────────────────────────────

    #[test]
    fn natural_text_passthrough() {
        let r = parser().parse("hôm nay trời đẹp quá");
        assert_eq!(r, ParseResult::Natural("hôm nay trời đẹp quá".to_string()));
    }

    #[test]
    fn natural_text_no_circle() {
        let r = parser().parse("tắt đèn phòng khách");
        assert!(matches!(r, ParseResult::Natural(_)));
    }

    // ── ○{} detection ────────────────────────────────────────────────────────

    #[test]
    fn extract_olang_basic() {
        assert_eq!(extract_olang("○{hello}"), Some("hello"));
        assert_eq!(extract_olang("○{🔥}"), Some("🔥"));
        assert_eq!(extract_olang("hello"), None);
        assert_eq!(extract_olang("○ hello"), None);
    }

    #[test]
    fn extract_olang_nested() {
        assert_eq!(extract_olang("○{○{🔥} ∈ ?}"), Some("○{🔥} ∈ ?"));
    }

    #[test]
    fn extract_olang_unclosed() {
        assert_eq!(extract_olang("○{hello"), None);
    }

    // ── Query ─────────────────────────────────────────────────────────────────

    #[test]
    fn parse_simple_query() {
        let r = parser().parse("○{🔥}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Query("🔥".to_string()))
        );
    }

    #[test]
    fn parse_word_query() {
        let r = parser().parse("○{lửa}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Query("lửa".to_string()))
        );
    }

    // ── Relation queries ──────────────────────────────────────────────────────

    #[test]
    fn parse_relation_member() {
        let r = parser().parse("○{🔥 ∈ ?}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::Member,
                object: None,
            })
        );
    }

    #[test]
    fn parse_reverse_query() {
        let r = parser().parse("○{? → 💧}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "?".to_string(),
                relation: RelationOp::Causes,
                object: Some("💧".to_string()),
            })
        );
    }

    #[test]
    fn parse_similarity_query() {
        let r = parser().parse("○{🔥 ≈ ?}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::Similar,
                object: None,
            })
        );
    }

    // ── Compose ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_compose() {
        let r = parser().parse("○{🔥 ∘ 💧}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Compose {
                a: "🔥".to_string(),
                b: "💧".to_string(),
            })
        );
    }

    // ── Context query ─────────────────────────────────────────────────────────

    #[test]
    fn parse_context_query() {
        let r = parser().parse("○{bank ∂ finance}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::ContextQuery {
                term: "bank".to_string(),
                context: "finance".to_string(),
            })
        );
    }

    // ── Commands ──────────────────────────────────────────────────────────────

    #[test]
    fn parse_dream_command() {
        let r = parser().parse("○{dream}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Command("dream".to_string()))
        );
    }

    #[test]
    fn parse_stats_command() {
        let r = parser().parse("○{stats}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Command("stats".to_string()))
        );
    }

    // ── Tiếng Việt ────────────────────────────────────────────────────────────

    #[test]
    fn parse_vietnamese_query() {
        let r = parser().parse("○{lửa ∘ nước}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Compose {
                a: "lửa".to_string(),
                b: "nước".to_string(),
            })
        );
    }

    #[test]
    fn parse_vietnamese_context() {
        let r = parser().parse("○{ngân hàng ∂ tài chính}");
        // "ngân hàng" có space → 2 tokens → fallback to Query
        // Đây là expected behavior — multi-word node cần quote hoặc viết liền
        assert!(matches!(r, ParseResult::OlangExpr(_)));
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn parse_empty_olang() {
        let r = parser().parse("○{}");
        assert!(matches!(r, ParseResult::Error(_)));
    }

    #[test]
    fn parse_whitespace_trimmed() {
        let r = parser().parse("  ○{  🔥  }  ");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Query("🔥".to_string()))
        );
    }

    #[test]
    fn parse_circle_without_brace_is_natural() {
        // ○ standalone = SDF Torus node trong text bình thường
        let r = parser().parse("○ là hình tròn");
        assert!(matches!(r, ParseResult::Natural(_)));
    }

    // ── Math commands ───────────────────────────────────────────────────────

    #[test]
    fn parse_solve_command() {
        let r = parser().parse("○{solve \"2x + 3 = 7\"}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Command("solve \"2x + 3 = 7\"".to_string()))
        );
    }

    #[test]
    fn parse_derive_command() {
        let r = parser().parse("○{derive \"x^2 + 3x\"}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Command("derive \"x^2 + 3x\"".to_string()))
        );
    }

    #[test]
    fn parse_integrate_command() {
        let r = parser().parse("○{integrate \"2x\"}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Command("integrate \"2x\"".to_string()))
        );
    }

    #[test]
    fn parse_simplify_command() {
        let r = parser().parse("○{simplify \"2x + 3x\"}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Command("simplify \"2x + 3x\"".to_string()))
        );
    }

    #[test]
    fn parse_eval_command() {
        let r = parser().parse("○{eval \"x^2 + 1\" x=3}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Command("eval \"x^2 + 1\" x=3".to_string()))
        );
    }

    // ── Constant commands ─────────────────────────────────────────────────

    #[test]
    fn parse_const_command() {
        let r = parser().parse("○{const pi}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Command("const pi".to_string()))
        );
    }

    #[test]
    fn parse_const_all_command() {
        let r = parser().parse("○{const all}");
        assert!(matches!(r, ParseResult::OlangExpr(OlangExpr::Command(_))));
    }

    #[test]
    fn parse_fib_command() {
        let r = parser().parse("○{fib 10}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Command("fib 10".to_string()))
        );
    }

    #[test]
    fn parse_fib_ratio_command() {
        let r = parser().parse("○{fib ratio 20}");
        assert!(matches!(r, ParseResult::OlangExpr(OlangExpr::Command(_))));
    }

    // ── Arithmetic ─────────────────────────────────────────────────────

    #[test]
    fn parse_arithmetic_add() {
        let r = parser().parse("○{1 + 2}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Arithmetic {
                lhs: 1.0,
                op: ArithOp::Add,
                rhs: 2.0,
            })
        );
    }

    #[test]
    fn parse_arithmetic_sub() {
        let r = parser().parse("○{10 - 3}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Arithmetic {
                lhs: 10.0,
                op: ArithOp::Sub,
                rhs: 3.0,
            })
        );
    }

    #[test]
    fn parse_arithmetic_mul() {
        let r = parser().parse("○{6 × 7}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Arithmetic {
                lhs: 6.0,
                op: ArithOp::Mul,
                rhs: 7.0,
            })
        );
    }

    #[test]
    fn parse_arithmetic_div() {
        let r = parser().parse("○{8 ÷ 2}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Arithmetic {
                lhs: 8.0,
                op: ArithOp::Div,
                rhs: 2.0,
            })
        );
    }

    #[test]
    fn parse_arithmetic_mul_ascii() {
        let r = parser().parse("○{3 * 4}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Arithmetic {
                lhs: 3.0,
                op: ArithOp::Mul,
                rhs: 4.0,
            })
        );
    }

    #[test]
    fn parse_arithmetic_div_ascii() {
        let r = parser().parse("○{10 / 5}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Arithmetic {
                lhs: 10.0,
                op: ArithOp::Div,
                rhs: 5.0,
            })
        );
    }

    #[test]
    fn parse_arithmetic_decimal() {
        let r = parser().parse("○{3.14 + 2.86}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::Arithmetic {
                lhs: 3.14,
                op: ArithOp::Add,
                rhs: 2.86,
            })
        );
    }

    #[test]
    fn parse_non_numeric_plus_is_compose() {
        // Non-numeric operands → Compose (emoji + emoji)
        let r = parser().parse("○{🔥 + 💧}");
        // Should NOT be Arithmetic since 🔥 is not a number
        assert!(matches!(r, ParseResult::OlangExpr(OlangExpr::Compose { .. })));
    }

    #[test]
    fn parse_vietnamese_math_commands() {
        let r = parser().parse("○{giai \"x + 5 = 10\"}");
        assert!(matches!(r, ParseResult::OlangExpr(OlangExpr::Command(_))));

        let r = parser().parse("○{dao-ham \"x^2\"}");
        assert!(matches!(r, ParseResult::OlangExpr(OlangExpr::Command(_))));

        let r = parser().parse("○{tich-phan \"3x^2\"}");
        assert!(matches!(r, ParseResult::OlangExpr(OlangExpr::Command(_))));
    }

    // ── Phase 11: New RelOps ────────────────────────────────────────────────

    #[test]
    fn parse_orthogonal() {
        let r = parser().parse("○{🔥 ⊥ 💧}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::Orthogonal,
                object: Some("💧".to_string()),
            })
        );
    }

    #[test]
    fn parse_setminus() {
        let r = parser().parse("○{🔥 ∖ 💧}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::SetMinus,
                object: Some("💧".to_string()),
            })
        );
    }

    #[test]
    fn parse_bidir() {
        let r = parser().parse("○{🔥 ↔ 💧}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::Bidir,
                object: Some("💧".to_string()),
            })
        );
    }

    #[test]
    fn parse_flows() {
        let r = parser().parse("○{🔥 ⟶ 💧}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::Flows,
                object: Some("💧".to_string()),
            })
        );
    }

    #[test]
    fn parse_repeats() {
        let r = parser().parse("○{🔥 ⟳ ?}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::Repeats,
                object: None,
            })
        );
    }

    #[test]
    fn parse_resolves() {
        let r = parser().parse("○{🔥 ↑ ?}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::Resolves,
                object: None,
            })
        );
    }

    #[test]
    fn parse_trigger() {
        let r = parser().parse("○{🔥 ⚡ 💧}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::Trigger,
                object: Some("💧".to_string()),
            })
        );
    }

    #[test]
    fn parse_parallel() {
        let r = parser().parse("○{🔥 ∥ 💧}");
        assert_eq!(
            r,
            ParseResult::OlangExpr(OlangExpr::RelationQuery {
                subject: "🔥".to_string(),
                relation: RelationOp::Parallel,
                object: Some("💧".to_string()),
            })
        );
    }

    #[test]
    fn relop_roundtrip_all_18() {
        let chars = [
            '∈', '⊂', '≡', '∘', '→', '≈', '←', '∂', '∪', '∩',
            '⊥', '∖', '↔', '⟶', '⟳', '↑', '⚡', '∥',
        ];
        for c in chars {
            let op = RelationOp::from_char(c).unwrap_or_else(|| panic!("from_char failed for {c}"));
            let s = op.as_str();
            assert_eq!(s.chars().next().unwrap(), c, "roundtrip failed for {c}");
        }
    }

    // ── If/Else ────────────────────────────────────────────────────────────

    #[test]
    fn parse_if_then() {
        let r = parser().parse("○{if fire { stats }}");
        match r {
            ParseResult::OlangExpr(OlangExpr::IfElse {
                condition,
                then_body,
                else_body,
            }) => {
                assert_eq!(*condition, OlangExpr::Query("fire".to_string()));
                assert_eq!(then_body.len(), 1);
                assert_eq!(then_body[0], OlangExpr::Command("stats".to_string()));
                assert!(else_body.is_empty());
            }
            other => panic!("expected IfElse, got {:?}", other),
        }
    }

    #[test]
    fn parse_if_else() {
        let r = parser().parse("○{if fire { stats } else { dream }}");
        match r {
            ParseResult::OlangExpr(OlangExpr::IfElse {
                condition,
                then_body,
                else_body,
            }) => {
                assert_eq!(*condition, OlangExpr::Query("fire".to_string()));
                assert_eq!(then_body.len(), 1);
                assert_eq!(else_body.len(), 1);
                assert_eq!(else_body[0], OlangExpr::Command("dream".to_string()));
            }
            other => panic!("expected IfElse, got {:?}", other),
        }
    }

    #[test]
    fn parse_if_multi_stmt() {
        let r = parser().parse("○{if fire { stats; dream }}");
        match r {
            ParseResult::OlangExpr(OlangExpr::IfElse { then_body, .. }) => {
                assert_eq!(then_body.len(), 2);
            }
            other => panic!("expected IfElse, got {:?}", other),
        }
    }

    // ── Loop ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_loop_basic() {
        let r = parser().parse("○{loop 3 { stats }}");
        match r {
            ParseResult::OlangExpr(OlangExpr::LoopBlock { count, body }) => {
                assert_eq!(count, 3);
                assert_eq!(body.len(), 1);
                assert_eq!(body[0], OlangExpr::Command("stats".to_string()));
            }
            other => panic!("expected LoopBlock, got {:?}", other),
        }
    }

    #[test]
    fn parse_loop_zero_is_none() {
        let r = parser().parse("○{loop 0 { stats }}");
        // loop 0 returns None from try_parse_loop, falls through to query
        assert!(matches!(r, ParseResult::OlangExpr(OlangExpr::Query(_))));
    }

    #[test]
    fn parse_loop_multi_stmt() {
        let r = parser().parse("○{loop 5 { stats; dream }}");
        match r {
            ParseResult::OlangExpr(OlangExpr::LoopBlock { count, body }) => {
                assert_eq!(count, 5);
                assert_eq!(body.len(), 2);
            }
            other => panic!("expected LoopBlock, got {:?}", other),
        }
    }

    // ── Function Definition ────────────────────────────────────────────────

    #[test]
    fn parse_fn_def() {
        let r = parser().parse("○{fn test { stats }}");
        match r {
            ParseResult::OlangExpr(OlangExpr::FnDef { name, body }) => {
                assert_eq!(name, "test");
                assert_eq!(body.len(), 1);
                assert_eq!(body[0], OlangExpr::Command("stats".to_string()));
            }
            other => panic!("expected FnDef, got {:?}", other),
        }
    }

    #[test]
    fn parse_fn_multi_stmt() {
        let r = parser().parse("○{fn boot { stats; dream }}");
        match r {
            ParseResult::OlangExpr(OlangExpr::FnDef { name, body }) => {
                assert_eq!(name, "boot");
                assert_eq!(body.len(), 2);
            }
            other => panic!("expected FnDef, got {:?}", other),
        }
    }

    // ── Braces helper ──────────────────────────────────────────────────────

    #[test]
    fn find_matching_brace_simple() {
        assert_eq!(find_matching_brace("{ hello }", 0), Some(8));
    }

    #[test]
    fn find_matching_brace_nested() {
        assert_eq!(find_matching_brace("{ { a } }", 0), Some(8));
    }

    #[test]
    fn find_matching_brace_unclosed() {
        assert_eq!(find_matching_brace("{ hello", 0), None);
    }

    // ── Spawn (Go-style) ──────────────────────────────────────────────────

    #[test]
    fn parse_spawn() {
        let r = parser().parse("○{spawn { stats; dream }}");
        match r {
            ParseResult::OlangExpr(OlangExpr::Spawn { body }) => {
                assert_eq!(body.len(), 2);
                assert_eq!(body[0], OlangExpr::Command("stats".to_string()));
                assert_eq!(body[1], OlangExpr::Command("dream".to_string()));
            }
            other => panic!("expected Spawn, got {:?}", other),
        }
    }

    // ── Pipe (Julia-style) ────────────────────────────────────────────────

    #[test]
    fn parse_pipe() {
        let r = parser().parse("○{fire |> typeof}");
        match r {
            ParseResult::OlangExpr(OlangExpr::Pipe(exprs)) => {
                assert_eq!(exprs.len(), 2);
                assert_eq!(exprs[0], OlangExpr::Query("fire".to_string()));
            }
            other => panic!("expected Pipe, got {:?}", other),
        }
    }

    #[test]
    fn parse_pipe_three_stages() {
        let r = parser().parse("○{fire |> typeof |> stats}");
        match r {
            ParseResult::OlangExpr(OlangExpr::Pipe(exprs)) => {
                assert_eq!(exprs.len(), 3);
            }
            other => panic!("expected Pipe 3 stages, got {:?}", other),
        }
    }

    // ── Use (Python-style) ────────────────────────────────────────────────

    #[test]
    fn parse_use_module() {
        let r = parser().parse("○{use cluster}");
        match r {
            ParseResult::OlangExpr(OlangExpr::Use(module)) => {
                assert_eq!(module, "cluster");
            }
            other => panic!("expected Use, got {:?}", other),
        }
    }

    // ── Emit / Return ─────────────────────────────────────────────────────

    #[test]
    fn parse_emit_expr() {
        let r = parser().parse("○{emit fire}");
        match r {
            ParseResult::OlangExpr(OlangExpr::Emit(inner)) => {
                assert_eq!(*inner, OlangExpr::Query("fire".to_string()));
            }
            other => panic!("expected Emit, got {:?}", other),
        }
    }

    #[test]
    fn parse_return_expr() {
        let r = parser().parse("○{return fire}");
        match r {
            ParseResult::OlangExpr(OlangExpr::Return(inner)) => {
                assert_eq!(*inner, OlangExpr::Query("fire".to_string()));
            }
            other => panic!("expected Return, got {:?}", other),
        }
    }

    #[test]
    fn parse_match_expr() {
        let r = parser().parse("○{match fire { SDF => { stats } _ => { dream } }}");
        match r {
            ParseResult::OlangExpr(OlangExpr::Match { subject, arms }) => {
                assert_eq!(*subject, OlangExpr::Query("fire".to_string()));
                assert_eq!(arms.len(), 2);
                assert_eq!(arms[0].0, "SDF");
                assert_eq!(arms[1].0, "_");
            }
            other => panic!("expected Match, got {:?}", other),
        }
    }
}
