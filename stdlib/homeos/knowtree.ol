// homeos/knowtree.ol — KnowTree (G1+G2+G3+G5)
// REBUILT from SPEC_G_COMPLETE.md. Zero legacy code.

let __kt_tbl = [];
let __kt_inited = [0];

fn _kt_ensure_init() {
    if __array_get(__kt_inited, 0) == 1 { return; };
    let _ = __set_at(__kt_inited, 0, 1);
    __kt_tbl = __file_read_bytes("json/udc_p_table.bin");
}

// A2: Unpack P_weight
fn _kt_mol_s(_m) { return (__floor(_m / 4096)) % 16; }
fn _kt_mol_r(_m) { return (__floor(_m / 256)) % 16; }
fn _kt_mol_v(_m) { return (__floor(_m / 32)) % 8; }
fn _kt_mol_a(_m) { return (__floor(_m / 4)) % 8; }
fn _kt_mol_t(_m) { return _m % 4; }
fn _kt_abs(_v) { if _v < 0 { return 0 - _v; }; return _v; }
fn _kt_pack(_s, _r, _v, _a, _t) { return (_s * 4096) + (_r * 256) + (_v * 32) + (_a * 4) + _t; }

// G2: Distance
fn _kt_mol_dist(_a, _b) {
    return _kt_abs(_kt_mol_s(_a) - _kt_mol_s(_b)) + _kt_abs(_kt_mol_r(_a) - _kt_mol_r(_b)) + _kt_abs(_kt_mol_v(_a) - _kt_mol_v(_b)) + _kt_abs(_kt_mol_a(_a) - _kt_mol_a(_b)) + _kt_abs(_kt_mol_t(_a) - _kt_mol_t(_b));
}

// G2: Dominant dimension
pub fn mol_dominant_dim(_m) {
    let _ds = _kt_abs((_kt_mol_s(_m) * 66) - 500);
    let _dr = _kt_abs((_kt_mol_r(_m) * 66) - 500);
    let _dv = _kt_abs((_kt_mol_v(_m) * 142) - 500);
    let _da = _kt_abs((_kt_mol_a(_m) * 142) - 500);
    let _dt = _kt_abs((_kt_mol_t(_m) * 333) - 500);
    let _mx = _ds; let _di = 0;
    if _dr > _mx { let _mx = _dr; let _di = 1; };
    if _dv > _mx { let _mx = _dv; let _di = 2; };
    if _da > _mx { let _mx = _da; let _di = 3; };
    if _dt > _mx { let _di = 4; };
    return _di;
}

pub fn mol_get_dim(_m, _d) {
    if _d == 0 { return _kt_mol_s(_m); };
    if _d == 1 { return _kt_mol_r(_m); };
    if _d == 2 { return _kt_mol_v(_m); };
    if _d == 3 { return _kt_mol_a(_m); };
    return _kt_mol_t(_m);
}

pub fn mol_dim_range(_d) { if _d <= 1 { return 15; }; if _d <= 3 { return 7; }; return 3; }

// G2: Compose (A4: S=max R=Zipf V=amplify A=max T=vote)
pub fn compose(_mols) {
    let _n = len(_mols);
    if _n == 0 { return 0; };
    if _n == 1 { return __array_get(_mols, 0); };
    let _sm = [0]; let _rs = [0]; let _rw = [0]; let _vs = [0]; let _am = [0]; let _t = [0,0,0,0];
    let _i = 0;
    while _i < _n {
        let _m = __array_get(_mols, _i);
        let _w = __floor(1000 / (_i + 1));
        if _kt_mol_s(_m) > __array_get(_sm, 0) { let _ = __set_at(_sm, 0, _kt_mol_s(_m)); };
        let _ = __set_at(_rs, 0, __array_get(_rs, 0) + (_kt_mol_r(_m) * _w));
        let _ = __set_at(_rw, 0, __array_get(_rw, 0) + _w);
        let _ = __set_at(_vs, 0, __array_get(_vs, 0) + _kt_mol_v(_m));
        if _kt_mol_a(_m) > __array_get(_am, 0) { let _ = __set_at(_am, 0, _kt_mol_a(_m)); };
        let _ti = _kt_mol_t(_m); let _ = __set_at(_t, _ti, __array_get(_t, _ti) + 1);
        let _i = _i + 1;
    };
    let _S = __array_get(_sm, 0) % 16;
    let _tw = __array_get(_rw, 0); let _R = 0;
    if _tw > 0 { let _R = (__floor(__array_get(_rs, 0) / _tw)) % 16; };
    let _V = __floor(__array_get(_vs, 0) / _n); if _V > 7 { let _V = 7; }; if _V < 0 { let _V = 0; };
    let _A = __array_get(_am, 0) % 8;
    let _T = 0; let _tm = __array_get(_t, 0);
    if __array_get(_t, 1) > _tm { let _T = 1; let _tm = __array_get(_t, 1); };
    if __array_get(_t, 2) > _tm { let _T = 2; let _tm = __array_get(_t, 2); };
    if __array_get(_t, 3) > _tm { let _T = 3; };
    return _kt_pack(_S, _R, _V, _A, _T);
}

