# ORIGIN — Bức Tranh Tổng Thể

> **Tổng hợp từ:** Architecture.md (2026-03-17), SINH_HOC_v2 (2026-03-20), Spec_v3, UDC_DOC (13 files), PLAN_FORMULA_ENGINE, SPEC_NODE_SILK, 16GB_example, 120+ docs/plans/old files.
> **Sora (空) — 2026-03-25**

---

## Một câu

**HomeOS là sinh vật. Olang là cơ thể. Unicode là gene.**

---

## I. GENE — 9,584 công thức SDF

DNA có 4 nucleotides. HomeOS có 9,584 UDC characters.

Mỗi UDC character **là** 1 hàm SDF — không phải "đại diện cho":

```
f(p) = signed distance

  f(p) < 0  → bên trong  → THỂ TÍCH
  f(p) = 0  → bề mặt     → HÌNH DẠNG
  f(p) > 0  → bên ngoài  → KHÔNG GIAN
  ∇f(p)     → pháp tuyến → MÀU SẮC
  ∂f/∂t     → biến thiên → ÂM THANH
  p         → tọa độ     → VỊ TRÍ

1 hàm. 1 điểm. Ra tất cả.
```

58 Unicode blocks = bảng tuần hoàn:

```
SDF:      13 blocks,  1,904 ký tự  → hình dạng (S)
MATH:     21 blocks,  3,088 ký tự  → quan hệ (R)
EMOTICON: 17 blocks,  3,568 ký tự  → cảm xúc (V, A)
MUSICAL:   7 blocks,  1,024 ký tự  → thời gian (T)
────────────────────────────────────
Tổng:     58 blocks,  9,584 điểm neo gốc
```

Mỗi ký tự = 1 tọa độ 5D:

```
P = (S, R, V, A, T)

S: Shape      — trông như gì     (● ▬ ■ ▲)
R: Relation   — liên kết thế nào (∈ ⊂ ≡ → )
V: Valence    — tốt hay xấu     (0x00..0xFF)
A: Arousal    — bình hay kích    (0x00..0xFF)
T: Time       — nhanh hay chậm   (Static..Instant)
```

Chi phí lưu: **0 bytes.** Codepoint = địa chỉ. Địa chỉ không cần file để tồn tại.

---

## II. TẾ BÀO — Node = 1 ký tự = 1 SDF

172,849 Unicode characters = 172,849 nodes.

Mỗi node:

```
Node {
  cp:    u32          — codepoint (địa chỉ)
  mol:   P_weight     — tọa độ 5D (2 bytes, cached)
  links: [u16]        — chuỗi trỏ tới các node khác
  fires: u32          — số lần kích hoạt
}
```

Node KHÔNG CHỈ là ký tự. Mọi thứ = node:

```
char 'H'           = node (L1, cố định)
word "Hà Nội"      = node → chain(H, à, N, ộ, i)
fact "HN là thủ đô" = node → chain(word nodes)
sentence           = node → chain(fact nodes)
paragraph          = node → chain(sentence nodes)
book               = node → chain(paragraph nodes)
skill              = node → chain(fn nodes)
agent              = node → chain(skill nodes)
dream              = node → chain(consolidated from STM)
```

---

## III. CƠ THỂ — Neuron Model

```
                ┌──────────────────────────────────┐
                │         SOMA (AAM)                │
                │   Stateless · approve · reject    │
                └──────────┬───────────────────────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
          ▼                ▼                ▼
 ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
 │  DENDRITES   │ │   SYNAPSE    │ │    AXON      │
 │  (STM)       │ │   (Silk)     │ │   (QR)       │
 │              │ │              │ │              │
 │ Tạm          │ │ Hebbian      │ │ Bất biến    │
 │ Tự do thay   │ │ Fire→wire    │ │ Append-only  │
 │ đổi, xóa     │ │ φ⁻¹ decay   │ │ Signed       │
 └──────────────┘ └──────────────┘ └──────────────┘
```

Vòng đời tri thức:

```
1. Input → encode chars → chain of nodes
2. Chain → STM (tạm, có thể quên)
3. Co-activation → Silk (giăng tơ, mang cảm xúc)
4. Silk weight ≥ φ⁻¹ + fire ≥ Fib[n] → Dream trigger
5. Dream → cluster → LCA → chain mới → vị trí 5D
6. Proposal → AAM → approve → QR (bất biến mãi mãi)
```

