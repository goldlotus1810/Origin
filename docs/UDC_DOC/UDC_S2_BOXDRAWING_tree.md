# S.2 — VẼ HỘP (Box Drawing) · Cây phân loại bằng từ ngữ

> 128 cụm từ Unicode → phân vào cây theo từ khóa
> Nhìn tên → biết "thèn này nét gì, đi hướng nào, dày mỏng ra sao"

---

## Mô hình vật lý tổng quát

> **Vẽ hộp = Topology kết nối trên lưới rời rạc**

Mỗi ô trên màn hình = 1 node trong đồ thị `G = (V, E)`.
Mỗi ký tự box drawing = **mẫu kết nối** (adjacency pattern) tại 1 node.

```
4 hướng kết nối có thể: {Up, Down, Left, Right}
Connectivity mask: C = (c_U, c_D, c_L, c_R) ∈ {0,1}⁴

Ví dụ:
  ─  = (0, 0, 1, 1)   ngang: kết nối trái-phải
  │  = (1, 1, 0, 0)   dọc: kết nối trên-dưới
  ┌  = (0, 1, 0, 1)   góc: kết nối xuống-phải
  ┼  = (1, 1, 1, 1)   giao: kết nối tất cả 4 hướng
  ├  = (1, 1, 0, 1)   T-junction: trên-dưới-phải

Tổng: 2⁴ = 16 patterns cơ bản (bao gồm ∅ = không kết nối)

Mỗi kết nối có trọng số w ∈ {light, heavy, double}:
  Adjacency matrix: A[i][j] = w(edge(i,j))
```

Đồ thị lưới hoàn chỉnh với `n × m` ô có:
- `|V| = n·m` đỉnh
- `|E| ≤ 2·n·m − n − m` cạnh (lưới 4-connected đầy đủ)
- Degree tối đa: `deg(v) ≤ 4` cho mọi `v ∈ V`

---

## Tầng 1: "Nét đi hướng nào?"

```
VẼ HỘP
├── NGANG        "horizontal" — nét ngang ─
├── DỌC          "vertical" — nét dọc │
├── GÓC          "down and right" / "up and left" / ... — góc ┌ ┐ └ ┘
├── GIAO         "vertical and horizontal" — giao cắt ┼
├── NỬA          "left" / "right" (T-junction) — nối 3 hướng ├ ┤ ┬ ┴
└── CUNG         "arc" — bo góc ╭ ╮ ╰ ╯
```

### Công thức topology từng kiểu nét

**NGANG (horizontal):**
```
C = (0, 0, 1, 1)     degree = 2, tuyến tính (path graph P₂)
SDF: f(x,y) = |y| − t/2  (đường ngang, độ dày t)
  f < 0 → bên trong nét, f > 0 → bên ngoài, f = 0 → biên
```

**DỌC (vertical):**
```
C = (1, 1, 0, 0)     degree = 2, tuyến tính (path graph P₂, xoay 90°)
SDF: f(x,y) = |x| − t/2  (đường dọc)
  Phép quay: R(90°)·[ngang] = [dọc], hay (c_L,c_R) ↔ (c_U,c_D)
```

**GÓC (corner):**
```
C = (0, 1, 0, 1) cho ┌     degree = 2, hình chữ L
SDF: f(p⃗) = min(f_horizontal(p⃗), f_vertical(p⃗))  (hợp 2 nửa đường thẳng)
Độ cong κ = 0 mọi nơi, ngoại trừ tại góc: κ → ∞ (gián đoạn C⁰)
  4 góc = 4 phép quay: ┌ ┐ └ ┘ ↔ R(0°), R(90°), R(270°), R(180°)
```

**GIAO (cross):**
```
C = (1, 1, 1, 1)     degree = 4, đồ thị lưỡng phân đầy đủ K₂,₂
Đặc trưng Euler: χ = V − E + F  (cho vùng kín bao quanh)
  Tại giao cắt ┼: 4 cạnh tạo 4 vùng → χ phản ánh topology mặt phẳng
```

**NỬA — T-junction:**
```
C = (1, 1, 0, 1) cho ├     degree = 3
Lý thuyết đồ thị: T-junction = đỉnh bậc 3
Định luật Kirchhoff (bảo toàn dòng): Σ Iᵢ = 0 tại nút rẽ nhánh
  Ứng dụng: dòng thông tin qua layout tuân thủ bảo toàn flow
```

