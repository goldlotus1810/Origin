// repl.ol — REPL compile-and-execute entry point
//
// Orchestrates the bootstrap compiler pipeline:
//   tokenize → parse → analyze → generate → eval
//
// Called by the VM's REPL loop with user input string.
// Returns output string (from emit) or error message.

// ════════════════════════════════════════════════════════
// REPL eval — main entry point
// ════════════════════════════════════════════════════════

let __boot_learned = 0;

fn _boot_learn() {
    if __boot_learned == 1 { return; };
    let __boot_learned = 1;
    // Init L2 tree structure
    _kt_boot_tree();
    // Load persistent KnowTree first
    kt_load("homeos.knowledge");
    // If empty, load embedded facts directly into KnowTree
    if len(__kt_facts) == 0 {
        _boot_embedded_kt();
    };
}

fn _boot_embedded_kt() {
    kt_learn("Origin la du an tao ngon ngu lap trinh tu hosting ten Olang");
    kt_learn("Olang tu compile chinh minh trong 1021 kilobyte khong dependency");
    kt_learn("VM cua Olang viet bang x86 64 assembly khoang 5987 dong code");
    kt_learn("Compiler cua Olang gom lexer parser semantic va codegen");
    kt_learn("HomeOS la he dieu hanh tri thuc chay tren Olang");
    kt_learn("HomeOS biet doc sach nho va tra loi tu tri thuc da hoc");
    kt_learn("goldlotus1810 la nguoi tao du an Origin va dan duong cac AI session");
    kt_learn("Origin bat dau ngay 11 thang 3 nam 2026");
    kt_learn("Tu hosting dat duoc ngay 23 thang 3 nam 2026 sau 13 ngay");
    kt_learn("Viet Nam la quoc gia o Dong Nam A voi thu do Ha Noi");
    kt_learn("Ho Chi Minh City la thanh pho lon nhat cua Viet Nam");
    kt_learn("Da Nang la thanh pho bien dep nam giua Viet Nam");
    kt_learn("Vinh Ha Long la di san the gioi UNESCO o Quang Ninh");
    kt_learn("Phu Quoc la dao lon nhat cua Viet Nam o Kien Giang");
    kt_learn("Trai Dat quay quanh Mat Troi mat 365 ngay mot vong");
    kt_learn("Nuoc soi o 100 do C va dong bang o 0 do C");
    kt_learn("Einstein phat minh thuyet tuong doi nam 1905");
    kt_learn("Newton phat minh luc hap dan khi thay tao roi");
    kt_learn("DNA la phan tu mang thong tin di truyen cua moi sinh vat");
    kt_learn("Internet bat dau tu ARPANET nam 1969");
    kt_learn("khi nguoi ta chao nen chao lai than thien va hoi ho the nao");
    kt_learn("khi nguoi ta buon nen lang nghe va dong cam truoc khi khuyen");
    kt_learn("khi nguoi ta hoi ve ban than nen tra loi trung thuc va khiem ton");
    kt_learn("khi nguoi ta cam on nen nhan va chuc ho tot dep");
    kt_learn("khi nguoi ta gian nen binh tinh lang nghe va khong phan ung gay gat");
    kt_learn("SHA-256 la thuat toan bam mat ma tao chuoi 64 ky tu hex");
    kt_learn("Olang co map filter reduce any all va pipe cho functional programming");
    kt_learn("Moi function trong Olang tu dong dang ky thanh node voi mol va fire count");
}

// _boot_embedded and _learn_text REMOVED — KnowTree only (Sprint 5)

// ── Module expansion: replace `use "path";` with file contents ──
// Find position of `use "` after a `;` or at start. Returns -1 if not found.
fn _find_use_pos(_fup_src) {
    let _fup_len = len(_fup_src);
    if _fup_len < 6 { return 0 - 1; };
    // Check at position 0
    if __substr(_fup_src, 0, 5) == "use \"" { return 0; };
    // Scan for "; use " or ";use " after semicolons
    let _fup_i = 0;
    let _fup_result = 0 - 1;
    while _fup_i < _fup_len {
        if char_at(_fup_src, _fup_i) == ";" {
            let _fup_j = _fup_i + 1;
            // Skip spaces after ;
            while _fup_j < _fup_len {
                if char_at(_fup_src, _fup_j) != " " {
                    _fup_j = _fup_j + _fup_len;
                };
                _fup_j = _fup_j + 1;
            };
            _fup_j = _fup_j - _fup_len - 1;
            if _fup_j + 5 <= _fup_len {
                if __substr(_fup_src, _fup_j, _fup_j + 5) == "use \"" {
                    _fup_result = _fup_j;
                    _fup_i = _fup_i + _fup_len;
                };
            };
        };
        _fup_i = _fup_i + 1;
    };
    return _fup_result;
}

