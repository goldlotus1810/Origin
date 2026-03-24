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
    // Pre-allocate capacity based on source size (~1 token per 5 chars)
    let _tok_cap = __floor(len(source) / 4) + 100;
    if _tok_cap < 1024 { _tok_cap = 1024; };
    let tokens = __array_with_cap(_tok_cap);
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
            // Hex literal: 0x... or 0X...
            let _nl_is_hex = 0;
            if ch == "0" && (pos + 1) < src_len {
                let _nl_next = char_at(source, pos + 1);
                if _nl_next == "x" || _nl_next == "X" {
                    _nl_is_hex = 1;
                    let pos = pos + 2;
                    let col = col + 2;
                    while pos < src_len {
                        let _nl_hc = char_at(source, pos);
                        if is_digit(_nl_hc) || (_nl_hc >= "a" && _nl_hc <= "f") || (_nl_hc >= "A" && _nl_hc <= "F") {
                            let pos = pos + 1;
                            let col = col + 1;
                        } else {
                            break;
                        };
                    };
                };
            };
            if _nl_is_hex == 0 {
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
            };
            let text = substr(source, start, pos);
            let _nl_numval = 0;
            if _nl_is_hex == 1 {
                // Convert hex string to number: skip "0x" prefix
                let _nl_hi = 2;
                while _nl_hi < len(text) {
                    let _nl_hd = char_at(text, _nl_hi);
                    _nl_numval = _nl_numval * 16;
                    if _nl_hd >= "0" && _nl_hd <= "9" { _nl_numval = _nl_numval + (__char_code(_nl_hd) - 48); };
                    if _nl_hd >= "a" && _nl_hd <= "f" { _nl_numval = _nl_numval + (__char_code(_nl_hd) - 87); };
                    if _nl_hd >= "A" && _nl_hd <= "F" { _nl_numval = _nl_numval + (__char_code(_nl_hd) - 55); };
                    let _nl_hi = _nl_hi + 1;
                };
            } else {
                _nl_numval = to_num(text);
            };
            push(tokens, Token {
                kind: TokenKind::Number { value: _nl_numval },
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

        // Interpolated string: $"hello {expr}!"
        // Desugars to: ("hello " + __to_string(expr) + "!")
        // Emits: ( StringLit + Ident + StringLit ... )
        if ch == "$" && pos + 1 < src_len && char_at(source, pos + 1) == "\"" {
            let _is_start_col = col;
            let pos = pos + 2;  // skip $"
            let col = col + 2;
            // Emit opening (
            push(tokens, Token { kind: TokenKind::Symbol { ch: "(" }, text: "(", line: line, col: _is_start_col });
            let _is_first = 1;
            let _is_seg_start = pos;
            while pos < src_len && char_at(source, pos) != "\"" {
                if char_at(source, pos) == "{" {
                    // Emit string segment before {
                    let _is_seg = __substr(source, _is_seg_start, pos);
                    if _is_first == 0 {
                        push(tokens, Token { kind: TokenKind::Symbol { ch: "+" }, text: "+", line: line, col: col });
                    };
                    push(tokens, Token { kind: TokenKind::StringLit { value: _is_seg }, text: _is_seg, line: line, col: col });
                    let _is_first = 0;
                    let pos = pos + 1;  // skip {
                    let col = col + 1;
                    // Emit + __to_string(
                    push(tokens, Token { kind: TokenKind::Symbol { ch: "+" }, text: "+", line: line, col: col });
                    push(tokens, Token { kind: TokenKind::Ident { name: "__to_string" }, text: "__to_string", line: line, col: col });
                    push(tokens, Token { kind: TokenKind::Symbol { ch: "(" }, text: "(", line: line, col: col });
                    // Tokenize expr inside {} (simple: just read ident/number until })
                    let _is_expr_start = pos;
                    while pos < src_len && char_at(source, pos) != "}" {
                        let pos = pos + 1;
                        let col = col + 1;
                    };
                    // Emit the expr as ident
                    let _is_expr = __substr(source, _is_expr_start, pos);
                    if is_digit(char_at(_is_expr, 0)) {
                        push(tokens, Token { kind: TokenKind::Number { value: __to_number(_is_expr) }, text: _is_expr, line: line, col: col });
                    } else {
                        push(tokens, Token { kind: TokenKind::Ident { name: _is_expr }, text: _is_expr, line: line, col: col });
                    };
                    // Close __to_string()
                    push(tokens, Token { kind: TokenKind::Symbol { ch: ")" }, text: ")", line: line, col: col });
                    let pos = pos + 1;  // skip }
                    let col = col + 1;
                    let _is_seg_start = pos;
                } else {
                    let pos = pos + 1;
                    let col = col + 1;
                };
            };
            // Emit final string segment
            let _is_final = __substr(source, _is_seg_start, pos);
            if _is_first == 0 {
                push(tokens, Token { kind: TokenKind::Symbol { ch: "+" }, text: "+", line: line, col: col });
            };
            push(tokens, Token { kind: TokenKind::StringLit { value: _is_final }, text: _is_final, line: line, col: col });
            // Emit closing )
            push(tokens, Token { kind: TokenKind::Symbol { ch: ")" }, text: ")", line: line, col: col });
            let pos = pos + 1;  // skip closing "
            let col = col + 1;
            continue;
        };

        // Multi-char symbols: ==, !=, <=, >=, =>, ->, ::, &&, ||
        if pos + 1 < src_len {
            let two = substr(source, pos, pos + 2);
            if two == "==" || two == "!=" || two == "<=" || two == ">="
                || two == "=>" || two == "->" || two == "::" || two == "&&" || two == "||"
                || two == "<<" || two == ">>" {
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
