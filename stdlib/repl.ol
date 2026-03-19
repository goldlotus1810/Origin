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
  // Strip trailing newline if present
  let src = str_trim(input);
  if len(src) == 0 { return ""; }

  // Check for REPL commands
  if src == "exit" || src == "quit" { return "__exit__"; }
  if src == "help" {
    return "Commands: let, fn, emit, if, while, match, exit\nType Olang code or natural text.";
  }

  // Phase 1: Tokenize
  let tokens = tokenize(src);
  if len(tokens) == 0 { return ""; }

  // Check for tokenizer error token
  let last = tokens[len(tokens) - 1];
  if last.type == "Error" {
    return "Lex error: " + last.value;
  }

  // Phase 2: Parse
  let ast = parse(tokens);
  if ast.error {
    return "Parse error: " + ast.error;
  }

  // Phase 3: Semantic analysis
  let state = analyze(ast);
  if len(state.errors) > 0 {
    return "Error: " + state.errors[0];
  }

  // Phase 4: Code generation
  let bc = generate(state.ops);
  if len(bc) == 0 { return ""; }

  // Phase 5: Execute compiled bytecode
  return __eval_bytecode(bc);
}

// ════════════════════════════════════════════════════════
// Input classification (for natural text vs code)
// ════════════════════════════════════════════════════════

pub fn is_olang_code(input) {
  let src = str_trim(input);
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

fn str_trim(s) {
  let n = len(s);
  if n == 0 { return s; }
  // Trim trailing whitespace (newline, space, tab, CR)
  let end = n;
  while end > 0 {
    let c = char_at(s, end - 1);
    if c != 10 && c != 13 && c != 32 && c != 9 { break; }
    end = end - 1;
  }
  // Trim leading whitespace
  let start = 0;
  while start < end {
    let c = char_at(s, start);
    if c != 10 && c != 13 && c != 32 && c != 9 { break; }
    start = start + 1;
  }
  if start == 0 && end == n { return s; }
  return substr(s, start, end);
}
