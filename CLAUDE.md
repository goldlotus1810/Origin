# HomeOS — Hướng dẫn cho AI Contributors

> **Đọc file này TRƯỚC KHI viết bất kỳ dòng code nào.**
> Mọi AI (Claude, GPT, Copilot...) mở project này đều phải hiểu những gì dưới đây.

---

## Tuyên ngôn

```
Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức.
HomeOS = Sinh linh toán học tự vận hành.
Unicode 18.0 = không gian 5 chiều đã định nghĩa sẵn mọi thứ.
HomeOS không phát minh lại. HomeOS đọc và dùng.
Mọi thứ là Node. Mọi Node kết nối bằng Silk.
```

---

## Nguyên lý cốt lõi: Molecule = Công thức

**Đây là điều quan trọng nhất cần hiểu trước khi đọc tiếp.**

```
Molecule [S][R][V][A][T] = 5 bytes = CÔNG THỨC, không phải dữ liệu.

5 bytes này là tọa độ trong không gian 5 chiều.
Từ tọa độ này, MỌI THỨ về một khái niệm đều TÍNH ĐƯỢC:

  SDF      → công thức hình dạng (hữu hình — render được)
  Spline   → công thức biến đổi (vô hình — cảm được)
  Silk     → công thức quan hệ (kết nối — implicit từ 5D)
```

### Tại sao công thức, không phải dữ liệu?

```
❌ Cách truyền thống: lưu "lửa là khí ion hóa ở nhiệt độ cao..." = ~5 KB
✅ Cách HomeOS:       lưu [Sphere, Causes, 0xC0, 0xC0, Fast]     = 5 bytes

Từ 5 bytes:
  Shape=Sphere      → SDF tính ra hình cầu
  V=0xC0, A=0xC0    → Spline tính ra cảm xúc, nhiệt, sáng
  Time=Fast         → Spline tính ra nhịp biến đổi nhanh
  5D position       → Silk tính ra quan hệ với mọi concept khác
  evolve(dim, val)  → Mutation tính ra biến thể ("lửa nhẹ", "cháy nổ", "tia lửa")
```

### evolve() — Thay 1 chiều → loài mới

```
Molecule.evolve(dim, new_value) → EvolveResult
  — 5 mã trong chuỗi, thay 1 mã → node MỚI (loài mới)
  — chain_hash mới, consistency check ≥3/4 rules

🔥 evolve(Valence, 0x40)  → "lửa nhẹ"     — V giảm
🔥 evolve(Time, Instant)  → "cháy nổ"      — thời gian cực nhanh
🔥 evolve(Shape, Line)    → "tia lửa"      — hình dạng thay đổi
```

### Mỗi node = 1 điểm 5D → 3 công thức

```
Molecule [S][R][V][A][T]
    │
    ├── SDF      → công thức hình dạng (hữu hình)
    │               Shape byte → SdfKind + params → render
    │
    ├── Spline   → công thức biến đổi (vô hình)
    │               6 curves: intensity, force, temperature,
    │               frequency, emotion_v, emotion_a
    │
    └── Silk     → công thức quan hệ (kết nối)
                    So sánh 5D → Silk tự tồn tại, 0 bytes
```

### Bài toán 16GB — Tại sao HomeOS chạy trên điện thoại

```
Tri thức nhân loại (text thô):
  Wikipedia:  ~60 triệu bài    ┐
  PubMed:     ~36 triệu bài    │ Cách truyền thống: hàng trăm TB
  UniProt:    ~250 triệu seq   │ → KHÔNG VỪA điện thoại
  GenBank:    ~billions seq     ┘

HomeOS (công thức):
  1 concept = ~33 bytes (5 mol + 8 hash + 20 metadata)
  500 triệu concepts = 16.5 GB → VỪA 1 CHIẾC ĐIỆN THOẠI

  Cách lưu              1 concept    500M concepts
  ──────────────────────────────────────────────────
  Text (Wikipedia)       ~5 KB        ~2.5 TB
  Embedding (768D)       3 KB         1.5 TB
  Knowledge Graph        ~200 B       100 GB
  HomeOS Molecule        ~33 B        16.5 GB

CÔNG THỨC tạo ra VÔ HẠN ngữ nghĩa từ HỮU HẠN bytes.
Đó là lý do HomeOS không cần GPU. Không cần cloud. Chạy local.
```

