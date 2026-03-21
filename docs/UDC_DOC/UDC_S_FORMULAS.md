# S — Shape · Công thức toán học thật

> Mỗi nhóm = 1 công thức có ý nghĩa hình học thật sự.
> Không đánh số tuple — chỉ toán, chỉ SDF, chỉ biến đổi.

---

## Tổng quan: S là gì?

```
S(P) = signed distance function: f(P) → ℝ

  f(P) < 0   →  bên trong hình
  f(P) = 0   →  bề mặt (biên)
  f(P) > 0   →  bên ngoài hình

Mọi hình trong S đều có gradient analytical:
  ∇f(P) = normal vector tại P  (không cần numerical differentiation)
```

---

## Phép biến đổi chung (Modifiers)

Các modifier áp dụng cho MỌI nhóm S.x:

```
Cho f(P) là SDF gốc của bất kỳ nhóm nào:

HEAVY (dày):        f_heavy(P)   = f(P) − t           t > 0: dilation (Minkowski sum)
LIGHT (mỏng):       f_light(P)   = f(P) + t           t > 0: erosion
VERY HEAVY:         f(P) − t₂,  t₂ > t₁              dilation lớn hơn

BLACK (đặc):        region = { P : f(P) < 0 }         bên trong
WHITE (rỗng):       f_shell(P)  = |f(P)| − w          vỏ, w = độ dày viền

SMALL:              f_small(P)  = f(P/s) · s           s < 1: co lại
LARGE:              f_large(P)  = f(P/s) · s           s > 1: phóng to
MEDIUM:             f_med(P)    = f(P/s) · s           s ≈ 0.7

ROTATED θ:          f_rot(P)    = f(R_θ · P)           R_θ = ma trận xoay góc θ
TURNED:             f(R₁₈₀ · P)                        xoay 180°

CIRCLED:            f_circled(P) = max(f(P), |P| − r_c)   cắt bởi vòng tròn r_c
CONTAINING X:       f_contain(P) = min(f_outer(P), f_inner(P))   hình lồng hình
```

---

## S.0 — Mũi tên (Arrow)

### Công thức cơ sở: Tia có hướng + đầu nhọn

```
Arrow(P) = min( f_shaft(P), f_head(P) )

Thân (shaft):
  f_shaft(P) = capsule(P, A, B, r)
             = |P − clamp(P·d̂, 0, L)·d̂| − r

  A = điểm đầu,  B = điểm cuối
  d̂ = (B−A)/|B−A|  = unit direction vector
  L = |B−A|         = chiều dài thân
  r = bán kính thân (→ HEAVY: r lớn, LIGHT: r nhỏ)

Đầu nhọn (head) — tam giác:
  f_head(P) = cone(P − B, h, r_head)
            = dot_blend(P', h, r_head)

  h = chiều cao đầu nhọn
  r_head = bán kính đáy đầu nhọn
```

### Biến đổi theo kiểu

```
ĐƠN:       Arrow(P)                                    1 shaft + 1 head
ĐÔI:       min(Arrow(P), Arrow(−P))                    2 đầu nhọn, đối xứng
BA:         min(Arrow(P, r₁), Arrow(P, r₂), Arrow(P, r₃))  3 shaft song song
MÓC:       min(f_shaft(P), f_half_head(P))             chỉ nửa đầu nhọn (harpoon)
GẠCH:      f_shaft_dashed(P) = f_shaft(P) + A·sin(2πP·d̂/λ)   sóng cắt thân
LƯỢN:      f_shaft_wave(P) = capsule(P, curve(t))      thân cong theo sin/spline
VÒNG:      f_arc(P) = ||(|P.xz|−R, P.y)|| − r         cung tròn (torus 2D)
GẤP:       min(f_seg1(P), f_seg2(P))                   2 đoạn thẳng nối góc
```

### Hướng = xoay

