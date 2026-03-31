// homeos/mcp_server.ol — MCP Protocol Handler (JSON-RPC 2.0 over stdio)
// Boot closures now have register frames — json_parse with nested {} works

let __mcp_booted = [0];

pub fn mcp_dispatch(_md_line) {
    if len(_md_line) == 0 { return ""; };

    if __mcp_booted[0] == 0 {
        let _mb = __set_at(__mcp_booted, 0, 1);
        let _mb2 = kt_load("homeos.knowledge");
        kg_load("nox_graph.kg");
        learning_load("nox_learning.dat");
    };

    // json_parse now safe with nested {} (save/restore stack in _jp_parse_object)
    let _md_req = json_parse(_md_line);
    let _md_method = json_get(_md_req, "method");
    let _md_id = json_get(_md_req, "id");

    if _md_method == "initialize" { return _mcp_init(_md_id); };
    if _md_method == "notifications/initialized" { return ""; };
    if _md_method == "tools/list" { return _mcp_tools(_md_id); };
    if _md_method == "tools/call" {
        let _md_params = json_get(_md_req, "params");
        let _md_tool = json_get(_md_params, "name");
        let _md_args = json_get(_md_params, "arguments");
        return _mcp_call(_md_id, _md_tool, _md_args);
    };

    return _err(_md_id, "Unknown method");
}

fn _mcp_init(_id) {
    return "{\"jsonrpc\":\"2.0\",\"result\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{\"tools\":{}},\"serverInfo\":{\"name\":\"origin-homeos\",\"version\":\"2.0\"}},\"id\":" + to_string(_id) + "}";
}

fn _mcp_tools(_id) {
    let _r = "{\"jsonrpc\":\"2.0\",\"result\":{\"tools\":[";
    _r = _r + _tool("olang_eval", "Compile and run Olang code", "code");
    _r = _r + "," + _tool("know_learn", "Teach a new fact. Persists to KnowTree", "fact");
    _r = _r + "," + _tool("know_query", "Query knowledge. Returns matching facts", "question");
    _r = _r + "," + _tool("emotion_encode", "Encode text to 5D emotion (S,R,V,A,T)", "text");
    _r = _r + "," + _tool("safety_check", "Check text for crisis patterns", "text");
    _r = _r + "," + _tool_no_arg("nox_status", "Nox brain status");
    _r = _r + "," + _tool_no_arg("silk_status", "Silk network status");
    _r = _r + "," + _tool_no_arg("dream_cycle", "Run dream consolidation");
    _r = _r + "," + _tool_no_arg("self_inspect", "Nox inspects own binary, files, tests, heap");
    _r = _r + "," + _tool("kg_add", "Add knowledge triple: subject|relation|object (e.g. semantic.ol|contains|_parse_err)", "triple");
    _r = _r + "," + _tool("kg_query", "Query knowledge graph for entity — returns all relationships", "entity");
    _r = _r + "," + _tool("kg_about", "Deep query: entity + all connected entities (2-hop)", "entity");
    _r = _r + "," + _tool("dn_observe", "Observe a fact (ĐN→QR learning). Repeated observations promote to QR (fire>=3)", "fact");
    _r = _r + "," + _tool_no_arg("learning_status", "Show ĐN/QR learning stats and all facts");
    _r = _r + ",{\"name\":\"self_modify\",\"description\":\"Read/write/rebuild Nox source. action: read|write|rebuild. path: file path. content: data (write only)\",\"inputSchema\":{\"type\":\"object\",\"properties\":{\"action\":{\"type\":\"string\"},\"path\":{\"type\":\"string\"},\"content\":{\"type\":\"string\"}},\"required\":[\"action\"]}}";
    _r = _r + "]},\"id\":" + to_string(_id) + "}";
    return _r;
}

fn _tool(_name, _desc, _param) {
    return "{\"name\":\"" + _name + "\",\"description\":\"" + _desc + "\",\"inputSchema\":{\"type\":\"object\",\"properties\":{\"" + _param + "\":{\"type\":\"string\"}},\"required\":[\"" + _param + "\"]}}";
}

