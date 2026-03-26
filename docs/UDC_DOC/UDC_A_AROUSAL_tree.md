# A — AROUSAL (Cường độ kích thích) · Cây phân loại + Công thức vật lý

> 5 mức từ vựng + 5 nhóm cụm từ
> Nhìn từ → biết ngay "thèn này kích thích hay yên tĩnh, mức nào"
> Mỗi nhóm = một chế độ năng lượng (energy regime) có công thức vật lý riêng

---

## Mô hình vật lý tổng quát

Arousal = **trạng thái năng lượng** của một hệ dao động tắt dần (damped harmonic oscillator):

```
  ẍ + 2γẋ + ω₀²x = F(t)/m

  Trong đó:
    x     = biên độ phản ứng (response amplitude)
    γ     = hệ số tắt dần (damping coefficient)
    ω₀    = tần số riêng (natural frequency)
    F(t)  = lực kích thích bên ngoài (external stimulus)

  Arousal A(w) = trạng thái năng lượng tổng:

              E_kinetic + E_potential       ½mẋ² + ½mω₀²x²
  A(w) = tanh(─────────────────────── ) = tanh(─────────────────── )
                    E_threshold                    E_th

  A ∈ [-1, +1]:
    A → +1  khi E >> E_th   (năng lượng vượt ngưỡng → hệ bùng nổ)
    A →  0  khi E ≈ E_th    (cân bằng nhiệt động)
    A → -1  khi E << E_th   (hệ về trạng thái cơ bản)
```

---

## Tầng 1: "Kích thích hay yên tĩnh?" — 5 chế độ năng lượng

```
AROUSAL
├── CỰC KÍCH THÍCH   A > +0.5   (2862 từ, 52 cụm)  │ E >> E_th (supercritical)
├── KÍCH THÍCH CAO     +0.2 < A ≤ +0.5  (1029 từ)   │ E > E_th  (excited state)
├── TRUNG TÍNH        -0.2 ≤ A ≤ +0.2  (98 từ)      │ E ≈ E_th  (thermal equilibrium)
├── YÊN TĨNH          -0.5 < A < -0.2  (1880 từ, 20 cụm) │ E < E_th (damped)
└── CỰC YÊN TĨNH     A ≤ -0.5  (2603 từ)            │ E → E₀   (ground state)
```

### Hàm phân vùng (partition function):

```
           ⎧  tanh(3(a − 0.5))       nếu a > +0.5     → bão hòa dương (saturation)
           ⎪  (a − 0.2) / 0.3        nếu +0.2 < a ≤ +0.5  → tuyến tính dương
  Â(a) =   ⎨  a / 0.2                nếu -0.2 ≤ a ≤ +0.2  → vùng tuyến tính trung tâm
           ⎪  (a + 0.2) / 0.3        nếu -0.5 < a < -0.2  → tuyến tính âm
           ⎩  tanh(3(a + 0.5))       nếu a ≤ -0.5     → bão hòa âm
```

---

## Tầng 2: "Thuộc loại năng lượng gì?"

### CỰC KÍCH THÍCH — Hệ siêu tới hạn (supercritical regime)

> **Mô hình:** Hệ vượt ngưỡng tới hạn — năng lượng tăng phi tuyến, phản hồi dương (positive feedback).
> Giống phản ứng dây chuyền: một kích thích nhỏ → khuếch đại theo cấp số nhân.

**Hành động mạnh / Vận động — Động năng thuần (pure kinetic energy):**

```
  Công thức:  E_k = ½mv²     (v → v_max)

  Ý nghĩa vật lý:
    Hệ chuyển hóa toàn bộ thế năng thành động năng.
    v = tốc độ hành động,  m = khối lượng/trọng lượng của hành vi.
    E_k → ∞  khi v không bị giới hạn (burst, sprint, explode).

  Đặc trưng: gia tốc a = dv/dt > 0 liên tục (acceleration phase)
```
```
accelerate, attack, battle, blast, burst, charge
chase, clash, combat, crash, dash, erupt
explode, fight, fire, force, hit, jump
kick, launch, race, rush, slam, smash
sprint, strike, surge, tackle, thrust, zoom
→ "ah, hành động mạnh — cơ thể hoạt động cường độ cao"
```