// G3: P_weight lookup
pub fn p_weight(_cp) {
    _kt_ensure_init();
    if _cp < 0 { return 0; }; if _cp >= 196608 { return 0; };
    return __floor(__bytes_get(__kt_tbl, _cp * 2) + (__bytes_get(__kt_tbl, _cp * 2 + 1) * 256));
}

// G3: Encode text → chain
pub fn chain_encode(_text) {
    let _ch = []; let _i = 0;
    while _i < len(_text) {
        let _m = p_weight(__char_code(char_at(_text, _i)));
        if _m > 0 { push(_ch, _m); };
        let _i = _i + 1;
    };
    return _ch;
}

pub fn chain_summary(_ch) { return compose(_ch); }

// Real mol from text — SEMANTIC compose (A4 rules) + hash for uniqueness WITHIN bucket
pub fn _kt_real_mol(_text) {
    _kt_ensure_init();
    let _tlen = len(_text);
    if _tlen == 0 { return 0; };

    // Compose SRVAT: S,R,T from char P_weights. V,A from CONTENT hash.
    // Hash ensures different text → different mol even with same char distribution.
    let _s_max = [0]; let _r_max = [0];
    let _t_vote = [0, 0, 0, 0];
    let _hash = [5381];

    let _i = 0;
    while _i < _tlen {
        let _cp = __char_code(char_at(_text, _i));
        let _pw = p_weight(_cp);
        let _ = __set_at(_hash, 0, __bit_and((__array_get(_hash, 0) * 33) + _cp, 65535));
        if _pw > 0 {
            let _s = (__floor(_pw / 4096)) % 16;
            let _r = (__floor(_pw / 256)) % 16;
            let _t = _pw % 4;
            if _s > __array_get(_s_max, 0) { let _ = __set_at(_s_max, 0, _s); };
            if _r > __array_get(_r_max, 0) { let _ = __set_at(_r_max, 0, _r); };
            let _ = __set_at(_t_vote, _t, __array_get(_t_vote, _t) + 1);
        };
        let _i = _i + 1;
    };

    // S, R from char compose (structure: math symbols, arrows → high S/R)
    let _S = __array_get(_s_max, 0);
    let _R = __array_get(_r_max, 0);
    // V, A from hash (differentiation: different text → different V,A bucket)
    let _h = __array_get(_hash, 0);
    let _V = (__floor(_h / 32)) % 8;
    let _A = (__floor(_h / 4)) % 8;
    // T from vote
    let _T = 0; let _tm = __array_get(_t_vote, 0);
    if __array_get(_t_vote, 1) > _tm { let _T = 1; let _tm = __array_get(_t_vote, 1); };
    if __array_get(_t_vote, 2) > _tm { let _T = 2; let _tm = __array_get(_t_vote, 2); };
    if __array_get(_t_vote, 3) > _tm { let _T = 3; };

    return _kt_pack(_S, _R, _V, _A, _T);
}

// ═══ NRC-VAD: word → emotion lookup ═══
let __nrc_vad = [];
let __nrc_vad_ok = [0];

fn _vad_init() {
    if __array_get(__nrc_vad_ok, 0) == 1 { return; };
    let _ = __set_at(__nrc_vad_ok, 0, 1);
    // 256 hash buckets, each = [word, v*1000, a*1000, word, v, a, ...]
    let _i = 0; while _i < 256 { push(__nrc_vad, []); let _i = _i + 1; };
}

fn _vad_hash(_w) {
    let _h = [0]; let _i = 0;
    while _i < len(_w) {
        let _ = __set_at(_h, 0, __bit_and((__array_get(_h, 0) * 31) + __char_code(char_at(_w, _i)), 255));
        let _i = _i + 1;
    };
    return __array_get(_h, 0);
}

// Load NRC-VAD from tab-separated file
pub fn vad_load(_path) {
    _vad_init();
    let _c = __file_read(_path);
    if len(_c) == 0 { return 0; };
    let _count = [0];
    let _start = [0];
    let _line_start = 1;  // skip header
    let _i = 0;
    while _i < len(_c) {
        if __char_code(char_at(_c, _i)) == 10 {
            if _line_start == 0 {
                let _line = substr(_c, __array_get(_start, 0), _i);
                // Parse: word\tvalence\tarousal\tdominance
                let _tab1 = __str_index_of(_line, "	");
                if _tab1 > 0 {
                    let _word = substr(_line, 0, _tab1);
                    let _rest = substr(_line, _tab1 + 1, len(_line));
                    let _tab2 = __str_index_of(_rest, "	");
                    if _tab2 > 0 {
                        let _vs = substr(_rest, 0, _tab2);
                        let _rest2 = substr(_rest, _tab2 + 1, len(_rest));
                        let _tab3 = __str_index_of(_rest2, "	");
                        let _as = _rest2;
                        if _tab3 > 0 { let _as = substr(_rest2, 0, _tab3); };
                        let _v = __to_number(_vs);
                        let _a = __to_number(_as);
                        let _h = _vad_hash(_word);
                        push(__nrc_vad[_h], _word);
                        push(__nrc_vad[_h], __floor(_v * 1000));
                        push(__nrc_vad[_h], __floor(_a * 1000));
                        let _ = __set_at(_count, 0, __array_get(_count, 0) + 1);
                    };
                };
            };
            let _line_start = 0;
            let _ = __set_at(_start, 0, _i + 1);
        };
        let _i = _i + 1;
    };
    __heap_pin();
    return __array_get(_count, 0);
}

