# SPEC: Node & Silk

> Tài liệu kỹ thuật — mô tả cấu trúc Node và Silk trong HomeOS.
> Dựa trên source code thực tế + thiết kế từ `node va silk.md`.

---

## 1. Molecule — 5 bytes = Tọa độ 5D

### 1.1 Struct (molecular.rs)

```rust
pub struct Molecule {
    pub shape: u8,              // Chiều hình dạng (hierarchical byte)
    pub relation: u8,           // Chiều quan hệ (hierarchical byte)
    pub emotion: EmotionDim,    // Valence + Arousal (2 bytes)
    pub time: u8,               // Chiều thời gian (hierarchical byte)
}
// RAM: 5 bytes cố định
// Wire: 1-6 bytes (tagged sparse encoding)
```

### 1.2 Năm chiều

| Chiều | Enum | Bases | Sub-variants | Encoding |
|-------|------|-------|-------------|----------|
| **Shape** | `ShapeBase` | 8 | 31/base | `base + sub_index * 8` |
| **Relation** | `RelationBase` | 8 | 31/base | `base + sub_index * 8` |
| **Valence** | `u8` | 256 levels | — | `0x00`=V− `0x7F`=V0 `0xFF`=V+ |
| **Arousal** | `u8` | 256 levels | — | `0x00`=calm `0xFF`=excited |
| **Time** | `TimeDim` | 5 | 51/base | `base + sub_index * 5` |

**ShapeBase (8 primitives):**
```
Sphere=0x01  Capsule=0x02  Box=0x03  Cone=0x04
Torus=0x05   Union=0x06    Intersect=0x07  Subtract=0x08
```

**RelationBase (8 relations):**
```
Member=0x01  Subset=0x02  Equiv=0x03  Orthogonal=0x04
Compose=0x05 Causes=0x06  Similar=0x07  DerivedFrom=0x08
```

**TimeDim (5 tempos):**
```
Static=0x01  Slow=0x02  Medium=0x03  Fast=0x04  Instant=0x05
```

### 1.3 Tagged Sparse Encoding (v0.05)

```
Wire format: [presence_mask: 1B][non-default values: 0-5B]

Defaults (bỏ qua nếu bằng):
  Shape=0x01(Sphere)  Relation=0x01(Member)
  Valence=0x80        Arousal=0x80          Time=0x03(Medium)

Presence mask bits:
  bit 0: PRESENT_SHAPE     (0x01)
  bit 1: PRESENT_RELATION  (0x02)
  bit 2: PRESENT_VALENCE   (0x04)
  bit 3: PRESENT_AROUSAL   (0x08)
  bit 4: PRESENT_TIME      (0x10)

Ví dụ:
  ●  (all defaults, time=Static)     → [0x10][0x01]           = 2 bytes
  🔥 (V=0xC0, A=0xC0, time=Fast)    → [0x1C][0xC0][0xC0][0x04] = 4 bytes
```

### 1.4 MolecularChain

```rust
pub struct MolecularChain(pub Vec<Molecule>);
// Chuỗi molecules = DNA của một khái niệm
// chain_hash() → FNV-1a u64 — địa chỉ duy nhất

// Key methods:
fn chain_hash(&self) -> u64           // FNV-1a hash
fn to_bytes(&self) -> Vec<u8>        // len × 5 bytes
fn to_tagged_bytes(&self) -> Vec<u8> // sparse encoding
fn from_number(n: f64) -> Self       // encode f64 → 4 molecules
fn evolve_at(idx, dim, val) -> Option<EvolveResult>
fn apply_evolution(idx, result) -> Option<Self>
fn evolve_and_apply(idx, dim, val) -> Option<(Self, EvolveResult)>
```

### 1.5 Evolution — Mutate 1 chiều → Loài mới

