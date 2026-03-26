# S.0 — MŨI TÊN (Arrow) · Cây phân loại bằng từ ngữ

> Nhìn vào từ khóa → biết ngay "thèn này thuộc nhóm nào"
> Mỗi tầng = 1 câu hỏi. Trả lời = chọn nhánh.

---

## Mô hình vật lý tổng quát

> **Mũi tên Unicode = vector trong trường vector trên mặt phẳng.**
> Toàn bộ S.0 ARROW được mô hình hóa bằng hình học vi phân (differential geometry):
> mỗi mũi tên là một vector thuộc trường vector F⃗: R² → R².

```
Mũi tên = vector trong trường vector F⃗: R² → R²

Mỗi mũi tên Unicode = một vector được xác định bởi 5 tham số:
  v⃗ = (type, direction, weight, fill, tail)

  Tương đương vật lý:
    direction = góc θ ∈ [0°, 360°) — hướng trong R²
    weight = ||v⃗|| = chuẩn (norm) — độ lớn vector
    type = loại trường: gradient (∇f), curl (∇×F), divergence (∇·F)
    fill = mật độ ρ(x) — đặc/rỗng
    tail = điều kiện biên (boundary condition)

  Tuple đầy đủ:
    v⃗ = (field_type, θ, ||v⃗||, ρ, BC)

  Trong đó:
    field_type ∈ {∇f, ∇×F, ∇·F, piecewise, sinusoidal, ...}
    θ ∈ {0°, 45°, 90°, 135°, 180°, 225°, 270°, 315°, bidirectional}
    ||v⃗|| ∈ {ε, 1, M}   (infinitesimal, unit, large)
    ρ ∈ {1, δ(|x|−r), H(x)}   (uniform, boundary-only, half)
    BC ∈ {linear, geodesic, C⁰, attractor}
```

---

## Tầng 1: "Nó là kiểu mũi tên gì?"

```
MŨI TÊN
├── ĐƠN         "arrow" — 1 đầu nhọn
├── ĐÔI          "double arrow" — 2 đầu nhọn ⇔
├── BA            "triple arrow" — 3 nét ⇛
├── MÓC           "harpoon" — nửa đầu nhọn ⇀
├── GẠCH NGANG   "dashed arrow" — nét đứt ⇢
├── LƯỢN SÓNG    "wave/squiggle arrow" — nét lượn ↝
├── VÒNG          "circular/anticlockwise" — xoay vòng ↺
└── ĐẶC BIỆT     "bent/hook/loop/curved" — gấp khúc ↩
```

### Giải nghĩa vật lý — Tầng 1

| Kiểu | Mô hình toán/vật lý | Giải thích |
|------|---------------------|------------|
| ĐƠN | Trường gradient ∇f | 1 hướng duy nhất, irrotational (không xoáy). Dòng chảy từ cao → thấp. |
| ĐÔI | Diffeomorphism f: M → N, f⁻¹ tồn tại | Ánh xạ song ánh — đi được cả 2 chiều (⇔). |
| BA | Tích ba vector a⃗ × (b⃗ × c⃗) | Ba thành phần tương tác, kết quả nằm trong mặt phẳng (b⃗, c⃗). |
| MÓC (harpoon) | Nửa đạo hàm, giới hạn một phía: lim(x→a⁺) | Chỉ "kéo" một nửa — như đạo hàm chỉ xét 1 phía. |
| GẠCH NGANG | Xấp xỉ rời rạc (finite difference): Δf/Δx ≈ f'(x) | Nét đứt = rời rạc hóa, không liên tục. |
| LƯỢN SÓNG | Trường hình sin: A·sin(kx − ωt) | Dao động điều hòa — sóng lan truyền. |
| VÒNG | Trường curl ∇ × F⃗ | Xoáy (rotational field) — dòng chảy xoắn quanh tâm. |
| ĐẶC BIỆT | Hàm từng khúc (piecewise) với điểm gián đoạn | Gấp khúc = đổi hướng đột ngột, không khả vi tại điểm gấp. |