// Query: word → [v_raw*1000, a_raw*1000] or [0, 0]
pub fn vad_query(_word) {
    _vad_init();
    let _h = _vad_hash(_word);
    let _bkt = __nrc_vad[_h];
    let _i = 0;
    while _i < len(_bkt) {
        if __array_get(_bkt, _i) == _word {
            return [__array_get(_bkt, _i + 1), __array_get(_bkt, _i + 2)];
        };
        let _i = _i + 3;
    };
    return [0, 0];
}

// ═══ STM — moved to knowtree.ol for global scope compatibility ═══
let __kt_stm_text = __array_with_cap(64);
let __kt_stm_mol = __array_with_cap(64);

pub fn kt_stm_push(_text) {
    let _mol = _kt_real_mol(_text);
    push(__kt_stm_text, _text);
    push(__kt_stm_mol, _mol);
    // Auto-silk with previous
    let _n = len(__kt_stm_mol);
    if _n >= 2 { kt_silk_fire(_mol, __array_get(__kt_stm_mol, _n - 2)); };
}

pub fn kt_stm_count() { return len(__kt_stm_text); }
pub fn kt_stm_mol_at(_i) { return __array_get(__kt_stm_mol, _i); }
pub fn kt_stm_text_at(_i) { return __array_get(__kt_stm_text, _i); }

// ═══ String split: global buffer in knowtree scope (works across boot↔eval) ═══
let __kt_words = __array_with_cap(64);

pub fn kt_split_words(_text) {
    // Clear: set len to 0 by recreating (push appends, can't shrink)
    // Workaround: use count tracker
    let _ws = [0];
    let _wc = [0];
    let _i = 0;
    while _i <= len(_text) {
        let _is_sp = 0;
        if _i == len(_text) { let _is_sp = 1; } else {
            let _ch = __char_code(char_at(_text, _i));
            if _ch == 32 { let _is_sp = 1; };
            if _ch == 63 { let _is_sp = 1; };
        };
        if _is_sp == 1 {
            let _s = __array_get(_ws, 0);
            if _i > _s {
                push(__kt_words, substr(_text, _s, _i));
                let _ = __set_at(_wc, 0, __array_get(_wc, 0) + 1);
            };
            let _ = __set_at(_ws, 0, _i + 1);
        };
        let _i = _i + 1;
    };
    return __array_get(_wc, 0);
}

pub fn kt_word_at(_idx) {
    if _idx < len(__kt_words) { return __array_get(__kt_words, _idx); };
    return "";
}

// ═══ L0 Registry: everything = node = SRVAT ═══
// Olang keywords, builtins, identity — all registered as KnowTree nodes
pub fn kt_register_l0() {
    _kt_ensure_init(); _bkt_init(); _silk_init();
    // Identity
    kt_learn("Nox la AI tu viet lai chinh minh bang Olang");
    kt_learn("Olang la ngon ngu lap trinh tu hosting");
    kt_learn("Lupin la nguoi tao ra Nox va Olang");
    kt_learn("HomeOS la he dieu hanh tri thuc sinh hoc");
    // Greetings (English + Vietnamese aliases)
    kt_learn("hello is a greeting. chao = hello. xin chao = hello");
    kt_learn("who are you? I am Nox, an AI that rewrites itself using Olang. ban la ai = who are you");
    // Language keywords (each = node with unique SRVAT)
    kt_learn("emit: output a value to stdout");
    kt_learn("let: bind a value to a variable name");
    kt_learn("fn: define a function");
    kt_learn("pub fn: define a public function");
    kt_learn("if: conditional branch");
    kt_learn("while: loop until condition false");
    kt_learn("return: exit function with value");
    kt_learn("try catch: error handling");
    kt_learn("match: pattern matching");
    kt_learn("type: define a struct type");
    kt_learn("const: immutable binding");
    // Core builtins
    kt_learn("__file_read: read file contents");
    kt_learn("__file_write: write string to file");
    kt_learn("__spawn: fork process with pipes");
    kt_learn("__system: execute shell command");
    kt_learn("__syscall: raw Linux system call");
    kt_learn("__tcp_listen: start TCP server");
    kt_learn("__tcp_accept: accept TCP connection");
    kt_learn("__tcp_send: send data over TCP");
    kt_learn("__heap_pin: move heap checkpoint forward");
    kt_learn("__eval_bytecode: evaluate compiled bytecode");
    // Math/logic
    kt_learn("compose: combine P_weights using A4 rules");
    kt_learn("p_weight: lookup P_weight for Unicode codepoint");
    kt_learn("chain_encode: text to chain of P_weights");
    kt_learn("kt_learn: add fact to KnowTree");
    kt_learn("kt_nearest: find nearest node by P_weight distance");
    kt_learn("kt_silk_fire: strengthen Hebbian edge between nodes");
    kt_learn("pipeline: process input through 14 DNA mechanisms");
    __heap_pin();
}

