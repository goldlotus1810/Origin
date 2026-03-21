# AUDIT TỔNG HỢP — Olang + L0 + PLAN_REWRITE vs v2.7

> **Ngày:** 2026-03-21
> **Trạng thái:** Đợi team kiểm toán xong → lên Plan fix 1 lần
> **Tham chiếu:** `AUDIT_OLANG_VS_V2.md` (9 vấn đề) + `AUDIT_L0_ERRORS.md` (27 lỗi L0)

---

## I. BỨC TRANH TOÀN CẢNH

```
v2.7 thay đổi CẤU TRÚC DỮ LIỆU GỐC:
  P_weight: 5B → 2B
  Chain link: Molecule (11B RAM) → u16 (2B)
  KnowTree: hash-based → array 65,536 × 2B = 128KB
  SDF: 8 types → 18 primitives
  LCA: weighted avg → amplify/Union/max/dominant
  L0 anchors: 5,400 → 9,584

PLAN_REWRITE Phase 0-11 ALL "DONE" — nhưng xây trên cấu trúc CŨ.
→ Toàn bộ output cần REBUILD theo v2.
```

---

## II. THỐNG KÊ ẢNH HƯỞNG

| Thành phần | Files | LOC ước tính | Ảnh hưởng |
|-----------|-------|-------------|-----------|
| Rust olang crate | 41 files | ~34,000 LOC | Molecule/Chain/LCA/KnowTree/Registry/Writer/Reader |
| Rust ucd crate | build.rs + lib.rs | ~1,200 LOC | Bảng L0 thiếu 4,184 cp, 8/18 SDF, heuristic V/A |
| Rust agents crate | encoder.rs + pipeline | ~700 LOC | SdfPrimitive 5 types, ContentEncoder dùng Mol 5B |
| Rust silk crate | hebbian.rs | ~500 LOC | Dùng Molecule 5B cho EmotionTag |
| Rust context crate | emotion/curve/intent | ~2,000 LOC | Dùng Molecule 5B, weighted avg |
| Rust memory crate | stm/dream | ~1,500 LOC | Dùng chain_hash 8B thay vì u16 index |
| Olang stdlib (.ol) | ~22 files | ~2,500 LOC | mol.ol/chain.ol/hash.ol dùng 5D riêng |
| Olang HomeOS (.ol) | ~15 files | ~1,900 LOC | emotion.ol/dream.ol/silk_ops.ol dùng 5D riêng |
| Olang agent (.ol) | ~5 files | ~600 LOC | gate.ol/leo.ol/response.ol dùng Mol cũ |
| VM ASM | 3 files | ~2,400 LOC | PushMol 6B, LCA 5D, dispatch table |
| Builder | 3 files | ~700 LOC | Pack 5B molecules |
| Integration tests | 12 files | ~2,000 LOC | Test theo logic cũ, pass = false positive |

**Tổng:** ~100+ files, ~47,000+ LOC bị ảnh hưởng

---

## III. DANH SÁCH LỖI (tổng hợp từ 2 audit)

### A. CRITICAL — Sai cấu trúc nền tảng (6 lỗi)

| # | Lỗi | v2 yêu cầu | Code hiện tại | Impact |
|---|-----|------------|---------------|--------|
| C1 | Molecule kích thước | 2B packed | 5B core + 6B metadata = 11B | 5.5x memory, format khác |
| C2 | Chain link type | u16 (2B) = codepoint address | Molecule (11B) = inline value | Mang value thay vì address |
| C3 | LCA Valence | amplify(Va,Vb,w) — KHÔNG trung bình | mode_or_wavg = weighted avg | CẢM XÚC SAI — vi phạm QT cốt lõi |
| C4 | LCA Arousal | max(A,B) | mode_or_wavg | Cường độ giảm thay vì giữ cao |
| C5 | L0 coverage | 9,584 anchors (58 blocks) | ~5,400 (29 ranges) | 44% anchors thiếu → distance vô nghĩa |
| C6 | Phase 0-11 nền tảng | Xây trên v2 data model | Xây trên cấu trúc cũ | TOÀN BỘ output cần rebuild |

### B. HIGH — Sai thiết kế (7 lỗi)

| # | Lỗi | v2 | Code | Impact |
|---|-----|-----|------|--------|
| H1 | KnowTree kiến trúc | array 65,536×2B O(1) 128KB | TieredStore hash-based O(log n) | Kiến trúc khác hoàn toàn |
| H2 | ShapeBase | 18 SDF primitives | 8 types (gộp CSG ops) | Thiếu 13 SDF, sai 3 CSG |
| H3 | LCA Shape | Union(A,B) | mode_or_wavg_base | Hình dạng compose sai |
| H4 | LCA Relation | fixed = Compose | mode_or_wavg_base | Có thể ra bất kỳ relation |
| H5 | LCA Time | dominant(A,B) | mode_or_wavg_base | Thời gian compose sai |
| H6 | Valence source | udc.json (323K dòng) | Heuristic tên (~38 rules) | False positives ("CROSS" match nhiều thứ) |
| H7 | L0 seed count | 9,584 | 35 nodes | Registry chỉ biết 35/9,584 nodes |

