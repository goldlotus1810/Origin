//! # alphabet — Bảng chữ cái của Olang
//!
//! Định nghĩa tập ký tự hợp lệ, phân loại ký tự, Token, và Lexer.
//!
//! ## Bảng chữ cái Olang
//!
//! Olang nhận diện 5 lớp ký tự:
//!
//! | Lớp           | Ký tự                              | Ý nghĩa              |
//! |---------------|------------------------------------|-----------------------|
//! | **Relation**  | ∈ ⊂ ≡ ⊥ ∘ → ≈ ← ∪ ∩ ∂           | Toán tử quan hệ 5D   |
//! | **Delimiter** | { } ( ) ; , = ? |                  | Cấu trúc chương trình|
//! | **Digit**     | 0-9                                | Số nguyên (loop, v.v)|
//! | **Ident**     | Unicode letters, emoji, _, -       | Tên node / biến      |
//! | **Whitespace**| space, tab, newline, CR            | Phân cách token       |
//!
//! ## Keywords (từ khóa dành riêng):
//! `let`, `fn`, `if`, `else`, `loop`, `emit`
//!
//! ## Commands (lệnh hệ thống):
//! `dream`, `stats`, `health`, `seed`, `shutdown`, `reboot`, `status`, `help`

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// CharClass — phân loại ký tự
// ─────────────────────────────────────────────────────────────────────────────

/// Lớp ký tự trong bảng chữ cái Olang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    /// Toán tử quan hệ: ∈ ⊂ ≡ ⊥ ∘ → ≈ ← ∪ ∩ ∂
    Relation,
    /// Dấu phân cách cấu trúc: { } ( ) ; , = ? |
    Delimiter,
    /// Khoảng trắng: space, tab, newline
    Whitespace,
    /// Chữ số: 0-9
    Digit,
    /// Ký tự định danh: Unicode letter, emoji, _, -
    Ident,
}