// ═══ Load UDC aliases JSON into KnowTree ═══
// Each alias = "U+XXXX name [vi: translation] [keywords]"
pub fn kt_load_aliases(_path) {
    _kt_ensure_init(); _bkt_init();
    let _c = __file_read(_path);
    if len(_c) < 10 { return 0; };
    // Simple JSON parse: find "XXXX":{"n":"NAME",...} patterns
    let _count = [0];
    let _i = 0;
    let _clen = len(_c);
    while _i < _clen {
        // Find "XXXX":
        if char_at(_c, _i) == "\"" {
            let _key_start = _i + 1;
            let _i = _i + 1;
            while _i < _clen {
                if char_at(_c, _i) == "\"" { break; };
                let _i = _i + 1;
            };
            let _key = substr(_c, _key_start, _i);
            let _i = _i + 1;
            // Skip to value
            if _i < _clen {
                if char_at(_c, _i) == ":" {
                    let _i = _i + 1;
                    // Find "n":"..." (name)
                    let _chunk_end = _i + 500;
                    if _chunk_end > _clen { let _chunk_end = _clen; };
                    let _n_pos = __str_index_of(substr(_c, _i, _chunk_end), "\"n\":\"");
                    if _n_pos >= 0 {
                        let _name_start = _i + _n_pos + 5;
                        let _name_end = _name_start;
                        while _name_end < _clen {
                            if char_at(_c, _name_end) == "\"" { break; };
                            let _name_end = _name_end + 1;
                        };
                        let _name = substr(_c, _name_start, _name_end);
                        // Find Vietnamese "v":"..."
                        let _ch_end = _i + 500;
                        if _ch_end > _clen { let _ch_end = _clen; };
                        let _chunk = substr(_c, _i, _ch_end);
                        let _v_pos = __str_index_of(_chunk, "\"v\":\"");
                        let _vi = "";
                        if _v_pos >= 0 {
                            let _vi_start = _i + _v_pos + 5;
                            let _vi_end = _vi_start;
                            while _vi_end < _clen {
                                if char_at(_c, _vi_end) == "\"" { break; };
                                let _vi_end = _vi_end + 1;
                            };
                            let _vi = substr(_c, _vi_start, _vi_end);
                        };
                        // Learn: "ARROW: LEFTWARDS ARROW (mũi tên hướng trái)"
                        let _fact = "U+" + _key + " " + _name;
                        if len(_vi) > 0 { let _fact = _fact + " (" + _vi + ")"; };
                        kt_learn(_fact);
                        let _ = __set_at(_count, 0, __array_get(_count, 0) + 1);
                        // Pin every 1000 to avoid heap overflow
                        if (__array_get(_count, 0) % 1000) == 0 { __heap_pin(); };
                    };
                    // Skip to next entry (find next "})
                    while _i < _clen {
                        if char_at(_c, _i) == "}" { break; };
                        let _i = _i + 1;
                    };
                };
            };
        };
        let _i = _i + 1;
    };
    __heap_pin();
    return __array_get(_count, 0);
}

// ═══ G1+G5: KnowTree bucket structure ═══
// Pre-allocate with 8192 capacity to prevent relocation crash
let __kt_facts = __array_with_cap(8192);
let __kt_facts_mol = __array_with_cap(8192);
let __kt_buckets = [];
let __kt_bkt_ok = [0];

fn _bkt_init() {
    if __array_get(__kt_bkt_ok, 0) == 1 { return; };
    let _ = __set_at(__kt_bkt_ok, 0, 1);
    let _i = 0; while _i < 256 { push(__kt_buckets, []); let _i = _i + 1; };
}

let __kt_learn_count = [0];

// Fast learn: skip word-level compose, use simple hash (10x faster)
pub fn kt_learn_fast(_text) {
    _kt_ensure_init(); _bkt_init(); _silk_init();
    // Fast mol: hash only (no per-char P_weight lookup)
    let _h = [2166136261];
    let _i = 0;
    while _i < len(_text) {
        let _ = __set_at(_h, 0, __bit_xor(__array_get(_h, 0), __char_code(char_at(_text, _i))));
        let _ = __set_at(_h, 0, __bit_and(__array_get(_h, 0) * 16777619, 65535));
        let _i = _i + 1;
    };
    let _mol = __array_get(_h, 0);
    let _idx = len(__kt_facts);
    push(__kt_facts, _text);
    push(__kt_facts_mol, _mol);
    push(__kt_buckets[(_kt_mol_s(_mol) * 16) + _kt_mol_r(_mol)], _idx);
    if _idx > 0 { kt_silk_fire(_mol, __array_get(__kt_facts_mol, _idx - 1)); };
    let _ = __set_at(__kt_learn_count, 0, __array_get(__kt_learn_count, 0) + 1);
    if (__array_get(__kt_learn_count, 0) % 20) == 0 { __heap_pin(); };
    return _idx;
}

