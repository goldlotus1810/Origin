// stdlib/homeos/encode_formulas.ol — 42 Encode Formulas (TÍNH không TRA)
// Ported from SPEC_MOLECULAR_ENGINE.md §2.4 + §7.1
//
// Mỗi Unicode codepoint → P_weight u16 qua 42 formulas:
//   1 master dispatcher (codepoint_group)
//   5 dimension encoders (encode_s, encode_r, encode_v, encode_a, encode_t)
//   36 sub-classifiers (keyword flags, range checks, NRC-VAD lookup)
//
// NGUYÊN TẮC: TÍNH bằng formulas. KHÔNG tra bảng cứng.

// ═══ MASTER: codepoint_group — Classify codepoint into 4 groups ═══
// Group 1: SDF (S dominant) — 13 Unicode blocks
// Group 2: MATH (R dominant) — 18 Unicode blocks
// Group 3: EMOTICON (V+A dominant) — 17 Unicode blocks
// Group 4: MUSICAL (T dominant) — 7 Unicode blocks
// Group 0: Not in 59 blocks → neutral

pub fn codepoint_group(cp) {
  // GROUP 1 — SDF: Arrows, Geometric, Box Drawing, etc.
  if cp >= 0x2190 && cp <= 0x21FF { return 1; }  // Arrows
  if cp >= 0x27F0 && cp <= 0x27FF { return 1; }  // Supplemental Arrows-A
  if cp >= 0x2900 && cp <= 0x297F { return 1; }  // Supplemental Arrows-B
  if cp >= 0x2B00 && cp <= 0x2BFF { return 1; }  // Misc Symbols and Arrows
  if cp >= 0x25A0 && cp <= 0x25FF { return 1; }  // Geometric Shapes
  if cp >= 0x1F780 && cp <= 0x1F7FF { return 1; } // Geometric Shapes Ext
  if cp >= 0x2500 && cp <= 0x257F { return 1; }  // Box Drawing
  if cp >= 0x2580 && cp <= 0x259F { return 1; }  // Block Elements
  if cp >= 0x2800 && cp <= 0x28FF { return 1; }  // Braille Patterns
  if cp >= 0x2700 && cp <= 0x27BF { return 1; }  // Dingbats
  if cp >= 0x2300 && cp <= 0x23FF { return 1; }  // Misc Technical
  if cp >= 0x2600 && cp <= 0x26FF { return 1; }  // Misc Symbols
  if cp >= 0x2460 && cp <= 0x24FF { return 1; }  // Enclosed Alphanumerics

  // GROUP 2 — MATH: Math Operators, Number Forms, etc.
  if cp >= 0x2200 && cp <= 0x22FF { return 2; }  // Math Operators
  if cp >= 0x2A00 && cp <= 0x2AFF { return 2; }  // Supplemental Math Operators
  if cp >= 0x27C0 && cp <= 0x27EF { return 2; }  // Misc Math Symbols-A
  if cp >= 0x2980 && cp <= 0x29FF { return 2; }  // Misc Math Symbols-B
  if cp >= 0x1D400 && cp <= 0x1D7FF { return 2; } // Math Alphanumeric
  if cp >= 0x2100 && cp <= 0x214F { return 2; }  // Letterlike Symbols
  if cp >= 0x2150 && cp <= 0x218F { return 2; }  // Number Forms
  if cp >= 0x2070 && cp <= 0x209F { return 2; }  // Super/Subscripts
  if cp >= 0x20A0 && cp <= 0x20CF { return 2; }  // Currency Symbols
  if cp >= 0x2000 && cp <= 0x206F { return 2; }  // General Punctuation

  // GROUP 3 — EMOTICON: Emoji, Emoticons, etc.
  if cp >= 0x1F600 && cp <= 0x1F64F { return 3; } // Emoticons
  if cp >= 0x1F300 && cp <= 0x1F5FF { return 3; } // Misc Symbols & Pictographs
  if cp >= 0x1F900 && cp <= 0x1F9FF { return 3; } // Supplemental Symbols
  if cp >= 0x1FA00 && cp <= 0x1FA6F { return 3; } // Chess Symbols
  if cp >= 0x1FA70 && cp <= 0x1FAFF { return 3; } // Symbols Extended-A
  if cp >= 0x2702 && cp <= 0x27B0 { return 3; }  // Dingbats subset
  if cp >= 0x1F680 && cp <= 0x1F6FF { return 3; } // Transport & Map

  // GROUP 4 — MUSICAL: Musical Symbols, etc.
  if cp >= 0x1D100 && cp <= 0x1D1FF { return 4; } // Musical Symbols
  if cp >= 0x2669 && cp <= 0x266F { return 4; }  // Music note chars
  if cp >= 0x1D200 && cp <= 0x1D24F { return 4; } // Ancient Greek Musical
  if cp >= 0x4DC0 && cp <= 0x4DFF { return 4; }  // Yijing Hexagram

  return 0;  // Not in 59 blocks
}

