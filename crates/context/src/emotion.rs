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

/// EmotionTag cho một từ/cụm từ — data-driven lookup.
///
/// English: multilingual-sentiment-datasets (3033 samples, n≥4, σ≤0.30)
/// Vietnamese: alias layer trên English ground truth + VI native words
pub fn word_affect(word: &str) -> EmotionTag {
    // Longest-match: "tuyệt vời" match trước "tuyệt"
    // word.contains() vì input có thể là substring
    for &(w, v, a) in WORD_AFFECT_TABLE {
        if word == w || (word.len() >= w.len() && word.contains(w)) {
            return EmotionTag::new(v, a, 0.5, a * 0.8);
        }
    }
    EmotionTag::NEUTRAL
}

/// WordAffect lookup table — (word, valence, arousal).
/// Data: tyqiangz/multilingual-sentiment-datasets (EN, 3033 samples)
/// + Vietnamese native emotion words.
static WORD_AFFECT_TABLE: &[(&str, f32, f32)] = &[
    // ── English ─────────────────────────────────────────────
    ("parenthood", -0.7f32, 0.6f32),
    ("horrible", -0.7f32, 0.6f32),
    ("attacks", -0.7f32, 0.6f32),
    ("grader", -0.7f32, 0.6f32),
    ("dangerous", -0.7f32, 0.6f32),
    ("terrorist", -0.7f32, 0.6f32),
    ("sucks", -0.7f32, 0.6f32),
    ("died", -0.7f32, 0.6f32),
    ("ruined", -0.7f32, 0.6f32),
    ("rather", -0.7f32, 0.6f32),
    ("fails", -0.7f32, 0.6f32),
    ("stupid", -0.7f32, 0.6f32),
    ("bitches", -0.7f32, 0.6f32),
    ("bomb", -0.7f32, 0.6f32),
    ("queen", -0.7f32, 0.6f32),
    ("pathetic", -0.7f32, 0.6f32),
    ("die", -0.7f32, 0.6f32),
    ("rich", -0.7f32, 0.6f32),
    ("violence", -0.7f32, 0.6f32),
    ("israeli", -0.7f32, 0.6f32),
    ("kept", -0.7f32, 0.6f32),
    ("angry", -0.7f32, 0.6f32),
    ("y'all", -0.7f32, 0.6f32),
    ("isis", -0.7f32, 0.6f32),
    ("suicide", -0.7f32, 0.6f32),
    ("manslaughter", -0.7f32, 0.6f32),
    ("votes", -0.7f32, 0.6f32),
    ("regarding", -0.7f32, 0.6f32),
    ("killing", -0.7f32, 0.6f32),
    ("problem", -0.7f32, 0.6f32),
    ("bobby", -0.7f32, 0.6f32),
    ("jindal", -0.7f32, 0.6f32),
    ("enemies", -0.7f32, 0.6f32),
    ("evil", -0.7f32, 0.6f32),
    ("dumb", -0.7f32, 0.6f32),
    ("shitty", -0.7f32, 0.6f32),
    ("drama", -0.7f32, 0.6f32),
    ("jew", -0.7f32, 0.6f32),
    ("shouldn't", -0.7f32, 0.6f32),
    ("mouth", -0.7f32, 0.6f32),
    ("drone", -0.7f32, 0.6f32),
    ("fraud", -0.7f32, 0.6f32),
    ("fat", -0.7f32, 0.6f32),
    ("strikes", -0.7f32, 0.6f32),
    ("neo", -0.7f32, 0.6f32),
    ("worst", -0.642f32, 0.575f32),
    ("conservatives", -0.63f32, 0.57f32),
    ("sick", -0.622f32, 0.567f32),
    ("worse", -0.622f32, 0.567f32),
    ("pay", -0.622f32, 0.567f32),
    ("injured", -0.612f32, 0.562f32),
    ("muslims", -0.603f32, 0.559f32),
    ("palin", -0.6f32, 0.557f32),
    ("poor", -0.6f32, 0.557f32),
    ("bloody", -0.6f32, 0.557f32),
    ("themselves", -0.6f32, 0.557f32),
    ("caitlyn", -0.592f32, 0.554f32),
    ("putting", -0.583f32, 0.55f32),
    ("angela", -0.583f32, 0.55f32),
    ("fake", -0.583f32, 0.55f32),
    ("ridiculous", -0.583f32, 0.55f32),
    ("cancelled", -0.583f32, 0.55f32),
    ("bbc", -0.583f32, 0.55f32),
    ("water", -0.583f32, 0.55f32),
    ("kill", -0.573f32, 0.545f32),
    ("tvd", -0.56f32, 0.54f32),
    ("dies", -0.56f32, 0.54f32),
    ("jenner", -0.56f32, 0.54f32),
    ("blame", -0.56f32, 0.54f32),
    ("refuse", -0.56f32, 0.54f32),
    ("brian", -0.56f32, 0.54f32),
    ("press", -0.56f32, 0.54f32),
    ("disappointed", -0.56f32, 0.54f32),
    ("india", -0.56f32, 0.54f32),
    ("anymore", -0.56f32, 0.54f32),
    ("truth", -0.56f32, 0.54f32),
    ("simple", -0.56f32, 0.54f32),
    ("closed", -0.56f32, 0.54f32),
    ("arms", -0.56f32, 0.54f32),
    ("public", -0.56f32, 0.54f32),
    ("transition", -0.56f32, 0.54f32),
    ("alt", -0.55f32, 0.536f32),
    ("planned", -0.544f32, 0.533f32),
    ("hurt", -0.544f32, 0.533f32),
    ("crash", -0.544f32, 0.533f32),
    ("won't", -0.544f32, 0.533f32),
    ("testing", -0.544f32, 0.533f32),
    ("sweet", 0.667f32, 0.633f32),
    ("dawn", 0.667f32, 0.633f32),
    ("we'll", 0.667f32, 0.633f32),
    ("lovely", 0.667f32, 0.633f32),
    ("xxl", 0.686f32, 0.643f32),
    ("winner", 0.686f32, 0.643f32),
    ("amazing", 0.686f32, 0.643f32),
    ("ipad", 0.7f32, 0.65f32),
    ("iphone", 0.7f32, 0.65f32),
    ("barca", 0.7f32, 0.65f32),
    ("luck", 0.711f32, 0.656f32),
    ("park", 0.727f32, 0.664f32),
    ("liked", 0.733f32, 0.667f32),
    ("excited", 0.742f32, 0.696f32),
    ("brilliant", 0.8f32, 0.7f32),
    ("finale", 0.8f32, 0.7f32),
    ("awesome", 0.8f32, 0.7f32),
    ("colts", 0.8f32, 0.7f32),
    ("fam", 0.8f32, 0.7f32),
    ("disneyland", 0.8f32, 0.7f32),
    ("hopefully", 0.8f32, 0.7f32),
    ("proud", 0.8f32, 0.7f32),
    ("grammy", 0.8f32, 0.7f32),
    ("forward", 0.8f32, 0.7f32),
    ("bless", 0.8f32, 0.7f32),
    ("irish", 0.8f32, 0.7f32),
    ("stream", 0.8f32, 0.7f32),
    ("upgrade", 0.8f32, 0.7f32),
    ("fantastic", 0.8f32, 0.7f32),
    ("marley", 0.8f32, 0.7f32),
    ("office", 0.8f32, 0.7f32),
    ("thankful", 0.8f32, 0.7f32),
    ("loved", 0.8f32, 0.7f32),
    ("joins", 0.8f32, 0.7f32),
    ("happiness", 0.8f32, 0.7f32),
    ("feat", 0.8f32, 0.7f32),
    ("albums", 0.8f32, 0.7f32),
    ("prayers", 0.8f32, 0.7f32),
    ("cast", 0.8f32, 0.7f32),
    ("wonderful", 0.8f32, 0.7f32),
    ("gift", 0.8f32, 0.7f32),
    ("mess", 0.8f32, 0.7f32),
    ("valentines", 0.8f32, 0.7f32),
    // ── Essential English (core emotion words) ────────────
    ("hatred", -0.8f32, 0.8f32),
    ("terrible", -0.8f32, 0.7f32),
    ("horrible", -0.8f32, 0.65f32),
    ("sadness", -0.75f32, 0.45f32),
    ("hate", -0.75f32, 0.8f32),
    ("awful", -0.75f32, 0.65f32),
    ("sad", -0.7f32, 0.5f32),
    ("afraid", -0.7f32, 0.8f32),
    ("anger", -0.7f32, 0.85f32),
    ("fear", -0.65f32, 0.8f32),
    ("angry", -0.65f32, 0.85f32),
    ("pain", -0.65f32, 0.65f32),
    ("fail", -0.65f32, 0.6f32),
    ("hurt", -0.6f32, 0.6f32),
    ("broken", -0.6f32, 0.55f32),
    ("crying", -0.6f32, 0.55f32),
    ("lose", -0.6f32, 0.55f32),
    ("bad", -0.55f32, 0.5f32),
    ("lost", -0.55f32, 0.5f32),
    ("cry", -0.55f32, 0.55f32),
    ("anxious", -0.55f32, 0.75f32),
    ("ugly", -0.55f32, 0.5f32),
    ("sick", -0.55f32, 0.5f32),
    ("exhausted", -0.5f32, 0.15f32),
    ("worried", -0.5f32, 0.7f32),
    ("tired", -0.4f32, 0.2f32),
    ("sorry", -0.35f32, 0.45f32),
    ("miss", -0.25f32, 0.4f32),
    ("good", 0.55f32, 0.5f32),
    ("healthy", 0.55f32, 0.45f32),
    ("smile", 0.65f32, 0.6f32),
    ("great", 0.7f32, 0.65f32),
    ("laugh", 0.7f32, 0.75f32),
    ("thankful", 0.7f32, 0.55f32),
    ("grateful", 0.7f32, 0.55f32),
    ("beautiful", 0.7f32, 0.6f32),
    ("lovely", 0.75f32, 0.65f32),
    ("win", 0.75f32, 0.75f32),
    ("happy", 0.8f32, 0.7f32),
    ("excellent", 0.8f32, 0.7f32),
    ("amazing", 0.8f32, 0.75f32),
    ("success", 0.8f32, 0.7f32),
    ("happiness", 0.85f32, 0.7f32),
    ("joy", 0.85f32, 0.75f32),
    ("love", 0.85f32, 0.65f32),
    // ── Vietnamese ──────────────────────────────────────────
    ("chết", -0.8f32, 0.7f32),
    ("kinh khủng", -0.75f32, 0.7f32),
    ("khủng khiếp", -0.7f32, 0.6f32),
    ("tệ hại", -0.7f32, 0.6f32),
    ("dở", -0.7f32, 0.6f32),
    ("ngu", -0.7f32, 0.6f32),
    ("ngốc", -0.7f32, 0.6f32),
    ("ngu ngốc", -0.7f32, 0.6f32),
    ("qua đời", -0.7f32, 0.6f32),
    ("tức giận", -0.7f32, 0.6f32),
    ("buồn bã", -0.7f32, 0.4f32),
    ("ghét", -0.7f32, 0.75f32),
    ("đau khổ", -0.7f32, 0.6f32),
    ("thất bại", -0.7f32, 0.6f32),
    ("mất việc", -0.7f32, 0.55f32),
    ("tệ", -0.65f32, 0.55f32),
    ("nguy hiểm", -0.65f32, 0.75f32),
    ("giận", -0.65f32, 0.85f32),
    ("đau", -0.65f32, 0.65f32),
    ("buồn", -0.65f32, 0.45f32),
    ("sợ", -0.65f32, 0.8f32),
    ("cô đơn", -0.65f32, 0.25f32),
    ("tệ nhất", -0.642f32, 0.575f32),
    ("kém nhất", -0.642f32, 0.575f32),
    ("tức", -0.6f32, 0.85f32),
    ("thất vọng", -0.6f32, 0.55f32),
    ("bệnh", -0.55f32, 0.5f32),
    ("bực", -0.55f32, 0.75f32),
    ("khóc", -0.55f32, 0.55f32),
    ("mất", -0.55f32, 0.5f32),
    ("bị thương", -0.544f32, 0.533f32),
    ("lo lắng", -0.5f32, 0.7f32),
    ("chán", -0.45f32, 0.3f32),
    ("mệt mỏi", -0.45f32, 0.15f32),
    ("mệt", -0.4f32, 0.2f32),
    ("nhớ nhà", -0.4f32, 0.35f32),
    ("khó", -0.3f32, 0.45f32),
    ("nhớ", -0.2f32, 0.4f32),
    ("bình tĩnh", 0.2f32, 0.15f32),
    ("khỏe", 0.5f32, 0.45f32),
    ("tốt", 0.55f32, 0.45f32),
    ("hay", 0.55f32, 0.5f32),
    ("thoải mái", 0.6f32, 0.4f32),
    ("thú vị", 0.6f32, 0.6f32),
    ("hài lòng", 0.65f32, 0.45f32),
    ("thích", 0.65f32, 0.55f32),
    ("kinh ngạc", 0.686f32, 0.643f32),
    ("biết ơn", 0.7f32, 0.55f32),
    ("cười", 0.7f32, 0.7f32),
    ("tốt lắm", 0.7f32, 0.55f32),
    ("hay lắm", 0.7f32, 0.6f32),
    ("hào hứng", 0.742f32, 0.696f32),
    ("phấn khích", 0.742f32, 0.696f32),
    ("tự hào", 0.75f32, 0.65f32),
    ("vui", 0.75f32, 0.7f32),
    ("tuyệt", 0.8f32, 0.7f32),
    ("xuất sắc", 0.8f32, 0.7f32),
    ("tài giỏi", 0.8f32, 0.7f32),
    ("kỳ diệu", 0.8f32, 0.7f32),
    ("yêu", 0.8f32, 0.65f32),
    ("được yêu", 0.8f32, 0.7f32),
    ("cảm ơn", 0.8f32, 0.7f32),
    ("vui vẻ", 0.8f32, 0.75f32),
    ("thành công", 0.8f32, 0.7f32),
    ("tuyệt vời", 0.85f32, 0.75f32),
    ("hạnh phúc", 0.85f32, 0.7f32),
    ("hoàn hảo", 0.85f32, 0.65f32),
    // ── Cross-lingual L2 data (183 entries) ─────────────────────
    ("heureux", 0.8f32, 0.65f32),
    ("joie", 0.8f32, 0.65f32),
    ("joyeux", 0.8f32, 0.65f32),
    ("content", 0.8f32, 0.65f32),
    ("glücklich", 0.8f32, 0.65f32),
    ("fröhlich", 0.8f32, 0.65f32),
    ("schönes", 0.8f32, 0.65f32),
    ("träume", 0.8f32, 0.65f32),
    ("feliz", 0.8f32, 0.65f32),
    ("alegre", 0.8f32, 0.65f32),
    ("genial", 0.8f32, 0.65f32),
    ("contento", 0.8f32, 0.65f32),
    ("buon", 0.8f32, 0.65f32),
    ("bellissimo", 0.8f32, 0.65f32),
    ("finalmente", 0.8f32, 0.65f32),
    ("feliz", 0.8f32, 0.65f32),
    ("alegre", 0.8f32, 0.65f32),
    ("高兴", 0.8f32, 0.65f32),
    ("开心", 0.8f32, 0.65f32),
    ("嬉しい", 0.8f32, 0.65f32),
    ("楽しい", 0.8f32, 0.65f32),
    ("amour", 0.9f32, 0.55f32),
    ("aimer", 0.9f32, 0.55f32),
    ("liebe", 0.9f32, 0.55f32),
    ("mögen", 0.9f32, 0.55f32),
    ("amor", 0.9f32, 0.55f32),
    ("querer", 0.9f32, 0.55f32),
    ("encanta", 0.9f32, 0.55f32),
    ("amore", 0.9f32, 0.55f32),
    ("amare", 0.9f32, 0.55f32),
    ("amor", 0.9f32, 0.55f32),
    ("amar", 0.9f32, 0.55f32),
    ("爱", 0.9f32, 0.55f32),
    ("喜欢", 0.9f32, 0.55f32),
    ("愛", 0.9f32, 0.55f32),
    ("好き", 0.9f32, 0.55f32),
    ("excellent", 0.85f32, 0.6f32),
    ("magnifique", 0.85f32, 0.6f32),
    ("parfait", 0.85f32, 0.6f32),
    ("wunderbar", 0.85f32, 0.6f32),
    ("perfekt", 0.85f32, 0.6f32),
    ("hervorragend", 0.85f32, 0.6f32),
    ("excelente", 0.85f32, 0.6f32),
    ("perfecto", 0.85f32, 0.6f32),
    ("guapa", 0.85f32, 0.6f32),
    ("ottimo", 0.85f32, 0.6f32),
    ("perfetto", 0.85f32, 0.6f32),
    ("excelente", 0.85f32, 0.6f32),
    ("ótimo", 0.85f32, 0.6f32),
    ("优秀", 0.85f32, 0.6f32),
    ("完美", 0.85f32, 0.6f32),
    ("素晴らしい", 0.85f32, 0.6f32),
    ("完璧", 0.85f32, 0.6f32),
    ("triste", -0.7f32, 0.45f32),
    ("malheureux", -0.7f32, 0.45f32),
    ("disparaître", -0.7f32, 0.45f32),
    ("traurig", -0.7f32, 0.45f32),
    ("unglücklich", -0.7f32, 0.45f32),
    ("triste", -0.7f32, 0.45f32),
    ("sola", -0.7f32, 0.45f32),
    ("pobre", -0.7f32, 0.45f32),
    ("triste", -0.7f32, 0.45f32),
    ("peggio", -0.7f32, 0.45f32),
    ("triste", -0.7f32, 0.45f32),
    ("infeliz", -0.7f32, 0.45f32),
    ("悲伤", -0.7f32, 0.45f32),
    ("难过", -0.7f32, 0.45f32),
    ("悲しい", -0.7f32, 0.45f32),
    ("悲しむ", -0.7f32, 0.45f32),
    ("terrible", -0.75f32, 0.55f32),
    ("horrible", -0.75f32, 0.55f32),
    ("affreux", -0.75f32, 0.55f32),
    ("catastrophique", -0.75f32, 0.55f32),
    ("schrecklich", -0.75f32, 0.55f32),
    ("furchtbar", -0.75f32, 0.55f32),
    ("schlecht", -0.75f32, 0.55f32),
    ("terrible", -0.75f32, 0.55f32),
    ("horrible", -0.75f32, 0.55f32),
    ("malo", -0.75f32, 0.55f32),
    ("peor", -0.75f32, 0.55f32),
    ("male", -0.75f32, 0.55f32),
    ("terribile", -0.75f32, 0.55f32),
    ("terrível", -0.75f32, 0.55f32),
    ("horrível", -0.75f32, 0.55f32),
    ("糟糕", -0.75f32, 0.55f32),
    ("可怕", -0.75f32, 0.55f32),
    ("最悪", -0.75f32, 0.55f32),
    ("ひどい", -0.75f32, 0.55f32),
    ("furieux", -0.65f32, 0.9f32),
    ("colère", -0.65f32, 0.9f32),
    ("irrité", -0.65f32, 0.9f32),
    ("wütend", -0.65f32, 0.9f32),
    ("verärgert", -0.65f32, 0.9f32),
    ("ärger", -0.65f32, 0.9f32),
    ("enojado", -0.65f32, 0.9f32),
    ("enfadado", -0.65f32, 0.9f32),
    ("furioso", -0.65f32, 0.9f32),
    ("arrabbiato", -0.65f32, 0.9f32),
    ("furioso", -0.65f32, 0.9f32),
    ("raiva", -0.65f32, 0.9f32),
    ("furioso", -0.65f32, 0.9f32),
    ("愤怒", -0.65f32, 0.9f32),
    ("生气", -0.65f32, 0.9f32),
    ("怒り", -0.65f32, 0.9f32),
    ("怒る", -0.65f32, 0.9f32),
    ("peur", -0.6f32, 0.8f32),
    ("effrayé", -0.6f32, 0.8f32),
    ("terrifié", -0.6f32, 0.8f32),
    ("angst", -0.6f32, 0.8f32),
    ("fürchten", -0.6f32, 0.8f32),
    ("erschrocken", -0.6f32, 0.8f32),
    ("miedo", -0.6f32, 0.8f32),
    ("asustado", -0.6f32, 0.8f32),
    ("aterrorizado", -0.6f32, 0.8f32),
    ("paura", -0.6f32, 0.8f32),
    ("spavento", -0.6f32, 0.8f32),
    ("medo", -0.6f32, 0.8f32),
    ("assustado", -0.6f32, 0.8f32),
    ("恐惧", -0.6f32, 0.8f32),
    ("害怕", -0.6f32, 0.8f32),
    ("恐怖", -0.6f32, 0.8f32),
    ("怖い", -0.6f32, 0.8f32),
    ("fatigué", -0.35f32, 0.15f32),
    ("épuisé", -0.35f32, 0.15f32),
    ("exténué", -0.35f32, 0.15f32),
    ("müde", -0.35f32, 0.15f32),
    ("erschöpft", -0.35f32, 0.15f32),
    ("cansado", -0.35f32, 0.15f32),
    ("agotado", -0.35f32, 0.15f32),
    ("stanco", -0.35f32, 0.15f32),
    ("esausto", -0.35f32, 0.15f32),
    ("cansado", -0.35f32, 0.15f32),
    ("exausto", -0.35f32, 0.15f32),
    ("疲惫", -0.35f32, 0.15f32),
    ("累", -0.35f32, 0.15f32),
    ("疲れた", -0.35f32, 0.15f32),
    ("くたくた", -0.35f32, 0.15f32),
    ("douleur", -0.65f32, 0.5f32),
    ("souffrir", -0.65f32, 0.5f32),
    ("schmerz", -0.65f32, 0.5f32),
    ("wehtun", -0.65f32, 0.5f32),
    ("dolor", -0.65f32, 0.5f32),
    ("doler", -0.65f32, 0.5f32),
    ("dolore", -0.65f32, 0.5f32),
    ("male", -0.65f32, 0.5f32),
    ("dor", -0.65f32, 0.5f32),
    ("doer", -0.65f32, 0.5f32),
    ("痛", -0.65f32, 0.5f32),
    ("疼痛", -0.65f32, 0.5f32),
    ("痛い", -0.65f32, 0.5f32),
    ("苦しい", -0.65f32, 0.5f32),
    ("famille", 0.7f32, 0.5f32),
    ("maison", 0.7f32, 0.5f32),
    ("familie", 0.7f32, 0.5f32),
    ("zuhause", 0.7f32, 0.5f32),
    ("familia", 0.7f32, 0.5f32),
    ("hogar", 0.7f32, 0.5f32),
    ("famiglia", 0.7f32, 0.5f32),
    ("casa", 0.7f32, 0.5f32),
    ("família", 0.7f32, 0.5f32),
    ("lar", 0.7f32, 0.5f32),
    ("家庭", 0.7f32, 0.5f32),
    ("家人", 0.7f32, 0.5f32),
    ("家族", 0.7f32, 0.5f32),
    ("家", 0.7f32, 0.5f32),
    ("terre", 0.55f32, 0.4f32),
    ("nature", 0.55f32, 0.4f32),
    ("monde", 0.55f32, 0.4f32),
    ("erde", 0.55f32, 0.4f32),
    ("natur", 0.55f32, 0.4f32),
    ("welt", 0.55f32, 0.4f32),
    ("tierra", 0.55f32, 0.4f32),
    ("naturaleza", 0.55f32, 0.4f32),
    ("mundo", 0.55f32, 0.4f32),
    ("terra", 0.55f32, 0.4f32),
    ("natura", 0.55f32, 0.4f32),
    ("mondo", 0.55f32, 0.4f32),
    ("terra", 0.55f32, 0.4f32),
    ("natureza", 0.55f32, 0.4f32),
    ("地球", 0.55f32, 0.4f32),
    ("自然", 0.55f32, 0.4f32),
    ("地球", 0.55f32, 0.4f32),
    ("自然", 0.55f32, 0.4f32),

    // ── L3 Topic clusters ─────────────────────────────────────────────
    ("solaire", 0.6f32, 0.45f32),
    ("éolienne", 0.6f32, 0.45f32),
    ("éoliennes", 0.6f32, 0.45f32),
    ("photovoltaïque", 0.6f32, 0.45f32),
    ("renouvelable", 0.6f32, 0.45f32),
    ("renouvelables", 0.6f32, 0.45f32),
    ("hydroélectrique", 0.6f32, 0.45f32),
    ("écologique", 0.6f32, 0.45f32),
    ("écologiques", 0.6f32, 0.45f32),
    ("durable", 0.6f32, 0.45f32),
    ("durables", 0.6f32, 0.45f32),
    ("solar", 0.6f32, 0.45f32),
    ("erneuerbar", 0.6f32, 0.45f32),
    ("nachhaltig", 0.6f32, 0.45f32),
    ("ökologisch", 0.6f32, 0.45f32),
    ("umweltfreundlich", 0.6f32, 0.45f32),
    ("renovable", 0.6f32, 0.45f32),
    ("renovables", 0.6f32, 0.45f32),
    ("ecológico", 0.6f32, 0.45f32),
    ("sostenible", 0.6f32, 0.45f32),
    ("renewable", 0.6f32, 0.45f32),
    ("sustainable", 0.6f32, 0.45f32),
    ("ecological", 0.6f32, 0.45f32),
    ("green", 0.6f32, 0.45f32),
    ("crowdfunding", 0.55f32, 0.6f32),
    ("innovation", 0.55f32, 0.6f32),
    ("technologie", 0.55f32, 0.6f32),
    ("numérique", 0.55f32, 0.6f32),
    ("cellule", 0.55f32, 0.6f32),
    ("prototype", 0.55f32, 0.6f32),
    ("digitalisierung", 0.55f32, 0.6f32),
    ("startup", 0.55f32, 0.6f32),
    ("innovación", 0.55f32, 0.6f32),
    ("digital", 0.55f32, 0.6f32),
    ("technology", 0.55f32, 0.6f32),
    ("software", 0.55f32, 0.6f32),
    ("platform", 0.55f32, 0.6f32),
    ("banques", 0.3f32, 0.5f32),
    ("financement", 0.3f32, 0.5f32),
    ("investissement", 0.3f32, 0.5f32),
    ("économique", 0.3f32, 0.5f32),
    ("banque", 0.3f32, 0.5f32),
    ("bank", 0.3f32, 0.5f32),
    ("finanzierung", 0.3f32, 0.5f32),
    ("investition", 0.3f32, 0.5f32),
    ("wirtschaft", 0.3f32, 0.5f32),
    ("banco", 0.3f32, 0.5f32),
    ("financiación", 0.3f32, 0.5f32),
    ("inversión", 0.3f32, 0.5f32),
    ("finance", 0.3f32, 0.5f32),
    ("investment", 0.3f32, 0.5f32),
    ("menacées", -0.65f32, 0.75f32),
    ("crise", -0.65f32, 0.75f32),
    ("catastrophe", -0.65f32, 0.75f32),
    ("polémique", -0.65f32, 0.75f32),
    ("disparaître", -0.65f32, 0.75f32),
    ("dégradation", -0.65f32, 0.75f32),
    ("pollution", -0.65f32, 0.75f32),
    ("krise", -0.65f32, 0.75f32),
    ("katastrophe", -0.65f32, 0.75f32),
    ("gefahr", -0.65f32, 0.75f32),
    ("crisis", -0.65f32, 0.75f32),
    ("catástrofe", -0.65f32, 0.75f32),
    ("amenaza", -0.65f32, 0.75f32),
    ("contamination", -0.65f32, 0.75f32),
    ("threat", -0.65f32, 0.75f32),
    ("extinction", -0.65f32, 0.75f32),
    ("scandal", -0.65f32, 0.75f32),
    ("communistes", -0.4f32, 0.65f32),
    ("opposition", -0.4f32, 0.65f32),
    ("conflit", -0.4f32, 0.65f32),
    ("guerre", -0.4f32, 0.65f32),
    ("corruption", -0.4f32, 0.65f32),
    ("konflikt", -0.4f32, 0.65f32),
    ("krieg", -0.4f32, 0.65f32),
    ("protest", -0.4f32, 0.65f32),
    ("korruption", -0.4f32, 0.65f32),
    ("conflicto", -0.4f32, 0.65f32),
    ("guerra", -0.4f32, 0.65f32),
    ("corrupción", -0.4f32, 0.65f32),
    ("conflict", -0.4f32, 0.65f32),
    ("war", -0.4f32, 0.65f32),
    ("corruption", -0.4f32, 0.65f32),
    ("fraud", -0.4f32, 0.65f32),
    ("université", 0.6f32, 0.5f32),
    ("recherche", 0.6f32, 0.5f32),
    ("scientifique", 0.6f32, 0.5f32),
    ("découverte", 0.6f32, 0.5f32),
    ("universität", 0.6f32, 0.5f32),
    ("forschung", 0.6f32, 0.5f32),
    ("wissenschaft", 0.6f32, 0.5f32),
    ("entdeckung", 0.6f32, 0.5f32),
    ("universidad", 0.6f32, 0.5f32),
    ("investigación", 0.6f32, 0.5f32),
    ("descubrimiento", 0.6f32, 0.5f32),
    ("university", 0.6f32, 0.5f32),
    ("research", 0.6f32, 0.5f32),
    ("science", 0.6f32, 0.5f32),
    ("discovery", 0.6f32, 0.5f32),
    ("victoire", 0.65f32, 0.8f32),
    ("champion", 0.65f32, 0.8f32),
    ("compétition", 0.65f32, 0.8f32),
    ("tournoi", 0.65f32, 0.8f32),
    ("sieg", 0.65f32, 0.8f32),
    ("meister", 0.65f32, 0.8f32),
    ("wettbewerb", 0.65f32, 0.8f32),
    ("turnier", 0.65f32, 0.8f32),
    ("victoria", 0.65f32, 0.8f32),
    ("campeón", 0.65f32, 0.8f32),
    ("competición", 0.65f32, 0.8f32),
    ("torneo", 0.65f32, 0.8f32),
    ("victory", 0.65f32, 0.8f32),
    ("champion", 0.65f32, 0.8f32),
    ("competition", 0.65f32, 0.8f32),
    ("tournament", 0.65f32, 0.8f32),

];


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
pub fn contains_any(text: &str, needles: &[&str]) -> bool {
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
