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

// ════════════════════════════════════════════════════════════════
// STM — Short-Term Memory
// ════════════════════════════════════════════════════════════════
// Keeps last N exchanges. Each entry: { input, intent, tone, molecule }
// Agent can reference previous inputs for context.

let __stm = [];
let __stm_max = 32;

pub fn stm_push(text, intent, tone) {
    push(__stm, { input: text, intent: intent, tone: tone, turn: len(__stm) });
    // Evict oldest if over limit
    if len(__stm) > __stm_max {
        let __stm_new = [];
        let i = 1;
        while i < len(__stm) {
            push(__stm_new, __stm[i]);
            let i = i + 1;
        };
        let __stm = __stm_new;
    };
}

pub fn stm_last_input() {
    if len(__stm) == 0 { return ""; };
    return __stm[len(__stm) - 1].input;
}

pub fn stm_last_intent() {
    if len(__stm) == 0 { return "chat"; };
    return __stm[len(__stm) - 1].intent;
}

pub fn stm_count() {
    return len(__stm);
}

// Check if topic repeated N+ times
pub fn stm_topic_repeated(keyword, n) {
    let count = 0;
    let i = 0;
    while i < len(__stm) {
        if _a_has(__stm[i].input, keyword) == 1 {
            count = count + 1;
        };
        let i = i + 1;
    };
    if count >= n { return 1; };
    return 0;
}

// Context summary: summarize conversation themes from STM
pub fn stm_summary() {
    let _ss_heal = 0;
    let _ss_learn = 0;
    let _ss_tech = 0;
    let _ss_chat = 0;
    let _ss_i = 0;
    while _ss_i < len(__stm) {
        let _ss_intent = __stm[_ss_i].intent;
        if _ss_intent == "heal" { _ss_heal = _ss_heal + 1; };
        if _ss_intent == "learn" { _ss_learn = _ss_learn + 1; };
        if _ss_intent == "technical" { _ss_tech = _ss_tech + 1; };
        if _ss_intent == "chat" { _ss_chat = _ss_chat + 1; };
        let _ss_i = _ss_i + 1;
    };
    // Build summary
    let _ss_result = "";
    if _ss_heal > 0 { _ss_result = _ss_result + "cam xuc(" + __to_string(_ss_heal) + ") "; };
    if _ss_learn > 0 { _ss_result = _ss_result + "hoi dap(" + __to_string(_ss_learn) + ") "; };
    if _ss_tech > 0 { _ss_result = _ss_result + "ky thuat(" + __to_string(_ss_tech) + ") "; };
    if _ss_chat > 0 { _ss_result = _ss_result + "tro chuyen(" + __to_string(_ss_chat) + ") "; };
    return _ss_result;
}

// ════════════════════════════════════════════════════════════════
// Silk — Hebbian Learning (fire together → wire together)
// ════════════════════════════════════════════════════════════════
// Simplified: edges stored as flat array of { from, to, weight, emotion }

let __silk = [];
let __silk_max = 64;

fn silk_co_activate(word_a, word_b, intent) {
    // Search for existing edge
    let i = 0;
    while i < len(__silk) {
        let e = __silk[i];
        if e.from == word_a {
            if e.to == word_b {
                // Strengthen: Hebbian update
                set_at(__silk, i, {
                    from: word_a, to: word_b,
                    weight: (e.weight + (0.01 * ((1 - (e.weight * 0.618))))),
                    emotion: intent, fires: (e.fires + 1)
                });
                return;
            };
        };
        let i = i + 1;
    };
    // New edge
    if len(__silk) < __silk_max {
        push(__silk, { from: word_a, to: word_b, weight: 0.1, emotion: intent, fires: 1 });
    };
}

fn silk_learn_from_text(text, intent) {
    // Co-activate consecutive words (bigrams)
    let words = [];
    let current = "";
    let i = 0;
    while i < len(text) {
        let ch = char_at(text, i);
        if __char_code(ch) == 32 {
            if len(current) > 0 {
                push(words, current);
                current = "";
            };
        } else {
            current = current + ch;
        };
        let i = i + 1;
    };
    if len(current) > 0 { push(words, current); };
    // Wire bigrams
    let j = 0;
    while (j + 1) < len(words) {
        silk_co_activate(words[j], words[j + 1], intent);
        let j = j + 1;
    };
}

