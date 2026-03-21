# R — Relation · Công thức toán học thật

> Mỗi nhóm = 1 công thức logic/đại số thật sự.
> Không đánh số tuple — chỉ toán, chỉ logic, chỉ đại số.

---

## Tổng quan: R là gì?

```
R: (A, B) → {true, false}   hoặc   R: A → B

R là quan hệ toán học giữa các đối tượng.
Mỗi ký tự R là 1 phép toán hoặc 1 quan hệ có ý nghĩa đại số thật.
```

---

## R.0 — Toán tử (Operator)

### Phép số học cơ bản

```
PLUS  +:     f(a, b) = a + b                    phép cộng trên ℝ
MINUS −:     f(a, b) = a − b                    phép trừ
TIMES ×:     f(a, b) = a · b                    phép nhân
DIVIDE ÷:    f(a, b) = a / b,  b ≠ 0            phép chia
```

### Tích phân — họ ∫

```
INTEGRAL ∫:           ∫ₐᵇ f(x) dx                       tích phân Riemann
CONTOUR INTEGRAL ∮:   ∮_γ f(z) dz                       tích phân đường cong kín
SURFACE INTEGRAL ∯:   ∯_S F · dS                        tích phân mặt
VOLUME INTEGRAL ∰:    ∰_V f dV                          tích phân thể tích

Chiều xoay:
  CLOCKWISE ∫:        ∮_γ⁺ f(z) dz       γ thuận kim đồng hồ
  ANTICLOCKWISE ∮:    ∮_γ⁻ f(z) dz       γ ngược kim đồng hồ
```

### Tổng/Tích — họ Σ, ∏

```
SUMMATION Σ:    Σᵢ₌₁ⁿ aᵢ = a₁ + a₂ + ··· + aₙ
PRODUCT ∏:      ∏ᵢ₌₁ⁿ aᵢ = a₁ · a₂ · ··· · aₙ
COPRODUCT ∐:    ∐ᵢ Aᵢ     disjoint union (phạm trù)
```

### Căn — họ √

```
SQUARE ROOT √:   √x = x^(1/2)
CUBE ROOT ∛:     ∛x = x^(1/3)
FOURTH ROOT ∜:   ∜x = x^(1/4)

Tổng quát:       ⁿ√x = x^(1/n)
```

### Vi phân — họ ∂, ∇

```
PARTIAL ∂:      ∂f/∂x = lim_{h→0} [f(x+h,y) − f(x,y)] / h
NABLA ∇:        ∇f = (∂f/∂x, ∂f/∂y, ∂f/∂z)              gradient
                ∇ · F = ∂Fx/∂x + ∂Fy/∂y + ∂Fz/∂z         divergence
                ∇ × F                                       curl
LAPLACIAN ∆:    ∆f = ∇²f = ∂²f/∂x² + ∂²f/∂y² + ∂²f/∂z²
```

### Toán tử vòng — họ ⊕, ⊖, ⊗, ⊙

```
CIRCLED PLUS ⊕:    a ⊕ b = (a + b) mod n          cộng modular / XOR
CIRCLED MINUS ⊖:   a ⊖ b = (a − b) mod n          trừ modular
CIRCLED TIMES ⊗:   a ⊗ b = tích tensor              tensor product
CIRCLED DOT ⊙:     a ⊙ b = Hadamard product         nhân từng phần tử
```

### Dot/Ring — họ ·, ∘

```
DOT OPERATOR ·:     a · b = Σ aᵢbᵢ                  tích vô hướng
RING OPERATOR ∘:    f ∘ g = f(g(x))                  hợp hàm (composition)
BULLET •:           đánh dấu (không phép toán)
```

---

## R.1 — So sánh (Comparison)

### Quan hệ bằng

```
EQUAL =:                a = b    ⟺  a − b = 0
NOT EQUAL ≠:            a ≠ b    ⟺  |a − b| > 0
IDENTICAL ≡:            a ≡ b    ⟺  ∀x: P(a,x) ↔ P(b,x)       đồng nhất
CONGRUENT ≅:            A ≅ B    ⟺  ∃ isomorphism f: A → B
APPROXIMATELY ≈:        a ≈ b    ⟺  |a − b| < ε               xấp xỉ
ASYMPTOTICALLY ≃:       f ≃ g    ⟺  lim f/g = 1                tiệm cận
SIMILAR ∼:              A ∼ B    ⟺  ∃ scaling s: A = s·B
PROPORTIONAL ∝:         a ∝ b    ⟺  ∃k: a = k·b               tỉ lệ
```

