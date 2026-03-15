//! # phrase — PhraseDict top-down parser
//!
//! Parse text → phrase chains, top-down (phrase trước, word fallback).
//! "phòng khách" = 1 phrase, không tách thành "phòng" + "khách".
//!
//! Phrases đầy đủ tiếng Việt Unicode.

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
    pub text:       String,
    pub chain_hash: Option<u64>,
    pub emotion:    EmotionTag,
    pub word_count: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// PhraseEntry
// ─────────────────────────────────────────────────────────────────────────────

struct PhraseEntry {
    phrase:     &'static str,
    chain_hash: u64,
    emotion:    EmotionTag,
}

/// Phrase table — tiếng Việt đầy đủ Unicode.
/// Sorted longest first (greedy match).
static PHRASE_TABLE: &[PhraseEntry] = &[
    // Device commands
    PhraseEntry { phrase: "phòng khách",    chain_hash: 0x001, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "phòng ngủ",      chain_hash: 0x002, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "phòng bếp",      chain_hash: 0x003, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "tắt đèn",        chain_hash: 0x004, emotion: EmotionTag { valence:  0.0, arousal: 0.2, dominance: 0.6, intensity: 0.3 } },
    PhraseEntry { phrase: "bật đèn",        chain_hash: 0x005, emotion: EmotionTag { valence:  0.2, arousal: 0.3, dominance: 0.6, intensity: 0.4 } },
    PhraseEntry { phrase: "điều hòa",       chain_hash: 0x006, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "máy lạnh",       chain_hash: 0x007, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "cửa sổ",         chain_hash: 0x008, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "cửa chính",      chain_hash: 0x009, emotion: EmotionTag::NEUTRAL },
    // Emotional phrases
    PhraseEntry { phrase: "buồn quá",       chain_hash: 0x010, emotion: EmotionTag { valence: -0.75, arousal: 0.5, dominance: 0.2, intensity: 0.7 } },
    PhraseEntry { phrase: "vui quá",        chain_hash: 0x011, emotion: EmotionTag { valence:  0.85, arousal: 0.8, dominance: 0.6, intensity: 0.8 } },
    PhraseEntry { phrase: "mệt quá",        chain_hash: 0x012, emotion: EmotionTag { valence: -0.50, arousal: 0.2, dominance: 0.3, intensity: 0.6 } },
    PhraseEntry { phrase: "khó quá",        chain_hash: 0x013, emotion: EmotionTag { valence: -0.60, arousal: 0.4, dominance: 0.2, intensity: 0.6 } },
    PhraseEntry { phrase: "mất việc",       chain_hash: 0x014, emotion: EmotionTag { valence: -0.70, arousal: 0.5, dominance: 0.2, intensity: 0.8 } },
    PhraseEntry { phrase: "cô đơn",         chain_hash: 0x015, emotion: EmotionTag { valence: -0.65, arousal: 0.2, dominance: 0.2, intensity: 0.6 } },
    PhraseEntry { phrase: "hạnh phúc",      chain_hash: 0x016, emotion: EmotionTag { valence:  0.90, arousal: 0.7, dominance: 0.7, intensity: 0.8 } },
    PhraseEntry { phrase: "lo lắng",        chain_hash: 0x017, emotion: EmotionTag { valence: -0.55, arousal: 0.7, dominance: 0.3, intensity: 0.6 } },
    // Time phrases
    PhraseEntry { phrase: "hôm nay",        chain_hash: 0x020, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "hôm qua",        chain_hash: 0x021, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "ngày mai",       chain_hash: 0x022, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "bây giờ",        chain_hash: 0x023, emotion: EmotionTag::NEUTRAL },
    // Question phrases
    PhraseEntry { phrase: "tại sao",        chain_hash: 0x030, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "như thế nào",    chain_hash: 0x031, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "là gì",          chain_hash: 0x032, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "ở đâu",          chain_hash: 0x033, emotion: EmotionTag::NEUTRAL },
    PhraseEntry { phrase: "cái gì",         chain_hash: 0x034, emotion: EmotionTag::NEUTRAL },
];

// ─────────────────────────────────────────────────────────────────────────────
// PhraseDict
// ─────────────────────────────────────────────────────────────────────────────

/// Parser top-down: phrase → word → char.
pub struct PhraseDict;

impl PhraseDict {
    pub fn new() -> Self { Self }

    /// Parse text → Vec<PhraseMatch> top-down.
    /// Unicode-aware: split theo whitespace (hoạt động với tiếng Việt).
    pub fn parse(&self, text: &str) -> Vec<PhraseMatch> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut result = Vec::new();
        let mut pos    = 0usize;

