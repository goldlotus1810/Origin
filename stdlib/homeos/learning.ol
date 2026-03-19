// homeos/learning.ol — Learning pipeline orchestration
// Gate → Encode → Instinct → STM → Silk → Curve

pub fn process_one(text, emotion, context) {
  // Full learning pipeline for one input

  // 1. SecurityGate (LUÔN chạy trước — bất biến)
  let gate = gate_check(text);
  if gate.action == "crisis" {
    return { ok: false, response: gate.response, action: "crisis" };
  }
  if gate.action == "block" {
    return { ok: false, response: "Nội dung không phù hợp.", action: "block" };
  }

  // 2. Encode text → molecular observation
  let mol = text_to_mol(text, emotion);
  let chain_hash = hash_str(text);
  let observation = { mol: mol, hash: chain_hash, text: text };

  // 3. Run instincts
  let instinct = run_instincts(observation, context);
  if instinct.action == "silence" {
    return { ok: true, response: "", action: "silence" };
  }

  // 4. Push to STM
  stm_push(context.stm, chain_hash, mol, emotion);

  // 5. Co-activate Silk (words that appear together)
  co_activate_text(context.silk, text, emotion);

  // 6. Walk emotion through Silk (amplification)
  let amplified = walk_emotion(context.silk, chain_hash, emotion, 3);

  // 7. Update ConversationCurve
  curve_push(context.curve, amplified);

  // 8. Get response tone
  let tone = curve_tone(context.curve);

  return {
    ok: true,
    emotion: amplified,
    tone: tone,
    instinct: instinct,
    action: "learned"
  };
}

// ── Text → Molecule (simplified encoding) ──

fn text_to_mol(text, emotion) {
  // Map text + emotion → 5D molecule
  // Shape: based on text type (statement=Sphere, question=Circle, command=Triangle)
  let s = 1;  // default Sphere
  if ends_with(text, "?") { s = 5; }  // Circle for questions
  if starts_with(text, "!") || starts_with(text, "○{") { s = 4; }  // Triangle for commands

  // Relation: default Member
  let r = 1;

  // Valence/Arousal from emotion
  let v = clamp(128 + emotion.v * 127, 0, 255);
  let a = clamp(emotion.a * 255, 0, 255);

  // Time: based on text length
  let t = 3;  // Medium
  if len(text) < 10 { t = 4; }   // Short = Fast
  if len(text) > 100 { t = 2; }  // Long = Slow

  return mol_new(s, r, v, a, t);
}

// ── Co-activate words in text ──

fn co_activate_text(silk, text, emotion) {
  // Split text into words, co-activate adjacent pairs
  let words = split_words(text);
  let n = len(words);
  if n < 2 { return; }

  let i = 0;
  while i < n - 1 {
    let hash_a = hash_str(words[i]);
    let hash_b = hash_str(words[i + 1]);
    co_activate(silk, hash_a, hash_b, emotion);
    i = i + 1;
  }

  // Also co-activate with proximity decay (window of 5)
  i = 0;
  while i < n {
    let j = i + 2;
    while j < n && j < i + 5 {
      let decay = 0.618 / (j - i);  // farther = weaker
      let hash_a = hash_str(words[i]);
      let hash_b = hash_str(words[j]);
      let scaled_emotion = {
        v: emotion.v * decay,
        a: emotion.a * decay,
        d: emotion.d,
        i: emotion.i * decay
      };
      co_activate(silk, hash_a, hash_b, scaled_emotion);
      j = j + 1;
    }
    i = i + 1;
  }
}

fn split_words(text) {
  // Simple word splitter: split on spaces
  let words = [];
  let current = "";
  let i = 0;
  let n = len(text);
  while i < n {
    let ch = char_at(text, i);
    if ch == " " || ch == "\n" || ch == "\t" {
      if len(current) > 0 {
        push(words, current);
        current = "";
      }
    } else {
      current = current + ch;
    }
    i = i + 1;
  }
  if len(current) > 0 {
    push(words, current);
  }
  return words;
}

fn starts_with(s, prefix) {
  if len(s) < len(prefix) { return false; }
  return substr(s, 0, len(prefix)) == prefix;
}

fn ends_with(s, suffix) {
  if len(s) < len(suffix) { return false; }
  return substr(s, len(s) - len(suffix), len(suffix)) == suffix;
}

// ── Context factory ──

pub fn context_new() {
  return {
    stm: stm_new(),
    silk: silk_new(),
    curve: curve_new(),
  };
}

pub fn context_dream(context) {
  // Run dream cycle on current context
  return dream_cycle(context.stm, context.silk);
}
