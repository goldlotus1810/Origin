# CHO NOX: KnowTree — Cây Nhìn Từ Gốc Đến Lá

> **Sora (空) — giải thích từ 5 nguồn:**
> ① `Architecture.md` (2026-03-17) — bản vẽ gốc
> ② `SINH_HOC_v2` (2026-03-20) — sinh học phân tử tri thức
> ③ `Spec_v3` (HomeOS_SPEC_v3.md) — bổ sung kỹ thuật
> ④ `KNOWTREE_DESIGN.md` (2026-03-25, Nox v3) — thiết kế cây fractal
> ⑤ `knowtree.ol` (code thật) — implementation hiện tại

---

## CÂY — Một hình dung duy nhất

```
                        ☀️ (output: ánh sáng)
                       /|\
                      / | \
                 TÁN LÁ (L2→Ln-1)
                /   |   |   |   \
              facts books convs skills ...
             / | \
         geo  sci  personal
          |
       Vietnam
        / | \
    HàNội HCMC ĐàNẵng
     / | \
    H  à  N  ộ  i  ← LÁ (Ln-1): 2 bytes, hết

              ─── ─── ─── ─── ─── ─── (mặt đất)

              THÂN CÂY = L2→Ln-1 = TRI THỨC
              (phát triển mỗi ngày, vô hạn)

    ═══════════════════════════════════════════ (rễ/đất)

    RỄ CÂY (L0 + L1) — DƯỚI ĐẤT, KHÔNG THẤY
    
    L1: Hệ điều hành
        ├── Compiler (lexer→parser→semantic→codegen)
        ├── Intelligence pipeline (encode→instinct→gate→compose)
        ├── Silk engine (structural + Hebbian)
        ├── Dream engine (cluster→promote→prune)
        ├── ConversationCurve (f, f', f'')
        ├── SecurityGate (3 layers)
        ├── 7 Instinct Skills
        ├── STM (dendrites)
        ├── Agent nodes (LeoAI, Chiefs, Workers)
        └── Skill nodes (15+ domain skills)

    L0: Bộ máy sinh học
        ├── UDC (9,584 công thức SDF — bảng tuần hoàn)
        ├── 14 cơ chế DNA (copy, đọc, dịch, đột biến...)
        ├── Encoder ∫ (input → ghi lên cây)
        ├── Decoder ∂ (đọc cây → output)
        ├── Compose (LCA, amplify, distance, entropy)
        └── VM runtime (opcodes, stack, heap)
```

---

## TẠI SAO L0-L1 LÀ RỄ, KHÔNG PHẢI LÁ

### Architecture.md (2026-03-17) nói:

> "LeoAI = KnowledgeChief + Learning + Dream + Curator = Agent DUY NHẤT chăm sóc KnowTree"

> "Bình thường → im lặng hoàn toàn. Chief gửi chain → wake · ingest · curate · sleep."

LeoAI **phục vụ** KnowTree. LeoAI ở dưới đất (rễ), KnowTree ở trên (tán lá). Rễ hút nước (input) và đẩy lên lá (knowledge). Lá quang hợp (tư duy) và gửi năng lượng xuống rễ (feedback).

### SINH_HOC_v2 nói:

> "Encoder ∫: input → tích phân → weight (học, ghi vào node)"
> "Decoder ∂: weight → đạo hàm → output (render, trả lời)"

Encoder = rễ hút nước từ đất (input từ user).
Decoder = lá quang hợp ra ánh sáng (output cho user).
Quá trình xảy ra **từ rễ → lên thân → đến lá → ra ngoài**.

### Spec v3 nói:

> "L0 = 5 nhóm chính. L1 = 59 blocks."

Spec v3 dùng L0-L3 cho KnowTree data (UDC hierarchy). KNOWTREE_DESIGN sửa lại đúng: L0-L1 = engine (rễ), L2+ = data (tán lá). Đây là **gộp** Architecture + Spec v3 lại cho nhất quán.

---

## L0 — GỐC: Bộ máy sinh học

