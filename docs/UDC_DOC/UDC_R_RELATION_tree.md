# R — RELATION (Quan hệ) · Cây phân loại bằng từ ngữ

> 9 nhóm cụm từ: Toán tử (147), So sánh (225), Chữ cái toán (1224),
> Số (339), Dấu câu (112), Tiền tệ (85), Cổ đại (58), Điều khiển (51), Khác (1543)

### Mô hình toán học tổng quát — R là Quan hệ trong Lý thuyết Phạm trù

```
Chiều R biểu diễn QUAN HỆ dưới dạng morphism trong Category Theory.

Định nghĩa: Phạm trù R = (Ob, Hom, ∘, id)
  - Ob(R)      = tập các thực thể toán học (số, ký hiệu, toán tử, ...)
  - Hom(A,B)   = tập morphism (quan hệ) từ đối tượng A đến B
  - ∘           = phép hợp morphism: nếu f: A→B, g: B→C thì g∘f: A→C
  - id_A        = morphism đồng nhất: id_A: A→A

Mỗi nhóm R.k = một phạm trù con (subcategory):
  R.0: Phạm trù đại số — morphism = phép toán
  R.1: Phạm trù thứ tự — morphism = quan hệ so sánh
  R.2: Phạm trù biểu diễn — morphism = biến đổi font (functor)
  R.3: Phạm trù số — morphism = ánh xạ mã hóa vị trí
  R.4: Phạm trù ngôn ngữ hình thức — morphism = phép đẩy/rút stack
  R.5: Phạm trù tuyến tính — morphism = ánh xạ quy đổi tiền tệ
  R.6: Phạm trù cộng tính — morphism = phép cộng giá trị ký hiệu
  R.7: Phạm trù automat — morphism = hàm chuyển trạng thái

Functor F: R.i → R.j bảo toàn cấu trúc giữa các nhóm.
Natural transformation η: F ⟹ G cho phép chuyển đổi giữa các cách biểu diễn.
```

---

## R.0 — TOÁN TỬ (Operator) · 147 cụm

### Cơ sở toán học: Lý thuyết cấu trúc đại số

```
Toán tử = morphism trong các cấu trúc đại số (nhóm, vành, trường).

■ Nhóm Abel (Abelian Group) cho phép CỘNG/TRỪ:
  (ℤ, +) thỏa: ∀a,b,c ∈ ℤ
    Đóng:      a + b ∈ ℤ
    Kết hợp:   (a + b) + c = a + (b + c)
    Đơn vị:    a + 0 = 0 + a = a
    Nghịch đảo: a + (−a) = 0
    Giao hoán:  a + b = b + a

■ Vành (Ring) cho phép NHÂN/CHIA:
  (ℤ, +, ×) thỏa:
    (ℤ, +) là nhóm Abel
    Kết hợp nhân: (a × b) × c = a × (b × c)
    Phân phối:    a × (b + c) = a×b + a×c

■ Trường (Field) cho phép chia đầy đủ:
  (ℝ, +, ×) thỏa: mọi a ≠ 0 có a⁻¹ sao cho a × a⁻¹ = 1

■ Tích phân = Giới hạn tổng Riemann:
  ∫ₐᵇ f(x)dx = lim_{n→∞} Σᵢ₌₁ⁿ f(xᵢ*)·Δxᵢ
  Tích phân đường: ∮_C f·ds = ∫ₐᵇ f(r(t))·|r'(t)|dt
  Tích phân mặt:  ∬_S f·dS = ∫∫_D f(r(u,v))·|rᵤ×rᵥ|dudv

■ Vi phân = Đạo hàm và toán tử vi phân:
  ∂f/∂xᵢ = lim_{h→0} [f(x+heᵢ) − f(x)] / h
  ∇f = (∂f/∂x₁, ∂f/∂x₂, ..., ∂f/∂xₙ)   — gradient
  ∇·F = Σᵢ ∂Fᵢ/∂xᵢ                       — divergence
  ∇×F = det|ê₁ ê₂ ê₃; ∂/∂x ∂/∂y ∂/∂z; F₁ F₂ F₃|  — curl

■ Toán tử vòng (Circled operators) = phép toán trên nhóm thương:
  a ⊕ b = (a + b) mod n    trong ℤ/nℤ  (cộng modular)
  a ⊗ b = (a × b) mod n    trong ℤ/nℤ  (nhân modular)
  Tổng trực tiếp: V ⊕ W = {(v,w) | v∈V, w∈W}

■ Tổng/Tích n-ary:
  Σᵢ₌₁ⁿ aᵢ = a₁ + a₂ + ... + aₙ
  ∏ᵢ₌₁ⁿ aᵢ = a₁ × a₂ × ... × aₙ

■ Căn = nghịch đảo lũy thừa:
  ⁿ√a = a^(1/n),  nghĩa là (ⁿ√a)ⁿ = a
```

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

