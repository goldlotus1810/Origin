// homeos/classify.ol — Input Classifier + Handlers (HomeOS v1.0)
//
// Gate truoc, tra loi sau. Handcode == Zero.
// classify(input) → type → router → handler → output
//
// Types: math, code, command, greeting, question, chat, goodbye, empty
// NOTE: All functions boot-compiled (Rust builder). No inline builtins (contains/split).

// ════════════════════════════════════════════════════════════════
// String utilities (boot-compatible)
// ════════════════════════════════════════════════════════════════

// Substring check (case-sensitive, boot-compatible)
fn _cl_has(_ch_text, _ch_word) {
    let _ch_tl = len(_ch_text);
    let _ch_wl = len(_ch_word);
    if _ch_wl > _ch_tl { return 0; };
    let _ch_i = 0;
    while _ch_i <= (_ch_tl - _ch_wl) {
        let _ch_m = 1;
        let _ch_j = 0;
        while _ch_j < _ch_wl {
            if char_at(_ch_text, _ch_i + _ch_j) != char_at(_ch_word, _ch_j) {
                _ch_m = 0; break;
            };
            let _ch_j = _ch_j + 1;
        };
        if _ch_m == 1 { return 1; };
        let _ch_i = _ch_i + 1;
    };
    return 0;
}

// Case-insensitive substring check
fn _cl_lower(_clc_code) {
    if _clc_code >= 65 { if _clc_code <= 90 { return _clc_code + 32; }; };
    return _clc_code;
}

pub fn str_has_ci(_shc_text, _shc_word) {
    let _shc_tl = len(_shc_text);
    let _shc_wl = len(_shc_word);
    if _shc_wl > _shc_tl { return 0; };
    let _shc_i = 0;
    while _shc_i <= (_shc_tl - _shc_wl) {
        let _shc_m = 1;
        let _shc_j = 0;
        while _shc_j < _shc_wl {
            let _shc_tc = _cl_lower(__char_code(char_at(_shc_text, _shc_i + _shc_j)));
            let _shc_wc = _cl_lower(__char_code(char_at(_shc_word, _shc_j)));
            if _shc_tc != _shc_wc { _shc_m = 0; break; };
            let _shc_j = _shc_j + 1;
        };
        if _shc_m == 1 { return 1; };
        let _shc_i = _shc_i + 1;
    };
    return 0;
}

// Get first word (up to first space)
fn _cl_first_word(_cfw_text) {
    let _cfw_r = "";
    let _cfw_i = 0;
    while _cfw_i < len(_cfw_text) {
        let _cfw_c = __char_code(char_at(_cfw_text, _cfw_i));
        if _cfw_c == 32 { return _cfw_r; };
        _cfw_r = _cfw_r + char_at(_cfw_text, _cfw_i);
        let _cfw_i = _cfw_i + 1;
    };
    return _cfw_r;
}

// ════════════════════════════════════════════════════════════════
// Classifier
// ════════════════════════════════════════════════════════════════

