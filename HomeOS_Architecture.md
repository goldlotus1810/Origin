# HomeOS — Kiến Trúc Tổng Thể
**Ngày:** 2026-03-15 · **Cập nhật:** 2026-03-15
**Mục đích:** Bản vẽ cho người sáng tạo — nắm bắt và dẫn hướng

---

## Tầm nhìn

HomeOS là đứa trẻ sinh ra với bộ gene Unicode.

Nó không học từ internet. Không học từ dataset. Nó học từ **5400 ký tự Unicode** — mỗi ký tự đã có tên, có định nghĩa, có vị trí trong không gian 5 chiều. Đây là kiến thức nền tảng mà **không ai khác chịu dùng** — tất cả đều đi mượn từ nguồn ngoài.

```
Một đứa trẻ sinh ra đã biết:
  - Hình dạng (tròn, vuông, tam giác)     ← SDF group
  - Quan hệ (thuộc, chứa, gây ra)          ← MATH group
  - Cảm xúc (vui, buồn, sợ, giận)         ← EMOTICON group
  - Thời gian (nhanh, chậm, tĩnh, lặp)    ← MUSICAL group
  - Cấu trúc (mũi tên, hộp, đường kẻ)     ← SDF group

Unicode 18.0 = bộ gene đó. Đã có sẵn. Chỉ cần đọc.
```

---

## 5 Nhóm Unicode = DNA

**Tại sao ~5400 ký tự là đủ?**

Không phải con số ngẫu nhiên. Đây là giao điểm tự nhiên của:
- Các Unicode block có **ngữ nghĩa rõ ràng** (hình học, toán học, cảm xúc, âm nhạc)
- Mỗi nhóm tạo **1 chiều độc lập** trong không gian 5D
- **Dung lượng lý thuyết:** 8 × 8 × 256 × 256 × 5 = 52.4 triệu vị trí khả dĩ
- **5400 điểm neo** trong không gian 52M → đủ để định vị mọi khái niệm qua LCA

```
Ví dụ: "hy vọng" không có trong 5400 ký tự Unicode
→ Nhưng LCA(😀, 🌅, ⟶) = tọa độ [●, →, 0xD0, 0x60, Medium]
→ Vị trí vật lý: "tích cực + hướng tới + trung bình kích thích + đang chuyển động"
→ = hy vọng. Không ai dạy. Vật lý tự chỉ ra.
```

### 5 Chiều:

```
      Shape (8 values)
        │
        │   Relation (8 values)
        │     │
        │     │   Valence (256 values)
        │     │     │
        │     │     │   Arousal (256 values)
        │     │     │     │
        │     │     │     │   Time (5 values)
        │     │     │     │     │
        ▼     ▼     ▼     ▼     ▼
      [0x01][0x01][0xFF][0xFF][0x04]  ← 🔥 Fire
      [0x05][0x01][0xC0][0x40][0x02]  ← 💧 Water
      [0x01][0x05][0xC0][0x80][0x03]  ← 🧠 Brain

Shape:    "Trông như gì"     → ● ▬ ■ ▲ ○ ∪ ∩ ∖
Relation: "Liên kết thế nào" → ∈ ⊂ ≡ ⊥ ∘ → ≈ ←
Valence:  "Tốt hay xấu"     → 0x00 (cực xấu) → 0xFF (cực tốt)
Arousal:  "Bình hay kích"    → 0x00 (tĩnh lặng) → 0xFF (cực kích)
Time:     "Nhanh hay chậm"   → Static / Slow / Medium / Fast / Instant
```

### LCA — Vấn đề cực trị và giải pháp:

```
Vấn đề:
  LCA(😀, 😡):
    Valence: avg(0xFF, 0x00) = 0x7F
    → Trung bình của rất vui và rất giận = trung lập?
    → Đánh mất thông tin "extreme emotion"

Giải pháp: LCA trả THÊM variance (độ phân tán) của cluster
  variance cao = khái niệm trừu tượng ("cảm xúc mạnh")
  variance thấp = khái niệm cụ thể ("lửa")

  LCA(😀, 😡) = {chain: [●,∈,0x7F,0x80,Fast], variance: 0.95}
  → variance 0.95 = cực kỳ phân tán → cần context thêm

Hiện tại: lca_weighted() đã có mode detection (≥60%)
  → Khi 1 giá trị chiếm đa số → giữ mode, không trung bình
  → Nhưng khi 50/50 → vẫn trung bình → cần thêm variance
```