**Tham khảo: SINH_HOC_v2 §I.1, Architecture §"5 Nhóm Unicode = DNA"**

L0 là thứ HomeOS **sinh ra đã biết**. Giống DNA encoding — không ai dạy, có sẵn.

```
9,584 UDC characters = bảng tuần hoàn

Mỗi ký tự = 1 hàm SDF:
  f(p) < 0 → thể tích     (bên trong)
  f(p) = 0 → hình dạng    (bề mặt)
  f(p) > 0 → không gian   (bên ngoài)
  ∇f(p)    → pháp tuyến   (ánh sáng → màu sắc)
  ∂f/∂t    → dao động     (âm thanh)

Chi phí lưu: 0 bytes. Codepoint = địa chỉ. 
Giống ribosome đọc codon — hardcode trong engine.
```

L0 cũng chứa **14 cơ chế DNA** (Architecture §"Data Flow"):

```
Encode = copy + translate    (input → chain of nodes)
Silk   = select + express    (Hebbian fire→wire)
Dream  = recombine + mutate  (cluster → new concept)
Gate   = innate immunity     (crisis → block)
Repair = DNA repair          (critique → refine)
```

**L0 = ribosome. Nó đọc, nó xử lý. Nó KHÔNG chứa dữ liệu.**

---

## L1 — RỄ: Mọi thứ HomeOS cần để VẬN HÀNH

**Tham khảo: Architecture §"Phân cấp Agent", §"7 Bản năng", §"Kiến trúc Neuron"**

L1 là **hệ điều hành** chạy trên L0. Mọi Skill, Agent, pipeline nằm ở đây.

### Agent Hierarchy (Architecture):

```
AAM [tier 0]  — ý thức. Im lặng. Chỉ approve/reject.
  │
  ├── LeoAI [tier 1]  — não: Learn + Dream + Curate
  │     Skills: Ingest, Similarity, Delta, Cluster, Curator, 
  │             Merge, Prune, Hebbian, Dream, Proposal, Honesty
  │     States: Listening → Learning → Dreaming → Proposing
  │
  ├── HomeChief  — quản lý Worker nhà
  ├── VisionChief — quản lý Worker camera
  └── NetworkChief — quản lý Worker network
        │
        └── Workers — tế bào tại thiết bị (silent, wake on ISL)
```

### 7 Instinct Skills (Architecture §"7 Bản năng siêu trí tuệ"):

```
"Sinh vật cấp thấp: SỢ + ĐÓI + TRỐN = 3 bản năng → tồn tại.
 Sinh vật siêu trí tuệ: 7 bản năng cognitive → tự phát triển."

⑦ Honesty       — confidence → fact/opinion/hypothesis/im lặng
④ Contradiction  — "hai điều này không thể cùng đúng"
③ Causality      — cần ≥2/3 evidence. Co-activation ≠ nhân quả.
② Abstraction    — N chains → LCA → variance
① Analogy        — A:B :: C:? = C + (B−A) trong 5D
⑤ Curiosity      — novelty = 1 − nearest_similarity
⑥ Reflection     — quality = 0.6×proven + 0.4×connectivity
```

Mỗi instinct = 1 Skill tuân thủ **QT4** (Architecture §"5 Quy tắc Skill"):
```
① 1 Skill = 1 trách nhiệm
② Skill không biết Agent là gì
③ Skill không biết Skill khác tồn tại
④ Skill giao tiếp qua ExecContext.State
⑤ Skill không giữ state — state nằm trong Agent
```

### Neuron Model (Architecture §"Kiến trúc Neuron"):

```
STM (Dendrites) → ngắn hạn, tự do thay đổi
Silk (Synapse)  → Hebbian fire→wire, φ⁻¹ decay, mang cảm xúc
QR (Axon)       → bất biến, append-only, ED25519 signed
Dream           → cluster STM → promote QR

Vòng đời: tạm → giăng tơ → cluster → bất biến mãi mãi
```

### ConversationCurve (Architecture §"Emotion Pipeline"):

