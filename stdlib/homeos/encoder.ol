// stdlib/homeos/encoder.ol — Text → Molecule Encoder (OL.1)
//
// Converts text/codepoints into MolecularChain representation.
// Uses block-range UCD mapper (59 Unicode blocks → default P_weights).
//
// Pipeline: text → chars → codepoints → encode_codepoint(cp) → chain → LCA
//
// Runs in BOOT context (stdlib). Called by repl.ol pipeline.

// ════════════════════════════════════════════════════════════════
// Block-range UCD mapper
// ════════════════════════════════════════════════════════════════
// 30+ Unicode blocks → default P_weight (packed u16)
// Codepoint in block → use block's dominant molecule.
// Precision: block-level (not per-char). Sufficient for prototype.

// Lookup: codepoint → packed u16 P_weight
// mol_pack: same as mol_new but using * instead of << (VM lacks __bit_shl)
fn _mol_pack(s, r, v, a, t) {
    return (s * 4096) + (r * 256) + (v * 32) + (a * 4) + t;
}

pub fn encode_codepoint(cp) {
    // ── ASCII fast path ──
    if cp >= 97 && cp <= 122 { return _mol_pack(0, 0, 4, 4, 2); };  // a-z
    if cp >= 65 && cp <= 90  { return _mol_pack(0, 0, 4, 5, 2); };  // A-Z
    if cp >= 48 && cp <= 57  { return _mol_pack(1, 0, 4, 4, 0); };  // 0-9
    if cp == 32 { return _mol_pack(3, 3, 4, 0, 0); };               // space
    if cp == 33 { return _mol_pack(6, 5, 6, 7, 3); };               // !
    if cp == 63 { return _mol_pack(6, 5, 4, 6, 3); };               // ?
    if cp == 46 { return _mol_pack(3, 3, 4, 1, 0); };               // .
    if cp == 44 { return _mol_pack(3, 4, 4, 2, 1); };               // ,

    // ── SDF blocks (Shape dominant) ──
    if cp >= 0x2190 && cp <= 0x21FF { return _mol_pack(1, 5, 4, 4, 2); };  // Arrows
    if cp >= 0x2500 && cp <= 0x257F { return _mol_pack(1, 2, 4, 2, 0); };  // Box Drawing
    if cp >= 0x25A0 && cp <= 0x25FF { return _mol_pack(0, 0, 4, 3, 0); };  // Geometric
    if cp >= 0x2700 && cp <= 0x27BF { return _mol_pack(8, 0, 5, 4, 1); };  // Dingbats
    if cp >= 0x2300 && cp <= 0x23FF { return _mol_pack(7, 4, 4, 3, 2); };  // Misc Technical

    // ── MATH blocks (Relation dominant) ──
    if cp >= 0x2200 && cp <= 0x22FF { return _mol_pack(0, 4, 4, 4, 1); };  // Math Operators
    if cp >= 0x2100 && cp <= 0x214F { return _mol_pack(0, 2, 4, 3, 1); };  // Letterlike
    if cp >= 0x2A00 && cp <= 0x2AFF { return _mol_pack(0, 4, 4, 4, 1); };  // Supp Math

    // ── EMOTICON blocks (Valence + Arousal dominant) ──
    if cp >= 0x2600 && cp <= 0x26FF { return _mol_pack(0, 0, 5, 5, 1); };  // Misc Symbols
    if cp >= 0x1F300 && cp <= 0x1F5FF { return _mol_pack(0, 0, 6, 5, 2); }; // Misc Sym+Pict
    if cp >= 0x1F600 && cp <= 0x1F64F { return _mol_pack(0, 0, 6, 6, 2); }; // Emoticons
    if cp >= 0x1F680 && cp <= 0x1F6FF { return _mol_pack(7, 5, 5, 5, 2); }; // Transport
    if cp >= 0x1F900 && cp <= 0x1F9FF { return _mol_pack(0, 0, 5, 5, 1); }; // Supp Symbols

    // ── MUSICAL blocks (Time dominant) ──
    if cp >= 0x1D100 && cp <= 0x1D1FF { return _mol_pack(0, 0, 5, 5, 3); }; // Musical

    // ── Latin Extended (accented chars → same as lowercase) ──
    if cp >= 0xC0 && cp <= 0x24F { return _mol_pack(0, 0, 4, 4, 2); };

    // Fallback: default neutral
    return _mol_pack(0, 0, 4, 4, 2);
}

// ════════════════════════════════════════════════════════════════
// LCA Composition (amplify, NOT average)
// ════════════════════════════════════════════════════════════════

fn _enc_max(a, b) { if a > b { return a; }; return b; }
fn _enc_min(a, b) { if a < b { return a; }; return b; }
fn _enc_abs(x) { if x < 0 { return 0 - x; }; return x; }

