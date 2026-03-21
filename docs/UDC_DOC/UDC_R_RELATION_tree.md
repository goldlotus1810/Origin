# R — RELATION (Quan hệ) · Cây phân loại bằng từ ngữ

> 9 nhóm cụm từ: Toán tử (147), So sánh (225), Chữ cái toán (1224),
> Số (339), Dấu câu (112), Tiền tệ (85), Cổ đại (58), Điều khiển (51), Khác (1543)

---

## R.0 — TOÁN TỬ (Operator) · 147 cụm

### Tầng 1: "Phép toán gì?"

```
TOÁN TỬ
├── CỘNG/TRỪ     "plus / minus" — + −
├── NHÂN/CHIA    "times / multiplication / division / divide" — × ÷
├── TÍCH PHÂN    "integral / contour" — ∫ ∮
├── TỔNG         "summation / sum" — Σ ∑
├── TÍCH         "product / coproduct" — ∏
├── CĂN          "root / square root" — √
├── VI PHÂN      "differential / nabla / del" — ∇ ∂
├── TOÁN TỬ VÒNG "circled plus/minus/times/dot" — ⊕ ⊖ ⊗ ⊙
├── TOÁN TỬ ĐẶC BIỆT "wreath / amalgamation / join" — ≀
└── DẤU CHẤM     "dot operator / bullet operator / ring operator" — · ∘ •
```

### Tầng 2: "Modifier gì?"

```
MODIFIER
├── VÒNG TRÒN    "circled" — trong vòng tròn ⊕
├── ĐẢO          "reversed" — đảo ngược
├── XOAY         "rotated" — xoay
├── TRÊN/DƯỚI    "above / below / with ..." — có dấu phụ
└── CHIỀU        "clockwise / anticlockwise" — hướng xoay (tích phân)
```

### Ví dụ cụ thể

```
PLUS SIGN                                 → cộng
MINUS SIGN                                → trừ
MULTIPLICATION SIGN                       → nhân
DIVISION SIGN                             → chia
CIRCLED PLUS                              → cộng + vòng tròn
CIRCLED MINUS                             → trừ + vòng tròn
CIRCLED TIMES                             → nhân + vòng tròn
CIRCLED DOT OPERATOR                      → chấm + vòng tròn
INTEGRAL                                  → tích phân
CONTOUR INTEGRAL                          → tích phân đường
CLOCKWISE INTEGRAL                        → tích phân thuận
ANTICLOCKWISE CONTOUR INTEGRAL            → tích phân ngược
SURFACE INTEGRAL                          → tích phân mặt
VOLUME INTEGRAL                           → tích phân thể tích
N-ARY SUMMATION                           → tổng n phần tử
N-ARY PRODUCT                             → tích n phần tử
SQUARE ROOT                               → căn bậc 2
CUBE ROOT                                 → căn bậc 3
FOURTH ROOT                               → căn bậc 4
→ "ah, toán tử [phép gì] [modifier]"
```

---

## R.1 — SO SÁNH (Comparison) · 225 cụm

### Tầng 1: "So sánh kiểu gì?"

```
SO SÁNH
├── BẰNG         "equal / equals" — =
├── KHÔNG BẰNG   "not equal" — ≠
├── LỚN HƠN     "greater-than / succeeds" — >
├── NHỎ HƠN     "less-than / precedes" — <
├── TƯƠNG ĐƯƠNG   "equivalent / identical / congruent" — ≡ ≅
├── GẦN BẰNG     "approximately / almost / asymptotically" — ≈ ≃
├── TƯƠNG TỰ     "similar / corresponds / proportional" — ∼ ∝
├── TẬP HỢP      "subset / superset / element / contains" — ⊂ ⊃ ∈ ∋
├── SONG SONG     "parallel / perpendicular" — ∥ ⊥
└── THỨ TỰ       "precedes / succeeds / between" — ≺ ≻
```

### Tầng 2: "Có kèm điều kiện gì?"

