# PLAN: V2 Migration Tổng Thể — BIG BANG

> **Status:** DRAFT
> **Ngày:** 2026-03-21
> **Tham chiếu:** AUDIT_TONG_HOP.md (51 issues), PLAN_PWEIGHT_MIGRATION.md
> **Nguyên tắc:** Molecule thay → Chain thay → LCA thay → KnowTree thay → HẾT thay. Không incremental.

---

## Dependency Graph

```
T1 UCD build.rs (58 blocks, 9584 entries, đọc udc.json)
 ├→ T2 ShapeBase 8→18 SDF (tách CSG ops)
 │   └→ T2b SdfPrimitive agents 5→18
 └→ T3 Molecule 5B→2B (packed u16)
     ├→ T4 Chain Vec<Mol>→Vec<u16>
     │   ├→ T7 Writer/Reader (serialize 2B/link)
     │   ├→ T8 Registry (codepoint array thay hash)
     │   └→ T10 Downstream crates (silk/agents/memory/vsdf)
     ├→ T5 LCA (amplify/Union/max/dominant)
     │   └→ T10
     ├→ T6 KnowTree (array 65536×2B)
     │   └→ T8
     ├→ T9 VM + Bytecode (PushMol 2B)
     │   └→ T11 .ol files (stdlib + HomeOS + bootstrap)
     └→ T12 Tests rebuild
```

**Thứ tự thực hiện:**
```
Layer 0: T1 (UCD) + T2 (ShapeBase)     ← song song, không phụ thuộc nhau
Layer 1: T3 (Molecule)                  ← phụ thuộc T1+T2
Layer 2: T4+T5+T6 (Chain+LCA+KnowTree) ← song song, đều phụ thuộc T3
Layer 3: T7+T8+T9 (Storage+Registry+VM) ← phụ thuộc T4/T6
Layer 4: T10+T11 (Downstream+.ol)       ← phụ thuộc T3-T9
Layer 5: T12 (Tests)                    ← cuối cùng
```

---

## Tổng quan 12 Tasks

| Task | Tên | Files | Depends | Ước tính |
|------|-----|-------|---------|----------|
| T1 | UCD build.rs rebuild | 2 files | — | Lớn |
| T2 | ShapeBase 18 SDF | 3 files | — | Nhỏ |
| T3 | Molecule 2B packed | 2 files | T1,T2 | Lớn |
| T4 | Chain Vec<u16> | 2 files | T3 | TB |
| T5 | LCA v2 rules | 1 file | T3 | TB |
| T6 | KnowTree array | 2 files | T3 | TB |
| T7 | Writer/Reader v2 | 2 files | T4 | TB |
| T8 | Registry codepoint | 1 file | T4,T6 | TB |
| T9 | VM PushMol 2B | 3 files | T3 | Nhỏ |
| T10 | Downstream crates | ~10 files | T3-T8 | Lớn |
| T11 | .ol files update | ~15 files | T9 | Lớn |
| T12 | Tests rebuild | ~12 files | T10,T11 | TB |

---

## T1 — UCD build.rs Rebuild

**Depends:** không
**Files:** `crates/ucd/build.rs`, `crates/ucd/src/lib.rs`
**Audit refs:** C5, H6, H7, M6, M8

**Hiện tại (sai):**
- 29 ranges → ~5,400 entries
- Heuristic name matching (~38 rules) cho V/A
- Bug: `contains("PIANO") && contains("PIANO")` = always true
- UcdEntry ~24B/entry
- Không đọc `json/udc.json`

**v2 yêu cầu:**
- 58 blocks → 9,584 entries
- Đọc `json/udc.json` (323K dòng) làm source of truth
- UcdEntry = 2B (u16 packed P_weight)
- `json/udc_p_table.bin` (248KB) = lookup table sẵn

**Việc cần làm:**
1. build.rs đọc `json/udc.json` thay vì heuristic
2. Sinh UCD_TABLE với 9,584 entries, mỗi entry = u16
3. 58 blocks: SDF(13), MATH(21), EMOTICON(17), MUSICAL(7)
4. `lib.rs`: lookup trả u16 thay vì UcdEntry 24B
5. Verify roundtrip với `udc_p_table.bin`

