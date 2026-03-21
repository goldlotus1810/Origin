# V + A — Valence + Arousal · Công thức toán học thật

> V và A chia sẻ 17 blocks. Mỗi ký tự có CẢ V lẫn A.
> Công thức thật: không gian cảm xúc 2D, phương trình Russell, biến đổi affective.

---

## Tổng quan: V×A là gì?

```
Emotion: Word → ℝ² = (v, a) ∈ [-1, +1]²

v = valence  (trục ngang): tiêu cực ← 0 → tích cực
a = arousal  (trục dọc):  yên tĩnh ← 0 → kích thích

Đây là mô hình Circumplex (Russell, 1980):
  Mọi cảm xúc = 1 điểm trong không gian 2D liên tục.
```

---

## V — Valence

### Công thức gốc: Tra bảng NRC-VAD

```
v_raw(w) = NRC_VAD_Lexicon[w].valence ∈ [0, 1]

Chuẩn hóa về [-1, +1]:
  v(w) = 2 · v_raw(w) − 1

Quantize 3 bits:
  V(w) = ⌊ v_raw(w) × 7 + 0.5 ⌋ ∈ {0, 1, ..., 7}
```

### Công thức cụm từ

```
V(phrase) = Σᵢ wᵢ · v(wordᵢ) / Σᵢ wᵢ

  KHÔNG trung bình đơn giản — dùng weighted sum:
  wᵢ = trọng số theo vị trí và loại từ

  Negation (phủ định):
    v("not happy") = −v("happy") · α,  α ∈ [0.5, 0.8]

  Intensifier (tăng cường):
    v("very happy") = v("happy") · β,  β > 1
    v("slightly sad") = v("sad") · γ,  γ < 1
```

### 5 mức Valence — Ánh xạ thật

```
V = 0  ↔  v ∈ [-1.0, -0.71):   cực tiêu cực
  Hàm mật độ: P(w | V=0) tập trung tại {death, torture, horror}
  Biến đổi sinh lý: cortisol ↑↑, serotonin ↓↓

V = 1  ↔  v ∈ [-0.71, -0.43):  rất tiêu cực
V = 2  ↔  v ∈ [-0.43, -0.14):  tiêu cực
V = 3  ↔  v ∈ [-0.14, +0.14):  trung tính (phân bố đều)
V = 4  ↔  v ∈ [+0.14, +0.43):  tích cực
V = 5  ↔  v ∈ [+0.43, +0.71):  rất tích cực
V = 6  ↔  v ∈ [+0.71, +0.86):  dương

V = 7  ↔  v ∈ [+0.86, +1.0]:   cực tích cực
  Hàm mật độ: P(w | V=7) tập trung tại {love, joy, ecstatic}
  Biến đổi sinh lý: dopamine ↑↑, oxytocin ↑↑
```

---

## A — Arousal

### Công thức gốc: Tra bảng NRC-VAD

```
a_raw(w) = NRC_VAD_Lexicon[w].arousal ∈ [0, 1]

Chuẩn hóa:
  a(w) = 2 · a_raw(w) − 1 ∈ [-1, +1]

Quantize 3 bits:
  A(w) = ⌊ a_raw(w) × 7 + 0.5 ⌋ ∈ {0, 1, ..., 7}
```

### 5 mức Arousal — Ánh xạ sinh lý thật

```
A = 0  ↔  a ∈ [-1.0, -0.71):   cực yên tĩnh
  Sinh lý: nhịp tim ~60 bpm, thở chậm, EEG alpha/theta
  Hành vi: ngủ, thiền, bất động

A = 3  ↔  a ∈ [-0.14, +0.14):  trung tính
  Sinh lý: nhịp tim ~72 bpm, baseline
  Hành vi: đọc, đi bộ, trò chuyện bình thường

A = 7  ↔  a ∈ [+0.86, +1.0]:   cực kích thích
  Sinh lý: nhịp tim >120 bpm, adrenaline ↑↑, EEG beta/gamma
  Hành vi: chiến đấu, hoảng loạn, phấn khích cực độ
```

---

## Không gian V×A — Mô hình Circumplex

### Tọa độ Descartes

```
E(w) = (v(w), a(w)) ∈ [-1, +1]²

4 góc phần tư:
  Q₁: (V+, A+) = hào hứng, phấn khích     {ecstatic, thrilled, excited}
  Q₂: (V−, A+) = giận dữ, hoảng sợ        {furious, terrified, panicked}
  Q₃: (V−, A−) = buồn bã, chán nản        {depressed, bored, hopeless}
  Q₄: (V+, A−) = bình yên, hài lòng       {serene, content, relaxed}
```