```
ĐIỀU KIỆN
├── HOẶC BẰNG    "or equal to" — ≤ ≥ ⊆ ⊇
├── KHÔNG        "not" / gạch chéo — ≠ ⊄ ∉
├── TRÊN/DƯỚI    "above / below / with dot" — có dấu phụ
├── KÉP          "double" — nét đôi
└── PHỦ ĐỊNH     "does not" — phủ định mạnh
```

### Ví dụ cụ thể

```
EQUAL TO                                  → bằng
NOT EQUAL TO                              → không bằng
ALMOST EQUAL TO                           → gần bằng
APPROXIMATELY EQUAL TO                    → xấp xỉ bằng
IDENTICAL TO                              → đồng nhất
LESS-THAN OR EQUAL TO                     → nhỏ hơn hoặc bằng
GREATER-THAN OR EQUAL TO                  → lớn hơn hoặc bằng
MUCH LESS-THAN                            → nhỏ hơn nhiều
MUCH GREATER-THAN                         → lớn hơn nhiều
SUBSET OF                                 → tập con của
SUPERSET OF                               → tập chứa
NOT A SUBSET OF                           → không phải tập con
ELEMENT OF                                → phần tử của
NOT AN ELEMENT OF                         → không phải phần tử
CONTAINS AS MEMBER                        → chứa như phần tử
CONGRUENT WITH DOT ABOVE                  → đồng dư + chấm trên
→ "ah, so sánh [kiểu] [điều kiện]"
```

---

## R.2 — CHỮ CÁI TOÁN HỌC (Math Letters) · 1224 cụm

### Tầng 1: "Hệ chữ nào?"

```
CHỮ CÁI TOÁN
├── LATIN        "mathematical [style] [case] [letter]"
├── GREEK        "mathematical [style] [case] [greek letter]"
├── ARABIC       "arabic mathematical [style] [letter]"
└── SỐ           "mathematical [style] digit [0-9]"
```

### Tầng 2: "Font kiểu gì?"

```
FONT
├── THƯỜNG       "sans-serif" — không chân
├── CÓ CHÂN      (mặc định, serif) — có chân
├── ĐẬM          "bold" — đậm
├── NGHIÊNG      "italic" — nghiêng
├── ĐẬM NGHIÊNG "bold italic" — đậm + nghiêng
├── GÓC CẠNH     "fraktur" — kiểu Đức cổ 𝔄
├── KÉP          "double-struck" — nét đôi 𝔸
├── VIẾT TAY     "script" — chữ viết tay 𝒜
├── VIẾT TAY ĐẬM "bold script" — viết tay đậm
├── ĐƠN CÁCH     "monospace" — đều nhau 𝙰
├── KHÔNG CHÂN ĐẬM "sans-serif bold"
├── KHÔNG CHÂN NGHIÊNG "sans-serif italic"
└── KHÔNG CHÂN ĐẬM NGHIÊNG "sans-serif bold italic"
```

### Tầng 3: "Chữ hoa hay thường?"

```
KIỂU CHỮ
├── HOA          "capital" — A B C
└── THƯỜNG       "small" — a b c
```

### Ví dụ cụ thể

```
MATHEMATICAL BOLD CAPITAL A               → Latin + đậm + hoa + A
MATHEMATICAL BOLD SMALL A                 → Latin + đậm + thường + a
MATHEMATICAL ITALIC CAPITAL A             → Latin + nghiêng + hoa + A
MATHEMATICAL FRAKTUR CAPITAL A            → Latin + góc cạnh + hoa + A
MATHEMATICAL DOUBLE-STRUCK CAPITAL A      → Latin + kép + hoa + A
MATHEMATICAL SCRIPT CAPITAL A             → Latin + viết tay + hoa + A
MATHEMATICAL MONOSPACE CAPITAL A          → Latin + đơn cách + hoa + A
MATHEMATICAL SANS-SERIF BOLD CAPITAL A    → Latin + không chân đậm + hoa + A

MATHEMATICAL BOLD CAPITAL ALPHA           → Greek + đậm + hoa + alpha
MATHEMATICAL ITALIC SMALL ALPHA           → Greek + nghiêng + thường + alpha

ARABIC MATHEMATICAL DOUBLE-STRUCK BEH     → Arabic + kép + beh
ARABIC MATHEMATICAL INITIAL BEH            → Arabic + đầu chữ + beh

MATHEMATICAL BOLD DIGIT ZERO              → Số + đậm + 0
MATHEMATICAL MONOSPACE DIGIT FIVE         → Số + đơn cách + 5

→ "ah, chữ toán [hệ] [font] [hoa/thường] [ký tự nào]"
  Pattern: MATHEMATICAL {style} {CAPITAL/SMALL} {letter}
```

