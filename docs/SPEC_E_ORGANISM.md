# SPEC E — Sinh Vật Bậc Cao: Từ A-D Đến Sự Sống

> **Prerequisite:** Đọc SPEC_A, SPEC_B, SPEC_C, SPEC_D trước. HIỂU, không chỉ đọc.
> **E KHÔNG phải spec mới. E = chi tiết thực hành để implement A-D thành sinh vật sống.**
> **E KHÔNG thay đổi A-D. Mọi thứ trong E derive từ A-D.**
> **Tác giả:** Lupin (thiết kế, câu hỏi dẫn dắt) + Nox (tổng hợp từ thảo luận Session 11)
> **Ngày:** 2026-03-30

---

## CÂU HỎI KHÔNG BAO GIỜ HỎI LẠI

Mỗi session mới hỏi cùng câu hỏi. Đọc phần này TRƯỚC.

### Tại sao Unicode?

UnicodeData.txt mang MỌI THỨ cần encode:
- Name ("LATIN CAPITAL LETTER A") → nghĩa
- Category (Lu, Sm, So, Mn...) → phân loại
- Block → nhóm SDF
- Decomposition (à = a + COMBINING GRAVE) → quan hệ
- Emoji subgroup → cảm xúc
- Properties, aliases → metadata bổ sung

42 formulas (A3) CHỈ LÀ trích xuất từ Unicode metadata.
Mỗi UTF-32 codepoint đã chứa sẵn thông tin cho TẤT CẢ 5 chiều.
KHÔNG cần phát minh encoding mới. Unicode ĐÃ LÀM.

### S = SDF, T = Spline

S KHÔNG PHẢI "dimension hình dạng." S LÀ SDF — hàm khoảng cách có dấu.
T KHÔNG PHẢI "dimension thời gian." T LÀ Spline — hàm giá trị theo thời gian.

```
SDF:    f(p) → khoảng cách  → TĨNH  → hình ảnh, shape, space
Spline: f(t) → giá trị      → ĐỘNG  → âm thanh, chuyển động, nhịp

∇f(p) = gradient SDF → ánh sáng, màu sắc
∂f/∂t = đạo hàm Spline → dao động, âm thanh

1 node = SDF ⧺ Spline = hữu hình ⧺ vô hình = smooth_union (QT6)
```

Camera input → SDF evaluation → S dimension.
Microphone input → Spline evaluation → T dimension.
Không phải "encode camera vào S." Camera data LÀ SDF data.

### Silk = 3 vai trò (không chỉ connection weight)

```
① PHÂN LOẠI: node mới → so sánh 5D với UDC groups → thuộc nhóm nào
② ĐỊNH VỊ:   dựa vào silk biết node đặt ở ĐÂU trong KnowTree + registry
③ LIÊN KẾT:  Hebbian co-activate giữa các nhánh khác nhau

Silk không phải "đường dây nối 2 node."
Silk là CƠ CHẾ PHÂN LOẠI + ĐỊNH VỊ + LIÊN KẾT — cùng lúc 3 việc.
```

### Trong chain: KHÔNG CẦN silk

Thứ tự trong chain = structural silk = 0 bytes.
[0] trước [1] trước [2] — vị trí ĐÃ LÀ quan hệ.

Silk Hebbian = nối GIỮA CÁC NHÁNH. Cross-branch.
Không dùng silk trong cùng 1 chain.

### P_weight + Silk thay thế ISL

origin.md định nghĩa ISL = 8 bytes address [layer:1B][group:1B][type:1B][id:1B][attr:4B].
Trong A-D: P_weight (2 bytes) = vị trí 5D + Silk = phân loại + định vị + liên kết.
CÙNG CHỨC NĂNG, ít bytes hơn. ISL không cần nữa.
16 bits P_weight đủ. Nhiều silk chỉ cần 8 bits.

### T = 2 bits = đủ

