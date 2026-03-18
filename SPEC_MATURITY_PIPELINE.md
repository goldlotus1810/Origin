# Spec: Wire Maturity vào Dream Pipeline

**Ngày:** 2026-03-18  
**Cập nhật:** 2026-03-18 (sau audit old/2026-03-18/)  
**Ưu tiên:** HIGH  
**Scope:** 3 files, không thay đổi public API  
**Mục tiêu:** Node chuyển từ công thức (Formula) → đang học (Evaluating) → chín (Mature) dựa trên evidence thật từ STM và Dream.

---

## Bối cảnh

File `old/2026-03-18/node va silk.md` xác định:

> *"DNA không lưu CƠ THỂ. DNA lưu CÔNG THỨC TẠO cơ thể."*  
> *"Khi chưa có input → công thức = TIỀM NĂNG. Khi có input → thế vào → GIÁ TRỊ CỤ THỂ. Khi đủ giá trị → node CHÍN."*

Enum `Maturity` (`Formula → Evaluating → Mature`) đã tồn tại trong `crates/olang/src/mol/molecular.rs` với logic `advance()` đúng, nhưng **chưa được wire vào bất kỳ pipeline nào**. `Observation` không track maturity, `DreamResult` không trả về nodes nào đã chín.

### Vấn đề gốc rễ (từ audit 8 files old/2026-03-18/)

Đọc toàn bộ tài liệu cũ cho thấy **6 vấn đề hệ thống** tồn tại qua nhiều phiên (A→K), được nhắc lại trong 6-7/8 files nhưng chưa sửa. Spec này giải quyết vấn đề số 3 (Maturity pipeline). Các vấn đề còn lại được liệt kê chi tiết ở cuối.

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

**Khi tìm thấy observation đã có** (tăng `fire_count`), cập nhật maturity ngay sau khi tăng:
```rust
obs.fire_count += 1;
// fib(2) = 2 — threshold cho STM (depth=2)
let fib_threshold = silk::hebbian::fib(2);
obs.maturity = obs.maturity.advance(obs.fire_count, 0.0, fib_threshold);
// blend emotion như cũ...
```

**Khi tạo Observation mới** (push lần đầu), thêm field:
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

Đọc 8 files trong `old/2026-03-18/` cho thấy 6 vấn đề hệ thống tồn tại qua nhiều phiên (A→K). Được nhắc lại trong 6-7/8 files nhưng không phiên nào sửa được vì mỗi phiên AI mới lại ưu tiên thêm tính năng mới thay vì sửa mặt tiền. Spec này giải quyết #3.

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

1. **Instinct output bị bỏ qua.** `LeoAI.run_instincts()` chạy đủ 7 bản năng và trả về kết quả — Causality phát hiện "mất việc → buồn", Abstraction tạo LCA("buồn", "mất việc") = "mất mát", Analogy tìm được các khái niệm tương đồng. Nhưng `render_response()` trong `origin.rs` không đọc các kết quả này — chỉ dùng V/A score từ ConversationCurve để chọn tone, sau đó lookup trong `response_template.rs`.

2. **Silk walk kết quả không dùng.** `SilkGraph.unified_neighbors()` đã tìm được các nodes liên quan qua 5D similarity và Hebbian links — nhưng kết quả walk này không được đưa vào response text. Response không nhắc đến bất kỳ entity nào user đã nói.

3. **Template quá thưa.** `response_template.rs` có ~10 câu phân theo 6 tone (Supportive, Gentle, Reinforcing, Celebratory, Pause, Engaged). Mỗi tone chỉ có 1-2 câu, không có slot nào cho nội dung động.

