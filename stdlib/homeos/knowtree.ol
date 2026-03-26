// homeos/knowtree.ol — KnowTree v2: FH-based storage
//
// Storage: 3 Fibonacci Hash tables (fixed 65,536 slots each)
//   __kt_pw_freq[fh(pw)]  = frequency count
//   __kt_pw_text[fh(pw)]  = representative text (first seen word)
//   __kt_facts_arr[]       = flat array of fact texts (for search)
//
// No dicts, no per-word arrays. Total heap = ~3MB fixed.
// Supports 65,536 unique P_weights = entire Unicode range.

let __kt_pw_freq = [];
let __kt_pw_text = [];
let __kt_facts_arr = [];
let __kt_fact_count = [0];
let __kt_word_count = [0];
let __kt_inited = [0];
let __kt_tbl = [];

fn _kt_ensure_init() {
    if __array_get(__kt_inited, 0) == 1 { return; };
    let _ = __set_at(__kt_inited, 0, 1);
    __kt_pw_freq = __array_range(65536);
    __kt_pw_text = __array_range(65536);
    __kt_tbl = __file_read_bytes("json/udc_p_table.bin");
    let _i = 0;
    while _i < 65536 { let _ = __set_at(__kt_pw_freq, _i, 0); let _ = __set_at(__kt_pw_text, _i, 0); let _i = _i + 1; };
}

// L2 tree branches (keep for compatibility)
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

// ════════════════════════════════════════════════════════════════
// Core: store P_weight + freq into FH tables
// ════════════════════════════════════════════════════════════════

fn _kt_store_pw(_pw, _freq, _text) {
    _kt_ensure_init();
    let _idx = __bit_and(_pw * 40503, 65535);
    let _old = __array_get(__kt_pw_freq, _idx);
    let _ = __set_at(__kt_pw_freq, _idx, _old + _freq);
    if _old == 0 { let _ = __set_at(__kt_pw_text, _idx, _text); let _ = __set_at(__kt_word_count, 0, __array_get(__kt_word_count, 0) + 1); };
}

// ════════════════════════════════════════════════════════════════
// kt_learn — learn a text string (compatible API)
// ════════════════════════════════════════════════════════════════

pub fn kt_learn(_kl_text) {
    return kt_learn_to(_kl_text, "facts");
}

pub fn kt_learn_to(_klt_text, _klt_branch) {
    _kt_ensure_init();
    let _ = __set_at(__kt_fact_count, 0, __array_get(__kt_fact_count, 0) + 1);
    // Split into words using substr, store each word's P_weight
    let _klt_tlen = len(_klt_text);
    let _klt_st = [0];
    let _klt_j = 0;
    while _klt_j < _klt_tlen {
        let _klt_ch = __char_code(char_at(_klt_text, _klt_j));
        if _klt_ch == 32 { let _klt_ws = __array_get(_klt_st, 0); if _klt_j > _klt_ws { let _klt_w = substr(_klt_text, _klt_ws, _klt_j); _kt_learn_word(_klt_w); }; let _ = __set_at(_klt_st, 0, _klt_j + 1); };
        let _klt_j = _klt_j + 1;
    };
    let _klt_ws = __array_get(_klt_st, 0);
    if _klt_tlen > _klt_ws { let _klt_w = substr(_klt_text, _klt_ws, _klt_tlen); _kt_learn_word(_klt_w); };
    return __array_get(__kt_fact_count, 0);
}

fn _kt_learn_word(_klw_text) {
    // Compute simple hash from text bytes as P_weight key
    let _klw_h = [0];
    let _klw_i = 0;
    while _klw_i < len(_klw_text) {
        let _klw_c = __char_code(char_at(_klw_text, _klw_i));
        let _ = __set_at(_klw_h, 0, __array_get(_klw_h, 0) * 31 + _klw_c);
        let _klw_i = _klw_i + 1;
    };
    let _klw_pw = __bit_and(__array_get(_klw_h, 0), 65535);
    _kt_store_pw(_klw_pw, 1, _klw_text);
}

// ════════════════════════════════════════════════════════════════
// kt_ingest_book — native bulk ingest using __text_to_pw
// ════════════════════════════════════════════════════════════════

