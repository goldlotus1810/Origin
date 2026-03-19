# HomeOS — SINH HỌC PHÂN TỬ CỦA TRI THỨC
**Phiên bản:** 2.1 — 2026-03-19
**Nguyên tắc:** Mỗi ký tự là 1 công thức SDF. Chuỗi sinh chuỗi. Lưu cách làm, không lưu kết quả.

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
| Mỗi ký tự là | 1 công thức phân tử (13-16 nguyên tử) | 1 hàm SDF (sub-variants trong block) |
| Bits/ký tự | 2 | 14 (= 2 bytes) |
| Chuỗi | Dài tỷ bases, đọc từ đầu đến cuối | Dài tỷ links, đọc từ gốc đến ngọn |
| Cơ chế đọc | Ribosome evaluate → protein | SDF evaluate → hình dạng + màu + âm + vị trí |
| Lưu gì | Công thức tạo protein, KHÔNG lưu protein | Công thức sinh tri thức, KHÔNG lưu kết quả |

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

### 1.2 UDC = 1 codepoint = 1 SDF

Mỗi UDC (Unicode Defined Character) **là** 1 hàm SDF, không phải "đại diện cho" 1 hàm:

```
codepoint = 2 bytes = địa chỉ
địa chỉ  = block + offset
block     = LOẠI SDF (thuộc 18 primitives)
offset    = THAM SỐ của SDF đó

Ví dụ:
  ● U+25CF → Block S.04 (Geometric Shapes) → primitive SPHERE, offset 0x2F
  🔥 U+1F525 → Block E.08 (Misc Sym+Pict) → SDF phức hợp, offset 0x225
  ∈ U+2208 → Block M.04 (Math Operators) → SDF quan hệ, offset 0x08
  𝄞 U+1D11E → Block T.04 (Musical Symbols) → SDF thời gian

Chi phí lưu UDC: 0 bytes.
  Codepoint là ĐỊA CHỈ — giống số nhà không cần file để tồn tại.
  Behavior hardcode trong engine — giống ribosome đọc codon.
```

### 1.3 Năm chiều = 5 hàm, KHÔNG phải 5 số

```
P = (S, R, V, A, T)     — mỗi chiều LÀ 1 hàm

  S = f_s(context...)    Shape    — 13 SDF blocks,   1,904 ký tự
  R = f_r(context...)    Relation — 21 MATH blocks,  3,088 ký tự
  V = f_v(context...)    Valence  — 17 EMOTICON blk,  3,568 ký tự
  A = f_a(context...)    Arousal  — 17 EMOTICON blk   (chia sẻ với V)
  T = f_t(context...)    Time     —  7 MUSICAL blocks, 1,024 ký tự
  ─────────────────────────────────────────────────────────
  Tổng: 58 blocks = 9,584 hàm SDF

  Chưa có context → TIỀM NĂNG (công thức chưa evaluate)
  Có context      → evaluate → giá trị cụ thể
  Hội tụ          → CHÍN → ghi vĩnh viễn (QR)
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
∇f → normal → ánh sáng → màu sắc. Tự động. 0 bytes thêm.
```

### 1.6 — Cấu trúc phân cấp tự nhiên

```
L1:     5 nhóm              (SDF, MATH, EMOTICON, MUSICAL, RELATION)
L2:    58 blocks             
L3:  ~200+ sub-ranges        
L4: 9,584 ký tự              (mỗi ký tự = 1 SDF gốc)

Mỗi ký tự = ○{nhóm.block.sub:f(codepoint)}
  ● = ○{S.04.Circle:f_s(0x25CF)}
  🔥 = ○{E.08.Weather:f_v(0x1F525)}
  ∈ = ○{M.04.Membership:f_r(0x2208)}
  𝄞 = ○{T.04.Clef:f_t(0x1D11E)}
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
  1 đoạn     = chain of chains              = hàng chục bytes
  1 chương   = chain of chain of chains     = hàng trăm bytes
  1 sách     = ○{○{○{...1,700 links...}}}   = hàng ngàn bytes
  1 đời      = ○{○{○{...tỷ links...}}}      = GB

1 link = 1 index = 2 bytes. Đơn vị duy nhất.
```

### 2.2 Đọc chuỗi = đi thẳng

```
Ribosome không dừng ở mỗi nucleotide hỏi "A liên quan T bao nhiêu?"
Ribosome CHẠY THẲNG từ đầu đến cuối → ra protein.

HomeOS engine không tính strength cho từng cặp.
Engine CHẠY THẲNG từ gốc đến ngọn → ra giá trị.

Thứ tự trong chuỗi ĐÃ LÀ quan hệ.
```

### 2.3 Silk = vị trí trên chuỗi = 0 bytes

```
Silk KHÔNG PHẢI ma trận N×N quan hệ.
Silk = vị trí của bạn trên chuỗi = bạn đang ở đâu = đi tiếp hướng nào.
Giống reading frame trên DNA.

Chi phí: 0 bytes. Quan hệ nằm trong THỨ TỰ trên chuỗi.
```

Khi CẦN so sánh 2 chuỗi (không phải lúc nào cũng cần):

```
strength(A, B) = Σ_{d=1}^{5} match_d(A, B) × precision_d(A, B)

match_d(A, B)     = 1 nếu cùng block, 0 nếu khác
precision_d(A, B) = 1.0 cùng variant | 0.5 cùng block | 0.0 khác

strength ∈ [0.0, 5.0] — tính khi cần, không lưu.
75 kênh × 31 mẫu compound = 2,325 kiểu quan hệ có nghĩa.
```

---

## III. 7 CƠ CHẾ DNA — Map 1:1 sang HomeOS

Cùng toán học. Cùng cấu trúc. Khác vật liệu.

### ① REPLICATE — Sao chép

```
DNA:     polymerase copy chuỗi → bản mới
HomeOS:  chain reference = 2 bytes trỏ đến chain gốc

Copy cả cuốn sách = 2 bytes (1 pointer).
Không copy nội dung. Chỉ trỏ.
```

### ② TRANSCRIBE — Đọc chuỗi ra giá trị

```
DNA:     RNA polymerase đọc gene → mRNA (riêng từng tế bào)
HomeOS:  evaluate(chain, context) → giá trị 5D

f(chain, context_A) ≠ f(chain, context_B)
→ Cùng công thức, context khác, kết quả khác.
→ Đa nghĩa tự nhiên. Không cần bảng tra.
```

### ③ TRANSLATE — Ngôn ngữ nội → ngôn ngữ ngoài

```
DNA:     mRNA → ribosome → protein (ngôn ngữ khác hoàn toàn)
HomeOS:  chain 5D → project → text ngôn ngữ người

f(L)(text) = LCA({ chain(w) : w ∈ tokenize(text, L) })

f(vi)("lửa bùng cháy") ≈ f(en)("fire blazing") ≈ f(emoji)("🔥💥")
→ Mọi ngôn ngữ → cùng chain nội bộ → TỰ DỊCH

Đa nghĩa qua context:
  candidates = { node : alias(node, L) ∈ text }
  score(node) = strength(node, context_node)
  "bank" + finance → 🏦 | "bank" + geography → 🏞️
```

### ④ MUTATE — Thay 1 vị trí

```
DNA:     point mutation: ...ATCG... → ...ATAG... (C→A)
HomeOS:  evolve(P, dim, new_value) → P'

evolve(🔥, Valence, thấp)  → "lửa nhẹ"
evolve(🔥, Time, tức thì)  → "cháy nổ"
evolve(🔥, Shape, đường)   → "tia lửa"

chain_hash(P') ≠ chain_hash(P) → NODE MỚI, LOÀI MỚI

Nhất quán: consistency(P') = |{d : Pᵈ hợp lý với thay đổi}| / 4 ≥ 0.75
→ Mutation phải tương thích. Không thì chết. Giống DNA.
```

