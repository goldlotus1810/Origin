# S.1 — HÌNH HỌC (Geometric) · Cây phân loại bằng từ ngữ

> 321 cụm từ Unicode → phân vào cây theo từ khóa
> Nhìn tên → biết ngay "thèn này hình gì, tô kiểu gì, kích cỡ ra sao"

---

## Mô hình vật lý tổng quát

```
Hình học = Signed Distance Field (SDF)

  f(p⃗) = khoảng cách có dấu từ điểm p⃗ đến biên hình
    f(p⃗) < 0  → p⃗ nằm TRONG hình (filled/black)
    f(p⃗) = 0  → p⃗ nằm TRÊN biên (outline/white)
    f(p⃗) > 0  → p⃗ nằm NGOÀI hình

  Mọi hình = 1 hàm SDF duy nhất.
  Boolean operations: union = min(f₁,f₂), intersect = max(f₁,f₂), subtract = max(f₁,−f₂)
```

> Mỗi hình ở Tầng 1 được định nghĩa bởi một phương trình ẩn f(x,y)=0.
> Vùng f<0 là bên trong, f>0 là bên ngoài. Tầng 2 (tô) quyết định render vùng nào.
> Tầng 3 (kích cỡ) là phép co giãn (scale) trên SDF.

---

## Tầng 1: "Nó là hình gì?"

```
HÌNH HỌC
├── TRÒN         "circle" — hình tròn ● ○
│     SDF: f(x,y) = √(x²+y²) − r = ||p⃗|| − r
├── VUÔNG        "square" — hình vuông ■ □
│     SDF: f(x,y) = max(|x|,|y|) − a = ||p⃗||_∞ − a
├── TAM GIÁC     "triangle" — 3 cạnh ▲ △
│     SDF: f(p⃗) = max(n⃗ᵢ·(p⃗−vᵢ)) cho i=1,2,3
├── KIM CƯƠNG    "diamond / lozenge / rhombus" — thoi ◆ ◇
│     SDF: f(x,y) = |x|/a + |y|/b − 1 = ||p⃗||₁ − 1
├── SAO          "star" — nhiều cánh ★ ☆
│     SDF polar: r(θ) = R·cos(π/n) / cos(θ mod 2π/n − π/n)
├── CHỮ THẬP     "cross / saltire / maltese" — dấu cộng ✚ ✝
│     SDF: f(x,y) = min(max(|x|−a,|y|−b), max(|x|−b,|y|−a))
├── HÌNH NHIỀU CẠNH "pentagon / hexagon / octagon" — 5+ cạnh ⬠ ⎔
├── HÌNH ELIP    "ellipse" — bầu dục ⬮ ⬯
│     SDF: f(x,y) ≈ (x²/a² + y²/b² − 1)·min(a,b)
├── HÌNH CHỮ NHẬT "rectangle / parallelogram / trapezium" — 4 cạnh không đều
└── HOA          "florette / pinwheel / propeller / petalled" — hoa lá ✿ ❀
      SDF polar: r(θ) = R + A·cos(nθ)
```

---

## Tầng 2: "Nó tô kiểu gì?"

```
KIỂU TÔ (fill)
├── ĐẶC          "black / filled" — tô kín ● ■ ▲
├── RỖNG         "white / open / outline" — chỉ viền ○ □ △
├── CHẤM         "dotted" — viền chấm
├── NỬA          "half" — tô nửa ◐ ◑
├── CHÉO         "with diagonal" — có vạch chéo bên trong
└── CÓ HÌNH TRONG "containing / with ... inside" — có ký tự bên trong ⊕ ⊗
```

---

## Tầng 3: "Kích cỡ / Nét dày mỏng?"

```
KÍCH CỠ
├── NHỎ          "small" — nhỏ hơn bình thường
├── THƯỜNG       (mặc định) — kích thước chuẩn
├── LỚN          "large / very large" — to hơn
├── MỎNG         "light" — nét mảnh
└── DÀY          "heavy / bold" — nét đậm
```

---

## Tầng 4: "Nó có modifier gì?"