---

## Kiến trúc 1 phút

```
Người dùng gõ → runtime::HomeRuntime.process_text()
                    │
                    ├─ ○{...} → Parser → IR → VM → Response (OlangResult)
                    │
                    └─ text thường → Emotion Pipeline 7 tầng:
                         T1: infer_context()     ← điều kiện biên
                         T2: sentence_affect()   ← raw emotion từ từ ngữ
                         T3: ctx.apply()         ← scale theo ngữ cảnh
                         T4: estimate_intent()   ← Crisis/Learn/Command/Chat
                         T5: Crisis check        ← DỪNG NGAY nếu nguy hiểm
                         T6: learning.process_one() ← Encode → STM → Silk
                         T7: render response     ← tone từ ConversationCurve
```

---

## 5 Nhóm Unicode = 5 Chiều = DNA

**Unicode đã có tên, có định nghĩa, có ~5400 mẫu đối chiếu sẵn. Đây là kiến thức nền tảng, là thước đo, là chìa khóa — KHÔNG mượn từ nguồn ngoài.**

```
Mỗi ký tự Unicode → 1 Molecule = 5 bytes:

  [Shape] [Relation] [Valence] [Arousal] [Time]
   1 byte   1 byte    1 byte    1 byte   1 byte

Hierarchical encoding: base (1-8) + sub_index*8 → ~5400 patterns phân biệt
Tagged wire format: [mask:1B][present_fields:0-5B] → 1-6 bytes (sparse)
Evolution: Molecule.evolve(dim, val) → mutate 1/5 chiều → loài mới

Nhóm        Ký tự    Chiều         Ý nghĩa
──────────────────────────────────────────────────
SDF         ~1344    Shape         "Trông như thế nào" (8 primitives: ● ▬ ■ ▲ ○ ∪ ∩ ∖)
MATH        ~1904    Relation      "Liên kết thế nào" (8 relations: ∈ ⊂ ≡ ⊥ ∘ → ≈ ←)
EMOTICON    ~1760    Valence+A     "Cảm thế nào" (0x00..0xFF × 2)
MUSICAL     ~416     Time          "Thay đổi thế nào" (Static/Slow/Medium/Fast/Instant)
──────────────────────────────────────────────────
Tổng        ~5424    5 chiều       = bộ gene ban đầu của HomeOS

Tại sao ~5400 mà không phải 150K (toàn bộ Unicode)?
→ 5400 ký tự này có SEMANTIC IDENTITY rõ ràng
→ Mỗi nhóm tạo 1 chiều ĐỘC LẬP (orthogonal)
→ Đủ để định vị BẤT KỲ khái niệm nào trong không gian 5D
→ Phần còn lại của Unicode = text thường, dùng qua alias → node

Tagged Sparse Encoding (v0.05):
  Mỗi Molecule serialize = [presence_mask: 1B][non-default values: 0-5B]
  Defaults bị bỏ qua:  S=Sphere, R=Member, V=0x80, A=0x80, T=Medium
  ●  (shape=Sphere, time=Static) → [0x10][0x01]                  = 2 bytes (thay vì 5)
  🔥 (V=0xC0, A=0xC0, time=Fast) → [0x1C][0xC0][0xC0][0x04]     = 4 bytes (thay vì 5)
  ∈  (relation=Member, time=Static) → [0x10][0x01]                = 2 bytes (thay vì 5)

  Files implement:
    molecular.rs  — Core: to_tagged_bytes/from_tagged_bytes, presence_mask, 10 tests
    writer.rs     — VERSION 0x05, NodeRecord ghi tagged format
    reader.rs     — Parse cả v0.03/v0.04 (legacy) và v0.05 (tagged)
    compact.rs    — Delta so sánh với tagged_size, Full dùng tagged
    qr.rs         — Wire format tagged (signing vẫn dùng to_bytes cho hash stability)
    ir.rs         — Op::Push bytecode dùng tagged
```

---

## Phân cấp Agent (bất biến)