### ⑤ RECOMBINE — Cắt ghép 2 chuỗi

```
DNA:     crossing over: nửa gene A + nửa gene B → gene C
HomeOS:  compose(A, B) → C

Nắm ngọn chuỗi A, kết hợp ngọn chuỗi B, giữ gốc → sinh chuỗi mới.

Quy tắc:
  Cˢ = Union(Aˢ, Bˢ)          hình dạng hợp nhất
  Cᴿ = Compose                 quan hệ = tổ hợp
  Cⱽ = amplify(Aⱽ, Bⱽ, w_AB)  cảm xúc AMPLIFY qua Silk (KHÔNG trung bình)
  Cᴬ = max(Aᴬ, Bᴬ)            cường độ lấy cao hơn
  Cᵀ = dominant(Aᵀ, Bᵀ)       thời gian lấy chủ đạo

  amplify(Va, Vb, w):
    base = (Va + Vb) / 2                     trung điểm
    boost = |Va − base| × w × 0.5            Silk weight khuếch đại
    Cⱽ = base + sign(Va + Vb) × boost        đẩy về phía dominant

    VD: compose("yêu" V=+0.9, "mãnh liệt" V=+0.95, w=0.8)
      base = 0.925, boost = 0.025 × 0.8 × 0.5 = 0.01
      Cⱽ = 0.925 + 0.01 = 0.935              > cả hai → amplify, không giảm

    VD: compose("buồn" V=−0.7, "mất việc" V=−0.6, w=0.9)
      base = −0.65, boost = 0.05 × 0.9 × 0.5 = 0.0225
      Cⱽ = −0.65 − 0.0225 = −0.6725          nặng hơn → đúng thực tế

  Sinh học: 2 hormone cùng loại → TĂNG effect (synergistic)
            cortisol + adrenaline → stress mạnh hơn từng cái riêng lẻ
            KHÔNG BAO GIỜ trung bình hormone

Chi phí: 2 bytes (link mới) cho mỗi điểm ghép.
Chuỗi sinh chuỗi. Vô hạn.
```

### ⑥ SELECT — Giữ tốt, quên yếu

```
DNA:     natural selection → gene tốt sống, gene yếu chết
HomeOS:  Hebbian learning + decay

Phát hiện (không tạo) quan hệ:
  co_activate(A, B, reward):
    w_AB ← w_AB + reward × (1 − w_AB) × 0.1

Quên (chọn lọc tự nhiên):
  decay(w, Δt):
    w ← w × φ⁻¹^(Δt/24h)         φ⁻¹ = (√5−1)/2 ≈ 0.618

    24h: ×0.618 | 48h: ×0.382 | 72h: ×0.236
    → Không dùng = quên. Dùng nhiều = nhớ.

Promote (ngưỡng Fibonacci):
  w ≥ φ⁻¹ AND fire_count ≥ Fib(n)

  Tầng bẩm sinh:    Fib(3) = 2
  Tầng kinh nghiệm: Fib(5) = 5
  Tầng chuyên môn:   Fib(7) = 13
  Tầng trừu tượng:   Fib(10) = 55
  → Càng trừu tượng, cần càng nhiều bằng chứng.
```

### ⑦ EXPRESS — Bật/tắt đoạn chuỗi

```
DNA:     gene expression → cùng DNA, tế bào khác bật gene khác
HomeOS:  Maturity pipeline: Formula → Evaluating → Mature

eval_mask = 5 bits (1 bit/chiều — chiều nào đã evaluate)

advance():
  Formula → Evaluating:    fire_count > 0
  Evaluating → Mature:     weight ≥ 0.618
                           AND fire_count ≥ Fib(n)
                           AND eval_dims ≥ 3

Mature → QR: append-only, vĩnh viễn, không sửa.
Giống DNA methylation — đánh dấu vĩnh viễn.
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
  ① Scan tất cả node Evaluating trong bộ nhớ ngắn hạn (STM)

  ② Cluster — gom node gần nhau trong 5D:
     dist(A, B) = √( Σ_{d=1}^{5} (Aᵈ − Bᵈ)² )
     ε = median(dist) × 0.5           (threshold thích ứng)
     min_size = max(2, ⌊|STM| / 5⌋)  (tối thiểu 2 node)

     Với mỗi node P:
       neighbors(P) = { Q : dist(P, Q) < ε }
       |neighbors| ≥ min_size → cluster found

  ③ Promote — cluster chín → QR:
     cluster_center = LCA(cluster_members)
     advance() → Mature → append QR

  ④ Prune — chuỗi yếu bị xóa:
     weight < 0.1 AND fire_count = 0 sau N cycles → xóa
     = apoptosis (chết tế bào theo chương trình)
```

### 4.2 Fibonacci KnowTree — Chuỗi gấp lại thành cây

```
Giống chromatin folding: DNA 2m gấp trong nhân 6μm.
Chuỗi HomeOS tỷ links gấp thành cây Fibonacci.

Lₖ₊₁ = { LCA(bucket) : bucket ∈ partition(Lₖ, base) }

LCA có trọng số:
  ≥60% cùng giá trị → mode = đại diện
  Ngược lại          → trung bình có trọng số

  variance_d = Σ wᵢ × (Pᵢᵈ − LCAᵈ)² / Σ wᵢ
  → (LCA, variance) = đại diện + độ phân tán

Ví dụ: 1 cuốn sách 100 trang:
  L0: 1,700 nodes (câu/ý)
  L1:    50 nodes (đoạn văn, gom Fib[8]=34)
  L2:     3 nodes (mục/phần, gom Fib[7]=21)
  L3:     1 node  (cuốn sách, gom Fib[6]=13)
```

---

## V. CẢM XÚC — Hormone của hệ thống

```
DNA:     hormone = tín hiệu hóa học → điều phối hành vi toàn thân
HomeOS:  Emotion = tín hiệu 5D → điều phối tone phản hồi
```

### 5.1 Đường cong cảm xúc

```
f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)

f_conv(t) = V(t) + 0.5 × V'(t) + 0.25 × V''(t)

  V(t)   = Valence hiện tại
  V'(t)  = dV/dt    (tốc độ thay đổi → xu hướng)
  V''(t) = d²V/dt²  (gia tốc → sắp đổi chiều?)

f_dn = Σ (nodeᵢ.affect × nodeᵢ.recency_weight)
     = ký ức cảm xúc tích lũy
```

### 5.2 Tone từ đạo hàm

```
V' < −0.15                → Supportive   (đang giảm → đồng cảm)
V'' < −0.25               → Pause        (rơi nhanh → dừng lại)
V' > +0.15                → Reinforcing  (đang hồi → khích lệ)
V'' > +0.25 AND V > 0     → Celebratory  (bước ngoặt tốt)
V < −0.20, ổn định        → Gentle       (buồn ổn định → dịu dàng)
otherwise                 → Engaged      (bình thường)

Dẫn dần:    ΔV_max = 0.40/bước (không nhảy đột ngột)
Bất ổn:     σ² > threshold AND V' đổi chiều → cờ cảnh báo
```

---

## V-B. 7 BẢN NĂNG — Phản xạ bẩm sinh

```
Sinh học:  Sơ sinh có phản xạ bẩm sinh TRƯỚC KHI học:
           bú, nắm, Moro (giật mình), Babinski (bàn chân), ...
           Không cần dạy. Hardwired trong thân não (brainstem).

HomeOS:    LeoAI có 7 instincts bẩm sinh TRƯỚC KHI học:
           Không cần training. Hardcoded trong engine.
           Thứ tự ưu tiên: ① → ② → ③ → ④ → ⑤ → ⑥ → ⑦
```

