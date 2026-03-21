# AUDIT: L0 Errors — Lần từ gốc (UCD → Molecule → Chain → LCA → KnowTree → Registry)

> **Ngày:** 2026-03-21
> **Mục tiêu:** Kiểm tra từ cơ bản nhất — mỗi tầng lỗi sẽ lan truyền lên trên.

---

## Tóm tắt: 27 lỗi L0, chia 8 tầng

```
UCD build.rs (tầng 0) ─── 6 lỗi
    ↓ sinh ra
UCD lib.rs API (tầng 1) ─── 3 lỗi
    ↓ cung cấp cho
Molecule struct (tầng 2) ─── 4 lỗi
    ↓ tạo thành
MolecularChain (tầng 3) ─── 3 lỗi
    ↓ dùng bởi
LCA compose (tầng 4) ─── 5 lỗi
    ↓ lưu vào
KnowTree (tầng 5) ─── 3 lỗi
    ↓ serialize bởi
Writer/Reader (tầng 6) ─── 1 lỗi
    ↓ index bởi
Registry (tầng 7) ─── 2 lỗi
```

---

## Tầng 0: UCD build.rs — Nguồn gốc của mọi thứ

### E0.1 — GROUPS chỉ có 29 ranges, v2 yêu cầu 59 blocks

**File:** `crates/ucd/build.rs:45-99`

```
Hiện tại:
  SDF:      10 ranges (thiếu: Ornamental Dingbats, Misc Technical, Braille)
  MATH:      8 ranges (thiếu: 13 blocks Ancient/Siyaq/Arab)
  EMOTICON:  8 ranges (thiếu: 9 blocks Mahjong/Domino/Cards/Chess...)
  MUSICAL:   3 ranges (thiếu: 4 blocks Znamenny/Byzantine/Ancient Greek/Supp)
  ─────────
  Tổng: 29 ranges → ~5,400 entries
  v2:   59 blocks → 8,846 entries

Hậu quả: 4,184 codepoints L0 KHÔNG TỒN TẠI trong bảng
→ lookup() trả None → fallback defaults
→ L0 anchor thiếu → distance so sánh sai
```

### E0.2 — SDF_PRIMS = 8 loại, gộp CSG ops vào shapes

**File:** `crates/ucd/build.rs:105-114`

```
Hiện tại:
  SDF_PRIMS = [Sphere, Capsule, Box, Cone, Torus, Union, Intersect, Subtract]
  Union(∪), Intersect(∩), Subtract(∖) = CSG OPERATIONS, không phải SDF primitives

v2 (Section 1.5):
  18 SDF primitives: SPHERE, BOX, CAPSULE, PLANE, TORUS, ELLIPSOID,
  CONE, CYLINDER, OCTAHEDRON, PYRAMID, HEX_PRISM, PRISM, ROUND_BOX,
  LINK, REVOLVE, EXTRUDE, CUT_SPHERE, DEATH_STAR

Hậu quả: shape_of() trả về 8 loại thay vì 18
→ Mất phân biệt PLANE vs CYLINDER vs OCTAHEDRON
→ Mọi hình dạng không khớp 8 loại → fallback Sphere
```

### E0.3 — Valence hardcode không dùng udc.json

**File:** `crates/ucd/build.rs:486-614`

```
valence_of() dùng name pattern matching:
  "FIRE"|"FLAME" → 0xFF
  "HEART"|"LOVE" → 0xFF
  "SKULL"|"DEATH" → 0x00
  ...38 rules

v2: P_weight L0 xây 1 lần từ tài liệu — nhưng json/udc.json hiện có
  323,488 dòng JSON với đầy đủ valence/arousal data
  → build.rs KHÔNG đọc udc.json
  → Dùng heuristic tên → sai cho hàng ngàn codepoints

Ví dụ sai:
  "CROSS MARK" → contains("CROSS") → 0x10 (negative)
  Nhưng "CROSS" cũng match "CROSSBONES", "CROSSHATCH", "CROSSING SIGN"
  → false positives ở nhiều nơi
```

### E0.4 — Arousal formula có bug logic

**File:** `crates/ucd/build.rs:630-631`

```rust
if name.contains("PIANO") && name.contains("PIANO") {
    return (0x10, 3);  // pianissimo check
}
```

