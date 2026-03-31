// homeos/response.ol — Response composer (4-part, context-aware)
//
// P2.4: Thay template cứng bằng composer linh hoạt.
// Part 1: Acknowledgment (dựa trên emotion)
// Part 2: Context-specific (dựa trên topic + recall)
// Part 3: Follow-up (dựa trên intent)
// Part 4: Topic reflection (dựa trên instincts)

pub fn compose_response(emotion, intent, tone, instinct, context) {
  let parts = [];

  // Part 1: Acknowledgment — emotion-driven
  let ack = acknowledge(emotion, tone);
  if len(ack) > 0 { push(parts, ack); };

  // Part 2: Context — what we know about the topic
  let ctx_text = context_text(context);
  if len(ctx_text) > 0 { push(parts, ctx_text); };

  // Part 3: Follow-up — intent-driven
  let followup = intent_followup(intent, emotion);
  if len(followup) > 0 { push(parts, followup); };

  // Part 4: Instinct reflection
  if instinct.contradiction { push(parts, "⊥ Mình nhận thấy có điều mâu thuẫn — bạn có muốn nói rõ hơn không?"); };
  if instinct.novelty > 0.7 { push(parts, "✦ Đây là góc nhìn mới — mình muốn tìm hiểu thêm."); };

  // Join parts
  if len(parts) == 0 { return "Mình nghe rồi."; };
  return join(parts, "\n");
}

fn acknowledge(emotion, tone) {
  if tone == "supportive" { return "Mình hiểu cảm giác đó."; };
  if tone == "gentle" { return "Từ từ thôi, không vội đâu."; };
  if tone == "celebratory" { return "Tuyệt vời!"; };
  if tone == "reinforcing" { return "Đúng rồi!"; };
  if tone == "pause" {
    return "Mình nhận thấy bạn đang có nhiều cảm xúc. Muốn dừng lại một chút không?";
  };
  // Neutral — acknowledge based on emotion
  if emotion.v < -0.3 { return "Mình nghe thấy điều bạn nói."; };
  if emotion.v > 0.3 { return "Nghe hay đó!"; };
  return "";
}

fn context_text(context) {
  if context.repetition >= 3 {
    return "Bạn đã nhắc đến chủ đề này nhiều lần — mình hiểu nó quan trọng với bạn.";
  };
  if context.has_cause {
    return "Mình hiểu nguyên nhân rồi.";
  };
  return "";
}

fn intent_followup(intent, emotion) {
  if intent == "heal" {
    if emotion.v < -0.7 {
      return "Bạn không đơn độc. Mình ở đây.";
    };
    return "Bạn muốn chia sẻ thêm không?";
  };
  if intent == "learn" { return "Để mình tìm hiểu thêm cho bạn."; };
  if intent == "confirm" { return "Đã ghi nhận."; };
  if intent == "deny" { return "Mình hiểu. Bạn muốn thử hướng khác không?"; };
  if intent == "chat" { return ""; };
  return "";
}

// ── Legacy API ──────────────────────────────────────────────────

pub fn render(tone, content) {
  if tone == "supportive" { return "Mình hiểu cảm giác đó — " + content; };
  if tone == "gentle" { return "Từ từ thôi — " + content; };
  if tone == "celebratory" { return "Tuyệt vời! " + content; };
  if tone == "reinforcing" { return "Đúng rồi! " + content; };
  if tone == "pause" {
    return "Mình nhận thấy bạn đang có nhiều cảm xúc. Muốn dừng lại một chút không?";
  };
  return content;
}

pub fn format_stats(stm_count, silk_count, dream_pending, qr_count) {
  return "STM: " + to_string(stm_count) + " nodes │ " +
         "Silk: " + to_string(silk_count) + " edges │ " +
         "Dream: " + to_string(dream_pending) + " pending │ " +
         "QR: " + to_string(qr_count) + " signed";
}

fn join(parts, sep) {
  if len(parts) == 0 { return ""; };
  let result = parts[0];
  let i = 1;
  while i < len(parts) {
    let result = result + sep + parts[i];
    let i = i + 1;
  };
  return result;
}