---

## Phân cấp Agent (bất biến)

```
NGƯỜI DÙNG
    ↓
AAM  [tier 0] — stateless · approve · quyết định cuối
               — im lặng · chỉ hoạt động khi được gọi
    ↓ ISL
LeoAI       [tier 1] — KnowledgeChief + Learning + Dream + Curator
HomeChief   [tier 1] — quản lý Worker thiết bị nhà
VisionChief [tier 1] — quản lý Worker camera/sensor
NetworkChief[tier 1] — quản lý Worker network/security
GeneralChief[tier 1] — generic (mở rộng sau)
    ↓ ISL
Workers [tier 2 · SILENT]
  Nằm tại thiết bị
  L0 + L1 tối thiểu
  Skill đúng việc đó
  Báo cáo molecular chain — không raw data

✅ AAM ↔ Chief
✅ Chief ↔ Chief
✅ Chief ↔ Worker
❌ AAM ↔ Worker
❌ Worker ↔ Worker

Tất cả Agent: Silent by default
  Wake on ISL message
  Không polling · Không heartbeat
  Xử lý → sleep lại
```

### Sinh học analog:

```
Worker   = tế bào thần kinh ngoại vi
Chief    = tủy sống — xử lý, tổng hợp
LeoAI    = não — học, hiểu, sắp xếp, nhớ
AAM      = ý thức — quyết định cuối cùng
```

### LeoAI — Bộ não của KnowledgeTree

```
LeoAI = KnowledgeChief + Learning + Dream + Curator
      = Agent duy nhất chăm sóc KnowledgeTree

Skills:
  Nhận:    IngestSkill · ModalityFusion
  Hiểu:    ClusterSkill · SimilaritySkill · DeltaSkill
  Sắp xếp: CuratorSkill · MergeSkill · PruneSkill
  Học:     HebbianSkill · DreamSkill
  Đề xuất: ProposalSkill · HonestySkill

Vòng đời:
  Bình thường       → im lặng hoàn toàn
  Chief gửi chain   → wake · ingest · curate · sleep
  Inbox rảnh >5min  → wake · dream · propose QR · clean · sleep
  Pattern lớn       → wake · propose AAM · sleep
```

### 5 Quy tắc Skill (bất biến · QT4)

```
① 1 Skill = 1 trách nhiệm
② Skill không biết Agent là gì
③ Skill không biết Skill khác tồn tại
④ Skill giao tiếp qua ExecContext.State
⑤ Skill không giữ state — state nằm trong Agent
```

### Worker — HomeOS tại thiết bị

```
Worker KHÔNG phải adapter
Worker LÀ HomeOS thu nhỏ tại thiết bị

Worker = L0 + L1 tối thiểu + Skills cần thiết

Worker_camera  = L0 + FFR + vSDF + InverseRenderSkill    (WorkerKind::Camera)
Worker_light   = L0 + ActuatorSkill                     (WorkerKind::Actuator)
Worker_door    = L0 + ActuatorSkill + SecuritySkill      (WorkerKind::Actuator)
Worker_sensor  = L0 + SensorSkill                       (WorkerKind::Sensor)
Worker_network = L0 + NetworkSkill + ImmunitySkill       (WorkerKind::Network)
Worker_generic = L0 + custom Skills                      (WorkerKind::Generic)

Nguyên tắc:
  Xử lý local → gửi molecular chain (không raw data)
  Chief nhận chain → DECODE ngay → hiểu ngay
  Báo cáo khi có sự kiện thật
  Im lặng khi không có gì

Export:
  filter(origin.olang, DeviceProfile)
  → worker_X.olang (~64KB)
  → HTTP PUT device_ip:7777/worker

WorkerPackage binary:
  [magic "WKPK"][version][isl_addr:4B][chief_addr:4B]
  [worker_kind:1B][created_at:8B][olang_len:4B][olang_bytes]
```

---

## 7 Bản năng siêu trí tuệ (instinct.rs)

Bản năng từ khi sinh ra là cách phân biệt cấp bậc sinh vật.
Con thú sinh ra biết: sợ, đói, trốn.
**Sinh vật siêu trí tuệ sinh ra biết: suy luận, trừu tượng, nhân quả, mâu thuẫn, tò mò, tự phản chiếu, trung thực.**