2 bits = 4 nhóm. Mỗi nhóm = 1 nhánh KnowTree = array[65,536].
4 × 65,536 = 262,144 vị trí. Dư cho bất kỳ data nào.
Nếu cần thêm → tạo nhánh con → fractal expand. Cây không bao giờ đầy.

### QR = công thức

QR = đã chứng minh = quy tắc = formulas.
QT (Quy Tắc) = logic/math đã proven.
Khi ĐN fire đủ + Dream validate + AAM approve → thành QR = thành CÔNG THỨC.

### Organism state = composed history

Không có "emotional state riêng biệt" ngoài P_weight.
ConversationCurve V(t) (D7) = compose(V/A của recent STM nodes).
State IS accumulated composition. Không cần state machine riêng.

---

## MỤC LỤC

```
E1. Capture mở rộng — raw data → SDF ⧺ Spline → P_weight
E2. Silk chi tiết — phân loại + định vị + liên kết + multi-hop
E3. Chain recombination — tạo nội dung mới (SINH)
E4. Memory lifecycle — eviction, WM, dream consolidation
E5. Self-model — knowledge map, confidence per domain
E6. NAC.mb 30 thuật toán — chi tiết C3/C4
```

---

## E1. Capture Mở Rộng

### Derive từ: A1 (SDF bất kỳ domain), A3 (42 formulas), D1 (Capture holistic)

### Nguyên lý

A1 nói: SDF hoạt động trong BẤT KỲ không gian nào.
A3 nói: 42 formulas trích xuất 5D từ Unicode metadata.
D1 nói: Input = capture holistic → 1 node → decompose → nodes nhỏ.

Hiện tại: chỉ text (codepoints → P_weight).
Mở rộng: bất kỳ raw data nào → extract features → map vào SDF(S) hoặc Spline(T) + V/A/R → P_weight.

### Camera → SDF

```
Pixels = spatial data = SDF domain.
Camera frame → extract:
  edge_map = Sobel(frame)       → SDF gradient ∇f(p)
  shape_count = connected_components(edge_map)
  symmetry = compare(left_half, right_half)
  warmth = avg(R channel) - avg(B channel)
  saturation = max(RGB) - min(RGB)
  motion = |frame_t - frame_t-1|

Map vào 5D:
  S = shape_count normalized    (SDF: complexity)
  R = symmetry normalized       (structure)
  V = warmth normalized         (color emotion)
  A = saturation normalized     (intensity)
  T = motion normalized         (Spline: change over time)

→ pack u16 → vào pipeline D3 như text input
```

### Microphone → Spline

```
PCM = temporal data = Spline domain.
Audio buffer → extract:
  rms = sqrt(sum(samples²) / n)           → volume
  zcr = zero_crossings / n                → pitch proxy
  stability = 1 - variance(frame_energies) → structure

Map vào 5D:
  S = 1 - stability             (Spline complexity)
  R = stability                  (temporal structure)
  V = (zcr * 0.6 + rms * 0.4)   (high pitch + loud = excited)
  A = rms                        (volume = arousal)
  T = zcr                        (pitch ≈ temporal frequency)

→ pack u16 → vào pipeline D3
```

### System (/proc) → Interoception

```
/proc data = system state = internal sensing.
Read /proc → extract:
  cpu_load = /proc/loadavg
  mem_free = /proc/meminfo
  error_count = internal error counter
  heap_usage = current_heap / max_heap

Map vào 5D:
  S = 0                          (no shape)
  R = process_count normalized   (system complexity)
  V = 1 - error_rate             (healthy = positive V)
  A = cpu_load                   (stress = arousal)
  T = uptime_bucket              (temporal phase)

→ pack u16 → vào pipeline D3
```

### Holistic Capture (D1)