### Tọa độ cực (Polar)

```
E_polar(w) = (r, θ)

r = √(v² + a²)     ∈ [0, √2]    cường độ cảm xúc (emotional intensity)
θ = atan2(a, v)     ∈ [0, 2π)    loại cảm xúc (emotional category)

  θ ≈ 0°:    hài lòng (V+, A=0)
  θ ≈ 45°:   hào hứng (V+, A+)
  θ ≈ 90°:   kích thích (V=0, A+)
  θ ≈ 135°:  giận dữ (V−, A+)
  θ ≈ 180°:  bất mãn (V−, A=0)
  θ ≈ 225°:  buồn rầu (V−, A−)
  θ ≈ 270°:  mệt mỏi (V=0, A−)
  θ ≈ 315°:  thư giãn (V+, A−)
```

### Khoảng cách cảm xúc

```
d(w₁, w₂) = √( (v₁−v₂)² + (a₁−a₂)² )

  d("happy", "ecstatic") ≈ 0.3      gần nhau (cùng Q₁)
  d("happy", "furious")  ≈ 1.8      rất xa (Q₁ vs Q₂)
  d("bored", "excited")  ≈ 1.9      cực xa (Q₃ vs Q₁)
```

---

## Emotion Compose — KHÔNG trung bình

### Quy tắc: Amplify qua Silk walk

```
KHÔNG:  E(w₁, w₂) = (E(w₁) + E(w₂)) / 2              ← SAI (trung bình)

ĐÚNG:   E_composed = amplify( walk(w₁, w₂) )

walk: đi trên Silk graph từ w₁ đến w₂
  Mỗi cạnh có emotion tag → tích lũy, không trung bình

amplify(path):
  Cùng chiều (cả 2 tiêu cực hoặc cả 2 tích cực):
    |v_result| ≥ max(|v₁|, |v₂|)                       khuếch đại
  Ngược chiều:
    v_result ≈ v_dominant                                cái mạnh hơn thắng

  cortisol + adrenaline = mạnh hơn (không trung bình!)
  dopamine + oxytocin = mạnh hơn
```

### Phương trình biến đổi theo thời gian

```
dE/dt = −λE + Σᵢ Iᵢ(t) · kᵢ

E(t) = emotion state tại thời điểm t
λ = decay rate (cảm xúc suy giảm tự nhiên)
Iᵢ(t) = input thứ i tại thời điểm t (từ mới, sự kiện)
kᵢ = hệ số khuếch đại input i

Nghiệm:
  E(t) = E₀ · e^(−λt) + Σᵢ ∫₀ᵗ Iᵢ(τ) · kᵢ · e^(−λ(t−τ)) dτ

  → Cảm xúc = tích phân chập (convolution) của input với exponential decay
```

---

## Emoji → V×A

### Ánh xạ trực tiếp

```
Emoji có V và A xác định từ emoji-test.txt subgroup:

  face-smiling     →  V ∈ [+0.5, +1.0],  A ∈ [+0.2, +0.8]
  face-concerned   →  V ∈ [-0.8, -0.2],  A ∈ [+0.3, +0.9]
  face-tongue      →  V ∈ [+0.3, +0.7],  A ∈ [+0.4, +0.7]
  cat-face         →  V ∈ [-0.5, +0.8],  A ∈ [+0.2, +0.6]    (phạm vi rộng)

Ví dụ cụ thể:
  😂 = (v=+0.8, a=+0.7)     vui + kích thích cao
  😴 = (v=+0.1, a=-0.9)     trung tính + cực yên
  💀 = (v=-0.8, a=-0.3)     rất tiêu cực + hơi yên
  🔥 = (v=+0.2, a=+0.9)     hơi tích cực + cực kích thích
  😱 = (v=-0.6, a=+0.95)    tiêu cực + cực kích thích
```

---

## Tích phân tổng: o{V, A}

```
o{V×A} = ∫∫ dv da = không gian cảm xúc 2D liên tục

Discretized: Z₈ × Z₈ = 64 ô lưới
Mỗi ô = 1 vùng cảm xúc:
  (V=7, A=7) = cực vui + cực kích thích = ecstasy
  (V=0, A=0) = cực buồn + cực yên = despair
  (V=3, A=3) = trung tính = baseline
```
