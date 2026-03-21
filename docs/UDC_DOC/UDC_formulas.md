# UDC Encode Pipeline — Codepoint → P_weight (42 công thức)

> **Vai trò:** Spec kỹ thuật encode — định nghĩa cách chuyển Unicode codepoint → P_weight 2 bytes.
> Đây là **encode pipeline** (bit layout, quantization, data sources).
>
> **Công thức vật lý/hình học chi tiết** cho từng chiều → xem các file `_tree.md`:
> `UDC_A_AROUSAL_tree.md`, `UDC_V_VALENCE_tree.md`, `UDC_R_RELATION_tree.md`,
> `UDC_S0_ARROW_tree.md`, `UDC_S1_GEOMETRIC_tree.md`, `UDC_S2_BOXDRAWING_tree.md`,
> `UDC_S3_S7_tree.md`, `UDC_T_TIME_tree.md`

---

## Tổng quan: Bao nhiêu công thức?

```
Tổng: 42 công thức

  Tầng 1 — Master encoder:           1 công thức
  Tầng 2 — Dimension encoders:       5 công thức  (S, R, V, A, T)
  Tầng 3 — Group classifiers:       36 công thức  (phân loại nhóm con)

  ┌─────────────────────────────────────────────────────────┐
  │  f₀: encode(cp) → P_weight [S, R, V, A, T]             │  ← 1
  │       │                                                  │
  │       ├── f_S(cp) → shape_group                         │  ← 1
  │       │    └── 10 group classifiers                     │  ← 10
  │       ├── f_R(cp) → relation_channel                    │  ← 1
  │       │    └── 10 group classifiers                     │  ← 10
  │       ├── f_V(cp) → valence_score                       │  ← 1
  │       │    └── 5 level quantizers                       │  ← 5
  │       ├── f_A(cp) → arousal_score                       │  ← 1
  │       │    └── 5 level quantizers                       │  ← 5
  │       └── f_T(cp) → time_param                          │  ← 1
  │            └── 6 group classifiers                      │  ← 6
  │                                                          │
  │  TỔNG = 1 + 5 + (10+10+5+5+6) = 42                     │
  └─────────────────────────────────────────────────────────┘
```

---

## Tầng 1 — Master Encoder (1 công thức)

### F₀: encode_codepoint(cp)

```
Input:  cp ∈ [0x0000 .. 0xFFFF]  (Unicode codepoint)
Output: P_weight = [S, R, V, A, T] = 2 bytes (10 bits used)

F₀(cp) = [ f_S(cp),  f_R(cp),  f_V(cp),  f_A(cp),  f_T(cp) ]
            4 bit     4 bit     3 bit     3 bit     2 bit
            ───────── ───────── ───────── ───────── ─────────
            0..15     0..15     0..7      0..7      0..3

Nếu cp thuộc SDF blocks    → S có giá trị, R=0, V=0, A=0, T=0
Nếu cp thuộc MATH blocks   → R có giá trị, S=0, V=0, A=0, T=0
Nếu cp thuộc EMOTICON      → V và A có giá trị, S=0, R=0, T=0
Nếu cp thuộc MUSICAL       → T có giá trị, S=0, R=0, V=0, A=0
Nếu cp không thuộc 58 blk  → [0, 0, 0, 0, 0]
```

---

## Tầng 2 + 3 — S (Shape) · 1 encoder + 10 classifiers = 11 công thức

### F_S: shape_encode(cp)

```
Input:  cp thuộc 13 SDF blocks
Output: S ∈ [0..15] (4 bit) = chỉ số nhóm hình dạng

F_S(cp) = group_id  where:
```

