# UDC Real Formulas — Công thức toán học thật

> Mỗi nhóm = 1 công thức toán có ý nghĩa hình học / vật lý.
> Không đánh số — mà TÍNH TOÁN.

---

## S.0 — MŨI TÊN (Arrow)

### Hướng (Direction)

```
d⃗(θ) = (cos θ, sin θ)

  θ = 0      → RIGHTWARDS    d⃗ = (1, 0)
  θ = π/2    → UPWARDS       d⃗ = (0, 1)
  θ = π      → LEFTWARDS     d⃗ = (-1, 0)
  θ = 3π/2   → DOWNWARDS     d⃗ = (0, -1)
  θ = π/4    → NORTH EAST    d⃗ = (√2/2, √2/2)
  θ = 3π/4   → NORTH WEST    d⃗ = (-√2/2, √2/2)
  θ = 5π/4   → SOUTH WEST    d⃗ = (-√2/2, -√2/2)
  θ = 7π/4   → SOUTH EAST    d⃗ = (√2/2, -√2/2)
```

### Hai chiều (Bidirectional)

```
LEFT RIGHT:  D⃗ = d⃗(0) + d⃗(π)       = {(1,0), (-1,0)}
UP DOWN:     D⃗ = d⃗(π/2) + d⃗(3π/2)  = {(0,1), (0,-1)}
```

### Độ dày nét (Weight)

```
w(k) = w₀ · φᵏ       φ = (1+√5)/2 ≈ 1.618  (tỷ lệ vàng)

  k = -1  → LIGHT       w = w₀/φ ≈ 0.618 w₀
  k =  0  → NORMAL      w = w₀
  k =  1  → HEAVY       w = w₀·φ ≈ 1.618 w₀
  k =  2  → VERY HEAVY  w = w₀·φ² ≈ 2.618 w₀
```

### Kiểu đầu mũi tên (Arrowhead)

```
Tam giác đầu nhọn (filled):
  A(t) = p⃗ + t·d⃗·L,  t ∈ [0,1]

  đỉnh:  p⃗ + d⃗·L
  trái:  p⃗ + R(+α)·d⃗·h
  phải:  p⃗ + R(-α)·d⃗·h

  R(α) = | cos α  -sin α |     ma trận xoay
         | sin α   cos α |

  α = π/6  → đầu nhọn (normal)
  α = π/4  → đầu rộng (wide-headed)
  α = π/8  → đầu hẹp (narrow)
```

### Kiểu thân (Shaft)

```
Thẳng:    P(t) = p₀ + t·d⃗·L                    t ∈ [0,1]
Cong:     P(t) = (1-t)²p₀ + 2t(1-t)c + t²p₁    Bézier bậc 2
Gấp:      P(t) = p₀ + t·d⃗₁·L₁  (t<s),  p₁ + (t-s)·d⃗₂·L₂  (t≥s)
Gạch:     P(t) = p₀ + t·d⃗·L · ⌊2t/δ⌋ mod 2    (δ = chu kỳ gạch)
Lượn:     P(t) = p₀ + t·d⃗·L + A·sin(2πft)·n⃗   (n⃗ ⊥ d⃗)
```

### Vòng (Circular)

```
P(t) = c⃗ + r·(cos(ωt + φ₀), sin(ωt + φ₀))     t ∈ [0, T]

  ω = +1  → CLOCKWISE
  ω = -1  → ANTICLOCKWISE
  T = 2π  → FULL CIRCLE
  T = π   → SEMICIRCLE
```

### Harpoon (Móc)

```
Nửa đầu nhọn:

  BARB UPWARDS:   chỉ giữ nửa trên của tam giác đầu
    { (x,y) ∈ Arrowhead : y ≥ y_center }

  BARB DOWNWARDS: chỉ giữ nửa dưới
    { (x,y) ∈ Arrowhead : y ≤ y_center }
```

### Đôi / Ba (Double / Triple)

```
Double:  P₁(t) = P(t),  P₂(t) = P(t) + δ·n⃗     (2 nét song song, cách δ)
Triple:  P₁, P₂, P₃ = P(t) + {-δ, 0, +δ}·n⃗     (3 nét song song)

  n⃗ = (-sin θ, cos θ)   pháp tuyến của hướng d⃗
```

### Tô (Fill)

```
  BLACK/FILLED:  χ(p⃗) = 1    ∀ p⃗ ∈ interior
  WHITE/OPEN:    χ(p⃗) = 1    iff |SDF(p⃗)| < ε   (chỉ viền)
  SHADOWED:      χ(p⃗) = 1    ∀ p⃗ ∈ interior ∪ shadow(p⃗ + s⃗)
```

### Tổng hợp: Công thức đầy đủ 1 mũi tên

```
Arrow(θ, k, type, fill) = {
  shaft:     P(t; type)           t ∈ [0, 1-h/L]
  head:      A(α; fill)          tại P(1)
  weight:    w = w₀ · φᵏ
  direction: d⃗ = (cos θ, sin θ)
}
```

