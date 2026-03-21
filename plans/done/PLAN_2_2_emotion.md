# PLAN 2.2 — Emotion Pipeline bằng Olang (~380 LOC)

**Phụ thuộc:** PLAN_2_1 (stdlib: result.ol, format.ol, mol.ol, chain.ol)
**Mục tiêu:** Port emotion pipeline từ Rust sang Olang. Chạy trên ASM/WASM VM.
**Tham chiếu:** `crates/context/src/emotion.rs`, `curve.rs`, `intent.rs`

---

## Files cần viết

| File | LOC | Port từ Rust | Mô tả |
|------|-----|-------------|-------|
| `emotion.ol` | ~150 | `context/emotion.rs` | V/A/D/I blending, sentence_affect, word_affect |
| `curve.ol` | ~130 | `context/curve.rs` | ConversationCurve, tone detection, f'/f'' |
| `intent.ol` | ~100 | `context/intent.rs` | Crisis/Learn/Command/Chat classification |

---

## emotion.ol — Cảm xúc đa tầng

```
// EmotionTag = { v: f64, a: f64, d: f64, i: f64 }
// v = valence (-1..1), a = arousal (0..1), d = dominance (0..1), i = intensity (0..1)

fn emotion_new(v, a, d, i) { return { v: v, a: a, d: d, i: i }; }
fn emotion_zero() { return emotion_new(0.0, 0.0, 0.5, 0.0); }

fn blend(a, b, w) {
  // KHÔNG trung bình — amplify qua weight (QT: AMPLIFY, không average)
  return emotion_new(
    a.v * w + b.v * (1.0 - w),
    a.a * w + b.a * (1.0 - w),
    a.d * w + b.d * (1.0 - w),
    a.i * w + b.i * (1.0 - w)
  );
}

fn amplify(emo, silk_weight) {
  // Silk amplification: emo * (1 + weight * φ⁻¹)
  let factor = 1.0 + silk_weight * 0.618;
  return emotion_new(
    emo.v * factor,
    clamp(emo.a * factor, 0.0, 1.0),
    emo.d,
    clamp(emo.i * factor, 0.0, 1.0)
  );
}

fn word_affect(word) {
  // Lookup từ builtin lexicon (~3000 từ trong Rust VM)
  // Cho WASM/ASM VM: host_emit_event → host lookup → return
  // Tạm: builtin call __word_affect(word) → EmotionTag
}

fn sentence_affect(words) {
  // 50% paragraph + 50% word blend
  // Sliding window 5 từ, proximity decay
}
```

---

## curve.ol — ConversationCurve

```
// f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
// f_conv = V(t) + 0.5×V'(t) + 0.25×V''(t)

fn curve_new() {
  return {
    history: [],     // [{ v: f64, ts: f64 }]
    window: 8,       // lookback window
  };
}

fn curve_push(curve, emotion) {
  push(curve.history, { v: emotion.v, ts: 0.0 });
  if len(curve.history) > 32 {
    // Keep last 32 entries
    curve.history = skip(curve.history, len(curve.history) - 32);
  }
}

fn curve_tone(curve) {
  // Compute f' and f'' from last N entries
  let n = len(curve.history);
  if n < 3 { return "neutral"; }

  let v0 = curve.history[n - 3].v;
  let v1 = curve.history[n - 2].v;
  let v2 = curve.history[n - 1].v;

  let f_prime = v2 - v1;          // first derivative
  let f_double = (v2 - 2.0*v1 + v0);  // second derivative

  // Tone detection rules
  if f_prime < -0.15 { return "supportive"; }
  if f_double < -0.25 { return "pause"; }
  if f_prime > 0.15 { return "reinforcing"; }
  if f_double > 0.25 { if v2 > 0.0 { return "celebratory"; } }
  if v2 < -0.20 { return "gentle"; }

  return "neutral";
}

fn curve_variance(curve) {
  // Window variance — emotional instability detection
}
```

---

## intent.ol — Intent Classification

```
fn estimate_intent(text, emotion) {
  // IntentKind: Crisis | Learn | Command | Chat

  // Crisis keywords (highest priority)
  let crisis_words = ["tự tử", "chết", "không muốn sống", "suicide"];
  if contains_any(text, crisis_words) { return "crisis"; }

  // Command detection (starts with ○{ or system keywords)
  if starts_with(text, "○{") { return "command"; }

  // Learn detection
  let learn_words = ["dạy", "học", "giải thích", "tại sao", "là gì"];
  if contains_any(text, learn_words) { return "learn"; }

  // Default
  return "chat";
}
```

---

## Rào cản

| Rào cản | Giải pháp |
|---------|-----------|
| word_affect cần lexicon 3000 từ | Phase 1: builtin __word_affect. Phase 2+: load từ knowledge |
| sentence_affect cần NLP | Simplified: keyword matching + proximity |
| Curve cần history | In-memory array, reset per session |
| Crisis detection cần chính xác | Conservative: keyword list + low threshold |

---

## Definition of Done

- [ ] `emotion.ol` compile + 3 tests (blend, amplify, zero)
- [ ] `curve.ol` compile + 3 tests (push, tone detection, variance)
- [ ] `intent.ol` compile + 3 tests (crisis, command, chat)
- [ ] Emotion pipeline end-to-end: text → emotion → tone

## Ước tính: 1 ngày