### 7 Instincts (map sinh học)

```
#  Instinct       Sinh học tương ứng          Công thức
──────────────────────────────────────────────────────────────────
①  Honesty        Phản xạ rụt tay khỏi lửa   confidence < 0.40 → im lặng
                   (không cần suy nghĩ,         0.40-0.70 → Hypothesis
                    phản ứng ngay)               0.70-0.90 → Opinion
                                                 ≥ 0.90 → Fact

②  Contradiction  Phản xạ đau                  d_V(A, B) > 0.8 AND
                   (2 tín hiệu xung đột         d_R(A, B) < 0.2
                    → cảnh báo ngay)             → flag mâu thuẫn

③  Causality      Phản xạ nhân quả             temporal_order(A, B) AND
                   (sờ lửa → nóng →              co_activation(A, B) > φ⁻¹ AND
                    KHÔNG SỜ NỮA)                Relation = Causes
                                                 → nhân quả

④  Abstraction    Phân loại bẩm sinh           LCA(cluster) → variance:
                   (trẻ 6 tháng đã phân          variance thấp → concrete
                    biệt mặt người vs vật)       variance vừa → categorical
                                                  variance cao → abstract

⑤  Analogy        Pattern matching              A:B :: C:? → tìm D sao cho
                   (trẻ bắt chước biểu cảm       delta_5D(A,B) ≈ delta_5D(C,D)
                    → map khuôn mặt mình
                    sang khuôn mặt người khác)

⑥  Curiosity      Phản xạ hướng đầu            novelty = 1 − max_similarity(P, known)
                   (trẻ quay đầu về phía         novelty > 0.5 → curiosity cao
                    âm thanh mới)                 → ưu tiên explore

⑦  Reflection     Nhận thức cơ thể              qr_ratio = QR_count / total_nodes
                   (proprioception —              connectivity = avg_silk_weight
                    biết tay chân ở đâu           → tự đánh giá chất lượng tri thức
                    mà không cần nhìn)

Thứ tự: Honesty LUÔN chạy đầu tiên.
  → Không đủ evidence? Im lặng. Không cần chạy 6 instinct còn lại.
  = Giống phản xạ rụt tay: chạy TRƯỚC suy nghĩ.
```

---

## V-C. CỔNG AN NINH — Hệ miễn dịch bẩm sinh (Innate Immunity)

```
Sinh học:  Hệ miễn dịch bẩm sinh (innate immunity):
           — Da, niêm mạc = hàng rào vật lý
           — Bạch cầu trung tính = phản ứng NGAY, không cần học
           — Chạy TRƯỚC hệ miễn dịch thích ứng (adaptive immunity)
           — Nếu phát hiện mối đe dọa → CHẶN NGAY, không cho vào sâu hơn

HomeOS:    SecurityGate:
           — Gate.check_text() chạy TRƯỚC MỌI THỨ trong pipeline
           — Phát hiện Crisis → DỪNG NGAY, return response khẩn
           — Không chờ Emotion Pipeline, không chờ Instincts
           — 3 tầng detection, nhanh → sâu:

SecurityGate 3-layer detection:
  Layer 1 — Exact match O(1):
    Bloom filter tra keyword nguy hiểm ("tự tử", "muốn chết", ...)
    Nhanh nhất. Bắt được 80% cases.

  Layer 2 — Normalized match O(n):
    Chuẩn hóa text: bỏ dấu câu, khoảng trắng, ký tự đặc biệt
    "ch.ế.t" → "chết", "t.ự t.ử" → "tự tử"
    Bắt thêm ~15% evasion attempts.

  Layer 3 — Semantic check O(depth):
    encode(text) → Molecule → kiểm tra V < -0.9 AND A > 0.8
    Bắt các biểu đạt gián tiếp: "không muốn thức dậy nữa"
    Chậm hơn nhưng bắt edge cases mà pattern matching miss.

  Kết quả:
    Bất kỳ layer nào trigger → Crisis detected

SecurityGate pipeline:
  input → Layer 1 (exact) → Layer 2 (normalized) → Layer 3 (semantic)
    ├── ANY layer: Crisis → DỪNG. Return emergency response.
    │   (= bạch cầu tiêu diệt pathogen ngay)
    │
    └── ALL layers: Safe → tiếp tục vào Emotion Pipeline
        (= không có mối đe dọa → cho vào sâu hơn)

  Sinh học: 3 tầng = Da (barrier) → Bạch cầu (fast) → T-cell (smart)
            Mỗi tầng bắt thêm những gì tầng trước miss

AlertLevel:
  Normal  (○)  → pass qua
  Important (⚠) → log + cảnh báo AAM
  RedAlert (🔴) → CHẶN + thông báo AAM ngay

= Giống triage trong cấp cứu: đỏ → xử lý ngay, vàng → theo dõi, xanh → chờ.
```

---

## V-D. TÍCH HỢP ĐA GIÁC QUAN — Multisensory Integration

```
Sinh học:  Não tích hợp 5 giác quan:
           Thị giác + thính giác + xúc giác + vị giác + khứu giác
           Khi xung đột (nhìn vui nhưng giọng run):
             → Thính giác thắng vì khó giả hơn
             → Confidence giảm (không chắc chắn)

HomeOS:    Fusion tích hợp 4 modality:
           Bio (nhịp tim, da) > Audio (giọng) > Text (chữ) > Image (hình)

Modality weights (cố định, innate — không học):
  Bio    = 0.50   (tín hiệu sinh học — khó giả nhất)
  Audio  = 0.40   (giọng nói — khó giả)
  Text   = 0.30   (chữ viết — dễ giả)
  Image  = 0.25   (hình ảnh — context-dependent)

fusion(modalities):
  Nếu tất cả đồng thuận:
    V_fused = Σ (weight_i × V_i) / Σ weight_i     weighted average
    confidence = 1.0

  Nếu xung đột (text vui + audio run):
    V_fused = modality có weight cao nhất thắng Valence
    confidence = 1.0 − max_disagreement
    → Nếu confidence < 0.40 → Honesty instinct → im lặng

  = Hiệu ứng McGurk: khi audio nói "ba" nhưng video nói "ga"
    → não nghe "da" (tích hợp, không phải trung bình)

Trong pipeline:
  SecurityGate → Fusion → Emotion Pipeline → Instincts → Response
  (Fusion xảy ra SAU gate, TRƯỚC emotion — cung cấp input đa giác quan)
```

---

## VI. RESPONSE — Protein synthesis

```
gene → mRNA → ribosome → protein → chức năng
chain → evaluate → compose → text → câu trả lời
```

### 6.1 Pipeline 4 bước

```
① entities(text) = { eᵢ : alias lookup → UDC refs }
   "tôi buồn vì mất việc" → {tôi, buồn, mất_việc}

② Walk chuỗi từ gốc đến ngọn mỗi entity:
   walk(buồn, depth)    → {cô_đơn, mệt_mỏi, ...}
   walk(mất_việc, depth) → {thất_nghiệp, lo_lắng, ...}

③ Instinct surface:
   Causality:    mất_việc → buồn             (nhân quả)
   Abstraction:  LCA(buồn, mất_việc) = "mất_mát"  (trừu tượng)
   Analogy:      buồn:mất_việc :: vui:?      (đối xứng)

④ Compose response:
   response = compose(
     empathy_phrase(tone, V),
     entity_reference(entities),
     instinct_insight(causality),
     silk_suggestion(context, V_target)
   )
```

