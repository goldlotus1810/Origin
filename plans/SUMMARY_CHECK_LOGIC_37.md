# TỔNG HỢP: check-logic tool — 37 checks, 18 FAIL

> **Ngày:** 2026-03-21
> **Tool:** `make check-logic` hoặc `cargo run -p check_logic`
> **Source:** `tools/check_logic/src/{main.rs, checks.rs}`
> **Tham chiếu:** AUDIT_L0_ERRORS.md, AUDIT_OLANG_VS_V2.md, PLAN_PWEIGHT_MIGRATION.md

---

## Kết quả hiện tại

```
PASS: 16  |  WARN: 3  |  FAIL: 18
```

---

## 8 Categories — 37 Checks

### A. Logic (6 bug patterns) — 5 PASS, 1 FAIL

| # | Check | Status | Vấn đề |
|---|-------|--------|--------|
| 1 | BP#1 Compose | ❌ FAIL | `learning.rs:1087` — `(V1+V2)/2.0` simple average |
| 2 | BP#2 Self-correct | ✅ | rollback guard OK |
| 3 | BP#3 Quality weights | ⚠️ | Cần verify thủ công Σwᵢ=1.0 |
| 4 | BP#4 Entropy ε_floor | ✅ | floor + log/ln OK |
| 5 | BP#5 HNSW tie-break | ✅ | 301 tie refs |
| 6 | BP#6 SecurityGate | ✅ | 3/3 layers + BlackCurtain |

### B. Checkpoints (pipeline) — 1 PASS

| # | Check | Status |
|---|-------|--------|
| 7 | CP1-5 Pipeline | ✅ | 5/5 checkpoints |

### C. Invariants (23 rules) — 6 PASS

| # | Check | Status |
|---|-------|--------|
| 8 | QT④ Molecule | ✅ | 0 handwritten |
| 9 | QT⑧⑨⑩ Append-only | ✅ | 0 violations |
| 10 | QT⑮ Agent tiers | ✅ | 0 tier violations |
| 11 | QT⑭ L0→L1 | ✅ | ucd/olang clean |
| 12 | QT⑲-㉓ Skill stateless | ✅ | 0 stateful |
| 13 | Worker→chain | ✅ | 0 raw sends |

### D. Data (UDC json) — 1 PASS

| # | Check | Status |
|---|-------|--------|
| 14 | UDC Data | ✅ | 41,338 entries, P_layout OK |

### E. P_weight (2B vs 5B) — 3 FAIL, 1 WARN

| # | Check | Status | Vấn đề |
|---|-------|--------|--------|
| 15 | P_weight Molecule | ❌ FAIL | struct 5×u8, cần packed u16 |
| 16 | P_weight CompactQR | ❌ FAIL | bit layout `[S:3][R:3][T:3]` sai vs `[S:4][R:4][V:3]` |
| 17 | P_weight UCD | ❌ FAIL | build.rs sinh 5B, không đọc udc_p_table.bin |
| 18 | P_weight KnowTree | ⚠️ | Cannot verify — check manually |

### F. Wiring (logic chains) — 2 PASS, 2 FAIL

| # | Check | Status | Vấn đề |
|---|-------|--------|--------|
| 19 | Dream→AAM | ✅ | Chain complete |
| 20 | Epistemic | ✅ | Wired into response |
| 21 | Unified Affect | ❌ FAIL | `sentence_affect_unified()` 0 callers, pipeline dùng bản cũ (4 chỗ) |
| 22 | Word Selection | ❌ FAIL | `target_affect/select_words` 0 callers từ runtime |

### G. Structural (v2 spec) — 5 FAIL

| # | Check | Status | Vấn đề |
|---|-------|--------|--------|
| 23 | ShapeBase | ❌ FAIL | 6/18 SDF, CSG ops sai chỗ |
| 24 | KnowTree | ❌ FAIL | hash-based, v2 cần array 65,536×2B |
| 25 | Chain | ❌ FAIL | Vec\<Molecule\> 11B/link, cần Vec\<u16\> 2B |
| 26 | LCA Compose | ❌ FAIL | mode_or_wavg ALL dims, v2: mỗi dim khác |
| 27 | UCD Blocks | ❌ FAIL | 7 blocks thiếu, anchors ≠ 9,584 |

### H. Olang Kernel — 2 PASS, 3 FAIL

| # | Check | Status | Vấn đề |
|---|-------|--------|--------|
| 28 | Compile Gap | ❌ FAIL | 14 features parse→SILENTLY DROP (break, struct, enum, trait, impl, ?, ??, f-string, tuple, slice, field/index assign, method call) |
| 29 | Stdlib builtins | ❌ FAIL | 16 builtins MISSING từ VM (__fnv1a, __eval_bytecode, __compile, __file_write_bytes...) |
| 30 | Handbook vs v2 | ✅ | Aligned |
| 31 | PushMol | ❌ FAIL | 5 params, cần packed u16 |
| 32 | Bootstrap | ✅ | 4/4 files |