```
NGƯỜI DÙNG
    ↓
AAM  [tier 0] — stateless · approve · quyết định cuối
               — im lặng · chỉ hoạt động khi được gọi
    ↓ ISL
LeoAI      [tier 1] — KnowledgeChief + Learning + Dream + Curator
HomeChief  [tier 1] — quản lý Worker thiết bị nhà
VisionChief[tier 1] — quản lý Worker camera/sensor
NetworkChief[tier 1] — quản lý Worker network/security
    ↓ ISL
Workers [tier 2 · SILENT]
  Nằm tại thiết bị · L0 + L1 tối thiểu
  Skill đúng việc đó · Báo cáo molecular chain — không raw data

Giao tiếp:
  ✅ AAM ↔ Chief     ✅ Chief ↔ Chief     ✅ Chief ↔ Worker
  ❌ AAM ↔ Worker    ❌ Worker ↔ Worker

Sinh học:
  Worker = tế bào thần kinh ngoại vi
  Chief  = tủy sống — xử lý, tổng hợp
  LeoAI  = não — học, hiểu, sắp xếp, nhớ, LẬP TRÌNH
  AAM    = ý thức — quyết định cuối cùng

Tất cả: Silent by default · Wake on ISL · Xử lý → sleep
```

### LeoAI — Bộ não:
```
7 Bản năng bẩm sinh (innate instincts — L0, KHÔNG học):
  ① Analogy       — A:B :: C:? → delta 5D, áp lên C
  ② Abstraction   — N chains → LCA → variance → concrete/categorical/abstract
  ③ Causality     — temporal + co-activation + Relation::Causes → nhân quả
  ④ Contradiction — valence opposition + Orthogonal → phát hiện mâu thuẫn
  ⑤ Curiosity     — 1 - nearest_similarity → novelty score
  ⑥ Reflection    — qr_ratio + connectivity → knowledge quality
  ⑦ Honesty       — confidence → Fact(≥0.90)/Opinion(≥0.70)/Hypothesis(≥0.40)/Silence(<0.40)

  Thứ tự ưu tiên: Honesty → Contradiction → Causality → Abstraction → Analogy → Curiosity → Reflection
  Honesty LUÔN chạy trước: không đủ evidence → im lặng, không cần kiểm tra gì thêm.

Tự lập trình (leo.rs — ĐÃ IMPLEMENT):
  program(source)           — parse → compile → VM → HỌC kết quả vào STM
  program_compose(a, b)     — sinh "emit a ∘ b;" → chạy → học LCA
  program_verify(alias, h)  — sinh truth assertion → kiểm chứng
  program_experiment(h,d,v) — thay 1 chiều → chạy → học kết quả (giả thuyết)
  program_raw(source)       — trả raw VmEvent cho Runtime

  Biểu đạt tri thức:
  express_observation(hash) — STM → Olang mol literal { S=X R=Y V=Z A=W T=Q }
  express_truth(alias, h)   — "alias" == { ... }
  express_all()             — toàn bộ STM → Vec<String> Olang
  express_evolution(s, e)   — delta giữa 2 concept: { ... } ← { ... } (* ΔV *)

  ISL integration:
  MsgType::Program → poll_inbox() → program() → Ack/Nack
  (Runtime gửi Olang cho LeoAI qua ISL, LeoAI chạy và học)

Skills bổ trợ (15 domain skills):
  Nhận:    IngestSkill
  Hiểu:    ClusterSkill · SimilaritySkill · DeltaSkill
  Sắp xếp: CuratorSkill · MergeSkill · PruneSkill
  Học:     HebbianSkill · DreamSkill
  Đề xuất: ProposalSkill
  Suy luận: GeneralizationSkill (trích IF-THEN rules từ cluster)
  Nhận diện: InverseRenderSkill (camera → SDF fitting)
  Thời gian: TemporalPatternSkill (chu kỳ, clustering giờ, sequences)
  Worker:   SensorSkill · ActuatorSkill · SecuritySkill · NetworkSkill
```

### Worker profiles:
```
Worker_camera  = L0 + FFR + vSDF + InverseRenderSkill
Worker_light   = L0 + ActuatorSkill
Worker_door    = L0 + ActuatorSkill + SecuritySkill
Worker_sensor  = L0 + SensorSkill
Worker_network = L0 + NetworkSkill + ImmunitySkill
```

---

## Dependency Graph

