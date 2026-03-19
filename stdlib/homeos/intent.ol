// homeos/intent.ol — Intent classification
// Crisis | Learn | Command | Chat

pub fn estimate_intent(text, emotion) {
  // Crisis (highest priority — DỪNG NGAY nếu nguy hiểm)
  if is_crisis(text) { return "crisis"; }

  // Command (starts with ○{ or system keywords)
  if starts_with(text, "○{") { return "command"; }
  if starts_with(text, "○ ") { return "command"; }

  // System commands
  let sys = ["dream", "stats", "fuse", "trace", "inspect"];
  let i = 0;
  while i < len(sys) {
    if text == sys[i] { return "command"; }
    i = i + 1;
  }

  // Learn
  let learn = ["dạy", "học", "giải thích", "tại sao", "là gì",
               "teach", "learn", "explain", "why", "what is"];
  if contains_any(text, learn) { return "learn"; }

  return "chat";
}

fn is_crisis(text) {
  let keywords = ["tự tử", "muốn chết", "không muốn sống",
                   "suicide", "kill myself", "end my life",
                   "want to die", "tôi chết"];
  return contains_any(text, keywords);
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

fn starts_with(s, prefix) {
  if len(s) < len(prefix) { return false; }
  return substr(s, 0, len(prefix)) == prefix;
}
