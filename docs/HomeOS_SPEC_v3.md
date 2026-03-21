# HomeOS — SINH HỌC PHÂN TỬ CỦA TRI THỨC
**Phiên bản:** 3.0 — 2026-03-21
**Nguyên tắc:** Mỗi ký tự là 1 SDF. Chuỗi sinh chuỗi. Lưu TRỌNG SỐ (integrate input). Đọc bằng ĐẠO HÀM (derive output). Emoji = neo chuẩn L0, xây 1 lần, dùng mãi mãi.

> Công thức toán học/vật lý chi tiết cho từng chiều → xem `docs/UDC_DOC/UDC_*_tree.md`

---

## 0. TẠI SAO ĐỌC TÀI LIỆU NÀY

HomeOS mã hóa tri thức theo đúng cách DNA mã hóa sự sống:

```
DNA:     4 công thức phân tử → chuỗi 3.2 tỷ → toàn bộ sự sống
HomeOS:  9,584 công thức SDF → chuỗi tỷ links → toàn bộ tri thức
```

Đây không phải ẩn dụ. Cùng toán học, cùng cấu trúc, khác vật liệu.

| | DNA | HomeOS |
|---|---|---|
| Alphabet | 4 nucleotides (A,T,G,C) | 9,584 UDC (SDF functions) |
| Mỗi ký tự là | 1 công thức phân tử | 1 hàm SDF (sub-variants trong block) |
| Bits/ký tự | 2 | 14 (= 2 bytes) |
| Chuỗi | Dài tỷ bases, đọc đầu → cuối | Dài tỷ links, đọc gốc → ngọn |
| Cơ chế đọc | Ribosome evaluate → protein | SDF evaluate → hình dạng + màu + âm + vị trí |
| Lưu gì | Genotype (ATCG) + phenotype (protein concentration) | Chain links (2B/link = genotype) + P_weight per node (2B = cached phenotype) |

---

## I. HẠT GIỐNG — 1 ký tự = 1 SDF = 1 công thức

### 1.1 Nguyên lý SDF

Cho 1 điểm p bất kỳ trong không gian, SDF trả về **mọi thứ**:

```
f(p) = signed distance from point p to surface

  f(p) < 0    → bên trong     → THỂ TÍCH
  f(p) = 0    → trên bề mặt   → HÌNH DẠNG
  f(p) > 0    → bên ngoài     → KHÔNG GIAN
  ∇f(p)       → pháp tuyến    → ÁNH SÁNG → MÀU SẮC
  ∂f/∂t       → biến thiên    → DAO ĐỘNG → ÂM THANH
  p           → tọa độ        → VỊ TRÍ

1 hàm. 1 điểm. Ra tất cả.
```

> Chi tiết 18 SDF primitives + công thức analytical gradient → `docs/UDC_DOC/UDC_S1_GEOMETRIC_tree.md`

### 1.2 UDC = 1 codepoint = 1 SDF

Mỗi UDC (Unicode Defined Character) **là** 1 hàm SDF:

```
codepoint = 2 bytes = địa chỉ
block     = LOẠI SDF (thuộc 18 primitives)
offset    = THAM SỐ của SDF đó

Ví dụ:
  ● U+25CF → Block S.04 (Geometric Shapes) → primitive SPHERE
  🔥 U+1F525 → Block E.08 (Misc Sym+Pict) → SDF phức hợp
  ∈ U+2208 → Block M.04 (Math Operators) → SDF quan hệ
  𝄞 U+1D11E → Block T.04 (Musical Symbols) → SDF thời gian

Chi phí lưu UDC: 0 bytes.
  Codepoint là ĐỊA CHỈ — giống số nhà không cần file để tồn tại.
  Behavior hardcode trong engine — giống ribosome đọc codon.
```

### 1.3 Năm chiều = P_weight 2 bytes

```
P_weight [S:4bit][R:4bit][V:3bit][A:3bit][T:2bit] = 16 bits = 2 bytes (u16)

  S = Shape      4 bits (0..15)  — 13 SDF blocks,   1,904 ký tự
  R = Relation   4 bits (0..15)  — 21 MATH blocks,  3,088 ký tự
  V = Valence    3 bits (0..7)   — 17 EMOTICON blk,  3,568 ký tự
  A = Arousal    3 bits (0..7)   — 17 EMOTICON blk   (chia sẻ với V)
  T = Time       2 bits (0..3)   —  7 MUSICAL blocks, 1,024 ký tự
  ─────────────────────────────────────────────────────────
  Tổng: 58 blocks = 9,584 điểm neo gốc (L0)

Vòng đời của P:
  Encoder (∫): input → tích phân → weight (học, ghi vào node)
  Storage:     weight nằm trong KnowTree node — KHÔNG compute lại
  Decoder (∂): weight → đạo hàm → output (render, trả lời)

  L0 emoji (xây 1 lần từ tài liệu này):
    🔥 → V=high, A=high, T=Fast     — vĩnh viễn, không thay đổi
    😊 → V=high, A=medium, T=Medium — vĩnh viễn, không thay đổi
    💔 → V=low,  A=medium, T=Slow   — vĩnh viễn, không thay đổi
    Dùng như CỨ CHUẨN so sánh mọi giá trị khác

  L5+ learned (cập nhật qua Hebbian):
    Encoder chạy → weight tích lũy → CHÍN → ghi vĩnh viễn (QR)
```