// ═══ ENCODE MASTER: codepoint → P_weight u16 ═══

pub fn encode_codepoint(cp) {
  let group = codepoint_group(cp);
  let s = 0;
  let r = 0;
  let v = 4;  // neutral
  let a = 4;  // neutral
  let t = 0;

  if group == 1 { s = encode_shape(cp); }
  if group == 2 { r = encode_relation(cp); }
  if group == 3 {
    v = encode_valence(cp);
    a = encode_arousal(cp);
  }
  if group == 4 { t = encode_time(cp); }

  return _kt_pack(s, r, v, a, t);
}

// ═══ f_S: Shape Encoder (10 sub-classifiers) ═══
// S.0: arrow, S.1: geometric, S.2: line/box, S.3: fill/block,
// S.4: symbol/sign, S.5: size, S.6: position, S.7: pattern,
// S.8: astro, S.9: technical

pub fn encode_shape(cp) {
  // S.0 Arrows (→ ← ↑ ↓ ↔ ⇒)
  if cp >= 0x2190 && cp <= 0x21FF { return 0; }
  if cp >= 0x27F0 && cp <= 0x27FF { return 0; }
  if cp >= 0x2900 && cp <= 0x297F { return 0; }

  // S.1 Geometric (● ■ ▲ ◆ ○ □ △)
  if cp >= 0x25A0 && cp <= 0x25FF { return 1; }
  if cp >= 0x1F780 && cp <= 0x1F7FF { return 1; }

  // S.2 Line/Box Drawing (─ │ ┌ ┐ └ ┘)
  if cp >= 0x2500 && cp <= 0x257F { return 2; }

  // S.3 Fill/Block (█ ▓ ▒ ░)
  if cp >= 0x2580 && cp <= 0x259F { return 3; }

  // S.4 Symbol/Sign (✓ ✗ ✦ ★)
  if cp >= 0x2700 && cp <= 0x27BF { return 4; }

  // S.5-S.9: other SDF shapes
  if cp >= 0x2800 && cp <= 0x28FF { return 7; }  // Braille = pattern
  if cp >= 0x2300 && cp <= 0x23FF { return 9; }  // Technical
  if cp >= 0x2600 && cp <= 0x26FF { return 8; }  // Misc (astro, weather)

  return 5;  // default
}

// ═══ f_R: Relation Encoder (10 sub-classifiers) ═══
// R.0: operator, R.1: set/logic, R.2: comparison, R.3: number,
// R.4: letter, R.5: punctuation, R.6: currency, R.7: superscript

pub fn encode_relation(cp) {
  // R.0 Operators (+, -, ×, ÷, ∫, Σ, ∏)
  if cp >= 0x2200 && cp <= 0x22FF { return 0; }
  if cp >= 0x2A00 && cp <= 0x2AFF { return 0; }

  // R.1 Set/Logic (∈, ⊂, ∪, ∩, ∧, ∨, ¬)
  if cp >= 0x27C0 && cp <= 0x27EF { return 1; }

  // R.2 Comparison (=, <, >, ≈, ≡, ≤, ≥)
  if cp >= 0x2980 && cp <= 0x29FF { return 2; }

  // R.3 Number/Digit
  if cp >= 0x2150 && cp <= 0x218F { return 3; }  // Number Forms
  if cp >= 0x2070 && cp <= 0x209F { return 3; }  // Super/Subscripts

  // R.4 Letterlike (ℕ, ℤ, ℝ, ℂ)
  if cp >= 0x2100 && cp <= 0x214F { return 4; }

  // R.5 Punctuation
  if cp >= 0x2000 && cp <= 0x206F { return 5; }

  // R.6 Currency ($, €, £, ¥)
  if cp >= 0x20A0 && cp <= 0x20CF { return 6; }

  // R.7 Math Alphanumeric (𝐴, 𝑩, 𝒞)
  if cp >= 0x1D400 && cp <= 0x1D7FF { return 7; }

  return 4;  // default
}