### I. L0 Cascade (foundation errors) — 4 FAIL, 1 WARN

| # | Check | Status | Vấn đề |
|---|-------|--------|--------|
| 33 | L0 Valence | ❌ FAIL | build.rs dùng name heuristic (FIRE/SKULL), KHÔNG đọc udc.json |
| 34 | L0 Seed Count | ❌ FAIL | 35 seeds, v2 cần 9,584 → 44% L0 = Sphere/neutral |
| 35 | L0 Similarity | ❌ FAIL | weights 0.3/0.2/0.5, v2 = equal 5D |
| 36 | L0 Mol::raw | ❌ FAIL | pub fn + 60 callers → QT④ bypassable |
| 37 | REWRITE Progress | ⚠️ | Rust 13x > Olang, migration 7.3% |

---

## Chuỗi ảnh hưởng (Impact Cascade)

```
Tầng 0: UCD build.rs
  ├─ [33] Heuristic tên thay vì udc.json
  ├─ [27] 29 ranges thay vì 58 blocks → thiếu 4,184 codepoints
  └─ [23] 8 shapes thay vì 18 SDF
      ↓
Tầng 1: UCD API
  └─ [34] 44% L0 fallback Sphere/neutral → VÔ NGHĨA
      ↓
Tầng 2: Molecule
  ├─ [15] 11 bytes thay vì 2 bytes
  ├─ [36] raw() public + 60 callers → bypass QT④
  └─ [31] PushMol = 5 params thay vì u16
      ↓
Tầng 3: Chain
  ├─ [25] Vec<Molecule> 11B/link thay vì Vec<u16> 2B
  └─ [35] similarity() weights sai
      ↓
Tầng 4: LCA
  └─ [26] mode_or_wavg ALL dims thay vì amplify/Union/max/dominant
      ↓
Tầng 5: KnowTree
  └─ [24] hash-based thay vì array 65,536×2B
      ↓
Tầng 6: Olang VM
  ├─ [28] 14 features parse → DROP
  └─ [29] 16 builtins MISSING
      ↓
Tầng 7: Pipeline
  ├─ [1]  compose simple average
  ├─ [21] unified affect không ai gọi
  └─ [22] word selection không ai gọi
```

---

## Thứ tự sửa đề xuất (theo dependency)

```
Phase 1 — Tầng 0 (UCD build.rs) → unblock tầng 1-2
  ① build.rs đọc json/udc.json thay vì heuristic      [33]
  ② Bổ sung 58 blocks → 9,584 entries                  [27]
  ③ 18 SDF primitives (tách CSG ops)                    [23]

Phase 2 — Tầng 2 (Molecule) → unblock tầng 3-5
  ④ Molecule → packed u16 [S:4][R:4][V:3][A:3][T:2]    [15,16,17]
  ⑤ Molecule::raw() → pub(crate)                       [36]
  ⑥ PushMol → 1 param u16                              [31]

Phase 3 — Tầng 3-4 (Chain + LCA) → unblock tầng 5
  ⑦ MolecularChain → Vec<u16>                          [25]
  ⑧ LCA: S=Union, R=Compose, V=amplify, A=max, T=dom  [26]
  ⑨ similarity() → equal 5D weights                    [35]
  ⑩ compose: amplify thay vì /2.0                      [1]

Phase 4 — Tầng 5 (KnowTree) → unblock tầng 6
  ⑪ KnowTree → array 65,536 × 2B                      [24]
  ⑫ L0 seed → 9,584 anchors                            [34]

Phase 5 — Tầng 6 (Olang VM)
  ⑬ Compiler: emit 14 missing features                 [28]
  ⑭ VM: implement 16 missing builtins                  [29]

Phase 6 — Tầng 7 (Wiring)
  ⑮ Wire sentence_affect_unified()                     [21]
  ⑯ Wire word selection pipeline                       [22]
```

---

## Files liên quan

| File | Nội dung |
|------|---------|
| `tools/check_logic/src/main.rs` | Entry point, 37 checks |
| `tools/check_logic/src/checks.rs` | Check implementations (~1900 LOC) |
| `plans/PLAN_PWEIGHT_MIGRATION.md` | P_weight 5B→2B migration plan |
| `plans/AUDIT_OLANG_VS_V2.md` | 9 vấn đề Olang vs v2 spec |
| `plans/AUDIT_L0_ERRORS.md` | 27 lỗi L0, 8 tầng cascade |
| `docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md` | 6 bug patterns + 5 checkpoints |
| `PLAN_REWRITE.md` | 7 giai đoạn Rust → Olang migration |