        while pos < words.len() {
            let max_window = 5.min(words.len() - pos);
            let mut matched = false;

            // Greedy: thử match dài nhất trước
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

    fn lookup_phrase(&self, candidate: &str) -> Option<&'static PhraseEntry> {
        PHRASE_TABLE.iter().find(|e| e.phrase == candidate)
    }

    /// Aggregate EmotionTag theo weighted average.
    /// Phrase dài hơn → weight cao hơn.
    pub fn aggregate_emotion(matches: &[PhraseMatch]) -> EmotionTag {
        if matches.is_empty() { return EmotionTag::NEUTRAL; }
        let mut tv = 0.0f32; let mut ta = 0.0f32;
        let mut td = 0.0f32; let mut ti = 0.0f32;
        let mut tw = 0.0f32;
        for m in matches {
            let w = m.word_count as f32;
            tv += m.emotion.valence   * w;
            ta += m.emotion.arousal   * w;
            td += m.emotion.dominance * w;
            ti += m.emotion.intensity * w;
            tw += w;
        }
        EmotionTag::new(
            (tv/tw).max(-1.0).min(1.0),
            (ta/tw).max(0.0).min(1.0),
            (td/tw).max(0.0).min(1.0),
            (ti/tw).max(0.0).min(1.0),
        )
    }
}

impl Default for PhraseDict { fn default() -> Self { Self::new() } }

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn dict() -> PhraseDict { PhraseDict::new() }

    #[test]
    fn parse_tieng_viet_phong_khach() {
        let matches = dict().parse("tắt đèn phòng khách đi");
        let texts: Vec<&str> = matches.iter().map(|m| m.text.as_str()).collect();
        assert!(texts.contains(&"tắt đèn"),    "tắt đèn = 1 phrase");
        assert!(texts.contains(&"phòng khách"), "phòng khách = 1 phrase");
    }

    #[test]
    fn parse_no_split_phong_khach() {
        let matches = dict().parse("phòng khách");
        assert_eq!(matches.len(), 1, "phòng khách = 1 phrase");
        assert_eq!(matches[0].text, "phòng khách");
        assert_eq!(matches[0].word_count, 2);
    }

    #[test]
    fn parse_emotional_phrase_mat_viec() {
        let matches = dict().parse("tôi mất việc rồi");
        let mat_viec = matches.iter().find(|m| m.text == "mất việc");
        assert!(mat_viec.is_some(), "mất việc phải được parse");
        assert!(mat_viec.unwrap().emotion.valence < 0.0);
    }

    #[test]
    fn parse_met_qua() {
        let matches = dict().parse("tôi mệt quá hôm nay");
        let met_qua = matches.iter().find(|m| m.text == "mệt quá");
        assert!(met_qua.is_some(), "mệt quá phải được parse");
        assert!(met_qua.unwrap().emotion.valence < 0.0);
    }

    #[test]
    fn parse_fallback_word() {
        // "xin chào" chưa có trong phrase table → word fallback
        let matches = dict().parse("xin chào");
        assert_eq!(matches.len(), 2, "fallback → 2 words");
    }

    #[test]
    fn parse_empty() {
        assert!(dict().parse("").is_empty());
    }

    #[test]
    fn aggregate_weighted() {
        let matches = vec![
            PhraseMatch { text: "mất việc".to_string(), chain_hash: Some(0x014),
                emotion: EmotionTag::new(-0.7, 0.5, 0.2, 0.8), word_count: 2 },
            PhraseMatch { text: "buồn".to_string(), chain_hash: None,
                emotion: EmotionTag::new(-0.6, 0.5, 0.3, 0.7), word_count: 1 },
        ];
        let agg = PhraseDict::aggregate_emotion(&matches);
        // weighted avg: (-0.7×2 + -0.6×1) / 3 ≈ -0.667
        assert!(agg.valence < -0.5, "Aggregate negative: {}", agg.valence);
    }

    #[test]
    fn parse_co_don() {
        let matches = dict().parse("tôi cô đơn quá");
        let co_don = matches.iter().find(|m| m.text == "cô đơn");
        assert!(co_don.is_some(), "cô đơn phải được parse");
    }

    #[test]
    fn parse_tai_sao() {
        let matches = dict().parse("tại sao trời mưa");
        let tai_sao = matches.iter().find(|m| m.text == "tại sao");
        assert!(tai_sao.is_some(), "tại sao phải được parse");
    }
}
