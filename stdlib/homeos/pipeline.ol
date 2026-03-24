// homeos/analysis.ol — Analysis orchestrator (OL.2)
//
// Orchestrates: encode → fuse → infer → intent → response tone
// Entry point: analyze_input(text) → ActivationResult
//
// Pipeline: GATE → ENCODE → INFER → PROMOTE → RESPONSE

// ════════════════════════════════════════════════════════════════
// Response tone selection
// ════════════════════════════════════════════════════════════════

fn _select_tone(intent, emo_v, emo_a) {
    if intent == "crisis" { return "supportive"; };
    if intent == "heal" { return "empathetic"; };
    if intent == "learn" { return "explanatory"; };
    if intent == "research" { return "analytical"; };
    if intent == "technical" { return "precise"; };
    if intent == "creative" { return "inspiring"; };
    if intent == "command" { return "confirmatory"; };
    // Default based on emotion
    if emo_v < -0.3 { return "gentle"; };
    if emo_a > 0.6 { return "calming"; };
    return "neutral";
}

// ════════════════════════════════════════════════════════════════
// Main analysis pipeline
// ════════════════════════════════════════════════════════════════

pub fn analyze_input(text) {
    // Step 1: ENCODE — text → molecule
    let molecule = encode_text(text);

    // Step 2: Basic emotion from text_emotion (encoder.ol)
    let emo = text_emotion(text);
    let emo_v = ((emo.v - 4) / 4);
    let emo_a = (emo.a / 7);

    // Step 3: Context — inline (avoid broken function calls)
    let role = "observer";
    let source = "real_now";
    if _i_has(text, "toi") == 1 { role = "first"; };
    if _i_has(text, " I ") == 1 { role = "first"; };
    if _i_has(text, "my ") == 1 { role = "first"; };
    if _i_has(text, "vua") == 1 { source = "real_now"; };
    if _i_has(text, "now") == 1 { source = "real_now"; };
    if _i_has(text, "hoi do") == 1 { source = "real_past"; };

    // Step 4: Intent — inline keyword scan
    let intent = "chat";
    if _i_has(text, "buon") == 1 { intent = "heal"; };
    if _i_has(text, "sad") == 1 { intent = "heal"; };
    if _i_has(text, "met") == 1 { intent = "heal"; };
    if _i_has(text, "tired") == 1 { intent = "heal"; };
    if _i_has(text, "la gi") == 1 { intent = "learn"; };
    if _i_has(text, "how to") == 1 { intent = "learn"; };
    if _i_has(text, "?") == 1 { intent = "learn"; };
    if _i_has(text, "code") == 1 { intent = "technical"; };
    if _i_has(text, "bug") == 1 { intent = "technical"; };
    if _i_has(text, "turn on") == 1 { intent = "command"; };
    if _i_has(text, "turn off") == 1 { intent = "command"; };
    if _i_has(text, "bat den") == 1 { intent = "command"; };
    if _i_has(text, "tat den") == 1 { intent = "command"; };

    // Step 5: Tone
    let tone = _select_tone(intent, emo_v, emo_a);

    // Store in globals (dicts get corrupted)
    let __g_analysis_intent = intent;
    let __g_analysis_tone = tone;
    let __g_analysis_role = role;
    let __g_analysis_source = source;
    return molecule;
}

// Inline substring search (no external deps)
fn _i_has(text, word) {
    let tlen = len(text);
    let wlen = len(word);
    if wlen > tlen { return 0; };
    let i = 0;
    while i <= (tlen - wlen) {
        let match = 1;
        let j = 0;
        while j < wlen {
            if char_at(text, (i + j)) != char_at(word, j) {
                match = 0;
                break;
            };
            let j = j + 1;
        };
        if match == 1 { return 1; };
        let i = i + 1;
    };
    return 0;
}
}
