# PLAN: P_weight Migration — 5B (code) → 2B (v2 spec)

> Status: DRAFT
> Owner: chưa claim
> Depends: PLAN_UDC_REBUILD (đã xong data layer)
> Blocks: Mọi crate dùng Molecule

---

## Vấn đề

**Data đã đúng v2** — `json/udc_p_table.bin` dùng packed u16 `[S:4][R:4][V:3][A:3][T:2]`.

**Code vẫn sai v1** — `Molecule` struct dùng 5 × u8 riêng lẻ. CompactQR lossy dùng sai bit layout `[S:3][R:3][T:3][V:4][A:3]`.

```
v2 spec:  [S:4][R:4][V:3][A:3][T:2] = 16 bits = u16
Code now: [S:u8][R:u8][V:u8][A:u8][T:u8] = 5 bytes = [u8; 5]
CompactQR:[S:3][R:3][T:3][V:4][A:3] = 16 bits (SAI layout)
```

### So sánh bit allocation

| Dim | v2 spec | Code (Molecule) | Code (CompactQR lossy) |
|-----|---------|-----------------|----------------------|
| S   | 4 bits (0-15) | u8 (0-255) | 3 bits (0-7) |
| R   | 4 bits (0-15) | u8 (0-255) | 3 bits (0-7) |
| V   | 3 bits (0-7)  | u8 (0-255) | 4 bits (0-15) |
| A   | 3 bits (0-7)  | u8 (0-255) | 3 bits (0-7) |
| T   | 2 bits (0-3)  | u8 (0-255) | 3 bits (0-4) |

### Hậu quả

1. **chain_hash** = fnv1a([u8;5]) → khác hoàn toàn nếu dùng packed u16
2. **CompactQR lossy** pack sai thứ tự bit → reconstruct sai
3. **KnowTree** = 65,536 × 5B = 320 KB thay vì 65,536 × 2B = 128 KB (v2)
4. **Hebbian weight** quan hệ giữa 2 node dùng hash 5B → sẽ incompatible
5. **Data pipeline** đã tính đúng 2B nhưng code đọc vào lại expand ra 5B

---

## Migration Plan (3 giai đoạn)

### Phase 1: Fix CompactQR bit layout (nhỏ, an toàn)

**File:** `crates/olang/src/mol/molecular.rs`

Sửa `from_molecule_lossy()` + `to_molecule_lossy()` cho đúng v2 layout:

```rust
// ĐÚNG v2: [S:4][R:4][V:3][A:3][T:2] = 16 bits
fn from_molecule_lossy(mol: &Molecule) -> Self {
    let s = (mol.shape & 0x0F) as u16;
    let r = (mol.relation & 0x0F) as u16;
    let v = ((mol.emotion.valence as u16) * 7 / 255) & 0x07;  // quantize [0,255]→[0,7]
    let a = ((mol.emotion.arousal as u16) * 7 / 255) & 0x07;
    let t = (mol.time & 0x03) as u16;
    let bits = (s << 12) | (r << 8) | (v << 5) | (a << 2) | t;
    ...
}
```

**Test:** roundtrip lossy → verify layout khớp `udc_p_table.bin`.

### Phase 2: Thêm native u16 P_weight vào Molecule (trung bình)

Thêm field `p_packed: u16` vào Molecule — cached packed representation.

```rust
pub struct Molecule {
    // ... existing 5 × u8 fields giữ nguyên (backward compat)
    /// Packed P_weight theo v2: [S:4][R:4][V:3][A:3][T:2]
    pub p_packed: u16,
}
```

- `Molecule::raw()` tự tính `p_packed` từ 5 dims
- `Molecule::from_p(p: u16)` → unpack ra 5 dims
- `chain_hash` dùng `p_packed.to_le_bytes()` thay vì `[u8;5]`
- **Migration**: rehash toàn bộ KnowTree + Silk edges

### Phase 3: Loại bỏ 5 × u8, chỉ giữ u16 (lớn)

Molecule trở thành:
```rust
pub struct Molecule {
    pub p: u16,          // Packed [S:4][R:4][V:3][A:3][T:2]
    pub formula: u32,    // Packed formula IDs (nếu cần)
    pub evaluated: u8,   // Bitmask
}
```

Accessor methods:
```rust
impl Molecule {
    pub fn shape(&self) -> u8 { ((self.p >> 12) & 0x0F) as u8 }
    pub fn relation(&self) -> u8 { ((self.p >> 8) & 0x0F) as u8 }
    pub fn valence(&self) -> u8 { ((self.p >> 5) & 0x07) as u8 }
    pub fn arousal(&self) -> u8 { ((self.p >> 2) & 0x07) as u8 }
    pub fn time(&self) -> u8 { (self.p & 0x03) as u8 }
}
```

**Affected crates (chạm hết):**
- `ucd` — build.rs sinh UcdEntry với u16 thay vì 5×u8
- `olang` — Molecule, encoder, storage, VM
- `silk` — edge hashing, HebbianLink
- `context` — EmotionDim
- `agents` — pipeline, learning
- `memory` — STM, Dream
- `vsdf` — FFRCell.to_molecule()
- `runtime` — process_text path

---

## Rào cản

1. **chain_hash thay đổi** → toàn bộ persisted data (origin.olang files) incompatible
2. **Precision loss** — V từ u8 (256 levels) → 3 bits (8 levels). Cần verify v2 spec chấp nhận
3. **FormulaTable** hiện dùng index → Molecule 5B. Nếu Molecule = 2B thì FormulaTable = redundant?
4. **CompactQR** — nếu Molecule đã là 2B thì CompactQR = Molecule. Xóa CompactQR?

---

## DoD (Definition of Done)

```
□ CompactQR lossy bit layout = [S:4][R:4][V:3][A:3][T:2]
□ check-logic tool PASS cho P_weight checks
□ Molecule struct có p_packed: u16
□ chain_hash dùng p_packed (2B) thay vì [u8;5]
□ KnowTree = 65,536 × 2B = 128 KB
□ udc_p_table.bin roundtrip: pack → unpack → pack = lossless
□ cargo test + clippy + make smoke-binary pass
```

---

## Lịch sử

```
2026-03-21  Tạo plan. Audit phát hiện mâu thuẫn data(2B) vs code(5B).
```