### Quan hệ thứ tự

```
LESS THAN <:            a < b    ⟺  b − a > 0
GREATER THAN >:         a > b    ⟺  a − b > 0
LESS OR EQUAL ≤:        a ≤ b    ⟺  a < b ∨ a = b
GREATER OR EQUAL ≥:     a ≥ b    ⟺  a > b ∨ a = b
MUCH LESS ≪:            a ≪ b    ⟺  a/b → 0                    nhỏ hơn nhiều
MUCH GREATER ≫:         a ≫ b    ⟺  b/a → 0                    lớn hơn nhiều
PRECEDES ≺:             a ≺ b    (thứ tự bộ phận)
SUCCEEDS ≻:             a ≻ b    (thứ tự bộ phận)
```

### Quan hệ tập hợp

```
ELEMENT OF ∈:           a ∈ B    ⟺  a thuộc tập B
NOT ELEMENT ∉:          a ∉ B    ⟺  ¬(a ∈ B)
CONTAINS ∋:             A ∋ a    ⟺  a ∈ A
SUBSET ⊂:               A ⊂ B    ⟺  ∀x: x ∈ A → x ∈ B
SUPERSET ⊃:             A ⊃ B    ⟺  B ⊂ A
SUBSET OR EQUAL ⊆:      A ⊆ B    ⟺  A ⊂ B ∨ A = B
```

### Quan hệ hình học

```
PARALLEL ∥:             A ∥ B    ⟺  ∃k: d̂_A = ±d̂_B            cùng hướng
PERPENDICULAR ⊥:        A ⊥ B    ⟺  d̂_A · d̂_B = 0              trực giao
```

### Phủ định = biến đổi logic

```
Với mọi quan hệ R:
  NOT R = ¬R(a,b)       gạch chéo qua ký hiệu
  VD: ≠ = ¬(=),  ∉ = ¬(∈),  ⊄ = ¬(⊂)
```

---

## R.2 — Chữ cái toán (Math Letter)

### Công thức: Biến đổi typographic trên bảng chữ cái

```
MathLetter(cp) = T_style( char(script, index) )

char: (script, index) → ký tự gốc
  script ∈ {Latin, Greek, Arabic}
  index  ∈ [0, |alphabet|)

T_style: biến đổi font = phép biến hình trên glyph
  BOLD:          T(g) = g với stroke_width × k,  k > 1       nét dày hơn
  ITALIC:        T(g) = Shear_x(g, tan(12°))                 nghiêng
  BOLD ITALIC:   T(g) = Bold(Italic(g))                       kết hợp
  FRAKTUR:       T(g) = fractal_curve(g)                       nét gãy Gothic
  DOUBLE-STRUCK: T(g) = union(g, offset(g, δ))                nét đôi
  SCRIPT:        T(g) = bezier_cursive(g)                      nét viết tay
  MONOSPACE:     T(g) = scale_to_fixed_width(g, w₀)           cùng chiều rộng
  SANS-SERIF:    T(g) = remove_serifs(g)                       bỏ chân

Offset trong Unicode:
  cp = base_cp + script_offset + style_offset + case_offset + char_index
  Công thức CHÍNH XÁC — mỗi cp ánh xạ 1:1 tới (script, style, case, char)
```

---

## R.3 — Số (Number)

### Công thức: Ánh xạ giá trị

