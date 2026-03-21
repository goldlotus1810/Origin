# T — TIME (Thời gian) · Cây phân loại bằng từ ngữ

> 7 nhóm cụm từ: Quẻ Dịch (64), Tứ quái/Digram (92), Byzantine (241),
> Znamenny (185), Nhạc phương Tây (306), Nhạc Hy Lạp cổ (0), Khác (70)

---

## Mô hình vật lý tổng quát

```
  Time = tham số sóng trong cơ học sóng

  Sóng tổng quát: ψ(x,t) = A · sin(kx − ωt + φ)

  Trong đó:
    A = amplitude (biên độ) → dynamics/cường độ
    ω = 2πf = angular frequency → pitch/cao độ
    T = 2π/ω = period → duration/trường độ
    k = 2π/λ = wave number → spatial frequency
    φ = phase → trạng thái ban đầu

  Mỗi ký tự T = 1 tham số hoặc 1 modifier của hàm sóng.
  Chuỗi ký tự T = superposition: Ψ = Σ Aₙ·sin(kₙx − ωₙt + φₙ)  (Fourier series)
```

---

## T.0 — QUẺ DỊCH (Hexagram) · 64 cụm

### Tầng 1: "Thuộc nhóm trạng thái nào?"

```
QUẺ DỊCH (64 trạng thái rời rạc)
├── SÁNG TẠO / KHỞI ĐẦU    "creative / beginning / innocence / enthusiasm"
├── PHÁT TRIỂN / TĂNG       "development / increase / advance / progress"
├── ỔN ĐỊNH / TRUNG TÂM    "duration / keeping / peace / contemplation"
├── CHUYỂN ĐỔI / THAY ĐỔI   "following / influence / encounter / breakthrough"
├── KHÓ KHĂN / GIẢM         "difficulty / decrease / obstruction / limitation"
├── PHÂN TÁN / KẾT THÚC     "dispersion / completion / darkening / exhaustion"
└── LỰC LƯỢNG / TỤ HỢP     "army / gathering / fellowship / power"
```

### Phân loại cụ thể

```
HEXAGRAM FOR THE CREATIVE                → sáng tạo (quẻ Càn ☰)
HEXAGRAM FOR DIFFICULTY AT THE BEGINNING  → khó khăn đầu
HEXAGRAM FOR YOUTHFUL FOLLY              → dại dột trẻ
HEXAGRAM FOR CONFLICT                    → xung đột
HEXAGRAM FOR THE ARMY                    → quân đội
HEXAGRAM FOR PEACE                       → hòa bình
HEXAGRAM FOR FELLOWSHIP                  → tình bạn
HEXAGRAM FOR THE POWER OF THE GREAT      → sức mạnh lớn
HEXAGRAM FOR PROGRESS                    → tiến bộ
HEXAGRAM FOR DARKENING OF THE LIGHT      → ánh sáng tối dần
HEXAGRAM FOR DECREASE                    → suy giảm
HEXAGRAM FOR INCREASE                    → tăng trưởng
HEXAGRAM FOR BREAKTHROUGH                → đột phá
HEXAGRAM FOR GATHERING TOGETHER          → tụ họp
HEXAGRAM FOR EXHAUSTION                  → kiệt sức
HEXAGRAM FOR DEVELOPMENT                 → phát triển
HEXAGRAM FOR ABUNDANCE                   → dồi dào
HEXAGRAM FOR DURATION                    → trường tồn
HEXAGRAM FOR BEFORE COMPLETION           → trước hoàn thành
HEXAGRAM FOR AFTER COMPLETION            → sau hoàn thành
→ "ah, quẻ dịch — [trạng thái thời gian/biến đổi]"
  Mỗi quẻ = 1 phase trong chu kỳ biến đổi
```

### Mô hình toán học

```
  Mô hình: Máy trạng thái hữu hạn (FSM) với 2⁶ = 64 trạng thái

  Mỗi quẻ = vector v⃗ ∈ {0,1}⁶ (6 hào × âm/dương)
    Bit 0 (hào 1) = sơ quái hạ (lower trigram) bit 0
    ...
    Bit 5 (hào 6) = sơ quái thượng (upper trigram) bit 2

  Quẻ = upper_trigram ⊗ lower_trigram  (tensor product of 2 trigrams)

  State transition: biến quẻ = flip 1 bit
    d(q₁, q₂) = Hamming distance = số hào khác nhau
    Transition graph: hypercube Q₆ (6-dimensional hypercube)

  Chu kỳ: 64 states form a directed graph of I Ching sequence
  Duality: mỗi quẻ có quẻ đối (complement: v⃗' = 1⃗ − v⃗)
```

### Góc pha theo nhóm ngữ nghĩa

```
  SÁNG TẠO / KHỞI ĐẦU     → φ = 0       (phase = 0, đầu chu kỳ)
  PHÁT TRIỂN / TĂNG        → φ = π/3     (pha tăng trưởng)
  ỔN ĐỊNH                  → φ = 2π/3    (đỉnh/ổn định)
  CHUYỂN ĐỔI               → φ = π       (điểm uốn)
  KHÓ KHĂN / GIẢM          → φ = 4π/3    (pha suy giảm)
  PHÂN TÁN / KẾT THÚC      → φ = 5π/3    (tiến về không)
```

---

## T.1 — TỨ QUÁI / DIGRAM / MONOGRAM · 92 cụm

### Tầng 1: "Loại quái nào?"