```
MODIFIER
├── HƯỚNG         "pointing" — ▶ (right-pointing) ◀ (left-pointing) ▲ (up) ▼ (down)
├── VIỀN TRÒN     "circled" — ⊙ ⊕ ⊗ ký tự trong vòng tròn
├── CÁNH/SỐ CÁNH  "pointed / spoked / petalled" — sao 4 cánh, 6 cánh, 8 cánh
├── TRUNG TÂM     "centred / centre" — có điểm/hình ở giữa
├── XOAY          "rotated / turned" — xoay 45° 90°
└── CẮT/GHÉP      "minus / with ..." — bị cắt hoặc ghép thêm
```

---

## Phân loại cụ thể 321 cụm

### TRÒN (circle) — ~60 cụm

```
BLACK CIRCLE                              → tròn + đặc
WHITE CIRCLE                              → tròn + rỗng
LARGE CIRCLE                              → tròn + lớn
MEDIUM BLACK CIRCLE                       → tròn + đặc + vừa
MEDIUM WHITE CIRCLE                       → tròn + rỗng + vừa
BLACK CIRCLE FOR RECORD                   → tròn + đặc + chức năng (record)
BLACK CIRCLE WITH WHITE VERTICAL BAR      → tròn + đặc + có thanh trắng bên trong
BULLSEYE                                  → tròn + nhiều vòng lồng
FISHEYE                                   → tròn + đặc + có chấm trắng
INVERSE BULLET                            → tròn + đảo ngược
→ "ah, hình tròn [đặc/rỗng] [cỡ]"

APL FUNCTIONAL SYMBOL CIRCLE BACKSLASH    → tròn + có \ bên trong
APL FUNCTIONAL SYMBOL CIRCLE DIAERESIS    → tròn + có ¨ bên trong
APL FUNCTIONAL SYMBOL CIRCLE JOT          → tròn + có ○ bên trong
APL FUNCTIONAL SYMBOL CIRCLE STAR         → tròn + có ★ bên trong
APL FUNCTIONAL SYMBOL CIRCLE STILE        → tròn + có | bên trong
APL FUNCTIONAL SYMBOL CIRCLE UNDERBAR     → tròn + có _ bên trong
APL FUNCTIONAL SYMBOL QUAD CIRCLE         → vuông + có ○ bên trong
→ "ah, hình tròn APL, có ký tự bên trong"
```

**Công thức SDF — TRÒN:**
```
  f(x,y) = √(x²+y²) − r
  = ||p⃗|| − r    (Euclidean norm trừ bán kính)

  Tâm tại gốc, bán kính r.
  Đây là SDF đơn giản nhất — khoảng cách Euclid đến tâm trừ r.
```

### VUÔNG (square) — ~40 cụm

```
BLACK SQUARE                              → vuông + đặc
WHITE SQUARE                              → vuông + rỗng
BLACK MEDIUM SQUARE                       → vuông + đặc + vừa
WHITE MEDIUM SQUARE                       → vuông + rỗng + vừa
BLACK SMALL SQUARE                        → vuông + đặc + nhỏ
WHITE SMALL SQUARE                        → vuông + rỗng + nhỏ
BLACK LARGE SQUARE                        → vuông + đặc + lớn
WHITE LARGE SQUARE                        → vuông + rỗng + lớn
BLACK VERY SMALL SQUARE                   → vuông + đặc + rất nhỏ
WHITE VERY SMALL SQUARE                   → vuông + rỗng + rất nhỏ
BLACK SQUARE CENTRED                      → vuông + đặc + có tâm
SQUARE WITH BOTTOM HALF BLACK             → vuông + nửa dưới đặc
SQUARE WITH DIAGONAL CROSSHATCH FILL      → vuông + tô chéo
SQUARE WITH LEFT HALF BLACK               → vuông + nửa trái đặc
SQUARE WITH LOWER RIGHT DIAGONAL HALF BLACK → vuông + nửa chéo dưới đặc
SQUARE WITH RIGHT HALF BLACK              → vuông + nửa phải đặc
SQUARE WITH TOP HALF BLACK                → vuông + nửa trên đặc
SQUARE WITH UPPER LEFT DIAGONAL HALF BLACK → vuông + nửa chéo trên đặc
→ "ah, hình vuông [đặc/rỗng] [cỡ] [tô kiểu gì]"
```