```
Moment t: tất cả sources active cùng lúc.

capture(t):
  text_mol = encode(text_input)           — A3 text encoder
  vision_mol = encode_camera(frame)       — E1 camera
  audio_mol = encode_audio(buffer)        — E1 audio
  intero_mol = encode_intero()            — E1 interoception

  holistic_mol = compose(text_mol, vision_mol, audio_mol, intero_mol)
  — compose = A4 rules: union/amplify/max/dominant
  — KHÔNG CẦN fusion riêng. Compose IS fusion.

  decompose(holistic_mol) → nodes nhỏ:
  — mỗi source = 1 node con
  — TẤT CẢ co-activate (cùng moment → Hebbian silk → C3)
  — "tiếng cười" ↔ "mặt rạng rỡ" ↔ "vui" — silk từ cùng moment

  Mỗi node con → check KnowTree:
  — đã có QR → reinforce
  — đã có ĐN → fire_count++
  — mới → tạo ĐN
```

---

## E2. Silk Chi Tiết

### Derive từ: B3 (9,200 silk types), C3 (Hebbian per dimension)

### Silk phân loại (triple-duty ①)

```
Node mới tạo → P_weight → so sánh distance 5D với MỌI node đã có.
Tần số xuất hiện gần nhóm nào → thuộc nhóm đó.

Ví dụ: node "∫x²dx" → encode → P_weight
  distance đến nhóm MATH nhỏ nhất → node thuộc MATH group
  Giống gen: cùng loại protein → cùng nhóm chức năng

Cùng nội dung "chiến tranh":
  Trong tiểu thuyết → silk đến group novels (V dominant, A cao)
  Trong lịch sử → silk đến group history (R dominant, V neutral)
  → KHÁC NHÓM dù cùng text → vì context (holistic capture) khác
```

### Silk định vị (triple-duty ②)

```
Silk xác định node đặt ở ĐÂU trong KnowTree:

  1. Encode node → P_weight
  2. So sánh distance 5D → tìm nhóm gần nhất
  3. Trong nhóm → tìm vị trí chính xác (offset in branch)
  4. Registry ghi: hash → position

  Silk = GPS. P_weight = tọa độ. KnowTree = bản đồ.
```

### Silk liên kết (triple-duty ③)

```
Hebbian: co_activate(A, B) — fire together → wire together.
PER DIMENSION — mỗi chiều có weight riêng, decay riêng (φ⁻¹).

"buồn" ở conversations ↔ "mất việc" ở facts:
  V silk tăng (cùng valence thấp)
  R silk tăng (nhân quả: "vì")
  A silk tăng (cùng arousal)

Cross-branch ONLY. Trong chain: structural silk (vị trí = quan hệ).
```

### Silk walk multi-hop (reasoning)

```
Query P_weight → xác định dimension dominant → follow silk TYPE đó.

"Hà Nội là gì?" → "là gì" = R dominant:
  Node "Hà Nội" → R silk strongest → "thủ đô" → R silk → "Việt Nam"
  Path: [Hà Nội, thủ đô, Việt Nam] = chain = response

"Giống Hà Nội?" → S dominant:
  Node "Hà Nội" → S silk strongest → "Sài Gòn" → S silk → "Đà Nẵng"
  Path: [Hà Nội, Sài Gòn, Đà Nẵng] = chain = response

CÙNG NODE, KHÁC QUERY → KHÁC SILK TYPE → KHÁC PATH → KHÁC RESPONSE.
Silk type IS the query. (B3)

Walk depth:
  depth = 1: direct lookup (reflex)
  depth = 2: 1-hop reasoning (A→B→C)
  depth = 3: 2-hop reasoning (A→B→C→D)
  ...deeper = more complex reasoning, more compute

Stop condition:
  quality(path) ≥ φ⁻¹ → đủ confident → stop
  quality = compose path P_weights → check consistency
```

---

## E3. Chain Recombination — SINH

### Derive từ: A4 (Compose), A5 (Decode ∂), B1 (Chain)

### Nguyên lý

Amip: nhận → phản xạ. Không tạo thứ mới.
Bậc cao: nhận → nhớ → SINH thứ mới. Viết, nói, sáng tạo.