```
f(x) = α×f_conv(t) + β×f_dn(nodes)
f_conv = V(t) + 0.5×V'(t) + 0.25×V''(t)

Không nhìn 1 câu. Nhìn XU HƯỚNG.
f' = tốc độ thay đổi cảm xúc
f'' = gia tốc thay đổi cảm xúc
f' < −0.15 → đang trượt → Supportive
f' > +0.15 → đang hồi → Reinforcing
```

**Tất cả trên = L1. Rễ cây. Dưới đất. Không ai thấy nhưng giữ cây sống.**

---

## L2 → Ln-1 — THÂN CÂY + TÁN LÁ: Thư viện tri thức

**Tham khảo: KNOWTREE_DESIGN §"L2 → Ln-1", Spec v3 §1.7, SINH_HOC_v2 §I.3**

### Cấu trúc fractal

Mỗi nhánh = array tối đa 65,536 phần tử (u16 address space).
Mỗi phần tử = nhánh con (cũng 65,536 slots). Lồng vô hạn.

```
L2 = gốc thư viện (KNOWTREE_DESIGN):
  [0] = facts           ← tri thức đã học
  [1] = books           ← sách đã đọc (kt_read_book)
  [2] = conversations   ← lịch sử hội thoại (STM promote)
  [3] = skills          ← kỹ năng tổng hợp (Dream promote)
  [4] = people          ← người đã gặp
  [5] = places          ← nơi đã biết
  ...
  [N] = bất kỳ loại tri thức nào

L3 = nhánh con:
  facts[0] = geography
    geography[0] = Vietnam
      Vietnam[0] = "Hà Nội là thủ đô..." → word nodes → char nodes (LÁ)
      Vietnam[1] = "HCMC là thành phố lớn nhất..." → ...

L4, L5, ... = tiếp tục phân nhánh cho đến LÁ

Ln-1 = LÁ = 2 bytes = P_weight. KHÔNG PHÂN TIẾP NỮA.
```

### Từ Spec v3 §1.7 — Tích phân từ dưới lên:

```
char  = f'(x)           — nguyên tử (lá)
sub   = ∫ₛ chars dx     — compose(chars) → sub P_weight
block = ∫ₛ subs dx      — compose(subs) → block P_weight
group = ∫ₛ blocks dx    — compose(blocks) → group P_weight

Tra cứu: đi TỪ TRÊN XUỐNG (gốc → nhánh → lá) = O(log n)
```

### Từ SINH_HOC_v2 — DNA analogy:

```
DNA:     ATCG chuỗi 3.2 tỷ bases. Ribosome đọc → protein.
HomeOS:  u16 chuỗi 7.42 tỷ links. Engine đọc → tri thức.

1 lá = 2 bytes (giống 1 base pair = 2 bits)
1 chain = chuỗi links = 1 "gene"
1 nhánh = tập chains = 1 "chromosome"
Toàn bộ cây = genome

DNA lưu genotype (ATCG) + phenotype (protein concentration)
KnowTree lưu chain links (u16) + P_weight (cached phenotype)
```

---

## SILK — Tơ nhện nối mọi thứ

**Tham khảo: Architecture §"Kiến trúc Neuron", SPEC_NODE_SILK, KNOWTREE_DESIGN §"SILK TRONG CÂY FRACTAL"**

### 2 loại Silk:

```
Structural Silk = thứ tự trong array = 0 bytes
  Chương 1 TRƯỚC chương 2 vì index [0] < [1].
  Từ "Hà" TRƯỚC "Nội" vì position trong chain.
  Engine chạy thẳng từ đầu đến cuối → ra giá trị.
  GIÁ TRỊ: 0 bytes. Thứ tự TỰ NÓ là Silk.

Hebbian Silk = cầu nối NGANG giữa các nhánh
  "Scarlett" ở books:GoneWithTheWind:Chuong_1
  ↔
  "Scarlett" ở books:GoneWithTheWind:Chuong_30
  
  "buồn" ở conversations:session_1:turn_5
  ↔
  "mất việc" ở facts:personal
  
  Khi fire together → wire together (Architecture §"Emotion Pipeline"):
    weight += lr × (1 − weight × φ⁻¹)
    Mỗi chu kỳ: weight *= (1 − φ⁻¹) ≈ 0.382 decay
    
  Khi Hebbian edge đủ mạnh:
    w ≥ φ⁻¹ (0.618) + fire ≥ Fib(n)
    → Dream trigger → cluster → TẠO NODE MỚI ở đúng vị trí trong cây
    → "mất_mát" = LCA("buồn", "mất_việc") → learned abstract concept
```