**Công thức SDF — VUÔNG:**
```
  f(x,y) = max(|x|, |y|) − a
  = ||p⃗||_∞ − a    (Chebyshev / L∞ norm)

  Tâm tại gốc, cạnh = 2a.
  Chebyshev norm: khoảng cách = max của tọa độ tuyệt đối.
```

### TAM GIÁC (triangle) — ~40 cụm

```
BLACK UP-POINTING TRIANGLE                → tam giác + đặc + hướng lên
WHITE UP-POINTING TRIANGLE                → tam giác + rỗng + hướng lên
BLACK DOWN-POINTING TRIANGLE              → tam giác + đặc + hướng xuống
WHITE DOWN-POINTING TRIANGLE              → tam giác + rỗng + hướng xuống
BLACK LEFT-POINTING TRIANGLE              → tam giác + đặc + hướng trái
WHITE LEFT-POINTING TRIANGLE              → tam giác + rỗng + hướng trái
BLACK RIGHT-POINTING TRIANGLE             → tam giác + đặc + hướng phải
WHITE RIGHT-POINTING TRIANGLE             → tam giác + rỗng + hướng phải
BLACK DOWN-POINTING DOUBLE TRIANGLE       → tam giác đôi + đặc + xuống
BLACK UP-POINTING DOUBLE TRIANGLE         → tam giác đôi + đặc + lên
BLACK MEDIUM UP-POINTING TRIANGLE         → tam giác + đặc + vừa + lên
BLACK MEDIUM DOWN-POINTING TRIANGLE       → tam giác + đặc + vừa + xuống
BLACK MEDIUM LEFT-POINTING TRIANGLE       → tam giác + đặc + vừa + trái
BLACK MEDIUM RIGHT-POINTING TRIANGLE      → tam giác + đặc + vừa + phải
BLACK MEDIUM UP-POINTING TRIANGLE CENTRED → tam giác + đặc + vừa + lên + có tâm
BLACK SMALL UP-POINTING TRIANGLE          → tam giác + đặc + nhỏ + lên
WHITE SMALL UP-POINTING TRIANGLE          → tam giác + rỗng + nhỏ + lên
→ "ah, tam giác [đặc/rỗng] [hướng] [cỡ]"
```

**Công thức SDF — TAM GIÁC:**
```
  f(p⃗) = max(n⃗ᵢ · (p⃗ − vᵢ))  cho i=1,2,3
  (giao của 3 nửa mặt phẳng, mỗi cạnh có pháp tuyến ngoài n⃗ᵢ và đỉnh vᵢ)

  Tam giác đều cạnh a, tâm tại gốc:
    v₁ = (0, a/√3),  v₂ = (−a/2, −a/(2√3)),  v₃ = (a/2, −a/(2√3))
  Hướng pointing thay đổi bằng phép xoay các đỉnh.
```

### KIM CƯƠNG (diamond) — ~20 cụm

```
BLACK DIAMOND                             → kim cương + đặc
WHITE DIAMOND                             → kim cương + rỗng
BLACK DIAMOND CENTRED                     → kim cương + đặc + có tâm
BLACK DIAMOND MINUS WHITE X               → kim cương + đặc + có X rỗng
BLACK DIAMOND ON CROSS                    → kim cương + trên chữ thập
DIAMOND WITH LEFT HALF BLACK              → kim cương + nửa trái đặc
DIAMOND WITH RIGHT HALF BLACK             → kim cương + nửa phải đặc
DIAMOND WITH TOP HALF BLACK               → kim cương + nửa trên đặc
DIAMOND WITH BOTTOM HALF BLACK            → kim cương + nửa dưới đặc
WHITE DIAMOND CONTAINING BLACK SMALL DIAMOND → kim cương rỗng + có kim cương nhỏ đặc bên trong
LOZENGE                                   → hình thoi (= kim cương)
BLACK LOZENGE                             → hình thoi + đặc
→ "ah, hình thoi/kim cương [đặc/rỗng] [modifier]"
```