```
QUÁI
├── MONOGRAM     "monogram for" — 1 hào (âm/dương)
├── DIGRAM       "digram for" — 2 hào (4 tổ hợp)
└── TETRAGRAM    "tetragram for" — 4 hào (81 tổ hợp)
```

### Tầng 2 (Tetragram): "Trạng thái gì?"

```
TETRAGRAM
├── TĂNG / TIẾN    "advance / ascent / increase / branching out"
├── GIẢM / LÙI     "decrease / opposition / hindrance / barrier"
├── ỔN ĐỊNH        "centre / constancy / stillness / closeness"
├── THAY ĐỔI       "change / bold resolution / encounter"
├── SỨC MẠNH       "strength / hardness / endeavour / aggravation"
└── LIÊN KẾT       "unity / contact / gathering / fostering"
```

### Ví dụ cụ thể

```
MONOGRAM FOR EARTH                        → đơn quái + đất
DIGRAM FOR EARTH                          → nhị quái + đất
DIGRAM FOR EARTHLY HEAVEN                 → nhị quái + đất trời
DIGRAM FOR HEAVENLY EARTH                 → nhị quái + trời đất
DIGRAM FOR HUMAN EARTH                    → nhị quái + người đất
TETRAGRAM FOR ACCUMULATION                → tứ quái + tích lũy
TETRAGRAM FOR ADVANCE                     → tứ quái + tiến
TETRAGRAM FOR BARRIER                     → tứ quái + rào cản
TETRAGRAM FOR BOLD RESOLUTION             → tứ quái + quyết tâm
TETRAGRAM FOR CENTRE                      → tứ quái + trung tâm
TETRAGRAM FOR CHANGE                      → tứ quái + thay đổi
TETRAGRAM FOR CONTACT                     → tứ quái + tiếp xúc
TETRAGRAM FOR ENDEAVOUR                   → tứ quái + nỗ lực
TETRAGRAM FOR STILLNESS                   → tứ quái + tĩnh lặng
→ "ah, [mono/di/tetra]gram — [trạng thái]"
```

### Mô hình toán học

```
  Monogram: v ∈ {0,1} (binary, 2 states = 1 bit)
  Digram: v ∈ {0,1,2}² (ternary pairs)
  Tetragram: v ∈ {0,1,2}⁴ (ternary 4-tuples, 3⁴ = 81 states)

  Each hào has 3 values (not 2 like hexagram):
    0 = broken (âm), 1 = solid (dương), 2 = changing (biến)

  State space: |S| = 3⁴ = 81 (Tai Xuan Jing)
  Information: log₂(81) ≈ 6.34 bits per tetragram
```

---

## T.2 — BYZANTINE (Nhạc Byzantine) · 241 cụm

### Tầng 1: "Loại ký hiệu gì?"

```
BYZANTINE MUSICAL SYMBOL
├── AGOGI        "agogi" — tốc độ/tempo (7 mức)
├── NEUME        neume names — nốt giai điệu
├── FTHORA       "fthora / fhtora" — biến âm / chuyển điệu thức
├── DIESIS       "diesis" — nửa cung (vi phân cung)
├── YFESIS       "yfesis" — giảm nửa cung
├── ISON         "ison" — nốt giữ (drone)
├── CHRONON      "chronon / chronou" — đơn vị thời gian
└── CÁC NEUME KHÁC  apostrofos, climacus, clivis, oligon, petasti...
```

### Tầng 2 (Agogi): "Tốc độ nào?"

```
AGOGI (tempo)
├── POLI ARGI    "poli argi" — rất chậm
├── ARGI         "argi" — chậm
├── ARGOTERI     "argoteri" — hơi chậm
├── METRIA       "metria" — vừa
├── MESI         "mesi" — trung bình
├── GORGI        "gorgi" — nhanh
├── GORGOTERI    "gorgoteri" — hơi nhanh
└── POLI GORGI   "poli gorgi" — rất nhanh
```

### Ví dụ cụ thể

```
BYZANTINE MUSICAL SYMBOL AGOGI ARGI       → tempo + chậm
BYZANTINE MUSICAL SYMBOL AGOGI GORGI      → tempo + nhanh
BYZANTINE MUSICAL SYMBOL AGOGI METRIA     → tempo + vừa
BYZANTINE MUSICAL SYMBOL AGOGI POLI ARGI  → tempo + rất chậm
BYZANTINE MUSICAL SYMBOL AGOGI POLI GORGI → tempo + rất nhanh
BYZANTINE MUSICAL SYMBOL ANATRICHISMA     → neume + anatrichisma
BYZANTINE MUSICAL SYMBOL APOSTROFOS       → neume + apostrofos
BYZANTINE MUSICAL SYMBOL FTHORA SKLIRON   → chuyển điệu + skliron
BYZANTINE MUSICAL SYMBOL ISON             → nốt giữ
BYZANTINE MUSICAL SYMBOL OLIGON           → neume + oligon (lên 1)
BYZANTINE MUSICAL SYMBOL PETASTI          → neume + petasti (lên 1)
→ "ah, Byzantine [loại ký hiệu] [chi tiết]"
```

### Mô hình toán học