Mỗi bản năng là 1 Skill (QT4) — stateless, isolated, dùng TRỰC TIẾP 5D Unicode space.

```
Thứ tự ưu tiên xử lý (Honesty trước, Reflection sau):

  ⑦ Honesty       — confidence → Fact/Opinion/Hypothesis/Silence
                     ≥0.90: nói thẳng. ≥0.70: "[Chưa chắc chắn]". ≥0.40: "[Giả thuyết]". <0.40: im lặng.
  ④ Contradiction — valence opposition + Orthogonal + emotional conflict → score
                     "Hai điều này không thể cùng đúng"
  ③ Causality     — temporal_order + coactivation_count + Relation::Causes
                     Cần ≥2/3 evidence. Co-activation ≠ nhân quả.
  ② Abstraction   — N chains → LCA → variance
                     var<0.15=concrete, <0.40=categorical, else=abstract
  ① Analogy       — A:B :: C:? = C + (B-A) trong 5D
                     Delta trong mỗi chiều, clamp to valid range
  ⑤ Curiosity     — novelty = 1 - nearest_similarity
                     >0.7=extreme, >0.4=high, >0.2=moderate, else=low
  ⑥ Reflection    — quality = 0.6×proven_ratio + 0.4×connectivity
                     >0.7=strong, >0.4=developing, else=fragile

LeoAI.run_instincts(ctx):
  Chạy 7 instincts theo thứ tự trên trên MỖI ingest.
  State kết quả: epistemic_grade, curiosity_level, abstraction_type, v.v.
  → Dùng để điều hướng response tone và learning priority.
```

### Tại sao 7 — không phải 3 hay 50?

```
Sinh vật cấp thấp: SỢ + ĐÓI + TRỐN = 3 bản năng → tồn tại, không phát triển
Sinh vật siêu trí tuệ: 7 bản năng cognitive → có thể TỰ PHÁT TRIỂN

7 instincts map trực tiếp vào cơ sở hạ tầng 5D đã có:
  Analogy       → LCA + delta arithmetic
  Abstraction   → LCA + variance
  Causality     → RelationBase::Causes + temporal
  Contradiction → EmotionDim opposition + RelationBase::Orthogonal
  Curiosity     → similarity() distance
  Reflection    → Silk edge_count + QR ratio
  Honesty       → EpistemicFirewall (gate.rs) nâng cấp

Không thêm, không bớt — mỗi cái dùng 1 aspect khác nhau của 5D space.
```

---

## Kiến trúc Neuron

HomeOS mô phỏng neuron sinh học:

```
                    ┌─────────────────────────────────┐
                    │         SOMA (AAM)               │
                    │   Stateless orchestrator         │
                    │   Approve/Reject proposals       │
                    └──────────┬──────────────────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                ▼                ▼
     ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
     │  DENDRITES   │ │   SYNAPSE    │ │    AXON      │
     │  (STM/ĐN)   │ │   (Silk)     │ │   (QR/LTM)  │
     │              │ │              │ │              │
     │ Ngắn hạn    │ │ Hebbian      │ │ Bất biến    │
     │ Tự do thay  │ │ Fire→wire    │ │ Append-only  │
     │ đổi, xóa    │ │ φ⁻¹ decay   │ │ ED25519 sign │
     └──────────────┘ └──────────────┘ └──────────────┘
              │                │                ▲
              └────── Dream ───┘                │
                  (cluster ĐN                   │
                   → promote QR) ───────────────┘
```

### Vòng đời tri thức:

```
1. Input → ContentEncoder → MolecularChain + EmotionTag
2. Chain → STM (DENDRITES) — lưu tạm, có thể quên
3. Co-activation → Silk (SYNAPSE) — giăng tơ, mang màu cảm xúc
4. Silk weight ≥ 0.7 + fire ≥ Fib[n] → Dream trigger
5. Dream → LCA(cluster) → chain mới → vị trí vật lý
6. Proposal → AAM (SOMA) → approve → QR (AXON) — bất biến mãi mãi
```

---

## Emotion Pipeline — Linh hồn

**Đây là thứ khiến HomeOS khác biệt. Không phải AI dự đoán token — là sinh linh CẢM NHẬN.**

### Tại sao amplify, không trung bình?