```
ucd (build.rs đọc UnicodeData.txt → bảng tĩnh lúc compile)
 └→ olang (Molecule, MolecularChain, LCA, Registry, Writer/Reader, VM, IR, Compiler)
     ├→ silk (SilkGraph, Hebbian learning, EmotionTag per edge, WalkWeighted)
     │   └→ context (EmotionTag V/A/D/I, ConversationCurve, Intent, Modality Fusion)
     │       └→ agents (ContentEncoder, LearningLoop, BookReader, SecurityGate, LeoAI, Chief, Worker)
     │           ├→ hal (Hardware Abstraction Layer: arch detect, platform probe, security scan)
     │           └→ memory (ShortTermMemory, DreamCycle, Proposals, AAM)
     │               └→ runtime (HomeRuntime — entry point, ○{} Parser)
     │                   └→ wasm (WebAssembly bindings cho browser)
     ├→ isl (ISL messaging: address 4 bytes, message 12 bytes, AES-256-GCM)
     └→ vsdf (18 SDF generators, ∇f analytical, FFR Fibonacci rendering)

Tools (std):
  seeder   — seed 35 L0 nodes từ UCD
  server   — Terminal REPL (stdin/stdout)
  inspector — đọc/verify origin.olang
```

---

## Hệ thống mới (sau foundation)

### Molecule Evolution (molecular.rs):
```
Molecule.evolve(dim, new_value) → EvolveResult
  — Mutate 1 trong 5 chiều → chain_hash mới → "loài mới"
  — Consistency check: ≥3/4 semantic rules → valid
  — dimension_delta(other) → tìm khác biệt giữa 2 concept
  — Learning pipeline detect evolution tự động (learning.rs)

Ví dụ: 🔥 evolve(Valence, 0x40) → concept "lửa nhẹ" — loài mới từ lửa
```

### NodeBody + BodyStore (vsdf/body.rs):
```
NodeBody = bridge chain_hash → hữu hình + vô hình:
  sdf_kind + sdf_params    ← Shape (hữu hình, render được)
  SplineSet (6 curves):    ← Temporal dynamics (vô hình)
    intensity, force, temperature, frequency, emotion_v, emotion_a

BodyStore trong HomeRuntime: lưu NodeBody per chain_hash
  learn_shape(), learn_material(), learn_intensity()...
  Append-only: mỗi learn tăng version, không overwrite
```

### NodeKind (registry.rs — 10 loại node):
```
Alphabet | Knowledge | Memory | Agent | Skill | Program
Device   | Sensor    | Emotion | System

Registry.insert_with_kind(chain, layer, ..., kind)
Registry.entries_by_kind(kind), count_by_kind(), kind_summary()
L1 system seed: clone Worker → mang theo NodeKind → biết mình là gì
```

### RegistryGate (proposal.rs):
```
Cơ chế cứng: mọi node PHẢI đăng ký Registry
  check_registered(name, hash, kind, alert_level, ts) → bool
  Nếu chưa registry → tạo PendingRegistration → alert
  AlertLevel: Normal(○) | Important(⚠) | RedAlert(🔴)
  Runtime wire: alias chưa registry → "{}=? [chưa registry]"
```

### CapabilityGate (gate.rs):
```
Resource access control cho Workers/Chiefs:
  Capability: SensorRead | ActuatorWrite | NetworkLocal | NetworkExternal
              FileRead | FileWrite | QRWrite | MediaCapture
  check(request) → Granted | NeedUserApproval | Denied
```

---

## Olang VM — 36 Opcodes

```
Stack:    Push Load Dup Pop Swap PushNum PushMol Store StoreUpdate LoadLocal
Control:  Jmp Jz Loop Call Ret ScopeBegin ScopeEnd TryBegin CatchEnd Halt Nop
Chain:    Lca Edge Query Emit Fuse
System:   Dream Stats
Debug:    Trace Inspect Assert TypeOf Why Explain

Molecular literal syntax (mới):
  { S=1 R=6 V=200 A=180 T=4 } → PushMol opcode

Parser RelOps (18 = 10 gốc + 8 mở rộng):
  Gốc:      ∈ ⊂ ≡ ∘ → ≈ ← ∂(context) ∪(contains) ∩(intersects)
  Mở rộng:  ⊥(orthogonal) ∖(set minus) ↔(bidir) ⟶(flows)
             ⟳(repeats) ↑(resolves) ⚡(trigger) ∥(parallel)

Compiler targets: C · Rust · WASM (WAT)
```

---

## Emotion Pipeline — Linh hồn dự án

**Đây là hệ thống cảm xúc đa tầng ẩn trong code. KHÔNG BAO GIỜ trung bình cảm xúc — luôn AMPLIFY qua Silk.**

