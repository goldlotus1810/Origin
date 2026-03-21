# UDC Group Formulas — Tên gọi + Công thức toán học

> Mỗi nhóm = 1 công thức. Chỉ toán, không giải thích.

---

## Master Encoder

```
P(cp) = ( S(cp), R(cp), V(cp), A(cp), T(cp) ) ∈ Z₁₆ × Z₁₆ × Z₈ × Z₈ × Z₄
```

---

## S — Shape

### S.0 Mũi tên (Arrow)

```
Arrow(cp) = (τ, δ, ω, φ, λ) ∈ Z₈ × Z₁₁ × Z₄ × Z₄ × Z₅

τ (kiểu):    { 0:đơn, 1:đôi, 2:ba, 3:móc, 4:gạch, 5:lượn, 6:vòng, 7:gấp }
δ (hướng):   { 0:↑, 1:↓, 2:←, 3:→, 4:↗, 5:↖, 6:↘, 7:↙, 8:↔, 9:↕, 10:✦ }
ω (dày):     { 0:mỏng, 1:thường, 2:dày, 3:rất dày }
φ (tô):      { 0:mặc định, 1:đặc, 2:rỗng, 3:bóng }
λ (đuôi):    { 0:thẳng, 1:cong, 2:gấp, 3:móc, 4:không }
```

### S.1 Hình học (Geometric)

```
Geo(cp) = (σ, φ, μ, ξ) ∈ Z₁₀ × Z₆ × Z₅ × Z₆

σ (hình):    { 0:tròn, 1:vuông, 2:tam giác, 3:thoi, 4:sao, 5:chữ thập,
               6:đa giác, 7:elip, 8:chữ nhật, 9:hoa }
φ (tô):      { 0:đặc, 1:rỗng, 2:chấm, 3:nửa, 4:chéo, 5:lồng }
μ (cỡ):      { 0:rất nhỏ, 1:nhỏ, 2:thường, 3:vừa, 4:lớn }
ξ (modifier): { 0:không, 1:hướng, 2:viền tròn, 3:số cánh, 4:tâm, 5:xoay }
```

### S.2 Vẽ hộp (Box Drawing)

```
Box(cp) = (κ, ω, γ) ∈ Z₆ × Z₄ × Z₉

κ (kiểu nét): { 0:ngang, 1:dọc, 2:góc, 3:T-nối, 4:giao, 5:cung }
ω (dày):      { 0:mỏng, 1:dày, 2:đôi, 3:pha }
γ (góc):      { 0:↘, 1:↙, 2:↗, 3:↖, 4:├, 5:┤, 6:┬, 7:┴, 8:┼ }
```

### S.3 Chữ nổi (Braille)

```
Braille(cp) = β ∈ {0,1}⁸

β = (b₁, b₂, b₃, b₄, b₅, b₆, b₇, b₈)    bᵢ ∈ {0,1}

|β| = 2⁸ = 256 tổ hợp
```

### S.4 APL

```
APL(cp) = (α, m) ∈ Z₁₈ × Z₆

α (gốc):    { 0:α, 1:ι, 2:ω, 3:ρ, 4:∇, 5:Δ, 6:○, 7:◇, 8:★, 9:⎕,
               10:∘, 11:∩∪, 12:∧∨, 13:⊤⊥⊢⊣, 14:/, 15:,, 16:⌶, 17:⍬ }
m (modifier): { 0:không, 1:gạch dưới, 2:hai chấm, 3:ngã, 4:thanh dọc, 5:thanh ngang }
```

### S.5 Kỹ thuật (Technical)

```
Tech(cp) = (d, χ) ∈ Z₅ × Z_n

d (lĩnh vực): { 0:nha khoa, 1:điện, 2:hóa học, 3:đo lường, 4:thiết bị }
χ (chi tiết):  index trong lĩnh vực d
```

### S.6 Khối (Block)

```
Block(cp) = (κ, π, ρ) ∈ Z₆ × Z₈ × Q

κ (kiểu):   { 0:đầy, 1:nửa, 2:phần, 3:bóng, 4:góc, 5:cung }
π (vị trí):  { 0:trên, 1:dưới, 2:trái, 3:phải, 4:trên-trái, 5:trên-phải, 6:dưới-trái, 7:dưới-phải }
ρ (tỷ lệ):  ρ ∈ { 1/8, 1/4, 3/8, 1/2, 5/8, 3/4, 7/8, 1 }
```

### S.7 Khác