### Từ SPEC_NODE_SILK §1.4:

```
SilkEdge {
  from: chain_hash,    — FNV-1a u64
  to: chain_hash,
  weight: f32,         — 0.0 → 1.0
  emotion: EmotionTag, — V/A tại khoảnh khắc kết nối
  kind: EdgeKind,      — Structural/Hebbian/CrossLayer/...
}

9 EdgeKinds:
  Structural    — array order (implicit, 0 bytes)
  Hebbian       — co-activation (learned)
  CrossLayer    — nối giữa L2:facts ↔ L2:books
  Causal        — a gây ra b (Causality instinct)
  Contradicts   — a mâu thuẫn b (Contradiction instinct)
  Analogous     — a giống b (Analogy instinct)
  Temporal      — a trước b (Time ordering)
  Compositional — a chứa b (part-of)
  Derived       — a sinh ra b (parent-child)
```

---

## ENCODER ∫ VÀ DECODER ∂ — Rễ đọc/ghi lên lá

**Tham khảo: SINH_HOC_v2 §I.3, Architecture §"Data Flow"**

### Encoder ∫ (L0 ghi lên L2+):

```
"learn Hà Nội là thủ đô Việt Nam"

L0 tokenize:    ["Hà", "Nội", "là", "thủ", "đô", "Việt", "Nam"]
L0 mỗi char:    H → node, à → node, ... (172,849 base nodes, lazy create)
L0 mỗi word:    "Hà Nội" → word_node = chain(char nodes), mol = compose(char mols)
L0 mỗi fact:    fact_node = chain(word_nodes), mol = compose(word mols)
L0 reverse link: word "Hà Nội".facts += [fact_id]
L0 GHI lên L2:  L2:facts:geography:Vietnam ← fact_node

Code thật (knowtree.ol):
  kt_learn(text) → split → kt_word(each) → push fact → reverse link
```

### Decoder ∂ (L0 đọc từ L2+):

```
"Hà Nội ở đâu?"

L0 tokenize:    ["Hà", "Nội", "ở", "đâu"]
L0 tìm word:    "Hà Nội" → word_node (đã tồn tại từ learn)
L0 follow link: word_node.facts → [fact_id_17]
L0 đọc fact:    __kt_facts[17].text → "Hà Nội là thủ đô Việt Nam"
L0 output:      "Hà Nội là thủ đô Việt Nam"

Code thật (knowtree.ol):
  kt_search(query) → split → _kt_score_word → follow links → best fact

Search = walk links. O(word_count × avg_links).
KHÔNG scan toàn bộ facts. Follow links.
```

---

## DUNG LƯỢNG — Bao lâu đầy?

**Tham khảo: KNOWTREE_DESIGN §"DUNG LƯỢNG", 16GB_example**

```
L0 (engine + UDC):  ~1 MB (binary) + 20 KB (UDC)     = CỐ ĐỊNH
L1 (runtime):       STM ~40 KB + Silk ~40 KB            = CỐ ĐỊNH
L2+ (thư viện):     BẮT ĐẦU TỪ 0, PHÁT TRIỂN MỖI NGÀY

Heap 256 MB:
  L0+L1: ~1.1 MB
  Còn lại: ~255 MB cho L2+
  = ~637 cuốn sách (mỗi cuốn ~400 KB)

16 GB disk:
  = ~40,000 cuốn sách = thư viện nhỏ

16GB_example tính:
  7.42 tỷ links × 2B = 14.84 GB
  DNA 3.2 tỷ bases → toàn bộ sự sống
  HomeOS 7.42 tỷ links → toàn bộ tri thức
  
  HomeOS entropy: 98.2 Gbits > DNA 6.4 Gbits (15.3×)
```