---

## S.1 — HÌNH HỌC (Geometric) — SDF

### Tròn (Circle)

```
SDF_circle(p⃗, r) = |p⃗| - r

  |p⃗| = √(x² + y²)
  SDF < 0  → bên trong
  SDF = 0  → trên viền
  SDF > 0  → bên ngoài
```

### Vuông (Square)

```
SDF_square(p⃗, a) = max(|x| - a, |y| - a)
```

### Chữ nhật (Rectangle)

```
SDF_rect(p⃗, w, h) = max(|x| - w, |y| - h)
```

### Tam giác đều (Equilateral Triangle)

```
SDF_tri(p⃗, r) = max( |x| - r/2,  -y - r/(2√3),  y - r/√3 )
```

### Hình thoi / Kim cương (Diamond)

```
SDF_diamond(p⃗, a, b) = |x|/a + |y|/b - 1

  a = bán trục ngang,  b = bán trục dọc
```

### Elip (Ellipse)

```
SDF_ellipse(p⃗, a, b) ≈ (x²/a² + y²/b² - 1) · min(a,b)
```

### Sao n cánh (n-pointed Star)

```
Star(p⃗, n, r₁, r₂):

  θ = atan2(y, x)
  k = round(θ·n/(2π))
  α = 2πk/n

  Mỗi cánh = tam giác giữa:
    gốc (0,0),  đỉnh ngoài r₁·(cos α, sin α),
    đỉnh trong r₂·(cos(α ± π/n), sin(α ± π/n))

  r₁ = bán kính đỉnh cánh
  r₂ = bán kính kẽ cánh
  n = số cánh:  { 4:FOUR POINTED, 5:FIVE, 6:SIX, 8:EIGHT, 12:TWELVE }
```

### Đa giác đều n cạnh (Regular Polygon)

```
SDF_polygon(p⃗, n, r):

  θ = atan2(y, x)
  α = 2π/n
  θ_sector = mod(θ, α) - α/2

  SDF = |p⃗| · cos(θ_sector) - r · cos(α/2)

  n = 5: PENTAGON
  n = 6: HEXAGON
  n = 8: OCTAGON
```

### Chữ thập (Cross)

```
SDF_cross(p⃗, a, b) = min( SDF_rect(p⃗, a, b),  SDF_rect(p⃗, b, a) )

  Saltire (chéo):  SDF_cross( R(π/4)·p⃗, a, b )
```

### Hoa (Florette) — n cánh hoa

```
Florette(p⃗, n, r):

  θ = atan2(y, x)
  ρ = |p⃗|
  petal = r · |cos(nθ/2)|

  SDF = ρ - petal

  n cánh: { 4:FOUR PETALLED, 6:SIX PETALLED, 8:EIGHT PETALLED }
```

### Fill — Áp dụng cho MỌI hình

```
BLACK/FILLED:   Ω = { p⃗ : SDF(p⃗) ≤ 0 }
WHITE/OPEN:     Ω = { p⃗ : |SDF(p⃗)| ≤ w/2 }         (viền dày w)
HALF BLACK:     Ω = { p⃗ : SDF(p⃗) ≤ 0 ∧ n⃗·p⃗ ≥ 0 }  (n⃗ = mặt phẳng cắt)
DOTTED:         Ω = { p⃗ : |SDF(p⃗)| ≤ w/2 ∧ ⌊s/δ⌋ mod 2 = 0 }  (s = arc length)
```

### Kích cỡ — Fibonacci scaling (giống Arrow weight)

```
r(k) = r₀ · φᵏ       φ = (1+√5)/2

  k = -2  → VERY SMALL   r = r₀/φ²
  k = -1  → SMALL        r = r₀/φ
  k =  0  → NORMAL       r = r₀
  k =  1  → MEDIUM       r = r₀·φ
  k =  2  → LARGE        r = r₀·φ²
```

---

## S.2 — VẼ HỘP (Box Drawing) — Đồ thị

### Nét = đoạn thẳng trên lưới

```
Mỗi ô = 1 nút trên lưới Z²
Mỗi ký tự box drawing = tập cạnh E ⊆ {N, S, E, W}

  ─  →  E = {E, W}          ngang
  │  →  E = {N, S}          dọc
  ┌  →  E = {S, E}          góc xuống-phải
  ┐  →  E = {S, W}          góc xuống-trái
  └  →  E = {N, E}          góc lên-phải
  ┘  →  E = {N, W}          góc lên-trái
  ├  →  E = {N, S, E}       T dọc-phải
  ┤  →  E = {N, S, W}       T dọc-trái
  ┬  →  E = {S, E, W}       T ngang-xuống
  ┴  →  E = {N, E, W}       T ngang-lên
  ┼  →  E = {N, S, E, W}    giao cắt
```

