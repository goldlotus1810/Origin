# PLAN: Build Logic Check Test Suite

> **Mục tiêu:** Tạo bộ test kiểm tra toàn bộ mã nguồn theo 6 bug patterns + 5 checkpoints
> từ `docs/CHECK_TO_PASS_LOGIC_HANDBOOK.md`, đồng thời nạp + validate dữ liệu mới
> (`json/udc_utf32.json`).

---

## Hiện trạng (sau khảo sát sâu)

```
Tests hiện tại:    805 passed, 0 failed
SecurityGate:      ✅ 3-layer (crisis, harmful, manipulation, delete_attempt)
Compose/amplify:   ✅ silk/walk.rs amplify_emotion, blend_composite — KHÔNG simple average
                   ✅ context/curve.rs exponential moving average φ⁻¹ + φ⁻²
                   ⚠️ context/phrase.rs:284 comment nói "weighted average" nhưng code dùng blend
                   ⚠️ context/word_guide.rs:409 average_vad() — cần kiểm tra dùng ở đâu
5 Checkpoints:     ✅ CÓ nhưng implicit (Gate, QT8, fire_count, Variance, Semantic)
LCA:               ✅ 4/4 properties tested (idempotent, commutative, associative, similarity)
Append-only:       ✅ File write FIRST → RAM update SAU
Fibonacci:         ✅ Thresholds + dream schedule + fire_count
Rollback guard:    ⚠️ PARTIAL — fire_count maturity OK, chưa có explicit self_correct rollback
Entropy ε_floor:   ⚠️ PARTIAL — aesthetic floor 0.15×intensity cho Fiction/Music, chưa general
HNSW tie-breaking: ❌ Chưa có
Data mới:          ❌ json/udc_utf32.json chưa được nạp vào build.rs/runtime
```

---

## Plan — 6 Tasks

### Task 1: `tests/logic/test_compose_no_average.rs`
**Regression: đảm bảo amplify KHÔNG BAO GIỜ bị thay bằng average**

```
Test 1.1: amplify cùng dấu âm
  amplify(V=-0.7, V=-0.6, w=0.9) → -0.6725 (< -0.65, KHÔNG = -0.65)

Test 1.2: amplify cùng dấu dương
  amplify(V=+0.9, V=+0.95, w=0.8) → 0.935 (> 0.925)

Test 1.3: RECOMBINE 5 chiều đúng spec
  Cˢ = Union, Cᴿ = Compose, Cⱽ = amplify, Cᴬ = max, Cᵀ = dominant

Test 1.4: blend_composite KHÔNG dùng simple average
  Assert: kết quả ≠ (Va + Vb) / 2

Test 1.5: average_vad() in word_guide — kiểm tra context sử dụng
  Nếu dùng trong compose path → BUG. Nếu chỉ dùng cho statistics → OK.

Targets:
  silk/src/walk.rs — amplify_emotion, blend_composite
  context/src/emotion/curve.rs — exponential moving average
  context/src/language/phrase.rs — aggregate function
  context/src/language/word_guide.rs — average_vad
```

### Task 2: `tests/logic/test_self_correct_rollback.rs`
**Kiểm tra rollback guard cho self-correct**

```
Test 2.1: quality ≥ φ⁻¹ (0.618) → DỪNG ngay
Test 2.2: sửa dim → quality tăng → giữ
Test 2.3: sửa dim → quality giảm → ROLLBACK
Test 2.4: hết dim → DỪNG, giữ backup
Test 2.5: max_iter = 3 → luôn dừng (bounded)
Test 2.6: quality weights Σwᵢ = 1.0

Note: self_correct chưa có explicit function.
  → Tests này define EXPECTED behavior.
  → Implement nếu chưa có, hoặc verify fire_count path thỏa mãn.
```

### Task 3: `tests/logic/test_entropy_floor.rs`
**Entropy ε_floor = 0.01 cho general pipeline**

