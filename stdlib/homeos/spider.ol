// homeos/spider.ol — Bug Spider + HTTP Client + KnowTree Feeder
// Part 1: Bug Spider (test runner via eval)
// Part 2: HTTP client (curl workaround) + text → KnowTree pipeline
// Usage: type "spider" in REPL for tests, spider_crawl(url) for HTTP

pub fn spider() {
    // Spider runs tests via eval to avoid boot closure limitations.
    // Each batch is a single eval with inline test function.
    let _sp_code = "fn sp(d,g,e){if __to_string(g)==__to_string(e){return 1;};emit \"  FAIL:\"+d+\" got:\"+__to_string(g); return 0;};let p=0;";

    // Batch 1: nested builtins
    _sp_code = _sp_code + "p=p+sp(\"nest:sha\",__substr(__sha256(\"abc\"),0,8),\"ba7816bf\");";
    _sp_code = _sp_code + "p=p+sp(\"nest:chr\",char_at(__sha256(\"abc\"),0),\"b\");";
    _sp_code = _sp_code + "p=p+sp(\"nest:len\",len(__sha256(\"x\")),64);";
    _sp_code = _sp_code + "p=p+sp(\"nest:tostr\",__substr(__to_string(12345),0,3),\"123\");";
    _sp_code = _sp_code + "p=p+sp(\"nest:3lvl\",__substr(__sha256(__to_string(42)),0,8),__substr(__sha256(__to_string(42)),0,8));";
    _sp_code = _sp_code + "p=p+sp(\"nest:cat\",len(\"ab\"+\"cd\"),4);";
    _sp_code = _sp_code + "p=p+sp(\"nest:ts2\",__to_string(__to_string(99)),\"99\");";

    // Batch 2: interpolation
    _sp_code = _sp_code + "p=p+sp(\"int:add\",$\"{2+3}\",\"5\");";
    _sp_code = _sp_code + "p=p+sp(\"int:mul\",$\"{7*6}\",\"42\");";
    _sp_code = _sp_code + "p=p+sp(\"int:sub\",$\"{100-37}\",\"63\");";
    _sp_code = _sp_code + "let xi=10;p=p+sp(\"int:var\",$\"{xi*2+1}\",\"21\");";
    _sp_code = _sp_code + "let ni=5;p=p+sp(\"int:mix\",$\"v={ni*ni}\",\"v=25\");";

    // Batch 3: operators + strings + crypto
    _sp_code = _sp_code + "p=p+sp(\"op:prec\",2+3*4,14);";
    _sp_code = _sp_code + "p=p+sp(\"op:paren\",(2+3)*4,20);";
    _sp_code = _sp_code + "p=p+sp(\"str:len\",len(\"hello\"),5);";
    _sp_code = _sp_code + "p=p+sp(\"str:sub\",__substr(\"abcdef\",2,4),\"cd\");";
    _sp_code = _sp_code + "p=p+sp(\"str:cat\",\"ab\"+\"cd\",\"abcd\");";
    _sp_code = _sp_code + "p=p+sp(\"cry:empty\",__substr(__sha256(\"\"),0,8),\"e3b0c442\");";
    _sp_code = _sp_code + "p=p+sp(\"cry:hello\",len(__sha256(\"hello\")),64);";
    _sp_code = _sp_code + "p=p+sp(\"sort\",sort([3,1,2])[0],1);";

    // Batch 4: functions + recursion
    _sp_code = _sp_code + "fn fib(n){if n<2{return n;};return fib(n-1)+fib(n-2);};";
    _sp_code = _sp_code + "p=p+sp(\"fn:fib\",fib(10),55);";
    _sp_code = _sp_code + "fn fact(n){if n<2{return 1;};return n*fact(n-1);};";
    _sp_code = _sp_code + "p=p+sp(\"fn:fact\",fact(6),720);";
    _sp_code = _sp_code + "fn dbl(x){return x*2;};p=p+sp(\"fn:nest\",dbl(dbl(5)),20);";

    // Batch 5: lambda + HOF
    _sp_code = _sp_code + "let f=fn(x){return x*3;};p=p+sp(\"lam\",f(7),21);";
    _sp_code = _sp_code + "p=p+sp(\"map\",map([1,2,3],fn(x){return x*10;}),[10,20,30]);";
    _sp_code = _sp_code + "p=p+sp(\"filter\",filter([1,2,3,4,5],fn(x){return x>3;}),[4,5]);";
    _sp_code = _sp_code + "p=p+sp(\"reduce\",reduce([1,2,3,4],fn(a,b){return a+b;}),10);";

    // Final
    _sp_code = _sp_code + "emit \"SPIDER:\"+__to_string(p)+\"/28\";";

    // Compile + execute
    let _sp_tokens = tokenize(_sp_code);
    let _sp_ast = parse(_sp_tokens);
    if _g_parse_error == 1 { let _g_parse_error = 0; return "SPIDER: PARSE ERROR"; };
    let _g_pos = 0;
    _prefill_output();
    analyze(_sp_ast);
    let _sp_bc = _g_output;
    if _g_pos == 0 { return "SPIDER: EMPTY"; };
    __eval_bytecode(_sp_bc);
    return "";
}