```
Other_S(cp) = (g, i) ∈ Z₉ × Z_n

g (nhóm phụ): { 0:con trỏ, 1:âm nhạc, 2:thời tiết, 3:cờ/dấu, 4:dụng cụ,
                 5:thiên văn, 6:trang trí, 7:tôn giáo, 8:linh tinh }
i (index):     thứ tự trong nhóm g
```

---

## R — Relation

### R.0 Toán tử (Operator)

```
Op(cp) = (o, m) ∈ Z₁₀ × Z₅

o (phép):    { 0:+−, 1:×÷, 2:∫, 3:Σ, 4:∏, 5:√, 6:∂∇, 7:⊕⊖⊗, 8:≀, 9:·∘ }
m (modifier): { 0:không, 1:vòng tròn, 2:đảo, 3:xoay, 4:chiều }
```

### R.1 So sánh (Comparison)

```
Cmp(cp) = (c, q) ∈ Z₁₀ × Z₅

c (kiểu):    { 0:=, 1:≠, 2:>, 3:<, 4:≡≅, 5:≈≃, 6:∼∝, 7:⊂⊃∈, 8:∥⊥, 9:≺≻ }
q (điều kiện): { 0:không, 1:hoặc bằng, 2:phủ định, 3:dấu phụ, 4:kép }
```

### R.2 Chữ cái toán (Math Letter)

```
MathLetter(cp) = (s, f, c, l) ∈ Z₄ × Z₁₃ × Z₂ × Z_n

s (hệ chữ):  { 0:Latin, 1:Greek, 2:Arabic, 3:Số }
f (font):     { 0:serif, 1:sans, 2:bold, 3:italic, 4:bold-italic,
                5:fraktur, 6:double-struck, 7:script, 8:bold-script,
                9:mono, 10:sans-bold, 11:sans-italic, 12:sans-bold-italic }
c (kiểu):    { 0:hoa, 1:thường }
l (ký tự):   index trong bảng chữ cái hệ s
```

### R.3 Số (Number)

```
Num(cp) = (η, β, ν) ∈ Z₆ × Z₅ × Z_n

η (hệ đếm):  { 0:que, 1:hình nêm, 2:La Mã, 3:Ấn Độ, 4:Ottoman, 5:khác }
β (bậc):      { 0:đơn vị, 1:chục, 2:trăm, 3:ngàn, 4:phân số }
ν (giá trị):  ν ∈ N
```

### R.4 Dấu câu (Punctuation)

```
Punct(cp) = (p, π) ∈ Z₇ × Z₆

p (loại):    { 0:ngoặc, 1:gạch, 2:chấm, 3:ngoặc kép, 4:sao/dao, 5:đoạn, 6:cuneiform }
π (vị trí):  { 0:trái/mở, 1:phải/đóng, 2:trên, 3:dưới, 4:đôi, 5:đảo }
```

### R.5 Tiền tệ (Currency)

```
Currency(cp) = (r, i) ∈ Z₇ × Z_n

r (khu vực): { 0:Châu Âu, 1:Châu Á, 2:Châu Mỹ, 3:Trung Đông, 4:mã hóa, 5:Hy Lạp cổ, 6:chung }
i (tiền):    index trong khu vực r
```

### R.6 Cổ đại (Ancient)

```
Ancient(cp) = (ζ, ν, u) ∈ Z₇ × N × Z₅

ζ (hệ):     { 0:Attic, 1:Epidaurean, 2:Hermionian, 3:Messenian, 4:Naxian, 5:Troezenian, 6:Ottoman }
ν (giá trị): ν ∈ { 1, 5, 10, 50, 100, 500, 1000, 5000, 50000 }
u (đơn vị):  { 0:thuần, 1:stater, 2:talent, 3:drachma, 4:mina }
```

### R.7 Điều khiển (Control)

```
Ctrl(cp) = (g, a) ∈ Z₄ × Z_n

g (nhóm):    { 0:ký hiệu control, 1:định dạng, 2:ngăn cách, 3:chiều chữ }
a (hành động): index trong nhóm g
```

---

## V — Valence

