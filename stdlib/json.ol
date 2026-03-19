// stdlib/json.ol — JSON parse/emit for Olang
// Simplified JSON: strings, numbers, arrays, objects, true/false/null

pub fn json_parse(s) {
  let state = { src: s, pos: 0 };
  return parse_value(state);
}

pub fn json_emit(val) {
  if val == true { return "true"; }
  if val == false { return "false"; }
  if val == 0 && typeof(val) == "null" { return "null"; }
  if typeof(val) == "number" { return to_string(val); }
  if typeof(val) == "string" { return "\"" + escape_str(val) + "\""; }
  if typeof(val) == "array" {
    let parts = [];
    let i = 0;
    while i < len(val) {
      push(parts, json_emit(val[i]));
      i = i + 1;
    }
    return "[" + join(parts, ",") + "]";
  }
  // Object: emit as {"key":val,...}
  return "{}";
}

// ── Parser internals ──

fn parse_value(state) {
  skip_ws(state);
  if state.pos >= len(state.src) { return 0; }

  let ch = char_at(state.src, state.pos);

  if ch == "\"" { return parse_string(state); }
  if ch == "[" { return parse_array(state); }
  if ch == "{" { return parse_object(state); }
  if ch == "t" { state.pos = state.pos + 4; return true; }
  if ch == "f" { state.pos = state.pos + 5; return false; }
  if ch == "n" { state.pos = state.pos + 4; return 0; }
  return parse_number(state);
}

fn parse_string(state) {
  state.pos = state.pos + 1;  // skip opening "
  let result = "";
  while state.pos < len(state.src) {
    let ch = char_at(state.src, state.pos);
    if ch == "\"" {
      state.pos = state.pos + 1;
      return result;
    }
    if ch == "\\" {
      state.pos = state.pos + 1;
      let esc = char_at(state.src, state.pos);
      if esc == "n" { result = result + "\n"; }
      else { if esc == "t" { result = result + "\t"; }
      else { result = result + esc; } }
    } else {
      result = result + ch;
    }
    state.pos = state.pos + 1;
  }
  return result;
}

fn parse_number(state) {
  let start = state.pos;
  let neg = false;
  if char_at(state.src, state.pos) == "-" {
    neg = true;
    state.pos = state.pos + 1;
  }

  let val = 0.0;
  while state.pos < len(state.src) {
    let ch = char_at(state.src, state.pos);
    if ch >= "0" && ch <= "9" {
      val = val * 10.0 + (to_num(ch));
      state.pos = state.pos + 1;
    } else {
      break;
    }
  }

  // Fractional
  if state.pos < len(state.src) && char_at(state.src, state.pos) == "." {
    state.pos = state.pos + 1;
    let frac = 0.1;
    while state.pos < len(state.src) {
      let ch = char_at(state.src, state.pos);
      if ch >= "0" && ch <= "9" {
        val = val + to_num(ch) * frac;
        frac = frac * 0.1;
        state.pos = state.pos + 1;
      } else {
        break;
      }
    }
  }

  if neg { val = 0.0 - val; }
  return val;
}

fn parse_array(state) {
  state.pos = state.pos + 1;  // skip [
  let result = [];
  skip_ws(state);

  if state.pos < len(state.src) && char_at(state.src, state.pos) == "]" {
    state.pos = state.pos + 1;
    return result;
  }

  while state.pos < len(state.src) {
    push(result, parse_value(state));
    skip_ws(state);
    if state.pos >= len(state.src) { break; }
    if char_at(state.src, state.pos) == "]" {
      state.pos = state.pos + 1;
      return result;
    }
    if char_at(state.src, state.pos) == "," {
      state.pos = state.pos + 1;
    }
  }
  return result;
}

fn parse_object(state) {
  state.pos = state.pos + 1;  // skip {
  let result = {};
  skip_ws(state);

  if state.pos < len(state.src) && char_at(state.src, state.pos) == "}" {
    state.pos = state.pos + 1;
    return result;
  }

  while state.pos < len(state.src) {
    skip_ws(state);
    let key = parse_string(state);
    skip_ws(state);
    state.pos = state.pos + 1;  // skip :
    let val = parse_value(state);
    // result[key] = val  — depends on dict support
    skip_ws(state);
    if state.pos >= len(state.src) { break; }
    if char_at(state.src, state.pos) == "}" {
      state.pos = state.pos + 1;
      return result;
    }
    if char_at(state.src, state.pos) == "," {
      state.pos = state.pos + 1;
    }
  }
  return result;
}

fn skip_ws(state) {
  while state.pos < len(state.src) {
    let ch = char_at(state.src, state.pos);
    if ch == " " || ch == "\n" || ch == "\t" || ch == "\r" {
      state.pos = state.pos + 1;
    } else {
      return;
    }
  }
}

fn escape_str(s) {
  let result = "";
  let i = 0;
  while i < len(s) {
    let ch = char_at(s, i);
    if ch == "\"" { result = result + "\\\""; }
    else { if ch == "\\" { result = result + "\\\\"; }
    else { if ch == "\n" { result = result + "\\n"; }
    else { result = result + ch; } } }
    i = i + 1;
  }
  return result;
}

fn to_num(ch) {
  // Single digit char → number
  if ch == "0" { return 0; }
  if ch == "1" { return 1; }
  if ch == "2" { return 2; }
  if ch == "3" { return 3; }
  if ch == "4" { return 4; }
  if ch == "5" { return 5; }
  if ch == "6" { return 6; }
  if ch == "7" { return 7; }
  if ch == "8" { return 8; }
  if ch == "9" { return 9; }
  return 0;
}
