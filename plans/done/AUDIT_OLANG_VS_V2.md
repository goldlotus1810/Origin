# AUDIT: Olang crate vs v2 Spec (HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md)

> **Ngày:** 2026-03-21
> **Branch:** claude/project-audit-review-2pN6F
> **Mục tiêu:** So sánh code olang hiện tại với spec v2.7, liệt kê SAI LỆCH.

---

## Tóm tắt: 9 vấn đề phát hiện

| # | Vấn đề | Mức độ | File chính |
|---|--------|--------|------------|
| 1 | **Molecule = 5B+, v2 yêu cầu 2B** | CRITICAL | `mol/molecular.rs` |
| 2 | **ShapeBase = 8 primitives, v2 = 18 SDF** | HIGH | `mol/molecular.rs`, `ucd/build.rs` |
| 3 | **KnowTree ≠ array 65,536×2B** | HIGH | `storage/knowtree.rs`, `storage/compact.rs` |
| 4 | **MolecularChain = Vec\<Molecule\>, v2 = Vec\<u16\>** | CRITICAL | `mol/molecular.rs` |
| 5 | **LCA dùng weighted avg, v2 = amplify/Union/max/dominant** | HIGH | `mol/lca.rs` |
| 6 | **Storage writer format khác v2 origin.olang spec** | MEDIUM | `storage/writer.rs` |
| 7 | **UCD build.rs chỉ 4 groups, thiếu blocks** | MEDIUM | `ucd/build.rs` |
| 8 | **VM PushMol = 5 bytes, cần chuyển 2B** | MEDIUM | `exec/ir.rs`, `exec/vm.rs` |
| 9 | **SdfPrimitive (agents) = 5 loại, v2 = 18** | LOW | `agents/pipeline/encoder.rs` |

---

## Chi tiết từng vấn đề

### 1. CRITICAL — Molecule = 5B+ RAM, v2 yêu cầu 2B

**v2 spec (Section 1.3, 1.8):**
```
P_weight: Mol (2 bytes) — trọng số đã tính (S,R,V,A,T)
Node layout: 2 bytes/node — index là vị trí array, implicit
KnowTree: 65,536 × 2B = 128 KB
```

**Code hiện tại (`mol/molecular.rs:472-512`):**
```rust
pub struct Molecule {
    pub shape: u8,              // 1B
    pub relation: u8,           // 1B
    pub emotion: EmotionDim,    // 2B (valence + arousal)
    pub time: u8,               // 1B
    pub fs: u8,                 // 1B (formula metadata)
    pub fr: u8, fv: u8, fa: u8, ft: u8, // 4B
    pub evaluated: u8,          // 1B
}
// Total RAM: ~11 bytes/molecule
```

**Sai lệch:**
- v2: P_weight = 2 bytes gói 5 chiều (S,R,V,A,T)
- Code: 5 bytes cho 5 chiều + 6 bytes metadata = 11 bytes
- Chênh lệch: **5.5x** so với spec

**Hướng sửa:** Cần thiết kế packing scheme 5D → 2 bytes. Ví dụ:
```
Byte 0: [SSSS RRRR] = 4-bit shape (16 loại) + 4-bit relation (16 loại)
Byte 1: [VVVV AATT] = 4-bit valence + 2-bit arousal + 2-bit time
```
Hoặc dùng lookup table: P_weight = index vào bảng pre-computed.

---

### 2. HIGH — ShapeBase = 8 primitives, v2 = 18 SDF

**v2 spec (Section 1.5):**
```
18 SDF Primitives: SPHERE, BOX, CAPSULE, PLANE, TORUS, ELLIPSOID,
CONE, CYLINDER, OCTAHEDRON, PYRAMID, HEX_PRISM, PRISM, ROUND_BOX,
LINK, REVOLVE, EXTRUDE, CUT_SPHERE, DEATH_STAR
```

**Code hiện tại (`mol/molecular.rs:22-41`):**
```rust
pub enum ShapeBase {
    Sphere=1, Capsule=2, Box=3, Cone=4,
    Torus=5, Union=6, Intersect=7, Subtract=8,
}
// Chỉ 8 loại, thiếu 10: PLANE, ELLIPSOID, CYLINDER, OCTAHEDRON,
//   PYRAMID, HEX_PRISM, PRISM, ROUND_BOX, LINK, REVOLVE, EXTRUDE,
//   CUT_SPHERE, DEATH_STAR
// Union/Intersect/Subtract KHÔNG phải SDF primitives — là CSG operations
```

**Sai lệch:**
- Code gộp CSG operations (Union, Intersect, Subtract) vào ShapeBase
- Thiếu 13/18 SDF primitives thực sự
- v2 liệt kê rõ: Union/Intersect/Subtract là **operations**, không phải shapes