// ════════════════════════════════════════════════════════════════
// Part 2: HTTP Client — curl workaround for HTTPS
// ════════════════════════════════════════════════════════════════

pub fn http_get(_hg_url) {
    let _hg_cmd = "curl -sL -m 10 '" + _hg_url + "' 2>/dev/null";
    let _hg_body = __system(_hg_cmd);
    if len(_hg_body) == 0 { return ""; };
    return _hg_body;
}

// http_get_text removed — dead code (0 calls)

// ════════════════════════════════════════════════════════════════
// HTML → plain text (strip tags)
// ════════════════════════════════════════════════════════════════

// Strip markdown: remove code blocks, keep clean text ranges
// Uses substr ranges instead of char-by-char concat (O(n) vs O(n²))
pub fn md_strip(_ms_text) {
    let _ms_tlen = len(_ms_text);
    // Pass 1: find code fence ranges to skip
    // Pass 2: copy clean line ranges using substr
    let _ms_parts = [];
    let _ms_in_code = [0];
    let _ms_line_start = [0];
    let _ms_i = [0];
    while __array_get(_ms_i, 0) < _ms_tlen {
        let _ms_ci = __array_get(_ms_i, 0);
        let _ms_c = __char_code(char_at(_ms_text, _ms_ci));
        // Detect ``` code fence
        if _ms_c == 96 {
            if (_ms_ci + 2) < _ms_tlen {
                if __char_code(char_at(_ms_text, _ms_ci + 1)) == 96 {
                    if __char_code(char_at(_ms_text, _ms_ci + 2)) == 96 {
                        let _ = __set_at(_ms_in_code, 0, 1 - __array_get(_ms_in_code, 0));
                        // Skip to end of fence line
                        let _ = __set_at(_ms_i, 0, _ms_ci + 3);
                        while __array_get(_ms_i, 0) < _ms_tlen {
                            if __char_code(char_at(_ms_text, __array_get(_ms_i, 0))) == 10 { break; };
                            let _ = __set_at(_ms_i, 0, __array_get(_ms_i, 0) + 1);
                        };
                        let _ = __set_at(_ms_line_start, 0, __array_get(_ms_i, 0) + 1);
                    };
                };
            };
        };
        // On newline: save clean line as substr (one allocation, not n)
        if _ms_c == 10 {
            if __array_get(_ms_in_code, 0) == 0 {
                let _ms_ls = __array_get(_ms_line_start, 0);
                if (_ms_ci - _ms_ls) > 2 {
                    // Skip lines starting with markdown chars
                    let _ms_fc = __char_code(char_at(_ms_text, _ms_ls));
                    if _ms_fc != 35 { if _ms_fc != 62 { if _ms_fc != 124 { if _ms_fc != 96 {
                        push(_ms_parts, substr(_ms_text, _ms_ls, _ms_ci));
                    }; }; }; };
                };
            };
            let _ = __set_at(_ms_line_start, 0, _ms_ci + 1);
        };
        let _ = __set_at(_ms_i, 0, __array_get(_ms_i, 0) + 1);
    };
    // Join parts with spaces (one concat per line, not per char)
    let _ms_out = "";
    let _ms_pi = 0;
    while _ms_pi < len(_ms_parts) {
        if _ms_pi > 0 { let _ms_out = _ms_out + " "; };
        let _ms_out = _ms_out + __array_get(_ms_parts, _ms_pi);
        let _ms_pi = _ms_pi + 1;
    };
    return _ms_out;
}

pub fn html_strip(_hs_html) {
    let _hs_out = "";
    let _hs_in_tag = [0];
    let _hs_i = 0;
    while _hs_i < len(_hs_html) {
        let _hs_c = __char_code(char_at(_hs_html, _hs_i));
        if _hs_c == 60 { let _ = __set_at(_hs_in_tag, 0, 1); };
        if __array_get(_hs_in_tag, 0) == 0 {
            if _hs_c >= 32 { let _hs_out = _hs_out + char_at(_hs_html, _hs_i); };
        };
        if _hs_c == 62 { let _ = __set_at(_hs_in_tag, 0, 0); };
        let _hs_i = _hs_i + 1;
    };
    return _hs_out;
}

