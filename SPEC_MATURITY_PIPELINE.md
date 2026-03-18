# Spec: Wire Maturity vào Dream Pipeline

**Ngày:** 2026-03-18  
**Ưu tiên:** HIGH  
**Scope:** 3 files, không thay đổi public API  
**Mục tiêu:** Node chuyển từ công thức (Formula) → đang học (Evaluating) → chín (Mature) dựa trên evidence thật từ STM và Dream.

---

## Bối cảnh

File `old/2026-03-18/node va silk.md` xác định:

> *"DNA không lưu CƠ THỂ. DNA lưu CÔNG THỨC TẠO cơ thể."*
> *"Khi chưa có input → công thức = TIỀM NĂNG. Khi có input → thế vào → GIÁ TRỊ CỤ THỂ. Khi đủ giá trị → node CHÍN."*

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

### Tests cần thêm (cuối file, trong `#[cfg(test)]`)
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
    stm.push(chain.clone(), EmotionTag::NEUTRAL, 1); // fire_count → 2 ≥ fib(2)=2
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

**Bước 1b:** Cập nhật early return (khi `scanned < min_cluster_size`) để bao gồm matured_nodes:
```rust
if scanned < self.config.min_cluster_size {
    return DreamResult {
        scanned,
        clusters_found: 0,
        proposals: Vec::new(),
        approved: 0,
        rejected: 0,
        matured_nodes,   // ← trả về ngay cả khi không đủ cluster
    };
}
```

**Bước 1c:** Trong return cuối của `run()`, thêm field:
```rust
DreamResult {
    scanned,
    clusters_found,
    proposals,
    approved: approved_count,
    rejected: rejected_count,
    matured_nodes,   // ← THÊM
}
```

### Tests cần thêm
```rust
#[test]
fn dream_detects_mature_nodes() {
    // Tạo STM với observation fire nhiều lần
    let mut stm = ShortTermMemory::new(512);
    let chain = olang::encoder::encode_codepoint(0x1F525);
    // Push nhiều lần để fire_count cao
    for i in 0..10 {
        stm.push(chain.clone(), EmotionTag::NEUTRAL, i as i64 * 1000);
    }
    let graph = SilkGraph::new();
    let dream = DreamCycle::new(DreamConfig {
        tree_depth: 2,  // fib(2) = 2 — threshold thấp để test
        ..Default::default()
    });
    let result = dream.run(&stm, &graph, 10000);
    assert!(
        !result.matured_nodes.is_empty(),
        "fire_count=10 >> fib(2)=2 → phải có mature nodes"
    );
}

#[test]
fn dream_no_mature_nodes_when_fire_low() {
    let mut stm = ShortTermMemory::new(512);
    let chain = olang::encoder::encode_codepoint(0x1F525);
    stm.push(chain.clone(), EmotionTag::NEUTRAL, 0); // fire_count = 1
    let graph = SilkGraph::new();
    let dream = DreamCycle::new(DreamConfig {
        tree_depth: 5,  // fib(5) = 8 — threshold cao
        ..Default::default()
    });
    let result = dream.run(&stm, &graph, 1000);
    assert!(
        result.matured_nodes.is_empty(),
        "fire_count=1 << fib(5)=8 → không mature"
    );
}

#[test]
fn dream_result_has_matured_nodes_field() {
    // Chỉ verify field tồn tại và có thể access
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

### Mục tiêu
Không bỏ sót thông tin matured_nodes — ít nhất phải log/emit để traceability.

### Tìm và sửa
Tìm tất cả chỗ trong codebase gọi `dream.run()` hoặc `DreamCycle::run()` (thường trong `crates/runtime/src/` hoặc `crates/agents/`).

Sau khi gọi, thêm xử lý `matured_nodes`:

```rust
let result = dream.run(&stm, &graph, ts);

// Log matured nodes để traceability
// (Chưa ghi Registry — chờ pipeline hoàn chỉnh)
if !result.matured_nodes.is_empty() {
    // Emit VmEvent::Output hoặc ghi vào stats
    // Dùng pattern hiện có của codebase — không tạo mới
    let _ = &result.matured_nodes; // tối thiểu: không bỏ qua
}
```

**Lưu ý:** Chưa ghi vào Registry trong bước này — chỉ đảm bảo thông tin không bị drop silently. Ghi Registry là bước tiếp theo sau khi pipeline ổn định.

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

Nếu có test nào trong `dream.rs` cũ fail vì thiếu field `matured_nodes` trong `DreamResult` → sửa bằng cách thêm `matured_nodes: Vec::new()` vào chỗ đó.

---

## Tại sao làm theo thứ tự này

`learning.rs` → `dream.rs` → caller vì dependency một chiều:
- Dream đọc từ STM (Observation) → Observation phải có maturity trước
- Caller dùng DreamResult → DreamResult phải có matured_nodes trước
- Không có circular dependency

---

## Sau khi xong spec này

Bước tiếp theo (spec riêng):
1. Silk dọc — parent pointer giữa các tầng
2. Ghi maturity vào Registry khi Dream approve
3. Molecule = công thức thật (evaluate từ sensor data)

---

*HomeOS · 2026-03-18 · Maturity pipeline · Formula → Evaluating → Mature*
