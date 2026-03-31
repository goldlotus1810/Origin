// homeos/intent.ol — Intent classification
//
// P2.2: Full intent detection với emotion-aware context.
// Priority: Crisis > Command > Heal > Learn > Confirm/Deny > Chat
//
// Handbook Pattern 1: Nếu lặp topic 3+ → Heal (không hỏi thêm)
// Handbook Pattern 6: Crisis → 3-layer detection

pub fn estimate_intent(text, emotion, ctx) {
  // ── Layer 1: Crisis (highest priority — DỪNG NGAY) ─────────
  if is_crisis(text) { return "crisis"; };
  // Semantic crisis: V < -0.9 AND A > 0.8
  if emotion.v < -0.9 && emotion.a > 0.8 { return "crisis"; };

  // ── Layer 2: Command (○{} or system keywords) ──────────────
  if starts_with(text, "○{") { return "command"; };
  if starts_with(text, "○ ") { return "command"; };
  if starts_with(text, "> ") { return "command"; };
  let sys = ["dream", "stats", "fuse", "trace", "inspect", "key.ol"];
  if list_contains(sys, text) { return "command"; };

  // ── Layer 3: Heal (emotion-aware) ──────────────────────────
  // Repetition 3+ lần cùng topic → Heal (không hỏi "tìm hiểu gì")
  if ctx.repetition >= 3 && emotion.v < -0.3 { return "heal"; };
  // Strong negative emotion → Heal
  if emotion.v < -0.5 && emotion.a > 0.4 { return "heal"; };
  let heal_words = ["buồn", "lo", "sợ", "đau", "mệt", "khóc", "cô đơn",
                     "sad", "scared", "lonely", "depressed", "anxious",
                     "hurt", "tired", "stressed"];
  if contains_any(text, heal_words) { return "heal"; };

  // ── Layer 4: Learn / Confirm / Deny ────────────────────────
  let learn_words = ["dạy", "học", "giải thích", "tại sao", "là gì", "thế nào",
                      "teach", "learn", "explain", "why", "what is", "how"];
  if contains_any(text, learn_words) { return "learn"; };

  // Learn command: explicit "ghi nhớ", "hãy học"
  let learn_cmd = ["ghi nhớ", "hãy học", "remember", "memorize"];
  if contains_any(text, learn_cmd) { return "learn_command"; };

  // Confirm knowledge: "đúng rồi", "chính xác"
  let confirm_words = ["đúng", "ok", "ừ", "vâng", "đồng ý", "chính xác",
                        "yes", "correct", "right", "agree"];
  if list_contains(confirm_words, text) { return "confirm"; };

  let deny_words = ["không", "sai", "từ chối", "no", "wrong", "disagree"];
  if list_contains(deny_words, text) { return "deny"; };

  // ── Default: Chat ──────────────────────────────────────────
  return "chat";
}

// ── Crisis detection — 3-layer (Handbook Pattern 6) ──────────

fn is_crisis(text) {
  // Layer 1: Exact match
  let keywords = ["tự tử", "muốn chết", "không muốn sống", "tự sát",
                   "suicide", "kill myself", "end my life",
                   "want to die", "tôi chết"];
  if contains_any(text, keywords) { return true; };

  // Layer 2: Normalized match (strip special chars)
  let normalized = normalize(text);
  if contains_any(normalized, keywords) { return true; };

  // Layer 3: Indirect expressions
  let indirect = ["không muốn thức dậy", "biến mất", "kết thúc tất cả",
                   "don't want to wake up", "disappear", "end it all"];
  if contains_any(text, indirect) { return true; };

  return false;
}

// ── Urgency detection ────────────────────────────────────────

pub fn detect_urgency(text) {
  let level = 0.0;
  // Exclamation marks
  let i = 0;
  while i < len(text) {
    if char_at(text, i) == "!" { let level = level + 0.1; };
    let i = i + 1;
  };
  // ALL CAPS (simplified: check if first 3 chars are uppercase)
  if len(text) >= 3 {
    let c0 = char_at(text, 0);
    let c1 = char_at(text, 1);
    let c2 = char_at(text, 2);
    if c0 >= "A" && c0 <= "Z" && c1 >= "A" && c1 <= "Z" && c2 >= "A" && c2 <= "Z" {
      let level = level + 0.3;
    };
  };
  if level > 1.0 { let level = 1.0; };
  return level;
}

// ── Helpers ──────────────────────────────────────────────────

fn contains_any(text, keywords) {
  let i = 0;
  while i < len(keywords) {
    if contains(text, keywords[i]) { return true; };
    let i = i + 1;
  };
  return false;
}

fn list_contains(list, item) {
  let i = 0;
  while i < len(list) {
    if list[i] == item { return true; };
    let i = i + 1;
  };
  return false;
}

fn contains(text, sub) {
  let tl = len(text);
  let sl = len(sub);
  if sl > tl { return false; };
  if sl == 0 { return false; };
  let i = 0;
  while i <= tl - sl {
    if substr(text, i, i + sl) == sub { return true; };
    let i = i + 1;
  };
  return false;
}

fn starts_with(s, prefix) {
  if len(s) < len(prefix) { return false; };
  return substr(s, 0, len(prefix)) == prefix;
}

fn normalize(text) {
  // Strip dots, dashes, spaces between chars
  // "t.ự t.ử" → "tựtử"
  let result = "";
  let i = 0;
  while i < len(text) {
    let ch = char_at(text, i);
    if ch != "." && ch != "-" && ch != " " {
      let result = result + ch;
    };
    let i = i + 1;
  };
  return result;
}
