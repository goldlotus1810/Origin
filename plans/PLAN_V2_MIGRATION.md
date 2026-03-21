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

## T4 — Chain Vec\<Molecule\>→Vec\<u16\>

**Depends:** T3
**Files:** `crates/olang/src/mol/molecular.rs:1035`, `crates/olang/src/mol/hash.rs`
**Audit refs:** C2, M3

**Hiện tại (sai):**
```rust
pub struct MolecularChain(pub Vec<Molecule>);  // 11B/link
// chain_hash = fnv1a trên [u8;5] per molecule
```

**v2 yêu cầu:**
```
Chain link = u16 = codepoint address vào KnowTree
Chain = Vec<u16>, mỗi link 2B
chain_hash = fnv1a trên 2B per link
```

**Việc cần làm:**
1. `MolecularChain(pub Vec<Molecule>)` → `MolecularChain(pub Vec<u16>)`
2. Mỗi u16 = codepoint, KHÔNG phải inline value
3. `chain_hash()` → hash trên `Vec<u16>` (2B/link thay vì 5B)
4. API: `chain.first()` trả u16, `chain.len()` giữ nguyên
5. Helper: `chain.resolve(kt: &KnowTree) -> Vec<Molecule>` để lấy value khi cần

**DoD:**
```
□ MolecularChain = Vec<u16>
□ chain_hash trên 2B/link
□ resolve() helper cho downstream cần value
□ cargo build compile
```

---

## T5 — LCA v2 Compose Rules

**Depends:** T3
**Files:** `crates/olang/src/mol/lca.rs:78-168`
**Audit refs:** C3, C4, H3, H4, H5

**Hiện tại (sai):**
```rust
// ALL 5 dimensions dùng mode_or_wavg = weighted average
let shape_byte = mode_or_wavg_base(&shapes, total_weight, 8);
let relation_byte = mode_or_wavg_base(&relations, total_weight, 8);
let valence = mode_or_wavg(&valences, total_weight);
let arousal = mode_or_wavg(&arousals, total_weight);
let time_byte = mode_or_wavg_base(&times, total_weight, 5);
```

**v2 yêu cầu:**
```
Cˢ = Union(Aˢ, Bˢ)           → CsgOp::Union, KHÔNG avg
Cᴿ = Compose                  → fixed value, LUÔN = Compose
Cⱽ = amplify(Aⱽ, Bⱽ, w_AB)   → AMPLIFY qua Silk, TUYỆT ĐỐI KHÔNG trung bình
Cᴬ = max(Aᴬ, Bᴬ)             → max(), KHÔNG avg
Cᵀ = dominant(Aᵀ, Bᵀ)        → lấy cái có weight cao hơn
```

**Việc cần làm:**
1. Shape: `union_shape(a, b)` → chọn shape theo CSG Union logic
2. Relation: hardcode = `RelationBase::Compose`
3. Valence: `amplify(va, vb, w)` — KHÔNG trung bình, amplify theo Silk weight
4. Arousal: `std::cmp::max(a_arousal, b_arousal)`
5. Time: `dominant(a_time, b_time, a_weight, b_weight)` → lấy cái nặng hơn
6. Xóa `mode_or_wavg` và `mode_or_wavg_base` (dead code sau khi sửa)

**DoD:**
```
□ 5/5 chiều compose đúng v2
□ KHÔNG còn weighted average cho emotion
□ amplify function implement đúng
□ check-logic BP#1 PASS
```

---

## T6 — KnowTree Array 65536×2B

**Depends:** T3
**Files:** `crates/olang/src/storage/knowtree.rs:39`, `crates/olang/src/storage/compact.rs`
**Audit refs:** H1, H7

**Hiện tại (sai):**
- KnowTree = TieredStore (hash-based), O(log n)
- SlimKnowTree = BTreeMap index, 10-15B/node
- L0 seed = 35 nodes

**v2 yêu cầu:**
```
KnowTree = [u16; 65536]  // 128KB fixed
KnowTree[codepoint] = P_weight (2B Molecule)
O(1) lookup by codepoint index
L0 = 9,584 pre-filled entries
```