### Trọng số cạnh (Weight)

```
Mỗi cạnh e ∈ E mang trọng số:

  w(e) ∈ { 1:LIGHT, 2:HEAVY, (1,1):DOUBLE }

BOX DRAWINGS LIGHT DOWN AND RIGHT:
  E = {S:1, E:1}

BOX DRAWINGS DOWN HEAVY AND RIGHT LIGHT:
  E = {S:2, E:1}

BOX DRAWINGS DOUBLE DOWN AND RIGHT:
  E = {S:(1,1), E:(1,1)}
```

### Cung (Arc) — Bo tròn góc

```
Arc: thay góc vuông bằng 1/4 hình tròn

  P(t) = c⃗ + r·(cos(θ₀ + t·π/2), sin(θ₀ + t·π/2))    t ∈ [0,1]

  ╭: θ₀ = 3π/2  (arc down-right)
  ╮: θ₀ = π     (arc down-left)
  ╰: θ₀ = 0     (arc up-right)
  ╯: θ₀ = π/2   (arc up-left)
```

---

## S.3 — CHỮ NỔI (Braille) — Đại số Boolean

```
B = (b₁, b₂, b₃, b₄, b₅, b₆, b₇, b₈) ∈ GF(2)⁸

  Ma trận vật lý:     Giá trị nhị phân:
  [b₁] [b₄]           B = Σᵢ bᵢ · 2^(i-1)
  [b₂] [b₅]
  [b₃] [b₆]           B ∈ [0, 255]
  [b₇] [b₈]           |B| = popcount(B) = số chấm bật

  BRAILLE PATTERN DOTS-1     → B = 0b00000001 = 1
  BRAILLE PATTERN DOTS-135   → B = 0b00010101 = 21
  BRAILLE PATTERN DOTS-12345678 → B = 0b11111111 = 255
  BRAILLE PATTERN BLANK      → B = 0b00000000 = 0
```

---

## S.4 — APL — Đại số tổ hợp

```
APL(α, m) = α ⊕ m

  α ∈ Σ_base = { α, ι, ω, ρ, ∇, Δ, ○, ◇, ★, ⎕, ∘, ∩, ∪, ∧, ∨, ⊤, ⊥, ⊢, ⊣, /, \, ,, ⌶, ⍬ }
  m ∈ Σ_mod  = { ∅, ̲, ̈, ̃, |, — }

  ⊕: Σ_base × Σ_mod → APL_Symbol
  |Σ_base| = 24,  |Σ_mod| = 6

  α ⊕ ∅ = α           (không modifier)
  α ⊕ ̲ = α̲           (underbar)
  ∇ ⊕ ̈ = ∇̈           (del diaeresis)
  ∇ ⊕ ̃ = ∇̃           (del tilde)
```

---

## S.5 — KỸ THUẬT (Technical) — Miền chuyên biệt

```
Tech(d, i) = Ψ_d(i)

  d ∈ { nha_khoa, điện, hóa_học, đo_lường, thiết_bị }

  Ψ_nha_khoa(dir, wave):
    Nét = Box Drawing ∩ { wave modulation }
    P(t) = segment(dir) + A·sin(2πft)·n⃗ · [wave=1]

  Ψ_điện:
    AC:  I(t) = I₀ · sin(ωt)
    DC:  I(t) = I₀ = const

  Ψ_hóa_học:
    Benzene = SDF_polygon(p⃗, 6, r)          (lục giác đều)
    Benzene + circle = SDF_polygon ∩ SDF_circle

  Ψ_đo_lường:
    Scan line y = k/9,  k ∈ {1, 3, 7, 9}
```

---

## S.6 — KHỐI (Block) — Hàm đặc trưng trên [0,1]²

```
Block(π, ρ) = { p⃗ ∈ [0,1]² : C(p⃗; π, ρ) }

  FULL BLOCK:           C = true                  (∀ p⃗)
  UPPER HALF:           C = (y ≥ 1/2)
  LOWER HALF:           C = (y < 1/2)
  LEFT HALF:            C = (x < 1/2)
  RIGHT HALF:           C = (x ≥ 1/2)

  LEFT k/8 BLOCK:       C = (x < k/8)             k ∈ {1,2,3,4,5,6,7}
  LOWER k/8 BLOCK:      C = (y < k/8)

  UPPER LEFT QUADRANT:  C = (x < 1/2 ∧ y ≥ 1/2)
  LOWER RIGHT QUADRANT: C = (x ≥ 1/2 ∧ y < 1/2)

  LIGHT SHADE:          C = (hash(p⃗) < 0.25)     25% coverage
  MEDIUM SHADE:         C = (hash(p⃗) < 0.50)     50%
  DARK SHADE:           C = (hash(p⃗) < 0.75)     75%

  ARC (cung):           C = (SDF_circle(p⃗ - corner, r) ≤ 0)  ∩ quadrant
```
