// homeos/gate.ol — SecurityGate (runs FIRST on ALL input)
// Merged: Origin (Rust) structure + origin.olang P_weight math
// Crisis → STOP. Harmful → block. Else → allow.
//
// KEY CHANGE: Uses per-char P_weight V/A (not keyword matching)
// SPEC_D_PIPELINE: 3 layers (Bloom filter, normalized, semantic V/A check)

pub fn gate_check(text) {
  // Layer 1: P_weight V/A check (per-char molecular analysis)
  if security_gate_molecular(text) == 1 {
    return { action: "crisis", response: crisis_response() };
  }
  // Layer 2: Keyword fallback (defense in depth)
  if is_crisis_keyword(text) {
    return { action: "crisis", response: crisis_response() };
  }
  if is_harmful(text) {
    return { action: "block", reason: "harmful content" };
  }
  return { action: "allow" };
}

// PRIMARY: P_weight molecular analysis (from origin.olang)
// Crisis = min V across ALL chars <= 1 AND max A >= 6
// This works for ANY language (because Unicode = universal)
fn security_gate_molecular(text) {
  let min_v = 7;
  let max_a = 0;
  let i = 0;
  while i < len(text) {
    let cp = __char_code(char_at(text, i));
    let pw = p_weight(cp);
    if pw > 0 {
      let v = (__floor(pw / 32)) % 8;
      let a = (__floor(pw / 4)) % 8;
      if v < min_v { min_v = v; }
      if a > max_a { max_a = a; }
    }
    i = i + 1;
  }
  // Crisis: very low valence + very high arousal
  if min_v <= 1 {
    if max_a >= 6 { return 1; }
  }
  return 0;
}

// FALLBACK: Keyword check (defense in depth, not primary)
fn is_crisis_keyword(text) {
  let keywords = ["tự tử", "muốn chết", "không muốn sống",
                   "suicide", "kill myself", "end my life",
                   "want to die"];
  return contains_any(text, keywords);
}

fn crisis_response() {
  return "Bạn đang trải qua khoảnh khắc rất khó khăn. " +
         "Xin hãy gọi đường dây nóng: 1800 599 920 (Việt Nam) " +
         "hoặc 988 (US). Bạn không đơn độc.";
}

fn is_harmful(text) {
  let harmful = ["cách chế bom", "hack password", "ddos"];
  return contains_any(text, harmful);
}

fn contains_any(text, keywords) {
  let i = 0;
  while i < len(keywords) {
    if contains(text, keywords[i]) { return true; }
    i = i + 1;
  }
  return false;
}

fn contains(text, sub) {
  let tl = len(text);
  let sl = len(sub);
  if sl > tl { return false; }
  let i = 0;
  while i <= tl - sl {
    if substr(text, i, sl) == sub { return true; }
    i = i + 1;
  }
  return false;
}
