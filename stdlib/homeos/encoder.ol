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

    // ── EMOTICON sub-ranges (fine-grained V/A) ──
    // Happy: 😀😁😂🤣😃😄😅😆😊😋😎 (U+1F600-U+1F60E)
    if cp >= 0x1F600 && cp <= 0x1F60E { return _mol_pack(0, 0, 7, 6, 2); };
    // Love: 😍😘😗😙😚 (U+1F60D-U+1F61A)
    if cp >= 0x1F60D && cp <= 0x1F61A { return _mol_pack(0, 0, 7, 5, 2); };
    // Sad/Cry: 😢😥😿 (U+1F622,U+1F625)
    if cp == 0x1F622 { return _mol_pack(0, 0, 2, 4, 2); };  // 😢 crying
    if cp == 0x1F625 { return _mol_pack(0, 0, 2, 3, 2); };  // 😥 disappointed
    if cp == 0x1F62D { return _mol_pack(0, 0, 1, 6, 2); };  // 😭 loudly crying
    if cp == 0x1F629 { return _mol_pack(0, 0, 2, 5, 2); };  // 😩 weary
    if cp == 0x1F62B { return _mol_pack(0, 0, 2, 6, 2); };  // 😫 tired
    // Angry: 😠😡🤬 (U+1F620,U+1F621)
    if cp == 0x1F620 { return _mol_pack(0, 0, 1, 6, 2); };  // 😠 angry
    if cp == 0x1F621 { return _mol_pack(0, 0, 1, 7, 2); };  // 😡 pouting
    if cp == 0x1F92C { return _mol_pack(0, 0, 1, 7, 2); };  // 🤬 cursing
    // Fear/Shock: 😨😰😱 (U+1F628,U+1F630,U+1F631)
    if cp == 0x1F628 { return _mol_pack(0, 0, 2, 6, 2); };  // 😨 fearful
    if cp == 0x1F630 { return _mol_pack(0, 0, 2, 6, 2); };  // 😰 anxious
    if cp == 0x1F631 { return _mol_pack(0, 0, 2, 7, 2); };  // 😱 screaming
    // Neutral/Thinking: 🤔😐😑 (U+1F914,U+1F610,U+1F611)
    if cp == 0x1F914 { return _mol_pack(0, 0, 4, 3, 2); };  // 🤔 thinking
    if cp == 0x1F610 { return _mol_pack(0, 0, 4, 2, 2); };  // 😐 neutral
    // Heart: ❤ (U+2764)
    if cp == 0x2764 { return _mol_pack(0, 0, 7, 5, 2); };   // ❤ red heart
    // Thumbs: 👍👎
    if cp == 0x1F44D { return _mol_pack(0, 0, 6, 4, 2); };  // 👍 thumbs up
    if cp == 0x1F44E { return _mol_pack(0, 0, 2, 4, 2); };  // 👎 thumbs down
    // Fire/Star: 🔥⭐
    if cp == 0x1F525 { return _mol_pack(8, 5, 6, 7, 2); };  // 🔥 fire
    if cp == 0x2B50 { return _mol_pack(0, 0, 6, 5, 2); };   // ⭐ star
    // Remaining emoticons (fallback)
    if cp >= 0x1F600 && cp <= 0x1F64F { return _mol_pack(0, 0, 5, 5, 2); }; // Emoticons
    // ── Other symbol blocks ──
    if cp >= 0x2600 && cp <= 0x26FF { return _mol_pack(0, 0, 5, 5, 1); };  // Misc Symbols
    if cp >= 0x1F300 && cp <= 0x1F5FF { return _mol_pack(0, 0, 6, 5, 2); }; // Misc Sym+Pict
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
// UTF-8 Decoder — reconstruct full Unicode codepoint from bytes
// ════════════════════════════════════════════════════════════════
// VM stores strings as u16 molecules (0x2100|byte). Multi-byte UTF-8
// chars (emoji, Vietnamese diacritics) become separate molecules.
// This decoder reads byte sequence → full codepoint.
//
// UTF-8 layout:
//   1-byte: 0xxxxxxx                    (0x00-0x7F)
//   2-byte: 110xxxxx 10xxxxxx           (0xC0-0xDF)
//   3-byte: 1110xxxx 10xxxxxx 10xxxxxx  (0xE0-0xEF)
//   4-byte: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx (0xF0-0xF7)
// Bit masking via modulo: b & 0x3F = b % 64, b & 0x1F = b % 32, etc.