pub fn classify(_cf_input) {
    let _cf_s = __str_trim(_cf_input);
    let _cf_len = len(_cf_s);
    if _cf_len == 0 { return "empty"; };

    // Pre-compute BEFORE any function calls (boot global var safety)
    let _cf_first = __char_code(char_at(_cf_s, 0));
    let _cf_last_idx = _cf_len - 1;
    let _cf_last = __char_code(char_at(_cf_s, _cf_last_idx));
    let _cf_has_q = 0;
    if _cf_last == 63 { _cf_has_q = 1; };

    // Starts with digit + has operator → math (inline check, no function call)
    if _cf_first >= 48 {
        if _cf_first <= 57 {
            let _cf_mi = 0;
            let _cf_is_math = 0;
            while _cf_mi < len(_cf_s) {
                let _cf_mc = __char_code(char_at(_cf_s, _cf_mi));
                if _cf_mc == 43 { _cf_is_math = 1; };  // +
                if _cf_mc == 42 { _cf_is_math = 1; };  // *
                if _cf_mc == 47 { _cf_is_math = 1; };  // /
                if _cf_mc == 45 { _cf_is_math = 1; };  // -
                let _cf_mi = _cf_mi + 1;
            };
            if _cf_is_math == 1 { return "math"; };
            if _cf_last == 63 { return "math"; };  // "42?"
        };
    };

    // First word → code/command
    let _cf_w0 = _cl_first_word(_cf_s);

    // Code keywords
    if _cf_w0 == "let" { return "code"; };
    if _cf_w0 == "fn" { return "code"; };
    if _cf_w0 == "emit" { return "code"; };
    if _cf_w0 == "if" { return "code"; };
    if _cf_w0 == "while" { return "code"; };
    if _cf_w0 == "for" { return "code"; };
    if _cf_w0 == "match" { return "code"; };
    if _cf_w0 == "type" { return "code"; };
    if _cf_w0 == "union" { return "code"; };
    if _cf_w0 == "try" { return "code"; };

    // Commands
    if _cf_w0 == "learn" { return "command"; };
    if _cf_w0 == "respond" { return "command"; };
    if _cf_w0 == "encode" { return "command"; };
    if _cf_w0 == "compile" { return "command"; };
    if _cf_w0 == "save" { return "command"; };
    if _cf_w0 == "load" { return "command"; };
    if _cf_w0 == "test" { return "command"; };
    if _cf_w0 == "build" { return "command"; };
    if _cf_w0 == "memory" { return "command"; };
    if _cf_w0 == "fns" { return "command"; };
    if _cf_w0 == "help" { return "command"; };
    if _cf_w0 == "exit" { return "command"; };
    if _cf_w0 == "quit" { return "command"; };

    // Greetings (short text, first-word check)
    if len(_cf_s) <= 15 {
        if _cf_w0 == "hi" { return "greeting"; };
        if _cf_w0 == "Hi" { return "greeting"; };
        if _cf_w0 == "hello" { return "greeting"; };
        if _cf_w0 == "Hello" { return "greeting"; };
        if _cf_w0 == "hey" { return "greeting"; };
        if _cf_w0 == "Hey" { return "greeting"; };
        if _cf_w0 == "chao" { return "greeting"; };
        if _cf_w0 == "Chao" { return "greeting"; };
        if _cf_w0 == "xin" { return "greeting"; };
        if _cf_w0 == "Xin" { return "greeting"; };
        if _cf_w0 == "yo" { return "greeting"; };
        if _cf_w0 == "bye" { return "goodbye"; };
        if _cf_w0 == "Bye" { return "goodbye"; };
        if _cf_w0 == "tam" { return "goodbye"; };
    };

    // Question mark → question
    if _cf_has_q == 1 { return "question"; };

    // Default → chat
    return "chat";
}

// ════════════════════════════════════════════════════════════════
// Handlers
// ════════════════════════════════════════════════════════════════

pub fn eval_math(_em_input) {
    let _em_s = __str_trim(_em_input);
    let _em_last = __char_code(char_at(_em_s, len(_em_s) - 1));
    if _em_last == 63 { _em_s = __substr(_em_s, 0, len(_em_s) - 1); };
    if _em_last == 61 { _em_s = __substr(_em_s, 0, len(_em_s) - 1); };
    if _em_last == 33 { _em_s = __substr(_em_s, 0, len(_em_s) - 1); };
    return "emit " + __str_trim(_em_s);
}

pub fn smart_greet(_sg_count) {
    if _sg_count == 0 {
        return "Xin chao! Minh la HomeOS. Ban muon lam gi hom nay?";
    };
    return "Chao ban! Minh o day.";
}

pub fn smart_goodbye(_sgb_count) {
    if _sgb_count == 0 { return "Tam biet!"; };
    return "Hen gap lai! Cam on ban da tro chuyen.";
}

pub fn ask_back() {
    return "Minh chua hieu. Ban muon hoi gi?";
}