| ID | Công thức (classifier) | Tên nhóm | Điều kiện | Ví dụ |
|----|----------------------|----------|-----------|-------|
| **S.0** | `is_arrow(cp)` | ARROW/DIRECTION | char_name chứa "ARROW", "HARPOON", "WARDS" | → ← ↑ ↓ ↗ ⇒ |
| **S.1** | `is_geometric(cp)` | GEOMETRIC_SHAPE | char_name chứa "SQUARE", "CIRCLE", "TRIANGLE", "DIAMOND", "STAR" | ■ ● ▲ ◆ ★ |
| **S.2** | `is_line(cp)` | LINE/STROKE | char_name chứa "BOX DRAWINGS", "HORIZONTAL", "VERTICAL" | ─ │ ┌ ┐ └ ┘ |
| **S.3** | `is_fill(cp)` | FILL/SHADE | char_name chứa "BLOCK", "SHADE", "QUADRANT", "HALF" | ▀ ▄ █ ░ ▒ ▓ |
| **S.4** | `is_symbol(cp)` | SYMBOL_TYPE | char_name chứa "SYMBOL", "SIGN", "MARK", "KEYBOARD" | ⌘ ⏏ ✂ ✉ |
| **S.5** | `is_size(cp)` | SIZE/WEIGHT | char_name chứa "SMALL", "MEDIUM", "LARGE", "HEAVY", "LIGHT" | (modifier) |
| **S.6** | `is_position(cp)` | POSITION/ORIENTATION | char_name chứa "UPPER", "LOWER", "LEFT", "RIGHT", "TURNED" | (modifier) |
| **S.7** | `is_pattern(cp)` | PATTERN/TEXTURE | char_name chứa "BRAILLE", "PATTERN", "DOTS", "DINGBAT" | ⠁ ⠃ ⠇ |
| **S.8** | `is_astro(cp)` | ASTRO/PLANET | char_name chứa planet/astro names (ZEUS, HADES...) | ⯓ ⯔ |
| **S.9** | `is_technical(cp)` | TECHNICAL/DEVICE | char_name chứa "APL", "DENTISTRY", "BENZENE" | ⎕ ⍟ |

```
Mỗi classifier: is_X(cp) → bool
Logic: match char_name(cp) against keyword set

Nếu cp match nhiều nhóm → ưu tiên: arrow > geometric > line > fill > pattern > symbol > other
```

---

## Tầng 2 + 3 — R (Relation) · 1 encoder + 10 classifiers = 11 công thức

### F_R: relation_encode(cp)

```
Input:  cp thuộc 21 MATH blocks
Output: R ∈ [0..15] (4 bit) = kênh quan hệ

F_R(cp) = channel_id  where:
```

| ID | Công thức (classifier) | Tên nhóm | Điều kiện | Ví dụ |
|----|----------------------|----------|-----------|-------|
| **R.0** | `is_operator(cp)` | MATH_OPERATOR | "PLUS", "MINUS", "INTEGRAL", "SUMMATION", "PRODUCT" | + − ∫ ∑ ∏ |
| **R.1** | `is_set_logic(cp)` | SET/LOGIC | "ELEMENT", "SUBSET", "UNION", "INTERSECTION", "FOR ALL" | ∈ ⊂ ∪ ∩ ∀ |
| **R.2** | `is_comparison(cp)` | COMPARISON | "EQUAL", "GREATER", "LESS", "SIMILAR", "APPROXIMATE" | = ≈ ≤ ≥ ≡ |
| **R.3** | `is_number(cp)` | NUMBER/NUMERAL | "DIGIT", "NUMBER", "NUMERAL", "COUNTING" | 0-9 Ⅰ Ⅱ |
| **R.4** | `is_letter_script(cp)` | LETTER/SCRIPT | "MATHEMATICAL", "BOLD", "ITALIC", "FRAKTUR", "DOUBLE-STRUCK" | 𝐀 𝑨 𝔄 𝟎 |
| **R.5** | `is_fraction(cp)` | FRACTION/RATIO | "FRACTION", "HALF", "THIRD", "QUARTER" | ½ ⅓ ¼ |
| **R.6** | `is_punctuation(cp)` | PUNCTUATION | "COMMA", "COLON", "BRACKET", "DASH", "ELLIPSIS" | ‐ … ‹ › |
| **R.7** | `is_currency(cp)` | CURRENCY | "DOLLAR", "EURO", "POUND", "RUPEE", "BITCOIN" | $ € £ ₹ ₿ |
| **R.8** | `is_ancient(cp)` | ANCIENT_SYSTEM | "CUNEIFORM", "ACROPHONIC", "SIYAQ", "ROMAN" | 𐄂 𒐕 |
| **R.9** | `is_formatting(cp)` | FORMATTING/CONTROL | "SYMBOL FOR", "ACTIVATE", "INHIBIT", "FORMAT" | ␀ ␍ |

```
Mỗi classifier: is_X(cp) → bool
Logic: match char_name(cp) against keyword set

Nếu cp match nhiều nhóm → ưu tiên: operator > set > comparison > number > letter > fraction > other
```

---

## Tầng 2 + 3 — V (Valence) · 1 encoder + 5 quantizers = 6 công thức

### F_V: valence_encode(cp)

```
Input:  cp thuộc 17 EMOTICON blocks
Output: V ∈ [0..7] (3 bit) = mức valence

F_V(cp) = quantize_V( raw_valence(cp) )
```

### F_V.raw: raw_valence(cp) → score ∈ [-1.0, +1.0]