```
Trung bình:
  "buồn" = -0.65
  "mất việc" = -0.55
  Trung bình = -0.60  ← SAI. Nhẹ hơn thực tế.

Amplify qua Silk:
  "buồn" ←→ "mất việc" (Silk weight=0.90, co-activate nhiều lần)
  Composite = -0.65 × (1 + 0.90×0.5) = -0.94  ← ĐÚNG. Nặng hơn.
  Vì: "mất việc" KÍCH HOẠT "buồn", không phải cộng lại.

Silk walk = trajectory qua graph cảm xúc, không phải điểm trung bình.
```

### ConversationCurve — Nhịp đập:

```
Không nhìn 1 câu. Nhìn XU HƯỚNG.

f(x) = α×f_conv(t) + β×f_dn(nodes)
     = 60% cuộc trò chuyện hiện tại + 40% kiến thức tích lũy

f_conv = V(t) + 0.5×V'(t) + 0.25×V''(t)
       = hiện tại + tốc độ thay đổi + gia tốc thay đổi

Turn 1: "ok" → V=0.0
Turn 2: "hơi mệt" → V=-0.20, f'=-0.20
Turn 3: "buồn quá" → V=-0.50, f'=-0.30, f''=-0.10
→ f' < -0.15 → Supportive (đang trượt xuống, cần đỡ)

Turn 4: "nhưng mà..." → V=-0.35, f'=+0.15
→ f' > +0.15 → Reinforcing (đang hồi phục, tiếp tục)

Dẫn từng bước — KHÔNG nhảy quá 0.40/bước.
```

### Cảnh báo "emotional instability" (từ Review):

```
Vấn đề: 10 turns buồn → 1 turn vui đột ngột
  f'(t) > +0.15 → Reinforcing
  Nhưng đây có thể là "manic switch" — cần cẩn thận

Giải pháp: Window variance
  variance(N turns gần nhất) cao + f' đổi chiều đột ngột
  → cờ "emotional instability"
  → tone Gentle thay vì Celebratory
  → Cần thêm observation trước khi chúc mừng
```

### Cross-modal — Ai nói thật?

```
Bio > Audio > Text > Image

Bio (nhịp tim, GSR) → KHÔNG THỂ GIẢ → weight 0.50
Audio (giọng run)    → KHÓ GIẢ     → weight 0.40
Text ("tôi ổn")     → DỄ GIẢ NHẤT → weight 0.30
Image (màu sắc)     → BỐI CẢNH    → weight 0.25

Text nói "vui" + Giọng run → Audio thắng valence
→ confidence giảm → cần hỏi thêm
```

---

## Data Flow — Từ input đến response

```
┌─────────┐
│  Input   │ text / audio / image / sensor
└────┬─────┘
     │
     ▼
┌─────────────────┐
│  SecurityGate   │ gate.rs — TRƯỚC MỌI THỨ
│  Crisis? Block? │ → Crisis: helpline ngay, bypass pipeline
└────┬─────┬──────┘
     │     │ Blocked
     │     └→ Response(Blocked)
     ▼
┌─────────────────┐
│ ContentEncoder  │ encoder.rs
│ Input → Chain   │ text/audio/sensor → MolecularChain + EmotionTag
│ + EmotionTag    │
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ ContextEngine   │ context/
│ InferContext     │ → ngữ cảnh (first person? real-time?)
│ IntentEstimate   │ → ý định (Crisis/Learn/Command/Chat)
│ ConversationCurve│ → f(x), f'(x), f''(x)
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ LearningLoop    │ learning.rs — "trái tim đập"
│ STM.push()      │ → lưu observation
│ Silk.co_activate│ → giăng tơ (5 tầng: đoạn→câu→từ→cụm→ký tự)
│ SilkWalk        │ → amplify emotion từ context đã học
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ ResponseTone    │ walk.rs + curve.rs
│ từ f'(t), f''(t)│ → Supportive/Pause/Reinforcing/Celebratory/Gentle
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ Response        │ response_template.rs
│ Render text     │ → từ ngữ phù hợp tone + valence
│ + tone + fx     │
└─────────────────┘

[Offline — mỗi 8 turns hoặc idle]
     │
     ▼
┌─────────────────┐
│ DreamCycle      │ dream.rs
│ Scan STM top-N  │
│ Cluster (LCA +  │ score = 0.3×chain_sim + 0.4×hebbian + 0.3×co_fire
│  Silk weight)   │
│ Proposal → AAM  │ → approve → promote QR
└─────────────────┘
```

