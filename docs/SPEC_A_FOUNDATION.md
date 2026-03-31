# SPEC A — Nền Móng: SDF + P_weight + Encode + Compose + Decode

> **Tài liệu kỹ thuật chi tiết cho Phần A của HomeOS Unified Spec.**
> **Đọc file này = hiểu toàn bộ nền móng toán học.**
> **Tác giả:** Lupin (thiết kế) + Nox (tổng hợp + verify)
> **Ngày hoàn thành:** 2026-03-30
> **Verified against:** BLUEPRINT.md, UDC_formulas.md, UDC_DOC/*, HomeOS_SINH_HOC_v2, udc.json (data)

---

## MỤC LỤC

```
A1. SDF — Signed Distance Field tổng quát
A2. P_weight — 5 chiều, 2 bytes, bit layout
A3. Encode ∫ — 42 formulas, scale-invariant integral
A4. Compose — Non-commutative biological synergy
A5. Decode ∂ — Đạo hàm qua KnowTree
A6. Số liệu thực tế (verified from data)
```

---

## A1. SDF — Mỗi Ký Tự LÀ Một Hàm

### Nguyên lý

Mỗi ký tự Unicode trong 53 UDC blocks **LÀ** một hàm SDF.

SDF = hàm trả về khoảng cách có dấu từ 1 điểm p đến bề mặt.
Hoạt động trong BẤT KỲ không gian nào — không chỉ hình học.

```
f(p) = signed distance from point p to surface

  f(p) < 0    → bên trong     → THỂ TÍCH
  f(p) = 0    → trên bề mặt   → HÌNH DẠNG
  f(p) > 0    → bên ngoài     → KHÔNG GIAN
  ∇f(p)       → pháp tuyến    → ÁNH SÁNG → MÀU SẮC
  ∂f/∂t       → biến thiên    → DAO ĐỘNG → ÂM THANH
  p           → tọa độ        → VỊ TRÍ

1 hàm. 1 điểm. Ra tất cả.
Lưu giá trị hữu hình (hình dạng) VÀ vô hình (bước sóng, cảm xúc, chuyển động).
```

### 4 không gian SDF

```
S blocks (13): SDF trong không gian hình học
  ● = f(p) = |p| - r (sphere)
  ■ = f(p) = max(|p| - b, 0) (box)

R blocks (18): SDF trong không gian logic
  ∈ = membership distance (thuộc hay không thuộc)
  ≡ = equivalence distance

E blocks (15): SDF trong không gian cảm xúc
  😀 = vị trí trong V/A (high valence, moderate arousal)
  😢 = vị trí trong V/A (low valence, moderate arousal)

T blocks (7): SDF trong không gian thời gian
  𝄞 = frequency/duration position
  𝅘𝅥 = temporal phase
```

### 18 SDF Primitives (hình học)

```
#   Tên          f(P)                         ∇f (analytical)
0   SPHERE       |P| − r                      P / |P|
1   BOX          ||max(|P|−b, 0)||            sign(P)·step(|P|>b)
2   CAPSULE      |P−clamp(y,0,h)ĵ| − r       norm(P − closest)
3   PLANE        P.y − h                      (0, 1, 0)
4   TORUS        |(|P.xz|−R, P.y)| − r       chain rule
5   ELLIPSOID    |P/r| − 1                    P/r² / |P/r|
6   CONE         dot blend                    slope normal
7   CYLINDER     max(|P.xz|−r, |P.y|−h)      radial/cap
8   OCTAHEDRON   |x|+|y|+|z| − s             sign(P)/√3
9   PYRAMID      pyramid(P,h)                 slope analytical
10  HEX_PRISM    max(hex−r, |y|−h)            radial hex/cap
11  PRISM        max(|xz|−r, |y|−h)           radial/cap
12  ROUND_BOX    BOX − rounding               smooth corner
13  LINK         torus compound               chain rule
14  REVOLVE      revolve_Y                    radial
15  EXTRUDE      extrude_Z                    radial
16  CUT_SPHERE   max(|P|−r, P.y−h)           norm(P)/(0,1,0)
17  DEATH_STAR   opSubtract                   ±norm(P)

Boolean ops trên SDF:
  Union:     min(f₁, f₂)
  Intersect: max(f₁, f₂)
  Subtract:  max(f₁, −f₂)

Tất cả ∇f ANALYTICAL — không cần numerical differentiation.
Mọi hình phức tạp = compose từ 18 primitives + boolean ops.
```

> Chi tiết sub-variants: `docs/tailieu_nghiencuu/UDC_DOC/UDC_S1_GEOMETRIC_tree.md`

### 53 Unicode Blocks = Bảng tuần hoàn

```
SHAPE (S)      — 13 blocks, 1,904 chars
  S.01  Arrows                 U+2190..21FF    112
  S.02  Box Drawing            U+2500..257F    128
  S.03  Block Elements         U+2580..259F     32
  S.04  Geometric Shapes       U+25A0..25FF     96   ← ● ■ ▲ ○
  S.05  Dingbats               U+2700..27BF    192
  S.06  Supp Arrows-A          U+27F0..27FF     16
  S.07  Supp Arrows-B          U+2900..297F    128
  S.08  Misc Symbols+Arrows    U+2B00..2BFF    256
  S.09  Geometric Shapes Ext   U+1F780..1F7FF  128
  S.10  Supp Arrows-C          U+1F800..1F8FF  256
  S.11  Ornamental Dingbats    U+1F650..1F67F   48
  S.12  Misc Technical         U+2300..23FF    256
  S.13  Braille Patterns       U+2800..28FF    256

MATH (R)       — 18 blocks, 3,216 chars
  M.01..M.18   Mathematical Operators, Letterlike, Number Forms,
               Math Alphanum, Ancient numerics, etc.
               ← ∈ ⊂ ≡ → ∀ ∃ + − × ÷

EMOTICON (V,A) — 15 blocks, 3,056 chars
  E.01..E.15   Misc Symbols, Emoticons, Transport, Alchemical,
               Chess, Domino, Playing Cards, etc.
               ← 😀 😢 🔥 ❤ ⚡

MUSICAL (T)    — 7 blocks, 1,024 chars
  T.01..T.07   Yijing Hexagram, Byzantine, Musical Symbols,
               Znamenny, Ancient Greek Musical, etc.
               ← 𝄞 𝄢 ䷀

TỔNG: 53 blocks = 9,200 chars range
      8,284 curated entries (udc.json)
      140,382 non-zero P_weights (udc_p_table.bin, bao gồm block/category defaults)
```

> Chi tiết mỗi chiều:
> - Shape: `UDC_DOC/UDC_S0_ARROW_tree.md`, `UDC_S1_GEOMETRIC_tree.md`, `UDC_S2_BOXDRAWING_tree.md`, `UDC_S3_S7_tree.md`
> - Relation: `UDC_DOC/UDC_R_RELATION_tree.md`
> - Valence: `UDC_DOC/UDC_V_VALENCE_tree.md`
> - Arousal: `UDC_DOC/UDC_A_AROUSAL_tree.md`
> - Time: `UDC_DOC/UDC_T_TIME_tree.md`

---

## A2. P_weight — 5 Chiều, 2 Bytes

### Bit layout

```
P_weight = u16 (16 bits, 2 bytes)

  [S:4 bit][R:4 bit][V:3 bit][A:3 bit][T:2 bit]
   ╰─────╯ ╰─────╯  ╰─────╯ ╰─────╯  ╰────╯
   0..15    0..15    0..7     0..7     0..3
   Shape    Relation Valence  Arousal  Time

Pack:   mol = (S << 12) | (R << 8) | (V << 5) | (A << 2) | T
Unpack: S = (mol >> 12) & 0xF
        R = (mol >> 8)  & 0xF
        V = (mol >> 5)  & 0x7
        A = (mol >> 2)  & 0x7
        T = mol & 0x3

Tổng trạng thái: 16 × 16 × 8 × 8 × 4 = 65,536
```

### Mỗi codepoint → 1 P_weight DUY NHẤT

```
P_weight(cp) = ∫(Unicode metadata về cp)

Unicode metadata gồm:
  - Name (e.g., "LATIN CAPITAL LETTER A")
  - Category (Lu, Ll, Nd, Sm, So, Mn...)
  - Block
  - Decomposition (à = a + COMBINING GRAVE)
  - Bidirectional class
  - Properties
  - Aliases

42 formulas (xem A3) trích xuất metadata → 5 chiều → pack u16.
```

### Quantization (V và A)

```
Raw V/A ∈ [-1.0, +1.0] → quantize → V/A ∈ [0..7]

quantize(x) = clamp( ⌊(x + 1.0) / 2.0 × 7 + 0.5⌋, 0, 7 )

  raw = -1.0  →  V = 0  (rất tiêu cực / rất yên tĩnh)
  raw =  0.0  →  V = 4  (trung tính)
  raw = +1.0  →  V = 7  (rất tích cực / rất kích thích)

Nguồn raw V/A:
  1. NRC-VAD Lexicon (54,801 terms tiếng Anh)
  2. Emoji subgroup mapping (face-smiling → V=+0.8, face-negative → V=-0.7)
  3. Fallback: Unicode category (So → 0.0, Sm → 0.0)
```

### Distance 5D

```
Euclidean (general search):
  distance(A, B) = √( Σ (A_d_norm − B_d_norm)² )
  normalize: S/15, R/15, V/7, A/7, T/3
  range: [0.0, √5 ≈ 2.236]
  Giống: < 0.3 | Khác: > 1.0

Emotion-weighted (emotion context):
  distance(A, B) = 2|V₁ − V₂| + |A₁ − A₂|
  V weighted 2× vì emotion dominant trong nhận thức
```

### Effective dimensionality

```
1 molecule đơn:     16 bits = 65,536 states
1 chain (10 mols):  + ordering = ~10^48 states
+ Silk (1000 edges): 2^1000 ≈ 10^301 subgraphs
+ KnowTree position: 65,536^4 = 10^19 positions
→ THỰC TẾ: hàng ngàn chiều hiệu dụng qua composition

So sánh: DNA 4 ký tự × 3 tỷ = toàn bộ sự sống
HomeOS: 5 chiều × tỷ links = toàn bộ tri thức 1 đời
```

---

## A3. Encode ∫ — Tích Phân Scale-Invariant

### Nguyên lý cốt lõi

```
Encode = tích phân thật trên không gian rời rạc 5D.

P_w("tôi yêu bạn")
  ~ ∫(P_w("tôi"), P_w("yêu"), P_w("bạn"))         — word partition
  ~ ∫(P_w("tôi yêu"), P_w("yêu bạn"))              — bigram partition
  ~ ∫(P_w("t"), P_w("ô"), P_w("i"),...,P_w("n"))   — char partition
  ~ ∫(P_w("T"), P_w("Ô"),...,P_w("N"))              — uppercase variant

"~" = Silk link. Khác partition → kết quả GẦN nhau, Silk hội tụ.
Giống calculus: ∫₀¹f(x)dx không phụ thuộc cách chia Riemann sum.
Scale-invariant = fractal: cùng phép toán ở mọi zoom level.
```

### 42 formulas, 3 tầng

```
Tầng 1 — Master (1 formula):
  F₀(cp) = [f_S(cp), f_R(cp), f_V(cp), f_A(cp), f_T(cp)]

Tầng 2 — 5 Dimension Encoders (5 formulas):
  f_S: shape → S ∈ [0..15]     (13 SDF blocks)
  f_R: relation → R ∈ [0..15]  (18 MATH blocks)
  f_V: valence → V ∈ [0..7]    (15 EMOTICON blocks)
  f_A: arousal → A ∈ [0..7]    (15 EMOTICON blocks, shared with V)
  f_T: time → T ∈ [0..3]       (7 MUSICAL blocks)

Tầng 3 — 36 Sub-classifiers:
  S: 10 classifiers (is_arrow, is_geometric, is_line, is_fill,
     is_symbol, is_size, is_position, is_pattern, is_astro, is_technical)
  R: 10 classifiers (is_operator, is_set_logic, is_comparison, is_number,
     is_letter_script, is_fraction, is_punctuation, is_currency,
     is_ancient, is_formatting)
  V: 5 quantizers (very_positive, positive, neutral, negative, very_negative)
  A: 5 quantizers (very_excited, excited, moderate, calm, very_calm)
  T: 6 classifiers (is_note_duration, is_pitch_scale, is_dynamics,
     is_neume, is_hexagram, is_modifier)

TỔNG: 1 + 5 + 36 = 42 công thức
```

> Chi tiết formulas: `docs/tailieu_nghiencuu/UDC_DOC/UDC_formulas.md`

### Encode 1 codepoint: P_weight từ Unicode metadata

```
P_w(cp) = ∫(name, category, decomposition, block, aliases, properties)

Ví dụ — "A" (U+0041):
  Name: "LATIN CAPITAL LETTER A"
    LATIN   → R: is_letter_script → R.4
    CAPITAL → S: is_size (heavy)  → S > 0
    LETTER  → confirms R channel
    A       → offset in block
  Category: Lu (uppercase letter)
  → P_w("A") = packed [S, R, V, A, T] duy nhất

Ví dụ — Vietnamese diacritics:
  "à" = decomposition(U+0061 "a" + U+0300 "COMBINING GRAVE ACCENT")
  → P_w("à") = compose(P_w("a"), P_w(GRAVE))
  → GRAVE = Mn (Mark, nonspacing) → ảnh hưởng T/V dimension
  → Dấu huyền = falling tone → V giảm

  6 thanh tiếng Việt map tự nhiên vào 5D:
    không dấu = neutral V/A
    huyền (grave, falling) = V giảm
    sắc (acute, rising) = V tăng
    hỏi (hook, dipping) = complex V
    ngã (tilde, broken rising) = V dao động
    nặng (dot below, heavy) = A tăng
```

### Encode 1 câu: fractal compose

```
"Hà Nội đẹp"
  → UTF-8 decode → codepoints: [H, à, space, N, ộ, i, space, đ, ẹ, p]
  → mỗi cp → P_w: [pw_H, pw_à, pw_space, pw_N, pw_ộ, ...]
  → compose theo Zipf (word đầu nặng nhất):
      P_w("Hà") = compose(pw_H, pw_à)
      P_w("Nội") = compose(pw_N, pw_ộ, pw_i)
      P_w("đẹp") = compose(pw_đ, pw_ẹ, pw_p)
      P_w("Hà Nội đẹp") = compose(P_w("Hà"), P_w("Nội"), P_w("đẹp"))
  → 1 u16 = fingerprint 5D

  "Hà Nội đẹp" ≠ "Sài Gòn xấu" vì V khác (đẹp=V cao, xấu=V thấp)
  "Hà Nội đẹp" ≈ "Hà Nội xinh" vì V gần nhau (Silk nối)
```

### Hai pha encode

```
① ∫ₛ (spatial) — Bootstrap, chạy 1 lần:
   Mỗi UDC char → parse Unicode metadata → 42 formulas → P_weight
   Kết quả: 9,200 L0 anchors, SEALED vĩnh viễn
   Chi phí: O(1) per codepoint (binary search 53 blocks)
   Bảng đã compile: json/udc_p_table.bin (314KB)

② ∫ₜ (temporal) — Runtime, liên tục:
   Input text → tokens → compose → STM node → Silk co-activate
   "tôi buồn mất việc" → ΔV = -0.75 (amplified qua compose)
   Mỗi input: encode → compare với KnowTree → learn or respond
```

---

## A4. Compose — Tổng Hợp Sinh Học, Không Trung Bình

### 5 quy tắc compose (1 per dimension)

```
compose(A, B) → C:

  S:  Union     = max(A.S, B.S)
      Hình dạng hợp nhất — lớn nhất thắng.

  R:  Compose   = Zipf-weighted average
      Quan hệ tích lũy — phần tử đầu nặng hơn.

  V:  Amplify   = amplify(A.V, B.V, w)
      KHUẾCH ĐẠI, KHÔNG trung bình.
      Sinh học: cortisol + adrenaline → stress MẠNH HƠN từng cái riêng lẻ.

  A:  Max       = max(A.A, B.A)
      Cường độ lấy cao nhất — 1 tiếng hét trong phòng yên tĩnh = ồn.

  T:  Dominant  = vote(majority)
      Thời gian lấy đa số — nếu hầu hết ký tự static thì kết quả static.
```

### Amplify formula

```
amplify(Va, Vb, w):
  base  = (Va + Vb) / 2
  boost = |Va − base| × w × 0.5
  Cⱽ    = base + sign(Va + Vb) × boost

  w = Silk weight (0.0 default, >0 nếu đã co-activate)
  w = 0 → boost = 0 → amplify degenerates thành average
  w > 0 → amplification tỷ lệ với Silk strength
  → Strangers don't amplify. Connected concepts do.
```

### Compose KHÔNG commutative

```
"tôi yêu bạn" ≠ "bạn yêu tôi"

compose(tôi, yêu, bạn):
  tôi × w=1000 (Zipf: phần tử đầu, nặng nhất)
  yêu × w=500
  bạn × w=333
  → P_w = X

compose(bạn, yêu, tôi):
  bạn × w=1000
  yêu × w=500
  tôi × w=333
  → P_w = Y ≠ X

123 ≠ 321 ≠ 312 ≠ 231

Lý do: Zipf weighting — phần tử đầu nặng nhất.
Chain = đường spline qua 5D, mỗi thứ tự = đường đi khác.
Giống DNA: ATG ≠ GTA ≠ TAG — cùng nucleotides, khác codon, khác protein.

→ Collision (cùng P_w) chỉ khi cùng elements VÀ cùng thứ tự = gần như không thể.
→ 16 bit ĐỦ vì order tạo uniqueness.
```

### Compose nhiều cấp (fractal)

```
P(sách) = compose(
  P(chương A) = compose(
    P(đoạn 1) = compose(
      P(câu 1) = compose(mol₁, mol₂, mol₃...)
      P(câu 2) = ...
    )
    P(đoạn 2) = ...
  )
  P(chương B) = ...
)

Mỗi cấp đều có P_weight. Cùng phép toán ở mọi level.
P_w(từ), P_w(câu), P_w(đoạn), P_w(chương), P_w(sách) — cùng u16.
```

---

## A5. Decode ∂ — Đạo Hàm Qua KnowTree

### Nguyên lý

```
Encode ∫: text → chain of P_w → compose → summary P_w
Decode ∂: summary P_w → KnowTree lookup → chain → text output

∫ và ∂ là nghịch đảo. Encode tích phân, Decode đạo hàm.
```

### KnowTree = content-addressable memory

```
Lưu:
  - P_w chưa tồn tại → tạo node mới (lưu chain + text gốc)
  - P_w đã tồn tại → ghi pointer (2 bytes) tới node có sẵn
  - Nếu cùng P_w nhưng chain khác → 2 nodes riêng biệt (chain là identity thật)

Mọi cấp đều là node:
  P_w("tôi") = node → chain = [P_w(t), P_w(ô), P_w(i)]
  P_w("tôi yêu bạn") = node → chain = [P_w(tôi), P_w(yêu), P_w(bạn)]
  P_w(sách) = node → chain = [P_w(chương1), P_w(chương2), ...]

Miễn P_w unique thì lưu. Miễn chain khác thì khác node.
```

### Decode pipeline

```
1. Có P_w_query (từ input đã encode)
2. Tìm chain gần nhất trong KnowTree (by 5D distance)
3. Mỗi link trong chain → lookup node → lấy content đã lưu
4. Ghép content → output text

Ví dụ:
  Query: "Hà Nội là gì?"
  → encode → P_w_query
  → KnowTree tìm chain gần nhất: [P_w("Hà Nội"), P_w("là"), P_w("thủ đô"), P_w("VN")]
  → mỗi P_w → lookup node → "Hà Nội", "là", "thủ đô", "Việt Nam"
  → Output: "Hà Nội là thủ đô Việt Nam"
```

### 3 kênh decode (cùng toán tử ∂, khác không gian)

```
① ∂P/∂space = ∇f(p)       → render hình ảnh (SDF visualization)
② ∂V/∂time  = V'(t)       → chọn tone cảm xúc (ConversationCurve)
③ ∂P/∂experience = ΔP     → đo novelty (mới vs đã biết → learning signal)
```

### L0 / L1 / L5+ distinction

```
L0 = 9,200 UDC chars — P_weight SEALED, vĩnh viễn
     Calibration standard (0°C = nước đá, 100°C = sôi)
     Mọi concept mới đo khoảng cách từ L0

L1 = emoji/UTF-32 (32,492 chars) — ALIAS trỏ về L0
     🔥 U+1F525 trỏ về UDC char trong block E.08
     Emoji KHÔNG PHẢI L0. Chỉ là tên gọi khác.

L5+ = learned — tích lũy qua encode → Silk → Dream → QR
     Mutable: weight thay đổi theo Hebbian co-activation
     Mature: fire_count ≥ Fibonacci threshold → Dream → QR (vĩnh viễn)
```

---

## A6. Số Liệu Thực Tế

### Verified from udc.json (2026-03-30)

```
Blocks: 53 (NOT 58/59 as some docs say)
  SDF:      13 blocks, 1,904 chars
  MATH:     18 blocks, 3,216 chars
  EMOTICON: 15 blocks, 3,056 chars
  MUSICAL:   7 blocks, 1,024 chars

Characters:
  Range total:           9,200
  Curated entries:       8,284 (in udc.json)
  Non-zero P_weights:  140,382 (in udc_p_table.bin, including defaults)
  Binary table:        157,386 entries (314,772 bytes)

Formulas: 42 (1 master + 5 encoders + 36 sub-classifiers)
SDF primitives: 18 (with analytical gradients)
Boolean ops: 3 (union, intersect, subtract)
```

### Data files

```
json/udc.json           — 8,284 curated entries, block metadata, integral kernels
json/udc_p_table.bin    — 314KB compiled lookup table (157,386 × u16)
tools/build_full_udc.py — Python generator (3 priority tiers)

Tài liệu/Origin/UCD/   — Official Unicode Character Database (raw data)
```

---

## Sai Lệch Đã Sửa

| Tài liệu | Nói | Thực tế | Ghi chú |
|-----------|-----|---------|---------|
| BLUEPRINT.md | 59 blocks, 8,846 chars | 53 blocks, 9,200 chars | Data = source of truth |
| ORIGIN_VISION.md | 58 blocks | 53 blocks | Đếm sai |
| Previous sessions | "SDF chỉ cho S blocks" | SDF tổng quát mọi không gian | SDF = signed distance, bất kỳ domain |
| Previous sessions | "Decode aspirational" | Decode = ∂ qua KnowTree, cụ thể | Lookup chain → content |
| Previous sessions | "a-z neutral" | P_w = ∫(Unicode metadata), unique | Name/category/decomposition khác |
| Previous sessions | "Compose commutative" | NON-commutative, Zipf ordering | 123 ≠ 321 |
| Previous sessions | "16 bit có collision" | Order tạo uniqueness + chain phân biệt | Chain = identity, molecule = fingerprint |
| Previous sessions | "Compose = average" | Amplify/Max/Union/Dominant | Synergy sinh học, không trung bình |

---

> **Tham chiếu chi tiết:**
> - SDF primitives: `docs/tailieu_nghiencuu/UDC_DOC/UDC_S1_GEOMETRIC_tree.md`
> - 42 formulas: `docs/tailieu_nghiencuu/UDC_DOC/UDC_formulas.md`
> - Math formulas: `docs/tailieu_nghiencuu/UDC_DOC/UDC_real_formulas.md`
> - Block mapping: `docs/tailieu_nghiencuu/UDC_DOC/UDC_map.md`
> - Semantic groups: `docs/tailieu_nghiencuu/UDC_DOC/UDC_semantic_groups.md`
> - Group formulas: `docs/tailieu_nghiencuu/UDC_DOC/UDC_group_formulas.md`
> - BLUEPRINT: `docs/BLUEPRINT.md` §1-§3
> - Sinh học v2: `docs/tailieu_nghiencuu/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md`
> - Synthesis: `docs/For_Nox/HOMEOS_SYNTHESIS_CORRECTED.md`
test
