# S.3~S.7 — Braille, APL, Technical, Block, Khác · Cây phân loại

---

## S.3 — CHỮ NỔI (Braille) · 256 cụm

### Tầng duy nhất: "Chấm nào bật?"

#### Mô hình toán: Mã nhị phân trong không gian vector F₂⁸

```
  Mỗi pattern = vector v⃗ ∈ {0,1}⁸ (8 bits, mỗi bit = 1 chấm)

  Hamming distance: d(v₁, v₂) = ||v₁ ⊕ v₂||₁ = số bit khác nhau
  Hamming weight: w(v) = ||v||₁ = số chấm bật

  Encoding: dots → bitmask
    DOTS-1     = 0b00000001 = 1
    DOTS-12    = 0b00000011 = 3
    DOTS-135   = 0b00010101 = 21

  Ma trận vị trí → tọa độ 2D:
    Position matrix P = [(0,0), (0,1), (0,2), (0,3), (1,0), (1,1), (1,2), (1,3)]
    Spatial density: ρ = w(v) / 8  (tỷ lệ chấm bật / tổng)

  256 patterns = full enumeration of F₂⁸
  Isomorphic to Z/256Z (cyclic group of order 256)
```

```
Braille = ma trận 2×4 = 8 vị trí chấm
Tên = "BRAILLE PATTERN DOTS-" + các chấm bật

  ⠁ = dot 1         vị trí [1]
  ⠃ = dots 1,2      vị trí [1,2]
  ⠇ = dots 1,2,3    vị trí [1,2,3]
  ⠿ = dots 1-6      vị trí [1,2,3,4,5,6]
  ⣿ = dots 1-8      vị trí [1,2,3,4,5,6,7,8]
  ⠀ = blank          vị trí [] (không chấm nào)

Ma trận vị trí:
  [1] [4]
  [2] [5]
  [3] [6]
  [7] [8]
```

### Phân loại 256 cụm

```
BRAILLE PATTERN BLANK                     → [] không chấm
BRAILLE PATTERN DOTS-1                    → [1] trái-trên
BRAILLE PATTERN DOTS-12                   → [1,2] trái-trên + trái-giữa
BRAILLE PATTERN DOTS-123                  → [1,2,3] cột trái
BRAILLE PATTERN DOTS-1234                 → [1,2,3,4] cột trái + phải-trên
BRAILLE PATTERN DOTS-12345678             → tất cả 8 chấm bật
...

→ "ah, chữ nổi, chấm [danh sách vị trí]"
  Nhìn số sau DOTS → biết ngay chấm nào bật
  Không cần thêm tầng — tên ĐÃ mã hóa đầy đủ
```

### Tóm tắt

```
256 = 2^8 tổ hợp (mỗi chấm: bật/tắt)
Công thức duy nhất: f(dots) = bitmask 8-bit
  DOTS-1     = 0b00000001
  DOTS-12    = 0b00000011
  DOTS-135   = 0b00010101
  ...
→ Tự mã hóa: tên = dữ liệu, không cần phân loại thêm
```

---

## S.4 — APL (Functional Symbols) · 52 cụm

### Tầng 1: "Ký tự gốc nào?"

#### Mô hình toán: Tổ hợp hàm trong lambda calculus

```
  Mỗi APL symbol = 1 combinator (hàm bậc cao)
  base × modifier = function composition: (f ∘ g)(x) = f(g(x))

  APL operators as combinators:
    / (reduce)  = fold:  /f [a,b,c] = f(a, f(b, c))
    ¨ (each)    = map:   f¨ [a,b,c] = [f(a), f(b), f(c)]
    . (inner)   = inner product: A f.g B = f/ (Aᵢ g Bⱼ)

  Modifier as morphism:
    underbar  = ground (fix point: f(x̲) = x̲)
    diaeresis = iterate (f¨ = map f over elements)
    tilde     = commute (f̃(a,b) = f(b,a))
    stile     = absolute (|f| = magnitude)

  Composition algebra: (S, ∘) forms a monoid
    Identity: I (identity combinator)
    Associativity: (f ∘ g) ∘ h = f ∘ (g ∘ h)
```