---

## Tầng 2: "Nó chỉ hướng nào?"

```
(áp dụng cho MỌI kiểu ở tầng 1)

HƯỚNG
├── LÊN            "upwards" ↑
├── XUỐNG           "downwards" ↓
├── TRÁI            "leftwards" ←
├── PHẢI            "rightwards" →
├── LÊN-PHẢI        "upper right / north east" ↗
├── LÊN-TRÁI        "upper left / north west" ↖
├── XUỐNG-PHẢI       "lower right / south east" ↘
├── XUỐNG-TRÁI       "lower left / south west" ↙
├── TRÁI-PHẢI        "left right" ↔  (2 chiều ngang)
├── LÊN-XUỐNG        "up down" ↕     (2 chiều dọc)
└── TẤT CẢ           "four directions" — 4 hướng
```

### Giải nghĩa vật lý — Tầng 2

> Mỗi hướng = 1 vector đơn vị ê (unit vector) trong R².

| Hướng | Vector đơn vị ê | Góc θ | Ý nghĩa |
|-------|-----------------|-------|---------|
| PHẢI | ê = (1, 0) | θ = 0° | Trục x dương |
| LÊN | ê = (0, 1) | θ = 90° | Trục y dương |
| TRÁI | ê = (−1, 0) | θ = 180° | Trục x âm |
| XUỐNG | ê = (0, −1) | θ = 270° | Trục y âm |
| LÊN-PHẢI | ê = (1, 1)/√2 | θ = 45° | Chéo phần tư I |
| LÊN-TRÁI | ê = (−1, 1)/√2 | θ = 135° | Chéo phần tư II |
| XUỐNG-PHẢI | ê = (1, −1)/√2 | θ = 315° | Chéo phần tư IV |
| XUỐNG-TRÁI | ê = (−1, −1)/√2 | θ = 225° | Chéo phần tư III |
| TRÁI-PHẢI | ê₁ + ê₂ = {(−1,0), (1,0)} | θ ∈ {0°, 180°} | Cơ sở (basis) trải dài 1D ngang |
| LÊN-XUỐNG | ê₁ + ê₂ = {(0,1), (0,−1)} | θ ∈ {90°, 270°} | Cơ sở trải dài 1D dọc |
| TẤT CẢ | span{ê₁, ê₂, ê₃, ê₄} | toàn bộ | Trải đều R² — trường đẳng hướng (isotropic) |

---

## Tầng 3: "Nét nó dày mỏng ra sao?"

```
ĐỘ DÀY (weight)
├── MỎNG         "light" — nét mảnh
├── THƯỜNG        (không ghi gì) — nét bình thường
├── DÀY           "heavy" — nét đậm
└── RẤT DÀY       "very heavy / bold" — cực đậm
```

### Giải nghĩa vật lý — Tầng 3

> Độ dày nét = chuẩn (norm) ||v⃗|| của vector — tức độ lớn/cường độ.

| Độ dày | Chuẩn ||v⃗|| | Ý nghĩa vật lý |
|--------|------------|----------------|
| MỎNG | \|\|v⃗\|\| = ε (infinitesimal, ε → 0⁺) | Nhiễu loạn nhỏ, xấp xỉ tuyến tính. Như vi phân df = f'(x)·dx. |
| THƯỜNG | \|\|v⃗\|\| = 1 (unit vector) | Vector chuẩn hóa — chỉ mang thông tin hướng, không mang cường độ. |
| DÀY | \|\|v⃗\|\| = M (M >> 1) | Cường độ trường lớn — lực mạnh, gradient dốc. |
| RẤT DÀY | \|\|v⃗\|\| = M² | Cường độ cực đại — vùng kỳ dị (singularity) hoặc nguồn/hút mạnh. |

---

## Tầng 4: "Nó đặc hay rỗng?"

