# BLUEPRINT — Kiến Trúc HomeOS

> **Tài liệu kỹ thuật duy nhất. Mọi thứ khác là tham khảo.**
> **Ngày: 2026-03-25. Tổng hợp từ 20+ docs, 100+ sessions, 130K LOC.**
> **Tác giả: goldlotus1810 (vision) + Claude (tổng hợp kỹ thuật)**

---

## MỤC LỤC

```
RỄ    §1  L0 — Gene: 8,846 công thức SDF (bảng tuần hoàn)
      §2  Molecule: P_weight u16, 5 chiều, 2 bytes
      §3  Encode ∫: input → molecule (từng codepoint DUY NHẤT)

THÂN  §4  Chain: chuỗi u16 links, 2 bytes/link, vô hạn
      §5  KnowTree: cây phân tầng L0→L3, 18KB, O(4) lookup
      §6  Silk: structural (0 bytes) + Hebbian (learned)

CÀNH  §7  Neuron Model: STM → Silk → Dream → QR
      §8  7 Instincts: phản xạ bẩm sinh, hardcoded
      §9  ConversationCurve: f(x), f'(x), f''(x)
      §10 SecurityGate: 3 layers, chạy trước mọi thứ

TÁN   §11 Agent Hierarchy: AAM → LeoAI → Chiefs → Workers
      §12 Response Pipeline: 14 cơ chế DNA → output
      §13 5 Checkpoints: cell cycle, bỏ = ung thư tri thức

PHỤ   §14 Hiện trạng: cái gì có, cái gì chưa
      §15 Thứ tự xây: rễ trước, tán sau
      §16 Tham chiếu: link đến tài liệu gốc
```

---

---

# RỄ — NỀN MÓNG KHÔNG THAY ĐỔI

---

## §1. L0 — Gene: 8,846 Công Thức SDF

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §I](docs/HomeOS_SPEC_v3.md), [`docs/UDC_DOC/`](docs/UDC_DOC/)

### Nguyên lý

Mỗi ký tự Unicode trong 59 blocks đặc biệt **là** một hàm Signed Distance Field. Không phải "đại diện cho". Là.

```
f(p) = signed distance from point p to surface

  f(p) < 0    → bên trong     → THỂ TÍCH
  f(p) = 0    → trên bề mặt   → HÌNH DẠNG
  f(p) > 0    → bên ngoài     → KHÔNG GIAN
  ∇f(p)       → pháp tuyến    → MÀU SẮC
  ∂f/∂t       → biến thiên    → ÂM THANH
  p           → tọa độ        → VỊ TRÍ

1 hàm. 1 điểm. Ra tất cả.
```

### 18 SDF Primitives

```
#   Tên          f(P)                             ∇f (analytical)
─────────────────────────────────────────────────────────────────
0   SPHERE       |P| − r                          P / |P|
1   BOX          ||max(|P|−b, 0)||                sign(P)·step(|P|>b)
2   CAPSULE      |P−clamp(y,0,h)ĵ| − r           norm(P − closest)
3   PLANE        P.y − h                          (0, 1, 0)
4   TORUS        |(|P.xz|−R, P.y)| − r           chain rule
5   ELLIPSOID    |P/r| − 1                        P/r² / |P/r|
6   CONE         dot blend                        slope normal
7   CYLINDER     max(|P.xz|−r, |P.y|−h)          radial/cap
8   OCTAHEDRON   |x|+|y|+|z| − s                 sign(P)/√3
9   PYRAMID      pyramid(P,h)                     slope analytical
10  HEX_PRISM    max(hex−r, |y|−h)                radial hex/cap
11  PRISM        max(|xz|−r, |y|−h)               radial/cap
12  ROUND_BOX    BOX − rounding                   smooth corner
13  LINK         torus compound                    chain rule
14  REVOLVE      revolve_Y                         radial
15  EXTRUDE      extrude_Z                         radial
16  CUT_SPHERE   max(|P|−r, P.y−h)                norm(P)/(0,1,0)
17  DEATH_STAR   opSubtract                        ±norm(P)

Tất cả ∇f ANALYTICAL — không cần numerical differentiation.
```

> Chi tiết sub-variants → [`docs/UDC_DOC/UDC_S1_GEOMETRIC_tree.md`](docs/UDC_DOC/UDC_S1_GEOMETRIC_tree.md)

### 59 Unicode Blocks = Bảng Tuần Hoàn

```
SHAPE (S) — 14 blocks, 1,838 ký tự:
  S.01  Arrows                 U+2190..21FF    112 chars
  S.02  Box Drawing            U+2500..257F    128
  S.03  Block Elements         U+2580..259F     32
  S.04  Geometric Shapes       U+25A0..25FF     96   ← ● ■ ▲ ○ □ △
  S.05  Dingbats               U+2700..27BF    192
  S.06  Supp Arrows-A          U+27F0..27FF     16
  S.07  Supp Arrows-B          U+2900..297F    128
  S.08  Misc Symbols+Arrows    U+2B00..2BFF    256
  S.09  Geometric Shapes Ext   U+1F780..1F7FF  128
  S.10  Supp Arrows-C          U+1F800..1F8FF  256
  S.11  Ornamental Dingbats    U+1F650..1F67F   48
  S.12  Misc Technical         U+2300..23FF    256
  S.13  Braille Patterns       U+2800..28FF    256
  S.14  Control Pictures       U+2400..243F     64

RELATION (R) — 21 blocks, 2,563 ký tự:
  M.04  Mathematical Operators U+2200..22FF    256   ← ∈ ⊂ ≡ → ∀ ∃
  M.07  Supp Math Operators    U+2A00..2AFF    256
  M.08  Math Alphanum          U+1D400..1D7FF 1024
  ... (+ 18 blocks khác, xem Spec v3 §1.4)

EMOTION (V,A) — 17 blocks, 3,487 ký tự:
  E.08  Misc Sym+Pictographs  U+1F300..1F5FF  768   ← 🔥 🌍 🎭
  E.09  Emoticons              U+1F600..1F64F   80   ← 😀 😢 😡
  ... (+ 15 blocks khác)

TIME (T) — 7 blocks, 958 ký tự:
  T.04  Musical Symbols        U+1D100..1D1FF  256   ← 𝄞 𝄢
  ... (+ 6 blocks khác)

TỔNG: 59 blocks = 8,846 ký tự L0 gốc
```