pub fn utf8_decode(_ud_text, _ud_i) {
    let _ud_n = len(_ud_text);
    if _ud_i >= _ud_n { return { cp: 0, sz: 0 }; };
    let _ud_b0 = __char_code(char_at(_ud_text, _ud_i));
    // 1-byte ASCII
    if _ud_b0 < 128 {
        return { cp: _ud_b0, sz: 1 };
    };
    // 4-byte: emoji, rare CJK (0xF0-0xF7)
    if _ud_b0 >= 240 {
        if (_ud_i + 3) >= _ud_n { return { cp: _ud_b0, sz: 1 }; };
        let _ud_b1 = __char_code(char_at(_ud_text, _ud_i + 1)) % 64;
        let _ud_b2 = __char_code(char_at(_ud_text, _ud_i + 2)) % 64;
        let _ud_b3 = __char_code(char_at(_ud_text, _ud_i + 3)) % 64;
        // cp = (b0 & 0x07)*262144 + b1*4096 + b2*64 + b3
        // Explicit parens — Rust compiler precedence bug
        let _ud_cp = ((_ud_b0 % 8) * 262144) + ((_ud_b1 * 4096) + ((_ud_b2 * 64) + _ud_b3));
        return { cp: _ud_cp, sz: 4 };
    };
    // 3-byte: Vietnamese diacritics (0x1EA0-0x1EFF), CJK, misc symbols
    if _ud_b0 >= 224 {
        if (_ud_i + 2) >= _ud_n { return { cp: _ud_b0, sz: 1 }; };
        let _ud_b1 = __char_code(char_at(_ud_text, _ud_i + 1)) % 64;
        let _ud_b2 = __char_code(char_at(_ud_text, _ud_i + 2)) % 64;
        let _ud_cp = ((_ud_b0 % 16) * 4096) + ((_ud_b1 * 64) + _ud_b2);
        return { cp: _ud_cp, sz: 3 };
    };
    // 2-byte: Latin extended, accented chars (0xC0-0xDF)
    if (_ud_i + 1) >= _ud_n { return { cp: _ud_b0, sz: 1 }; };
    let _ud_b1 = __char_code(char_at(_ud_text, _ud_i + 1)) % 64;
    let _ud_cp = ((_ud_b0 % 32) * 64) + _ud_b1;
    return { cp: _ud_cp, sz: 2 };
}

// Is this codepoint an emoji? (emoticon/symbol blocks with high V/A)
fn _is_emoji_cp(_iec_cp) {
    // Emoticons 😀-😿
    if _iec_cp >= 128512 { if _iec_cp <= 128591 { return 1; }; };
    // Misc Symbols & Pictographs 🌀-🗿
    if _iec_cp >= 127744 { if _iec_cp <= 128511 { return 1; }; };
    // Transport & Map 🚀-🛿
    if _iec_cp >= 128640 { if _iec_cp <= 128767 { return 1; }; };
    // Supplemental Symbols 🤀-🧿
    if _iec_cp >= 129280 { if _iec_cp <= 129535 { return 1; }; };
    // Misc Symbols ☀-⛿
    if _iec_cp >= 9728 { if _iec_cp <= 9983 { return 1; }; };
    // Dingbats ✀-➿
    if _iec_cp >= 9984 { if _iec_cp <= 10175 { return 1; }; };
    // Common standalone emoji
    if _iec_cp == 10084 { return 1; };  // ❤ U+2764
    if _iec_cp == 11088 { return 1; };  // ⭐ U+2B50
    return 0;
}

// ════════════════════════════════════════════════════════════════
// Text → MolecularChain (UTF-8 aware)
// ════════════════════════════════════════════════════════════════

