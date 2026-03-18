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

Đọc toàn bộ tài liệu cũ cho thấy **5 vấn đề hệ thống** tồn tại qua nhiều phiên (A→K), được nhắc lại trong 6-7/8 files nhưng chưa sửa. Spec này giải quyết vấn đề số 3 (Maturity pipeline). Các vấn đề còn lại được liệt kê ở cuối để có spec riêng.

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
// Cập nhật maturity dựa trên fire_count mới
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
    pub matured_nodes: Vec<u64>,   // ← THÊM — chain_hash của nodes vừa đủ điều kiện Mature
}
```

### Sửa `DreamCycle::run()`

**Bước 1a:** Ngay sau `let top = stm.top_n(...)`, collect matured nodes:
```rust
let fib_threshold = silk::hebbian::fib(self.config.tree_depth);
let matured_nodes: Vec<u64> = top
    .iter()
    .filter(|obs| {
        obs.maturity.advance(obs.fire_count, 0.0, fib_threshold).is_mature()
    })
    .map(|obs| obs.chain.chain_hash())
    .collect();
```

**Bước 1b:** Cập nhật early return:
```rust
if scanned < self.config.min_cluster_size {
    return DreamResult {
        scanned,
        clusters_found: 0,
        proposals: Vec::new(),
        approved: 0,
        rejected: 0,
        matured_nodes,
    };
}
```

**Bước 1c:** Return cuối:
```rust
DreamResult {
    scanned,
    clusters_found,
    proposals,
    approved: approved_count,
    rejected: rejected_count,
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
    let dream = DreamCycle::new(DreamConfig {
        tree_depth: 2,
        ..Default::default()
    });
    let result = dream.run(&stm, &graph, 10000);
    assert!(!result.matured_nodes.is_empty(), "fire_count=10 >> fib(2)=2 → phải có mature nodes");
}

#[test]
fn dream_no_mature_nodes_when_fire_low() {
    let mut stm = ShortTermMemory::new(512);
    let chain = olang::encoder::encode_codepoint(0x1F525);
    stm.push(chain.clone(), EmotionTag::NEUTRAL, 0);
    let graph = SilkGraph::new();
    let dream = DreamCycle::new(DreamConfig {
        tree_depth: 5,
        ..Default::default()
    });
    let result = dream.run(&stm, &graph, 1000);
    assert!(result.matured_nodes.is_empty(), "fire_count=1 << fib(5)=8 → không mature");
}

#[test]
fn dream_result_has_matured_nodes_field() {
    let result = DreamResult {
        scanned: 0,
        clusters_found: 0,
        proposals: alloc::vec![],
        approved: 0,
        rejected: 0,
        matured_nodes: alloc::vec![0xDEADu64],
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

Đọc 8 files trong `old/2026-03-18/` cho thấy 6 vấn đề hệ thống tồn tại qua nhiều phiên (A→K). Spec này giải quyết vấn đề #3.

### #1 — Response template (nghiêm trọng nhất)
**Được nhắc trong:** AUDIT, USER_PERSPECTIVE_REVIEW, NEXT_PLAN, PLAN_PHAN_VIEC  
**Triệu chứng:** ~10 câu cố định cho mọi input. Causality/Abstraction/Silk walk kết quả bị bỏ qua ở bước render.  
**Gốc rễ:** Instinct output không surface vào text. ConversationCurve chọn đúng tone nhưng text không thay đổi.  
**Hướng sửa:** `crates/runtime/src/core/origin.rs` phần render — phản ánh NỘI DUNG + kết quả instinct, không chỉ TONE.  
**Effort:** Trung bình. **Impact:** Cực cao.

### #2 — Parser thiếu 6 commands (dễ sửa nhất)
**Triệu chứng:** `typeof`, `explain`, `why`, `trace`, `inspect`, `assert` trả lỗi khi dùng qua `○{}`.  
**Gốc rễ:** `is_command()` thiếu 6 keywords. `handle_command()` đã có code xử lý.  
**Hướng sửa:** Thêm vào `is_command()` trong `crates/runtime/src/core/parser.rs`.  
**Effort:** Nhỏ (vài dòng). **Impact:** Cao.

### #3 — Maturity pipeline ← spec này
Xem chi tiết ở các section trên.

### #4 — Dream threshold quá cao
**Triệu chứng:** 0 clusters, 0 proposals mỗi phiên. STM có 1-3 observations sau 5 turns.  
**Gốc rễ:** `min_cluster_size=3`, `cluster_threshold=0.6` quá cao cho hội thoại thật.  
**Hướng sửa:** Giảm xuống hoặc tạo `DreamConfig::for_conversation()` preset. File: `crates/memory/src/dream.rs`.  
**Effort:** Nhỏ. **Impact:** Cao — Dream mới thật sự học được.

### #5 — Silk dọc (parent pointer) chưa có
**Triệu chứng:** `SilkIndex` chỉ có 37 horizontal buckets. Không có cấu trúc liên tầng.  
**Gốc rễ:** `silk_architecture.md` mô tả 5460 vertical links nhưng `SilkGraph` không có field này.  
**Hướng sửa:** Thêm `parent_layer: HashMap<u64, u64>` vào `SilkGraph` + `register_parent()` API. File: `crates/silk/src/graph.rs`.  
**Effort:** Trung bình. **Impact:** Cao cho layer-aware queries.

### #6 — Agent hierarchy là code chết
**Triệu chứng:** Chiefs boot nhưng 0 messages. Workers = 0. Không có "Home" trong "HomeOS".  
**Gốc rễ:** Không có trigger mechanism, không có hardware events thật.  
**Hướng sửa:** Wire HomeChief vào flow khi user nói về thiết bị. Mock Workers trước. Files: `origin.rs`, `chief.rs`.  
**Effort:** Lớn. **Impact:** Cao nhưng phức tạp.

---

## Thứ tự thực hiện khuyến nghị

```
1. #2 — Parser commands      (vài dòng, test ngay)
2. #3 — Maturity pipeline    (spec này — PR #23)
3. #4 — Dream threshold      (nhỏ, impact lớn)
4. #1 — Response template    (trung bình, impact cực cao)
5. #5 — Silk parent pointer  (nền tảng cho tầng sau)
6. #6 — Agent hierarchy      (lớn nhất, để cuối)
```

---

*HomeOS · 2026-03-18 · Maturity pipeline · Formula → Evaluating → Mature*  
*Cập nhật sau audit old/2026-03-18/ — 6 vấn đề hệ thống được map đầy đủ*
