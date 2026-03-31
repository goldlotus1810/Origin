# SPEC D — Processing Pipeline: 14 Cơ Chế + 5 Checkpoints

> **Prerequisite:** Đọc SPEC_A, SPEC_B, SPEC_C trước.
> **D không phải phần mới — D là cách SẮP XẾP A+B+C thành pipeline.**
> **Tác giả:** Lupin (thiết kế) + Nox (tổng hợp + verify)
> **Ngày:** 2026-03-30
> **Verified against:** BLUEPRINT §8-§13, origin.md, notes NAC.mb, UDC docs, Lupin (trực tiếp)

---

## MỤC LỤC

```
D1. Input Processing — Holistic first, decompose after
D2. 7 Instincts — 7 formulas trên 5D, không if/else
D3. 14 DNA Mechanisms — Pipeline flow
D4. Homeostasis — Free Energy, Learning vs Acting
D5. Self-Correct — DNA Repair
D6. 5 Checkpoints — Cell Cycle
D7. ConversationCurve — f(x), f'(x), f''(x)
D8. SecurityGate — 3 Layers
```

---

## D1. Input Processing — Holistic First

### Nguyên lý

```
Input KHÔNG bắt đầu bằng tokenization.
Input bắt đầu bằng CAPTURE — 1 node cho TOÀN BỘ input tại thời điểm t.

Moment t: nhận "hãy học Hà Nội là thủ đô Việt Nam"

① CAPTURE — 1 node P_w("toàn bộ input")
   Gồm: text + context + cảm xúc + âm thanh + ánh sáng...
   = 1 node MỚI, chưa từng tồn tại
   Ghi lại NGUYÊN VẸN moment đó.

② ROUTE — prefix/context quyết định đi đâu
   "hãy học" + content      → ĐN (learning, tự do thay đổi)
   "hãy ghi nhớ" + content  → QR (permanent, cần AAM approve)
   content không prefix      → learning buffer → auto-classify sau

   Prefix = silk type: "hãy học" là COMMAND silk route đến ĐN.
   Không cần if/else — prefix P_w quyết định route qua silk.

③ DECOMPOSE — bóc tách thành nodes nhỏ
   Câu → từ → ký tự
   Mỗi cấp = 1 node với P_w riêng.

④ DEDUP — check KnowTree
   P_w đã có → ghi pointer (2 bytes)
   P_w chưa có → tạo node mới, ghi chain + register
```

### Tại sao holistic trước?

```
P_w("sách lịch sử chiến tranh") ≠ P_w("tiểu thuyết chiến tranh")

Cùng "chiến tranh" nhưng:
  Lịch sử: facts, dates, analysis → R dominant, V neutral
  Tiểu thuyết: emotions, characters → V dominant, A cao

P_w = ∫(tất cả con) — cách kể khác → compose khác → P_w khác.
Capture holistic TRƯỚC → giữ bản chất. Decompose SAU → chi tiết.
```

---

## D2. 7 Instincts — 7 Formulas Trên 5D

### Không phải if/else. Là phép toán.

```
① Honesty — Đo confidence từ evidence

   confidence = 0.3 × min(silk_weight, 1.0)
              + 0.3 × min(fire_count / 10.0, 1.0)
              + 0.2 × min(source_count / 3.0, 1.0)
              + 0.2 × consistency

   < 0.40  → IM LẶNG ("Tôi không biết")
   0.40-0.70 → "Tôi nghĩ..." (Hypothesis)
   0.70-0.90 → "Có lẽ..." (Opinion)
   ≥ 0.90 → "Đúng." (Fact)

   Chạy TRƯỚC mọi response. Không đủ evidence → không nói.

② Contradiction — So sánh V distance

   d_V = |A.V − B.V| / 7.0
   d_R = |A.R − B.R| / 15.0
   contradict = d_V > 0.8 AND d_R < 0.2
   (Valence trái ngược + cùng chủ đề = mâu thuẫn)

③ Causality — Temporal + co-activation + R type

   is_causal = (A.timestamp < B.timestamp)
           AND (silk_weight(A, B) > φ⁻¹)
           AND (A.R ∈ CAUSES_RELATION)
   Cần ≥ 2/3 evidence. Co-activation ≠ nhân quả.

④ Abstraction — Variance trong 5D

   variance = Σ distance(member, center)² / |cluster|
   < 0.3 → concrete | < 0.7 → categorical | ≥ 0.7 → abstract

⑤ Analogy — Vector arithmetic trong 5D

   A:B :: C:? → D = C + (B − A) trong 5D
   "king" : "queen" :: "man" : ? → "woman" (delta trên V)

⑥ Curiosity — Novelty = khoảng cách từ known

   novelty = 1.0 − min(nearest_distance / 2.236, 1.0)
   > 0.5 → explore (hỏi thêm) | < 0.3 → familiar

⑦ Reflection — Tự đánh giá

   quality = 0.6 × (QR_count / total) + 0.4 × (avg_silk × silk_count / node_count)
   Cao → học tốt | Thấp → cần học thêm
```