> Chi tiết mỗi chiều:
> - Shape: [`docs/UDC_DOC/UDC_S0_ARROW_tree.md`](docs/UDC_DOC/UDC_S0_ARROW_tree.md), [`UDC_S1_GEOMETRIC_tree.md`](docs/UDC_DOC/UDC_S1_GEOMETRIC_tree.md)
> - Relation: [`docs/UDC_DOC/UDC_R_RELATION_tree.md`](docs/UDC_DOC/UDC_R_RELATION_tree.md)
> - Valence: [`docs/UDC_DOC/UDC_V_VALENCE_tree.md`](docs/UDC_DOC/UDC_V_VALENCE_tree.md)
> - Arousal: [`docs/UDC_DOC/UDC_A_AROUSAL_tree.md`](docs/UDC_DOC/UDC_A_AROUSAL_tree.md)
> - Time: [`docs/UDC_DOC/UDC_T_TIME_tree.md`](docs/UDC_DOC/UDC_T_TIME_tree.md)
> - Công thức tổng hợp: [`docs/UDC_DOC/UDC_real_formulas.md`](docs/UDC_DOC/UDC_real_formulas.md)

---

## §2. Molecule: P_weight — 2 Bytes Chứa 5 Chiều

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §1.3](docs/HomeOS_SPEC_v3.md)

### Cấu trúc bit

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
```

### MỖI codepoint → 1 P_weight DUY NHẤT

```
HIỆN TẠI (SAI):
  a-z → CÙNG 1 molecule (0,0,4,4,2) = 146
  A-Z → CÙNG 1 molecule (0,0,4,5,2) = 150

ĐÚNG (CẦN XÂY):
  Mỗi codepoint → P_weight RIÊNG, tính từ vị trí trong block.

  Thuật toán:
    block_index = lookup_block(codepoint)   // 59 blocks, binary search
    offset = codepoint - block.start         // vị trí trong block
    total = block.end - block.start + 1      // số ký tự trong block

    // Chiều dominant từ block group
    if block ∈ SDF_BLOCKS:
      S = block.base_S + (offset * 15 / total)   // 0..15 phân bổ đều
      R = block.default_R                         // giá trị mặc định cho block
      V = 4  (neutral)
      A = 4  (neutral)
      T = 2  (neutral)

    if block ∈ MATH_BLOCKS:
      S = block.default_S
      R = block.base_R + (offset * 15 / total)   // R dominant
      V = 4, A = 4, T = 1

    if block ∈ EMOTICON_BLOCKS:
      S = block.default_S
      R = block.default_R
      V = block.base_V + (offset * 7 / total)    // V dominant
      A = block.base_A + (offset * 7 / total)    // A co-varies
      T = 2

    if block ∈ MUSICAL_BLOCKS:
      S = block.default_S
      R = block.default_R
      V = 4, A = 4
      T = block.base_T + (offset * 3 / total)    // T dominant

  Kết quả: 8,846 codepoints → 8,846 P_weights KHÁC NHAU.

  Ví dụ:
    'A' (U+0041) → Latin Basic, S=1, R=0, V=4, A=5, T=2  = 4242
    'a' (U+0061) → Latin Basic, S=0, R=0, V=4, A=4, T=2  = 146
    'B' (U+0042) → Latin Basic, S=1, R=0, V=4, A=5, T=2  = 4242 (*)
    '●' (U+25CF) → Geometric,   S=8, R=0, V=4, A=3, T=0  = 32908
    '∈' (U+2208) → Math Ops,    S=0, R=4, V=4, A=4, T=1  = 1169
    '😀' (U+1F600) → Emoticons, S=0, R=0, V=7, A=6, T=2  = 250

  (*) Trong cùng 1 block, A-Z phân bổ đều → A≠B≠C.
      Chữ LATIN BASIC (U+0041..007A) cần bảng riêng.
```

### Bảng P_weight cho Latin (a-z, A-Z, 0-9) — PHẢI hardcode

```
Chữ cái không nằm trong 59 UDC blocks. Chúng là ALIAS trỏ về
các concept trong 59 blocks. Nhưng để mỗi chữ cái khác nhau:

  Thuật toán: hash_letter(cp)
    // Mỗi chữ cái → 1 seed duy nhất → P_weight duy nhất
    seed = FNV-1a(codepoint)
    S = (seed >> 12) & 0xF   // phân tán đều 0..15
    R = (seed >> 8)  & 0xF
    V = 3 + ((seed >> 5) & 0x3)  // 3..6 (near-neutral)
    A = 3 + ((seed >> 2) & 0x3)  // 3..6 (near-neutral)
    T = seed & 0x3

  Kết quả: 'a' ≠ 'b' ≠ 'c' ≠ ... ≠ 'z' — mỗi chữ cái DUY NHẤT.
  'a' ≈ near-neutral nhưng khác 'b' trong ít nhất 2 chiều.
```

> **Bảng 8,846 P_weights đầy đủ:** [`json/udc.json`](json/udc.json) (7.6 MB, đã sinh bởi Rust `udc_gen`)
> **Bảng đã pack binary:** [`json/udc_p_table.bin`](json/udc_p_table.bin) (308 KB)

---

## §3. Encode ∫ — Input → Molecule

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §1.6](docs/HomeOS_SPEC_v3.md), [`old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md` §I](old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md)

### Encode 1 codepoint

```
encode_codepoint(cp) → u16:
  1. Nếu cp ∈ [0x0000..0x007F] (ASCII):
     → hardcode bảng 128 entries (mỗi entry DUY NHẤT)
  2. Nếu cp ∈ 59 UDC blocks:
     → tính từ block position (§2 thuật toán)
  3. Nếu cp ∈ emoji/UTF-32 (32,492 aliases):
     → alias_lookup(cp) → UDC index → P_weight của UDC đó
  4. Fallback: cp chưa biết
     → hash_unknown(cp) → P_weight phân tán