```
V(w) = quantize( v_raw(w) )

v_raw: Word → [-1.0, +1.0]       (tra NRC-VAD hoặc emoji subgroup)

quantize(x) = ⌊ (x + 1) / 2 × 7 + 0.5 ⌋    ∈ Z₈ = {0,1,...,7}

Thang:
  V = 0  ↔  v_raw ∈ [-1.0, -0.71)    rất tiêu cực
  V = 1  ↔  v_raw ∈ [-0.71, -0.43)
  V = 2  ↔  v_raw ∈ [-0.43, -0.14)   tiêu cực
  V = 3  ↔  v_raw ∈ [-0.14, +0.14)   trung tính
  V = 4  ↔  v_raw ∈ [+0.14, +0.43)
  V = 5  ↔  v_raw ∈ [+0.43, +0.71)   tích cực
  V = 6  ↔  v_raw ∈ [+0.71, +0.86)
  V = 7  ↔  v_raw ∈ [+0.86, +1.0]    rất tích cực
```

### V cụm từ — 7 nhóm chủ đề

```
V_theme(phrase) = θ ∈ Z₇

θ: { 0:cảm xúc tích cực, 1:cảm xúc tiêu cực, 2:giá trị đạo đức,
     3:quan hệ xã hội, 4:thành tựu, 5:sức khỏe, 6:khác }
```

---

## A — Arousal

```
A(w) = quantize( a_raw(w) )

a_raw: Word → [-1.0, +1.0]       (tra NRC-VAD hoặc emoji subgroup)

quantize(x) = ⌊ (x + 1) / 2 × 7 + 0.5 ⌋    ∈ Z₈ = {0,1,...,7}

Thang:
  A = 0  ↔  a_raw ∈ [-1.0, -0.71)    cực yên tĩnh
  A = 1  ↔  a_raw ∈ [-0.71, -0.43)
  A = 2  ↔  a_raw ∈ [-0.43, -0.14)   yên tĩnh
  A = 3  ↔  a_raw ∈ [-0.14, +0.14)   trung tính
  A = 4  ↔  a_raw ∈ [+0.14, +0.43)
  A = 5  ↔  a_raw ∈ [+0.43, +0.71)   kích thích
  A = 6  ↔  a_raw ∈ [+0.71, +0.86)
  A = 7  ↔  a_raw ∈ [+0.86, +1.0]    cực kích thích
```

### A cụm từ — 5 nhóm chủ đề

```
A_theme(phrase) = θ ∈ Z₅

θ: { 0:năng lượng cao, 1:năng lượng thấp, 2:công nghệ, 3:cấu trúc, 4:khác }
```

### Không gian V×A (2D cảm xúc)

```
Emotion(w) = ( V(w), A(w) ) ∈ Z₈ × Z₈

  (V cao, A cao)  = hào hứng, phấn khích
  (V cao, A thấp) = bình yên, hài lòng
  (V thấp, A cao) = giận dữ, hoảng sợ
  (V thấp, A thấp) = buồn bã, chán nản
```

---

## T — Time

### T.0 Quẻ Dịch (Hexagram)

```
Hex(cp) = h ∈ Z₆₄

h = Σᵢ₌₁⁶ yᵢ · 2^(i-1)     yᵢ ∈ {0:âm, 1:dương}    (6 hào)

Nhóm trạng thái:
  Ψ(h) ∈ { 0:sáng tạo, 1:phát triển, 2:ổn định, 3:chuyển đổi, 4:khó khăn, 5:phân tán, 6:tụ hợp }
```

### T.1 Tứ quái / Digram / Monogram

```
Tetra(cp) = t ∈ Z₈₁       (4 hào × 3 giá trị: âm/dương/trung)
Digram(cp) = d ∈ Z₉        (2 hào × 3 giá trị)
Mono(cp) = m ∈ Z₃          (1 hào × 3 giá trị)

t = Σᵢ₌₁⁴ yᵢ · 3^(i-1)     yᵢ ∈ {0,1,2}
```

### T.2 Byzantine

```
Byz(cp) = (κ, χ) ∈ Z₈ × Z_n

κ (loại):    { 0:agogi, 1:neume, 2:fthora, 3:diesis, 4:yfesis, 5:ison, 6:chronon, 7:khác }
χ (chi tiết): index trong loại κ

Agogi(cp) = tempo ∈ Z₈
  { 0:poli argi, 1:argi, 2:argoteri, 3:metria, 4:mesi, 5:gorgi, 6:gorgoteri, 7:poli gorgi }
```

### T.3 Znamenny

```
Znam(cp) = (τ, χ) ∈ Z₄ × Z_n

τ (loại):    { 0:combining mark, 1:neume, 2:tonal range, 3:priznak }
χ (chi tiết): index trong loại τ
```

### T.4 Nhạc phương Tây (Western Musical)

