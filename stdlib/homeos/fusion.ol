// homeos/fusion.ol — Cross-modal emotion fusion (OL.2)
//
// Blends emotions from multiple sources (text, audio, sensor, bio).
// Uses weighted averaging + conflict detection + φ⁻¹ amplification.
//
// Spec: "KHÔNG BAO GIỜ trung bình cảm xúc" — amplify via Silk weight.

// ── Constants ──────────────────────────────────────────────────
let PHI_INV = 0.618;
let PHI_INV_SQ = 0.382;
let CONFLICT_THRESHOLD = 0.382;
let BLACKCURTAIN_THRESHOLD = 0.29;

// ── Modality weights ───────────────────────────────────────────
// Bio > Audio > Text > Image (reliability order)
fn _modality_weight(source) {
    if source == "bio"   { return 0.50; };
    if source == "audio" { return 0.40; };
    if source == "text"  { return 0.30; };
    if source == "image" { return 0.25; };
    return 0.20;
}

// ── Helpers (explicit parens — Rust compiler precedence) ───────
fn _f_abs(x) { if x < 0 { return (0 - x); }; return x; }
fn _f_max(a, b) { if a > b { return a; }; return b; }
fn _f_min(a, b) { if a < b { return a; }; return b; }
fn _f_clamp(x, lo, hi) {
    if x < lo { return lo; };
    if x > hi { return hi; };
    return x;
}

// ════════════════════════════════════════════════════════════════
// Fusion: blend multiple modality inputs → single emotion
// ════════════════════════════════════════════════════════════════
//
// Input: array of { tag: {v,a,d,i}, confidence: f64, source: str }
// Output: { tag: {v,a,d,i}, confidence: f64, has_conflict: bool, conflict_level: f64 }

pub fn fuse(inputs) {
    let n = len(inputs);
    if n == 0 {
        return {
            tag: emotion_neutral(),
            confidence: 0.0,
            has_conflict: 0,
            conflict_level: 0.0
        };
    };
    if n == 1 {
        let inp = inputs[0];
        return {
            tag: inp.tag,
            confidence: inp.confidence,
            has_conflict: 0,
            conflict_level: 0.0
        };
    };

    // Step 1: Weighted sum
    let total_w = 0;
    let sum_v = 0; let sum_a = 0; let sum_d = 0; let sum_i = 0;
    let i = 0;
    while i < n {
        let inp = inputs[i];
        let w = (_modality_weight(inp.source)) * inp.confidence;
        sum_v = sum_v + (inp.tag.v * w);
        sum_a = sum_a + (inp.tag.a * w);
        sum_d = sum_d + (inp.tag.d * w);
        sum_i = sum_i + (inp.tag.i * w);
        total_w = total_w + w;
        let i = i + 1;
    };

    // Normalize
    let fv = 0; let fa = 0; let fd = 0.5; let fi = 0;
    if total_w > 0 {
        fv = sum_v / total_w;
        fa = sum_a / total_w;
        fd = sum_d / total_w;
        fi = sum_i / total_w;
    };

    // Step 2: Detect conflict (max |V_i - V_j|)
    let max_diff = 0;
    let ci = 0;
    while ci < n {
        let cj = ci + 1;
        while cj < n {
            let diff = _f_abs((inputs[ci].tag.v) - (inputs[cj].tag.v));
            if diff > max_diff { max_diff = diff; };
            let cj = cj + 1;
        };
        let ci = ci + 1;
    };
    let has_conflict = 0;
    let conflict_level = 0;
    if max_diff > CONFLICT_THRESHOLD {
        has_conflict = 1;
        conflict_level = max_diff;
    };

    // Step 3: Confidence = average confidence, penalized by conflict
    let conf_sum = 0;
    let ki = 0;
    while ki < n {
        conf_sum = conf_sum + inputs[ki].confidence;
        let ki = ki + 1;
    };
    let base_conf = conf_sum / n;
    if has_conflict == 1 {
        base_conf = base_conf * (1 - (conflict_level * PHI_INV));
    };
    let final_conf = _f_clamp(base_conf, 0, 1);

    // Step 4: BlackCurtain — if confidence too low, dampen emotion
    if final_conf < BLACKCURTAIN_THRESHOLD {
        fv = fv * 0.5;
        fa = fa * 0.5;
        fi = fi * 0.5;
    };

    return {
        tag: emotion_new(
            _f_clamp(fv, -1, 1),
            _f_clamp(fa, 0, 1),
            _f_clamp(fd, 0, 1),
            _f_clamp(fi, 0, 1)
        ),
        confidence: final_conf,
        has_conflict: has_conflict,
        conflict_level: conflict_level
    };
}

// ════════════════════════════════════════════════════════════════
// Simple text-only fusion (for REPL — no audio/sensor yet)
// ════════════════════════════════════════════════════════════════

pub fn fuse_text(text) {
    let emo = text_emotion(text);
    // Convert u8 range (0-7) to f64 range (-1 to 1 for V, 0 to 1 for A)
    let v_f = ((emo.v - 4) / 4);
    let a_f = (emo.a / 7);
    let tag = emotion_new(v_f, a_f, 0.5, _f_max(_f_abs(v_f), a_f));
    return {
        tag: tag,
        confidence: 0.7,
        has_conflict: 0,
        conflict_level: 0.0
    };
}