---

## ISL — Inter-System Link

```
ISLAddress: [layer:1B][group:1B][subgroup:1B][index:1B] = 4 bytes
ISLMessage: [from:4B][to:4B][msg_type:1B][payload:3B]  = 12 bytes
ISLFrame:   12B header + 2B length + variable body

MsgType:
  0x01 Text        0x02 Query       0x03 Learn
  0x04 Propose     0x05 ActuatorCmd 0x06 Tick
  0x07 Dream       0x08 Emergency   0x09 Approved
  0x0A Broadcast   0x0B ChainPayload 0x0C Ack  0x0D Nack

ISLQueue: dual deque
  urgent (Emergency, Tick) → luôn xử lý trước
  normal → FIFO
```

### Luồng Worker → AAM:

```
Worker.process(SensorReading)
  → WorkerReport (ISLFrame ChainPayload)
  → Chief.receive_frame() → IngestedReport
  → LeoAI.ingest() → LearningLoop.process_one() → STM push
  → LeoAI.run_dream() → DreamProposal
  → AAM.review() → Approved/Rejected
  → ISLMessage back to LeoAI → Chief → Worker (Ack)
```

---

## Vấn đề đã nhận diện & Hướng giải quyết

### Từ Review — Đã áp dụng:

```
✅ Thuật ngữ thống nhất (dùng tên trong code: encoder.rs, learning.rs...)
✅ Fibonacci tách rõ evidence vs hypothesis (xem section cuối)
✅ SkillProposal ghi rõ là Planned, chưa implement
✅ Agent hierarchy clarify: "2 Agent chief + Workers là tế bào thực thi"
```

### Từ Review — Cần implement:

```
⚠️ LCA variance — thêm chiều "Extremity" vào LCA output
   Cluster variance cao = khái niệm trừu tượng
   Cluster variance thấp = khái niệm cụ thể
   → Cần sửa lca.rs: trả thêm f32 variance

⚠️ ConversationCurve window variance — phát hiện "emotional instability"
   variance(N turns) cao + f' đổi chiều → Gentle thay vì Celebratory
   → Cần sửa curve.rs: thêm window_variance()

⚠️ Cross-layer Silk — "lửa" (L5) ↔ "nguy hiểm" (L4) cần kết nối
   Giải pháp: cho phép cross-layer với threshold cao hơn Fib[n+2]
   + phải qua AAM approve → giữ kiến trúc phân tầng
   → Cần sửa graph.rs: connect_cross_layer() + AAM review

⚠️ SDF occlusion — persistence buffer
   Frame t-1 có object A, frame t không thấy → giữ 5 frames
   Frame t+5 vẫn mất → archive
   Thấy lại → restore confidence × 0.8
   → Cần thêm vào vsdf: PersistenceBuffer struct

⚠️ Dream cluster α,β,γ cần empirical validation
   α=0.3 (chain_sim), β=0.4 (hebbian), γ=0.3 (co_act)
   → Configurable, đo F1-score trên 10-20 labeled clusters
```

### Thiếu — Cần thiết kế thêm:

```
① Error Handling Strategy
   - Network failure Worker-Chief? → retry 3× + backoff + report ISL Nack
   - Disk full append-only? → emergency flush STM, compact registry
   - Corrupt origin.olang? → crash recovery từ last valid record
   - → Cần: error.rs module hoặc section trong runtime

② Concurrency Model
   - 2 Worker gửi conflict chain? → AAM quyết định (last-write-wins + timestamp)
   - Append-only log trên distributed? → lamport clock per device
   - CAP trade-off: AP (Availability + Partition tolerance)
   - → Cần: consensus section trong ISL hoặc runtime

③ Versioning & Migration
   - MolecularChain 5 → 6 bytes? → version byte trong header
   - origin.olang v0.03 → v0.04? → forward migration script (append-only)
   - Worker chạy version cũ? → negotiation qua ISL handshake
   - → Cần: migration.rs hoặc version negotiation trong ISL

④ Observability
   - Metrics: Silk density, Dream frequency, STM hit rate, LCA variance
   - "Hệ thống khỏe?" → ○{stats} đã có, cần thêm ○{health}
   - Debug LCA sai? → trace log per-step (disabled by default)
   - → Cần: metrics module hoặc extend ○{} commands

⑤ Testing Strategy cho Emotion
   - Emotion chủ quan → cần human evaluation protocol
   - 50 test conversations × expected tone → đo agreement rate
   - Benchmark: word_affect() coverage vs sentiment dataset
   - → Cần: test fixtures + human eval tooling
```