// Raw learn: pre-computed mol, NO encoding, NO word index, NO silk = 0 temp strings
pub fn kt_learn_raw(_text, _mol) {
    _kt_ensure_init(); _bkt_init();
    let _idx = len(__kt_facts);
    push(__kt_facts, _text);
    push(__kt_facts_mol, _mol);
    push(__kt_buckets[(_kt_mol_s(_mol) * 16) + _kt_mol_r(_mol)], _idx);
    return _idx;
}

pub fn kt_learn(_text) {
    _kt_ensure_init(); _bkt_init(); _silk_init();
    let _mol = _kt_real_mol(_text);
    let _idx = len(__kt_facts);
    push(__kt_facts, _text);
    push(__kt_facts_mol, _mol);
    push(__kt_buckets[(_kt_mol_s(_mol) * 16) + _kt_mol_r(_mol)], _idx);
    // Auto-silk with previous
    if _idx > 0 { kt_silk_fire(_mol, __array_get(__kt_facts_mol, _idx - 1)); };
    // Pin BEFORE array relocation (capacity = 512)
    // Boot has 71 facts. Pin at 400 to prevent relocation crash.
    let _ = __set_at(__kt_learn_count, 0, __array_get(__kt_learn_count, 0) + 1);
    let _lc = __array_get(__kt_learn_count, 0);
    if (_lc % 50) == 0 { __heap_pin(); };
    // G20: Index words for O(1) lookup
    _widx_init();
    let _wi = 0; let _ws = [0];
    while _wi <= len(_text) {
        let _is_space = 0;
        if _wi == len(_text) { let _is_space = 1; } else {
            let _ch = __char_code(char_at(_text, _wi));
            if _ch == 32 { let _is_space = 1; };
            if _ch == 10 { let _is_space = 1; };
        };
        if _is_space == 1 {
            let _word = substr(_text, __array_get(_ws, 0), _wi);
            _widx_add(_word, _idx);
            let _ = __set_at(_ws, 0, _wi + 1);
        };
        let _wi = _wi + 1;
    };
    __heap_pin();
    return _idx;
}

pub fn kt_nearest(_mol) {
    _bkt_init();
    let _s = _kt_mol_s(_mol); let _r = _kt_mol_r(_mol);
    let _bi = [0 - 1]; let _bd = [99999];
    let _ds = 0 - 1;
    while _ds <= 1 {
        let _dr = 0 - 1;
        while _dr <= 1 {
            let _si = _s + _ds; let _ri = _r + _dr;
            if _si >= 0 { if _si < 16 { if _ri >= 0 { if _ri < 16 {
                let _bk = __kt_buckets[(_si * 16) + _ri];
                let _j = 0;
                while _j < len(_bk) {
                    let _fi = __array_get(_bk, _j);
                    let _d = _kt_mol_dist(_mol, __array_get(__kt_facts_mol, _fi));
                    if _d < __array_get(_bd, 0) { let _ = __set_at(_bd, 0, _d); let _ = __set_at(_bi, 0, _fi); };
                    let _j = _j + 1;
                };
            };};};}; let _dr = _dr + 1;
        }; let _ds = _ds + 1;
    };
    let _idx = __array_get(_bi, 0);
    if _idx < 0 { return ""; };
    return __array_get(__kt_facts, _idx);
}

// G20: Word index — O(1) word → fact indices
let __kt_widx = [];
let __kt_widx_ok = [0];

fn _widx_init() {
    if __array_get(__kt_widx_ok, 0) == 1 { return; };
    let _ = __set_at(__kt_widx_ok, 0, 1);
    let _i = 0; while _i < 256 { push(__kt_widx, []); let _i = _i + 1; };
}

fn _widx_hash(_w) {
    let _h = [0]; let _i = 0;
    while _i < len(_w) {
        let _ = __set_at(_h, 0, __bit_and((__array_get(_h, 0) * 31) + __char_code(char_at(_w, _i)), 255));
        let _i = _i + 1;
    };
    return __array_get(_h, 0);
}

fn _widx_add(_word, _fidx) {
    _widx_init();
    if len(_word) < 3 { return; };
    push(__kt_widx[_widx_hash(_word)], _fidx);
}

pub fn kt_word_lookup(_w) {
    _widx_init();
    if len(_w) < 3 { return []; };
    return __kt_widx[_widx_hash(_w)];
}