```
KIỂU TÔ (fill)
├── ĐẶC          "filled / black" — tô kín ▶
├── RỖNG          "open / white" — chỉ viền ▷
├── NỬA           "half-filled" — tô nửa
└── BÓNG          "shadowed" — có bóng đổ
```

### Giải nghĩa vật lý — Tầng 4

> Kiểu tô = hàm mật độ ρ(x) — phân bố vật chất/năng lượng bên trong vector.

| Kiểu tô | Hàm mật độ ρ(x) | Ý nghĩa vật lý |
|---------|-----------------|----------------|
| ĐẶC | ρ(x) = 1 (uniform density) | Phân bố đều — vật rắn đặc, trường đều khắp nơi. |
| RỖNG | ρ(x) = δ(\|x\| − r) (Dirac delta trên bề mặt) | Chỉ có trên biên — vỏ rỗng, mật độ bề mặt. Như vỏ cầu tích điện. |
| NỬA | ρ(x) = H(x) (hàm Heaviside) | Nửa đặc nửa rỗng — mật độ nhảy bậc tại x=0. Ranh giới pha. |
| BÓNG | ρ(x) = e^(−αx²) (Gaussian decay) | Mật độ giảm dần — bóng = suy giảm theo khoảng cách (attenuation). |

---

## Tầng 5: "Nó có đuôi gì?"

```
ĐUÔI (tail)
├── KHÔNG ĐUÔI    "arrowhead only" — chỉ mũi nhọn ➤
├── ĐUÔI THẲNG    (mặc định) — thân thẳng →
├── ĐUÔI CONG     "curved" — thân cong ⤵
├── ĐUÔI GẤP      "bent" — gấp khúc ↳
└── ĐUÔI MÓC      "hook / barb" — có móc ↩
```

### Giải nghĩa vật lý — Tầng 5

> Đuôi = quỹ đạo (trajectory) / điều kiện biên (boundary condition) của vector.

| Đuôi | Mô hình toán | Ý nghĩa vật lý |
|------|-------------|----------------|
| KHÔNG ĐUÔI | v⃗ tại điểm (point vector) | Vector tự do — chỉ có hướng và độ lớn, không có gốc cố định. |
| ĐUÔI THẲNG | x(t) = x₀ + v⃗·t (quỹ đạo tuyến tính) | Chuyển động thẳng đều — không có lực tác dụng (Newton I). |
| ĐUÔI CONG | Geodesic trên mặt cong: ∇_γ̇ γ̇ = 0 | Đường trắc địa — đường ngắn nhất trên không gian cong (GR). |
| ĐUÔI GẤP | Tuyến tính từng khúc: C⁰ nhưng không C¹ | Liên tục nhưng không khả vi — đổi hướng đột ngột (va chạm, phản xạ). |
| ĐUÔI MÓC | Quỹ đạo có điểm bất động (attractor): lim(t→∞) x(t) = x* | Hệ hội tụ về điểm hút — ổn định Lyapunov. |

---

## Ví dụ đọc: Từ tên → biết nhóm

```
"RIGHTWARDS ARROW"
  Tầng 1: ĐƠN (arrow, không double/triple)
  Tầng 2: PHẢI (rightwards)
  Tầng 3: THƯỜNG (không light/heavy)
  Tầng 4: — (không filled/open)
  Tầng 5: ĐUÔI THẲNG (mặc định)
  → Nhìn vào biết: "mũi tên đơn, hướng phải, nét thường"

"HEAVY WIDE-HEADED SOUTH EAST ARROW"
  Tầng 1: ĐƠN
  Tầng 2: XUỐNG-PHẢI (south east)
  Tầng 3: DÀY (heavy)
  Tầng 4: — (wide-headed = đầu to, modifier phụ)
  Tầng 5: ĐUÔI THẲNG
  → "mũi tên đơn, xuống-phải, nét dày, đầu to"

"LEFT RIGHT OPEN-HEADED ARROW"
  Tầng 1: ĐƠN
  Tầng 2: TRÁI-PHẢI (left right)
  Tầng 3: THƯỜNG
  Tầng 4: RỖNG (open-headed)
  Tầng 5: ĐUÔI THẲNG
  → "mũi tên đơn, 2 chiều ngang, rỗng"

"ANTICLOCKWISE TOP SEMICIRCLE ARROW"
  Tầng 1: VÒNG (anticlockwise)
  Tầng 2: LÊN (top)
  Tầng 3: THƯỜNG
  Tầng 4: —
  Tầng 5: — (semicircle = hình dạng thân)
  → "mũi tên vòng ngược chiều kim đồng hồ, nửa trên"

"LEFTWARDS HARPOON WITH BARB DOWNWARDS"
  Tầng 1: MÓC (harpoon)
  Tầng 2: TRÁI (leftwards)
  Tầng 3: THƯỜNG
  Tầng 4: —
  Tầng 5: barb downwards = ngạnh hướng xuống (modifier phụ)
  → "mũi tên móc, hướng trái, ngạnh dưới"
```