**Công thức SDF — KIM CƯƠNG:**
```
  f(x,y) = |x|/a + |y|/b − 1
  = ||p⃗||₁/(a,b) − 1    (L1 norm, có co giãn theo trục)

  Trường hợp đặc biệt a=b: hình vuông xoay 45° = ||p⃗||₁ − a
  Kim cương = hình thoi với bán trục ngang a, bán trục dọc b.
```

### SAO (star) — ~25 cụm

```
BLACK STAR                                → sao + đặc
WHITE STAR                                → sao + rỗng
BLACK CENTRE WHITE STAR                   → sao + rỗng + tâm đặc
OUTLINED BLACK STAR                       → sao + đặc + có viền
PINWHEEL STAR                             → sao + xoay
STRESS OUTLINED WHITE STAR                → sao + rỗng + viền nhấn
FOUR POINTED BLACK STAR                   → sao + đặc + 4 cánh
SIX POINTED BLACK STAR                    → sao + đặc + 6 cánh
EIGHT POINTED BLACK STAR                  → sao + đặc + 8 cánh
TWELVE POINTED BLACK STAR                 → sao + đặc + 12 cánh
SIX POINTED STAR WITH MIDDLE DOT          → sao + 6 cánh + chấm giữa
EIGHT SPOKED ASTERISK                     → sao + 8 nan
SIX SPOKED ASTERISK                       → sao + 6 nan
FOUR BALLOON-SPOKED ASTERISK              → sao + 4 nan tròn
HEAVY EIGHT POINTED RECTILINEAR BLACK STAR → sao + đặc + 8 cánh + vuông + dày
→ "ah, hình sao [đặc/rỗng] [số cánh] [modifier]"
```

**Công thức SDF — SAO:**
```
  Polar SDF: r(θ) = R · cos(π/n) / cos(θ mod 2π/n − π/n)
  n = số cánh (number of points)

  n=4: sao 4 cánh (four-pointed star)
  n=5: ngôi sao 5 cánh (pentagram)
  n=6: Star of David (hexagram)
  n=8: octagram (sao 8 cánh)

  R = bán kính ngoài, r_inner = bán kính trong (đáy cánh)
  Tỉ lệ R/r_inner quyết định độ nhọn của cánh.
```

### CHỮ THẬP (cross) — ~15 cụm

```
MALTESE CROSS                             → chữ thập Malta
LATIN CROSS                               → chữ thập Latin
ORTHODOX CROSS                            → chữ thập chính thống
SALTIRE (St Andrew cross)                 → chữ X / chéo
WHITE CROSS ON RED CIRCLE                 → chữ thập trên nền đỏ
→ "ah, chữ thập [kiểu]"
```

**Công thức SDF — CHỮ THẬP:**
```
  f(x,y) = min(max(|x|−a, |y|−b), max(|x|−b, |y|−a))
  (hợp — union — của 2 hình chữ nhật vuông góc)

  a = nửa chiều dài thanh, b = nửa chiều rộng thanh (a > b).
  Saltire (chữ X): xoay 45° → áp dụng rotation trước khi tính SDF.
```

### HOA (florette/decorative) — ~15 cụm

```
BLACK FLORETTE                            → hoa + đặc
WHITE FLORETTE                            → hoa + rỗng
SIX PETALLED BLACK AND WHITE FLORETTE     → hoa + 6 cánh + nửa đặc nửa rỗng
FOUR TEARDROP-SPOKED ASTERISK             → hoa + 4 cánh giọt nước
HEAVY FOUR BALLOON-SPOKED ASTERISK        → hoa + 4 cánh bóng + dày
PINWHEEL STAR                             → chong chóng
PROPELLER                                 → cánh quạt
→ "ah, hoa/trang trí [đặc/rỗng] [số cánh]"
```

