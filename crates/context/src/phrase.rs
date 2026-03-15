//! # phrase — PhraseDict top-down parser
//!
//! Parse text → phrase chains, top-down (phrase trước, word fallback).
//! "phong khach" = 1 phrase, không phải "phong" + "khach".
//!
//! Phrases được đăng ký từ Registry (alias) hoặc hardcoded L0.
//! Mọi phrase → chain_hash trong Registry.

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

use silk::edge::EmotionTag;
use crate::emotion::word_affect;

// ─────────────────────────────────────────────────────────────────────────────
// PhraseMatch
// ─────────────────────────────────────────────────────────────────────────────

/// Một phrase đã được parse.
#[derive(Debug, Clone)]
pub struct PhraseMatch {
    /// Text của phrase
    pub text:       String,
    /// chain_hash nếu có trong Registry
    pub chain_hash: Option<u64>,
    /// EmotionTag của phrase
    pub emotion:    EmotionTag,
    /// Số words trong phrase
    pub word_count: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// PhraseEntry — một entry trong phrase table
// ─────────────────────────────────────────────────────────────────────────────

struct PhraseEntry {
    phrase:     &'static str,
    chain_hash: u64,  // placeholder — real hash từ Registry
    emotion:    EmotionTag,
}

/// Phrase table tĩnh — L0 phrases bất biến.
/// Sorted longest first để match greedy.
static PHRASE_TABLE: &[PhraseEntry] = &[
    // Device commands (dài trước)
    PhraseEntry { phrase: "phong khach",     chain_hash: 0x001, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "phong ngu",       chain_hash: 0x002, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "phong bep",       chain_hash: 0x003, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "tat den",         chain_hash: 0x004, emotion: EmotionTag { valence: 0.0, arousal: 0.2, dominance: 0.6, intensity: 0.3 } },
    PhraseEntry { phrase: "bat den",         chain_hash: 0x005, emotion: EmotionTag { valence: 0.2, arousal: 0.3, dominance: 0.6, intensity: 0.4 } },
    PhraseEntry { phrase: "dieu hoa",        chain_hash: 0x006, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "may lanh",        chain_hash: 0x007, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "cua so",          chain_hash: 0x008, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "cua chinh",       chain_hash: 0x009, emotion: EmotionTag::NEUTRAL },
    // Emotional phrases
    PhraseEntry { phrase: "buon qua",        chain_hash: 0x010, emotion: EmotionTag { valence: -0.75, arousal: 0.5, dominance: 0.2, intensity: 0.7 } },
    PhraseEntry { phrase: "vui qua",         chain_hash: 0x011, emotion: EmotionTag { valence: 0.85, arousal: 0.8, dominance: 0.6, intensity: 0.8 } },
    PhraseEntry { phrase: "met qua",         chain_hash: 0x012, emotion: EmotionTag { valence: -0.5, arousal: 0.2, dominance: 0.3, intensity: 0.6 } },
    PhraseEntry { phrase: "kho qua",         chain_hash: 0x013, emotion: EmotionTag { valence: -0.6, arousal: 0.4, dominance: 0.2, intensity: 0.6 } },
    PhraseEntry { phrase: "mat viec",        chain_hash: 0x014, emotion: EmotionTag { valence: -0.7, arousal: 0.5, dominance: 0.2, intensity: 0.8 } },
    // Time phrases
    PhraseEntry { phrase: "hom nay",         chain_hash: 0x020, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "hom qua",         chain_hash: 0x021, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "ngay mai",        chain_hash: 0x022, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "bay gio",         chain_hash: 0x023, emotion: EmotionTag::NEUTRAL },
    // Question phrases
    PhraseEntry { phrase: "tai sao",         chain_hash: 0x030, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "nhu the nao",     chain_hash: 0x031, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "cai gi",          chain_hash: 0x032, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "o dau",           chain_hash: 0x033, emotion: EmotionTag::NEUTRAL },
];

// ─────────────────────────────────────────────────────────────────────────────
// PhraseDict
// ─────────────────────────────────────────────────────────────────────────────

/// Parser top-down: phrase → word → char.
pub struct PhraseDict;

impl PhraseDict {
    pub fn new() -> Self { Self }

    /// Parse text → Vec<PhraseMatch> top-down.
    ///
    /// Greedy: match phrase dài nhất có thể trước.
    /// Fallback: nếu không match phrase → word đơn.
    pub fn parse(&self, text: &str) -> Vec<PhraseMatch> {
        let normalized = normalize(text);
        let words: Vec<&str> = normalized.split_whitespace().collect();

        let mut result = Vec::new();
        let mut pos    = 0usize;

        while pos < words.len() {
            // Thử match phrase từ 5 words xuống 2 words
            let max_window = 5.min(words.len() - pos);
            let mut matched = false;

            for window in (2..=max_window).rev() {
                let candidate = words[pos..pos+window].join(" ");
                if let Some(entry) = self.lookup_phrase(&candidate) {
                    result.push(PhraseMatch {
                        text:       candidate,
                        chain_hash: Some(entry.chain_hash),
                        emotion:    entry.emotion,
                        word_count: window,
                    });
                    pos += window;
                    matched = true;
                    break;
                }
            }

            // Fallback: word đơn
            if !matched {
                let word = words[pos];
                result.push(PhraseMatch {
                    text:       word.to_string(),
                    chain_hash: None,
                    emotion:    word_affect(word),
                    word_count: 1,
                });
                pos += 1;
            }
        }

        result
    }