```
  Neume = Δpitch (thay đổi cao độ, không phải cao độ tuyệt đối)
    oligon = +1 step, petasti = +1 step (đi lên)
    apostrofos = −1 step (đi xuống)
    ison = 0 steps (giữ nguyên, drone)

  Melody m(t) = Σ Δpᵢ · H(t − tᵢ)  (tổng tích lũy các hàm bước)

  Agogi (tempo) — co giãn thời gian:
    t' = α·t  where α = hệ số tempo
    poli argi: α << 1 (rất chậm)
    metria: α = 1 (chuẩn)
    poli gorgi: α >> 1 (rất nhanh)

  Fthora = chuyển điệu thức = phép biến đổi cơ sở của không gian cao độ
  Diesis/Yfesis = vi chỉnh cung: Δf = ±ε (nhiễu loạn tần số)
```

### Công thức chi tiết theo loại ký hiệu

**Neume — Hàm bước giai điệu (melodic step function):**
```
  Mỗi neume n = toán tử trên cao độ hiện tại p(t):
    p(tₖ₊₁) = p(tₖ) + Δp(n)

  Bảng Δp(n) theo neume:
    oligon       = +1    (lên 1 bậc)
    petasti      = +1    (lên 1 bậc, variant khác)
    ison         =  0    (giữ nguyên — drone note)
    apostrofos   = -1    (xuống 1 bậc)
    elafron      = -1    (xuống nhẹ)
    kentemata    = +1    (nhấn lên)
    hypsili      = +2    (lên 2 bậc)
    chamili      = -2    (xuống 2 bậc)

  Giai điệu toàn bộ = tích lũy:
    p(tₙ) = p₀ + Σᵢ₌₁ⁿ Δpᵢ

  Biểu diễn liên tục (interpolation giữa các bước):
    p(t) = p₀ + Σᵢ Δpᵢ · σ((t − tᵢ)/τ)
    σ(x) = 1/(1+e⁻ˣ)  sigmoid (smooth step)
    τ = thời gian chuyển tiếp giữa 2 nốt
```

**Fthora — Biến đổi điệu thức (mode transformation):**
```
  Không gian cao độ P = span{d₁, d₂, ..., d₈} (octoechos — 8 điệu thức)

  Fthora = ma trận chuyển cơ sở: M: P_mode₁ → P_mode₂
    M_skliron  = chuyển sang điệu thức cứng (chromatic intervals lớn hơn)
    M_enharmonic = chuyển sang điệu thức vi phân (micro-intervals)

  Quãng giữa các bậc trong mỗi điệu thức (đơn vị: moria, 1 octave = 72 moria):
    Diatonic:    12, 10, 8, 12, 12, 10, 8   (tổng = 72)
    Chromatic:   6, 20, 4, 12, 6, 20, 4     (tổng = 72)
    Enharmonic:  12, 12, 6, 12, 12, 12, 6   (tổng = 72)

  Fthora F tại vị trí k: thay đổi chuỗi quãng từ bậc k trở đi
    intervals[k:] = F(intervals[k:])
```

**Diesis / Yfesis — Vi chỉnh cao độ (microtonal adjustment):**
```
  Diesis:  p → p + δ     (nâng lên δ moria, δ ∈ {2, 4, 6})
  Yfesis:  p → p − δ     (hạ xuống δ moria)

  Trong hệ 72 moria/octave:
    1 moria ≈ 16.67 cent (so với 100 cent/bán cung phương Tây)
    Diesis nhỏ nhất (δ=2) ≈ 33 cent ≈ 1/3 bán cung
    Diesis lớn nhất (δ=6) ≈ 100 cent = 1 bán cung

  Tần số sau vi chỉnh:
    f' = f × 2^(δ/72)    (với δ tính bằng moria)
```

**Chronon — Đơn vị thời gian (temporal unit):**
```
  Chronos protos χ = đơn vị thời gian nguyên tử (atomic beat)

  Trường độ nốt = bội số của χ:
    chronon đơn    = 1χ
    chronon đôi    = 2χ
    chronon ba     = 3χ (triplet)

  Thời gian thực: T_note = n × χ × (1/α)
    n = số chronon, α = hệ số agogi (tempo)

  Tỷ lệ trường độ Byzantine (khác Western 2ⁿ):
    Dựa trên tổ hợp {1, 2, 3} × χ
    Nhóm nhịp: 2χ (binary), 3χ (ternary), 2χ+3χ (mixed)
```

**Ison — Nốt giữ (drone/pedal point):**
```
  Ison I(t) = p_drone = const    ∀ t ∈ [t_start, t_end]

  Biểu diễn sóng: ψ_ison(t) = A · sin(2πf_drone · t)
    f_drone = tần số cố định (thường là bậc I hoặc V của điệu thức)

  Ison + melody = texture hai lớp:
    Ψ(t) = ψ_ison(t) + ψ_melody(t)
    ψ_melody(t) = A_m · sin(2πf_m(t) · t)    f_m(t) biến thiên theo neume
```

---

## T.3 — ZNAMENNY (Neume Slavonic) · 185 cụm

### Tầng 1: "Loại nào?"

```
ZNAMENNY
├── COMBINING MARK    "combining mark" — dấu phụ kết hợp
├── NEUME             znamenny neume names — nốt đơn
├── TONAL RANGE       "tonal range indicator" — chỉ quãng
└── PRIZNAK           "priznak" — dấu hiệu bổ sung
```

### Tầng 2 (Combining Mark): "Đánh dấu gì?"

```
MARK
├── ĐỘ CAO        "vysoko / nizko / povyshe" — cao/thấp/hơi cao
├── VỊ TRÍ         "on left / on right" — trái/phải
├── KIỂU NÉT       "curved / kupnaya / lomka" — cong/tròn/gãy
├── TỐC ĐỘ         "borzaya / borzy" — nhanh
└── ĐẶC BIỆT       "kachka / kryzh / dvoetochie" — lắc/thập/hai chấm
```