**Hướng sửa:**
```rust
// Trong origin.rs, hàm render_response() — thêm 3 nguồn dữ liệu:

// 1. Lấy topic từ Silk walk (top 2 related nodes)
let related = silk_graph.unified_neighbors(input_hash, Some(&mol_summary));
let topic_hint = related.first().map(|n| registry.lookup_alias(n.hash));

// 2. Lấy insight từ instinct output
let causal_hint = instinct_results.iter()
    .find(|r| matches!(r.kind, InsightKind::Causal { .. }));

// 3. Compose response với nội dung thật
// Thay vì: "Bạn đang tìm hiểu để làm gì?"
// Thành:   "Nghe như [topic_hint] đang ảnh hưởng [cảm xúc] của bạn."
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

**Gốc rễ:** `is_command()` trong `parser.rs` chỉ nhận: `dream`, `stats`, `health`, `seed`, `shutdown`, `reboot`, `status`, `help` + math commands + leo commands. 6 lệnh debug/reasoning thiếu trong danh sách này nên parser route chúng sang text query → alias lookup → fail vì không tìm được alias.

Paradox: `handle_command()` trong `origin.rs` **đã có** code xử lý đầy đủ cho cả 6 lệnh này — chỉ thiếu routing từ parser đến đó.

**Hướng sửa:**
```rust
// Trong parser.rs, hàm is_command() — thêm 6 dòng:
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

Xem chi tiết ở các section Thay đổi 1-3 ở trên.

**Tóm tắt:** `Maturity` enum + `advance()` đã có nhưng không wire. `Observation` không track trạng thái. `DreamResult` không báo cáo nodes chín. Node không biết mình là công thức hay đã có giá trị thật.

---

### #4 — Dream threshold quá cao

**Được nhắc trong:** AUDIT, USER_PERSPECTIVE_REVIEW, NEXT_PLAN  
**File cần sửa:** `crates/memory/src/dream.rs`  
**Effort:** Nhỏ | **Impact:** Cao

**Triệu chứng:**
```
Dream cycle chạy → scanned: 2, clusters: 0, proposals: 0, approved: 0
Dream cycle chạy → scanned: 1, clusters: 0, proposals: 0, approved: 0
→ Không phiên nào Dream học được gì
```

**Gốc rễ — số học:**

```
Pipeline học 1 observation/turn (process_one gọi stm.push 1 lần)
Sau 5 turns → STM có 5 observations (nếu 5 câu khác nhau hoàn toàn)
                      hoặc 1-2 (nếu câu lặp lại → fire_count tăng, không thêm entry)

DreamConfig::default():
  min_cluster_size = 3     → cần ít nhất 3 observations cùng chủ đề
  cluster_threshold = 0.6  → cần score >= 0.6 để nhóm

cluster_score = 0.3×chain_sim + 0.4×hebbian_weight + 0.3×fire_ratio

Vấn đề: chain_sim giữa câu "tôi buồn" và "mất việc" thấp (<0.3)
         hebbian_weight mới tạo = 0.1 (khởi đầu yếu)
         → score ≈ 0.3×0.2 + 0.4×0.1 + 0.3×0 = 0.10 << 0.6

→ KHÔNG BAO GIỜ cluster được trong hội thoại thông thường
```

**Hướng sửa:**
```rust
// Thêm vào DreamConfig (dream.rs):
impl DreamConfig {
    /// Preset cho hội thoại thật — threshold thấp hơn default.
    pub fn for_conversation() -> Self {
        Self {
            scan_top_n: 32,
            cluster_threshold: 0.30,  // giảm từ 0.6 → 0.30
            min_cluster_size: 2,      // giảm từ 3 → 2
            tree_depth: 2,            // fib(2)=2, dễ promote hơn
            alpha: 0.4,               // tăng weight cho chain_sim
            beta: 0.3,
            gamma: 0.3,
        }
    }
}

// Trong HomeRuntime::new() hoặc boot sequence:
// Dùng DreamConfig::for_conversation() thay vì DreamConfig::default()
```

---

### #5 — Silk dọc (parent pointer) chưa có

**Được nhắc trong:** silk_architecture.md, node va silk.md  
**File cần sửa:** `crates/silk/src/graph.rs`  
**Effort:** Trung bình | **Impact:** Cao (nền tảng cho layer-aware queries)

**Triệu chứng:**
- Không thể query "cho tôi biết concept cha của node này ở L2"
- Dream clustering không biết 2 nodes có cùng tầng không
- `co_activate_same_layer()` nhận layer từ caller nhưng không có nguồn sự thật

**Gốc rễ:**

`silk_architecture.md` mô tả rõ cấu trúc dọc:
```
L7: ○ (1 node)
L6: [Unity] (1 node)      ← parent của L5
L5: [H]═══[V] (2 nodes)   ← parent của L4
...
L1: 37 nodes              ← parent của L0 buckets
L0: 5400 UCD atoms

5460 parent pointers = 43 KB tổng — mỗi node có 1 pointer lên tầng trên
```