```rust
pub enum Dimension { Shape, Relation, Valence, Arousal, Time }

pub struct EvolveResult {
    pub molecule: Molecule,     // Molecule sau evolve
    pub dimension: Dimension,   // Chiều đã thay đổi
    pub old_value: u8,
    pub new_value: u8,
    pub consistency: u8,        // 0-4: bao nhiêu chiều còn lại hợp lệ
    pub valid: bool,            // consistency >= 3
}

// Molecule.evolve(dim, new_value) → EvolveResult
// Thay 1 trong 5 chiều → chain_hash MỚI → "loài mới"
// Consistency check: ≥3/4 chiều còn lại vẫn ok → valid

// Ví dụ:
// 🔥 evolve(Valence, 0x40)  → "lửa nhẹ"     (V giảm)
// 🔥 evolve(Time, Instant)  → "cháy nổ"      (thời gian cực nhanh)
// 🔥 evolve(Shape, Line)    → "tia lửa"      (hình dạng thay đổi)
```

### 1.6 Maturity — Vòng đời Molecule

```rust
pub enum Maturity {
    Formula    = 0x00,  // Tiềm năng — 5 chiều là CÔNG THỨC, chưa evaluate
    Evaluating = 0x01,  // Đang đánh giá — có evidence, tích lũy
    Mature     = 0x02,  // Chín — đủ evidence, sẵn sàng QR
}

// Chuyển trạng thái:
//   Formula → Evaluating:  khi fire_count > 0
//   Evaluating → Mature:   khi weight ≥ 0.854 (φ⁻¹+φ⁻³) AND fire_count ≥ Fib[depth]
//   Mature → Mature:       irreversible
```

**Ý nghĩa triết học:**
- **Formula**: DNA lưu CÔNG THỨC, không phải giá trị. Mỗi chiều = f(x), chưa biết x.
- **Evaluating**: Có input → thế vào công thức → tích lũy evidence.
- **Mature**: Đủ evidence → 5 chiều "đông đặc" → candidate cho QR (bất tử).

---

## 2. Node — Đơn vị tồn tại trong HomeOS

### 2.1 Node = MolecularChain + Registry + Body

Một Node không phải 1 struct đơn lẻ — nó là **tổ hợp** từ nhiều nơi:

```
Node = {
    MolecularChain  ← chuỗi 5-byte molecules (DNA)
    RegistryEntry   ← metadata trong sổ cái
    NodeBody        ← SDF + Spline (hữu hình + vô hình) [optional]
    Observation     ← STM state (nếu đang trong trí nhớ ngắn hạn)
    Silk positions  ← implicit từ 5D (0 bytes)
}
```

### 2.2 RegistryEntry — Metadata trong sổ cái

```rust
pub struct RegistryEntry {
    pub chain_hash: u64,    // FNV-1a hash — địa chỉ duy nhất
    pub layer: u8,          // Tầng: L0=0, L1=1, ..., L7=7
    pub file_offset: u64,   // Offset trong origin.olang
    pub created_at: i64,    // Timestamp (nanoseconds)
    pub is_qr: bool,        // false=ĐN (đang học), true=QR (bất biến)
    pub kind: NodeKind,     // Phân loại theo chức năng
}
```

### 2.3 NodeKind — 10 loại node

```rust
pub enum NodeKind {
    Alphabet  = 0,  // L0 Unicode — innate, immutable (35 seeded)
    Knowledge = 1,  // Kiến thức đã học, concepts, truths
    Memory    = 2,  // STM observations, trí nhớ ngắn hạn
    Agent     = 3,  // AAM, LeoAI, Chief, Worker definitions
    Skill     = 4,  // Stateless functions (7 instinct + 15 domain + 4 worker)
    Program   = 5,  // VM ops, built-in functions, compiler components
    Device    = 6,  // Thiết bị đang kết nối HomeOS
    Sensor    = 7,  // Cảm biến của device
    Emotion   = 8,  // Emotion states, conversation curve points
    System    = 9,  // Internal housekeeping (layer reps, branch markers)
}
```

### 2.4 NodeBody — Hữu hình + Vô hình (vsdf/body.rs)