**Công thức SDF — HOA:**
```
  r(θ) = R + A·cos(nθ)    (đường cong hoa hồng — polar rose curve)
  n = số cánh, A = biên độ cánh, R = bán kính cơ sở

  Florette: n=6, pinwheel: thêm xoắn φ(r), propeller: n=3 + xoắn.
  Biến thể: r(θ) = R + A·|cos(nθ/2)| cho cánh tròn hơn.
```

### HÌNH NHIỀU CẠNH (polygon) — ~10 cụm

```
PENTAGON                                  → 5 cạnh
BLACK HORIZONTAL ELLIPSE                  → elip ngang + đặc
WHITE HORIZONTAL ELLIPSE                  → elip ngang + rỗng
BLACK PARALLELOGRAM                       → hình bình hành + đặc
BLACK RECTANGLE                           → hình chữ nhật + đặc
WHITE RECTANGLE                           → hình chữ nhật + rỗng
→ "ah, hình [tên] [đặc/rỗng]"
```

**Công thức SDF — HÌNH ELIP:**
```
  f(x,y) ≈ (x²/a² + y²/b² − 1) · min(a,b)
  (SDF xấp xỉ qua phương trình ẩn, nhân min(a,b) để chuẩn hóa gradient)

  a = bán trục ngang, b = bán trục dọc.
  Khi a=b → tròn. SDF chính xác cho ellipse phức tạp hơn nhiều
  (cần giải phương trình bậc 4), công thức trên là xấp xỉ thực dụng.
```

---

## Công thức SDF cho Tầng 2 — Kiểu tô (fill)

> Tầng 2 không thay đổi hàm SDF, mà quyết định **vùng nào được render**.

```
Cho hàm SDF f(p⃗) của hình bất kỳ ở Tầng 1:

  ĐẶC (filled/black):
    render khi f(p⃗) ≤ 0
    → Tô kín toàn bộ vùng bên trong hình.

  RỖNG (outline/white):
    render khi |f(p⃗)| < ε    (ε = độ dày viền)
    → Chỉ vẽ đường biên, bỏ trống bên trong.

  NỬA (half):
    render khi f(p⃗) ≤ 0 AND (x > 0 hoặc y > 0)
    → Tô nửa hình. Điều kiện phụ chọn nửa nào:
      LEFT HALF BLACK:   f(p⃗) ≤ 0 AND x ≤ 0
      RIGHT HALF BLACK:  f(p⃗) ≤ 0 AND x ≥ 0
      TOP HALF BLACK:    f(p⃗) ≤ 0 AND y ≥ 0
      BOTTOM HALF BLACK: f(p⃗) ≤ 0 AND y ≤ 0

  CHẤM (dotted):
    render khi |f(p⃗)| < ε AND fract(arc_length/spacing) < duty
    → Viền gián đoạn (dashed/dotted pattern dọc biên).

  CÓ HÌNH TRONG (containing):
    render khi f_outer(p⃗) ≤ 0 AND f_inner(p⃗) > 0
    = max(f_outer, −f_inner)  → Boolean subtract
    → Hình ngoài trừ hình trong (⊕ ⊗).
```

---

## Công thức SDF cho Tầng 3 — Kích cỡ (scale)

> Tầng 3 là phép co giãn đồng dạng trên SDF.

```
  Scale transform:
    f_scaled(p⃗) = f(p⃗ / s) · s
    với s = hệ số co giãn (scale factor)

  Các cỡ trong Unicode geometric:
    NHỎ (small):       s < 1.0   (ví dụ s = 0.6)
    THƯỜNG (default):  s = 1.0
    LỚN (large):       s > 1.0   (ví dụ s = 1.5)
    RẤT LỚN (very large): s >> 1.0 (ví dụ s = 2.0)

  Nét dày/mỏng (stroke weight):
    MỎNG (light): ε nhỏ → viền mảnh
    DÀY (heavy/bold): ε lớn → viền đậm
    Với outline: render |f(p⃗)| < ε, thay đổi ε = thay đổi nét.
    Với filled: dilation f(p⃗) − δ (phình ra δ pixel).
```

