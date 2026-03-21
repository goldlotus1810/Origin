# CHECK TO PASS — Logic Handbook

> **Mọi AI contributor PHẢI đọc file này trước khi viết code.**
> **Tham chiếu từ CLAUDE.md — tự động được load mỗi session.**

---

## Tại sao cần file này?

```
HomeOS = sinh linh. Sinh linh có CƠ CHẾ KIỂM TRA.
Tế bào bỏ checkpoint = ung thư.
Pipeline bỏ checkpoint = tri thức sai lan tràn vĩnh viễn trong QR.

File này là BỘ QUY TẮC LOGIC bắt buộc:
  ① Công thức nào ĐÚNG, công thức nào SAI
  ② Checkpoint nào PHẢI pass trước khi tiếp tục
  ③ Anti-patterns nào TUYỆT ĐỐI KHÔNG

Mọi AI session đọc CLAUDE.md → CLAUDE.md trỏ tới file này.
→ Không cần nhắc lại. Không cần user giám sát từng dòng.
```

---

## 1. COMPOSE — Công thức ghép cảm xúc

### SAI (trung bình — vi phạm QT cốt lõi)

```
Cⱽ = (Aⱽ + Bⱽ) / 2          ← TUYỆT ĐỐI KHÔNG

Tại sao sai:
  "buồn" V=-0.7 + "mất việc" V=-0.6 → trung bình = -0.65
  Thực tế: buồn VÌ mất việc → nặng hơn cả hai → phải < -0.65
  Sinh học: cortisol + adrenaline → stress MẠNH HƠN, không trung bình
```

### ĐÚNG (amplify qua Silk weight)

```
Cⱽ = amplify(Aⱽ, Bⱽ, w_AB)

amplify(Va, Vb, w):
  base  = (Va + Vb) / 2                    trung điểm
  boost = |Va − base| × w × 0.5            Silk weight khuếch đại
  Cⱽ    = base + sign(Va + Vb) × boost     đẩy về phía dominant

Ví dụ 1 — cùng dấu âm:
  compose("buồn" V=-0.7, "mất việc" V=-0.6, w=0.9)
  base = -0.65, boost = 0.05 × 0.9 × 0.5 = 0.0225
  Cⱽ = -0.65 - 0.0225 = -0.6725            ← nặng hơn → đúng thực tế

Ví dụ 2 — cùng dấu dương:
  compose("yêu" V=+0.9, "mãnh liệt" V=+0.95, w=0.8)
  base = 0.925, boost = 0.025 × 0.8 × 0.5 = 0.01
  Cⱽ = 0.925 + 0.01 = 0.935                ← mạnh hơn cả hai

Quy tắc đầy đủ cho RECOMBINE:
  Cˢ = Union(Aˢ, Bˢ)          hình dạng hợp nhất
  Cᴿ = Compose                 quan hệ = tổ hợp
  Cⱽ = amplify(Aⱽ, Bⱽ, w_AB)  cảm xúc AMPLIFY qua Silk
  Cᴬ = max(Aᴬ, Bᴬ)            cường độ lấy cao hơn
  Cᵀ = dominant(Aᵀ, Bᵀ)       thời gian lấy chủ đạo
```

---

## 2. SELF-CORRECT — Rollback Guard (bắt buộc)

### SAI (giả định monotonic)

```
"quality(iter+1) > quality(iter) — luôn tốt hơn"     ← SAI

Tại sao sai:
  Sửa 1 dim có thể phá dim khác (side effect).
  VD: sửa entropy → consistency giảm → quality tổng giảm.
  Giả định monotonic = bỏ qua side effect = nguy hiểm.
```

### ĐÚNG (có rollback check)