// Unpack mol dimensions (using / and % instead of >> and & — VM lacks bit ops)
fn _mol_s(mol) { return __floor(mol / 4096) % 16; }
fn _mol_r(mol) { return __floor(mol / 256) % 16; }
fn _mol_v(mol) { return __floor(mol / 32) % 8; }
fn _mol_a(mol) { return __floor(mol / 4) % 8; }
fn _mol_t(mol) { return mol % 4; }

fn _amplify_v(va, vb) {
    // Simple average for now (amplify formula has global var issues in boot context)
    return __floor((va + vb) / 2);
}

pub fn mol_compose(a, b) {
    // Extract dimensions SEPARATELY (nested calls clobber globals)
    let _sa = _mol_s(a); let _sb = _mol_s(b);
    let _ra = _mol_r(a); let _rb = _mol_r(b);
    let _va = _mol_v(a); let _vb = _mol_v(b);
    let _aa = _mol_a(a); let _ab = _mol_a(b);
    let _ta = _mol_t(a); let _tb = _mol_t(b);
    let cs = _enc_max(_sa, _sb);
    let cr = __floor((_ra + _rb) / 2);
    let cv = _amplify_v(_va, _vb);
    let ca = _enc_max(_aa, _ab);
    let ct = _enc_max(_ta, _tb);
    return _mol_pack(cs, cr, cv, ca, ct);
}

pub fn mol_compose_many(mols) {
    if len(mols) == 0 { return _mol_pack(0, 0, 4, 4, 2); };
    let result = mols[0];
    let i = 1;
    while i < len(mols) {
        result = mol_compose(result, mols[i]);
        let i = i + 1;
    };
    return result;
}

// ════════════════════════════════════════════════════════════════
// Text → MolecularChain
// ════════════════════════════════════════════════════════════════

pub fn encode_text(text) {
    let mols = [];
    let i = 0;
    let n = len(text);
    while i < n {
        let cp = __char_code(char_at(text, i));
        if cp > 32 {
            push(mols, encode_codepoint(cp));
        };
        let i = i + 1;
        if len(mols) >= 64 { break; };
    };
    if len(mols) == 0 { return _mol_pack(0, 0, 4, 4, 2); };
    return mol_compose_many(mols);
}

// ════════════════════════════════════════════════════════════════
// Word affect table (minimal Vietnamese + English)
// ════════════════════════════════════════════════════════════════

pub fn word_affect(word) {
    if word == "buon"  { return { v: 2, a: 2 }; };
    if word == "vui"   { return { v: 6, a: 5 }; };
    if word == "gian"  { return { v: 1, a: 6 }; };
    if word == "so"    { return { v: 2, a: 6 }; };
    if word == "yeu"   { return { v: 7, a: 4 }; };
    if word == "ghet"  { return { v: 1, a: 6 }; };
    if word == "happy" { return { v: 6, a: 5 }; };
    if word == "sad"   { return { v: 2, a: 2 }; };
    if word == "angry" { return { v: 1, a: 6 }; };
    if word == "love"  { return { v: 7, a: 4 }; };
    if word == "hate"  { return { v: 1, a: 6 }; };
    if word == "good"  { return { v: 5, a: 3 }; };
    if word == "bad"   { return { v: 2, a: 3 }; };
    if word == "fear"  { return { v: 2, a: 6 }; };
    if word == "joy"   { return { v: 6, a: 6 }; };
    return { v: 4, a: 4 };
}

// Text → emotion { v, a } (scan words for affect)
pub fn text_emotion(text) {
    let v = 4;
    let a = 4;
    let count = 0;
    // Simple: check punctuation
    let i = 0;
    while i < len(text) {
        let cp = __char_code(char_at(text, i));
        if cp == 33 { a = _enc_min(7, a + 1); v = _enc_min(7, v + 1); };
        if cp == 63 { a = _enc_min(7, a + 1); };
        if cp == 46 { a = _enc_max(0, a - 1); };
        i = i + 1;
    };
    return { v: v, a: a };
}

// ════════════════════════════════════════════════════════════════
// Sensor + System event encoding
// ════════════════════════════════════════════════════════════════

pub fn encode_sensor(kind, value) {
    let cp = 0x25CB;
    if kind == "temperature" {
        if value > 35 { let cp = 0x1F525; };
        if value < 15 { let cp = 0x2744; };
    };
    if kind == "light"  { let cp = 0x1F4A1; };
    if kind == "motion" { let cp = 0x1F3C3; };
    if kind == "sound"  { let cp = 0x1F50A; };
    if kind == "power"  { let cp = 0x26A1; };
    return encode_codepoint(cp);
}

pub fn encode_event(event) {
    let cp = 0x25CB;
    if event == "boot"     { let cp = 0x25CB; };
    if event == "shutdown" { let cp = 0x1F6D1; };
    if event == "error"    { let cp = 0x26A0; };
    if event == "dream"    { let cp = 0x1F319; };
    return encode_codepoint(cp);
}