**DoD:**
```
□ 58 blocks, 9,584 entries
□ Đọc udc.json, không heuristic
□ UcdEntry = u16 packed [S:4][R:4][V:3][A:3][T:2]
□ check-logic PASS cho UCD checks
```

---

## T2 — ShapeBase 8→18 SDF Primitives

**Depends:** không
**Files:** `crates/olang/src/mol/molecular.rs:24`, `crates/agents/src/pipeline/encoder.rs:91`
**Audit refs:** H2, L1

**Hiện tại (sai):**
```rust
enum ShapeBase { Sphere=1, Capsule=2, Box=3, Cone=4, Torus=5, Union=6, Intersect=7, Subtract=8 }
// Union/Intersect/Subtract = CSG ops, KHÔNG phải SDF primitives
```

**v2 yêu cầu:**
```
18 SDF: SPHERE, BOX, CAPSULE, PLANE, TORUS, ELLIPSOID, CONE, CYLINDER,
        OCTAHEDRON, PYRAMID, HEX_PRISM, PRISM, ROUND_BOX, LINK,
        REVOLVE, EXTRUDE, CUT_SPHERE, DEATH_STAR
CSG ops tách riêng: Union, Intersect, Subtract (dùng trong LCA, không trong ShapeBase)
```

**Việc cần làm:**
1. ShapeBase enum → 18 variants (0x01-0x12), 4 bits đủ chứa (max 15, cần 18 → 5 bits?)
2. Tách CSG ops ra enum riêng `CsgOp { Union, Intersect, Subtract }`
3. SdfPrimitive trong agents crate → cùng 18 loại hoặc re-export
4. Cập nhật vsdf/sdf.rs mapping

**Lưu ý:** 18 > 15 → 4 bits không đủ. Cần 5 bits cho S → layout thay đổi:
```
Nếu S=5 bits: [S:5][R:4][V:3][A:2][T:2] = 16 bits ← R hoặc A giảm 1 bit
Hoặc: v2 nói 4 bits (0-15) → chỉ 16 SDF? Cần xác nhận spec.
```

**DoD:**
```
□ ShapeBase = 18 variants (hoặc 16 nếu spec confirm 4 bits)
□ CSG ops tách riêng
□ SdfPrimitive agents sync
□ vsdf mapping cập nhật
```

---

## T3 — Molecule 5B→2B Packed u16

**Depends:** T1, T2
**Files:** `crates/olang/src/mol/molecular.rs:473`, `crates/olang/src/mol/encoder.rs:16`
**Audit refs:** C1, M2, M5, L2
**Xem thêm:** PLAN_PWEIGHT_MIGRATION.md

**Hiện tại (sai):**
```rust
pub struct Molecule {
    shape: u8, relation: u8, emotion: EmotionDim, time: u8,  // 5B core
    fs: u8, fr: u8, fv: u8, fa: u8, ft: u8,                 // 5B formula
    evaluated: u8,                                             // 1B
} // = 11B RAM
```

**v2 yêu cầu:**
```
Molecule = u16 packed [S:4][R:4][V:3][A:3][T:2] = 2 bytes
```

**Việc cần làm:**
1. Molecule struct → `pub struct Molecule(pub u16)`
2. Accessor methods: `shape()`, `relation()`, `valence()`, `arousal()`, `time()`
3. Constructor: `Molecule::pack(s, r, v, a, t) -> Self`
4. Xóa formula fields (fs/fr/fv/fa/ft) — v2 không có
5. Xóa CompactQR — redundant khi Molecule đã 2B
6. `Molecule::raw()` → `Molecule::pack()`
7. `encode_codepoint()` trả Molecule(u16) thay vì 5 field

**Rào cản:**
- Precision loss: V từ 256→8 levels, A từ 256→8 levels
- Nếu S cần 5 bits (18 SDF) → layout phải điều chỉnh (xem T2)
- FormulaTable có thể xóa nếu không cần formula fields

**DoD:**
```
□ Molecule = u16 (2 bytes)
□ Pack/unpack roundtrip lossless
□ Khớp udc_p_table.bin
□ encode_codepoint() trả Molecule(u16)
□ CompactQR xóa
□ cargo build --workspace compile
```

---

_(T4-T12 tiếp theo)_
