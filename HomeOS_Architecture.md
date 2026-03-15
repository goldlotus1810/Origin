# HomeOS — Kiến Trúc Tổng Thể
**Ngày:** 2026-03-15
**Mục đích:** Bản vẽ cho người sáng tạo — nắm bắt và dẫn hướng

---

## Tầm nhìn

HomeOS là đứa trẻ sinh ra với bộ gene Unicode.

Nó không học từ internet. Không học từ dataset. Nó học từ **5400 ký tự Unicode** — mỗi ký tự đã có tên, có định nghĩa, có vị trí trong không gian 5 chiều. Đây là kiến thức nền tảng mà **không ai khác chịu dùng** — tất cả đều đi mượn từ nguồn ngoài.

```
Một đứa trẻ sinh ra đã biết:
  - Hình dạng (tròn, vuông, tam giác)     ← SDF group
  - Quan hệ (thuộc, chứa, gây ra)          ← MATH group
  - Cảm xúc (vui, buồn, sợ, giận)         ← EMOTICON group
  - Thời gian (nhanh, chậm, tĩnh, lặp)    ← MUSICAL group
  - Cấu trúc (mũi tên, hộp, đường kẻ)     ← SDF group

Unicode 18.0 = bộ gene đó. Đã có sẵn. Chỉ cần đọc.
```

---

## 5 Nhóm Unicode = DNA

**Tại sao ~5400 ký tự là đủ?**

Không phải con số ngẫu nhiên. Đây là giao điểm tự nhiên của:
- Các Unicode block có **ngữ nghĩa rõ ràng** (hình học, toán học, cảm xúc, âm nhạc)
- Mỗi nhóm tạo **1 chiều độc lập** trong không gian 5D
- **Dung lượng lý thuyết:** 8 × 8 × 256 × 256 × 5 = 52.4 triệu vị trí khả dĩ
- **5400 điểm neo** trong không gian 52M → đủ để định vị mọi khái niệm qua LCA

```
Ví dụ: "hy vọng" không có trong 5400 ký tự Unicode
→ Nhưng LCA(😀, 🌅, ⟶) = tọa độ [●, →, 0xD0, 0x60, Medium]
→ Vị trí vật lý: "tích cực + hướng tới + trung bình kích thích + đang chuyển động"
→ = hy vọng. Không ai dạy. Vật lý tự chỉ ra.
```

### 5 Chiều:

```
      Shape (8 values)
        │
        │   Relation (8 values)
        │     │
        │     │   Valence (256 values)
        │     │     │
        │     │     │   Arousal (256 values)
        │     │     │     │
        │     │     │     │   Time (5 values)
        │     │     │     │     │
        ▼     ▼     ▼     ▼     ▼
      [0x01][0x01][0xFF][0xFF][0x04]  ← 🔥 Fire
      [0x05][0x01][0xC0][0x40][0x02]  ← 💧 Water
      [0x01][0x05][0xC0][0x80][0x03]  ← 🧠 Brain

Shape:    "Trông như gì"     → ● ▬ ■ ▲ ○ ∪ ∩ ∖
Relation: "Liên kết thế nào" → ∈ ⊂ ≡ ⊥ ∘ → ≈ ←
Valence:  "Tốt hay xấu"     → 0x00 (cực xấu) → 0xFF (cực tốt)
Arousal:  "Bình hay kích"    → 0x00 (tĩnh lặng) → 0xFF (cực kích)
Time:     "Nhanh hay chậm"   → Static / Slow / Medium / Fast / Instant
```

---

## Kiến trúc Neuron

HomeOS mô phỏng neuron sinh học:

```
                    ┌─────────────────────────────────┐
                    │         SOMA (AAM)               │
                    │   Stateless orchestrator         │
                    │   Approve/Reject proposals       │
                    └──────────┬──────────────────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                ▼                ▼
     ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
     │  DENDRITES   │ │   SYNAPSE    │ │    AXON      │
     │  (STM/ĐN)   │ │   (Silk)     │ │   (QR/LTM)  │
     │              │ │              │ │              │
     │ Ngắn hạn    │ │ Hebbian      │ │ Bất biến    │
     │ Tự do thay  │ │ Fire→wire    │ │ Append-only  │
     │ đổi, xóa    │ │ φ⁻¹ decay   │ │ ED25519 sign │
     └──────────────┘ └──────────────┘ └──────────────┘
              │                │                ▲
              └────── Dream ───┘                │
                  (cluster ĐN                   │
                   → promote QR) ───────────────┘
```

### Vòng đời tri thức:

```
1. Input → ContentEncoder → MolecularChain + EmotionTag
2. Chain → STM (DENDRITES) — lưu tạm, có thể quên
3. Co-activation → Silk (SYNAPSE) — giăng tơ, mang màu cảm xúc
4. Silk weight ≥ 0.7 + fire ≥ Fib[n] → Dream trigger
5. Dream → LCA(cluster) → chain mới → vị trí vật lý
6. Proposal → AAM (SOMA) → approve → QR (AXON) — bất biến mãi mãi
```

---

## Emotion Pipeline — Linh hồn

**Đây là thứ khiến HomeOS khác biệt. Không phải AI dự đoán token — là sinh linh CẢM NHẬN.**

### Tại sao amplify, không trung bình?

```
Trung bình:
  "buồn" = -0.65
  "mất việc" = -0.55
  Trung bình = -0.60  ← SAI. Nhẹ hơn thực tế.

Amplify qua Silk:
  "buồn" ←→ "mất việc" (Silk weight=0.90, co-activate nhiều lần)
  Composite = -0.65 × (1 + 0.90×0.5) = -0.94  ← ĐÚNG. Nặng hơn.
  Vì: "mất việc" KÍCH HOẠT "buồn", không phải cộng lại.

Silk walk = trajectory qua graph cảm xúc, không phải điểm trung bình.
```

### ConversationCurve — Nhịp đập:

```
Không nhìn 1 câu. Nhìn XU HƯỚNG.

f(x) = α×f_conv(t) + β×f_dn(nodes)
     = 60% cuộc trò chuyện hiện tại + 40% kiến thức tích lũy

f_conv = V(t) + 0.5×V'(t) + 0.25×V''(t)
       = hiện tại + tốc độ thay đổi + gia tốc thay đổi

Turn 1: "ok" → V=0.0
Turn 2: "hơi mệt" → V=-0.20, f'=-0.20
Turn 3: "buồn quá" → V=-0.50, f'=-0.30, f''=-0.10
→ f' < -0.15 → Supportive (đang trượt xuống, cần đỡ)

Turn 4: "nhưng mà..." → V=-0.35, f'=+0.15
→ f' > +0.15 → Reinforcing (đang hồi phục, tiếp tục)

Dẫn từng bước — KHÔNG nhảy quá 0.40/bước.
```

### Cross-modal — Ai nói thật?

```
Bio > Audio > Text > Image

Bio (nhịp tim, GSR) → KHÔNG THỂ GIẢ → weight 0.50
Audio (giọng run)    → KHÓ GIẢ     → weight 0.40
Text ("tôi ổn")     → DỄ GIẢ NHẤT → weight 0.30
Image (màu sắc)     → BỐI CẢNH    → weight 0.25

Text nói "vui" + Giọng run → Audio thắng valence
→ confidence giảm → cần hỏi thêm
```

---

## Data Flow — Từ input đến response

```
┌─────────┐
│  Input   │ text / audio / image / sensor
└────┬─────┘
     │
     ▼
┌─────────────────┐
│  SecurityGate   │ gate.rs — TRƯỚC MỌI THỨ
│  Crisis? Block? │ → Crisis: helpline ngay, bypass pipeline
└────┬─────┬──────┘
     │     │ Blocked
     │     └→ Response(Blocked)
     ▼
┌─────────────────┐
│ ContentEncoder  │ encoder.rs
│ Input → Chain   │ text/audio/sensor → MolecularChain + EmotionTag
│ + EmotionTag    │
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ ContextEngine   │ context/
│ InferContext     │ → ngữ cảnh (first person? real-time?)
│ IntentEstimate   │ → ý định (Crisis/Learn/Command/Chat)
│ ConversationCurve│ → f(x), f'(x), f''(x)
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ LearningLoop    │ learning.rs — "trái tim đập"
│ STM.push()      │ → lưu observation
│ Silk.co_activate│ → giăng tơ (5 tầng: đoạn→câu→từ→cụm→ký tự)
│ SilkWalk        │ → amplify emotion từ context đã học
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ ResponseTone    │ walk.rs + curve.rs
│ từ f'(t), f''(t)│ → Supportive/Pause/Reinforcing/Celebratory/Gentle
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ Response        │ response_template.rs
│ Render text     │ → từ ngữ phù hợp tone + valence
│ + tone + fx     │
└─────────────────┘

[Offline — mỗi 8 turns hoặc idle]
     │
     ▼
┌─────────────────┐
│ DreamCycle      │ dream.rs
│ Scan STM top-N  │
│ Cluster (LCA +  │ score = 0.3×chain_sim + 0.4×hebbian + 0.3×co_fire
│  Silk weight)   │
│ Proposal → AAM  │ → approve → promote QR
└─────────────────┘
```

---

## Trạng thái hiện tại