### C. MEDIUM — Sai format/API (8 lỗi)

| # | Lỗi | Chi tiết |
|---|-----|---------|
| M1 | Storage thiếu record | Thiếu 0x09 Curve record |
| M2 | VM PushMol | 6B bytecode (1 tag + 5 data), cần 2B |
| M3 | chain_hash | Hash trên 5B, cần hash trên 2B khi chuyển |
| M4 | similarity() | So 2/5 chiều, v2 = 5 chiều đều nhau |
| M5 | Molecule::raw() | Public API cho phép hardcode — vi phạm QT④ |
| M6 | UcdEntry size | ~24B/entry, v2 = 2B/node |
| M7 | Registry index | BTreeMap<u64,u64> hash-based, v2 = codepoint array |
| M8 | Arousal formula bug | `contains("PIANO") && contains("PIANO")` = always true |

### D. LOW — Cần cập nhật theo (6 lỗi)

| # | Lỗi | Chi tiết |
|---|-----|---------|
| L1 | SdfPrimitive (agents) | 5 types thay vì 18 |
| L2 | Formula fields | fs/fr/fv/fa/ft không có trong v2 spec |
| L3 | Collision resolution | Perturb V/A phá ngữ nghĩa L0 anchors |
| L4 | Context crate dead code | infer_and_apply, crisis_text_for, target_affect... không gọi |
| L5 | Dream pipeline đứt | Dream sinh proposals nhưng không submit AAM |
| L6 | QR promotion đứt | QR không promote → KnowTree không grow |

---

## IV. CHUỖI ẢNH HƯỞNG

```
┌─ UCD build.rs (29/58 blocks, heuristic, 8/18 SDF)
│    ↓
├─ UCD lib.rs (lookup fallback → 44% = Sphere/neutral)
│    ↓
├─ Molecule (11B thay vì 2B)
│    ↓
├─ Chain (Vec<Molecule> thay vì Vec<u16>)
│    ↓
├─ LCA (5/5 chiều compose SAI)
│    ↓
├─ KnowTree (hash-based, 35 seeds thay vì 9,584)
│    ↓
├─ Registry (hash index thay vì codepoint array)
│    ↓
├─ Writer/Reader (serialize 5B/mol)
│    ↓
├─ VM (PushMol 6B, LCA 5D)
│    ↓
├─ .ol stdlib (mol.ol/chain.ol 5D logic)
│    ↓
├─ .ol HomeOS (emotion/dream/silk 5D logic)
│    ↓
└─ Builder (pack 5B)
    ↓
  TOÀN BỘ Phase 0-11 output = nền cũ
```

---

## V. GHI CHÚ CHO PLAN FIX

```
KHÔNG fix từng cái một — cấu trúc thay đổi từ gốc.
Đợi team kiểm toán xong → lên 1 PLAN duy nhất:

  1. Thiết kế packing 5D → 2B (tham khảo json/udc_p_table.bin 248KB)
  2. UCD build.rs → đọc udc.json, 58 blocks, 18 SDF, 9,584 entries
  3. Molecule → 2B struct mới
  4. Chain → Vec<u16>
  5. KnowTree → [u16; 65536]
  6. LCA → amplify/Union/max/dominant
  7. Tất cả downstream: Writer/Reader/Registry/VM/Builder/.ol files

Migration = BIG BANG, không incremental.
Lý do: thay Molecule → thay Chain → thay LCA → thay KnowTree → thay hết.
Không có điểm dừng giữa chừng mà code vẫn compile.
```

---

## VI. PHẦN ĐỢI TỪ TEAM KIỂM TOÁN KHÁC

```
Branch: claude/read-homeOS-biology-jg1ji (đang review)
Phát hiện sơ bộ từ screenshot:
  - Context crate: 6 functions defined nhưng KHÔNG AI GỌI
    (infer_and_apply, crisis_text_for, target_affect,
     select_words, affect_components, fuse_all)
  - Dream pipeline: sinh proposals nhưng KHÔNG submit AAM
  - AAM: KHÔNG review, KHÔNG approve
  - QR: KHÔNG promote → KnowTree KHÔNG grow
  - Toàn bộ cơ chế học dài hạn = dead code

→ Đợi report đầy đủ từ branch đó rồi tổng hợp chung vào PLAN fix.

UPDATE 2026-03-21: main merge thêm:
  - plans/PLAN_PWEIGHT_MIGRATION.md — Plan 3 phase cho P_weight 5B→2B
  - tools/check_logic/ — Tool kiểm tra logic tự động
  → PLAN_PWEIGHT_MIGRATION chỉ cover P_weight packing.
    CHƯA cover: LCA rules, KnowTree array, Chain Vec<u16>, 18 SDF, 9584 L0.
    Cần tổng hợp với audit này để ra PLAN fix toàn diện.
```