### Ví dụ cụ thể

```
ZNAMENNY COMBINING MARK BORZAYA          → dấu + nhanh
ZNAMENNY COMBINING MARK GORAZDO NIZKO    → dấu + rất thấp
ZNAMENNY COMBINING MARK GORAZDO VYSOKO   → dấu + rất cao
ZNAMENNY COMBINING MARK KACHKA           → dấu + lắc
ZNAMENNY COMBINING MARK KRYZH            → dấu + thập
ZNAMENNY COMBINING MARK LOMKA            → dấu + gãy
ZNAMENNY COMBINING MARK MALO POVYSHE ON LEFT  → dấu + hơi cao + trái
ZNAMENNY COMBINING MARK MALO POVYSHE ON RIGHT → dấu + hơi cao + phải
ZNAMENNY COMBINING LOWER TONAL RANGE INDICATOR → chỉ quãng thấp
→ "ah, Znamenny [loại] [chi tiết cao/thấp/nhanh/vị trí]"
```

### Mô hình toán học

```
  Mỗi dấu kết hợp = toán tử vi phân trên hàm cao độ p(t):
    vysoko (cao) = p(t) + Δ (dịch dương)
    nizko (thấp) = p(t) − Δ (dịch âm)
    borzaya (nhanh) = dp/dt tăng (gia tốc)
    lomka (gãy) = d²p/dt² có gián đoạn (góc gãy trong giai điệu)
    kachka (lắc) = p(t) + A·sin(ωt) (vibrato/tremolo)
```

### Công thức chi tiết theo loại dấu

**Dấu độ cao (Pitch modifiers) — Toán tử dịch chuyển:**
```
  Toán tử T_pitch: p(t) → p(t) + Δ

  Bảng dịch chuyển Δ (đơn vị: bậc trong thang âm Znamenny):
    gorazdo vysoko   = +3Δ₀   (rất rất cao)
    vysoko           = +2Δ₀   (rất cao)
    malo povyshe     = +Δ₀    (hơi cao)
    (không dấu)     =  0      (bình thường)
    malo ponizhe     = -Δ₀    (hơi thấp)
    nizko            = -2Δ₀   (rất thấp)
    gorazdo nizko    = -3Δ₀   (rất rất thấp)

  Δ₀ = bậc cơ sở ≈ 1 bậc trong osmoglasie (hệ 8 điệu thức)
  Chuỗi dịch: {..., -3, -2, -1, 0, +1, +2, +3} × Δ₀  (đối xứng quanh 0)
```

**Dấu tốc độ (Tempo modifiers) — Toán tử co giãn thời gian:**
```
  Toán tử T_tempo: dt → dt/β    (β > 1 = nhanh hơn, β < 1 = chậm hơn)

  borzaya (nhanh):
    β = 2   → dt' = dt/2  (thời gian nốt giảm một nửa)
    Gia tốc: a = d²p/dt² > 0 (melodic acceleration)

  borzy (rất nhanh):
    β = 3   → dt' = dt/3  (nốt chạy 3 lần nhanh hơn)

  Không có dấu chậm tương ứng — chậm = mặc định (β = 1)
```

**Dấu kiểu nét (Contour modifiers) — Hình dạng giai điệu:**
```
  kupnaya (tròn):
    p(t) = p₀ + R·sin(πt/T)     (cung tròn lên rồi về)
    Hình dạng: arc, smooth peak

  lomka (gãy):
    p(t) = { p₀ + v₁t           nếu t < t_break
           { p_break + v₂(t-t_break)  nếu t ≥ t_break
    v₁ ≠ v₂ → gián đoạn đạo hàm tại t_break (angular point)
    d²p/dt² = (v₂-v₁)·δ(t-t_break)  (xung Dirac tại điểm gãy)

  curved (cong):
    p(t) = p₀ + at + bt²    (parabol — cong đều)
    Curvature κ = 2b/(1+(a+2bt)²)^(3/2)
```

**Dấu đặc biệt (Special modifiers):**
```
  kachka (lắc — vibrato):
    p(t) = p_base + A_vib·sin(2πf_vib·t)
    A_vib = biên độ lắc ≈ ±Δ₀/4 (nhỏ hơn 1 bậc)
    f_vib ≈ 5-7 Hz (tần số vibrato tự nhiên)

  kryzh (thập — nhấn mạnh):
    A(t) = A₀ · (1 + k·rect(t/τ))    k > 0
    Tăng biên độ tạm thời trong khoảng τ (accent)

  dvoetochie (hai chấm — lặp):
    p(t) = p_note ∀ t ∈ [t₁, t₁+d] ∪ [t₂, t₂+d]
    Nốt được lặp 2 lần, cách nhau khoảng nghỉ ngắn
```

**Tonal Range Indicator — Chỉ quãng:**
```
  Xác định vùng cao độ hoạt động (tessitura):
    lower range:  p ∈ [p_min, p_mid]           (quãng thấp)
    upper range:  p ∈ [p_mid, p_max]           (quãng cao)

  Transfer function (chuyển vùng):
    T_range: p → p + n·octave    (n ∈ ℤ, chuyển quãng tám)
    lower indicator: n = -1  (xuống 1 octave)
    upper indicator: n = +1  (lên 1 octave)
```

---