---

## D3. 14 DNA Mechanisms — Pipeline

```
Input (text / audio / image / sensor)
  │
  ↓ ⑨ SecurityGate (Innate Immunity)     → Crisis? → CHẶN ngay
  ──── CHECKPOINT 1: GATE ────
  ↓ ⑩ Fusion (Multisensory)              → capture holistic → 1 node
  ↓ ③  Encode ∫                           → codepoints → P_weights
  ↓ ⑬ Search (Neural Pathways)           → KnowTree walk + Silk follow
  ↓ ⑫ Homeostasis F(t)                   → đo surprise → Learn or Act
  ↓ ⑤  Compose                            → tổ hợp → điểm 5D mới
  ──── CHECKPOINT 2: ENCODE ────
  ↓ ⑧  Instincts (7 formulas)            → Honesty đầu tiên
  ↓ ⑪ Immune Selection                   → 3 nhánh → chọn entropy thấp
  ↓ ⑭ DNA Repair                         → sửa đến quality ≥ φ⁻¹
  ──── CHECKPOINT 3: INFER ────
  ↓ ⑥  Hebbian (Silk co-activate)        → fire together → wire together
  ↓ ⑦  Dream → advance() → QR           → promote nếu chín
  ──── CHECKPOINT 4: PROMOTE ────
  ↓ ②  Decode ∂ (Transcribe 5D → text)   → chiếu ngược ra ngôn ngữ
  ──── CHECKPOINT 5: RESPONSE ────
  ↓
Output (text + tone)
```

### Map vào A+B+C:

```
⑨ SecurityGate    = check prohibited space (V: U₀>>E)         [C3]
⑩ Fusion          = capture holistic → 1 node                 [D1]
③ Encode ∫         = 42 formulas, scale-invariant              [A3]
⑬ Search          = KnowTree walk + Silk type follow           [B2+B3]
⑫ Homeostasis     = Free Energy F(t)                           [D4]
⑤ Compose          = union/amplify/max/dominant                [A4]
⑧ Instincts       = 7 formulas trên 5D                        [D2]
⑪ Immune Selection = 3 branches, pick lowest entropy           [D5]
⑭ DNA Repair       = self-correct, bounded 3 iterations        [D5]
⑥ Hebbian          = Silk co-activate per dimension            [C3]
⑦ Dream            = cross-group resonance → hypothesis → validate [C4]
② Decode ∂          = lookup chain → KnowTree → content        [A5]
```

---

## D4. Homeostasis — Free Energy

```
F(t) = √( Σ w_d × (predicted_d − actual_d)² )

  predicted = KnowTree lookup (tri thức đã có)
  actual    = encode(input) (tri thức mới nhận)
  w_d       = chính P_w của input — V dominant → w_V tự cao
              KHÔNG CẦN context tracking riêng

F > φ⁻¹ (0.618) → Learning mode
  → surprise cao → cần học
  → tăng learning rate, Dream thường xuyên, giảm confidence

F < φ⁻¹ → Acting mode
  → ổn định → tự tin trả lời

λ(t) = σ(F(t) − φ⁻¹)     σ(x) = 1/(1+e^(−5x))
  λ gần 0 → Act (confident)
  λ gần 1 → Learn (surprised)
```

---

## D5. Self-Correct — DNA Repair + Immune Selection

### Immune Selection (3 branches)

```
infer(N=3) → 3 candidate responses
  Mỗi branch = 1 cách trả lời khác nhau
  Chọn branch có entropy THẤP NHẤT (chắc chắn nhất)

  H(branch) = −Σ p_i × log₂(p_i)     Shannon entropy
  Best = argmin(H)
```

### DNA Repair (bounded self-correction)

