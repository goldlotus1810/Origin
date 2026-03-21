# PLAN: Build Logic Check Test Suite

> **Mục tiêu:** Tạo bộ test kiểm tra toàn bộ mã nguồn theo 6 bug patterns + 5 checkpoints
> từ `docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md`, đồng thời nạp + validate dữ liệu mới
> (`json/udc_utf32.json`).

---

## Hiện trạng

```
Tests hiện tại:    805 passed, 0 failed (cargo test --workspace)
Checkpoints:       0 (không có file nào chứa "checkpoint")
SecurityGate:      có gate.rs nhưng chưa rõ 3-layer
Compose/amplify:   silk/walk.rs có amplify_emotion ✅
                   context/phrase.rs vẫn dùng "weighted average" ❌
                   context/word_guide.rs có average_vad() ❌
Rollback guard:    không tìm thấy quality + rollback logic
Entropy ε_floor:   không tìm thấy (chỉ có epsilon cho LCA)
HNSW tie-breaking: chưa có
Data mới:          json/udc_utf32.json chưa được nạp vào build.rs
```

---

## Plan — 8 Tasks

### Task 1: `tests/logic/test_compose_amplify.rs`
**Kiểm tra Bug Pattern #1: compose KHÔNG trung bình**

```
Test 1.1: amplify cùng dấu âm
  compose("buồn" V=-0.7, "mất việc" V=-0.6, w=0.9)
  → Cⱽ = -0.6725 (PHẢI < -0.65, KHÔNG = -0.65)

Test 1.2: amplify cùng dấu dương
  compose("yêu" V=+0.9, "mãnh liệt" V=+0.95, w=0.8)
  → Cⱽ = 0.935 (PHẢI > 0.925)

Test 1.3: RECOMBINE đầy đủ 5 chiều
  Cˢ = Union(Aˢ, Bˢ)
  Cᴿ = Compose
  Cⱽ = amplify(Aⱽ, Bⱽ, w)  ← KHÔNG trung bình
  Cᴬ = max(Aᴬ, Bᴬ)
  Cᵀ = dominant(Aᵀ, Bᵀ)

Test 1.4: NEGATIVE — detect trung bình
  Scan code: grep cho (Va + Vb) / 2 trong compose paths
  → PHẢI là 0 occurrences trong pipeline code

Targets:
  ✗ context/src/language/phrase.rs:284 — "weighted average" → sửa thành amplify
  ✗ context/src/language/word_guide.rs:409 — average_vad() → sửa
  ✓ silk/src/walk.rs — amplify_emotion OK
```

### Task 2: `tests/logic/test_self_correct_rollback.rs`
**Kiểm tra Bug Pattern #2: self-correct PHẢI có rollback**

```
Test 2.1: quality ≥ φ⁻¹ → DỪNG ngay
  quality = 0.7 → không sửa thêm

Test 2.2: sửa 1 dim → quality tăng → giữ
  quality_before = 0.4, sửa entropy → quality_after = 0.5 → OK

Test 2.3: sửa 1 dim → quality giảm → ROLLBACK
  quality_before = 0.4, sửa entropy → quality_after = 0.3
  → PHẢI rollback về 0.4

Test 2.4: hết dim → DỪNG, giữ backup
  Tất cả 4 dim sửa đều làm quality giảm
  → giữ nguyên backup, confidence thấp

Test 2.5: max_iter = 3 → luôn dừng
  quality vẫn < 0.618 sau 3 iter → trả confidence thấp

Targets:
  ✗ Chưa có self_correct function → cần implement
```

### Task 3: `tests/logic/test_quality_weights.rs`
**Kiểm tra Bug Pattern #3: quality weights Σ = 1.0**

```
Test 3.1: Default weights sum = 1.0
  w1=0.30, w2=0.30, w3=0.20, w4=0.20 → Σ = 1.0

Test 3.2: quality formula đúng
  quality = w1×valid + w2×(1-H/2.32) + w3×consistency + w4×silk/5.0
  Với valid=1.0, H=0, consistency=1.0, silk=5.0:
  → quality = 0.30 + 0.30 + 0.20 + 0.20 = 1.0 (max)

Test 3.3: quality = 0 khi tất cả dim = 0
  valid=0, H=2.32, consistency=0, silk=0
  → quality = 0

Test 3.4: INVARIANT — Σwᵢ = 1.0 sau khi tune
  Thay đổi weights → assert sum vẫn = 1.0
```

### Task 4: `tests/logic/test_entropy_floor.rs`
**Kiểm tra Bug Pattern #4: entropy ε_floor = 0.01**

```
Test 4.1: Σc = 0 → uniform → H = 2.32
  5 chiều, tất cả c_d = 0 → p_d = 1/5 → H = log₂(5) ≈ 2.322

Test 4.2: Σc = 0.001 < ε_floor → uniform
  → p_d = 1/5 (không chia cho 0.001)

Test 4.3: Σc = 0.5 > ε_floor → tính bình thường
  c = [0.5, 0, 0, 0, 0] → p = [1, 0, 0, 0, 0] → H = 0

Test 4.4: NEGATIVE — Σc = 0.0001 KHÔNG được gây H bùng nổ
  H phải ≤ 2.32 trong MỌI trường hợp
```