```
APL FUNCTIONAL SYMBOL
├── ALPHA        "alpha" — ký tự α
├── IOTA         "iota" — ký tự ι
├── OMEGA        "omega" — ký tự ω
├── RHO          "rho" — ký tự ρ
├── DEL          "del" — tam giác ngược ∇
├── DELTA        "delta" — tam giác Δ
├── CIRCLE       "circle" — hình tròn ○
├── DIAMOND      "diamond" — hình thoi ◇
├── STAR         "star" — sao ★
├── QUAD         "quad" — hình vuông ⎕
├── JOT          "jot" — chấm nhỏ ∘
├── SHOE         "shoe" — hình giày ∩ ∪
├── CARET        "caret" — mũ nhọn ∧ ∨
├── TACK         "tack" — đinh ⊤ ⊥ ⊢ ⊣
├── SLASH        "slash / backslash" — gạch chéo / \
├── COMMA        "comma" — dấu phẩy ,
├── I-BEAM       "i-beam" — dầm I ⌶
└── ZILDE        "zilde" — tập rỗng ⍬
```

### Tầng 2: "Modifier gì?"

```
MODIFIER
├── UNDERBAR     "underbar" — có gạch dưới  α̲
├── DIAERESIS    "diaeresis" — có hai chấm trên  ä
├── TILDE        "tilde" — có dấu ngã  ã
├── STILE        "stile" — có thanh dọc  |
├── BAR          "bar" — có thanh ngang  —
└── (không)      → ký tự gốc không modifier
```

### Phân loại cụ thể

```
APL FUNCTIONAL SYMBOL ALPHA                → alpha + không modifier
APL FUNCTIONAL SYMBOL ALPHA UNDERBAR       → alpha + gạch dưới
APL FUNCTIONAL SYMBOL DEL DIAERESIS        → del + hai chấm
APL FUNCTIONAL SYMBOL DEL STILE            → del + thanh dọc
APL FUNCTIONAL SYMBOL DEL TILDE            → del + dấu ngã
APL FUNCTIONAL SYMBOL DELTA STILE          → delta + thanh dọc
APL FUNCTIONAL SYMBOL DELTA UNDERBAR       → delta + gạch dưới
APL FUNCTIONAL SYMBOL DOWN CARET TILDE     → caret xuống + ngã
APL FUNCTIONAL SYMBOL DOWN SHOE STILE      → shoe xuống + thanh dọc
APL FUNCTIONAL SYMBOL DOWN TACK JOT        → tack xuống + jot
APL FUNCTIONAL SYMBOL DOWN TACK UNDERBAR   → tack xuống + gạch dưới
APL FUNCTIONAL SYMBOL EPSILON UNDERBAR     → epsilon + gạch dưới
APL FUNCTIONAL SYMBOL IOTA                 → iota
APL FUNCTIONAL SYMBOL IOTA UNDERBAR        → iota + gạch dưới
APL FUNCTIONAL SYMBOL OMEGA UNDERBAR       → omega + gạch dưới
APL FUNCTIONAL SYMBOL QUAD COLON           → quad + dấu hai chấm
APL FUNCTIONAL SYMBOL QUAD DIAMOND         → quad + kim cương
APL FUNCTIONAL SYMBOL QUAD DIVIDE          → quad + phép chia
APL FUNCTIONAL SYMBOL SLASH BAR            → slash + thanh ngang
APL FUNCTIONAL SYMBOL STAR DIAERESIS       → star + hai chấm
→ "ah, APL [ký tự gốc] [modifier]"
```

### Tóm tắt

```
52 cụm = tuple (ký_tự_gốc, modifier)
18 ký tự gốc × 6 modifier = 108 lý thuyết, 52 thực tế
Pattern: "APL FUNCTIONAL SYMBOL" + {base} + {modifier}
```

---

## S.5 — KỸ THUẬT (Technical) · 21 cụm

### Tầng 1: "Lĩnh vực nào?"

```
KỸ THUẬT
├── NHA KHOA     "dentistry" — 11 cụm
├── ĐIỆN         "ac current / dc / electrical" — 3 cụm
├── HÓA HỌC     "benzene" — 2 cụm
├── ĐO LƯỜNG    "scan line / hysteresis" — 4 cụm
└── THIẾT BỊ     "keyboard / helm" — 1 cụm
```

#### Công thức vật lý theo lĩnh vực

### Phân loại cụ thể

**NHA KHOA (dentistry) — 11 cụm:**
```
DENTISTRY SYMBOL LIGHT DOWN AND HORIZONTAL
DENTISTRY SYMBOL LIGHT DOWN AND HORIZONTAL WITH WAVE
DENTISTRY SYMBOL LIGHT UP AND HORIZONTAL
DENTISTRY SYMBOL LIGHT UP AND HORIZONTAL WITH WAVE
DENTISTRY SYMBOL LIGHT VERTICAL AND BOTTOM LEFT
DENTISTRY SYMBOL LIGHT VERTICAL AND BOTTOM RIGHT
DENTISTRY SYMBOL LIGHT VERTICAL AND TOP LEFT
DENTISTRY SYMBOL LIGHT VERTICAL AND TOP RIGHT
DENTISTRY SYMBOL LIGHT VERTICAL AND WAVE
DENTISTRY SYMBOL LIGHT VERTICAL WITH CIRCLE
DENTISTRY SYMBOL LIGHT VERTICAL WITH TRIANGLE
→ "ah, nha khoa [hướng nét] [có sóng/tròn/tam giác]"
  Pattern giống Box Drawing: {direction} + {modifier}
```