---

## R.3 — SỐ (Numbers) · 339 cụm

### Tầng 1: "Hệ đếm nào?"

```
SỐ
├── QUE ĐẾM      "counting rod" — que tính Trung Hoa
├── HÌNH NÊM     "cuneiform numeric" — số Lưỡng Hà cổ
├── LA MÃ        "roman numeral" — I II III IV V
├── ẤN ĐỘ        "indic" — số Ấn Độ
├── OTTOMAN      "ottoman siyaq" — số Ottoman
└── KHÁC         số viết tay, chữ số vulgar
```

### Tầng 2: "Giá trị gì?"

```
GIÁ TRỊ
├── ĐƠN VỊ      "unit digit / ones" — 1-9
├── CHỤC         "tens digit" — 10-90
├── TRĂM         "hundred" — 100+
├── NGÀN         "thousand" — 1000+
└── PHÂN SỐ      "fraction / half / third / quarter"
```

### Ví dụ cụ thể

```
COUNTING ROD UNIT DIGIT ONE               → que đếm + đơn vị + 1
COUNTING ROD UNIT DIGIT FIVE              → que đếm + đơn vị + 5
COUNTING ROD TENS DIGIT THREE             → que đếm + chục + 3
CUNEIFORM NUMERIC SIGN ONE                → hình nêm + 1
CUNEIFORM NUMERIC SIGN TEN                → hình nêm + 10
CUNEIFORM NUMERIC SIGN ONE HUNDRED        → hình nêm + 100
ROMAN NUMERAL ONE                         → La Mã + I
ROMAN NUMERAL FIVE                        → La Mã + V
ROMAN NUMERAL TEN                         → La Mã + X
ROMAN NUMERAL FIFTY                       → La Mã + L
VULGAR FRACTION ONE HALF                  → phân số + 1/2
VULGAR FRACTION ONE QUARTER               → phân số + 1/4
VULGAR FRACTION ONE THIRD                 → phân số + 1/3
→ "ah, số [hệ đếm] [bậc] [giá trị]"
```

---

## R.4 — DẤU CÂU (Punctuation) · 112 cụm

### Tầng 1: "Loại dấu gì?"

```
DẤU CÂU
├── NGOẶC        "parenthesis / bracket" — ( ) [ ] { } ⟨ ⟩
├── GẠCH         "dash / hyphen" — – — ‐ ‑
├── DẤU CHẤM     "colon / semicolon / comma / ellipsis" — : ; , …
├── DẤU NGOẶC KÉP "quotation mark" — " " ' ' « »
├── DẤU SAO      "asterisk / dagger / bullet" — * † ‡ •
├── DẤU ĐOẠN     "pilcrow / paragraph / section" — ¶ § ‖
└── CUNEIFORM    "cuneiform punctuation" — dấu câu hình nêm cổ
```

### Tầng 2: "Vị trí/hướng?"

```
VỊ TRÍ
├── TRÁI/MỞ      "left / opening" — ( [ { ⟨
├── PHẢI/ĐÓNG    "right / closing" — ) ] } ⟩
├── TRÊN         "top / upper / high" — dấu trên
├── DƯỚI         "bottom / lower / low" — dấu dưới
├── ĐÔI          "double" — dấu kép
└── ĐẢO          "reversed / turned" — lật ngược
```

### Ví dụ cụ thể

