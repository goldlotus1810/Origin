// homeos/response.ol — Tone-based response rendering
// ConversationCurve tone → response style

pub fn render(tone, content) {
  if tone == "supportive" { return "Mình hiểu cảm giác đó — " + content; }
  if tone == "gentle" { return "Từ từ thôi — " + content; }
  if tone == "celebratory" { return "Tuyệt vời! " + content; }
  if tone == "reinforcing" { return "Đúng rồi! " + content; }
  if tone == "pause" {
    return "Mình nhận thấy bạn đang có nhiều cảm xúc. Muốn dừng lại một chút không?";
  }
  return content;
}

pub fn render_with_emotion(tone, emotion, content) {
  let prefix = render(tone, "");
  if intensity(emotion) > 0.8 {
    return prefix + content + " 💛";
  }
  return prefix + content;
}

pub fn format_stats(stm_count, silk_count, dream_pending, qr_count) {
  return "STM: " + to_string(stm_count) + " nodes │ " +
         "Silk: " + to_string(silk_count) + " edges │ " +
         "Dream: " + to_string(dream_pending) + " pending │ " +
         "QR: " + to_string(qr_count) + " signed";
}