// String contains: check if haystack contains needle (pure Olang, no builtins)
pub fn _str_has(_hay, _needle) {
    let _hlen = len(_hay);
    let _nlen = len(_needle);
    if _nlen == 0 { return 1; };
    if _nlen > _hlen { return 0; };
    let _i = 0;
    while _i <= _hlen - _nlen {
        let _match = [1];
        let _j = 0;
        while _j < _nlen {
            if char_at(_hay, _i + _j) != char_at(_needle, _j) {
                let _ = __set_at(_match, 0, 0);
                let _j = _nlen;
            };
            let _j = _j + 1;
        };
        if __array_get(_match, 0) == 1 { return 1; };
        let _i = _i + 1;
    };
    return 0;
}

pub fn kt_find(_q, _max) {
    // Word index → candidates → VERIFY with _str_has
    _widx_init();
    let _indices = kt_word_lookup(_q);
    let _out = [];
    if len(_indices) > 0 {
        let _i = 0;
        while _i < len(_indices) {
            if len(_out) >= _max { return _out; };
            let _fi = __array_get(_indices, _i);
            if _fi < len(__kt_facts) {
                let _fact = __array_get(__kt_facts, _fi);
                // VERIFY: fact must actually contain query
                if _str_has(_fact, _q) { push(_out, _fact); };
            };
            let _i = _i + 1;
        };
        if len(_out) > 0 { return _out; };
    };
    // Fallback: linear scan (slow but accurate)
    let _i = 0;
    while _i < len(__kt_facts) {
        if len(_out) >= _max { return _out; };
        if _str_has(__array_get(__kt_facts, _i), _q) { push(_out, __array_get(__kt_facts, _i)); };
        let _i = _i + 1;
    };
    return _out;
}

pub fn kt_fact_count() { return len(__kt_facts); }
pub fn kt_stats() { return "KT: " + __to_string(len(__kt_facts)) + " facts"; }

// ═══ DIAGNOSTICS — see what's inside KnowTree ═══

// Full map: how many nodes per (S,R) bucket
pub fn kt_map() {
    _bkt_init();
    let _out = "KnowTree Map (S×R buckets with nodes):\n";
    let _nonempty = [0];
    let _s = 0;
    while _s < 16 {
        let _r = 0;
        while _r < 16 {
            let _n = len(__kt_buckets[(_s * 16) + _r]);
            if _n > 0 {
                let _out = _out + "  S=" + __to_string(_s) + " R=" + __to_string(_r) + ": " + __to_string(_n) + " nodes\n";
                let _ = __set_at(_nonempty, 0, __array_get(_nonempty, 0) + 1);
            };
            let _r = _r + 1;
        };
        let _s = _s + 1;
    };
    let _out = _out + "Active buckets: " + __to_string(__array_get(_nonempty, 0)) + "/256\n";
    let _out = _out + "Total nodes: " + __to_string(len(__kt_facts));
    return _out;
}

// Silk stats: how many edges, average weight
pub fn kt_silk_stats() {
    _silk_init();
    let _total_edges = [0];
    let _total_weight = [0];
    let _active_buckets = [0];
    let _hi = 0;
    while _hi < 256 {
        let _edges = __kt_silk[_hi];
        let _n = __floor(len(_edges) / 6);
        if _n > 0 {
            let _ = __set_at(_active_buckets, 0, __array_get(_active_buckets, 0) + 1);
            let _ = __set_at(_total_edges, 0, __array_get(_total_edges, 0) + _n);
            let _ei = 0;
            while _ei < len(_edges) {
                let _max_w = 0;
                let _j = 1;
                while _j <= 5 {
                    let _w = __array_get(_edges, _ei + _j);
                    if _w > _max_w { let _max_w = _w; };
                    let _j = _j + 1;
                };
                let _ = __set_at(_total_weight, 0, __array_get(_total_weight, 0) + _max_w);
                let _ei = _ei + 6;
            };
        };
        let _hi = _hi + 1;
    };
    let _te = __array_get(_total_edges, 0);
    let _avg = 0;
    if _te > 0 { let _avg = __floor(__array_get(_total_weight, 0) / _te); };
    return "Silk: " + __to_string(_te) + " edges, "
         + __to_string(__array_get(_active_buckets, 0)) + " active buckets, "
         + "avg_w=" + __to_string(_avg);
}

// Show sample facts from each bucket
pub fn kt_sample(_max_per_bucket) {
    _bkt_init();
    let _out = "";
    let _s = 0;
    while _s < 16 {
        let _r = 0;
        while _r < 16 {
            let _bkt = __kt_buckets[(_s * 16) + _r];
            if len(_bkt) > 0 {
                let _out = _out + "[S=" + __to_string(_s) + " R=" + __to_string(_r) + "] ";
                let _j = 0;
                while _j < len(_bkt) {
                    if _j >= _max_per_bucket { break; };
                    let _fi = __array_get(_bkt, _j);
                    let _text = __array_get(__kt_facts, _fi);
                    if len(_text) > 60 { let _text = substr(_text, 0, 60) + "..."; };
                    let _out = _out + _text;
                    if _j < len(_bkt) - 1 { if _j < _max_per_bucket - 1 { let _out = _out + " | "; }; };
                    let _j = _j + 1;
                };
                let _out = _out + "\n";
            };
            let _r = _r + 1;
        };
        let _s = _s + 1;
    };
    return _out;
}