```rust
pub struct NodeBody {
    pub chain_hash: u64,
    pub sdf_kind: Option<SdfKind>,       // SDF primitive (hữu hình)
    pub sdf_params: Option<SdfParams>,   // Tham số SDF
    pub material: Option<Material>,      // Vật liệu render
    pub transform: Option<Transform>,    // Vị trí 3D
    pub splines: SplineSet,              // 6 curves (vô hình)
    pub version: u32,                    // Append-only version counter
}
```

**SplineSet — 6 chiều vô hình:**

```rust
pub struct SplineSet {
    pub intensity: VectorSpline,     // Ánh sáng theo t
    pub force: VectorSpline,         // Gió/lực theo t
    pub temperature: VectorSpline,   // Nhiệt theo t
    pub frequency: VectorSpline,     // Âm thanh/nhịp theo t
    pub emotion_v: VectorSpline,     // Valence theo t
    pub emotion_a: VectorSpline,     // Arousal theo t
}

pub struct SplineSnapshot {
    pub intensity: f32,     // 0..1
    pub force: f32,         // 0..1
    pub temperature: f32,   // 0..1
    pub frequency: f32,     // 0..1
    pub emotion_v: f32,     // -1..+1
    pub emotion_a: f32,     // 0..1
}
```

**Ba công thức từ 1 Node:**
```
Molecule [S][R][V][A][T]
    │
    ├── SDF      → Shape byte → SdfKind + params → render (hữu hình)
    ├── Spline   → 6 curves: temporal dynamics (vô hình)
    └── Silk     → So sánh 5D → implicit connections (0 bytes)
```

### 2.5 Observation — Node trong STM

```rust
pub struct Observation {
    pub chain: MolecularChain,
    pub emotion: EmotionTag,
    pub timestamp: i64,
    pub fire_count: u32,
    pub mol_summary: Option<MolSummary>,  // 5D cache cho Silk
    pub maturity: Maturity,               // Formula → Evaluating → Mature
}
```

### 2.6 Registry — Sổ cái (registry.rs)

```rust
pub struct Registry {
    entries: Vec<(u64, RegistryEntry)>,           // hash → entry (sorted)
    names: Vec<(String, u64)>,                    // alias → hash (sorted)
    hash_to_name: Vec<(u64, String)>,             // reverse: hash → first alias
    layer_rep: [Option<u64>; 16],                 // Lx → NodeLx hash
    layer_rep_chain: [Option<MolecularChain>; 16], // Lx → representative chain
    branch_wm: Vec<(u64, u8)>,                    // branch → leaf_layer
    qr_supersede: Vec<(u64, u64)>,                // old → new QR hash
    // + bulk mode fields
}

// Thứ tự bắt buộc (QT8):
//   1. file.append(node)      ← TRƯỚC TIÊN
//   2. registry.insert(hash)  ← sau khi file OK
//   3. layer_rep.update(LCA)  ← cập nhật đại diện
//   4. silk.connect(node)     ← nối Silk
//   5. log.append(event)      ← CUỐI CÙNG

// Key methods:
fn insert(chain, layer, offset, ts, is_qr) -> u64
fn insert_with_kind(chain, layer, offset, ts, is_qr, kind) -> u64
fn register_alias(name, chain_hash)
fn lookup_hash(hash) -> Option<&RegistryEntry>
fn lookup_name(name) -> Option<u64>
fn entries_by_kind(kind) -> impl Iterator<Item = &RegistryEntry>
```

### 2.7 RegistryGate — Cơ chế cứng (proposal.rs)