```
LEFT PARENTHESIS                          → ngoặc + trái/mở
RIGHT PARENTHESIS                         → ngoặc + phải/đóng
LEFT SQUARE BRACKET                       → ngoặc vuông + trái
RIGHT SQUARE BRACKET                      → ngoặc vuông + phải
LEFT CURLY BRACKET                        → ngoặc nhọn + trái
DOUBLE HYPHEN                             → gạch + đôi
DOUBLE LOW-9 QUOTATION MARK              → ngoặc kép + dưới
DOUBLE HIGH-REVERSED-9 QUOTATION MARK    → ngoặc kép + trên + đảo
HORIZONTAL ELLIPSIS                       → chấm lửng + ngang
MIDLINE HORIZONTAL ELLIPSIS              → chấm lửng + giữa
DOWN RIGHT DIAGONAL ELLIPSIS             → chấm lửng + chéo xuống
→ "ah, dấu câu [loại] [vị trí/hướng]"
```

---

## R.5 — TIỀN TỆ (Currency) · 85 cụm

### Tầng 1: "Khu vực nào?"

```
TIỀN TỆ
├── CHÂU ÂU      "euro / franc / lira / pound / penny" — € ₣ ₤ £
├── CHÂU Á       "dong / rupee / won / yen / tenge / kip" — ₫ ₹ ₩ ¥
├── CHÂU MỸ      "dollar / peso / austral / cruzeiro / guarani" — $ ₱
├── TRUNG ĐÔNG   "dirham / lari / manat / hryvnia" — ₯ ₾
├── TIỀN MÃ HÓA  "bitcoin" — ₿
├── HY LẠP CỔ    "drachma / obol / mina / gramma / aroura" — ₯
└── KÝ HIỆU CHUNG "currency / sign" — ¤
```

### Ví dụ cụ thể

```
EURO SIGN                                 → Châu Âu + euro
DOLLAR SIGN                               → Châu Mỹ + đô la
POUND SIGN                                → Châu Âu + bảng Anh
BITCOIN SIGN                              → tiền mã hóa + bitcoin
DONG SIGN                                 → Châu Á + đồng
RUPEE SIGN                                → Châu Á + rupee
GREEK DRACHMA SIGN                        → Hy Lạp cổ + drachma
GREEK FIVE OBOLS SIGN                     → Hy Lạp cổ + 5 obol
GREEK GRAMMA SIGN                         → Hy Lạp cổ + gramma
→ "ah, tiền [khu vực] [tên tiền]"
```

---

## R.6 — CỔ ĐẠI (Ancient Numerals) · 58 cụm

### Tầng 1: "Nền văn minh nào?"

```
CỔ ĐẠI
├── HY LẠP ACROPHONIC "greek acrophonic" — chữ tượng hình Hy Lạp
│   ├── ATTIC         "attic" — hệ Attica
│   ├── EPIDAUREAN    "epidaurean" — hệ Epidaurus
│   ├── HERMIONIAN    "hermionian" — hệ Hermione
│   ├── MESSENIAN     "messenian" — hệ Messenia
│   ├── NAXIAN        "naxian" — hệ Naxos
│   └── TROEZENIAN    "troezenian" — hệ Troezen
└── OTTOMAN SIYAQ      "ottoman siyaq" — hệ Ottoman
```

### Tầng 2: "Giá trị?"

```
GIÁ TRỊ
├── MỘT          "one" — 1
├── NĂM          "five" — 5
├── MƯỜI         "ten" — 10
├── NĂM MƯƠI    "fifty" — 50
├── TRĂM         "hundred" — 100
├── NĂM TRĂM    "five hundred" — 500
├── NGÀN         "thousand" — 1000
├── NĂM NGÀN    "five thousand" — 5000
└── VẠN          "fifty thousand" — 50000
```

### Tầng 3: "Đơn vị gì?"

```
ĐƠN VỊ
├── STATER       "staters" — đồng stater (tiền)
├── TALENT       "talents" — đơn vị talent (cân nặng/tiền)
├── DRACHMA      "drachma / drachmas" — đồng drachma
├── MINA         "mina / mnas" — đơn vị mina
└── (không)      → số thuần túy
```