---

## Tổng: 5 tầng × số nhánh

```
Tầng 1 — KIỂU:     8 nhánh
Tầng 2 — HƯỚNG:    11 nhánh
Tầng 3 — ĐỘ DÀY:   4 nhánh
Tầng 4 — TÔ:        4 nhánh
Tầng 5 — ĐUÔI:      5 nhánh

Tổ hợp lý thuyết: 8 × 11 × 4 × 4 × 5 = 7,040 khả năng
Unicode thực tế:   ~618 ký tự mũi tên (không dùng hết tổ hợp)

Mỗi ký tự mũi tên = 1 tuple (kiểu, hướng, độ_dày, tô, đuôi)
```

---

## Từ khóa → Tầng (cheat sheet)

| Thấy từ này trong tên | → Thuộc tầng | → Giá trị |
|----------------------|-------------|----------|
| arrow | 1 | ĐƠN |
| double | 1 | ĐÔI |
| triple | 1 | BA |
| harpoon | 1 | MÓC |
| dashed | 1 | GẠCH NGANG |
| wave, squiggle | 1 | LƯỢN SÓNG |
| clockwise, anticlockwise | 1 | VÒNG |
| bent, hook, loop, curved | 1 | ĐẶC BIỆT |
| upwards, up, north | 2 | LÊN |
| downwards, down, south | 2 | XUỐNG |
| leftwards, left, west | 2 | TRÁI |
| rightwards, right, east | 2 | PHẢI |
| upper right, north east | 2 | LÊN-PHẢI |
| upper left, north west | 2 | LÊN-TRÁI |
| lower right, south east | 2 | XUỐNG-PHẢI |
| lower left, south west | 2 | XUỐNG-TRÁI |
| left right | 2 | TRÁI-PHẢI |
| up down | 2 | LÊN-XUỐNG |
| light | 3 | MỎNG |
| heavy, bold | 3 | DÀY |
| very heavy | 3 | RẤT DÀY |
| filled, black | 4 | ĐẶC |
| open, white | 4 | RỖNG |
| half | 4 | NỬA |
| shadowed | 4 | BÓNG |
| barb | 5 | MÓC |
| curved | 5 | CONG |
| bent | 5 | GẤP |

---

## Phân loại 618 cụm cụ thể vào cây

### Tầng 1 = KIỂU → Tầng 2 = HƯỚNG

#### ĐƠN (arrow) — ~350 cụm