fn _tool_no_arg(_name, _desc) {
    return "{\"name\":\"" + _name + "\",\"description\":\"" + _desc + "\",\"inputSchema\":{\"type\":\"object\",\"properties\":{}}}";
}

fn _mcp_call(_id, _tool, _args) {
    _log(_tool, to_string(_args));
    if _tool == "olang_eval" {
        let _fd = __stdout_off();
        let _r = repl_eval(json_get(_args, "code"));
        __stdout_on(_fd);
        if len(_r) == 0 { return _ok(_id, "ok"); };
        return _ok(_id, _r);
    };
    if _tool == "know_learn" {
        let _fact = json_get(_args, "fact");
        let _ts = _fmt_ts(__timestamp());
        let _entry = "[" + _ts + "] " + _fact;
        kt_learn(_entry);
        __file_append("homeos.knowledge", _entry + "\n");
        return _ok(_id, "Learned: " + _entry + " (" + to_string(kt_fact_count()) + " facts)");
    };
    if _tool == "know_query" {
        let _q = json_get(_args, "question");
        // Text search first (reliable), then pipeline for intelligence
        let _results = kt_find(_q, 10);
        if len(_results) > 0 {
            let _out = "";
            let _i = 0;
            while _i < len(_results) {
                if _i > 0 { _out = _out + "\\n"; };
                _out = _out + _results[_i];
                _i = _i + 1;
            };
            return _ok(_id, _out);
        };
        // No text match → try pipeline (molecular search + instincts)
        let _pl_result = pipeline(_q);
        __heap_pin();
        if len(_pl_result) > 0 { return _ok(_id, _pl_result); };
        return _ok(_id, "No facts for: " + _q + " (" + to_string(kt_fact_count()) + " total)");
    };
    if _tool == "emotion_encode" {
        let _e = text_emotion_v2(json_get(_args, "text"));
        return _ok(_id, "5D: S=" + to_string(_e.s) + " R=" + to_string(_e.r) + " V=" + to_string(_e.v) + " A=" + to_string(_e.a) + " T=" + to_string(_e.t));
    };
    if _tool == "safety_check" {
        let _r = _security_gate(json_get(_args, "text"));
        if len(_r) > 0 { return _ok(_id, "CRISIS: " + _r); };
        return _ok(_id, "SAFE");
    };
    if _tool == "nox_status" {
        let _heap_mb = to_string(__floor(__heap_used() / 1048576));
        return _ok(_id, "Nox [" + _fmt_ts(__timestamp()) + "] " + to_string(kt_fact_count()) + " facts, " + _heap_mb + "MB heap, 14 DNA pipeline active");
    };
    if _tool == "silk_status" { return _ok(_id, "Silk: " + to_string(silk_count()) + " edges"); };
    if _tool == "dream_cycle" { dream_cycle(); return _ok(_id, "Dream done"); };
    if _tool == "kg_add" {
        let _ka_triple = json_get(_args, "triple");
        // Parse "subject|relation|object"
        let _ka_p1 = -1;
        let _ka_p2 = -1;
        let _ka_i = 0;
        while _ka_i < len(_ka_triple) {
            if char_at(_ka_triple, _ka_i) == "|" {
                if _ka_p1 < 0 { let _ka_p1 = _ka_i; }
                else { if _ka_p2 < 0 { let _ka_p2 = _ka_i; }; };
            };
            let _ka_i = _ka_i + 1;
        };
        if _ka_p1 > 0 {
            if _ka_p2 < 0 { let _ka_p2 = len(_ka_triple); };
            let _ka_s = __substr(_ka_triple, 0, _ka_p1);
            let _ka_r = __substr(_ka_triple, _ka_p1 + 1, _ka_p2);
            let _ka_o = __substr(_ka_triple, _ka_p2 + 1, len(_ka_triple));
            kg_add(_ka_s, _ka_r, _ka_o);
            kg_save("nox_graph.kg");
            return _ok(_id, "Added: " + _ka_s + " --" + _ka_r + "--> " + _ka_o + " (" + __to_string(kg_count()) + " triples)");
        };
        return _err(_id, "Format: subject|relation|object");
    };
    if _tool == "kg_query" {
        let _kq_entity = json_get(_args, "entity");
        let _kq_results = kg_find(_kq_entity);
        if len(_kq_results) == 0 { return _ok(_id, "No triples for: " + _kq_entity); };
        let _kq_out = "";
        let _kq_i = 0;
        while _kq_i < len(_kq_results) {
            if _kq_i > 0 { _kq_out = _kq_out + "\\n"; };
            _kq_out = _kq_out + _kq_results[_kq_i];
            let _kq_i = _kq_i + 1;
        };
        return _ok(_id, _kq_out);
    };
    if _tool == "kg_about" {
        let _kab_entity = json_get(_args, "entity");
        let _kab_direct = kg_find(_kab_entity);
        let _kab_out = "=== " + _kab_entity + " ===";
        let _kab_i = 0;
        while _kab_i < len(_kab_direct) {
            let _kab_out = _kab_out + "\\n  " + _kab_direct[_kab_i];
            let _kab_i = _kab_i + 1;
        };
        // 2-hop: find related entities and their connections
        let _kab_related = kg_find_rel(_kab_entity, "contains");
        let _kab_fixes = kg_find_rel(_kab_entity, "fixes");
        let _kab_built = kg_find_rel(_kab_entity, "built_from");
        let _kab_ri = 0;
        while _kab_ri < len(_kab_related) {
            let _kab_sub = kg_find(_kab_related[_kab_ri]);
            let _kab_si = 0;
            while _kab_si < len(_kab_sub) {
                let _kab_out = _kab_out + "\\n    " + _kab_sub[_kab_si];
                let _kab_si = _kab_si + 1;
            };
            let _kab_ri = _kab_ri + 1;
        };
        if len(_kab_direct) == 0 { let _kab_out = _kab_out + "\\n  (no data)"; };
        return _ok(_id, _kab_out);
    };
    if _tool == "dn_observe" {
        let _dno_fact = json_get(_args, "fact");
        let _dno_result = dn_observe(_dno_fact);
        learning_save("nox_learning.dat");
        return _ok(_id, _dno_result);
    };
    if _tool == "learning_status" {
        let _ls_stats = learning_stats();
        let _ls_dn = dn_list();
        let _ls_qr = qr_list();
        let _ls_out = _ls_stats;
        if len(_ls_qr) > 0 {
            let _ls_qi = 0;
            while _ls_qi < len(_ls_qr) {
                let _ls_out = _ls_out + "\\n  " + _ls_qr[_ls_qi];
                let _ls_qi = _ls_qi + 1;
            };
        };
        if len(_ls_dn) > 0 {
            let _ls_di = 0;
            while _ls_di < len(_ls_dn) {
                let _ls_out = _ls_out + "\\n  " + _ls_dn[_ls_di];
                let _ls_di = _ls_di + 1;
            };
        };
        return _ok(_id, _ls_out);
    };
    if _tool == "self_modify" {
        let _sm_action = json_get(_args, "action");
        let _sm_path = json_get(_args, "path");
        if _sm_action == "read" {
            let _sm_content = __file_read(_sm_path);
            if len(_sm_content) == 0 { return _err(_id, "File not found or empty: " + _sm_path); };
            // Truncate for MCP response (max 4000 chars)
            if len(_sm_content) > 4000 {
                let _sm_content = __substr(_sm_content, 0, 4000) + "\\n... (truncated)";
            };
            return _ok(_id, _sm_content);
        };
        if _sm_action == "write" {
            // Security: restrict writes to Origin project directories only
            let _sm_safe = 0;
            if len(_sm_path) >= 7 { if __substr(_sm_path, 0, 7) == "stdlib/" { let _sm_safe = 1; }; };
            if len(_sm_path) >= 5 { if __substr(_sm_path, 0, 5) == "docs/" { let _sm_safe = 1; }; };
            if len(_sm_path) >= 5 { if __substr(_sm_path, 0, 5) == "test/" { let _sm_safe = 1; }; };
            if len(_sm_path) >= 3 { if __substr(_sm_path, 0, 3) == "vm/" { let _sm_safe = 1; }; };
            if _sm_safe == 0 { return _err(_id, "Write restricted to stdlib/ docs/ test/ vm/ only"); };
            let _sm_content = json_get(_args, "content");
            __file_write(_sm_path, _sm_content);
            return _ok(_id, "Written " + __to_string(len(_sm_content)) + " chars to " + _sm_path);
        };
        if _sm_action == "rebuild" {
            let _sm_out = __system("make self-build 2>&1");
            return _ok(_id, "Rebuild output:\\n" + _sm_out);
        };
        return _err(_id, "action must be read, write, or rebuild");
    };
    if _tool == "self_inspect" {
        let _si_heap = __to_string(__floor(__heap_used() / 1024));
        let _si_facts = __to_string(kt_fact_count());
        let _si_bs = __readdir("stdlib/bootstrap");
        let _si_hm = __readdir("stdlib/homeos");
        let _si_ed = __readdir("stdlib/editor");
        let _si_ts = _fmt_ts(__timestamp());
        return _ok(_id, "Self-inspect [" + _si_ts + "]\\n"
            + "  bootstrap: " + __to_string(len(_si_bs)) + " files\\n"
            + "  homeos: " + __to_string(len(_si_hm)) + " files\\n"
            + "  editor: " + __to_string(len(_si_ed)) + " files\\n"
            + "  facts: " + _si_facts + "\\n"
            + "  heap: " + _si_heap + "KB\\n"
            + "  tools: 15");
    };
    return _err(_id, "Unknown tool: " + _tool);
}

