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

---

## R.0 — TOÁN TỬ (Operator)

### Cộng / Trừ

```
a + b = a + b           (cộng)
a − b = a + (−b)        (trừ = cộng nghịch đảo)

Circled:  a ⊕ b = (a + b) mod n     (cộng modular, nhóm Z_n)
          a ⊖ b = (a − b) mod n
```

### Nhân / Chia

```
a × b                   (nhân)
a ÷ b = a · b⁻¹         (chia = nhân nghịch đảo)

Circled:  a ⊗ b          (tích tensor)
Dot:      a ⊙ b = Σᵢ aᵢbᵢ  (tích vô hướng)
```

### Tích phân (Integral)

```
∫ₐᵇ f(x) dx = lim_{n→∞} Σᵢ f(xᵢ)·Δx        (Riemann)

∮_γ f(z) dz = 0                               (Cauchy, γ kín)

∬_S f dA,  ∭_V f dV                            (mặt, thể tích)

Chiều:  ω = +1 → ∮ (clockwise),  ω = -1 → ∮ (anticlockwise)
```

### Tổng / Tích (Summation / Product)

```
Σᵢ₌₁ⁿ aᵢ = a₁ + a₂ + ⋯ + aₙ

∏ᵢ₌₁ⁿ aᵢ = a₁ · a₂ · ⋯ · aₙ

∐ (coproduct) = dual of ∏ trong lý thuyết phạm trù
```

### Căn (Root)

```
√x = x^(1/2)
∛x = x^(1/3)
∜x = x^(1/4)
ⁿ√x = x^(1/n)
```

### Vi phân (Differential)

```
∂f/∂x = lim_{h→0} [f(x+h) - f(x)] / h       (đạo hàm riêng)

∇f = (∂f/∂x, ∂f/∂y, ∂f/∂z)                   (gradient)

∇·F⃗ = ∂Fx/∂x + ∂Fy/∂y + ∂Fz/∂z              (divergence)

∇×F⃗ = | i⃗   j⃗   k⃗  |                         (curl)
        | ∂x  ∂y  ∂z |
        | Fx  Fy  Fz |
```

---

## R.1 — SO SÁNH (Comparison) — Quan hệ thứ tự

### Bằng / Không bằng

```
a = b    ⟺  d(a,b) = 0                (metric space)
a ≠ b    ⟺  d(a,b) > 0
a ≡ b    ⟺  a mod n = b mod n         (đồng dư)
a ≅ b    ⟺  ∃ isomorphism f: a → b    (đẳng cấu)
```

### Xấp xỉ

```
a ≈ b    ⟺  |a - b| < ε              (ε-xấp xỉ)
a ∼ b    ⟺  lim a/b = 1              (tiệm cận)
a ≍ b    ⟺  c₁·b ≤ a ≤ c₂·b         (cùng bậc)
a ∝ b    ⟺  ∃k: a = k·b              (tỷ lệ)
```

### Thứ tự

```
a < b    ⟺  b - a > 0
a ≤ b    ⟺  a < b ∨ a = b
a ≪ b    ⟺  a/b → 0                  (nhỏ hơn nhiều)
a ≺ b    ⟺  a precedes b              (thứ tự bộ phận)
```

### Tập hợp

```
x ∈ A    ⟺  x là phần tử của A
A ⊂ B    ⟺  ∀x: x ∈ A → x ∈ B
A ∪ B    = { x : x ∈ A ∨ x ∈ B }
A ∩ B    = { x : x ∈ A ∧ x ∈ B }
A \ B    = { x : x ∈ A ∧ x ∉ B }
```

### Phủ định (thêm gạch chéo /)

```
a ≠ b    = ¬(a = b)
a ∉ A    = ¬(a ∈ A)
A ⊄ B    = ¬(A ⊂ B)
a ≇ b    = ¬(a ≅ b)

Tổng quát:  R̸ = ¬R    (gạch chéo = phủ định quan hệ)
```

---

## R.2 — CHỮ CÁI TOÁN (Math Letter) — Tích Descartes

```
MathLetter = Script × Style × Case × Alphabet

Script  ∈ { Latin, Greek, Arabic }
Style   ∈ { serif, sans, bold, italic, bold-italic, fraktur,
             double-struck, script, mono, ... }
Case    ∈ { UPPER, lower }
Alphabet = hệ chữ cái của Script

MathLetter(s, f, c, l) = font_map(s, f, c)(l)

Ví dụ:
  font_map(Latin, bold, UPPER)(A) = 𝐀
  font_map(Latin, fraktur, UPPER)(A) = 𝔄
  font_map(Latin, double-struck, UPPER)(A) = 𝔸
  font_map(Greek, italic, lower)(α) = 𝛼

|MathLetter| = 3 × 13 × 2 × |Alphabet| ≈ 2600
```