**ĐƠN → PHẢI (rightwards):**
```
RIGHTWARDS ARROW
RIGHTWARDS ARROW TO BAR
RIGHTWARDS ARROW WITH CORNER DOWNWARDS
RIGHTWARDS ARROW WITH DOTTED STEM
RIGHTWARDS ARROW WITH DOUBLE VERTICAL STROKE
RIGHTWARDS ARROW WITH HOOK
RIGHTWARDS ARROW WITH LOOP
RIGHTWARDS ARROW WITH PLUS BELOW
RIGHTWARDS ARROW WITH SMALL CIRCLE
RIGHTWARDS ARROW WITH STROKE
RIGHTWARDS ARROW WITH TAIL
RIGHTWARDS ARROW WITH TIP DOWNWARDS
RIGHTWARDS ARROW WITH TIP UPWARDS
RIGHTWARDS ARROW-TAIL
RIGHTWARDS SQUIGGLE ARROW
RIGHTWARDS TWO-HEADED ARROW
RIGHTWARDS TWO-HEADED ARROW WITH TAIL
RIGHTWARDS TWO-HEADED ARROW WITH TRIANGLE ARROWHEADS
RIGHTWARDS WHITE ARROW
RIGHTWARDS WHITE ARROW FROM WALL
DRAFTING POINT RIGHTWARDS ARROW
LONG RIGHTWARDS ARROW
LONG RIGHTWARDS SQUIGGLE ARROW
→ "ah, mũi tên đơn, hướng phải" — nhìn RIGHTWARDS là biết
```

**ĐƠN → TRÁI (leftwards):**
```
LEFTWARDS ARROW
LEFTWARDS ARROW FROM BAR
LEFTWARDS ARROW OVER RIGHTWARDS ARROW
LEFTWARDS ARROW TO BAR
LEFTWARDS ARROW TO BAR OVER RIGHTWARDS ARROW TO BAR
LEFTWARDS ARROW WITH HOOK
LEFTWARDS ARROW WITH LOOP
LEFTWARDS ARROW WITH STROKE
LEFTWARDS ARROW WITH TAIL
LEFTWARDS ARROW-TAIL
LEFTWARDS SQUIGGLE ARROW
LEFTWARDS TWO-HEADED ARROW
LEFTWARDS WHITE ARROW
LONG LEFTWARDS ARROW
→ "ah, mũi tên đơn, hướng trái"
```

**ĐƠN → LÊN (upwards):**
```
UPWARDS ARROW
UPWARDS ARROW FROM BAR
UPWARDS ARROW TO BAR
UPWARDS ARROW WITH DOUBLE STROKE
UPWARDS ARROW WITH HORIZONTAL STROKE
UPWARDS ARROW WITH TIP LEFTWARDS
UPWARDS ARROW WITH TIP RIGHTWARDS
UPWARDS WHITE ARROW
UPWARDS WHITE ARROW FROM BAR
UPWARDS WHITE ARROW ON PEDESTAL
UPWARDS WHITE DOUBLE ARROW
UPWARDS WHITE DOUBLE ARROW ON PEDESTAL
→ "ah, mũi tên đơn, hướng lên"
```

**ĐƠN → XUỐNG (downwards):**
```
DOWNWARDS ARROW
DOWNWARDS ARROW FROM BAR
DOWNWARDS ARROW WITH CORNER LEFTWARDS
DOWNWARDS ARROW WITH HORIZONTAL STROKE
DOWNWARDS ARROW WITH SMALL EQUILATERAL ARROWHEAD
DOWNWARDS ARROW WITH TIP LEFTWARDS
DOWNWARDS ARROW WITH TIP RIGHTWARDS
DOWNWARDS WHITE ARROW
→ "ah, mũi tên đơn, hướng xuống"
```

**ĐƠN → LÊN-PHẢI (north east):**
```
NORTH EAST ARROW
NORTH EAST ARROW WITH HOOK
NORTH EAST AND SOUTH WEST ARROW
→ "ah, mũi tên đơn, chéo lên-phải"
```

**ĐƠN → LÊN-TRÁI (north west):**
```
NORTH WEST ARROW
NORTH WEST ARROW TO CORNER
NORTH WEST AND SOUTH EAST ARROW
→ "ah, mũi tên đơn, chéo lên-trái"
```

**ĐƠN → XUỐNG-PHẢI (south east):**
```
SOUTH EAST ARROW
SOUTH EAST ARROW WITH HOOK
→ "ah, mũi tên đơn, chéo xuống-phải"
```