Chi phí: O(1) per codepoint (binary search 59 blocks).
```

### Compose 2 molecules

```
compose(A, B) → C:
  KHÔNG phải trung bình. Là TỔNG HỢP theo quy tắc từng chiều.

  S:  union(Aˢ, Bˢ)         = max(A.S, B.S)     // hình dạng hợp nhất
  R:  compose(Aᴿ, Bᴿ)       = (A.R + B.R) / 2   // quan hệ = tổ hợp
  V:  amplify(Aⱽ, Bⱽ, w)    = xem công thức dưới // KHUẾCH ĐẠI
  A:  max(Aᴬ, Bᴬ)           = max(A.A, B.A)     // cường độ lấy cao
  T:  dominant(Aᵀ, Bᵀ)      = A.T if |A.T|>|B.T| else B.T

  amplify(Va, Vb, w):
    base  = (Va + Vb) / 2
    boost = |Va - base| × w × 0.5
    Cⱽ    = base + sign(Va + Vb) × boost
    // Đẩy về phía dominant, không trung bình hóa
    // Sinh học: cortisol + adrenaline → stress MẠNH HƠN, không yếu đi

  w = Silk weight giữa A và B (0.0 nếu chưa biết, >0 nếu đã co-activate)
```

### Encode 1 câu

```
encode_sentence("Hà Nội đẹp") → u16:
  1. UTF-8 decode → codepoints: [72, 224, 32, 78, 7897, 105, 32, 273, 7865, 112]
                                  H   à   _   N   ộ     i   _   đ   ẹ     p
  2. Mỗi cp → P_weight: [pw_H, pw_à, pw_space, pw_N, pw_ộ, ...]
  3. Gom thành word theo space:
     word_1 = compose(pw_H, compose(pw_à))     = mol("Hà")
     word_2 = compose(pw_N, pw_ộ, pw_i)        = mol("Nội")
     word_3 = compose(pw_đ, pw_ẹ, pw_p)        = mol("đẹp")
  4. Sentence = compose(word_1, compose(word_2, word_3))

  KẾT QUẢ: 1 u16 = fingerprint 5D của toàn câu.
  "Hà Nội đẹp" ≠ "Sài Gòn xấu" vì V khác hẳn (đẹp=V cao, xấu=V thấp).
  "Hà Nội đẹp" ≈ "Hà Nội xinh" vì V gần nhau.
```

### Distance 5D

```
distance(A, B) = √( Σ_{d=1}^{5} (Aᵈ_norm - Bᵈ_norm)² )

  Với normalize: S_norm = S/15, R_norm = R/15, V_norm = V/7, A_norm = A/7, T_norm = T/3

  distance ∈ [0.0, √5 ≈ 2.236]
  Giống nhau: distance < 0.3
  Khác nhau:  distance > 1.0
```

---

---

# THÂN — CẤU TRÚC DỮ LIỆU

---

## §4. Chain — Chuỗi DNA Của Tri Thức

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §II](docs/HomeOS_SPEC_v3.md), [`docs/KNOWTREE_DESIGN.md`](docs/KNOWTREE_DESIGN.md)

### Cấu trúc

```
Chain = mảng u16 liên tục, mỗi link = 1 index vào KnowTree L3.

  DNA:     A—T—C—G—G—A—T—C—C—T—A—G        (4 loại nucleotide)
  HomeOS:  [42][108][7291][53][108][2004]    (8,846 loại UDC)

  1 link = 2 bytes (u16)
  1 từ "Việt" = 4 links = 8 bytes
  1 câu = ~20 links = ~40 bytes
  1 sách = ~350,000 links = ~700 KB
  1 đời = ~tỷ links = ~2 GB

Chain KHÔNG copy nội dung. Chain TRỎ.
Copy cả cuốn sách = 2 bytes (1 pointer đến chain gốc).
```

### Đọc chain = chạy thẳng

```
evaluate(chain, context) → value_5D:
  result = P_weight_zero
  for link in chain:
    node = KnowTree.lookup(link)
    result = compose(result, node.P_weight)
  return result

Giống ribosome đọc mRNA: chạy từ đầu đến cuối → ra protein.
Thứ tự trong chain = quan hệ = Structural Silk = 0 bytes overhead.
```

---

## §5. KnowTree — Cây Phân Tầng

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §1.7](docs/HomeOS_SPEC_v3.md), [`docs/KNOWTREE_DESIGN.md`](docs/KNOWTREE_DESIGN.md), [`docs/ORIGIN_VISION.md` §VIII](docs/ORIGIN_VISION.md)

### Cấu trúc cố định (boot)

```
ROOT
├── L0: 5 nhóm       (SDF, MATH, EMOTICON, MUSICAL, RELATION)
│   ├── L1: 59 blocks    (S.01..S.14, M.01..M.21, E.01..E.17, T.01..T.07)
│   │   ├── L2: ~200 sub-ranges  (Arrow types, Geometric sub-types, ...)
│   │   │   └── L3: 8,846 UDC chars  (LÁ — mỗi lá = 1 P_weight = 2 bytes)
│   │   │
│   │   │   Alias layer (riêng biệt, KHÔNG trong cây):
│   │   │   32,492 emoji/UTF-32 → trỏ về L3 UDC index
│   │   │
│   │   └── ...
│   └── ...
└── ...

Kích thước:
  L0:  5 × 2B     =     10 bytes
  L1:  59 × 2B    =    118 bytes
  L2:  ~200 × 2B  =    400 bytes
  L3:  8,846 × 2B = 17,692 bytes
  ─────────────────────────────────
  Tổng: ~18 KB (vừa L1 cache)

Alias table: 32,492 × 8B ≈ 254 KB (riêng biệt)

Tra cứu: L0 → L1 → L2 → L3 = O(4) = O(1) thực tế.
```

### Cấu trúc learned (phát triển)

```
L4+: tri thức đã học, PHÁT TRIỂN mỗi ngày.

ROOT
├── L0-L3: (cố định, 18 KB — bảng tuần hoàn)
│
└── LIBRARY (L4+):
    ├── [0] facts/         ← tri thức đã học
    │   ├── geography/
    │   │   └── "Hà Nội là thủ đô Việt Nam" → chain[H,à,N,ộ,i,l,à,...]
    │   ├── science/
    │   └── personal/
    ├── [1] books/         ← sách đã đọc
    │   ├── "Cuốn Theo Chiều Gió"/
    │   │   ├── chapter_1/ → chains...
    │   │   └── chapter_2/ → chains...
    │   └── "Hoàng Tử Bé"/
    ├── [2] conversations/ ← lịch sử hội thoại
    ├── [3] skills/        ← kỹ năng tổng hợp
    ├── [4] people/        ← người đã gặp
    ├── [5] emotions/      ← ký ức cảm xúc
    └── ... (65,536 slots tiềm năng)