```
Nguồn 1 — NRC-VAD Lexicon:
  Nếu char_name(cp) → word match trong NRC-VAD:
    raw_V = nrc_vad[word].valence        (đã có sẵn: 54,801 terms)

Nguồn 2 — Emoji subgroup mapping:
  Nếu cp ∈ emoji-test subgroup:
    "face-smiling"        → +0.8
    "face-affection"      → +0.9
    "heart"               → +0.85
    "face-tongue"         → +0.5
    "face-neutral"        → +0.0
    "face-sleepy"         → -0.1
    "face-concerned"      → -0.4
    "face-negative"       → -0.7
    "face-unwell"         → -0.6
    "warning"             → -0.5
    (chi tiết cho mỗi subgroup)

Nguồn 3 — Fallback theo General_Category:
  gc = So (Other_Symbol):   raw_V = 0.0 (trung tính)
  gc = Sm (Math_Symbol):    raw_V = 0.0
```

### F_V.q[0..4]: quantize_V(raw) → V ∈ [0..7]

| Quantizer | Điều kiện | V output | Ý nghĩa |
|-----------|-----------|----------|---------|
| **V.q0** | raw > +0.5 | 7 | Rất tích cực (joy, love, triumph) |
| **V.q1** | +0.2 < raw ≤ +0.5 | 5~6 | Tích cực (happy, pleased, hopeful) |
| **V.q2** | -0.2 ≤ raw ≤ +0.2 | 3~4 | Trung tính (neutral, calm, factual) |
| **V.q3** | -0.5 < raw < -0.2 | 1~2 | Tiêu cực (sad, disappointed, worried) |
| **V.q4** | raw ≤ -0.5 | 0 | Rất tiêu cực (hate, horror, despair) |

```
quantize_V(raw) = clamp( round((raw + 1.0) / 2.0 × 7), 0, 7 )

  raw = -1.0  →  (0.0 / 2.0) × 7 = 0.0  →  V = 0
  raw =  0.0  →  (1.0 / 2.0) × 7 = 3.5  →  V = 4  (trung tính)
  raw = +1.0  →  (2.0 / 2.0) × 7 = 7.0  →  V = 7
```

---

## Tầng 2 + 3 — A (Arousal) · 1 encoder + 5 quantizers = 6 công thức

### F_A: arousal_encode(cp)

```
Input:  cp thuộc 17 EMOTICON blocks (CÙNG blocks với V)
Output: A ∈ [0..7] (3 bit) = mức arousal

F_A(cp) = quantize_A( raw_arousal(cp) )
```

### F_A.raw: raw_arousal(cp) → score ∈ [-1.0, +1.0]

```
Nguồn 1 — NRC-VAD Lexicon:
  raw_A = nrc_vad[word].arousal

Nguồn 2 — Emoji subgroup mapping:
  "face-smiling"        → +0.3  (vui nhưng vừa phải)
  "face-affection"      → +0.5  (kích thích tình cảm)
  "face-concerned"      → +0.6  (lo lắng = kích thích cao)
  "face-negative"       → +0.8  (giận dữ = rất kích thích)
  "face-sleepy"         → -0.7  (buồn ngủ = rất yên tĩnh)
  "face-neutral"        → -0.3  (bình thản)
  "person-sport"        → +0.9  (thể thao = cực kích thích)
  "person-resting"      → -0.8  (nghỉ ngơi = cực yên tĩnh)
  "transport-ground"    → +0.4  (di chuyển = kích thích vừa)
  "sky & weather"       → +0.2  (thời tiết = nhẹ)

Nguồn 3 — Fallback:
  gc = So: raw_A = 0.0
```

### F_A.q[0..4]: quantize_A(raw) → A ∈ [0..7]

| Quantizer | Điều kiện | A output | Ý nghĩa |
|-----------|-----------|----------|---------|
| **A.q0** | raw > +0.5 | 7 | Cực kích thích (rage, ecstasy, panic) |
| **A.q1** | +0.2 < raw ≤ +0.5 | 5~6 | Kích thích cao (excited, alert, tense) |
| **A.q2** | -0.2 ≤ raw ≤ +0.2 | 3~4 | Trung tính (neutral, content, pensive) |
| **A.q3** | -0.5 < raw < -0.2 | 1~2 | Yên tĩnh (calm, relaxed, serene) |
| **A.q4** | raw ≤ -0.5 | 0 | Cực yên tĩnh (sleepy, bored, lethargic) |

```
quantize_A(raw) = clamp( round((raw + 1.0) / 2.0 × 7), 0, 7 )

  (cùng công thức với V, khác đầu vào)
```

---

## Tầng 2 + 3 — T (Time) · 1 encoder + 6 classifiers = 7 công thức