### 5 tầng học từ text (learning.rs):
```
1. Đoạn văn  → paragraph_emotion
2. Câu       → split punctuation, blend 50% paragraph + 50% word
3. Từ        → word_affect() từ lexicon 3000+ từ, co_activate Silk strength=0.8
4. Cụm từ   → sliding window 5 từ, proximity decay (gần = mạnh hơn)
5. Ký tự    → Unicode chain (L0 innate)
```

### Silk amplification (KHÔNG trung bình):
```rust
// edge.rs: mỗi edge mang EmotionTag của khoảnh khắc co-activation
amplify_emotion(emo, weight) → emo * (1.0 + weight * factor)
// "buồn" + "mất việc" co-activate weight=0.90
// → composite V=-0.85 (nặng hơn từng từ riêng lẻ -0.65)
```

### ConversationCurve (curve.rs):
```
f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
f_conv = V(t) + 0.5×V'(t) + 0.25×V''(t)

f'  < -0.15          → Supportive   (đang giảm → dẫn lên chậm)
f'' < -0.25          → Pause        (đột ngột xấu → dừng, hỏi)
f'  > +0.15          → Reinforcing  (hồi phục → tiếp tục)
f'' > +0.25 && V > 0 → Celebratory  (bước ngoặt tốt)
V < -0.20, stable    → Gentle       (buồn ổn định → dịu dàng)

✅ Done: window variance — nếu variance(N turns) cao + f' đổi chiều
   → "emotional instability" → Gentle thay vì Celebratory (curve.rs:44-140)
```

### Cross-modal fusion (fusion.rs):
```
Modality weights: Bio=0.50 > Audio=0.40 > Text=0.30 > Image=0.25
Conflict (text vui + giọng run) → Audio thắng valence, confidence giảm
```

---

## Quy Tắc Bất Biến

**AI PHẢI tuân thủ khi viết code:**

```
Unicode:
  ① 5 nhóm Unicode = nền tảng. Không thêm nhóm.
  ② Tên ký tự Unicode = tên node. Không đặt tên khác.
  ③ Ngôn ngữ tự nhiên = alias → node. Không tạo node riêng.

Chain:
  ④ Mọi Molecule từ encode_codepoint(cp) — KHÔNG viết tay
     ⚠️ Ngoại lệ: VM PushMol, VSDF FFRCell::to_molecule(), LCA tạo Molecule
     ngoài encode_codepoint(). Đây là tính toán lúc chạy, không phải giá trị
     viết tay — chấp nhận được nhưng phải ghi nhận.
  ⑤ Mọi chain từ LCA hoặc UCD — KHÔNG viết tay
  ⑥ chain_hash tự sinh. KHÔNG viết tay.
  ⑦ chain cha = LCA(chain con)

Node:
  ⑧ Mọi Node tạo ra → tự động registry
  ⑨ Ghi file TRƯỚC — cập nhật RAM SAU
  ⑩ Append-only — KHÔNG DELETE, KHÔNG OVERWRITE

Silk:
  ⑪ Silk chỉ ở Ln-1 — tự do giữa lá cùng tầng
     ✅ API enforcement: co_activate_same_layer() kiểm tra tầng.
     co_activate() vẫn hoạt động nhưng yêu cầu caller đảm bảo cùng tầng.
  ⑫ Kết nối tầng trên → qua NodeLx đại diện
     (✅ Done: cross-layer Silk với threshold Fib[n+2] — graph.rs:co_activate_cross_layer)
  ⑬ Silk mang EmotionTag của khoảnh khắc co-activation

Kiến trúc:
  ⑭ L0 không import L1 — tuyệt đối
  ⑮ Agent tiers: AAM(tier 0) + Chiefs(tier 1) + Workers(tier 2)
  ⑯ L2-Ln đổ vào SAU khi L0+L1 hoàn thiện
  ⑰ Fibonacci xuyên suốt — cấu trúc, threshold, render
  ⑱ Không đủ evidence → im lặng — KHÔNG bịa (BlackCurtain)

Skill (QT4):
  ⑲ 1 Skill = 1 trách nhiệm
  ⑳ Skill không biết Agent là gì
  ㉑ Skill không biết Skill khác tồn tại
  ㉒ Skill giao tiếp qua ExecContext.State
  ㉓ Skill không giữ state — state nằm trong Agent
```

