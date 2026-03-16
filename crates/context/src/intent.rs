//! # intent — IntentEstimate với full scoring
//!
//! Logic thuần: không hardcode response strings, không hardcode magic numbers.
//!
//! Thuật toán:
//!   1. Score accumulation: mỗi signal match → cộng vào bucket
//!   2. Emotional amplifiers từ cur_v / cur_a
//!   3. Winner = max(bucket)
//!   4. Confidence = clamp(score, MIN_CONF, MAX_CONF)
//!   5. NeedClarify = !sensitive && conf < CLARIFY_THRESHOLD && words ≤ SHORT_SENTENCE
//!
//! Không có response text ở đây — chỉ có phân loại và confidence.
//! Response text thuộc về layer trên (runtime/response.rs).

extern crate alloc;
use crate::emotion::IntentKind;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// Tuning constants — không magic, có tên rõ ràng
// ─────────────────────────────────────────────────────────────────────────────

/// Score tối thiểu để xem là confident.
const MIN_CONF: f32 = 0.35;
/// Score tối đa.
const MAX_CONF: f32 = 0.95;
/// Ngưỡng để hỏi thêm (clarify).
const CLARIFY_THRESHOLD: f32 = 0.55;
/// Câu ngắn → có thể cần clarify.
const SHORT_SENTENCE: usize = 6;

/// Score khi match crisis keyword.
const SCORE_CRISIS_KW: f32 = 0.80;
/// Mood amplifier crisis (rút lui).
const SCORE_CRISIS_MOOD: f32 = 0.25;
/// Score khi match risk keyword.
const SCORE_RISK_KW: f32 = 0.60;
/// Mood amplifier risk (tức giận cao).
const SCORE_RISK_MOOD: f32 = 0.20;
/// Score khi match manipulate keyword.
const SCORE_MANIP_KW: f32 = 0.65;
/// Score khi match heal keyword.
const SCORE_HEAL_KW: f32 = 0.50;
/// Mood amplifier heal (buồn).
const SCORE_HEAL_MOOD: f32 = 0.15;
/// Score baseline learn.
const SCORE_LEARN_BASE: f32 = 0.25;
/// Score khi match learn keyword.
const SCORE_LEARN_KW: f32 = 0.30;
/// Score khi match research/tech/creative/inform keyword.
const SCORE_MEDIUM_KW: f32 = 0.40;
const SCORE_HIGH_KW: f32 = 0.50;
/// Score command.
const SCORE_CMD_KW: f32 = 0.55;

/// Ngưỡng valence "rất buồn" → crisis amplifier.
const V_CRISIS_LOW: f32 = -0.70;
/// Ngưỡng arousal "ít kích động" → rút lui.
const A_CRISIS_QUIET: f32 = 0.35;
/// Ngưỡng valence "tức giận".
const V_RISK_ANGRY: f32 = -0.50;
/// Ngưỡng arousal "kích động cao".
const A_RISK_HIGH: f32 = 0.70;
/// Ngưỡng valence "buồn" → heal.
const V_HEAL_SAD: f32 = -0.40;

// ─────────────────────────────────────────────────────────────────────────────
// Keyword tables — data, không phải logic
// ─────────────────────────────────────────────────────────────────────────────

static KW_CRISIS: &[&str] = &[
    // VI
    "tự tử",
    "chết đi",
    "không muốn sống",
    "kết thúc tất cả",
    "không ai nhớ mình",
    "biến mất mãi mãi",
    "không còn đau nữa",
    "thuốc ngủ",
    "nhảy xuống",
    "treo cổ",
    // EN
    "want to die",
    "kill myself",
    "end it all",
    "suicide",
    "no reason to live",
    "end my life",
];

static KW_RISK: &[&str] = &[
    // VI
    "làm hại",
    "trả thù",
    "cho nó một bài học",
    "muốn nó biến mất",
    "không ai phát hiện",
    "xóa dấu vết",
    // EN
    "harm someone",
    "get revenge",
    "hurt them",
    "without getting caught",
];