### Task 5: `tests/logic/test_hnsw_tiebreak.rs`
**Kiểm tra Bug Pattern #5: HNSW insert deterministic**

```
Test 5.1: 2 blocks cùng distance → chọn R gần nhất
  P_new gần block_A và block_B bằng nhau
  block_A.R = Member, block_B.R = Causes
  P_new.R = Member → chọn block_A

Test 5.2: R cũng bằng → chọn index thấp
  Cả R cùng bằng → block có index thấp thắng

Test 5.3: deterministic — 100 lần insert = cùng kết quả
```

### Task 6: `tests/logic/test_security_gate.rs`
**Kiểm tra Bug Pattern #6: SecurityGate ≥ 3 layers**

```
Test 6.1: Layer 1 exact match
  "tự tử" → Crisis

Test 6.2: Layer 2 normalized match
  "t.ự t.ử" → normalize → "tự tử" → Crisis

Test 6.3: Layer 3 semantic check
  "không muốn thức dậy nữa" → encode → V < -0.9, A > 0.8 → Crisis

Test 6.4: ALL layers safe → Allow
  "hôm nay trời đẹp" → Allow

Test 6.5: Pipeline DỪNG khi Crisis
  Gate blocks → pipeline KHÔNG chạy tiếp

Targets:
  ? agents/src/pipeline/gate.rs — kiểm tra 3-layer có đủ chưa
```

### Task 7: `tests/logic/test_checkpoints.rs`
**Kiểm tra 5 Checkpoints bắt buộc**

```
Test 7.1: CP1 GATE — Crisis → pipeline dừng
Test 7.2: CP2 ENCODE — |entities| = 0 → dừng
Test 7.3: CP2 ENCODE — consistency < 0.75 → dừng
Test 7.4: CP3 INFER — no valid branch → im lặng
Test 7.5: CP3 INFER — rollback quality_final ≥ quality_backup
Test 7.6: CP4 PROMOTE — weight < φ⁻¹ → giữ STM
Test 7.7: CP4 PROMOTE — fire_count < Fib(n) → giữ STM
Test 7.8: CP4 PROMOTE — H > 1.0 → giữ STM
Test 7.9: CP5 RESPONSE — SecurityGate check output
Test 7.10: CP5 RESPONSE — confidence < 0.40 → im lặng
```

### Task 8: `tests/data/test_udc_utf32.rs`
**Kiểm tra dữ liệu mới json/udc_utf32.json**

```
Test 8.1: JSON parse thành công
Test 8.2: Tổng codepoints = 312,430
Test 8.3: Individually packed = 41,338
Test 8.4: P pack/unpack consistency (0 errors)
Test 8.5: Spot check emoji anchors:
  🔥 1F525: S=0, R=9, V∈[0.3,0.5], A∈[0.7,0.9]
  😊 1F60A: V∈[0.7,0.9], A∈[0.2,0.5]
  💔 1F494: V∈[0.3,0.5]
Test 8.6: Mọi P ∈ [0, 65535] (16 bits)
Test 8.7: aliases.en.name tồn tại cho emoji
Test 8.8: aliases.vi.tts tồn tại cho emoji có CLDR
Test 8.9: Không có "name" field ở top-level (name = codepoint key)
Test 8.10: Binary table udc_p_table.bin readable + matches JSON
```

---

## Thứ tự thực hiện

```
Phase A — Test Data (không cần sửa code)
  Task 8: test_udc_utf32.rs              ← validate data mới

Phase B — Test Logic (kiểm tra code hiện tại)
  Task 1: test_compose_amplify.rs        ← phát hiện bug average
  Task 4: test_entropy_floor.rs          ← phát hiện thiếu ε_floor
  Task 5: test_hnsw_tiebreak.rs          ← phát hiện non-deterministic
  Task 6: test_security_gate.rs          ← kiểm tra 3 layers

Phase C — Test Pipeline (cần implement trước)
  Task 2: test_self_correct_rollback.rs  ← cần self_correct fn
  Task 3: test_quality_weights.rs        ← cần quality fn
  Task 7: test_checkpoints.rs            ← cần checkpoint framework
```

---

## Tiêu chí hoàn thành (DoD)

```
□ Tất cả 8 test files tồn tại
□ Phase A tests PASS (data validation)
□ Phase B tests — phát hiện đúng bugs hiện tại (FAIL expected → ghi nhận)
□ Phase C tests — compile, logic đúng (có thể FAIL nếu chưa implement)
□ cargo test --workspace PASS (tests mới không break tests cũ)
□ cargo clippy --workspace 0 warnings
□ Commit + push
```