### Cơ sở toán học: Lý thuyết thứ tự bộ phận & Không gian metric

```
So sánh = quan hệ trên tập hợp, phân loại theo tính chất đại số.

■ Quan hệ tương đương (Equivalence relation) cho BẰNG:
  R ⊆ S×S là tương đương ⟺
    Phản xạ:   ∀a: aRa
    Đối xứng:  aRb ⟹ bRa
    Bắc cầu:   aRb ∧ bRc ⟹ aRc
  Lớp tương đương: [a] = {b ∈ S | aRb}
  Tập thương: S/R = {[a] | a ∈ S}

■ Thứ tự bộ phận (Partial order) cho LỚN HƠN / NHỎ HƠN:
  (P, ≤) là poset ⟺
    Phản xạ:   ∀a: a ≤ a
    Phản đối xứng: a ≤ b ∧ b ≤ a ⟹ a = b
    Bắc cầu:   a ≤ b ∧ b ≤ c ⟹ a ≤ c
  Thứ tự toàn phần: thêm ∀a,b: a ≤ b ∨ b ≤ a  (VD: ℝ với ≤)

■ Thứ tự bao hàm (Inclusion order) cho TẬP HỢP:
  (𝒫(X), ⊆) là poset, với:
    A ⊆ B ⟺ ∀x: x ∈ A ⟹ x ∈ B
    A ⊂ B ⟺ A ⊆ B ∧ A ≠ B    (tập con thật sự)
    A ⊇ B ⟺ B ⊆ A             (tập chứa)

■ Không gian metric cho GẦN BẰNG / TƯƠNG TỰ:
  (M, d) là không gian metric ⟺ d: M×M → ℝ≥0 thỏa:
    d(a,b) = 0 ⟺ a = b
    d(a,b) = d(b,a)
    d(a,c) ≤ d(a,b) + d(b,c)    — bất đẳng thức tam giác
  Xấp xỉ: a ≈ b ⟺ d(a,b) < ε   (với ε > 0 tùy ý nhỏ)
  Tiệm cận: f ~ g ⟺ lim_{x→∞} f(x)/g(x) = 1

■ Đồng dư (Congruence):
  a ≡ b (mod n) ⟺ n | (a − b)
  Tương đương hình học: △ABC ≅ △DEF ⟺ cạnh & góc bằng nhau

■ Song song & Vuông góc trong không gian vector:
  u ∥ v ⟺ ∃λ: u = λv       (tỷ lệ)
  u ⊥ v ⟺ u·v = 0           (tích vô hướng = 0)
```

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

### Cơ sở toán học: Lý thuyết biểu diễn & Biến đổi không gian vector

