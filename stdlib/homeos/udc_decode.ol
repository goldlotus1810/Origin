// homeos/udc_decode.ol — UDC Decoder + Output Emoji
//
// Reverse pipeline: DN/QR → UDC decode → emoji → alias → output
//
// UDC decode: molecule dimensions → human-readable description
// Output emoji: emotion state → appropriate emoji for response

// ════════════════════════════════════════════════════════════════
// UDC Decoder — molecule → meaning description
// ════════════════════════════════════════════════════════════════
// Molecule: [S:4][R:4][V:3][A:3][T:2] = 16 bits
// S = Shape (0-15), R = Relation (0-15), V = Valence (0-7)
// A = Arousal (0-7), T = Time (0-3)

pub fn udc_decode_mol(_udm_mol) {
    let _udm_s = __floor(_udm_mol / 4096) % 16;
    let _udm_r = __floor(_udm_mol / 256) % 16;
    let _udm_v = __floor(_udm_mol / 32) % 8;
    let _udm_a = __floor(_udm_mol / 4) % 8;
    let _udm_t = _udm_mol % 4;

    let _udm_shape = _decode_shape(_udm_s);
    let _udm_rel = _decode_relation(_udm_r);
    let _udm_val = _decode_valence(_udm_v);
    let _udm_aro = _decode_arousal(_udm_a);
    let _udm_time = _decode_time(_udm_t);

    return {
        shape: _udm_shape,
        relation: _udm_rel,
        valence: _udm_val,
        arousal: _udm_aro,
        time: _udm_time,
        mood: _decode_mood(_udm_v, _udm_a)
    };
}

fn _decode_shape(s) {
    if s == 0 { return "point"; };
    if s == 1 { return "sphere"; };
    if s == 2 { return "cube"; };
    if s == 3 { return "cylinder"; };
    if s == 4 { return "triangle"; };
    if s == 5 { return "circle"; };
    if s == 6 { return "star"; };
    if s == 7 { return "diamond"; };
    if s == 8 { return "cross"; };
    return "shape_" + __to_string(s);
}

fn _decode_relation(r) {
    if r == 0 { return "identity"; };
    if r == 1 { return "member"; };
    if r == 2 { return "contains"; };
    if r == 3 { return "adjacent"; };
    if r == 4 { return "parallel"; };
    if r == 5 { return "opposing"; };
    if r == 6 { return "causes"; };
    return "rel_" + __to_string(r);
}

fn _decode_valence(v) {
    if v <= 1 { return "very_negative"; };
    if v == 2 { return "negative"; };
    if v == 3 { return "slightly_negative"; };
    if v == 4 { return "neutral"; };
    if v == 5 { return "slightly_positive"; };
    if v == 6 { return "positive"; };
    return "very_positive";
}

fn _decode_arousal(a) {
    if a <= 1 { return "very_calm"; };
    if a == 2 { return "calm"; };
    if a == 3 { return "relaxed"; };
    if a == 4 { return "neutral"; };
    if a == 5 { return "alert"; };
    if a == 6 { return "excited"; };
    return "very_excited";
}

fn _decode_time(t) {
    if t == 0 { return "slow"; };
    if t == 1 { return "medium"; };
    if t == 2 { return "fast"; };
    return "instant";
}

fn _decode_mood(v, a) {
    // Valence × Arousal → mood label (Russell's circumplex)
    // High V + High A = excited/happy
    // High V + Low A = calm/content
    // Low V + High A = angry/anxious
    // Low V + Low A = sad/depressed
    if v >= 6 {
        if a >= 5 { return "excited"; };
        if a >= 3 { return "happy"; };
        return "content";
    };
    if v >= 4 {
        if a >= 5 { return "alert"; };
        if a >= 3 { return "calm"; };
        return "relaxed";
    };
    if v >= 2 {
        if a >= 5 { return "anxious"; };
        if a >= 3 { return "sad"; };
        return "melancholy";
    };
    if a >= 5 { return "angry"; };
    if a >= 3 { return "distressed"; };
    return "depressed";
}

// ════════════════════════════════════════════════════════════════
// Output Emoji — emotion → emoji for response
// ════════════════════════════════════════════════════════════════
// Maps V/A coordinates to most appropriate emoji

pub fn emoji_for_emotion(_efe_v, _efe_a) {
    // Very positive
    if _efe_v >= 6 {
        if _efe_a >= 6 { return "😄"; };
        if _efe_a >= 4 { return "😊"; };
        return "😌";
    };
    // Slightly positive
    if _efe_v >= 5 {
        if _efe_a >= 5 { return "🙂"; };
        return "😊";
    };
    // Neutral
    if _efe_v >= 3 {
        if _efe_a >= 6 { return "😮"; };
        if _efe_a >= 4 { return "🤔"; };
        return "😐";
    };
    // Negative
    if _efe_v >= 2 {
        if _efe_a >= 5 { return "😰"; };
        return "😢";
    };
    // Very negative
    if _efe_a >= 6 { return "😡"; };
    if _efe_a >= 4 { return "😭"; };
    return "😔";
}

// Emoji for intent type
pub fn emoji_for_intent(_efi_intent) {
    if _efi_intent == "heal" { return "💙"; };
    if _efi_intent == "learn" { return "📚"; };
    if _efi_intent == "technical" { return "🔧"; };
    if _efi_intent == "command" { return "⚡"; };
    if _efi_intent == "creative" { return "✨"; };
    return "💬";
}

// ════════════════════════════════════════════════════════════════
// Full decode: molecule → human description string
// ════════════════════════════════════════════════════════════════

pub fn udc_describe(_udd_mol) {
    let _udd_dec = udc_decode_mol(_udd_mol);
    return _udd_dec.mood + " (" + _udd_dec.valence + "/" + _udd_dec.arousal + ") " + _udd_dec.shape;
}