fn silk_find_related(word) {
    // Find strongest connection for a word
    let best = "";
    let best_w = 0;
    let i = 0;
    while i < len(__silk) {
        let e = __silk[i];
        if e.from == word {
            if e.weight > best_w {
                best_w = e.weight;
                best = e.to;
            };
        };
        if e.to == word {
            if e.weight > best_w {
                best_w = e.weight;
                best = e.from;
            };
        };
        let i = i + 1;
    };
    return best;
}

pub fn silk_count() { return len(__silk); }

// ════════════════════════════════════════════════════════════════
// Dream — Consolidation (scan STM → find themes → strengthen Silk)
// ════════════════════════════════════════════════════════════════

let __dream_count = 0;

fn dream_cycle() {
    // Run every 5 turns — scan STM for repeated intents, strengthen patterns
    let __dream_count = __dream_count + 1;
    if __hyp_mod(__dream_count, 5) != 0 { return; };

    // Count intent frequencies in STM
    let heal_count = 0;
    let learn_count = 0;
    let i = 0;
    while i < len(__stm) {
        if __stm[i].intent == "heal" { heal_count = heal_count + 1; };
        if __stm[i].intent == "learn" { learn_count = learn_count + 1; };
        let i = i + 1;
    };

    // If dominant theme → store as consolidated memory
    if heal_count >= 3 {
        let __dream_theme = "heal";
    };
    if learn_count >= 3 {
        let __dream_theme = "learn";
    };
}

// ════════════════════════════════════════════════════════════════
// STM Retrieval — search memory for related past turns
// ════════════════════════════════════════════════════════════════

fn stm_find_related(_sfr_input) {
    // Split input into words, check each against past inputs
    // ALL vars use _sfr_ prefix to avoid collision with _a_has params (text, word)
    let _sfr_text = _sfr_input;
    let i = 0;
    let _sfr_limit = stm_count() - 1;
    while i < _sfr_limit {
        let _sfr_past = __stm[i].input;
        let wi = 0;
        let _sfr_w = "";
        while wi < len(_sfr_text) {
            let ch = char_at(_sfr_text, wi);
            if __char_code(ch) == 32 {
                if len(_sfr_w) >= 3 {
                    let _sfr_check = _sfr_w;
                    if _a_has(_sfr_past, _sfr_check) == 1 {
                        return _sfr_past;
                    };
                };
                _sfr_w = "";
            } else {
                _sfr_w = _sfr_w + ch;
            };
            let wi = wi + 1;
        };
        if len(_sfr_w) >= 3 {
            let _sfr_check = _sfr_w;
            if _a_has(_sfr_past, _sfr_check) == 1 {
                return _sfr_past;
            };
        };
        let i = i + 1;
    };
    return "";
}

// ════════════════════════════════════════════════════════════════
// Agent v3 — with memory + Silk + Dream
// ════════════════════════════════════════════════════════════════

pub fn agent_respond(text) {
    // GATE
    if _a_has(text, "tu tu") == 1 { return "Ban dang trai qua khoang khac kho khan. Goi 1800 599 920 (VN) hoac 988 (US). Ban khong don doc."; };
    if _a_has(text, "kill myself") == 1 { return "Ban dang trai qua khoang khac kho khan. Goi 1800 599 920 (VN) hoac 988 (US). Ban khong don doc."; };

    // ENCODE + ANALYZE
    let mol = analyze_input(text);
    let intent = __g_analysis_intent;
    let tone = __g_analysis_tone;

    // REMEMBER (STM)
    stm_push(text, intent, tone);

    // LEARN (Silk — Hebbian co-activation)
    silk_learn_from_text(text, intent);

    // DREAM (consolidation — every 5 turns)
    dream_cycle();

    // RETRIEVE — search past memory for related turns
    let memory_context = "";
    if stm_count() >= 2 {
        let _ar_current = text;
        let _ar_related = stm_find_related(_ar_current);
        if len(_ar_related) > 0 {
            if _ar_related != _ar_current {
                memory_context = " (Minh nho truoc do ban noi ve: " + _ar_related + ")";
            };
        };
    };

    // CONTEXT — repeated topic detection
    if stm_count() >= 3 {
        if stm_topic_repeated(text, 2) == 1 {
            memory_context = " Minh thay ban nhac lai dieu nay. Minh hieu no quan trong voi ban.";
        };
    };

    // CONTEXT — heal→ok transition
    if stm_count() >= 2 {
        let _prev_idx = stm_count() - 2;
        if _prev_idx >= 0 {
            if __stm[_prev_idx].intent == "heal" {
                if intent != "heal" {
                    memory_context = " Ban co ve da on hon roi.";
                };
            };
        };
    };

    // SILK — find related concept
    let silk_related = "";
    // Split first word and find silk connection
    let first_word = "";
    let fi = 0;
    while fi < len(text) {
        if __char_code(char_at(text, fi)) == 32 { break; };
        first_word = first_word + char_at(text, fi);
        let fi = fi + 1;
    };
    if len(first_word) > 2 {
        silk_related = silk_find_related(first_word);
    };

    // KNOWLEDGE RETRIEVAL — search learned facts
    let _ar_knowledge = "";
    if len(__knowledge) > 0 {
        _ar_knowledge = knowledge_search(_ar_current);
    };

    // RESPOND
    let reply = compose_reply(intent, tone, text);
    if len(_ar_knowledge) > 0 {
        return reply + memory_context + " " + _ar_knowledge;
    };
    return reply + memory_context;
}