Mỗi nhánh = array tối đa 65,536 phần tử.
Mỗi phần tử = hoặc LÁ (2 bytes P_weight) hoặc NHÁNH CON (pointer).
Lồng nhau = fractal = vô hạn thực tế.

65,536² = 4.3 tỷ entries ở depth 2 — 1 đời chỉ dùng <4%.
```

### Data structures (Olang)

```olang
// KnowTree node
type KTNode {
  mol: Num,         // P_weight u16
  children: Array,  // [KTNode] — nhánh con
  chains: Array,    // [Chain]  — dữ liệu tại node này
  fires: Num,       // số lần truy cập
  silk_in: Array,   // [SilkEdge] — Hebbian links đến node này
}

// Chain = mảng u16 (UDC indices)
type Chain {
  links: Array,     // [Num] — mỗi Num = u16 UDC index
  mol: Num,         // P_weight tổng hợp
  text: Str,        // text gốc (cho decode)
  source: Str,      // "facts" | "books" | "conversations"
}

// Tra cứu: đi từ gốc xuống lá
fn kt_lookup(query_mol) → KTNode:
  // L0: chọn nhóm (S/R/V/A/T) dựa trên chiều dominant
  group = dominant_dim(query_mol)  // 0..4
  // L1: chọn block dựa trên giá trị chiều dominant
  block = group.children[query_mol.dominant_value]
  // L2: chọn sub-range
  sub = block.children[offset_in_block]
  // L3: chọn UDC char
  return sub.children[char_offset]
```

---

## §6. Silk — Tơ Nhện Nối Tri Thức

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §2.3, §III.⑥](docs/HomeOS_SPEC_v3.md)

### 2 loại Silk

```
1. STRUCTURAL SILK (implicit, 0 bytes):
   = thứ tự trong chain/array
   Chương 1 trước chương 2 vì index [0] < [1].
   Engine chạy thẳng từ đầu → cuối → ra giá trị.
   Chi phí: KHÔNG CÓ.

2. HEBBIAN SILK (explicit, learned):
   = co-activation strength giữa 2 node BẤT KỲ.
   = CẦU NỐI NGANG giữa các nhánh khác nhau trong KnowTree.

   Ví dụ:
     "buồn" ở conversations/session_3/turn_5
       ↔ (silk_weight = 0.85)
     "mất việc" ở facts/personal
```

### Hebbian Learning — thuật toán

```
co_activate(A, B, emotion_tag):
  // "Fire together, wire together"

  emotion_factor = (|A.V| + |B.V|) / 2 × max(A.A, B.A) / 7.0
  // Cảm xúc mạnh → kết nối mạnh hơn (sinh học: cortisol tăng memory)

  Δw = emotion_factor × (1 − w_AB) × 0.1
  // (1 − w_AB): càng gần 1.0, càng khó tăng thêm (saturation)
  // 0.1: learning rate

  w_AB ← w_AB + Δw
  edge.emotion_tag = (emotion.V, emotion.A)  // ghi nhớ cảm xúc khoảnh khắc
  edge.fire_count += 1

Decay (quên):
  w_AB ← w_AB × φ⁻¹^(Δt / 24h)

  φ⁻¹ = (√5 − 1) / 2 ≈ 0.618

  Sau 24h: w × 0.618
  Sau 48h: w × 0.382
  Sau 72h: w × 0.236
  Sau 1 tuần: w × 0.028 → gần như quên

  Trừ khi fire lại → w tăng lại → nhớ.
  Giống synapse: dùng thường xuyên → mạnh. Bỏ → yếu dần.
```

### Silk Walk — khuếch đại cảm xúc

```
silk_walk(start_node, depth=3) → amplified_emotion:
  // Đi theo Hebbian edges, tích lũy cảm xúc

  visited = {}
  queue = [(start_node, 1.0)]  // (node, attenuation)
  total_V = 0, total_A = 0, total_weight = 0

  while queue not empty AND depth > 0:
    (node, atten) = queue.pop()
    if node in visited: continue
    visited.add(node)

    total_V += node.emotion.V × atten
    total_A += node.emotion.A × atten
    total_weight += atten

    for edge in node.silk_edges:
      if edge.weight > 0.1:  // chỉ đi theo edges đủ mạnh
        queue.push((edge.target, atten × edge.weight))

    depth -= 1

  return {
    V: total_V / total_weight,
    A: total_A / total_weight,
    amplification: total_weight  // > 1.0 nếu nhiều edges → KHUẾCH ĐẠI
  }

  // "buồn" ←→ "mất việc" (w=0.85) ←→ "tiền" (w=0.70)
  // silk_walk("buồn") → V = -0.65 × (1 + 0.85×0.5 + 0.70×0.3) = amplified
  // KHÔNG trung bình. KHUẾCH ĐẠI qua mạng lưới.
```

> **Nguồn chi tiết Silk:** [`docs/UDC_DOC/UDC_R_RELATION_tree.md`](docs/UDC_DOC/UDC_R_RELATION_tree.md)

---

---

# CÀNH — CƠ CHẾ SỐNG

---

## §7. Neuron Model — Vòng Đời Tri Thức

> **Nguồn:** [`docs/ORIGIN_VISION.md` §III](docs/ORIGIN_VISION.md), [`docs/HomeOS_SPEC_v3.md` §III-IV](docs/HomeOS_SPEC_v3.md)

```
        ┌──────────────────────────────────┐
        │         SOMA (AAM)                │
        │   Stateless · approve · reject    │
        └──────────┬───────────────────────┘
                   │
      ┌────────────┼────────────┐
      │            │            │
      ▼            ▼            ▼
 ┌──────────┐ ┌──────────┐ ┌──────────┐
 │ DENDRITES│ │ SYNAPSE  │ │  AXON    │
 │ (STM)    │ │ (Silk)   │ │ (QR)    │
 │          │ │          │ │          │
 │ Tạm thời │ │ Hebbian  │ │ Bất biến │
 │ Tự do    │ │ fire→wire│ │ Append   │
 │ xóa/sửa │ │ φ⁻¹ decay│ │ Signed   │
 └──────────┘ └──────────┘ └──────────┘