    /// Lookup phrase trong PHRASE_TABLE.
    fn lookup_phrase(&self, candidate: &str) -> Option<&'static PhraseEntry> {
        PHRASE_TABLE.iter().find(|e| e.phrase == candidate)
    }

    /// Tổng hợp EmotionTag từ Vec<PhraseMatch>.
    ///
    /// Phrase dài hơn có weight cao hơn (nhiều context hơn).
    pub fn aggregate_emotion(matches: &[PhraseMatch]) -> EmotionTag {
        if matches.is_empty() { return EmotionTag::NEUTRAL; }

        let mut total_v = 0.0f32;
        let mut total_a = 0.0f32;
        let mut total_d = 0.0f32;
        let mut total_i = 0.0f32;
        let mut total_w = 0.0f32;

        for m in matches {
            let w = m.word_count as f32; // weight = số words
            total_v += m.emotion.valence   * w;
            total_a += m.emotion.arousal   * w;
            total_d += m.emotion.dominance * w;
            total_i += m.emotion.intensity * w;
            total_w += w;
        }

        if total_w == 0.0 { return EmotionTag::NEUTRAL; }

        EmotionTag::new(
            (total_v / total_w).max(-1.0).min(1.0),
            (total_a / total_w).max(0.0).min(1.0),
            (total_d / total_w).max(0.0).min(1.0),
            (total_i / total_w).max(0.0).min(1.0),
        )
    }
}

impl Default for PhraseDict {
    fn default() -> Self { Self::new() }
}

/// Normalize text: lowercase, bỏ dấu câu cơ bản.
fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn dict() -> PhraseDict { PhraseDict::new() }

    #[test]
    fn parse_phrase_two_words() {
        let matches = dict().parse("tat den phong khach di");
        // "tat den" = 1 phrase, "phong khach" = 1 phrase
        let texts: Vec<&str> = matches.iter().map(|m| m.text.as_str()).collect();
        assert!(texts.contains(&"tat den"),    "tat den = 1 phrase");
        assert!(texts.contains(&"phong khach"), "phong khach = 1 phrase");
    }

    #[test]
    fn parse_no_split_phrase() {
        // "phong khach" không bị tách thành "phong" + "khach"
        let matches = dict().parse("phong khach");
        assert_eq!(matches.len(), 1, "phong khach = 1 phrase, không tách");
        assert_eq!(matches[0].text, "phong khach");
        assert_eq!(matches[0].word_count, 2);
    }

    #[test]
    fn parse_fallback_to_word() {
        let matches = dict().parse("xin chao");
        // "xin chao" không có trong phrase table → word fallback
        assert_eq!(matches.len(), 2, "fallback to word: 2 words");
    }

    #[test]
    fn parse_emotional_phrase() {
        let matches = dict().parse("toi met qua hom nay");
        let met_qua = matches.iter().find(|m| m.text == "met qua");
        assert!(met_qua.is_some(), "met qua phải được parse");
        if let Some(m) = met_qua {
            assert!(m.emotion.valence < 0.0,
                "met qua → negative valence: {}", m.emotion.valence);
        }
    }

    #[test]
    fn parse_empty() {
        let matches = dict().parse("");
        assert!(matches.is_empty());
    }

    #[test]
    fn aggregate_phrase_weighted() {
        // Phrase dài hơn có weight cao hơn
        let matches = vec![
            PhraseMatch {
                text: "mat viec".to_string(),
                chain_hash: Some(0x014),
                emotion: EmotionTag::new(-0.7, 0.5, 0.2, 0.8),
                word_count: 2,
            },
            PhraseMatch {
                text: "buon".to_string(),
                chain_hash: None,
                emotion: EmotionTag::new(-0.6, 0.5, 0.3, 0.7),
                word_count: 1,
            },
        ];
        let agg = PhraseDict::aggregate_emotion(&matches);
        // Weighted avg: (-0.7×2 + -0.6×1) / 3 = -2.0/3 ≈ -0.667
        assert!(agg.valence < -0.5, "Aggregate negative: {}", agg.valence);
    }

    #[test]
    fn aggregate_empty() {
        let agg = PhraseDict::aggregate_emotion(&[]);
        assert_eq!(agg, EmotionTag::NEUTRAL);
    }

    #[test]
    fn normalize_punctuation() {
        let d = dict();
        // "tat den!" → normalize → "tat den" → match
        let matches = d.parse("tat den!");
        let has_tat_den = matches.iter().any(|m| m.text == "tat den");
        assert!(has_tat_den, "Punctuation stripped → phrase match");
    }
}
