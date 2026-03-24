// stdlib/homeos/encoder.ol — Text → Molecule Encoder (OL.1)
//
// Converts text input into MolecularChain representation.
// Uses ASCII-level character encoding + LCA composition.
//
// Pipeline: text → words → char molecules → word chain → sentence chain
//
// Dependencies: mol.ol (mol_new, shape, relation, valence, arousal, time)

// ════════════════════════════════════════════════════════════════
// ASCII → Molecule lookup (inline, no UCD table needed)
// ════════════════════════════════════════════════════════════════
//
// Each ASCII char maps to a 5D coordinate:
//   Letters: S=SPHERE(0), R=MEMBER(0), V=4(neutral), A=4(neutral), T=MEDIUM(2)
//   Digits:  S=BOX(1), R=MEMBER(0), V=4, A=4, T=STATIC(0)
//   Space:   S=PLANE(3), R=ORTHOGONAL(3), V=4, A=0(calm), T=STATIC(0)
//   Punct:   S=CONE(6), R=CAUSES(5), V varies, A varies, T varies
//
// Default: S=SPHERE(0), R=MEMBER(0), V=4, A=4, T=MEDIUM(2)

pub fn encode_char(ch) {
    let code = __char_code(ch);

    // Space/whitespace
    if code == 32 { return mol_new(3, 3, 4, 0, 0); };

    // Digits 0-9: Box shape, static time
    if code >= 48 && code <= 57 {
        return mol_new(1, 0, 4, 4, 0);
    };

    // Uppercase A-Z: Sphere, slightly higher arousal
    if code >= 65 && code <= 90 {
        return mol_new(0, 0, 4, 5, 2);
    };

    // Lowercase a-z: Sphere, neutral
    if code >= 97 && code <= 122 {
        return mol_new(0, 0, 4, 4, 2);
    };

    // Punctuation with semantic meaning
    if code == 33 { return mol_new(6, 5, 6, 7, 3); };  // ! → excited, positive
    if code == 63 { return mol_new(6, 5, 4, 6, 3); };  // ? → curious, high arousal
    if code == 46 { return mol_new(3, 3, 4, 1, 0); };  // . → calm, static
    if code == 44 { return mol_new(3, 4, 4, 2, 1); };  // , → pause, compose
    if code == 59 { return mol_new(3, 4, 4, 2, 1); };  // ; → pause, compose
    if code == 58 { return mol_new(3, 5, 4, 3, 1); };  // : → causal
    if code == 45 { return mol_new(2, 3, 3, 2, 1); };  // - → negative lean
    if code == 43 { return mol_new(2, 4, 5, 4, 2); };  // + → positive, compose
    if code == 42 { return mol_new(4, 4, 5, 5, 2); };  // * → emphasis
    if code == 47 { return mol_new(6, 3, 4, 3, 2); };  // / → divide
    if code == 40 { return mol_new(7, 4, 4, 3, 1); };  // ( → group open
    if code == 41 { return mol_new(7, 4, 4, 3, 1); };  // ) → group close
    if code == 34 { return mol_new(7, 0, 5, 3, 1); };  // " → quote
    if code == 39 { return mol_new(7, 0, 4, 2, 1); };  // ' → quote light

    // Default: neutral sphere
    return mol_new(0, 0, 4, 4, 2);
}

// ════════════════════════════════════════════════════════════════
// LCA Composition (proper amplify, not average)
// ════════════════════════════════════════════════════════════════
//
// S: max(Sa, Sb) — union of shapes (keep most complex)
// R: (Ra + Rb) / 2 — blend relations
// V: amplify(Va, Vb) — bias toward dominant (NOT average)
// A: max(Aa, Ab) — keep highest arousal
// T: dominant(Ta, Tb) — keep more active time

fn _max(a, b) { if a > b { return a; }; return b; }
fn _min(a, b) { if a < b { return a; }; return b; }
fn _abs(x) { if x < 0 { return 0 - x; }; return x; }