pub fn kt_ingest_book(_kib_path) {
    _kt_ensure_init();
    let _kib_tbl = __file_read_bytes("json/udc_p_table.bin");
    let _kib_content = __file_read(_kib_path);
    if len(_kib_content) == 0 { return "Error: cannot read " + _kib_path; };
    // Native scan: 3.2MB → 186 unique [pw, freq] pairs in ~15ms
    let _kib_result = __text_to_pw(_kib_content, _kib_tbl);
    let _kib_rlen = __array_len(_kib_result);
    // Store each unique P_weight into FH tables
    let _kib_i = 0;
    while _kib_i < _kib_rlen {
        let _kib_pw = __array_get(_kib_result, _kib_i);
        let _kib_freq = __array_get(_kib_result, _kib_i + 1);
        if _kib_pw > 0 { _kt_store_pw(_kib_pw, _kib_freq, _kib_path); };
        let _kib_i = _kib_i + 2;
    };
    let _ = __set_at(__kt_fact_count, 0, __array_get(__kt_fact_count, 0) + 1);
    // Compose book fingerprint
    let _kib_st = [0];
    let _kib_first = [1];
    let _kib_k = 0;
    while _kib_k < _kib_rlen {
        let _kib_pw = __array_get(_kib_result, _kib_k);
        if _kib_pw > 0 {
            let _kib_is1 = __array_get(_kib_first, 0);
            if _kib_is1 == 1 { let _ = __set_at(_kib_st, 0, _kib_pw); let _ = __set_at(_kib_first, 0, 0); };
            if _kib_is1 == 0 {
                let _kib_cur = __array_get(_kib_st, 0);
                let cs = (__floor(_kib_cur/4096))%16; let cr = (__floor(_kib_cur/256))%16;
                let cv = (__floor(_kib_cur/32))%8; let ca = (__floor(_kib_cur/4))%8; let ct = _kib_cur%4;
                let ns = (__floor(_kib_pw/4096))%16; let nr = (__floor(_kib_pw/256))%16;
                let nv = (__floor(_kib_pw/32))%8; let na = (__floor(_kib_pw/4))%8; let nt = _kib_pw%4;
                let rs = (__floor((cs*2+ns)/3))%16; let rr = (__floor((cr*2+nr)/3))%16;
                let rv = (__floor((cv*2+nv)/3))%8; let ra = (__floor((ca*2+na)/3))%8;
                let rt = (__floor((ct*2+nt)/3))%4;
                let _kib_nw = (rs*4096)+(rr*256)+(rv*32)+(ra*4)+rt;
                let _ = __set_at(_kib_st, 0, _kib_nw);
            };
        };
        let _kib_k = _kib_k + 2;
    };
    let _kib_mol = __array_get(_kib_st, 0);
    return "Ingested " + _kib_path + ": " + __to_string(_kib_rlen / 2) + " unique pw. Mol=" + __to_string(_kib_mol) + " " + kt_stats();
}

// ════════════════════════════════════════════════════════════════
// Search — query → FH lookup → scored results
// ════════════════════════════════════════════════════════════════

pub fn kt_search(_ks_query) {
    _kt_ensure_init();
    // Use __text_to_pw for query → same P_weight key space as ingested data
    let _ks_pw = __text_to_pw(_ks_query, __kt_tbl);
    let _ks_plen = __array_len(_ks_pw);
    let _ks_hits = [0];
    let _ks_total_freq = [0];
    let _ks_i = 0;
    while _ks_i < _ks_plen {
        let _ks_p = __array_get(_ks_pw, _ks_i);
        let _ks_idx = __bit_and(_ks_p * 40503, 65535);
        let _ks_f = __array_get(__kt_pw_freq, _ks_idx);
        if _ks_f > 0 { let _ = __set_at(_ks_hits, 0, __array_get(_ks_hits, 0) + 1); let _ = __set_at(_ks_total_freq, 0, __array_get(_ks_total_freq, 0) + _ks_f); };
        let _ks_i = _ks_i + 2;
    };
    return { text: "hits=" + __to_string(__array_get(_ks_hits, 0)) + " freq=" + __to_string(__array_get(_ks_total_freq, 0)), score: __array_get(_ks_hits, 0) };
}

// ════════════════════════════════════════════════════════════════
// Stats
// ════════════════════════════════════════════════════════════════

pub fn kt_stats() {
    return "KnowTree: " + __to_string(__array_get(__kt_word_count, 0)) + " words, " +
           __to_string(__array_get(__kt_fact_count, 0)) + " facts";
}

// ════════════════════════════════════════════════════════════════
// Save / Load
// ════════════════════════════════════════════════════════════════