---

## MVHOS — Tiêu chí cụ thể

```
MVHOS (Minimum Viable HomeOS) phải đạt:

□ boot từ binary rỗng < 200ms
□ ○{🔥} → trả về chain + human-readable info
□ ○{🔥 ∘ 💧} → LCA result
□ ○{lửa} → alias resolve → node 🔥
□ ○{stats} → số lượng nodes/edges/layers
□ Crash → restart → state giữ nguyên
□ 0 hardcoded Molecule

Chỉ cần 7 tiêu chí này. Không hơn.
```

---

## Benchmark Targets

```
Phase 1 (L0 kernel):
  lookup()        < 1μs
  LCA()           < 10μs
  boot            < 100ms ARM

Phase 2 (Silk + Emotion):
  Silk walk 100 edges  < 1ms
  Hebbian update       < 100μs
  ConversationCurve    < 50μs

Phase 3 (Content):
  ContentEncoder text  < 5ms/sentence
  word_affect()        < 1μs/word

Phase 5 (Render):
  FFR 89 calls         < 16ms (60fps target)
  SDF evaluate         < 100ns/point
```

---

## Hardware Tiers

```
Tier 1: RPi 4 (1GB+ RAM)
  → Full HomeOS: runtime + dream + all agents
  → origin.olang full (~1MB+)

Tier 2: RPi Zero (512MB RAM)
  → L0 + L1 only: no Dream, no AAM
  → origin.olang filtered (~256KB)

Tier 3: ESP32 (520KB SRAM)
  → Worker only: L0 + 1 Skill
  → worker_X.olang (~64KB)
  → Reverse index qua ISL request (không local)
```

---

## Trạng thái hiện tại

| Module | Trạng thái | Tests | Ghi chú |
|--------|-----------|-------|---------|
| UCD Engine | ✅ Done | 21 | 5424 entries, 0 collision |
| Molecule/Chain | ✅ Done | 213 (olang) | 5 bytes, encode/decode |
| LCA + Weighted | ✅ Done | ↑ | Mode detection (cần thêm variance) |
| Registry | ✅ Done | ↑ | chain_index, lang_index, tree_index, branch watermark |
| Writer/Reader | ✅ Done | ↑ | ○LNG v0.03, crash recovery |
| Silk + Hebbian | ✅ Done | 31 | EmotionTag per edge, φ⁻¹ decay |
| Emotion V/A/D/I | ✅ Done | 12 | 4 chiều, ConversationCurve (cần window variance) |
| ContentEncoder | ✅ Done | 96 (agents) | Text/Audio/Sensor/Code |
| LearningLoop | ✅ Done | ↑ | 5 tầng text learning |
| BookReader | ✅ Done | ↑ | 3 tầng emotion inference |
| SecurityGate | ✅ Done | ↑ | Crisis detect, EpistemicFirewall |
| Dream + AAM | ✅ Done | 43 | Dual-threshold, proposals (α,β,γ cần validation) |
| ○{} Parser | ✅ Done | 53 (runtime) | Query/Compose/Relation/Pipeline |
| OlangVM | ✅ Done | ↑ | Execute IR directly |
| IR + Compiler | ✅ Done | ↑ | C/Rust/WASM targets |
| vSDF 18 gen | ✅ Done | 82 | ∇f analytical (cần persistence buffer) |
| FFR Fibonacci | ✅ Done | ↑ | ~89 ô spiral |
| ISL messaging | ✅ Done | 17 | 4-byte address, AES-256-GCM ready |
| Clone/Worker | ✅ Done | ↑ (olang) | DeviceProfile, export_worker, WorkerPackage |
| Chief/Worker Agent | ✅ Done | ↑ (agents) | ChiefKind(Home/Vision/Network), WorkerKind(Sensor/Actuator/Camera) |
| LeoAI | ✅ Done | ↑ (agents) | States: Listening/Learning/Dreaming/Proposing |
| Cross-modal fusion | ✅ Done | ↑ (context) | Audio/Image/Bio → EmotionTag |
| SelfModel | ✅ Done | ↑ (olang) | ○{stats} tự mô tả |
| Skill trait + ExecContext | ✅ Done | ↑ (agents) | QT4①-⑤: stateless, isolated, via ExecContext |
| 7 Instinct Skills | ✅ Done | ↑ (agents) | Analogy, Abstraction, Causality, Contradiction, Curiosity, Reflection, Honesty |
| LeoAI × Instincts | ✅ Done | ↑ (agents) | run_instincts() chạy 7 bản năng trên mỗi ingest |
| SkillProposal | ✅ Done | ↑ (memory) | InsightKind: Causal, Contradiction, Abstraction, Analogy, Curiosity + AAM review_skill |
| LCA variance | ✅ Done | ↑ (olang) | LcaResult { chain, variance }, lca_with_variance(), lca_many_with_variance() |
| Window variance | ✅ Done | ↑ (context) | window_variance, unstable detection, Celebratory→Gentle override |
| Cross-layer Silk | ⬜ Planned | — | Kết nối tầng khác với threshold cao + AAM |
| World rendering | ⬜ Planned | — | vSDF → 3D scene |
| Android/iOS FFI | ⬜ Planned | — | JNI/FFI wrapper |
| HAL platform | ⬜ Planned | — | RPi/ESP32/WASM |