```
RIGHTWARDS →:   θ = 0°          R_θ = I
UPWARDS ↑:      θ = 90°         R_θ = [[0,-1],[1,0]]
LEFTWARDS ←:    θ = 180°        R_θ = [[-1,0],[0,-1]]
DOWNWARDS ↓:    θ = 270°        R_θ = [[0,1],[-1,0]]
NORTH EAST ↗:   θ = 45°
SOUTH WEST ↙:   θ = 225°
LEFT RIGHT ↔:   min(Arrow(P), Arrow(R₁₈₀·P))
UP DOWN ↕:      min(Arrow(R₉₀·P), Arrow(R₂₇₀·P))
```

---

## S.1 — Hình học (Geometric)

### Công thức SDF cho từng hình gốc

```
TRÒN (circle):
  f_circle(P) = |P| − r                               ● ○

VUÔNG (square):
  f_square(P) = ‖max(|P| − b, 0)‖ + min(max(|Px|−b, |Py|−b), 0)    ■ □

TAM GIÁC (triangle):
  f_triangle(P) = max(dot(P−v₀, n₀), dot(P−v₁, n₁), dot(P−v₂, n₂))
  vᵢ = đỉnh,  nᵢ = normal cạnh hướng ra ngoài          ▲ △

THOI (diamond/lozenge):
  f_diamond(P) = |Px|·cos(45°) + |Py|·sin(45°) − s      ◆ ◇
               = (|Px| + |Py|)/√2 − s                   (= octahedron 2D)

SAO (star):
  f_star(P, n, r₁, r₂) = polygon SDF xen kẽ r₁, r₂ với n cánh    ★ ☆
  Góc mỗi cánh: 2π/n
  r₁ = bán kính ngoài,  r₂ = bán kính trong

CHỮ THẬP (cross):
  f_cross(P) = min(f_box_h(P), f_box_v(P))             ✚
  f_box_h = box(P, (w, h_long, 0))     ngang
  f_box_v = box(P, (h_long, w, 0))     dọc

ĐA GIÁC (polygon, n cạnh):
  f_polygon(P, n, r) = SDF chính n-giác đều bán kính r   ⬠ ⬡
  n=5: ngũ giác,  n=6: lục giác,  n=8: bát giác

ELIP (ellipse):
  f_ellipse(P) = (|P/r|² − 1) · min(rₓ, r_y)           ⬮ ⬯
  r = (rₓ, r_y)  bán trục

CHỮ NHẬT (rectangle):
  f_rect(P) = ‖max(|P| − b, 0)‖                        ▬ ▭
  b = (bₓ, b_y),  bₓ ≠ b_y                             (box không vuông)

HOA (florette):
  f_flower(P, n) = |P| − r + A·cos(n·atan2(Py, Px))    ✿ ❀
  n = số cánh,  A = biên độ sóng cánh
```

### Modifier hình học = phép biến đổi SDF

```
Ví dụ tổng hợp — "HEAVY WHITE ROTATED SQUARE":

  f(P) = |f_square(R₄₅ · P)| − w  − t

  Bước 1: f_square(P)         ← SDF hình vuông gốc
  Bước 2: f_square(R₄₅ · P)   ← xoay 45° (ROTATED)
  Bước 3: |...| − w           ← lấy vỏ (WHITE = rỗng), w = độ dày viền
  Bước 4: ... − t             ← dilation (HEAVY), t = độ dày thêm
```

---

## S.2 — Vẽ hộp (Box Drawing)

### Công thức cơ sở: Đoạn thẳng SDF

```
Segment(P, A, B, w) = capsule(P, A, B, w/2)

  w = độ dày nét (LIGHT: w₁,  HEAVY: w₂ > w₁,  DOUBLE: 2 nét cách d)
```

### Các kiểu nối

```
NGANG ─:     Segment(P, (−1,0), (+1,0), w)
DỌC │:       Segment(P, (0,−1), (0,+1), w)

GÓC ┌:       min(Segment(P, (0,0), (+1,0), w),     ngang phải
               Segment(P, (0,0), (0,+1), w))        dọc xuống

GÓC ┐:       min(Segment(P, (−1,0), (0,0), w),     ngang trái
               Segment(P, (0,0), (0,+1), w))        dọc xuống

GÓC └:       min(Segment(P, (0,0), (+1,0), w),
               Segment(P, (0,−1), (0,0), w))

GÓC ┘:       min(Segment(P, (−1,0), (0,0), w),
               Segment(P, (0,−1), (0,0), w))

T-NỐI ├:     min(dọc toàn bộ, ngang phải)
T-NỐI ┤:     min(dọc toàn bộ, ngang trái)
T-NỐI ┬:     min(ngang toàn bộ, dọc xuống)
T-NỐI ┴:     min(ngang toàn bộ, dọc lên)

GIAO ┼:      min(ngang toàn bộ, dọc toàn bộ)

CUNG ╭:      arc(P, center=(0,0), R, θ₁=0°, θ₂=90°, w)
             = ||(|P−C| − R)|| − w/2   cắt theo góc phần tư
```