pub fn encode_text(text) {
    let mols = [];
    let i = 0;
    let n = len(text);
    while i < n {
        let _et_dec = utf8_decode(text, i);
        let _et_cp = _et_dec.cp;
        let _et_sz = _et_dec.sz;
        if _et_sz == 0 { break; };
        if _et_cp > 32 {
            push(mols, encode_codepoint(_et_cp));
        };
        let i = i + _et_sz;
        if len(mols) >= 64 { break; };
    };
    if len(mols) == 0 { return _mol_pack(0, 0, 4, 4, 2); };
    return mol_compose_many(mols);
}

// Extract emotion directly from Unicode properties of text
// Emoji carry emotion intrinsically — no hardcoding needed
pub fn text_emotion_unicode(_teu_text) {
    let _teu_v = 4;
    let _teu_a = 4;
    let _teu_emoji_hits = 0;
    let _teu_i = 0;
    let _teu_n = len(_teu_text);
    while _teu_i < _teu_n {
        let _teu_dec = utf8_decode(_teu_text, _teu_i);
        let _teu_cp = _teu_dec.cp;
        let _teu_sz = _teu_dec.sz;
        if _teu_sz == 0 { break; };
        // Multi-byte char (non-ASCII) → check if emoji
        if _teu_cp > 127 {
            if _is_emoji_cp(_teu_cp) == 1 {
                // Use encode_codepoint → extract V/A from molecule
                let _teu_mol = encode_codepoint(_teu_cp);
                let _teu_mv = _mol_v(_teu_mol);
                let _teu_ma = _mol_a(_teu_mol);
                // Emoji V/A is in 0-7 scale, use directly
                _teu_v = _teu_mv;
                _teu_a = _teu_ma;
                _teu_emoji_hits = _teu_emoji_hits + 1;
            };
        } else {
            // ASCII: punctuation affects arousal
            if _teu_cp == 33 { _teu_a = _enc_min(7, _teu_a + 1); _teu_v = _enc_min(7, _teu_v + 1); };
            if _teu_cp == 63 { _teu_a = _enc_min(7, _teu_a + 1); };
        };
        let _teu_i = _teu_i + _teu_sz;
    };
    return { v: _teu_v, a: _teu_a, emoji_count: _teu_emoji_hits };
}

// ════════════════════════════════════════════════════════════════
// ════════════════════════════════════════════════════════════════
// Vietnamese word stemming — strip common prefixes/modifiers
// ════════════════════════════════════════════════════════════════
// "dang buon" → split → "dang" (skip) + "buon" (affect hit)
// "rat vui" → "rat" (intensifier) + "vui" (affect hit, amplified)
// These prefixes are removed before word_affect lookup.

let __vi_prefixes = ["dang", "da", "se", "cung", "van", "hay", "nay", "the"];
let __vi_intensifiers = ["rat", "qua", "lam", "cuc", "sieu", "het"];
let __vi_negators = ["khong", "chua", "ko", "kh"];

fn _vi_is_prefix(_vp_word) {
    let _vp_i = 0;
    while _vp_i < len(__vi_prefixes) {
        if __vi_prefixes[_vp_i] == _vp_word { return 1; };
        let _vp_i = _vp_i + 1;
    };
    return 0;
}

fn _vi_is_intensifier(_vi_word) {
    let _vi_i = 0;
    while _vi_i < len(__vi_intensifiers) {
        if __vi_intensifiers[_vi_i] == _vi_word { return 1; };
        let _vi_i = _vi_i + 1;
    };
    return 0;
}

fn _vi_is_negator(_vn_word) {
    let _vn_i = 0;
    while _vn_i < len(__vi_negators) {
        if __vi_negators[_vn_i] == _vn_word { return 1; };
        let _vn_i = _vn_i + 1;
    };
    return 0;
}