// Amplify: boost toward dominant valence (not average)
// If both positive → more positive. If mixed → toward stronger.
fn _amplify_v(va, vb) {
    let neutral = 4;
    let da = _abs(va - neutral);
    let db = _abs(vb - neutral);
    // Pick the one further from neutral, then nudge further
    if da >= db {
        let boost = da + 1;
        if va >= neutral { return _min(7, neutral + boost); };
        return _max(0, neutral - boost);
    } else {
        let boost = db + 1;
        if vb >= neutral { return _min(7, neutral + boost); };
        return _max(0, neutral - boost);
    };
}

pub fn mol_compose(a, b) {
    let sa = shape(a);    let sb = shape(b);
    let ra = relation(a); let rb = relation(b);
    let va = valence(a);  let vb = valence(b);
    let aa = arousal(a);  let ab = arousal(b);
    let ta = time(a);     let tb = time(b);

    return mol_new(
        _max(sa, sb),              // S: union
        __floor((ra + rb) / 2),    // R: blend
        _amplify_v(va, vb),        // V: amplify (NOT average)
        _max(aa, ab),              // A: max arousal
        _max(ta, tb)               // T: dominant time
    );
}

// Compose N molecules via fold
pub fn mol_compose_many(mols) {
    if len(mols) == 0 { return mol_default(); };
    let result = mols[0];
    let i = 1;
    while i < len(mols) {
        result = mol_compose(result, mols[i]);
        i = i + 1;
    };
    return result;
}

// ════════════════════════════════════════════════════════════════
// Word Encoding
// ════════════════════════════════════════════════════════════════

// Encode a single word → molecule (LCA of char molecules)
pub fn encode_word(word) {
    let mols = [];
    let i = 0;
    while i < len(word) {
        let ch = char_at(word, i);
        push(mols, encode_char(ch));
        i = i + 1;
    };
    return mol_compose_many(mols);
}

// ════════════════════════════════════════════════════════════════
// Text Encoding (main entry point for OL.1)
// ════════════════════════════════════════════════════════════════

// Split text by spaces → array of words
fn _split_words(text) {
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
        i = i + 1;
    };
    if len(current) > 0 {
        push(words, current);
    };
    return words;
}

// Encode text → MolecularChain (array of u16 molecules)
// Each word becomes 1 molecule. Sentence = chain of word molecules.
pub fn encode_text(text) {
    let words = _split_words(text);
    let chain = [];
    let i = 0;
    while i < len(words) {
        push(chain, encode_word(words[i]));
        i = i + 1;
    };
    return chain;
}

// Encode text → single representative molecule (LCA of all words)
pub fn encode_text_single(text) {
    let chain = encode_text(text);
    return mol_compose_many(chain);
}

// ════════════════════════════════════════════════════════════════
// Emotion extraction (simple heuristic for now)
// ════════════════════════════════════════════════════════════════

// Extract emotion from text → { valence, arousal }
// Simple: based on punctuation and exclamation patterns
pub fn text_emotion(text) {
    let v = 4;  // neutral
    let a = 4;  // neutral

    let i = 0;
    while i < len(text) {
        let code = __char_code(char_at(text, i));
        if code == 33 { a = _min(7, a + 1); v = _min(7, v + 1); };  // ! → more excited/positive
        if code == 63 { a = _min(7, a + 1); };                        // ? → more aroused
        if code == 46 { a = _max(0, a - 1); };                        // . → calmer
        i = i + 1;
    };

    return { valence: v, arousal: a };
}

// ════════════════════════════════════════════════════════════════
// Convenience: full encode pipeline
// ════════════════════════════════════════════════════════════════

// Full encode: text → { chain, emotion, molecule }
pub fn encode(text) {
    let chain = encode_text(text);
    let emotion = text_emotion(text);
    let molecule = mol_compose_many(chain);
    return {
        chain: chain,
        emotion: emotion,
        molecule: molecule
    };
}