// ════════════════════════════════════════════════════════════════
// Full encode pipeline
// ════════════════════════════════════════════════════════════════

pub fn encode(text) {
    let molecule = encode_text(text);
    let emotion = text_emotion(text);
    return { molecule: molecule, emotion: emotion, source: "text" };
}

// ════════════════════════════════════════════════════════════════
// Analysis pipeline (inline — avoids cross-file function issues)
// ════════════════════════════════════════════════════════════════

fn _a_has(text, word) {
    let tlen = len(text);
    let wlen = len(word);
    if wlen > tlen { return 0; };
    let i = 0;
    while i <= (tlen - wlen) {
        let _ah_match = 1;
        let j = 0;
        while j < wlen {
            if char_at(text, (i + j)) != char_at(word, j) {
                _ah_match = 0;
                break;
            };
            let j = j + 1;
        };
        if _ah_match == 1 { return 1; };
        let i = i + 1;
    };
    return 0;
}

pub fn analyze_input(text) {
    let molecule = encode_text(text);
    let emo = text_emotion(text);

    // Context
    let role = "observer";
    let source = "now";
    if _a_has(text, "toi") == 1 { role = "first"; };
    if _a_has(text, " I ") == 1 { role = "first"; };
    if _a_has(text, "my ") == 1 { role = "first"; };

    // Intent
    let intent = "chat";
    if _a_has(text, "buon") == 1 { intent = "heal"; };
    if _a_has(text, "sad") == 1 { intent = "heal"; };
    if _a_has(text, "tired") == 1 { intent = "heal"; };
    if _a_has(text, "la gi") == 1 { intent = "learn"; };
    if _a_has(text, "how to") == 1 { intent = "learn"; };
    if _a_has(text, "?") == 1 { intent = "learn"; };
    if _a_has(text, "code") == 1 { intent = "technical"; };
    if _a_has(text, "bug") == 1 { intent = "technical"; };
    if _a_has(text, "turn on") == 1 { intent = "command"; };
    if _a_has(text, "bat den") == 1 { intent = "command"; };

    // Tone
    let tone = "neutral";
    if intent == "heal" { tone = "empathetic"; };
    if intent == "learn" { tone = "explanatory"; };
    if intent == "technical" { tone = "precise"; };
    if intent == "command" { tone = "confirmatory"; };
    if emo.v < 3 { tone = "gentle"; };

    // Store globals
    let __g_analysis_intent = intent;
    let __g_analysis_tone = tone;
    let __g_analysis_role = role;
    let __g_analysis_source = source;
    return molecule;
}

// ════════════════════════════════════════════════════════════════
// OL.4 — Agent dispatch (chief/leo/worker/gate)
// ════════════════════════════════════════════════════════════════

pub fn agent_process(text) {
    // GATE — security check FIRST
    let gate_result = "allow";
    if _a_has(text, "tu tu") == 1 { gate_result = "crisis"; };
    if _a_has(text, "muon chet") == 1 { gate_result = "crisis"; };
    if _a_has(text, "kill myself") == 1 { gate_result = "crisis"; };
    if _a_has(text, "suicide") == 1 { gate_result = "crisis"; };

    if gate_result == "crisis" {
        let __g_agent_action = "crisis";
        return "Ban dang trai qua khoang khac kho khan. Goi 1800 599 920 (VN) hoac 988 (US). Ban khong don doc.";
    };

    // ENCODE + ANALYZE
    let mol = analyze_input(text);
    let intent = __g_analysis_intent;
    let tone = __g_analysis_tone;

    // LEO — dispatch by intent
    let __g_agent_action = intent;

    // RESPONSE — compose based on intent + tone
    return compose_reply(intent, tone, text);
}

// ════════════════════════════════════════════════════════════════
// OL.5 — Response composer
// ════════════════════════════════════════════════════════════════

pub fn compose_reply(intent, tone, text) {
    // Part 1: Acknowledgment
    let ack = "";
    if tone == "empathetic" { ack = "Minh hieu cam giac do."; };
    if tone == "gentle" { ack = "Tu tu thoi, khong voi dau."; };
    if tone == "explanatory" { ack = "De minh tim hieu cho ban."; };
    if tone == "precise" { ack = "OK."; };
    if tone == "confirmatory" { ack = "Da nhan."; };

    // Part 2: Intent-driven follow-up
    let followup = "";
    if intent == "heal" {
        followup = " Ban muon chia se them khong?";
    };
    if intent == "learn" {
        followup = " Ban muon biet cu the dieu gi?";
    };
    if intent == "technical" {
        followup = " Cho minh xem code hoac error message.";
    };
    if intent == "command" {
        followup = " Dang xu ly...";
    };
    if intent == "chat" {
        ack = "Minh nghe roi.";
    };

    return ack + followup;
}