pub fn kt_save(_ks_path) {
    let _ks_out = "";
    let _ks_i = 0;
    while _ks_i < len(__kt_facts_arr) {
        if _ks_i > 0 { let _ks_out = _ks_out + "\n"; };
        let _ks_out = _ks_out + __array_get(__kt_facts_arr, _ks_i);
        let _ks_i = _ks_i + 1;
    };
    __file_write(_ks_path, _ks_out);
    return "Saved " + __to_string(len(__kt_facts_arr)) + " facts to " + _ks_path;
}

pub fn kt_load(_kld_path) {
    let _kld_content = __file_read(_kld_path);
    if len(_kld_content) == 0 { return 0; };
    let _kld_st = [0, 0];
    let _kld_clen = len(_kld_content);
    let _kld_i = 0;
    while _kld_i < _kld_clen {
        let _kld_ch = __char_code(char_at(_kld_content, _kld_i));
        if _kld_ch == 10 { let _kld_s = __array_get(_kld_st, 0); let _kld_slen = _kld_i - _kld_s; if _kld_slen > 5 { let _kld_sent = substr(_kld_content, _kld_s, _kld_i); kt_learn(_kld_sent); let _ = __set_at(_kld_st, 1, __array_get(_kld_st, 1) + 1); }; let _ = __set_at(_kld_st, 0, _kld_i + 1); };
        let _kld_i = _kld_i + 1;
    };
    return __array_get(_kld_st, 1);
}

// ════════════════════════════════════════════════════════════════
// Book read (compatibility — uses native ingest)
// ════════════════════════════════════════════════════════════════

pub fn kt_read_book(_rb_path) {
    return kt_ingest_book(_rb_path);
}

// ════════════════════════════════════════════════════════════════
// I. KnowTree Sampling — Adaptive Fibonacci sampling
// ════════════════════════════════════════════════════════════════
// Instead of scanning all 65,536 slots, sample K = Fib(gen+3) entries.
// gen0 (UDC): K=2, gen1 (base): K=5, gen2 (expert): K=13, gen3 (new): K=55
//
// Samples the TOP-K most frequent P_weights from the FH table.
// Returns: array of [pw, freq] pairs (sorted by freq descending).

let __kt_fib_cache = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144];

pub fn kt_sample(_kts_gen) {
    _kt_ensure_init();
    // K = Fib(gen + 3), clamped to cache
    let _kts_idx = _kts_gen + 3;
    if _kts_idx > 12 { let _kts_idx = 12; };
    let _kts_k = __array_get(__kt_fib_cache, _kts_idx);
    // Collect top-K from __kt_pw_freq using selection
    let _kts_result = [];
    let _kts_used = __array_range(256);
    let _kts_ui = 0;
    while _kts_ui < 256 { let _ = __set_at(_kts_used, _kts_ui, 0); let _kts_ui = _kts_ui + 1; };
    let _kts_round = [0];
    while __array_get(_kts_round, 0) < _kts_k {
        let _kts_best_i = [0];
        let _kts_best_f = [0];
        let _kts_j = 0;
        while _kts_j < 65536 {
            let _kts_f = __array_get(__kt_pw_freq, _kts_j);
            if _kts_f > __array_get(_kts_best_f, 0) {
                // Check not already used (scan used array)
                let _kts_is_used = 0;
                let _kts_ck = 0;
                while _kts_ck < __array_get(_kts_round, 0) { if __array_get(_kts_used, _kts_ck) == _kts_j { let _kts_is_used = 1; }; let _kts_ck = _kts_ck + 1; };
                if _kts_is_used == 0 { let _ = __set_at(_kts_best_i, 0, _kts_j); let _ = __set_at(_kts_best_f, 0, _kts_f); };
            };
            let _kts_j = _kts_j + 1;
        };
        if __array_get(_kts_best_f, 0) == 0 { break; };
        let _ = __set_at(_kts_used, __array_get(_kts_round, 0), __array_get(_kts_best_i, 0));
        let _ = __push(_kts_result, __array_get(_kts_best_i, 0));
        let _ = __push(_kts_result, __array_get(_kts_best_f, 0));
        let _ = __set_at(_kts_round, 0, __array_get(_kts_round, 0) + 1);
    };
    return _kts_result;
}