`name.contains("PIANO") && name.contains("PIANO")` = luôn true nếu PIANO xuất hiện 1 lần. Cần check xuất hiện 2 lần (pp = pianissimo).

### E0.5 — Collision resolution perturb V/A phá ngữ nghĩa

**File:** `crates/ucd/build.rs:1047-1110`

```
Phase 2: nếu 2 codepoints trùng 5-tuple → perturb valence/arousal ±1..±127

Vấn đề:
  - Emoji "vui" V=0xE0 bị shift thành V=0xE1, 0xE2, 0xDF... → sai nghĩa
  - v2 nói L0 anchors "vĩnh viễn, không thay đổi" — perturbation = vi phạm
  - Sai anchor → mọi distance so sánh sau đó sai theo
```

### E0.6 — chain_hash dựa trên 5 bytes (shape,R,V,A,T)

**File:** `crates/ucd/build.rs:877-879`

```rust
fn chain_hash(shape: u8, relation: u8, valence: u8, arousal: u8, time: u8) -> u64 {
    fnv1a_hash(&[shape, relation, valence, arousal, time])
}
```

v2: P_weight = 2 bytes → hash nên dựa trên 2B, không phải 5B.
Khi chuyển sang 2B packing, toàn bộ hash table thay đổi.

---

## Tầng 1: UCD lib.rs API

### E1.1 — lookup() binary search trên UCD_TABLE, KHÔNG O(1)

**File:** `crates/ucd/src/lib.rs:36-41`

```rust
pub fn lookup(cp: u32) -> Option<&'static UcdEntry> {
    UCD_TABLE.binary_search_by_key(&cp, |e| e.cp)
        .ok().map(|i| &UCD_TABLE[i])
}
```

v2: KnowTree[codepoint] = O(1) array lookup. UCD dùng binary search O(log n).
→ Không phải lỗi nặng (cùng kết quả) nhưng khác kiến trúc.

### E1.2 — UcdEntry mang 5 chiều riêng biệt (5B), không phải P_weight (2B)

**File:** `crates/ucd/src/lib.rs` (generated code)

```
Mỗi UcdEntry = cp(4B) + group(1B) + shape(1B) + relation(1B) + valence(1B) + arousal(1B) + time(1B) + hash(8B) + 5 formula IDs = ~24 bytes/entry
v2: P_weight = 2 bytes/node
→ 12x overhead so với spec
```

### E1.3 — Fallback defaults khi cp không tìm thấy

**File:** `crates/ucd/src/lib.rs:95-121`

```rust
pub fn shape_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.shape).unwrap_or(0x01)  // Sphere
}
pub fn valence_of(cp: u32) -> u8 {
    lookup(cp).map(|e| e.valence).unwrap_or(0x80) // neutral
}
```

Với 4,184 codepoints bị thiếu (E0.1), tất cả đều trả Sphere/neutral.
→ 44% anchor points L0 = Sphere + neutral → vô nghĩa.

---

## Tầng 2: Molecule struct

### E2.1 — Molecule = 11 bytes RAM thay vì 2B

**File:** `crates/olang/src/mol/molecular.rs:472-512`

```
5 core + fs/fr/fv/fa/ft(5B) + evaluated(1B) = 11 bytes
v2: P_weight = 2B
→ 5.5x overhead
```

### E2.2 — ShapeBase enum = 8 loại thay vì 18

**File:** `crates/olang/src/mol/molecular.rs:22-41`

```
Code:    8 (Sphere, Capsule, Box, Cone, Torus, Union, Intersect, Subtract)
v2:     18 SDF primitives
→ Thiếu 10 SDF types: PLANE, ELLIPSOID, CYLINDER, OCTAHEDRON, PYRAMID,
  HEX_PRISM, PRISM, ROUND_BOX, LINK, REVOLVE, EXTRUDE, CUT_SPHERE, DEATH_STAR
→ Union/Intersect/Subtract là CSG ops, KHÔNG phải shape primitives
```

### E2.3 — Molecule::raw() tạo molecule cho phép hardcode

**File:** `crates/olang/src/mol/molecular.rs:543-556`

```rust
pub fn raw(shape: u8, relation: u8, valence: u8, arousal: u8, time: u8) -> Self { ... }
```