// Full diagnostic
pub fn kt_diagnostic() {
    let _out = "═══ KnowTree Diagnostic ═══\n";
    let _out = _out + "Nodes: " + __to_string(len(__kt_facts)) + "\n";
    let _out = _out + kt_silk_stats() + "\n";
    let _out = _out + kt_map();
    return _out;
}
pub fn kt_search(q) { return kt_nearest(_kt_real_mol(q)); }
pub fn kt_search_n(q, n) { return kt_find(q, n); }
// Classify = the mol itself. SRVAT IS the classification.
// No labels. No limits. Position in 5D = identity.
// Similar content → similar mol → same neighborhood → group emerges.
pub fn kt_classify(_t) { return _kt_real_mol(_t); }
pub fn kt_decode(_q) { return kt_nearest(_kt_real_mol(_q)); }
pub fn kt_learn_tagged(_t, _x) { return kt_learn(_x); }
pub fn kt_learn_to(_x, _b) { return kt_learn(_x); }
// G19: Persistence — save KnowTree to disk, load at boot
pub fn kt_save(_path) {
    let _out = "";
    let _i = 0;
    while _i < len(__kt_facts) {
        let _fact = __array_get(__kt_facts, _i);
        if len(_fact) > 0 {
            let _mol = __array_get(__kt_facts_mol, _i);
            let _out = _out + __to_string(_mol) + "\t" + _fact + "\n";
        };
        let _i = _i + 1;
    };
    __file_write(_path, _out);
    return "Saved " + __to_string(len(__kt_facts)) + " to " + _path;
}

pub fn kt_load(_path) {
    _kt_ensure_init(); _bkt_init(); _silk_init();
    let _c = __file_read(_path);
    if len(_c) == 0 { return 0; };
    let _count = [0];
    let _start = [0];
    let _i = 0;
    while _i < len(_c) {
        if __char_code(char_at(_c, _i)) == 10 {
            let _line = substr(_c, __array_get(_start, 0), _i);
            // Format: mol\tfact
            let _tab = [0 - 1];
            let _j = 0;
            while _j < len(_line) {
                if __char_code(char_at(_line, _j)) == 9 {
                    let _ = __set_at(_tab, 0, _j);
                    let _j = len(_line);
                };
                let _j = _j + 1;
            };
            let _tp = __array_get(_tab, 0);
            if _tp > 0 {
                let _mol_str = substr(_line, 0, _tp);
                let _fact = substr(_line, _tp + 1, len(_line));
                let _mol = __to_number(_mol_str);
                if len(_fact) > 0 {
                    if len(__kt_facts) < 500 {
                        let _idx = len(__kt_facts);
                        push(__kt_facts, _fact);
                        push(__kt_facts_mol, _mol);
                        push(__kt_buckets[(_kt_mol_s(_mol) * 16) + _kt_mol_r(_mol)], _idx);
                        let _ = __set_at(_count, 0, __array_get(_count, 0) + 1);
                    };
                };
            };
            let _ = __set_at(_start, 0, _i + 1);
        };
        let _i = _i + 1;
    };
    __heap_pin();
    return __array_get(_count, 0);
}
pub fn kt_dim_stats() { return ""; }
// ═══ G6: Silk — Hebbian per-dimension edges ═══
// Edge = [target_mol, wS, wR, wV, wA, wT] = 6 values
// Adjacency list: __kt_silk[hash(mol)] = [edge, edge, ...]
let __kt_silk = [];
let __kt_silk_ok = [0];

fn _silk_init() {
    if __array_get(__kt_silk_ok, 0) == 1 { return; };
    let _ = __set_at(__kt_silk_ok, 0, 1);
    let _i = 0; while _i < 256 { push(__kt_silk, []); let _i = _i + 1; };
}

fn _silk_hash(_mol) { return __bit_and(_mol, 255); }

pub fn kt_silk_init() { _silk_init(); }

// G6: Hebbian fire — strengthen edge between two mols
pub fn kt_silk_fire(_a, _b) {
    _silk_init();
    let _va = _kt_mol_v(_a); let _aa = _kt_mol_a(_a);
    let _emo = ((_kt_abs(_va - 4) * _aa) + 1) / 28;
    if _emo > 1000 { let _emo = 1000; };
    // Find or create edge
    let _h = _silk_hash(_a);
    let _edges = __kt_silk[_h];
    let _found = [0 - 1];
    let _ei = 0;
    while _ei < len(_edges) {
        if __array_get(_edges, _ei) == _b { let _ = __set_at(_found, 0, _ei); };
        let _ei = _ei + 6;
    };
    let _fi = __array_get(_found, 0);
    if _fi < 0 {
        // New edge
        push(_edges, _b);
        push(_edges, _emo); push(_edges, _emo); push(_edges, _emo);
        push(_edges, _emo); push(_edges, _emo);
    } else {
        // Update existing: w += emo * (1000 - w) / 10000
        let _j = 1;
        while _j <= 5 {
            let _w = __array_get(_edges, _fi + _j);
            let _dw = (_emo * (1000 - _w)) / 10000;
            let _ = __set_at(_edges, _fi + _j, _w + _dw);
            let _j = _j + 1;
        };
    };
}