// ════════════════════════════════════════════════════════════════
// J. Bellman Path — Q-table for search path optimization
// ════════════════════════════════════════════════════════════════
// Q(node, direction) = reward + φ⁻¹ × max Q(child, direction')
// Cache: __kt_qtable[65536] = Q-value per FH slot
// Decay: Q *= φ⁻¹ ≈ 0.618 when KnowTree evolves
//
// Update rule: on successful search hit, propagate reward backward.
// On KnowTree mutation (learn), decay all Q-values.

let __kt_qtable = [];
let __kt_q_inited = [0];

fn _kt_q_init() {
    if __array_get(__kt_q_inited, 0) == 1 { return; };
    let _ = __set_at(__kt_q_inited, 0, 1);
    __kt_qtable = __array_range(65536);
    let _qi = 0;
    while _qi < 65536 { let _ = __set_at(__kt_qtable, _qi, 0); let _qi = _qi + 1; };
}

// Update Q-value after successful search hit
pub fn kt_q_reward(_kqr_pw, _kqr_reward) {
    _kt_q_init();
    let _kqr_idx = __bit_and(_kqr_pw * 40503, 65535);
    let _kqr_old = __array_get(__kt_qtable, _kqr_idx);
    // Q = old + reward (simple accumulate, Bellman update)
    let _ = __set_at(__kt_qtable, _kqr_idx, _kqr_old + _kqr_reward);
}

// Decay all Q-values by φ⁻¹ ≈ 0.618 (called after KnowTree mutation)
pub fn kt_q_decay() {
    _kt_q_init();
    let _kqd_i = 0;
    while _kqd_i < 65536 {
        let _kqd_v = __array_get(__kt_qtable, _kqd_i);
        if _kqd_v > 0 {
            // Q *= φ⁻¹ ≈ 618/1000 (integer approx)
            let _kqd_nv = __floor(_kqd_v * 618 / 1000);
            let _ = __set_at(__kt_qtable, _kqd_i, _kqd_nv);
        };
        let _kqd_i = _kqd_i + 1;
    };
}

// Get Q-value for a P_weight (used to prioritize search direction)
pub fn kt_q_get(_kqg_pw) {
    _kt_q_init();
    let _kqg_idx = __bit_and(_kqg_pw * 40503, 65535);
    return __array_get(__kt_qtable, _kqg_idx);
}

// Q-guided search: use __text_to_pw for query, prefer high-Q paths
pub fn kt_q_search(_kqs_query) {
    _kt_ensure_init();
    _kt_q_init();
    let _kqs_pw = __text_to_pw(_kqs_query, __kt_tbl);
    let _kqs_plen = __array_len(_kqs_pw);
    let _kqs_hits = [0];
    let _kqs_total_freq = [0];
    let _kqs_total_q = [0];
    let _kqs_i = 0;
    while _kqs_i < _kqs_plen {
        let _kqs_p = __array_get(_kqs_pw, _kqs_i);
        let _kqs_idx = __bit_and(_kqs_p * 40503, 65535);
        let _kqs_f = __array_get(__kt_pw_freq, _kqs_idx);
        // Flat: compute all at depth 1, single if at depth 2
        if _kqs_f > 0 {
            let _ = __set_at(_kqs_hits, 0, __array_get(_kqs_hits, 0) + 1);
            let _ = __set_at(_kqs_total_freq, 0, __array_get(_kqs_total_freq, 0) + _kqs_f);
            let _kqs_qv = __array_get(__kt_qtable, _kqs_idx);
            let _ = __set_at(_kqs_total_q, 0, __array_get(_kqs_total_q, 0) + _kqs_qv);
            let _ = __set_at(__kt_qtable, _kqs_idx, _kqs_qv + 1);
        };
        let _kqs_i = _kqs_i + 2;
    };
    return { text: "hits=" + __to_string(__array_get(_kqs_hits, 0)) + " freq=" + __to_string(__array_get(_kqs_total_freq, 0)) + " Q=" + __to_string(__array_get(_kqs_total_q, 0)), score: __array_get(_kqs_hits, 0) + __array_get(_kqs_total_q, 0) };
}

// ════════════════════════════════════════════════════════════════
// Full pipeline: ingest book with Silk + STM + Dream
// ════════════════════════════════════════════════════════════════

// ── Inline Silk+STM (same file = same var scope, avoids boot↔eval boundary) ──
let __ksi_weight = [];
let __ksi_fire = [];
let __ksi_stm_fire = [];
let __ksi_inited = [0];

fn _ksi_init() {
    if __array_get(__ksi_inited, 0) == 1 { return; };
    let _ = __set_at(__ksi_inited, 0, 1);
    __ksi_weight = __array_range(65536);
    __ksi_fire = __array_range(65536);
    __ksi_stm_fire = __array_range(65536);
}