Dù comment nói "Không hardcode", `Molecule::raw()` là public API → bất kỳ code nào cũng gọi được. Chỉ `encode_codepoint()` mới nên tạo Molecule.
→ VI PHẠM QT④: Molecule từ encode_codepoint(cp) — KHÔNG viết tay.

### E2.4 — Formula fields (fs/fr/fv/fa/ft) là runtime metadata, không thuộc v2

v2 không đề cập formula rule IDs trong P_weight. Chúng là metadata của quá trình sinh, không phải identity.
→ Chiếm 5B mà không có trong spec.

---

## Tầng 3: MolecularChain

### E3.1 — Chain = Vec<Molecule> thay vì Vec<u16>

**File:** `crates/olang/src/mol/molecular.rs:1035`

```rust
pub struct MolecularChain(pub Vec<Molecule>);
```

v2: chain link = u16 (2B) = codepoint trỏ vào KnowTree
Code: chain link = Molecule (11B RAM) = full value inline
→ Mỗi link 5.5x overhead → chain 1000 links = 11KB vs 2KB

### E3.2 — chain_hash() hash 5B per molecule

**File:** `crates/olang/src/mol/molecular.rs:1086-1088`

```rust
pub fn chain_hash(&self) -> u64 {
    crate::hash::fnv1a(&self.to_bytes())  // to_bytes() = N×5 bytes
}
```

v2: chain = sequence of u16 → hash trên u16 sequence.
Hiện tại: hash trên 5B/molecule → hash không tương thích khi chuyển 2B.

### E3.3 — similarity() chỉ so shape+relation base

**File:** `crates/olang/src/mol/molecular.rs:1094-1110`

```
v2 (Section 2.3):
  strength(A, B) = Σ match_d × precision_d (5 chiều)
  strength ∈ [0.0, 5.0]

Code:
  similarity() chỉ so shape_base + relation_base (2/5 chiều)
  similarity_full() = 0.3×shape + 0.2×relation + 0.5×emotion
  → Weight giữa các chiều KHÁC v2 (v2 = đều nhau, code = S heavy)
```

---

## Tầng 4: LCA compose

### E4.1 — Valence dùng weighted avg → VI PHẠM QT cốt lõi

**File:** `crates/olang/src/mol/lca.rs:166`

```rust
let valence = mode_or_wavg(&valences, total_weight);
```

v2: `Cⱽ = amplify(Aⱽ, Bⱽ, w_AB)` — TUYỆT ĐỐI KHÔNG trung bình.
Code: weighted average khi không có mode.
→ "buồn" + "mất việc" → trung bình thay vì amplify → SAI CẢM XÚC.

### E4.2 — Arousal dùng weighted avg thay vì max()

**File:** `crates/olang/src/mol/lca.rs:167`

```rust
let arousal = mode_or_wavg(&arousals, total_weight);
```

v2: `Cᴬ = max(Aᴬ, Bᴬ)` — cường độ lấy cao hơn.
Code: weighted average.

### E4.3 — Shape dùng mode_or_wavg thay vì Union()

**File:** `crates/olang/src/mol/lca.rs:164`

```rust
let shape_byte = mode_or_wavg_base(&shapes, total_weight, 8);
```

v2: `Cˢ = Union(Aˢ, Bˢ)` — hình dạng hợp nhất.
Code: mode or weighted average of base categories.

### E4.4 — Relation tính mode thay vì fixed Compose

**File:** `crates/olang/src/mol/lca.rs:165`

```rust
let relation_byte = mode_or_wavg_base(&relations, total_weight, 8);
```

v2: `Cᴿ = Compose` — luôn là Compose (fixed).
Code: mode or weighted average → có thể ra bất kỳ relation nào.

### E4.5 — Time dùng mode thay vì dominant()

**File:** `crates/olang/src/mol/lca.rs:168`

```rust
let time_byte = mode_or_wavg_base(&times, total_weight, 5);
```

v2: `Cᵀ = dominant(Aᵀ, Bᵀ)` — thời gian lấy chủ đạo.
Code: mode or weighted average.

---

## Tầng 5: KnowTree

### E5.1 — KnowTree = TieredStore (hash-based), không phải array 65,536×2B

**File:** `crates/olang/src/storage/knowtree.rs:39-48`

```rust
pub struct KnowTree {
    store: TieredStore,    // hash-based, NOT array
    ...
}
```