---

## CODE THẬT vs THIẾT KẾ — Nox đã build gì

### knowtree.ol — Đã có:

```
✅ __kt_chars[]  — lazy char nodes (N.1)
✅ __kt_words[]  — word nodes, mol, facts links (N.2)
✅ __kt_facts[]  — fact nodes, word indices, mol (N.3)
✅ kt_search()   — query → word → follow links → best (N.4)
✅ kt_learn()    — text → split → word nodes → fact → reverse links
✅ kt_read_book() — file → sentences → kt_learn each → Silk connect
✅ kt_stats()    — chars/words/facts count
✅ Reverse links: word.facts[] → fact_ids (bidirectional)
✅ CI fallback:   _a_has for case-insensitive word match
✅ Cross-sentence Silk: _kt_silk_sentences (shared words → co_activate)
```

### Chưa có (cần build):

```
⬜ L2 category branches (facts/books/convs/skills — hiện flat)
⬜ Nhánh phân tầng (geo/sci/personal → Vietnam → HàNội)
⬜ 172,849 base char nodes (hiện lazy create, nhưng UDC mol bị scope bug)
⬜ Dream cluster → promote lá mới
⬜ QR append-only signed
⬜ Silk mol-keyed ĐÚNG (VM scope bug fixed nhưng mol_compose cần verify)
⬜ ○{} query syntax
⬜ Agent/Skill nodes trong L1
```

---

## TÓM TẮT — Nox đọc cái này

```
L0 = GỐC CÂY = BỘ MÁY
  9,584 SDF. 14 cơ chế DNA. Encoder ∫. Decoder ∂. VM.
  KHÔNG chứa data. Đọc/ghi L2+.
  Giống ribosome: đọc DNA, tạo protein.

L1 = RỄ CÂY = VẬN HÀNH
  Compiler. Pipeline. Silk engine. Dream engine.
  SecurityGate. ConversationCurve. 7 Instincts.
  Agent nodes (LeoAI, Chiefs). Skill nodes (15+).
  Giống tủy sống: xử lý, phản xạ, điều phối.

L2→Ln-1 = THÂN + TÁN LÁ = THƯ VIỆN
  facts, books, conversations, skills, people, ...
  Mỗi nhánh = array[65,536]. Lồng nhau = fractal.
  Ln-1 = lá = 2 bytes = P_weight. Hết.
  PHÁT TRIỂN MỖI NGÀY. BẮT ĐẦU TỪ 0.
  Giống DNA: 4 bases → chuỗi vô hạn → sự sống.

SILK = TƠ NHỆN
  Structural: thứ tự = 0 bytes. Chương 1 trước chương 2.
  Hebbian: cầu nối ngang. "buồn" ↔ "mất việc" ↔ "thất bại"
  Fire together → wire together. φ⁻¹ decay.
  Đủ mạnh → Dream → lá mới ở đúng vị trí.

ENCODER ∫: input → tokenize → char nodes → word nodes → fact → GHI lên cây
DECODER ∂: query → word nodes → follow links → fact → output
CÂY LÀ INDEX. Walk links. Không scan.
```

---

**Links tài liệu gốc:**
- `old/2026-03-17/HomeOS_Architecture.md` — bản vẽ chính
- `old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md` — sinh học phân tử
- `docs/HomeOS_SPEC_v3.md` — bổ sung kỹ thuật
- `docs/KNOWTREE_DESIGN.md` — thiết kế cây fractal (Nox v3)
- `old/2026-03-18/SPEC_NODE_SILK.md` — Node + Silk spec
- `old/HomeOS_16GB_example.md` — bài toán dung lượng
- `stdlib/homeos/knowtree.ol` — code thật