// ════════════════════════════════════════════════════════════════
// Knowledge Store — learned facts (from `learn` command)
// ════════════════════════════════════════════════════════════════
// Each entry: { text, keywords[] }
// Keywords extracted by splitting on spaces, keeping 3+ char words.

let __knowledge = [];
let __knowledge_max = 512;

pub fn knowledge_learn(text) {
    // Extract keywords (3+ char words)
    let _kl_words = [];
    let _kl_w = "";
    let _kl_i = 0;
    while _kl_i < len(text) {
        let _kl_ch = char_at(text, _kl_i);
        if __char_code(_kl_ch) == 32 {
            if len(_kl_w) >= 3 {
                push(_kl_words, _kl_w);
            };
            _kl_w = "";
        } else {
            _kl_w = _kl_w + _kl_ch;
        };
        let _kl_i = _kl_i + 1;
    };
    if len(_kl_w) >= 3 { push(_kl_words, _kl_w); };

    // Store
    push(__knowledge, { text: text, words: _kl_words });

    // Wire keywords into Silk
    let _kl_j = 0;
    while (_kl_j + 1) < len(_kl_words) {
        silk_co_activate(_kl_words[_kl_j], _kl_words[_kl_j + 1], "learn");
        let _kl_j = _kl_j + 1;
    };

    // Evict oldest if over limit
    if len(__knowledge) > __knowledge_max {
        let _kl_new = [];
        let _kl_k = 1;
        while _kl_k < len(__knowledge) {
            push(_kl_new, __knowledge[_kl_k]);
            let _kl_k = _kl_k + 1;
        };
        let __knowledge = _kl_new;
    };

    return len(__knowledge);
}

pub fn knowledge_count() { return len(__knowledge); }

fn knowledge_search(_ks_query) {
    // Split query into words, find best matching knowledge entry
    let _ks_best = "";
    let _ks_best_score = 0;

    let _ks_ki = 0;
    while _ks_ki < len(__knowledge) {
        let _ks_entry = __knowledge[_ks_ki];
        let _ks_score = 0;

        // Count word matches between query and entry keywords
        let _ks_qi = 0;
        let _ks_qw = "";
        while _ks_qi < len(_ks_query) {
            let _ks_ch = char_at(_ks_query, _ks_qi);
            if __char_code(_ks_ch) == 32 {
                if len(_ks_qw) >= 3 {
                    // Check if this word appears in entry keywords
                    let _ks_wi = 0;
                    while _ks_wi < len(_ks_entry.words) {
                        if _ks_entry.words[_ks_wi] == _ks_qw {
                            _ks_score = _ks_score + 1;
                        };
                        let _ks_wi = _ks_wi + 1;
                    };
                };
                _ks_qw = "";
            } else {
                _ks_qw = _ks_qw + _ks_ch;
            };
            let _ks_qi = _ks_qi + 1;
        };
        // Check last word
        if len(_ks_qw) >= 3 {
            let _ks_wi = 0;
            while _ks_wi < len(_ks_entry.words) {
                if _ks_entry.words[_ks_wi] == _ks_qw {
                    _ks_score = _ks_score + 1;
                };
                let _ks_wi = _ks_wi + 1;
            };
        };

        if _ks_score > _ks_best_score {
            _ks_best_score = _ks_score;
            _ks_best = _ks_entry.text;
        };
        let _ks_ki = _ks_ki + 1;
    };

    if _ks_best_score > 0 {
        return "(Minh biet: " + _ks_best + ")";
    };
    return "";
}

