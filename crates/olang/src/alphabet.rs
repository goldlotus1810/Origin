//! # alphabet — Bảng chữ cái của Olang
//!
//! Định nghĩa tập ký tự hợp lệ, phân loại ký tự, Token, và Lexer.
//!
//! ## Bảng chữ cái Olang
//!
//! ```text
//! Lớp           Ký tự                                    Ý nghĩa
//! ─────────────────────────────────────────────────────────────────
//! Relation      ∈ ⊂ ≡ ⊥ ∘ → ≈ ← ∪ ∩ ∂ ∖ ↔ ⟳ ⚡ ∥     16 toán tử quan hệ
//! Symbol        ≔ ⇒ ↻ ○                                  Gán, suy ra, lặp, xuất
//! Arithmetic    + - × ÷                                   Giả thuyết (QT3: chưa chứng minh)
//! Physical      ⧺ ⊖                                      Vật lý (QT3: đã chứng minh)
//! Delimiter     { } ( ) ; , = ? " |                       Cấu trúc
//! Digit         0-9                                       Số nguyên
//! Ident         Unicode letters, emoji, _                 Tên node / biến
//! Whitespace    space, tab, newline, CR                   Phân cách token
//!
//! ## QT3: Ba tầng nhận thức
//!
//! ```text
//! +/-   = giả thuyết (chưa chứng minh)
//! ⧺/⊖   = vật lý (đã chứng minh — thêm/bớt thật)
//! ==    = sự thật chắc chắn
//! ```
//!
//! ## Hai dạng cú pháp (tương đương)
//!
//! ```text
//! Keyword style          Symbol style         Ý nghĩa
//! ────────────────────────────────────────────────────
//! let x = expr;          x ≔ expr;            Gán
//! emit expr;             ○ expr;              Xuất
//! if cond { }            cond ⇒ { }           Điều kiện
//! if c { } else { }      c ⇒ { } ⊥ { }       Điều kiện + ngược lại
//! loop N { }             ↻ N { }              Lặp
//! fn f(a,b) { }         f ≔ (a,b) { }        Hàm
//! ```

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// CharClass — phân loại ký tự
// ─────────────────────────────────────────────────────────────────────────────

/// Lớp ký tự trong bảng chữ cái Olang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    /// Toán tử quan hệ: ∈ ⊂ ≡ ⊥ ∘ → ≈ ← ∪ ∩ ∂ ∖ ↔ ⟳ ⚡ ∥
    Relation,
    /// Ký hiệu cấu trúc: ≔ ⇒ ↻ ○
    Symbol,
    /// Số học giả thuyết (QT3): + - × ÷
    Arithmetic,
    /// Vật lý đã chứng minh (QT3): ⧺ ⊖
    Physical,
    /// Dấu phân cách: { } ( ) ; , = ? " |
    Delimiter,
    /// Khoảng trắng
    Whitespace,
    /// Chữ số: 0-9
    Digit,
    /// Ký tự định danh: Unicode letter, emoji, _
    Ident,
}