```rust
pub enum AlertLevel { Normal, Important, RedAlert }

pub struct PendingRegistration {
    pub name: String,
    pub chain_hash: Option<u64>,
    pub suggested_kind: u8,
    pub alert_level: AlertLevel,
    pub reason: String,
    pub discovered_at: i64,
    pub user_response: UserConfirmation,
    pub auto_resolved: bool,
    pub qt_checked: u32,           // bitmask: bit N = QT(N+1)
}

pub struct RegistryGate {
    pending: Vec<PendingRegistration>,
    notified: Vec<PendingRegistration>,
    resolved_count: u32,
    auto_resolved_count: u32,
}

// Flow:
// 1. check_registered(name, hash, kind, alert, ts) → false nếu chưa registry
// 2. Tạo PendingRegistration → pending queue
// 3. drain_notifications() → chuyển pending → notified → AAM thông báo user
// 4. respond(index, approved) → user quyết định
// 5. drain_approved() → lấy approved registrations
// 6. RedAlert + user offline → auto_resolve_red_alerts() (đối chiếu 9 QT)
//    Ngoại lệ: L0 (Alphabet) KHÔNG được auto-resolve (QT14)

// Alert icons:
//   Normal    → ○
//   Important → ⚠
//   RedAlert  → 🔴
```

### 2.8 File Format — origin.olang

```
Header: [○LNG] [0x05] [created_ts:8]  = 13 bytes

Records (append-only):
  0x01 = Node     [chain_hash:8][layer:1][is_qr:1][ts:8]
  0x02 = Edge     [from:8][to:8][rel:1][ts:8]
  0x03 = Alias    [chain_hash:8][lang:2][name_len:2][name:N]
  0x04 = NodeKind [chain_hash:8][kind:1]

Molecule trong Node record: tagged encoding (1-6 bytes)
```

### 2.9 Node tạo ra từ đâu?

```
Nguồn gốc duy nhất:
  1. encode_codepoint(cp)  ← từ UnicodeData.txt (L0, bẩm sinh)
  2. lca(&[chain_a, chain_b, ...]) ← từ LCA (Ln, tổ hợp)
  3. VM PushMol             ← runtime computation (ngoại lệ cho phép)
  4. evolve(dim, val)       ← mutation từ node có sẵn

TUYỆT ĐỐI KHÔNG viết tay Molecule hay chain_hash.
```

---

## 3. Silk — Kết nối implicit từ không gian 5D

### 3.1 Nguyên lý

```
Silk KHÔNG PHẢI dữ liệu. Silk LÀ PHÉP SO SÁNH.

2 node chia sẻ tọa độ trên 1 chiều → Silk TỰ TỒN TẠI.
Không ai tạo. Không ai lưu. Chỉ cần NHÌN.

Sức mạnh kết nối = SỐ CHIỀU CHUNG:
  1 chiều → liên quan nhẹ
  2 chiều → liên quan rõ
  3 chiều → gần giống
  4 chiều → gần như cùng khái niệm
  5 chiều → CÙNG node
```

### 3.2 SilkGraph — 3 lớp (graph.rs)

```rust
pub struct SilkGraph {
    edges: Vec<SilkEdge>,       // Structural + legacy edges
    index: SilkIndex,           // Implicit 5D — 37 buckets, 0-cost
    learned: Vec<HebbianLink>,  // Slim Hebbian links (sorted by key)
}
```

**Ba lớp kết nối:**
```
Layer 1: SilkIndex (implicit)    — 0 bytes, O(1) lookup
Layer 2: HebbianLink (learned)   — 19 bytes/link, Hebbian co-activation
Layer 3: SilkEdge (structural)   — 46 bytes/edge, rich metadata
```

### 3.3 SilkIndex — 37 kênh implicit (index.rs)

```rust
pub struct SilkIndex {
    shape:    [Vec<u64>; 8],   // 8 shape buckets
    relation: [Vec<u64>; 8],   // 8 relation buckets
    valence:  [Vec<u64>; 8],   // 8 valence zone buckets (zone = byte/32)
    arousal:  [Vec<u64>; 8],   // 8 arousal zone buckets
    time:     [Vec<u64>; 5],   // 5 time buckets
    node_count: usize,
}
// 8 + 8 + 8 + 8 + 5 = 37 kênh

pub enum SilkDim {
    Shape(u8),       // base 1..8
    Relation(u8),    // base 1..8
    Valence(u8),     // zone 0..7
    Arousal(u8),     // zone 0..7
    Time(u8),        // base 1..5
}

pub struct ImplicitSilk {
    pub shared_dims: Vec<SilkDim>,
    pub strength: f32,      // shared_count/5 base + precision bonuses
    pub shared_count: u8,   // 0..5
}
```