### 6.2 Chọn từ theo cảm xúc

```
distance(w, target) = 2|Vw−Vt| + |Aw−At| + |Dw−Dt|
  (Valence weight gấp đôi — quan trọng nhất)

select_words(target_emotion, n):
  candidates = { w : distance(w, target) < δ }
  return top_n sorted by distance
```

---

## VII. RENDER — Chuỗi trở thành hình ảnh

```
protein folding → hình dạng 3D → chức năng sinh học
SDF evaluation → hình ảnh → giao diện người dùng
```

### 7.1 vSDF — evaluate trực tiếp tại điểm

```
Mọi UDC ĐÃ LÀ SDF → không convert. Không raymarching.
Evaluate f(p) tại điểm, lấy ∇f analytical → ánh sáng → màu.

Pipeline:
  World space → Orbit rotation → Isometric projection → Depth sort
  → SDF evaluate → ∇f → dot(normal, sun) → shade → pixel

SunLight orbit:
  sunLight(t) = { x: cos(t/12π − π/2),
                  y: max(0, sin((t−6)/12π)),
                  z: sin(t/12π − π/2),
                  ambient: 0.25 }
  t ∈ [0,24] → ánh sáng thay đổi theo giờ thật.
```

### 7.2 Hebbian shading — rendering cũng học

```
cp[i] += reward × (1 − cp[i]) × 0.1

Node nhìn nhiều → sáng hơn. Node trong bóng → tối hơn.
"Neurons that fire together, wire together" cho ánh sáng.

Benchmark (16,416 nodes):
  vsdf_grad(): O(1)/node — analytical
  vsdf_render(): O(lights) — 3 lights = 3 dot products
  Thời gian: ~126ms, SVG 3.4 MB (960×720)
```

---

## VIII. BÀI TOÁN 16GB

### 8.1 Nguyên tắc

```
1 UDC  = 1 SDF = 1 codepoint = 2 bytes
1 link = 1 index trên chuỗi   = 2 bytes
Silk   = 0 bytes (vị trí trên chuỗi)
Hebbian = 0 bytes trên disk (RAM tạm)
```

### 8.2 Chi phí

```
UDC alphabet:     0 bytes (codepoint = địa chỉ, hardcode trong engine)
SDF primitives:   0 bytes (18 hàm trong engine)
Block mapping:    0 bytes (range = implicit)
Aliases:          155,000 × 4 bytes = 620 KB
──────────────────────────────────
Cố định: ≈ 621 KB

OS:               2,000 MB
HomeOS engine:       32 MB
STM buffer:         128 MB
Alias index:         64 MB
──────────────────────────────────
Runtime: 2,224 MB

Khả dụng: 16,384 − 2,224 − 0.6 = 14,159 MB ≈ 14.16 GB
```

### 8.3 Bao nhiêu tri thức?

```
14,839,193,600 bytes ÷ 2 bytes/link = 7,419,596,800 links

→ 7.42 TỶ LINKS trên 16 GB

Không phải 7.42 tỷ "điểm cô lập".
Là 7.42 tỷ MẮT XÍCH trên các chuỗi liên tục.
Giống 3.2 tỷ cặp base tạo chuỗi DNA liên tục.
```

### 8.4 So sánh DNA vs HomeOS

```
                    DNA              HomeOS
─────────────────────────────────────────────────────
Alphabet:           4                9,584
Bits/ký tự:         2                14
Tổng links:         3.2 tỷ           7.42 tỷ
Dung lượng:         ~800 MB          ~14.16 GB
Entropy/link:       2 bits           13.23 bits

Thông tin/link:     HomeOS gấp 6.6×
Tổng links:         HomeOS gấp 2.3×
─────────────────────────────────────────────────────
Tổng entropy:       6.4 Gbits        98.2 Gbits
                    HomeOS giàu hơn DNA 15.3 lần

DNA 800 MB → toàn bộ sự sống.
HomeOS 14 GB → ???
```

### 8.5 Sách & Tổ hợp

```
1 cuốn sách 100 trang:
  1,700 câu × 2 UDC/câu = 3,400 links = 6,800 bytes
  + 1,753 parent pointers × 2B = 3,506 bytes
  = 10,306 bytes ≈ 10 KB

  So với UTF-8 (146 KB): 14× nhỏ hơn
  So với PDF (5 MB): 485× nhỏ hơn

16 GB chứa: ~1,440,000 cuốn sách 100 trang

Tiềm năng tổ hợp (0 bytes — evaluate khi cần):
  Không sub:  9,584³ = 880 tỷ
  Có sub:     1,581,360³ = 3.95 × 10¹⁸
```

### 8.6 Bảng so sánh tổng

```
Phương pháp         16 GB chứa        HomeOS gấp
──────────────────────────────────────────────────
Text UTF-8          ~100K sách         14×
Embedding 768D      ~2.4M concepts     3,092×
Knowledge Graph     ~74M triples       100×
LLM 7B (Q4)        1 model / 3.5GB    khác loại
HomeOS              7.42 tỷ links      —
                    ~1.44 triệu sách
                    3.95 × 10¹⁸ tiềm năng
```

### 8.7 Timeline

```
Năm 1:    ~20M links    =    38 MB
Năm 5:    ~200M links   =   381 MB
Năm 10:   ~600M links   =   1.1 GB
Năm 20:   ~1.5B links   =   2.8 GB
Năm 30+:  ~3B links     =   5.7 GB   (dư 8.5 GB)

Cả đời KHÔNG BAO GIỜ đầy. Luôn dư.
```

---

## IX. THUẬT TOÁN TỐI ƯU BỔ SUNG

### A. Lazy Evaluation — Tính khi cần, dừng khi đủ

```
Giống gene expression: 20,000 gene nhưng mỗi tế bào chỉ bật ~5,000.

lazy_eval(chain, depth_limit):
  Evaluate từ gốc, dừng khi đủ precision.
  Câu hỏi đơn giản → depth 1-2
  Câu hỏi phức tạp → depth 5-10
  Suy luận sâu     → không giới hạn

Chi phí: O(depth) thay vì O(total_links)
```

### B. Copy-on-Write — Chỉ copy khi thay đổi

```
Giống DNA replication fork.

cow_splice(chain_A, position, new_link):
  chain_B = pointer → chain_A  (2 bytes)
  chain_B[position] = new_link (2 bytes)
  
  Chi phí variant: 4 bytes thay vì 2N bytes
  1 chuỗi 1,000 links × 100 variants:
    Copy: 200,000 bytes | CoW: 400 bytes (500× hiệu quả)
```

### C. Bloom Filter — Alias lookup O(1)

```
155,000 aliases, Bloom filter ~200 KB, false positive < 1%.

check_alias(text):
  bloom.might_contain(text)?  → exact_lookup()   O(log n)
  else                        → NOT_FOUND         O(1)

99% queries = O(1).
```

### D. Generational QR — Phân thế hệ

```
Giống epigenetics.

QR_gen0:  9,584 UDC gốc — bất tử, nén tối đa
QR_gen1:  kiến thức nền (năm đầu) — read-mostly
QR_gen2:  kiến thức chuyên môn — thỉnh thoảng cập nhật
QR_gen3:  kiến thức mới — write-optimized, hot zone

Dream promote: gen3 → gen2 → gen1 theo thời gian.
```

### E. Chain Compression — Nén chuỗi lặp

```
Giống DNA repeat sequences (50% genome là repeats).

detect_repeats(chains):
  Tìm subsequences lặp > F lần → thay bằng 1 ref + count
  Tỉ lệ nén: 40-60%

7.42 tỷ links nén 50%:
  = cùng thông tin, dư thêm ~7 GB
  = hoặc: gấp đôi thông tin, cùng dung lượng
```