**Cảm xúc cực điểm (cả vui lẫn buồn) — Cộng hưởng tại tần số riêng (resonance):**

```
  Công thức:  |X(ω)| = F₀ / √((ω₀² − ω²)² + 4γ²ω²)

  Tại cộng hưởng ω = ω₀:
    |X_max| = F₀ / (2γω₀)  →  khi γ → 0 (tắt dần nhỏ), biên độ → ∞

  Ý nghĩa:
    Cảm xúc cực = kích thích đúng tần số riêng của hệ thần kinh.
    Cả vui (ecstatic) và buồn (terrified) đều cộng hưởng — khác pha, cùng biên độ.
    ecstatic:  pha 0° (cùng chiều kích thích)
    terrified: pha 180° (ngược chiều, nhưng cùng |X|)
```
```
ecstatic, euphoric, frenzied, hysterical, manic
enraged, furious, livid, outraged, infuriated
panicked, terrified, horrified, petrified, frantic
screaming, shouting, yelling, roaring, shrieking
→ "ah, cảm xúc cực — tim đập mạnh, thở gấp"
```

**Khẩn cấp / Nguy hiểm — Hàm bước Heaviside × phản ứng dây chuyền:**

```
  Công thức:  R(t) = H(t − t₀) · R₀ · e^(λt)

  Trong đó:
    H(t − t₀) = hàm bước Heaviside: = 0 khi t < t₀, = 1 khi t ≥ t₀
    t₀ = thời điểm phát hiện nguy hiểm (trigger)
    R₀ = phản ứng ban đầu (fight-or-flight baseline)
    λ  = tốc độ khuếch đại (cascade rate), λ > 0

  Ý nghĩa:
    Trước t₀: hệ im lặng (R = 0).
    Tại t₀: kích hoạt đột ngột (step function).
    Sau t₀: phản ứng tăng theo hàm mũ — adrenaline cascade.
    Nếu λ lớn → panic;  nếu λ vừa → alert.
```
```
alarm, alert, danger, emergency, crisis, urgent
explosive, volatile, threatening, aggressive, violent
assassination, bombing, catastrophe, earthquake, tsunami
→ "ah, nguy hiểm — phản ứng chiến-hoặc-chạy (fight-or-flight)"
```

**Cạnh tranh / Thi đấu — Điểm yên ngựa trong lý thuyết trò chơi (saddle point):**

```
  Công thức:  U(x, y) = x² − y²    (hyperbolic paraboloid)

  Tại điểm yên ngựa (0, 0):
    ∂U/∂x = 2x = 0,  ∂U/∂y = −2y = 0
    nhưng det(H) = |2  0 | = −4 < 0  → KHÔNG ổn định
                    |0 −2|

  Ý nghĩa:
    Cạnh tranh = trạng thái bất ổn vốn có (inherently unstable).
    Một nhiễu nhỏ → hệ lao đi theo hướng x (thắng) hoặc y (thua).
    Năng lượng cao KHÔNG phải vì hệ mạnh, mà vì hệ KHÔNG THỂ yên.
    Adrenaline = hệ cố gắng duy trì tại điểm bất ổn.
```
```
compete, contest, duel, rival, opponent, challenge
champion, conquer, dominate, overpower, overthrow
gambling, betting, racing, wrestling, boxing
→ "ah, cạnh tranh — adrenaline bơm"
```

### KÍCH THÍCH CAO — Trạng thái kích thích (excited state)

> **Mô hình:** Hệ ở trạng thái kích thích — năng lượng trên mức cân bằng nhưng chưa vượt ngưỡng.
> Giống electron ở orbital cao hơn ground state: có thể phát photon (hành động) bất kỳ lúc nào.