**implicit_silk(a, b) — O(1) so sánh 5D:**

| Chiều | Match condition | Base | Precision bonus |
|-------|----------------|------|-----------------|
| Shape | `base(a) == base(b)` | +0.15 | +0.05 nếu `a == b` exact |
| Relation | `base(a) == base(b)` | +0.15 | +0.05 nếu exact |
| Valence | `a/32 == b/32` | +0.15 | +0.05 nếu `delta < 8` |
| Arousal | `a/32 == b/32` | +0.15 | +0.05 nếu `delta < 8` |
| Time | `base(a) == base(b)` | +0.15 | +0.05 nếu exact |

Max strength = 1.0 (5 chiều match exact).

**MolSummary — 5D coordinates cho Silk:**

```rust
pub struct MolSummary {
    pub shape: u8,
    pub relation: u8,
    pub valence: u8,
    pub arousal: u8,
    pub time: u8,
}
// similarity() → 0.0..1.0 (per-dimension comparison)
```

### 3.4 SilkEdge — Rich metadata (edge.rs)

```rust
pub struct SilkEdge {
    pub from_hash: u64,
    pub to_hash: u64,
    pub kind: EdgeKind,
    pub emotion: EmotionTag,    // Cảm xúc khoảnh khắc co-activation
    pub weight: f32,            // Hebbian strength [0.0, 1.0]
    pub fire_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
    pub source: ModalitySource,
    pub confidence: f32,
}
// 46 bytes serialized
```

**EdgeKind (22 loại):**

```
Structural (weight=1.0, immutable):
  Member=0x01 Subset=0x02 Equiv=0x03 Orthogonal=0x04
  Compose=0x05 Causes=0x06 Similar=0x07 DerivedFrom=0x08

Space:
  Contains=0x09 Intersects=0x0A Subtracts=0x0B Mirror=0x0C

Time:
  Flows=0x0D Repeats=0x0E Resolves=0x0F Activates=0x10 Sync=0x11

Language:
  Translates=0x12

Associative (Hebbian, learned):
  Assoc=0xFF     — generic co-activation
  EdgeAssoc=0xA0 — learned + EmotionTag + source
  EdgeCausal=0xA1 — learned causal + confidence

QR:
  Supersedes=0xF0
```

**EmotionTag — 4D emotion trên edge:**

```rust
pub struct EmotionTag {
    pub valence: f32,    // [-1.0, +1.0]
    pub arousal: f32,    // [0.0, 1.0]
    pub dominance: f32,  // [0.0, 1.0]
    pub intensity: f32,  // [0.0, 1.0]
}
// NEUTRAL = { valence: 0.0, arousal: 0.3, dominance: 0.5, intensity: 0.2 }
```

**ModalitySource:**
```
Text=0x01  Audio=0x02  Image=0x03  Bio=0x04  Fused=0x05
```

### 3.5 HebbianLink — Slim learned (19 bytes)

```rust
pub struct HebbianLink {
    pub from_hash: u64,
    pub to_hash: u64,
    pub weight: u8,       // quantized: value/255.0
    pub fire_count: u16,
}
// 19 bytes vs 46 bytes SilkEdge (59% savings)
// Không mang EmotionTag — emotion nằm trong node V+A
// Initial weight = 26 (≈ 0.10)
```

### 3.6 Hebbian Learning (hebbian.rs)

```
Hằng số (từ φ = (1+√5)/2):
  PHI_INV        = φ⁻¹ ≈ 0.618     — decay factor per 24h
  LR             = φ⁻³ ≈ 0.236     — learning rate
  PROMOTE_WEIGHT = φ⁻¹ + φ⁻³ ≈ 0.854 — promotion threshold

hebbian_strengthen(weight, reward):
  weight += reward × (1 - weight) × LR
  Clamped to 1.0

hebbian_decay(weight, elapsed_ns):
  weight × φ⁻¹^days
  (Golden ratio decay — optimal forgetting)

should_promote(weight, fire_count, depth):
  weight ≥ 0.854 AND fire_count ≥ Fib[depth]
```