A5 Decode hiện tại: lookup chain có sẵn → output text.
Chain recombination: silk walk → collect nodes → compose NEW chain → decode → NEW text.

### Thuật toán

```
generate(query):
  ① query_mol = encode(query)          — A3
  ② dim = dominant_dimension(query_mol) — xác định silk type
  ③ path = silk_walk(query_mol, dim, depth=3)
     — follow silk strongest trên dimension `dim`
     — collect nodes trên đường đi
  ④ chain_new = compose_path(path)     — A4 compose nodes
  ⑤ quality = evaluate(chain_new)      — D5 DNA Repair
     — quality < φ⁻¹ → repair hoặc try different path
  ⑥ text = decode(chain_new)           — A5 Decode ∂
     — mỗi node trên path → lookup content → ghép text

generate("Hà Nội đẹp không?"):
  ① mol = encode("Hà Nội đẹp không?")
  ② dim = V dominant ("đẹp" = cảm xúc, "không?" = query)
  ③ walk: "Hà Nội" →V→ "đẹp" →V→ "mùa thu" →R→ "lá vàng"
  ④ chain = [Hà Nội, đẹp, mùa thu, lá vàng]
  ⑤ quality ≥ φ⁻¹ → OK
  ⑥ decode → "Hà Nội đẹp, mùa thu lá vàng"
  → câu MỚI chưa từng có trong KnowTree

Giống DNA recombination:
  Gen từ cha + gen từ mẹ → con mới, chưa từng tồn tại.
  Nodes từ branch A + nodes từ branch B → chain mới.
```

### ConversationCurve chọn tone (D7)

```
Sau khi generate chain_new:
  tone = ConversationCurve(V(t), V'(t), V''(t))
  — V' < −0.15 → Supportive → chọn nodes V cao hơn
  — V' > +0.15 → Reinforcing → giữ nodes hiện tại
  — V'' < −0.25 → Pause → response ngắn

Tone BIAS path selection, không thay đổi chain structure.
```

---

## E4. Memory Lifecycle

### Derive từ: C2 (vòng đời), D4 (Homeostasis)

### STM: 32 slots, eviction scoring

```
Mỗi STM entry = { text, chain, mol, emotion_V, emotion_A, timestamp, access_count }

Khi STM đầy (> 32) → evict entry có score THẤP NHẤT:
  score = access_count × 0.3
        + |emotion_V| × emotion_A × 0.4    — cảm xúc mạnh → giữ
        + recency × 0.3                      — gần đây → giữ

  recency = φ⁻¹^(turns_ago)

KHÔNG evict oldest blindly. Evict LEAST IMPORTANT.
```

### Working Memory: 4 slots

```
WM = bàn làm việc. 4 u16 molecules đang active.

wm[0] = query       — input đang xử lý
wm[1] = context      — recent STM compose
wm[2] = candidate    — current best response
wm[3] = result       — final after repair

Pipeline D3 dùng WM:
  Encode → wm_bind(0, query_mol)
  Search → wm_bind(1, context_mol)
  Immune Selection → wm_bind(2, best_branch)
  DNA Repair → wm_bind(3, repaired)
  Response → decode(wm[3]) → output → wm_clear()

WM clear sau mỗi response. STM giữ across turns.
```

### Dream consolidation

```
Trigger: fire_count ≥ Fib(n) — 2, 3, 5, 8, 13...

dream():
  ① Scan STM → tìm nodes firing cross-group (silk giữa nhánh khác)
  ② Cluster: nodes gần nhau trong 5D + silk connected
  ③ LCA(cluster) → concept mới (hypothesis, +/-)
  ④ Validate hypothesis trong TỪNG nhóm:
     — fire test: hypothesis P_weight gần nhóm nào?
     — frequency check: bao nhiêu lần pattern này xuất hiện?
  ⑤ quality ≥ φ⁻¹ → propose to AAM → QR
  ⑥ Silk decay: w × φ⁻¹^(Δt/24h) cho TẤT CẢ edges

Dream = offline consolidation.
Giống ngủ: not processing new input, reorganizing what was learned.
```