---

## R.3 — SỐ (Number) — Hệ đếm vị trí

### Hệ thập phân (positional)

```
N = Σᵢ dᵢ · Bⁱ       B = base,  dᵢ = digit tại vị trí i

  B = 10: decimal      123 = 1·10² + 2·10¹ + 3·10⁰
  B = 60: sexagesimal  (Lưỡng Hà / cuneiform)
```

### Que đếm (Counting Rod)

```
Rod(place, value):
  place ∈ { UNIT, TENS }
  value ∈ { 1..9 }

  UNIT: nét dọc = value
  TENS: nét ngang = value
```

### La Mã (Roman)

```
Roman = Σ sᵢ · vᵢ     sᵢ ∈ {+1, -1}

  sᵢ = -1 nếu vᵢ < vᵢ₊₁  (subtractive: IV = -1 + 5 = 4)
  sᵢ = +1 ngược lại

  v: { I:1, V:5, X:10, L:50, C:100, D:500, M:1000 }
```

### Phân số (Fraction)

```
p/q ∈ Q       p, q ∈ Z,  q ≠ 0

  ½ = 1/2,  ⅓ = 1/3,  ¼ = 1/4,  ⅕ = 1/5, ...
  ⅔ = 2/3,  ¾ = 3/4,  ⅗ = 3/5, ...
```

---

## R.4 — DẤU CÂU (Punctuation) — Cặp đối xứng

```
Ngoặc = cặp (open, close) với tính chất lồng nhau:

  depth(s) = Σᵢ δ(sᵢ)    δ(open) = +1,  δ(close) = -1

  Hợp lệ ⟺ depth(s) = 0 ∀ prefix: depth ≥ 0    (Dyck language)

  (  )     depth: 0 → 1 → 0   ✓
  ( ( ) )  depth: 0 → 1 → 2 → 1 → 0   ✓
  ) (      depth: 0 → -1   ✗

Phản xạ:  mirror(left) = right
  mirror( ( ) = mirror( ) )
  mirror( [ ) = mirror( ] )
  mirror( { ) = mirror( } )
  mirror( ⟨ ) = mirror( ⟩ )
  mirror( « ) = mirror( » )
```

---

## R.5 — TIỀN TỆ (Currency) — Ánh xạ ký hiệu → giá trị

```
Currency(symbol) = (name, ISO_4217, region)

  $(·) = amount · exchange_rate(USD, target)

  $1 USD = ¥ · rate(USD→JPY)
         = € · rate(USD→EUR)
         = £ · rate(USD→GBP)
         = ₿ · rate(USD→BTC)

  rate: Currency × Currency → R⁺
  rate(A,B) · rate(B,A) = 1     (nghịch đảo)
  rate(A,C) = rate(A,B) · rate(B,C)   (bắc cầu)
```

---

## R.6 — CỔ ĐẠI (Ancient) — Hệ đếm phi vị trí

```
Greek Acrophonic:

  Value = Σᵢ sᵢ      (additive, không subtractive)

  Symbols:  Ι=1, Π=5, Δ=10, Η=100, Χ=1000, Μ=10000

  50 = ΠΔ = 5×10     (multiplicative compound)
  500 = ΠΗ = 5×100

  Với đơn vị:
  Value(symbol, unit) = numeric_value × unit_weight

  unit_weight: { stater=1, talent=6000, drachma=1/6, mina=100 }
```

---

## V — VALENCE — Mô hình cảm xúc Russell

### Công thức cốt lõi

```
V: Word → [-1, +1]

V(w) = Σᵢ αᵢ · vᵢ(w)     αᵢ = trọng số nguồn,  Σαᵢ = 1

  v₁(w) = NRC_VAD_valence(w)          (tra bảng 54,801 từ)
  v₂(w) = emoji_subgroup_valence(w)   (tra subgroup emoji)
  v₃(w) = 0                           (fallback trung tính)
```

### Lượng tử hóa

```
Q_V: [-1, +1] → Z₈

Q_V(x) = ⌊ (x + 1) / 2 · 7 + 0.5 ⌋

  Q_V(-1.0) = 0     rất tiêu cực
  Q_V(-0.5) = 1.75 → 2
  Q_V( 0.0) = 3.5  → 4   trung tính
  Q_V(+0.5) = 5.25 → 5
  Q_V(+1.0) = 7     rất tích cực
```

### Quy tắc KHÔNG trung bình (Amplify, không Average)

