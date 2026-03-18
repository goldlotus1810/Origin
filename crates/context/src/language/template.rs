//! # template — Data Template Registry
//!
//! Thay vì hardcode lexicon, keywords, aliases trong Rust static slices,
//! DataTemplate cho phép:
//!   1. Default data — built-in, compile-time (hiện tại)
//!   2. Learned data — từ Node storage (Silk edges + aliases)
//!   3. Extend runtime — học thêm từ mới qua conversation
//!
//! Khi hệ thống học từ mới qua Silk co-activation:
//!   word + emotion → WordEntry mới → push vào DynamicLexicon
//!   → select_words() sẽ include từ mới
//!
//! Đây là bước đầu thay thế hardcode:
//!   Phase 1: DynamicLexicon wraps CORE_LEXICON + learned words (hiện tại)
//!   Phase 2: Load từ data files (JSON/TOML) khi có std
//!   Phase 3: Toàn bộ lexicon từ Node storage (no hardcode)

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// DynamicWordEntry — version mở rộng của WordEntry
// ─────────────────────────────────────────────────────────────────────────────

/// Từ vựng có thể học thêm — owned string thay vì &'static str.
#[derive(Debug, Clone)]
pub struct DynamicWordEntry {
    /// Từ (có thể là từ mới học được)
    pub word: String,
    /// Emotion values
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
    /// Nguồn: built-in hoặc learned
    pub source: WordSource,
    /// Số lần được co-activate (fire count) — từ Silk
    pub fire_count: u32,
}

/// Nguồn gốc của từ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordSource {
    /// Built-in từ compile-time (CORE_LEXICON)
    BuiltIn,
    /// Learned từ conversation qua Silk co-activation
    Learned,
    /// Loaded từ data file (Phase 2)
    DataFile,
}

// ─────────────────────────────────────────────────────────────────────────────
// DynamicLexicon — wraps static + learned words
// ─────────────────────────────────────────────────────────────────────────────

/// DynamicLexicon — bao gồm built-in + learned words.
///
/// select_words() dùng lexicon này thay vì chỉ CORE_LEXICON.
/// Từ mới học được qua conversation sẽ tự động thêm vào.
pub struct DynamicLexicon {
    /// Learned words — mở rộng runtime
    learned: Vec<DynamicWordEntry>,
}

impl DynamicLexicon {
    /// Tạo mới — chỉ có built-in.
    pub fn new() -> Self {
        Self {
            learned: Vec::new(),
        }
    }

    /// Thêm từ mới đã học được.
    ///
    /// Nếu từ đã có → cập nhật fire_count (không xóa — QT8).
    pub fn learn_word(&mut self, word: &str, valence: f32, arousal: f32, dominance: f32) {
        // Check đã có chưa
        if let Some(existing) = self.learned.iter_mut().find(|e| e.word == word) {
            existing.fire_count += 1;
            // Update emotion qua moving average
            let w = 1.0 / (existing.fire_count as f32 + 1.0);
            existing.valence = existing.valence * (1.0 - w) + valence * w;
            existing.arousal = existing.arousal * (1.0 - w) + arousal * w;
            existing.dominance = existing.dominance * (1.0 - w) + dominance * w;
            return;
        }

        self.learned.push(DynamicWordEntry {
            word: String::from(word),
            valence,
            arousal,
            dominance,
            source: WordSource::Learned,
            fire_count: 1,
        });
    }

    /// Tổng số từ learned.
    pub fn learned_count(&self) -> usize {
        self.learned.len()
    }

    /// Iterator over learned words.
    pub fn learned_words(&self) -> &[DynamicWordEntry] {
        &self.learned
    }

    /// Tìm từ trong learned lexicon.
    pub fn find_learned(&self, word: &str) -> Option<&DynamicWordEntry> {
        self.learned.iter().find(|e| e.word == word)
    }

    /// Emotion tag cho 1 từ — ưu tiên learned, fallback built-in.
    pub fn word_emotion(&self, word: &str) -> Option<EmotionTag> {
        // 1. Learned (ưu tiên — đã adapted theo context)
        if let Some(e) = self.find_learned(word) {
            return Some(EmotionTag {
                valence: e.valence,
                arousal: e.arousal,
                dominance: e.dominance,
                intensity: e.valence.abs(),
            });
        }

        // 2. Built-in (CORE_LEXICON từ word_guide.rs)
        // Tránh import circular — caller dùng trực tiếp word_guide
        None
    }
}

impl Default for DynamicLexicon {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// KeywordTemplate — extensible keyword tables
// ─────────────────────────────────────────────────────────────────────────────

/// Loại keyword — matching intent categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordCategory {
    /// Confirm/approve keywords
    Confirm,
    /// Deny/reject keywords
    Deny,
    /// Command keywords (device control)
    Command,
    /// Emotion keywords (cảm xúc mở rộng)
    Emotion,
    /// Custom category (user-defined)
    Custom,
}

