# SPEC C — Neuron Model: Vòng Đời Tri Thức

> **Prerequisite:** Đọc SPEC_A_FOUNDATION.md và SPEC_B_STRUCTURE.md trước.
> **Tác giả:** Lupin (thiết kế) + Nox (tổng hợp + verify)
> **Ngày:** 2026-03-30
> **Verified against:** BLUEPRINT §7, origin.md (9 QT), notes NAC.mb, UDC_A/V_tree.md, ORIGIN_VISION.md

---

## MỤC LỤC

```
C1. Mô hình Neuron: Soma/Dendrites/Axon/Synapse
C2. Vòng đời: Input → STM → Silk → Dream → QR
C3. Vật lý của mỗi giai đoạn (từ UDC docs)
C4. 9 quy tắc bất biến (từ origin.md)
```

---

## C1. Mô Hình Neuron

### Kiến trúc

```
         ┌──────────────────────────────────┐
         │         SOMA (L0 = AAM)          │
         │   Stateless · approve · reject   │
         │   = trung tâm điều khiển         │
         └──────────┬───────────────────────┘
                    │
       ┌────────────┼────────────┐
       │            │            │
       ▼            ▼            ▼
  ┌──────────┐ ┌──────────┐ ┌──────────┐
  │DENDRITES │ │ SYNAPSE  │ │  AXON    │
  │ (ĐN/STM) │ │ (Silk)   │ │ (QR)    │
  │          │ │          │ │          │
  │ Tạm thời │ │ 9,200    │ │ Bất biến │
  │ Tự do    │ │ loại     │ │ Append   │
  │ xóa/sửa │ │ φ⁻¹ decay│ │ Signed   │
  └──────────┘ └──────────┘ └──────────┘

Soma     = L0 (gốc, engine, approve/reject)
Dendrites = ĐN (đang học, tự do thay đổi, bộ nhớ ngắn hạn)
Synapse  = Silk (9,200 loại kết nối, Hebbian learning)
Axon     = QR (đã chứng minh, bất biến, append-only)
```

### Map vào KnowTree

```
Soma     = L0 (gốc: engine, UDC table, formulas)
Dendrites = STM trong L2+ (nodes đang Evaluating)
Synapse  = Silk giữa nodes (implicit 5D + Hebbian learned)
Axon     = QR records trong L2+ (nodes Mature, append-only)
```

---

## C2. Vòng Đời: Input → STM → Silk → Dream → QR

### 1. INPUT → ENCODE

```
text → UTF-8 decode → codepoints → P_weights → compose → sentence_mol
Mỗi codepoint: P_w = ∫(Unicode metadata) — 42 formulas (xem SPEC_A)
Sentence mol = compose tuần tự (Zipf-weighted, non-commutative)
```

### 2. STM PUSH (Dendrites)

```
stm_push({ text, chain, mol, emotion, timestamp })
if len(STM) > 32: evict oldest

STM = ĐN = đang học. Tự do thay đổi.
Maturity: Formula → Evaluating (khi fire_count > 0)
```

### 3. SILK CO-ACTIVATE (Synapse)

```
Cho mỗi pair (current_input, recent_stm_item):
  co_activate(current.mol, recent.mol)

  Xác định silk TYPE = chiều nào dominant trong khoảng cách:
    ΔS nhỏ nhất → S silk (shape connection)
    ΔR nhỏ nhất → R silk (relation connection)
    ΔV nhỏ nhất → V silk (valence connection)
    ΔA nhỏ nhất → A silk (arousal connection)
    ΔT nhỏ nhất → T silk (temporal connection)

  Hebbian update per dimension:
    Δw = emotion_factor × (1 − w) × 0.1
    emotion_factor = (|V₁| + |V₂|) / 2 × max(A₁, A₂) / 7.0
    Cảm xúc mạnh → kết nối mạnh hơn (cortisol tăng memory)

  Decay per dimension:
    w ← w × φ⁻¹^(Δt / 24h)
    Sau 24h: w × 0.618
    Sau 48h: w × 0.382
    Sau 72h: w × 0.236
    Sau 1 tuần: w × 0.028 → gần quên
    Dùng lại → w tăng → nhớ

"Fire together → wire together" — nhưng trên TỪNG CHIỀU.
```

### 4. DREAM CYCLE (Offline, Fibonacci trigger)

