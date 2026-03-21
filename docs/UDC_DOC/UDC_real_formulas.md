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