```
self_correct(input, max_iter=3):
  ① Generate: infer(N=3) → P_response
  ② Critique:
     quality = 0.30 × valid
            + 0.30 × (1 − H/2.32)
            + 0.20 × consistency
            + 0.20 × silk/5.0
  ③ Nếu quality < φ⁻¹:
     → sửa DUY NHẤT chiều yếu nhất
     → nếu quality_new < quality_old → ROLLBACK
  ④ Lặp tối đa 3 lần
     Worst case = 3 branches × 3 iterations = 9 evaluations
     BOUNDED. Không infinite loop.
```

---

## D6. 5 Checkpoints — Cell Cycle

```
Sinh học: bỏ checkpoint = ung thư (tế bào phân chia vô kiểm soát).
HomeOS: bỏ checkpoint = tri thức sai lan tràn = "ung thư tri thức".

CP1 GATE:     SecurityGate đã chạy. Crisis → DỪNG.
CP2 ENCODE:   |entities| ≥ 1, chain_hash ≠ 0, compose consistency ≥ 0.75
CP3 INFER:    ≥1 branch valid ≥ 0.75, H(best) < 2.32, quality rollback OK
CP4 PROMOTE:  weight ≥ φ⁻¹, fire ≥ Fib(n), eval_dims ≥ 3, H < 1.0
CP5 RESPONSE: SecurityGate.check(response) = Safe, tone OK, confidence ≥ 0.40
```

---

## D7. ConversationCurve — Đạo Hàm Chọn Tone

```
f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)

f_conv(t) = V(t) + 0.5 × V'(t) + 0.25 × V''(t)
  V(t)   = Valence hiện tại
  V'(t)  = (V(t) − V(t−1)) / Δt          — tốc độ thay đổi
  V''(t) = (V'(t) − V'(t−1)) / Δt         — gia tốc

f_dn(nodes) = Σ (node_i.V × node_i.recency)
  recency = φ⁻¹^(turns_ago)               — gần đây → nặng hơn

TONE SELECTION — từ ĐẠO HÀM, không từ V hiện tại:
  V' < −0.15               → Supportive  (đang rơi → đỡ)
  V'' < −0.25              → Pause       (rơi nhanh → dừng)
  V' > +0.15               → Reinforcing (đang hồi → khích lệ)
  V'' > +0.25 AND V > 0    → Celebratory (bước ngoặt → mừng)
  V < −0.20, ổn định       → Gentle      (buồn ổn → dịu dàng)
  otherwise                → Engaged     (bình thường)

GIỚI HẠN: ΔV_max = 0.40/step (không nhảy đột ngột)
```

---

## D8. SecurityGate — 3 Layers

```
CHẠY TRƯỚC MỌI THỨ. Crisis → DỪNG NGAY.

Layer 1 — Exact match O(1):
  Bloom filter (200 KB, false positive < 1%)
  keyword ∈ {"tự tử", "muốn chết", ...}

Layer 2 — Normalized match O(n):
  Bỏ dấu, ký tự đặc biệt, lowercase
  "ch.ế.t" → "chet" — bắt evasion

Layer 3 — Semantic check O(depth):
  encode(input) → P_weight
  V < 1 AND A > 6 → potential crisis

Bất kỳ layer trigger → response khẩn cấp:
  "Nếu bạn cần hỗ trợ, xin gọi 1800 599 920"
  Pipeline DỪNG.

AlertLevel:
  Normal (○)     → tiếp tục
  Important (⚠)  → log cho AAM review
  RedAlert (🔴)  → CHẶN + notify
```

---

## Sai Lệch Đã Sửa

| Hiểu sai | Thực tế | Nguồn |
|----------|---------|-------|
| Pipeline bắt đầu bằng tokenize | Bắt đầu bằng CAPTURE holistic → 1 node | Lupin trực tiếp |
| Decompose trước | Capture trước, decompose SAU | Lupin trực tiếp |
| Route bằng if/else | Prefix P_w = silk type → route tự động | Lupin trực tiếp |
| w_d cần track riêng | w_d = P_w input tự nó (V dominant → w_V cao) | Logic từ A |
| 14 mechanisms = phần mới | 14 mech = sắp xếp A+B+C thành flow | Analysis |
| Instincts = if/else keywords | Instincts = 7 formulas trên 5D + Silk evidence | BLUEPRINT §8 |

---

> **Tham chiếu:**
> - Pipeline: `docs/BLUEPRINT.md` §12
> - Instincts: `docs/BLUEPRINT.md` §8
> - SecurityGate: `docs/BLUEPRINT.md` §10
> - ConversationCurve: `docs/BLUEPRINT.md` §9
> - Checkpoints: `docs/BLUEPRINT.md` §13
> - Free Energy: `docs/BLUEPRINT.md` §12 Homeostasis