### F_T: time_encode(cp)

```
Input:  cp thuộc 7 MUSICAL blocks
Output: T ∈ [0..3] (2 bit) = tham số thời gian / spline

F_T(cp) = time_class(cp)
```

| ID | Công thức (classifier) | Tên nhóm | Ý nghĩa spline | Ví dụ |
|----|----------------------|----------|----------------|-------|
| **T.0** | `is_note_duration(cp)` | NOTE/DURATION | wavelength (trường độ = bước sóng) | 𝅝 𝅗𝅥 𝅘𝅥 𝅘𝅥𝅮 |
| **T.1** | `is_pitch_scale(cp)` | PITCH/SCALE | frequency (cao độ = tần số) | 𝄞 ♯ ♭ ♮ |
| **T.2** | `is_dynamics(cp)` | DYNAMICS/ARTICULATION | amplitude (cường độ = biên độ) | 𝆒 𝆓 |
| **T.3** | `is_neume(cp)` | NEUME/MELODIC | melodic contour (đường cong giai điệu) | neume shapes |
| **T.4** | `is_hexagram(cp)` | HEXAGRAM STATE | discrete phase (64 trạng thái = 6-bit) | ䷀ ䷁ ䷂ |
| **T.5** | `is_modifier(cp)` | MODIFIER/ORNAMENT | modulation (biến điệu: trill, vibrato) | ornaments |

```
Ánh xạ classifier → T value (2 bit):
  T = 0: Static      ← hexagram/tetragram (trạng thái tĩnh)
  T = 1: Slow        ← whole note, fermata, rest (chậm)
  T = 2: Medium      ← quarter note, neume (vừa)
  T = 3: Fast        ← eighth, sixteenth, dynamics, modifier (nhanh)
```

### Spline interpretation:

```
Mỗi ký tự T là 1 knot trên spline:

  f(t) = Σ amplitude(cp) × pitch(cp) × duration(cp) × modulation(cp)

  Trong đó:
    duration(cp)   = is_note_duration → wavelength λ
    pitch(cp)      = is_pitch_scale → frequency ν
    amplitude(cp)  = is_dynamics → loudness dB
    modulation(cp) = is_modifier → vibrato/trill envelope

  Hexagram/Tetragram → discrete state machine:
    state(n) = hexagram_id ∈ [0..63]  (6-bit: 6 hào × âm/dương)
    state(n) = tetragram_id ∈ [0..80] (4-trit: 4 hào × 3 giá trị)
```

---

## Tổng kết: 42 công thức

```
                    ┌─────────────────────────────────────────┐
                    │  F₀: encode_codepoint(cp)               │
                    │  = [f_S, f_R, f_V, f_A, f_T]            │
                    └─────┬───┬───┬───┬───┬───────────────────┘
                          │   │   │   │   │
     ┌────────────────────┘   │   │   │   └─────────────────────┐
     ▼                        ▼   │   ▼                         ▼
  ┌──────────┐         ┌────────┐ │ ┌────────┐           ┌──────────┐
  │ f_S (1)  │         │ f_R(1) │ │ │ f_A(1) │           │ f_T (1)  │
  │ 10 cls   │         │ 10 cls │ │ │ 5 q    │           │ 6 cls    │
  │ = 11     │         │ = 11   │ │ │ = 6    │           │ = 7      │
  └──────────┘         └────────┘ │ └────────┘           └──────────┘
                                  ▼
                           ┌──────────┐
                           │ f_V (1)  │
                           │ 5 q      │
                           │ = 6      │
                           └──────────┘

  Tầng 1:  1 (master)
  Tầng 2:  5 (dimension encoders)
  Tầng 3: 36 (10 + 10 + 5 + 5 + 6 classifiers)
  ─────────────────────
  TỔNG:   42 công thức
```

### Phân loại theo bản chất toán học:

| Loại công thức | Số lượng | Mô tả |
|---------------|----------|-------|
| **Keyword matcher** | 26 | is_X(cp): match char_name vs keyword set → bool |
| **Score lookup** | 2 | raw_V(cp), raw_A(cp): tra NRC-VAD hoặc emoji subgroup → float |
| **Linear quantizer** | 2 | quantize_V(raw), quantize_A(raw): float → int (3 bit) |
| **Subgroup mapper** | 10 | emoji subgroup → V/A default score |
| **Compositor** | 1 | encode_codepoint: gom 5 chiều → 2 bytes |
| **Spline assembler** | 1 | f(t): gom duration × pitch × amplitude × modulation |
| ─── | ─── | ─── |
| **TỔNG** | **42** | |