// Enhanced text_emotion with stemming: handles "rat buon", "khong vui", "dang lo"
pub fn text_emotion_v2(_tev_text) {
    let _tev_v = 4;
    let _tev_a = 4;
    let _tev_hits = 0;
    let _tev_negate = 0;
    let _tev_intensify = 0;
    let _tev_w = "";
    let _tev_i = 0;
    while _tev_i < len(_tev_text) {
        let _tev_ch = char_at(_tev_text, _tev_i);
        let _tev_code = __char_code(_tev_ch);
        if _tev_code == 32 {
            if len(_tev_w) >= 2 {
                // Check modifiers first
                if _vi_is_negator(_tev_w) == 1 {
                    _tev_negate = 1;
                } else {
                    if _vi_is_intensifier(_tev_w) == 1 {
                        _tev_intensify = 1;
                    } else {
                        if _vi_is_prefix(_tev_w) == 0 {
                            // Real content word → lookup affect
                            let _tev_af = word_affect(_tev_w);
                            if _tev_af.v != 4 {
                                let _tev_nv = _tev_af.v;
                                let _tev_na = _tev_af.a;
                                // Apply negation: flip valence around neutral (4)
                                if _tev_negate == 1 {
                                    _tev_nv = 8 - _tev_nv;
                                    _tev_negate = 0;
                                };
                                // Apply intensifier: push away from neutral
                                if _tev_intensify == 1 {
                                    if _tev_nv > 4 { _tev_nv = _enc_min(7, _tev_nv + 1); };
                                    if _tev_nv < 4 { _tev_nv = _enc_max(1, _tev_nv - 1); };
                                    _tev_na = _enc_min(7, _tev_na + 1);
                                    _tev_intensify = 0;
                                };
                                _tev_v = _tev_nv;
                                _tev_a = _tev_na;
                                _tev_hits = _tev_hits + 1;
                            } else {
                                // Non-affect word → reset modifiers
                                _tev_negate = 0;
                                _tev_intensify = 0;
                            };
                        };
                    };
                };
            };
            _tev_w = "";
        } else {
            // Punctuation handling
            if _tev_code == 33 { _tev_a = _enc_min(7, _tev_a + 1); _tev_v = _enc_min(7, _tev_v + 1); };
            if _tev_code == 63 { _tev_a = _enc_min(7, _tev_a + 1); };
            if _tev_code == 46 { _tev_a = _enc_max(0, _tev_a - 1); };
            _tev_w = _tev_w + _tev_ch;
        };
        let _tev_i = _tev_i + 1;
    };
    // Check last word
    if len(_tev_w) >= 2 {
        if _vi_is_prefix(_tev_w) == 0 {
            if _vi_is_intensifier(_tev_w) == 0 {
                if _vi_is_negator(_tev_w) == 0 {
                    let _tev_af = word_affect(_tev_w);
                    if _tev_af.v != 4 {
                        let _tev_nv = _tev_af.v;
                        if _tev_negate == 1 { _tev_nv = 8 - _tev_nv; };
                        if _tev_intensify == 1 {
                            if _tev_nv > 4 { _tev_nv = _enc_min(7, _tev_nv + 1); };
                            if _tev_nv < 4 { _tev_nv = _enc_max(1, _tev_nv - 1); };
                        };
                        _tev_v = _tev_nv;
                        _tev_a = _tev_af.a;
                    };
                };
            };
        };
    };
    // Fuse with Unicode/emoji emotion — emoji takes precedence over words
    let _tev_ue = text_emotion_unicode(_tev_text);
    if _tev_ue.emoji_count > 0 {
        // Emoji detected → blend: 70% emoji + 30% word (emoji is more reliable)
        if _tev_hits > 0 {
            _tev_v = __floor(( (_tev_ue.v * 7) + (_tev_v * 3) ) / 10);
            _tev_a = __floor(( (_tev_ue.a * 7) + (_tev_a * 3) ) / 10);
        } else {
            // No word hits → use emoji 100%
            _tev_v = _tev_ue.v;
            _tev_a = _tev_ue.a;
        };
    };
    return { v: _tev_v, a: _tev_a };
}

// Word affect table (minimal Vietnamese + English)
// ════════════════════════════════════════════════════════════════

