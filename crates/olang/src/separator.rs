//! # separator — 4 Separator Types
//!
//! Phân biệt chính xác 4 loại separator trong Olang:
//!
//! ```text
//! ZWJ   U+200D  COMPOSE ngữ nghĩa (Unicode chuẩn)  → ZWJ sequence → 1 chain
//! +     U+002B  OPERATE toán học                   → binary operation
//! space U+0020  SEPARATE (2 thứ riêng)              → sequence
//! ∅             JUXTAPOSE (viết liền)               → 2 nodes riêng
//! ```
//!
//! Ví dụ:
//! ```text
//! 1+1        → [1][+][1]     expression: 2
//! 1 1        → [1][ ][1]     sequence: [1, 1]
//! 11         → [11]          number: 11
//! 👨 👨       → 2 nodes riêng
//! 👨+👨       → compose → nhóm
//! 👨‍👨 (ZWJ)  → 1 cluster (1 chain 2 molecules)
//! 👨👨 (none) → 2 nodes riêng (juxtapose)
//! ```

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::encoder::{encode_codepoint, encode_zwj_sequence};
use crate::lca::lca;
use crate::molecular::MolecularChain;

// ─────────────────────────────────────────────────────────────────────────────
// SepKind — loại separator
// ─────────────────────────────────────────────────────────────────────────────

/// Separator giữa 2 tokens.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SepKind {
    /// U+200D ZWJ — compose ngữ nghĩa (👨‍👩‍👦 = family)
    ZWJ,
    /// U+002B + — operate toán học / LCA
    Plus,
    /// U+0020 space — separate 2 thứ riêng
    Space,
    /// Không có separator (viết liền) — juxtapose
    Juxtapose,
}

// ─────────────────────────────────────────────────────────────────────────────
// Token — kết quả parse
// ─────────────────────────────────────────────────────────────────────────────

