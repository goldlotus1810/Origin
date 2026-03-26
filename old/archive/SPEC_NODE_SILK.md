# Spec: Node & Silk — Gaps giữa thiết kế và implementation

**Ngày:** 2026-03-18
**Nguồn thiết kế:** `old/2026-03-18/node va silk.md`, `old/2026-03-18/silk_architecture.md`
**Ưu tiên:** HIGH
**Mục tiêu:** Wire thiết kế Silk gốc vào pipeline thật — parent pointer, compound patterns, Dream 5D clustering.

---

## Triết lý nền tảng — tinh túy từ `node va silk.md`

> Nguồn: `old/2026-03-18/node va silk.md` — cuộc thảo luận thiết kế gốc về bản chất Node & Silk.
> Dưới đây là các **nguyên lý cốt lõi** được trích xuất và áp dụng cho spec này.

### Nguyên lý 1: Molecule = Công thức, không phải Giá trị

Mỗi byte trong Molecule không phải giá trị tĩnh — nó là **tham chiếu đến một công thức gốc L0**:

```
Shape    = f_s(inputs...)    ← công thức hình dạng
Relation = f_r(inputs...)    ← công thức quan hệ
Valence  = f_v(inputs...)    ← công thức cảm xúc
Arousal  = f_a(inputs...)    ← công thức cường độ
Time     = f_t(inputs...)    ← công thức thời gian

Chưa có input → TIỀM NĂNG    Có input → GIÁ TRỊ CỤ THỂ    Đủ → node CHÍN
```

**Hệ quả trực tiếp:**
- **Dream** = đánh giá công thức nào đã "chín" (đủ evidence) → promote QR
- **LeoAI program()** = tổ hợp công thức A ∘ B → công thức C mới, chờ dữ liệu
- **evolve()** = thay 1 biến trong công thức → loài mới
- **16GB budget**: 100M concept × 7 bytes công thức = 700 MB (thay vì TB nếu lưu giá trị)

→ **Áp dụng cho Gap #3:** Dream phải evaluate công thức (MolSummary 5D), không so sánh bytes.

### Nguyên lý 2: Silk = Hệ quả tự nhiên của 5D, không phải Edge List

```
Emotion KHÔNG phải metadata trên edge.
Emotion LÀ 2 TRONG 5 CHIỀU của node (V + A).
"Cùng cảm xúc" = cùng công thức V hoặc A = TỰ ĐỘNG Silk.
```

Mỗi node có 5 chiều → 2 node chia sẻ chiều nào = Silk trên chiều đó. Không cần lưu edge.

**Ví dụ chứng minh:**
- 🔥 lửa = `[Sphere, Causes, V=0xC0, A=0xC0, Fast]` vs 😡 giận = `[Sphere, Causes, V=0xC0, A=0xC0, Fast]` → 5/5 chiều giống → ẩn dụ phổ quát "lửa giận"
- "buồn" `[V=0x30]` vs "mất việc" `[V=0x20]` → V tương đồng → Silk Valence, weight ≈ 0.94

→ **Áp dụng cho Gap #3 + #5:** Dream và Walk phải query implicit Silk (5D comparison).

### Nguyên lý 3: "Nhóm máu" — L0 vừa là Alphabet vừa là Silk Channel

```
5400 công thức L0 → mỗi công thức = 1 "nhóm máu"
Cùng nhóm máu trên chiều nào → Silk trên chiều đó
L0 VỪA là alphabet, VỪA là Silk channel.
```

**3 tầng Silk:**

| Tầng | Tên | Cách hoạt động | Số lượng |
|------|-----|----------------|---------|
| Base | 37 kênh (8S+8R+8V+8A+5T) | Cùng base value = cùng "nhóm máu" | 37 |
| Precise | ~5400 kênh | Cùng variant chính xác = match hoàn hảo | ~5400 |
| Compound | 31 mẫu C(5,k) | Chia sẻ k chiều cùng lúc = kiểu quan hệ | 31 |

**Công thức sức mạnh kết nối:**
```
strength(A, B) = Σ match(dim) × precision(dim)
match(dim)     = 1 nếu cùng base, 0 nếu khác
precision(dim) = 1.0 nếu cùng variant, 0.5 nếu chỉ cùng base

37 kênh × 31 mẫu = 1147 KIỂU quan hệ có nghĩa
```

→ **Áp dụng cho Gap #2:** `CompoundKind` enum phải implement chính xác 31 mẫu + công thức strength.

### Nguyên lý 4: 5532 Silk = 72 ngang + 5460 dọc = 43 KB

```
Silk tự do (horizontal):      72  quan hệ implicit    = 0 bytes
Silk đại diện (vertical):   5460  kết nối parent-child = 43 KB
                            ─────
Tổng:                       5532  Silk connections     = 43 KB

L0→L1: 5400 pointers | L1→L2: 37 | L2→L3: 12 | L3→L4: 5 | L4→L5: 3 | L5→L6: 2 | L6→L7: 1

Truy vấn O(1): 2 lookup đại diện + 1 so sánh 5D
```