**Việc cần làm:**
1. `KnowTree` → `pub struct KnowTree([u16; 65536])`
2. `get(cp: u16) -> u16` = O(1) array index
3. `set(cp: u16, mol: u16)` = O(1) write
4. Bootstrap: fill 9,584 entries từ UCD table (T1)
5. Còn lại 55,952 slots = 0 (chưa learn)
6. Xóa TieredStore, CompactNode, SlimKnowTree (dead code)

**DoD:**
```
□ KnowTree = [u16; 65536] = 128KB
□ O(1) lookup
□ 9,584 L0 entries pre-filled
□ TieredStore/SlimKnowTree xóa
□ check-logic PASS
```

---

## T7 — Writer/Reader v2 Format

**Depends:** T4
**Files:** `crates/olang/src/storage/writer.rs`, `crates/olang/src/storage/reader.rs`
**Audit refs:** C6, M1

**Việc cần làm:**
1. NodeRecord (0x01): serialize chain = `[len:2][u16×N]` thay vì tagged molecules
2. KnowTree record (0x08): serialize `[u16; 65536]` compact
3. Thêm CurveRecord (0x09): `[valence:4][fx_dn:4][ts:8]` — hiện thiếu
4. STM record (0x06): giữ nguyên format (đã khớp spec)
5. Reader: deserialize u16 links thay vì Molecule bytes

**DoD:**
```
□ NodeRecord serialize Vec<u16> chain
□ 0x09 Curve record có
□ Reader roundtrip đúng
```

---

## T8 — Registry Codepoint Array

**Depends:** T4, T6
**Files:** `crates/olang/src/storage/registry.rs:136`
**Audit refs:** M7, H7

**Hiện tại (sai):**
- `BTreeMap<u64, u64>` hash-based index
- NodeKind::Alphabet = 35 seeded nodes

**v2 yêu cầu:**
- Registry index by codepoint (u16), NOT by hash
- 9,584 L0 nodes known at bootstrap

**Việc cần làm:**
1. Registry index: codepoint-based lookup (có thể dùng array hoặc HashMap<u16, _>)
2. Bootstrap seed 9,584 nodes từ UCD table
3. `registry.get(cp: u16)` thay vì `registry.get(hash: u64)`
4. Giữ append-only semantics

**DoD:**
```
□ Registry lookup by codepoint
□ 9,584 L0 nodes seeded
□ Append-only giữ nguyên
```

---

## T9 — VM PushMol 2B

**Depends:** T3
**Files:** `crates/olang/src/exec/ir.rs:115`, `crates/olang/src/exec/vm.rs:774`, `crates/olang/src/exec/bytecode.rs`
**Audit refs:** M2

**Hiện tại (sai):**
```rust
PushMol(u8, u8, u8, u8, u8)  // 5 params
// Bytecode: [0x19][S][R][V][A][T] = 6 bytes
```

**v2 yêu cầu:**
```
PushMol(u16)  // 1 param = packed molecule
// Bytecode: [0x19][lo][hi] = 3 bytes
```

**Việc cần làm:**
1. IR: `PushMol(u8,u8,u8,u8,u8)` → `PushMol(u16)`
2. Bytecode emit: 3 bytes thay vì 6
3. VM dispatch: push `Molecule(u16)` lên stack
4. Compiler: MolLiteral `{S=x R=y V=z A=w T=v}` → pack thành u16

**DoD:**
```
□ PushMol = u16 (3B bytecode)
□ VM dispatch đúng
□ Compiler emit đúng
```

---

## T10 — Downstream Rust Crates

**Depends:** T3, T4, T5, T6, T7, T8
**Audit refs:** L1, L4, L5, L6

Mỗi crate dưới đây cần update API calls theo Molecule(u16) + Chain(Vec<u16>):

### T10a — silk crate (~2 files)
- `graph.rs`: EmotionTag dùng u16 thay vì 5B summary
- `edge.rs`: Hebbian weight trên u16 pairs

### T10b — agents crate (~5 files)
- `encoder.rs`: SdfPrimitive 18 loại (xem T2b), EncodedContent.chain = Vec<u16>
- `learning.rs`: Observation.chain = Vec<u16>, `.first()` trả u16
- `instinct.rs`: Molecule::from_bytes() → Molecule(u16)
- `skill.rs`: ExecContext chains = Vec<u16>
- `domain_skills.rs`: IngestSkill trả Vec<u16>