## T.4 — NHẠC PHƯƠNG TÂY (Western Musical) · 306 cụm

### Tầng 1: "Loại ký hiệu gì?"

```
NHẠC PHƯƠNG TÂY
├── NỐT          "note / notehead / breve / brevis / whole / half / quarter / eighth / sixteenth"
├── DẤU LẶNG     "rest" — nghỉ
├── KHÓA         "clef" — khóa nhạc (C, F, G)
├── THĂNG/GIÁNG  "sharp / flat / natural" — #♭♮
├── DẤU BIỂU CẢM "forte / piano / crescendo / decrescendo / accent / staccato / tenuto"
├── CẤU TRÚC     "beam / slur / tie / brace / bracket / barline / repeat"
├── NEUME TÂY    "climacus / clivis / podatus / scandicus / torculus"
├── ORNAMENT     "turn / mordent / trill / arpeggiato / glissando"
└── KHÁC         "fermata / caesura / segno / coda / breath mark"
```

### Tầng 2 (Nốt): "Trường độ nào?"

```
TRƯỜNG ĐỘ (từ dài → ngắn)
├── MAXIMA       "maxima" — cực dài (8 phách)
├── LONGA        "longa" — rất dài (4 phách)
├── BREVE        "breve / brevis" — dài (2 phách)
├── TRÒN         "whole note / semibrevis" — 1 phách
├── TRẮNG        "half note / minima" — 1/2 phách
├── ĐEN          "quarter note / semiminima" — 1/4 phách
├── MÓC ĐƠN     "eighth note / fusa" — 1/8 phách
└── MÓC KÉP     "sixteenth note / semifusa" — 1/16 phách
```

### Tầng 2 (Dynamics): "Cường độ nào?"

```
CƯỜNG ĐỘ (từ nhỏ → to)
├── PIANO        "piano" — nhỏ
├── MEZZO PIANO  "mezzo piano" — hơi nhỏ
├── MEZZO FORTE  "mezzo forte" — hơi to
├── FORTE        "forte" — to
├── RINFORZANDO  "rinforzando" — nhấn mạnh đột ngột
├── CRESCENDO    "crescendo" — to dần
└── DECRESCENDO  "decrescendo" — nhỏ dần
```

### Ví dụ cụ thể

```
MUSICAL SYMBOL WHOLE NOTE                 → nốt + tròn (1 phách)
MUSICAL SYMBOL HALF NOTE                  → nốt + trắng (1/2)
MUSICAL SYMBOL QUARTER NOTE               → nốt + đen (1/4)
MUSICAL SYMBOL EIGHTH NOTE               → nốt + móc đơn (1/8)
MUSICAL SYMBOL SIXTEENTH NOTE            → nốt + móc kép (1/16)
MUSICAL SYMBOL QUARTER REST              → lặng + 1/4
MUSICAL SYMBOL WHOLE REST                → lặng + tròn
MUSICAL SYMBOL G CLEF                    → khóa Sol
MUSICAL SYMBOL C CLEF                    → khóa Đô
MUSICAL SYMBOL F CLEF                    → khóa Fa
MUSICAL SYMBOL SHARP                     → thăng #
MUSICAL SYMBOL FLAT                      → giáng ♭
MUSICAL SYMBOL NATURAL                   → bình ♮
MUSICAL SYMBOL FORTE                     → to (f)
MUSICAL SYMBOL PIANO                     → nhỏ (p)
MUSICAL SYMBOL CRESCENDO                 → to dần
MUSICAL SYMBOL FERMATA                   → kéo dài tùy ý
MUSICAL SYMBOL ARPEGGIATO UP             → rải hợp âm lên
MUSICAL SYMBOL BEGIN BEAM                → bắt đầu nối nốt
MUSICAL SYMBOL BEGIN SLUR                → bắt đầu legato
MUSICAL SYMBOL CODA                      → kết thúc
MUSICAL SYMBOL SEGNO                     → dấu hiệu nhảy
→ "ah, nhạc phương Tây [loại] [chi tiết]"
```

### Mô hình toán học — Phân tích Fourier đầy đủ

```
  Mỗi nốt = 1 thành phần Fourier: fₙ(t) = Aₙ · sin(2πfₙt + φₙ) · w(t)

  Duration w(t): hàm bao (envelope function)
    whole note: w(t) = rect(t/4T)  (4 phách)
    half note:  w(t) = rect(t/2T)  (2 phách)
    quarter:    w(t) = rect(t/T)   (1 phách)
    eighth:     w(t) = rect(t/(T/2))  (½ phách)
    sixteenth:  w(t) = rect(t/(T/4))  (¼ phách)

    Chuỗi trường độ: {4, 2, 1, ½, ¼, ⅛, ...}T = cấp số nhân 2⁻ⁿ·4T

  Pitch (khóa nhạc + vị trí nốt):
    f = f₀ · 2^(n/12)  (bình quân luật, n bán cung từ tần số tham chiếu f₀)
    Sharp: n → n+1, Flat: n → n−1, Natural: hủy dấu trước

  Dynamics — biên độ:
    ppp → pp → p → mp → mf → f → ff → fff
    Xấp xỉ: A = A₀ · 10^(L/20) trong đó L ∈ [-40, +20] dB

  Crescendo/Decrescendo: dA/dt > 0 / dA/dt < 0 (biên độ tăng/giảm dần)
  Fermata: w(t) kéo giãn hệ số α > 1 (giãn nở thời gian)

  Ornament — điều biến:
    Trill: f(t) = f₀ + Δf·square(2πf_trill·t)  (luân phiên nhanh)
    Mordent: một xung luân phiên đơn
    Glissando: f(t) = f₁ + (f₂−f₁)·t/T  (quét tần số tuyến tính)
```