// ════════════════════════════════════════════════════════════════
// Text → sentences → kt_learn (feed KnowTree)
// ════════════════════════════════════════════════════════════════

pub fn spider_feed(_sf_text, _sf_source) {
    let _sf_count = [0];
    let _sf_start = [0];
    let _sf_tlen = len(_sf_text);
    let _sf_i = 0;
    while _sf_i < _sf_tlen {
        let _sf_c = __char_code(char_at(_sf_text, _sf_i));
        let _sf_is_end = [0];
        if _sf_c == 46 { let _ = __set_at(_sf_is_end, 0, 1); };
        if _sf_c == 33 { let _ = __set_at(_sf_is_end, 0, 1); };
        if _sf_c == 63 { let _ = __set_at(_sf_is_end, 0, 1); };
        if __array_get(_sf_is_end, 0) == 1 {
            let _sf_s = __array_get(_sf_start, 0);
            let _sf_slen = _sf_i - _sf_s;
            if _sf_slen > 10 {
                let _sf_sent = substr(_sf_text, _sf_s, _sf_i);
                if _sf_is_prose(_sf_sent) == 1 {
                    kt_learn(_sf_sent);
                    let _ = __set_at(_sf_count, 0, __array_get(_sf_count, 0) + 1);
                };
            };
            let _ = __set_at(_sf_start, 0, _sf_i + 1);
        };
        let _sf_i = _sf_i + 1;
    };
    let _sf_s = __array_get(_sf_start, 0);
    if (_sf_tlen - _sf_s) > 10 {
        let _sf_last = substr(_sf_text, _sf_s, _sf_tlen);
        if _sf_is_prose(_sf_last) == 1 {
            kt_learn(_sf_last);
            let _ = __set_at(_sf_count, 0, __array_get(_sf_count, 0) + 1);
        };
    };
    __heap_pin();
    return "Fed " + __to_string(__array_get(_sf_count, 0)) + " sentences from " + _sf_source + ". " + kt_stats();
}

// Filter: is this text prose (not code)?
// Code indicators: { } ; // fn let if while emit return → skip
fn _sf_is_prose(_sip_text) {
    let _sip_len = len(_sip_text);
    let _sip_braces = [0];
    let _sip_slashes = [0];
    let _sip_semis = [0];
    let _sip_i = 0;
    while _sip_i < _sip_len {
        let _sip_c = __char_code(char_at(_sip_text, _sip_i));
        if _sip_c == 123 { let _ = __set_at(_sip_braces, 0, __array_get(_sip_braces, 0) + 1); };
        if _sip_c == 125 { let _ = __set_at(_sip_braces, 0, __array_get(_sip_braces, 0) + 1); };
        if _sip_c == 59 { let _ = __set_at(_sip_semis, 0, __array_get(_sip_semis, 0) + 1); };
        if _sip_c == 47 {
            if (_sip_i + 1) < _sip_len {
                if __char_code(char_at(_sip_text, _sip_i + 1)) == 47 {
                    let _ = __set_at(_sip_slashes, 0, __array_get(_sip_slashes, 0) + 1);
                };
            };
        };
        let _sip_i = _sip_i + 1;
    };
    // Too many code indicators → not prose
    if __array_get(_sip_braces, 0) >= 2 { return 0; };
    if __array_get(_sip_semis, 0) >= 3 { return 0; };
    if __array_get(_sip_slashes, 0) >= 2 { return 0; };
    // Check first 3 chars for code keywords
    if _sip_len >= 3 {
        let _sip_3 = substr(_sip_text, 0, 3);
        if _sip_3 == "fn " { return 0; };
        if _sip_3 == "let" { return 0; };
        if _sip_3 == "if " { return 0; };
    };
    return 1;
}

// Feed markdown file → strip → sentences → KnowTree
pub fn spider_feed_md(_sfm_text, _sfm_source) {
    let _sfm_clean = md_strip(_sfm_text);
    return spider_feed(_sfm_clean, _sfm_source);
}

// Full pipeline: URL → fetch → strip HTML → feed KnowTree
pub fn spider_crawl(_sc_url) {
    let _sc_html = http_get(_sc_url);
    if len(_sc_html) == 0 { return "Error: no response from " + _sc_url; };
    let _sc_text = html_strip(_sc_html);
    if len(_sc_text) < 20 { return "Error: no text content from " + _sc_url; };
    return spider_feed(_sc_text, _sc_url);
}

// Feed from local file → KnowTree
// spider_file removed — dead code (use study command instead)