fn _expand_use(_eu_src) {
    let _eu_pos = _find_use_pos(_eu_src);
    if _eu_pos < 0 { return _eu_src; };
    // Find closing quote
    let _eu_qstart = _eu_pos + 5;
    let _eu_i = _eu_qstart;
    let _eu_found = 0;
    while _eu_i < len(_eu_src) {
        if char_at(_eu_src, _eu_i) == "\"" {
            _eu_found = 1;
            _eu_i = _eu_i + len(_eu_src);
        };
        _eu_i = _eu_i + 1;
    };
    if _eu_found == 0 { return _eu_src; };
    let _eu_qend = _eu_i - len(_eu_src) - 1;
    let _eu_path = __substr(_eu_src, _eu_qstart, _eu_qend);
    // Skip "; after closing quote
    let _eu_after = _eu_qend + 1;
    if _eu_after < len(_eu_src) {
        if char_at(_eu_src, _eu_after) == ";" { _eu_after = _eu_after + 1; };
    };
    if _eu_after < len(_eu_src) {
        if char_at(_eu_src, _eu_after) == " " { _eu_after = _eu_after + 1; };
    };
    // Read file (modules should be single-line, no // comments)
    let _eu_content = __file_read(_eu_path);
    let _eu_prefix = __substr(_eu_src, 0, _eu_pos);
    let _eu_rest = __substr(_eu_src, _eu_after, len(_eu_src));
    return _eu_prefix + _eu_content + " " + _eu_rest;
}