/// Phân loại một ký tự Unicode theo bảng chữ cái Olang.
pub fn classify(c: char) -> CharClass {
    match c {
        // 16 relation operators
        '∈' | '⊂' | '≡' | '⊥' | '∘' | '→' | '≈' | '←' | '∪' | '∩' | '∂' | '∖' | '↔'
        | '⟳' | '⚡' | '∥' => CharClass::Relation,
        // Symbol operators (control flow)
        '≔' | '⇒' | '↻' | '○' => CharClass::Symbol,
        // Arithmetic (QT3: hypothesis — chưa chứng minh)
        '+' | '-' | '×' | '÷' => CharClass::Arithmetic,
        // Physical (QT3: proven — đã chứng minh)
        '⧺' | '⊖' => CharClass::Physical,
        // Delimiters
        '{' | '}' | '(' | ')' | ';' | ',' | '=' | '?' | '"' | '|' => CharClass::Delimiter,
        // Whitespace
        ' ' | '\t' | '\n' | '\r' => CharClass::Whitespace,
        // Digits
        '0'..='9' => CharClass::Digit,
        // Everything else = identifier character
        _ => CharClass::Ident,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RelOp — 16 toán tử quan hệ
// ─────────────────────────────────────────────────────────────────────────────

/// Toán tử quan hệ trong Olang.
///
/// 8 gốc (→ RelationBase byte) + 8 mở rộng (semantic level).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelOp {
    // ── 8 gốc (map trực tiếp sang Molecule.relation byte) ──
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

    // ── 8 mở rộng (xử lý ở semantic level) ──
    /// ∪ (U+222A) — Contains / hợp
    Contains,
    /// ∩ (U+2229) — Intersects / giao
    Intersects,
    /// ∂ (U+2202) — Context / ngữ cảnh
    Context,
    /// ∖ (U+2216) — SetMinus / loại trừ
    SetMinus,
    /// ↔ (U+2194) — Bidirectional / hai chiều
    Bidir,
    /// ⟳ (U+27F3) — Feedback / vòng lặp nhân quả
    Feedback,
    /// ⚡ (U+26A1) — Trigger / kích hoạt tức thì
    Trigger,
    /// ∥ (U+2225) — Parallel / song song, độc lập
    Parallel,
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
            '∖' => Some(Self::SetMinus),
            '↔' => Some(Self::Bidir),
            '⟳' => Some(Self::Feedback),
            '⚡' => Some(Self::Trigger),
            '∥' => Some(Self::Parallel),
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
            Self::SetMinus => '∖',
            Self::Bidir => '↔',
            Self::Feedback => '⟳',
            Self::Trigger => '⚡',
            Self::Parallel => '∥',
        }
    }

    /// Map sang relation byte cho IR (8 gốc).
    /// Returns None cho 8 extended ops.
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
            // Extended — handled at semantic level
            Self::Contains
            | Self::Intersects
            | Self::Context
            | Self::SetMinus
            | Self::Bidir
            | Self::Feedback
            | Self::Trigger
            | Self::Parallel => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ArithOp — Số học (chỉ cho numbers, KHÔNG cho nodes)
// ─────────────────────────────────────────────────────────────────────────────

/// Toán tử số học GIẢ THUYẾT (QT3: +/- = chưa chứng minh).
/// Nodes dùng ∘ (compose) hoặc ZWJ, KHÔNG dùng +.
/// Quá trình: quan sát → +/- → chứng minh → ==
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    /// `+` — cộng (giả thuyết)
    Add,
    /// `-` — trừ (giả thuyết)
    Sub,
    /// `×` (U+00D7) — nhân (giả thuyết)
    Mul,
    /// `÷` (U+00F7) — chia (giả thuyết)
    Div,
}

impl ArithOp {
    /// Parse ký tự → ArithOp.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Add),
            '-' => Some(Self::Sub),
            '×' => Some(Self::Mul),
            '÷' => Some(Self::Div),
            _ => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PhysOp — Vật lý đã chứng minh (QT3: ⧺/⊖ = thêm/bớt thật)
// ─────────────────────────────────────────────────────────────────────────────

/// Toán tử vật lý ĐÃ CHỨNG MINH (QT3: ⧺/⊖ = sự thật).
///
/// Khác với +/- (giả thuyết): ⧺/⊖ chỉ dùng khi có bằng chứng.
/// ⧺ = thêm vật lý (đã xác nhận)
/// ⊖ = bớt vật lý (đã xác nhận)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysOp {
    /// ⧺ (U+29FA) — thêm vật lý (đã chứng minh)
    PhysAdd,
    /// ⊖ (U+2296) — bớt vật lý (đã chứng minh)
    PhysSub,
}