```
  Công thức:  E_n = E₀ + n·ΔE     (n = 1, 2, 3... mức kích thích)

  Với dao động tắt dần:
    x(t) = A₀ · e^(−γt) · cos(ω_d · t)    nhưng γ nhỏ (tắt chậm)
    ω_d = √(ω₀² − γ²) ≈ ω₀                (gần tần số riêng)

  Ý nghĩa:
    Hệ dao động rõ ràng, có nhịp, có năng lượng — nhưng KHÔNG phát tán.
    Biên độ giảm chậm (γ nhỏ) → hoạt động bền, không kiệt sức ngay.
    Khác với cực kích thích: ở đây hệ CÓ THỂ dừng một cách nhẹ nhàng.
```
```
active, adventure, athletic, bold, brave, climb
eager, energetic, enthusiastic, excited, explore
intense, lively, passionate, spirited, vibrant
working, performing, presenting, debating, arguing
→ "ah, kích thích cao — hoạt động nhưng chưa cực điểm"
```

### TRUNG TÍNH — Cân bằng nhiệt động (thermal equilibrium)

> **Mô hình:** Hệ ở cân bằng nhiệt động — năng lượng vào = năng lượng ra.
> Giống khí lý tưởng ở nhiệt độ phòng: các phân tử chuyển động ngẫu nhiên, không có xu hướng.

```
  Công thức:  ΔG = ΔH − TΔS = 0    (Gibbs free energy = 0 tại cân bằng)

  Phân bố Boltzmann:
    P(E) = (1/Z) · e^(−E/kT)

  Trong đó:
    Z = Σ e^(−Eᵢ/kT)   (hàm phân hoạch — partition function)
    kT = năng lượng nhiệt (thermal energy)

  Ý nghĩa:
    Mọi trạng thái đều có xác suất tương đương → không thiên lệch.
    Không có lực đẩy (driving force) nào nổi trội.
    Từ ngữ ở mức này = MÔ TẢ sự thật, không mang năng lượng cảm xúc.
    entropy S đạt cực đại cục bộ → hệ "vô hướng" (directionless).
```
```
airline, attorney, based, blend, busy, carrier
collective, consumption, cycle, density, dose, duty
economy, evidence, frequency, government, groups
having, load, material, merely, mixed, normal
→ "ah, trung tính — không kích thích không yên tĩnh"
```

### YÊN TĨNH — Dao động tắt dần (overdamped regime)

> **Mô hình:** Hệ dao động tắt dần mạnh — năng lượng đang tiêu tán, hệ về cân bằng từ từ.
> Giống con lắc trong mật ong: di chuyển chậm, mượt, không dao động.

```
  Công thức:  x(t) = (C₁ + C₂t) · e^(−γt)     (overdamped: γ > ω₀)

  Hoặc dạng tổng quát:
    x(t) = C₁·e^(r₁t) + C₂·e^(r₂t)
    r₁,₂ = −γ ± √(γ² − ω₀²)    (cả hai nghiệm thực âm)

  Ý nghĩa:
    Hệ KHÔNG dao động — trở về cân bằng đơn điệu (monotonic decay).
    γ > ω₀ → lực cản thắng lực đàn hồi → mọi chuyển động đều chậm dần.
    Năng lượng tiêu tán:  dE/dt = −2γ·E_k < 0 (luôn mất năng lượng)
    Đặc trưng: mượt (smooth), dần dần (gradual), không giật (no oscillation).
```
```
adjust, calm, comfortable, contemplate, drift, ease
gentle, gradual, linger, mild, moderate, patient
peaceful, quiet, relax, rest, settle, silent
slow, smooth, soft, steady, still, subtle
tender, tranquil, unhurried, wait, walk, wander
→ "ah, yên tĩnh — nhịp chậm, thư giãn"
```

### CỰC YÊN TĨNH — Trạng thái cơ bản (ground state)

> **Mô hình:** Hệ ở mức năng lượng thấp nhất có thể — không đủ năng lượng để chuyển trạng thái.
> Giống trạng thái cơ bản lượng tử: vẫn có zero-point energy (E₀ > 0) nhưng không thể thấp hơn.

