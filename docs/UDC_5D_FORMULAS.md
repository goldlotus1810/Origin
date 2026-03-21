# UDC 5D Formulas — 9,584 Codepoints × [S, R, V, A, T]

> **Mục đích:** Gán công thức cho mỗi dimension của 9,584 UDC codepoints.
> Dùng UDC + công thức để đọc emoji/text → tạo node mới → hiểu alias UTF-32.
>
> **Quy tắc:** Không sửa unicode JSON. File này là layer công thức riêng.

---

## Tổng quan

```
P_weight [S][R][V][A][T] = 2 bytes = tọa độ 5D
Packed:  [S:4][R:4][V:3][A:3][T:2] = 16 bits

         ○{9,584 codepoints × 2B = ~19 KB L0 anchors}
    ─────────────|───────────────
   |      |      |      |       |
   S      R      V      A       T
  4bit   4bit   3bit   3bit    2bit
  0-15   0-15   0-7    0-7     0-3
```

**4 nhóm nguồn:**

| Nhóm     | Blocks | Chars | Dominant | Integral Kernel                    |
|----------|--------|-------|----------|------------------------------------|
| SDF      | 13     | 1,904 | S        | ∫ₛ[Shape → SDF_Primitive]          |
| MATH     | 18     | 3,088 | R        | ∫ₛ[Relation → Logic_Channel]       |
| EMOTICON | 15     | 3,568 | VA       | ∫ₛ[Valence+Arousal → Emotion_Space]|
| MUSICAL  | 7      | 1,024 | T        | ∫ₛ[Time → Temporal_Pattern]        |
| **Tổng** | **53** |**9,584**|        |                                    |

**Bootstrap 3 tầng:**

```
char  = f'(x)           — nguyên tử (Unicode codepoint)
sub   = ∫ₛ chars dx     — tích phân các char → sub-group
block = ∫ₛ subs dx      — tích phân các sub → block
P     = ∫ₛ blocks dx    — tích phân các block → tọa độ 5D
```

---

## S — Shape (4 bits, 16+2 SDF Primitives)

### Tổng quan

- **13 blocks SDF**, 1,904 ký tự
- **18 SDF primitives** = 18 sub-categories
- Mỗi primitive có **signed distance function f(P)** và **gradient ∇f**
- Dominant axis: chars trong SDF blocks có S là giá trị quyết định

### 18 Sub-categories (SDF Primitives)

Mỗi primitive có 1 ký tự đại diện (canonical char) và 1 công thức SDF:

```
┌────┬──────────────┬────────┬───┬─────────────────────────────┬──────────────────────┐
│ ID │ Tên          │ Hex    │ ℂ │ f(P) — Signed Distance      │ ∇f — Gradient        │
├────┼──────────────┼────────┼───┼─────────────────────────────┼──────────────────────┤
│  0 │ SPHERE       │ 0x25CF │ ● │ |P| − r                    │ P / |P|              │
│  1 │ BOX          │ 0x25A0 │ ■ │ ‖max(|P|−b, 0)‖            │ sign(P)·step(|P|>b)  │
│  2 │ CAPSULE      │ 0x25AC │ ▬ │ |P−clamp(y,0,h)ĵ| − r     │ norm(P − closest)    │
│  3 │ PLANE        │ 0x25BD │ ▽ │ P.y − h                    │ (0, 1, 0)            │
│  4 │ TORUS        │ 0x25CB │ ○ │ |(|P.xz|−R, P.y)| − r     │ chain rule            │
│  5 │ ELLIPSOID    │ 0x2B2E │ ⬮ │ |P/r| − 1                  │ P/r² / |P/r|         │
│  6 │ CONE         │ 0x25B2 │ ▲ │ dot blend                   │ slope normal          │
│  7 │ CYLINDER     │ 0x25AD │ ▭ │ max(|P.xz|−r, |P.y|−h)    │ radial/cap            │
│  8 │ OCTAHEDRON   │ 0x25C6 │ ◆ │ |x|+|y|+|z| − s            │ sign(P)/√3            │
│  9 │ PYRAMID      │ 0x25B3 │ △ │ pyramid(P, h)               │ slope analytical      │
│ 10 │ HEX_PRISM    │ 0x2B21 │ ⬡ │ max(hex−r, |y|−h)          │ radial hex/cap        │
│ 11 │ PRISM        │ 0x25B1 │ ▱ │ max(|xz|−r, |y|−h)         │ radial/cap            │
│ 12 │ ROUND_BOX    │ 0x25A2 │ ▢ │ BOX(P,b) − rounding         │ smooth corner         │
│ 13 │ LINK         │ 0x221E │ ∞ │ torus compound               │ chain rule            │
│ 14 │ REVOLVE      │ 0x21BB │ ↻ │ revolve_Y(profile)           │ radial                │
│ 15 │ EXTRUDE      │ 0x21E7 │ ⇧ │ extrude_Z(profile)           │ radial                │
│ 16 │ CUT_SPHERE   │ 0x25D0 │ ◐ │ max(|P|−r, P.y−h)          │ norm(P) / (0,1,0)    │
│ 17 │ DEATH_STAR   │ 0x2606 │ ☆ │ opSubtract(sphere, sphere)  │ ±norm(P)              │
└────┴──────────────┴────────┴───┴─────────────────────────────┴──────────────────────┘
```

