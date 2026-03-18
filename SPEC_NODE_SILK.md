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
| Compound | 31 mẫu (C(5,1)+...+C(5,5)) | 31 | **Chưa implement** |
| Precise | ~5400 kênh (= số L0 nodes) | ~5400 | **Chưa implement** |

2 hướng Silk:

| Hướng | Tên | Lưu trữ | Trạng thái |
|-------|-----|---------|-----------|
| Ngang | Silk tự do (implicit, cùng tầng) | 0 bytes | **SilkIndex ✅** |
| Dọc | Silk đại diện (parent pointer) | 5460 × 8B = 43 KB | **Chưa implement** |

---

## 5 Gaps giữa thiết kế và code

### Gap #1 — Silk dọc (parent pointer) chưa có

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

**Code thực tế (`graph.rs:118-125`):**
```rust
pub struct SilkGraph {
    edges: Vec<SilkEdge>,
    index: SilkIndex,
    learned: Vec<HebbianLink>,
    // ❌ Không có parent_map
}
```

**Hệ quả:**
- Không thể query "concept cha của node X"
- `co_activate_same_layer()` nhận layer từ caller — không có nguồn sự thật independent
- Dream clustering không biết 2 nodes có cùng tầng không
- Cross-layer Silk (`co_activate_cross_layer()`) không có cấu trúc dọc để anchor
- Truy vấn O(1) qua parent pointer (thiết kế gốc) — impossible

**Effort:** Trung bình | **Impact:** Cao — nền tảng cho tất cả layer-aware operations

---

### Gap #2 — 31 compound patterns chưa implement

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

**Thiếu gì:**
- Enum `CompoundPattern` với 31 variants
- Method `classify_compound(shared_dims) -> CompoundPattern`
- Method `compound_name(pattern) -> &str` (tên tự nhiên)
- Integration vào Dream clustering và response rendering

**Effort:** Nhỏ-Trung bình | **Impact:** Trung bình — giàu ngữ nghĩa cho response + Dream

---

### Gap #3 — Dream bỏ qua 5D similarity

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

**Effort:** Trung bình | **Impact:** Cao — unblock Dream clustering

---

### Gap #4 — Dream không kiểm tra layer

**Thiết kế (QT⑪):** Silk chỉ ở Ln-1 — tự do giữa lá cùng tầng.

**Code thực tế:** Dream clustering không filter observations theo layer. Tất cả observations được cluster chung, bất kể layer.

**Verification (`dream.rs`):**
- Không gọi `co_activate_same_layer()` (cần layer param)
- Không gọi `co_activate_cross_layer()` (cần layer + fire_count)
- `Observation` struct (`learning.rs:50-59`) không có field `layer`

**Hệ quả:**
- Dream có thể cluster L0 node với L2 node → vi phạm QT⑪
- Khi promote cluster → LCA không biết input thuộc layer nào → output layer sai
- Kết hợp Gap #1 (no parent pointer): Dream hoàn toàn blind về cấu trúc tầng

**Effort:** Nhỏ | **Impact:** Trung bình — correctness cho Dream clustering

---

### Gap #5 — `unified_neighbors()` không được dùng

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

**Hiện tại `unified_neighbors()` chỉ được gọi trong tests.**

**Effort:** Nhỏ | **Impact:** Trung bình — method tốt, chưa được wire

---

## Thay đổi đề xuất

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

## Thứ tự thực hiện

```
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
| #3 Maturity pipeline | Gap #3 Dream 5D | Dream cần cả Maturity + 5D để cluster đúng |
| #4 Dream threshold | Gap #3 Dream 5D | implicit Silk bonus tăng cluster score → giảm cần hạ threshold |
| #5 Silk parent pointer | Gap #1 Parent map | Cùng vấn đề, spec này chi tiết hơn |
| — | Gap #2 Compound patterns | Mới — chưa có trong SPEC_MATURITY |
| — | Gap #4 Layer awareness | Mới — Dream không filter layer |
| — | Gap #5 unified_neighbors | Mới — method tốt chưa được wire |

**Khuyến nghị:** Implement Gap #1 (parent pointer) và Gap #3 (Dream 5D) cùng lúc với SPEC_MATURITY Thay đổi 1-2, vì chúng touch cùng files và bổ trợ nhau.

---

*HomeOS · 2026-03-18 · Node & Silk gaps · Design vs Implementation*