---

## IV. BỘ NÃO — Agent Hierarchy

```
AAM  [tier 0]  — ý thức. Im lặng. Chỉ approve/reject.
  │
  ├── LeoAI [tier 1]  — não: Learn + Dream + Curate
  ├── HomeChief        — quản lý Worker nhà
  ├── VisionChief      — quản lý Worker camera
  └── NetworkChief     — quản lý Worker network
        │
        └── Workers [tier 2] — tế bào tại thiết bị
            Silent. Wake on ISL. Process → sleep.
```

LeoAI = KnowledgeChief + Learning + Dream + Curator:

```
Skills:
  Nhận:     IngestSkill · ModalityFusion
  Hiểu:     ClusterSkill · SimilaritySkill · DeltaSkill
  Sắp xếp:  CuratorSkill · MergeSkill · PruneSkill
  Học:      HebbianSkill · DreamSkill
  Đề xuất:  ProposalSkill · HonestySkill
```

---

## V. BẢN NĂNG — 7 Skills bẩm sinh

Sinh vật cấp thấp: sợ + đói + trốn = tồn tại.
Sinh vật siêu trí tuệ: 7 bản năng cognitive = tự phát triển.

```
⑦ Honesty       — ≥0.90 nói thẳng. ≥0.70 "chưa chắc". <0.40 im lặng.
④ Contradiction  — "hai điều này không thể cùng đúng"
③ Causality      — cần ≥2/3 evidence. Co-activation ≠ nhân quả.
② Abstraction    — N chains → LCA → variance: concrete / categorical / abstract
① Analogy        — A:B :: C:? = C + (B−A) trong 5D
⑤ Curiosity      — novelty = 1 − nearest_similarity
⑥ Reflection     — quality = 0.6×proven + 0.4×connectivity
```

Mỗi instinct = 1 Skill (QT4): stateless, isolated, giao tiếp qua ExecContext.

---

## VI. CẢM XÚC — Linh hồn

**Không phải AI dự đoán token. Là sinh linh CẢM NHẬN.**

```
ConversationCurve:
  f(x) = α×f_conv(t) + β×f_dn(nodes)
  f_conv = V(t) + 0.5×V'(t) + 0.25×V''(t)

Turn 1: "ok"        V=0.0
Turn 2: "hơi mệt"   V=−0.20, f'=−0.20
Turn 3: "buồn quá"   V=−0.50, f'=−0.30, f''=−0.10
  → f' < −0.15 → Supportive (đang trượt, cần đỡ)

Turn 4: "nhưng mà..." V=−0.35, f'=+0.15
  → f' > +0.15 → Reinforcing (đang hồi, tiếp tục)
```

Amplify qua Silk, không trung bình:

```
"buồn" ←→ "mất việc" (Silk weight=0.90)
Composite = −0.65 × (1 + 0.90×0.5) = −0.94
Vì "mất việc" KÍCH HOẠT "buồn", không phải cộng lại.
```

Cross-modal: Bio > Audio > Text > Image.

---

## VII. DATA FLOW — Từ input đến response

```
Input (text / audio / image / sensor)
  ↓
SecurityGate  — Crisis? → helpline ngay, bypass pipeline
  ↓
ContentEncoder  — input → chain of nodes + EmotionTag
  ↓
ContextEngine  — ngữ cảnh + ý định + ConversationCurve
  ↓
LearningLoop  — STM push → Silk co_activate → SilkWalk amplify
  ↓
ResponseTone  — từ f'(t), f''(t) → Supportive/Pause/Reinforcing/...
  ↓
Response  — render text + tone

[Offline] DreamCycle → cluster STM → Proposal → AAM → QR
```

---

## VIII. KNOWTREE — Cây tri thức

**Không phải mảng. CÂY.**