**ĐƠN → XUỐNG-TRÁI (south west):**
```
SOUTH WEST ARROW
→ "ah, mũi tên đơn, chéo xuống-trái"
```

**ĐƠN → TRÁI-PHẢI (hai chiều ngang):**
```
LEFT RIGHT ARROW
LEFT RIGHT ARROW WITH STROKE
LEFT RIGHT DOUBLE ARROW
LEFT RIGHT DOUBLE ARROW WITH STROKE
LEFT RIGHT OPEN-HEADED ARROW
LEFT RIGHT WAVE ARROW
LEFT RIGHT WHITE ARROW
→ "ah, mũi tên đơn, 2 chiều ngang"
```

**ĐƠN → LÊN-XUỐNG (hai chiều dọc):**
```
UP DOWN ARROW
UP DOWN ARROW WITH BASE
UP DOWN DOUBLE ARROW
UP DOWN WHITE ARROW
→ "ah, mũi tên đơn, 2 chiều dọc"
```

#### ĐÔI (double) — ~30 cụm

```
LEFTWARDS DOUBLE ARROW
RIGHTWARDS DOUBLE ARROW
UPWARDS DOUBLE ARROW
DOWNWARDS DOUBLE ARROW
LEFT RIGHT DOUBLE ARROW
UP DOWN DOUBLE ARROW
LEFTWARDS PAIRED ARROWS
RIGHTWARDS PAIRED ARROWS
UPWARDS PAIRED ARROWS
DOWNWARDS PAIRED ARROWS
LEFTWARDS DOUBLE ARROW WITH STROKE
→ "ah, mũi tên đôi" — thấy DOUBLE hoặc PAIRED
```

#### BA (triple) — ~8 cụm

```
LEFTWARDS TRIPLE ARROW
RIGHTWARDS TRIPLE ARROW
THREE LEFTWARDS ARROWS
THREE RIGHTWARDS ARROWS
→ "ah, mũi tên ba" — thấy TRIPLE hoặc THREE
```

#### MÓC (harpoon) — ~20 cụm

```
LEFTWARDS HARPOON WITH BARB DOWNWARDS
LEFTWARDS HARPOON WITH BARB UPWARDS
RIGHTWARDS HARPOON WITH BARB DOWNWARDS
RIGHTWARDS HARPOON WITH BARB UPWARDS
UPWARDS HARPOON WITH BARB LEFTWARDS
UPWARDS HARPOON WITH BARB RIGHTWARDS
DOWNWARDS HARPOON WITH BARB LEFTWARDS
DOWNWARDS HARPOON WITH BARB RIGHTWARDS
LEFTWARDS HARPOON OVER RIGHTWARDS HARPOON
RIGHTWARDS HARPOON OVER LEFTWARDS HARPOON
UPWARDS HARPOON ON LEFT SIDE
DOWNWARDS HARPOON ON LEFT SIDE
LEFT BARB UP RIGHT BARB UP HARPOON
LEFT BARB DOWN RIGHT BARB DOWN HARPOON
→ "ah, mũi tên móc" — thấy HARPOON là biết
  Tầng phụ: BARB UP/DOWN/LEFT/RIGHT = ngạnh hướng nào
```

#### VÒNG (circular/clockwise) — ~25 cụm

```
ANTICLOCKWISE CLOSED CIRCLE ARROW
ANTICLOCKWISE GAPPED CIRCLE ARROW
ANTICLOCKWISE OPEN CIRCLE ARROW
ANTICLOCKWISE TOP SEMICIRCLE ARROW
CLOCKWISE CLOSED CIRCLE ARROW
CLOCKWISE GAPPED CIRCLE ARROW
CLOCKWISE OPEN CIRCLE ARROW
CLOCKWISE TOP SEMICIRCLE ARROW
CLOCKWISE RIGHTWARDS AND LEFTWARDS OPEN CIRCLE ARROWS
ANTICLOCKWISE TRIANGLE-HEADED OPEN CIRCLE ARROW
ANTICLOCKWISE TRIANGLE-HEADED TOP U-SHAPED ARROW
ANTICLOCKWISE TRIANGLE-HEADED BOTTOM U-SHAPED ARROW
ANTICLOCKWISE TRIANGLE-HEADED LEFT U-SHAPED ARROW
ANTICLOCKWISE TRIANGLE-HEADED RIGHT U-SHAPED ARROW
→ "ah, mũi tên vòng" — thấy CLOCKWISE/ANTICLOCKWISE
  Tầng phụ: CIRCLE/SEMICIRCLE/U-SHAPED = hình vòng kiểu gì
```