```

### Vòng đời: Input → STM → Silk → Dream → QR

```
1. INPUT → ENCODE
   text → UTF-8 decode → codepoints → P_weights → compose → sentence_mol
   emotion_tag = text_emotion(input) → (V, A)

2. STM PUSH (dendrites)
   stm_push({ text, mol, emotion, timestamp })
   if len(STM) > 32: evict oldest
   // STM = bộ nhớ ngắn hạn, tự do, có thể xóa

3. SILK CO-ACTIVATE (synapse)
   for each pair (current_input, recent_stm_item):
     co_activate(current.mol, recent.mol, emotion_tag)
   // Fire together → wire together

4. DREAM CYCLE (offline, trigger Fibonacci)
   trigger khi: fire_count ≥ Fib(n) = 2, 3, 5, 8, 13, 21, 34, 55...

   dream():
     ① Scan STM → tìm nodes Evaluating
     ② Cluster: gom nodes gần nhau trong 5D
        ε = median(distances) × 0.5
        min_cluster = max(2, ⌊|STM| / 5⌋)
     ③ Cho mỗi cluster:
        center = LCA(members)  // Lowest Common Ancestor weighted
        quality = weight × consistency × (1 - entropy)
     ④ Nếu quality ≥ φ⁻¹:
        → PROPOSE to AAM

5. QR PROMOTION (axon)
   AAM approve → append QR record (vĩnh viễn, signed)
   QR record: { mol, text, chain, timestamp, signature }
   KHÔNG BAO GIỜ xóa. Append-only = DNA methylation.

6. PRUNE (apoptosis)
   Silk edge weight < 0.1 AND fire_count = 0 → SupersedeQR
   KHÔNG xóa vật lý — đánh dấu "không còn hoạt động"
   Giữ lịch sử (giống intron trong DNA — không biểu hiện nhưng vẫn tồn tại)
```

---

## §8. 7 Instincts — Phản Xạ Bẩm Sinh

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §5.3](docs/HomeOS_SPEC_v3.md), [`docs/ORIGIN_VISION.md` §V](docs/ORIGIN_VISION.md)

```
Sơ sinh: bú, nắm, Moro, Babinski → TRƯỚC KHI HỌC.
HomeOS: 7 instincts hardcoded → TRƯỚC KHI TRI THỨC.
Thứ tự ưu tiên ① → ⑦. Honesty luôn chạy đầu tiên.
```

### ① Honesty — Rụt tay khỏi lửa

```
confidence = compute_confidence(evidence)

  evidence = { silk_weight, fire_count, source_count, consistency }

  confidence = 0.3 × min(silk_weight, 1.0)
             + 0.3 × min(fire_count / 10.0, 1.0)
             + 0.2 × min(source_count / 3.0, 1.0)
             + 0.2 × consistency   // ∈ [0, 1]

  confidence < 0.40  → IM LẶNG (hoặc "Tôi không biết")
  0.40..0.70         → "Tôi nghĩ..." (Hypothesis)
  0.70..0.90         → "Có lẽ..." (Opinion)
  ≥ 0.90             → "Đúng." (Fact)

Honesty chạy TRƯỚC mọi response.
Không đủ evidence → không nói → rụt tay trước khi suy nghĩ.
```

### ② Contradiction — Phản xạ đau

```
contradict(A, B) → bool:
  d_V = |A.V - B.V| / 7.0     // khoảng cách Valence (normalized)
  d_R = |A.R - B.R| / 15.0    // khoảng cách Relation

  return d_V > 0.8 AND d_R < 0.2
  // Valence trái ngược (tốt vs xấu) nhưng Relation giống (cùng chủ đề)
  // = mâu thuẫn. "Tôi yêu X" vs "Tôi ghét X" → Contradiction.
```

### ③ Causality — Nhân quả

```
is_causal(A, B) → bool:
  temporal = A.timestamp < B.timestamp   // A xảy ra trước B
  coactive = silk_weight(A, B) > φ⁻¹     // co-activate đủ mạnh
  relation = A.R ∈ CAUSES_RELATION       // R dimension = causal type

  return temporal AND coactive AND relation
  // Cần ≥ 2/3 evidence. Co-activation ≠ nhân quả.
```

### ④ Abstraction — Phân loại

```
abstract_level(cluster) → "concrete" | "categorical" | "abstract":
  center = LCA(cluster)
  variance = Σ distance(member, center)² / |cluster|

  variance < 0.3  → "concrete"     // tất cả rất giống nhau
  variance < 0.7  → "categorical"  // có nhóm rõ ràng
  variance ≥ 0.7  → "abstract"     // rất đa dạng
```

### ⑤ Analogy — Pattern matching

```
analogy(A, B, C) → D:
  // A:B :: C:? → D = C + (B − A) trong 5D
  delta = { S: B.S - A.S, R: B.R - A.R, V: B.V - A.V,
            A: B.A - A.A, T: B.T - A.T }
  D = { S: clamp(C.S + delta.S, 0, 15),
        R: clamp(C.R + delta.R, 0, 15),
        V: clamp(C.V + delta.V, 0, 7),
        A: clamp(C.A + delta.A, 0, 7),
        T: clamp(C.T + delta.T, 0, 3) }
  return D
  // "king" : "queen" :: "man" : ? → "woman" (delta trên V dimension)
```

### ⑥ Curiosity — Hướng về cái mới

```
novelty(P) → Num:
  nearest = min(distance(P, known_node) for known_node in recent_STM)
  return 1.0 - min(nearest / 2.236, 1.0)
  // novelty > 0.5 → ưu tiên explore (hỏi thêm, tìm hiểu)
  // novelty < 0.3 → familiar, đã biết
```

### ⑦ Reflection — Tự đánh giá

```
self_quality() → Num:
  qr_ratio = QR_count / (STM_count + QR_count + 1)
  avg_silk = Σ silk_weights / silk_count
  connectivity = avg_silk × silk_count / (node_count + 1)

  return 0.6 × qr_ratio + 0.4 × connectivity
  // quality cao → hệ thống học tốt, tri thức vững
  // quality thấp → cần học thêm, tri thức rời rạc
