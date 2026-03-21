# T — Time · Công thức toán học thật

> Mỗi nhóm = 1 công thức sóng/spline/phase thật sự.
> T = chiều thời gian. Biểu diễn dao động, nhịp, biến đổi trạng thái.

---

## Tổng quan: T là gì?

```
T: t → f(t)    hàm trên trục thời gian

  Spline knots:   chuỗi điểm (tᵢ, yᵢ) trên trục t
  Waveform:       f(t) = Σ Aₖ · sin(ωₖt + φₖ)
  Phase state:    Ψ(t) ∈ {trạng thái rời rạc}

Mọi ký tự T đều mô tả 1 khía cạnh của tín hiệu trên trục thời gian.
```

---

## T.0 — Quẻ Dịch (Hexagram)

### Công thức: Trạng thái rời rạc 6-bit

```
Hex(cp) = Σᵢ₌₁⁶ yᵢ · 2^(i-1)     ∈ Z₆₄

yᵢ ∈ {0: âm (⚋), 1: dương (⚊)}     hào thứ i (từ dưới lên)

64 quẻ = 2⁶ trạng thái = spline rời rạc trên không gian pha
```

### Biến đổi giữa các quẻ

```
Biến hào: flip bit thứ k
  Hex' = Hex ⊕ 2^(k-1)           XOR = lật 1 hào

Quẻ đối (complement):
  Hex' = 63 − Hex                 lật tất cả 6 hào
  VD: Càn (111111) ↔ Khôn (000000)

Quẻ đảo (reverse):
  Hex' = reverse_bits(Hex)        đảo thứ tự hào
  VD: ☰ (111) + ☷ (000) → ☷ (000) + ☰ (111)

Entropy quẻ:
  H(Hex) = −Σ p(yᵢ) · log₂ p(yᵢ)
  Quẻ thuần (000000, 111111): H = 0       cực ổn định
  Quẻ lẫn (010101):           H = 1       cực biến đổi
```

### 7 nhóm trạng thái = 7 phase

```
Ψ(Hex) = phase_classify(Hex) ∈ Z₇

  Phase 0: SÁNG TẠO     Hex ∈ {Càn, Độn, Đại Hữu...}     năng lượng tăng
  Phase 1: PHÁT TRIỂN   Hex ∈ {Tiệm, Tấn, Ích...}         mở rộng
  Phase 2: ỔN ĐỊNH      Hex ∈ {Hằng, Trung Phu, Tiết...}  giữ vững
  Phase 3: CHUYỂN ĐỔI   Hex ∈ {Cách, Quy Muội, Tiệm...}  biến đổi
  Phase 4: KHÓ KHĂN     Hex ∈ {Truân, Mông, Kiển...}      suy giảm
  Phase 5: PHÂN TÁN     Hex ∈ {Hoán, Bác, Minh Di...}     tan rã
  Phase 6: TỤ HỢP       Hex ∈ {Tỵ, Tụy, Đồng Nhân...}   hội tụ

Chu kỳ: 0 → 1 → 2 → 3 → 4 → 5 → 6 → 0  (vòng lặp)
         tạo  phát  ổn   đổi  khó  tan  tụ  → tạo lại
```

---

## T.1 — Tứ quái / Digram / Monogram

### Công thức: Hệ 3 giá trị (ternary)

```
Tetragram(cp) = Σᵢ₌₁⁴ yᵢ · 3^(i-1)     ∈ Z₈₁

yᵢ ∈ {0: âm, 1: dương, 2: trung}       3-valued logic

81 tứ quái = 3⁴ trạng thái = lưới pha mịn hơn hexagram

Digram(cp) = Σᵢ₌₁² yᵢ · 3^(i-1)     ∈ Z₉
Monogram(cp) = y₁                     ∈ Z₃

So sánh resolution:
  Monogram: 3 trạng thái   (thô nhất)
  Digram:   9 trạng thái
  Hexagram: 64 trạng thái
  Tetragram: 81 trạng thái  (mịn nhất)
```

---

## T.2 — Byzantine (Nhạc Byzantine)

### Agogi: Tempo = tần số lấy mẫu