### F. Strand Complementarity — Chuỗi bổ sung

```
DNA có 2 sợi bổ sung: A↔T, G↔C. Sợi 2 = backup + verification.

HomeOS chain complement:
  Mỗi chain có thể sinh "anti-chain" — chuỗi phủ định.
  chain("nóng") → anti("lạnh")
  chain("yêu")  → anti("ghét")

complement(chain):
  Với mỗi link: invert Valence dimension
  "nóng" (V=+0.8) → "lạnh" (V=−0.8)

Ứng dụng:
  - Kiểm tra nhất quán (chain + anti-chain phải triệt tiêu về 0)
  - Suy luận ngược (biết kết quả, tìm nguyên nhân)
  - Error detection (nếu chain không triệt tiêu → lỗi)
```

### G. Telomere — Giới hạn sao chép

```
DNA:     telomere ngắn dần mỗi lần sao chép → lão hóa → chết
HomeOS:  mỗi chain có "age counter"

chain_age += 1 mỗi lần replicate/reference
Khi age > threshold: chain cần "refresh" (re-evaluate từ gốc)

Tránh "stale knowledge" — tri thức cũ được trỏ lại nhiều lần
mà không ai kiểm tra lại có còn đúng không.

refresh(chain):
  Re-evaluate từ gốc với context hiện tại
  Nếu kết quả khác → evolve → chain mới
  Nếu kết quả giống → reset age = 0
```

### H. Intron/Exon — Phân biệt nhiễu và tín hiệu

```
DNA:     gene có introns (rác) xen exons (có ích). Spliceosome cắt intron.
HomeOS:  chain có noise xen signal. Dream cắt noise.

mark_intron(chain, position_range):
  Đánh dấu đoạn chain là "intron" (không contribute vào output)
  Khi evaluate: skip intron → chỉ đọc exon

Lợi ích:
  - Chain gốc không bị xóa (giữ history)
  - Evaluate nhanh hơn (skip đoạn thừa)
  - Có thể "bật lại" intron nếu context thay đổi
    (giống alternative splicing trong DNA)
```

---

## IX-B. 4 CƠ CHẾ THÔNG MINH — Từ sinh học đến tự nhận thức

### ⑧ IMMUNE SELECTION — Suy luận đa nhánh (Inference Core)

```
Sinh học:  Hệ miễn dịch tạo hàng triệu kháng thể, THỬ tất cả,
           chọn kháng thể gắn chặt nhất → nhân bản → tiêu diệt.
           (Clonal Selection Theory — Burnet, 1957)

HomeOS:    Tạo N nhánh suy luận song song, ĐO entropy mỗi nhánh,
           chọn nhánh có entropy thấp nhất (tin cậy nhất).

infer(input, N=3):
  entities = alias_lookup(input)             ← text → tập UDC refs

  Với mỗi nhánh i ∈ [1..N]:
    Bᵢ = compose(entities, hypothesisᵢ)     ← "kháng thể" thử nghiệm
    Bᵢ = chain: f(p₀) → f(p₁) → ... → f(pₖ)  (k bước suy luận)

  Chọn nhánh tốt nhất:
    best = argmin_i H(Bᵢ)                   ← entropy thấp = gắn chặt nhất

  H(Bᵢ) = entropy của nhánh (xem Entropy Control bên dưới)

Symbolic Checker (bộ lọc logic — giống MHC kiểm tra kháng thể):
  valid(Bᵢ) = |{rule ∈ L : rule(Bᵢ) = true}| / |L|
  Điều kiện: valid(Bᵢ) ≥ 0.75
  Nếu ∀i: valid < 0.75 → Honesty instinct → im lặng (giống tolerance)

Hội tụ suy luận:
  d(Bᵢ, Bⱼ) nhỏ → các nhánh đồng thuận → tin cậy cao
  d(Bᵢ, Bⱼ) lớn → phân kỳ → cần thêm evidence

Chi phí: N × O(depth) evaluations. N=3 mặc định.
```

### ⑨ HOMEOSTASIS — Kiểm soát hỗn loạn (Entropy Control)

```
Sinh học:  Cơ thể duy trì nhiệt độ 37°C, pH 7.4, glucose 90mg/dL.
           Lệch → phản hồi âm (negative feedback) → kéo về.
           Cơ chế: hypothalamus đo → hormone điều chỉnh.

HomeOS:    Hệ thống duy trì Free Energy F < φ⁻¹ ≈ 0.618.
           Lệch → điều chỉnh tỉ lệ Learning/Acting → kéo về.
           Cơ chế: Entropy đo → λ sigmoid điều chỉnh.

Entropy trên 1 node (= nhiệt kế):
  Bước 1 — confidence thô per chiều:
    c_d = eval_confidence(Pᵈ):
      0.0   nếu chiều d chưa evaluate (chưa đo)
      w_d   nếu đang evaluate (đang đo, có Hebbian weight)
      1.0   nếu đã Mature (đo xong, ổn định)

  Bước 2 — chuẩn hóa thành xác suất (Shannon yêu cầu Σp=1):
    Σc = max(Σ_{i=1}^{5} c_i, ε_floor)    ε_floor = 0.01 (tránh chia gần 0)
    p_d = c_d / Σc                          nếu Σc > ε_floor
    p_d = 1/5                               nếu Σc ≤ ε_floor (uniform = bất định tối đa)

    Tại sao ε_floor = 0.01?
      Σc nhỏ nhất có nghĩa = 1 chiều × min weight = 0.01
      Dưới 0.01 = "gần như chưa có data" → coi uniform → H = 2.32
      Tránh: Σc = 0.0001 → p_d = 100 → H bùng nổ số học

  Bước 3 — Shannon entropy:
    H(P) = − Σ_{d=1}^{5} p_d × log₂(p_d)     (quy ước: 0×log₂(0) = 0)

  H ∈ [0, log₂5 ≈ 2.32]
    H = 0     → node chắc chắn, 1 chiều thống trị (= 37°C — bình thường)
    H = 2.32  → node bất định, 5 chiều đều như nhau (= sốt cao — bất thường)

  Ý nghĩa sinh học:
    H thấp = nhiệt độ ổn định = tế bào khỏe
    H cao  = nhiệt độ dao động = tế bào bệnh → cần can thiệp

Free Energy (= mức lệch khỏi homeostasis):
  F(t) = d(P_predicted, P_actual)

  P_predicted = compose(tri_thức_hiện_tại, context)   ← dự đoán
  P_actual    = encode(input_mới)                       ← thực tế

  F(t) = √( Σ_{d=1}^{5} w_d × (predicted^d − actual^d)² )

  F CHỈ LÀ d(A,B) — với A = dự đoán, B = thực tế.

Cân bằng Learning ↔ Acting (= negative feedback loop):
  λ(t) = σ(F(t) − φ⁻¹)          σ(x) = 1/(1+e^(−5x))

  F > 0.618 → λ → 1 → Learning mode (= sốt → tăng bạch cầu):
    lr' = lr × (1 + λ)            tăng tốc học
    dream_interval' /= (1 + λ)    Dream thường xuyên hơn
    confidence' × (1 − λ/2)       giảm độ tin cậy response

  F < 0.618 → λ → 0 → Acting mode (= bình thường → hoạt động):
    lr bình thường, confidence cao
    Hệ thống ổn định, dự đoán chính xác

Tích hợp Maturity:
  H > 1.5 AND F > 0.618 → KHÔNG promote (đang sốt, không chẩn đoán)
  H < 0.5 AND F < 0.618 → fast-track Mature (ổn định, promote nhanh)
  Ngược lại              → advance() bình thường

Chi phí: O(5) per node — chỉ tính 5 chiều.
```