/// KeywordTemplate — keyword tables extensible tại runtime.
///
/// Built-in keywords nằm trong intent.rs (KW_COMMAND, KW_CONFIRM, KW_DENY).
/// Từ mới thêm qua learn_keyword() sẽ tham gia classify intent.
pub struct KeywordTemplate {
    /// Learned keywords
    keywords: Vec<(String, KeywordCategory)>,
}

impl KeywordTemplate {
    /// Tạo mới.
    pub fn new() -> Self {
        Self {
            keywords: Vec::new(),
        }
    }

    /// Thêm keyword mới.
    pub fn learn_keyword(&mut self, word: &str, category: KeywordCategory) {
        if !self.keywords.iter().any(|(w, _)| w == word) {
            self.keywords.push((String::from(word), category));
        }
    }

    /// Tìm keywords theo category.
    pub fn by_category(&self, cat: KeywordCategory) -> Vec<&str> {
        self.keywords
            .iter()
            .filter(|(_, c)| *c == cat)
            .map(|(w, _)| w.as_str())
            .collect()
    }

    /// Số learned keywords.
    pub fn len(&self) -> usize {
        self.keywords.len()
    }

    /// Rỗng?
    pub fn is_empty(&self) -> bool {
        self.keywords.is_empty()
    }

    /// Kiểm tra text chứa keyword nào thuộc category.
    pub fn contains_category(&self, text: &str, cat: KeywordCategory) -> bool {
        let lo = text.to_lowercase();
        self.keywords
            .iter()
            .any(|(w, c)| *c == cat && lo.contains(w.as_str()))
    }
}

impl Default for KeywordTemplate {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── DynamicLexicon ─────────────────────────────────────────────────────────

    #[test]
    fn lexicon_new_empty() {
        let lex = DynamicLexicon::new();
        assert_eq!(lex.learned_count(), 0);
    }

    #[test]
    fn learn_new_word() {
        let mut lex = DynamicLexicon::new();
        lex.learn_word("thất vọng", -0.55, 0.40, 0.30);
        assert_eq!(lex.learned_count(), 1);
        let w = lex.find_learned("thất vọng").unwrap();
        assert_eq!(w.source, WordSource::Learned);
        assert_eq!(w.fire_count, 1);
    }

    #[test]
    fn learn_same_word_updates() {
        let mut lex = DynamicLexicon::new();
        lex.learn_word("hào hứng", 0.70, 0.80, 0.70);
        lex.learn_word("hào hứng", 0.75, 0.85, 0.75);
        assert_eq!(lex.learned_count(), 1, "Same word → update, not duplicate");
        let w = lex.find_learned("hào hứng").unwrap();
        assert_eq!(w.fire_count, 2);
        // Emotion is moving average, not raw overwrite
        assert!(w.valence > 0.70);
    }

    #[test]
    fn word_emotion_from_learned() {
        let mut lex = DynamicLexicon::new();
        lex.learn_word("phức tạp", -0.15, 0.55, 0.45);
        let emo = lex.word_emotion("phức tạp").unwrap();
        assert!((emo.valence - (-0.15)).abs() < 0.01);
    }

    #[test]
    fn word_emotion_unknown_none() {
        let lex = DynamicLexicon::new();
        assert!(lex.word_emotion("xyz_unknown").is_none());
    }

    // ── KeywordTemplate ────────────────────────────────────────────────────────

    #[test]
    fn keyword_template_empty() {
        let kt = KeywordTemplate::new();
        assert!(kt.is_empty());
    }

    #[test]
    fn learn_keyword() {
        let mut kt = KeywordTemplate::new();
        kt.learn_keyword("chắc chắn", KeywordCategory::Confirm);
        assert_eq!(kt.len(), 1);
        assert!(kt.contains_category("tôi chắc chắn rồi", KeywordCategory::Confirm));
    }

    #[test]
    fn keyword_by_category() {
        let mut kt = KeywordTemplate::new();
        kt.learn_keyword("phê duyệt", KeywordCategory::Confirm);
        kt.learn_keyword("cấm", KeywordCategory::Deny);
        let confirms = kt.by_category(KeywordCategory::Confirm);
        assert_eq!(confirms.len(), 1);
        assert_eq!(confirms[0], "phê duyệt");
    }

    #[test]
    fn keyword_no_duplicate() {
        let mut kt = KeywordTemplate::new();
        kt.learn_keyword("ok", KeywordCategory::Confirm);
        kt.learn_keyword("ok", KeywordCategory::Confirm);
        assert_eq!(kt.len(), 1);
    }

    #[test]
    fn keyword_not_contains() {
        let kt = KeywordTemplate::new();
        assert!(!kt.contains_category("hello world", KeywordCategory::Confirm));
    }
}