### T10c — memory crate (~3 files)
- `dream.rs`: chain_hash trên 2B/link, EmotionDim → extract từ u16
- `proposal.rs`: NewNode.chain = Vec<u16>
- `build.rs`: Hypothesis.chain = Vec<u16>

### T10d — vsdf crate (~2 files)
- `ffr.rs`: FfrPoint.to_molecule() trả u16, molecule_similarity trên u16
- `body.rs`: ShapeBase mapping 18 SDF

### T10e — runtime crate (~1 file)
- `origin.rs`: ShapeBase matching 18 variants, classify_chain_type trên Vec<u16>

### T10f — context crate — dead code cleanup
- `infer_and_apply`, `crisis_text_for`, `target_affect`, `select_words`, `affect_components`, `fuse_all` → KHÔNG AI GỌI → xóa hoặc wire lại

**DoD:**
```
□ Tất cả crates compile với Molecule(u16) + Chain(Vec<u16>)
□ context dead code xử lý
□ cargo build --workspace pass
```

---

## T11 — .ol Files Update

**Depends:** T9
**15 files cần sửa:**

### T11a — stdlib core (3 files)
- `mol.ol`: accessors dùng bit ops thay field access, 18 shape constants
- `chain.ol`: chain = list of u16 codepoints, chain_lca dùng v2 rules
- `hash.ol`: distance_5d trên packed u16, hash trên 2B

### T11b — HomeOS pipeline (5 files)
- `learning.ol`: text_to_mol trả u16, pipeline dùng amplify
- `instinct.ol`: instincts operate trên u16 dimensions
- `dream.ol`: STM mol = u16, fire/cluster logic
- `silk_ops.ol`: implicit_strength trên u16, walk_emotion amplify
- `mol_pool.ol`: slot = 2B mol + metadata (không còn 5B+3B=8B)

### T11c — bootstrap compiler (3 files)
- `semantic.ol`: MolLiteral parse → u16
- `codegen.ol`: PushMol emit 3B `[tag][lo][hi]`
- `parser.ol`: expression parsing cho 2B mol literals

### T11d — agents + output (4 files)
- `worker.ol`: sensor_read → u16 mol
- `leo.ol`: express u16 mol literal format
- `gate.ol`, `response.ol`: chain = Vec<u16>

**DoD:**
```
□ Tất cả .ol files dùng u16 molecule
□ mol.ol 18 shape constants
□ chain.ol = u16 list
□ bootstrap compiler emit 3B PushMol
```

---

## T12 — Tests Rebuild

**Depends:** T10, T11
**~12 test files**

**Vấn đề:** 1198 tests hiện PASS nhưng test logic CŨ = false positive.

**Việc cần làm:**
1. Unit tests: assertions verify u16 packed values
2. LCA tests: verify amplify/Union/max/dominant (không avg)
3. Chain tests: verify Vec<u16> operations
4. KnowTree tests: verify [u16; 65536] lookup
5. Integration tests: end-to-end với v2 data model
6. `check-logic` tool: ALL checks PASS

**DoD:**
```
□ cargo test --workspace PASS
□ cargo clippy --workspace 0 warnings
□ make smoke-binary PASS
□ check-logic ALL PASS
□ Không còn false positive tests
```

---

## Checklist Tổng

```
Layer 0 (song song):
  □ T1 UCD build.rs rebuild
  □ T2 ShapeBase 18 SDF

Layer 1:
  □ T3 Molecule 2B packed

Layer 2 (song song):
  □ T4 Chain Vec<u16>
  □ T5 LCA v2 rules
  □ T6 KnowTree array

Layer 3 (song song):
  □ T7 Writer/Reader
  □ T8 Registry
  □ T9 VM PushMol

Layer 4 (song song):
  □ T10a-f Downstream crates
  □ T11a-d .ol files

Layer 5:
  □ T12 Tests rebuild

FINAL:
  □ cargo test + clippy + make smoke-binary
  □ check-logic ALL PASS
```