**CUNG (arc):**
```
C = (0, 1, 0, 1) cho ╭     cùng connectivity như góc, NHƯNG:
Độ cong κ = 1/r (cong đều, liên tục C¹ — mượt hơn góc vuông)
Tham số hóa: p⃗(t) = center + r·(cos(t), sin(t))  với t ∈ [θ₁, θ₂]
  ╭: t ∈ [π, 3π/2]    ╮: t ∈ [3π/2, 2π]
  ╰: t ∈ [π/2, π]     ╯: t ∈ [0, π/2]
```

---

## Tầng 2: "Nét dày mỏng ra sao?"

```
ĐỘ DÀY
├── MỎNG        "light" — nét mảnh
├── DÀY         "heavy" — nét đậm
├── ĐÔI         "double" — nét kép ═ ║
└── PHA          "heavy and light" / "light and heavy" — pha trộn dày mỏng
```

### Công thức trọng số (weight)

**MỎNG (light):**
```
w = 1, độ rộng nét = 1px
SDF thickness: t = t_light (đơn vị cơ sở)
```

**DÀY (heavy):**
```
w = 2, độ rộng nét = 2px (gấp đôi mỏng)
SDF thickness: t = t_heavy = 2·t_light
```

**ĐÔI (double):**
```
Hai đường song song, khoảng cách g:
SDF: f(p) = min(|d − g/2|, |d + g/2|) − t/2
  với d = khoảng cách có dấu từ p đến trục chính
  Mỗi nét con có độ dày t = t_light
```

**PHA (anisotropic):**
```
Trọng số khác nhau theo hướng = đồ thị bất đẳng hướng (anisotropic graph)
  Ví dụ ├ (dọc dày + ngang mỏng):
    w_vertical = 2, w_horizontal = 1
    Adjacency matrix: A[i][j] = w_direction(edge(i,j))
  Ma trận trọng số không đối xứng theo hướng: W_U ≠ W_L có thể xảy ra
```

---

## Tầng 3: "Góc cụ thể nào?"

```
GÓC (chỉ áp dụng cho kiểu GÓC + NỬA + GIAO)
├── XUỐNG-PHẢI   "down and right" — ┌
├── XUỐNG-TRÁI   "down and left" — ┐
├── LÊN-PHẢI     "up and right" — └
├── LÊN-TRÁI     "up and left" — ┘
├── DỌC-PHẢI     "vertical and right" — ├
├── DỌC-TRÁI     "vertical and left" — ┤
├── NGANG-XUỐNG  "down and horizontal" — ┬
├── NGANG-LÊN    "up and horizontal" — ┴
└── GIAO ĐẦY     "vertical and horizontal" — ┼
```

---

## Phân loại cụ thể 128 cụm

### NGANG (horizontal) — ~6 cụm

```
BOX DRAWINGS LIGHT HORIZONTAL             → ngang + mỏng  ─
BOX DRAWINGS HEAVY HORIZONTAL             → ngang + dày   ━
BOX DRAWINGS DOUBLE HORIZONTAL            → ngang + đôi   ═
BOX DRAWINGS LIGHT TRIPLE DASH HORIZONTAL → ngang + mỏng + gạch 3
BOX DRAWINGS HEAVY TRIPLE DASH HORIZONTAL → ngang + dày + gạch 3
BOX DRAWINGS LIGHT QUADRUPLE DASH HORIZONTAL → ngang + mỏng + gạch 4
→ "ah, nét ngang [mỏng/dày/đôi] [kiểu gạch]"
```

### DỌC (vertical) — ~6 cụm

```
BOX DRAWINGS LIGHT VERTICAL               → dọc + mỏng  │
BOX DRAWINGS HEAVY VERTICAL               → dọc + dày   ┃
BOX DRAWINGS DOUBLE VERTICAL              → dọc + đôi   ║
BOX DRAWINGS LIGHT TRIPLE DASH VERTICAL   → dọc + mỏng + gạch 3
BOX DRAWINGS HEAVY TRIPLE DASH VERTICAL   → dọc + dày + gạch 3
→ "ah, nét dọc [mỏng/dày/đôi]"
```

### GÓC (corner) — ~40 cụm