### 3.7 co_activate() family (graph.rs)

```rust
// Basic co-activation (caller ensures same layer — QT11)
fn co_activate(from, to, emotion, reward, ts)
  // Existing edge: strengthen + blend emotion + fire_count++
  // New edge: SilkEdge::associative(weight=0.1)

// Same-layer enforcement (QT11)
fn co_activate_same_layer(from, to, from_layer, to_layer, emotion, reward, ts) -> bool
  // from_layer != to_layer → return false, no edge

// Cross-layer with Fibonacci threshold (QT12)
fn co_activate_cross_layer(from, to, layers, emotion, reward, ts) -> bool
  // layer_diff = |L1 - L2|
  // threshold = Fib[diff + 2]
  // Needs fire_count >= threshold AND weight >= PROMOTE_WEIGHT

// Molecular-aware (uses 5D similarity bonus)
fn co_activate_mol(from, to, from_mol, to_mol, emotion, reward, ts)
  // sim_bonus = from_mol.similarity(&to_mol)
  // sim >= 0.4 → boosted_reward = reward × (1 + sim × 0.5)
  // New edge + sim >= 0.6 → weight += sim × 0.3 (cap 0.8)

// HebbianLink path (slim, no EmotionTag)
fn learn(from, to, reward)
fn learn_mol(from, to, from_mol, to_mol, reward)
```

### 3.8 Walk & Amplify (walk.rs)

```rust
pub struct WalkResult {
    pub composite: EmotionTag,
    pub path: Vec<u64>,
    pub total_weight: f32,
}

fn sentence_affect(graph, word_hashes, word_emotions, max_depth) -> WalkResult
fn sentence_affect_unified(graph, hashes, emotions, mols, max_depth) -> WalkResult
```

**amplify_emotion — KHÔNG BAO GIỜ trung bình:**

```rust
fn amplify_emotion(emo, weight) -> EmotionTag {
    valence:   emo.valence   × (1.0 + weight × 0.5)
    arousal:   emo.arousal   × (1.0 + weight × 0.3)
    dominance: emo.dominance  // unchanged
    intensity: emo.intensity × (1.0 + weight × 0.4)
}
// Edge weight=0.9 → valence ×1.45, arousal ×1.27, intensity ×1.36
```

**blend_composite** (sau amplify):
```
w_norm = weight / (1 + weight)
valence = composite.valence × (1 - w_norm) + new.valence × w_norm
```

### 3.9 Silk SilkNeighbor — unified query

```rust
pub struct SilkNeighbor {
    pub hash: u64,
    pub weight: f32,      // max(implicit, hebbian)
    pub implicit: f32,    // từ SilkIndex 5D comparison
    pub hebbian: f32,     // từ HebbianLink
    pub shared_dims: u8,  // 0..5
}
```

---

## 4. Cấu trúc tầng: L0 → L7

### 4.1 UCD → Origin

```
L7:  ○ (Origin)                          — 1 node
L6:  [Unity]                             — 1 node
L5:  [Hữu hình] ══ [Vô hình]            — 2 nodes
L4:  [Physical] ══ [Relational] ══ [Temporal] — 3 nodes
L3:  5 cross-dim nodes                   — 5 nodes
L2:  12 category nodes                   — 12 nodes
L1:  37 base nodes (= 37 Silk channels)  — 37 nodes
L0:  5400 UCD nodes                      — 5400 nodes
─────────────────────────────────────────────────────
Tổng: 5461 nodes

log_3(5400) ≈ 7.8 → 7 tầng (Fibonacci natural depth)
```

### 4.2 Silk topology

