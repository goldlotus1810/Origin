// homeos/emotion.ol — Emotion pipeline (V/A/D/I)
//
// P2.2: Chuyển logic từ Rust (context::emotion) sang Olang.
// KHÔNG BAO GIỜ trung bình cảm xúc — luôn AMPLIFY qua Silk.
//
// Flow: text → words → word_affect(w) → compose_sentence → amplify

// ── Constructor ─────────────────────────────────────────────────

pub fn emotion_new(v, a, d, i) {
  return { v: v, a: a, d: d, i: i };
}

pub fn emotion_zero() {
  return emotion_new(0.0, 0.0, 0.5, 0.0);
}

pub fn emotion_neutral() {
  return emotion_new(0.0, 0.2, 0.5, 0.1);
}

// ── Core operations ─────────────────────────────────────────────

// Blend 2 emotions with weight (cho cross-modal fusion)
pub fn blend(a, b, w) {
  return emotion_new(
    a.v * w + b.v * (1.0 - w),
    a.a * w + b.a * (1.0 - w),
    a.d * w + b.d * (1.0 - w),
    a.i * w + b.i * (1.0 - w)
  );
}

// AMPLIFY — spec rule: KHÔNG trung bình, amplify qua Silk weight
// factor = 1 + w × φ⁻¹ (Golden Ratio boost)
pub fn amplify(emo, silk_weight) {
  let factor = 1.0 + silk_weight * 0.618;
  return emotion_new(
    clamp(emo.v * factor, -1.0, 1.0),
    clamp(emo.a * factor, 0.0, 1.0),
    emo.d,
    clamp(emo.i * factor, 0.0, 1.0)
  );
}

// Compose 2 emotions — AMPLIFY, NOT average
// Handbook Pattern 1: compose("buồn", "mất việc") → nặng hơn cả hai
pub fn compose(a, b, silk_weight) {
  let base_v = (a.v + b.v) / 2.0;
  let boost = abs(a.v - base_v) * silk_weight * 0.5;
  let sign = if a.v + b.v > 0.0 { 1.0 } else { -1.0 };
  let composed_v = clamp(base_v + sign * boost, -1.0, 1.0);
  // Arousal: max (không trung bình)
  let composed_a = max(a.a, b.a);
  // Dominance: weighted average OK (không phải emotion)
  let composed_d = (a.d + b.d) / 2.0;
  // Intensity: max
  let composed_i = max(a.i, b.i);
  return emotion_new(composed_v, composed_a, composed_d, composed_i);
}

// ── Sentence affect ─────────────────────────────────────────────

// Process toàn bộ câu → emotion tag
// Dùng word_affect (builtin) cho mỗi từ, compose tất cả
pub fn sentence_affect(text) {
  let words = split(text, " ");
  if len(words) == 0 { return emotion_zero(); };

  let result = emotion_zero();
  let has_emotion = false;
  let i = 0;
  while i < len(words) {
    let w = words[i];
    if len(w) > 0 {
      let word_emo = word_affect(w);
      if abs(word_emo.v) > 0.05 || word_emo.a > 0.1 {
        if has_emotion {
          // Compose — amplify, NOT average
          let result = compose(result, word_emo, 0.8);
        } else {
          let result = word_emo;
          let has_emotion = true;
        };
      };
    };
    let i = i + 1;
  };

  return result;
}

// Word affect lookup — delegate to VM builtin
// word_affect("buồn") → emotion_new(-0.7, 0.6, 0.5, 0.48)
pub fn word_affect(word) {
  // __word_affect is a VM builtin that looks up WORD_AFFECT_TABLE
  // Returns { v, a } or { v: 0, a: 0 } for unknown words
  let result = __word_affect(word);
  return emotion_new(result.v, result.a, 0.5, result.a * 0.8);
}

// ── Queries ─────────────────────────────────────────────────────

pub fn intensity(emo) {
  return sqrt(emo.v * emo.v + emo.a * emo.a);
}

pub fn is_positive(emo) { return emo.v > 0.1; }
pub fn is_negative(emo) { return emo.v < -0.1; }
pub fn is_neutral(emo) { return abs(emo.v) <= 0.1; }
pub fn is_calm(emo) { return emo.a < 0.3; }
pub fn is_urgent(emo) { return emo.a > 0.618; } // φ⁻¹ threshold

// ── Modifiers ───────────────────────────────────────────────────

// Apply urgency modifier (! marks, CAPS, etc.)
pub fn apply_urgency(emo, level) {
  return emotion_new(
    emo.v,
    clamp(emo.a + level * 0.2, 0.0, 1.0),
    emo.d,
    clamp(emo.i + level * 0.15, 0.0, 1.0)
  );
}

// Apply politeness modifier (reduces dominance)
pub fn apply_politeness(emo, level) {
  return emotion_new(
    emo.v,
    emo.a,
    clamp(emo.d - level * 0.1, 0.0, 1.0),
    emo.i
  );
}

// ── Context-aware processing ────────────────────────────────────

// Apply context: repetition → emphasize, contradiction → conflict
pub fn apply_context(emo, ctx) {
  let result = emo;
  // Repetition: same topic 3+ times → amplify
  if ctx.repetition >= 3 {
    let result = amplify(result, 0.3);
  };
  // Contradiction: conflicting emotions → increase arousal
  if ctx.contradiction {
    let result = emotion_new(result.v, clamp(result.a + 0.2, 0.0, 1.0), result.d, result.i);
  };
  // Causality: known cause → reduce uncertainty
  if ctx.has_cause {
    let result = emotion_new(result.v, result.a, clamp(result.d + 0.1, 0.0, 1.0), result.i);
  };
  return result;
}

// ── Helpers ─────────────────────────────────────────────────────

fn clamp(val, lo, hi) {
  if val < lo { return lo; };
  if val > hi { return hi; };
  return val;
}

fn abs(x) {
  if x < 0.0 { return 0.0 - x; };
  return x;
}

fn max(a, b) {
  if a > b { return a; };
  return b;
}

fn sqrt(x) {
  // Newton's method — 5 iterations sufficient for f32
  if x <= 0.0 { return 0.0; };
  let r = x;
  let r = (r + x / r) / 2.0;
  let r = (r + x / r) / 2.0;
  let r = (r + x / r) / 2.0;
  let r = (r + x / r) / 2.0;
  let r = (r + x / r) / 2.0;
  return r;
}