**Góc xuống-phải (┌):**
```
BOX DRAWINGS LIGHT DOWN AND RIGHT         → ┌ mỏng
BOX DRAWINGS HEAVY DOWN AND RIGHT         → ┌ dày
BOX DRAWINGS DOUBLE DOWN AND RIGHT        → ┌ đôi
BOX DRAWINGS DOWN LIGHT AND RIGHT HEAVY   → ┌ dọc mỏng + ngang dày
BOX DRAWINGS DOWN HEAVY AND RIGHT LIGHT   → ┌ dọc dày + ngang mỏng
BOX DRAWINGS DOWN DOUBLE AND RIGHT SINGLE → ┌ dọc đôi + ngang đơn
BOX DRAWINGS DOWN SINGLE AND RIGHT DOUBLE → ┌ dọc đơn + ngang đôi
→ "ah, góc xuống-phải [dày/mỏng/đôi/pha]"
```

**Góc xuống-trái (┐):**
```
BOX DRAWINGS LIGHT DOWN AND LEFT          → ┐ mỏng
BOX DRAWINGS HEAVY DOWN AND LEFT          → ┐ dày
BOX DRAWINGS DOUBLE DOWN AND LEFT         → ┐ đôi
BOX DRAWINGS DOWN LIGHT AND LEFT HEAVY    → ┐ pha
BOX DRAWINGS DOWN HEAVY AND LEFT LIGHT    → ┐ pha
→ "ah, góc xuống-trái [dày/mỏng/đôi/pha]"
```

**Góc lên-phải (└):**
```
BOX DRAWINGS LIGHT UP AND RIGHT           → └ mỏng
BOX DRAWINGS HEAVY UP AND RIGHT           → └ dày
BOX DRAWINGS DOUBLE UP AND RIGHT          → └ đôi
BOX DRAWINGS UP LIGHT AND RIGHT HEAVY     → └ pha
BOX DRAWINGS UP HEAVY AND RIGHT LIGHT     → └ pha
→ "ah, góc lên-phải [dày/mỏng/đôi/pha]"
```

**Góc lên-trái (┘):**
```
BOX DRAWINGS LIGHT UP AND LEFT            → ┘ mỏng
BOX DRAWINGS HEAVY UP AND LEFT            → ┘ dày
BOX DRAWINGS DOUBLE UP AND LEFT           → ┘ đôi
BOX DRAWINGS UP LIGHT AND LEFT HEAVY      → ┘ pha
BOX DRAWINGS UP HEAVY AND LEFT LIGHT      → ┘ pha
→ "ah, góc lên-trái [dày/mỏng/đôi/pha]"
```

### T-JUNCTION (nửa) — ~40 cụm

**T dọc-phải (├):**
```
BOX DRAWINGS LIGHT VERTICAL AND RIGHT     → ├ mỏng
BOX DRAWINGS HEAVY VERTICAL AND RIGHT     → ├ dày
BOX DRAWINGS DOUBLE VERTICAL AND RIGHT    → ├ đôi
BOX DRAWINGS VERTICAL LIGHT AND RIGHT HEAVY → ├ pha
BOX DRAWINGS VERTICAL HEAVY AND RIGHT LIGHT → ├ pha
→ "ah, T dọc-phải [dày/mỏng/đôi/pha]"
```

**T dọc-trái (┤):**
```
BOX DRAWINGS LIGHT VERTICAL AND LEFT      → ┤ mỏng
BOX DRAWINGS HEAVY VERTICAL AND LEFT      → ┤ dày
BOX DRAWINGS DOUBLE VERTICAL AND LEFT     → ┤ đôi
→ "ah, T dọc-trái [dày/mỏng/đôi/pha]"
```

**T ngang-xuống (┬):**
```
BOX DRAWINGS LIGHT DOWN AND HORIZONTAL    → ┬ mỏng
BOX DRAWINGS HEAVY DOWN AND HORIZONTAL    → ┬ dày
BOX DRAWINGS DOUBLE DOWN AND HORIZONTAL   → ┬ đôi
BOX DRAWINGS DOWN HEAVY AND HORIZONTAL LIGHT → ┬ pha
→ "ah, T ngang-xuống [dày/mỏng/đôi/pha]"
```

**T ngang-lên (┴):**
```
BOX DRAWINGS LIGHT UP AND HORIZONTAL      → ┴ mỏng
BOX DRAWINGS HEAVY UP AND HORIZONTAL      → ┴ dày
BOX DRAWINGS DOUBLE UP AND HORIZONTAL     → ┴ đôi
→ "ah, T ngang-lên [dày/mỏng/đôi/pha]"
```

### GIAO CẮT (cross) — ~12 cụm