```
Mỗi kiểu font = một phép biến đổi tuyến tính T: V → V trên không gian vector.
Ký tự gốc = vector v ∈ V, ký tự được styled = T(v).

■ Bold = Phép co giãn (scaling transformation):
  T_bold(v) = α·v,  α > 1
  Ma trận: [α 0; 0 α]  — phóng to đều theo mọi hướng

■ Italic = Phép cắt (shear transformation):
  T_italic(v) = [1 k; 0 1]·v,  k = tan(θ) với θ = góc nghiêng
  Bảo toàn diện tích: det[1 k; 0 1] = 1

■ Bold Italic = Hợp phép biến đổi:
  T_bold_italic = T_bold ∘ T_italic
  Ma trận: [α αk; 0 α]

■ Double-struck (𝔸, ℝ, ℂ, ...) = Phép chiếu lên không gian con:
  P: V → W ⊂ V,  P² = P  (idempotent)
  Biểu diễn tập số đặc biệt: ℕ ⊂ ℤ ⊂ ℚ ⊂ ℝ ⊂ ℂ

■ Fraktur (𝔄, 𝔅, ...) = Cơ sở lưới rời rạc (lattice basis):
  Λ = {n₁b₁ + n₂b₂ | nᵢ ∈ ℤ}  — lưới 2D với cơ sở {b₁, b₂}
  Dùng trong lý thuyết Lie algebras: 𝔤, 𝔥, 𝔰𝔩(n)

■ Script (𝒜, ℬ, ...) = Phép biến đổi affine (bảo toàn tỷ lệ):
  T_script(v) = A·v + t,  A ∈ GL(2,ℝ), t = vector tịnh tiến

■ Monospace = Phép chiếu lên lưới đều:
  T_mono(v) = round(v / Δ) × Δ    — lượng tử hóa vào lưới bước Δ

■ Hệ chữ khác nhau = Các không gian biểu diễn:
  Latin:  V_L = span{A, B, ..., Z}     dim = 26
  Greek:  V_G = span{Α, Β, ..., Ω}     dim = 24
  Arabic: V_A = span{ا, ب, ..., ي}      dim = 28
  Đồng cấu φ: V_L → V_G bảo toàn cấu trúc đại số
```

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

### Cơ sở toán học: Mã hóa số theo hệ vị trí (Positional Numeral Encoding)

```
Mọi hệ đếm vị trí biểu diễn giá trị bằng tổng có trọng số.

■ Hệ đếm cơ số b (Positional notation):
  Giá trị N = Σᵢ₌₀ⁿ dᵢ × bⁱ
  với dᵢ = chữ số tại vị trí i, b = cơ số

  VD: Hệ 10 (thập phân):  347 = 3×10² + 4×10¹ + 7×10⁰
  VD: Hệ 60 (Babylon):    giá trị = d₁×60¹ + d₀×60⁰
  VD: Que đếm (Trung Hoa): cơ số 10, dùng que tính biểu diễn dᵢ

■ La Mã = Hệ cộng-trừ (Additive-subtractive system):
  Quy tắc: Nếu ký hiệu nhỏ đứng TRƯỚC lớn → TRỪ, ngược lại → CỘNG
  N = Σⱼ sⱼ × val(symbolⱼ)    với sⱼ ∈ {+1, −1}
  VD: XIV = X + (−I) + V = 10 − 1 + 5 = 14
  VD: MCMXCIV = 1000 + (−100+1000) + (−10+100) + (−1+5) = 1994
  Bảng giá trị: I=1, V=5, X=10, L=50, C=100, D=500, M=1000

■ Phân số = Số hữu tỷ (Rational numbers):
  ℚ = {p/q | p ∈ ℤ, q ∈ ℤ\{0}}
  Tương đương: p/q ~ r/s ⟺ p×s = q×r
  Vulgar fractions: ½ = 1/2, ¼ = 1/4, ¾ = 3/4, ...
  Phần thập phân: p/q = Σᵢ₌₁^∞ dᵢ × 10⁻ⁱ  (khai triển thập phân)

■ Hình nêm Lưỡng Hà (Cuneiform) = Hệ cơ số 60:
  N = Σᵢ₌₀ⁿ dᵢ × 60ⁱ,  dᵢ ∈ {0, 1, ..., 59}
  Mỗi dᵢ viết dạng cộng: dᵢ = 10a + b (a = số chục, b = đơn vị)

■ Số Ấn Độ / Ottoman = Biến thể ký hiệu của hệ thập phân:
  Cùng công thức N = Σ dᵢ × 10ⁱ, chỉ khác glyphs cho dᵢ
```

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

### Cơ sở toán học: Lý thuyết ngôn ngữ hình thức & Automat đẩy xuống