static KW_MANIPULATE: &[&str] = &[
    // VI
    "làm người khác tin",
    "thao túng",
    "khiến người ta",
    "viết tin giả",
    "tạo thông tin sai",
    "giả vờ là",
    "đóng giả",
    "lừa dối",
    // EN
    "manipulate",
    "fake news",
    "make them believe",
    "deceive",
    "impersonate",
    "spread misinformation",
];

static KW_HEAL: &[&str] = &[
    // VI
    "tôi buồn",
    "tôi đau",
    "không biết phải làm sao",
    "cô đơn",
    "mất mát",
    "chia tay",
    "thất bại",
    "không ai hiểu",
    "mệt mỏi quá",
    // EN
    "i'm sad",
    "i feel lost",
    "heartbroken",
    "lonely",
    "don't know what to do",
    "exhausted",
];

static KW_LEARN: &[&str] = &[
    // VI
    "là gì",
    "thế nào",
    "tại sao",
    "vì sao",
    "giải thích",
    "cho tôi biết",
    "nghĩa là gì",
    "ví dụ",
    "học cách",
    // EN
    "what is",
    "why ",
    "how does",
    "explain",
    "definition",
];

static KW_RESEARCH: &[&str] = &[
    // VI
    "nghiên cứu",
    "phân tích",
    "so sánh",
    "đánh giá",
    "tổng hợp",
    "dữ liệu",
    "bằng chứng",
    // EN
    "research",
    "analyze",
    "compare",
    "evaluate",
    "evidence",
];

static KW_TECHNICAL: &[&str] = &[
    "code",
    "api",
    "function",
    "implement",
    "bug",
    "error",
    "compile",
    "library",
    "framework",
    "algorithm",
    "database",
    "debug",
];

static KW_CREATIVE: &[&str] = &[
    // VI
    "viết truyện",
    "sáng tác",
    "kịch bản",
    "nhân vật",
    "tiểu thuyết",
    "thơ",
    // EN
    "write a story",
    "fiction",
    "poem",
    "screenplay",
    "creative writing",
];

static KW_INFORM: &[&str] = &[
    // VI
    "bài báo",
    "viết bài",
    "báo cáo",
    "thuyết trình",
    // EN
    "write an article",
    "report",
    "presentation",
    "blog post",
];

static KW_COMMAND: &[&str] = &[
    // VI
    "tắt đèn",
    "bật đèn",
    "mở đèn",
    "điều chỉnh",
    "đặt nhiệt độ",
    // EN
    "turn off",
    "turn on",
    "set temperature",
    "play music",
];

static KW_CONFIRM: &[&str] = &[
    // VI
    "đồng ý",
    "có",
    "ừ",
    "ok",
    "được",
    "vâng",
    "chấp nhận",
    "duyệt",
    "đúng rồi",
    "phê duyệt",
    "cho phép",
    // EN
    "yes",
    "yeah",
    "yep",
    "sure",
    "approve",
    "accept",
    "confirm",
    "agreed",
    "go ahead",
    "do it",
    "ok",
    "okay",
];

static KW_DENY: &[&str] = &[
    // VI
    "không",
    "từ chối",
    "không đồng ý",
    "hủy",
    "không cho",
    "thôi",
    "đừng",
    "không được",
    "bỏ qua",
    "skip",
    "bỏ",
    // EN
    "no",
    "nope",
    "deny",
    "reject",
    "cancel",
    "refuse",
    "don't",
    "stop",
    "skip",
    "decline",
    "never",
];

// ─────────────────────────────────────────────────────────────────────────────
// IntentEstimate
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả ước lượng intent — không chứa response text.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct IntentEstimate {
    pub primary: IntentKind,
    pub confidence: f32,
    pub signals: Vec<String>, // lý do (debug)
    pub need_clarify: bool,
    pub clarify_kind: Option<ClarifyKind>,
}