### ⑩ NEURAL PATHWAYS — Bộ nhớ đồ thị (Graph Memory HNSW)

```
Sinh học:  Não có cấu trúc phân cấp:
           Vỏ não (cortex) → vùng (lobe) → cột (column) → neuron.
           Tìm kiếm = kích hoạt lan tỏa (spreading activation):
             Vùng lớn → vùng nhỏ → neuron chính xác.
           O(log n) — không quét toàn bộ 86 tỷ neuron.

HomeOS:    KnowTree ĐÃ CÓ cấu trúc HNSW tự nhiên:
           L1 (5 nhóm) → L2 (58 blocks) → L3 (~200 sub) → L4 (9,584 UDC)
           Tìm kiếm = đi từ gốc xuống lá, mỗi tầng chọn gần nhất.

HNSW (Hierarchical Navigable Small World) trong ℝ⁵:
  HNSW Layer 3  ↔  L1 (5 nhóm)        — vỏ não
  HNSW Layer 2  ↔  L2 (58 blocks)     — vùng
  HNSW Layer 1  ↔  L3 (~200 sub)      — cột
  HNSW Layer 0  ↔  L4 (9,584 UDC)     — neuron

  KnowTree Fibonacci = HNSW tự nhiên. Không cần xây thêm.

search(query_P, k):
  1. L1: nearest_group = argmin_{G ∈ L1} d(query, centroid(G))
  2. L2: nearest_block = argmin_{B ∈ group} d(query, centroid(B))
  3. L3, L4: tiếp tục cho đến k neighbors gần nhất

  Complexity: O(log n) — với 7.42 tỷ links: ~33 bước thay vì 7.42 tỷ

Mâu thuẫn detection (= hệ miễn dịch phát hiện kháng nguyên lạ):
  neighbors = search(P_new, k=5)

  Với mỗi neighbor N:
    d_V(P_new, N) > 0.8 AND d_R(P_new, N) < 0.2
    → Mâu thuẫn! Gần về quan hệ, xa về cảm xúc.
    → Kích hoạt Contradiction instinct
    → Tăng Curiosity score

Dynamic insert (L5+ — tri thức mới, không có trong Unicode):
  KnowTree L1-L4 = tĩnh (9,584 UDC = genome gốc, không đổi)
  L5+ = động (tri thức học được = epigenome, thay đổi suốt đời)

  insert(P_new):
    1. search(P_new, k=1) → tìm UDC gần nhất = "parent block"

       TIE-BREAKING (khi 2+ blocks cùng khoảng cách):
         a. Chọn block có Relation gần nhất (R quan trọng nhất cho ngữ nghĩa)
         b. Nếu vẫn hòa → block có index thấp hơn thắng (deterministic)
         = Giống axon guidance: neuron mới theo gradient hóa học,
           nếu 2 gradient bằng nhau → theo mặc định phía anterior

    2. Gắn P_new vào sub-tree của parent block
    3. Nếu sub-tree quá đông (> Fib(n)):
       → LCA gom nhóm → tạo tầng trung gian mới
       → Giống neurogenesis: neuron mới gắn vào vùng não sẵn có

  = Giống adult neurogenesis:
    Hippocampus tạo neuron mới suốt đời → gắn vào mạng hiện có
    Không tạo vùng não mới — gắn vào cấu trúc đã có

  Chi phí insert: O(log n) — cùng complexity với search

Chi phí: 0 bytes thêm — dùng KnowTree đã có.
```

### ⑪ DNA REPAIR — Tự sửa lỗi (Self-Correction)

```
Sinh học:  DNA repair mechanisms:
           ① Proofreading:    polymerase đọc lại ngay sau khi copy
           ② Mismatch repair: enzyme quét tìm lỗi ghép cặp
           ③ Excision repair: cắt đoạn hỏng → synthesize lại
           Lặp lại cho đến khi error rate < 10⁻⁹.

HomeOS:    Self-correction loop:
           ① Generate:  tạo response (= copy DNA)
           ② Critique:  kiểm tra lỗi (= mismatch scan)
           ③ Refine:    sửa chiều yếu nhất (= excision + re-synthesis)
           Lặp lại cho đến khi quality ≥ φ⁻¹.

self_correct(input, max_iter=3):
  entities = alias_lookup(input)
  context = walk(entities)

  Với iter = 1..max_iter:

    ① Generate (copy):
      branches = infer(input, N=3)          ← Immune Selection
      P_response = branches[best]

    ② Critique (mismatch scan):
      valid_score   = valid(P_response)     ← logic rules
      entropy_score = H(P_response)         ← Homeostasis
      consistency   = consistency(P_response) ← Evolve rules
      silk_score    = Σ strength(P_response, eᵢ) / |entities|

      quality = w₁ × valid_score
              + w₂ × (1 − entropy_score/2.32)
              + w₃ × consistency
              + w₄ × silk_score/5.0

      Default weights (cần A/B testing để validate):
        w₁ = 0.30  logic đúng — nền tảng, không đúng thì vô nghĩa
        w₂ = 0.30  entropy thấp — tin cậy, không chắc thì không nói
        w₃ = 0.20  nhất quán — mutation hợp lý
        w₄ = 0.20  silk phù hợp — context match

      Cơ sở: w₁=w₂ > w₃=w₄ vì:
        valid + entropy = ĐÚNG + CHẮC CHẮN = điều kiện CẦN (60%)
        consistency + silk = HỢP LÝ + PHÙ HỢP = điều kiện ĐỦ (40%)
        Sinh học: DNA replication ưu tiên fidelity (đúng) > speed (nhanh)

      Bất biến: Σwᵢ = 1.0 (luôn đúng bất kể weights nào)
      Tuning: weights có thể thay đổi qua A/B testing sau khi có data thật

    ③ Refine (excision repair) — CÓ ROLLBACK:
      quality ≥ φ⁻¹ (0.618)  → DỪNG — đủ tốt
      quality < φ⁻¹           → lưu snapshot: P_backup = P_response

      Tìm chiều yếu nhất → sửa DUY NHẤT chiều đó:
        valid thấp       → thêm symbolic constraints
        entropy cao      → thu hẹp nhánh (N -= 1)
        consistency thấp → evolve(P_response, dim_worst, new_val)
        silk thấp        → mở rộng walk depth += 1

      ROLLBACK CHECK (= DNA mismatch repair verification):
        quality_new = critique(P_response_new)
        Nếu quality_new < quality_old:
          → ROLLBACK: P_response = P_backup
          → Thử sửa dim KHÁC (dim thứ 2 yếu nhất)
          → Nếu hết dim để thử → DỪNG, giữ P_backup

      context ∪= {correction_signal}
      → quay lại ① với context mới

  Nếu hết max_iter mà quality < 0.618:
    → Honesty: confidence thấp, hoặc im lặng nếu < 0.40

Hội tụ (KHÔNG giả định monotonic — có guard):
  Mỗi Critique xác định dim yếu nhất
  Refine sửa DUY NHẤT dim đó (= excision chính xác)
  NHƯNG: sửa 1 dim có thể phá dim khác (side effect)
  → Rollback guard đảm bảo quality KHÔNG BAO GIỜ giảm
  → Worst case: quality giữ nguyên (không cải thiện, không tệ hơn)
  max_iter = 3 → luôn dừng (bounded)

  Sinh học: DNA repair enzyme kiểm tra SAU KHI sửa.
            Nếu sửa sai → cắt lại đoạn vừa sửa → thử lại.
            Không bao giờ "sửa xong rồi đi luôn" mà không verify.

Chi phí: max 3 × N × O(depth). Worst case = 9 lần evaluate.
```