**Tất cả ∇f là ANALYTICAL (không cần numerical differentiation).**
**∇f → normal → lighting → color. 0 bytes overhead.**

### 13 Blocks SDF

```
┌────┬──────────────────────────────────┬──────────────┬───────┬────────────────────┐
│  # │ Block                            │ Range        │ Chars │ P_default {S,R,V,A,T}│
├────┼──────────────────────────────────┼──────────────┼───────┼────────────────────┤
│  1 │ Arrows                           │ 2190..21FF   │  112  │ {1, 7, 128, 80, 3} │
│  2 │ Miscellaneous Technical          │ 2300..23FF   │  256  │ {6, 6, 112, 80, 0} │
│  3 │ Box Drawing                      │ 2500..257F   │  128  │ {2, 6, 128, 48, 0} │
│  4 │ Block Elements                   │ 2580..259F   │   32  │ {2, 1, 128, 48, 0} │
│  5 │ Geometric Shapes                 │ 25A0..25FF   │   96  │ {0, 0, 128, 64, 0} │
│  6 │ Dingbats                         │ 2700..27BF   │  192  │ {5, 4, 144, 96, 0} │
│  7 │ Supplemental Arrows-A            │ 27F0..27FF   │   16  │ {1, 7, 128, 80, 3} │
│  8 │ Braille Patterns                 │ 2800..28FF   │  256  │ {4, 3, 128, 32, 0} │
│  9 │ Supplemental Arrows-B            │ 2900..297F   │  128  │ {1, 7, 128, 80, 3} │
│ 10 │ Misc Symbols and Arrows          │ 2B00..2BFF   │  256  │ {3, 4, 128, 96, 0} │
│ 11 │ Ornamental Dingbats              │ 1F650..1F67F │   48  │ {5, 4, 144, 80, 0} │
│ 12 │ Geometric Shapes Extended        │ 1F780..1F7FF │  128  │ {0, 0, 128, 64, 0} │
│ 13 │ Supplemental Arrows-C            │ 1F800..1F8FF │  256  │ {1, 7, 128, 80, 3} │
└────┴──────────────────────────────────┴──────────────┴───────┴────────────────────┘
```

### Phân bố S value trong Geometric Shapes (25A0..25FF)

```
S=0  (SPHERE)    │ 34 chars │ Circles, arcs, bullets         │ ●○◌◎◉◐◑◒...
S=1  (BOX)       │ 28 chars │ Squares, rectangles            │ ■□▢▣▤▥▦▧...
S=2  (CAPSULE)   │  1 char  │ Lozenge                        │ ◊
S=6  (CONE)      │ 26 chars │ Triangles (all directions)     │ ▲△▴▵▶▷▸▹...
S=8  (OCTAHEDRON)│  3 chars │ Diamonds                       │ ◆◇◈
S=14 (REVOLVE)   │  4 chars │ Pointers (directional)         │ ►▻◄◅
```

### Công thức tích phân cho S

```
Với mỗi char trong SDF block:
  S_value = P_weight[0]                    — lấy từ udc.json
  formula = SDF_PRIMITIVE[S_value].f(P)    — tra bảng 18 primitives ở trên

Với sub (nhóm chars cùng S_value trong 1 block):
  sub_S = S_value chung                    — đại diện
  sub_formula = SDF_PRIMITIVE[sub_S].f(P)

Với block:
  block_S = p_default.S                    — giá trị mặc định của block
  block_formula = SDF_PRIMITIVE[block_S].f(P)

Recombine (compose 2 shapes):
  S_composed = Union(A.S, B.S)             — unified shape (silhouette merge)
```

### Cách dùng S cho TEXT blocks

```
Khi gặp ký tự TEXT (VD: chữ 'A' Latin, S=1):
  S=1 → BOX → f(P) = ‖max(|P|−b, 0)‖
  Nghĩa: chữ cái Latin có shape vuông (bounding box)

Khi gặp ký tự Arabic (S=3):
  S=3 → PLANE → f(P) = P.y − h
  Nghĩa: chữ Arabic có shape phẳng (baseline writing)

→ S cho TEXT block = shape hình học của hệ chữ viết đó
→ Dùng để tạo node mới khi alias UTF-32 vào UDC
```

---

## R — Relation (4 bits, 16 Logic Channels)

### Tổng quan

- **18 blocks MATH**, 3,088 ký tự
- **16 relation types** = 16 sub-categories
- Mỗi relation có **ký tự đại diện** và **công thức logic**
- **75 kênh × 31 mẫu = 2,325 kiểu quan hệ** (implicit, 0 bytes — Silk ngang)

### 16 Sub-categories (Relation Primitives)