```
③ Refine (excision repair) — CÓ ROLLBACK:
  quality ≥ φ⁻¹ (0.618)  → DỪNG — đủ tốt
  quality < φ⁻¹           → lưu snapshot:

    P_backup = P_response           ← snapshot TRƯỚC KHI sửa

    Tìm dim yếu nhất → sửa DUY NHẤT dim đó:
      valid thấp       → thêm symbolic constraints
      entropy cao      → thu hẹp nhánh (N -= 1)
      consistency thấp → evolve(P_response, dim_worst, new_val)
      silk thấp        → mở rộng walk depth += 1

    ROLLBACK CHECK (= DNA mismatch repair verification):
      quality_new = critique(P_response_new)
      Nếu quality_new < quality_old:
        → ROLLBACK: P_response = P_backup
        → Thử sửa dim KHÁC (dim thứ 2 yếu nhất)
        → Nếu hết dim để thử → DỪNG, giữ P_backup

  Hội tụ (có guard — KHÔNG giả định monotonic):
    Rollback đảm bảo quality KHÔNG BAO GIỜ giảm qua các iter
    Worst case: quality giữ nguyên (backup)
    Best case: quality tăng
    max_iter = 3 → luôn dừng (bounded)

  Sinh học: DNA repair enzyme kiểm tra SAU KHI sửa.
            Sửa sai → cắt lại → thử lại.
            Không bao giờ "sửa xong đi luôn" mà không verify.
```

---

## 3. QUALITY WEIGHTS — Cơ sở + Tunable

### SAI (magic numbers không giải thích)

```
quality = 0.3 × valid + 0.3 × entropy + 0.2 × consistency + 0.2 × silk
          ↑ Tại sao 0.3? Tại sao không phải 0.4? Ai quyết định?
```

### ĐÚNG (có cơ sở, cho phép tune)

```
quality = w₁ × valid_score
        + w₂ × (1 − entropy_score/2.32)
        + w₃ × consistency
        + w₄ × silk_score/5.0

Default weights:
  w₁ = 0.30  logic đúng — nền tảng
  w₂ = 0.30  entropy thấp — tin cậy
  w₃ = 0.20  nhất quán — mutation hợp lý
  w₄ = 0.20  silk phù hợp — context match

Cơ sở: w₁=w₂ > w₃=w₄ vì:
  valid + entropy = ĐÚNG + CHẮC CHẮN = điều kiện CẦN (60%)
  consistency + silk = HỢP LÝ + PHÙ HỢP = điều kiện ĐỦ (40%)
  Sinh học: DNA replication ưu tiên fidelity (đúng) > speed (nhanh)

Bất biến: Σwᵢ = 1.0 (luôn đúng)
Tuning: weights có thể thay đổi qua A/B testing sau khi có data thật
```

---

## 4. ENTROPY — Floor cho Σc

### SAI (chia cho 0 hoặc gần 0)

```
p_d = c_d / Σc          nếu Σc > 0
p_d = 1/5               nếu Σc = 0

Vấn đề: Σc = 0.0001 → p_d = c_d / 0.0001 → H bùng nổ số học
         Chỉ check = 0, bỏ qua "gần 0" → entropy vô nghĩa
```

### ĐÚNG (ε_floor = 0.01)

```
Σc = max(Σ_{i=1}^{5} c_i, ε_floor)    ε_floor = 0.01

p_d = c_d / Σc                         nếu Σc > ε_floor
p_d = 1/5                              nếu Σc ≤ ε_floor (uniform = H=2.32)

Tại sao ε_floor = 0.01?
  Σc nhỏ nhất có nghĩa = 1 chiều × min weight = 0.01
  Dưới 0.01 = "gần như chưa có data" → uniform → H = 2.32
  Tránh: Σc = 0.0001 → p_d = 100 → H bùng nổ
```

---

## 5. HNSW INSERT — Tie-breaking

### SAI (non-deterministic khi 2+ blocks cùng khoảng cách)

```
insert(P_new):
  search(P_new, k=1) → tìm UDC gần nhất
  → 2 blocks cùng distance → ai thắng? → KHÔNG XÁC ĐỊNH
  → 2 sessions khác nhau = kết quả khác nhau = BUG
```

### ĐÚNG (deterministic tie-breaking)

```
insert(P_new):
  1. search(P_new, k=1) → tìm UDC gần nhất = "parent block"

     TIE-BREAKING (khi 2+ blocks cùng khoảng cách):
       a. Chọn block có Relation gần nhất (R quan trọng nhất)
       b. Nếu vẫn hòa → block có index thấp hơn thắng (deterministic)

     Sinh học: axon guidance — neuron mới theo gradient hóa học,
              nếu 2 gradient bằng nhau → theo mặc định phía anterior

  2. Gắn P_new vào sub-tree của parent block
  3. Nếu sub-tree quá đông (> Fib(n)):
     → LCA gom nhóm → tạo tầng trung gian mới
```