```
Tempo(cp) = BPM(agogi_level)

agogi_level ∈ Z₈:
  0: poli argi    BPM ≈ 40     T_beat = 1.50 s
  1: argi         BPM ≈ 52     T_beat = 1.15 s
  2: argoteri     BPM ≈ 66     T_beat = 0.91 s
  3: metria       BPM ≈ 80     T_beat = 0.75 s
  4: mesi         BPM ≈ 96     T_beat = 0.63 s
  5: gorgi        BPM ≈ 112    T_beat = 0.54 s
  6: gorgoteri    BPM ≈ 132    T_beat = 0.45 s
  7: poli gorgi   BPM ≈ 160    T_beat = 0.38 s

Công thức:  BPM(k) ≈ 40 · φ^(k/3)     φ = (1+√5)/2 ≈ 1.618   (Fibonacci!)
```

### Neume: Interval = ∆pitch

```
Neume(cp) = Δp ∈ Z   (bước nhảy pitch)

Ison:        Δp = 0     giữ nguyên pitch (drone)
Oligon:      Δp = +1    lên 1 bước
Petasti:     Δp = +1    lên 1 bước (biến thể)
Apostrofos:  Δp = −1    xuống 1 bước
Elafron:     Δp = −2    xuống 2 bước
Kentima:     Δp = +2    lên 2 bước

Giai điệu = chuỗi neumes:
  melody(t) = Σᵢ Δpᵢ · H(t − tᵢ)     H = Heaviside step

  → Giai điệu = hàm bậc thang (staircase function) trên trục t
```

### Fthora: Modulation = đổi điệu thức

```
Fthora(cp) = mode_shift ∈ Z₈

Chuyển từ điệu thức A sang B:
  scale_B = rotate(scale_A, Fthora)
  → Thay đổi tập hợp interval được phép
```

---

## T.3 — Znamenny (Neume Slavonic)

### Công thức: Contour = đường cong pitch

```
Znamenny_neume(cp) = contour: [0, duration] → pitch_offset

Mỗi neume = 1 đường cong pitch nhỏ:
  Kryuk:      contour(t) = +1              lên và giữ
  Stomitsa:   contour(t) = A·(1 − t/T)     xuống dần (linear decay)
  Golubchik:  contour(t) = A·sin(πt/T)     lên rồi xuống (hill)
```

### Combining marks: Biến đổi contour

```
Mark(cp) = transform(contour)

VYSOKO (cao):     contour' = contour + offset_up
NIZKO (thấp):     contour' = contour + offset_down
BORZAYA (nhanh):  contour' = contour(t/α),  α < 1     nén thời gian
KACHKA (lắc):     contour' = contour + A·sin(ω·t)      vibrato
LOMKA (gãy):      contour' = contour VỚI discontinuity  nhảy pitch
KRYZH (thập):     contour' = fermata(contour)           kéo dài
```

---

## T.4 — Nhạc phương Tây (Western Musical)

### Nốt nhạc: Duration = bước sóng

```
Duration(cp) = 2^(3−d) phách

d ∈ Z₈:
  0: maxima       = 8 phách     T = 8/BPM × 60s
  1: longa        = 4 phách
  2: breve        = 2 phách
  3: whole (tròn) = 1 phách     ← chuẩn
  4: half (trắng) = 1/2 phách
  5: quarter (đen)= 1/4 phách
  6: eighth (móc) = 1/8 phách
  7: sixteenth    = 1/16 phách

Công thức: Duration = 2^(3−d) phách = 2^(3−d) × 60/BPM giây
```

### Pitch: Khóa nhạc = frequency register

```
Clef(cp) = f_base    tần số cơ sở

G clef (Sol): f_base = 392 Hz    (G4)
C clef (Đô):  f_base = 262 Hz    (C4 = middle C)
F clef (Fa):  f_base = 175 Hz    (F3)

Pitch từ khóa:
  f(note) = f_base · 2^(n/12)     n = số nửa cung từ nốt chuẩn

  → Thang 12-TET: fₙ = 440 · 2^((n−69)/12) Hz
```

### Thăng/Giáng: Biến đổi tần số

