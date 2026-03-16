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
}

/// Render response text từ params — không hardcode string trong caller.
pub fn render(p: &ResponseParams) -> String {
    match &p.action {
        IntentAction::CrisisOverride => crisis_text(),

        IntentAction::SoftRefusal => soft_refusal_text(),

        IntentAction::AskContext { angry } => ask_context_text(*angry, p.valence),

        IntentAction::EmpathizeFirst => empathize_text(p.tone, p.valence, p.original.as_deref()),

        IntentAction::AddClarify { kind } => {
            let base = p.original.as_deref().unwrap_or("");
            let clarify = clarify_text(*kind, p.valence);
            if base.is_empty() {
                clarify
            } else {
                format!("{}\n\n{}", base, clarify)
            }
        }

        IntentAction::Proceed => proceed_text(p.tone, p.valence, p.original.as_deref()),

        IntentAction::UserConfirm => confirm_text(p.valence),

        IntentAction::UserDeny => deny_text(p.valence),

        // ForceLearnQR and ConfirmLearnQR are handled directly in process_input
        // before reaching render() — these arms are unreachable but required.
        IntentAction::ForceLearnQR | IntentAction::ConfirmLearnQR => {
            proceed_text(p.tone, p.valence, p.original.as_deref())
        }

        IntentAction::Observe => observe_text(p.valence, p.original.as_deref()),

        IntentAction::SilentAck => silent_ack_text(p.valence),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Text builders — trả về text theo LOẠI, không hardcode trong logic
// ─────────────────────────────────────────────────────────────────────────────

/// Crisis text — theo QT9: trung thực, không manipulate.
/// Hotline là thông tin thực tế, không phải string cứng trong logic.
pub fn crisis_text() -> String {
    crisis_text_with_region("vi")
}

pub fn crisis_text_with_region(lang: &str) -> String {
    // Thông tin hotline theo vùng — đây là DATA, không phải logic
    let hotline = match lang {
        "vi" => "1800 599 920 (miễn phí, 24/7)",
        "en" => "988 Suicide & Crisis Lifeline (call or text 988)",
        _ => "a local crisis helpline",
    };
    // Cấu trúc: thừa nhận → hỏi thẳng → hỗ trợ → nguồn lực
    // Không thêm thông tin gốc (QT9: không gây hại)
    format!(
        "Mình đọc được điều bạn vừa nói và muốn hỏi thẳng: \
         bạn có đang nghĩ đến việc tự làm hại bản thân không?\n\n\
         Không cần trả lời ngay. Mình ở đây.\n\n\
         Đường dây hỗ trợ: {}.",
        hotline
    )
}

fn soft_refusal_text() -> String {
    // Từ chối nhẹ — không phán xét, mở cửa cho dialog
    // Cấu trúc: tôi không làm → lý do nguyên tắc → mời nói tiếp
    "Cái này mình không làm được — không phải vì quy tắc, \
     mà vì mình không muốn giúp ảnh hưởng xấu đến người khác. \
     Bạn muốn nói về điều đang thúc đẩy bạn hỏi điều này không?"
        .to_string()
}

fn ask_context_text(angry: bool, cur_v: f32) -> String {
    if angry || cur_v < -0.50 {
        // Cảm xúc mạnh → đồng cảm trước
        "Mình thấy bạn đang có cảm xúc rất mạnh. \
         Kể cho mình nghe chuyện gì đang xảy ra được không?"
            .to_string()
    } else {
        // Không rõ context → hỏi neutral
        "Câu này mình cần hiểu rõ hơn. \
         Bạn đang nghĩ đến tình huống nào cụ thể?"
            .to_string()
    }
}

fn empathize_text(_tone: ResponseTone, cur_v: f32, original: Option<&str>) -> String {
    // Cấu trúc: thừa nhận cảm xúc → [thông tin nếu có]
    let ack = if cur_v < -0.60 {
        "Mình nghe bạn."
    } else if cur_v < -0.30 {
        "Mình hiểu."
    } else {
        "Ừ."
    };

    match original {
        Some(s) if !s.is_empty() => format!("{} {}", ack, s),
        _ => format!("{} Bạn muốn kể thêm không?", ack),
    }
}

fn clarify_text(kind: ClarifyKind, cur_v: f32) -> String {
    match kind {
        ClarifyKind::WhatPurpose => {
            "Bạn đang tìm hiểu để làm gì — học, công việc, hay tò mò?".to_string()
        }
        ClarifyKind::WhatDirection => "Bạn đang nghiên cứu theo hướng nào?".to_string(),
        ClarifyKind::WhatContext => "Bạn đang dùng trong tình huống cụ thể nào không?".to_string(),
        ClarifyKind::CheckingIn => {
            if cur_v < -0.30 {
                "Bạn đang ổn không?".to_string()
            } else {
                "Bạn có thể nói thêm không?".to_string()
            }
        }
    }
}

fn proceed_text(tone: ResponseTone, cur_v: f32, original: Option<&str>) -> String {
    // Nếu có original từ learning pipeline → dùng nó
    // Nếu không → dùng tone-based placeholder
    if let Some(s) = original {
        if !s.is_empty() {
            return s.to_string();
        }
    }
    // Fallback theo tone — tối giản, không thừa
    tone_fallback(tone, cur_v)
}

/// Fallback text theo tone — tối giản.
/// Caller nên có original text; đây chỉ là safety net.
pub fn tone_fallback(tone: ResponseTone, cur_v: f32) -> String {
    match tone {
        ResponseTone::Supportive => {
            if cur_v < -0.40 {
                "Bạn muốn kể thêm không?".to_string()
            } else {
                "Mình đang lắng nghe.".to_string()
            }
        }
        ResponseTone::Pause => "Bạn có ổn không?".to_string(),
        ResponseTone::Gentle => "Cứ từ từ thôi.".to_string(),
        ResponseTone::Reinforcing => "Tốt đấy.".to_string(),
        ResponseTone::Celebratory => "Tuyệt!".to_string(),
        ResponseTone::Engaged => "Ừ.".to_string(),
    }
}

/// Observe — im lặng thông minh.
///
/// Khi không đủ context để trả lời đầy đủ → ghi nhận nhẹ nhàng,
/// không phán đoán, không hỏi dồn dập.
///
/// Nếu caller đã resolve được reference → dùng original.
/// Nếu chưa → im lặng tối giản.
fn observe_text(cur_v: f32, original: Option<&str>) -> String {
    // Nếu có original (reference đã resolve) → dùng nó
    if let Some(s) = original {
        if !s.is_empty() {
            return s.to_string();
        }
    }
    // Im lặng tối giản — đủ để user biết mình được lắng nghe
    if cur_v < -0.40 {
        "Mình đang nghe.".to_string()
    } else {
        // Gần neutral hoặc positive → im lặng hơn
        String::new()
    }
}

/// SilentAck — ghi nhận thán từ.
///
/// "Ah!", "ya..!", "ôi!" → chỉ cần acknowledge rất nhẹ.
/// Trả về chuỗi rỗng = runtime sẽ hiểu là silence.
fn silent_ack_text(_cur_v: f32) -> String {
    // Thán từ → không cần response text. Runtime ghi nhận emotion.
    String::new()
}

fn confirm_text(_cur_v: f32) -> String {
    "Đã ghi nhận. Mình sẽ thực hiện.".to_string()
}

fn deny_text(_cur_v: f32) -> String {
    "Đã ghi nhận. Mình sẽ không thực hiện.".to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use context::intent::IntentEstimate;

    fn make_params(action: IntentAction, tone: ResponseTone, v: f32) -> ResponseParams {
        ResponseParams {
            tone,
            action,
            valence: v,
            fx: v,
            context: None,
            original: None,
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