```
Silk ngang (horizontal, implicit, 0 bytes):
  L0: 37 kênh (implicit index lookup)
  L1: ~20 cặp
  L2: ~8 cặp
  L3: ~4 cặp
  L4:  2 cặp
  L5:  1 cặp
  ─────────
  Total: ~72 quan hệ implicit

Silk dọc (vertical, parent pointer):
  L1→L0:  5400 pointers
  L2→L1:    37 pointers
  L3→L2:    12 pointers
  L4→L3:     5 pointers
  L5→L4:     3 pointers
  L6→L5:     2 pointers
  L7→L6:     1 pointer
  ────────────────────
  Total: 5460 pointers × 8B = 43 KB

Tổng: 5532 Silk connections
  72 ngang  = 0 bytes (implicit)
  5460 dọc  = 43 KB (parent pointers)
```

### 4.3 37 kênh × 31 compound = 1147 kiểu quan hệ

```
Compound patterns (C(5,k) cho k=1..5):
  1 chiều chung: C(5,1) =  5   (liên quan nhẹ)
  2 chiều chung: C(5,2) = 10   (liên quan rõ)
  3 chiều chung: C(5,3) = 10   (gần giống)
  4 chiều chung: C(5,4) =  5   (gần như cùng)
  5 chiều chung: C(5,5) =  1   (cùng node)
  ──────────────────────────────
  Tổng: 31 mẫu compound

37 kênh × 31 mẫu = 1147 kiểu quan hệ có nghĩa

Ví dụ compound:
  S+V     = "trông giống + cảm giống"         → ẩn dụ thị giác
  R+V     = "quan hệ giống + cảm giống"       → moral analog
  V+A     = "cùng trạng thái cảm xúc"         → empathy link
  S+R+V   = "hình + quan hệ + cảm xúc giống"  → gần cùng khái niệm
```

### 4.4 Cross-layer Silk (QT11 + QT12)

```
QT11: Silk tự do chỉ cùng tầng (Ln-1 leaves)
QT12: Kết nối tầng trên → qua NodeLx đại diện

Cross-layer exception — Fibonacci threshold:
  L0 → L2: cần Fib[0+2] = Fib[2] = 1 co-activation
  L0 → L3: cần Fib[0+3] = Fib[3] = 2 co-activations
  L0 → L5: cần Fib[0+5] = Fib[5] = 5 co-activations

  Càng xa → threshold cao → càng khó nhảy tầng
  → Bắt buộc đi qua đại diện = ĐI ĐÚNG ĐƯỜNG
```

---

## 5. Ví dụ: Silk từ 5D

```
🔥 lửa  = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
😡 giận = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
→ 5/5 chiều giống → GẦN NHƯ CÙNG NODE
→ "giận dữ" = "lửa" là ẩn dụ phổ quát

🔥 lửa  = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
❄️ băng  = [Sphere, Causes, V=0x30, A=0x30, Slow]
→ 2/5 giống (Shape + Relation)
→ Đối lập cảm xúc + nhịp

"buồn"    = [○, Member, V=0x30, A=0x30, Slow]
"mất việc" = [■, Causes, V=0x20, A=0x50, Medium]
→ 0/5 giống chính xác
→ Nhưng V TƯƠNG ĐỒNG (0x30 ≈ 0x20, cùng zone)
→ Silk Valence: weight = 1.0 - |0x30-0x20|/0xFF ≈ 0.94
```

---

## 6. Bộ nhớ: Node lifecycle

```
                        encode_codepoint()
                              │
                              ▼
 ┌─────────┐  fire_count>0  ┌────────────┐  weight≥0.854   ┌────────┐
 │ Formula │ ─────────────→ │ Evaluating │ ──────────────→ │ Mature │
 └─────────┘                └────────────┘  +Fib[depth]    └────────┘
                                  │                             │
                              STM.push()                    Dream propose
                              Silk co_activate              AAM approve
                                                                │
                                                                ▼
                                                          QR (bất biến)
                                                          ED25519 signed
                                                          append-only
```