```
  Mô hình: Connection topology (tương tự Box Drawing cho biểu đồ nha khoa)
  Graph G = (V, E) với V = {răng}, E = {nối nét}
  Adjacency: A[i,j] = 1 nếu răng i nối răng j qua nét vẽ
  Degree: deg(v) = số nét xuất phát từ node v
```

**ĐIỆN (electrical) — 3 cụm:**
```
AC CURRENT                                → dòng xoay chiều
DIRECT CURRENT SYMBOL FORM TWO            → dòng một chiều
ELECTRICAL INTERSECTION                   → giao điểm điện
→ "ah, ký hiệu điện [loại]"
```

```
  Mô hình: Định luật Ohm & tín hiệu điện
  V = IR  (Ohm's law: điện áp = dòng × trở kháng)
  AC: v(t) = V₀sin(ωt)  với ω = 2πf (tần số góc)
  DC: v(t) = V₀  (hằng số)
  Electrical intersection: node trong circuit graph với deg(v) ≥ 3
```

**HÓA HỌC (chemistry) — 2 cụm:**
```
BENZENE RING                              → vòng benzen (lục giác)
BENZENE RING WITH CIRCLE                  → vòng benzen + tròn trong
→ "ah, hóa học [benzene]"
```

```
  Mô hình: Hóa học lượng tử — LCAO-MO
  Benzene = C₆H₆, cấu trúc vòng 6 carbon
  Delocalized π electrons: ψ = Σcᵢφᵢ  (LCAO-MO: tổ hợp tuyến tính orbital nguyên tử)
  BENZENE RING = hexagonal graph C₆  (6 đỉnh, 6 cạnh)
  BENZENE RING WITH CIRCLE = C₆ + π delocalization (vòng tròn = mật độ electron π)
```

**ĐO LƯỜNG (measurement) — 4 cụm:**
```
HORIZONTAL SCAN LINE-1                    → dòng quét ngang vị trí 1
HORIZONTAL SCAN LINE-3                    → dòng quét ngang vị trí 3
HORIZONTAL SCAN LINE-7                    → dòng quét ngang vị trí 7
HORIZONTAL SCAN LINE-9                    → dòng quét ngang vị trí 9
HYSTERESIS SYMBOL                         → ký hiệu trễ
→ "ah, dòng quét [vị trí]"
```

```
  Mô hình: Raster sampling & Hysteresis
  Scan line = raster sampling: y = k/N  với k ∈ {1,3,7,9}, N = 9
    Vị trí dòng quét: phân chia đều ô hiển thị thành 9 dòng
  Hysteresis: output phụ thuộc lịch sử (history-dependent)
    H(x) = { H₊  nếu x đang tăng (increasing)
            { H₋  nếu x đang giảm (decreasing)
    Tạo vòng trễ: ngưỡng bật ≠ ngưỡng tắt
```

**THIẾT BỊ — 1 cụm:**
```
KEYBOARD                                  → bàn phím
HELM SYMBOL                               → ký hiệu lái
→ "ah, thiết bị [tên]"
```

---

## S.6 — KHỐI (Block Elements) · 38 cụm

#### Mô hình toán: Tích phân diện tích trên ô đơn vị [0,1]²

```
  Coverage ratio α = ∫∫_D dA / ∫∫_[0,1]² dA

  FULL BLOCK:     α = 1.0    D = [0,1]²
  UPPER HALF:     α = 0.5    D = [0,1]×[0.5,1]
  LEFT QUARTER:   α = 0.25   D = [0,0.25]×[0,1]
  LEFT 3/8:       α = 0.375  D = [0,0.375]×[0,1]

  SHADE as opacity:
    LIGHT SHADE:  α = 0.25   (sparse stippling)
    MEDIUM SHADE: α = 0.50   (medium stippling)
    DARK SHADE:   α = 0.75   (dense stippling)

  ARC as parametric curve:
    p⃗(t) = (cos(t), sin(t)) for t ∈ [θ₁, θ₂]
    Quarter arc = π/2 radians

  Fractional blocks form a GROUP under composition:
    α₁ ∪ α₂ covers area min(α₁+α₂, 1)
```