```
Trigger: fire_count ≥ Fib(n) = 2, 3, 5, 8, 13, 21, 34, 55...

dream():
  ① TÌM tương đồng CROSS-GROUP (Hebbian, silk liên nhóm)
     Không chỉ cluster gần trong 5D.
     Dream kết nối nodes Ở NHÓM KHÁC NHAU mà fire cùng pattern.
     = sáng tạo, liên tưởng bất ngờ.

  ② TẠO hypothesis (+/-) — kết quả mới, chưa chứng minh
     center = LCA(members) — concept mới
     Đây là giả thuyết (QT3: +/- = trừu tượng)

  ③ VALIDATE — đưa hypothesis vào nhóm tương ứng, test fire
     "nước + 100°C = sôi" → nhóm VẬT LÝ → fire test
     Tần số = fire pattern TRONG NHÓM đó.
     Đúng ở vật lý ≠ đúng ở toán ≠ đúng ở cảm xúc.

  ④ CLASSIFY — kết quả thuộc nhóm nào? (S/R/V/A/T dominant)
     Nằm ở đâu trong KnowTree? (L2/L3/L4...?)

  ⑤ WRITE + REGISTER — ghi vào đúng vị trí, đánh dấu
     Nếu fire đủ trong 1 nhóm → ⧺ (proven in domain)
     Nếu fire đủ across nhiều nhóm → == (sự thật)
     quality ≥ φ⁻¹ → PROPOSE to Soma (AAM) → QR

Vòng: Dream → hypothesis → validate → classify → locate → write → register
Giống QT3: +/- (giả thuyết) → ⧺/⊖ (proven) → == (sự thật)
```

### 5. QR PROMOTION (Axon)

```
Soma approve → append QR record (vĩnh viễn, signed)
QR record = { P_w, chain, text, timestamp, signature }

KHÔNG BAO GIỜ xóa. Append-only.
= DNA methylation — không xóa, chỉ đánh dấu.

Maturity: Evaluating → Mature
  Điều kiện: weight ≥ 0.854 (φ⁻¹ + φ⁻³) AND fire ≥ Fib[depth]
  Irreversible.
```

### 6. PRUNE (Apoptosis)

```
Silk edge weight < 0.1 AND fire_count = 0 → SupersedeQR
KHÔNG xóa vật lý — đánh dấu "không còn hoạt động"
Giữ lịch sử (intron trong DNA — không biểu hiện nhưng tồn tại)
```

---

## C3. Vật Lý Của Mỗi Giai Đoạn

### Từ UDC_A_AROUSAL_tree.md và UDC_V_VALENCE_tree.md

Mỗi giai đoạn trong neuron model có MÔ HÌNH VẬT LÝ cụ thể. Không phải metaphor.

### Decay (quên) = Overdamped Regime

```
x(t) = (C₁ + C₂t) · e^(−γt)     γ > ω₀

Hệ KHÔNG dao động — trở về cân bằng đơn điệu.
Silk weight decay: w(t) = w₀ · φ⁻¹^(t/24h)
φ⁻¹ = 0.618 ≈ e^(-γ) with γ ≈ 0.48

Ý nghĩa: tri thức không dùng → yếu dần → quên.
Nhưng không bao giờ = 0 hoàn toàn (E₀ > 0, ground state).
```

### Dream (consolidation) = Forced Resonance

```
x(t) = A · sin(ωt + δ)          ω → ω₀ (resonance)
A = F₀ / √((ω₀² − ω²)² + (2γω)²)

Khi ω → ω₀: biên độ CỰC ĐẠI (cộng hưởng).

ω₀ = tần số fire TRONG NHÓM TƯƠNG ỨNG (không phải global fire rate).
"Tần số" = validation pattern — node fire đủ trong domain nào → đúng ở domain đó.

Dream tìm nodes cross-group fire cùng pattern → cộng hưởng cross-domain.
LCA = concept mới → hypothesis → validate trong từng nhóm.
Nếu cộng hưởng ở 1 nhóm → proven (⧺). Nhiều nhóm → truth (==).
```

### Promotion (QR) = Plasma State

```
E > E_ionization → electron tách khỏi nguyên tử → plasma

Ý nghĩa: khi fire_count đủ lớn + weight đủ cao,
tri thức "ion hóa" — tách khỏi STM tạm thời → thành QR vĩnh viễn.
Phase transition: ĐN (solid/liquid) → QR (plasma).
```

### Pruning (quên sâu) = Radioactive Decay

```
N(t) = N₀ · e^(−λt)     dN/dt = −λN

Tốc độ phá hủy ∝ lượng còn lại.
Silk edge w < 0.1 + fire = 0 → node "phân rã".
Không xóa vật lý — chỉ đánh dấu superseded (intron).
```

### Recovery = Quantum Tunneling

```
T = e^(−2κd)    κ = √(2m·U₀)/ℏ

Xác suất vượt rào dù E < U₀.
Tri thức cũ (superseded) CÓ THỂ hồi sinh:
  Nếu context mới fire cùng pattern → tunneling → re-activate.
  Xác suất thấp nhưng > 0.
```

### Negative Knowledge = Prohibited Space