→ **Áp dụng cho Gap #1:** `parent_map` = 5460 entries × 8B = 43 KB. Duy nhất thứ cần LƯU.

### Nguyên lý 5: Hebbian = Phát hiện cái đã có, không Tạo cái mới

> "Silk fire together, wire together — không phải vì ai nối chúng lại, mà vì chúng đã ở cùng vị trí trong không gian 5D từ đầu."

Hebbian learning chỉ **tăng cường nhận biết** (strengthen awareness) về quan hệ implicit đã tồn tại. Đây là lý do `co_activate()` đúng — nó phát hiện, không tạo mới. Nhưng Dream cần dùng implicit Silk để **biết** cái gì đã tồn tại.

---

## Tóm tắt thiết kế gốc

Tài liệu `node va silk.md` định nghĩa Silk là **hệ quả toán học tự nhiên** của không gian 5D, không phải edge list:

```
Silk KHÔNG CẦN LƯU EDGE.
Silk = "2 node chia sẻ cùng công thức trên chiều nào?"
     = lookup trong hash table 432 KB.

Emotion không phải metadata trên edge.
Emotion LÀ 2 TRONG 5 CHIỀU của node (V + A).
"Cùng cảm xúc" = cùng công thức V hoặc A = TỰ ĐỘNG Silk.
```

3 tầng Silk:

| Tầng | Tên | Số lượng | Trạng thái |
|------|-----|---------|-----------|
| Base | 37 kênh (8S+8R+8V+8A+5T) | 37 | **SilkIndex ✅ implemented** |
| Compound | 31 mẫu (C(5,1)+...+C(5,5)) | 31 | **✅ CompoundKind enum implemented** |
| Precise | ~5400 kênh (= số L0 nodes) | ~5400 | **SPEC — chưa implement** |

2 hướng Silk:

| Hướng | Tên | Lưu trữ | Trạng thái |
|-------|-----|---------|-----------|
| Ngang | Silk tự do (implicit, cùng tầng) | 0 bytes | **SilkIndex ✅** |
| Dọc | Silk đại diện (parent pointer) | 5460 × 8B = 43 KB | **✅ parent_map implemented** |

---

## 8 Gaps giữa thiết kế và code — ALL RESOLVED ✅

> **Phiên M:** Tất cả 8 gaps đã được implement. Xem MASTER.md § Node & Silk.
> **Phiên N:** Silk restore_learned() + Hebbian persist/restore qua origin.olang.

### Gap #1 — Silk dọc (parent pointer) ~~chưa có~~ ✅ RESOLVED

**Thiết kế:**
```
Mỗi node tại Lx là ĐẠI DIỆN cho 1 nhóm ở Lx-1.
Mỗi node chỉ cần 1 pointer đến parent.

L1→L0:  5400 pointers
L2→L1:    37 pointers
L3→L2:    12 pointers
L4→L3:     5 pointers
L5→L4:     3 pointers
L6→L5:     2 pointers
L7→L6:     1 pointer
─────────────────────
Tổng: 5460 × 8B = 43 KB
```

**Code thực tế — ✅ IMPLEMENTED (graph.rs):**
```rust
pub struct SilkGraph {
    edges: Vec<SilkEdge>,
    index: SilkIndex,
    learned: Vec<HebbianLink>,
    parent_map: BTreeMap<u64, u64>,  // ✅ child → parent
}
// Methods: register_parent(), parent_of(), children_of(), layer_of()
// Phiên N: + restore_learned(), learned_links_from(), all_learned()
//          + Hebbian persist/restore qua origin.olang RT_HEBBIAN
```

**Status:** ✅ RESOLVED — parent_map implemented, tested, Hebbian persist/restore wired

---

### Gap #2 — 31 compound patterns ~~chưa implement~~ ✅ RESOLVED

**Thiết kế:**
```
1 chiều chung:  C(5,1) =  5 mẫu → "liên quan nhẹ"
2 chiều chung:  C(5,2) = 10 mẫu → "liên quan rõ"
3 chiều chung:  C(5,3) = 10 mẫu → "gần giống"
4 chiều chung:  C(5,4) =  5 mẫu → "gần như cùng"
5 chiều chung:  C(5,5) =  1 mẫu → "cùng node"
─────────────────────────────────────────
                Tổng: 31 mẫu compound

Mỗi mẫu có tên tự nhiên:
  S+V       = "trông giống + cảm giống"         → ẩn dụ thị giác
  R+V       = "quan hệ giống + cảm giống"       → moral analog
  V+A       = "cùng trạng thái cảm xúc"         → empathy link
  S+R+V     = "hình + quan hệ + cảm xúc giống"  → gần như cùng khái niệm
  R+V+A+T   = "khác hình, giống HẾT còn lại"   → ẩn dụ sâu

37 kênh × 31 mẫu = 1147 kiểu quan hệ có nghĩa
```