impl PhysOp {
    /// Parse ký tự → PhysOp.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '⧺' => Some(Self::PhysAdd),
            '⊖' => Some(Self::PhysSub),
            _ => None,
        }
    }

    /// Ký tự Unicode đại diện.
    pub fn as_char(self) -> char {
        match self {
            Self::PhysAdd => '⧺',
            Self::PhysSub => '⊖',
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Keyword — từ khóa (alias tiếng Anh cho symbols)
// ─────────────────────────────────────────────────────────────────────────────

/// Từ khóa dành riêng — alias cho ký hiệu toán học.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    /// `let` = ≔
    Let,
    /// `fn` = ≔ (...) { }
    Fn,
    /// `if` = ⇒
    If,
    /// `else` = ⊥
    Else,
    /// `loop` = ↻
    Loop,
    /// `emit` = ○
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
        "dream"
            | "stats"
            | "health"
            | "seed"
            | "shutdown"
            | "reboot"
            | "status"
            | "help"
            | "learn"
            | "fuse"
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
    /// Số nguyên
    Int(u32),
    /// Chuỗi ký tự trong ngoặc kép: "text"
    Str(String),

    // ── Keywords (alias tiếng Anh cho symbols) ──
    /// `let` (= ≔)
    Let,
    /// `fn` (= ≔ () {})
    Fn,
    /// `if` (= ⇒)
    If,
    /// `else` (= ⊥ {})
    Else,
    /// `loop` (= ↻)
    Loop,
    /// `emit` (= ○)
    Emit,

    // ── Commands ──
    /// System command: dream, stats, learn, ...
    Command(String),

    // ── Operators ──
    /// Relation operator (16 loại)
    Rel(RelOp),
    /// Arithmetic operator — giả thuyết (QT3: +/-)
    Arith(ArithOp),
    /// Physical operator — đã chứng minh (QT3: ⧺/⊖)
    Phys(PhysOp),
    /// == sự thật chắc chắn (QT3)
    Truth,

    // ── Symbol operators (Unicode math) ──
    /// ≔ (U+2254) — define / gán
    Define,
    /// ⇒ (U+21D2) — implies / nếu...thì
    Implies,
    /// ↻ (U+21BB) — cycle / lặp
    Cycle,
    /// ○ (U+25CB) — origin / xuất
    Circle,

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

        // Relation operators (16)
        if let Some(op) = RelOp::from_char(c) {
            self.chars.next();
            return Token::Rel(op);
        }

        // Symbol operators
        match c {
            '≔' => {
                self.chars.next();
                return Token::Define;
            }
            '⇒' => {
                self.chars.next();
                return Token::Implies;
            }
            '↻' => {
                self.chars.next();
                return Token::Cycle;
            }
            '○' => {
                self.chars.next();
                return Token::Circle;
            }
            _ => {}
        }

        // Physical operators (QT3: ⧺/⊖ — proven)
        if let Some(op) = PhysOp::from_char(c) {
            self.chars.next();
            return Token::Phys(op);
        }

        // Arithmetic operators (QT3: +/- — hypothesis)
        if let Some(op) = ArithOp::from_char(c) {
            self.chars.next();
            return Token::Arith(op);
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
                // == → Truth (QT3: sự thật chắc chắn)
                if let Some(&(_, '=')) = self.chars.peek() {
                    self.chars.next();
                    return Token::Truth;
                }
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
            '"' => {
                return self.lex_string();
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

    fn lex_string(&mut self) -> Token {
        self.chars.next(); // consume opening "
        let mut s = String::new();
        while let Some(&(_, c)) = self.chars.peek() {
            self.chars.next();
            if c == '"' {
                break;
            }
            s.push(c);
        }
        Token::Str(s)
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
        let rels = [
            '∈', '⊂', '≡', '⊥', '∘', '→', '≈', '←', '∪', '∩', '∂', '∖', '↔', '⟳', '⚡',
            '∥',
        ];
        for c in rels {
            assert_eq!(classify(c), CharClass::Relation, "'{c}' phải là Relation");
        }
    }

    #[test]
    fn classify_symbols() {
        for c in ['≔', '⇒', '↻', '○'] {
            assert_eq!(classify(c), CharClass::Symbol, "'{c}' phải là Symbol");
        }
    }

    #[test]
    fn classify_arithmetic() {
        for c in ['+', '-', '×', '÷'] {
            assert_eq!(
                classify(c),
                CharClass::Arithmetic,
                "'{c}' phải là Arithmetic (hypothesis)"
            );
        }
    }

    #[test]
    fn classify_physical() {
        for c in ['⧺', '⊖'] {
            assert_eq!(
                classify(c),
                CharClass::Physical,
                "'{c}' phải là Physical (proven)"
            );
        }
    }

    #[test]
    fn classify_delimiters() {
        for c in ['{', '}', '(', ')', ';', ',', '=', '?', '"', '|'] {
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
        for c in ['a', 'ử', '火', '🔥', '_'] {
            assert_eq!(classify(c), CharClass::Ident, "'{c}' phải là Ident");
        }
    }

    // ── RelOp (16 operators) ────────────────────────────────────────────────

    #[test]
    fn relop_roundtrip_all_16() {
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
            ('∖', RelOp::SetMinus),
            ('↔', RelOp::Bidir),
            ('⟳', RelOp::Feedback),
            ('⚡', RelOp::Trigger),
            ('∥', RelOp::Parallel),
        ];
        for (c, expected) in ops {
            let op = RelOp::from_char(c).unwrap();
            assert_eq!(op, expected);
            assert_eq!(op.as_char(), c, "roundtrip failed for {c}");
        }
    }

    #[test]
    fn relop_byte_mapping() {
        // 8 core → Some(byte)
        assert_eq!(RelOp::Member.to_rel_byte(), Some(0x01));
        assert_eq!(RelOp::Causes.to_rel_byte(), Some(0x06));
        assert_eq!(RelOp::Derived.to_rel_byte(), Some(0x08));
        // 8 extended → None
        assert_eq!(RelOp::Context.to_rel_byte(), None);
        assert_eq!(RelOp::SetMinus.to_rel_byte(), None);
        assert_eq!(RelOp::Bidir.to_rel_byte(), None);
        assert_eq!(RelOp::Feedback.to_rel_byte(), None);
        assert_eq!(RelOp::Trigger.to_rel_byte(), None);
        assert_eq!(RelOp::Parallel.to_rel_byte(), None);
    }

    // ── ArithOp ─────────────────────────────────────────────────────────────

    #[test]
    fn arith_from_char() {
        assert_eq!(ArithOp::from_char('+'), Some(ArithOp::Add));
        assert_eq!(ArithOp::from_char('-'), Some(ArithOp::Sub));
        assert_eq!(ArithOp::from_char('×'), Some(ArithOp::Mul));
        assert_eq!(ArithOp::from_char('÷'), Some(ArithOp::Div));
        assert_eq!(ArithOp::from_char('∘'), None); // not arithmetic
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
    }

    #[test]
    fn command_detection() {
        assert!(is_command("dream"));
        assert!(is_command("stats"));
        assert!(is_command("learn"));
        assert!(!is_command("fire"));
        assert!(!is_command("let"));
    }

    // ── Lexer: existing patterns ────────────────────────────────────────────

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
    fn lex_empty() {
        assert!(Lexer::tokenize_all("").is_empty());
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

    // ── Lexer: NEW symbol forms ─────────────────────────────────────────────

    #[test]
    fn lex_define_symbol() {
        let tokens = Lexer::tokenize_all("steam ≔ fire ∘ water;");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("steam".into()),
                Token::Define,
                Token::Ident("fire".into()),
                Token::Rel(RelOp::Compose),
                Token::Ident("water".into()),
                Token::Semi,
            ]
        );
    }

    #[test]
    fn lex_implies_symbol() {
        let tokens = Lexer::tokenize_all("fire ⇒ { }");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fire".into()),
                Token::Implies,
                Token::LBrace,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn lex_cycle_symbol() {
        let tokens = Lexer::tokenize_all("↻ 3 { }");
        assert_eq!(
            tokens,
            vec![Token::Cycle, Token::Int(3), Token::LBrace, Token::RBrace,]
        );
    }

    #[test]
    fn lex_circle_emit() {
        let tokens = Lexer::tokenize_all("○ fire;");
        assert_eq!(
            tokens,
            vec![
                Token::Circle,
                Token::Ident("fire".into()),
                Token::Semi,
            ]
        );
    }

    // ── Lexer: NEW operators ────────────────────────────────────────────────

    #[test]
    fn lex_new_relops() {
        let tokens = Lexer::tokenize_all("fire ∖ water");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fire".into()),
                Token::Rel(RelOp::SetMinus),
                Token::Ident("water".into()),
            ]
        );
    }

    #[test]
    fn lex_bidir() {
        let tokens = Lexer::tokenize_all("fire ↔ water");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fire".into()),
                Token::Rel(RelOp::Bidir),
                Token::Ident("water".into()),
            ]
        );
    }

    #[test]
    fn lex_trigger() {
        let tokens = Lexer::tokenize_all("🔥 ⚡ 💧");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("🔥".into()),
                Token::Rel(RelOp::Trigger),
                Token::Ident("💧".into()),
            ]
        );
    }

    #[test]
    fn lex_parallel() {
        let tokens = Lexer::tokenize_all("fire ∥ water");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fire".into()),
                Token::Rel(RelOp::Parallel),
                Token::Ident("water".into()),
            ]
        );
    }

    #[test]
    fn lex_arithmetic() {
        let tokens = Lexer::tokenize_all("1 + 2 × 3");
        assert_eq!(
            tokens,
            vec![
                Token::Int(1),
                Token::Arith(ArithOp::Add),
                Token::Int(2),
                Token::Arith(ArithOp::Mul),
                Token::Int(3),
            ]
        );
    }

    #[test]
    fn lex_string_literal() {
        let tokens = Lexer::tokenize_all("learn \"tôi buồn\"");
        assert_eq!(
            tokens,
            vec![
                Token::Command("learn".into()),
                Token::Str("tôi buồn".into()),
            ]
        );
    }

    #[test]
    fn lex_chain_query() {
        // ○{🌞 → ? → 🌵}
        let tokens = Lexer::tokenize_all("🌞 → ? → 🌵");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("🌞".into()),
                Token::Rel(RelOp::Causes),
                Token::Wild,
                Token::Rel(RelOp::Causes),
                Token::Ident("🌵".into()),
            ]
        );
    }

    #[test]
    fn lex_learn_command() {
        assert!(is_command("learn"));
        let tokens = Lexer::tokenize_all("learn");
        assert_eq!(tokens, vec![Token::Command("learn".into())]);
    }

    // ── QT3: hypothesis vs physical vs truth ────────────────────────────────

    #[test]
    fn lex_physical_add() {
        let tokens = Lexer::tokenize_all("fire ⧺ water");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fire".into()),
                Token::Phys(PhysOp::PhysAdd),
                Token::Ident("water".into()),
            ]
        );
    }

    #[test]
    fn lex_physical_sub() {
        let tokens = Lexer::tokenize_all("fire ⊖ water");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fire".into()),
                Token::Phys(PhysOp::PhysSub),
                Token::Ident("water".into()),
            ]
        );
    }

    #[test]
    fn lex_truth_double_eq() {
        let tokens = Lexer::tokenize_all("fire == water");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fire".into()),
                Token::Truth,
                Token::Ident("water".into()),
            ]
        );
    }

    #[test]
    fn lex_single_eq_still_works() {
        let tokens = Lexer::tokenize_all("let x = fire;");
        assert!(tokens.contains(&Token::Eq));
        assert!(!tokens.contains(&Token::Truth));
    }

    #[test]
    fn physop_roundtrip() {
        assert_eq!(PhysOp::from_char('⧺'), Some(PhysOp::PhysAdd));
        assert_eq!(PhysOp::from_char('⊖'), Some(PhysOp::PhysSub));
        assert_eq!(PhysOp::PhysAdd.as_char(), '⧺');
        assert_eq!(PhysOp::PhysSub.as_char(), '⊖');
        assert_eq!(PhysOp::from_char('x'), None);
    }
}