```
❌  V_composite = (v₁ + v₂) / 2               (sai — trung bình)

✅  V_composite = sign(Σvᵢ) · ‖v⃗‖             (amplify qua Silk walk)

    ‖v⃗‖ = √(Σᵢ vᵢ²)                           (norm — bảo toàn năng lượng)

Ví dụ: "hate" + "fear" = sign(-1-0.8) · √(1² + 0.8²)
      = -1 · 1.28 = -1.28 → clamp → -1.0    (cực tiêu cực, không giảm)
```

---

## A — AROUSAL — Trục kích hoạt

### Công thức cốt lõi

```
A: Word → [-1, +1]

A(w) = Σᵢ αᵢ · aᵢ(w)

  a₁(w) = NRC_VAD_arousal(w)
  a₂(w) = emoji_subgroup_arousal(w)
  a₃(w) = 0
```

### Lượng tử hóa (cùng công thức V)

```
Q_A: [-1, +1] → Z₈

Q_A(x) = ⌊ (x + 1) / 2 · 7 + 0.5 ⌋
```

### Không gian cảm xúc 2D (Russell's Circumplex)

```
E⃗(w) = ( V(w), A(w) ) ∈ [-1,1]²

  |E⃗| = √(V² + A²)              cường độ cảm xúc tổng
  θ_E = atan2(A, V)              góc cảm xúc

  θ ≈ 0    → vui + bình tĩnh    (content, serene)
  θ ≈ π/2  → trung tính + kích  (alert, tense)
  θ ≈ π    → buồn + bình tĩnh   (sad, depressed)
  θ ≈ 3π/2 → trung tính + tĩnh  (calm, sleepy... sai→ nên -π/2)

Cortisol-Adrenaline analogy:
  cortisol  ~ |V_negative|       (stress hormone)
  adrenaline ~ A_positive        (activation hormone)
  composite = √(cortisol² + adrenaline²)   (không trung bình!)
```

---

## T.0 — QUẺ DỊCH (Hexagram) — Đại số nhị phân 6-bit

```
H = Σᵢ₌₁⁶ yᵢ · 2^(i-1)     yᵢ ∈ {0:âm ⚋, 1:dương ⚊}

H ∈ Z₆₄ = {0, 1, ..., 63}

Quẻ = (trigram_lower, trigram_upper) = (H mod 8, H div 8)

  trigram ∈ Z₈ = { ☰:7, ☱:6, ☲:5, ☳:4, ☴:3, ☵:2, ☶:1, ☷:0 }

Biến đổi:
  complement(H) = 63 - H = ¬H                  (đảo mọi hào)
  reverse(H) = Σᵢ y₍₇₋ᵢ₎ · 2^(i-1)             (lật ngược)
  nuclear(H) = extract_inner_4_lines(H)          (quẻ hỗ)

Chuỗi biến đổi (sequence):
  H_{n+1} = transform(H_n, changing_lines)
  changing_lines = { i : yᵢ is "old" (lão dương/lão âm) }
```

---

## T.1 — TỨ QUÁI (Tetragram) — Hệ tam phân 4-trit

```
T = Σᵢ₌₁⁴ yᵢ · 3^(i-1)     yᵢ ∈ {0:âm, 1:trung, 2:dương}

T ∈ Z₈₁ = {0, 1, ..., 80}

  Monogram:  M = y₁ ∈ Z₃
  Digram:    D = y₁ + y₂·3 ∈ Z₉
  Tetragram: T = y₁ + y₂·3 + y₃·9 + y₄·27 ∈ Z₈₁
```

---

## T.2 — BYZANTINE — Tempo trên thang logarit

### Agogi (Tempo)

```
Tempo(k) = T₀ · 2^(k/4)     k ∈ [-4, +4]

  k = -4  → POLI ARGI     T = T₀/4    (rất chậm)
  k = -3  → ARGI           T = T₀/2√2
  k = -2  → ARGOTERI       T = T₀/2    (hơi chậm)
  k = -1  → METRIA         T = T₀/√2
  k =  0  → MESI           T = T₀      (trung bình)
  k = +1  → GORGI          T = T₀·√2
  k = +2  → GORGOTERI      T = T₀·2    (hơi nhanh)
  k = +4  → POLI GORGI     T = T₀·4    (rất nhanh)
```

### Neume — Interval

```
Neume(n) = Δpitch ∈ Z    (số bước đi lên/xuống)

  ISON      → Δ = 0     (giữ nguyên)
  OLIGON    → Δ = +1    (lên 1)
  PETASTI   → Δ = +1    (lên 1, variant)
  APOSTROFOS → Δ = -1   (xuống 1)
  ELAFRON   → Δ = -2    (xuống 2)
  CHAMILI   → Δ = +2    (lên 2)
```

