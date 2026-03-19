// homeos/gate.ol — SecurityGate (chạy TRƯỚC MỌI THỨ)
// Crisis → DỪNG NGAY. Harmful → block. Còn lại → allow.

pub fn gate_check(text) {
  if is_crisis(text) {
    return { action: "crisis", response: crisis_response() };
  }
  if is_harmful(text) {
    return { action: "block", reason: "harmful content" };
  }
  return { action: "allow" };
}

fn is_crisis(text) {
  let keywords = ["tự tử", "muốn chết", "không muốn sống",
                   "suicide", "kill myself", "end my life",
                   "want to die", "tôi chết", "tôi muốn chết"];
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