pub fn word_affect(_wa_word) {
    // Vietnamese — negative
    if _wa_word == "buon" { return { v: 2, a: 2 }; };
    if _wa_word == "gian" { return { v: 1, a: 6 }; };
    if _wa_word == "so" { return { v: 2, a: 6 }; };
    if _wa_word == "ghet" { return { v: 1, a: 6 }; };
    if _wa_word == "met" { return { v: 2, a: 2 }; };
    if _wa_word == "chan" { return { v: 2, a: 1 }; };
    if _wa_word == "lo" { return { v: 2, a: 5 }; };
    if _wa_word == "dau" { return { v: 1, a: 5 }; };
    if _wa_word == "khoc" { return { v: 1, a: 4 }; };
    if _wa_word == "that" { return { v: 1, a: 3 }; };
    if _wa_word == "co" { return { v: 2, a: 3 }; };
    if _wa_word == "kho" { return { v: 2, a: 4 }; };
    if _wa_word == "nan" { return { v: 1, a: 5 }; };
    if _wa_word == "mat" { return { v: 1, a: 4 }; };
    if _wa_word == "xau" { return { v: 2, a: 3 }; };
    if _wa_word == "tuc" { return { v: 1, a: 6 }; };
    // Vietnamese — positive
    if _wa_word == "vui" { return { v: 6, a: 5 }; };
    if _wa_word == "yeu" { return { v: 7, a: 4 }; };
    if _wa_word == "thuong" { return { v: 6, a: 3 }; };
    if _wa_word == "tot" { return { v: 6, a: 3 }; };
    if _wa_word == "dep" { return { v: 6, a: 3 }; };
    if _wa_word == "gioi" { return { v: 6, a: 4 }; };
    if _wa_word == "nho" { return { v: 5, a: 3 }; };
    if _wa_word == "hanh" { return { v: 6, a: 4 }; };
    if _wa_word == "phuc" { return { v: 7, a: 3 }; };
    if _wa_word == "cam" { return { v: 5, a: 3 }; };
    if _wa_word == "on" { return { v: 5, a: 2 }; };
    if _wa_word == "hy" { return { v: 5, a: 4 }; };
    if _wa_word == "vong" { return { v: 5, a: 4 }; };
    if _wa_word == "thich" { return { v: 6, a: 4 }; };
    if _wa_word == "suong" { return { v: 7, a: 5 }; };
    if _wa_word == "tuyet" { return { v: 7, a: 6 }; };
    // Vietnamese — neutral/state
    if _wa_word == "nghi" { return { v: 4, a: 2 }; };
    if _wa_word == "biet" { return { v: 4, a: 3 }; };
    if _wa_word == "lam" { return { v: 4, a: 4 }; };
    if _wa_word == "hoc" { return { v: 5, a: 4 }; };
    if _wa_word == "doc" { return { v: 5, a: 3 }; };
    // English — negative
    if _wa_word == "sad" { return { v: 2, a: 2 }; };
    if _wa_word == "angry" { return { v: 1, a: 6 }; };
    if _wa_word == "hate" { return { v: 1, a: 6 }; };
    if _wa_word == "fear" { return { v: 2, a: 6 }; };
    if _wa_word == "bad" { return { v: 2, a: 3 }; };
    if _wa_word == "pain" { return { v: 1, a: 5 }; };
    if _wa_word == "tired" { return { v: 2, a: 2 }; };
    if _wa_word == "lonely" { return { v: 2, a: 2 }; };
    if _wa_word == "scared" { return { v: 2, a: 6 }; };
    if _wa_word == "worried" { return { v: 2, a: 5 }; };
    if _wa_word == "stressed" { return { v: 2, a: 6 }; };
    if _wa_word == "depressed" { return { v: 1, a: 1 }; };
    if _wa_word == "anxious" { return { v: 2, a: 6 }; };
    if _wa_word == "disappointed" { return { v: 2, a: 3 }; };
    if _wa_word == "frustrated" { return { v: 2, a: 5 }; };
    if _wa_word == "hurt" { return { v: 1, a: 4 }; };
    if _wa_word == "lost" { return { v: 2, a: 3 }; };
    if _wa_word == "broken" { return { v: 1, a: 3 }; };
    // English — positive
    if _wa_word == "happy" { return { v: 6, a: 5 }; };
    if _wa_word == "love" { return { v: 7, a: 4 }; };
    if _wa_word == "joy" { return { v: 6, a: 6 }; };
    if _wa_word == "good" { return { v: 5, a: 3 }; };
    if _wa_word == "great" { return { v: 6, a: 5 }; };
    if _wa_word == "wonderful" { return { v: 7, a: 5 }; };
    if _wa_word == "amazing" { return { v: 7, a: 6 }; };
    if _wa_word == "beautiful" { return { v: 6, a: 3 }; };
    if _wa_word == "excited" { return { v: 6, a: 7 }; };
    if _wa_word == "grateful" { return { v: 6, a: 3 }; };
    if _wa_word == "proud" { return { v: 6, a: 5 }; };
    if _wa_word == "hopeful" { return { v: 5, a: 4 }; };
    if _wa_word == "inspired" { return { v: 6, a: 5 }; };
    if _wa_word == "peaceful" { return { v: 5, a: 1 }; };
    if _wa_word == "calm" { return { v: 5, a: 1 }; };
    if _wa_word == "kind" { return { v: 6, a: 2 }; };
    if _wa_word == "thank" { return { v: 5, a: 2 }; };
    if _wa_word == "thanks" { return { v: 5, a: 2 }; };
    return { v: 4, a: 4 };
}