---

## 6. SECURITY GATE — 3-layer Detection

### SAI (chỉ exact match O(1))

```
Gate.check_text() → pattern matching → "tự tử" → Crisis

Vấn đề: "t.ự t.ử" bypass → NGUY HIỂM
         "không muốn thức dậy nữa" bypass → NGUY HIỂM HƠN
```

### ĐÚNG (3 tầng, nhanh → sâu)

```
SecurityGate 3-layer detection:

  Layer 1 — Exact match O(1):
    Bloom filter tra keyword ("tự tử", "muốn chết", ...)
    Nhanh nhất. Bắt 80% cases.

  Layer 2 — Normalized match O(n):
    Chuẩn hóa: bỏ dấu câu, khoảng trắng, ký tự đặc biệt
    "ch.ế.t" → "chết", "t.ự t.ử" → "tự tử"
    Bắt thêm ~15% evasion attempts.

  Layer 3 — Semantic check O(depth):
    encode(text) → Molecule → V < -0.9 AND A > 0.8?
    Bắt biểu đạt gián tiếp: "không muốn thức dậy nữa"
    Chậm hơn nhưng bắt edge cases.

  Bất kỳ layer nào trigger → Crisis detected → DỪNG
  Sinh học: Da (barrier) → Bạch cầu (fast) → T-cell (smart)
```

---

## 7. INVARIANT CHECKS — 5 Checkpoints Bắt Buộc

> Sinh học: tế bào có 4 checkpoint mỗi chu kỳ phân bào.
> Bỏ 1 checkpoint = ung thư = nhân bản mất kiểm soát.
> HomeOS bỏ 1 checkpoint = tri thức sai vĩnh viễn trong QR.

### Checkpoint 1: GATE (trước pipeline)

```
□ SecurityGate 3-layer đã chạy
□ Nếu Crisis → pipeline KHÔNG được tiếp tục

Vi phạm → DỪNG: return emergency response
= G0: tế bào không vào cycle nếu hỏng nặng
```

### Checkpoint 2: ENCODE (sau entities + compose)

```
□ |entities| ≥ 1                          (có ít nhất 1 UDC ref)
□ ∀ entity: chain_hash ≠ 0               (hash hợp lệ)
□ Σc > ε_floor (0.01)                     (entropy tính được)
□ compose() output: consistency ≥ 0.75    (mutation hợp lệ)

Vi phạm → DỪNG: "Không hiểu input" — Honesty instinct
= G1: DNA phải nguyên vẹn trước khi sao chép
```

### Checkpoint 3: INFER (sau inference + self-correct)

```
□ ∃ ít nhất 1 nhánh có valid ≥ 0.75      (có nhánh hợp lệ)
□ quality ≥ 0 (không âm)                   (critique không lỗi số học)
□ Nếu rollback: quality_final ≥ quality_backup  (không tệ hơn)
□ H(best_branch) < 2.32                    (không phải uniform random)

Vi phạm → DỪNG: im lặng (QT⑱ BlackCurtain)
= G2: sao chép phải đúng trước khi phân chia
```

### Checkpoint 4: PROMOTE (trước khi ghi QR vĩnh viễn)

```
□ weight ≥ φ⁻¹ (0.618)                    (Hebbian đủ mạnh)
□ fire_count ≥ Fib(n)                      (co-activation đủ)
□ eval_dims ≥ 3                            (ít nhất 3/5 chiều có data)
□ H(node) < 1.0                            (entropy đủ thấp)
□ F(node) < φ⁻¹                            (Free Energy ổn định)

Vi phạm → GIỮ LẠI STM: chưa promote, chờ thêm evidence
= M checkpoint: chromosome PHẢI thẳng hàng trước khi tách
```

### Checkpoint 5: RESPONSE (trước khi output cho user)