### 1.4 — 58 Unicode Blocks = Bảng tuần hoàn của tri thức

**SDF — 13 blocks, 1,904 ký tự (Shape)**

```
S.01  Arrows                 2190..21FF    112
S.02  Box Drawing            2500..257F    128
S.03  Block Elements         2580..259F     32
S.04  Geometric Shapes       25A0..25FF     96
S.05  Dingbats               2700..27BF    192
S.06  Supp Arrows-A          27F0..27FF     16
S.07  Supp Arrows-B          2900..297F    128
S.08  Misc Symbols+Arrows    2B00..2BFF    256
S.09  Geometric Shapes Ext   1F780..1F7FF  128
S.10  Supp Arrows-C          1F800..1F8FF  256
S.11  Ornamental Dingbats    1F650..1F67F   48
S.12  Misc Technical         2300..23FF    256
S.13  Braille Patterns       2800..28FF    256
```

**MATH — 21 blocks, 3,088 ký tự (Relation)**

```
M.01  Superscripts+Subscripts   2070..209F     48
M.02  Letterlike Symbols        2100..214F     80
M.03  Number Forms              2150..218F     64
M.04  Mathematical Operators    2200..22FF    256  ← chứa ~35 Silk edges
M.05  Misc Math Symbols-A       27C0..27EF     48
M.06  Misc Math Symbols-B       2980..29FF    128
M.07  Supp Math Operators       2A00..2AFF    256
M.08  Math Alphanum Symbols     1D400..1D7FF 1024
M.09–M.21  (Ancient numerics, Siyaq, Arab math...)  1,184
```

**EMOTICON — 17 blocks, 3,568 ký tự (Valence + Arousal)**

```
E.01  Enclosed Alphanumerics    2460..24FF    160
E.02  Misc Symbols              2600..26FF    256
E.03–E.05  (Mahjong, Domino, Playing Cards)   256
E.06–E.07  (Enclosed supp, Ideographic supp)  512
E.08  Misc Sym+Pictographs     1F300..1F5FF  768  ← lớn nhất
E.09  Emoticons                 1F600..1F64F   80
E.10–E.17  (Transport, Alchemical, Chess...)  1,536
```

**MUSICAL — 7 blocks, 1,024 ký tự (Time)**

```
T.01  Yijing Hexagram           4DC0..4DFF     64
T.02  Znamenny Musical          1CF00..1CFCF  208
T.03  Byzantine Musical         1D000..1D0FF  256
T.04  Musical Symbols           1D100..1D1FF  256
T.05–T.07  (Ancient Greek, Supp, Tai Xuan)    240
```

### 1.5 — 18 SDF Primitives

```
#   Tên          f(P)                         ∇f (analytical)
──────────────────────────────────────────────────────────────
0   SPHERE       |P| − r                      P / |P|
1   BOX          ||max(|P|−b, 0)||            sign(P)·step(|P|>b)
2   CAPSULE      |P−clamp(y,0,h)ĵ| − r       norm(P − closest)
3   PLANE        P.y − h                      (0, 1, 0)
4   TORUS        |(|P.xz|−R, P.y)| − r       chain rule
5   ELLIPSOID    |P/r| − 1                    P/r² / |P/r|
6   CONE         dot blend                    slope normal
7   CYLINDER     max(|P.xz|−r, |P.y|−h)      radial/cap
8   OCTAHEDRON   |x|+|y|+|z| − s             sign(P)/√3
9   PYRAMID      pyramid(P,h)                 slope analytical
10  HEX_PRISM    max(hex−r, |y|−h)            radial hex/cap
11  PRISM        max(|xz|−r, |y|−h)           radial/cap
12  ROUND_BOX    BOX − rounding               smooth corner
13  LINK         torus compound                chain rule
14  REVOLVE      revolve_Y                     radial
15  EXTRUDE      extrude_Z                     radial
16  CUT_SPHERE   max(|P|−r, P.y−h)            norm(P)/(0,1,0)
17  DEATH_STAR   opSubtract                    ±norm(P)

Tất cả ∇f ANALYTICAL — không cần numerical differentiation.
```