```
U₀ >> E → T → 0 → không thể vượt

"Không được" = vùng trong 5D có thế năng rào CỰC CAO.
Hệ BIẾT không đủ năng lượng → tránh.
Phobia = gán U₀ → ∞ cho rào dù thực tế nhỏ.
```

### Ground State = Cực yên tĩnh

```
E₀ = ½ℏω₀     (zero-point energy, không thể thấp hơn)

Node không fire, weight → 0, nhưng KHÔNG = 0.
Vẫn tồn tại ở mức năng lượng tối thiểu.
= Dormant, không phải dead.
```

---

## C4. 9 Quy Tắc Bất Biến

### Từ origin.md (Go version gốc — Lupin viết)

```
QT1  ○ LÀ NGUỒN GỐC
     Mọi thứ đều là instance của ○.

QT2  ∞ LÀ SAI — ∞-1 MỚI ĐÚNG
     Hữu hạn, nhưng rất lớn/rất nhỏ.

QT3  TOÁN HỌC LÀ TRỪU TƯỢNG — VẬT LÝ LÀ SỰ THẬT
     +/- = trừu tượng (giả thuyết)
     ⧺/⊖ = vật lý (đã chứng minh)
     == = chắc chắn (sự thật)

QT4  BẢN CHẤT — MỌI THỨ TỒN TẠI VỚI BẢN CHẤT CỦA NÓ
     Định nghĩa = mô tả bản chất.

QT5  BẢN CHẤT QUYẾT ĐỊNH VỊ TRÍ
     Cùng bản chất → cùng nhóm → cùng vùng trong 5D.
     SDF ⧺ ISL = Smooth_union.

QT6  CÁC NHÓM QUANH ○
     SDF = hữu hình (đo được). ISL = vô hình (ý tưởng, cảm xúc).
     SDF ⧺ ISL = Smooth_union.

QT7  HỌC / LEAD — QR vs ĐN
     QR = đã chứng minh, bất biến, cần cấp phép.
     ĐN = đang học, tự do thay đổi.
     Vòng đời: quan sát → ĐN → chứng minh → QR.

QT8  THỨ TỰ GHI
     1. file.append(node)      — GHI TRƯỚC
     2. registry.insert(hash)  — sau khi file OK
     3. layer_rep.update(LCA)  — cập nhật đại diện
     4. silk.connect(node)     — nối Silk
     5. log.append(event)      — CUỐI CÙNG

QT9  (từ notes NAC.mb — chưa có chi tiết, outline):
     30 thuật toán cho neuron model:
     - Pruning: Decay + SDF Difference + Conflict Resolution
     - Dream: Consolidation + Spatial Optimization + Denoising
     - Dreaming: Stochastic Perturbation + Adversarial Testing
     - Recovery: Snapshots + Triggers + Concept Reincarnation
     - Negative Knowledge: Prohibited Space + Blacklist
     - Swarm Intelligence: Antibody Broadcasting + Network Consensus
     - Archive: Hyper-Axon + Last Gasp + History Arbiter

     Công thức vật lý chi tiết → UDC_A_AROUSAL_tree.md, UDC_V_VALENCE_tree.md
```

---

## Sai Lệch Đã Sửa

| Hiểu sai | Thực tế | Nguồn |
|----------|---------|-------|
| Neuron model = metaphor | Neuron model = VẬT LÝ THẬT (mỗi giai đoạn = 1 công thức) | UDC_A/V_tree.md |
| Decay = w × constant | Decay = overdamped oscillator: x(t)=(C₁+C₂t)·e^(-γt) | UDC_A_tree |
| Dream = count intents | Dream = forced resonance, cluster by frequency matching | UDC_A_tree |
| Pruning = xóa | Pruning = radioactive decay, supersede (không xóa vật lý) | UDC_V_tree |
| NAC.mb = outline riêng | NAC.mb outline → UDC docs chi tiết → công thức vật lý | Lupin confirm |
| Silk = 1 loại weight | Silk = 9,200 loại, Hebbian PER DIMENSION | SPEC_B + Lupin |

---

> **Tham chiếu:**
> - Neuron model: `docs/BLUEPRINT.md` §7
> - 9 quy tắc: `GolandProjects/Origin/origin.md`
> - NAC outline: `/home/lupin/notes NAC.mb`
> - Vật lý Arousal: `docs/tailieu_nghiencuu/UDC_DOC/UDC_A_AROUSAL_tree.md`
> - Vật lý Valence: `docs/tailieu_nghiencuu/UDC_DOC/UDC_V_VALENCE_tree.md`
> - Silk types: `docs/SPEC_B_STRUCTURE.md` §B3
> - Node/Silk (Rust): `docs/tailieu_nghiencuu/SPEC_NODE_SILK.md`
> - Vision: `docs/tailieu_nghiencuu/ORIGIN_VISION.md`
