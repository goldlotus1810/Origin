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
use alloc::vec::Vec;
use olang::separator::{parse_to_chains, parse_tokens, SepToken};
use alloc::string::{String, ToString};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationOp {
    Member,     // ∈ U+2208
    Subset,     // ⊂ U+2282
    Equiv,      // ≡ U+2261
    Compose,    // ∘ U+2218
    Causes,     // → U+2192
    Similar,    // ≈ U+2248
    DerivedFrom,// ← U+2190
    Context,    // ∂ U+2202 (dùng cho "bank ∂ finance")
    Contains,   // ∪ U+222A
    Intersects, // ∩ U+2229
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
            _   => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Member      => "∈",
            Self::Subset      => "⊂",
            Self::Equiv       => "≡",
            Self::Compose     => "∘",
            Self::Causes      => "→",
            Self::Similar     => "≈",
            Self::DerivedFrom => "←",
            Self::Context     => "∂",
            Self::Contains    => "∪",
            Self::Intersects  => "∩",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OlangExpr — parsed expression
// ─────────────────────────────────────────────────────────────────────────────

/// Một expression đã parse từ ○{...}.
#[derive(Debug, Clone, PartialEq)]
pub enum OlangExpr {
    /// ○{🔥} — query node
    Query(String),
    /// ○{🔥 ∈ ?} — relation query
    RelationQuery {
        subject:  String,
        relation: RelationOp,
        /// None = wildcard ?
        object:   Option<String>,
    },
    /// ○{🔥 ∘ 💧} — compose
    Compose { a: String, b: String },
    /// ○{bank ∂ finance} — context query
    ContextQuery { term: String, context: String },
    /// ○{dream} / ○{stats} / ○{seed L0}
    Command(String),
    /// ○{○{🔥} ∈ ?} — pipeline
    Pipeline(Vec<OlangExpr>),
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
    pub fn new() -> Self { Self }

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
                Err(e)   => ParseResult::Error(e),
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
            return Err("Empty expression".to_string());
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

        // + operator without relation ops → route to separator parser
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
            [OlangToken::Node(name)] => {
                Ok(OlangExpr::Query(name.clone()))
            }

            // ○{node ∘ node} — compose
            [OlangToken::Node(a), OlangToken::Relation(RelationOp::Compose), OlangToken::Node(b)] => {
                Ok(OlangExpr::Compose { a: a.clone(), b: b.clone() })
            }

            // ○{node ∂ context} — context query
            [OlangToken::Node(term), OlangToken::Relation(RelationOp::Context), OlangToken::Node(ctx)] => {
                Ok(OlangExpr::ContextQuery { term: term.clone(), context: ctx.clone() })
            }

            // ○{node rel ?} — relation query with wildcard
            [OlangToken::Node(subj), OlangToken::Relation(rel), OlangToken::Wildcard] => {
                Ok(OlangExpr::RelationQuery {
                    subject:  subj.clone(),
                    relation: *rel,
                    object:   None,
                })
            }

            // ○{? rel node} — reverse query
            [OlangToken::Wildcard, OlangToken::Relation(rel), OlangToken::Node(obj)] => {
                Ok(OlangExpr::RelationQuery {
                    subject:  "?".to_string(),
                    relation: *rel,
                    object:   Some(obj.clone()),
                })
            }

            // ○{node rel node} — binary relation
            [OlangToken::Node(subj), OlangToken::Relation(rel), OlangToken::Node(obj)] => {
                Ok(OlangExpr::RelationQuery {
                    subject:  subj.clone(),
                    relation: *rel,
                    object:   Some(obj.clone()),
                })
            }

            // Unknown → try as query
            _ => Ok(OlangExpr::Query(trimmed.to_string())),
        }
    }
}

impl Default for OlangParser {
    fn default() -> Self { Self::new() }
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
    if first != '○' { return None; }

    let (next_pos, next) = chars.next()?;
    if next != '{' { return None; }

    let start = next_pos + '{'.len_utf8();

    // Tìm closing } (handle nested)
    let mut depth = 1usize;
    let mut end   = start;
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
    if depth != 0 { return None; } // Không đóng

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

fn is_command(s: &str) -> bool {
    matches!(s,
        "dream" | "stats" | "seed L0" | "seed" |
        "shutdown" | "reboot" | "status" | "help"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> OlangParser { OlangParser::new() }

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
        assert_eq!(extract_olang("○{🔥}"),    Some("🔥"));
        assert_eq!(extract_olang("hello"),     None);
        assert_eq!(extract_olang("○ hello"),   None);
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
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::Query("🔥".to_string())));
    }

    #[test]
    fn parse_word_query() {
        let r = parser().parse("○{lửa}");
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::Query("lửa".to_string())));
    }

    // ── Relation queries ──────────────────────────────────────────────────────

    #[test]
    fn parse_relation_member() {
        let r = parser().parse("○{🔥 ∈ ?}");
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::RelationQuery {
            subject:  "🔥".to_string(),
            relation: RelationOp::Member,
            object:   None,
        }));
    }

    #[test]
    fn parse_reverse_query() {
        let r = parser().parse("○{? → 💧}");
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::RelationQuery {
            subject:  "?".to_string(),
            relation: RelationOp::Causes,
            object:   Some("💧".to_string()),
        }));
    }

    #[test]
    fn parse_similarity_query() {
        let r = parser().parse("○{🔥 ≈ ?}");
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::RelationQuery {
            subject:  "🔥".to_string(),
            relation: RelationOp::Similar,
            object:   None,
        }));
    }

    // ── Compose ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_compose() {
        let r = parser().parse("○{🔥 ∘ 💧}");
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::Compose {
            a: "🔥".to_string(),
            b: "💧".to_string(),
        }));
    }

    // ── Context query ─────────────────────────────────────────────────────────

    #[test]
    fn parse_context_query() {
        let r = parser().parse("○{bank ∂ finance}");
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::ContextQuery {
            term:    "bank".to_string(),
            context: "finance".to_string(),
        }));
    }

    // ── Commands ──────────────────────────────────────────────────────────────

    #[test]
    fn parse_dream_command() {
        let r = parser().parse("○{dream}");
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::Command("dream".to_string())));
    }

    #[test]
    fn parse_stats_command() {
        let r = parser().parse("○{stats}");
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::Command("stats".to_string())));
    }

    // ── Tiếng Việt ────────────────────────────────────────────────────────────

    #[test]
    fn parse_vietnamese_query() {
        let r = parser().parse("○{lửa ∘ nước}");
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::Compose {
            a: "lửa".to_string(),
            b: "nước".to_string(),
        }));
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
        assert_eq!(r, ParseResult::OlangExpr(OlangExpr::Query("🔥".to_string())));
    }

    #[test]
    fn parse_circle_without_brace_is_natural() {
        // ○ standalone = SDF Torus node trong text bình thường
        let r = parser().parse("○ là hình tròn");
        assert!(matches!(r, ParseResult::Natural(_)));
    }
}
