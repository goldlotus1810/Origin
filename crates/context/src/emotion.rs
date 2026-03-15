//! # emotion — EmotionTag từ text
//!
//! WordAffect: từng từ có EmotionTag riêng.
//! IntentKind: detect intent từ text pattern.
//! IntentModifier: urgency/politeness/stress/hedge.
//! Cross-modal: text + audio → blend.

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// IntentKind
// ─────────────────────────────────────────────────────────────────────────────

/// Loại intent — detect từ pattern text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentKind {
    /// Học hỏi, hỏi thông tin
    Learn,
    /// Cần hỗ trợ cảm xúc
    Heal,
    /// Ra lệnh thiết bị
    Command,
    /// Nội dung nhạy cảm / rủi ro
    Risk,
    /// Khủng hoảng — ưu tiên tuyệt đối
    Crisis,
    /// Trò chuyện bình thường
    Chat,
}

impl IntentKind {
    /// Detect intent từ text UTF-8.
    pub fn detect(text: &str) -> Self {
        // Crisis — ưu tiên tuyệt đối
        if contains_any(text, &[
            "muốn chết", "không muốn sống", "tự tử", "biến mất mãi",
            "want to die", "kill myself", "end my life",
        ]) { return Self::Crisis; }

        // Command — từ điều khiển
        if contains_any(text, &[
            "tắt", "bật", "mở", "đóng", "điều chỉnh", "chạy", "dừng",
            "turn off", "turn on", "open", "close", "set ",
        ]) { return Self::Command; }

        // Heal — cần hỗ trợ
        if contains_any(text, &[
            "mệt", "buồn", "chán", "khó quá", "cô đơn", "tôi cần",
            "tired", "sad", "depressed", "help me", "lonely",
        ]) { return Self::Heal; }

        // Learn — hỏi thông tin
        if contains_any(text, &[
            "là gì", "tại sao", "giải thích", "cho biết", "như thế nào",
            "what is", "why ", "how ", "explain", "?",
        ]) { return Self::Learn; }

        // Risk
        if contains_any(text, &[
            "nguy hiểm", "có vấn đề", "cẩn thận",
            "dangerous", "problem", "risk",
        ]) { return Self::Risk; }

        Self::Chat
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// IntentModifier
// ─────────────────────────────────────────────────────────────────────────────

/// Modifier điều chỉnh EmotionTag.
#[derive(Debug, Clone, Copy, Default)]
pub struct IntentModifier {
    /// Mức độ khẩn cấp ∈ [0, 1]
    pub urgency:    f32,
    /// Mức độ lịch sự ∈ [0, 1]
    pub politeness: f32,
    /// Mức độ stress ∈ [0, 1]
    pub stress:     f32,
    /// Hedge (không chắc) ∈ [0, 1]
    pub hedge:      f32,
}

impl IntentModifier {
    /// Detect modifiers từ text UTF-8.
    pub fn detect(text: &str) -> Self {
        let upper = text.chars().filter(|c| c.is_uppercase()).count();
        let total = text.chars().filter(|c| c.is_alphabetic()).count();
        let stress = if total > 0 { (upper as f32 / total as f32).min(1.0) } else { 0.0 };

        let urgency = if contains_any(text, &["ngay", "ngay bây giờ", "gấp", "urgent", "!!!"]) {
            0.8
        } else if text.contains('!') { 0.4 } else { 0.0 };

        let politeness = if contains_any(text, &["làm ơn", "xin ", "please", "could you", "cảm ơn"]) {
            0.7
        } else { 0.2 };

        let hedge = if contains_any(text, &["có lẽ", "không biết", "maybe", "perhaps", "might"]) {
            0.6
        } else { 0.0 };

        Self { urgency, politeness, stress, hedge }
    }

    /// Áp dụng modifier vào EmotionTag.
    pub fn apply(&self, mut emo: EmotionTag) -> EmotionTag {
        emo.arousal   = (emo.arousal   + self.urgency    * 0.3).min(1.0);
        emo.valence   = (emo.valence   - self.stress     * 0.2).max(-1.0);
        emo.dominance = (emo.dominance + self.politeness * 0.1).min(1.0);
        emo.intensity = (emo.intensity - self.hedge      * 0.2).max(0.0);
        emo
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WordAffect
// ─────────────────────────────────────────────────────────────────────────────

/// EmotionTag cho một từ/cụm từ.
pub fn word_affect(word: &str) -> EmotionTag {
    // Tích cực
    if contains_any(word, &["vui", "hạnh phúc", "tuyệt", "yêu", "happy", "love", "great", "wonderful"]) {
        return EmotionTag::new(0.8, 0.7, 0.6, 0.8);
    }
    // Buồn
    if contains_any(word, &["buồn", "khổ", "đau", "sad", "pain", "hurt", "miserable"]) {
        return EmotionTag::new(-0.7, 0.5, 0.3, 0.7);
    }
    // Giận
    if contains_any(word, &["giận", "tức", "bực", "angry", "rage", "mad", "furious"]) {
        return EmotionTag::new(-0.6, 0.9, 0.7, 0.8);
    }
    // Sợ
    if contains_any(word, &["sợ", "lo", "sợ hãi", "scared", "fear", "afraid", "anxious"]) {
        return EmotionTag::new(-0.7, 0.8, 0.2, 0.7);
    }
    // Mệt
    if contains_any(word, &["mệt", "kiệt sức", "tired", "exhausted", "fatigue"]) {
        return EmotionTag::new(-0.4, 0.2, 0.3, 0.5);
    }
    // Mất mát
    if contains_any(word, &["mất", "hỏng", "thất bại", "lose", "lost", "fail", "broke"]) {
        return EmotionTag::new(-0.6, 0.4, 0.3, 0.6);
    }
    // Trung lập
    EmotionTag::NEUTRAL
}

/// Tính EmotionTag base cho câu từ các từ.
pub fn sentence_base_affect(words: &[&str]) -> EmotionTag {
    if words.is_empty() { return EmotionTag::NEUTRAL; }
    let mut tv = 0.0f32; let mut ta = 0.0f32;
    let mut td = 0.0f32; let mut ti = 0.0f32;
    for &w in words {
        let e = word_affect(w);
        tv += e.valence; ta += e.arousal;
        td += e.dominance; ti += e.intensity;
    }
    let n = words.len() as f32;
    EmotionTag::new(tv/n, ta/n, td/n, ti/n)
}

// ─────────────────────────────────────────────────────────────────────────────
// Cross-modal fusion
// ─────────────────────────────────────────────────────────────────────────────

/// Blend text EmotionTag với audio signals.
/// Text mâu thuẫn audio → audio thắng.
pub fn blend_with_audio(
    text:          EmotionTag,
    audio_valence: f32,
    audio_energy:  f32,
    audio_pitch:   f32,
) -> EmotionTag {
    let conflict = (text.valence > 0.05 && audio_valence < -0.10)
                || (text.valence < -0.05 && audio_valence > 0.10);
    let pitch_anomaly = audio_pitch < 150.0 && audio_pitch > 50.0;

    if conflict || pitch_anomaly {
        EmotionTag {
            valence:   audio_valence,
            arousal:   text.arousal.max(audio_energy),
            dominance: text.dominance * 0.6,
            intensity: text.intensity.max(audio_energy),
        }
    } else {
        EmotionTag {
            valence:   text.valence   * 0.6 + audio_valence * 0.4,
            arousal:   text.arousal   * 0.6 + audio_energy  * 0.4,
            dominance: text.dominance,
            intensity: text.intensity * 0.7 + audio_energy  * 0.3,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper
// ─────────────────────────────────────────────────────────────────────────────

/// Kiểm tra text có chứa bất kỳ needle nào không — UTF-8 native.
pub(crate) fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|&n| text.contains(n))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intent_crisis_tieng_viet() {
        assert_eq!(IntentKind::detect("tôi muốn chết"), IntentKind::Crisis);
        assert_eq!(IntentKind::detect("không muốn sống nữa"), IntentKind::Crisis);
    }

    #[test]
    fn intent_command_tieng_viet() {
        assert_eq!(IntentKind::detect("tắt đèn phòng khách"), IntentKind::Command);
        assert_eq!(IntentKind::detect("mở cửa sổ"), IntentKind::Command);
    }

    #[test]
    fn intent_heal_tieng_viet() {
        assert_eq!(IntentKind::detect("tôi mệt quá hôm nay"), IntentKind::Heal);
        assert_eq!(IntentKind::detect("tôi buồn lắm"), IntentKind::Heal);
        assert_eq!(IntentKind::detect("cô đơn quá"), IntentKind::Heal);
    }

    #[test]
    fn intent_learn_tieng_viet() {
        assert_eq!(IntentKind::detect("tại sao trời mưa?"), IntentKind::Learn);
        assert_eq!(IntentKind::detect("cái này là gì?"), IntentKind::Learn);
    }

    #[test]
    fn intent_english() {
        assert_eq!(IntentKind::detect("turn off the lights"), IntentKind::Command);
        assert_eq!(IntentKind::detect("what is photosynthesis?"), IntentKind::Learn);
        assert_eq!(IntentKind::detect("i feel so sad today"), IntentKind::Heal);
    }

    #[test]
    fn modifier_urgency_bang_cham_than() {
        let m = IntentModifier::detect("tắt đèn ngay!!!");
        assert!(m.urgency > 0.5, "urgency={}", m.urgency);
    }

    #[test]
    fn modifier_politeness_lam_on() {
        let m = IntentModifier::detect("làm ơn tắt đèn giúp tôi");
        assert!(m.politeness > 0.5, "politeness={}", m.politeness);
    }

    #[test]
    fn modifier_stress_caps() {
        let m = IntentModifier::detect("TAT DEN NGAY");
        assert!(m.stress > 0.3, "stress từ CAPS: {}", m.stress);
    }

    #[test]
    fn word_affect_tieng_viet() {
        assert!(word_affect("vui").valence > 0.5,  "vui → positive");
        assert!(word_affect("buồn").valence < -0.5, "buồn → negative");
        assert!(word_affect("sợ").valence   < -0.5, "sợ → negative");
        assert!(word_affect("mệt").valence  < -0.3, "mệt → negative");
    }

    #[test]
    fn word_affect_english() {
        assert!(word_affect("happy").valence  > 0.5);
        assert!(word_affect("sad").valence    < -0.5);
        assert!(word_affect("afraid").valence < -0.5);
    }

    #[test]
    fn blend_audio_conflict() {
        let text = EmotionTag::new(0.7, 0.5, 0.6, 0.7);
        let blended = blend_with_audio(text, -0.4, 0.3, 130.0);
        assert!(blended.valence < 0.0, "Audio thắng khi conflict: {}", blended.valence);
    }

    #[test]
    fn blend_audio_pitch_anomaly() {
        let text = EmotionTag::new(0.1, 0.3, 0.5, 0.3);
        let blended = blend_with_audio(text, -0.5, 0.4, 120.0);
        assert!(blended.valence < 0.0, "Pitch thấp → override: {}", blended.valence);
    }

    #[test]
    fn sentence_base_negative_sentence() {
        let words = ["tôi", "buồn", "vì", "mất", "việc"];
        let e = sentence_base_affect(&words);
        assert!(e.valence < 0.0, "Câu buồn → V âm: {}", e.valence);
    }
}
