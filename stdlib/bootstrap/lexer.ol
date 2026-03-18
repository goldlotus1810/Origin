// ── Olang Bootstrap Lexer ─────────────────────────────────────────
// Self-hosting preparation: tokenizer written in Olang.
// Reads source string → produces token list.
//
// Phase 4 / A9 — compiler self-hosting foundation.

union TokenKind {
    Keyword { name: Str },
    Ident { name: Str },
    Number { value: Num },
    StringLit { value: Str },
    Symbol { ch: Str },
    Eof,
}

type Token {
    kind: TokenKind,
    text: Str,
    line: Num,
    col: Num,
}

// ── Keyword table ────────────────────────────────────────────────
let KEYWORDS = [
    "let", "fn", "if", "else", "loop", "while", "for", "in",
    "return", "break", "continue", "emit", "type", "union",
    "impl", "trait", "match", "try", "catch", "spawn", "select",
    "timeout", "from", "use", "mod", "pub", "true", "false",
];

fn is_keyword(name) {
    let i = 0;
    while i < len(KEYWORDS) {
        if KEYWORDS[i] == name {
            return true;
        };
        let i = i + 1;
    };
    return false;
}

fn is_alpha(ch) {
    return (ch >= "a" && ch <= "z")
        || (ch >= "A" && ch <= "Z")
        || ch == "_";
}

fn is_digit(ch) {
    return ch >= "0" && ch <= "9";
}

fn is_whitespace(ch) {
    return ch == " " || ch == "\n" || ch == "\r" || ch == "\t";
}

// ── Main tokenizer ───────────────────────────────────────────────
pub fn tokenize(source) {
    let tokens = [];
    let pos = 0;
    let line = 1;
    let col = 1;
    let src_len = len(source);

    while pos < src_len {
        let ch = char_at(source, pos);

        // Skip whitespace
        if is_whitespace(ch) {
            if ch == "\n" {
                let line = line + 1;
                let col = 1;
            } else {
                let col = col + 1;
            };
            let pos = pos + 1;
            continue;
        };

        // Skip line comments: //
        if ch == "/" && pos + 1 < src_len && char_at(source, pos + 1) == "/" {
            while pos < src_len && char_at(source, pos) != "\n" {
                let pos = pos + 1;
            };
            continue;
        };

        // Identifiers and keywords
        if is_alpha(ch) {
            let start = pos;
            let start_col = col;
            while pos < src_len && (is_alpha(char_at(source, pos)) || is_digit(char_at(source, pos))) {
                let pos = pos + 1;
                let col = col + 1;
            };
            let text = substr(source, start, pos);
            let kind = if is_keyword(text) {
                TokenKind::Keyword { name: text }
            } else {
                TokenKind::Ident { name: text }
            };
            push(tokens, Token { kind: kind, text: text, line: line, col: start_col });
            continue;
        };

        // Number literals
        if is_digit(ch) {
            let start = pos;
            let start_col = col;
            while pos < src_len && is_digit(char_at(source, pos)) {
                let pos = pos + 1;
                let col = col + 1;
            };
            // Decimal point
            if pos < src_len && char_at(source, pos) == "." {
                let pos = pos + 1;
                let col = col + 1;
                while pos < src_len && is_digit(char_at(source, pos)) {
                    let pos = pos + 1;
                    let col = col + 1;
                };
            };
            let text = substr(source, start, pos);
            push(tokens, Token {
                kind: TokenKind::Number { value: to_num(text) },
                text: text,
                line: line,
                col: start_col,
            });
            continue;
        };

        // String literals
        if ch == "\"" {
            let start = pos;
            let start_col = col;
            let pos = pos + 1;
            let col = col + 1;
            while pos < src_len && char_at(source, pos) != "\"" {
                if char_at(source, pos) == "\\" {
                    let pos = pos + 1;
                    let col = col + 1;
                };
                let pos = pos + 1;
                let col = col + 1;
            };
            let pos = pos + 1; // closing quote
            let col = col + 1;
            let text = substr(source, start, pos);
            let value = substr(source, start + 1, pos - 1);
            push(tokens, Token {
                kind: TokenKind::StringLit { value: value },
                text: text,
                line: line,
                col: start_col,
            });
            continue;
        };

        // Multi-char symbols: ==, !=, <=, >=, =>, ->, ::, &&, ||
        if pos + 1 < src_len {
            let two = substr(source, pos, pos + 2);
            if two == "==" || two == "!=" || two == "<=" || two == ">="
                || two == "=>" || two == "->" || two == "::" || two == "&&" || two == "||" {
                push(tokens, Token {
                    kind: TokenKind::Symbol { ch: two },
                    text: two,
                    line: line,
                    col: col,
                });
                let pos = pos + 2;
                let col = col + 2;
                continue;
            };
        };

        // Single-char symbols
        push(tokens, Token {
            kind: TokenKind::Symbol { ch: ch },
            text: ch,
            line: line,
            col: col,
        });
        let pos = pos + 1;
        let col = col + 1;
    };

    // EOF token
    push(tokens, Token {
        kind: TokenKind::Eof,
        text: "",
        line: line,
        col: col,
    });

    return tokens;
}