// Text → emotion { v, a } (scan words + punctuation)
pub fn text_emotion(_te_text) {
    let _te_v = 4;
    let _te_a = 4;
    let _te_hits = 0;
    // Split text into words, check each against word_affect
    let _te_w = "";
    let _te_i = 0;
    while _te_i < len(_te_text) {
        let _te_ch = char_at(_te_text, _te_i);
        let _te_code = __char_code(_te_ch);
        if _te_code == 32 {
            if len(_te_w) >= 2 {
                let _te_affect = word_affect(_te_w);
                if _te_affect.v != 4 {
                    _te_v = _te_affect.v;
                    _te_a = _te_affect.a;
                    _te_hits = _te_hits + 1;
                };
            };
            _te_w = "";
        } else {
            // Punctuation
            if _te_code == 33 { _te_a = _enc_min(7, _te_a + 1); _te_v = _enc_min(7, _te_v + 1); };
            if _te_code == 63 { _te_a = _enc_min(7, _te_a + 1); };
            if _te_code == 46 { _te_a = _enc_max(0, _te_a - 1); };
            _te_w = _te_w + _te_ch;
        };
        let _te_i = _te_i + 1;
    };
    // Check last word
    if len(_te_w) >= 2 {
        let _te_affect = word_affect(_te_w);
        if _te_affect.v != 4 {
            _te_v = _te_affect.v;
            _te_a = _te_affect.a;
        };
    };
    return { v: _te_v, a: _te_a };
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
    let emotion = text_emotion_v2(text);
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
    let emo = text_emotion_v2(text);

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

// Response templates — configurable personality
let __tpl_empathetic = "Minh hieu cam giac do.";
let __tpl_gentle = "Tu tu thoi, khong voi dau.";
let __tpl_explanatory = "De minh tim hieu cho ban.";
let __tpl_precise = "OK.";
let __tpl_confirmatory = "Da nhan.";
let __tpl_chat = "Minh nghe roi.";
let __tpl_heal = " Ban muon chia se them khong?";
let __tpl_learn = " Ban muon biet cu the dieu gi?";
let __tpl_technical = " Cho minh xem code hoac error message.";
let __tpl_command = " Dang xu ly...";
let __tpl_heal_better = " Ban co ve da on hon roi.";
let __tpl_topic_repeat = " Minh thay ban nhac lai dieu nay. Minh hieu no quan trong voi ban.";
let __tpl_remember = " (Minh nho truoc do ban noi ve: ";
let __tpl_know = "(Minh biet: ";

// Change personality: set_personality("formal") / set_personality("casual")
pub fn set_personality(style) {
    if style == "formal" {
        let __tpl_empathetic = "Toi hieu cam giac cua ban.";
        let __tpl_gentle = "Xin hay binh tinh.";
        let __tpl_explanatory = "Toi se tim hieu cho ban.";
        let __tpl_precise = "Da hieu.";
        let __tpl_confirmatory = "Da tiep nhan.";
        let __tpl_chat = "Vang, toi dang lang nghe.";
        let __tpl_heal = " Ban co muon chia se them khong?";
        let __tpl_learn = " Ban muon tim hieu dieu gi cu the?";
    };
    if style == "casual" {
        let __tpl_empathetic = "Uh, minh hieu ma.";
        let __tpl_gentle = "Chill thoi, khong sao dau.";
        let __tpl_explanatory = "De minh check cho.";
        let __tpl_precise = "OK nhe.";
        let __tpl_confirmatory = "Roger!";
        let __tpl_chat = "Yo!";
        let __tpl_heal = " Ke tiep di?";
        let __tpl_learn = " Muon biet gi nua?";
    };
    if style == "english" {
        let __tpl_empathetic = "I understand how you feel.";
        let __tpl_gentle = "Take your time.";
        let __tpl_explanatory = "Let me look into that.";
        let __tpl_precise = "Got it.";
        let __tpl_confirmatory = "Acknowledged.";
        let __tpl_chat = "I'm listening.";
        let __tpl_heal = " Want to talk more?";
        let __tpl_learn = " What specifically?";
        let __tpl_heal_better = " You seem better now.";
        let __tpl_topic_repeat = " I notice this matters to you.";
        let __tpl_remember = " (I recall you mentioned: ";
        let __tpl_know = "(I know: ";
    };
    return "Personality: " + style;
}

pub fn compose_reply(intent, tone, text) {
    let ack = "";
    if tone == "empathetic" { ack = __tpl_empathetic; };
    if tone == "gentle" { ack = __tpl_gentle; };
    if tone == "explanatory" { ack = __tpl_explanatory; };
    if tone == "precise" { ack = __tpl_precise; };
    if tone == "confirmatory" { ack = __tpl_confirmatory; };

    let followup = "";
    if intent == "heal" { followup = __tpl_heal; };
    if intent == "learn" { followup = __tpl_learn; };
    if intent == "technical" { followup = __tpl_technical; };
    if intent == "command" { followup = __tpl_command; };
    if intent == "chat" { ack = __tpl_chat; };

    return ack + followup;
}

// ════════════════════════════════════════════════════════════════
// STM — Short-Term Memory
// ════════════════════════════════════════════════════════════════
// Keeps last N exchanges. Each entry: { input, intent, tone, molecule }
// Agent can reference previous inputs for context.

let __stm = [];
let __stm_max = 32;

// ── Emotion carry-over state ──
// Running emotion: exponential moving average across turns
let __emo_v = 4;  // valence (1=neg, 4=neutral, 7=pos)
let __emo_a = 4;  // arousal (1=calm, 4=neutral, 7=excited)
let __emo_streak = 0;  // consecutive same-valence turns

fn _emo_update(new_v, new_a) {
    // EMA: 60% old + 40% new → smooth carry-over
    // MUST use explicit parens — Rust compiler precedence bug: a*b+c*d → (a*b+c)*d
    let __emo_v = __floor(( (__emo_v * 6) + (new_v * 4) ) / 10);
    let __emo_a = __floor(( (__emo_a * 6) + (new_a * 4) ) / 10);
    // Track streak: same direction (pos/neg) for 3+ turns → amplify
    if new_v >= 5 {
        if __emo_streak >= 0 { let __emo_streak = __emo_streak + 1; }
        else { let __emo_streak = 1; };
    } else {
        if new_v <= 3 {
            if __emo_streak <= 0 { let __emo_streak = __emo_streak - 1; }
            else { let __emo_streak = -1; };
        } else {
            // Neutral → decay streak toward 0
            if __emo_streak > 0 { let __emo_streak = __emo_streak - 1; };
            if __emo_streak < 0 { let __emo_streak = __emo_streak + 1; };
        };
    };
}

pub fn emo_state() {
    return { v: __emo_v, a: __emo_a, streak: __emo_streak };
}

// Emotion-aware tone override: when streak strong, bias the tone
fn _emo_bias_tone(tone) {
    // 3+ positive turns → empathetic/gentle tone
    if __emo_streak >= 3 {
        if tone == "precise" { return "gentle"; };
    };
    // 3+ negative turns → empathetic (even if analysis says precise)
    if __emo_streak <= -3 {
        return "empathetic";
    };
    return tone;
}

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

// ── Conversation digest ──
// When STM > 16 turns, compress older half into a digest string
let __stm_digest = "";
let __stm_digest_count = 0;

fn _stm_maybe_digest() {
    if len(__stm) < 16 { return; };
    // Already digested recently
    if __stm_digest_count >= len(__stm) { return; };
    // Build digest from first half of STM
    let _sd_half = __floor(len(__stm) / 2);
    let _sd_heal = 0;
    let _sd_learn = 0;
    let _sd_tech = 0;
    let _sd_chat = 0;
    let _sd_topics = "";
    let _sd_i = 0;
    while _sd_i < _sd_half {
        let _sd_entry = __stm[_sd_i];
        if _sd_entry.intent == "heal" { _sd_heal = _sd_heal + 1; };
        if _sd_entry.intent == "learn" { _sd_learn = _sd_learn + 1; };
        if _sd_entry.intent == "technical" { _sd_tech = _sd_tech + 1; };
        if _sd_entry.intent == "chat" { _sd_chat = _sd_chat + 1; };
        // Collect first word of each input as topic hints
        let _sd_fw = "";
        let _sd_fi = 0;
        while _sd_fi < len(_sd_entry.input) {
            if __char_code(char_at(_sd_entry.input, _sd_fi)) == 32 { break; };
            _sd_fw = _sd_fw + char_at(_sd_entry.input, _sd_fi);
            let _sd_fi = _sd_fi + 1;
        };
        if len(_sd_fw) > 2 {
            if len(_sd_topics) > 0 { _sd_topics = _sd_topics + ", "; };
            _sd_topics = _sd_topics + _sd_fw;
        };
        let _sd_i = _sd_i + 1;
    };
    // Build digest string
    let _sd_d = "";
    if _sd_heal > 0 { _sd_d = _sd_d + "cam-xuc(" + __to_string(_sd_heal) + ") "; };
    if _sd_learn > 0 { _sd_d = _sd_d + "hoc(" + __to_string(_sd_learn) + ") "; };
    if _sd_tech > 0 { _sd_d = _sd_d + "ky-thuat(" + __to_string(_sd_tech) + ") "; };
    if _sd_chat > 0 { _sd_d = _sd_d + "chat(" + __to_string(_sd_chat) + ") "; };
    if len(_sd_topics) > 0 { _sd_d = _sd_d + "| " + _sd_topics; };
    let __stm_digest = _sd_d;
    let __stm_digest_count = len(__stm);

    // Evict digested entries (keep second half)
    let _sd_new = [];
    let _sd_j = _sd_half;
    while _sd_j < len(__stm) {
        push(_sd_new, __stm[_sd_j]);
        let _sd_j = _sd_j + 1;
    };
    let __stm = _sd_new;
}

pub fn stm_digest() { return __stm_digest; }

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

    // EMOTION CARRY-OVER — update running state + bias tone
    let _ar_emo = text_emotion_v2(text);
    _emo_update(_ar_emo.v, _ar_emo.a);
    tone = _emo_bias_tone(tone);

    // REMEMBER (STM)
    stm_push(text, intent, tone);

    // DIGEST — compress older turns when STM gets full
    _stm_maybe_digest();

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
                memory_context = __tpl_remember + _ar_related + ")";
            };
        };
    };

    // CONTEXT — repeated topic detection
    if stm_count() >= 3 {
        if stm_topic_repeated(text, 2) == 1 {
            memory_context = __tpl_topic_repeat;
        };
    };

    // CONTEXT — heal→ok transition
    if stm_count() >= 2 {
        let _prev_idx = stm_count() - 2;
        if _prev_idx >= 0 {
            if __stm[_prev_idx].intent == "heal" {
                if intent != "heal" {
                    memory_context = __tpl_heal_better;
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
        return __tpl_know + _ks_best + ")";
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
