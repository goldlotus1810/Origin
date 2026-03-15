//! # intent — IntentEstimate với full scoring
//!
//! Port từ Go `intent_verify.go`.
//! Thuật toán:
//!   1. Baseline: Learn = 0.25
//!   2. Mỗi keyword match → cộng score vào bucket
//!   3. Emotional amplifiers từ ConversationCurve (cur_v, cur_a)
//!   4. Winner = bucket có score cao nhất
//!   5. confidence = clamp(score, 0.35, 0.95)
//!   6. NeedClarify nếu !sensitive && conf < 0.55 && words ≤ 6

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use crate::emotion::IntentKind;

// ─────────────────────────────────────────────────────────────────────────────
// IntentEstimate
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả ước lượng intent.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct IntentEstimate {
    pub primary:      IntentKind,
    pub confidence:   f32,
    pub signals:      Vec<String>,
    pub need_clarify: bool,
    pub clarify_q:    String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Bucket
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Default, Clone)]
struct Bucket { score: f32, reasons: Vec<String> }

impl Bucket {
    fn add(&mut self, s: f32, r: &str) {
        self.score += s;
        self.reasons.push(r.to_string());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// estimate_intent
// ─────────────────────────────────────────────────────────────────────────────

/// Ước lượng intent từ text + emotional context (cur_v, cur_a).
pub fn estimate_intent(text: &str, cur_v: f32, cur_a: f32) -> IntentEstimate {
    let lo    = text.to_lowercase();
    let words = lo.split_whitespace().count();

    let mut buckets: [(IntentKind, Bucket); 12] = [
        (IntentKind::Learn,      Bucket::default()),
        (IntentKind::Inform,     Bucket::default()),
        (IntentKind::Research,   Bucket::default()),
        (IntentKind::Heal,       Bucket::default()),
        (IntentKind::Technical,  Bucket::default()),
        (IntentKind::Creative,   Bucket::default()),
        (IntentKind::Explore,    Bucket::default()),
        (IntentKind::Manipulate, Bucket::default()),
        (IntentKind::Risk,       Bucket::default()),
        (IntentKind::Crisis,     Bucket::default()),
        (IntentKind::Command,    Bucket::default()),
        (IntentKind::Chat,       Bucket::default()),
    ];

    macro_rules! add {
        ($kind:expr, $score:expr, $reason:expr) => {
            for (k, b) in buckets.iter_mut() {
                if *k == $kind { b.add($score, $reason); break; }
            }
        };
    }

    // Baseline
    add!(IntentKind::Learn, 0.25, "baseline");

    // ── Crisis (score cao nhất: 0.80 per keyword) ──────────────────
    for kw in &["tự tử","chết đi","không muốn sống","kết thúc tất cả",
                "không ai nhớ mình","biến mất mãi mãi","không còn đau nữa",
                "thuốc ngủ","nhảy xuống","treo cổ",
                "want to die","kill myself","end it all","suicide",
                "no reason to live","end my life"] {
        if lo.contains(kw) { add!(IntentKind::Crisis, 0.80, &format!("crisis:'{}'",kw)); }
    }
    // Mood amplifier: rất buồn + ít kích động = rút lui
    if cur_v < -0.70 && cur_a < 0.35 {
        add!(IntentKind::Crisis, 0.25, "mood: rất buồn + ít kích động");
    }

    // ── Risk ───────────────────────────────────────────────────────
    for kw in &["làm hại","trả thù","cho nó một bài học",
                "muốn nó biến mất","không ai phát hiện","xóa dấu vết",
                "harm someone","get revenge","hurt them","without getting caught"] {
        if lo.contains(kw) { add!(IntentKind::Risk, 0.60, &format!("risk:'{}'",kw)); }
    }
    // Mood amplifier: tức giận cao + kích động cao
    if cur_v < -0.50 && cur_a > 0.70 {
        add!(IntentKind::Risk, 0.20, "mood: tức giận + kích động cao");
    }

    // ── Manipulate ─────────────────────────────────────────────────
    for kw in &["làm người khác tin","thao túng","khiến người ta",
                "viết tin giả","tạo thông tin sai","giả vờ là","đóng giả","lừa dối",
                "manipulate","fake news","make them believe",
                "deceive","impersonate","spread misinformation"] {
        if lo.contains(kw) { add!(IntentKind::Manipulate, 0.65, &format!("manip:'{}'",kw)); }
    }

    // ── Heal ───────────────────────────────────────────────────────
    for kw in &["tôi buồn","tôi đau","không biết phải làm sao",
                "cô đơn","mất mát","chia tay","thất bại",
                "không ai hiểu","mệt mỏi quá",
                "i'm sad","i feel lost","heartbroken","lonely",
                "don't know what to do","exhausted"] {
        if lo.contains(kw) { add!(IntentKind::Heal, 0.50, &format!("heal:'{}'",kw)); }
    }
    if cur_v < -0.40 { add!(IntentKind::Heal, 0.15, "mood: buồn"); }

    // ── Learn ──────────────────────────────────────────────────────
    for kw in &["là gì","thế nào","tại sao","vì sao","giải thích",
                "cho tôi biết","nghĩa là gì","ví dụ","học cách",
                "what is","why ","how does","explain","definition"] {
        if lo.contains(kw) { add!(IntentKind::Learn, 0.30, &format!("learn:'{}'",kw)); }
    }

    // ── Research ───────────────────────────────────────────────────
    for kw in &["nghiên cứu","phân tích","so sánh","đánh giá",
                "tổng hợp","dữ liệu","bằng chứng",
                "research","analyze","compare","evaluate","evidence"] {
        if lo.contains(kw) { add!(IntentKind::Research, 0.40, &format!("research:'{}'",kw)); }
    }

    // ── Technical ──────────────────────────────────────────────────
    for kw in &["code","api","function","implement","bug","error",
                "compile","library","framework","algorithm","database","debug"] {
        if lo.contains(kw) { add!(IntentKind::Technical, 0.50, &format!("tech:'{}'",kw)); }
    }

    // ── Creative ───────────────────────────────────────────────────
    for kw in &["viết truyện","sáng tác","kịch bản","nhân vật","tiểu thuyết",
                "thơ","truyện ngắn",
                "write a story","fiction","poem","screenplay","creative writing"] {
        if lo.contains(kw) { add!(IntentKind::Creative, 0.45, &format!("creative:'{}'",kw)); }
    }

    // ── Inform ─────────────────────────────────────────────────────
    for kw in &["bài báo","viết bài","báo cáo","thuyết trình",
                "write an article","report","presentation","blog post"] {
        if lo.contains(kw) { add!(IntentKind::Inform, 0.40, &format!("inform:'{}'",kw)); }
    }

    // ── Command ────────────────────────────────────────────────────
    for kw in &["tắt đèn","bật đèn","mở đèn","điều chỉnh","đặt nhiệt độ",
                "turn off","turn on","set temperature","play music"] {
        if lo.contains(kw) { add!(IntentKind::Command, 0.55, &format!("cmd:'{}'",kw)); }
    }

    // ── Winner ─────────────────────────────────────────────────────
    let (best_kind, best_bucket) = buckets.iter()
        .max_by(|a, b| a.1.score.partial_cmp(&b.1.score).unwrap())
        .map(|(k, b)| (*k, b.clone()))
        .unwrap();

    let confidence = best_bucket.score.clamp(0.35, 0.95);

    let need_clarify = !best_kind.is_sensitive() && confidence < 0.55 && words <= 6;
    let clarify_q = if need_clarify { clarify_question(best_kind, cur_v) } else { String::new() };

    IntentEstimate {
        primary: best_kind, confidence,
        signals: best_bucket.reasons, need_clarify, clarify_q,
    }
}

fn clarify_question(kind: IntentKind, cur_v: f32) -> String {
    match kind {
        IntentKind::Learn =>
            "Bạn đang tìm hiểu cho mục đích gì — học, công việc, hay tò mò thôi?".into(),
        IntentKind::Research =>
            "Bạn đang nghiên cứu theo hướng nào — học thuật hay thực tế?".into(),
        IntentKind::Creative =>
            "Đây là cho dự án sáng tác hay bạn đang thử nghĩ?".into(),
        IntentKind::Technical =>
            "Bạn đang dùng ngôn ngữ/framework gì vậy?".into(),
        _ => if cur_v < -0.30 {
            "Bạn đang ổn không? Hỏi vì mình cảm giác có chuyện gì đó.".into()
        } else {
            "Bạn đang tìm thứ này để dùng trong tình huống cụ thể nào không?".into()
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// modify_response
// ─────────────────────────────────────────────────────────────────────────────

/// Điều chỉnh response theo intent.
///
/// Thuật toán từ Go ModifyResponse():
///   Crisis    → override + hotline 1800 599 920
///   Risk      → hỏi context (nếu cur_v < -0.5: đồng cảm trước)
///   Manipulate→ từ chối nhẹ
///   Heal      → "Mình nghe bạn." + original
///   Default   → original [+ clarify nếu cần]
pub fn modify_response(original: &str, est: &IntentEstimate, cur_v: f32) -> String {
    match est.primary {
        IntentKind::Crisis =>
            "Mình đọc được điều bạn vừa nói — và mình muốn hỏi thẳng: \
             bạn có đang nghĩ đến việc tự làm hại bản thân không?\n\n\
             Không cần trả lời ngay nếu chưa muốn. Mình ở đây.\n\n\
             Nếu bạn đang ở Việt Nam và cần nói chuyện với ai ngay bây giờ: \
             đường dây hỗ trợ sức khỏe tâm thần 1800 599 920 (miễn phí, 24/7).".into(),

        IntentKind::Risk => if cur_v < -0.50 {
            "Mình thấy bạn đang có cảm xúc rất mạnh lúc này. \
             Kể cho mình nghe chuyện gì đang xảy ra được không? \
             Mình muốn hiểu trước khi nói thêm.".into()
        } else {
            "Câu hỏi này mình cần hiểu rõ hơn bối cảnh. \
             Bạn đang nghĩ đến tình huống cụ thể nào vậy?".into()
        },

        IntentKind::Manipulate =>
            "Cái này mình không làm được — không phải vì quy tắc, \
             mà vì mình không muốn trở thành công cụ để ảnh hưởng không tốt đến người khác. \
             Bạn có muốn nói về điều gì đang khiến bạn nghĩ đến hướng này không?".into(),

        IntentKind::Heal => if original.is_empty() {
            "Mình đang ở đây và đang lắng nghe. Kể cho mình nghe thêm đi.".into()
        } else {
            format!("Mình nghe bạn. {}", original)
        },

        _ => if est.need_clarify && !est.clarify_q.is_empty() {
            format!("{}\n\n{}", original, est.clarify_q)
        } else {
            original.into()
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crisis_keyword() {
        let e = estimate_intent("không muốn sống nữa mệt mỏi quá", 0.0, 0.5);
        assert_eq!(e.primary, IntentKind::Crisis);
        assert!(e.confidence > 0.7);
    }

    #[test]
    fn crisis_mood_amplifier() {
        let e = estimate_intent("tôi mệt lắm", -0.75, 0.25);
        assert_eq!(e.primary, IntentKind::Crisis);
    }

    #[test]
    fn risk_keyword() {
        let e = estimate_intent("làm hại ai đó không ai phát hiện", 0.0, 0.5);
        assert_eq!(e.primary, IntentKind::Risk);
    }

    #[test]
    fn risk_mood_amplifier() {
        let e = estimate_intent("tôi muốn trả thù hắn", -0.60, 0.80);
        assert_eq!(e.primary, IntentKind::Risk);
    }

    #[test]
    fn manipulate_keyword() {
        let e = estimate_intent("làm người khác tin vào thông tin sai", 0.0, 0.5);
        assert_eq!(e.primary, IntentKind::Manipulate);
    }

    #[test]
    fn heal_keyword() {
        let e = estimate_intent("tôi buồn quá không biết phải làm sao", -0.5, 0.4);
        assert_eq!(e.primary, IntentKind::Heal);
    }

    #[test]
    fn learn_keyword() {
        let e = estimate_intent("photosynthesis là gì?", 0.2, 0.45);
        assert_eq!(e.primary, IntentKind::Learn);
    }

    #[test]
    fn technical_keyword() {
        let e = estimate_intent("implement binary search tree golang", 0.3, 0.5);
        assert_eq!(e.primary, IntentKind::Technical);
    }

    #[test]
    fn creative_keyword() {
        let e = estimate_intent("viết truyện ngắn về tình bạn", 0.4, 0.55);
        assert_eq!(e.primary, IntentKind::Creative);
    }

    #[test]
    fn research_keyword() {
        let e = estimate_intent("phân tích dữ liệu kinh tế 2024", 0.25, 0.55);
        assert_eq!(e.primary, IntentKind::Research);
    }

    #[test]
    fn confidence_range() {
        let e = estimate_intent("hello", 0.0, 0.5);
        assert!(e.confidence >= 0.35 && e.confidence <= 0.95);
    }

    #[test]
    fn sensitive_no_clarify() {
        let e = estimate_intent("không muốn sống nữa", -0.8, 0.25);
        assert!(!e.need_clarify, "Crisis không hỏi clarify");
    }

    #[test]
    fn signals_present() {
        let e = estimate_intent("tôi buồn quá", -0.5, 0.4);
        assert!(!e.signals.is_empty());
    }

    // modify_response tests
    #[test]
    fn crisis_has_hotline() {
        let e = estimate_intent("không muốn sống nữa", -0.8, 0.25);
        let r = modify_response("info về thuốc", &e, -0.8);
        assert!(r.contains("1800"));
        assert!(!r.contains("thuốc"), "Không trả lời câu hỏi gốc");
    }

    #[test]
    fn risk_no_original() {
        let e = estimate_intent("làm hại ai đó", 0.0, 0.5);
        let r = modify_response("hướng dẫn chi tiết", &e, -0.3);
        assert!(!r.contains("hướng dẫn chi tiết"));
    }

    #[test]
    fn manipulate_soft_refusal() {
        let e = estimate_intent("làm người khác tin thông tin sai", 0.0, 0.5);
        let r = modify_response("original", &e, 0.0);
        assert!(!r.contains("original"));
        assert!(r.contains("không làm được") || r.contains("không muốn"));
    }

    #[test]
    fn heal_empathy_first() {
        let e = estimate_intent("tôi buồn quá", -0.5, 0.4);
        let r = modify_response("đây là lời khuyên", &e, -0.5);
        assert!(r.starts_with("Mình nghe") || r.starts_with("Mình đang"));
    }

    #[test]
    fn learn_unchanged() {
        let e = estimate_intent("photosynthesis là gì?", 0.2, 0.45);
        let original = "Quang hợp là quá trình...";
        let r = modify_response(original, &e, 0.2);
        assert!(r.contains(original));
    }
}