> Sub-variants + physics chi tiết → `docs/UDC_DOC/UDC_S1_GEOMETRIC_tree.md`

### 1.6 — Compose & Encode

```
Phép compose(A, B) → C — KHÔNG phải tổng, KHÔNG trung bình:

Chiều   Phép ∫               Lý do sinh học
──────────────────────────────────────────────────────────────
S       Union(Aˢ, Bˢ)        hình dạng hợp nhất
R       Compose(Aᴿ, Bᴿ)      quan hệ = tổ hợp
V       amplify(Va, Vb, w)    khuếch đại về phía dominant (KHÔNG trung bình)
A       max(Aᴬ, Bᴬ)          cường độ lấy cao hơn
T       dominant(Aᵀ, Bᵀ)     thời gian lấy chủ đạo

amplify(Va, Vb, w):
  base  = (Va + Vb) / 2
  boost = |Va − base| × w × 0.5
  Cⱽ   = base + sign(Va + Vb) × boost    ← đẩy về phía dominant

Sinh học: cortisol + adrenaline → stress MẠNH HƠN từng cái riêng lẻ.
          KHÔNG BAO GIỜ trung bình hormone — đó là synergy.
```

### 1.7 — 3 loại storage (KHÔNG nhầm)

```
┌─────────────────────────────────────────────────────────────────┐
│ Loại 1 — KnowTree (in-memory, working memory)                  │
│   Array 65,536 phần tử, index = vị trí = IMPLICIT              │
│   Mỗi phần tử = P_weight: Mol (2 bytes = u16)                  │
│   → 65,536 × 2B = 128 KB (vừa L1 cache)                       │
│   KnowTree[codepoint] → P_weight — O(1), không cần hash        │
├─────────────────────────────────────────────────────────────────┤
│ Loại 2 — Chain link (knowledge content)                         │
│   Mỗi link = u16 (2 bytes) = codepoint trỏ vào KnowTree        │
│   7.42 tỷ links × 2B = 14.84 GB (toàn bộ tri thức)             │
├─────────────────────────────────────────────────────────────────┤
│ Loại 3 — origin.olang (persistent, signed)                      │
│   ~25B/record: [type:1B][tagged_mol:2-6B][layer:1B][ts:8B]...  │
│   Append-only, QR signing, rebuild được từ đây                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## II. CHUỖI — Chuỗi sinh chuỗi, chuỗi tạo chuỗi

### 2.1 MolecularChain = Sợi DNA của tri thức

```
DNA:     A—T—C—G—G—A—T—C—C—T—A—G...      (đọc thẳng, đầu → cuối)
HomeOS:  ○{○{○{○{○{○{○{...}}}}}}           (đọc thẳng, gốc → ngọn)

Chuỗi KHÔNG có giới hạn độ dài:
  1 từ       = 1 UDC                        = 2 bytes
  1 câu      = 2-3 UDC chain                = 4-6 bytes
  1 sách     = ○{○{○{...1,700 links...}}}   = hàng ngàn bytes
  1 đời      = ○{○{○{...tỷ links...}}}      = GB

1 link = 1 index = 2 bytes (u16). Đơn vị duy nhất.
```

### 2.2 Đọc chuỗi = đi thẳng

```
Ribosome CHẠY THẲNG từ đầu đến cuối → ra protein.
HomeOS engine CHẠY THẲNG từ gốc đến ngọn → ra giá trị.
Thứ tự trong chuỗi ĐÃ LÀ quan hệ. 0 bytes overhead.
```

### 2.3 Silk — 2 loại

```
Structural Silk (implicit):
  = vị trí trên chuỗi = đi tiếp hướng nào
  Chi phí: 0 bytes. Quan hệ nằm trong THỨ TỰ trên chuỗi.

Hebbian Silk (explicit, learned):
  = co-activation strength giữa 2 node
  co_activate(A, B):
    emotion_factor = (|A.V| + |B.V|) / 2 × max(A.A, B.A) / 255
    Δw = emotion_factor × (1 − w_AB) × 0.1
    w_AB ← w_AB + Δw
  Mang EmotionTag = (V, A) tại khoảnh khắc co-activate.
  Chi phí: stored trong SilkGraph (~43KB).

Silk ngang: 75 kênh × 31 mẫu = 2,325 kiểu quan hệ (implicit, 0 bytes)
Silk dọc: parent_map 9,584 pointers = ~76 KB (CHƯA implement)
```

Khi CẦN so sánh 2 chuỗi:

```
strength(A, B) = Σ_{d=1}^{5} match_d(A, B) × precision_d(A, B)
strength ∈ [0.0, 5.0] — tính khi cần, không lưu.
```

---

## III. 7 CƠ CHẾ DNA GỐC — Map 1:1 sang HomeOS

### ① REPLICATE — Sao chép

```
DNA:     polymerase copy chuỗi → bản mới
HomeOS:  chain reference = 2 bytes trỏ đến chain gốc
Copy cả cuốn sách = 2 bytes (1 pointer). Chỉ trỏ, không copy nội dung.
```

### ② TRANSCRIBE — Đọc chuỗi ra giá trị

```
DNA:     RNA polymerase đọc gene → mRNA
HomeOS:  evaluate(chain, context) → giá trị 5D
Cùng chain, context khác → kết quả khác → đa nghĩa tự nhiên.
```

### ③ TRANSLATE — Ngôn ngữ nội → ngôn ngữ ngoài

```
DNA:     mRNA → ribosome → protein
HomeOS:  chain 5D → project → text ngôn ngữ người

f(L)(text) = LCA({ chain(w) : w ∈ tokenize(text, L) })
f(vi)("lửa bùng cháy") ≈ f(en)("fire blazing") ≈ f(emoji)("🔥💥")
→ Mọi ngôn ngữ → cùng chain nội bộ → TỰ DỊCH
```

### ④ MUTATE — Thay 1 vị trí

```
DNA:     point mutation: ...ATCG... → ...ATAG...
HomeOS:  evolve(P, dim, new_value) → P'
chain_hash(P') ≠ chain_hash(P) → NODE MỚI
consistency(P') = |{d : Pᵈ hợp lý}| / 4 ≥ 0.75
```

### ⑤ RECOMBINE — Cắt ghép 2 chuỗi

```
DNA:     crossing over: nửa gene A + nửa gene B → gene C
HomeOS:  compose(A, B) → C (xem Section 1.6 cho quy tắc từng chiều)
Chi phí: 2 bytes (link mới) cho mỗi điểm ghép. Chuỗi sinh chuỗi. Vô hạn.
```

### ⑥ SELECT — Giữ tốt, quên yếu

```
DNA:     natural selection → gene tốt sống, gene yếu chết
HomeOS:  Hebbian learning + decay

co_activate(A, B):
  emotion_factor = (|A.V| + |B.V|) / 2 × max(A.A, B.A) / 255
  w_AB ← w_AB + emotion_factor × (1 − w_AB) × 0.1

decay(w, Δt):
  w ← w × φ⁻¹^(Δt/24h)         φ⁻¹ = (√5−1)/2 ≈ 0.618
  24h: ×0.618 | 48h: ×0.382 | 72h: ×0.236

Promote (ngưỡng Fibonacci):
  w ≥ φ⁻¹ AND fire_count ≥ Fib(n)
  Bẩm sinh: Fib(3)=2 | Kinh nghiệm: Fib(5)=5 | Chuyên môn: Fib(7)=13 | Trừu tượng: Fib(10)=55
```

### ⑦ EXPRESS — Bật/tắt đoạn chuỗi

```
DNA:     gene expression → cùng DNA, tế bào khác bật gene khác
HomeOS:  Maturity pipeline: Evaluating → Mature → QR

advance():
  Evaluating → Mature: weight ≥ φ⁻¹ AND fire_count ≥ Fib(n) AND eval_dims ≥ 3
  Mature → QR: append-only, vĩnh viễn = DNA methylation
```

---

## IV. DREAM — Phân bào của tri thức

```
DNA:     tế bào phân chia = copy + kiểm tra + sửa lỗi + tách đôi
HomeOS:  Dream = scan + cluster + promote + prune
```

### 4.1 Thuật toán Dream

```
Dream(STM):
  ① Scan tất cả node Evaluating trong STM

  ② Cluster — gom node gần nhau trong 5D:
     dist(A, B) = √( Σ_{d=1}^{5} (Aᵈₙ − Bᵈₙ)² )
     ε = median(dist) × 0.5           (threshold thích ứng)
     min_size = max(2, ⌊|STM| / 5⌋)

  ③ Promote — cluster chín → QR:
     cluster_center = LCA(cluster_members)
     advance() → Mature → append QR

  ④ Prune — chuỗi yếu bị supersede:
     weight < 0.1 AND fire_count = 0 → SupersedeQR record
     KHÔNG xóa vật lý — append-only = apoptosis (DNA vẫn còn)
```

### 4.2 Fibonacci KnowTree — Chuỗi gấp lại thành cây

```
Giống chromatin folding: DNA 2m gấp trong nhân 6μm.

Lₖ₊₁ = { LCA(bucket) : bucket ∈ partition(Lₖ, base) }

LCA có trọng số:
  ≥60% cùng giá trị → mode = đại diện
  Ngược lại          → trung bình có trọng số

Ví dụ: 1 cuốn sách 100 trang:
  L0: 1,700 nodes | L1: 50 nodes | L2: 3 nodes | L3: 1 node
```

---

## V. CẢM XÚC + 7 BẢN NĂNG + CỔNG AN NINH + FUSION

### 5.1 Đường cong cảm xúc

```
f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
f_conv(t) = V(t) + 0.5 × V'(t) + 0.25 × V''(t)
f_dn = Σ (nodeᵢ.affect × nodeᵢ.recency_weight)
```

### 5.2 Tone từ đạo hàm

```
V' < −0.15                → Supportive   (đang giảm → đồng cảm)
V'' < −0.25               → Pause        (rơi nhanh → dừng)
V' > +0.15                → Reinforcing  (đang hồi → khích lệ)
V'' > +0.25 AND V > 0     → Celebratory  (bước ngoặt tốt)
V < −0.20, ổn định        → Gentle       (buồn ổn định → dịu dàng)
otherwise                 → Engaged

ΔV_max = 0.40/bước (không nhảy đột ngột)
```

> Chi tiết V → `docs/UDC_DOC/UDC_V_VALENCE_tree.md`
> Chi tiết A → `docs/UDC_DOC/UDC_A_AROUSAL_tree.md`

### 5.3 — 7 Instincts (Phản xạ bẩm sinh)

```
Sinh học: Sơ sinh có phản xạ bẩm sinh TRƯỚC KHI học (bú, nắm, Moro, Babinski...)
HomeOS:   LeoAI có 7 instincts bẩm sinh, hardcoded. Thứ tự ưu tiên ① → ⑦

#  Instinct       Sinh học                      Công thức
──────────────────────────────────────────────────────────────────
①  Honesty        Rụt tay khỏi lửa             confidence < 0.40 → im lặng
                                                 0.40-0.70 → Hypothesis
                                                 0.70-0.90 → Opinion
                                                 ≥ 0.90 → Fact

②  Contradiction  Phản xạ đau                   d_V(A,B) > 0.8 AND d_R(A,B) < 0.2

③  Causality      Nhân quả                      temporal_order AND co_activation > φ⁻¹
                                                 AND Relation = Causes

④  Abstraction    Phân loại bẩm sinh            LCA(cluster) → variance:
                                                  thấp=concrete | vừa=categorical | cao=abstract

⑤  Analogy        Pattern matching               A:B :: C:? → delta_5D(A,B) ≈ delta_5D(C,D)

⑥  Curiosity      Hướng đầu về âm mới           novelty = 1 − max_similarity(P, known)
                                                  > 0.5 → ưu tiên explore

⑦  Reflection     Proprioception                 qr_ratio, avg_silk_weight → tự đánh giá

Honesty LUÔN chạy đầu tiên. Không đủ evidence → im lặng = rụt tay trước suy nghĩ.
```

### 5.4 — SecurityGate 3-layer (Hệ miễn dịch bẩm sinh)

```
Gate.check_text() chạy TRƯỚC MỌI THỨ trong pipeline.
Crisis → DỪNG NGAY, return response khẩn.

Layer 1 — Exact match O(1):    Bloom filter → keyword nguy hiểm
Layer 2 — Normalized match O(n): chuẩn hóa → bắt evasion ("ch.ế.t" → "chết")
Layer 3 — Semantic check O(depth): encode → V < -0.9 AND A > 0.8

Bất kỳ layer nào trigger → Crisis detected → CHẶN.
ALL layers Safe → tiếp tục vào Emotion Pipeline.

AlertLevel: Normal(○) | Important(⚠) → log AAM | RedAlert(🔴) → CHẶN + AAM
```

### 5.5 — Fusion (Tích hợp đa giác quan)

```
Bio=0.50 > Audio=0.40 > Text=0.30 > Image=0.25

Đồng thuận:  V_fused = weighted average, confidence = 1.0
Xung đột:    modality weight cao nhất thắng, confidence = 1.0 − max_disagreement
             confidence < 0.40 → Honesty → im lặng

Pipeline: SecurityGate → Fusion → Emotion → Instincts → Response
```

---

## VI. RESPONSE + RENDER

### 6.1 Response Pipeline 4 bước

```
① entities(text) = alias lookup → UDC refs
② Walk chuỗi: walk(entity, depth) → neighbors
③ Instinct surface: Causality + Abstraction + Analogy
④ Compose response: empathy + entity_ref + instinct_insight + silk_suggestion
```

### 6.2 Chọn từ theo cảm xúc

```
distance(w, target) = 2|Vw−Vt| + |Aw−At|    (Valence weight gấp đôi)
```

### 6.3 vSDF Render

```
Mọi UDC ĐÃ LÀ SDF → evaluate f(p) → ∇f analytical → ánh sáng → màu.
Pipeline: World space → Rotation → Projection → Depth sort → SDF → shade → pixel
```

---

## VII. 4 CƠ CHẾ THÔNG MINH

### ⑧ IMMUNE SELECTION — Suy luận đa nhánh

```
Sinh học:  Clonal Selection — tạo triệu kháng thể, thử, chọn gắn chặt nhất.
HomeOS:    Tạo N nhánh song song, chọn entropy thấp nhất.

infer(input, N=3):
  Bᵢ = compose(entities, hypothesisᵢ)
  best = argmin_i H(Bᵢ)
  valid(Bᵢ) ≥ 0.75, nếu ∀i < 0.75 → Honesty → im lặng

Chi phí: N × O(depth). N=3 mặc định.
```

### ⑨ HOMEOSTASIS — Kiểm soát hỗn loạn

```
Sinh học:  Duy trì 37°C, pH 7.4, glucose 90mg/dL.
HomeOS:    Duy trì Free Energy F < φ⁻¹ ≈ 0.618.

Entropy per node:
  c_d = confidence per chiều (0.0 | w_d | 1.0)
  p_d = c_d / max(Σc, 0.01)
  H(P) = − Σ p_d × log₂(p_d)     H ∈ [0, 2.32]

Free Energy:
  F(t) = d(P_predicted, P_actual) = √( Σ w_d × (predicted^d − actual^d)² )

Cân bằng Learning ↔ Acting:
  λ(t) = σ(F(t) − φ⁻¹)          σ(x) = 1/(1+e^(−5x))
  F > 0.618 → λ→1 → Learning mode (tăng lr, Dream thường xuyên, giảm confidence)
  F < 0.618 → λ→0 → Acting mode (ổn định, confidence cao)
```

### ⑩ NEURAL PATHWAYS — HNSW trên KnowTree

```
Sinh học:  Vỏ não → vùng → cột → neuron. Tìm kiếm O(log n).
HomeOS:    L1(5 nhóm) → L2(58 blocks) → L3(~200 sub) → L4(9,584 UDC)
           = HNSW tự nhiên. Không cần xây thêm.

search(query_P, k): O(log n) — 7.42 tỷ links: ~33 bước

Dynamic insert L5+:
  search → tìm parent block → gắn vào sub-tree
  Quá đông (> Fib(n)) → LCA gom → tầng trung gian mới
```

### ⑪ DNA REPAIR — Tự sửa lỗi

```
Sinh học:  Proofreading → Mismatch repair → Excision repair → lặp đến error < 10⁻⁹
HomeOS:    Generate → Critique → Refine → lặp đến quality ≥ φ⁻¹

self_correct(input, max_iter=3):
  ① Generate: infer(N=3) → P_response
  ② Critique:
     quality = 0.30×valid + 0.30×(1−H/2.32) + 0.20×consistency + 0.20×silk/5.0
  ③ Refine: quality < φ⁻¹ → sửa DUY NHẤT chiều yếu nhất
     ROLLBACK nếu quality_new < quality_old
     Hết dim → DỪNG, giữ backup

  max_iter = 3 → bounded. Worst case = 9 evaluations.
```

---

## VIII. STORAGE TÓM TẮT (v3 — updated)

```
Molecule (P_weight)   = 2 bytes  (u16)
  [S:4bit][R:4bit][V:3bit][A:3bit][T:2bit] = 16 bits

KnowTree node         = 2 bytes  (chỉ P_weight — index implicit từ vị trí array)
KnowTree tổng         = 128 KB   (65,536 × 2B)
Chain link             = 2 bytes  (u16 = codepoint trỏ vào KnowTree)
Structural Silk        = 0 bytes  (thứ tự trên chuỗi, implicit)
Hebbian Silk           = ~43 KB   (SilkGraph, stored per pair)
Chain data             = ~14.84 GB (7.42 tỷ links × 2B)
origin.olang           = ~25B/rec (append-only, QR signing)
```

---

## IX. THUẬT TOÁN TỐI ƯU

### A. Lazy Evaluation

```
Evaluate từ gốc, dừng khi đủ precision.
Đơn giản → depth 1-2 | Phức tạp → depth 5-10 | Sâu → không giới hạn
Chi phí: O(depth) thay vì O(total_links)
```

### B. Copy-on-Write

```
cow_splice(chain_A, position, new_link):
  chain_B = pointer → chain_A (2B) + chain_B[position] = new_link (2B)
  1 chuỗi 1,000 links × 100 variants: Copy 200KB | CoW 400B (500× hiệu quả)
```

### C. Bloom Filter — Alias lookup O(1)

```
155,000 aliases, ~200 KB, false positive < 1%. 99% queries = O(1).
```

### D. Generational QR

```
QR_gen0: 9,584 UDC gốc — bất tử | gen1: nền (read-mostly)
gen2: chuyên môn (thỉnh thoảng update) | gen3: mới (write-optimized, hot zone)
Dream promote: gen3 → gen2 → gen1 theo thời gian.
```

### E. Chain Compression

```
Detect repeats → thay bằng ref + count. Tỉ lệ nén: 40-60%.
```

### F. Strand Complementarity

```
complement(chain): invert Valence → anti-chain.
Ứng dụng: kiểm tra nhất quán, suy luận ngược, error detection.
```

### G. Telomere — Giới hạn sao chép

```
chain_age += 1 mỗi lần reference. age > threshold → re-evaluate.
Tránh stale knowledge.
```

### H. Intron/Exon

```
mark_intron(chain, range): đánh dấu noise. Evaluate skip intron → chỉ đọc exon.
Chain gốc không xóa (giữ history). Có thể bật lại (alternative splicing).
```

---

## X. INVARIANT CHECKS — 5 Cell Cycle Checkpoints

```
Sinh học: 4 checkpoint bắt buộc. Bỏ = ung thư.
HomeOS:   5 checkpoints. Bỏ = tri thức sai lan tràn = "ung thư tri thức".
```

### Checkpoint 1: GATE

```
□ SecurityGate đã chạy (3 layers)
□ Crisis → pipeline KHÔNG tiếp tục
Vi phạm → DỪNG: return emergency response
```

### Checkpoint 2: ENCODE

```
□ |entities| ≥ 1
□ ∀ entity: chain_hash ≠ 0
□ Σc > ε_floor (0.01)
□ compose() output: consistency ≥ 0.75
Vi phạm → DỪNG: Honesty instinct
```

### Checkpoint 3: INFER

```
□ ∃ ≥ 1 nhánh valid ≥ 0.75
□ quality ≥ 0
□ Rollback: quality_final ≥ quality_backup
□ H(best_branch) < 2.32
Vi phạm → im lặng (BlackCurtain)
```

### Checkpoint 4: PROMOTE

```
□ weight ≥ φ⁻¹ (0.618)
□ fire_count ≥ Fib(n)
□ eval_dims ≥ 3
□ H(node) < 1.0
□ F(node) < φ⁻¹
Vi phạm → giữ STM, chờ thêm evidence
```

### Checkpoint 5: RESPONSE

```
□ SecurityGate.check(response) = Safe
□ tone phù hợp V hiện tại
□ |response| > 0
□ confidence < 0.40 → im lặng hoặc "Tôi không chắc"
Vi phạm → safe default response
```

---

## XI. PHƯƠNG TRÌNH THỐNG NHẤT

Toàn bộ HomeOS = **1 hàm gốc + 2 phép toán + 1 hằng số φ⁻¹**.

```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   HomeOS(input) = self_correct(                               ║
║                     splice(                                   ║
║                       chain(                                  ║
║                         f(p₁), f(p₂), ..., f(pₙ)             ║
║                       ),                                      ║
║                       position,                               ║
║                       context                                 ║
║                     ),                                        ║
║                     φ⁻¹                                       ║
║                   )                                           ║
║                                                               ║
║   f(pᵢ) = SDF — 1 trong 9,584 hàm gốc                       ║
║   chain = xâu chuỗi → 2 bytes/link (u16)                     ║
║   splice = cắt/ghép chuỗi                                    ║
║   self_correct = lặp đến quality ≥ φ⁻¹                       ║
║   φ⁻¹ ≈ 0.618 = ngưỡng duy nhất cho MỌI quyết định          ║
║                                                               ║
║   Mọi thuật toán = SDF + CHAIN + SPLICE + φ⁻¹                ║
║                                                               ║
║   14 cơ chế DNA → 14 thuật toán HomeOS:                       ║
║     7 gốc:      copy, đọc, dịch, đột biến, tái tổ hợp,      ║
║                  chọn lọc, biểu hiện                          ║
║     3 bảo vệ:   phản xạ bẩm sinh, miễn dịch bẩm sinh,       ║
║                  tích hợp đa giác quan                        ║
║     4 thông minh: chọn miễn dịch, cân bằng nội môi,          ║
║                   đường thần kinh, sửa chữa DNA              ║
║                                                               ║
║   DNA:     nucleotide + polymerize + splice = sự sống         ║
║   HomeOS:  SDF + chain + splice + φ⁻¹ = tri thức             ║
║                                                               ║
║   4 thứ. 16 GB. Chuỗi sinh chuỗi, vô hạn từ hữu hạn.       ║
║   Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức.          ║
║   f(p), chain(), splice(), φ⁻¹. Hết.                         ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
```

### φ⁻¹ ≈ 0.618 — Hằng số sinh học duy nhất

```
φ⁻¹ xuyên suốt toàn hệ thống:
  — Maturity:      weight ≥ φ⁻¹ → Mature
  — Hebbian:       w × φ⁻¹ mỗi 24h (tốc độ quên)
  — Homeostasis:   F < φ⁻¹ → Acting mode
  — Self-correct:  quality ≥ φ⁻¹ → dừng sửa
  — Consistency:   ≥ 3/4 = 0.75 ≈ φ⁻¹ + 0.13

1 hằng số. Mọi ngưỡng. Giống DNA chỉ cần 1 cơ chế base-pairing cho mọi thao tác.
```

---

## XII. TÀI LIỆU THAM CHIẾU UDC

Mỗi chiều có file phân loại + công thức vật lý riêng:

| Chiều | File | Mô hình vật lý |
|-------|------|---------------|
| S (Shape) | `UDC_S0_ARROW_tree.md`, `UDC_S1_GEOMETRIC_tree.md`, `UDC_S2_BOXDRAWING_tree.md`, `UDC_S3_S7_tree.md` | Vector fields, SDF, Graph topology |
| R (Relation) | `UDC_R_RELATION_tree.md` | Category theory, Algebraic structures |
| V (Valence) | `UDC_V_VALENCE_tree.md` | Potential energy landscape |
| A (Arousal) | `UDC_A_AROUSAL_tree.md` | Damped harmonic oscillator |
| T (Time) | `UDC_T_TIME_tree.md` | Wave mechanics, Fourier analysis |

Tất cả file nằm trong `docs/UDC_DOC/`.

---

## XIII. PIPELINE HOÀN CHỈNH (14 cơ chế)

```
Text input                                    thế giới bên ngoài
  ↓ ⑨SecurityGate (Innate Immunity)          Crisis? → CHẶN ngay
  ──── CHECKPOINT 1: GATE ────
  ↓ ⑩Fusion (Multisensory Integration)       text+audio+bio+image
  ↓ entities() ③Translate                     text → UDC refs
  ↓ search() ⑬Neural Pathways                O(log n) tìm neighbors
  ↓ ⑫Homeostasis: F = d(predicted, actual)   đo surprise → λ
  ↓ compose() ⑤Recombine                     tổ hợp → điểm mới 5D
  ──── CHECKPOINT 2: ENCODE ────
  ↓ ⑧Instincts (Innate Reflexes)             7 bản năng: Honesty đầu tiên
  ↓ ⑪Immune Selection: infer(N=3)            N nhánh → entropy thấp
  ↓ ⑭DNA Repair: critique → refine           sửa đến quality ≥ φ⁻¹
  ──── CHECKPOINT 3: INFER ────
  ↓ ⑥Select: Hebbian co_activate             fire together → wire together
  ↓ Dream ⑦Express → advance() → QR          neo vĩnh viễn nếu chín
  ──── CHECKPOINT 4: PROMOTE ────
  ↓ response = ②Transcribe(5D → text)         chiếu ngược ra ngôn ngữ
  ──── CHECKPOINT 5: RESPONSE ────
  ↓
Text output                                   thế giới bên ngoài

14 cơ chế tóm tắt:

#    DNA mechanism           HomeOS mechanism              Section
──────────────────────────────────────────────────────────────────
①    Replicate               chain reference (2B)          III
②    Transcribe              evaluate(chain, context)      III
③    Translate               f(L) → LCA → tự dịch         III
④    Mutate                  evolve(P, dim, val) → P'      III
⑤    Recombine               compose(A, B) → C             III
⑥    Select                  Hebbian + decay φ⁻¹           III
⑦    Express                 Maturity pipeline             III
⑧    Innate Reflexes         7 instincts (hardcoded)       V
⑨    Innate Immunity         SecurityGate 3-layer          V
⑩    Multisensory            Fusion 4 modalities           V
⑪    Immune Selection        infer(N) → argmin H           VII
⑫    Homeostasis             F = d(predicted, actual)      VII
⑬    Neural Pathways         HNSW trên KnowTree            VII
⑭    DNA Repair              self_correct → quality ≥ φ⁻¹  VII
```