### DOUBLE (nét đôi)

```
f_double(P) = min(f(P + n·d/2), f(P − n·d/2))

n = normal vuông góc với nét,  d = khoảng cách 2 nét
═:  2 nét ngang cách nhau d
║:  2 nét dọc cách nhau d
```

### PHA (dày-mỏng mixed)

```
"DOWN HEAVY AND RIGHT LIGHT" ┌ với dọc dày + ngang mỏng:
  min(Segment(P, (0,0), (0,+1), w_heavy),
      Segment(P, (0,0), (+1,0), w_light))
```

---

## S.3 — Chữ nổi (Braille)

### Công thức: Ma trận chấm tròn

```
Braille(P, β) = min_{i: βᵢ=1} ( |P − cᵢ| − r_dot )

β = (b₁,...,b₈) ∈ {0,1}⁸       bitmask 8-bit
cᵢ = tọa độ chấm thứ i:
  c₁ = (0, 3),  c₄ = (1, 3)     hàng trên
  c₂ = (0, 2),  c₅ = (1, 2)     hàng giữa trên
  c₃ = (0, 1),  c₆ = (1, 1)     hàng giữa dưới
  c₇ = (0, 0),  c₈ = (1, 0)     hàng dưới

r_dot = bán kính mỗi chấm

Mỗi chấm = 1 sphere SDF.
Toàn bộ pattern = union (min) của các sphere bật.
cp = U+2800 + Σᵢ βᵢ · 2^(i-1)    (mã hóa trực tiếp trong Unicode)
```

---

## S.4 — APL (Functional Symbols)

### Công thức: Ký tự gốc + biến đổi

```
APL(P) = modifier( f_base(P) )

f_base: SDF của ký hiệu gốc
  α  → f_circle(P)      (alpha = vòng tròn nhỏ)
  ⎕  → f_square(P)      (quad = hình vuông)
  ◇  → f_diamond(P)     (diamond = hình thoi)
  ★  → f_star(P, 5)     (star = sao 5 cánh)
  ∇  → f_triangle(R₁₈₀·P)  (del = tam giác ngược)
  Δ  → f_triangle(P)    (delta = tam giác)
  ∘  → f_circle(P) với r nhỏ  (jot = chấm tròn nhỏ)

modifier:
  UNDERBAR:    min(f_base(P), Segment(P, (-w,−h), (w,−h), t))    gạch dưới
  DIAERESIS:   min(f_base(P), |P−(−d,h)|−r, |P−(d,h)|−r)       2 chấm trên
  TILDE:       min(f_base(P), |P.y − h − A·sin(ωPx)| − t)       dấu ngã trên
  STILE:       min(f_base(P), Segment(P, (0,−H), (0,H), t))     thanh dọc xuyên qua
  BAR:         min(f_base(P), Segment(P, (−W,0), (W,0), t))     thanh ngang xuyên qua
```

---

## S.5 — Kỹ thuật (Technical)

### Nha khoa: Box Drawing + modifier

```
Dentistry(P) = min(Segment_vert(P), Segment_horiz(P), modifier(P))

modifier:
  WITH WAVE:      y_offset = A·sin(ωx)    nét lượn sóng
  WITH CIRCLE:    min(..., f_circle(P − c_mid))
  WITH TRIANGLE:  min(..., f_triangle(P − c_mid))
```

### Điện

```
AC:     f(P) = |P.y − A·sin(ωPx)| − t          sóng sin (dòng xoay chiều)
DC:     f(P) = min(Segment_h(P), dashed_h(P))   ngang + ngang gạch
```

