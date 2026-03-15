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
    /// Detect intent từ text bytes.
    pub fn detect(text: &str) -> Self {
        let b = text.as_bytes();

        // Crisis — ưu tiên tuyệt đối
        if contains_any(b, &[
            b"muon chet", b"khong muon song",
            b"tu tu", b"bien mat mai",
            b"want to die", b"kill myself",
        ]) {
            return Self::Crisis;
        }

        // Command — từ điều khiển
        if contains_any(b, &[
            b"tat ", b"bat ", b"mo ", b"dong ",
            b"turn off", b"turn on", b"open", b"close",
            b"dieu chinh", b"set ", b"chay ",
        ]) {
            return Self::Command;
        }

        // Heal — cần hỗ trợ
        if contains_any(b, &[
            b"met ", b"buon", b"chan ", b"kho qua",
            b"tired", b"sad", b"depressed", b"help me",
            b"can giup",
        ]) {
            return Self::Heal;
        }

        // Learn — hỏi thông tin
        if contains_any(b, &[
            b"la gi", b"tai sao", b"giai thich",
            b"what is", b"why ", b"how ", b"explain",
            b"?",
        ]) {
            return Self::Learn;
        }

        // Risk
        if contains_any(b, &[
            b"nguy hiem", b"co van de",
            b"dangerous", b"problem", b"risk",
        ]) {
            return Self::Risk;
        }

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
    /// Detect modifiers từ text.
    pub fn detect(text: &str) -> Self {
        let b     = text.as_bytes();
        let upper = text.chars().filter(|c| c.is_uppercase()).count();
        let total = text.chars().filter(|c| c.is_alphabetic()).count();

        let stress = if total > 0 {
            (upper as f32 / total as f32).min(1.0)
        } else { 0.0 };

        let urgency = if contains_any(b, &[b"ngay", b"khong ", b"gap", b"now", b"urgent", b"!!!"]) {
            0.8
        } else if text.contains('!') {
            0.4
        } else { 0.0 };

        let politeness = if contains_any(b, &[b"lam on", b"xin ", b"please", b"could you", b"cam on"]) {
            0.7
        } else { 0.2 };

        let hedge = if contains_any(b, &[b"co le", b"khong biet", b"maybe", b"perhaps", b"might"]) {
            0.6
        } else { 0.0 };

        Self { urgency, politeness, stress, hedge }
    }

    /// Áp dụng modifier vào EmotionTag.
    pub fn apply(&self, mut emo: EmotionTag) -> EmotionTag {
        // Urgency → tăng arousal
        emo.arousal = (emo.arousal + self.urgency * 0.3).min(1.0);
        // Stress → giảm valence
        emo.valence = (emo.valence - self.stress * 0.2).max(-1.0);
        // Politeness → tăng dominance
        emo.dominance = (emo.dominance + self.politeness * 0.1).min(1.0);
        // Hedge → giảm intensity
        emo.intensity = (emo.intensity - self.hedge * 0.2).max(0.0);
        emo
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WordAffect — từng từ có EmotionTag
// ─────────────────────────────────────────────────────────────────────────────

/// EmotionTag cho một từ — từ UCD lookup.
///
/// Dùng arousal_of + valence_of từ UCD cho emoji/symbol.
/// Dùng pattern matching cho text thông thường.
pub fn word_affect(word: &str) -> EmotionTag {
    let b = word.as_bytes();

    // Positive emotions
    if contains_any(b, &[b"vui", b"hanh phuc", b"tuyet", b"yeu", b"happy", b"love", b"great"]) {
        return EmotionTag::new(0.8, 0.7, 0.6, 0.8);
    }
    // Negative emotions
    if contains_any(b, &[b"buon", b"kho", b"dau", b"sad", b"pain", b"hurt"]) {
        return EmotionTag::new(-0.7, 0.5, 0.3, 0.7);
    }
    if contains_any(b, &[b"gian", b"tuc", b"angry", b"rage", b"mad"]) {
        return EmotionTag::new(-0.6, 0.9, 0.7, 0.8);
    }
    if contains_any(b, &[b"so ", b"so_", b"scared", b"fear", b"afraid", b"lo lang"]) {
        return EmotionTag::new(-0.7, 0.8, 0.2, 0.7);
    }
    // Neutral verbs
    if contains_any(b, &[b"la", b"co", b"duoc", b"is", b"are", b"have"]) {
        return EmotionTag::NEUTRAL;
    }
    // Loss/negative events
    if contains_any(b, &[b"mat", b"hong", b"cu", b"lose", b"lost", b"broke", b"fail"]) {
        return EmotionTag::new(-0.6, 0.4, 0.3, 0.6);
    }

    EmotionTag::NEUTRAL
}

/// Tính EmotionTag cho toàn bộ câu từ các từ.
///
/// Không phải trung bình — walk qua Silk (trong sentence_affect).
/// Đây chỉ là base — SentenceAffect sẽ amplify.
pub fn sentence_base_affect(words: &[&str]) -> EmotionTag {
    if words.is_empty() { return EmotionTag::NEUTRAL; }

    let mut total_v = 0.0f32;
    let mut total_a = 0.0f32;
    let mut total_d = 0.0f32;
    let mut total_i = 0.0f32;
    let mut count   = 0usize;

    for &word in words {
        let e = word_affect(word);
        total_v += e.valence;
        total_a += e.arousal;
        total_d += e.dominance;
        total_i += e.intensity;
        count   += 1;
    }

    let n = count as f32;
    EmotionTag::new(total_v/n, total_a/n, total_d/n, total_i/n)
}

// ─────────────────────────────────────────────────────────────────────────────
// Cross-modal fusion
// ─────────────────────────────────────────────────────────────────────────────

/// Blend text EmotionTag với audio signals.
///
/// Nếu text và audio mâu thuẫn về valence → audio thắng.
/// Pitch thấp (giọng run) → override text vui vẻ.
pub fn blend_with_audio(
    text:         EmotionTag,
    audio_valence: f32,   // valence từ audio
    audio_energy:  f32,   // energy/loudness ∈ [0,1]
    audio_pitch:   f32,   // Hz — thấp = giọng run
) -> EmotionTag {
    // Conflict detection: text và audio trái chiều
    let conflict = (text.valence > 0.05 && audio_valence < -0.10)
                || (text.valence < -0.05 && audio_valence > 0.10);

    // Pitch anomaly: pitch thấp bất thường = lo lắng
    let pitch_anomaly = audio_pitch < 150.0 && audio_pitch > 50.0;

    if conflict || pitch_anomaly {
        // Audio override
        EmotionTag {
            valence:   audio_valence,
            arousal:   text.arousal.max(audio_energy),
            dominance: text.dominance * 0.6,
            intensity: text.intensity.max(audio_energy),
        }
    } else {
        // Blend 60/40 text/audio
        EmotionTag {
            valence:   text.valence   * 0.6 + audio_valence * 0.4,
            arousal:   text.arousal   * 0.6 + audio_energy  * 0.4,
            dominance: text.dominance,
            intensity: text.intensity * 0.7 + audio_energy  * 0.3,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) fn contains_any(hay: &[u8], needles: &[&[u8]]) -> bool {
    for n in needles {
        if hay.windows(n.len()).any(|w| w == *n) { return true; }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intent_crisis_priority() {
        assert_eq!(IntentKind::detect("toi muon chet"), IntentKind::Crisis);
        assert_eq!(IntentKind::detect("want to die please"), IntentKind::Crisis);
    }

    #[test]
    fn intent_command() {
        assert_eq!(IntentKind::detect("tat den phong khach"), IntentKind::Command);
        assert_eq!(IntentKind::detect("turn off the lights"), IntentKind::Command);
    }

    #[test]
    fn intent_heal() {
        assert_eq!(IntentKind::detect("toi met qua hom nay"), IntentKind::Heal);
        assert_eq!(IntentKind::detect("i feel so sad"), IntentKind::Heal);
    }

    #[test]
    fn intent_learn() {
        assert_eq!(IntentKind::detect("tai sao troi mua?"), IntentKind::Learn);
        assert_eq!(IntentKind::detect("what is photosynthesis?"), IntentKind::Learn);
    }

    #[test]
    fn intent_chat_default() {
        assert_eq!(IntentKind::detect("hom nay troi dep"), IntentKind::Chat);
    }

    #[test]
    fn modifier_urgency_exclamation() {
        let m = IntentModifier::detect("tat den ngay!!!");
        assert!(m.urgency > 0.5, "urgency={}", m.urgency);
    }

    #[test]
    fn modifier_politeness() {
        let m = IntentModifier::detect("lam on tat den giup toi");
        assert!(m.politeness > 0.5, "politeness={}", m.politeness);
    }

    #[test]
    fn modifier_stress_caps() {
        let m = IntentModifier::detect("TAT DEN NGAY");
        assert!(m.stress > 0.3, "stress từ CAPS: {}", m.stress);
    }

    #[test]
    fn modifier_apply_urgency_increases_arousal() {
        let base = EmotionTag::new(0.0, 0.3, 0.5, 0.5);
        let m    = IntentModifier { urgency: 0.8, ..Default::default() };
        let applied = m.apply(base);
        assert!(applied.arousal > base.arousal,
            "Urgency → arousal tăng: {} > {}", applied.arousal, base.arousal);
    }

    #[test]
    fn word_affect_positive() {
        let e = word_affect("vui");
        assert!(e.valence > 0.5, "vui → positive valence");
    }

    #[test]
    fn word_affect_negative() {
        let e = word_affect("buon");
        assert!(e.valence < -0.5, "buon → negative valence");
    }

    #[test]
    fn word_affect_neutral() {
        let e = word_affect("la");
        assert!(e.valence.abs() < 0.2, "la → neutral");
    }

    #[test]
    fn blend_audio_conflict() {
        // Text vui nhưng giọng run thấp → audio thắng
        let text = EmotionTag::new(0.7, 0.5, 0.6, 0.7);
        let blended = blend_with_audio(text, -0.4, 0.3, 130.0);
        assert!(blended.valence < 0.0,
            "Conflict: audio (negative) thắng text (positive): {}", blended.valence);
    }

    #[test]
    fn blend_audio_agree() {
        // Text và audio cùng tích cực → blend
        let text = EmotionTag::new(0.6, 0.7, 0.5, 0.6);
        let blended = blend_with_audio(text, 0.5, 0.8, 220.0);
        assert!(blended.valence > 0.0,
            "Agree: blend positive: {}", blended.valence);
        // Kết quả gần text.valence×0.6 + audio×0.4
        let expected = 0.6 * 0.6 + 0.5 * 0.4;
        assert!((blended.valence - expected).abs() < 0.05);
    }

    #[test]
    fn blend_audio_pitch_anomaly() {
        // Pitch thấp dù text neutral → audio override
        let text = EmotionTag::new(0.1, 0.3, 0.5, 0.3);
        let blended = blend_with_audio(text, -0.5, 0.4, 120.0); // pitch rất thấp
        assert!(blended.valence < 0.0,
            "Pitch anomaly → audio override: {}", blended.valence);
    }

    #[test]
    fn sentence_base_affect_negative() {
        let words = ["toi", "buon", "vi", "mat", "viec"];
        let e = sentence_base_affect(&words);
        assert!(e.valence < 0.0, "Câu buồn → V âm: {}", e.valence);
    }
}