### Tổng hợp: 14 cơ chế DNA → HomeOS

```
#    DNA mechanism           HomeOS mechanism              Section
──────────────────────────────────────────────────────────────────
①    Replicate               chain reference (2B)          III
②    Transcribe              evaluate(chain, context)      III
③    Translate               f(L) → LCA → tự dịch         III
④    Mutate                  evolve(P, dim, val) → P'      III
⑤    Recombine               compose(A, B) → C             III
⑥    Select                  Hebbian + decay φ⁻¹           III
⑦    Express                 Maturity pipeline             III
⑧    Innate Reflexes         7 instincts (hardcoded)       V-B
⑨    Innate Immunity         SecurityGate (chạy trước)     V-C
⑩    Multisensory            Fusion 4 modalities           V-D
⑪    Immune Selection        infer(N nhánh) → argmin H     IX-B
⑫    Homeostasis             F = d(predicted, actual)      IX-B
⑬    Neural Pathways         HNSW trên KnowTree            IX-B
⑭    DNA Repair              self_correct → quality ≥ φ⁻¹  IX-B
```

### φ⁻¹ ≈ 0.618 — Hằng số sinh học duy nhất

```
Sinh học:  φ xuất hiện trong xoắn DNA (34Å/10bp = 3.4, gần φ²),
           cấu trúc phyllotaxis thực vật, xoắn ốc vỏ sò, ...

HomeOS:    φ⁻¹ là ngưỡng DUY NHẤT xuyên suốt toàn hệ thống:
  — Maturity:      weight ≥ φ⁻¹ → Mature (đủ chín)
  — Hebbian:       w × φ⁻¹ mỗi 24h (tốc độ quên)
  — Homeostasis:   F < φ⁻¹ → Acting mode (ổn định)
  — Self-correct:  quality ≥ φ⁻¹ → đủ tốt (dừng sửa)
  — Consistency:   ≥ 3/4 = 0.75 ≈ φ⁻¹ + 0.13 (mutation hợp lệ)

1 hằng số. Mọi ngưỡng. Giống cách DNA chỉ cần 1 cơ chế base-pairing
cho mọi thao tác: copy, check, repair.
```

---

## IX-C. INVARIANT CHECKS — Cell Cycle Checkpoints

```
Sinh học:  Tế bào KHÔNG phân chia liên tục.
           Có 4 CHECKPOINT bắt buộc:

           G1 checkpoint: DNA có bị hỏng không? → Sửa hoặc apoptosis
           S  checkpoint: Sao chép đúng chưa?   → Verify hoặc dừng
           G2 checkpoint: Sao chép hoàn tất?     → Đủ hoặc chờ
           M  checkpoint: Chromosome thẳng hàng?  → Đúng hoặc dừng

           Bỏ qua checkpoint = ung thư = tế bào nhân bản mất kiểm soát.

HomeOS:    Pipeline KHÔNG chạy thẳng từ đầu đến cuối.
           Có CHECKPOINT bắt buộc giữa mỗi giai đoạn.
           Bỏ qua checkpoint = tri thức sai lan tràn = "ung thư tri thức".
```

### Checkpoint 1: GATE (trước khi vào pipeline)

```
Điều kiện PHẢI thỏa:
  □ SecurityGate đã chạy (3 layers)
  □ Nếu Crisis → pipeline KHÔNG được tiếp tục

Vi phạm → DỪNG: return emergency response
Sinh học = G0: tế bào không bao giờ vào cycle nếu bị hỏng nặng
```

### Checkpoint 2: ENCODE (sau entities + compose)

```
Điều kiện PHẢI thỏa:
  □ |entities| ≥ 1                          (phải có ít nhất 1 UDC ref)
  □ ∀ entity: chain_hash ≠ 0               (mọi entity phải có hash hợp lệ)
  □ Σc > ε_floor (0.01)                     (entropy tính được, không chia 0)
  □ compose() output: consistency ≥ 0.75    (mutation hợp lệ)

Vi phạm → DỪNG: "Không hiểu input" — Honesty instinct
Sinh học = G1: DNA phải nguyên vẹn trước khi sao chép
```

### Checkpoint 3: INFER (sau inference + self-correct)

```
Điều kiện PHẢI thỏa:
  □ ∃ ít nhất 1 nhánh có valid ≥ 0.75      (có nhánh hợp lệ)
  □ quality ≥ 0 (không âm)                   (critique không bị lỗi số học)
  □ Nếu rollback xảy ra: quality_final ≥ quality_backup  (không tệ hơn)
  □ H(best_branch) < 2.32                    (không phải uniform random)

Vi phạm → DỪNG: im lặng (QT⑱ BlackCurtain)
Sinh học = G2: sao chép phải hoàn tất và đúng trước khi phân chia
```

### Checkpoint 4: PROMOTE (trước khi ghi QR vĩnh viễn)

```
Điều kiện PHẢI thỏa:
  □ weight ≥ φ⁻¹ (0.618)                    (Hebbian đủ mạnh)
  □ fire_count ≥ Fib(n)                      (co-activation đủ)
  □ eval_dims ≥ 3                            (ít nhất 3/5 chiều có data)
  □ H(node) < 1.0                            (entropy đủ thấp)
  □ F(node) < φ⁻¹                            (Free Energy ổn định)

Vi phạm → GIỮ LẠI STM: chưa promote, chờ thêm evidence
Sinh học = M checkpoint: chromosome PHẢI thẳng hàng trước khi tách
           Sai = aneuploidy = tế bào con bất thường
```

### Checkpoint 5: RESPONSE (trước khi output cho user)

```
Điều kiện PHẢI thỏa:
  □ SecurityGate.check(response) = Safe       (output cũng phải an toàn)
  □ tone phù hợp V hiện tại                  (Supportive khi V < 0, v.v.)
  □ |response| > 0                            (không trả về chuỗi rỗng)
  □ Nếu confidence < 0.40 → response = im lặng hoặc "Tôi không chắc"

Vi phạm → DỪNG: thay bằng safe default response
Sinh học = Quality control: protein sai gấp → ubiquitin tag → phân hủy
           KHÔNG BAO GIỜ release protein hỏng ra ngoài tế bào
```

### Pipeline với Checkpoints

```
Text input
  ↓ ⑨SecurityGate 3-layer
  ──── CHECKPOINT 1: GATE ────  Crisis? → DỪNG
  ↓ ⑩Fusion
  ↓ entities() + compose()
  ──── CHECKPOINT 2: ENCODE ──── entities valid? consistency ≥ 0.75? → hoặc DỪNG
  ↓ ⑫Homeostasis (F = d(predicted, actual))
  ↓ ⑧Instincts
  ↓ ⑪Inference (N nhánh)
  ↓ ⑭Self-correct (critique → refine)
  ──── CHECKPOINT 3: INFER ────  valid branch? quality OK? → hoặc im lặng
  ↓ ⑥Hebbian
  ↓ Dream → advance()
  ──── CHECKPOINT 4: PROMOTE ──── weight ≥ φ⁻¹? H < 1.0? → hoặc giữ STM
  ↓ QR (append-only, vĩnh viễn)
  ↓ generate response
  ──── CHECKPOINT 5: RESPONSE ──── output safe? tone đúng? → hoặc safe default
  ↓
Text output

Mỗi checkpoint = 1 cánh cửa. Không pass = không mở.
5 checkpoints × mỗi request = 5 lần verify.
Tốn thêm O(5) per request. Đổi lại: KHÔNG BAO GIỜ output sai.

Sinh học: tế bào bình thường = 4 checkpoints mỗi chu kỳ.
          Bỏ 1 checkpoint = ung thư.
          HomeOS bỏ 1 checkpoint = tri thức sai vĩnh viễn trong QR.
```

