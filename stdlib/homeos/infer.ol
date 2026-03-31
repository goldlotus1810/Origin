// homeos/infer.ol — Context inference (OL.2/OL.3)
//
// Detects: role (1st/3rd person), source (real/past/fiction),
// recency (how fresh), shared emotion, expected emotion.
//
// Used to modulate emotion intensity based on immediacy.

// ════════════════════════════════════════════════════════════════
// Role detection
// ════════════════════════════════════════════════════════════════

fn _contains(text, word) {
    let tlen = len(text);
    let wlen = len(word);
    if wlen > tlen { return 0; };
    let i = 0;
    while i <= (tlen - wlen) {
        let match = 1;
        let j = 0;
        while j < wlen {
            if char_at(text, i + j) != char_at(word, j) {
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

pub fn infer_context(text) {
    let role = "observer";
    let source = "real_now";
    let recency = 0.5;
    let shared = 0;
    let expected = 0;

    // ── First person (tôi/I/mình) ──
    if _contains(text, "toi") == 1 { role = "first"; };
    if _contains(text, "minh") == 1 { role = "first"; };
    if _contains(text, " I ") == 1 { role = "first"; };
    if _contains(text, "my ") == 1 { role = "first"; };

    // ── Third person (bạn/anh/chị/they) ──
    if _contains(text, "ban toi") == 1 { role = "third"; };
    if _contains(text, "anh ay") == 1 { role = "third"; };
    if _contains(text, "they") == 1 { role = "third"; };
    if _contains(text, "friend") == 1 { role = "third"; };

    // ── Past (hồi đó/when I was/remember) ──
    if _contains(text, "hoi do") == 1 { source = "real_past"; recency = 0.3; };
    if _contains(text, "hoi nho") == 1 { source = "real_past"; recency = 0.2; };
    if _contains(text, "remember") == 1 { source = "memory"; recency = 0.5; };
    if _contains(text, "nho") == 1 { source = "memory"; recency = 0.5; };

    // ── Fiction (phim/truyện/movie/book) ──
    if _contains(text, "phim") == 1 { source = "fiction"; recency = 0.4; };
    if _contains(text, "truyen") == 1 { source = "fiction"; recency = 0.4; };
    if _contains(text, "movie") == 1 { source = "fiction"; recency = 0.4; };
    if _contains(text, "book") == 1 { source = "fiction"; recency = 0.5; };

    // ── Music ──
    if _contains(text, "nhac") == 1 { source = "music"; recency = 0.9; };
    if _contains(text, "song") == 1 { source = "music"; recency = 0.9; };
    if _contains(text, "music") == 1 { source = "music"; recency = 0.9; };

    // ── Now/immediate (vừa/đang/just/now) ──
    if _contains(text, "vua") == 1 { source = "real_now"; recency = 0.95; };
    if _contains(text, "dang") == 1 { source = "real_now"; recency = 1.0; };
    if _contains(text, "just") == 1 { source = "real_now"; recency = 0.95; };
    if _contains(text, "now") == 1 { source = "real_now"; recency = 1.0; };

    // ── Shared ──
    if _contains(text, "chung") == 1 { shared = 1; };
    if _contains(text, "cung") == 1 { shared = 1; };
    if _contains(text, "together") == 1 { shared = 1; };
    if _contains(text, " we ") == 1 { shared = 1; };

    // ── Expected ──
    if _contains(text, "chuan bi") == 1 { expected = 1; };
    if _contains(text, "expected") == 1 { expected = 1; };
    if _contains(text, "knew") == 1 { expected = 1; };

    // Recency boost for first person
    if role == "first" {
        if source == "real_now" { recency = 1.0; };
    };

    return {
        role: role,
        source: source,
        recency: recency,
        shared: shared,
        expected: expected
    };
}

// Apply context to emotion intensity
pub fn apply_context(emo, ctx) {
    let scale = ctx.recency;
    // First person → full intensity
    if ctx.role == "first" { scale = scale * 1.0; };
    // Third person → reduced
    if ctx.role == "third" { scale = scale * 0.7; };
    // Observer → further reduced
    if ctx.role == "observer" { scale = scale * 0.5; };
    // Fiction → dampened
    if ctx.source == "fiction" { scale = scale * 0.6; };
    // Expected → less surprise
    if ctx.expected == 1 { scale = scale * 0.8; };

    return emotion_new(
        emo.v * scale,
        emo.a * scale,
        emo.d,
        emo.i * scale
    );
}