```
┌────┬──────────────┬────────┬───┬──────────────────────────────────────────────────┐
│ ID │ Tên          │ Hex    │ ℂ │ Công thức / Ý nghĩa                             │
├────┼──────────────┼────────┼───┼──────────────────────────────────────────────────┤
│  0 │ IDENTITY     │ 0x003D │ = │ A ≡ A — tự đồng nhất                            │
│  1 │ MEMBER       │ 0x2208 │ ∈ │ a ∈ B — phần tử thuộc tập                       │
│  2 │ SUBSET       │ 0x2282 │ ⊂ │ A ⊂ B — tập con                                │
│  3 │ EQUALITY     │ 0x2261 │ ≡ │ A ≡ B — tương đương logic                       │
│  4 │ ORDER        │ 0x2264 │ ≤ │ A ≤ B — thứ tự (partial/total)                  │
│  5 │ ARITHMETIC   │ 0x2202 │ ∂ │ ∂f/∂x — vi phân, phép toán số học               │
│  6 │ LOGICAL      │ 0x2200 │ ∀ │ ∀x, ∃x — lượng từ logic                        │
│  7 │ SET_OP       │ 0x2229 │ ∩ │ A ∩ B, A ∪ B — phép toán tập hợp               │
│  8 │ COMPOSE      │ 0x2218 │ ∘ │ f ∘ g = f(g(x)) — hợp thành                    │
│  9 │ CAUSES       │ 0x2192 │ → │ A → B — nhân quả, suy ra                        │
│ 10 │ APPROXIMATE  │ 0x2248 │ ≈ │ A ≈ B — xấp xỉ, gần bằng                       │
│ 11 │ ORTHOGONAL   │ 0x22A5 │ ⊥ │ A ⊥ B — trực giao, độc lập                     │
│ 12 │ AGGREGATE    │ 0x2211 │ Σ │ Σ, ∏ — tổng hợp, tích lũy                      │
│ 13 │ DIRECTIONAL  │ 0x2190 │ ← │ A ← B — hướng, derived from                    │
│ 14 │ BRACKET      │ 0x27E8 │ ⟨ │ ⟨A, B⟩ — bao đóng, pairing                     │
│ 15 │ INVERSE      │ 0x223D │ ∽ │ A⁻¹ — nghịch đảo                               │
└────┴──────────────┴────────┴───┴──────────────────────────────────────────────────┘
```

### 18 Blocks MATH

```
┌────┬──────────────────────────────────────┬──────────────────┬───────┬────────────────────┐
│  # │ Block                                │ Range            │ Chars │ P_default {S,R,V,A,T}│
├────┼──────────────────────────────────────┼──────────────────┼───────┼────────────────────┤
│  1 │ Superscripts and Subscripts          │ 2070..209F       │   48  │ {6, 0, 128, 64, 2} │
│  2 │ Letterlike Symbols                   │ 2100..214F       │   80  │ {6, 2, 128, 64, 2} │
│  3 │ Number Forms                         │ 2150..218F       │   64  │ {2, 0, 128, 64, 2} │
│  4 │ Mathematical Operators               │ 2200..22FF       │  256  │ {6, 4, 128, 80, 2} │
│  5 │ Misc Mathematical Symbols-A          │ 27C0..27EF       │   48  │ {6, 3, 128, 64, 2} │
│  6 │ Misc Mathematical Symbols-B          │ 2980..29FF       │  128  │ {6, 4, 128, 80, 2} │
│  7 │ Supplemental Math Operators          │ 2A00..2AFF       │  256  │ {6, 4, 128, 80, 2} │
│  8 │ Math Alphanumeric Symbols            │ 1D400..1D7FF     │ 1024  │ {2, 2, 128, 64, 2} │
│  9 │ Ancient Greek Numbers                │ 10140..1018F     │   80  │ {2, 0, 128, 48, 1} │
│ 10 │ Common Indic Number Forms            │ A830..A83F       │   16  │ {2, 0, 128, 48, 1} │
│ 11 │ Counting Rod Numerals                │ 1D360..1D37F     │   32  │ {2, 0, 128, 48, 1} │
│ 12 │ Cuneiform Numbers & Punctuation      │ 12400..1247F     │  128  │ {2, 0, 128, 48, 1} │
│ 13 │ Archaic Cuneiform Numerals           │ 12550..1268F     │  320  │ {2, 0, 128, 48, 1} │
│ 14 │ Indic Siyaq Numbers                  │ 1EC70..1ECBF     │   80  │ {2, 0, 128, 48, 1} │
│ 15 │ Ottoman Siyaq Numbers                │ 1ED00..1ED4F     │   80  │ {2, 0, 128, 48, 1} │
│ 16 │ Arabic Math Alphabetic Symbols       │ 1EE00..1EEFF     │  256  │ {2, 2, 128, 64, 2} │
│ 17 │ Misc Symbols Supplement              │ 1CEC0..1CEFF     │   64  │ {6, 3, 128, 64, 2} │
│ 18 │ Misc Symbols & Arrows Extended       │ 1DB00..1DBFF     │  256  │ {1, 7, 128, 80, 2} │
└────┴──────────────────────────────────────┴──────────────────┴───────┴────────────────────┘
```