#### GẤP KHÚC (bent/curved/hook) — ~30 cụm

```
ARROW POINTING RIGHTWARDS THEN CURVING DOWNWARDS
ARROW POINTING RIGHTWARDS THEN CURVING UPWARDS
ARROW POINTING DOWNWARDS THEN CURVING LEFTWARDS
ARROW POINTING DOWNWARDS THEN CURVING RIGHTWARDS
RIGHTWARDS ARROW WITH CORNER DOWNWARDS
DOWNWARDS ARROW WITH CORNER LEFTWARDS
RIGHTWARDS ARROW WITH HOOK
LEFTWARDS ARROW WITH HOOK
RIGHTWARDS ARROW WITH LOOP
LEFTWARDS ARROW WITH LOOP
→ "ah, mũi tên gấp khúc" — thấy CURVING/CORNER/HOOK/LOOP
```

#### DẸT/TAM GIÁC (triangle-headed) — ~80 cụm

```
TRIANGLE-HEADED RIGHTWARDS ARROW
TRIANGLE-HEADED LEFTWARDS ARROW
TRIANGLE-HEADED UPWARDS ARROW
TRIANGLE-HEADED DOWNWARDS ARROW
HEAVY TRIANGLE-HEADED RIGHTWARDS ARROW
LIGHT TRIANGLE-HEADED RIGHTWARDS ARROW
→ "ah, mũi tên đầu tam giác" — thấy TRIANGLE-HEADED
  Đây là modifier: kiểu đầu (head shape), không phải kiểu thân
```

#### APL VANE (lá gió) — ~8 cụm

```
APL FUNCTIONAL SYMBOL DOWNWARDS VANE
APL FUNCTIONAL SYMBOL LEFTWARDS VANE
APL FUNCTIONAL SYMBOL RIGHTWARDS VANE
APL FUNCTIONAL SYMBOL UPWARDS VANE
APL FUNCTIONAL SYMBOL QUAD DOWNWARDS ARROW
APL FUNCTIONAL SYMBOL QUAD LEFTWARDS ARROW
APL FUNCTIONAL SYMBOL QUAD RIGHTWARDS ARROW
APL FUNCTIONAL SYMBOL QUAD UPWARDS ARROW
→ "ah, mũi tên APL" — thấy APL + (VANE hoặc ARROW)
```

---

## Tầng 3+4+5: Modifier xếp chồng (ví dụ cụ thể)

```
Cùng là "hướng phải" nhưng khác modifier:

RIGHTWARDS ARROW                         → thường, mặc định
HEAVY RIGHTWARDS ARROW                   → dày (tầng 3)
LIGHT RIGHTWARDS ARROW                   → mỏng (tầng 3)
BLACK RIGHTWARDS ARROW                   → đặc (tầng 4)
WHITE RIGHTWARDS ARROW                   → rỗng (tầng 4)
RIGHTWARDS ARROW WITH HOOK               → có móc (tầng 5)
RIGHTWARDS ARROW WITH LOOP               → có vòng (tầng 5)
RIGHTWARDS ARROW WITH TAIL               → có đuôi (tầng 5)
RIGHTWARDS ARROW WITH DOTTED STEM        → thân chấm (tầng 5)
RIGHTWARDS ARROW WITH STROKE             → gạch ngang (tầng 5)

Nhìn vào → biết ngay:
  "HEAVY BLACK RIGHTWARDS ARROW"
  = đơn + phải + dày + đặc + thẳng
  = (ĐƠN, PHẢI, DÀY, ĐẶC, THẲNG)
```