### Công thức chi tiết theo loại ký hiệu

**Nốt nhạc — Hàm sóng với envelope ADSR:**
```
  Mỗi nốt = sóng × envelope:
    s(t) = A(t) · sin(2πf·t + φ)

  Envelope ADSR (Attack-Decay-Sustain-Release):
                 ⎧  t/t_A                          0 ≤ t < t_A        (attack)
                 ⎪  1 − (1−S)(t−t_A)/t_D           t_A ≤ t < t_A+t_D  (decay)
    A(t)/A_max = ⎨  S                               t_A+t_D ≤ t < t_R  (sustain)
                 ⎪  S · (1 − (t−t_R)/(t_end−t_R))  t_R ≤ t ≤ t_end    (release)
                 ⎩  0                               t > t_end

  Trường độ xác định t_end:
    maxima:    t_end = 8T    (8 phách, trung cổ)
    longa:     t_end = 4T    (4 phách, trung cổ)
    breve:     t_end = 2T
    whole:     t_end = T      (= 1 ô nhịp trong 4/4)
    half:      t_end = T/2
    quarter:   t_end = T/4
    eighth:    t_end = T/8
    sixteenth: t_end = T/16

  Quy luật: t_end(n) = T · 2^(2−n)    n ∈ {-2, -1, 0, 1, 2, 3, 4, 5}
    n = -2 → maxima, n = 0 → whole, n = 3 → eighth, ...
```

**Dấu lặng (Rest) — Khoảng im (silence interval):**
```
  s(t) = 0    ∀ t ∈ [t_start, t_start + t_rest]

  t_rest tuân cùng chuỗi 2⁻ⁿ:
    whole rest = T, half rest = T/2, quarter rest = T/4, ...

  Ý nghĩa tín hiệu: silence = zero-energy window
    E_rest = ∫ |s(t)|² dt = 0
```

**Khóa nhạc (Clef) — Hàm ánh xạ vị trí → tần số:**
```
  Clef = hàm cơ sở C: line_position → pitch_class

  G clef (khóa Sol):  C_G(line_2) = G4 → f₀ = 392 Hz
    f(pos) = 392 · 2^((pos − 2) / 7)    (pos = vị trí trên khuông)

  F clef (khóa Fa):   C_F(line_4) = F3 → f₀ = 174.6 Hz
    f(pos) = 174.6 · 2^((pos − 4) / 7)

  C clef (khóa Đô):   C_C(line_k) = C4 → f₀ = 261.6 Hz
    f(pos) = 261.6 · 2^((pos − k) / 7)   (k = vị trí của clef)

  Tổng quát: f(pos) = f_ref · 2^((pos − pos_ref) / 7)
    7 vị trí = 1 octave (diatonic, không chromatic)
```

**Thăng / Giáng / Bình — Toán tử chuyển cung (pitch operators):**
```
  Sharp (#):   f → f · 2^(1/12)     (tăng 1 bán cung = +100 cent)
  Flat (♭):    f → f · 2^(-1/12)    (giảm 1 bán cung = -100 cent)
  Natural (♮): f → f_diatonic        (hủy sharp/flat, về bậc tự nhiên)

  Double sharp (𝄪):  f → f · 2^(2/12)   (+200 cent)
  Double flat (𝄫):   f → f · 2^(-2/12)  (-200 cent)

  Đại số: Sharp ∘ Flat = Identity, Sharp⁻¹ = Flat
  Nhóm: ({♯, ♭, ♮, 𝄪, 𝄫}, ∘) ≅ (ℤ, +) mod 12
```

**Dynamics — Hàm biên độ (amplitude function):**
```
  Mức dynamics = biên độ rời rạc:
    ppp:  A = A₀ · 10^(-30/20) ≈ 0.032 A₀   (pianississimo)
    pp:   A = A₀ · 10^(-20/20) ≈ 0.1 A₀      (pianissimo)
    p:    A = A₀ · 10^(-10/20) ≈ 0.316 A₀     (piano)
    mp:   A = A₀ · 10^(-5/20)  ≈ 0.562 A₀     (mezzo piano)
    mf:   A = A₀ · 10^(0/20)   = A₀            (mezzo forte, reference)
    f:    A = A₀ · 10^(5/20)   ≈ 1.778 A₀     (forte)
    ff:   A = A₀ · 10^(10/20)  ≈ 3.162 A₀     (fortissimo)
    fff:  A = A₀ · 10^(15/20)  ≈ 5.623 A₀     (fortississimo)

  Crescendo (to dần):   A(t) = A_start + (A_end − A_start) · t/T_cresc
    dA/dt = (A_end − A_start) / T_cresc > 0

  Decrescendo (nhỏ dần): A(t) = A_start − (A_start − A_end) · t/T_decresc
    dA/dt < 0

  Rinforzando: A(t) = A₀ + A_peak · δ_smooth(t − t_rf)
    Nhấn đột ngột rồi về, δ_smooth = xung Gaussian hẹp
```