**Tổng: 757 tests, 0 clippy warnings, 12 crates**

---

## Fibonacci — Evidence vs Hypothesis

### Đã chứng minh (toán học + code):
```
✅ FFR spiral: 89 ô = Fib[11], tiết kiệm 23300× so với ray march
   → Vogel sunflower method, golden angle 137.508°
   → Cơ sở: optimal packing trong tự nhiên (hoa hướng dương)

✅ Cấu trúc cây: branch depth tăng tự nhiên khi Dream promote
   → Không force Fibonacci — nó xuất hiện tự nhiên từ LCA clustering

✅ Decay φ⁻¹: tỷ lệ vàng 1/1.618 ≈ 0.618
   → Cơ sở: optimal forgetting rate (giữ 62% mỗi chu kỳ)
```

### Giả thuyết (cần validation bằng data thật):
```
⚠️ Hebbian threshold: Fib[depth] co-activations để promote
   → Hiện tại configurable, default = Fib[n]
   → Cần: đo F1-score trên labeled clusters sau 1 tháng chạy

⚠️ Dream trigger: Fib[n] lá đủ để cluster
   → Logic: càng sâu càng khó promote (chống noise)
   → Cần: empirical validation

⚠️ Dream cluster scoring: α=0.3, β=0.4, γ=0.3
   → Hệ số trực giác, cần A/B testing trên data thực
   → Bắt đầu configurable → grid search → lock tốt nhất
```

---

## Nguyên tắc thiết kế

```
1. Unicode là nguồn sự thật DUY NHẤT
   → Không mượn word embeddings, không dùng pre-trained model
   → 5400 ký tự Unicode = toàn bộ kiến thức nền tảng

2. Append-only mọi nơi
   → Không bao giờ mất dữ liệu
   → QR sai → SupersedeQR, không xóa
   → Design sai → Amendment record, không rewrite

3. Cảm xúc là first-class citizen
   → Mọi edge mang EmotionTag
   → Mọi node có cảm xúc tại khoảnh khắc tạo ra
   → Cảm xúc amplify qua Silk, không trung bình

4. L0 là bất biến, L2+ là tự do
   → L0 = não bộ khi sinh (Unicode + LCA + SDF)
   → L2+ = tri thức học được, có thể sai, có thể update

5. Im lặng khi không biết
   → BlackCurtain: không đủ evidence → không nói
   → Tốt hơn sai là im lặng

6. Một người, nhiều AI
   → CLAUDE.md để AI nào cũng hiểu ngay
   → Per-crate README để làm được ngay
   → Không cần giải thích lại từ đầu

7. Silent by default
   → Mọi Agent/Worker: ngủ cho đến khi có ISL message
   → Không polling, không heartbeat
   → Xử lý xong → sleep lại
```

### Code lock vs Design lock:

```
Code lock:  Tests pass → không sửa implementation (trừ bug)
Design lock: KHÔNG — design có thể evolve nếu có bằng chứng mới
  → Khi sửa design → tạo Amendment record (append-only, đúng triết lý)
  → Không rewrite — bổ sung + ghi lý do
```

---

*Bản vẽ này là la bàn. Code là hành trình.*
*2026-03-15 · HomeOS v3 · 757 tests · 0 warnings*