// G6: Get max silk weight between two mols
pub fn kt_silk_weight(_a, _b) {
    _silk_init();
    let _edges = __kt_silk[_silk_hash(_a)];
    let _ei = 0;
    while _ei < len(_edges) {
        if __array_get(_edges, _ei) == _b {
            let _max = 0; let _j = 1;
            while _j <= 5 {
                let _w = __array_get(_edges, _ei + _j);
                if _w > _max { let _max = _w; };
                let _j = _j + 1;
            };
            return _max;
        };
        let _ei = _ei + 6;
    };
    return 0;
}

// G6: Silk walk — follow strongest edges on dominant dimension
pub fn kt_silk_walk(_start_mol, _depth, _threshold) {
    _silk_init(); _bkt_init();
    let _dim = mol_dominant_dim(_start_mol);
    let _path = [_start_mol];
    let _cur = _start_mol;
    let _d = 0;
    while _d < _depth {
        // Find best neighbor on _dim
        let _edges = __kt_silk[_silk_hash(_cur)];
        let _best_mol = [0]; let _best_w = [0];
        let _ei = 0;
        while _ei < len(_edges) {
            let _target = __array_get(_edges, _ei);
            let _w = __array_get(_edges, _ei + 1 + _dim);
            if _w > __array_get(_best_w, 0) {
                let _ = __set_at(_best_w, 0, _w);
                let _ = __set_at(_best_mol, 0, _target);
            };
            let _ei = _ei + 6;
        };
        // If no Hebbian edge, find nearest DIFFERENT node in same bucket
        if __array_get(_best_w, 0) < _threshold {
            let _s = _kt_mol_s(_cur);
            let _r = _kt_mol_r(_cur);
            let _bkt = __kt_buckets[(_s * 16) + _r];
            let _bd = [99999]; let _bm = [0];
            let _bi = 0;
            while _bi < len(_bkt) {
                let _fi = __array_get(_bkt, _bi);
                let _fm = __array_get(__kt_facts_mol, _fi);
                if _fm != _cur {
                    // Check not already in path
                    let _in_path = [0]; let _pi = 0;
                    while _pi < len(_path) {
                        if __array_get(_path, _pi) == _fm { let _ = __set_at(_in_path, 0, 1); };
                        let _pi = _pi + 1;
                    };
                    if __array_get(_in_path, 0) == 0 {
                        let _dd = _kt_mol_dist(_cur, _fm);
                        if _dd < __array_get(_bd, 0) {
                            let _ = __set_at(_bd, 0, _dd);
                            let _ = __set_at(_bm, 0, _fm);
                        };
                    };
                };
                let _bi = _bi + 1;
            };
            if __array_get(_bm, 0) > 0 {
                let _ = __set_at(_best_mol, 0, __array_get(_bm, 0));
            };
        };
        let _next = __array_get(_best_mol, 0);
        if _next == 0 { return _path; };
        if _next == _cur { return _path; };
        push(_path, _next);
        let _cur = _next;
        let _d = _d + 1;
    };
    return _path;
}

// G6: Decay all silk edges by φ⁻¹
pub fn kt_silk_decay() {
    _silk_init();
    let _hi = 0;
    while _hi < 256 {
        let _edges = __kt_silk[_hi];
        let _ei = 0;
        while _ei < len(_edges) {
            let _j = 1;
            while _j <= 5 {
                let _w = __array_get(_edges, _ei + _j);
                let _ = __set_at(_edges, _ei + _j, __floor(_w * 618 / 1000));
                let _j = _j + 1;
            };
            let _ei = _ei + 6;
        };
        let _hi = _hi + 1;
    };
}
pub fn kt_word_lookup(_w) { return []; }
pub fn kt_get_dim(_d, _v) { return []; }
pub fn kt_get_path(_p) { return []; }
pub fn kt_nearby(_m, _r) { return []; }
pub fn kt_fact_mol_at(_i) { if _i < len(__kt_facts_mol) { return __array_get(__kt_facts_mol, _i); }; return 0; }
pub fn kt_ingest_book(_p) { return ""; }
pub fn kt_ingest_full(_p) { return ""; }
pub fn kt_read_book(_p) { return ""; }
pub fn kt_find_fast(_w, _m) { return kt_find(_w, _m); }
pub fn kt_char(_c) { return 0; }
pub fn kt_word(_w) { return 0; }