**Cấu trúc — Toán tử trên chuỗi nốt:**
```
  Beam:     nhóm nốt ngắn → quantize vào nhịp con
    beam(n₁, n₂, ..., nₖ) = group{nᵢ : Σt(nᵢ) = T_beat}

  Slur/Tie: nối legato → bỏ attack giữa các nốt
    slur(n₁, n₂) = { s₁(t) concat s₂(t) without re-attack }
    tie(n₁, n₂):  f(n₁) = f(n₂) → duration = t(n₁) + t(n₂)

  Barline:  chia dòng nhạc thành ô nhịp
    measure_k = {notes ∈ [kT_measure, (k+1)T_measure]}

  Repeat:   lặp đoạn
    repeat(section, n) = section ∘ section ∘ ... (n lần)
    |: ... :| = repeat(section, 2)
```

**Ornament — Điều biến tần số (frequency modulation):**
```
  Trill:      f(t) = f_main + Δf · sgn(sin(2πf_trill·t))
    Luân phiên giữa f_main và f_main+Δf
    f_trill ≈ 4-8 Hz, Δf = 1 hoặc 2 bán cung

  Mordent:    f(t) = f_main + Δf · rect(t/τ_mordent)
    Một lần lên-về nhanh, τ_mordent << T_note

  Turn:       f(t) = f_main + Δf · sin(2πt/τ_turn)    t ∈ [0, τ_turn]
    Đi lên - về - xuống - về (1 chu kỳ sin)

  Arpeggiato: rải hợp âm theo thời gian
    f(t) = fₖ   ∀ t ∈ [t₀ + k·δ, t₀ + k·δ + d]    k = 0,1,...,n
    δ = khoảng cách rải, fₖ = nốt thứ k trong hợp âm

  Glissando:  quét tần số liên tục
    f(t) = f₁ · (f₂/f₁)^(t/T)    (exponential sweep, giữ đều trên thang log)
```

**Neume phương Tây (cổ nhạc) — Đường viền giai điệu:**
```
  Neume Tây = nhóm nốt liền nhau, mã hóa hướng đi giai điệu:

  climacus:   Δp = [0, -1, -1, ...]   (đi xuống dần)
    p(k) = p₀ − k·step    k = 0, 1, 2, ...

  clivis:     Δp = [0, -1]             (lên rồi xuống)
    p = [p₀, p₀ − step]

  podatus:    Δp = [0, +1]             (xuống rồi lên)
    p = [p₀, p₀ + step]

  scandicus:  Δp = [0, +1, +1, ...]    (đi lên dần)
    p(k) = p₀ + k·step

  torculus:   Δp = [0, +1, -1]         (lên - đỉnh - xuống)
    p = [p₀, p₀ + step, p₀]
```

---

## T.5 — KHÁC (Greek notation, supplement) · 70 cụm

### Phân nhóm

```
KHÁC
├── GREEK INSTRUMENTAL  "greek instrumental notation symbol-N" — 32 ký hiệu nhạc cụ Hy Lạp
├── GREEK VOCAL         "greek vocal notation symbol-N" — 24 ký hiệu thanh nhạc Hy Lạp
├── COMBINING GREEK     "combining greek musical" — 3 dấu nhịp (triseme, tetraseme, pentaseme)
└── SUPPLEMENT          musical symbols supplement — ký hiệu bổ sung
```

### Ví dụ cụ thể

```
GREEK INSTRUMENTAL NOTATION SYMBOL-1     → nhạc cụ Hy Lạp #1
GREEK INSTRUMENTAL NOTATION SYMBOL-17    → nhạc cụ Hy Lạp #17
GREEK VOCAL NOTATION SYMBOL-1            → thanh nhạc Hy Lạp #1
COMBINING GREEK MUSICAL TRISEME          → nhịp 3 (3 mora)
COMBINING GREEK MUSICAL TETRASEME        → nhịp 4 (4 mora)
COMBINING GREEK MUSICAL PENTASEME        → nhịp 5 (5 mora)
→ "ah, nhạc Hy Lạp cổ [instrumental/vocal] [số thứ tự]"
```

### Mô hình toán học

```
  Greek notation: cao độ mã hóa bằng số thứ tự
    Symbol-N → lớp cao độ N trong hệ thống điệu thức Hy Lạp

  Triseme/Tetraseme/Pentaseme = nhóm nhịp:
    mora = đơn vị thời gian nguyên tử μ
    triseme = 3μ, tetraseme = 4μ, pentaseme = 5μ
    Tỷ lệ: tạo thành các tỷ số nhịp hữu tỉ 3:4:5
```

### Công thức chi tiết

**Greek Instrumental Notation — Ánh xạ ký hiệu → cao độ:**
```
  Hệ điệu thức Hy Lạp: quãng dựa trên tetrachord (nhóm 4 nốt)

  Tetrachord = 4 nốt bao trùm quãng 4 đúng (ratio 4:3)
    Diatonic:    f₁, f₁·9/8, f₁·81/64, f₁·4/3    (quãng: 9:8, 9:8, 256:243)
    Chromatic:   f₁, f₁·22/21, f₁·8/7, f₁·4/3     (quãng: 22:21, 12:11, 7:6)
    Enharmonic:  f₁, f₁·28/27, f₁·16/15, f₁·4/3   (quãng: 28:27, 16:15, 5:4)

  Symbol-N → vị trí trong chuỗi tetrachord:
    N mod 4 = vị trí trong tetrachord hiện tại
    N div 4 = tetrachord thứ mấy (từ thấp lên)

  Tần số: f(N) = f_base × ∏ intervals[0..N]
    intervals = chuỗi tỷ số tùy genus (diatonic/chromatic/enharmonic)

  Instrumental vs Vocal: cùng hệ cao độ, khác ký hiệu
    Bijection: φ: Instrumental_N → Vocal_M   (map 1-1 giữa 2 bộ ký hiệu)
```

