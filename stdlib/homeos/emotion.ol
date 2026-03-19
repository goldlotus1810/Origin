// homeos/emotion.ol — Emotion pipeline (V/A/D/I)
// KHÔNG BAO GIỜ trung bình cảm xúc — luôn AMPLIFY qua Silk.

pub fn emotion_new(v, a, d, i) {
  return { v: v, a: a, d: d, i: i };
}

pub fn emotion_zero() {
  return emotion_new(0.0, 0.0, 0.5, 0.0);
}

pub fn blend(a, b, w) {
  return emotion_new(
    a.v * w + b.v * (1.0 - w),
    a.a * w + b.a * (1.0 - w),
    a.d * w + b.d * (1.0 - w),
    a.i * w + b.i * (1.0 - w)
  );
}

pub fn amplify(emo, silk_weight) {
  let factor = 1.0 + silk_weight * 0.618;
  return emotion_new(
    emo.v * factor,
    clamp(emo.a * factor, 0.0, 1.0),
    emo.d,
    clamp(emo.i * factor, 0.0, 1.0)
  );
}

pub fn intensity(emo) {
  return sqrt(emo.v * emo.v + emo.a * emo.a);
}

pub fn is_positive(emo) { return emo.v > 0.1; }
pub fn is_negative(emo) { return emo.v < -0.1; }
pub fn is_neutral(emo) { return abs(emo.v) <= 0.1; }

fn clamp(val, lo, hi) {
  if val < lo { return lo; }
  if val > hi { return hi; }
  return val;
}