### Phân bố R value trong Mathematical Operators (2200..22FF)

```
R= 1 (MEMBER)     │ 24 chars │ ∈ ∉ ∋ ∌ ⋲ ⋳ ⋴ ⋵...
R= 2 (SUBSET)     │ 16 chars │ ⊂ ⊃ ⊆ ⊇ ⊈ ⊉ ⊊ ⊋...
R= 3 (EQUALITY)   │ 50 chars │ ≅ ≆ ≇ ≊ ≋ ≌...
R= 4 (ORDER)      │ 28 chars │ ≤ ≥ ≦ ≧ ≨ ≩ ≪ ≫...
R= 5 (ARITHMETIC) │ 88 chars │ ∂ ∅ ∆ ∇ − ± ∓ ∗ ∘...    ← lớn nhất
R= 6 (LOGICAL)    │ 16 chars │ ∀ ∃ ∄ ∧ ∨...
R= 7 (SET_OP)     │  8 chars │ ∁ ∩ ∪ ⊎...
R= 8 (COMPOSE)    │  5 chars │ ∘ ⊕ ⊗...
R=10 (APPROXIMATE)│  3 chars │ ∼ ≈ ≃...
R=12 (AGGREGATE)  │ 15 chars │ ∏ ∐ ∑ ⊓ ⊔...
R=15 (INVERSE)    │  1 char  │ ∽
```

### Công thức tích phân cho R

```
Với mỗi char trong MATH block:
  R_value = P_weight[1]                       — lấy từ udc.json
  formula = RELATION_TYPE[R_value]             — tra bảng 16 relations ở trên

Với sub (nhóm chars cùng R_value trong 1 block):
  sub_R = R_value chung                        — đại diện
  sub_formula = RELATION_TYPE[sub_R]

Với block:
  block_R = p_default.R                        — giá trị mặc định
  block_formula = RELATION_TYPE[block_R]

Recombine (compose 2 relations):
  R_composed = Compose(A.R, B.R)               — relations compose (transitive logic)
  VD: Member ∘ Subset = Member (a ∈ B ⊂ C → a ∈ C)
```

### Cách dùng R cho TEXT blocks

```
Hầu hết TEXT blocks có R=0 (IDENTITY):
  Mỗi chữ cái tự đồng nhất với chính nó: 'A' ≡ 'A'
  Nghĩa: text characters không tự mang quan hệ logic

Ngoại lệ — ký tự đặc biệt trong TEXT blocks:
  Dấu câu (. , ; :) có thể mang R=8 (COMPOSE) hoặc R=4 (ORDER)
  Vì chúng tạo cấu trúc trong câu

→ R cho TEXT = "ký tự này TỰ NÓ thể hiện quan hệ gì?"
→ Đa số text: IDENTITY. Quan hệ thật sinh ra từ Silk edges.
```

---

## V — Valence (3 bits, quantized [0, 7])

### Tổng quan

- **15 blocks EMOTICON**, 3,568 ký tự (chia sẻ với A)
- Valence = **cực tính cảm xúc**: âm ← 0.0 ... 0.5 ... 1.0 → dương
- Raw: float [0.0, 1.0] → quantized 3 bits [0, 7]
- Nguồn: **NRC-VAD-Lexicon v2.1** + **Emoji Discrete Emotions Database**

### Bảng quantize V

```
Quantized │ Raw Range   │ Ý nghĩa              │ Ví dụ đại diện
──────────┼─────────────┼───────────────────────┼──────────────────────
  0       │ 0.000-0.124 │ Cực âm (despair)     │ ☠ 0x2620  V=0.135
  1       │ 0.125-0.249 │ Rất âm (grief)       │ 😭 0x1F62D V=0.237
  2       │ 0.250-0.374 │ Âm (sad)             │ 😢 0x1F622 V=0.354
  3       │ 0.375-0.499 │ Hơi âm (uneasy)     │ 😟 0x1F61F V=0.456
  4       │ 0.500-0.624 │ Trung tính           │ 😐 0x1F610 V=0.523
  5       │ 0.625-0.749 │ Hơi dương (pleased)  │ 😊 0x1F60A V=0.771
  6       │ 0.750-0.874 │ Dương (happy)        │ 😍 0x1F60D V=0.788
  7       │ 0.875-1.000 │ Cực dương (ecstatic) │ 🎆 (extreme joy)
```

### 15 Blocks EMOTICON