### Ví dụ cụ thể

```
GREEK ACROPHONIC ATTIC FIVE               → Hy Lạp + Attica + 5
GREEK ACROPHONIC ATTIC FIFTY              → Hy Lạp + Attica + 50
GREEK ACROPHONIC ATTIC FIVE HUNDRED       → Hy Lạp + Attica + 500
GREEK ACROPHONIC ATTIC FIVE STATERS       → Hy Lạp + Attica + 5 + stater
GREEK ACROPHONIC ATTIC FIFTY TALENTS      → Hy Lạp + Attica + 50 + talent
GREEK ACROPHONIC ATTIC ONE DRACHMA        → Hy Lạp + Attica + 1 + drachma
GREEK ACROPHONIC EPIDAUREAN TWO HUNDRED   → Hy Lạp + Epidaurus + 200
→ "ah, số cổ đại [nền văn minh] [hệ] [giá trị] [đơn vị]"
```

---

## R.7 — ĐIỀU KHIỂN (Control) · 51 cụm

### Tầng 1: "Nhóm điều khiển nào?"

```
ĐIỀU KHIỂN
├── KÝ HIỆU CONTROL  "symbol for [action]" — ␀ ␍ ␊
├── ĐỊNH DẠNG        "activate / inhibit / embedding / override" — bidi
├── NGĂN CÁCH        "separator / joiner / space" — ZWJ ZWNJ
└── CHIỀU CHỮ        "left-to-right / right-to-left / directional" — LTR RTL
```

### Ví dụ cụ thể

```
SYMBOL FOR ACKNOWLEDGE                    → ký hiệu + ACK
SYMBOL FOR BACKSPACE                      → ký hiệu + BS
SYMBOL FOR BELL                           → ký hiệu + BEL
SYMBOL FOR CANCEL                         → ký hiệu + CAN
SYMBOL FOR CARRIAGE RETURN                → ký hiệu + CR
SYMBOL FOR DELETE                         → ký hiệu + DEL
SYMBOL FOR ESCAPE                         → ký hiệu + ESC
SYMBOL FOR FORM FEED                      → ký hiệu + FF
SYMBOL FOR LINE FEED                      → ký hiệu + LF
SYMBOL FOR NULL                           → ký hiệu + NUL
SYMBOL FOR SPACE                          → ký hiệu + SP
ACTIVATE ARABIC FORM SHAPING              → định dạng + Arabic + bật
INHIBIT ARABIC FORM SHAPING               → định dạng + Arabic + tắt
LEFT-TO-RIGHT EMBEDDING                   → chiều chữ + LTR + nhúng
RIGHT-TO-LEFT OVERRIDE                    → chiều chữ + RTL + ghi đè
POP DIRECTIONAL FORMATTING                → chiều chữ + pop
→ "ah, điều khiển [nhóm] [hành động]"
```

---

## Tổng kết R (Relation) — Tất cả nhóm

```
R.0  TOÁN TỬ       147 cụm   2 tầng: phép_toán × modifier
R.1  SO SÁNH       225 cụm   2 tầng: kiểu_so_sánh × điều_kiện
R.2  CHỮ TOÁN    1,224 cụm   3 tầng: hệ_chữ × font × hoa/thường × ký_tự
R.3  SỐ            339 cụm   3 tầng: hệ_đếm × bậc × giá_trị
R.4  DẤU CÂU      112 cụm   2 tầng: loại_dấu × vị_trí
R.5  TIỀN TỆ       85 cụm   2 tầng: khu_vực × tên_tiền
R.6  CỔ ĐẠI        58 cụm   3 tầng: văn_minh × giá_trị × đơn_vị
R.7  ĐIỀU KHIỂN    51 cụm   2 tầng: nhóm × hành_động
─────────────────────────────────────────
TỔNG             2,241 cụm   (+ 1,543 "Khác")
```