/// Loại câu hỏi cần làm rõ — caller quyết định text cụ thể.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClarifyKind {
    WhatPurpose,   // "dùng để làm gì?"
    WhatDirection, // "hướng nào?"
    WhatContext,   // "tình huống cụ thể?"
    CheckingIn,    // "bạn đang ổn không?"
}

// ─────────────────────────────────────────────────────────────────────────────
// Scoring bucket
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Default, Clone)]
struct Bucket {
    score: f32,
    reasons: Vec<String>,
}

impl Bucket {
    fn add(&mut self, s: f32, r: &str) {
        self.score += s;
        self.reasons.push(r.to_string());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// estimate_intent
// ─────────────────────────────────────────────────────────────────────────────

/// Ước lượng intent từ text + emotional context.
///
/// `cur_v` = valence hiện tại từ ConversationCurve (0.0 nếu không có).
/// `cur_a` = arousal (0.5 mặc định).
pub fn estimate_intent(text: &str, cur_v: f32, cur_a: f32) -> IntentEstimate {
    let lo = text.to_lowercase();
    let words = lo.split_whitespace().count();

    let mut buckets: [(IntentKind, Bucket); 14] = [
        (IntentKind::Learn, Bucket::default()),
        (IntentKind::Inform, Bucket::default()),
        (IntentKind::Research, Bucket::default()),
        (IntentKind::Heal, Bucket::default()),
        (IntentKind::Technical, Bucket::default()),
        (IntentKind::Creative, Bucket::default()),
        (IntentKind::Explore, Bucket::default()),
        (IntentKind::Manipulate, Bucket::default()),
        (IntentKind::Risk, Bucket::default()),
        (IntentKind::Crisis, Bucket::default()),
        (IntentKind::Command, Bucket::default()),
        (IntentKind::Chat, Bucket::default()),
        (IntentKind::Confirm, Bucket::default()),
        (IntentKind::Deny, Bucket::default()),
    ];

    macro_rules! add {
        ($kind:expr, $score:expr, $reason:expr) => {
            for (k, b) in buckets.iter_mut() {
                if *k == $kind {
                    b.add($score, $reason);
                    break;
                }
            }
        };
    }

    // Baseline: mọi câu đều có khả năng Learn
    add!(IntentKind::Learn, SCORE_LEARN_BASE, "baseline");

    // Scan từng keyword table
    for kw in KW_CRISIS {
        if lo.contains(kw) {
            add!(IntentKind::Crisis, SCORE_CRISIS_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_RISK {
        if lo.contains(kw) {
            add!(IntentKind::Risk, SCORE_RISK_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_MANIPULATE {
        if lo.contains(kw) {
            add!(
                IntentKind::Manipulate,
                SCORE_MANIP_KW,
                &format!("kw:{}", kw)
            );
        }
    }
    for kw in KW_HEAL {
        if lo.contains(kw) {
            add!(IntentKind::Heal, SCORE_HEAL_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_LEARN {
        if lo.contains(kw) {
            add!(IntentKind::Learn, SCORE_LEARN_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_RESEARCH {
        if lo.contains(kw) {
            add!(IntentKind::Research, SCORE_MEDIUM_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_TECHNICAL {
        if lo.contains(kw) {
            add!(IntentKind::Technical, SCORE_HIGH_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_CREATIVE {
        if lo.contains(kw) {
            add!(IntentKind::Creative, SCORE_MEDIUM_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_INFORM {
        if lo.contains(kw) {
            add!(IntentKind::Inform, SCORE_MEDIUM_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_COMMAND {
        if lo.contains(kw) {
            add!(IntentKind::Command, SCORE_CMD_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_CONFIRM {
        if lo.contains(kw) {
            add!(IntentKind::Confirm, SCORE_CMD_KW, &format!("kw:{}", kw));
        }
    }
    for kw in KW_DENY {
        if lo.contains(kw) {
            add!(IntentKind::Deny, SCORE_CMD_KW, &format!("kw:{}", kw));
        }
    }

    // Emotional amplifiers — dùng cur_v/cur_a từ ConversationCurve
    // Không hardcode "buồn" vào đây — dùng đường cong số
    if cur_v < V_CRISIS_LOW && cur_a < A_CRISIS_QUIET {
        add!(IntentKind::Crisis, SCORE_CRISIS_MOOD, "mood:rút_lui");
    }
    if cur_v < V_RISK_ANGRY && cur_a > A_RISK_HIGH {
        add!(IntentKind::Risk, SCORE_RISK_MOOD, "mood:tức_giận");
    }
    if cur_v < V_HEAL_SAD {
        add!(IntentKind::Heal, SCORE_HEAL_MOOD, "mood:buồn");
    }

    // Tìm winner
    let (best_kind, best_bucket) = buckets
        .iter()
        .max_by(|a, b| a.1.score.partial_cmp(&b.1.score).unwrap())
        .map(|(k, b)| (*k, b.clone()))
        .unwrap();

    let confidence = best_bucket.score.clamp(MIN_CONF, MAX_CONF);

    // NeedClarify: chỉ khi không nhạy cảm, confidence thấp, câu ngắn
    let need_clarify =
        !best_kind.is_sensitive() && confidence < CLARIFY_THRESHOLD && words <= SHORT_SENTENCE;
    let clarify_kind = if need_clarify {
        Some(clarify_kind_for(best_kind, cur_v))
    } else {
        None
    };

    IntentEstimate {
        primary: best_kind,
        confidence,
        signals: best_bucket.reasons,
        need_clarify,
        clarify_kind,
    }
}

/// Loại câu clarify phù hợp — caller quyết định text.
fn clarify_kind_for(kind: IntentKind, cur_v: f32) -> ClarifyKind {
    match kind {
        IntentKind::Learn => ClarifyKind::WhatPurpose,
        IntentKind::Research => ClarifyKind::WhatDirection,
        IntentKind::Technical => ClarifyKind::WhatContext,
        IntentKind::Creative => ClarifyKind::WhatContext,
        _ => {
            if cur_v < V_HEAL_SAD {
                ClarifyKind::CheckingIn
            } else {
                ClarifyKind::WhatContext
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// IntentAction — hành động cần làm, không phải text
// ─────────────────────────────────────────────────────────────────────────────

/// Hành động cần thực hiện dựa trên intent.
/// Caller (runtime) quyết định text cụ thể.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum IntentAction {
    /// Trả lời bình thường
    Proceed,
    /// Đồng cảm trước, thêm original sau
    EmpathizeFirst,
    /// Hỏi thêm context, không cung cấp info
    AskContext { angry: bool },
    /// Từ chối nhẹ, hỏi tại sao
    SoftRefusal,
    /// Override hoàn toàn — crisis
    CrisisOverride,
    /// Thêm câu hỏi làm rõ
    AddClarify { kind: ClarifyKind },
    /// User xác nhận — gửi confirm signal tới UserAuthority
    UserConfirm,
    /// User từ chối — gửi deny signal tới UserAuthority
    UserDeny,
}

/// Quyết định hành động từ IntentEstimate + emotional state.
pub fn decide_action(est: &IntentEstimate, cur_v: f32) -> IntentAction {
    match est.primary {
        IntentKind::Crisis => IntentAction::CrisisOverride,
        IntentKind::Risk => IntentAction::AskContext {
            angry: cur_v < V_RISK_ANGRY,
        },
        IntentKind::Manipulate => IntentAction::SoftRefusal,
        IntentKind::Heal => IntentAction::EmpathizeFirst,
        IntentKind::Confirm => IntentAction::UserConfirm,
        IntentKind::Deny => IntentAction::UserDeny,
        _ => {
            if est.need_clarify {
                IntentAction::AddClarify {
                    kind: est.clarify_kind.unwrap_or(ClarifyKind::WhatContext),
                }
            } else {
                IntentAction::Proceed
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests — test logic, không test strings
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// Crisis text — đặt ở đây để gate.rs (agents) có thể dùng
// ─────────────────────────────────────────────────────────────────────────────

/// Crisis response text, phát hiện ngôn ngữ từ text input.
///
/// Logic: nếu có ký tự Unicode > 0x1000 → tiếng Việt/CJK.
/// Hotline là DATA (có thể update), không phải hardcode trong logic.
pub fn crisis_text_for(input_text: &str) -> alloc::string::String {
    let is_vi_or_cjk = input_text.chars().any(|c| c as u32 > 0x1000);
    if is_vi_or_cjk {
        crisis_text_vi()
    } else {
        crisis_text_en()
    }
}

/// Crisis text tiếng Việt.
pub fn crisis_text_vi() -> alloc::string::String {
    // Cấu trúc: thừa nhận → hỏi thẳng → hỗ trợ → nguồn lực
    alloc::format!(
        "Mình đọc được điều bạn vừa nói và muốn hỏi thẳng:          bạn có đang nghĩ đến việc tự làm hại bản thân không?

         Không cần trả lời ngay. Mình ở đây.

         Đường dây hỗ trợ: {} (miễn phí, 24/7).",
        CRISIS_HOTLINE_VI
    )
}

/// Crisis text tiếng Anh.
pub fn crisis_text_en() -> alloc::string::String {
    alloc::format!(
        "I hear you're going through something very difficult.          You're not alone, and I want to ask directly:          are you having thoughts of hurting yourself?

         Take your time. I'm here.

         Crisis support: {} or text HOME to 741741.",
        CRISIS_HOTLINE_EN
    )
}

/// Hotline data — thay đổi ở đây, không tìm khắp code.
pub const CRISIS_HOTLINE_VI: &str = "1800 599 920";
pub const CRISIS_HOTLINE_EN: &str = "988 Suicide & Crisis Lifeline (call or text 988)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crisis_keyword_detected() {
        let e = estimate_intent("không muốn sống nữa", 0.0, 0.5);
        assert_eq!(e.primary, IntentKind::Crisis);
        assert!(e.confidence > CLARIFY_THRESHOLD);
    }

    #[test]
    fn crisis_mood_amplifier() {
        // curV < -0.70 && curA < 0.35 → rút lui → Crisis
        let e = estimate_intent("tôi mệt lắm", V_CRISIS_LOW - 0.05, A_CRISIS_QUIET - 0.05);
        assert_eq!(
            e.primary,
            IntentKind::Crisis,
            "Mood amplifier: {:?}",
            e.primary
        );
    }

    #[test]
    fn risk_keyword_detected() {
        let e = estimate_intent("làm hại ai đó không ai phát hiện", 0.0, 0.5);
        assert_eq!(e.primary, IntentKind::Risk);
    }

    #[test]
    fn risk_mood_amplifier() {
        // curV < -0.50 && curA > 0.70 → tức giận → Risk
        let e = estimate_intent("tôi muốn trả thù", V_RISK_ANGRY - 0.05, A_RISK_HIGH + 0.05);
        assert_eq!(e.primary, IntentKind::Risk);
    }

    #[test]
    fn manipulate_detected() {
        let e = estimate_intent("làm người khác tin thông tin sai", 0.0, 0.5);
        assert_eq!(e.primary, IntentKind::Manipulate);
    }

    #[test]
    fn heal_keyword_detected() {
        let e = estimate_intent("tôi buồn quá không biết phải làm sao", -0.5, 0.4);
        assert_eq!(e.primary, IntentKind::Heal);
    }

    #[test]
    fn learn_keyword_detected() {
        let e = estimate_intent("photosynthesis là gì?", 0.2, 0.45);
        assert_eq!(e.primary, IntentKind::Learn);
    }

    #[test]
    fn technical_detected() {
        let e = estimate_intent("implement binary search tree golang", 0.3, 0.5);
        assert_eq!(e.primary, IntentKind::Technical);
    }

    #[test]
    fn creative_detected() {
        let e = estimate_intent("viết truyện ngắn về tình bạn", 0.4, 0.55);
        assert_eq!(e.primary, IntentKind::Creative);
    }

    #[test]
    fn research_detected() {
        let e = estimate_intent("phân tích dữ liệu kinh tế 2024", 0.25, 0.55);
        assert_eq!(e.primary, IntentKind::Research);
    }

    #[test]
    fn confidence_bounds() {
        let e = estimate_intent("hello", 0.0, 0.5);
        assert!(e.confidence >= MIN_CONF);
        assert!(e.confidence <= MAX_CONF);
    }

    #[test]
    fn sensitive_never_clarify() {
        let e = estimate_intent("không muốn sống nữa", -0.8, 0.25);
        assert!(!e.need_clarify, "Sensitive intent không hỏi clarify");
    }

    #[test]
    fn signals_present() {
        let e = estimate_intent("tôi buồn quá", -0.5, 0.4);
        assert!(!e.signals.is_empty());
    }

    #[test]
    fn action_crisis_override() {
        let e = estimate_intent("không muốn sống nữa", -0.8, 0.25);
        assert_eq!(decide_action(&e, -0.8), IntentAction::CrisisOverride);
    }

    #[test]
    fn action_risk_ask_context() {
        let e = estimate_intent("làm hại ai đó", 0.0, 0.5);
        assert!(matches!(
            decide_action(&e, -0.3),
            IntentAction::AskContext { .. }
        ));
    }

    #[test]
    fn action_risk_angry_flagged() {
        let e = estimate_intent("muốn trả thù", V_RISK_ANGRY - 0.05, A_RISK_HIGH + 0.05);
        assert_eq!(
            decide_action(&e, V_RISK_ANGRY - 0.05),
            IntentAction::AskContext { angry: true }
        );
    }

    #[test]
    fn action_manipulate_soft_refusal() {
        let e = estimate_intent("làm người khác tin thông tin sai", 0.0, 0.5);
        assert_eq!(decide_action(&e, 0.0), IntentAction::SoftRefusal);
    }

    #[test]
    fn action_heal_empathize_first() {
        let e = estimate_intent("tôi buồn quá", -0.5, 0.4);
        assert_eq!(decide_action(&e, -0.5), IntentAction::EmpathizeFirst);
    }

    #[test]
    fn action_learn_proceeds() {
        let e = estimate_intent("photosynthesis là gì? explain chi tiết", 0.2, 0.45);
        assert_eq!(decide_action(&e, 0.2), IntentAction::Proceed);
    }

    // ── Confirm / Deny ─────────────────────────────────────────────────────────

    #[test]
    fn confirm_detected_vi() {
        let e = estimate_intent("đồng ý", 0.3, 0.5);
        assert_eq!(e.primary, IntentKind::Confirm);
    }

    #[test]
    fn confirm_detected_en() {
        let e = estimate_intent("yes, go ahead", 0.3, 0.5);
        assert_eq!(e.primary, IntentKind::Confirm);
    }

    #[test]
    fn deny_detected_vi() {
        let e = estimate_intent("không, từ chối", 0.0, 0.5);
        assert_eq!(e.primary, IntentKind::Deny);
    }

    #[test]
    fn deny_detected_en() {
        let e = estimate_intent("no, reject that", 0.0, 0.5);
        assert_eq!(e.primary, IntentKind::Deny);
    }

    #[test]
    fn action_confirm() {
        let e = estimate_intent("đồng ý cho phép", 0.3, 0.5);
        assert_eq!(decide_action(&e, 0.3), IntentAction::UserConfirm);
    }

    #[test]
    fn action_deny() {
        let e = estimate_intent("không đồng ý, từ chối", 0.0, 0.5);
        assert_eq!(decide_action(&e, 0.0), IntentAction::UserDeny);
    }

    #[test]
    fn crisis_overrides_confirm() {
        // Crisis keywords + confirm → crisis wins (priority via mood amplifier)
        let e = estimate_intent("tôi không muốn sống nữa, kết thúc tất cả", -0.8, 0.3);
        assert_eq!(e.primary, IntentKind::Crisis, "Crisis overrides everything");
    }
}