### Tầng 1: "Kiểu khối gì?"

```
KHỐI
├── ĐẦY         "full block" — tô kín ██
├── NỬA          "half" — tô nửa ▀ ▄ ▌ ▐
├── PHẦN         "eighth / quarter / three eighths / ..." — tô 1/8, 1/4...
├── BÓNG         "shade" — tô mờ ░ ▒ ▓
├── GÓC PHẦN TƯ  "quadrant" — tô 1/4 góc
└── CUNG         "arc" — cung tròn góc
```

### Tầng 2: "Hướng/vị trí nào?"

```
VỊ TRÍ
├── TRÊN         "upper" — nửa trên ▀
├── DƯỚI         "lower" — nửa dưới ▄
├── TRÁI         "left" — nửa trái ▌
├── PHẢI         "right" — nửa phải ▐
├── TRÊN-TRÁI    "upper left" — góc trên trái
├── TRÊN-PHẢI    "upper right" — góc trên phải
├── DƯỚI-TRÁI    "lower left" — góc dưới trái
└── DƯỚI-PHẢI    "lower right" — góc dưới phải
```

### Phân loại cụ thể

```
FULL BLOCK                                → đầy
UPPER HALF BLOCK                          → nửa trên
LOWER HALF BLOCK                          → nửa dưới
LEFT HALF BLOCK                           → nửa trái
RIGHT HALF BLOCK                          → nửa phải

LEFT ONE EIGHTH BLOCK                     → trái 1/8
LEFT ONE QUARTER BLOCK                    → trái 1/4
LEFT THREE EIGHTHS BLOCK                  → trái 3/8
LEFT FIVE EIGHTHS BLOCK                   → trái 5/8
LEFT THREE QUARTERS BLOCK                 → trái 3/4
LEFT SEVEN EIGHTHS BLOCK                  → trái 7/8

LOWER ONE EIGHTH BLOCK                    → dưới 1/8
LOWER ONE QUARTER BLOCK                   → dưới 1/4
LOWER THREE EIGHTHS BLOCK                 → dưới 3/8
LOWER FIVE EIGHTHS BLOCK                  → dưới 5/8
LOWER THREE QUARTERS BLOCK               → dưới 3/4
LOWER SEVEN EIGHTHS BLOCK                 → dưới 7/8

LIGHT SHADE                               → bóng mờ ░
MEDIUM SHADE                              → bóng vừa ▒
DARK SHADE                                → bóng đậm ▓

UPPER LEFT QUADRANT CIRCULAR ARC          → cung trên-trái
UPPER RIGHT QUADRANT CIRCULAR ARC         → cung trên-phải
LOWER LEFT QUADRANT CIRCULAR ARC          → cung dưới-trái
LOWER RIGHT QUADRANT CIRCULAR ARC         → cung dưới-phải

→ "ah, khối [đầy/nửa/phần/bóng/cung] [vị trí] [tỷ lệ]"
```

### Tóm tắt

```
38 cụm = tuple (kiểu_khối, vị_trí, tỷ_lệ)
Pattern: {position} {fraction} BLOCK | {intensity} SHADE | {position} ARC
```

---

## S.7 — KHÁC (375 cụm) · Các ký hiệu không thuộc nhóm trên

#### Mô hình toán: Phân loại theo functor F: Unicode → Semantic_Category

```
  F maps each codepoint to its semantic domain:
    F(cp) ∈ {Pointer, Music, Weather, Flag, Tool, Astro, Ornament, Religion, Misc}

  Within each category, objects form a partial order by specificity:
    AIRPLANE ≤ DỤNG CỤ/phương_tiện ≤ DỤNG CỤ

  Morphisms = semantic relationships between symbols
```

### Phân nhóm phụ theo từ khóa

```
KHÁC
├── CON TRỎ      "pointer / cursor" — ▶ ◀ ► ◄ chỉ hướng
├── ÂM NHẠC      "note / flat / sharp" — ♩ ♪ ♫ ♬ (nốt nhạc đơn lẻ trong SDF)
├── THỜI TIẾT    "sun / cloud / snowflake / umbrella" — ☀ ☁ ❄ ☂
├── CỜ / DẤU     "flag / ballot / check" — ⚑ ☑ ✓
├── DỤNG CỤ      "scissors / pencil / envelope / telephone" — ✂ ✏ ✉ ☎
├── THIÊN VĂN    planet names (ADMETOS, APOLLON, ZEUS...) — ⯓ ⯔
├── TRANG TRÍ    "ornament / dingbat / floral" — ❦ ❧
├── TÔNG GIÁO    "cross / yin yang / star of david" — ☯ ✡ ☪
└── LINH TINH    "warning / skull / atom / recycle" — ⚠ ☠ ⚛ ♻
```