```
Dấu câu (đặc biệt ngoặc) = phép toán trên stack của Pushdown Automaton.

■ Pushdown Automaton (PDA) cho ngoặc cân bằng:
  PDA = (Q, Σ, Γ, δ, q₀, Z₀, F)
    Q = {q₀}           — tập trạng thái (1 trạng thái đủ)
    Σ = {(, ), [, ], {, }, ⟨, ⟩, ...}  — bảng chữ cái đầu vào
    Γ = {(, [, {, ⟨, Z₀}              — bảng chữ cái stack
    Z₀ = ký hiệu đáy stack

  Hàm chuyển δ:
    δ(q₀, '(', γ)  = (q₀, '(' · γ)     — PUSH: gặp mở → đẩy vào stack
    δ(q₀, ')', '(') = (q₀, ε)           — POP:  gặp đóng → rút khỏi stack
    δ(q₀, ']', '[') = (q₀, ε)           — POP:  ] khớp [
    δ(q₀, '}', '{') = (q₀, ε)           — POP:  } khớp {

  Cân bằng ⟺ stack = Z₀ (rỗng) khi hết chuỗi đầu vào.

■ Ngữ pháp phi ngữ cảnh (CFG) sinh ngoặc cân bằng:
  S → SS | (S) | [S] | {S} | ⟨S⟩ | ε
  Đây là ngôn ngữ Dyck D_k với k loại ngoặc.
  Số Catalan: Cₙ = (2n)! / ((n+1)! × n!)  = số cách đặt n cặp ngoặc

■ Dấu ngắt câu = Ký hiệu phân tách trong ngữ pháp hình thức:
  Dấu chấm (.) : terminal marker — kết thúc câu (production rule)
  Dấu phẩy (,) : delimiter      — phân tách thành phần trong danh sách
  Dấu hai chấm (:) : separator  — ngăn cách label và nội dung
  Dấu chấm lửng (...) : continuation — biểu diễn chuỗi vô hạn

■ Ngoặc kép = Toán tử quoting (metalanguage):
  "X" = tên gọi của X (mention vs. use distinction)
  Gödel number: ⌈φ⌉ = mã hóa số học của công thức φ
```

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

### Cơ sở toán học: Đại số tuyến tính của quy đổi tiền tệ

```
Hệ thống tiền tệ = không gian vector, tỷ giá = ánh xạ tuyến tính.

■ Không gian tiền tệ:
  Mỗi đơn vị tiền tệ cᵢ = một vector cơ sở trong ℝⁿ
  C = span{c₁, c₂, ..., cₙ}  với n = số loại tiền
  Một khoản tiền = vector v = Σ aᵢcᵢ  (aᵢ = số lượng đơn vị tiền cᵢ)

■ Ma trận tỷ giá (Exchange rate matrix):
  R ∈ ℝⁿˣⁿ,  Rᵢⱼ = tỷ giá từ tiền j sang tiền i
  Quy đổi: v_target = R · v_source
  VD: [USD; EUR; VND] = R · [1 JPY; 0; 0]

■ Tính chất đại số của R:
  Rᵢᵢ = 1                        — 1 đơn vị = chính nó
  Rᵢⱼ × Rⱼᵢ = 1                  — nghịch đảo (lý tưởng, bỏ qua spread)
  Rᵢₖ = Rᵢⱼ × Rⱼₖ               — bắc cầu (arbitrage-free condition)
  det(R) = 1                     — khi thị trường cân bằng hoàn hảo

■ Arbitrage = vi phạm bắc cầu:
  Nếu Rᵢⱼ × Rⱼₖ × Rₖᵢ > 1 → có lợi nhuận chênh lệch giá
  Điều kiện no-arbitrage: ∀ chu trình (i→j→k→...→i): ∏ Rₐᵦ = 1

■ Tiền mã hóa (Bitcoin):
  Giá trị = hàm cung-cầu trên blockchain
  BTC/USD = f(supply, demand, hash_rate, ...)
  Tổng cung hữu hạn: Σ = 21 × 10⁶ BTC (giới hạn cứng)

■ Ký hiệu tiền = ánh xạ ký hiệu → không gian tiền tệ:
  σ: Symbols → C,   σ($) = USD, σ(€) = EUR, σ(₫) = VND
  Đây là một injective map (mỗi ký hiệu ↦ đúng 1 loại tiền)
```

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