```
┌────┬──────────────────────────────────────┬──────────────────┬───────┬────────────────────┐
│  # │ Block                                │ Range            │ Chars │ P_default {S,R,V,A,T}│
├────┼──────────────────────────────────────┼──────────────────┼───────┼────────────────────┤
│  1 │ Enclosed Alphanumerics               │ 2460..24FF       │  160  │ {0, 0, 144, 96, 2} │
│  2 │ Miscellaneous Symbols                │ 2600..26FF       │  256  │ {0, 0, 128,128, 2} │
│  3 │ Mahjong Tiles                        │ 1F000..1F02F     │   48  │ {2, 4, 144,128, 2} │
│  4 │ Domino Tiles                         │ 1F030..1F09F     │  112  │ {2, 4, 144,128, 2} │
│  5 │ Playing Cards                        │ 1F0A0..1F0FF     │   96  │ {2, 4, 144,128, 2} │
│  6 │ Enclosed Alphanumeric Supplement     │ 1F100..1F1FF     │  256  │ {0, 0, 144, 96, 2} │
│  7 │ Enclosed Ideographic Supplement      │ 1F200..1F2FF     │  256  │ {0, 0, 144, 96, 2} │
│  8 │ Misc Symbols and Pictographs         │ 1F300..1F5FF     │  768  │ {0, 0, 144,128, 2} │
│  9 │ Emoticons                            │ 1F600..1F64F     │   80  │ {0, 0, 192,160, 3} │
│ 10 │ Transport and Map Symbols            │ 1F680..1F6FF     │  128  │ {3, 4, 128,160, 3} │
│ 11 │ Alchemical Symbols                   │ 1F700..1F77F     │  128  │ {0, 5, 112, 96, 1} │
│ 12 │ Supplemental Symbols & Pictographs   │ 1F900..1F9FF     │  256  │ {0, 0, 144,128, 2} │
│ 13 │ Chess Symbols                        │ 1FA00..1FA6F     │  112  │ {3, 3, 128,128, 2} │
│ 14 │ Symbols & Pictographs Extended-A     │ 1FA70..1FAFF     │  144  │ {0, 0, 144,128, 2} │
│ 15 │ Symbols for Legacy Computing         │ 1FB00..1FBFF     │  256  │ {2, 1, 128, 96, 0} │
└────┴──────────────────────────────────────┴──────────────────┴───────┴────────────────────┘
```

### Emoji Subgroups trong Emoticons (1F600..1F64F)

```
Subgroup               │ Chars │ V range         │ Node đại diện
───────────────────────┼───────┼─────────────────┼────────────────
face-smiling           │  12   │ 0.570 - 0.771   │ 😊 0x1F60A V=0.771
face-affection         │   5   │ 0.688 - 0.788   │ 😍 0x1F60D V=0.788
face-tongue            │   4   │ 0.734 - 0.789   │ 😜 0x1F61C V=0.789
face-neutral-skeptical │   7   │ 0.523 - 0.708   │ 😐 0x1F610 V=0.523
face-sleepy            │   4   │ 0.507 - 0.727   │ 😴 0x1F634 V=0.507
face-concerned         │  21   │ 0.354 - 0.771   │ 😢 0x1F622 V=0.354
face-negative          │   4   │ 0.468 - 0.763   │ 😡 0x1F621 V=0.468
face-glasses           │   1   │ 0.726           │ 😎 0x1F60E V=0.726
face-unwell            │   2   │ 0.499 - 0.714   │ 😷 0x1F637 V=0.499
cat-face               │   9   │ 0.668 - 0.798   │ 😻 0x1F63B V=0.798
monkey-face            │   3   │ 0.567 - 0.724   │ 🙈 0x1F648 V=0.567
hands                  │   2   │ 0.567 - 0.771   │ 🙏 0x1F64F V=0.567
person-gesture         │   6   │ 0.374 - 0.674   │ 🙅 0x1F645 V=0.374
```

### Công thức Valence

```
① Đọc V từ UDC:
  V_raw = udc.json → character → physics_logic.P_weight[2]
  V_quantized = floor(V_raw × 7)     — quantize [0.0,1.0] → [0,7]

② Amplify (KHÔNG trung bình — quy tắc bất biến):
  amplify(Va, Vb, w):
    base  = (Va + Vb) / 2
    boost = |Va − base| × w × 0.5
    V_out = base + sign(Va + Vb − 1.0) × boost

  VD: compose("love" V=0.9, "intense" V=0.95, w=0.8)
    base  = 0.925
    boost = 0.025 × 0.8 × 0.5 = 0.01
    V_out = 0.925 + 0.01 = 0.935  → amplified ✓

  VD: compose("sad" V=0.3, "job_loss" V=0.4, w=0.9)
    base  = 0.35
    boost = 0.05 × 0.9 × 0.5 = 0.0225
    V_out = 0.35 − 0.0225 = 0.3275  → heavier (correct) ✓

③ Tích phân thời gian (encoder):
  ΔV = ∫ affect(token) dt          — cumulative, NOT snapshot

④ Emotion Curve:
  f_conv(t) = V(t) + 0.5 × V'(t) + 0.25 × V''(t)
  f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
```

### Cách dùng V cho TEXT blocks

```
TEXT characters có V từ NRC-VAD Lexicon:
  'A' (0x0041): V=0.547 → trung tính nhẹ dương
  Ký tự control: V=0.729 → mặc định từ block

→ Khi tạo node mới cho alias UTF-32:
  V_node = lookup V từ udc.json cho codepoint gần nhất
  Nếu không có → V_node = p_default.V của block chứa nó
```