pub fn repl_eval(input) {
  // Auto-learn on first call
  _boot_learn();
  // Strip trailing newline if present (use ASM builtin __str_trim)
  let src = __str_trim(input);
  if len(src) == 0 { return ""; }

  // Check for REPL commands
  if src == "exit" || src == "quit" { return "__exit__"; }
  // Persistent knowledge: save/load
  if src == "save" {
    return kt_save("homeos.knowledge");
  }
  if src == "load" {
    let _ld_n = kt_load("homeos.knowledge");
    if _ld_n > 0 { return "Loaded " + __to_string(_ld_n) + " facts. " + kt_stats(); };
    return "No homeos.knowledge file found.";
  }
  if src == "help" {
    return "Code: let fn emit if while for match lambda | HOF: map filter reduce pipe any all | AI: learn respond memory | test build exit";
  }

  // Respond command: full agent pipeline with memory → response
  if len(src) > 8 {
    if __substr(src, 0, 8) == "respond " {
      let _rr_text = __substr(src, 8, len(src));
      return agent_respond(_rr_text);
    };
  }

  // Test command: run inline tests (boot closures restore scope → can't update counters)
  if src == "test" {
    let _tp = 0;
    let _tf = 0;
    // Arithmetic
    if (1 + 2) == 3 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: add"; };
    if (10 - 3) == 7 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: sub"; };
    if (4 * 5) == 20 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: mul"; };
    if (10 / 2) == 5 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: div"; };
    if __floor(3.7) == 3 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: floor"; };
    if __ceil(3.2) == 4 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: ceil"; };
    // Strings
    if len("hello") == 5 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: strlen"; };
    if __to_string(42) == "42" { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: tostr"; };
    // Arrays
    if len([1,2,3]) == 3 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: arrlen"; };
    // SHA-256
    if len(__sha256("abc")) == 64 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: sha256"; };
    if __sha256("abc") == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: sha256val"; };
    // Encoder
    if encode_codepoint(65) == 150 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: encode_A"; };
    // mol_new (uses << and | — compiled by Rust, safe in boot)
    if mol_new(0, 0, 4, 4, 2) == 146 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: mol_new"; };
    // File I/O
    if len(__file_read("TASKBOARD.md")) > 100 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: fileread"; };
    // Emotion v2: "rat buon" → v < 4 (negation/intensifier)
    let _t_emo = text_emotion_v2("rat buon");
    if _t_emo.v < 4 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: emo_v2_intense"; };
    // Emotion v2: "khong vui" → negated positive → v < 4
    let _t_emo2 = text_emotion_v2("khong vui");
    if _t_emo2.v < 4 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: emo_v2_negate"; };
    // a[expr] BinOp (BUG-INDEX regression)
    let _t_arr = [10,20,30];
    if _t_arr[0 + 1] == 20 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: idx_binop"; };
    let _t_j = 0;
    if _t_arr[_t_j + 2] == 30 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: idx_var_add"; };
    // Bubble sort (BUG-SORT regression)
    let _t_sa = [5,2,8,1,9]; let _t_sn = 5; let _t_si = 0;
    while _t_si < _t_sn - 1 { let _t_sj = 0; while _t_sj < _t_sn - 1 - _t_si { if _t_sa[_t_sj] > _t_sa[_t_sj + 1] { let _t_tmp = _t_sa[_t_sj]; set_at(_t_sa, _t_sj, _t_sa[_t_sj + 1]); set_at(_t_sa, _t_sj + 1, _t_tmp); }; _t_sj = _t_sj + 1; }; _t_si = _t_si + 1; };
    if _t_sa[0] == 1 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: sort_first"; };
    if _t_sa[4] == 9 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: sort_last"; };
    // Lambda + map/filter/reduce/any/all: tested via REPL eval context only
    // Boot context cannot call eval closures (known VM limitation)
    // Summary
    if _tf == 0 {
      return "ALL PASS: " + __to_string(_tp) + "/" + __to_string(_tp + _tf);
    } else {
      return "FAILED: " + __to_string(_tf) + " of " + __to_string(_tp + _tf);
    };
  }

  // Compile command: read file → tokenize → stream parse+compile (incremental)
  if len(src) > 8 {
    if __substr(src, 0, 8) == "compile " {
      let _rc_path = __substr(src, 8, len(src));
      let _rc_src = __file_read(_rc_path);
      if len(_rc_src) == 0 { return "Error: cannot read " + _rc_path; };
      // Tokenize (single pass — uses __array_with_cap, no heap issue)
      let _rc_tokens = tokenize(_rc_src);
      let _rc_ntok = len(_rc_tokens);
      // Stream compile: parse+analyze one statement at a time
      _prefill_output();
      let _g_pos = 0;
      let _rc_parser = { tokens: _rc_tokens, pos: 0 };
      let _g_parse_error = 0;
      let _rc_stmts = 0;
      while _rc_parser.pos < _rc_ntok {
        let _rc_peek = _rc_tokens[_rc_parser.pos];
        if _rc_peek.text == "" { break; };
        let _rc_hp = __heap_save();
        let _rc_stmt = parse_stmt(_rc_parser);
        if _g_parse_error == 1 {
          _g_parse_error = 0;
          __heap_restore(_rc_hp);
          continue;
        };
        let _rc_ast1 = [_rc_stmt];
        analyze(_rc_ast1);
        _rc_stmts = _rc_stmts + 1;
      };
      return "Compiled " + _rc_path + ": " + __to_string(len(_rc_src)) + " chars → " + __to_string(_rc_ntok) + " tokens → " + __to_string(_g_pos) + " bytes (" + __to_string(_rc_stmts) + " stmts)";
    };
  }

  // Build command: self-build (compile all .ol → pack binary)
  if src == "build" {
    return self_build();
  }

  // Memory command: show STM + Silk + Knowledge state
  // Personality command
  if len(src) > 12 {
    if __substr(src, 0, 12) == "personality " {
      return set_personality(__substr(src, 12, len(src)));
    };
  }

  if src == "fns" {
    let _rf_out = "Fn nodes: " + __to_string(fn_node_count());
    let _rf_i = 0;
    while _rf_i < fn_node_count() {
        let _rf_n = __fn_nodes[_rf_i];
        _rf_out = _rf_out + "\n  " + _rf_n.name + "(" + __to_string(_rf_n.params) + ") fires=" + __to_string(_rf_n.fires);
        let _rf_i = _rf_i + 1;
    };
    return _rf_out;
  }

  if src == "memory" {
    let _rm_s = stm_summary();
    let _rm_emo = emo_state();
    let _rm_d = stm_digest();
    let _rm_out = "STM: " + __to_string(stm_count()) + " turns | Silk: " + __to_string(silk_count()) + " edges | " + kt_stats();
    _rm_out = _rm_out + " | Nodes: " + __to_string(node_count());
    _rm_out = _rm_out + " | Fn: " + __to_string(fn_node_count());
    _rm_out = _rm_out + "\nEmo: V=" + __to_string(_rm_emo.v) + " A=" + __to_string(_rm_emo.a) + " f'=" + __to_string(__emo_deriv) + " f''=" + __to_string(__emo_accel) + " var=" + __to_string(__emo_variance) + " FE=" + __to_string(__free_energy) + " " + emoji_for_emotion(_rm_emo.v, _rm_emo.a);
    if len(_rm_d) > 0 { _rm_out = _rm_out + "\nDigest: " + _rm_d; };
    if len(_rm_s) > 0 { _rm_out = _rm_out + "\nThemes: " + _rm_s; };
    return _rm_out;
  }

  // Read book: ingest file into KnowTree (hierarchical nodes + Silk)
  if len(src) > 5 {
    if __substr(src, 0, 5) == "read " {
      let _rd_path = __substr(src, 5, len(src));
      return kt_read_book(_rd_path);
    };
  }

  // Learn file: redirect to kt_read_book
  if len(src) > 11 {
    if __substr(src, 0, 11) == "learn_file " {
      return kt_read_book(__substr(src, 11, len(src)));
    };
  }

  // Learn command: teach HomeOS a fact
  if len(src) > 6 {
    if __substr(src, 0, 6) == "learn " {
      let _rl_text = __substr(src, 6, len(src));
      kt_learn(_rl_text);
      return "Da hoc. " + kt_stats();
    };
  }

  // Encode command: encode <text> → show molecular encoding
  if len(src) > 7 {
    if __substr(src, 0, 7) == "encode " {
      let _re_text = __substr(src, 7, len(src));
      let _re_mol = analyze_input(_re_text);
      let _re_emo = text_emotion_v2(_re_text);
      let _re_ue = text_emotion_unicode(_re_text);
      return "Mol=" + __to_string(_re_mol) +
             " S=" + __to_string(_mol_s(_re_mol)) +
             " R=" + __to_string(_mol_r(_re_mol)) +
             " V=" + __to_string(_mol_v(_re_mol)) +
             " A=" + __to_string(_mol_a(_re_mol)) +
             " T=" + __to_string(_mol_t(_re_mol)) +
             " | Emo: V=" + __to_string(_re_emo.v) + " A=" + __to_string(_re_emo.a) +
             " Emoji=" + __to_string(_re_ue.emoji_count) +
             " | Intent=" + __g_analysis_intent +
             " Tone=" + __g_analysis_tone +
             " Ctx=" + __g_analysis_role + "/" + __g_analysis_source;
    };
  }

  // Check if input looks like code (starts with keyword or symbol)
  // If not → treat as natural text conversation
  let _re_first = char_at(src, 0);
  let _re_is_code = 0;
  // Code starts with: letter (let/fn/if/emit/match/try/for/while/type/union)
  // or symbol ([ for array, { for dict, ( for group, " for string, digit)
  if __char_code(_re_first) >= 48 { if __char_code(_re_first) <= 57 { _re_is_code = 1; }; };
  if _re_first == "[" { _re_is_code = 1; };
  if _re_first == "\"" { _re_is_code = 1; };
  if _re_first == "(" { _re_is_code = 1; };
  if _re_first == "-" { _re_is_code = 1; };
  // Check keyword starts
  if len(src) >= 2 {
    let _re_2 = __substr(src, 0, 2);
    if _re_2 == "le" { _re_is_code = 1; };
    if _re_2 == "fn" { _re_is_code = 1; };
    if _re_2 == "if" { _re_is_code = 1; };
    if _re_2 == "em" { _re_is_code = 1; };
    if _re_2 == "ma" { _re_is_code = 1; };
    if _re_2 == "tr" { _re_is_code = 1; };
    if _re_2 == "fo" { _re_is_code = 1; };
    if _re_2 == "wh" { _re_is_code = 1; };
    if _re_2 == "ty" { _re_is_code = 1; };
    if _re_2 == "un" { _re_is_code = 1; };
    if _re_2 == "pu" { _re_is_code = 1; };
    if _re_2 == "re" { _re_is_code = 1; };
    if _re_2 == "__" { _re_is_code = 1; };
    if _re_2 == "us" { _re_is_code = 1; };  // use "module.ol"
  };

  // Not code → classify: greeting / question / chat
  if _re_is_code == 0 {
    // Short greetings → smart response (no knowledge lookup)
    if len(src) <= 15 {
        if src == "hi" || src == "Hi" || src == "hello" || src == "Hello" { return smart_greet(stm_count()); };
        if src == "hey" || src == "Hey" || src == "yo" || src == "Yo" { return smart_greet(stm_count()); };
        if src == "chao" || src == "Chao" || src == "xin chao" || src == "Xin chao" { return smart_greet(stm_count()); };
        if src == "bye" || src == "Bye" || src == "tam biet" { return smart_goodbye(stm_count()); };
    };
    return agent_respond(src);
  }

  // Strip ALL trailing ? = ! for math expressions ("2+3=?" → "2+3")
  let _re_code = src;
  let _re_strip = 1;
  while _re_strip == 1 {
    _re_strip = 0;
    if len(_re_code) > 0 {
        let _re_lc = __char_code(char_at(_re_code, len(_re_code) - 1));
        if _re_lc == 63 { _re_code = __substr(_re_code, 0, len(_re_code) - 1); _re_strip = 1; };  // ?
        if _re_lc == 61 { _re_code = __substr(_re_code, 0, len(_re_code) - 1); _re_strip = 1; };  // =
        if _re_lc == 33 { _re_code = __substr(_re_code, 0, len(_re_code) - 1); _re_strip = 1; };  // !
    };
  };
  if len(_re_code) == 0 { return ""; };

  // Phase 0.5: Expand `use "path";` directives by inlining file contents
  let _re_code = _expand_use(_re_code);

  // Phase 1: Tokenize
  let tokens = tokenize(_re_code);
  if len(tokens) == 0 { return ""; }

  // Phase 2: Parse
  let ast = parse(tokens);

  // Parse error → try agent, or show helpful message
  if _g_parse_error == 1 {
    let _g_parse_error = 0;
    return agent_respond(src);
  }

  // Phase 3: Semantic analysis
  let _g_pos = 0;
  let state = analyze(ast);

  // Phase 4: Bytecode in _g_output (pre-filled array with set_at, no push)
  let bc = _g_output;
  if _g_pos == 0 { return ""; }

  // Phase 5: Execute compiled bytecode
  return __eval_bytecode(bc);
}

// ════════════════════════════════════════════════════════
// Input classification (for natural text vs code)
// ════════════════════════════════════════════════════════

pub fn is_olang_code(input) {
  let src = __str_trim(input);
  if len(src) == 0 { return false; }

  // Check first token — if it's a keyword, it's code
  let tokens = tokenize(src);
  if len(tokens) == 0 { return false; }

  let first_type = tokens[0].type;
  if first_type == "Let" { return true; }
  if first_type == "Fn" { return true; }
  if first_type == "If" { return true; }
  if first_type == "While" { return true; }
  if first_type == "For" { return true; }
  if first_type == "Match" { return true; }
  if first_type == "Return" { return true; }
  if first_type == "Pub" { return true; }
  if first_type == "Emit" { return true; }

  // Check for ○{...} command syntax
  if len(src) >= 4 {
    if char_at(src, 0) == 0xE2 && char_at(src, 1) == 0x97 && char_at(src, 2) == 0x8B {
      return true;
    }
  }

  return false;
}

// ════════════════════════════════════════════════════════
// REPL helpers
// ════════════════════════════════════════════════════════

pub fn repl_format_error(err) {
  return "\x1b[31m" + err + "\x1b[0m";
}

pub fn repl_format_output(text) {
  return text;
}

// str_trim: now uses ASM builtin __str_trim directly (see call sites above)