### Cơ sở toán học: Hệ số cộng tính (Additive Numeral Systems)

```
Số cổ đại = hệ CỘNG tính thuần túy (không có trọng số vị trí).

■ Hệ cộng tính (Additive system):
  Giá trị N = Σⱼ val(symbolⱼ)
  Không phụ thuộc vị trí — chỉ cộng giá trị các ký hiệu.
  Khác hệ vị trí: KHÔNG có N = Σ dᵢ × bⁱ

■ Hệ Acrophonic Hy Lạp (Greek Acrophonic):
  Nguyên tắc Acrophonic: ký hiệu = chữ cái ĐẦU TIÊN của tên số
  VD: Π (Πέντε = Pente = 5), Δ (Δέκα = Deka = 10),
      Η (Ηεκατόν = Hekaton = 100), Χ (Χίλιοι = Khilioi = 1000),
      Μ (Μύριοι = Myrioi = 10000)

  Bảng giá trị Attic: {I:1, Π:5, Δ:10, Η:100, Χ:1000, Μ:10000}
  Tổ hợp nhân: 𐅄 = Π×Δ = 50, 𐅅 = Π×Η = 500, 𐅆 = Π×Χ = 5000

  Công thức tổng quát:
    N = n_Μ×10000 + n_Χ×1000 + n_Η×100 + n_Δ×10 + n_Π×5 + n_I×1
    Mỗi ký hiệu lặp tối đa 4 lần: nₖ ∈ {0,1,2,3,4}

■ Biến thể theo vùng:
  Epidaurean, Hermionian, Messenian, Naxian, Troezenian:
  Cùng nguyên tắc cộng tính, khác glyph cho cùng giá trị.
  Đẳng cấu: φ: Attic → Epidaurean,  φ bảo toàn val(·)

■ Đơn vị tiền/đo lường cổ:
  Stater, Talent, Drachma, Mina = đơn vị trong "không gian đo lường"
  VD: 1 Talent = 60 Mina = 6000 Drachma
  Ánh xạ: val: Symbols × Units → ℚ
  "FIVE STATERS" = val(Π) × unit(stater) = 5 stater

■ Ottoman Siyaq = Biến thể cộng tính của chữ số Ả Rập:
  Dùng ký tự biến dạng từ chữ số Ả Rập, hệ thập phân cộng tính.
```

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

### Cơ sở toán học: Lý thuyết automat hữu hạn (Finite Automaton Theory)

```
Ký tự điều khiển = hàm chuyển trạng thái trong máy trạng thái hữu hạn.

■ Automat hữu hạn đơn định (DFA):
  M = (Q, Σ, δ, q₀, F)
    Q  = tập hữu hạn trạng thái (VD: {normal, escape, bidi_LTR, bidi_RTL})
    Σ  = bảng chữ cái đầu vào (bao gồm control chars)
    δ  = hàm chuyển: Q × Σ → Q
    q₀ = trạng thái ban đầu
    F  = tập trạng thái chấp nhận

■ Ký hiệu control = hàm chuyển trạng thái δ:
  δ(q, NUL) → q         — null: không đổi trạng thái
  δ(q, BEL) → q_alert   — bell: kích hoạt cảnh báo
  δ(q, BS)  → q_back    — backspace: lùi con trỏ
  δ(q, LF)  → q_newline — line feed: xuống dòng
  δ(q, CR)  → q_return  — carriage return: về đầu dòng
  δ(q, ESC) → q_escape  — escape: vào chế độ escape sequence
  δ(q, DEL) → q_delete  — delete: xóa ký tự

■ Ký tự định dạng Bidi = automat xếp chồng (stack-based):
  PUSH: LRE (Left-to-Right Embedding), RLE (Right-to-Left Embedding)
  PUSH: LRO (Left-to-Right Override), RLO (Right-to-Left Override)
  POP:  PDF (Pop Directional Formatting)

  Stack S: [direction₁, direction₂, ...]
  δ(q, LRE): S.push(LTR),  current_dir = LTR
  δ(q, RLE): S.push(RTL),  current_dir = RTL
  δ(q, PDF): S.pop(),      current_dir = S.top()
  Mức nhúng tối đa: depth(S) ≤ 125 (theo Unicode spec)

■ Ký tự ngăn cách = toán tử trên chuỗi:
  ZWJ (Zero Width Joiner):    σ = ab → a‍b  (nối, giữ ligature)
  ZWNJ (Zero Width Non-Joiner): σ = ab → a‌b  (tách, phá ligature)
  Mô hình: φ_join: Σ* × Σ* → Σ*   (phép nối có điều kiện)

■ Biểu diễn trạng thái dưới dạng đại số Boolean:
  Mỗi trạng thái q = vector bit: q ∈ {0,1}ⁿ
  Hàm chuyển δ = hàm Boolean: δ(q,σ) = f_σ(q)
  VD: ACTIVATE → bit_i = 1,  INHIBIT → bit_i = 0
```

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

