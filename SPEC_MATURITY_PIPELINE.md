# Spec: Wire Maturity vào Dream Pipeline

**Ngày:** 2026-03-18
**Cập nhật:** 2026-03-18 (sau audit old/2026-03-18/)
**Audit:** 2026-03-18 — xem [AUDIT RESULTS](#audit-results-2026-03-18) ở cuối file
**Ưu tiên:** HIGH
**Scope:** 3 files, không thay đổi public API
**Mục tiêu:** Node chuyển từ công thức (Formula) → đang học (Evaluating) → chín (Mature) dựa trên evidence thật từ STM và Dream.

---

## Nguồn gốc thiết kế — `old/2026-03-18/node va silk.md`

File này là nền tảng triết học của toàn bộ spec. Những ý tưởng cốt lõi:

### Molecule = công thức, không phải giá trị

> *"Thay vì lưu DATA (pixel, byte, string), Olang lưu BẢN CHẤT"*

Hiện tại `Molecule` lưu 5 giá trị tĩnh:
```
🔥 = [Sphere, Causes, 0xC0, 0xC0, Fast]  ← 5 bytes GIÁ TRỊ
```

Thiết kế đúng: mỗi chiều là một **hàm** (công thức), không phải hằng số:
```
Shape    = f_s(inputs...)    ← công thức hình dạng
Relation = f_r(inputs...)    ← công thức quan hệ
Valence  = f_v(inputs...)    ← công thức cảm xúc
Arousal  = f_a(inputs...)    ← công thức cường độ
Time     = f_t(inputs...)    ← công thức thời gian

Khi chưa có input  → công thức = TIỀM NĂNG  (Maturity::Formula)
Khi có input       → thế vào  → GIÁ TRỊ CỤ THỂ (Maturity::Evaluating)
Khi đủ giá trị     → node CHÍN → thay công thức bằng hằng số (Maturity::Mature)
```

Đây chính là lý do `Maturity` enum tồn tại — nhưng chưa được wire vào pipeline.

### Silk = hệ quả tự nhiên của 5D, không phải dữ liệu

> *"Silk KHÔNG PHẢI graph riêng. Silk LÀ CẤU TRÚC TỰ NHIÊN của không gian 5D."*

Node A và Node B chia sẻ cùng base value trên bất kỳ chiều nào → Silk **tự tồn tại**. Không cần lưu edge. Không cần ai tạo.

```
🔥 lửa  = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
😡 giận = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
→ 5/5 chiều giống → GẦN NHƯ CÙNG NODE
→ Đây là lý do "giận dữ" và "lửa" là ẩn dụ phổ quát trong mọi ngôn ngữ
```

**37 kênh Silk cơ bản** (đã implement trong `SilkIndex`):
- Shape: 8 loại (Sphere, Capsule, Box, Cone, Torus, Union, Intersect, Subtract)
- Relation: 8 loại (Member, Subset, Equiv, Ortho, Compose, Causes, Similar, DerivedFrom)
- Valence: 8 vùng (chia 256 thành 8 zone)
- Arousal: 8 vùng
- Time: 5 loại (Static, Slow, Medium, Fast, Instant)

**31 mẫu compound** (chưa implement):
- 1 chiều chung: C(5,1) = 5 mẫu → "liên quan nhẹ"
- 2 chiều chung: C(5,2) = 10 mẫu → "liên quan rõ"
- 3 chiều chung: C(5,3) = 10 mẫu → "gần giống"
- 4 chiều chung: C(5,4) = 5 mẫu → "gần như cùng khái niệm"
- 5 chiều chung: C(5,5) = 1 mẫu → "cùng node"

37 kênh × 31 mẫu = **1147 kiểu quan hệ có nghĩa** — đủ mô tả bất kỳ mối quan hệ nào.

### Silk dọc — parent pointer liên tầng

> *"Kết nối tầng trên → qua NodeLx đại diện (Fib[n+2] threshold)"*

```
 5400 nodes  ─── Silk tự do (horizontal, 0 bytes)
     │ parent pointer (vertical, 8 bytes/node)
   37 nodes  ─── Silk tự do
     │
   12 nodes
     │
...
   1 node (○)
```

Tổng: **5460 parent pointers × 8 bytes = 43 KB** cho mạng Silk dọc.  
Hiện tại: `SilkGraph` **không có** field này. `SilkIndex` chỉ có 37 horizontal buckets.

### Sức mạnh kết nối = số chiều chung

```rust
strength(A, B) = Σ match(dim) × precision(dim)

match(dim)     = 1.0 nếu cùng base, 0.0 nếu khác
precision(dim) = 1.0 nếu cùng variant, 0.5 nếu chỉ cùng base
```

Hiện tại `MolSummary::similarity()` tính gần đúng theo delta — đúng tinh thần nhưng chưa tách biệt `match` và `precision` rõ ràng như thiết kế.

### Node chín từ data thật — vòng đời đầy đủ

```
Dream = "ngủ để hiểu"
  STM đầy công thức chưa evaluate (Formula nodes)
  Dream đi qua → thế giá trị vào → node chín → promote QR
  Node chưa đủ data → giữ công thức → chờ thêm input

Programming (LeoAI) = TẠO CÔNG THỨC MỚI
  program("emit A ∘ B;")
  = lấy công thức A, lấy công thức B
  = TỔ HỢP thành công thức C
  = C là node mới — chưa có giá trị — CHỜ dữ liệu (Formula)

Evolve = THAY 1 BIẾN trong công thức
  🔥.evolve(Valence, 0x40) → "lửa nhẹ"
  = giữ nguyên f_s, f_r, f_a, f_t, chỉ thay f_v → loài mới
```

### Trạng thái hiện tại so với thiết kế

| Thiết kế (node va silk.md) | Hiện tại (phiên K) | Gap |
|---|---|---|
| Molecule = 5 công thức | Molecule = 5 giá trị tĩnh | Lớn — cần spec riêng |
| Silk implicit 37 kênh | `SilkIndex` 37 buckets ✅ | Xong |
| 31 mẫu compound | Chưa có | Trung bình |
| Silk dọc 5460 parent ptr | Chưa có | Vấn đề #5 |
| Maturity Formula→Mature | Enum có, chưa wire | **Spec này** |
| Node chín từ Dream | Dream 0 clusters | Vấn đề #3+#4 |
| Strength = match × precision | Similarity delta-based | Gần đúng |

---

## Bối cảnh kỹ thuật

Enum `Maturity` (`Formula → Evaluating → Mature`) đã tồn tại trong `crates/olang/src/mol/molecular.rs` với logic `advance()` đúng, nhưng **chưa được wire vào bất kỳ pipeline nào**. `Observation` không track maturity, `DreamResult` không trả về nodes nào đã chín.

---

## Thay đổi 1 — `crates/agents/src/pipeline/learning.rs`

### Mục tiêu
`Observation` phải biết node đang ở trạng thái nào trong vòng đời.

### Thêm import ở đầu file
```rust
use olang::molecular::Maturity;
```

### Sửa struct `Observation`
```rust
pub struct Observation {
    pub chain: MolecularChain,
    pub emotion: EmotionTag,
    pub timestamp: i64,
    pub fire_count: u32,
    pub mol_summary: Option<MolSummary>,
    pub maturity: Maturity,   // ← THÊM — default: Formula
}
```

### Sửa `ShortTermMemory::push()`

**Khi tìm thấy observation đã có** (tăng `fire_count`):
```rust
obs.fire_count += 1;
// fib(2) = 2 — threshold cho STM (depth=2)
let fib_threshold = silk::hebbian::fib(2);
obs.maturity = obs.maturity.advance(obs.fire_count, 0.0, fib_threshold);
// blend emotion như cũ...
```

**Khi tạo Observation mới** (push lần đầu):
```rust
Observation {
    chain: chain.clone(),
    emotion,
    timestamp: ts,
    fire_count: 1,
    mol_summary: Some(summary),
    maturity: Maturity::Formula,   // ← THÊM
}
```

### Tests cần thêm
```rust
#[test]
fn observation_starts_as_formula() {
    let mut stm = ShortTermMemory::new(512);
    let chain = olang::encoder::encode_codepoint(0x1F525);
    stm.push(chain.clone(), EmotionTag::NEUTRAL, 0);
    let obs = stm.top_n(1);
    assert_eq!(obs[0].maturity, Maturity::Formula, "Lần đầu push → Formula");
}

#[test]
fn observation_advances_to_evaluating_on_second_fire() {
    let mut stm = ShortTermMemory::new(512);
    let chain = olang::encoder::encode_codepoint(0x1F525);
    stm.push(chain.clone(), EmotionTag::NEUTRAL, 0);
    stm.push(chain.clone(), EmotionTag::NEUTRAL, 1); // fire_count → 2 >= fib(2)=2
    let obs = stm.top_n(1);
    assert!(
        obs[0].maturity == Maturity::Evaluating || obs[0].maturity == Maturity::Mature,
        "fire_count=2 → ít nhất Evaluating: {:?}", obs[0].maturity
    );
}
```

---

## Thay đổi 2 — `crates/memory/src/dream.rs`

### Mục tiêu
Dream phải phát hiện và báo cáo nodes nào đã chín trong quá trình chạy.

### Sửa struct `DreamResult`
```rust
pub struct DreamResult {
    pub scanned: usize,
    pub clusters_found: usize,
    pub proposals: Vec<DreamProposal>,
    pub approved: usize,
    pub rejected: usize,
    pub matured_nodes: Vec<u64>,   // ← THÊM
}
```

### Sửa `DreamCycle::run()`

**Bước 1a:** Ngay sau `let top = stm.top_n(...)`:
```rust
let fib_threshold = silk::hebbian::fib(self.config.tree_depth);
let matured_nodes: Vec<u64> = top
    .iter()
    .filter(|obs| obs.maturity.advance(obs.fire_count, 0.0, fib_threshold).is_mature())
    .map(|obs| obs.chain.chain_hash())
    .collect();
```

**Bước 1b:** Early return:
```rust
if scanned < self.config.min_cluster_size {
    return DreamResult {
        scanned, clusters_found: 0, proposals: Vec::new(),
        approved: 0, rejected: 0, matured_nodes,
    };
}
```

**Bước 1c:** Return cuối:
```rust
DreamResult {
    scanned, clusters_found, proposals,
    approved: approved_count, rejected: rejected_count,
    matured_nodes,
}
```

### Tests cần thêm
```rust
#[test]
fn dream_detects_mature_nodes() {
    let mut stm = ShortTermMemory::new(512);
    let chain = olang::encoder::encode_codepoint(0x1F525);
    for i in 0..10 {
        stm.push(chain.clone(), EmotionTag::NEUTRAL, i as i64 * 1000);
    }
    let graph = SilkGraph::new();
    let dream = DreamCycle::new(DreamConfig { tree_depth: 2, ..Default::default() });
    let result = dream.run(&stm, &graph, 10000);
    assert!(!result.matured_nodes.is_empty(), "fire_count=10 >> fib(2)=2 → phải có mature nodes");
}

#[test]
fn dream_no_mature_nodes_when_fire_low() {
    let mut stm = ShortTermMemory::new(512);
    let chain = olang::encoder::encode_codepoint(0x1F525);
    stm.push(chain.clone(), EmotionTag::NEUTRAL, 0);
    let graph = SilkGraph::new();
    let dream = DreamCycle::new(DreamConfig { tree_depth: 5, ..Default::default() });
    let result = dream.run(&stm, &graph, 1000);
    assert!(result.matured_nodes.is_empty(), "fire_count=1 << fib(5)=8 → không mature");
}

#[test]
fn dream_result_has_matured_nodes_field() {
    let result = DreamResult {
        scanned: 0, clusters_found: 0, proposals: alloc::vec![],
        approved: 0, rejected: 0, matured_nodes: alloc::vec![0xDEADu64],
    };
    assert_eq!(result.matured_nodes.len(), 1);
}
```

---

## Thay đổi 3 — Caller của `dream.run()`

Tìm tất cả chỗ gọi `dream.run()` trong `crates/runtime/src/` hoặc `crates/agents/`. Sau khi gọi thêm:

```rust
let result = dream.run(&stm, &graph, ts);
if !result.matured_nodes.is_empty() {
    let _ = &result.matured_nodes; // tối thiểu: không bỏ qua
}
```

Chưa ghi vào Registry — chỉ đảm bảo thông tin không bị drop silently.

---

## Những gì KHÔNG được thay đổi

```
❌ Không sửa Molecule struct (vẫn 5 bytes giá trị)
❌ Không sửa SilkEdge hay HebbianLink
❌ Không xóa EmotionTag khỏi SilkEdge
❌ Không sửa AAM::review() hay DreamProposal
❌ Không thêm dependency mới
❌ Không đụng vào L0 seeding hay UCD
```

---

## Checklist trước khi commit

```bash
cargo test --workspace          # phải pass 2063 tests cũ + tests mới
cargo clippy --workspace -- -D warnings  # 0 warnings
```

Nếu test cũ trong `dream.rs` fail vì thiếu field `matured_nodes` → thêm `matured_nodes: Vec::new()`.

---

## Bản đồ vấn đề toàn hệ thống (audit old/2026-03-18/)

Đọc 8 files trong `old/2026-03-18/` cho thấy 6 vấn đề hệ thống tồn tại qua nhiều phiên (A→K). Mỗi phiên AI mới lại ưu tiên thêm tính năng thay vì sửa mặt tiền — đây là lý do các vấn đề lặp lại.

---

### #1 — Response template (nghiêm trọng nhất)

**Được nhắc trong:** AUDIT, USER_PERSPECTIVE_REVIEW, NEXT_PLAN, PLAN_PHAN_VIEC  
**File cần sửa:** `crates/runtime/src/core/origin.rs`  
**Effort:** Trung bình | **Impact:** Cực cao

**Triệu chứng:**
- User nói "tôi buồn vì mất việc" → "Ừ. Bạn muốn kể thêm không?"
- User nói "hôm nay tôi rất vui" → "Bạn đang tìm hiểu để làm gì?"
- User nói "con mèo dễ thương quá" → "Bạn đang tìm hiểu để làm gì?"
- ~10 câu template cố định cho mọi input, bất kể nội dung

**Gốc rễ — 3 tầng:**

1. **Instinct output bị bỏ qua.** `LeoAI.run_instincts()` chạy đủ 7 bản năng — Causality phát hiện "mất việc → buồn", Abstraction tạo LCA("buồn", "mất việc") = "mất mát". Nhưng `render_response()` không đọc kết quả này — chỉ dùng V/A score để chọn tone rồi lookup template.

2. **Silk walk kết quả không dùng.** `SilkGraph.unified_neighbors()` tìm được related nodes qua 5D similarity và Hebbian links — nhưng không được đưa vào response text. Response không nhắc đến bất kỳ entity nào user đã nói.

3. **Template quá thưa.** `response_template.rs` có ~10 câu phân theo 6 tone, mỗi tone 1-2 câu, không có slot cho nội dung động.

**Hướng sửa:**
```rust
// Trong origin.rs, hàm render_response():

// 1. Topic từ Silk walk (top related nodes)
let related = silk_graph.unified_neighbors(input_hash, Some(&mol_summary));
let topic_hint = related.first().map(|n| registry.lookup_alias(n.hash));

// 2. Insight từ instinct output
let causal_hint = instinct_results.iter()
    .find(|r| matches!(r.kind, InsightKind::Causal { .. }));

// 3. Compose response phản ánh nội dung thật
// Thay vì: "Bạn đang tìm hiểu để làm gì?"
// Thành:   "Nghe như [topic_hint] đang ảnh hưởng đến bạn."
```

---

### #2 — Parser thiếu 6 commands (dễ sửa nhất)

**Được nhắc trong:** AUDIT, USER_PERSPECTIVE_REVIEW  
**File cần sửa:** `crates/runtime/src/core/parser.rs`  
**Effort:** Nhỏ (vài dòng) | **Impact:** Cao

**Triệu chứng:**
```
○{typeof fire}    → lỗi "chưa registry"
○{explain fire}   → lỗi "chưa registry"
○{why fire}       → lỗi "chưa registry"
○{trace}          → lỗi "chưa registry"
○{inspect fire}   → lỗi "chưa registry"
○{assert fire}    → lỗi "chưa registry"
```

**Gốc rễ:** `is_command()` thiếu 6 keywords. `handle_command()` **đã có** code xử lý — chỉ thiếu routing từ parser đến đó.

**Hướng sửa:**
```rust
fn is_command(s: &str) -> bool {
    matches!(s,
        "dream" | "stats" | "health" | "seed" | "help" | ...
        | "typeof"   // ← THÊM
        | "explain"  // ← THÊM
        | "why"      // ← THÊM
        | "trace"    // ← THÊM
        | "inspect"  // ← THÊM
        | "assert"   // ← THÊM
    )
}
```

---

### #3 — Maturity pipeline ← spec này

Xem chi tiết ở phần "Thay đổi 1-3" ở trên và phần "Nguồn gốc thiết kế" về Molecule = công thức.

**Tóm tắt:** `Maturity` enum + `advance()` đã có nhưng không wire. `Observation` không track trạng thái. `DreamResult` không báo cáo nodes chín. Vòng đời Formula → Evaluating → Mature chưa chạy trong thực tế.

---

### #4 — Dream threshold quá cao

**Được nhắc trong:** AUDIT, USER_PERSPECTIVE_REVIEW, NEXT_PLAN  
**File cần sửa:** `crates/memory/src/dream.rs`  
**Effort:** Nhỏ | **Impact:** Cao

**Triệu chứng:**
```
Dream cycle → scanned: 2, clusters: 0, proposals: 0
Dream cycle → scanned: 1, clusters: 0, proposals: 0
→ Không phiên nào Dream học được gì
```

**Gốc rễ — số học:**
```
Pipeline học 1 observation/turn
Sau 5 turns → STM có tối đa 5 observations

DreamConfig::default():
  min_cluster_size = 3     → cần >= 3 observations cùng chủ đề
  cluster_threshold = 0.6  → cần score >= 0.6

cluster_score = 0.3×chain_sim + 0.4×hebbian_weight + 0.3×fire_ratio

Thực tế:
  chain_sim("tôi buồn", "mất việc") ≈ 0.20  (khác chủ đề)
  hebbian_weight mới tạo = 0.10              (khởi đầu yếu)
  fire_ratio ≈ 0

  score ≈ 0.3×0.20 + 0.4×0.10 + 0.3×0 = 0.10  << ngưỡng 0.6

→ KHÔNG BAO GIỜ cluster được trong hội thoại thông thường
```

**Hướng sửa:**
```rust
impl DreamConfig {
    /// Preset cho hội thoại thật — threshold thực tế hơn default.
    pub fn for_conversation() -> Self {
        Self {
            scan_top_n: 32,
            cluster_threshold: 0.30,  // từ 0.6 → 0.30
            min_cluster_size: 2,      // từ 3 → 2
            tree_depth: 2,            // fib(2)=2, dễ promote hơn
            alpha: 0.4,               // tăng weight cho chain_sim
            beta: 0.3,
            gamma: 0.3,
        }
    }
}
// Dùng DreamConfig::for_conversation() trong HomeRuntime::new()
```

---

### #5 — Silk dọc (parent pointer) chưa có

**Được nhắc trong:** `old/2026-03-18/node va silk.md`, `old/2026-03-18/silk_architecture.md`  
**File cần sửa:** `crates/silk/src/graph.rs`  
**Effort:** Trung bình | **Impact:** Cao

**Thiết kế từ node va silk.md:**

> *"Silk đại diện (liên tầng): node Lx là ĐẠI DIỆN cho 1 nhóm ở Lx-1. Mỗi node chỉ cần 1 pointer đến parent. 5460 pointers × 8 bytes = 43 KB — toàn bộ mạng Silk dọc."*

```
L1 → L0: 5400 pointers  (mỗi UCD atom trỏ lên L1 representative)
L2 → L1:   37 pointers
L3 → L2:   12 pointers
L4 → L3:    5 pointers
L5 → L4:    3 pointers
L6 → L5:    2 pointers
L7 → L6:    1 pointer
─────────────────────
Tổng:    5460 × 8B = 43 KB
```

**Triệu chứng thiếu:**
- Không thể query "concept cha của node này ở tầng trên"
- `co_activate_same_layer()` nhận layer từ caller nhưng không có nguồn sự thật độc lập
- Dream clustering không biết 2 nodes có cùng tầng không
- Cross-layer Silk (QT12) không có cấu trúc để enforce

**Hướng sửa:**
```rust
// Trong graph.rs — thêm vào SilkGraph:
pub struct SilkGraph {
    edges: Vec<SilkEdge>,
    index: SilkIndex,
    learned: Vec<HebbianLink>,
    parent_map: alloc::collections::BTreeMap<u64, u64>, // child → parent
}

impl SilkGraph {
    /// Đăng ký quan hệ cha-con — gọi khi Dream promote node lên tầng trên.
    pub fn register_parent(&mut self, child_hash: u64, parent_hash: u64) {
        self.parent_map.insert(child_hash, parent_hash);
    }

    pub fn parent_of(&self, hash: u64) -> Option<u64> {
        self.parent_map.get(&hash).copied()
    }

    pub fn children_of(&self, parent_hash: u64) -> alloc::vec::Vec<u64> {
        self.parent_map
            .iter()
            .filter(|(_, &p)| p == parent_hash)
            .map(|(&c, _)| c)
            .collect()
    }

    /// Layer của node — đi ngược parent chain đếm tầng.
    pub fn layer_of(&self, hash: u64) -> u8 {
        let mut current = hash;
        let mut depth = 0u8;
        while let Some(parent) = self.parent_of(current) {
            depth += 1;
            current = parent;
            if depth > 16 { break; } // an toàn
        }
        depth
    }
}
```

---

### #6 — Agent hierarchy là code chết

**Được nhắc trong:** AUDIT, USER_PERSPECTIVE_REVIEW, PLAN_PHAN_VIEC  
**File cần sửa:** `crates/runtime/src/core/origin.rs`, `crates/agents/src/hierarchy/chief.rs`  
**Effort:** Lớn | **Impact:** Cao nhưng phức tạp

**Triệu chứng:**
```
Router Ticks:  2-4 mỗi phiên
Worker→Chief:  0
Chief→LeoAI:   0
Workers:       0 (không bao giờ được spawn)
ISL messages:  0 forwarded
```

**Gốc rễ — 3 tầng:**

1. **Không có trigger.** Router.tick() chạy nhưng không có message nào để forward. Chiefs không tự generate message. Workers chưa được spawn.

2. **Intent routing thiếu.** Khi user nói "bật đèn phòng khách", `estimate_intent()` phân loại là `Chat` — không có `HomeControl` intent. Không có code nào route đến HomeChief.

3. **HAL chưa kết nối thiết bị.** `hal::detect::platform()` nhận biết platform đúng nhưng không có driver thật nào phía sau.

**Hướng sửa — 3 bước theo thứ tự:**

Bước A — thêm `HomeControl` intent:
```rust
// context/analysis/intent.rs:
pub enum IntentAction {
    Chat, Learn, Command, Crisis,
    HomeControl { device_hint: Option<String> },  // ← THÊM
}

// origin.rs — route đến HomeChief:
if let IntentAction::HomeControl { device_hint } = &intent {
    let msg = ISLMessage::new(LEO_ADDR, HOME_CHIEF_ADDR, MsgType::Task, ...);
    self.router.send(msg);
}
```

Bước B — Mock Workers cho test:
```rust
// origin.rs boot sequence:
let mock_light_worker = Worker::new("mock_light", WorkerKind::Device);
self.register_worker(mock_light_worker);
```

Bước C — Wire Chief xử lý ISL message:
```rust
// chief.rs:
fn process_task(&mut self, msg: &ISLMessage) -> Option<ISLMessage> {
    // dispatch đến Worker phù hợp, trả về response
}
```

---

## Thiết kế gốc từ `node va silk.md`

File `old/2026-03-18/node va silk.md` là tài liệu thiết kế quan trọng nhất — mô tả cách Silk và Molecule **thật sự** nên hoạt động. Đây là nguồn gốc của các vấn đề #3, #4, #5 và ảnh hưởng đến #1. Cần đọc kỹ trước khi implement bất kỳ thay đổi nào liên quan đến Silk hoặc Molecule.

### Ý tưởng cốt lõi 1 — Molecule = công thức, không phải giá trị

```
Hiện tại:   Molecule = 5 giá trị tĩnh
            [Sphere, Causes, 0xC0, 0xC0, Fast]

Thiết kế:   Molecule = 5 CÔNG THỨC
            Shape    = f_s(inputs...)   ← "cách TRỞ THÀNH Sphere"
            Relation = f_r(inputs...)
            Valence  = f_v(inputs...)
            Arousal  = f_a(inputs...)
            Time     = f_t(inputs...)
```

Vòng đời của một node:
- **Chưa có input** → 5 chiều là công thức tiềm năng, chưa xác định
- **Có input** → thế giá trị thật vào công thức → giá trị cụ thể
- **Đủ giá trị** → node "chín" → thay công thức bằng hằng số → promote QR

Đây chính là lý do `Maturity` enum tồn tại. `Formula` = chưa có input thật.
`Evaluating` = đang tích lũy. `Mature` = đủ evidence, sẵn sàng QR.

**Trạng thái hiện tại:** `Maturity` enum có, logic `advance()` đúng, nhưng
không được wire vào STM hay Dream. Xem Thay đổi 1-3 ở trên.

---

### Ý tưởng cốt lõi 2 — Silk = 0 bytes, không phải edge list

```
Thiết kế:   Silk edge = KHÔNG CẦN LƯU
            Khi 2 node chia sẻ base value trên cùng chiều → Silk TỰ TỒN TẠI
            "2 node cùng họ Sphere" = Silk trên chiều Shape, implicit, 0 bytes

Hiện tại:   SilkIndex có 37 horizontal buckets — ĐÚNG THIẾT KẾ ✅
            HebbianLink 19 bytes — ĐÚNG (học được, không phải tạo mới) ✅
            SilkEdge 46 bytes — backward compat ⚠️
```

37 kênh Silk cơ bản = 8 Shape + 8 Relation + 8 Valence zone + 8 Arousal zone + 5 Time.

Sức mạnh kết nối tính theo số chiều chung:
```
1 chiều chung = liên quan nhẹ    (strength ≈ 0.20)
2 chiều chung = liên quan rõ     (strength ≈ 0.40)
3 chiều chung = gần giống        (strength ≈ 0.60)
4 chiều chung = gần như cùng     (strength ≈ 0.80)
5 chiều chung = cùng node        (strength = 1.00)
```

Ví dụ từ tài liệu:
```
🔥 lửa  = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
😡 giận = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
→ 5/5 chiều giống → gần như cùng node
→ Đây là lý do "giận dữ" và "lửa" là ẩn dụ phổ quát!

🔥 lửa  = [Sphere, Causes, V=0xC0, A=0xC0, Fast]
❄️ băng  = [Sphere, Causes, V=0x30, A=0x30, Slow]
→ 2/5 chiều chung (Shape + Relation)
→ Đối lập ở cảm xúc và nhịp — nhưng cùng "gây ra" điều gì đó
```

**Trạng thái hiện tại:** `SilkIndex.implicit_silk()` và `implicit_neighbors()`
đã implement đúng — so sánh 5D, trả về `ImplicitSilk` với `shared_dims` và
`strength`. Phần này **đã hoạt động**. Vấn đề là kết quả implicit Silk chưa
được dùng trong response rendering (#1) và Dream clustering (#4).

---

### Ý tưởng cốt lõi 3 — Silk dọc: từ L0 đến ○ qua 7 tầng

Tài liệu tính toán chi tiết số lượng Silk cần thiết:

```
Silk tự do (horizontal, 0 bytes):
  L0: 5400 nodes → 37 kênh index
  L1:   37 nodes → ~20 cặp
  L2:   12 nodes → ~8 cặp
  L3:    5 nodes → ~4 cặp
  L4:    3 nodes → ~2 cặp
  L5:    2 nodes → ~1 cặp
  Tổng: ~72 quan hệ, 0 bytes

Silk đại diện (vertical, 8 bytes/node = parent pointer):
  L1→L0:  5400 pointers
  L2→L1:    37 pointers
  L3→L2:    12 pointers
  L4→L3:     5 pointers
  L5→L4:     3 pointers
  L6→L5:     2 pointers
  L7→L6:     1 pointer
  Tổng: 5460 × 8 bytes = 43 KB

TOÀN BỘ mạng Silk từ L0 đến ○ = 43 KB.
```

Truy vấn qua Silk là O(1):
```
"🔥 liên quan gì đến ∈?"
Bước 1: So sánh 5D → Shape=Sphere giống nhau → 1 chiều chung
Bước 2: Qua parent pointer → cùng L1 representative
→ Chi phí: 2 lookups + 1 compare
```

**Trạng thái hiện tại:** Parent pointer (`parent_map`) **chưa có** trong
`SilkGraph`. Đây là vấn đề #5. Không có cấu trúc liên tầng → Dream không
biết 2 nodes có cùng tầng không → `co_activate_same_layer()` phải nhận
layer từ caller thay vì tự biết.

---

### Ý tưởng cốt lõi 4 — EmotionTag không nên nằm trên Hebbian edge

Tài liệu chỉ ra mâu thuẫn trong thiết kế hiện tại:

```
Thiết kế đúng:
  EmotionTag NẰM TRONG node (Molecule V+A)
  "Cùng cảm xúc" = cùng công thức V hoặc A = TỰ ĐỘNG Silk
  → Không cần lưu emotion trên edge

Hiện tại:
  HebbianLink KHÔNG có EmotionTag → ĐÚNG ✅ (19 bytes, slim)
  SilkEdge    CÓ EmotionTag       → DƯ THỪA ⚠️ (46 bytes, backward compat)
```

`HebbianLink` đã đúng — emotion không trên edge. `SilkEdge` vẫn giữ
`EmotionTag` vì backward compat. Không cần sửa ngay, chỉ cần biết khi
refactor sau này.

---

### Kết nối với 6 vấn đề hệ thống

| Ý tưởng từ node va silk.md | Vấn đề liên quan | Trạng thái |
|---|---|---|
| Molecule = công thức, node chín qua evidence | #3 Maturity pipeline | Spec này (PR #23) |
| Silk implicit 37 kênh, 0 bytes | #4 Dream threshold | SilkIndex đúng nhưng Dream không dùng |
| Silk dọc: parent pointer 43KB | #5 Silk parent pointer | Chưa implement |
| Instinct output surface vào response | #1 Response template | Chưa implement |
| Dream = evaluate công thức đã chín | #4 Dream threshold | Threshold quá cao |

---

## Thứ tự thực hiện khuyến nghị

```
1. #2 — Parser commands      (vài dòng, test ngay, 0 rủi ro)
2. #3 — Maturity pipeline    (spec này — PR #23)
3. #4 — Dream threshold      (nhỏ, unblock Dream learning)
4. #1 — Response template    (trung bình, impact cực cao với user)
5. #5 — Silk parent pointer  (nền tảng cho layer-aware queries)
6. #6 — Agent hierarchy      (lớn nhất, làm sau khi 1-5 ổn định)
```

---

*HomeOS · 2026-03-18 · Maturity pipeline · Formula → Evaluating → Mature*
*Cập nhật sau audit old/2026-03-18/ — thiết kế từ node va silk.md được phân tích đầy đủ*

---

## AUDIT RESULTS (2026-03-18)

**Auditor:** Claude (automated code verification)
**Phương pháp:** So sánh từng claim trong Spec với code thực tế trong codebase (2227 tests pass).
**Kết quả tổng:** 18/19 claims kỹ thuật verify được là đúng (95%). Phát hiện **1 bug logic nghiêm trọng**, **1 sai số liệu**, và **1 claim thiếu chính xác**.

---

### BUG NGHIÊM TRỌNG: `advance()` với weight=0.0 KHÔNG BAO GIỜ đạt Mature

**Ảnh hưởng:** Thay đổi 1 (dòng 165) + Thay đổi 2 (dòng 232) + test `dream_detects_mature_nodes` (dòng 268) sẽ FAIL.

**Vấn đề:** Spec truyền `weight=0.0` vào `advance()` ở cả 2 nơi:

```rust
// Thay đổi 1 — learning.rs (dòng 165)
obs.maturity = obs.maturity.advance(obs.fire_count, 0.0, fib_threshold);

// Thay đổi 2 — dream.rs (dòng 232)
obs.maturity.advance(obs.fire_count, 0.0, fib_threshold).is_mature()
```

**Code thực tế** (`molecular.rs:324-330`):
```rust
Self::Evaluating => {
    // φ⁻¹ + φ⁻³ ≈ 0.854 (PROMOTE_WEIGHT from hebbian.rs)
    if weight >= 0.854 && fire_count >= fib_threshold {
        Self::Mature
    } else {
        Self::Evaluating
    }
}
```

**Hệ quả:**
- `0.0 >= 0.854` = **luôn false** → Evaluating KHÔNG BAO GIỜ → Mature
- Thay đổi 1: STM nodes chỉ tới `Evaluating`, kẹt ở đó vĩnh viễn
- Thay đổi 2: `matured_nodes` luôn rỗng (`Vec::new()`)
- Test `dream_detects_mature_nodes` sẽ **FAIL** — fire_count=10 nhưng weight=0.0 nên vẫn Evaluating

**Sửa đề xuất (chọn 1 trong 2):**
1. **Truyền Hebbian weight thật:** STM push và Dream cần query `SilkGraph` cho weight thực tế của chain_hash. Đòi hỏi thêm `&SilkGraph` param vào `push()`.
2. **Fire-count-only path:** Thêm method `advance_by_fire(fire_count, fib_threshold)` bỏ qua weight check — dùng cho STM context nơi chưa có Hebbian link. Đơn giản hơn, nhưng bypass thiết kế "cần cả weight lẫn fire_count".

---

### SAI SỐ LIỆU: Test count

**Spec ghi (dòng 325):** `# phải pass 2063 tests cũ`
**Thực tế:** 2227 tests pass

Chênh +164 tests. Con số cần cập nhật.

---

### CLAIM THIẾU CHÍNH XÁC: MolSummary similarity vs match/precision

**Spec ghi (dòng 95):** "MolSummary::similarity() tính gần đúng theo delta — đúng tinh thần nhưng chưa tách biệt match và precision rõ ràng như thiết kế."

**Thực tế:**
- `MolSummary::similarity()` (`graph.rs:44-91`) — dùng delta-based, **đúng như spec nói** ✅
- NHƯNG `SilkIndex::implicit_silk()` (`index.rs:180-236`) — **ĐÃ tách match vs precision**: exact match=0.20, base-only match=0.15. Spec không nhắc đến điều này.

**Kết luận:** Spec đúng về MolSummary nhưng thiếu thông tin — ImplicitSilk đã implement pattern match/precision mà spec nói "chưa có".

---

### TẤT CẢ CLAIMS ĐÃ VERIFY

| # | Claim | Nguồn verify | Kết quả |
|---|-------|-------------|---------|
| 1 | `Maturity` enum (Formula/Evaluating/Mature) | `molecular.rs:286-294` | ✅ |
| 2 | `advance()` logic (fire>0 → Eval, weight≥0.854 && fire≥fib → Mature) | `molecular.rs:315-334` | ✅ |
| 3 | `is_mature()` tồn tại | `molecular.rs:341` | ✅ |
| 4 | `Observation` chưa có field `maturity` | `learning.rs:50-59` | ✅ cần thêm |
| 5 | `DreamResult` chưa có field `matured_nodes` | `dream.rs:94-106` | ✅ cần thêm |
| 6 | `DreamCycle::run()` signature | `dream.rs:133` | ✅ |
| 7 | `DreamConfig` defaults (threshold=0.6, min=3, depth=3, α=0.3, β=0.4, γ=0.3) | `dream.rs:75-86` | ✅ |
| 8 | `silk::hebbian::fib()` tồn tại | `hebbian.rs:87-98` | ✅ |
| 9 | `dream.run()` có 2 callers | `origin.rs:1157, 4643` | ✅ |
| 10 | `is_command()` thiếu 6 keywords (typeof/explain/why/trace/inspect/assert) | `parser.rs:1136-1159` | ✅ |
| 11 | `SilkGraph` không có `parent_layer`/`parent_map` | `graph.rs:118-125` | ✅ |
| 12 | SilkIndex 37 horizontal buckets (8+8+8+8+5) | `index.rs:22-30, 84-97` | ✅ |
| 13 | `implicit_silk()` và `implicit_neighbors()` tồn tại | `index.rs:180, 270` | ✅ |
| 14 | `ImplicitSilk` có `shared_dims` + `strength` + `shared_count` | `index.rs:50-71` | ✅ |
| 15 | `HebbianLink` 19 bytes, KHÔNG có EmotionTag | `edge.rs:327-334` | ✅ |
| 16 | `SilkEdge` 46 bytes, CÓ EmotionTag | `edge.rs:231-253, 296` | ✅ |
| 17 | cluster_score = α×chain_sim + β×hebbian + γ×co_act | `dream.rs:293` | ✅ |
| 18 | MolSummary::similarity() delta-based | `graph.rs:44-91` | ✅ |
| 19 | ImplicitSilk chưa tách match/precision | `index.rs:180-236` | ❌ đã tách (0.20 exact, 0.15 base) |

---

### VỀ NỘI DUNG MỚI: "Nguồn gốc thiết kế" + "Thiết kế gốc"

Spec mới thêm ~470 dòng phân tích thiết kế từ `node va silk.md`. Nội dung này:
- **Hữu ích** cho context — giải thích TẠI SAO Maturity cần wire
- **Chính xác** về mặt kỹ thuật (tất cả code references verify đúng)
- **Lặp lại** — "Ý tưởng cốt lõi 1-4" (dòng 609-738) gần như trùng nội dung với "Nguồn gốc thiết kế" (dòng 11-127). Cùng topic được viết 2 lần.

**Khuyến nghị:** Gộp 2 section trùng lặp thành 1, giảm ~200 dòng. Section "Thiết kế gốc từ node va silk.md" (dòng 605-751) có thể xóa vì nội dung đã có ở đầu file.

---

### TỔNG KẾT AUDIT

```
🔴 CRITICAL:  advance(weight=0.0) bug — Mature state unreachable, 1 test sẽ FAIL
🟡 MINOR:     Test count sai (2063 → 2227)
🟡 MINOR:     ImplicitSilk đã tách match/precision — spec nói chưa có
🟡 STYLE:     ~200 dòng nội dung lặp (section đầu vs section cuối)
🟢 POSITIVE:  95% claims kỹ thuật chính xác (18/19)
🟢 POSITIVE:  Phân tích #1-#6 chi tiết và chính xác
🟢 POSITIVE:  Context thiết kế từ node va silk.md rất hữu ích
```

**Trước khi implement spec này, PHẢI sửa bug weight=0.0.**

---

*Audit completed 2026-03-18 · Claude automated verification · synced with main*