---

## Từ khóa → Tầng (cheat sheet)

| Thấy từ này | → Tầng | → Giá trị |
|-------------|--------|----------|
| circle | 1 | TRÒN |
| square | 1 | VUÔNG |
| triangle | 1 | TAM GIÁC |
| diamond, lozenge, rhombus | 1 | KIM CƯƠNG |
| star, asterisk | 1 | SAO |
| cross, saltire, maltese | 1 | CHỮ THẬP |
| pentagon, hexagon, octagon | 1 | NHIỀU CẠNH |
| ellipse | 1 | ELIP |
| rectangle, parallelogram | 1 | CHỮ NHẬT |
| florette, pinwheel, propeller | 1 | HOA |
| black, filled | 2 | ĐẶC |
| white, open, outline | 2 | RỖNG |
| half | 2 | NỬA |
| dotted | 2 | CHẤM |
| containing, with...inside | 2 | CÓ HÌNH TRONG |
| small, very small | 3 | NHỎ |
| medium | 3 | VỪA |
| large, very large | 3 | LỚN |
| light | 3 | MỎNG |
| heavy, bold | 3 | DÀY |
| pointing, up/down/left/right | 4 | HƯỚNG |
| circled | 4 | VIỀN TRÒN |
| pointed, spoked, petalled | 4 | SỐ CÁNH |
| centred, centre | 4 | CÓ TÂM |
| rotated, turned | 4 | XOAY |

---

## Tóm tắt

```
321 cụm hình học → tuple (hình, tô, cỡ, modifier)

Bước 1: Tìm HÌNH   → circle? square? triangle? diamond? star? cross? florette?
Bước 2: Tìm TÔ     → black? white? half? dotted?
Bước 3: Tìm CỠ     → small? medium? large? heavy? light?
Bước 4: Tìm MODIFIER → pointing? centred? spoked? containing?

→ "ah cái thèn này là [hình gì] [tô kiểu gì] [cỡ nào] [có gì đặc biệt]"
```

### Bảng công thức SDF tổng hợp

| Hình | SDF f(p⃗) | Norm | Tham số |
|------|----------|------|---------|
| TRÒN | √(x²+y²) − r | L2 (Euclid) | r = bán kính |
| VUÔNG | max(\|x\|,\|y\|) − a | L∞ (Chebyshev) | a = nửa cạnh |
| TAM GIÁC | max(n⃗ᵢ·(p⃗−vᵢ)) | giao nửa phẳng | vᵢ = đỉnh, n⃗ᵢ = pháp tuyến |
| KIM CƯƠNG | \|x\|/a+\|y\|/b − 1 | L1 (Manhattan) | a,b = bán trục |
| SAO | R·cos(π/n)/cos(θ mod 2π/n−π/n) | polar | n = số cánh, R = bán kính |
| CHỮ THẬP | min(max(\|x\|−a,\|y\|−b), max(\|x\|−b,\|y\|−a)) | union rect | a,b = kích thước thanh |
| ELIP | (x²/a²+y²/b²−1)·min(a,b) | xấp xỉ | a,b = bán trục |
| HOA | R+A·cos(nθ) | polar rose | n = cánh, A = biên độ |

| Tầng | Thao tác | Công thức |
|------|----------|-----------|
| Tầng 2: ĐẶC | render inside | f(p⃗) ≤ 0 |
| Tầng 2: RỖNG | render biên | \|f(p⃗)\| < ε |
| Tầng 2: NỬA | render nửa | f(p⃗) ≤ 0 AND điều kiện nửa |
| Tầng 3: CỠ | co giãn | f(p⃗/s)·s, s = scale factor |
| Tầng 3: NÉT | dày/mỏng | thay đổi ε (viền) hoặc δ (dilation) |
| Boolean | union | min(f₁, f₂) |
| Boolean | intersect | max(f₁, f₂) |
| Boolean | subtract | max(f₁, −f₂) |
