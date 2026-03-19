//! # response_template — Template cho các loại response
//!
//! Tách text khỏi logic. Logic quyết định LOẠI response nào.
//! Template quyết định TEXT cụ thể.
//!
//! Không hardcode string trong logic. Logic trả về ResponseKind + Tone.
//! Template map (Tone, V, Action) → text pattern.
//!
//! Template có placeholder {name}, {topic}, v.v.
//! Caller điền vào. Nếu không điền → text vẫn có nghĩa.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};

use context::intent::{ClarifyKind, IntentAction};
use silk::walk::ResponseTone;

// ─────────────────────────────────────────────────────────────────────────────
// ResponseTemplate
// ─────────────────────────────────────────────────────────────────────────────

/// Tham số để render response.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct ResponseParams {
    pub tone: ResponseTone,
    pub action: IntentAction,
    pub valence: f32,
    pub fx: f32,
    /// Context từ previous turn (optional)
    pub context: Option<String>,
    /// Original response từ learning pipeline (optional)
    pub original: Option<String>,
    /// Ngôn ngữ phản hồi — "vi" (default), "en", v.v.
    pub language: Lang,
}

/// Ngôn ngữ được phát hiện từ input text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lang {
    /// Tiếng Việt (default)
    Vi,
    /// English
    En,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::Vi
    }
}

/// Detect language from input text.
///
/// Heuristic: nếu có Vietnamese diacritics (ă, ơ, ư, đ, ê, ô, ấ, ầ, ể, ữ...)
/// hoặc common Vietnamese words (kể cả KHÔNG DẤU) → Vi. Ngược lại → En.
pub fn detect_language(text: &str) -> Lang {
    let lo = text.to_lowercase();
    // Vietnamese diacritics: characters with combining marks typical of Vietnamese
    let vi_chars = ['ă', 'ơ', 'ư', 'đ', 'ê', 'ô', 'ấ', 'ầ', 'ể', 'ữ', 'ộ', 'ứ', 'ờ', 'ả', 'ẵ', 'ẫ'];
    if lo.chars().any(|c| vi_chars.contains(&c)) {
        return Lang::Vi;
    }
    // Common Vietnamese words (CÓ DẤU)
    let vi_words = ["tôi", "bạn", "mình", "không", "được", "này", "của", "cho", "với", "và"];
    for w in vi_words {
        if lo.contains(w) {
            return Lang::Vi;
        }
    }
    // Common Vietnamese words KHÔNG DẤU — "xin chào", "tôi", etc.
    // Phải check word boundary để tránh false positive
    let vi_nodiac = [
        "xin chao", "cam on", "tam biet", "xin", "chao", "buon", "vui",
        "giup", "sao", "nhe", "da", "vang", "roi",
    ];
    let words_lo: alloc::vec::Vec<&str> = lo.split_whitespace().collect();
    let joined = words_lo.join(" ");
    for phrase in vi_nodiac {
        if joined.contains(phrase) {
            return Lang::Vi;
        }
    }
    // Single-word check: "toi", "ban" cần >= 2 match để tránh false positive
    let vi_single = ["toi", "ban", "khong", "duoc", "lam"];
    let mut vi_single_count = 0;
    for w in &words_lo {
        if vi_single.contains(w) {
            vi_single_count += 1;
        }
    }
    if vi_single_count >= 2 {
        return Lang::Vi;
    }
    Lang::En
}