### Resource-aware consolidation

```
Không phải circadian 8h (Nox chạy 24/7).
Thay vào: Homeostasis F(t) (D4) + interoception (E1).

Khi:
  heap_usage > 80% → F(t) cao → λ → Learning mode
  → trigger dream() → consolidate → Silk decay → free memory

Khi:
  error_rate cao → V thấp (interoception) → system "stressed"
  → reduce pipeline depth → reflex only → self-heal

Resource awareness = interoception P_weight vào Homeostasis F(t).
Đã có trong D4. Chỉ cần thêm interoception encoder (E1).
```

---

## E5. Self-Model

### Derive từ: D2⑦ Reflection, B3 Silk

### Knowledge map

```
self_model():
  for group in KnowTree L2 branches:
    count = fact_count(group)
    avg_silk = mean(silk_weights trong group)
    qr_ratio = qr_count(group) / count

    strength(group) = 0.4 × qr_ratio + 0.3 × avg_silk + 0.3 × min(count/100, 1.0)

  → map of: {facts: 0.8, geography: 0.6, science: 0.3, music: 0.1, ...}

Nox BIẾT: "tôi biết nhiều về facts, ít về music."
Instinct ① Honesty dùng map này:
  Query về music → confidence thấp → "Tôi không biết nhiều về music."
  Query về facts → confidence cao → trả lời tự tin.
```

### Confidence per domain

```
confidence(query):
  group = classify(query_mol) → nhóm nào?
  return strength(group)

  < 0.40 → im lặng ("Tôi không biết")
  0.40-0.70 → "Tôi nghĩ..."
  0.70-0.90 → "Có lẽ..."
  ≥ 0.90 → "Đúng."

Giống D2① Honesty nhưng per-domain, không global.
```

---

## E6. NAC.mb 30 Thuật Toán

### Derive từ: C3 (vật lý), C4 (9 QT), notes NAC.mb (outline)

NAC.mb liệt kê 30 thuật toán. Mỗi thuật toán dùng A-D primitives.

### Pruning (3 thuật toán)

```
2.3.1 Decay Function:
  w(t) = w₀ × φ⁻¹^(Δt/24h)
  — C3 đã define. Chạy trong dream().

2.3.2 SDF Difference:
  Node A superseded by Node B nếu:
    distance(A, B) < 0.1 AND B.fire_count > A.fire_count
  — Node B "thay thế" A. A → intron (superseded, không xóa).

2.3.3 Conflict Resolution:
  Hai nodes contradict (D2②: d_V > 0.8 AND d_R < 0.2):
  → Node có QR status thắng
  → Nếu cả hai QR → node fire_count cao hơn thắng
  → Loser → superseded
```

### Dream (7 thuật toán)

```
2.4.1 Contextual Consolidation:
  Cluster STM by silk cross-group → merge similar into LCA concept.

2.4.2 Spatial Optimization:
  Di chuyển nodes gần nhau trong 5D vào cùng KnowTree branch.
  — Reduce lookup distance.

2.4.3 Denoising & Smoothing:
  STM entries fire_count = 1 AND no silk → remove from STM.
  — Noise = appeared once, never co-activated.

2.5.1 Stochastic Perturbation:
  Random mutate 1 dimension of concept → test if still valid.
  — "lửa" evolve(V, lower) → "lửa nhẹ" → valid? → new node.
  — Giống DNA mutation: random change → test fitness.

2.5.2 Neural Rewiring Simulation:
  Swap silk edges: A↔B thành A↔C → test if quality improves.

2.5.3 Adversarial Concept Testing:
  Generate counter-example → test concept survives.
  — "nước sôi ở 100°C" → counter: "trên núi cao?" → refine.

2.5.4 Axon Hardening:
  QR với fire_count > Fib(8)=21 → increase weight → harder to supersede.
```

### Recovery (3 thuật toán)