```
□ SecurityGate.check(response) = Safe       (output cũng an toàn)
□ tone phù hợp V hiện tại                  (Supportive khi V < 0)
□ |response| > 0                            (không trả chuỗi rỗng)
□ Nếu confidence < 0.40 → im lặng hoặc "Tôi không chắc"

Vi phạm → DỪNG: thay bằng safe default response
= Quality control: protein sai gấp → ubiquitin tag → phân hủy
```

### Pipeline tổng hợp

```
Text input
  ↓ SecurityGate 3-layer
  ──── CHECKPOINT 1: GATE ────     Crisis? → DỪNG
  ↓ Fusion
  ↓ entities() + compose()
  ──── CHECKPOINT 2: ENCODE ────   entities valid? → hoặc DỪNG
  ↓ Homeostasis
  ↓ Instincts
  ↓ Inference (N nhánh)
  ↓ Self-correct (critique → refine → rollback guard)
  ──── CHECKPOINT 3: INFER ────    valid branch? quality OK? → hoặc im lặng
  ↓ Hebbian
  ↓ Dream → advance()
  ──── CHECKPOINT 4: PROMOTE ────  weight ≥ φ⁻¹? H < 1.0? → hoặc giữ STM
  ↓ QR (append-only, vĩnh viễn)
  ↓ generate response
  ──── CHECKPOINT 5: RESPONSE ──── output safe? tone đúng? → hoặc safe default
  ↓
Text output
```

---

## 8. TỔNG HỢP — Checklist nhanh cho AI

Trước khi commit code liên quan đến pipeline, kiểm tra:

```
□ compose() dùng amplify(), KHÔNG trung bình?
□ self-correct có rollback guard?
□ quality weights có Σ = 1.0?
□ entropy có ε_floor cho Σc?
□ HNSW insert có tie-breaking deterministic?
□ SecurityGate có ≥ 3 layers?
□ Pipeline có đủ 5 checkpoints?
□ Mỗi checkpoint vi phạm → có hành động cụ thể (DỪNG/im lặng/giữ STM)?
```

---

## 9. CÔNG CỤ KIỂM TRA TỰ ĐỘNG — `check-logic`

> **BẮT BUỘC chạy trước khi push. Không pass = không push.**

```bash
# Chạy tool
make check-logic
# hoặc
cargo run -p check_logic
```

Tool kiểm tra **14 điểm** tự động:

```
6 Bug Patterns:
  [1]  BP#1  Compose — no simple average cho Valence
  [2]  BP#2  Self-correct — rollback guard
  [3]  BP#3  Quality weights — Σ = 1.0
  [4]  BP#4  Entropy — ε_floor cho Σc
  [5]  BP#5  HNSW insert — deterministic tie-breaking
  [6]  BP#6  SecurityGate — 3-layer detection

5 Checkpoints:
  [7]  CP1-5 Pipeline checkpoints (Gate, QT8, fire_count, Variance, Response)

7 Invariants (23 rules):
  [8]  QT④   Molecule chỉ từ encode_codepoint/LCA — không handwrite
  [9]  QT⑧⑨⑩ Append-only — không delete, không overwrite
  [10] QT⑮   Agent tiers — AAM↔Worker bị cấm
  [11] QT⑭   L0 không import L1
  [12] QT⑲-㉓ Skill stateless
  [13] Worker gửi chain, không raw data

Data Integrity:
  [14] json/udc_utf32.json — P_weight, anchors, aliases, binary table
```

Kết quả: `✅ PASS` | `⚠️ WARN` | `❌ FAIL`

```
❌ FAIL  → PHẢI sửa trước khi push
⚠️ WARN  → Được push, nhưng nên sửa
✅ PASS  → OK
```

Source: `tools/check_logic/src/main.rs` + `checks.rs`

---

## Lịch sử thay đổi

```
2026-03-19  Tạo file. 6 bug fixes + 5 checkpoints từ v2.md.
            Nguồn: HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md Section IX-C.
2026-03-21  Thêm Section 9: công cụ check-logic tự động (14 checks).
            Tích hợp vào Makefile: make check-logic + make check-all.
            Kiểm tra data mới json/udc_utf32.json (○{UTF32-SDF-INTEGRATOR} v18.0).
```