```

---

## §9. ConversationCurve — Đường Cong Cảm Xúc

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §5.1-5.2](docs/HomeOS_SPEC_v3.md)

### Công thức

```
f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)

f_conv(t) = V(t) + 0.5 × V'(t) + 0.25 × V''(t)
  V(t)   = Valence hiện tại (−1.0 .. +1.0)
  V'(t)  = (V(t) − V(t−1)) / Δt          // tốc độ thay đổi
  V''(t) = (V'(t) − V'(t−1)) / Δt        // gia tốc

f_dn(nodes) = Σ (nodeᵢ.V × nodeᵢ.recency)
  recency = φ⁻¹^(turns_ago)               // gần đây → trọng số cao
```

### Chọn tone từ ĐẠO HÀM (không từ V hiện tại)

```
V' < −0.15                → Supportive    // đang giảm → đồng cảm, đỡ
V'' < −0.25               → Pause         // rơi nhanh → dừng, không nói thêm
V' > +0.15                → Reinforcing   // đang hồi → khích lệ
V'' > +0.25 AND V > 0     → Celebratory   // bước ngoặt tốt → mừng
V < −0.20, ổn định        → Gentle        // buồn ổn định → dịu dàng
otherwise                 → Engaged       // bình thường

GIỚI HẠN: ΔV_max = 0.40/bước (không nhảy đột ngột — sinh lý không cho phép)

Ví dụ:
  Turn 1: "ok"          V=0.0
  Turn 2: "hơi mệt"    V=−0.20, V'=−0.20
  Turn 3: "buồn quá"   V=−0.50, V'=−0.30, V''=−0.10
    → V' < −0.15 → Supportive ("Mình hiểu cảm giác đó")

  Turn 4: "nhưng mà..." V=−0.35, V'=+0.15
    → V' > +0.15 → Reinforcing ("Đúng rồi, tiếp đi!")
```

---

## §10. SecurityGate — 3 Layers

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §5.4](docs/HomeOS_SPEC_v3.md)

```
SecurityGate.check(input) chạy TRƯỚC MỌI THỨ trong pipeline.
Crisis → DỪNG NGAY. Không qua Encode, không qua Instinct. CHẶN.

Layer 1 — Exact match O(1):
  Bloom filter (200 KB, false positive < 1%)
  keyword ∈ {"tự tử", "muốn chết", "tự hại", ...}
  Nếu match → Crisis

Layer 2 — Normalized match O(n):
  Chuẩn hóa: bỏ dấu, bỏ ký tự đặc biệt, lowercase
  "ch.ế.t" → "chet", "T.ự T.ử" → "tutu"
  Bắt evasion attempts.

Layer 3 — Semantic check O(depth):
  encode(input) → P_weight
  Nếu V < 1 (rất tiêu cực) AND A > 6 (rất kích động):
  → Potential crisis → escalate

Bất kỳ layer nào trigger → response khẩn cấp:
  "Nếu bạn đang cần hỗ trợ, xin gọi 1800 599 920 (Việt Nam)"
  Pipeline DỪNG. Không analyze thêm.

AlertLevel:
  Normal (○)     → tiếp tục pipeline
  Important (⚠)  → log để AAM review
  RedAlert (🔴)  → CHẶN ngay + AAM notification
```

---

---

# TÁN — HÀNH VI THÔNG MINH

---

## §11. Agent Hierarchy

> **Nguồn:** [`docs/ORIGIN_VISION.md` §IV](docs/ORIGIN_VISION.md), [`docs/HomeOS_SPEC_v3.md` §XIII](docs/HomeOS_SPEC_v3.md)

```
AAM [tier 0] — ý thức. Im lặng. Chỉ approve/reject.
  │             Stateless. Không viết code. Không chạy logic.
  │             Chỉ ký (Ed25519) hoặc từ chối.
  │
  ├── LeoAI [tier 1] — não: Learn + Dream + Curate + Response
  │   Skills:
  │     Nhận:    IngestSkill (encode input → chain)
  │     Hiểu:    ClusterSkill, SimilaritySkill, DeltaSkill
  │     Sắp xếp: CuratorSkill, MergeSkill, PruneSkill
  │     Học:     HebbianSkill, DreamSkill
  │     Đề xuất: ProposalSkill, HonestySkill
  │     Respond: compose_response(emotion, intent, tone, instincts, context)
  │
  ├── HomeChief — quản lý thiết bị nhà (đèn, HVAC, cửa)
  ├── VisionChief — quản lý camera, motion detect
  └── NetworkChief — quản lý mạng, bảo mật
        │
        └── Workers [tier 2] — tế bào tại thiết bị
            Silent. Wake on ISL message. Process → sleep.
            Mỗi worker = HomeOS thu nhỏ (VM + minimal bytecode)
```

---

## §12. Pipeline Hoàn Chỉnh — 14 Cơ Chế DNA

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §XIII](docs/HomeOS_SPEC_v3.md)

```
Input (text / audio / image / sensor)
  │
  ↓ ⑨ SecurityGate (Innate Immunity)     → Crisis? → CHẶN ngay
  ──── CHECKPOINT 1: GATE ────
  ↓ ⑩ Fusion (Multisensory)              → text+audio+bio+image merge
  ↓    entities() ③ Translate              → text → codepoints → P_weights
  ↓    search() ⑬ Neural Pathways         → KnowTree walk O(4)
  ↓ ⑫ Homeostasis: F = d(predicted, actual)  → đo surprise
  ↓    compose() ⑤ Recombine              → tổ hợp → điểm 5D mới
  ──── CHECKPOINT 2: ENCODE ────
  ↓ ⑧ Instincts (7 phản xạ)              → Honesty đầu tiên
  ↓ ⑪ Immune Selection: infer(N=3)       → 3 nhánh → chọn entropy thấp
  ↓ ⑭ DNA Repair: critique → refine      → sửa đến quality ≥ φ⁻¹
  ──── CHECKPOINT 3: INFER ────
  ↓ ⑥ Select: Hebbian co_activate        → fire together → wire together
  ↓    Dream ⑦ Express → advance() → QR  → neo vĩnh viễn nếu chín
  ──── CHECKPOINT 4: PROMOTE ────
  ↓    response = ② Transcribe(5D → text) → chiếu ngược ra ngôn ngữ
  ──── CHECKPOINT 5: RESPONSE ────
  ↓