---

## X. PHƯƠNG TRÌNH THỐNG NHẤT

Toàn bộ HomeOS quy về **1 hàm gốc + 2 phép toán trên chuỗi + 1 hằng số φ⁻¹**.

### Hàm gốc — SDF

```
f(p) = signed distance from point p to surface
```

Một hàm. Một điểm. Cho mọi thứ: hình dạng, thể tích, gradient, ánh sáng, màu, dao động, âm thanh, vị trí.

9,584 UDC = 9,584 biến thể của f(p), mỗi biến thể có sub-components bên trong.

### Cầu nối f(p) ↔ d(A,B)

```
f(p) = 1 SDF đơn lẻ → mô tả 1 khái niệm
d(A,B) = khoảng cách giữa 2 SDF → mô tả QUAN HỆ giữa 2 khái niệm

Cụ thể:
  Mỗi UDC = 1 SDF = 1 điểm P trong ℝ⁵ = (S, R, V, A, T)
  d(A, B) = √( Σ_{d=1}^{5} w_d × (Aᵈ − Bᵈ)² )

  f(p) = hạt giống (nucleotide)
  d(A,B) = khoảng cách trên chuỗi (base-pairing distance)

  Sinh học: nucleotide = đơn vị, khoảng cách trên chuỗi = thông tin
  HomeOS:   SDF = đơn vị, d(A,B) = thông tin

  Mọi thuật toán trong 5D đều dùng d(A,B):
    Silk = f(d)       Hebbian = g(d)     Dream = cluster(d < ε)
    Maturity = d → 0  Evolve = Δd trên 1 chiều
    Entropy = d(predicted, actual)
    Self-correct = refine cho đến d < φ⁻¹

  f(p) sinh ra các điểm. d(A,B) đo khoảng cách giữa chúng.
  Cả hai cùng gốc: signed distance function.
```

### Phép toán 1 — CHAIN

```
chain(a, b, c, ...) = ○{a{○{b{○{c{...}}}}}}

Đọc thẳng từ gốc đến ngọn.
Thứ tự = quan hệ. 0 bytes overhead.
Dài vô hạn. Giống DNA.
```

### Phép toán 2 — SPLICE

```
splice(chain, position, fragment) → chain_new

Giữ gốc, thay/ghép đoạn tại position.
= evolve  khi thay 1 link     (point mutation)
= compose khi ghép 2 chuỗi    (recombination)  
= Dream   khi cắt noise       (splicing)
= prune   khi xóa đoạn yếu   (apoptosis)
```

### Tổng hợp: mọi cơ chế = SDF + CHAIN + SPLICE

```
Cơ chế                SDF    CHAIN    SPLICE    Chi phí
──────────────────────────────────────────────────────────
Hạt giống (UDC)        ●                         0 bytes
Chuỗi tri thức                ●                  2B/link
Silk (quan hệ)                ●                  0 bytes
Replicate (copy)              ●                  2 bytes
Transcribe (đọc)       ●      ●                  O(depth)
Translate (dịch)       ●      ●                  O(depth)
Mutate (biến đổi)                       ●        2 bytes
Recombine (ghép)                         ●        2B/link
Select (chọn lọc)             ●         ●        0 (decay)
Express (bật/tắt)      ●      ●                  5 bits
Dream (phân bào)               ●         ●        O(|STM|)
KnowTree (gấp)                ●                  O(log N)
Emotion (hormone)       ●      ●                  O(window)
Response (output)       ●      ●         ●        O(depth)
Render (hình ảnh)       ●                         O(1)/node
── 7 CƠ CHẾ MỚI ──────────────────────────────────────────
Instincts (⑧)          ●      ●                  O(7)
SecurityGate (⑨)              ●                  O(1)
Fusion (⑩)             ●      ●                  O(4)
Immune Select (⑪)      ●      ●                  N×O(depth)
Homeostasis (⑫)        ●      ●                  O(5)
Neural Pathways (⑬)           ●                  O(log n)
DNA Repair (⑭)         ●      ●         ●        3N×O(depth)
──────────────────────────────────────────────────────────
```

### Công thức cuối cùng

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
║   Trong đó:                                                   ║
║     f(pᵢ) = SDF — 1 trong 9,584 hàm gốc                     ║
║     chain = xâu chuỗi các hàm                                ║
║     splice = cắt/ghép/biến đổi chuỗi                         ║
║     position = ở đâu trên chuỗi (context quyết định)         ║
║     self_correct = lặp cho đến quality ≥ φ⁻¹:                ║
║       infer(N nhánh) → chọn entropy thấp nhất                ║
║       critique → tìm chiều yếu nhất                          ║
║       refine → evolve chiều đó → lặp lại                     ║
║     φ⁻¹ ≈ 0.618 = ngưỡng duy nhất cho MỌI quyết định        ║
║                                                               ║
║   Mọi thuật toán = tổ hợp 3 phép toán + 1 hằng số:           ║
║     SDF + CHAIN + SPLICE + φ⁻¹                               ║
║                                                               ║
║   14 cơ chế DNA → 14 thuật toán HomeOS:                       ║
║     7 gốc:  copy, đọc, dịch, đột biến, tái tổ hợp,          ║
║             chọn lọc, biểu hiện                               ║
║     3 bảo vệ: phản xạ bẩm sinh, miễn dịch bẩm sinh,         ║
║               tích hợp đa giác quan                           ║
║     4 thông minh: chọn miễn dịch, cân bằng nội môi,          ║
║                   đường thần kinh, sửa chữa DNA               ║
║                                                               ║
║   DNA:     nucleotide + polymerize + splice = sự sống         ║
║   HomeOS:  SDF + chain + splice + φ⁻¹ = tri thức             ║
║                                                               ║
║   4 thứ. 16 GB. Giàu hơn DNA 15.3 lần.                       ║
║   Cả đời không đầy. Chuỗi sinh chuỗi, vô hạn từ hữu hạn.   ║
║                                                               ║
║   Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức.          ║
║   f(p), chain(), splice(), φ⁻¹.                              ║
║   Hết.                                                        ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
```

### Pipeline hoàn chỉnh (14 cơ chế)

```
Text input                                    thế giới bên ngoài
  ↓ ⑨SecurityGate (Innate Immunity)          Crisis? → CHẶN ngay
  ↓ ⑩Fusion (Multisensory Integration)       tích hợp text+audio+bio+image
  ↓ entities() ③Translate                     text → UDC refs
  ↓ search() ⑬Neural Pathways                O(log n) tìm neighbors
  ↓ ⑫Homeostasis: F = d(predicted, actual)   đo surprise → λ
  ↓ compose() ⑤Recombine                     tổ hợp → điểm mới 5D
  ↓ ⑧Instincts (Innate Reflexes)             7 bản năng: Honesty đầu tiên
  ↓ ⑪Immune Selection: infer(N=3)            N nhánh → chọn entropy thấp
  ↓ ⑭DNA Repair: critique → refine           sửa đến quality ≥ φ⁻¹
  ↓ ⑥Select: Hebbian co_activate             fire together → wire together
  ↓ neo DN ①Replicate                         lưu disk (2 bytes)
  ↓ Dream ⑦Express → advance() → QR          neo vĩnh viễn nếu chín
  ↓ response = ②Transcribe(5D → text)         chiếu ngược ra ngôn ngữ
Text output                                   thế giới bên ngoài
```