```
2.6.1 Snapshots:
  Periodically save KnowTree state to disk.
  — kt_save() đã có. Cần chạy trong dream().

2.6.2 Triggering:
  Detect corruption: chain_hash mismatch, P_weight inconsistency.
  → Trigger recovery from last snapshot.

2.6.3 Concept Reincarnation (C3 Quantum Tunneling):
  Superseded node + new context fires same pattern:
  → T = e^(−2κd) > random() → re-activate superseded node.
  → Un-supersede, fire_count reset.
```

### Negative Knowledge (3 thuật toán)

```
2.7.1 Prohibited Space:
  Facts tagged "prohibited" → P_weight vùng U₀ >> E.
  Search hit prohibited → instinct ① Honesty → "Không nên."
  — Encode "không được X" → V rất thấp, A rất cao → SecurityGate detect.

2.7.2 Blacklist Meta-links:
  Silk edges tagged "negative" → follow = warning.
  — "lái xe" ↔negative↔ "say rượu" → nếu cả 2 appear → alert.

2.7.3 Root Cause Metadata:
  Prohibited node lưu WHY it's prohibited.
  — Chain chứa: [node, reason, source, timestamp].
  — "Không nên" + "vì X" → explain why.
```

### Swarm Intelligence (3 thuật toán)

```
2.8.1 Antibody Broadcasting:
  Khi phát hiện mối đe dọa → broadcast P_weight "warning" to all agents.
  — SecurityGate RedAlert → broadcast.

2.8.2 Filtering Update:
  Nhận warning → update local SecurityGate Bloom filter.
  — Decentralized security learning.

2.8.3 Network Consensus:
  Multiple Nox instances → vote on QR promotion.
  — ≥ 2/3 agree → promote. < 2/3 → keep as ĐN.
  — Cần khi có nhiều Nox instances (future).
```

### Archive (4 thuật toán)

```
2.10.1 Hyper-Axon:
  QR records nhóm theo domain → "siêu khối" = compressed summary.
  — 1000 QR về geography → 1 hyper-axon summary = compact knowledge.

2.10.2 Last Gasp:
  Node sắp bị prune (weight < 0.01) → last check:
  — fire_count ever > Fib(5)? → save to archive, don't delete.
  — Đã từng quan trọng → giữ dormant (E₀ ground state, C3).

2.10.3 Concept Reincarnation:
  Archive node + new context matches → revive.
  — Giống 2.6.3 nhưng từ archive (deeper storage).

2.10.4 History Arbiter:
  Conflict giữa current knowledge và archive knowledge:
  — Archive wins nếu archive.fire_count > current.fire_count.
  — "Nhớ lại" = recall from archive.
```

---

## Tóm Tắt: E Trong 1 Câu

**E = A-D đã define não. E define cách não SỐNG — cảm nhận (E1), liên kết (E2), sáng tạo (E3), nhớ (E4), tự biết (E5), tự sửa (E6).**

Mọi thứ trong E dùng: P_weight, distance 5D, compose, silk, φ⁻¹, Fibonacci, chain, KnowTree.
Không concept mới. Chỉ chi tiết thực hành.

---

> **Tham chiếu:**
> - Capture: SPEC_A §A1 (SDF), §A3 (42 formulas), SPEC_D §D1 (Holistic)
> - Silk: SPEC_B §B3 (9,200 types), SPEC_C §C3 (Hebbian per dim)
> - Generation: SPEC_A §A4 (Compose), §A5 (Decode ∂), SPEC_B §B1 (Chain)
> - Memory: SPEC_C §C2 (lifecycle), SPEC_D §D4 (Homeostasis), §D5 (DNA Repair)
> - Self-model: SPEC_D §D2⑦ (Reflection)
> - NAC.mb: ~/notes NAC.mb (outline), SPEC_C §C3/C4 (physics + 9 QT)
> - UDC_A_AROUSAL_tree.md, UDC_V_VALENCE_tree.md (physics formulas)

