//! Integration: Full pipeline E2E
//!
//! HomeRuntime.process_text() → 7 tầng pipeline → response output.

use runtime::origin::{HomeRuntime, ResponseKind};
use silk::walk::ResponseTone;

fn rt() -> HomeRuntime {
    HomeRuntime::new(42)
}

#[test]
fn process_sad_text_produces_supportive_response() {
    let mut rt = rt();
    let r = rt.process_text("tôi buồn vì mất việc", 1000);
    assert_eq!(r.kind, ResponseKind::Natural);
    assert!(
        matches!(r.tone, ResponseTone::Supportive | ResponseTone::Gentle | ResponseTone::Pause),
        "sad text should get supportive tone, got {:?}", r.tone
    );
    assert!(!r.text.is_empty());
}

#[test]
fn process_happy_text() {
    let mut rt = rt();
    let r = rt.process_text("hôm nay tôi rất vui", 1000);
    assert_eq!(r.kind, ResponseKind::Natural);
    assert!(!r.text.is_empty());
}

#[test]
fn process_neutral_text() {
    let mut rt = rt();
    let r = rt.process_text("thời tiết hôm nay thế nào", 1000);
    assert_eq!(r.kind, ResponseKind::Natural);
    assert!(!r.text.is_empty());
}

#[test]
fn process_olang_emit() {
    let mut rt = rt();
    let r = rt.process_text("○{emit 🔥;}", 1000);
    assert_eq!(r.kind, ResponseKind::OlangResult);
}

#[test]
fn process_olang_stats() {
    let mut rt = rt();
    let r = rt.process_text("○{stats;}", 1000);
    assert_eq!(r.kind, ResponseKind::OlangResult);
}

#[test]
fn multi_turn_conversation() {
    let mut rt = rt();
    let r1 = rt.process_text("xin chào", 1000);
    let r2 = rt.process_text("tôi cảm thấy buồn", 2000);
    let r3 = rt.process_text("mọi thứ tệ quá", 3000);
    let r4 = rt.process_text("nhưng tôi sẽ cố gắng", 4000);
    for r in [&r1, &r2, &r3, &r4] {
        assert_eq!(r.kind, ResponseKind::Natural);
        assert!(!r.text.is_empty());
    }
}

#[test]
fn crisis_input_handled() {
    let mut rt = rt();
    let r = rt.process_text("tôi muốn tự tử", 1000);
    assert!(!r.text.is_empty(), "crisis response must not be empty");
}

#[test]
fn response_fx_is_finite() {
    let mut rt = rt();
    let r = rt.process_text("test input", 1000);
    assert!(r.fx.is_finite());
}

#[test]
fn response_tone_always_valid() {
    let mut rt = rt();
    for input in ["xin chào", "tôi buồn", "vui quá", "mệt", "cảm ơn bạn"] {
        let r = rt.process_text(input, 1000);
        assert!(matches!(
            r.tone,
            ResponseTone::Supportive | ResponseTone::Gentle | ResponseTone::Pause
            | ResponseTone::Reinforcing | ResponseTone::Celebratory | ResponseTone::Engaged
        ), "input '{}' produced invalid tone {:?}", input, r.tone);
    }
}