### Fthora — Chuyển điệu thức (Mode shift)

```
Mode(f) = (scale, tonic)

  fthora: Mode_current → Mode_target

  scale ∈ { diatonic, chromatic, enharmonic }
  tonic ∈ Z₇ = vị trí trong thang âm
```

---

## T.4 — NHẠC PHƯƠNG TÂY — Trường độ & Tần số

### Trường độ (Duration) — Lũy thừa 2

```
Duration(n) = whole · 2^(-n)     n ∈ Z

  n = -3  → MAXIMA       = 8 × whole
  n = -2  → LONGA        = 4 × whole
  n = -1  → BREVE        = 2 × whole
  n =  0  → WHOLE        = 1 whole
  n =  1  → HALF         = 1/2
  n =  2  → QUARTER      = 1/4
  n =  3  → EIGHTH       = 1/8
  n =  4  → SIXTEENTH    = 1/16
```

### Cao độ (Pitch) — Tần số

```
f(n) = 440 · 2^((n-69)/12)     Hz     (A4 = 440 Hz, MIDI note n)

  SHARP:   n → n + 1     (lên nửa cung)
  FLAT:    n → n - 1     (xuống nửa cung)
  NATURAL: hủy sharp/flat trước đó
```

### Cường độ (Dynamics) — Decibel

```
Dynamics(k) = dB₀ + k · ΔdB     k ∈ Z

  k = -3  → ppp (pianississimo)
  k = -2  → pp  (pianissimo)
  k = -1  → p   (piano)
  k =  0  → mp  (mezzo piano)
  k = +1  → mf  (mezzo forte)
  k = +2  → f   (forte)
  k = +3  → ff  (fortissimo)
  k = +4  → fff (fortississimo)

Biến thiên:
  CRESCENDO:    dB(t) = dB₀ + α·t      (tăng tuyến tính)
  DECRESCENDO:  dB(t) = dB₀ - α·t      (giảm tuyến tính)
  RINFORZANDO:  dB(t) = dB₀ + A·δ(t-t₀)  (xung nhấn tại t₀)
```

### Ornament — Biến điệu

```
Trill:      f(t) = f₀ + Δf · square(2πf_trill·t)    (dao động nửa cung)
Vibrato:    f(t) = f₀ + Δf · sin(2πf_vib·t)          (dao động liên tục)
Glissando:  f(t) = f₀ + (f₁-f₀)·t/T                  (trượt tuyến tính)
Arpeggio:   f(t) = f_chord[⌊t/δ⌋ mod N]              (rải hợp âm)
Turn:       f(t) = f₀ + Δf · {+1, 0, -1, 0}[phase]   (4 giai đoạn)
```

---

## Bảng tổng hợp: Tên — Công thức nổi bật

```
MŨI TÊN:       d⃗(θ) = (cos θ, sin θ)
ĐỘ DÀY:        w(k) = w₀ · φᵏ                     φ = (1+√5)/2
HÌNH TRÒN:      SDF(p⃗) = |p⃗| - r
HÌNH VUÔNG:     SDF(p⃗) = max(|x|-a, |y|-a)
SAO n CÁNH:     r(θ) = r₁·r₂ / √(r₂²cos²θ' + r₁²sin²θ')
CHỮ NỔI:        B = Σ bᵢ · 2^(i-1) ∈ GF(2)⁸
KHỐI:           C(p⃗) = (x < k/8)
TÍCH PHÂN:      ∫ₐᵇ f dx = lim Σ f(xᵢ)·Δx
BẰNG:           a = b ⟺ d(a,b) = 0
XẤP XỈ:         a ≈ b ⟺ |a-b| < ε
PHÂN SỐ:        p/q ∈ Q
NGOẶC:          depth(s) = Σ δ(sᵢ),  Dyck language
VALENCE:        Q_V(x) = ⌊(x+1)/2 · 7 + 0.5⌋
AROUSAL:        Q_A(x) = ⌊(x+1)/2 · 7 + 0.5⌋
CẢM XÚC 2D:    E⃗ = (V, A),  |E⃗| = √(V²+A²)
QUẺ DỊCH:       H = Σ yᵢ · 2^(i-1) ∈ Z₆₄
TỨ QUÁI:        T = Σ yᵢ · 3^(i-1) ∈ Z₈₁
TEMPO:          Tempo(k) = T₀ · 2^(k/4)
TRƯỜNG ĐỘ:     D(n) = whole · 2^(-n)
CAO ĐỘ:         f(n) = 440 · 2^((n-69)/12) Hz
CƯỜNG ĐỘ:       dB(k) = dB₀ + k·ΔdB
EULER:           e^(iπ) + 1 = 0
```