```
Test 3.1: Σc = 0 → uniform → H = log₂(5) ≈ 2.322
Test 3.2: Σc = 0.001 < ε_floor → uniform (không chia gần 0)
Test 3.3: Σc = 0.5 > ε_floor → tính bình thường
Test 3.4: H luôn ≤ 2.32 trong MỌI trường hợp
Test 3.5: Aesthetic floor (Fiction/Music) vẫn hoạt động

Note: General ε_floor = 0.01 có thể chưa có.
  → Hiện có aesthetic floor = 0.15×intensity cho Fiction/Music.
  → Cần thêm ε_floor cho general entropy computation.
```

### Task 4: `tests/logic/test_security_gate_3layer.rs`
**Regression: SecurityGate 3 layers vẫn hoạt động**

```
Test 4.1: Layer 1 — is_crisis ("tự tử") → Crisis
Test 4.2: Layer 2 — is_harmful → Block
Test 4.3: Layer 3 — is_manipulation → Block
Test 4.4: is_delete_attempt → Block
Test 4.5: Safe input → Allow
Test 4.6: BlackCurtain — insufficient evidence → im lặng
Test 4.7: Pipeline DỪNG khi Crisis (không tiếp T2-T7)
```

### Task 5: `tests/logic/test_checkpoints_pipeline.rs`
**Regression: 5 checkpoints trong pipeline**

```
Test 5.1: CP1 GATE — SecurityGate chạy TRƯỚC mọi thứ
Test 5.2: CP2 QT8 — append-only: file write trước, RAM sau
Test 5.3: CP3 fire_count — promotion cần fire_count ≥ Fib(n)
Test 5.4: CP4 Variance — LCA variance detect concrete/categorical/abstract
Test 5.5: CP5 Semantic — contradiction detection trước response
Test 5.6: Pipeline order: T1→T2→T3→T4→T5→T6→T7 sequential
Test 5.7: T5 Crisis → return immediately (không chạy T6-T7)
```

### Task 6: `tests/data/test_udc_utf32_data.rs`
**Validate dữ liệu mới json/udc_utf32.json**

```
Test 6.1: JSON parse thành công
Test 6.2: Tổng individually packed = 41,338
Test 6.3: P pack/unpack consistency — 0 errors
Test 6.4: Spot check emoji anchors:
  🔥 1F525: R=9(CAUSES), A∈[0.7,0.9] (high arousal)
  😊 1F60A: V∈[0.7,0.9] (high valence)
  💔 1F494: V∈[0.3,0.5] (low valence)
Test 6.5: Mọi P ∈ [0, 65535] (16 bits)
Test 6.6: Key = codepoint hex, KHÔNG có "name" field ở top-level
Test 6.7: aliases.en.name tồn tại cho emoji
Test 6.8: aliases.vi.tts tồn tại cho emoji có CLDR
Test 6.9: Binary table udc_p_table.bin matches JSON
Test 6.10: Group distribution: SDF=1838, MATH=2325, EMOTICON=2542, MUSICAL=958
```

---

## Thứ tự thực hiện

```
Phase A — Data validation (không cần sửa code)
  Task 6: test_udc_utf32_data.rs

Phase B — Regression tests (verify code hiện tại ĐÃ ĐÚNG)
  Task 1: test_compose_no_average.rs
  Task 4: test_security_gate_3layer.rs
  Task 5: test_checkpoints_pipeline.rs

Phase C — Gap tests (test cho phần CHƯA ĐỦ)
  Task 3: test_entropy_floor.rs
  Task 2: test_self_correct_rollback.rs
```

---

## Tiêu chí hoàn thành (DoD)

```
□ 6 test files tồn tại
□ Phase A tests PASS (data validation)
□ Phase B tests PASS (regression — code đã đúng)
□ Phase C tests — define expected behavior, PASS hoặc ghi nhận gaps
□ cargo test --workspace PASS (tests mới không break tests cũ)
□ cargo clippy --workspace 0 warnings
□ Commit + push
```