**Tĩnh lặng / Bất động — Năng lượng điểm không (zero-point energy):**

```
  Công thức:  E₀ = ½ℏω₀     (năng lượng tối thiểu, không thể = 0)

  Hệ quả:
    ΔE = E_n − E₀ > 0  cho mọi chuyển trạng thái
    Nhưng hệ KHÔNG CÓ ΔE để chi → bị "đóng băng" tại E₀.
    v → 0,  a = 0,  dx/dt ≈ 0

  Ý nghĩa:
    Bất động KHÔNG phải = chết (E₀ > 0 luôn).
    Mà là không đủ năng lượng để HÀNH ĐỘNG (ΔE = 0).
    frozen, paralyzed, numb = hệ bị mắc kẹt tại ground state.
```
```
asleep, dormant, frozen, immobile, inert, lifeless
motionless, numb, paralyzed, passive, stagnant
stationary, still, unconscious, unmoving, vacant
→ "ah, bất động — gần như không hoạt động"
```

**Buồn chán / Mệt mỏi — Cực đại entropy (heat death):**

```
  Công thức:  S → S_max     (entropy cực đại)
              dS/dt = 0      (không còn gradient để tạo công)
              η = W/Q → 0    (hiệu suất Carnot → 0 khi T_hot → T_cold)

  Ý nghĩa:
    Buồn chán = trạng thái entropy cực đại — mọi thứ đồng nhất, không có gradient.
    Không có chênh lệch → không có lực đẩy → không có hành động.
    "heat death" vi mô: năng lượng có nhưng KHÔNG THỂ chuyển hóa thành công.
    exhausted: Q đã chuyển hết thành S (nhiệt → entropy, không còn W).
```
```
bored, boring, dull, monotonous, tedious, tiresome
drowsy, exhausted, fatigued, lethargic, sleepy
apathetic, indifferent, listless, unmotivated, weary
→ "ah, mệt mỏi — năng lượng cạn kiệt"
```

**Trừu tượng / Tri thức tĩnh — Thông tin thuần (pure information, no work):**

```
  Công thức:  H = −Σ pᵢ · log₂(pᵢ)    (Shannon entropy — đo lượng thông tin)

  Nhưng:  W = 0    (không có công cơ học)
          F = 0    (không có lực)
          v = 0    (không có chuyển động)

  Ý nghĩa:
    Thông tin H có thể rất cao (tri thức phong phú).
    Nhưng năng lượng vật lý = 0 → không hành động.
    Suy nghĩ ≠ hành động:  H >> 0  nhưng  E_k = 0.
    Arousal thấp vì ĐO NĂNG LƯỢNG, không đo thông tin.
```
```
abstract, academic, algebraic, analytical, archival
categorical, conceptual, philosophical, theoretical
administrative, bureaucratic, procedural, regulatory
→ "ah, trí tuệ tĩnh — suy nghĩ nhưng không hành động"
```

---

## Tầng 2 cho CỤM TỪ: "Chủ đề gì?"

### Năng lượng cao (52 cụm)

```
action figure         → hành động + đồ chơi
action film           → hành động + phim
adventure game        → phiêu lưu + trò chơi
air force             → không quân
armed forces          → lực lượng vũ trang
battle royal          → trận chiến
catch fire            → bắt lửa
drag race             → đua xe
driving force         → lực đẩy
fast forward          → tua nhanh
fight back            → đánh trả
fire alarm            → báo cháy
full speed            → tốc độ tối đa
high speed            → tốc độ cao
→ "ah, cụm năng lượng cao — hành động/chiến đấu/tốc độ"
```

### Năng lượng thấp (20 cụm)

```
arm chair             → ghế bành
at ease               → thoải mái
at rest               → nghỉ ngơi
calm down             → bình tĩnh lại
peace out             → yên bình
quiet down            → im lặng
rest day              → ngày nghỉ
slow cooker           → nồi nấu chậm
slow pace             → nhịp chậm
soft tissue           → mô mềm
→ "ah, cụm năng lượng thấp — nghỉ/chậm/êm"
```