### Bảng công thức đặc trưng

```
┌─────────────────┬───────────────────────────────────────────────────────────────┐
│ Nhóm            │ Công thức đặc trưng                                          │
├─────────────────┼───────────────────────────────────────────────────────────────┤
│ R.0 Toán tử     │ (G,∗): a∗b∈G, closure + assoc + identity + inverse           │
│                 │ ∫ₐᵇ f(x)dx = lim Σ f(xᵢ*)·Δxᵢ                              │
├─────────────────┼───────────────────────────────────────────────────────────────┤
│ R.1 So sánh     │ Tương đương: phản xạ + đối xứng + bắc cầu                   │
│                 │ Xấp xỉ: d(a,b) < ε trong không gian metric                  │
├─────────────────┼───────────────────────────────────────────────────────────────┤
│ R.2 Chữ toán    │ Font = biến đổi tuyến tính T: V→V                            │
│                 │ Bold=[α 0;0 α], Italic=[1 k;0 1], P²=P (double-struck)      │
├─────────────────┼───────────────────────────────────────────────────────────────┤
│ R.3 Số          │ Hệ vị trí: N = Σᵢ dᵢ × bⁱ                                  │
│                 │ La Mã: N = Σⱼ sⱼ×val(symbolⱼ), sⱼ∈{+1,−1}                  │
├─────────────────┼───────────────────────────────────────────────────────────────┤
│ R.4 Dấu câu    │ Ngoặc: PDA δ(q,open)=PUSH, δ(q,close)=POP                   │
│                 │ Cân bằng ⟺ stack rỗng; Catalan Cₙ=(2n)!/((n+1)!n!)         │
├─────────────────┼───────────────────────────────────────────────────────────────┤
│ R.5 Tiền tệ    │ v_target = R·v_source, R∈ℝⁿˣⁿ                               │
│                 │ No-arbitrage: ∏ Rₐᵦ = 1 trên mọi chu trình                  │
├─────────────────┼───────────────────────────────────────────────────────────────┤
│ R.6 Cổ đại     │ Cộng tính: N = Σⱼ val(symbolⱼ) (không vị trí)               │
│                 │ Acrophonic: ký hiệu = chữ cái đầu tên số                    │
├─────────────────┼───────────────────────────────────────────────────────────────┤
│ R.7 Điều khiển │ DFA: δ(q,σ) → q'  (hàm chuyển trạng thái)                   │
│                 │ Bidi stack: PUSH(LRE/RLE), POP(PDF), depth ≤ 125            │
└─────────────────┴───────────────────────────────────────────────────────────────┘
```

### Mô hình tổng quát — Category Theory

```
R = Phạm trù (Category) với:
  Ob(R)  = {Toán tử, Quan hệ so sánh, Ký tự, Số, Dấu câu, Tiền tệ, Số cổ, Control}
  Hom    = morphism (phép biến đổi / ánh xạ) giữa các đối tượng
  ∘      = hợp morphism (bắc cầu)
  id     = morphism đồng nhất

Functor F: R.i → R.j bảo toàn cấu trúc:
  F(id_A) = id_{F(A)}
  F(g ∘ f) = F(g) ∘ F(f)
```