v2: `KnowTree[codepoint] = P_weight` — O(1) array, 128KB fixed.
Code: hash-based, dynamic, ~11-15B/node.

### E5.2 — L0 seed = 35 nodes thay vì 8,846

**File:** `crates/olang/src/storage/knowtree.rs:11`

```
//! L0: UCD base (35 seeded nodes) — always in RAM
```

v2: L0 = 8,846 anchor points (toàn bộ 59 blocks).
Code: chỉ 35 seeded nodes.
→ KnowTree thiếu 9,549 L0 anchors.

### E5.3 — SlimKnowTree vẫn hash-based (10-15B/node)

**File:** `crates/olang/src/storage/knowtree.rs:316-329`

```rust
pub struct SlimKnowTree {
    pages: Vec<(u8, SlimPage)>,
    hash_index: BTreeMap<u64, (u8, usize)>,  // hash-based
    ...
}
```

"Slim" nhưng vẫn ~10-15B/node. v2 = 2B/node.

---

## Tầng 6: Writer/Reader

### E6.1 — NodeRecord serialize 5B/molecule (tagged 1-6B)

**File:** `crates/olang/src/storage/writer.rs:21-25`

```
NodeRecord (v0.05 — tagged):
  [0x01][mol_count: u8][tagged_chain_bytes...][layer: u8][is_qr: u8][ts: 8B]
  Mỗi molecule: [mask: u8][present_values: 0-5B]
```

v2 origin.olang spec:
```
0x01 Node [tagged_chain][layer:1][is_qr:1][ts:8]
```

Format gần khớp nhưng molecule bên trong = 5B thay vì 2B.
Khi Molecule chuyển 2B, format serialize thay đổi hoàn toàn.

---

## Tầng 7: Registry

### E7.1 — Registry index bằng chain_hash (8B) thay vì codepoint (2B)

**File:** `crates/olang/src/storage/registry.rs:8`

```
chain_index: BTreeMap<u64, u64> — hash → file offset
```

v2: KnowTree[codepoint] = O(1). Registry dùng hash 8B → BTreeMap O(log n).
→ Codepoint 2B đã đủ unique, không cần hash 8B cho L0.

### E7.2 — NodeKind::Alphabet = "35 seeded nodes" hardcode

**File:** `crates/olang/src/storage/registry.rs:49`

```rust
/// L0 Unicode alphabet — innate, immutable (35 seeded nodes)
Alphabet = 0,
```

Comment "35 seeded nodes" → phải là 8,846.
Registry chưa hỗ trợ full L0 seeding.

---

## Chuỗi ảnh hưởng (Impact Chain)

```
E0.1 (thiếu 4,184 codepoints)
  → E1.3 (fallback Sphere/neutral cho 44% L0)
    → E5.2 (chỉ 35 L0 seeds thay vì 8,846)
      → Toàn bộ distance comparison thiếu anchors → vô nghĩa

E0.2 (8 shapes thay vì 18)
  → E2.2 (ShapeBase 8 types)
    → E3.3 (similarity chỉ so 2/5 chiều)
      → E4.3 (LCA shape = weighted avg thay vì Union)
        → Kết quả compose sai shape

E0.3 (valence heuristic)
  → E0.5 (collision perturb phá nghĩa)
    → E4.1 (LCA avg thay vì amplify)
      → CẢM XÚC TOÀN BỘ HỆ THỐNG SAI

E2.1 (Molecule 11B thay vì 2B)
  → E3.1 (Chain link 11B thay vì 2B)
    → E5.1 (KnowTree ~15B/node thay vì 2B)
      → MEMORY FOOTPRINT 5-7x SPEC
```

---

## Ưu tiên sửa (theo thứ tự dependency)

```
1. UCD build.rs → đọc json/udc.json thay vì heuristic → đủ 8,846 entries
2. Molecule → thiết kế packing 5D → 2B (hoặc dùng udc_p_table.bin)
3. MolecularChain → Vec<u16> thay vì Vec<Molecule>
4. LCA → implement amplify/Union/max/dominant
5. KnowTree → array 65,536 × 2B
6. Registry → codepoint index thay vì hash
7. Writer/Reader → format mới cho 2B molecules
8. ShapeBase → 18 SDF primitives
```