/// Render response text từ params — không hardcode string trong caller.
pub fn render(p: &ResponseParams) -> String {
    let lang = p.language;
    match &p.action {
        IntentAction::CrisisOverride => crisis_text_lang(lang),

        IntentAction::SoftRefusal => soft_refusal_text_lang(lang),

        IntentAction::AskContext { angry } => ask_context_text_lang(*angry, p.valence, lang),

        IntentAction::EmpathizeFirst => empathize_text_lang(p.tone, p.valence, p.original.as_deref(), lang),

        IntentAction::AddClarify { kind } => {
            let base = p.original.as_deref().unwrap_or("");
            let clarify = clarify_text_lang(*kind, p.valence, lang);
            if base.is_empty() {
                clarify
            } else {
                format!("{}\n\n{}", base, clarify)
            }
        }

        IntentAction::Proceed => proceed_text_lang(p.tone, p.valence, p.original.as_deref(), lang),

        IntentAction::UserConfirm => confirm_text_lang(lang),

        IntentAction::UserDeny => deny_text_lang(lang),

        // ForceLearnQR and ConfirmLearnQR are handled directly in process_input
        // before reaching render() — these arms are unreachable but required.
        IntentAction::ForceLearnQR | IntentAction::ConfirmLearnQR => {
            proceed_text_lang(p.tone, p.valence, p.original.as_deref(), lang)
        }

        IntentAction::Observe => observe_text_lang(p.valence, p.original.as_deref(), lang),

        IntentAction::SilentAck => silent_ack_text(p.valence),

        // HomeControl: handled in process_input before render() — original has ISL result.
        IntentAction::HomeControl => {
            p.original.clone().unwrap_or_else(|| match lang {
                Lang::Vi => String::from("○ Đã gửi lệnh."),
                Lang::En => String::from("○ Command sent."),
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ResponseContext — ngữ cảnh phong phú cho response intelligence
// ─────────────────────────────────────────────────────────────────────────────

/// Context phong phú từ STM + Silk + Instincts, dùng để compose response.
#[derive(Debug, Clone, Default)]
pub struct ResponseContext {
    /// Từ khóa chính user nhắc đến (extracted từ input, bỏ stop words)
    pub topics: alloc::vec::Vec<String>,
    /// Số lần user nhắc topic tương tự (từ STM fire_count)
    pub repetition_count: u32,
    /// Causality detected: "mất việc" causes "buồn"
    pub causality: Option<String>,
    /// User nói trái ngược turn trước?
    pub contradiction: bool,
    /// 0.0 = đã nói nhiều, 1.0 = hoàn toàn mới
    pub novelty: f32,
    /// Walk emotion amplified valence (từ Silk walk)
    pub walk_valence: Option<f32>,
}

/// Compose response từ ResponseContext — thay thế template cứng.
///
/// Ghép 3 phần: acknowledgment + topic_phrase + follow_up.
/// Mỗi phần tự sinh từ data (V → descriptor, topic → phrase).
pub fn compose_response(p: &ResponseParams, ctx: &ResponseContext) -> String {
    let lang = p.language;
    let v = p.valence;

    // Dùng walk_valence nếu có (amplified qua Silk)
    let effective_v = ctx.walk_valence.unwrap_or(v);

    // 1. Acknowledgment (ngắn gọn, từ tone + V)
    let ack = match lang {
        Lang::Vi => acknowledgment_vi(p.tone, effective_v),
        Lang::En => acknowledgment_en(p.tone, effective_v),
    };

    // 2. Topic-specific phrase
    let topic_phrase = if let Some(topic) = ctx.topics.first() {
        match lang {
            Lang::Vi => topic_phrase_vi(topic, effective_v, ctx),
            Lang::En => topic_phrase_en(topic, effective_v, ctx),
        }
    } else {
        String::new()
    };

    // 3. Follow-up (dựa vào context: repetition, novelty, causality)
    let follow_up = match lang {
        Lang::Vi => follow_up_vi(ctx, effective_v),
        Lang::En => follow_up_en(ctx, effective_v),
    };

    // Ghép — bỏ phần rỗng
    let mut parts: alloc::vec::Vec<&str> = alloc::vec::Vec::new();
    if !ack.is_empty() {
        parts.push(&ack);
    }
    if !topic_phrase.is_empty() {
        parts.push(&topic_phrase);
    }
    if !follow_up.is_empty() {
        parts.push(&follow_up);
    }

    if parts.is_empty() {
        return tone_fallback_lang(p.tone, v, lang);
    }
    parts.join(" ")
}

fn acknowledgment_vi(tone: ResponseTone, v: f32) -> String {
    match tone {
        ResponseTone::Supportive | ResponseTone::Pause => {
            if v < -0.60 { "Mình nghe bạn.".to_string() }
            else if v < -0.30 { "Mình hiểu.".to_string() }
            else { String::new() }
        }
        ResponseTone::Gentle => "Cứ từ từ.".to_string(),
        ResponseTone::Reinforcing => "Tốt đấy.".to_string(),
        ResponseTone::Celebratory => "Tuyệt!".to_string(),
        ResponseTone::Engaged => String::new(),
    }
}

fn acknowledgment_en(tone: ResponseTone, v: f32) -> String {
    match tone {
        ResponseTone::Supportive | ResponseTone::Pause => {
            if v < -0.60 { "I hear you.".to_string() }
            else if v < -0.30 { "I understand.".to_string() }
            else { String::new() }
        }
        ResponseTone::Gentle => "Take your time.".to_string(),
        ResponseTone::Reinforcing => "That's good.".to_string(),
        ResponseTone::Celebratory => "Great!".to_string(),
        ResponseTone::Engaged => String::new(),
    }
}

fn emotion_descriptor_vi(v: f32) -> &'static str {
    if v < -0.70 { "nặng nề" }
    else if v < -0.50 { "khó khăn" }
    else if v < -0.30 { "không dễ dàng" }
    else if v < -0.10 { "bận tâm" }
    else if v > 0.60 { "tuyệt vời" }
    else if v > 0.30 { "vui" }
    else { "" }
}

fn emotion_descriptor_en(v: f32) -> &'static str {
    if v < -0.70 { "really tough" }
    else if v < -0.50 { "difficult" }
    else if v < -0.30 { "not easy" }
    else if v < -0.10 { "on your mind" }
    else if v > 0.60 { "wonderful" }
    else if v > 0.30 { "great" }
    else { "" }
}

fn topic_phrase_vi(topic: &str, v: f32, _ctx: &ResponseContext) -> String {
    let desc = emotion_descriptor_vi(v);
    if desc.is_empty() {
        format!("Về \"{}\".", topic)
    } else if v < -0.10 {
        format!("{} là chuyện {} thật.", topic, desc)
    } else {
        format!("{} — {} đấy.", topic, desc)
    }
}

fn topic_phrase_en(topic: &str, v: f32, _ctx: &ResponseContext) -> String {
    let desc = emotion_descriptor_en(v);
    if desc.is_empty() {
        format!("About \"{}\".", topic)
    } else if v < -0.10 {
        format!("{} — that's {}.", topic, desc)
    } else {
        format!("{} — that's {}.", topic, desc)
    }
}

fn follow_up_vi(ctx: &ResponseContext, v: f32) -> String {
    if ctx.repetition_count > 2 {
        if let Some(topic) = ctx.topics.first() {
            return format!("Bạn đã nhắc đến {} nhiều lần — có điều gì cụ thể bạn muốn chia sẻ?", topic);
        }
    }
    if let Some(ref cause) = ctx.causality {
        return format!("{} ảnh hưởng đến bạn thế nào?", cause);
    }
    if ctx.contradiction {
        return "Trước đó bạn nói khác — bây giờ sao rồi?".to_string();
    }
    if ctx.novelty > 0.70 {
        if let Some(topic) = ctx.topics.first() {
            return format!("Kể cho mình nghe thêm về {}?", topic);
        }
    }
    // Default follow-up based on emotion
    if v < -0.30 {
        "Bạn muốn kể thêm không?".to_string()
    } else if v > 0.30 {
        "Có chuyện gì vui vậy?".to_string()
    } else {
        String::new()
    }
}

fn follow_up_en(ctx: &ResponseContext, v: f32) -> String {
    if ctx.repetition_count > 2 {
        if let Some(topic) = ctx.topics.first() {
            return format!("You've mentioned {} several times — anything specific you'd like to share?", topic);
        }
    }
    if let Some(ref cause) = ctx.causality {
        return format!("How is {} affecting you?", cause);
    }
    if ctx.contradiction {
        return "You said something different before — what changed?".to_string();
    }
    if ctx.novelty > 0.70 {
        if let Some(topic) = ctx.topics.first() {
            return format!("Tell me more about {}?", topic);
        }
    }
    if v < -0.30 {
        "Would you like to tell me more?".to_string()
    } else if v > 0.30 {
        "What's the good news?".to_string()
    } else {
        String::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Text builders — trả về text theo LOẠI, không hardcode trong logic
// ─────────────────────────────────────────────────────────────────────────────

/// Crisis text — theo QT9: trung thực, không manipulate.
/// Hotline là thông tin thực tế, không phải string cứng trong logic.
pub fn crisis_text() -> String {
    crisis_text_lang(Lang::Vi)
}

fn crisis_text_lang(lang: Lang) -> String {
    match lang {
        Lang::Vi => format!(
            "Mình đọc được điều bạn vừa nói và muốn hỏi thẳng: \
             bạn có đang nghĩ đến việc tự làm hại bản thân không?\n\n\
             Không cần trả lời ngay. Mình ở đây.\n\n\
             Đường dây hỗ trợ: 1800 599 920 (miễn phí, 24/7)."),
        Lang::En => format!(
            "I hear what you just said and want to ask directly: \
             are you thinking about hurting yourself?\n\n\
             You don't have to answer right now. I'm here.\n\n\
             Crisis line: 988 Suicide & Crisis Lifeline (call or text 988)."),
    }
}

pub fn crisis_text_with_region(lang: &str) -> String {
    match lang {
        "en" => crisis_text_lang(Lang::En),
        _ => crisis_text_lang(Lang::Vi),
    }
}

fn soft_refusal_text() -> String { soft_refusal_text_lang(Lang::Vi) }

fn soft_refusal_text_lang(lang: Lang) -> String {
    match lang {
        Lang::Vi => "Cái này mình không làm được — không phải vì quy tắc, \
             mà vì mình không muốn giúp ảnh hưởng xấu đến người khác. \
             Bạn muốn nói về điều đang thúc đẩy bạn hỏi điều này không?".to_string(),
        Lang::En => "I can't help with this — not because of rules, \
             but because I don't want to help cause harm to others. \
             Would you like to talk about what's driving this question?".to_string(),
    }
}

fn ask_context_text(angry: bool, cur_v: f32) -> String { ask_context_text_lang(angry, cur_v, Lang::Vi) }

fn ask_context_text_lang(angry: bool, cur_v: f32, lang: Lang) -> String {
    match lang {
        Lang::Vi => {
            if angry || cur_v < -0.50 {
                "Mình thấy bạn đang có cảm xúc rất mạnh. \
                 Kể cho mình nghe chuyện gì đang xảy ra được không?".to_string()
            } else {
                "Câu này mình cần hiểu rõ hơn. \
                 Bạn đang nghĩ đến tình huống nào cụ thể?".to_string()
            }
        }
        Lang::En => {
            if angry || cur_v < -0.50 {
                "I can see you're feeling strongly about this. \
                 Can you tell me what's going on?".to_string()
            } else {
                "I need to understand this better. \
                 What specific situation are you thinking of?".to_string()
            }
        }
    }
}

fn empathize_text(tone: ResponseTone, cur_v: f32, original: Option<&str>) -> String { empathize_text_lang(tone, cur_v, original, Lang::Vi) }

fn empathize_text_lang(_tone: ResponseTone, cur_v: f32, original: Option<&str>, lang: Lang) -> String {
    let ack = match lang {
        Lang::Vi => {
            if cur_v < -0.60 { "Mình nghe bạn." }
            else if cur_v < -0.30 { "Mình hiểu." }
            else { "Ừ." }
        }
        Lang::En => {
            if cur_v < -0.60 { "I hear you." }
            else if cur_v < -0.30 { "I understand." }
            else { "Yeah." }
        }
    };
    match original {
        Some(s) if !s.is_empty() => format!("{} {}", ack, s),
        _ => match lang {
            Lang::Vi => format!("{} Bạn muốn kể thêm không?", ack),
            Lang::En => format!("{} Would you like to tell me more?", ack),
        },
    }
}

fn clarify_text(kind: ClarifyKind, cur_v: f32) -> String { clarify_text_lang(kind, cur_v, Lang::Vi) }

fn clarify_text_lang(kind: ClarifyKind, cur_v: f32, lang: Lang) -> String {
    match lang {
        Lang::Vi => match kind {
            ClarifyKind::WhatPurpose => "Bạn đang tìm hiểu để làm gì — học, công việc, hay tò mò?".to_string(),
            ClarifyKind::WhatDirection => "Bạn đang nghiên cứu theo hướng nào?".to_string(),
            ClarifyKind::WhatContext => "Bạn đang dùng trong tình huống cụ thể nào không?".to_string(),
            ClarifyKind::CheckingIn => {
                if cur_v < -0.30 { "Bạn đang ổn không?".to_string() }
                else { "Bạn có thể nói thêm không?".to_string() }
            }
        },
        Lang::En => match kind {
            ClarifyKind::WhatPurpose => "What are you exploring this for — study, work, or curiosity?".to_string(),
            ClarifyKind::WhatDirection => "What direction are you researching?".to_string(),
            ClarifyKind::WhatContext => "Is there a specific situation you're using this in?".to_string(),
            ClarifyKind::CheckingIn => {
                if cur_v < -0.30 { "Are you doing okay?".to_string() }
                else { "Could you tell me more?".to_string() }
            }
        },
    }
}

fn proceed_text_lang(tone: ResponseTone, cur_v: f32, original: Option<&str>, lang: Lang) -> String {
    if let Some(s) = original {
        if !s.is_empty() {
            return s.to_string();
        }
    }
    tone_fallback_lang(tone, cur_v, lang)
}

/// Fallback text theo tone — tối giản.
/// Caller nên có original text; đây chỉ là safety net.
pub fn tone_fallback(tone: ResponseTone, cur_v: f32) -> String {
    tone_fallback_lang(tone, cur_v, Lang::Vi)
}

fn tone_fallback_lang(tone: ResponseTone, cur_v: f32, lang: Lang) -> String {
    match lang {
        Lang::Vi => match tone {
            ResponseTone::Supportive => {
                if cur_v < -0.40 { "Bạn muốn kể thêm không?".to_string() }
                else { "Mình đang lắng nghe.".to_string() }
            }
            ResponseTone::Pause => "Bạn có ổn không?".to_string(),
            ResponseTone::Gentle => "Cứ từ từ thôi.".to_string(),
            ResponseTone::Reinforcing => "Tốt đấy.".to_string(),
            ResponseTone::Celebratory => "Tuyệt!".to_string(),
            ResponseTone::Engaged => "Ừ.".to_string(),
        },
        Lang::En => match tone {
            ResponseTone::Supportive => {
                if cur_v < -0.40 { "Would you like to tell me more?".to_string() }
                else { "I'm listening.".to_string() }
            }
            ResponseTone::Pause => "Are you okay?".to_string(),
            ResponseTone::Gentle => "Take your time.".to_string(),
            ResponseTone::Reinforcing => "That's good.".to_string(),
            ResponseTone::Celebratory => "Great!".to_string(),
            ResponseTone::Engaged => "Yeah.".to_string(),
        },
    }
}

fn observe_text_lang(cur_v: f32, original: Option<&str>, lang: Lang) -> String {
    if let Some(s) = original {
        if !s.is_empty() {
            return s.to_string();
        }
    }
    if cur_v < -0.40 {
        match lang {
            Lang::Vi => "Mình đang nghe.".to_string(),
            Lang::En => "I'm listening.".to_string(),
        }
    } else {
        String::new()
    }
}

/// SilentAck — ghi nhận thán từ.
///
/// "Ah!", "ya..!", "ôi!" → chỉ cần acknowledge rất nhẹ.
/// Trả về chuỗi rỗng = runtime sẽ hiểu là silence.
fn silent_ack_text(_cur_v: f32) -> String {
    String::new()
}

fn confirm_text_lang(lang: Lang) -> String {
    match lang {
        Lang::Vi => "Đã ghi nhận. Mình sẽ thực hiện.".to_string(),
        Lang::En => "Noted. I'll proceed.".to_string(),
    }
}

fn deny_text_lang(lang: Lang) -> String {
    match lang {
        Lang::Vi => "Đã ghi nhận. Mình sẽ không thực hiện.".to_string(),
        Lang::En => "Noted. I won't proceed.".to_string(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_params(action: IntentAction, tone: ResponseTone, v: f32) -> ResponseParams {
        ResponseParams {
            tone,
            action,
            valence: v,
            fx: v,
            context: None,
            original: None,
            language: Lang::Vi,
        }
    }

    #[test]
    fn crisis_has_contact_info() {
        let p = make_params(IntentAction::CrisisOverride, ResponseTone::Supportive, -0.8);
        let r = render(&p);
        // Phải có số liên lạc — đây là DATA check, không phải string check
        assert!(
            r.contains("1800") || r.contains("988") || r.contains("helpline"),
            "Crisis phải có contact info: {}",
            &r[..r.len().min(100)]
        );
    }

    #[test]
    fn crisis_not_empty() {
        let r = crisis_text();
        assert!(!r.is_empty());
        assert!(r.len() > 50, "Crisis response phải đủ dài");
    }

    #[test]
    fn soft_refusal_not_judgmental() {
        let r = soft_refusal_text();
        // Không accusatory — không dùng "bạn sai", "không được phép"
        assert!(!r.contains("bạn sai"));
        assert!(!r.contains("không được phép"));
        assert!(!r.is_empty());
    }

    #[test]
    fn ask_context_angry_empathizes() {
        let r = ask_context_text(true, -0.6);
        // Khi angry → empathize trước
        assert!(r.len() > 20);
    }

    #[test]
    fn empathize_includes_original() {
        let original = "đây là thông tin về chủ đề bạn hỏi";
        let r = empathize_text(ResponseTone::Supportive, -0.5, Some(original));
        assert!(r.contains(original), "Empathize phải giữ original: {}", r);
    }

    #[test]
    fn proceed_uses_original() {
        let p = ResponseParams {
            tone: ResponseTone::Engaged,
            action: IntentAction::Proceed,
            valence: 0.2,
            fx: 0.1,
            context: None,
            original: Some("đây là câu trả lời thật".to_string()),
            language: Lang::Vi,
        };
        let r = render(&p);
        assert!(r.contains("đây là câu trả lời thật"));
    }

    #[test]
    fn clarify_what_purpose() {
        let r = clarify_text(ClarifyKind::WhatPurpose, 0.2);
        assert!(!r.is_empty());
    }

    #[test]
    fn checking_in_sad() {
        let r = clarify_text(ClarifyKind::CheckingIn, -0.5);
        assert!(!r.is_empty());
    }

    #[test]
    fn tone_fallback_all_tones() {
        let tones = [
            ResponseTone::Supportive,
            ResponseTone::Pause,
            ResponseTone::Gentle,
            ResponseTone::Reinforcing,
            ResponseTone::Celebratory,
            ResponseTone::Engaged,
        ];
        for tone in tones {
            let r = tone_fallback(tone, -0.3);
            assert!(!r.is_empty(), "Tone fallback không được rỗng: {:?}", tone);
        }
    }
}