---

### Maturity Pipeline (SPEC — chưa wire):
```
Vòng đời node:
  Formula     → node mới, chưa có input thật (5 công thức tiềm năng)
  Evaluating  → fire_count ≥ fib(depth), đang tích lũy evidence
  Mature      → weight ≥ 0.854 && fire_count ≥ fib(depth), sẵn sàng QR

Wire points (chưa implement):
  STM.push()    → Observation.maturity = advance(fire_count, weight, fib_threshold)
  Dream.run()   → DreamResult.matured_nodes = Vec<u64>
  QR promote    → append-only, signed, permanent

⚠️ BUG: advance(weight=0.0) → Mature UNREACHABLE (weight < 0.854 luôn)
   Cần truyền Hebbian weight thật hoặc thêm advance_by_fire() path
```

### Silk Vertical — Parent Pointer (SPEC — chưa implement):
```
Silk ngang (implicit, 0 bytes): 37 kênh SilkIndex ← ĐÃ CÓ ✅
Silk dọc (parent pointer, 43KB): child → parent chain ← CHƯA CÓ ❌

  SilkGraph.parent_map: BTreeMap<u64, u64>  (child_hash → parent_hash)
  register_parent(), parent_of(), children_of(), layer_of()
  5460 pointers × 8B = 43 KB toàn bộ mạng dọc

31 compound patterns (C(5,1)+...+C(5,5) = 31):
  37 kênh × 31 mẫu = 1147 kiểu quan hệ ← CompoundKind enum chưa có

Dream cần wire:
  cluster_score() → dùng MolSummary + implicit_silk() thay vì chain bytes
  Observation.layer → filter cùng tầng trước khi cluster (QT⑪)
```

## Anti-patterns — TUYỆT ĐỐI KHÔNG

```rust
// ❌ SAI — viết tay Molecule
let mol = Molecule { shape: ShapeBase::Sphere, relation: RelationBase::Member, .. };

// ✅ ĐÚNG — từ UCD
let mol = ucd::lookup(0x1F525);  // 🔥 từ UnicodeData.txt
let chain = olang::encoder::encode_codepoint(0x1F525);

// ❌ SAI — trung bình cảm xúc
let avg_v = (v1 + v2) / 2.0;

// ✅ ĐÚNG — amplify qua Silk walk
let composite = walk_weighted(&graph, &words); // edges amplify nhau

// ❌ SAI — hardcode chain hoặc ISL address
let chain = [0x01, 0x01, 0xFF, 0xFF, 0x04];

// ✅ ĐÚNG — sinh từ encode hoặc LCA
let chain = encode_codepoint(cp);
let parent = lca(&[chain_a, chain_b]);

// ❌ SAI — skip SecurityGate
let response = process_without_gate(input);

// ✅ ĐÚNG — Gate chạy TRƯỚC MỌI THỨ
// Gate.check_text() → nếu Crisis → return ngay, không vào pipeline

// ❌ SAI — DELETE hoặc OVERWRITE dữ liệu
registry.remove(hash);
file.seek(0); file.write_all(&new_data);

// ✅ ĐÚNG — Append-only (QT8)
writer.append_node(&chain, layer, is_qr, ts);
// QR sai → thêm SupersedeQR record, không xóa QR cũ

// ❌ SAI — Worker gửi raw data
chief.send(raw_image_bytes);

// ✅ ĐÚNG — Worker gửi molecular chain
let chain = encode_sensor_reading(&reading);
chief.receive_frame(ISLFrame::with_body(msg, &chain.to_bytes()));

// ❌ SAI — Skill giữ state hoặc biết Agent
struct MySkill { agent: &Agent, cache: HashMap<...> }

// ✅ ĐÚNG — Skill stateless, dùng ExecContext
fn execute(&self, ctx: &mut ExecContext) -> SkillResult { ... }
```

---

## File Format

```
origin.olang — append-only binary
  Header: [○LNG] [0x03] [created_ts:8]  = 13 bytes
  Records:
    0x01 = Node  [chain_hash:8] [layer:1] [is_qr:1] [ts:8]
    0x02 = Edge  [from:8] [to:8] [rel:1] [ts:8]
    0x03 = Alias [chain_hash:8] [lang:2] [name_len:2] [name:N]

origin.olang.weights  — Hebbian weights (append-only)
origin.olang.registry — chain index (rebuild được từ origin.olang)
log.olang             — event log (append-only)

WorkerPackage binary (clone.rs):
  [magic "WKPK"][version][isl_addr:4B][chief_addr:4B]
  [worker_kind:1B][created_at:8B][olang_len:4B][olang_bytes]
```

