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

### Ánh xạ từng loại → tham số sóng cụ thể

```
  Ánh xạ từng loại → tham số sóng ψ(x,t) = A·sin(kx − ωt + φ):

  ┌─────────────┬──────────────────────────────────────────────────────────┐
  │ Loại        │ Tham số sóng ψ(x,t) = A·sin(kx − ωt + φ)              │
  ├─────────────┼──────────────────────────────────────────────────────────┤
  │ AGOGI       │ ω_tempo: hệ số co giãn thời gian t' = t/α             │
  │             │   poli argi α=0.25, argi α=0.5, metria α=1.0          │
  │             │   gorgi α=2.0, poli gorgi α=4.0                       │
  │ NEUME       │ Δp: bước cao độ rời rạc                                │
  │             │   oligon Δp=+1, petasti Δp=+1, apostrofos Δp=−1       │
  │             │   ison Δp=0, climacus Δp=−2, scandicus Δp=+2          │
  │ FTHORA      │ M: ma trận chuyển điệu thức (mode transformation)     │
  │             │   p'(t) = M · p(t),  M ∈ GL(n,ℤ)                     │
  │ DIESIS      │ Δf = +ε (tăng vi cung), ε ≈ 1/4 tone                 │
  │ YFESIS      │ Δf = −ε (giảm vi cung)                                │
  │ ISON        │ p(t) = p₀ = const (drone, pedal point)               │
  │ CHRONON     │ Δt = τ (đơn vị thời gian cơ bản, chronon)            │
  └─────────────┴──────────────────────────────────────────────────────────┘
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

### Ánh xạ từng dấu → toán tử vi phân cụ thể

```
  Ánh xạ từng dấu → toán tử vi phân cụ thể:

  ┌─────────────────────┬──────────────────────────────────────────┐
  │ Dấu                 │ Toán tử trên p(t)                       │
  ├─────────────────────┼──────────────────────────────────────────┤
  │ gorazdo vysoko      │ p(t) + 2Δ    (dịch rất cao)             │
  │ vysoko              │ p(t) + Δ     (dịch cao)                 │
  │ malo povyshe        │ p(t) + Δ/2   (dịch hơi cao)            │
  │ nizko               │ p(t) − Δ     (dịch thấp)               │
  │ gorazdo nizko       │ p(t) − 2Δ    (dịch rất thấp)           │
  │ borzaya/borzy       │ dt → dt/2    (tăng tốc ×2)             │
  │ kupnaya             │ dt → 2·dt    (giảm tốc ×2)             │
  │ lomka               │ d²p/dt² = δ(t−t₀) (gián đoạn đạo hàm) │
  │ kachka              │ p + A·sin(ω_vib·t) (vibrato)            │
  │ kryzh               │ p(t) = p_final (kết thúc, fermata)      │
  │ dvoetochie          │ repeat{p(t₁..t₂)} (nhắc lại đoạn)     │
  │ on left / on right  │ position modifier (trái/phải neume)     │
  └─────────────────────┴──────────────────────────────────────────┘
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

### Ánh xạ từng loại → tham số sóng

```
  Ánh xạ từng loại → tham số sóng:

  ┌─────────────────┬─────────────────────────────────────────────────┐
  │ Loại            │ Tham số trong fₙ(t) = Aₙ·sin(2πfₙt+φₙ)·w(t)  │
  ├─────────────────┼─────────────────────────────────────────────────┤
  │ NỐT (note)      │ w(t): envelope, trường độ = 2⁻ⁿ·4T            │
  │ DẤU LẶNG (rest) │ A = 0, w(t) = rect (silence envelope)         │
  │ KHÓA (clef)     │ f₀: tần số tham chiếu                         │
  │                 │   G clef: f₀ = 392 Hz (G4)                    │
  │                 │   F clef: f₀ = 174 Hz (F3)                    │
  │                 │   C clef: f₀ = 262 Hz (C4)                    │
  │ THĂNG/GIÁNG     │ n → n±1 bán cung: f' = f·2^(±1/12)           │
  │ DYNAMICS        │ A: biên độ theo thang dB                      │
  │                 │   ppp≈−40dB, pp≈−30, p≈−20, mp≈−10           │
  │                 │   mf≈0dB, f≈+5, ff≈+10, fff≈+15              │
  │ CẤU TRÚC        │ Topology: beam=nhóm, slur=legato, tie=kéo dài│
  │ NEUME TÂY       │ Δp multi-step: climacus=[−1,−1], podatus=[+1]│
  │ ORNAMENT        │ Modulation: trill ω_mod, mordent δ(t), gliss  │
  │ KHÁC            │ fermata α>1, segno/coda=goto, breath=silence  │
  └─────────────────┴─────────────────────────────────────────────────┘
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
