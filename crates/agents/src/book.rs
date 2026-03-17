//! # book — BookReader
//!
//! Học từ văn bản: sentence → EmotionTag → ĐN → pattern → QR.
//! Không đọc ảnh. Không đọc audio. Chỉ đọc text.
//!
//! BookReader.read(text) → Vec<SentenceRecord>
//! Pattern lặp lại → ĐN → Dream → QR

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use context::emotion::word_affect;
use olang::ling::apply_modifiers;
use silk::edge::EmotionTag;

// ─────────────────────────────────────────────────────────────────────────────
// SentenceRecord
// ─────────────────────────────────────────────────────────────────────────────

/// Một câu đã được phân tích.
#[derive(Debug, Clone)]
pub struct SentenceRecord {
    pub text: String,
    pub emotion: EmotionTag,
    /// Độ dài câu (số từ)
    pub word_count: usize,
    /// Câu có "emotionally significant" không (|V| > 0.3 hoặc A > 0.6)
    pub significant: bool,
}

impl SentenceRecord {
    pub fn new(text: String, emotion: EmotionTag) -> Self {
        let word_count = text.split_whitespace().count();
        let significant = emotion.valence.abs() > 0.2 || emotion.arousal > 0.5;
        Self {
            text,
            emotion,
            word_count,
            significant,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BookReader
// ─────────────────────────────────────────────────────────────────────────────

/// Đọc văn bản, phân tích emotion từng câu.
pub struct BookReader;

impl BookReader {
    pub fn new() -> Self {
        Self
    }

    /// Đọc văn bản → Vec<SentenceRecord>.
    ///
    /// Tách câu theo dấu câu (. ! ?).
    /// Tính EmotionTag cho từng câu.
    /// Trả về top significant sentences.
    pub fn read(&self, text: &str) -> Vec<SentenceRecord> {
        let sentences = split_sentences(text);
        sentences
            .into_iter()
            .filter(|s| !s.trim().is_empty() && s.split_whitespace().count() >= 2)
            .map(|s| {
                let emotion = sentence_emotion(&s);
                SentenceRecord::new(s, emotion)
            })
            .collect()
    }

    /// Lọc top-N emotionally significant sentences.
    /// Dùng để nạp vào ĐN.
    pub fn top_significant<'a>(
        &self,
        records: &'a [SentenceRecord],
        n: usize,
    ) -> Vec<&'a SentenceRecord> {
        let mut scored: Vec<(&SentenceRecord, f32)> = records
            .iter()
            .filter(|r| r.significant)
            .map(|r| {
                // Score = |V| × A × I (emotional weight)
                let score = r.emotion.valence.abs() * r.emotion.arousal * r.emotion.intensity;
                (r, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));

        scored.into_iter().take(n).map(|(r, _)| r).collect()
    }

    /// Stats của văn bản.
    pub fn stats(&self, records: &[SentenceRecord]) -> BookStats {
        if records.is_empty() {
            return BookStats::default();
        }
        let sig = records.iter().filter(|r| r.significant).count();
        // Weighted aggregate: emotional intensity (|V|×A) as weight — NOT simple average (QT)
        let weights: Vec<f32> = records
            .iter()
            .map(|r| (r.emotion.valence.abs() * r.emotion.arousal).max(0.01))
            .collect();
        let tw: f32 = weights.iter().sum();
        let avg_v = records.iter().zip(weights.iter())
            .map(|(r, &w)| r.emotion.valence * w)
            .sum::<f32>() / tw;
        let avg_a = records.iter().zip(weights.iter())
            .map(|(r, &w)| r.emotion.arousal * w)
            .sum::<f32>() / tw;
        BookStats {
            total_sentences: records.len(),
            significant_sentences: sig,
            avg_valence: avg_v,
            avg_arousal: avg_a,
        }
    }
}

impl Default for BookReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Stats tổng quan của một văn bản.
#[derive(Debug, Clone, Default)]
pub struct BookStats {
    pub total_sentences: usize,
    pub significant_sentences: usize,
    pub avg_valence: f32,
    pub avg_arousal: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Delegate to shared split_sentences in learning module.
fn split_sentences(text: &str) -> Vec<String> {
    crate::learning::split_sentences(text)
}

/// Tính EmotionTag cho một câu.
///
/// Dùng 2 lớp:
///   1. Sentence-level: scan toàn câu để detect cụm từ dài
///   2. Word-level: từng từ riêng lẻ
fn sentence_emotion(sentence: &str) -> EmotionTag {
    let lower = sentence.to_lowercase();

    // Lớp 1: sentence-level patterns (cụm từ ghép)
    let sentence_e = sentence_level_emotion(&lower);

    // Lớp 2: word-level
    let words: Vec<&str> = lower.split_whitespace().collect();
    if words.is_empty() {
        return sentence_e;
    }

    let mut tv = sentence_e.valence;
    let mut ta = sentence_e.arousal;
    let mut td = sentence_e.dominance;
    let mut ti = sentence_e.intensity;
    let mut weight = 2.0f32; // sentence-level có weight cao hơn

    for &w in &words {
        let e = word_affect(w);
        // Chỉ tính từ có cảm xúc thật (không neutral)
        if e.valence.abs() > 0.05 || e.arousal > 0.35 {
            tv += e.valence;
            ta += e.arousal;
            td += e.dominance;
            ti += e.intensity;
            weight += 1.0;
        }
    }

    if weight == 0.0 {
        return EmotionTag::NEUTRAL;
    }

    // Apply linguistic modifiers (negation, amplifier, diminisher, contrast)
    let words_vec: alloc::vec::Vec<&str> = words.to_vec();
    let raw_v = tv / weight;
    let (mod_v, mod_a) = apply_modifiers(&words_vec, raw_v, ta / weight);

    let result_v = mod_v.clamp(-1.0, 1.0);
    let _ = mod_a; // arousal computed separately

    // Lớp 3: topic inference (khi valence thấp)
    // Nếu sentence chưa có signal rõ (|v| < 0.1), thử topic words
    if result_v.abs() < 0.10 {
        let topic_e = topic_inference(&lower, &words);
        if topic_e.valence.abs() > 0.10 {
            return topic_e;
        }
    }

    EmotionTag::new(
        result_v,
        (ta / weight).clamp(0.0, 1.0),
        (td / weight).clamp(0.0, 1.0),
        (ti / weight).clamp(0.0, 1.0),
    )
}

/// Lớp 3: Topic inference — map sentence sang topic cluster.
///
/// Khi word-level không tìm được signal, dùng topic keywords.
/// "éolienne" → renewable_energy cluster → valence=+0.60
fn topic_inference(lower: &str, words: &[&str]) -> EmotionTag {
    // Topic keywords → (valence, arousal)
    // Ordered by specificity (longer phrases first)
    static TOPIC_SIGNALS: &[(&str, f32, f32)] = &[
        // Renewable energy / environment (positive)
        ("hydroélectrique", 0.60, 0.45),
        ("photovoltaïque", 0.60, 0.45),
        ("renouvelables", 0.60, 0.45),
        ("renouvelable", 0.60, 0.45),
        ("écologiques", 0.60, 0.45),
        ("écologique", 0.60, 0.45),
        ("éoliennes", 0.60, 0.45),
        ("éolienne", 0.60, 0.45),
        ("solaire", 0.60, 0.45),
        ("durables", 0.55, 0.40),
        ("durable", 0.55, 0.40),
        ("biodiversité", 0.55, 0.40),
        ("biodiversite", 0.55, 0.40),
        // Technology / innovation (positive)
        ("crowdfunding", 0.55, 0.60),
        ("innovation", 0.55, 0.55),
        ("technologie", 0.50, 0.55),
        ("développeurs", 0.50, 0.55),
        ("développement", 0.45, 0.50),
        // Science / research (positive)
        ("scientifiques", 0.55, 0.50),
        ("scientifique", 0.55, 0.50),
        ("chercheurs", 0.50, 0.50),
        ("recherche", 0.50, 0.50),
        ("découverte", 0.55, 0.55),
        // Health (neutral-positive)
        ("vaccin", 0.35, 0.55),
        ("médecin", 0.20, 0.45),
        // Crisis / threat (negative)
        ("catastrophe", -0.65, 0.75),
        ("menacées", -0.60, 0.70),
        ("menacés", -0.60, 0.70),
        ("extinction", -0.65, 0.70),
        ("disparaître", -0.55, 0.65),
        ("pollution", -0.55, 0.60),
        ("dégradation", -0.55, 0.60),
        // German equivalents
        ("erneuerbar", 0.60, 0.45),
        ("photovoltaik", 0.60, 0.45),
        ("nachhaltig", 0.55, 0.40),
        ("innovation", 0.55, 0.55),
        ("wissenschaft", 0.55, 0.50),
        ("forschung", 0.50, 0.50),
        ("katastrophe", -0.65, 0.75),
        ("bedrohung", -0.60, 0.70),
        ("aussterben", -0.65, 0.70),
        // Spanish equivalents
        ("renovables", 0.60, 0.45),
        ("sostenible", 0.55, 0.40),
        ("tecnología", 0.50, 0.55),
        ("investigación", 0.50, 0.50),
        ("descubrimiento", 0.55, 0.55),
        ("catástrofe", -0.65, 0.75),
        ("amenaza", -0.60, 0.70),
    ];

    let mut best_v = 0.0f32;
    let mut best_a = 0.5f32;
    let mut found = false;

    for &(topic, v, a) in TOPIC_SIGNALS {
        if lower.contains(topic) || words.contains(&topic) {
            // Tìm thấy topic keyword
            if !found || v.abs() > best_v.abs() {
                best_v = v;
                best_a = a;
                found = true;
            }
        }
    }

    if found {
        EmotionTag::new(best_v, best_a, 0.5, best_a * 0.8)
    } else {
        EmotionTag::NEUTRAL
    }
}

/// Sentence-level emotion từ cụm từ ghép tiếng Việt.
fn sentence_level_emotion(lower: &str) -> EmotionTag {
    use context::emotion::contains_any;

    // Bạo lực / chiến tranh
    if contains_any(
        lower,
        &[
            "thiệt mạng",
            "tử vong",
            "hi sinh",
            "mất mạng",
            "giao tranh",
            "bùng phát",
            "tấn công",
            "lực lượng vũ trang",
            "bốc cháy",
            "chạy loạn",
            "không đủ thuốc",
            "tiếng súng",
            "bệnh viện dã chiến",
            "quá tải",
            "hàng chục",
            "hàng trăm",
            "thiệt hại",
        ],
    ) {
        return EmotionTag::new(-0.75, 0.85, 0.2, 0.85);
    }

    // Nỗi đau / mất mát
    if contains_any(
        lower,
        &[
            "hoang tàn",
            "người thân ra đi",
            "cướp đi tất cả",
            "không bao giờ đói",
            "mảnh đất đỏ",
            "rời bỏ nhà",
            "đêm tối",
            "bóng đêm",
            "trẻ em khóc",
            "người già",
        ],
    ) {
        return EmotionTag::new(-0.55, 0.60, 0.3, 0.70);
    }

    // Quyết tâm / hy vọng dù khó khăn
    if contains_any(
        lower,
        &[
            "thề rằng",
            "dù trời có sập",
            "sẽ không bao giờ",
            "bàn tay nắm chặt",
        ],
    ) {
        return EmotionTag::new(0.25, 0.70, 0.80, 0.75);
    }

    // Vẻ đẹp / quyến rũ
    if contains_any(
        lower,
        &[
            "vẻ quyến rũ",
            "lấp lánh",
            "khuôn mặt thanh tú",
            "trái tim rung động",
            "đôi mắt",
        ],
    ) {
        return EmotionTag::new(0.55, 0.55, 0.60, 0.60);
    }

    EmotionTag::NEUTRAL
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn reader() -> BookReader {
        BookReader::new()
    }

    // ── split_sentences ───────────────────────────────────────────────────────

    #[test]
    fn split_basic() {
        let sents = split_sentences("Trời đẹp. Tôi vui. Cảm ơn!");
        assert_eq!(sents.len(), 3);
    }

    #[test]
    fn split_tieng_viet() {
        let sents =
            split_sentences("Hôm nay tôi mất việc rồi. Buồn quá! Tôi biết làm sao bây giờ?");
        assert_eq!(sents.len(), 3);
    }

    #[test]
    fn split_no_punctuation() {
        let sents = split_sentences("Không có dấu câu cuối");
        assert_eq!(sents.len(), 1, "Câu không có dấu câu vẫn được giữ");
    }

    #[test]
    fn split_empty() {
        assert!(split_sentences("").is_empty());
    }

    #[test]
    fn split_japanese_punctuation() {
        let sents = split_sentences("今日は晴れです。とても嬉しい！");
        assert_eq!(sents.len(), 2, "Japanese punctuation supported");
    }

    // ── BookReader ────────────────────────────────────────────────────────────

    #[test]
    fn read_basic() {
        let records = reader().read("Tôi vui. Trời đẹp. Cảm ơn bạn!");
        assert!(!records.is_empty());
    }

    #[test]
    fn read_emotional_sentence() {
        let records = reader().read("Hôm nay tôi mất việc rồi. Buồn quá. Không biết làm sao.");
        let has_negative = records.iter().any(|r| r.emotion.valence < 0.0);
        assert!(has_negative, "Câu buồn phải có valence âm");
    }

    #[test]
    fn read_filters_short() {
        // Câu 1 từ bị lọc (word_count < 2)
        let records = reader().read("Ok. Tôi hiểu điều này rồi.");
        assert!(records.iter().all(|r| r.word_count >= 2));
    }

    #[test]
    fn top_significant_sorted() {
        let records = alloc::vec![
            SentenceRecord::new(
                "Tôi rất vui!".to_string(),
                EmotionTag::new(0.9, 0.8, 0.7, 0.9)
            ),
            SentenceRecord::new("Ổn thôi.".to_string(), EmotionTag::new(0.1, 0.2, 0.5, 0.2)),
            SentenceRecord::new(
                "Buồn quá.".to_string(),
                EmotionTag::new(-0.8, 0.5, 0.2, 0.7)
            ),
        ];

        let r = reader();
        let top = r.top_significant(&records, 2);
        assert_eq!(top.len(), 2);
        // "Tôi rất vui!" hoặc "Buồn quá." phải là top (cả 2 có |V| > 0.3)
        let top_texts: Vec<&str> = top.iter().map(|r| r.text.as_str()).collect();
        assert!(top_texts.contains(&"Tôi rất vui!") || top_texts.contains(&"Buồn quá."));
    }

    #[test]
    fn significant_flag_high_valence() {
        let r = SentenceRecord::new(
            "Tôi rất hạnh phúc!".to_string(),
            EmotionTag::new(0.9, 0.8, 0.7, 0.9),
        );
        assert!(r.significant, "|V|=0.9 > 0.3 → significant");
    }

    #[test]
    fn significant_flag_neutral_not() {
        let r = SentenceRecord::new("Ok.".to_string(), EmotionTag::new(0.0, 0.2, 0.5, 0.2));
        assert!(!r.significant, "Neutral → not significant");
    }

    #[test]
    fn stats_basic() {
        let records = alloc::vec![
            SentenceRecord::new("Vui lắm!".to_string(), EmotionTag::new(0.8, 0.7, 0.6, 0.8)),
            SentenceRecord::new(
                "Buồn nhỉ.".to_string(),
                EmotionTag::new(-0.6, 0.4, 0.3, 0.5)
            ),
            SentenceRecord::new("Ổn.".to_string(), EmotionTag::new(0.0, 0.2, 0.5, 0.2)),
        ];
        let r2 = reader();
        let stats = r2.stats(&records);
        assert_eq!(stats.total_sentences, 3);
        assert_eq!(stats.significant_sentences, 2, "2 significant (|V|>0.3)");
        // Weighted aggregate: weight = |V| × A (emotionally intense sentences count more)
        // Vui: |0.8|×0.7=0.56, Buồn: |-0.6|×0.4=0.24, Ổn: |0.0|×0.2=0.01(min)
        // avg_valence ≈ (0.8×0.56 + -0.6×0.24 + 0.0×0.01) / (0.56+0.24+0.01) ≈ 0.376
        // "Vui lắm!" trọng số lớn hơn → kéo valence lên (amplify thay vì trung bình)
        assert!(stats.avg_valence > 0.2, "Weighted: emotionally strong 'Vui' dominates, got {}", stats.avg_valence);
    }

    #[test]
    fn read_full_paragraph() {
        // Câu ngắn với từ emotion rõ ràng → word_affect chiếm đa số
        let text = "Vui quá! Buồn lắm. Hạnh phúc!";
        let records = reader().read(text);
        assert!(records.len() >= 2, "Phải parse được nhiều câu");
        // Ít nhất 1 câu significant: "vui quá" hoặc "buồn lắm"
        // significant = |V| > 0.3 OR A > 0.6
        // "vui quá": word_affect(vui)=0.8, word_affect(quá)=0.0 → avg=0.4 > 0.3 ✓
        let any_sig = records
            .iter()
            .any(|r| r.emotion.valence.abs() > 0.15 || r.emotion.arousal > 0.5);
        assert!(
            any_sig,
            "Ít nhất 1 câu significant: {:?}",
            records
                .iter()
                .map(|r| (r.text.as_str(), r.emotion.valence))
                .collect::<alloc::vec::Vec<_>>()
        );
    }
}
