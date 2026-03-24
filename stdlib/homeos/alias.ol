// homeos/alias.ol — Vietnamese text normalization (alias system)
//
// input → alias_normalize → cleaned text
// output ← alias_output ← formatted text
//
// Handles: slang, abbreviations, common misspellings, emoji shortcodes

// ════════════════════════════════════════════════════════════════
// Vietnamese slang → standard mapping
// ════════════════════════════════════════════════════════════════
// Stored as flat arrays: _alias_from[i] maps to _alias_to[i]

let _alias_from = [
    "ko", "k", "dc", "dk", "bn", "mk", "ms", "nc",
    "ng", "ntn", "trc", "j", "gj", "r", "ck",
    "vs", "bt", "ik", "nyc", "okla", "vn",
    "hk", "hok", "hem", "kg", "kh",
    "zui", "bik", "bít", "wá", "wa"
];
let _alias_to = [
    "khong", "khong", "duoc", "duoc", "ban", "minh", "mat", "noi chuyen",
    "nguoi", "nhu the nao", "truoc", "gi", "gi", "roi", "con",
    "voi", "binh thuong", "it", "nhu yeu cau", "ok la", "viet nam",
    "khong", "khong", "khong", "khong", "khong",
    "vui", "biet", "biet", "qua", "qua"
];

// Emoji shortcodes → emoji (common text shortcuts)
let _alias_emo_from = [":)", ":(", ":D", ":'(", "<3", ":P", "^^", "xD", ":o", "T_T"];
let _alias_emo_to = [
    "😊", "😢", "😄", "😭", "❤", "😋", "😊", "😆", "😮", "😭"
];

// ════════════════════════════════════════════════════════════════
// Forward: input → normalized text
// ════════════════════════════════════════════════════════════════

pub fn alias_normalize(_an_text) {
    // Step 1: Replace emoji shortcodes
    let _an_result = _an_text;
    let _an_ei = 0;
    while _an_ei < len(_alias_emo_from) {
        _an_result = _alias_replace_all(_an_result, _alias_emo_from[_an_ei], _alias_emo_to[_an_ei]);
        let _an_ei = _an_ei + 1;
    };

    // Step 2: Word-level slang replacement
    // Split into words, check each, rejoin
    let _an_words = [];
    let _an_w = "";
    let _an_i = 0;
    while _an_i < len(_an_result) {
        let _an_ch = char_at(_an_result, _an_i);
        if __char_code(_an_ch) == 32 {
            if len(_an_w) > 0 {
                push(_an_words, _alias_map_word(_an_w));
                _an_w = "";
            };
        } else {
            _an_w = _an_w + _an_ch;
        };
        let _an_i = _an_i + 1;
    };
    if len(_an_w) > 0 { push(_an_words, _alias_map_word(_an_w)); };

    // Rejoin
    let _an_out = "";
    let _an_j = 0;
    while _an_j < len(_an_words) {
        if _an_j > 0 { _an_out = _an_out + " "; };
        _an_out = _an_out + _an_words[_an_j];
        let _an_j = _an_j + 1;
    };
    return _an_out;
}

fn _alias_map_word(_amw_word) {
    // Lowercase comparison (simple: only check exact match)
    let _amw_i = 0;
    while _amw_i < len(_alias_from) {
        if _alias_from[_amw_i] == _amw_word {
            return _alias_to[_amw_i];
        };
        let _amw_i = _amw_i + 1;
    };
    return _amw_word;
}

fn _alias_replace_all(_ara_text, _ara_find, _ara_repl) {
    // Simple substring replace (first occurrence only for shortcodes)
    let _ara_tlen = len(_ara_text);
    let _ara_flen = len(_ara_find);
    if _ara_flen > _ara_tlen { return _ara_text; };
    let _ara_i = 0;
    while _ara_i <= (_ara_tlen - _ara_flen) {
        let _ara_match = 1;
        let _ara_j = 0;
        while _ara_j < _ara_flen {
            if char_at(_ara_text, _ara_i + _ara_j) != char_at(_ara_find, _ara_j) {
                _ara_match = 0;
                break;
            };
            let _ara_j = _ara_j + 1;
        };
        if _ara_match == 1 {
            // Found → replace
            let _ara_before = "";
            if _ara_i > 0 { _ara_before = __substr(_ara_text, 0, _ara_i); };
            let _ara_after = "";
            if (_ara_i + _ara_flen) < _ara_tlen {
                _ara_after = __substr(_ara_text, _ara_i + _ara_flen, _ara_tlen);
            };
            return _ara_before + _ara_repl + _ara_after;
        };
        let _ara_i = _ara_i + 1;
    };
    return _ara_text;
}

// ════════════════════════════════════════════════════════════════
// Reverse: internal → output-friendly text
// ════════════════════════════════════════════════════════════════

pub fn alias_output(_ao_text) {
    // Currently pass-through — future: format for display
    return _ao_text;
}