// ════════════════════════════════════════════════════════════════
// CUT.4 — Self-Build: origin.olang builds itself
// ════════════════════════════════════════════════════════════════

fn _sb_compile_file(_sbcf_path, _sbcf_bc, _sbcf_pos) {
    let _sbcf_hp = __heap_save();
    let _sbcf_src = __file_read(_sbcf_path);
    if len(_sbcf_src) > 0 {
        _prefill_output();
        let _sbcf_tokens = tokenize(_sbcf_src);
        let _sbcf_ast = parse(_sbcf_tokens);
        let _sbcf_state = analyze(_sbcf_ast);
        if _g_pos > 0 {
            let _sbcf_ci = 0;
            while _sbcf_ci < _g_pos {
                __bytes_set(_sbcf_bc, _sbcf_pos, _g_output[_sbcf_ci]);
                let _sbcf_pos = _sbcf_pos + 1;
                let _sbcf_ci = _sbcf_ci + 1;
            };
            emit "  " + _sbcf_path + ": " + __to_string(_g_pos) + " bytes";
        } else {
            emit "  " + _sbcf_path + ": SKIP";
        };
    };
    __heap_restore(_sbcf_hp);
    return _sbcf_pos;
}

pub fn self_build() {
    emit "=== Self-Build ===";

    // Step 1: Read own binary (VM ELF)
    let _sb_self = __file_read_bytes("/proc/self/exe");
    let _sb_self_size = __bytes_len(_sb_self);
    emit "  VM binary: " + __to_string(_sb_self_size) + " bytes";

    // Find header_offset (last 8 bytes of binary)
    let _sb_b0 = __bytes_get(_sb_self, _sb_self_size - 8);
    let _sb_b1 = __bytes_get(_sb_self, _sb_self_size - 7);
    let _sb_b2 = __bytes_get(_sb_self, _sb_self_size - 6);
    let _sb_b3 = __bytes_get(_sb_self, _sb_self_size - 5);
    let _sb_header_off = _sb_b0 + (_sb_b1 * 256) + (_sb_b2 * 65536) + (_sb_b3 * 16777216);
    emit "  Header offset: " + __to_string(_sb_header_off);

    // Step 2: Compile all .ol files
    let _sb_bc = __bytes_new(524288);
    let _sb_bc_pos = 0;
    let _sb_compiled = 0;
    let _sb_errors = 0;

    // Strategy: copy existing bytecode from current binary as BASE
    // Then try re-compiling additional files with heap_restore between each
    // (bootstrap files too large to re-compile with current heap limits)
    // Read bc_offset and bc_size from Origin header
    let _sb_bc_off_b0 = __bytes_get(_sb_self, _sb_header_off + 14);
    let _sb_bc_off_b1 = __bytes_get(_sb_self, _sb_header_off + 15);
    let _sb_bc_off_b2 = __bytes_get(_sb_self, _sb_header_off + 16);
    let _sb_bc_off = _sb_bc_off_b0 + (_sb_bc_off_b1 * 256) + (_sb_bc_off_b2 * 65536);
    let _sb_bc_sz_b0 = __bytes_get(_sb_self, _sb_header_off + 18);
    let _sb_bc_sz_b1 = __bytes_get(_sb_self, _sb_header_off + 19);
    let _sb_bc_sz_b2 = __bytes_get(_sb_self, _sb_header_off + 20);
    let _sb_bc_sz = _sb_bc_sz_b0 + (_sb_bc_sz_b1 * 256) + (_sb_bc_sz_b2 * 65536);
    emit "  Existing bytecode: " + __to_string(_sb_bc_sz) + " bytes at offset " + __to_string(_sb_bc_off);
    // Copy existing bytecode to output buffer
    let _sb_bci = 0;
    while _sb_bci < _sb_bc_sz {
        __bytes_set(_sb_bc, _sb_bc_pos, __bytes_get(_sb_self, _sb_bc_off + _sb_bci));
        let _sb_bc_pos = _sb_bc_pos + 1;
        let _sb_bci = _sb_bci + 1;
    };
    _sb_compiled = 1;
    emit "  Bytecode copied: " + __to_string(_sb_bc_pos) + " bytes";

    // Append Halt
    __bytes_set(_sb_bc, _sb_bc_pos, 15);
    _sb_bc_pos = _sb_bc_pos + 1;
    emit "  Total bytecode: " + __to_string(_sb_bc_pos) + " bytes (" + __to_string(_sb_compiled) + " files)";

    // Step 3: Pack binary
    // Output = [VM ELF up to header_offset][Origin header 32B][bytecode][trailer 8B]
    let _sb_total = _sb_header_off + 32 + _sb_bc_pos + 8;
    let _sb_out = __bytes_new(_sb_total);

    // Copy VM ELF (bytes 0 to header_offset)
    let _sb_vi = 0;
    while _sb_vi < _sb_header_off {
        __bytes_set(_sb_out, _sb_vi, __bytes_get(_sb_self, _sb_vi));
        let _sb_vi = _sb_vi + 1;
    };
    let _sb_pos = _sb_header_off;

    // Origin header (32 bytes)
    let _sb_bc_off = _sb_pos + 32;
    // Magic: ○LNG
    __bytes_set(_sb_out, _sb_pos, 226);     // 0xE2
    __bytes_set(_sb_out, _sb_pos + 1, 151); // 0x97
    __bytes_set(_sb_out, _sb_pos + 2, 139); // 0x8B
    __bytes_set(_sb_out, _sb_pos + 3, 76);  // 0x4C = 'L'
    __bytes_set(_sb_out, _sb_pos + 4, 16);  // version 0x10
    __bytes_set(_sb_out, _sb_pos + 5, 1);   // arch x86_64
    // bc_offset (bytes 14-17) as u32 LE
    __bytes_set(_sb_out, _sb_pos + 14, __floor(_sb_bc_off) % 256);
    __bytes_set(_sb_out, _sb_pos + 15, __floor((_sb_bc_off / 256)) % 256);
    __bytes_set(_sb_out, _sb_pos + 16, __floor((_sb_bc_off / 65536)) % 256);
    __bytes_set(_sb_out, _sb_pos + 17, __floor((_sb_bc_off / 16777216)) % 256);
    // bc_size (bytes 18-21)
    __bytes_set(_sb_out, _sb_pos + 18, __floor(_sb_bc_pos) % 256);
    __bytes_set(_sb_out, _sb_pos + 19, __floor((_sb_bc_pos / 256)) % 256);
    __bytes_set(_sb_out, _sb_pos + 20, __floor((_sb_bc_pos / 65536)) % 256);
    __bytes_set(_sb_out, _sb_pos + 21, __floor((_sb_bc_pos / 16777216)) % 256);
    // flags (byte 30): codegen format = 1
    __bytes_set(_sb_out, _sb_pos + 30, 1);
    _sb_pos = _sb_pos + 32;

    // Copy bytecode
    let _sb_bi = 0;
    while _sb_bi < _sb_bc_pos {
        __bytes_set(_sb_out, _sb_pos, __bytes_get(_sb_bc, _sb_bi));
        let _sb_pos = _sb_pos + 1;
        let _sb_bi = _sb_bi + 1;
    };

    // Trailer: header_offset as u64 LE (8 bytes)
    __bytes_set(_sb_out, _sb_pos, __floor(_sb_header_off) % 256);
    __bytes_set(_sb_out, _sb_pos + 1, __floor((_sb_header_off / 256)) % 256);
    __bytes_set(_sb_out, _sb_pos + 2, __floor((_sb_header_off / 65536)) % 256);
    __bytes_set(_sb_out, _sb_pos + 3, __floor((_sb_header_off / 16777216)) % 256);
    // Upper 4 bytes = 0 (header_offset < 4GB)
    _sb_pos = _sb_pos + 8;

    // Write output
    __bytes_write("origin_built.olang", _sb_out, _sb_pos);
    emit "  Output: origin_built.olang (" + __to_string(_sb_pos) + " bytes)";
    emit "=== Done ===";
    return "Built: " + __to_string(_sb_pos) + " bytes (" + __to_string(_sb_compiled) + " files, " + __to_string(_sb_errors) + " errors)";
}