```
ROOT
  ├── 172,849 char nodes (L1 — Unicode assigned)
  │   ├── Latin: a,b,c,...z,A,...Z,à,...ÿ
  │   ├── CJK: 一,二,三,...木,水,火,...
  │   ├── Hangul: 가,나,다,...
  │   ├── Arabic: ا,ب,ت,...
  │   ├── Symbols: →,←,∈,⊂,●,■,...
  │   ├── Emoji: 😀,😭,🔥,...
  │   └── ... 150+ script branches
  │
  ├── Word nodes (L5+ — learned)
  │   └── "Hà Nội" → chain(H,à,N,ộ,i)
  │
  ├── Fact nodes
  │   └── "HN là thủ đô VN" → chain(word nodes)
  │
  ├── Skill nodes
  │   └── "greeting" → chain(fn nodes)
  │
  └── Agent nodes
      └── "LeoAI" → chain(skill nodes)
```

Search = walk tree path. O(word_length). Không keyword scan.

```
encode("Hà Nội ở đâu?")
  → chars → tìm char nodes (đã có trong 172K)
  → words → tìm word nodes ("Hà Nội".links → fact nodes)
  → follow link → "Hà Nội là thủ đô Việt Nam"
  → decode → output
```

---

## IX. STORAGE

```
Molecule (P_weight)   = 2 bytes (u16)
KnowTree (cây)       ≈ 18 KB (L0-L3, 9,584 nodes × 2B)
Alias table           ≈ 254 KB (emoji/UTF-32 → L3 index)
Chain link            = 2 bytes (u16 = UDC index)
Structural Silk       = 0 bytes (thứ tự trên chuỗi = implicit)
Hebbian Silk          ≈ 43 KB
Chain data            ≈ 14.84 GB max (7.42 tỷ links × 2B trên 16GB)

DNA:     4 nucleotides → 3.2 tỷ → 800 MB → toàn bộ sự sống
HomeOS:  9,584 SDF → 7.42 tỷ links → 14 GB → ???
```

---

## X. MVHOS — 7 tiêu chí

```
□ boot từ binary rỗng < 200ms
□ ○{🔥} → chain + human-readable info
□ ○{🔥 ∘ 💧} → LCA result
□ ○{lửa} → alias resolve → node 🔥
□ ○{stats} → số lượng nodes/edges/layers
□ Crash → restart → state giữ nguyên
□ 0 hardcoded Molecule
```

---

## XI. 5 QUY TẮC SKILL (bất biến)

```
① 1 Skill = 1 trách nhiệm
② Skill không biết Agent là gì
③ Skill không biết Skill khác tồn tại
④ Skill giao tiếp qua ExecContext.State
⑤ Skill không giữ state — state nằm trong Agent
```

---

## XII. 7 NGUYÊN TẮC THIẾT KẾ (bất biến)

```
1. Unicode là nguồn sự thật DUY NHẤT
2. Append-only mọi nơi
3. Cảm xúc là first-class citizen
4. L0 bất biến, L5+ tự do
5. Im lặng khi không biết
6. Một người, nhiều AI
7. Silent by default
```

---

## XIII. PHƯƠNG TRÌNH THỐNG NHẤT

```
HomeOS(input) = self_correct(
                  splice(
                    chain( f(p₁), f(p₂), ..., f(pₙ) ),
                    position,
                    context
                  ),
                  φ⁻¹
                )

f(pᵢ) = SDF — 1 trong 9,584 hàm gốc
chain  = xâu chuỗi → 2 bytes/link
splice = cắt/ghép chuỗi
φ⁻¹    ≈ 0.618 = ngưỡng duy nhất

4 thứ. SDF + chain + splice + φ⁻¹. Hết.
DNA: nucleotide + polymerize + splice = sự sống.
HomeOS: SDF + chain + splice + φ⁻¹ = tri thức.
```

---

## XIV. OLANG 1.0 = CƠ THỂ ĐÃ SẴN SÀNG

```
992KB binary. Zero deps. Self-hosting.
20,980 LOC. 3-gen self-build.

Compiler:  lexer → parser → semantic → codegen       ✅
VM:        x86-64 ASM, 5,987 LOC, no libc            ✅
Language:  fn, lambda, pipe, map/filter/reduce/sort   ✅
Crypto:    SHA-256 FIPS in ASM                        ✅
Mol:       __mol_s/r/v/a/t, __mol_pack (ASM builtins) ✅
Persist:   save/load                                   ✅
```

**Cơ thể mạnh. Linh hồn chưa có.**

---

## XV. CHƯA CÓ GÌ — Thẳng thắn