| Module | Trạng thái | Tests | Ghi chú |
|--------|-----------|-------|---------|
| UCD Engine | ✅ Done | 21 | 5424 entries, 0 collision |
| Molecule/Chain | ✅ Done | 213 (olang) | 5 bytes, encode/decode |
| LCA + Weighted | ✅ Done | ↑ | Mode detection |
| Registry | ✅ Done | ↑ | chain_index, lang_index, tree_index, branch watermark |
| Writer/Reader | ✅ Done | ↑ | ○LNG v0.03, crash recovery |
| Silk + Hebbian | ✅ Done | 31 | EmotionTag per edge, φ⁻¹ decay |
| Emotion V/A/D/I | ✅ Done | 12 | 4 chiều, ConversationCurve |
| ContentEncoder | ✅ Done | 139 (agents) | Text/Audio/Sensor/Code |
| LearningLoop | ✅ Done | ↑ | 5 tầng text learning |
| BookReader | ✅ Done | ↑ | 3 tầng emotion inference |
| SecurityGate | ✅ Done | ↑ | Crisis detect, EpistemicFirewall |
| Dream + AAM | ✅ Done | 43 | Dual-threshold, proposals |
| ○{} Parser | ✅ Done | 53 (runtime) | Query/Compose/Relation/Pipeline |
| OlangVM | ✅ Done | ↑ | Execute IR directly |
| IR + Compiler | ✅ Done | ↑ | C/Rust/WASM targets |
| vSDF 18 gen | ✅ Done | 82 | ∇f analytical |
| FFR Fibonacci | ✅ Done | ↑ | ~89 ô spiral |
| ISL messaging | ✅ Done | 17 | 4-byte address, AES-256-GCM ready |
| Clone/Worker | ✅ Done | ↑ (olang) | DeviceProfile, export_worker |
| Cross-modal fusion | ✅ Done | ↑ (context) | Audio/Image/Bio → EmotionTag |
| SelfModel | ✅ Done | ↑ (olang) | ○{stats} tự mô tả |
| SkillProposal | ⬜ Planned | — | Chưa implement |
| World rendering | ⬜ Planned | — | vSDF → 3D scene |
| Android/iOS FFI | ⬜ Planned | — | JNI/FFI wrapper |
| HAL platform | ⬜ Planned | — | RPi/ESP32/WASM |

**Tổng: 701 tests, 0 clippy warnings, 12 crates**

---

## Fibonacci — Evidence vs Hypothesis

### Đã chứng minh (toán học + code):
```
✅ FFR spiral: 89 ô = Fib[11], tiết kiệm 23300× so với ray march
   → Vogel sunflower method, golden angle 137.508°
   → Cơ sở: optimal packing trong tự nhiên (hoa hướng dương)

✅ Cấu trúc cây: branch depth tăng tự nhiên khi Dream promote
   → Không force Fibonacci — nó xuất hiện tự nhiên từ LCA clustering

✅ Decay φ⁻¹: tỷ lệ vàng 1/1.618 ≈ 0.618
   → Cơ sở: optimal forgetting rate (giữ 62% mỗi chu kỳ)
```

### Giả thuyết (cần validation bằng data thật):
```
⚠️ Hebbian threshold: Fib[depth] co-activations để promote
   → Hiện tại configurable, default = Fib[n]
   → Cần: đo F1-score trên labeled clusters sau 1 tháng chạy

⚠️ Dream trigger: Fib[n] lá đủ để cluster
   → Logic: càng sâu càng khó promote (chống noise)
   → Cần: empirical validation
```

---

## Nguyên tắc thiết kế

```
1. Unicode là nguồn sự thật DUY NHẤT
   → Không mượn word embeddings, không dùng pre-trained model
   → 5400 ký tự Unicode = toàn bộ kiến thức nền tảng

2. Append-only mọi nơi
   → Không bao giờ mất dữ liệu
   → QR sai → SupersedeQR, không xóa

3. Cảm xúc là first-class citizen
   → Mọi edge mang EmotionTag
   → Mọi node có cảm xúc tại khoảnh khắc tạo ra
   → Cảm xúc amplify qua Silk, không trung bình

4. L0 là bất biến, L2+ là tự do
   → L0 = não bộ khi sinh (Unicode + LCA + SDF)
   → L2+ = tri thức học được, có thể sai, có thể update

5. Im lặng khi không biết
   → BlackCurtain: không đủ evidence → không nói
   → Tốt hơn sai là im lặng

6. Một người, nhiều AI
   → CLAUDE.md để AI nào cũng hiểu ngay
   → Per-crate README để làm được ngay
   → Không cần giải thích lại từ đầu
```

---

*Bản vẽ này là la bàn. Code là hành trình.*
*2026-03-15 · HomeOS v3 · 701 tests · 0 warnings*