```
Num(cp) = value(system, digit_class, position)

Que đếm (Counting Rod):
  value = digit × 10^position
  digit ∈ {1,...,9},  position ∈ {0: đơn vị, 1: chục}
  Ký hiệu: ngang = đơn vị, dọc = chục (hoặc ngược lại)

Hình nêm (Cuneiform):
  value = Σ face_values                    hệ cộng dồn (base-60)
  VD: 𒐕 = 10 + 10 + 10 = 30

La Mã:
  value = Σ roman_values   với quy tắc trừ
  I=1, V=5, X=10, L=50, C=100, D=500, M=1000
  IV=4, IX=9, XL=40, XC=90, CD=400, CM=900

Phân số (Vulgar):
  value = p/q ∈ ℚ
  ½ = 0.5,  ⅓ = 0.333...,  ¼ = 0.25,  ⅛ = 0.125
```

---

## R.4 — Dấu câu (Punctuation)

### Công thức: Cấu trúc phân tách

```
Bracket(a, b):   ⟨a, b⟩ → ordered pair              ghép cặp
Parenthesis:     (expr) → grouping                    nhóm ưu tiên
Dash:            A — B  → range/break                 phạm vi / ngắt
Comma:           a, b   → sequence: ⟨a⟩ ∘ ⟨b⟩        chuỗi
Colon:           A : B  → ratio A/B hoặc mapping     tỷ lệ / ánh xạ
Semicolon:       S₁; S₂ → sequential composition     tuần tự
Ellipsis:        a,...  → continuation                 tiếp tục

Ngoặc là INVOLUTION:
  LEFT ∘ RIGHT = identity     mở rồi đóng = không đổi
  ( ∘ ) = id,   [ ∘ ] = id,   { ∘ } = id
```

---

## R.5 — Tiền tệ (Currency)

### Công thức: Ánh xạ giá trị quy đổi

```
Currency(cp) = (symbol, exchange_rate)

exchange_rate: Currency → ℝ⁺    (giá trị quy đổi)
  $1 = €0.92 = ¥157 = ₫25,400 = ...

Phép toán trên tiền:
  a·$ + b·€ = (a + b·rate_€/$) · $       quy đổi về cùng đơn vị
  Đây là vector space trên ℝ với basis = các loại tiền
```

---

## R.6 — Cổ đại (Ancient Numerals)

### Công thức: Hệ cộng dồn (additive numeral system)

```
value(cp) = base_value × unit_multiplier

Hy Lạp Acrophonic:
  value = Σ symbol_values                  hệ cộng dồn
  Πεντε (5) + Δέκα (10) + Ηεκατόν (100) = 115

  symbol_value(cp) = base × 10^magnitude
  base ∈ {1, 5}
  magnitude ∈ {0, 1, 2, 3, 4}             → 1, 10, 100, 1000, 10000

  Với đơn vị tiền tệ:
    value_currency = value × unit_weight
    unit_weight: stater=1, drachma=1/6, talent=60, mina=1/60_talent
```

---

## R.7 — Điều khiển (Control)

### Công thức: Automaton trạng thái

```
Control(cp) = transition(state, action)

state ∈ {normal, formatting, embedding, override}
action: State → State

NUL:   → state₀ (reset)
CR:    cursor → (0, y)           về đầu dòng
LF:    cursor → (x, y+1)        xuống dòng
BS:    cursor → (x−1, y)        lùi 1
DEL:   buffer[x] → ∅            xóa ký tự

LTR:   direction → left-to-right
RTL:   direction → right-to-left
POP:   direction → stack.pop()   trả về hướng trước

Tính chất:  CR ∘ LF = newline   (CR+LF = xuống dòng mới từ đầu)
```

---

## Compose — Quan hệ kết hợp

```
Transitive: R₁ ∘ R₂ = R₃
  VD: (a ∈ B) ∧ (B ⊂ C) → (a ∈ C)     ∈ ∘ ⊂ = ∈

Inverse: R⁻¹
  VD: ∈⁻¹ = ∋,   <⁻¹ = >,   ⊂⁻¹ = ⊃

Negation: ¬R
  VD: ¬(=) = ≠,   ¬(∈) = ∉,   ¬(⊂) = ⊄
```

---

## Tích phân tổng: o{R}

```
o{R} = tập hợp mọi quan hệ trong chiều Relation

     = { R.0_operators, R.1_comparisons, R.2_letters,
         R.3_numbers, R.4_punctuation, R.5_currency,
         R.6_ancient, R.7_control }

     = đại số quan hệ (relational algebra) trên Unicode
```