**Combining Greek Musical — Nhóm nhịp (metrical grouping):**
```
  Mora μ = đơn vị thời gian nguyên tử (indivisible)
    t_mora = T_beat / n    (n phụ thuộc nhịp điệu)

  Triseme:    T = 3μ     (nhịp 3 — triple meter)
  Tetraseme:  T = 4μ     (nhịp 4 — quadruple meter)
  Pentaseme:  T = 5μ     (nhịp 5 — quintuple meter, asymmetric)

  Nhịp phức = tổ hợp:
    7μ = 3μ + 4μ  hoặc 4μ + 3μ   (nhịp 7, Hy Lạp cổ)
    5μ = 2μ + 3μ  hoặc 3μ + 2μ   (nhịp 5)

  Tỷ số nhịp (rhythmic ratio):
    Triseme : Tetraseme = 3:4    (polyrhythm cơ bản)
    Triseme : Pentaseme = 3:5    (golden-like ratio)

  Ý nghĩa: mỗi combining mark = toán tử nhóm thời gian
    apply(triseme, note) → note.duration = 3μ
    Không thay đổi cao độ, chỉ thay đổi trường độ
```

---

## Từ khóa → Nhóm T (cheat sheet)

| Thấy từ này | → Nhóm | → Nhìn biết |
|-------------|--------|------------|
| hexagram for | T.0 | "quẻ dịch — trạng thái biến đổi" |
| tetragram / digram / monogram | T.1 | "tứ/nhị/đơn quái — trạng thái rời rạc" |
| byzantine musical symbol | T.2 | "nhạc Byzantine — neume/agogi/fthora" |
| znamenny | T.3 | "nhạc Slavonic — dấu phụ neume" |
| musical symbol | T.4 | "nhạc phương Tây — nốt/khóa/dynamics" |
| greek...notation | T.5 | "nhạc Hy Lạp cổ — instrumental/vocal" |
| note, rest, clef | T.4 | "nốt/lặng/khóa" |
| agogi, gorgi, argi | T.2 | "tempo Byzantine" |
| forte, piano, crescendo | T.4 | "cường độ" |
| sharp, flat, natural | T.4 | "thăng/giáng/bình" |

---

## Tổng kết T (Time) — Tất cả nhóm

```
T.0  QUẺ DỊCH         64 cụm   2 tầng: nhóm_trạng_thái × quẻ_cụ_thể
T.1  TỨ QUÁI/DIGRAM   92 cụm   2 tầng: loại_quái × trạng_thái
T.2  BYZANTINE        241 cụm   2 tầng: loại_ký_hiệu × chi_tiết
T.3  ZNAMENNY         185 cụm   2 tầng: loại_dấu × cao/thấp/nhanh
T.4  NHẠC TÂY         306 cụm   2 tầng: loại × trường_độ/cường_độ
T.5  KHÁC              70 cụm   2 tầng: hệ × số_thứ_tự
─────────────────────────────────────────
TỔNG                  958 cụm
```

### Bảng tóm tắt công thức

| Nhóm | Mô hình toán/vật lý | Công thức chính | Không gian trạng thái |
|------|---------------------|----------------|----------------------|
| T.0 Quẻ Dịch | Máy trạng thái hữu hạn (FSM) trên siêu khối 6 chiều | v⃗ ∈ {0,1}⁶; chuyển trạng thái = lật bit; khoảng cách Hamming | 2⁶ = 64 trạng thái |
| T.1 Tứ Quái | Mã hóa tam phân (ternary encoding) | v ∈ {0,1,2}⁴; thông tin = log₂(81) ≈ 6.34 bit | 3⁴ = 81 trạng thái |
| T.2 Byzantine | Đường viền giai điệu = hàm bước từng đoạn | m(t) = Σ Δpᵢ·H(t−tᵢ); co giãn thời gian t' = α·t | Liên tục (bước rời rạc) |
| T.3 Znamenny | Toán tử vi phân trên hàm cao độ | vysoko: p+Δ; borzaya: dp/dt↑; kachka: p+A·sin(ωt) | Liên tục (biến đổi) |
| T.4 Nhạc Tây | Phân tích Fourier đầy đủ | fₙ(t) = Aₙ·sin(2πfₙt+φₙ)·w(t); f = f₀·2^(n/12) | Rời rạc (12-TET) × liên tục |
| T.5 Hy Lạp cổ | Quãng điệu thức + tỷ số nhịp hữu tỉ | mora μ; nhóm nhịp 3:4:5; cao độ = số thứ tự | Rời rạc (thứ tự) |

```
  Mô hình thống nhất: Mỗi chiều T mã hóa một tham số của hàm sóng tổng quát

  ψ(x,t) = Σₙ Aₙ · sin(kₙx − ωₙt + φₙ) · wₙ(t)

  T.0/T.1 → φₙ (pha, trạng thái trong chu kỳ)
  T.2     → Δp (đường viền giai điệu, biến thiên cao độ)
  T.3     → dp/dt, d²p/dt² (toán tử vi phân trên cao độ)
  T.4     → Aₙ, fₙ, wₙ(t) (biên độ, tần số, bao thời gian)
  T.5     → μ (đơn vị thời gian nguyên tử, tỷ số nhịp)
```