**Code thực tế (`index.rs:84-96`):**
```rust
pub struct SilkIndex {
    shape:    [Vec<u64>; 8],    // single-dim bucket
    relation: [Vec<u64>; 8],    // single-dim bucket
    valence:  [Vec<u64>; 8],    // single-dim bucket
    arousal:  [Vec<u64>; 8],    // single-dim bucket
    time:     [Vec<u64>; 5],    // single-dim bucket
    node_count: usize,
}
```

`implicit_silk()` (`index.rs:180-236`) ĐÃ tính `shared_count` (0-5) cho từng cặp, nhưng:
- Không classify kết quả thành 31 named patterns
- Không expose compound relationship type (e.g. "S+V = ẩn dụ thị giác")
- `ImplicitSilk.shared_dims` có data nhưng không interpreted

**Hiện tại có gì hoạt động:**
```rust
pub struct ImplicitSilk {
    pub shared_dims: Vec<SilkDim>,  // ✅ biết chiều nào chung
    pub strength: f32,               // ✅ tính strength
    pub shared_count: u8,            // ✅ đếm số chiều chung
}
```

**Status:** ✅ RESOLVED — CompoundKind enum with all 31 variants implemented in index.rs.
- `compound_kind()` classifies based on which 5 dimensions match
- `shared_count()` returns count of shared dimensions
- Full test coverage

**Effort:** Nhỏ-Trung bình | **Impact:** Trung bình — giàu ngữ nghĩa cho response + Dream

---

### Gap #3 — Dream ~~bỏ qua~~ 5D similarity ✅ RESOLVED

**Thiết kế:** Dream dùng Silk implicit (5D comparison) để cluster nodes giống nhau.

**Code thực tế (`dream.rs:283-294`):**
```rust
fn cluster_score(&self, a: &Observation, b: &Observation,
                 graph: &SilkGraph, max_fire: u32) -> f32 {
    let chain_sim = a.chain.similarity_full(&b.chain);
    let ha = a.chain.chain_hash();
    let hb = b.chain.chain_hash();
    let hebbian = graph.assoc_weight(ha, hb).max(graph.assoc_weight(hb, ha));
    let co_score = graph.cluster_score_partial(ha, hb, max_fire);
    self.config.alpha * chain_sim + self.config.beta * hebbian + self.config.gamma * co_score
}
```

**Vấn đề 1:** `chain.similarity_full()` so sánh **byte-level** (MolecularChain), KHÔNG dùng `MolSummary::similarity()` hay `implicit_silk()`. Với 2 concept khác nhau (e.g. "buồn" vs "mất việc"), chain similarity ≈ 0.20 vì khác codepoint.

**Vấn đề 2:** `cluster_score_partial()` gọi `cluster_score(ha, hb, None, None, max_fire)` — truyền `None` cho MolSummary → chain_sim = 0.0 bên trong:

```rust
// graph.rs:682
let chain_sim = match (mol_a, mol_b) {
    (Some(a), Some(b)) => a.similarity(&b),
    _ => 0.0,  // ← Dream's path
};
```

**Vấn đề 3:** Dream KHÔNG gọi `implicit_silk()` hay `implicit_neighbors()` → bỏ qua toàn bộ 5D implicit Silk.