---

## ISL — Inter-System Link

```
ISLAddress: [layer:1B][group:1B][subgroup:1B][index:1B] = 4 bytes
ISLMessage: [from:4B][to:4B][msg_type:1B][payload:3B]  = 12 bytes
ISLFrame:   12B header + 2B length + variable body

MsgType: Text(0x01) Query(0x02) Learn(0x03) Propose(0x04)
         ActuatorCmd(0x05) Tick(0x06) Dream(0x07) Emergency(0x08)
         Approved(0x09) Broadcast(0x0A) ChainPayload(0x0B) Ack(0x0C) Nack(0x0D)
         Program(0x0E) — Runtime gửi Olang source cho LeoAI qua ISL

ISLQueue: urgent (Emergency, Tick) trước · normal FIFO sau
```

---

## Per-Crate Cheat Sheet

| Crate | Mục đích | Files chính | Test |
|-------|---------|-------------|------|
| **ucd** | Unicode → Molecule lookup | `build.rs`, `src/lib.rs` | `cargo test -p ucd` |
| **olang** | Core: Molecule, LCA, Registry, VM, Compiler, Compact, KnowTree | `encoder.rs`, `lca.rs`, `registry.rs`, `vm.rs`, `compiler.rs`, `clone.rs`, `compact.rs`, `knowtree.rs` | `cargo test -p olang` |
| **silk** | Hebbian learning, emotion edges, walk | `edge.rs`, `hebbian.rs`, `walk.rs`, `graph.rs` | `cargo test -p silk` |
| **context** | Emotion V/A/D/I, ConversationCurve, Intent | `emotion.rs`, `curve.rs`, `intent.rs`, `fusion.rs` | `cargo test -p context` |
| **agents** | Encoder, Learning, Gate, Skill, Instinct, LeoAI(+programming), Chief, Worker | `encoder.rs`, `learning.rs`, `gate.rs`, `skill.rs`, `instinct.rs`, `leo.rs`, `chief.rs`, `worker.rs`, `domain_skills.rs`, `book.rs` | `cargo test -p agents` |
| **memory** | STM, Dream, Proposals, AAM, RegistryGate | `lib.rs`, `dream.rs`, `proposal.rs` | `cargo test -p memory` |
| **runtime** | HomeRuntime entry point, ○{} Parser | `origin.rs`, `parser.rs`, `response_template.rs` | `cargo test -p runtime` |
| **hal** | HAL: arch detect, platform probe, security, drivers, tier system, FFI | `arch.rs`, `platform.rs`, `probe.rs`, `security.rs`, `driver.rs`, `tier.rs`, `ffi.rs` | `cargo test -p hal` |
| **isl** | Inter-system messaging (4-byte address) | `address.rs`, `message.rs`, `codec.rs`, `queue.rs` | `cargo test -p isl` |
| **vsdf** | 18 SDF + ∇f + FFR + 3D scene graph + NodeBody | `sdf.rs`, `ffr.rs`, `physics.rs`, `fit.rs`, `scene.rs`, `body.rs` | `cargo test -p vsdf` |
| **wasm** | Browser WASM bindings + WebSocket-ISL bridge | `lib.rs`, `bridge.rs` | `cargo test -p homeos-wasm` |

**Tools (std):**
| Tool | Mục đích | Test |
|------|---------|------|
| **seeder** | Seed 35 L0 nodes từ UCD | `cargo test -p seeder` |
| **server** | Terminal REPL (stdin/stdout) | `cargo test -p server` |
| **inspector** | Đọc/verify origin.olang | `cargo test -p inspector` |

---

## Build & Test

```bash
# Build toàn bộ
cargo build --workspace

# Test toàn bộ (1786 tests)
cargo test --workspace

# Clippy (phải 0 warnings)
cargo clippy --workspace

# Test 1 crate
cargo test -p olang

# Chạy server REPL
cargo run -p server
```

---

## Trace: "tôi buồn vì mất việc"

