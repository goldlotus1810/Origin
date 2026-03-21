# S.1 — HÌNH HỌC (Geometric) · Cây phân loại bằng từ ngữ

> 321 cụm từ Unicode → phân vào cây theo từ khóa
> Nhìn tên → biết ngay "thèn này hình gì, tô kiểu gì, kích cỡ ra sao"

---

## Tầng 1: "Nó là hình gì?"

```
HÌNH HỌC
├── TRÒN         "circle" — hình tròn ● ○
├── VUÔNG        "square" — hình vuông ■ □
├── TAM GIÁC     "triangle" — 3 cạnh ▲ △
├── KIM CƯƠNG    "diamond / lozenge / rhombus" — thoi ◆ ◇
├── SAO          "star" — nhiều cánh ★ ☆
├── CHỮ THẬP     "cross / saltire / maltese" — dấu cộng ✚ ✝
├── HÌNH NHIỀU CẠNH "pentagon / hexagon / octagon" — 5+ cạnh ⬠ ⎔
├── HÌNH ELIP    "ellipse" — bầu dục ⬮ ⬯
├── HÌNH CHỮ NHẬT "rectangle / parallelogram / trapezium" — 4 cạnh không đều
└── HOA          "florette / pinwheel / propeller / petalled" — hoa lá ✿ ❀
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

### CHỮ THẬP (cross) — ~15 cụm

```
MALTESE CROSS                             → chữ thập Malta
LATIN CROSS                               → chữ thập Latin
ORTHODOX CROSS                            → chữ thập chính thống
SALTIRE (St Andrew cross)                 → chữ X / chéo
WHITE CROSS ON RED CIRCLE                 → chữ thập trên nền đỏ
→ "ah, chữ thập [kiểu]"
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