/// Phân loại một ký tự Unicode theo bảng chữ cái Olang.
pub fn classify(c: char) -> CharClass {
    match c {
        // 11 relation operators
        '∈' | '⊂' | '≡' | '⊥' | '∘' | '→' | '≈' | '←' | '∪' | '∩' | '∂' => {
            CharClass::Relation
        }
        // Delimiters
        '{' | '}' | '(' | ')' | ';' | ',' | '=' | '?' | '|' => CharClass::Delimiter,
        // Whitespace
        ' ' | '\t' | '\n' | '\r' => CharClass::Whitespace,
        // Digits
        '0'..='9' => CharClass::Digit,
        // Everything else = identifier character
        _ => CharClass::Ident,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RelOp — Relation Operator (11 toán tử quan hệ)
// ─────────────────────────────────────────────────────────────────────────────

/// Toán tử quan hệ trong Olang.
///
/// 8 toán tử gốc từ MATH Unicode group (→ RelationBase byte),
/// + 3 mở rộng (Context, Contains, Intersects) xử lý ở tầng semantic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelOp {
    /// ∈ (U+2208) — Member / thuộc về
    Member,
    /// ⊂ (U+2282) — Subset / tập con
    Subset,
    /// ≡ (U+2261) — Equivalent / tương đương
    Equiv,
    /// ⊥ (U+22A5) — Orthogonal / trực giao
    Ortho,
    /// ∘ (U+2218) — Compose / kết hợp → LCA
    Compose,
    /// → (U+2192) — Causes / nhân quả
    Causes,
    /// ≈ (U+2248) — Similar / tương tự
    Similar,
    /// ← (U+2190) — DerivedFrom / bắt nguồn từ
    Derived,
    /// ∪ (U+222A) — Contains / chứa
    Contains,
    /// ∩ (U+2229) — Intersects / giao nhau
    Intersects,
    /// ∂ (U+2202) — Context / ngữ cảnh
    Context,
}

impl RelOp {
    /// Parse ký tự Unicode → RelOp.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '∈' => Some(Self::Member),
            '⊂' => Some(Self::Subset),
            '≡' => Some(Self::Equiv),
            '⊥' => Some(Self::Ortho),
            '∘' => Some(Self::Compose),
            '→' => Some(Self::Causes),
            '≈' => Some(Self::Similar),
            '←' => Some(Self::Derived),
            '∪' => Some(Self::Contains),
            '∩' => Some(Self::Intersects),
            '∂' => Some(Self::Context),
            _ => None,
        }
    }

    /// Ký tự Unicode đại diện.
    pub fn as_char(self) -> char {
        match self {
            Self::Member => '∈',
            Self::Subset => '⊂',
            Self::Equiv => '≡',
            Self::Ortho => '⊥',
            Self::Compose => '∘',
            Self::Causes => '→',
            Self::Similar => '≈',
            Self::Derived => '←',
            Self::Contains => '∪',
            Self::Intersects => '∩',
            Self::Context => '∂',
        }
    }

    /// Map sang relation byte cho IR (8 gốc).
    /// Returns None cho 3 extended ops (Context, Contains, Intersects).
    pub fn to_rel_byte(self) -> Option<u8> {
        match self {
            Self::Member => Some(0x01),
            Self::Subset => Some(0x02),
            Self::Equiv => Some(0x03),
            Self::Ortho => Some(0x04),
            Self::Compose => Some(0x05),
            Self::Causes => Some(0x06),
            Self::Similar => Some(0x07),
            Self::Derived => Some(0x08),
            // Extended ops — handled at semantic level
            Self::Contains | Self::Intersects | Self::Context => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Keyword — từ khóa dành riêng
// ─────────────────────────────────────────────────────────────────────────────

/// Từ khóa dành riêng trong Olang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    /// `let` — gán biến cục bộ
    Let,
    /// `fn` — định nghĩa hàm
    Fn,
    /// `if` — điều kiện
    If,
    /// `else` — nhánh phụ
    Else,
    /// `loop` — lặp N lần
    Loop,
    /// `emit` — xuất chain ra caller
    Emit,
}

/// Check xem string có phải keyword không.
pub fn keyword_from_str(s: &str) -> Option<Keyword> {
    match s {
        "let" => Some(Keyword::Let),
        "fn" => Some(Keyword::Fn),
        "if" => Some(Keyword::If),
        "else" => Some(Keyword::Else),
        "loop" => Some(Keyword::Loop),
        "emit" => Some(Keyword::Emit),
        _ => None,
    }
}

/// Check xem string có phải system command không.
pub fn is_command(s: &str) -> bool {
    matches!(
        s,
        "dream" | "stats" | "health" | "seed" | "shutdown" | "reboot" | "status" | "help"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Token — đơn vị từ vựng
// ─────────────────────────────────────────────────────────────────────────────

/// Token — đơn vị nhỏ nhất trong chương trình Olang.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Identifiers & Literals ──
    /// Tên: node alias, biến, emoji
    Ident(String),
    /// Số nguyên (dùng trong loop, v.v.)
    Int(u32),

    // ── Keywords ──
    /// `let`
    Let,
    /// `fn`
    Fn,
    /// `if`
    If,
    /// `else`
    Else,
    /// `loop`
    Loop,
    /// `emit`
    Emit,

    // ── Commands ──
    /// System command: dream, stats, ...
    Command(String),

    // ── Operators ──
    /// Relation operator
    Rel(RelOp),

    // ── Delimiters ──
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `;`
    Semi,
    /// `,`
    Comma,
    /// `=`
    Eq,
    /// `?`
    Wild,
    /// `|`
    Pipe,

    // ── End ──
    /// End of input
    Eof,
}

// ─────────────────────────────────────────────────────────────────────────────
// Lexer — scanner sinh token từ source text
// ─────────────────────────────────────────────────────────────────────────────

/// Lexer: chuyển source text → dãy Token.
pub struct Lexer<'a> {
    src: &'a str,
    chars: core::iter::Peekable<core::str::CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    /// Tạo lexer mới từ source.
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            chars: src.char_indices().peekable(),
        }
    }

    /// Đọc token tiếp theo.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let (pos, c) = match self.chars.peek().copied() {
            Some(pair) => pair,
            None => return Token::Eof,
        };

        // Relation operators
        if let Some(op) = RelOp::from_char(c) {
            self.chars.next();
            return Token::Rel(op);
        }

        // Delimiters
        match c {
            '{' => {
                self.chars.next();
                return Token::LBrace;
            }
            '}' => {
                self.chars.next();
                return Token::RBrace;
            }
            '(' => {
                self.chars.next();
                return Token::LParen;
            }
            ')' => {
                self.chars.next();
                return Token::RParen;
            }
            ';' => {
                self.chars.next();
                return Token::Semi;
            }
            ',' => {
                self.chars.next();
                return Token::Comma;
            }
            '=' => {
                self.chars.next();
                return Token::Eq;
            }
            '?' => {
                self.chars.next();
                return Token::Wild;
            }
            '|' => {
                self.chars.next();
                return Token::Pipe;
            }
            _ => {}
        }

        // Numbers
        if c.is_ascii_digit() {
            return self.lex_number();
        }

        // Identifiers, keywords, commands
        self.lex_ident(pos)
    }

    /// Tokenize toàn bộ source → Vec<Token> (không bao gồm Eof).
    pub fn tokenize_all(src: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(src);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok == Token::Eof {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    fn skip_whitespace(&mut self) {
        while let Some(&(_, c)) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn lex_number(&mut self) -> Token {
        let mut n: u32 = 0;
        while let Some(&(_, c)) = self.chars.peek() {
            if let Some(d) = c.to_digit(10) {
                n = n.saturating_mul(10).saturating_add(d);
                self.chars.next();
            } else {
                break;
            }
        }
        Token::Int(n)
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        let mut end = start;

        while let Some(&(i, c)) = self.chars.peek() {
            match classify(c) {
                CharClass::Ident | CharClass::Digit => {
                    end = i + c.len_utf8();
                    self.chars.next();
                }
                _ => break,
            }
        }

        let word = &self.src[start..end];

        // Keyword?
        if let Some(kw) = keyword_from_str(word) {
            return match kw {
                Keyword::Let => Token::Let,
                Keyword::Fn => Token::Fn,
                Keyword::If => Token::If,
                Keyword::Else => Token::Else,
                Keyword::Loop => Token::Loop,
                Keyword::Emit => Token::Emit,
            };
        }

        // Command?
        if is_command(word) {
            return Token::Command(word.to_string());
        }

        // Identifier
        Token::Ident(word.to_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    // ── CharClass ───────────────────────────────────────────────────────────

    #[test]
    fn classify_relations() {
        for c in ['∈', '⊂', '≡', '⊥', '∘', '→', '≈', '←', '∪', '∩', '∂'] {
            assert_eq!(classify(c), CharClass::Relation, "'{c}' phải là Relation");
        }
    }

    #[test]
    fn classify_delimiters() {
        for c in ['{', '}', '(', ')', ';', ',', '=', '?', '|'] {
            assert_eq!(
                classify(c),
                CharClass::Delimiter,
                "'{c}' phải là Delimiter"
            );
        }
    }

    #[test]
    fn classify_whitespace() {
        for c in [' ', '\t', '\n', '\r'] {
            assert_eq!(
                classify(c),
                CharClass::Whitespace,
                "'{c:?}' phải là Whitespace"
            );
        }
    }

    #[test]
    fn classify_digits() {
        for c in '0'..='9' {
            assert_eq!(classify(c), CharClass::Digit, "'{c}' phải là Digit");
        }
    }

    #[test]
    fn classify_ident_chars() {
        // Latin, Vietnamese, CJK, emoji — tất cả là Ident
        for c in ['a', 'ử', '火', '🔥', '_', '-'] {
            assert_eq!(classify(c), CharClass::Ident, "'{c}' phải là Ident");
        }
    }

    // ── RelOp ───────────────────────────────────────────────────────────────

    #[test]
    fn relop_roundtrip() {
        let ops = [
            ('∈', RelOp::Member),
            ('⊂', RelOp::Subset),
            ('≡', RelOp::Equiv),
            ('⊥', RelOp::Ortho),
            ('∘', RelOp::Compose),
            ('→', RelOp::Causes),
            ('≈', RelOp::Similar),
            ('←', RelOp::Derived),
            ('∪', RelOp::Contains),
            ('∩', RelOp::Intersects),
            ('∂', RelOp::Context),
        ];
        for (c, expected) in ops {
            let op = RelOp::from_char(c).unwrap();
            assert_eq!(op, expected);
            assert_eq!(op.as_char(), c, "roundtrip failed for {c}");
        }
    }

    #[test]
    fn relop_byte_mapping() {
        assert_eq!(RelOp::Member.to_rel_byte(), Some(0x01));
        assert_eq!(RelOp::Causes.to_rel_byte(), Some(0x06));
        assert_eq!(RelOp::Derived.to_rel_byte(), Some(0x08));
        // Extended ops → None
        assert_eq!(RelOp::Context.to_rel_byte(), None);
        assert_eq!(RelOp::Contains.to_rel_byte(), None);
        assert_eq!(RelOp::Intersects.to_rel_byte(), None);
    }

    // ── Keywords & Commands ─────────────────────────────────────────────────

    #[test]
    fn keyword_detection() {
        assert_eq!(keyword_from_str("let"), Some(Keyword::Let));
        assert_eq!(keyword_from_str("fn"), Some(Keyword::Fn));
        assert_eq!(keyword_from_str("if"), Some(Keyword::If));
        assert_eq!(keyword_from_str("else"), Some(Keyword::Else));
        assert_eq!(keyword_from_str("loop"), Some(Keyword::Loop));
        assert_eq!(keyword_from_str("emit"), Some(Keyword::Emit));
        assert_eq!(keyword_from_str("fire"), None);
        assert_eq!(keyword_from_str("dream"), None); // command, not keyword
    }

    #[test]
    fn command_detection() {
        assert!(is_command("dream"));
        assert!(is_command("stats"));
        assert!(is_command("health"));
        assert!(is_command("seed"));
        assert!(!is_command("fire"));
        assert!(!is_command("let")); // keyword, not command
    }

    // ── Lexer ───────────────────────────────────────────────────────────────

    #[test]
    fn lex_simple_ident() {
        let tokens = Lexer::tokenize_all("fire");
        assert_eq!(tokens, vec![Token::Ident("fire".into())]);
    }

    #[test]
    fn lex_emoji() {
        let tokens = Lexer::tokenize_all("🔥");
        assert_eq!(tokens, vec![Token::Ident("🔥".into())]);
    }

    #[test]
    fn lex_compose() {
        let tokens = Lexer::tokenize_all("fire ∘ water");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fire".into()),
                Token::Rel(RelOp::Compose),
                Token::Ident("water".into()),
            ]
        );
    }

    #[test]
    fn lex_relation_query() {
        let tokens = Lexer::tokenize_all("🔥 ∈ ?");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("🔥".into()),
                Token::Rel(RelOp::Member),
                Token::Wild,
            ]
        );
    }

    #[test]
    fn lex_let_binding() {
        let tokens = Lexer::tokenize_all("let steam = fire ∘ water;");
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident("steam".into()),
                Token::Eq,
                Token::Ident("fire".into()),
                Token::Rel(RelOp::Compose),
                Token::Ident("water".into()),
                Token::Semi,
            ]
        );
    }

    #[test]
    fn lex_fn_def() {
        let tokens = Lexer::tokenize_all("fn blend(a, b) { a ∘ b }");
        assert_eq!(
            tokens,
            vec![
                Token::Fn,
                Token::Ident("blend".into()),
                Token::LParen,
                Token::Ident("a".into()),
                Token::Comma,
                Token::Ident("b".into()),
                Token::RParen,
                Token::LBrace,
                Token::Ident("a".into()),
                Token::Rel(RelOp::Compose),
                Token::Ident("b".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn lex_if_else() {
        let tokens = Lexer::tokenize_all("if fire { emit fire; } else { emit water; }");
        assert_eq!(tokens[0], Token::If);
        assert_eq!(tokens[1], Token::Ident("fire".into()));
        assert_eq!(tokens[2], Token::LBrace);
        assert!(tokens.contains(&Token::Else));
    }

    #[test]
    fn lex_loop() {
        let tokens = Lexer::tokenize_all("loop 3 { emit fire; }");
        assert_eq!(
            tokens,
            vec![
                Token::Loop,
                Token::Int(3),
                Token::LBrace,
                Token::Emit,
                Token::Ident("fire".into()),
                Token::Semi,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn lex_command() {
        let tokens = Lexer::tokenize_all("dream");
        assert_eq!(tokens, vec![Token::Command("dream".into())]);
    }

    #[test]
    fn lex_vietnamese() {
        let tokens = Lexer::tokenize_all("lửa ∘ nước");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("lửa".into()),
                Token::Rel(RelOp::Compose),
                Token::Ident("nước".into()),
            ]
        );
    }

    #[test]
    fn lex_number() {
        let tokens = Lexer::tokenize_all("42");
        assert_eq!(tokens, vec![Token::Int(42)]);
    }

    #[test]
    fn lex_empty() {
        let tokens = Lexer::tokenize_all("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn lex_whitespace_only() {
        let tokens = Lexer::tokenize_all("   \t\n  ");
        assert!(tokens.is_empty());
    }

    #[test]
    fn lex_pipe() {
        let tokens = Lexer::tokenize_all("fire | water");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fire".into()),
                Token::Pipe,
                Token::Ident("water".into()),
            ]
        );
    }

    #[test]
    fn lex_context_query() {
        let tokens = Lexer::tokenize_all("bank ∂ finance");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("bank".into()),
                Token::Rel(RelOp::Context),
                Token::Ident("finance".into()),
            ]
        );
    }
}