```
WestMusic(cp) = (κ, χ) ∈ Z₉ × Z_n

κ (loại):    { 0:nốt, 1:lặng, 2:khóa, 3:thăng/giáng, 4:dynamics,
               5:cấu trúc, 6:neume Tây, 7:ornament, 8:khác }

Nốt:
  Duration(cp) = d ∈ Z₈
    { 0:maxima, 1:longa, 2:breve, 3:tròn, 4:trắng, 5:đen, 6:móc đơn, 7:móc kép }
  d → trường độ = 2^(3-d) phách    (d=3: 1 phách, d=5: 1/4 phách, ...)

Dynamics:
  Dyn(cp) = δ ∈ Z₇
    { 0:piano, 1:mezzo piano, 2:mezzo forte, 3:forte, 4:rinforzando, 5:crescendo, 6:decrescendo }

Khóa:
  Clef(cp) = κ ∈ Z₃     { 0:C, 1:F, 2:G }
```

### T.5 Nhạc Hy Lạp cổ

```
Greek(cp) = (τ, i) ∈ Z₃ × Z_n

τ (loại):    { 0:instrumental, 1:vocal, 2:combining }
i (số):      index trong loại τ
```

---

## Bảng tổng hợp: Tên — Công thức — Kích thước

```
Tên                 Công thức                                    |Không gian|
─────────────────────────────────────────────────────────────────────────────
Master              P(cp) = (S,R,V,A,T)                          16×16×8×8×4

S.0 Mũi tên        Arrow(cp) = (τ,δ,ω,φ,λ) ∈ Z₈×Z₁₁×Z₄×Z₄×Z₅   7,040
S.1 Hình học        Geo(cp) = (σ,φ,μ,ξ) ∈ Z₁₀×Z₆×Z₅×Z₆           1,800
S.2 Vẽ hộp         Box(cp) = (κ,ω,γ) ∈ Z₆×Z₄×Z₉                    216
S.3 Chữ nổi        Braille(cp) = β ∈ {0,1}⁸                          256
S.4 APL             APL(cp) = (α,m) ∈ Z₁₈×Z₆                         108
S.5 Kỹ thuật       Tech(cp) = (d,χ) ∈ Z₅×Z_n                      ~5×5
S.6 Khối            Block(cp) = (κ,π,ρ) ∈ Z₆×Z₈×Q                    ~48
S.7 Khác            Other_S(cp) = (g,i) ∈ Z₉×Z_n                  ~9×42

R.0 Toán tử        Op(cp) = (o,m) ∈ Z₁₀×Z₅                          50
R.1 So sánh        Cmp(cp) = (c,q) ∈ Z₁₀×Z₅                         50
R.2 Chữ toán       MathLetter(cp) = (s,f,c,l) ∈ Z₄×Z₁₃×Z₂×Z_n    ~2,600
R.3 Số              Num(cp) = (η,β,ν) ∈ Z₆×Z₅×Z_n                  ~600
R.4 Dấu câu        Punct(cp) = (p,π) ∈ Z₇×Z₆                        42
R.5 Tiền tệ        Currency(cp) = (r,i) ∈ Z₇×Z_n                   ~100
R.6 Cổ đại         Ancient(cp) = (ζ,ν,u) ∈ Z₇×N×Z₅                 ~315
R.7 Điều khiển     Ctrl(cp) = (g,a) ∈ Z₄×Z_n                       ~60

V   Valence         V(w) = ⌊(v_raw+1)/2 × 7 + 0.5⌋ ∈ Z₈             8
V   Chủ đề         V_theme(phrase) = θ ∈ Z₇                           7

A   Arousal         A(w) = ⌊(a_raw+1)/2 × 7 + 0.5⌋ ∈ Z₈             8
A   Chủ đề         A_theme(phrase) = θ ∈ Z₅                           5

V×A Cảm xúc 2D     Emotion(w) = (V,A) ∈ Z₈×Z₈                      64

T.0 Quẻ Dịch       Hex(cp) = Σ yᵢ·2^(i-1) ∈ Z₆₄                    64
T.1 Tứ quái        Tetra(cp) = Σ yᵢ·3^(i-1) ∈ Z₈₁                  81
T.2 Byzantine       Byz(cp) = (κ,χ) ∈ Z₈×Z_n                       ~8×30
T.3 Znamenny        Znam(cp) = (τ,χ) ∈ Z₄×Z_n                      ~4×46
T.4 Nhạc Tây        WestMusic(cp) = (κ,χ) ∈ Z₉×Z_n                 ~9×34
T.5 Hy Lạp cổ      Greek(cp) = (τ,i) ∈ Z₃×Z_n                     ~3×24
```