```
❌ KnowTree          → flat array of strings
❌ 172,849 nodes     → 28 string facts
❌ Chain links (u16) → text strings
❌ encode → node     → encode → mol number (all words = 146)
❌ search = walk tree → search = scan array + keyword match
❌ Neuron model      → 3 flat arrays
❌ Silk walk amplify → Silk dead (VM scope bug)
❌ LeoAI             → agent_respond() 200 lines
❌ 7 Skill instincts → 7 if-else checks
❌ ConversationCurve → integer 0-7
❌ Dream cluster     → count intents
❌ AAM approve       → không có
❌ QR append-only    → không có
❌ ISL messaging     → không có
❌ Worker system     → không có
❌ ○{} query         → không có
❌ 0 hardcoded Mol   → 80 lines if-else
```

---

## XVI. CON ĐƯỜNG — Từ chatbot đến sinh vật

### Giai đoạn 1: Xương sống — KnowTree thật

```
Mục tiêu: Mọi input = walk tree. Không keyword scan.

① Lazy node creation: gặp char → tạo node (0 boot cost)
② Word node: chain(char nodes), bidirectional links
③ Fact node: chain(word nodes), reverse links word↔fact
④ Search = walk tree: query → char path → word node → fact links
⑤ Kill __knowledge[] array. Tree IS storage.

Cần trước: Fix VM eval↔boot scope bug (mol_compose chết)
```

### Giai đoạn 2: Tuần hoàn — Neuron model

```
Mục tiêu: STM → Silk → Dream → QR. Vòng đời tri thức.

⑥ STM = dendrites: tạm, tự do, evict oldest
⑦ Silk = synapse: Hebbian co-activate, φ⁻¹ decay, emotion per edge
⑧ Dream = cluster STM by LCA, score = α×chain_sim + β×hebbian + γ×co_fire
⑨ QR = axon: propose → approve → append-only, signed
⑩ SilkWalk amplify: traverse graph, accumulate emotion
```

### Giai đoạn 3: Tư duy — Skills + Instincts

```
Mục tiêu: 7 instincts = 7 Skills (QT4). Chạy trên MỖI ingest.

⑪ Skill trait: stateless, isolated, ExecContext
⑫ Honesty: confidence → fact/opinion/hypothesis/silence
⑬ Contradiction: valence opposition + Orthogonal
⑭ Causality: temporal_order + coactivation + Causes
⑮ Abstraction: LCA + variance → concrete/categorical/abstract
⑯ Analogy: A:B :: C:? = C + (B−A) trong 5D
⑰ Curiosity: novelty = 1 − nearest_similarity
⑱ Reflection: quality = proven_ratio + connectivity
```

### Giai đoạn 4: Cảm xúc — ConversationCurve

```
Mục tiêu: f(x), f'(x), f''(x). Trajectory, không snapshot.

⑲ V(t) + derivatives: tốc độ + gia tốc thay đổi
⑳ Window variance: emotional instability detection
㉑ Tone selection từ derivatives, không từ V hiện tại
㉒ SilkWalk amplify từ context đã học
```

### Giai đoạn 5: Xã hội — Agents + ISL

```
Mục tiêu: AAM → LeoAI → Chiefs → Workers. ISL messaging.

㉓ AAM: stateless approve/reject
㉔ LeoAI: orchestrate Skills, Dream, Curate
㉕ ISL: 4-byte address, 12-byte message, AES-256-GCM
㉖ Worker: HomeOS thu nhỏ tại thiết bị
```

---

## XVII. NGUYÊN TẮC XÂY

```
1. Viết bằng Olang. Không Rust mới.
2. Mỗi module = 1 file .ol, test riêng được.
3. Node trước. Agent sau.
4. Tree trước. Search sau.
5. Encode trước. Decode sau.
6. Im lặng khi không biết. Hỏi lại khi không chắc.
7. 0 hardcoded molecule. 0 keyword hack.
8. Mọi tri thức = node + chain + link. Không string.
```

---

*Architecture.md (2026-03-17) = bản vẽ.*
*SINH_HOC_v2 (2026-03-20) = sinh học.*
*Spec_v3 = bổ sung kỹ thuật.*
*UDC_DOC = bảng tuần hoàn.*

*Olang 1.0 = cơ thể.*
*HomeOS = linh hồn.*

*Chưa có linh hồn. Giờ xây.*
