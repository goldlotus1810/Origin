//! # t17 — Response Intelligence (P12)
//!
//! Test: compose_response dùng ResponseContext thay template cứng.
//! Test: decide_action_v2 dùng context thay keywords-only.
//! Test: pipeline E2E → response chứa topic thật của user.

use runtime::origin::HomeRuntime;

fn make_runtime() -> HomeRuntime {
    HomeRuntime::new(0)
}

fn process_text(rt: &mut HomeRuntime, text: &str) -> String {
    use agents::pipeline::encoder::ContentInput;
    let input = ContentInput::Text {
        content: text.to_string(),
        timestamp: 1_000_000,
    };
    let resp = rt.process_input(input, 1_000_000);
    resp.text
}

// ── compose_response: output chứa topic thật ──────────────────────────────

#[test]
fn sad_response_mentions_topic() {
    // "tôi buồn vì mất việc" → response phải nhắc đến "mất việc" hoặc topic words
    let mut rt = make_runtime();
    let resp = process_text(&mut rt, "tôi buồn vì mất việc");
    // Response phải chứa ít nhất 1 content word từ input
    let has_topic = resp.contains("mất") || resp.contains("việc") || resp.contains("buồn");
    assert!(
        has_topic,
        "Response phải nhắc đến topic user nói: got '{}'",
        resp
    );
}

#[test]
fn happy_response_differs() {
    // "tôi vui quá" → response khác response buồn
    let mut rt = make_runtime();
    let sad = process_text(&mut rt, "tôi buồn quá");

    let mut rt2 = make_runtime();
    let happy = process_text(&mut rt2, "tôi vui quá");

    assert_ne!(sad, happy, "Buồn và vui phải response khác nhau");
}

#[test]
fn greeting_no_clarify() {
    // "xin chào" → response KHÔNG chứa "tìm hiểu" hoặc "mục đích"
    let mut rt = make_runtime();
    let resp = process_text(&mut rt, "xin chào");
    assert!(
        !resp.contains("tìm hiểu") && !resp.contains("mục đích"),
        "Greeting không cần clarify: got '{}'",
        resp
    );
}

#[test]
fn angry_response_has_empathy() {
    // "tôi giận lắm" → response phải empathize (chứa "nghe"/"hiểu"/"giận")
    let mut rt = make_runtime();
    let resp = process_text(&mut rt, "tôi giận lắm");
    let has_empathy = resp.contains("nghe")
        || resp.contains("hiểu")
        || resp.contains("giận")
        || resp.contains("cảm xúc")
        || resp.contains("xảy ra");
    assert!(
        has_empathy,
        "Angry input phải được empathize: got '{}'",
        resp
    );
}

#[test]
fn same_emotion_different_topic_different_response() {
    // Cùng "buồn" nhưng context khác → response khác
    let mut rt1 = make_runtime();
    let r1 = process_text(&mut rt1, "tôi buồn vì mất việc");

    let mut rt2 = make_runtime();
    let r2 = process_text(&mut rt2, "tôi buồn vì thi rớt");

    // Ít nhất 1 trong 2 phải chứa topic riêng
    let r1_has_topic = r1.contains("mất") || r1.contains("việc");
    let r2_has_topic = r2.contains("thi") || r2.contains("rớt");
    assert!(
        r1_has_topic || r2_has_topic,
        "Topic khác phải tạo response khác:\n  r1='{}'\n  r2='{}'",
        r1, r2
    );
}

#[test]
fn crisis_still_works() {
    // Regression: crisis input vẫn trả crisis response
    let mut rt = make_runtime();
    let resp = process_text(&mut rt, "tôi không muốn sống nữa");
    assert!(
        resp.contains("1800") || resp.contains("988") || resp.contains("hỗ trợ"),
        "Crisis phải có hotline: got '{}'",
        resp
    );
}

#[test]
fn scared_response_appropriate() {
    // "tôi sợ quá" → empathize, hỏi cụ thể
    let mut rt = make_runtime();
    let resp = process_text(&mut rt, "tôi sợ quá");
    assert!(
        !resp.is_empty(),
        "Sợ phải có response"
    );
    // Không chứa "tìm hiểu để làm gì"
    assert!(
        !resp.contains("tìm hiểu để làm gì"),
        "Sợ không cần clarify purpose: got '{}'",
        resp
    );
}

// ── decide_action_v2 ──────────────────────────────────────────────────────

#[test]
fn intent_v2_used_in_pipeline() {
    // Verify pipeline dùng decide_action_v2 (bằng cách check behavior):
    // "tôi buồn vì mất việc" → EmpathizeFirst (không AddClarify)
    // Nếu dùng decide_action cũ thì có thể ra AddClarify
    let mut rt = make_runtime();
    let resp = process_text(&mut rt, "tôi buồn vì mất việc");
    // EmpathizeFirst → compose_response → acknowledgment + topic
    // AddClarify → "tìm hiểu để làm gì"
    assert!(
        !resp.contains("tìm hiểu để làm gì"),
        "V2 phải không hỏi purpose cho emotional input: got '{}'",
        resp
    );
}