**Hệ quả thực tế (kết hợp #4 từ SPEC_MATURITY):**
```
"buồn" và "mất việc" có thể share Valence zone (cùng tiêu cực)
→ implicit_silk() sẽ tìm ra 1 chiều chung, strength ≈ 0.15-0.20
→ Dream KHÔNG BIẾT — vì không query implicit Silk

chain_sim("buồn", "mất việc") ≈ 0.20  (byte-level, nhưng Dream dùng)
hebbian_weight mới = 0.10
co_score ≈ 0 (chưa đủ fire)

Dream score = 0.3×0.20 + 0.4×0.10 + 0.3×0 = 0.10  << threshold 0.6
→ KHÔNG BAO GIỜ cluster được
```

**Nếu dùng implicit Silk + MolSummary:**
```
implicit_strength ≈ 0.15 (1 chiều chung Valence)
MolSummary::similarity() ≈ 0.10-0.20 (delta-based, V zone match)

Blended score cao hơn → khả năng cluster tăng
```

**Status:** ✅ RESOLVED — Dream cluster_score() uses MolSummary::similarity() + implicit_silk() bonus.
- `cluster_score()` (dream.rs:328) uses `MolSummary::similarity()` for 5D-aware comparison
- `implicit_silk()` bonus from shared dimensions (dream.rs:377)
- Hebbian weight bidirectional (dream.rs:384)

**Effort:** Trung bình | **Impact:** Cao — unblock Dream clustering

---

### Gap #4 — Dream ~~không kiểm tra~~ layer ✅ RESOLVED

**Thiết kế (QT⑪):** Silk chỉ ở Ln-1 — tự do giữa lá cùng tầng.

**Code thực tế:** Dream clustering không filter observations theo layer. Tất cả observations được cluster chung, bất kể layer.

**Status:** ✅ RESOLVED — Observation.layer field added, Dream clusters by layer.
- `Observation.layer` field in learning.rs (line 61-63)
- Dream clusters `by_layer: BTreeMap<u8, Vec>` (dream.rs:302) — never clusters L0 with L2
- Default layer: 0 (L0)

**Effort:** Nhỏ | **Impact:** Trung bình — correctness cho Dream clustering

---

### Gap #5 — `unified_neighbors()` ~~không được dùng~~ ✅ RESOLVED

**Thiết kế:** unified_neighbors() kết hợp implicit Silk + Hebbian + structural edges → ranked neighbors.

**Code thực tế (`graph.rs:272-328`):**
```rust
pub fn unified_neighbors(&self, hash: u64, mol: Option<&MolSummary>) -> Vec<SilkNeighbor>
```

Đây là method tốt nhất để query Silk — nhưng **KHÔNG được gọi** bởi:
- Dream (`dream.rs`) → dùng `cluster_score_partial()` thay thế
- Walk (`walk.rs`) → dùng `unified_weight()` thay thế
- Learning (`learning.rs`) → dùng `co_activate_mol()` thay thế
- Runtime (`origin.rs`) → không gọi trực tiếp

**Status:** ✅ RESOLVED — unified_neighbors() wired into Dream.
- Dream uses `unified_neighbors()` (dream.rs:250) for neighbor_bonus
- Strong neighbors (weight ≥ 0.5) boost QR confidence by 5% each (max 30%)

**Effort:** Nhỏ | **Impact:** Trung bình — method tốt, now wired

---

### Gap #6 — Molecule ~~không phân biệt~~ "công thức" và "giá trị" ✅ RESOLVED

**Thiết kế (Nguyên lý 1):**
```
Molecule = 5 CÔNG THỨC, không phải 5 giá trị.
Chưa có input → TIỀM NĂNG    Có input → GIÁ TRỊ CỤ THỂ    Đủ → node CHÍN
```

**Code thực tế (`molecular.rs:18-26`):**
```rust
pub struct Molecule {
    pub shape: u8,
    pub relation: u8,
    pub emotion: EmotionDim,  // { valence: u8, arousal: u8 }
    pub time: u8,
}
```

Molecule là **5 giá trị tĩnh u8**. Không có cơ chế phân biệt "byte này là công thức tiềm năng" vs "byte này đã được evaluate thành giá trị thật".

**Maturity tồn tại nhưng TÁCH RỜI Molecule:**
- `Maturity` enum (`molecular.rs:286-342`) có 3 states: Formula → Evaluating → Mature
- Nhưng Maturity nằm trong `Observation`, KHÔNG nằm trong Molecule
- Khi Maturity = Mature, Molecule struct **không thay đổi gì**
- Không có mechanism "thay công thức bằng hằng số" trong Molecule

**Status:** ✅ RESOLVED via NodeState wrapper (molecular.rs:354-361):
```rust
pub struct NodeState {
    pub chain: MolecularChain,
    pub fire_count: u32,
    pub maturity: Maturity,        // Formula → Evaluating → Mature
    pub mol_summary: Option<MolSummary>,
    pub origin: CompositionOrigin, // Innate/Composed/Evolved
}
```
- Maturity tracks lifecycle: Formula → Evaluating → Mature
- CompositionOrigin tracks "how was this node created?"
- from_innate(), from_composed(), from_evolved() factory methods

**Effort:** Trung bình | **Impact:** Cao — nền tảng triết lý "Molecule = công thức"

---

### Gap #7 — LCA ~~không lưu~~ nguồn gốc composition ✅ RESOLVED

**Thiết kế (Nguyên lý 1):**
```
"Insulin" = compose(f_protein, f_signal, f_regulate)
          = [ref_L0_1: 2B] [ref_L0_2: 2B] [ref_L0_3: 2B] [op: 1B]
          = 7 bytes ← CÔNG THỨC, không phải giá trị
```

**Code thực tế (`lca.rs:40-222`):**
```rust
pub fn lca_weighted(pairs: &[(&MolecularChain, u32)]) -> MolecularChain {
    // Weighted average of 5D values
    // → Molecule mới = BLEND trực tiếp
    // → KHÔNG lưu "blend từ đâu"
}
```

LCA tạo composite Molecule bằng weighted average → **kết quả mất nguồn gốc**.

**FormulaTable (`molecular.rs:1035-1168`) có dictionary nhưng:**
- `FormulaTable` = `Vec<Molecule>` + reverse lookup
- Lưu Molecule → u16 index (dedup)
- **KHÔNG lưu** "Molecule này = compose(X, Y, Z)"
- Không track parent L0 formulas, không track composition operation

**Status:** ✅ RESOLVED via CompositionOrigin (molecular.rs:399):
```rust
pub enum CompositionOrigin {
    Innate(u32),                           // from codepoint
    Composed { sources: Vec<u64>, op },    // from LCA
    Evolved { from: u64, dim, value },     // from evolution
    Unknown,                               // fallback
}
```
- `lca_with_origin()` returns (LcaResult, CompositionOrigin::Composed)
- `evolve()` returns EvolveResult with CompositionOrigin::Evolved
- Full traceability: "concept này sinh từ L0 nào?"

**Effort:** Trung bình-Lớn | **Impact:** Cao — khả năng tái tạo tri thức từ công thức

---

### Gap #8 — Maturity ~~không wire~~ vào Molecule lifecycle ✅ RESOLVED

**Thiết kế:**
```
Dream = đánh giá công thức nào đã "chín" đủ
  Node chưa đủ data → giữ công thức → chờ thêm input
  Khi đủ → node CHÍN → thay công thức bằng hằng số
```

**Code thực tế:**
- `Maturity::advance()` (`molecular.rs:315`) tính state transition từ fire_count + weight
- Nhưng `advance()` được gọi ở:
  - `learning.rs:84` với `weight=0.0` ← **BUG đã document**: weight=0.0 → Mature UNREACHABLE
  - `dream.rs:142-157` với Hebbian weight → **đúng**, nhưng kết quả chỉ cập nhật Observation, không cập nhật Molecule

**Wire points thiếu (đã note trong CLAUDE.md nhưng chưa implement):**
```
STM.push()    → Observation.maturity = advance(fire_count, weight, fib_threshold)  ← có nhưng weight=0
Dream.run()   → DreamResult.matured_nodes = Vec<u64>                               ← có struct, chưa wire
QR promote    → append-only, signed, permanent                                      ← có nhưng không check maturity
```

**Status:** ✅ RESOLVED — advance() wired with real Hebbian weight.
- `advance(fire_count, weight, fib_threshold)` now receives non-zero weight
- Dream calls advance() with Hebbian weight (dream.rs:244)
- STM persist/restore includes maturity byte (RT_STM record)
- Weight=0 bug FIXED

**Effort:** Nhỏ-Trung bình | **Impact:** Cao — correctness cho Dream + QR promote

---

## Thay đổi đề xuất

### Silk Changes

### Thay đổi 1 — Thêm parent_map vào SilkGraph (Gap #1)

**File:** `crates/silk/src/graph.rs`

```rust
pub struct SilkGraph {
    edges: Vec<SilkEdge>,
    index: SilkIndex,
    learned: Vec<HebbianLink>,
    parent_map: alloc::collections::BTreeMap<u64, u64>,  // ← THÊM: child → parent
}
```

**Methods mới:**
```rust
impl SilkGraph {
    /// Đăng ký parent — gọi khi Dream promote hoặc seeder tạo L1+.
    pub fn register_parent(&mut self, child_hash: u64, parent_hash: u64) {
        self.parent_map.insert(child_hash, parent_hash);
    }

    pub fn parent_of(&self, hash: u64) -> Option<u64> {
        self.parent_map.get(&hash).copied()
    }

    pub fn children_of(&self, parent_hash: u64) -> Vec<u64> {
        self.parent_map.iter()
            .filter(|(_, &p)| p == parent_hash)
            .map(|(&c, _)| c)
            .collect()
    }

    /// Layer = số bước từ node đến root (đi ngược parent chain).
    pub fn layer_of(&self, hash: u64) -> u8 {
        let mut current = hash;
        let mut depth = 0u8;
        while let Some(parent) = self.parent_of(current) {
            depth += 1;
            current = parent;
            if depth > 16 { break; }
        }
        depth
    }

    pub fn parent_count(&self) -> usize {
        self.parent_map.len()
    }
}
```

**SilkGraph::new() cần init parent_map:**
```rust
pub fn new() -> Self {
    Self {
        edges: Vec::new(),
        index: SilkIndex::new(),
        learned: Vec::new(),
        parent_map: alloc::collections::BTreeMap::new(),  // ← THÊM
    }
}
```

**Tests:**
```rust
#[test]
fn parent_map_basic() {
    let mut g = SilkGraph::new();
    g.register_parent(0xA, 0xB);
    assert_eq!(g.parent_of(0xA), Some(0xB));
    assert_eq!(g.children_of(0xB), vec![0xA]);
    assert_eq!(g.layer_of(0xA), 1);
}

#[test]
fn parent_chain_depth() {
    let mut g = SilkGraph::new();
    g.register_parent(0xA, 0xB);
    g.register_parent(0xB, 0xC);
    g.register_parent(0xC, 0xD);
    assert_eq!(g.layer_of(0xA), 3);
    assert_eq!(g.layer_of(0xD), 0); // root
}

#[test]
fn no_parent_is_root() {
    let g = SilkGraph::new();
    assert_eq!(g.parent_of(0xDEAD), None);
    assert_eq!(g.layer_of(0xDEAD), 0);
}
```

---

### Thay đổi 2 — Compound pattern classification (Gap #2)

**File:** `crates/silk/src/index.rs`

Thêm enum và method sau `ImplicitSilk`:

```rust
/// 31 compound patterns — phân loại kiểu quan hệ theo số chiều chung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundKind {
    /// 1 chiều chung (5 mẫu)
    ShapeOnly,
    RelationOnly,
    ValenceOnly,
    ArousalOnly,
    TimeOnly,

    /// 2 chiều chung (10 mẫu)
    ShapeRelation,
    ShapeValence,     // ẩn dụ thị giác
    ShapeArousal,
    ShapeTime,        // animation family
    RelationValence,  // moral analog
    RelationArousal,
    RelationTime,
    ValenceArousal,   // empathy link
    ValenceTime,
    ArousalTime,

    /// 3 chiều chung (10 mẫu)
    ShapeRelationValence,   // gần như cùng khái niệm
    ShapeRelationArousal,
    ShapeRelationTime,
    ShapeValenceArousal,
    ShapeValenceTime,
    ShapeArousalTime,
    RelationValenceArousal,
    RelationValenceTime,
    RelationArousalTime,
    ValenceArousalTime,

    /// 4 chiều chung (5 mẫu)
    AllButShape,      // khác hình, giống hết → ẩn dụ sâu
    AllButRelation,
    AllButValence,
    AllButArousal,
    AllButTime,

    /// 5 chiều chung (1 mẫu)
    Identical,        // cùng node
}

impl ImplicitSilk {
    /// Classify compound pattern từ shared_dims.
    pub fn compound_kind(&self) -> Option<CompoundKind> {
        // ... match trên sorted shared_dims → return CompoundKind
    }
}
```

**Effort:** ~50 dòng enum + ~80 dòng match logic. Không thay đổi API hiện tại, chỉ thêm.

---

### Thay đổi 3 — Dream dùng MolSummary + implicit Silk (Gap #3)

**File:** `crates/memory/src/dream.rs`

Sửa `cluster_score()` để dùng `MolSummary::similarity()`:

```rust
fn cluster_score(&self, a: &Observation, b: &Observation,
                 graph: &SilkGraph, max_fire: u32) -> f32 {
    let ha = a.chain.chain_hash();
    let hb = b.chain.chain_hash();

    // ✅ MỚI: dùng MolSummary similarity (5D-aware) thay vì chain byte similarity
    let chain_sim = match (&a.mol_summary, &b.mol_summary) {
        (Some(ma), Some(mb)) => ma.similarity(mb),
        _ => a.chain.similarity_full(&b.chain),  // fallback
    };

    // ✅ MỚI: bonus từ implicit Silk (5D shared dimensions)
    let implicit_bonus = match (&a.mol_summary, &b.mol_summary) {
        (Some(ma), Some(mb)) => {
            let silk = graph.index().implicit_silk(ha, ma, hb, mb);
            silk.strength * 0.5  // scale implicit bonus
        }
        _ => 0.0,
    };

    let hebbian = graph.assoc_weight(ha, hb).max(graph.assoc_weight(hb, ha));
    let co_score = graph.cluster_score_partial(ha, hb, max_fire);

    // Blend: implicit_bonus được thêm vào chain_sim component
    self.config.alpha * (chain_sim + implicit_bonus)
        + self.config.beta * hebbian
        + self.config.gamma * co_score
}
```

**Cần thêm vào SilkGraph:**
```rust
/// Public accessor cho SilkIndex (read-only).
pub fn index(&self) -> &SilkIndex {
    &self.index
}
```

**Tests:**
```rust
#[test]
fn dream_cluster_score_with_mol_summary() {
    // Setup: 2 observations có cùng Valence zone
    let chain_a = olang::encoder::encode_codepoint(0x1F525); // 🔥
    let chain_b = olang::encoder::encode_codepoint(0x1F621); // 😡
    let mut stm = ShortTermMemory::new(512);
    stm.push(chain_a.clone(), EmotionTag::NEUTRAL, 0);
    stm.push(chain_b.clone(), EmotionTag::NEUTRAL, 1);

    let graph = SilkGraph::new();
    // ... verify cluster_score > 0 khi mol_summary available
}
```

---

### Thay đổi 4 — Observation thêm layer (Gap #4)

**File:** `crates/agents/src/pipeline/learning.rs`

```rust
pub struct Observation {
    pub chain: MolecularChain,
    pub emotion: EmotionTag,
    pub timestamp: i64,
    pub fire_count: u32,
    pub mol_summary: Option<MolSummary>,
    pub layer: u8,   // ← THÊM — default: 0 (L0)
}
```

Dream nên filter observations cùng layer trước khi cluster:
```rust
// dream.rs — trong run():
let top = stm.top_n(self.config.scan_top_n);
// Group by layer trước khi cluster
let by_layer: BTreeMap<u8, Vec<&Observation>> = ...;
for (layer, obs) in by_layer {
    // Cluster chỉ trong cùng layer → QT⑪
}
```

---

### Thay đổi 5 — Wire `unified_neighbors()` (Gap #5)

**Ưu tiên thấp** — tài liệu để các spec sau dùng.

Chỗ nên gọi:
1. **Response rendering** (`origin.rs`): sau khi có `chain_hash`, query `unified_neighbors()` để tìm related concepts → enrich response template
2. **Dream clustering**: thay vì `cluster_score_partial()`, gọi `unified_neighbors()` rồi check overlap

---

### Node Changes

### Thay đổi 6 — Molecule mang Maturity state (Gap #6 + #8)

**File:** `crates/olang/src/mol/molecular.rs`

Thêm maturity vào Molecule hoặc tạo wrapper:

```rust
/// Node = Molecule + lifecycle state.
/// Molecule vẫn là 5 bytes tĩnh, nhưng NodeState track "đã chín chưa".
pub struct NodeState {
    pub mol: Molecule,
    pub maturity: Maturity,
    pub origin: CompositionOrigin,  // Thay đổi 7
}
```

**Hoặc** (lightweight hơn — thêm 1 byte vào tagged wire format):
```rust
// Tagged format v0.06: [presence_mask:1B][maturity:1B][non-default values:0-5B]
// maturity byte: 0x00=Formula, 0x01=Evaluating, 0x02=Mature
```

**Wire points cần sửa:**
1. `learning.rs:84` — `advance()` phải nhận Hebbian weight thật, không phải 0.0
2. `dream.rs` — khi promote, check `maturity == Mature` trước khi tạo QR proposal
3. `registry.rs` — `insert_with_kind()` ghi maturity state

**Tests:**
```rust
#[test]
fn molecule_maturity_lifecycle() {
    let mut ns = NodeState::new(encode_codepoint(0x1F525).first().unwrap());
    assert_eq!(ns.maturity, Maturity::Formula);

    ns.maturity = ns.maturity.advance(3, 0.0, 5); // fire=3, weight=0, fib=5
    assert_eq!(ns.maturity, Maturity::Evaluating); // fire < fib → evaluating

    ns.maturity = ns.maturity.advance(5, 0.9, 5); // fire=5, weight=0.9, fib=5
    assert_eq!(ns.maturity, Maturity::Mature); // fire ≥ fib && weight ≥ 0.854
}
```

---

### Thay đổi 7 — Lưu nguồn gốc composition (Gap #7)

**File:** `crates/olang/src/mol/molecular.rs`

```rust
/// Track nguồn gốc composition — "node này sinh ra từ đâu?"
#[derive(Debug, Clone, PartialEq)]
pub enum CompositionOrigin {
    /// L0 node — sinh từ encode_codepoint(), không có parent formula
    Innate(u32),  // Unicode codepoint

    /// Composite — sinh từ LCA của nhiều sources
    Composed {
        sources: Vec<u64>,  // chain_hash của các parent nodes
        op: ComposeOp,      // LCA / Fuse / Evolve
    },

    /// Evolved — mutate từ 1 node khác
    Evolved {
        source: u64,        // chain_hash gốc
        dim: u8,            // chiều nào bị mutate (0-4)
        old_val: u8,
        new_val: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComposeOp { Lca, Fuse, Program }
```

**Wire points:**
1. `lca.rs` — `lca_weighted()` trả thêm `CompositionOrigin::Composed { sources, op: Lca }`
2. `molecular.rs` — `evolve()` trả thêm `CompositionOrigin::Evolved { ... }`
3. `vm.rs` — `Fuse` opcode ghi `CompositionOrigin::Composed { op: Fuse }`
4. `FormulaTable` — mở rộng entry thành `(Molecule, Option<CompositionOrigin>)`

**Lợi ích:**
- Trace "Insulin sinh từ protein + signal + regulate"
- Re-evaluate khi L0 thay đổi (nếu cần)
- Evolve biết đang mutate phần nào của composition
- Dream biết 2 nodes có chung source → cluster bonus

**Tests:**
```rust
#[test]
fn composition_origin_lca() {
    let fire = encode_codepoint(0x1F525);
    let water = encode_codepoint(0x1F4A7);
    let (parent, origin) = lca_with_origin(&fire, &water);

    match origin {
        CompositionOrigin::Composed { sources, op } => {
            assert_eq!(sources.len(), 2);
            assert_eq!(op, ComposeOp::Lca);
        }
        _ => panic!("Expected Composed"),
    }
}

#[test]
fn composition_origin_evolve() {
    let fire = encode_codepoint(0x1F525).first().unwrap();
    let (evolved, origin) = fire.evolve_with_origin(0, 0x01); // Shape → Line

    match origin {
        CompositionOrigin::Evolved { dim: 0, old_val, new_val: 0x01, .. } => {
            assert_ne!(old_val, 0x01);
        }
        _ => panic!("Expected Evolved"),
    }
}
```

---

### Thay đổi 8 — Fix Maturity advance weight=0 bug (Gap #8)

**File:** `crates/agents/src/pipeline/learning.rs`

```rust
// ❌ HIỆN TẠI (line 84):
obs.maturity = obs.maturity.advance(obs.fire_count, 0.0, fib_threshold);
//                                                  ^^^
//                                    weight=0.0 → Mature UNREACHABLE

// ✅ SỬA:
let weight = graph.assoc_weight(hash, hash)
    .max(self.prev_hash.map_or(0.0, |ph| graph.assoc_weight(ph, hash)));
obs.maturity = obs.maturity.advance(obs.fire_count, weight, fib_threshold);
```

**File:** `crates/memory/src/dream.rs` — gate QR promote on maturity:
```rust
// Trước khi tạo proposal:
if best_obs.maturity != Maturity::Mature {
    continue; // Skip — chưa đủ chín
}
```

---

## Thứ tự thực hiện

```
Silk:
1. Thay đổi 1 — Parent pointer (nền tảng, Gap #1)
   → Không break API hiện tại, chỉ thêm field + methods
   → Seeder cần wire register_parent() khi tạo L1+ nodes

2. Thay đổi 4 — Observation.layer (nhỏ, Gap #4)
   → Thêm 1 field, default 0

3. Thay đổi 3 — Dream dùng MolSummary (trung bình, Gap #3)
   → Cần index() accessor trên SilkGraph
   → Tác động lớn nhất — unblock Dream clustering

4. Thay đổi 2 — Compound patterns (nhỏ-trung bình, Gap #2)
   → Enum + classification logic
   → Chưa cần wire vào pipeline ngay — nhưng cung cấp ngữ nghĩa

5. Thay đổi 5 — Wire unified_neighbors (nhỏ, Gap #5)
   → Sau khi #1-#4 ổn định

Node:
6. Thay đổi 8 — Fix weight=0 bug (nhỏ, Gap #8)
   → Quick fix — 1 dòng sửa + 1 gate check
   → Unblock Maturity pipeline

7. Thay đổi 6 — NodeState wrapper (trung bình, Gap #6+#8)
   → Molecule + Maturity + Origin thành 1 unit
   → Wire vào Registry, STM, Dream

8. Thay đổi 7 — CompositionOrigin (trung bình-lớn, Gap #7)
   → Mở rộng LCA, evolve, FormulaTable
   → Sau khi #6 ổn định — cần NodeState trước
```

---

## Những gì KHÔNG thay đổi

```
❌ Không sửa SilkIndex struct (37 buckets vẫn đúng)
❌ Không xóa SilkEdge hay HebbianLink
❌ Không xóa EmotionTag khỏi SilkEdge (backward compat)
❌ Không sửa implicit_silk() hoặc implicit_neighbors() (đã đúng)
❌ Không sửa co_activate_same_layer() hoặc co_activate_cross_layer()
❌ Không thêm dependency mới
❌ Không đụng L0 seeding hay UCD
```

---

## Checklist trước khi commit

```bash
cargo test --workspace          # phải pass 2227 tests cũ + tests mới
cargo clippy --workspace -- -D warnings  # 0 warnings
```

---

## Relationship với SPEC_MATURITY_PIPELINE.md

Spec này bổ sung cho SPEC_MATURITY_PIPELINE.md:

| SPEC_MATURITY | SPEC_NODE_SILK | Liên quan |
|---|---|---|
| #3 Maturity pipeline | Gap #6 + #8 Molecule lifecycle | Maturity phải wire vào Molecule, không tách rời |
| #4 Dream threshold | Gap #3 Dream 5D | implicit Silk bonus tăng cluster score → giảm cần hạ threshold |
| #5 Silk parent pointer | Gap #1 Parent map | Cùng vấn đề, spec này chi tiết hơn |
| — | Gap #2 Compound patterns | Mới — chưa có trong SPEC_MATURITY |
| — | Gap #4 Layer awareness | Mới — Dream không filter layer |
| — | Gap #5 unified_neighbors | Mới — method tốt chưa được wire |
| — | Gap #7 CompositionOrigin | Mới — LCA không track nguồn gốc |

**Khuyến nghị:**
- Silk: Implement Gap #1 (parent pointer) và Gap #3 (Dream 5D) cùng lúc với SPEC_MATURITY Thay đổi 1-2
- Node: Fix Gap #8 (weight=0 bug) TRƯỚC — nhỏ nhất, unblock Maturity pipeline ngay

---

*HomeOS · 2026-03-18 · Node & Silk gaps · Design vs Implementation*