---

## VII. AUDIT CÁC PLAN FILES (47 files trong plans/)

> Kiểm tra toàn bộ PLAN Phase 0-12 xem có thống nhất với v2 không.
> **Kết luận: KHÔNG — tất cả plans xây trên cấu trúc cũ (Molecule 5B, LCA avg, 5400 L0).**

### A. Phase 0 Plans — Nền tảng sai từ đầu (10 vấn đề)

| # | Plan file | Vấn đề | v2 yêu cầu |
|---|-----------|--------|------------|
| P1 | PLAN_0_1 (Olang Spec) | MolLiteral = 5 Num `{1,6,60,180,4}` | Molecule = 2B packed, không phải 5 số riêng lẻ |
| P2 | PLAN_0_1 | PushMol = 5 params `PushMol(s,r,v,a,t)` | PushMol cần 2B, không phải 5 args |
| P3 | PLAN_0_2 (Bytecode) | PushMol bytecode = `[0x19][S][R][V][A][T]` = 6B | Cần 3B: [opcode][byte0][byte1] |
| P4 | PLAN_0_2 | Dispatch table dùng 5 field Molecule | Dispatch cần match 2B packed format |
| P5 | PLAN_0_3 (Stdlib) | mol.ol dùng `mol_shape()`, `mol_valence()` riêng | 2B packed → extract qua bit ops, không phải field access |
| P6 | PLAN_0_3 | chain.ol dùng `chain_push(mol)` với Molecule object | Chain = Vec<u16>, push codepoint u16 |
| P7 | PLAN_0_3 | hash.ol dùng `fnv1a` trên 5B mol | Hash trên 2B, không phải 5B |
| P8 | PLAN_0_4 (FFI) | FFI MolecularChain args = Vec<Molecule> | FFI cần Vec<u16> |
| P9 | PLAN_0_5 (Builder) | Builder pack 5B per molecule | Builder pack 2B per molecule |
| P10 | PLAN_0_6 (Tests) | Test assertions dựa trên 5D field access | Test cần verify 2B packed values |

### B. Phase 1-3 Plans — Logic sai (6 vấn đề)

| # | Plan file | Vấn đề | v2 yêu cầu |
|---|-----------|--------|------------|
| P11 | PLAN_1_4 (VM LCA) | LCA = `(A+B)/2` weighted average | amplify/Union/max/dominant — KHÔNG trung bình |
| P12 | PLAN_1_4 | PushMol = 5B bytecode inline | 2B bytecode inline |
| P13 | PLAN_2_1 (Emotion) | Emotion pipeline dùng Molecule 5B fields | Emotion extract từ 2B packed |
| P14 | PLAN_2_2 (Dream) | dream_score dùng averages | v2: amplify, không average |
| P15 | PLAN_3_1 (STM) | STM chain_hash = hash(5B×n) | hash(2B×n) |
| P16 | PLAN_3_2 (Hebbian) | Hebbian weight trên Molecule pairs | Weight trên u16 codepoint pairs |

### C. Phase 4-12 Plans — Hardcode sai (8 vấn đề)

| # | Plan file | Vấn đề | v2 yêu cầu |
|---|-----------|--------|------------|
| P17 | PLAN_5_3 (Origin Spec) | Hardcode: "Molecule = 5 bytes" | Molecule = 2 bytes |
| P18 | SPEC_ORIGIN | "5,400 công thức" L0 | 9,584 L0 anchors |
| P19 | PLAN_UDC_REBUILD | Mâu thuẫn nội bộ: nói 9,584 nhưng design vẫn 5B | Phải 9,584 VÀ 2B |
| P20 | PLAN_UDC_REBUILD | UCD schema vẫn dùng heuristic name matching | Đọc udc.json trực tiếp |
| P21 | PLAN_7_1 (ISL) | ISL payload mang Molecule 5B | ISL payload mang u16 codepoint hoặc 2B packed |
| P22 | PLAN_8_1 (Security) | SecurityGate check trên Molecule fields | Check trên 2B packed |
| P23 | PLAN_10_1 (WASM) | WASM serialize Molecule 5B | Serialize 2B |
| P24 | PLAN_12 (Integration) | Integration tests verify 5B logic | Verify 2B logic |

### D. Tổng kết Plans

```
47 plan files kiểm tra → 24 vấn đề phát hiện
  - Phase 0: 10 vấn đề (nền tảng bytecode/stdlib/builder)
  - Phase 1-3: 6 vấn đề (LCA/emotion/dream/STM)
  - Phase 4-12: 8 vấn đề (spec hardcode/ISL/WASM/tests)

KẾT LUẬN:
  TOÀN BỘ 47 plan files xây trên giả định Molecule = 5B.
  Khi migration sang 2B, TẤT CẢ plans cần REWRITE.
  Đây là lý do phải migration BIG BANG — không thể fix từng plan.
```