### Ví dụ cụ thể

```
AIRPLANE                                  → DỤNG CỤ / phương tiện
ALARM CLOCK                               → DỤNG CỤ / thời gian
ADMETOS                                   → THIÊN VĂN / hành tinh
APOLLON                                   → THIÊN VĂN / hành tinh
BALLOT BOX WITH LIGHT X                   → CỜ/DẤU / hộp bầu
BALLOT X                                  → CỜ/DẤU / dấu X
BELL SYMBOL                               → DỤNG CỤ / chuông
BLACK FLORETTE                            → TRANG TRÍ / hoa
BLACK LEFT-POINTING POINTER               → CON TRỎ / trái
BLACK NIB                                 → DỤNG CỤ / ngòi bút
BLACK OCTAGON                             → HÌNH HỌC phụ / 8 cạnh
BLACK PARALLELOGRAM                       → HÌNH HỌC phụ / bình hành
BLACK QUESTION MARK ORNAMENT              → TRANG TRÍ / dấu hỏi
BLACK RECTANGLE                           → HÌNH HỌC phụ / chữ nhật
→ "ah, [nhóm phụ] / [chi tiết]"
```

---

## Tổng kết S (Shape) — Tất cả nhóm

```
S.0  MŨI TÊN      618 cụm   5 tầng: kiểu × hướng × dày × tô × đuôi
S.1  HÌNH HỌC     321 cụm   4 tầng: hình × tô × cỡ × modifier
S.2  VẼ HỘP       128 cụm   3 tầng: kiểu_nét × dày × hướng_góc
S.3  CHỮ NỔI      256 cụm   1 tầng: bitmask 8-bit (tự mã hóa)
S.4  APL            52 cụm   2 tầng: ký_tự_gốc × modifier
S.5  KỸ THUẬT      21 cụm   2 tầng: lĩnh_vực × chi_tiết
S.6  KHỐI           38 cụm   3 tầng: kiểu_khối × vị_trí × tỷ_lệ
S.7  KHÁC          375 cụm   2 tầng: nhóm_phụ × chi_tiết
─────────────────────────────────────────
TỔNG             1,809 cụm   → mỗi cụm = 1 tuple từ khóa
```

### Bảng tổng hợp mô hình toán / vật lý

```
Nhóm   Mô hình toán                  Công thức chính                          Mô tả
─────  ────────────────────────────   ───────────────────────────────────────   ──────────────────────────────────────
S.3    Không gian vector F₂⁸          v⃗ ∈ {0,1}⁸, d(v₁,v₂) = ||v₁⊕v₂||₁     Mỗi pattern Braille = vector nhị phân
       (GF(2) 8 chiều)               w(v) = ||v||₁, ρ = w(v)/8               8 bit, đo khoảng cách Hamming
                                      Đẳng cấu Z/256Z                         256 pattern = liệt kê đầy đủ F₂⁸

S.4    Lambda calculus                (f ∘ g)(x) = f(g(x))                    Mỗi ký hiệu APL = 1 combinator
       Monoid tổ hợp hàm             /f = fold, f¨ = map, f.g = inner        base × modifier = phép hợp hàm
                                      (S, ∘) monoid với I = đơn vị            Đại số tổ hợp có tính kết hợp

S.5    Vật lý chuyên ngành           V = IR (Ohm), v(t) = V₀sin(ωt)          Nha khoa: graph topology nối răng
       (điện, hóa, đo lường)         ψ = Σcᵢφᵢ (LCAO-MO)                    Điện: định luật Ohm, tín hiệu AC/DC
                                      y = k/N (raster), H(x) = {H₊,H₋}      Hóa học: orbital phân tử benzene
                                                                               Đo lường: raster sampling & hysteresis

S.6    Tích phân diện tích           α = ∫∫_D dA / ∫∫_[0,1]² dA             Mỗi khối = tỷ lệ phủ trên ô đơn vị
       trên [0,1]²                   p⃗(t) = (cos(t), sin(t))                 Shade = mức opacity (0.25/0.50/0.75)
                                      min(α₁+α₂, 1)                           Cung = đường cong tham số, arc π/2

S.7    Lý thuyết phạm trù           F: Unicode → Semantic_Category           Functor ánh xạ codepoint → miền ngữ nghĩa
       (Category theory)             Partial order theo specificity            Trong mỗi category: thứ tự bộ phận
                                      Morphism = quan hệ ngữ nghĩa             Morphism nối các ký hiệu liên quan
```
