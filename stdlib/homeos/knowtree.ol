// homeos/knowtree.ol — KnowTree: moi thu = node, tree IS index
//
// CRITICAL: Boot functions modify heap arrays (survive scope restore).
// Do NOT use global counter vars — use len(array) instead.
// Scope restore resets var_table but NOT heap arrays.

// ════════════════════════════════════════════════════════════════
// N.1: Lazy char nodes
// ════════════════════════════════════════════════════════════════

let __kt_chars = [];        // [{cp, mol}]

pub fn kt_char(_kc_cp) {
    let _kc_i = 0;
    while _kc_i < len(__kt_chars) {
        if __kt_chars[_kc_i].cp == _kc_cp { return _kc_i; };
        let _kc_i = _kc_i + 1;
    };
    let _kc_mol = encode_codepoint(_kc_cp);
    push(__kt_chars, { cp: _kc_cp, mol: _kc_mol });
    return len(__kt_chars) - 1;
}

// ════════════════════════════════════════════════════════════════
// N.2: Word nodes — chain of char indices + fact links
// ════════════════════════════════════════════════════════════════

let __kt_words = [];        // [{text, mol, facts}]

pub fn kt_word(_kw_text) {
    let _kw_i = 0;
    while _kw_i < len(__kt_words) {
        if __kt_words[_kw_i].text == _kw_text { return _kw_i; };
        let _kw_i = _kw_i + 1;
    };
    // Compute mol from chars
    let _kw_mol = 0;
    let _kw_pos = 0;
    while _kw_pos < len(_kw_text) {
        let _kw_cp = __utf8_cp(_kw_text, _kw_pos);
        let _kw_cplen = __utf8_len(_kw_text, _kw_pos);
        let _kw_cmol = encode_codepoint(_kw_cp);
        if _kw_cp > 127 { _kw_cmol = _kw_cp % 65536; };
        if _kw_mol == 0 { _kw_mol = _kw_cmol; } else { _kw_mol = mol_compose(_kw_mol, _kw_cmol); };
        let _kw_pos = _kw_pos + _kw_cplen;
    };
    push(__kt_words, { text: _kw_text, mol: _kw_mol, facts: [] });
    return len(__kt_words) - 1;
}

// ════════════════════════════════════════════════════════════════
// N.3: Fact nodes — chain of word indices + text
// ════════════════════════════════════════════════════════════════

let __kt_facts = [];        // [{text, words, mol}]

pub fn kt_learn(_kl_text) {
    // Dedup
    let _kl_i = 0;
    while _kl_i < len(__kt_facts) {
        if __kt_facts[_kl_i].text == _kl_text { return len(__kt_facts); };
        let _kl_i = _kl_i + 1;
    };
    // Split → word nodes → compose mol
    let _kl_wids = [];
    let _kl_cur = "";
    let _kl_mol = 0;
    let _kl_j = 0;
    while _kl_j < len(_kl_text) {
        let _kl_ch = __char_code(char_at(_kl_text, _kl_j));
        if _kl_ch == 32 {
            if len(_kl_cur) > 0 {
                let _kl_wi = kt_word(_kl_cur);
                push(_kl_wids, _kl_wi);
                let _kl_wmol = __kt_words[_kl_wi].mol;
                if _kl_mol == 0 { _kl_mol = _kl_wmol; } else { _kl_mol = mol_compose(_kl_mol, _kl_wmol); };
                _kl_cur = "";
            };
        } else {
            _kl_cur = _kl_cur + char_at(_kl_text, _kl_j);
        };
        let _kl_j = _kl_j + 1;
    };
    if len(_kl_cur) > 0 {
        let _kl_wi = kt_word(_kl_cur);
        push(_kl_wids, _kl_wi);
        let _kl_wmol = __kt_words[_kl_wi].mol;
        if _kl_mol == 0 { _kl_mol = _kl_wmol; } else { _kl_mol = mol_compose(_kl_mol, _kl_wmol); };
    };
    // Create fact
    let _kl_fid = len(__kt_facts);
    push(__kt_facts, { text: _kl_text, words: _kl_wids, mol: _kl_mol });
    // Reverse links: word.facts → fact_id
    let _kl_li = 0;
    while _kl_li < len(_kl_wids) {
        push(__kt_words[_kl_wids[_kl_li]].facts, _kl_fid);
        let _kl_li = _kl_li + 1;
    };
    return len(__kt_facts);
}