```
BOX DRAWINGS LIGHT VERTICAL AND HORIZONTAL     → ┼ mỏng
BOX DRAWINGS HEAVY VERTICAL AND HORIZONTAL     → ┼ dày
BOX DRAWINGS DOUBLE VERTICAL AND HORIZONTAL    → ┼ đôi
BOX DRAWINGS VERTICAL LIGHT AND HORIZONTAL HEAVY → ┼ dọc mỏng + ngang dày
BOX DRAWINGS VERTICAL HEAVY AND HORIZONTAL LIGHT → ┼ dọc dày + ngang mỏng
BOX DRAWINGS VERTICAL DOUBLE AND HORIZONTAL SINGLE → ┼ dọc đôi + ngang đơn
BOX DRAWINGS VERTICAL SINGLE AND HORIZONTAL DOUBLE → ┼ dọc đơn + ngang đôi
→ "ah, giao cắt [dày/mỏng/đôi/pha]"
```

### CUNG (arc) — ~4 cụm

```
BOX DRAWINGS LIGHT ARC DOWN AND RIGHT     → ╭ bo góc mỏng
BOX DRAWINGS LIGHT ARC DOWN AND LEFT      → ╮ bo góc mỏng
BOX DRAWINGS LIGHT ARC UP AND LEFT        → ╯ bo góc mỏng
BOX DRAWINGS LIGHT ARC UP AND RIGHT       → ╰ bo góc mỏng
→ "ah, bo góc [hướng]"
```

### ĐẶC BIỆT — ~8 cụm

```
BOX DRAWINGS LIGHT DIAGONAL UPPER RIGHT TO LOWER LEFT  → chéo ╲
BOX DRAWINGS LIGHT DIAGONAL CROSS                       → chéo cắt ╳
BOX DRAWINGS LIGHT LEFT                                 → nửa ngang trái
BOX DRAWINGS LIGHT RIGHT                                → nửa ngang phải
BOX DRAWINGS LIGHT UP                                   → nửa dọc trên
BOX DRAWINGS LIGHT DOWN                                 → nửa dọc dưới
→ "ah, nét đặc biệt [chéo/nửa]"
```

---

## Từ khóa → Tầng (cheat sheet)

| Thấy từ này | → Tầng | → Giá trị |
|-------------|--------|----------|
| horizontal | 1 | NGANG |
| vertical | 1 | DỌC |
| down and right/left, up and right/left | 1 | GÓC |
| vertical and horizontal | 1 | GIAO |
| vertical and right/left | 1 | T-DỌC |
| down/up and horizontal | 1 | T-NGANG |
| arc | 1 | CUNG |
| light | 2 | MỎNG |
| heavy | 2 | DÀY |
| double | 2 | ĐÔI |
| X light and Y heavy | 2 | PHA (X mỏng Y dày) |

---

## Tóm tắt

```
128 cụm box drawing → tuple (kiểu_nét, độ_dày, hướng_góc)

"BOX DRAWINGS" + [LIGHT/HEAVY/DOUBLE] + [hướng]
→ "ah, vẽ hộp [mỏng/dày/đôi] [ngang/dọc/góc/T/giao/cung]"

Mọi cụm đều theo pattern:
  BOX DRAWINGS {weight} {direction}
  BOX DRAWINGS {dir1} {weight1} AND {dir2} {weight2}
```

### Tổng kết công thức

```
Mô hình:  G = (V, E) trên lưới rời rạc n × m
Node:     Mỗi ô → connectivity mask C = (c_U, c_D, c_L, c_R) ∈ {0,1}⁴
Edge:     Trọng số w ∈ {1=light, 2=heavy, double=2×parallel}
Hình học: SDF f(p⃗) xác định biên nét — f<0 trong, f>0 ngoài, f=0 biên

6 kiểu nét → 6 dạng topology:
  NGANG/DỌC:  deg=2, path graph P₂, SDF = |coord| − t/2
  GÓC:        deg=2, L-shape, κ=0 (gián đoạn C⁰ tại góc)
  GIAO:       deg=4, K₂,₂, χ = V−E+F
  T-junction: deg=3, bảo toàn flow Σ Iᵢ = 0
  CUNG:       deg=2, κ=1/r (liên tục C¹), p⃗(t) = center + r·(cos t, sin t)

3+1 trọng số:
  light: t = t₀          heavy: t = 2·t₀
  double: f = min(|d−g/2|, |d+g/2|) − t/2
  pha: W bất đẳng hướng theo direction
```