### Hóa học

```
Benzene(P) = f_hexagon(P, r) − t                vòng 6 cạnh
Benzene+Circle: min(f_hexagon(P,r)−t, f_circle(P)−t₂)   vòng + tròn trong
```

---

## S.6 — Khối (Block Elements)

### Công thức: Hình chữ nhật cắt theo tỷ lệ

```
Block(P, ρ, side) = f_box(P − offset, half_extents)

FULL BLOCK █:
  f(P) = f_box(P, (0.5, 0.5))                  toàn bộ ô

UPPER HALF ▀:
  f(P) = f_box(P − (0, 0.25), (0.5, 0.25))     nửa trên

LOWER HALF ▄:
  f(P) = f_box(P + (0, 0.25), (0.5, 0.25))     nửa dưới

LEFT ρ BLOCK (ρ = 1/8, 1/4, 3/8, 1/2, 5/8, 3/4, 7/8):
  f(P) = f_box(P − ((ρ−1)/2, 0), (ρ/2, 0.5))  trái ρ

SHADE (bóng):
  f_shade(P, α) = f_box(P, (0.5, 0.5))  VỚI  opacity = α
  LIGHT ░:   α = 0.25
  MEDIUM ▒:  α = 0.50
  DARK ▓:    α = 0.75

  Hoặc dither: f(P) = f_box AND (hash(P·scale) < α)

QUADRANT (góc phần tư):
  f_quad(P, mask) = min_{i ∈ mask}( f_box(P − cᵢ, (0.25, 0.25)) )
  c₀ = (−.25,+.25)  trên-trái
  c₁ = (+.25,+.25)  trên-phải
  c₂ = (−.25,−.25)  dưới-trái
  c₃ = (+.25,−.25)  dưới-phải

ARC (cung góc):
  f_arc(P, quadrant) = max(f_circle(P − corner, R), −f_box(P, (0.5,0.5)))
  Chỉ giữ phần cung nằm trong ô
```

---

## S.7 — Khác (Miscellaneous)

### Con trỏ (Pointer)

```
Pointer(P) = f_triangle(R_θ · P, h, base)      tam giác chỉ hướng
  θ phụ thuộc hướng: RIGHT=0°, LEFT=180°, UP=90°, DOWN=270°
```

### Thời tiết

```
SUN ☀:       f(P) = min(f_circle(P, r), f_star(P, n_rays, r₁, r₂))
CLOUD ☁:     f(P) = smooth_union(sphere₁, sphere₂, sphere₃, k)    3+ sphere gộp mịn
SNOWFLAKE ❄: f(P) = min_{i=0..5}(f_line(R_{60°·i} · P))           6 nét xoay 60°
UMBRELLA ☂:  f(P) = min(f_arc_top(P), f_segment_handle(P))
```

### Tôn giáo

```
CROSS ✝:     f(P) = f_cross(P, w, h₁, h₂)      chữ thập dọc dài hơn ngang
YIN YANG ☯:  f(P) = f_circle(P,r) VỚI 2 nửa S-curve: f_sigmoid chia vùng đen/trắng
STAR OF DAVID ✡: f(P) = max(f_triangle(P), f_triangle(R₁₈₀·P))   2 tam giác ngược
```

---

## Compose — Gộp hình

```
Union (gộp):        S_union(P)     = min(fA(P), fB(P))
Intersection (cắt): S_intersect(P) = max(fA(P), fB(P))
Subtraction (trừ):  S_subtract(P)  = max(fA(P), −fB(P))
Smooth union:       S_smooth(P, k) = −ln(e^(−k·fA) + e^(−k·fB)) / k

Mọi compose giữ analytical gradient:
  ∇S_union = ∇fᵢ  với i = argmin(fᵢ(P))
```

---

## Tích phân tổng: o{S}

```
o{S} = ∫_S dP = tập hợp mọi SDF trong chiều Shape

     = Union( S.0_arrows, S.1_geometric, S.2_box, S.3_braille,
              S.4_apl, S.5_tech, S.6_block, S.7_other )

     = min( f_S.0(P), f_S.1(P), ..., f_S.7(P) )
```