**agents/pipeline/encoder.rs cũng sai:**
```rust
pub enum SdfPrimitive { Sphere=0, Box=1, Cylinder=2, Plane=3, Mixed=4 }
// Chỉ 5 loại
```

---

### 3. HIGH — KnowTree khác hoàn toàn v2

**v2 spec (Section 1.8):**
```
KnowTree = array 65,536 phần tử
  Position (u16) = codepoint = INDEX IMPLICIT — không lưu trong node
  Value = P_weight (2B Molecule)
  KnowTree[0x1F525] → P(🔥) = [S=Sphere, R=Causes, V=0xC0, A=0xC0, T=Fast]
  Một nhánh: 65,536 × 2B = 128 KB
```

**Code hiện tại (`storage/knowtree.rs`):**
- `KnowTree` = wrapper quanh `TieredStore` (hash-based)
- Dùng `CompactNode` (hash:8B + mol bytes + metadata)
- Node lookup qua hash, KHÔNG qua array index
- Có `SlimKnowTree` dùng `SlimNode` nhưng vẫn hash-based (10-15B/node)
- L0 chỉ seed 35 nodes (comment dòng 11), v2 = 8,846 nodes

**Sai lệch:**
- v2: O(1) array lookup bằng codepoint index, 128KB fixed
- Code: O(log n) hash lookup, kích thước dynamic, ~11-15B/node
- v2: KnowTree[codepoint] = P_weight trực tiếp
- Code: chain_hash → CompactNode → Molecule → giá trị

---

### 4. CRITICAL — MolecularChain = Vec\<Molecule\>, v2 = Vec\<u16\>

**v2 spec (Section 2.1):**
```
Mỗi link = 1 index = 2 bytes. Đơn vị duy nhất.
Chain link (u16): [0x1F525][0x25CF][0x2208]... = trình tự khái niệm
Chain data: 7.42 tỷ u16 links × 2B = 14.84 GB
```

**Code hiện tại (`mol/molecular.rs:1035`):**
```rust
pub struct MolecularChain(pub Vec<Molecule>);
// Mỗi link = 1 Molecule = 11 bytes RAM, 5 bytes wire
```

**Sai lệch:**
- v2: chain link = u16 (2B) = codepoint trỏ vào KnowTree
- Code: chain link = Molecule (11B RAM) = full 5D value inline
- v2 design: chain mang **address** (codepoint), không mang **value** (P_weight)
- Giống DNA: chuỗi nucleotide = sequence of **references** to molecules
- Code hiện tại: chuỗi = sequence of **embedded** molecules

**Tác động:** 5.5x memory overhead cho mỗi chain. Chain 1000 links = 11KB thay vì 2KB.

---

### 5. HIGH — LCA compose rules sai

**v2 spec (Section 1.7, CHECK_TO_PASS §1):**
```
Cˢ = Union(Aˢ, Bˢ)           hình dạng hợp nhất
Cᴿ = Compose                  quan hệ = tổ hợp
Cⱽ = amplify(Aⱽ, Bⱽ, w_AB)   cảm xúc AMPLIFY qua Silk (KHÔNG trung bình)
Cᴬ = max(Aᴬ, Bᴬ)             cường độ lấy cao hơn
Cᵀ = dominant(Aᵀ, Bᵀ)        thời gian lấy chủ đạo
```

**Code hiện tại (`mol/lca.rs:164-168`):**
```rust
let shape_byte = mode_or_wavg_base(&shapes, total_weight, 8);
let relation_byte = mode_or_wavg_base(&relations, total_weight, 8);
let valence = mode_or_wavg(&valences, total_weight);
let arousal = mode_or_wavg(&arousals, total_weight);
let time_byte = mode_or_wavg_base(&times, total_weight, 5);
```

**Sai lệch:**
| Chiều | v2 | Code | Khớp? |
|-------|-----|------|-------|
| S | Union(A,B) | mode_or_wavg_base | ❌ |
| R | Compose (fixed) | mode_or_wavg_base | ❌ |
| V | amplify(Va,Vb,w) | mode_or_wavg (= weighted avg!) | ❌ CRITICAL |
| A | max(A,B) | mode_or_wavg | ❌ |
| T | dominant(A,B) | mode_or_wavg_base | ❌ |

- **Valence:** v2 nói TUYỆT ĐỐI KHÔNG trung bình. Code dùng `mode_or_wavg` = weighted average khi không có mode. VI PHẠM QUY TẮC CỐT LÕI.
- **Arousal:** v2 = max(). Code = weighted avg.
- **Shape:** v2 = Union(). Code = mode or avg.
- **Relation:** v2 = always Compose. Code = mode or avg.
- **Time:** v2 = dominant(). Code = mode or avg.