// ════════════════════════════════════════════════════════════════
// N.4: Search — query words → word nodes → fact links → best
// ════════════════════════════════════════════════════════════════

pub fn kt_search(_ks_query) {
    let _ks_best = "";
    let _ks_best_score = 0;
    // Reset scores (heap array survives, zero each element)
    let _ks_ri = 0;
    while _ks_ri < len(__kt_search_scores) {
        set_at(__kt_search_scores, _ks_ri, 0);
        let _ks_ri = _ks_ri + 1;
    };
    // Score facts by matching query words
    let _ks_cur = "";
    let _ks_qi = 0;
    while _ks_qi < len(_ks_query) {
        let _ks_ch = __char_code(char_at(_ks_query, _ks_qi));
        if _ks_ch == 32 {
            if len(_ks_cur) >= 2 {
                _kt_score_word(_ks_cur);
            };
            _ks_cur = "";
        } else {
            _ks_cur = _ks_cur + char_at(_ks_query, _ks_qi);
        };
        let _ks_qi = _ks_qi + 1;
    };
    if len(_ks_cur) >= 2 {
        _kt_score_word(_ks_cur);
    };
    // Find best from __kt_search_scores
    let _ks_si = 0;
    while _ks_si < len(__kt_facts) {
        if _ks_si < len(__kt_search_scores) {
            if __kt_search_scores[_ks_si] > _ks_best_score {
                _ks_best_score = __kt_search_scores[_ks_si];
                _ks_best = __kt_facts[_ks_si].text;
            };
        };
        let _ks_si = _ks_si + 1;
    };
    return { text: _ks_best, score: _ks_best_score };
}

let __kt_search_scores = [];

fn _kt_score_word(_ksw_word) {
    // Ensure scores array matches facts
    while len(__kt_search_scores) < len(__kt_facts) {
        push(__kt_search_scores, 0);
    };
    // Find word node (exact or CI)
    let _ksw_wi = -1;
    let _ksw_i = 0;
    while _ksw_i < len(__kt_words) {
        if __kt_words[_ksw_i].text == _ksw_word { _ksw_wi = _ksw_i; break; };
        let _ksw_i = _ksw_i + 1;
    };
    // CI fallback
    if _ksw_wi < 0 {
        let _ksw_i = 0;
        while _ksw_i < len(__kt_words) {
            if _a_has(__kt_words[_ksw_i].text, _ksw_word) == 1 {
                if len(__kt_words[_ksw_i].text) == len(_ksw_word) {
                    _ksw_wi = _ksw_i; break;
                };
            };
            let _ksw_i = _ksw_i + 1;
        };
    };
    if _ksw_wi < 0 { return; };
    // Score linked facts
    let _ksw_facts = __kt_words[_ksw_wi].facts;
    let _ksw_fi = 0;
    while _ksw_fi < len(_ksw_facts) {
        let _ksw_fid = _ksw_facts[_ksw_fi];
        if _ksw_fid < len(__kt_search_scores) {
            set_at(__kt_search_scores, _ksw_fid, __kt_search_scores[_ksw_fid] + 10);
        };
        let _ksw_fi = _ksw_fi + 1;
    };
}

// ════════════════════════════════════════════════════════════════
// Stats
// ════════════════════════════════════════════════════════════════

pub fn kt_stats() {
    return "KnowTree: " + __to_string(len(__kt_chars)) + " chars, " +
           __to_string(len(__kt_words)) + " words, " +
           __to_string(len(__kt_facts)) + " facts";
}