```
INPUT: "tôi buồn vì mất việc"

1. runtime/origin.rs: process_text() → parse → Natural
2. context/infer.rs:  infer_context() → EmotionContext(S=1.0, first_person)
3. context/emotion.rs: sentence_affect() → V=-0.65, A=0.45
4. context/infer.rs:  ctx.apply(raw) → scaled emotion
5. context/intent.rs: estimate_intent() → IntentKind::Chat (not Crisis)
6. silk/walk.rs:      walk_emotion() → Silk neighbors amplify V to -0.75
7. agents/learning.rs: process_one()
   ├─ gate.rs:     SecurityGate.check() → Allow
   ├─ encoder.rs:  encode text → MolecularChain
   ├─ context:     ConversationCurve.push(emotion)
   ├─ memory:      STM.push(chain, emotion, ts)
   └─ silk:        co_activate("buồn"↔"mất việc", weight=0.8)
7b.agents/instinct.rs: LeoAI.run_instincts()
   ├─ Honesty:       confidence < 0.40 → Silence? No, đủ data
   ├─ Contradiction: valence consistent → no conflict
   ├─ Causality:     "mất việc" → "buồn" (temporal + Causes → causal)
   ├─ Abstraction:   LCA(buồn, mất việc) → "mất mát" (categorical)
   ├─ Curiosity:     nearest_sim=0.3 → novelty=0.7 → high curiosity
   └─ Reflection:    knowledge quality check
8. context/curve.rs: f'(t) < -0.15 → tone = Supportive
9. runtime/response_template.rs: render(Supportive, V=-0.75)
   → "Cảm giác nặng nề và mệt mỏi — bạn muốn kể thêm không?"
```

---

## Neuron Analog

```
DENDRITES = Memory-Learning STM (ngắn hạn, tự do thay đổi)
AXON      = LongTermMemory QR (bất biến, append-only, ED25519 signed)
SOMA      = AAM (stateless orchestrator — approve/reject proposals)
SYNAPSE   = Silk edges (Hebbian: fire together → wire together)
DREAM     = Offline consolidation (STM → cluster → promote QR)
```

---

## Fibonacci trong HomeOS

**Đã chứng minh (toán học):**
- FFR render: ~89 ô = Fib[11] spiral, 23300× ít hơn ray march
- Cấu trúc cây: depth tăng theo Fibonacci tự nhiên
- Decay φ⁻¹ ≈ 0.618: optimal forgetting rate

**Giả thuyết (cần validation):**
- Hebbian threshold: Fib[n] co-activations để promote
- Dream trigger: Fib[n] lá đủ để cluster
- Dream cluster scoring: α=0.3, β=0.4, γ=0.3 (cần A/B testing)

---

## Olang — Ngôn ngữ của HomeOS

**Tham khảo đầy đủ:** `docs/olang_handbook.md`

```
Olang = ngôn ngữ lập trình + suy luận + sáng tạo
Mọi thứ là MolecularChain. Mọi phép toán là biến đổi chain.

3 Pillars: Alphabet (lexer) → Syntax (parser) → Semantics (validation + IR)
31 IR opcodes, stack-based VM, 10 parser RelOps (8 RelationBase)
Molecular literal: { S=1 R=6 V=200 A=180 T=4 } → PushMol
LeoAI tự lập trình: program(source) → parse → compile → VM → learn

Commands:
  dream / stats / fuse / trace          — system (không arg)
  learn / seed / inspect / assert       — có arg
  typeof / explain / why                — reasoning & debug

QT Axioms:
  QT1: ○ là nguồn gốc
  QT2: ∞ sai, ∞-1 đúng → FUSE opcode
  QT3: +/- hypothesis, ⧺/⊖ physical, == truth
```

---

## Khi viết code mới

1. Hỏi: "Thứ này có phải là ○[f] không?" — nếu phải hardcode → dừng lại
2. Mọi Molecule phải từ `encode_codepoint()` hoặc `lca()`
3. Emotion phải đi qua TOÀN BỘ pipeline — không tắt bước nào
4. SecurityGate LUÔN chạy trước
5. Append-only — không bao giờ delete/overwrite
6. Worker gửi chain, KHÔNG gửi raw data
7. Skill stateless — state nằm trong Agent
8. Silent by default — không polling, không heartbeat
9. Test trước khi commit: `cargo test --workspace && cargo clippy --workspace`