// ═══ f_V: Valence Encoder ═══
// Emoji subgroup → default V score → quantize to 3 bits (0-7)
// V=0: very negative, V=4: neutral, V=7: very positive

pub fn encode_valence(cp) {
  // Face-smiling, face-affection → positive
  if cp >= 0x1F600 && cp <= 0x1F64F { return 6; }  // +0.8
  if cp >= 0x1F970 && cp <= 0x1F97F { return 7; }  // +0.9

  // Face-neutral → neutral
  if cp >= 0x1F910 && cp <= 0x1F92F { return 4; }  // 0.0

  // Face-negative → negative
  if cp >= 0x1F630 && cp <= 0x1F640 { return 1; }  // -0.7

  // Heart → positive
  if cp == 0x2764 { return 7; }  // +0.85

  // Nature → slightly positive
  if cp >= 0x1F300 && cp <= 0x1F3FF { return 5; }  // +0.3

  // Transport → neutral
  if cp >= 0x1F680 && cp <= 0x1F6FF { return 4; }

  // Objects-negative → slightly negative
  if cp >= 0x1F480 && cp <= 0x1F4FF { return 3; }  // -0.3

  return 4;  // neutral default
}

// ═══ f_A: Arousal Encoder ═══
// Energy level → quantize to 3 bits (0-7)
// A=0: very calm, A=4: neutral, A=7: very excited

pub fn encode_arousal(cp) {
  // Face-smiling → mild arousal
  if cp >= 0x1F600 && cp <= 0x1F64F { return 5; }  // +0.3

  // Face-neutral → low arousal
  if cp >= 0x1F910 && cp <= 0x1F92F { return 3; }  // -0.3

  // Face-negative → HIGH arousal (fear, anger)
  if cp >= 0x1F630 && cp <= 0x1F640 { return 7; }  // +0.8

  // Sport → HIGH arousal
  if cp >= 0x1F3C0 && cp <= 0x1F3CF { return 7; }  // +0.9

  // Sleeping → LOW arousal
  if cp == 0x1F634 { return 1; }  // -0.7

  // Nature → low-mid
  if cp >= 0x1F300 && cp <= 0x1F3FF { return 3; }

  return 4;  // neutral default
}

// ═══ f_T: Time Encoder ═══
// T.0: static, T.1: slow, T.2: medium, T.3: fast

pub fn encode_time(cp) {
  // Hexagrams → Static (timeless divination)
  if cp >= 0x4DC0 && cp <= 0x4DFF { return 0; }

  // Musical notes → Slow (whole, half notes)
  if cp >= 0x1D15C && cp <= 0x1D164 { return 1; }

  // Musical dynamics (forte, piano) → Fast
  if cp >= 0x1D18C && cp <= 0x1D1A9 { return 3; }

  // Ancient Greek Musical → Medium
  if cp >= 0x1D200 && cp <= 0x1D24F { return 2; }

  // Default musical → Medium
  return 2;
}

// ═══ Encode text → chain of P_weights ═══

pub fn encode_text(text) {
  let chain = [];
  let i = 0;
  while i < len(text) {
    let cp = __char_code(char_at(text, i));
    let pw = encode_codepoint(cp);
    if pw > 0 { push(chain, pw); }
    i = i + 1;
  }
  return chain;
}

// ═══ Compose chain → single molecule (LCA) ═══

pub fn chain_summary(chain) {
  if len(chain) == 0 { return 0; }
  if len(chain) == 1 { return chain[0]; }
  return compose(chain);  // from knowtree.ol
}
