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

pub fn repl_eval(input) {
  // Strip trailing newline if present (use ASM builtin __str_trim)
  let src = __str_trim(input);
  if len(src) == 0 { return ""; }

  // Check for REPL commands
  if src == "exit" || src == "quit" { return "__exit__"; }
  if src == "help" {
    return "Commands: let, fn, emit, if, while, match, encode <text>, respond <text>, exit\nType Olang code or natural text.";
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
    // Arithmetic (6)
    if (1 + 2) == 3 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: add"; };
    if (10 - 3) == 7 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: sub"; };
    if (4 * 5) == 20 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: mul"; };
    if (10 / 2) == 5 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: div"; };
    if __floor(3.7) == 3 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: floor"; };
    if __ceil(3.2) == 4 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: ceil"; };
    // Bit ops (3)
    if (1 << 4) == 16 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: shl"; };
    if (255 >> 4) == 15 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: shr"; };
    if (1 && 2) == 2 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: and"; };
    // Strings (2)
    if len("hello") == 5 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: strlen"; };
    if __to_string(42) == "42" { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: tostr"; };
    // Arrays (2)
    if len([1,2,3]) == 3 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: arrlen"; };
    if [x * 2 for x in [1,2,3]][0] == 2 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: comp"; };
    // SHA-256 (2)
    if len(__sha256("abc")) == 64 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: sha256"; };
    if __sha256("abc") == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: sha256val"; };
    // Encoder (2)
    if encode_codepoint(65) == 150 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: encode_A"; };
    if mol_new(0, 0, 4, 4, 2) == 146 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: mol_new"; };
    // Sentiment (2)
    if word_affect("buon").v == 2 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: buon_v"; };
    if word_affect("happy").v == 6 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: happy_v"; };
    // File I/O (1)
    if len(__file_read("TASKBOARD.md")) > 100 { _tp = _tp + 1; } else { _tf = _tf + 1; emit "FAIL: fileread"; };
    // Summary
    if _tf == 0 {
      return "ALL PASS: " + __to_string(_tp) + "/" + __to_string(_tp + _tf);
    } else {
      return "FAILED: " + __to_string(_tf) + " of " + __to_string(_tp + _tf);
    };
  }

  // Compile command: read file → tokenize → parse → compile (in BOOT context)
  if len(src) > 8 {
    if __substr(src, 0, 8) == "compile " {
      let _rc_path = __substr(src, 8, len(src));
      let _rc_src = __file_read(_rc_path);
      if len(_rc_src) == 0 { return "Error: cannot read " + _rc_path; };
      let _rc_tokens = tokenize(_rc_src);
      let _rc_ast = parse(_rc_tokens);
      let _rc_state = analyze(_rc_ast);
      return "Compiled " + _rc_path + ": " + __to_string(len(_rc_src)) + " chars → " + __to_string(len(_rc_tokens)) + " tokens → " + __to_string(_g_pos) + " bytes bytecode";
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

  if src == "memory" {
    let _rm_s = stm_summary();
    return "STM: " + __to_string(stm_count()) + " turns | Silk: " + __to_string(silk_count()) + " edges | Knowledge: " + __to_string(knowledge_count()) + " facts | Themes: " + _rm_s;
  }

  // Learn file: read file and split into sentences for knowledge
  if len(src) > 11 {
    if __substr(src, 0, 11) == "learn_file " {
      let _lf_path = __substr(src, 11, len(src));
      let _lf_content = __file_read(_lf_path);
      if len(_lf_content) == 0 { return "Error: cannot read " + _lf_path; };
      // Split by sentences (. or newline) and learn each
      let _lf_sent = "";
      let _lf_count = 0;
      let _lf_i = 0;
      while _lf_i < len(_lf_content) {
        let _lf_ch = char_at(_lf_content, _lf_i);
        let _lf_code = __char_code(_lf_ch);
        // Split on: newline, period+space
        let _lf_split = 0;
        if _lf_code == 10 { _lf_split = 1; };
        if _lf_code == 46 {
          if (_lf_i + 1) < len(_lf_content) {
            if __char_code(char_at(_lf_content, _lf_i + 1)) == 32 {
              _lf_split = 1;
              _lf_sent = _lf_sent + _lf_ch;
            };
          };
        };
        if _lf_split == 1 {
          if len(_lf_sent) > 15 {
            knowledge_learn(_lf_sent);
            _lf_count = _lf_count + 1;
          };
          _lf_sent = "";
        } else {
          _lf_sent = _lf_sent + _lf_ch;
        };
        let _lf_i = _lf_i + 1;
      };
      if len(_lf_sent) > 15 {
        knowledge_learn(_lf_sent);
        _lf_count = _lf_count + 1;
      };
      return "Learned " + __to_string(_lf_count) + " facts from " + _lf_path + ". Knowledge: " + __to_string(knowledge_count());
    };
  }

  // Learn command: teach HomeOS a fact
  if len(src) > 6 {
    if __substr(src, 0, 6) == "learn " {
      let _rl_text = __substr(src, 6, len(src));
      let _rl_count = knowledge_learn(_rl_text);
      return "Da hoc. Knowledge: " + __to_string(_rl_count) + " facts.";
    };
  }

  // Encode command: encode <text> → show molecular encoding
  if len(src) > 7 {
    if __substr(src, 0, 7) == "encode " {
      let _re_text = __substr(src, 7, len(src));
      let _re_mol = analyze_input(_re_text);
      return "Mol=" + __to_string(_re_mol) +
             " S=" + __to_string(_mol_s(_re_mol)) +
             " R=" + __to_string(_mol_r(_re_mol)) +
             " V=" + __to_string(_mol_v(_re_mol)) +
             " A=" + __to_string(_mol_a(_re_mol)) +
             " T=" + __to_string(_mol_t(_re_mol)) +
             " | Intent=" + __g_analysis_intent +
             " Tone=" + __g_analysis_tone +
             " Ctx=" + __g_analysis_role + "/" + __g_analysis_source;
    };
  }

  // Phase 1: Tokenize
  let tokens = tokenize(src);
  if len(tokens) == 0 { return ""; }

  // Phase 2: Parse
  let ast = parse(tokens);

  // Parse error → fallback to agent
  if _g_parse_error == 1 {
    return agent_respond(src);
  }

  // Phase 3: Semantic analysis
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