---

## Tóm tắt: Đọc tên → phân loại tức thì

```
Bước 1: Tìm từ KIỂU    → DOUBLE? TRIPLE? HARPOON? CLOCKWISE? CURVING?
         Không có        → ĐƠN (mặc định)

Bước 2: Tìm từ HƯỚNG   → RIGHTWARDS? LEFTWARDS? UPWARDS? DOWNWARDS?
                         → NORTH EAST? SOUTH WEST? LEFT RIGHT? UP DOWN?

Bước 3: Tìm từ ĐỘ DÀY → HEAVY? LIGHT? BOLD?
         Không có        → THƯỜNG (mặc định)

Bước 4: Tìm từ KIỂU TÔ → BLACK/FILLED? WHITE/OPEN? HALF? SHADOWED?
         Không có        → mặc định

Bước 5: Tìm từ ĐUÔI    → HOOK? LOOP? TAIL? DOTTED STEM? BARB? CURVED?
         Không có        → THẲNG (mặc định)

→ Kết quả: tuple (kiểu, hướng, độ_dày, tô, đuôi)
→ Nhìn vào biết: "ah cái thèn này là mũi tên [kiểu] [hướng] [dày/mỏng] [đặc/rỗng] [đuôi gì]"
```

---

## Bảng tổng hợp công thức vật lý

> Mỗi mũi tên Unicode = 1 vector v⃗ = (field_type, θ, ||v⃗||, ρ, BC) trong trường vector F⃗: R² → R².

| Tầng | Tham số | Ký hiệu | Miền giá trị | Ý nghĩa vật lý |
|------|---------|---------|-------------|----------------|
| 1 — Kiểu | field_type | F⃗ | {∇f, f⇔f⁻¹, a⃗×(b⃗×c⃗), lim₊, Δf/Δx, sin, ∇×F, piecewise} | Loại trường vector / phép toán vi phân |
| 2 — Hướng | direction | θ, ê | θ ∈ [0°, 360°), ê ∈ S¹ | Góc hướng — vector đơn vị trên đường tròn đơn vị |
| 3 — Độ dày | weight | \|\|v⃗\|\| | {ε, 1, M, M²} ⊂ R⁺ | Chuẩn (norm) — cường độ/độ lớn vector |
| 4 — Tô | fill | ρ(x) | {1, δ(\|x\|−r), H(x), e^(−αx²)} | Hàm mật độ — phân bố bên trong vector |
| 5 — Đuôi | tail | BC | {x₀+v⃗t, geodesic, C⁰\C¹, attractor} | Điều kiện biên / quỹ đạo |

### Công thức tổng hợp

```
Trường vector tổng quát cho mũi tên:

  F⃗(x, y) = ||v⃗|| · ê(θ) · ρ(x, y)

  trong đó:
    ê(θ) = (cos θ, sin θ)         — vector đơn vị hướng θ
    ||v⃗|| ∈ {ε, 1, M, M²}        — chuẩn (cường độ)
    ρ(x,y) ∈ {1, δ, H, Gaussian}  — hàm mật độ

  Điều kiện biên (đuôi):
    dx/dt = F⃗(x, y),  x(0) = x₀  — bài toán Cauchy

  Ví dụ: "HEAVY BLACK RIGHTWARDS ARROW"
    = F⃗ = M · (1, 0) · 1 = (M, 0)
    = trường gradient, hướng phải, cường độ lớn, đặc, quỹ đạo thẳng
    = (∇f, 0°, M, 1, linear)

Tổ hợp lý thuyết:  8 × 11 × 4 × 4 × 5 = 7,040 khả năng
Unicode thực tế:    ~618 ký tự mũi tên
Mỗi ký tự = 1 điểm trong không gian tham số 5 chiều.
```