```
SHARP #:     f' = f · 2^(1/12)      lên nửa cung     ≈ ×1.0595
FLAT ♭:      f' = f · 2^(-1/12)     xuống nửa cung   ≈ ×0.9439
NATURAL ♮:   f' = f_natural         trả về cung tự nhiên
DOUBLE SHARP: f' = f · 2^(2/12)     lên 1 cung
DOUBLE FLAT:  f' = f · 2^(-2/12)    xuống 1 cung
```

### Dynamics: Amplitude = cường độ âm

```
Amplitude(cp) = A(dyn_level)

dyn_level ∈ Z₇:
  0: piano (p)         A ≈ 0.15     ~45 dB
  1: mezzo piano (mp)  A ≈ 0.30     ~55 dB
  2: mezzo forte (mf)  A ≈ 0.50     ~65 dB
  3: forte (f)         A ≈ 0.70     ~75 dB
  4: rinforzando (rfz) A ≈ 0.85     ~80 dB   (đột ngột)
  5: crescendo (cresc) A(t) = A₀ + (A₁−A₀)·t/T    tăng dần (linear ramp)
  6: decrescendo       A(t) = A₁ − (A₁−A₀)·t/T    giảm dần

Công thức sóng âm tổng hợp:
  signal(t) = A(t) · sin(2π · f(t) · t + φ)
```

### Ornament: Biến đổi waveform

```
TRILL:       f(t) = f₀ + Δf · square(ω_trill · t)    lắc giữa 2 nốt
MORDENT:     f(t) = f₀ nhanh → (f₀+Δf) → f₀          chạm nốt trên rồi về
TURN:        f(t) = f₀ → f₁ → f₀ → f₋₁ → f₀          trên-gốc-dưới-gốc
ARPEGGIATO:  f(t) = chord_notes[⌊t/Δt⌋]               rải hợp âm tuần tự
GLISSANDO:   f(t) = f₀ + (f₁−f₀) · t/T                trượt liên tục
FERMATA:     duration' = duration × k,  k > 1          kéo dài tùy ý
```

### Cấu trúc: Phase boundary

```
BARLINE |:       phase_end(t)           kết thúc ô nhịp
DOUBLE BARLINE:  section_end(t)         kết thúc đoạn
FINAL BARLINE:   piece_end(t)           kết thúc bài
REPEAT :|:       t' = t_repeat_start    nhảy về đầu đoạn lặp
SEGNO 𝄋:        t' = t_segno           nhảy về dấu segno
CODA 𝄌:         t' = t_coda            nhảy tới coda
```

---

## T.5 — Nhạc Hy Lạp cổ

### Công thức: 2 kênh sóng

```
Instrumental: f_inst(t)    sóng nhạc cụ
Vocal:        f_vocal(t)   sóng giọng hát

Tổng hợp: f(t) = f_inst(t) + f_vocal(t)    2 lớp chồng

Mora (đơn vị thời gian Hy Lạp):
  TRISEME:    duration = 3 mora
  TETRASEME:  duration = 4 mora
  PENTASEME:  duration = 5 mora

  1 mora ≈ 1 short syllable ≈ 250ms
```

---

## Compose — Chuỗi thời gian

```
Nối tiếp (sequential):
  f(t) = f₁(t) · H(T₁−t) + f₂(t−T₁) · H(t−T₁)     H = Heaviside
  → Nốt 1 rồi đến nốt 2

Chồng (simultaneous):
  f(t) = f₁(t) + f₂(t)                                hợp âm = cộng sóng

Biến tốc:
  f'(t) = f(t/α)    α > 1: chậm lại,  α < 1: nhanh lên
```

---

## Tích phân tổng: o{T}

```
o{T} = ∫_T f(t) dt = tích phân toàn bộ tín hiệu trên trục thời gian

     = { T.0_hexagram ∪ T.1_tetragram ∪ T.2_byzantine
         ∪ T.3_znamenny ∪ T.4_western ∪ T.5_greek }

Spline interpretation:
  Mỗi ký tự T = 1 knot trên spline thời gian
  Chuỗi ký tự T = chuỗi knots → nội suy → waveform liên tục

  f(t) = Σₖ cₖ · B_{k,n}(t)     B-spline basis functions
```