---

### 6. MEDIUM — Storage writer format khác v2

**v2 spec (CLAUDE.md File Format):**
```
0x01 Node [tagged_chain][layer:1][is_qr:1][ts:8]
0x06 STM  [hash:8][V:4][A:4][D:4][I:4][fire:4][mat:1][layer:1][ts:8]
0x08 KnowTree [data_len:2][compact:N][ts:8]
0x09 Curve [valence:4][fx_dn:4][ts:8]
```

**Code hiện tại (`storage/writer.rs`):**
- Có: 0x01-0x08 (Node, Edge, Alias, Amend, NodeKind, STM, Hebbian, KnowTree)
- Thiếu: 0x09 Curve record
- STM format khác: code dùng f32 (4B cho V,A,D,I), khớp spec
- KnowTree record: có nhưng format khác biệt nhẹ

**Sai lệch nhẹ:** Thiếu record type 0x09 (Curve).

---

### 7. MEDIUM — UCD build.rs chỉ 4 groups, thiếu blocks

**v2 spec (Section 1.4):**
```
59 blocks tổng cộng:
  SDF: 13 blocks (Arrows, Box Drawing, Block Elements, Geometric Shapes,
       Dingbats, Supp Arrows-A/B, Misc Sym+Arrows, Geom Ext, Supp Arrows-C,
       Ornamental Dingbats, Misc Technical, Braille Patterns)
  MATH: 21 blocks
  EMOTICON: 17 blocks
  MUSICAL: 7 blocks
```

**Code hiện tại (`ucd/build.rs:45-99`):**
```rust
GROUPS = [SDF(10 ranges), MATH(8 ranges), EMOTICON(8 ranges), MUSICAL(3 ranges)]
// Total: 29 ranges vs v2's 59 blocks
```

**Sai lệch:**
- SDF: 10 ranges vs v2's 13 blocks → thiếu Ornamental Dingbats, Misc Technical, Braille
- MATH: 8 ranges vs v2's 21 blocks → thiếu Ancient numerics, Siyaq, Arab math...
- EMOTICON: 8 ranges vs v2's 17 blocks → thiếu Mahjong, Domino, Playing Cards...
- MUSICAL: 3 ranges vs v2's 7 blocks → thiếu Znamenny, Byzantine, Ancient Greek...
- **Kết quả:** ~5400 entries trong UCD_TABLE vs v2's 8,846 expected

---

### 8. MEDIUM — VM PushMol = 5 bytes, cần chuyển 2B

**Code (`exec/ir.rs:115`, `exec/bytecode.rs:197-203`):**
```rust
PushMol(u8, u8, u8, u8, u8)  // 5 bytes: S, R, V, A, T
// Bytecode: [0x19][S][R][V][A][T] = 6 bytes per PushMol
```

**v2:** Nếu Molecule = 2B, PushMol cũng phải = 2B.

---

### 9. LOW — SdfPrimitive trong agents/pipeline chỉ 5 loại

**Code (`agents/pipeline/encoder.rs:91-97`):**
```rust
enum SdfPrimitive { Sphere=0, Box=1, Cylinder=2, Plane=3, Mixed=4 }
```

**v2:** 18 SDF primitives. Crate agents tham chiếu cần cập nhật theo.

---

## Tổng kết

### CRITICAL (cần sửa trước khi tiếp tục phát triển)
1. **Molecule 5B→2B** — thiết kế lại packing scheme
2. **MolecularChain Vec\<Mol\>→Vec\<u16\>** — chain = codepoint references

### HIGH (sửa sớm)
3. **KnowTree** — chuyển sang array 65,536 × 2B
4. **ShapeBase** — 8→18 SDF primitives (tách CSG ops)
5. **LCA compose** — implement amplify/Union/max/dominant thay weighted avg

### MEDIUM (có thể sửa dần)
6. Storage writer — thêm Curve record 0x09
7. UCD build.rs — bổ sung còn thiếu blocks (29→58)
8. VM PushMol — resize khi Molecule resize

### LOW
9. SdfPrimitive trong agents — cập nhật theo

---

## Ghi chú quan trọng

Sửa #1 và #2 (Molecule + Chain) là **phá vỡ toàn bộ API**. Cần:
1. Thiết kế packing 5D→2B trước (có thể tham khảo `json/udc_p_table.bin` — 248KB)
2. Migration path cho existing data
3. Cập nhật toàn bộ crate phụ thuộc: silk, context, agents, memory, runtime, wasm

Đề xuất: Tạo PLAN riêng cho migration Molecule 5B→2B, ưu tiên cao nhất.