---

## A — Arousal (3 bits, quantized [0, 7])

### Tổng quan

- **Cùng 15 blocks EMOTICON** với V (chia sẻ)
- Arousal = **cường độ kích thích**: tĩnh ← 0.0 ... 0.5 ... 1.0 → kích động
- Raw: float [0.0, 1.0] → quantized 3 bits [0, 7]
- Nguồn: **NRC-VAD-Lexicon v2.1** + **Emoji Discrete Emotions Database**

### Bảng quantize A

```
Quantized │ Raw Range   │ Ý nghĩa              │ Ví dụ đại diện
──────────┼─────────────┼───────────────────────┼──────────────────────
  0       │ 0.000-0.124 │ Rất tĩnh (comatose)  │ 💤 (deep sleep)
  1       │ 0.125-0.249 │ Tĩnh (calm)          │ 😌 0x1F60C A=0.344
  2       │ 0.250-0.374 │ Nhẹ nhàng (relaxed)  │ 😊 0x1F60A A=0.363
  3       │ 0.375-0.499 │ Trung bình           │ 😀 0x1F600 A=0.387
  4       │ 0.500-0.624 │ Hơi kích thích       │ 😈 0x1F608 A=0.558
  5       │ 0.625-0.749 │ Kích thích           │ 😨 0x1F628 A=0.610
  6       │ 0.750-0.874 │ Mạnh (excited)       │ 🔥 (fire) A=0.80+
  7       │ 0.875-1.000 │ Cực kích (crisis)    │ ⚡ (extreme alert)
```

### Phân bố A trong Emoticons block

```
A < 0.35  (rất tĩnh)  │ 😴😌😇     │ sleepy, relieved, halo
0.35-0.45 (trung bình) │ 😀😁😂😃😄 │ smiling faces  ← đa số
0.45-0.55 (hơi kích)   │ 😈😜😝     │ devil, tongue
0.55+     (kích thích)  │ 😨😰😱     │ fearful, anxious, scream
```

### Công thức Arousal

```
① Đọc A từ UDC:
  A_raw = udc.json → character → physics_logic.P_weight[3]
  A_quantized = floor(A_raw × 7)

② Recombine:
  A_composed = max(A_a, A_b)         — take stronger intensity
  KHÔNG trung bình! Max vì: nếu 1 stimulus mạnh → toàn bộ hệ thống kích hoạt

③ Crisis detection (SecurityGate):
  V < 0.1 AND A > 0.8 → potential crisis (despair + high arousal)
  → Gate trả về ngay, không qua pipeline

④ Co-activation (Hebbian):
  emotion_factor = (|A.V − 0.5| + |B.V − 0.5|) / 2 × max(A.A, B.A)
  → Arousal cao = kết nối Silk mạnh hơn
```

### Cách dùng A cho TEXT blocks

```
TEXT characters có A từ NRC-VAD:
  'A' (0x0041): A=0.500 → trung bình
  '!' (0x0021): A cao hơn → exclamation = kích thích
  '.' (0x002E): A thấp → period = bình tĩnh

→ A cho TEXT = "ký tự này gây kích thích bao nhiêu?"
→ Dấu chấm than > dấu chấm > dấu phẩy
```

---

## T — Time (2 bits, 4 Temporal Modes)

### Tổng quan

- **7 blocks MUSICAL**, 1,024 ký tự
- Time = **mô hình thời gian/nhịp** — spline, sóng, dao động
- 2 bits → 4 giá trị: mã hóa loại temporal pattern
- Liên hệ vật lý: sóng âm, bước sóng, dao động, chu kỳ

### 4 Sub-categories (Time Modes)

```
┌────┬─────────────┬────────────────┬─────────────────────────────────────────────────┐
│ ID │ Tên         │ Nhạc tương ứng │ Công thức Spline / Dao động                     │
├────┼─────────────┼────────────────┼─────────────────────────────────────────────────┤
│  0 │ TIMELESS    │ Fermata (𝄐)    │ f(t) = c                                        │
│    │             │                │ Hằng số — không biến thiên theo thời gian        │
│    │             │                │ ∂f/∂t = 0 (gradient zero)                        │
├────┼─────────────┼────────────────┼─────────────────────────────────────────────────┤
│  1 │ SEQUENTIAL  │ Largo (𝅝)      │ f(t) = a₀ + a₁t                                 │
│    │             │                │ Linear spline — tiến trình đều, 1 hướng          │
│    │             │                │ Bước sóng: λ → ∞ (không lặp lại)                │
├────┼─────────────┼────────────────┼─────────────────────────────────────────────────┤
│  2 │ CYCLICAL    │ Andante (♩)    │ f(t) = A·sin(2πt/T + φ)                         │
│    │             │                │ Sóng sin — chu kỳ lặp lại                        │
│    │             │                │ T = chu kỳ, A = biên độ, φ = pha                 │
│    │             │                │ Bước sóng: λ = v × T                             │
├────┼─────────────┼────────────────┼─────────────────────────────────────────────────┤
│  3 │ RHYTHMIC    │ Presto (♬)     │ f(t) = Σₖ Aₖ·sin(2πkf₀t + φₖ)                 │
│    │             │                │ Fourier series — tổ hợp sóng phức                │
│    │             │                │ f₀ = tần số cơ bản, k = bội số                   │
│    │             │                │ Harmonic overtones → nhịp phức tạp               │
└────┴─────────────┴────────────────┴─────────────────────────────────────────────────┘
```

