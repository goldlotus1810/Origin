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

// L2 tree: each branch = { name, children, facts, mol }
// children = array of sub-nodes (cây con)
// facts = array of fact_ids (lá ở tầng cuối)
let __kt_tree = [];

fn _kt_boot_tree() {
    push(__kt_tree, { name: "facts", children: [], facts: [], mol: 0 });
    push(__kt_tree, { name: "books", children: [], facts: [], mol: 0 });
    push(__kt_tree, { name: "conversations", children: [], facts: [], mol: 0 });
    push(__kt_tree, { name: "skills", children: [], facts: [], mol: 0 });
    push(__kt_tree, { name: "personal", children: [], facts: [], mol: 0 });
}

fn kt_sub_branch(_ksb_parent_idx, _ksb_name) {
    let _ksb_node = { name: _ksb_name, children: [], facts: [], mol: 0 };
    push(__kt_tree[_ksb_parent_idx].children, _ksb_node);
    return len(__kt_tree[_ksb_parent_idx].children) - 1;
}

pub fn kt_learn(_kl_text) {
    return kt_learn_to(_kl_text, "facts");
}

pub fn kt_learn_to(_klt_text, _klt_branch) {
    let _klt_fid = _kt_learn_branch(_klt_text, _klt_branch);
    // Attach to tree branch
    let _klt_bi = 0;
    while _klt_bi < len(__kt_tree) {
        if __kt_tree[_klt_bi].name == _klt_branch {
            push(__kt_tree[_klt_bi].facts, _klt_fid);
            return _klt_fid;
        };
        let _klt_bi = _klt_bi + 1;
    };
    // Default: attach to facts[0]
    if len(__kt_tree) > 0 { push(__kt_tree[0].facts, _klt_fid); };
    return _klt_fid;
}