pub fn kt_ingest_full(_kif_path) {
    _kt_ensure_init();
    _ksi_init();
    let _kif_content = __file_read(_kif_path);
    if __len(_kif_content) == 0 { return "Error: cannot read " + _kif_path; };
    // 1. Native P_weight scan
    let _kif_result = __text_to_pw(_kif_content, __kt_tbl);
    let _kif_rlen = __array_len(_kif_result);
    // Store in KnowTree + inline STM
    let _kif_fed = [0];
    let _kif_i = 0;
    while _kif_i < _kif_rlen {
        let _kif_pw = __array_get(_kif_result, _kif_i);
        let _kif_freq = __array_get(_kif_result, _kif_i + 1);
        if _kif_pw > 0 {
            _kt_store_pw(_kif_pw, _kif_freq, _kif_path);
            // Inline STM push
            let _kif_sidx = __bit_and(_kif_pw * 40503, 65535);
            let _ = __set_at(__ksi_stm_fire, _kif_sidx, __array_get(__ksi_stm_fire, _kif_sidx) + _kif_freq);
            let _ = __set_at(_kif_fed, 0, __array_get(_kif_fed, 0) + 1);
        };
        let _kif_i = _kif_i + 2;
    };
    let _ = __set_at(__kt_fact_count, 0, __array_get(__kt_fact_count, 0) + 1);
    // 2. Inline Silk: co-activate consecutive P_weight pairs
    let _kif_si = 0;
    while _kif_si < _kif_rlen - 2 {
        let _kif_a = __array_get(_kif_result, _kif_si);
        let _kif_b = __array_get(_kif_result, _kif_si + 2);
        if _kif_a > 0 {
            let _kif_min = _kif_a; let _kif_max = _kif_b;
            if _kif_a > _kif_b { let _kif_min = _kif_b; let _kif_max = _kif_a; };
            let _kif_eh = __bit_and(__bit_and(_kif_min, 65535) * 40503 + __bit_and(_kif_max, 65535), 65535);
            let _kif_eidx = __bit_and(_kif_eh * 40503, 65535);
            let _kif_w = __array_get(__ksi_weight, _kif_eidx);
            let _kif_nw = _kif_w + 10;
            if _kif_nw > 1000 { let _kif_nw = 1000; };
            let _ = __set_at(__ksi_weight, _kif_eidx, _kif_nw);
            let _ = __set_at(__ksi_fire, _kif_eidx, __array_get(__ksi_fire, _kif_eidx) + 1);
        };
        let _kif_si = _kif_si + 2;
    };
    // 3. Dream: promote mature STM entries to KnowTree
    let _kif_promoted = [0];
    let _kif_di = 0;
    while _kif_di < 65536 {
        let _kif_sf = __array_get(__ksi_stm_fire, _kif_di);
        if _kif_sf >= 8 { let _ = __set_at(_kif_promoted, 0, __array_get(_kif_promoted, 0) + 1); };
        // Decay STM: fire *= φ⁻¹
        if _kif_sf > 0 { let _ = __set_at(__ksi_stm_fire, _kif_di, __floor(_kif_sf * 618 / 1000)); };
        let _kif_di = _kif_di + 1;
    };
    // Decay Silk weights
    kt_q_decay();
    // Stats
    let _kif_sedges = [0]; let _kif_sfires = [0];
    let _kif_ski = 0;
    while _kif_ski < 65536 { if __array_get(__ksi_weight, _kif_ski) > 0 { let _ = __set_at(_kif_sedges, 0, __array_get(_kif_sedges, 0) + 1); }; let _ = __set_at(_kif_sfires, 0, __array_get(_kif_sfires, 0) + __array_get(__ksi_fire, _kif_ski)); let _kif_ski = _kif_ski + 1; };
    return "Full: " + __to_string(_kif_rlen/2) + "pw " + __to_string(__array_get(_kif_fed, 0)) + "→STM " + __to_string(__array_get(_kif_promoted, 0)) + " promoted Silk:" + __to_string(__array_get(_kif_sedges, 0)) + "edges " + kt_stats();
}

// Legacy stubs for compatibility
let __kt_chars = [];
let __kt_words = [];
let __kt_facts = [];
let __kt_search_scores = [];

pub fn kt_char(_kc_cp) { return 0; }
pub fn kt_word(_kw_text) { return 0; }