**DreamProposal — từ Dream → AAM:**

```rust
pub enum ProposalKind {
    NewNode { chain, emotion, sources },
    PromoteQR { chain_hash, fire_count },
    NewEdge { from_hash, to_hash, edge_kind },
    SupersedeQR { old_hash, new_hash, reason },
}

pub struct DreamProposal {
    pub kind: ProposalKind,
    pub confidence: f32,  // ≥ 0.6 → confident enough for AAM
    pub timestamp: i64,
}
```

---

## 7. Tóm tắt chi phí lưu trữ

```
Component              Per-unit     100M nodes      Notes
─────────────────────────────────────────────────────────────
Molecule (5D)          5 bytes      500 MB          DNA
Tagged wire            1-6 bytes    ~300 MB         Sparse
RegistryEntry          ~26 bytes    2.6 GB          Sổ cái
NodeBody (optional)    ~100 bytes   —               Chỉ 10M nodes có body
SplineSet              ~48 bytes    480 MB          Chỉ nodes có dynamics
Silk implicit          0 bytes      0 bytes         37 index buckets = 432 KB
HebbianLink            19 bytes     —               Chỉ learned pairs
SilkEdge               46 bytes     —               Chỉ structural
Aliases                ~20 bytes    2 GB            Text vẫn phải lưu text
Parent pointer         8 bytes      43 KB           Vertical Silk
─────────────────────────────────────────────────────────────
Tổng ước tính (100M):              ~6 GB           Dư 10 GB trên 16GB phone
```

---

## 8. Status: Implemented vs Spec

| Feature | Status | File |
|---------|--------|------|
| Molecule 5D + tagged encoding | ✅ Implemented | `mol/molecular.rs` |
| MolecularChain + evolve | ✅ Implemented | `mol/molecular.rs` |
| Maturity enum | ✅ Implemented | `mol/molecular.rs` |
| Registry + NodeKind (10) | ✅ Implemented | `storage/registry.rs` |
| RegistryGate + AlertLevel | ✅ Implemented | `memory/proposal.rs` |
| NodeBody + SplineSet | ✅ Implemented | `vsdf/body.rs` |
| SilkGraph (3-layer) | ✅ Implemented | `silk/graph.rs` |
| SilkIndex 37 channels | ✅ Implemented | `silk/index.rs` |
| implicit_silk() 5D comparison | ✅ Implemented | `silk/index.rs` |
| SilkEdge + EmotionTag (46B) | ✅ Implemented | `silk/edge.rs` |
| HebbianLink slim (19B) | ✅ Implemented | `silk/edge.rs` |
| co_activate / same_layer / cross_layer | ✅ Implemented | `silk/graph.rs` |
| sentence_affect + amplify_emotion | ✅ Implemented | `silk/walk.rs` |
| Hebbian strengthen/decay (φ-derived) | ✅ Implemented | `silk/hebbian.rs` |
| **Maturity wire (STM → Dream → QR)** | ⚠️ SPEC only | `CLAUDE.md` |
| **Silk Vertical (parent_map 43KB)** | ❌ NOT implemented | — |
| **CompoundKind (31 patterns)** | ❌ NOT implemented | — |

---

## 9. Quy Tắc Bất Biến liên quan

```
QT④  Mọi Molecule từ encode_codepoint(cp) — KHÔNG viết tay
QT⑤  Mọi chain từ LCA hoặc UCD — KHÔNG viết tay
QT⑥  chain_hash tự sinh — KHÔNG viết tay
QT⑦  chain cha = LCA(chain con)
QT⑧  Mọi Node → tự động registry
QT⑨  Ghi file TRƯỚC — cập nhật RAM SAU
QT⑩  Append-only — KHÔNG DELETE, KHÔNG OVERWRITE
QT⑪  Silk chỉ ở Ln-1 — tự do giữa lá cùng tầng
QT⑫  Kết nối tầng trên → qua NodeLx đại diện
QT⑬  Silk mang EmotionTag của khoảnh khắc co-activation
```