fn _kt_learn_branch(_kl_text, _kl_branch) {
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
    // Find word node — case-insensitive via _a_has (inline lowercase)
    let _ksw_wi = -1;
    let _ksw_i = 0;
    while _ksw_i < len(__kt_words) {
        if _a_has(__kt_words[_ksw_i].text, _ksw_word) == 1 {
            if len(__kt_words[_ksw_i].text) == len(_ksw_word) {
                _ksw_wi = _ksw_i; break;
            };
        };
        let _ksw_i = _ksw_i + 1;
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
    let _kst_f = 0; let _kst_b = 0; let _kst_c = 0;
    if len(__kt_tree) > 0 { _kst_f = len(__kt_tree[0].facts); };
    if len(__kt_tree) > 1 { _kst_b = len(__kt_tree[1].facts); };
    if len(__kt_tree) > 2 { _kst_c = len(__kt_tree[2].facts); };
    return "KnowTree: " + __to_string(len(__kt_words)) + " words, " +
           __to_string(len(__kt_facts)) + " facts (" +
           "F:" + __to_string(_kst_f) +
           " B:" + __to_string(_kst_b) +
           " C:" + __to_string(_kst_c) + ")";
}

// ════════════════════════════════════════════════════════════════
// Save / Load — persistent KnowTree
// ════════════════════════════════════════════════════════════════

pub fn kt_save(_ks_path) {
    let _ks_out = "";
    let _ks_i = 0;
    while _ks_i < len(__kt_facts) {
        if _ks_i > 0 { _ks_out = _ks_out + "\n"; };
        _ks_out = _ks_out + __kt_facts[_ks_i].text;
        let _ks_i = _ks_i + 1;
    };
    __file_write(_ks_path, _ks_out);
    return "Saved " + __to_string(len(__kt_facts)) + " facts to " + _ks_path;
}

pub fn kt_load(_kl_path) {
    let _kl_content = __file_read(_kl_path);
    if len(_kl_content) == 0 { return 0; };
    let _kl_sent = "";
    let _kl_count = 0;
    let _kl_i = 0;
    while _kl_i < len(_kl_content) {
        let _kl_ch = __char_code(char_at(_kl_content, _kl_i));
        if _kl_ch == 10 {
            if len(_kl_sent) > 5 {
                if _kl_count < 30 { kt_learn(_kl_sent); };
                _kl_count = _kl_count + 1;
            };
            _kl_sent = "";
        } else {
            _kl_sent = _kl_sent + char_at(_kl_content, _kl_i);
        };
        let _kl_i = _kl_i + 1;
    };
    if len(_kl_sent) > 5 {
        if _kl_count < 30 { kt_learn(_kl_sent); };
        _kl_count = _kl_count + 1;
    };
    return _kl_count;
}

// ════════════════════════════════════════════════════════════════
// Book ingest — read file → paragraphs → sentences → words → nodes
// Hierarchy: book → paragraph → sentence → word → char
// Each level = node. Silk connects within level.
// ════════════════════════════════════════════════════════════════

pub fn kt_read_book(_rb_path) {
    let _rb_content = __file_read(_rb_path);
    if len(_rb_content) == 0 { return "Error: cannot read " + _rb_path; };
    // Create book sub-node under L2:books (index 1)
    if len(__kt_tree) > 1 {
        kt_sub_branch(1, _rb_path);
    };
    // Split into sentences (by newline or period+space)
    let _rb_sent = "";
    let _rb_count = 0;
    let _rb_prev_fact = -1;
    let _rb_i = 0;
    while _rb_i < len(_rb_content) {
        let _rb_ch = __char_code(char_at(_rb_content, _rb_i));
        let _rb_split = 0;
        if _rb_ch == 10 { _rb_split = 1; };  // newline
        if _rb_ch == 46 {                      // period
            if (_rb_i + 1) < len(_rb_content) {
                let _rb_next = __char_code(char_at(_rb_content, _rb_i + 1));
                if _rb_next == 32 { _rb_split = 1; };  // ". "
                if _rb_next == 10 { _rb_split = 1; };  // ".\n"
            };
        };
        if _rb_split == 1 {
            if len(_rb_sent) > 10 {
                // Skip comment lines
                if __char_code(char_at(_rb_sent, 0)) != 35 {  // not #
                    let _rb_fid = len(__kt_facts);
                    kt_learn_to(_rb_sent, "books");
                    // Silk: connect consecutive sentences (implicit paragraph structure)
                    if _rb_prev_fact >= 0 {
                        if _rb_fid < len(__kt_facts) {
                            // Co-activate words from consecutive sentences
                            _kt_silk_sentences(_rb_prev_fact, _rb_fid);
                        };
                    };
                    _rb_prev_fact = _rb_fid;
                    _rb_count = _rb_count + 1;
                };
            };
            _rb_sent = "";
        } else {
            _rb_sent = _rb_sent + char_at(_rb_content, _rb_i);
        };
        let _rb_i = _rb_i + 1;
    };
    // Last sentence
    if len(_rb_sent) > 10 {
        if __char_code(char_at(_rb_sent, 0)) != 35 {
            kt_learn(_rb_sent);
            _rb_count = _rb_count + 1;
        };
    };
    return "Read " + _rb_path + ": " + __to_string(_rb_count) + " sentences. " + kt_stats();
}

fn _kt_silk_sentences(_kss_f1, _kss_f2) {
    // Cross-sentence Silk: connect shared words between consecutive sentences
    if _kss_f1 >= len(__kt_facts) { return; };
    if _kss_f2 >= len(__kt_facts) { return; };
    let _kss_w1 = __kt_facts[_kss_f1].words;
    let _kss_w2 = __kt_facts[_kss_f2].words;
    // For each word in sentence 1, check if also in sentence 2
    let _kss_i = 0;
    while _kss_i < len(_kss_w1) {
        let _kss_j = 0;
        while _kss_j < len(_kss_w2) {
            if _kss_w1[_kss_i] == _kss_w2[_kss_j] {
                // Same word in both sentences → strong Silk connection
                let _kss_wtext = __kt_words[_kss_w1[_kss_i]].text;
                silk_co_activate(_kss_wtext, _kss_wtext, "read");
            };
            let _kss_j = _kss_j + 1;
        };
        let _kss_i = _kss_i + 1;
    };
}