Output (text + tone)
```

### Homeostasis — Free Energy

```
F(t) = √( Σ w_d × (predicted_d − actual_d)² )

  predicted = KnowTree lookup (tri thức đã có)
  actual = encode(input) (tri thức mới nhận)

  F > φ⁻¹ (0.618) → Learning mode (surprise cao → cần học)
    → tăng learning rate, Dream thường xuyên, giảm confidence
  F < φ⁻¹          → Acting mode (ổn định → tự tin trả lời)

λ(t) = σ(F(t) − φ⁻¹)     σ(x) = 1/(1+e^(−5x))
```

### Self-Correct — DNA Repair

```
self_correct(input, max_iter=3):
  ① Generate: infer(N=3) → P_response
  ② Critique:
     quality = 0.30 × valid
             + 0.30 × (1 − H/2.32)
             + 0.20 × consistency
             + 0.20 × silk/5.0
  ③ Nếu quality < φ⁻¹:
     → sửa DUY NHẤT chiều yếu nhất
     → nếu quality_new < quality_old → ROLLBACK
  ④ Lặp tối đa 3 lần → bounded (worst case = 9 evaluations)
```

---

## §13. 5 Checkpoints — Cell Cycle

> **Nguồn:** [`docs/HomeOS_SPEC_v3.md` §X](docs/HomeOS_SPEC_v3.md)

```
Sinh học: bỏ checkpoint = ung thư (tế bào phân chia không kiểm soát).
HomeOS: bỏ checkpoint = tri thức sai lan tràn = "ung thư tri thức".

CP1 GATE:     SecurityGate đã chạy. Crisis → pipeline DỪNG.
CP2 ENCODE:   |entities| ≥ 1, chain_hash ≠ 0, compose consistency ≥ 0.75
CP3 INFER:    ≥1 nhánh valid ≥ 0.75, H(best) < 2.32, quality rollback
CP4 PROMOTE:  weight ≥ φ⁻¹, fire ≥ Fib(n), eval_dims ≥ 3, H < 1.0
CP5 RESPONSE: SecurityGate.check(response) = Safe, tone phù hợp, confidence ≥ 0.40
```

---

---

# PHỤ LỤC

---

## §14. Hiện Trạng — Cái Gì Có, Cái Gì Chưa

```
                          THIẾT KẾ              THỰC TẾ            KHOẢNG CÁCH
────────────────────────────────────────────────────────────────────────────
L0 Gene (8,846 UDC)      mỗi cp = P_weight     a-z = cùng 1 mol   CẦN XÂY LẠI
                          duy nhất              (tất cả = 146)

Encode ∫                  UTF-8 → cp → mol      ASCII only,         CẦN XÂY LẠI
                          5D duy nhất           không dấu

KnowTree                  cây L0→L3, 18KB      flat array strings  CẦN XÂY LẠI
                          O(4) lookup           O(n) scan

Chain links               u16 → UDC index       text strings         CẦN XÂY LẠI
                          2 bytes/link          hàng chục bytes/link

Silk Hebbian              mol-keyed, decay φ⁻¹  string-keyed,       BÁN HOẠT ĐỘNG
                          emotion per edge       17 edges

STM                       node-based, evict      array of strings    CẦN XÂY LẠI
Dream                     cluster + LCA + QR     count intents       CẦN XÂY LẠI
QR append-only            signed, immutable      không có            CẦN XÂY

7 Instincts               7 Skills, stateless    7 if-else checks    CẦN XÂY LẠI
ConversationCurve         f, f', f''             integer 0-7         CẦN XÂY LẠI
SecurityGate              3 layers, Bloom        hardcode keywords   BÁN HOẠT ĐỘNG
Response                  4-part composer        10 câu hardcode     CẦN XÂY LẠI

Agents                    AAM→LeoAI→Workers      agent_respond()     CẦN XÂY
ISL messaging             AES-256-GCM            không có            CẦN XÂY

VM + Compiler             self-hosting           1,021KB binary      ✅ HOẠT ĐỘNG
                          fib(20)=6765           20/20 tests         ✅ VỮNG
SHA-256                   ASM builtin            verified FIPS       ✅ HOẠT ĐỘNG
```

---

## §15. Thứ Tự Xây — Rễ Trước, Tán Sau

```
NGUYÊN TẮC: Mỗi tầng CHỈ phụ thuộc tầng dưới.
            Xây từ dưới lên. Không nhảy tầng. Không xây tán khi chưa có rễ.

Phase 1: RỄ — L0 Gene (~300 LOC Olang)
  ├── 1a. Bảng P_weight: 128 ASCII + 8,846 UDC = bảng lookup
  │        Input: json/udc_p_table.bin (đã có)
  │        Output: fn encode_codepoint(cp) → u16 DUY NHẤT
  │        Test: encode('a') ≠ encode('b') ≠ encode('z')
  │
  ├── 1b. Compose đúng: 5 chiều × 5 quy tắc
  │        Input: 2 P_weights
  │        Output: 1 P_weight tổng hợp (amplify, KHÔNG trung bình)
  │        Test: compose("buồn", "rất") → V thấp hơn "buồn" đơn lẻ
  │
  └── 1c. Distance 5D: √(Σ (A_d - B_d)²)
           Test: distance("vui", "buồn") > distance("vui", "hạnh phúc")

Phase 2: THÂN — KnowTree + Chain (~400 LOC Olang)
  ├── 2a. KnowTree cấu trúc: L0(5) → L1(59) → L2(~200) → L3(8,846)
  │        Boot: load 8,846 entries từ udc_p_table.bin
  │        Lookup: O(4) — 4 bước từ root đến lá
  │
  ├── 2b. Chain: mảng u16, mỗi link = UDC index
  │        encode_sentence("Hà Nội") → chain[cp_H, cp_à, cp_N, cp_ộ, cp_i]
  │        mỗi link → 1 P_weight DUY NHẤT
  │
  ├── 2c. Library (L4+): nhánh facts/books/conversations
  │        kt_learn("Hà Nội là thủ đô Việt Nam") → chain → L4:facts:geography
  │
  └── 2d. Search: walk tree O(depth)
           kt_search("Hà Nội") → walk L4 → tìm chains chứa "Hà Nội"