Hiện tại `SilkGraph` không có field này. `SilkIndex` chỉ có 37 horizontal buckets theo base value, không có chiều dọc.

**Hướng sửa:**
```rust
// Trong graph.rs — thêm vào SilkGraph:
pub struct SilkGraph {
    edges: Vec<SilkEdge>,
    index: SilkIndex,
    learned: Vec<HebbianLink>,
    // ← THÊM:
    parent_map: alloc::collections::BTreeMap<u64, u64>, // child_hash → parent_hash
}

impl SilkGraph {
    /// Đăng ký quan hệ cha-con giữa 2 tầng.
    /// Gọi khi Dream promote node lên layer cao hơn.
    pub fn register_parent(&mut self, child_hash: u64, parent_hash: u64) {
        self.parent_map.insert(child_hash, parent_hash);
    }

    /// Lấy parent của node tại tầng trên.
    pub fn parent_of(&self, hash: u64) -> Option<u64> {
        self.parent_map.get(&hash).copied()
    }

    /// Lấy tất cả children của một parent node.
    pub fn children_of(&self, parent_hash: u64) -> alloc::vec::Vec<u64> {
        self.parent_map
            .iter()
            .filter(|(_, &p)| p == parent_hash)
            .map(|(&c, _)| c)
            .collect()
    }
}
```

Khi boot, build parent_map từ Registry layer info. Khi Dream promote node → gọi `register_parent()`.

---

### #6 — Agent hierarchy là code chết

**Được nhắc trong:** AUDIT, USER_PERSPECTIVE_REVIEW, PLAN_PHAN_VIEC  
**File cần sửa:** `crates/runtime/src/core/origin.rs`, `crates/agents/src/hierarchy/chief.rs`  
**Effort:** Lớn | **Impact:** Cao nhưng phức tạp

**Triệu chứng:**
```
Router Ticks:     2-4 mỗi phiên
Worker→Chief:     0
Chief→LeoAI:      0
Chief↔Chief:      0
Workers:          0 (không bao giờ được spawn)
ISL messages:     0 forwarded
```

**Gốc rễ — 3 tầng:**

1. **Không có trigger.** Router.tick() được gọi mỗi turn nhưng không có message nào để forward vì không ai tạo message. Chiefs không tự generate message. Workers chưa được spawn.

2. **Intent routing thiếu.** Khi user nói "bật đèn phòng khách", `estimate_intent()` phân loại là `Chat` hoặc `Command` — không có `HomeControl` intent. Không có code nào route HomeControl → HomeChief.

3. **HAL chưa kết nối thiết bị thật.** `hal::detect::platform()` nhận biết x86/ARM/RISC-V đúng nhưng không có driver thật nào phía sau. Workers cần hardware events để biết có gì để làm.

**Hướng sửa (theo thứ tự):**

Bước A — thêm `HomeControl` intent và route đến HomeChief:
```rust
// Trong context/analysis/intent.rs — thêm variant:
pub enum IntentAction {
    Chat,
    Learn,
    Command,
    Crisis,
    HomeControl { device_hint: Option<String> },  // ← THÊM
}

// Trong origin.rs — thêm routing:
if let IntentAction::HomeControl { device_hint } = &intent {
    let msg = ISLMessage::new(LEO_ADDR, HOME_CHIEF_ADDR, MsgType::Task, ...);
    self.router.send(msg);
}
```

Bước B — Mock Workers cho test:
```rust
// Trong origin.rs boot sequence:
let mock_light_worker = Worker::new("mock_light", WorkerKind::Device);
self.register_worker(mock_light_worker);
```

Bước C — Wire Chiefs xử lý ISL message:
```rust
// Trong chief.rs — implement process_task():
fn process_task(&mut self, msg: &ISLMessage) -> Option<ISLMessage> {
    // Phân tích task, dispatch đến Worker phù hợp
    // Trả về response ISL message
}
```

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
*Cập nhật sau audit old/2026-03-18/ — 6 vấn đề hệ thống với chi tiết đầy đủ*