### 7 Blocks MUSICAL

```
┌────┬────────────────────────────┬──────────────────┬───────┬────────────────────┐
│  # │ Block                      │ Range            │ Chars │ P_default {S,R,V,A,T}│
├────┼────────────────────────────┼──────────────────┼───────┼────────────────────┤
│  1 │ Yijing Hexagram Symbols    │ 4DC0..4DFF       │   64  │ {2, 2, 128, 48, 1} │
│  2 │ Znamenny Musical Notation  │ 1CF00..1CFCF     │  208  │ {1, 4, 128, 80, 2} │
│  3 │ Byzantine Musical Symbols  │ 1D000..1D0FF     │  256  │ {1, 4, 128, 80, 2} │
│  4 │ Musical Symbols            │ 1D100..1D1FF     │  256  │ {3, 4, 128, 96, 3} │
│  5 │ Ancient Greek Musical Not. │ 1D200..1D24F     │   80  │ {1, 2, 128, 80, 2} │
│  6 │ Musical Symbols Supplement │ 1D250..1D28F     │   64  │ {3, 4, 128, 96, 3} │
│  7 │ Tai Xuan Jing Symbols      │ 1D300..1D35F     │   96  │ {2, 2, 128, 48, 1} │
└────┴────────────────────────────┴──────────────────┴───────┴────────────────────┘
```

### Phân bố T value

```
T=1 (SEQUENTIAL) │ Yijing + Tai Xuan Jing │ 160 chars │ Tuyến tính, bói toán
T=2 (CYCLICAL)   │ Znamenny + Byzantine    │ 464 chars │ Phụng vụ, chu kỳ lễ
                  │ + Ancient Greek Musical │           │
T=3 (RHYTHMIC)   │ Musical Symbols (2 blk) │ 320 chars │ Score, nhịp phức ← core
T=0 (TIMELESS)   │ (không có trong MUSICAL)│   0 chars │ (TEXT blocks dùng T=0)
```

### Công thức Spline cho T

```
① Đọc T từ UDC:
  T_value = P_weight[4]                       — lấy từ udc.json
  formula = TIME_MODE[T_value]                 — tra bảng 4 modes ở trên

② Recombine:
  T_composed = dominant(A.T, B.T)              — temporal dominance
  Rule: T cao hơn thắng (RHYTHMIC > CYCLICAL > SEQUENTIAL > TIMELESS)

③ Áp dụng vào Emotion Curve:
  Tone selection dùng derivatives:
    V' < −0.15              → Supportive
    V'' < −0.25             → Pause
    V' > +0.15              → Reinforcing
    V'' > +0.25 AND V > 0.5 → Celebratory

  Gradual change: ΔV_max = 0.40/step
```

### Cách dùng T cho TEXT blocks

```
Hầu hết TEXT blocks: T=0 (TIMELESS)
  Chữ cái, số, dấu câu = không mang thời gian tự thân
  'A' → TIMELESS, '1' → TIMELESS

25 ký tự TEXT đặc biệt có T=1 (SEQUENTIAL):
  Ký tự ordering, numbering, sequencing

→ T cho TEXT = "ký tự này có tính thời gian không?"
→ Đa số text: TIMELESS. Thời gian sinh ra từ ngữ cảnh câu.
```

---

## TEXT → Formula Mapping (Alias UTF-32 Flow)

### Vấn đề

63 blocks TEXT trong UTF-32 (Latin, Greek, Arabic, CJK...) có ~100,000+ ký tự.
9,584 UDC codepoints chỉ bao phủ 4 nhóm chính (SDF, MATH, EMOTICON, MUSICAL).

**Câu hỏi: Khi gặp TEXT char, làm sao tạo node mới?**

### Giải pháp: UDC + Công thức → Node mới

```
INPUT: ký tự UTF-32 bất kỳ (VD: 'Ω' U+03A9, Greek)

BƯỚC 1 — Tra alias mapping:
  udc.json → alias_mapping → tìm alias cho 'Ω'
  Nếu có alias → lấy UDC codepoint tương ứng → XONG

BƯỚC 2 — Nếu không có alias, dùng P_weight mặc định của block:
  Block "Greek and Coptic" → p_default → {S, R, V, A, T}
  Ω: S=0, R=0, V_raw, A_raw, T=0

BƯỚC 3 — Gán công thức từ bảng:
  S=0 → SPHERE   → f(P) = |P| − r
  R=0 → IDENTITY → A ≡ A
  V   → V_raw từ NRC-VAD (nếu có) hoặc p_default.V
  A   → A_raw từ NRC-VAD (nếu có) hoặc p_default.A
  T=0 → TIMELESS → f(t) = c

BƯỚC 4 — Tạo node mới:
  Molecule = pack(S, R, V, A, T)   — u16 packed
  chain    = encode_codepoint(cp)   — KHÔNG viết tay
  hash     = chain_hash(chain)      — tự sinh

OUTPUT: Node mới với đầy đủ 5D công thức
  → Có thể so sánh, compose, silk với 9,584 UDC nodes
  → Hiểu UTF-32 thông qua hệ tọa độ 5D chung
```