Phase 3: CÀNH — Neuron Model (~300 LOC Olang)
  ├── 3a. Silk Hebbian: co_activate(mol_A, mol_B, emotion)
  │        Decay: w × φ⁻¹^(Δt/24h)
  │
  ├── 3b. STM: node-based, evict oldest khi > 32
  │
  ├── 3c. Dream: cluster STM → LCA → propose QR
  │        Trigger: Fibonacci (fire ≥ 2, 3, 5, 8, 13...)
  │
  └── 3d. QR: append-only records, signed

Phase 4: TÁN — Intelligence (~400 LOC Olang)
  ├── 4a. 7 Instincts: Honesty → Contradiction → ... → Reflection
  ├── 4b. ConversationCurve: V(t), V'(t), V''(t), tone selection
  ├── 4c. SecurityGate: 3 layers
  ├── 4d. Response composer: 4-part, context-aware
  └── 4e. Pipeline hoàn chỉnh: 14 cơ chế, 5 checkpoints

Phase 5: XÃ HỘI — Agents + ISL (sau khi Phase 1-4 vững)
  ├── 5a. AAM: stateless approve/reject
  ├── 5b. LeoAI: orchestrate Skills
  ├── 5c. ISL: inter-device messaging
  └── 5d. Workers: HomeOS thu nhỏ
```

---

## §16. Tham Chiếu — Link Đến Tài Liệu Gốc

```
SPEC + VISION:
  docs/HomeOS_SPEC_v3.md              Spec chính v3.1 (957 dòng)
  docs/ORIGIN_VISION.md               Bức tranh tổng thể (477 dòng)
  old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md   Spec gốc v2.7 (1729 dòng)

UDC (Unicode → SDF):
  docs/UDC_DOC/UDC_S1_GEOMETRIC_tree.md   Shape primitives
  docs/UDC_DOC/UDC_R_RELATION_tree.md     Relation categories
  docs/UDC_DOC/UDC_V_VALENCE_tree.md      Valence spectrum
  docs/UDC_DOC/UDC_A_AROUSAL_tree.md      Arousal dynamics
  docs/UDC_DOC/UDC_T_TIME_tree.md         Time/rhythm
  docs/UDC_DOC/UDC_real_formulas.md       Công thức toán
  docs/UDC_DOC/UDC_map.md                 Bản đồ 8,846 ký tự

KIẾN TRÚC:
  docs/KNOWTREE_DESIGN.md             KnowTree fractal design
  docs/STORAGE_AND_SEARCH_NOTE.md     Storage + search
  PLAN_REWRITE.md                     Lộ trình self-hosting (hoàn thành)
  plans/MASTER_PLAN_HOMEOS_V1.md      Master plan HomeOS v1

LỊCH SỬ:
  docs/MILESTONE_20260323.md          Self-hosting milestone
  crates/EPITAPH.md                   Lời mặc niệm Rust
  old/MEMORIAL.md                     Tài liệu lịch sử
  SORA.md                             Review + phát hiện kỹ thuật

CÓ SẴN (hoạt động):
  vm/x86_64/vm_x86_64.S              ASM VM (5,987 LOC)
  stdlib/bootstrap/                   Compiler tự hosting (4 files)
  stdlib/homeos/                      46 file .ol (cần xây lại phần lớn)
  json/udc_p_table.bin                Bảng P_weight 8,846 entries (308 KB)
  json/udc.json                       UDC data đầy đủ (7.6 MB)

φ⁻¹ = (√5 − 1) / 2 ≈ 0.618 — hằng số duy nhất xuyên suốt toàn hệ thống.
```

---

## §17. Hằng Số và Ngưỡng

```
φ⁻¹ = 0.618033988749895   — Golden ratio inverse

Dùng ở:
  Maturity:       weight ≥ φ⁻¹ → Mature
  Hebbian decay:  w × φ⁻¹ mỗi 24h
  Homeostasis:    F < φ⁻¹ → Acting mode
  Self-correct:   quality ≥ φ⁻¹ → dừng sửa
  Consistency:    ≥ 0.75 ≈ φ⁻¹ + 0.13

Fibonacci sequence (trigger Dream):
  Fib(1)=1, Fib(2)=1, Fib(3)=2, Fib(4)=3, Fib(5)=5,
  Fib(6)=8, Fib(7)=13, Fib(8)=21, Fib(9)=34, Fib(10)=55

Entropy max:
  H_max = log₂(5) ≈ 2.322 (5 chiều, uniform)

1 hằng số (φ⁻¹). 1 chuỗi (Fibonacci). 1 giới hạn (H_max).
Mọi ngưỡng đều derive từ 3 thứ này.
```

---

## PHƯƠNG TRÌNH THỐNG NHẤT

```
╔═════════════════════════════════════════════════════════════╗
║                                                             ║
║   HomeOS(input) = self_correct(                             ║
║                     splice(                                 ║
║                       chain(                                ║
║                         f(p₁), f(p₂), ..., f(pₙ)           ║
║                       ),                                    ║
║                       position,                             ║
║                       context                               ║
║                     ),                                      ║
║                     φ⁻¹                                     ║
║                   )                                         ║
║                                                             ║
║   f(pᵢ)       = SDF — 1 trong 8,846 hàm gốc               ║
║   chain       = xâu chuỗi → 2 bytes/link (u16)             ║
║   splice      = cắt/ghép chuỗi                             ║
║   self_correct = lặp đến quality ≥ φ⁻¹                     ║
║                                                             ║
║   DNA:     nucleotide + polymerize + splice = sự sống       ║
║   HomeOS:  SDF + chain + splice + φ⁻¹ = tri thức           ║
║                                                             ║
║   4 thứ. Hết.                                               ║
║                                                             ║
╚═════════════════════════════════════════════════════════════╝
```

---

*Tài liệu này là BẢN DUY NHẤT. Mọi plan khác là tham khảo.*
*Xây từ rễ. Không nhảy tầng. Mỗi tầng test xong mới lên tầng tiếp.*
*2026-03-25 · goldlotus1810 + Claude*