---

## So sánh V vs A: Hai trục KHÁC NHAU

```
                     A cao (kích thích)
                          │
     giận dữ ←────────────┼────────────→ hào hứng
     (V âm, A cao)        │              (V dương, A cao)
                          │
  V âm ←─────────────────┼─────────────────→ V dương
     (tiêu cực)          │                  (tích cực)
                          │
     buồn bã ←────────────┼────────────→ thư giãn
     (V âm, A thấp)       │              (V dương, A thấp)
                          │
                     A thấp (yên tĩnh)

Ví dụ:
  "furious"   = V rất âm  + A rất cao   → giận + kích thích
  "terrified" = V rất âm  + A rất cao   → sợ + kích thích
  "depressed" = V rất âm  + A rất thấp  → buồn + bất động
  "bored"     = V hơi âm  + A rất thấp  → chán + mệt
  "ecstatic"  = V rất dương + A rất cao → vui + phấn khích
  "serene"    = V dương    + A rất thấp  → bình yên + tĩnh lặng
  "content"   = V dương    + A thấp      → hài lòng + nhẹ nhàng
```

---

## Từ khóa → Mức Arousal (cheat sheet)

| Thấy từ/cụm này | → Mức A | → Nhìn biết |
|-----------------|---------|------------|
| explode, attack, scream, panic, race | Cực kích thích (+0.5~+1.0) | "nổ/đánh/la/hoảng/chạy" |
| active, brave, excited, eager, debate | Kích thích cao (+0.2~+0.5) | "sống động, hăng hái" |
| airline, duty, economy, process | Trung tính (-0.2~+0.2) | "bình thường, khách quan" |
| calm, gentle, quiet, slow, patient | Yên tĩnh (-0.5~-0.2) | "nhẹ nhàng, chậm rãi" |
| asleep, bored, numb, frozen, inert | Cực yên tĩnh (-1.0~-0.5) | "ngủ/chán/tê/đông/trơ" |

---

## Tóm tắt — Arousal như phổ năng lượng

```
A = 1 trục liên tục [-1.0, +1.0]  (giống V, nhưng đo CƯỜNG ĐỘ không phải THIỆN/ÁC)

  -1.0        -0.5        -0.2    0    +0.2        +0.5        +1.0
   |──────────|──────────|────|────|──────────|──────────|
   ground      overdamped  equi-    excited     super-
   state       (tắt dần)   librium  state       critical
   E = E₀     γ > ω₀      ΔG = 0   γ < ω₀     E >> E_th

8,472 từ + 77 cụm = mỗi cái 1 vị trí trên phổ năng lượng

Bản chất vật lý mỗi nhóm:
  ┌─────────────────┬──────────────────────────────────────────┐
  │ Nhóm            │ Công thức đặc trưng                      │
  ├─────────────────┼──────────────────────────────────────────┤
  │ Hành động mạnh  │ E_k = ½mv²  (động năng thuần)           │
  │ Cảm xúc cực     │ |X| = F₀/(2γω₀)  (cộng hưởng)          │
  │ Khẩn cấp        │ R(t) = H(t−t₀)·R₀·e^(λt)  (cascade)   │
  │ Cạnh tranh      │ U = x²−y²  (điểm yên ngựa bất ổn)      │
  │ Kích thích cao  │ x(t) = A₀·e^(−γt)·cos(ωt), γ nhỏ       │
  │ Trung tính      │ ΔG = 0, P(E) = e^(−E/kT)/Z              │
  │ Yên tĩnh        │ x(t) = (C₁+C₂t)·e^(−γt), γ > ω₀       │
  │ Bất động         │ E₀ = ½ℏω₀  (zero-point)                │
  │ Mệt mỏi         │ S → S_max, η → 0  (heat death)         │
  │ Trí tuệ tĩnh    │ H = −Σp·log₂(p), nhưng W = 0           │
  └─────────────────┴──────────────────────────────────────────┘
```