### Ví dụ cụ thể

```
INPUT: "lửa" (text tiếng Việt)

BƯỚC 1: alias_mapping.vi["lửa"] = "1F525" (🔥 FIRE)
BƯỚC 2: skip (đã có alias)
BƯỚC 3: UDC lookup 0x1F525:
  S=0  → SPHERE   → f(P) = |P| − r        (lửa = hình cầu phát sáng)
  R=9  → CAUSES   → A → B                  (lửa GÂY RA cháy/nóng)
  V=0.735          → amplify formula         (cảm xúc dương — ấm áp)
  A=0.820          → max formula             (kích thích cao — nguy hiểm)
  T=3  → RHYTHMIC → Fourier series          (ngọn lửa dao động phức tạp)

OUTPUT: Node 🔥 với 5 công thức:
  node["🔥"].S_formula = |P| − r
  node["🔥"].R_formula = A → B
  node["🔥"].V_formula = amplify(0.735, V_context, w_silk)
  node["🔥"].A_formula = max(0.820, A_context)
  node["🔥"].T_formula = Σₖ Aₖ·sin(2πkf₀t + φₖ)
```

```
INPUT: 'A' (0x0041, Basic Latin)

BƯỚC 1: Không có alias trực tiếp cho 'A'
BƯỚC 2: Block "Basic Latin" → p_default + NRC-VAD
  S=1, R=0, V=0.547, A=0.500, T=0
BƯỚC 3: Gán công thức:
  S=1  → BOX       → f(P) = ‖max(|P|−b, 0)‖   (chữ A = bounding box)
  R=0  → IDENTITY  → A ≡ A                       (chữ A tự đồng nhất)
  V=0.547           → amplify                     (nhẹ dương — neutral+)
  A=0.500           → max                         (trung bình)
  T=0  → TIMELESS  → f(t) = c                    (chữ cái = vĩnh cửu)

OUTPUT: Node 'A' với 5 công thức
```

### Bảng tổng hợp: Dimension → Recombine Rule

```
┌───────────┬──────────────────────────────────┬──────────────────────────┐
│ Dimension │ Recombine Formula                │ Lý do sinh học           │
├───────────┼──────────────────────────────────┼──────────────────────────┤
│ S         │ Union(Aˢ, Bˢ)                   │ unified silhouette       │
│ R         │ Compose(Aᴿ, Bᴿ)                 │ transitive logic         │
│ V         │ amplify(Va, Vb, w_silk)          │ synergistic emotion      │
│ A         │ max(Aᴬ, Bᴬ)                     │ strongest intensity      │
│ T         │ dominant(Aᵀ, Bᵀ)                │ temporal dominance       │
└───────────┴──────────────────────────────────┴──────────────────────────┘

Ngưỡng vàng: φ⁻¹ = (√5−1)/2 ≈ 0.618 — xuyên suốt mọi threshold
```

### Distance metric cho so sánh

```
d(A, B) = √( Σ_{d=1}^{5} (Aᵈₙ − Bᵈₙ)² )

Normalization:
  S: enum_index / 17    (0..17 → [0,1])
  R: enum_index / 15    (0..15 → [0,1])
  V: raw_value           (đã [0,1])
  A: raw_value           (đã [0,1])
  T: enum_index / 3     (0..3  → [0,1])

→ Tất cả 5 chiều trên cùng scale [0.0, 1.0]
→ d(A,B) ∈ [0, √5 ≈ 2.236]
```

---

## Tổng kết

```
9,584 UDC codepoints × [S, R, V, A, T] × [công thức]
= 9,584 × 5 = 47,920 công thức instances

Nhưng chỉ cần:
  18 SDF formulas      (S)
  16 Relation formulas (R)
   1 Amplify formula   (V)
   1 Max formula       (A)
   4 Spline formulas   (T)
  ─────────────────────────
  40 công thức unique tổng cộng

Mỗi char = tổ hợp 5 công thức từ bảng tra → O(1) lookup
Mỗi node mới = pack(S, R, V, A, T) = 2 bytes = vị trí trong không gian 5D
Alias UTF-32 = mapping text → UDC codepoint → 5 công thức → node

        f(P)       logic      amplify    max       spline
         │           │          │         │          │
    ┌────┴────┬──────┴────┬─────┴───┬─────┴────┬─────┴────┐
    │ S:4bit  │  R:4bit   │ V:3bit  │  A:3bit  │  T:2bit  │
    └─────────┴───────────┴─────────┴──────────┴──────────┘
                    = 16 bits = 2 bytes = P_weight
```