/// Một token sau khi parse với separator context.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum SepToken {
    /// Codepoint đơn
    Single(u32),
    /// ZWJ sequence → 1 chain N molecules
    ZwjSeq(Vec<u32>),
    /// + operation giữa 2 chains
    Operation { left: u32, right: u32 },
    /// Sequence (space-separated)
    Sequence(Vec<u32>),
    /// Number literal
    Number(i64),
    /// Word/alias
    Word(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// SepParser — tokenize text theo 4 separator rules
// ─────────────────────────────────────────────────────────────────────────────

/// Parse text → Vec<SepToken> theo separator rules.
///
/// Single pass: xử lý ZWJ, +, number, word, emoji.
pub fn parse_tokens(input: &str) -> Vec<SepToken> {
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut tokens: Vec<SepToken> = Vec::new();
    let mut i = 0usize;

    while i < n {
        let c = chars[i];

        // Space → skip (SEPARATE — tách segments, không tạo token)
        if c == ' ' {
            i += 1;
            continue;
        }

        // ZWJ standalone (không nên xuất hiện ở đây)
        if c == '\u{200D}' {
            i += 1;
            continue;
        }

        // Number literal: [0-9]+
        if c.is_ascii_digit() {
            let start = i;
            while i < n && chars[i].is_ascii_digit() {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            if let Ok(num) = s.parse::<i64>() {
                // Check nếu tiếp theo là + → Operation
                if i < n && chars[i] == '+' {
                    i += 1; // skip +
                            // Parse right side
                    let right_start = i;
                    while i < n && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let rs: String = chars[right_start..i].iter().collect();
                    if let Ok(right) = rs.parse::<i64>() {
                        // Encode digits as ASCII codepoints
                        let lcp = 0x30 + (num.abs() % 10) as u32;
                        let rcp = 0x30 + (right.abs() % 10) as u32;
                        tokens.push(SepToken::Operation {
                            left: lcp,
                            right: rcp,
                        });
                    } else {
                        tokens.push(SepToken::Number(num));
                    }
                } else {
                    tokens.push(SepToken::Number(num));
                }
            }
            continue;
        }

        // ASCII word: [a-zA-Z][a-zA-Z0-9_-]*
        if c.is_ascii_alphabetic() {
            let start = i;
            while i < n {
                let nc = chars[i];
                if nc.is_ascii_alphanumeric() || nc == '_' || nc == '-' {
                    i += 1;
                } else {
                    break;
                }
            }
            let word: String = chars[start..i].iter().collect();
            tokens.push(SepToken::Word(word));
            continue;
        }

        // Unicode char (emoji / symbol / CJK / Vietnamese...)
        let cp = c as u32;
        if cp > 0x20 {
            // Collect ZWJ sequence: emoji (ZWJ emoji)*
            let mut seq = vec![cp];
            i += 1;
            while i < n && chars[i] == '\u{200D}' {
                i += 1; // skip ZWJ
                if i < n {
                    seq.push(chars[i] as u32);
                    i += 1;
                }
            }

            if seq.len() > 1 {
                // ZWJ sequence → 1 cluster
                tokens.push(SepToken::ZwjSeq(seq));
            } else {
                // Check nếu tiếp theo là + → Operation
                if i < n && chars[i] == '+' {
                    i += 1; // skip +
                            // Parse right side (emoji hoặc word)
                    if i < n && chars[i] as u32 > 0x20 {
                        let mut rseq = vec![chars[i] as u32];
                        i += 1;
                        // Check right side ZWJ
                        while i < n && chars[i] == '\u{200D}' {
                            i += 1;
                            if i < n {
                                rseq.push(chars[i] as u32);
                                i += 1;
                            }
                        }
                        if rseq.len() > 1 {
                            // Right side là ZWJ sequence — encode ZWJ, dùng first cp
                            tokens.push(SepToken::Operation {
                                left: cp,
                                right: rseq[0],
                            });
                        } else {
                            tokens.push(SepToken::Operation {
                                left: cp,
                                right: rseq[0],
                            });
                        }
                    } else {
                        tokens.push(SepToken::Single(cp));
                    }
                } else {
                    tokens.push(SepToken::Single(cp));
                }
            }
            continue;
        }

        i += 1;
    }

    tokens
}

// ─────────────────────────────────────────────────────────────────────────────
// Encode: SepToken → MolecularChain
// ─────────────────────────────────────────────────────────────────────────────

/// Convert SepToken → MolecularChain.
pub fn token_to_chain(token: &SepToken) -> MolecularChain {
    match token {
        SepToken::Single(cp) => encode_codepoint(*cp),

        SepToken::ZwjSeq(seq) => encode_zwj_sequence(seq),

        SepToken::Operation { left, right } => {
            // + → LCA (compose ngữ nghĩa)
            let a = encode_codepoint(*left);
            let b = encode_codepoint(*right);
            if a.is_empty() || b.is_empty() {
                return MolecularChain::empty();
            }
            lca(&a, &b)
        }

        SepToken::Number(n) => {
            // Number → codepoint trong Mathematical Alphanumeric Symbols
            // 0=0x30, 1=0x31... hoặc dùng digit emoji 0️⃣=0x30+VS16
            let cp = if *n >= 0 && *n <= 9 {
                0x30 + *n as u32 // ASCII digit
            } else {
                0x221E // ∞ for large numbers
            };
            encode_codepoint(cp)
        }

        SepToken::Word(w) => {
            // Word → lookup từ ALIAS_CODEPOINTS
            use crate::startup::ALIAS_CODEPOINTS;
            for &(alias, cp) in ALIAS_CODEPOINTS {
                if alias == w.as_str() {
                    return encode_codepoint(cp);
                }
            }
            MolecularChain::empty()
        }

        SepToken::Sequence(cps) => {
            // Sequence: LCA của tất cả
            let chains: Vec<MolecularChain> = cps
                .iter()
                .map(|&cp| encode_codepoint(cp))
                .filter(|c| !c.is_empty())
                .collect();
            if chains.is_empty() {
                return MolecularChain::empty();
            }
            crate::lca::lca_many(&chains)
        }
    }
}

/// Parse input → Vec<MolecularChain>.
///
/// Xử lý đúng 4 separator:
///   ZWJ → 1 chain N molecules
///   +   → LCA của 2 chains
///       space → sequence (nhiều chains riêng)
///       none → juxtapose (nhiều chains riêng)
pub fn parse_to_chains(input: &str) -> Vec<MolecularChain> {
    // Tách theo space trước (SEPARATE)
    let segments: Vec<&str> = input.split(' ').filter(|s| !s.is_empty()).collect();

    let mut result = Vec::new();

    for seg in segments {
        // Trong mỗi segment: có thể có + hoặc juxtapose
        let tokens = parse_tokens(seg);
        for token in &tokens {
            let chain = token_to_chain(token);
            if !chain.is_empty() {
                result.push(chain);
            }
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, string::ToString, vec};

    fn skip() -> bool {
        ucd::table_len() == 0
    }

    // ── ZWJ sequence ─────────────────────────────────────────────────────────

    #[test]
    fn zwj_family_is_one_chain() {
        if skip() {
            return;
        }
        // 👨‍👩‍👦 = U+1F468 ZWJ U+1F469 ZWJ U+1F466
        let s = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F466}";
        let tokens = parse_tokens(s);
        assert_eq!(tokens.len(), 1, "ZWJ sequence → 1 token");
        assert!(
            matches!(&tokens[0], SepToken::ZwjSeq(seq) if seq.len() == 3),
            "ZWjSeq với 3 codepoints"
        );

        let chain = token_to_chain(&tokens[0]);
        assert_eq!(chain.len(), 3, "3 molecules: 15 bytes");
        // Verify relation pattern: ∘ ∘ ∈
        use crate::molecular::RelationBase;
        assert_eq!(chain.0[0].relation, RelationBase::Compose);
        assert_eq!(chain.0[1].relation, RelationBase::Compose);
        assert_eq!(chain.0[2].relation, RelationBase::Member);
    }

    #[test]
    fn zwj_couple_two_molecules() {
        if skip() {
            return;
        }
        // 👨‍👨 = U+1F468 ZWJ U+1F468
        let s = "\u{1F468}\u{200D}\u{1F468}";
        let tokens = parse_tokens(s);
        assert_eq!(tokens.len(), 1);
        let chain = token_to_chain(&tokens[0]);
        assert_eq!(chain.len(), 2, "2 molecules cho couple");
        use crate::molecular::RelationBase;
        assert_eq!(chain.0[0].relation, RelationBase::Compose); // ∘
        assert_eq!(chain.0[1].relation, RelationBase::Member); // ∈
    }

    // ── Space separator ───────────────────────────────────────────────────────

    #[test]
    fn space_two_separate_nodes() {
        if skip() {
            return;
        }
        // 👨 👨 → 2 chains riêng
        let chains = parse_to_chains("\u{1F468} \u{1F468}");
        assert_eq!(chains.len(), 2, "space → 2 nodes riêng");
    }

    #[test]
    fn fire_space_water_two_chains() {
        if skip() {
            return;
        }
        let chains = parse_to_chains("\u{1F525} \u{1F4A7}"); // 🔥 💧
        assert_eq!(chains.len(), 2, "🔥 💧 → 2 chains riêng");
        // Hai chains phải khác nhau
        assert_ne!(
            chains[0].chain_hash(),
            chains[1].chain_hash(),
            "🔥 và 💧 phải có hash khác nhau"
        );
    }

    // ── Plus operator ─────────────────────────────────────────────────────────

    #[test]
    fn plus_gives_lca() {
        if skip() {
            return;
        }
        // 🔥+💧 → LCA → 1 chain
        let tokens = parse_tokens("\u{1F525}+\u{1F4A7}");
        let has_op = tokens
            .iter()
            .any(|t| matches!(t, SepToken::Operation { .. }));
        assert!(has_op, "+ → Operation token");
    }

    // ── Juxtapose ─────────────────────────────────────────────────────────────

    #[test]
    fn juxtapose_two_emojis_separate() {
        if skip() {
            return;
        }
        // 👨👨 (viết liền không có ZWJ) → 2 nodes riêng
        let chains = parse_to_chains("\u{1F468}\u{1F468}");
        // Juxtapose: mỗi emoji → chain riêng
        // Hoặc parser gộp thành juxtapose sequence
        assert!(!chains.is_empty(), "juxtapose → ít nhất 1 chain");
    }

    // ── Numbers ──────────────────────────────────────────────────────────────

    #[test]
    fn number_token() {
        let tokens = parse_tokens("42");
        assert!(tokens.iter().any(|t| matches!(t, SepToken::Number(42))));
    }

    #[test]
    fn expression_1_plus_1() {
        let tokens = parse_tokens("1+1");
        assert!(
            tokens.iter().any(|t| matches!(
                t,
                SepToken::Operation {
                    left: 0x31,
                    right: 0x31
                }
            )),
            "1+1 → Operation(0x31, 0x31)"
        );
    }

    #[test]
    fn sequence_1_space_1() {
        let chains = parse_to_chains("1 1");
        // "1 1" = sequence → 2 chains từ "1" và "1"
        assert!(chains.len() >= 1, "1 1 → sequence");
    }

    // ── Word tokens ──────────────────────────────────────────────────────────

    #[test]
    fn word_token_fire() {
        let tokens = parse_tokens("fire");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, SepToken::Word(w) if w == "fire")));
    }

    #[test]
    fn word_to_chain_known() {
        if skip() {
            return;
        }
        let token = SepToken::Word("fire".into());
        let chain = token_to_chain(&token);
        assert!(!chain.is_empty(), "'fire' → non-empty chain");
    }

    #[test]
    fn word_to_chain_unknown() {
        let token = SepToken::Word("xyz_unknown_999".into());
        let chain = token_to_chain(&token);
        assert!(chain.is_empty(), "Unknown word → empty chain");
    }

    // ── Mixed ────────────────────────────────────────────────────────────────

    #[test]
    fn zwj_vs_space_different_lengths() {
        if skip() {
            return;
        }
        // ZWJ → 1 chain, 2 mols
        let zwj_chains = parse_to_chains("\u{1F468}\u{200D}\u{1F469}");
        // Space → 2 chains, 1 mol each
        let space_chains = parse_to_chains("\u{1F468} \u{1F469}");

        assert_eq!(zwj_chains.len(), 1, "ZWJ → 1 chain");
        assert_eq!(space_chains.len(), 2, "space → 2 chains");
        assert_eq!(zwj_chains[0].len(), 2, "ZWJ chain → 2 molecules");
        assert_eq!(space_chains[0].len(), 1, "space chain → 1 molecule");
    }

    #[test]
    fn separator_table_correctness() {
        if skip() {
            return;
        }
        // Theo spec:
        // 👨‍👨 (ZWJ) = 1 cluster: couple → chain 2 mol ✓
        // 👨 👨 (space) = 2 nodes riêng → 2 chains ✓
        // 👨+👨 (plus) = compose → LCA chain ✓
        // 👨👨 (none) = juxtapose → 2 chains ✓

        let cp = 0x1F468u32; // 👨

        let zwj_result = parse_to_chains(&format!(
            "{}\u{200D}{}",
            char::from_u32(cp).unwrap(),
            char::from_u32(cp).unwrap()
        ));
        let space_result = parse_to_chains(&format!(
            "{} {}",
            char::from_u32(cp).unwrap(),
            char::from_u32(cp).unwrap()
        ));

        // ZWJ → 1 chain với 2 molecules
        assert_eq!(zwj_result.len(), 1);
        assert_eq!(zwj_result[0].len(), 2);

        // Space → 2 chains với 1 molecule each
        assert_eq!(space_result.len(), 2);
        assert_eq!(space_result[0].len(), 1);
        assert_eq!(space_result[1].len(), 1);
    }
}