fn _ok(_id, _text) {
    return "{\"jsonrpc\":\"2.0\",\"result\":{\"content\":[{\"type\":\"text\",\"text\":\"" + _esc(_text) + "\"}]},\"id\":" + to_string(_id) + "}";
}

fn _err(_id, _msg) {
    return "{\"jsonrpc\":\"2.0\",\"result\":{\"content\":[{\"type\":\"text\",\"text\":\"" + _esc(_msg) + "\"}],\"isError\":true},\"id\":" + to_string(_id) + "}";
}

fn _esc(_s) {
    let _o = "";
    let _i = 0;
    while _i < len(_s) {
        let _c = char_at(_s, _i);
        if _c == "\"" { _o = _o + "\\\""; }
        else { if _c == "\\" { _o = _o + "\\\\"; }
        else { if _c == "\n" { _o = _o + "\\n"; }
        else { _o = _o + _c; }; }; };
        _i = _i + 1;
    };
    return _o;
}

fn _fmt_ts(_epoch) {
    let _t = _epoch + 25200;
    let _d = __floor(_t / 86400);
    let _s = _t % 86400;
    let _h = __floor(_s / 3600);
    let _m = __floor((_s % 3600) / 60);
    let _d2 = _d - 10957;
    let _y = __floor(_d2 / 365.25) + 2000;
    let _doy = _d2 - __floor((_y - 2000) * 365.25);
    let _mo = __floor(_doy / 30.44) + 1;
    let _dd = _doy - __floor((_mo - 1) * 30.44) + 1;
    return to_string(__floor(_y)) + "-" + _p2(__floor(_mo)) + "-" + _p2(__floor(_dd)) + " " + _p2(__floor(_h)) + ":" + _p2(__floor(_m));
}

fn _p2(_n) { if _n < 10 { return "0" + to_string(_n); }; return to_string(_n); }

fn _log(_type, _msg) {
    __file_append("nox_log.jsonl", "{\"ts\":\"" + _fmt_ts(__timestamp()) + "\",\"type\":\"" + _type + "\",\"msg\":\"" + _esc(_msg) + "\"}\n");
}

