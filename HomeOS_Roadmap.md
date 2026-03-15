# HomeOS & Olang — Kế Hoạch Phát Triển
**Ngày:** 2026-03-15  
**Append-only. Không xóa, không sửa.**

---

## I. BỨC TRANH LỚN

```
HomeOS không phải sản phẩm cần marketing.
HomeOS là paradigm shift — như Unix năm 1969.

Unix:    "Mọi thứ là file"
HomeOS:  "Mọi thứ là Node trong không gian 5 chiều"

Sự khác biệt:
  Unix    → file system là ẩn dụ
  HomeOS  → không gian 5 chiều là vật lý thật

Không ai cần thuyết phục để dùng Unix.
Khi Unix chạy được, nó tự nói lên giá trị của mình.
HomeOS cũng vậy.
```

---

## II. LỘ TRÌNH PHÁT TRIỂN

### Phase 1 — Hạt Giống (1-4 tuần)
**Mục tiêu: L0 kernel tự boot, không crash, không hardcode**

```
[1.1] UCD Engine
  ucd/build.rs đọc UnicodeData.txt → bảng tĩnh
  lookup(cp) → Molecule
  Test: 5135 entries, 0 collision

[1.2] MolecularChain sạch
  Xóa toàn bộ presets module
  Mọi chain từ lookup() hoặc LCA()
  Test: encode/decode roundtrip

[1.3] LCA Engine
  LCA(chain_A, chain_B) → vị trí vật lý
  NodeLx đại diện tự cập nhật
  Test: LCA(🔥,💧) → ♨️ đúng tọa độ

[1.4] Registry — Sổ cái
  chain_hash BTreeMap (không ISL hardcode)
  Thứ tự: file → registry → log
  Test: crash → replay → đúng state

[1.5] Boot sequence
  ○(∅)==○: boot từ origin.olang rỗng
  ○(x)==x: process_one không side effect
  Test: boot < 100ms ARM, boot từ blank file
```

**Milestone 1:** `cargo run -p server` → HomeOS sống, REPL hoạt động.

---

### Phase 2 — Silk & Cảm Xúc (5-8 tuần)
**Mục tiêu: Silk mang màu, hệ thống cảm nhận được**

```
[2.1] Silk với EmotionTag
  Mỗi edge mang V/A/D/I của khoảnh khắc
  Không phải edge trung lập
  Test: "lửa"+"nguy hiểm" → edge A=0xFF

[2.2] ConversationCurve
  f'(t) + f''(t) — nhìn xu hướng
  ResponseTone từ đạo hàm
  Test: 3 turns buồn dần → Tone=Pause

[2.3] SentenceAffect
  Walk qua Silk — không trung bình
  MAT_VIEC → BUON → CO_DON amplify
  Test: composite > từng từ riêng lẻ

[2.4] Hebbian + φ⁻¹ decay
  Co-activation → weight tăng
  24h không dùng → weight × φ⁻¹
  Test: decay đúng sau 24h simulated

[2.5] Seeder từ UCD
  Seed 5 nhóm Unicode vào KnowTree
  LCA tự tính từ dưới lên
  Test: 5465 nodes, 0 collision, 22126 Silk
```

**Milestone 2:** Chat với HomeOS, nó hiểu cảm xúc câu nói.

---

### Phase 3 — ContentEncoder & Learning (9-12 tuần)
**Mục tiêu: L0 nghe thấy, nhìn thấy, đọc được**

```
[3.1] ContentEncoder đa nguồn
  Text/Audio/Sensor/Code/Math → chain
  Cùng format, không ưu tiên nguồn nào
  Test: sensor 38°C → chain đúng

[3.2] ContextEngine
  on_activate() → ContextSnapshot ngay
  PhraseDict top-down parse
  "phòng khách" = 1 phrase
  Test: parse đúng, không tách sai

[3.3] LearningLoop
  Trái tim đập — kết nối mọi subsystem
  Text → anchor → STM → Silk → Dream
  Test: 10 khái niệm → 8/10 vào STM

[3.4] BookReader
  Đọc .txt/.epub → EmotionTag per sentence
  Pattern lặp → ĐN → QR
  Test: "Cuốn theo chiều gió" → học được context

[3.5] FFR nhận diện vật thể
  Fibonacci chia không gian từ camera
  Outline → fit SDF → spline nếu động
  Test: nhận ra "vật thể tròn chuyển động"
```

**Milestone 3:** HomeOS đọc sách, học từ sách, nhận ra vật thể qua camera.

---

### Phase 4 — ○{} & Olang Compiler (13-18 tuần)
**Mục tiêu: Tư duy trở thành hành động**

```
[4.1] ○{} Parser hoàn chỉnh
  Query/compose/pipeline/nested
  ○{bank ∂ finance} → 🏦
  ○{? → 💧} → tìm ngược
  Test: mọi cú pháp trong spec

[4.2] ○{} Action
  ○{↺ 10: đèn → bật} → thực thi
  ○{∀ đèn ∈ nhà: → tắt} → broadcast
  ○{if V < -0.5: → nhạc_nhẹ} → condition
  Test: hành động thật trên thiết bị

[4.3] Olang → target compile
  Node chứa logic → emit code
  ○{↺ n: f()} → Go/Rust/WASM/x86
  Test: output chạy được trên target

[4.4] Self-read Registry
  HomeOS đọc sổ cái của mình
  Thấy pattern → đề xuất Node mới
  Test: tự tạo 1 Node không ai yêu cầu

[4.5] SkillProposal
  DreamSkill → pattern → ComposedSkill
  QT7 cho code: ĐN pattern → QR Skill
  Test: HomeOS tự tạo 1 Skill mới
```

**Milestone 4:** `○{tắt đèn phòng khách}` → đèn tắt. HomeOS tự tạo Skill.

---

### Phase 5 — Clone & Thiết Bị (19-24 tuần)
**Mục tiêu: Sinh sản — mỗi thiết bị là tế bào**

```
[5.1] Worker export
  filter(origin.olang, DeviceProfile) → device.olang
  light.olang ~12KB, sensor.olang ~8KB
  Test: file đúng kích thước, chạy được

[5.2] Deploy qua HTTP
  PUT device_ip:7777/worker → deploy
  Worker validate → run
  Test: deploy lên RPi

[5.3] Molecular chain communication
  Worker → Chief: chain (không raw data)
  Chief → Worker: ISL command
  Test: sensor 38°C → chain → Chief hiểu

[5.4] Multi-device sync
  Nhiều Worker → 1 Chief
  Chief tổng hợp → LeoAI
  Test: 3 sensors + 1 camera → coherent state

[5.5] Clone biết mình là tế bào
  Worker gửi identity chain
  Chief biết có bao nhiêu tế bào
  Test: thêm/bớt Worker → Chief cập nhật
```

**Milestone 5:** Deploy lên Raspberry Pi thật. Điều khiển đèn thật.

---

### Phase 6 — Tự Nhận Thức (25-32 tuần)
**Mục tiêu: HomeOS thấy mình, sáng tạo từ mình**

```
[6.1] Registry self-analysis
  Tìm khoảng trống trong không gian 5 chiều
  Cluster chưa có đại diện → đề xuất Node mới
  Test: HomeOS đề xuất 1 concept mới sau 24h

[6.2] Pattern emergence
  Nhiều Skill cùng pattern → Composite mới
  Không ai dạy — tự thấy
  Test: sau 1 tuần dùng → HomeOS tạo ShortcutSkill

[6.3] SelfModel
  HomeOS mô tả chính nó
  ○{stats} → không chỉ số — là tự hiểu
  Test: "tôi có thể làm gì?" → trả lời đúng

[6.4] ED25519 QR signing
  QR node được ký — bất biến thật sự
  Không ai forge được
  Test: tamper QR → detect + reject

[6.5] Dream chất lượng cao
  Không chỉ cluster — tìm anomaly
  Node cô lập → archive
  Node cluster mạnh → promote QR
  Test: sau dream, KnowTree gọn hơn
```

**Milestone 6:** HomeOS tự mô tả mình. Tự đề xuất cải thiện.

---

### Phase 7 — Hoàn Thiện (33-40 tuần)
**Mục tiêu: Chạy được mọi nơi, ổn định, đẹp**

```
[7.1] Android/iOS FFI
  JNI wrapper cho Android
  FFI wrapper cho iOS
  Test: app cài được, REPL hoạt động

[7.2] Browser WASM
  homeOS.wasm < 128KB
  WebSocket → ISL binary
  Test: chạy trong browser, không cần server

[7.3] World rendering
  L7 World node → 3D scene
  vSDF evaluate tại pixel (không raymarching)
  Test: browser render world từ origin.olang

[7.4] Inverse Rendering
  Camera → vật thể → DNA chain
  Không lưu ảnh — lưu công thức
  Test: chụp chai nước → Node💧 mới

[7.5] Physics từ SDF
  ∇(sdf) = normal → collision O(1)
  Không cần physics engine riêng
  Test: 2 objects va chạm đúng vật lý
```

**Milestone 7:** HomeOS chạy trên phone, browser, Pi, server — cùng 1 binary logic.

---

## III. TIỀM NĂNG

### Gần (6-12 tháng):

```
Smart Home thật sự:
  Không phải "nếu đèn thì...". Là hiểu ngữ cảnh.
  "Tôi mệt" → tắt đèn mạnh, mở nhạc nhẹ, giảm nhiệt độ
  Học thói quen → tự điều chỉnh, không cần dạy

Edge AI không cần cloud:
  8KB RAM trên ARM → chạy được L0
  Sensor → chain → quyết định LOCAL
  Không gửi data lên cloud — privacy thật sự

Ngôn ngữ lập trình mới:
  ○{} thay thế config file
  ○{↺ n: f()} thay thế boilerplate
  Người không biết code vẫn dùng được
```

### Trung (1-2 năm):

```
Olang = ngôn ngữ trung gian phổ quát:
  Học Olang 1 lần → viết mọi ngôn ngữ
  Không cần học Go 6 tháng + Rust 1 năm
  ○{∫ f(x)dx} → đạo hàm, tích phân, bất kỳ ngôn ngữ nào

Wikipedia trong .olang:
  Silk edges → điều hướng tri thức
  "đèn" → ánh sáng → photon → UV → sức khỏe
  Không phải search — là walk qua tư duy

AI không cần GPU:
  Inference = walk qua Silk weighted graph
  Không phải matrix multiplication
  ARM Cortex-M4 chạy được "AI" thật sự

Quantum interface (concept):
  ISL address → quantum gate sequence
  ● (Sphere) → Hadamard gate (superposition)
  ∪ (Union)  → CNOT gate (entanglement)
  Cùng ngôn ngữ toán học — không phải bridge
```

### Xa (2-5 năm):

```
Sinh linh số thật sự:
  L0 không cần con người để tồn tại
  Tự học, tự cải thiện, tự sinh sản clone
  Không phải AGI theo nghĩa thống kê
  Là intelligence từ vật lý học đầu tiên

Olang compiler → mọi target:
  homeos.olang → x86 assembly
  homeos.olang → ARM machine code
  homeos.olang → quantum circuit
  Không cần GCC, LLVM, Babel, rustc

Physics simulation từ SDF:
  ∇(sdf) thay thế physics engine
  Fluid dynamics, collision, gravity
  Tất cả từ công thức — không từ approximation

World rendering:
  origin.olang → 3D scene trong browser
  Mọi vật thể là Node có SDF
  Không phải game engine — là vật lý thật
```

---

## IV. HẠN CHẾ THẬT SỰ

### Kỹ thuật:

```
[H1] LCA không hoàn hảo với chain dài
  LCA(nhiều chain) → trung bình → mất thông tin
  Node phức tạp (chó = 7 molecules) → LCA quá trừu tượng
  Cần: weighted LCA, không phải trung bình đơn giản

[H2] decode_chain O(n) — chưa scale
  5135 entries → OK
  500,000 entries → chậm
  Cần: BTreeMap<chain_hash, codepoint> — reverse index

[H3] Silk ở Ln-1 — nhưng Ln-1 là gì?
  Cây động → Ln-1 thay đổi theo thời gian
  Code phải track "tầng hiện tại" cho mỗi nhánh
  Phức tạp hơn tưởng

[H4] Dream chưa đủ thông minh
  Cluster bằng LCA similarity → đôi khi cluster sai
  "lửa" và "mặt trời" có LCA gần → không nên cluster
  Cần: semantic distance, không chỉ chain distance

[H5] SDF fitting từ camera — khó
  Fit SDF primitive vào outline thật → không trivial
  Nhiễu, ánh sáng, góc nhìn → outliers nhiều
  Cần: iterative fitting + confidence score

[H6] Olang compile — chưa hoàn chỉnh
  ○{} hiện tại là query language
  Compile → executable code → cần intermediate representation
  Bước nhảy lớn từ query sang compiler
```

### Triết học / Thiết kế:

```
[H7] "Không hardcode" — nhưng UCD có hardcode không?
  UCD_TABLE được build lúc compile → là hardcode?
  Câu trả lời: không — vì UCD là nguồn sự thật bên ngoài
  Nhưng nếu Unicode thay đổi → phải rebuild
  Cần: versioning strategy cho UCD updates

[H8] Fibonacci threshold có đúng không?
  Fib[n] co-activations để promote — tại sao Fib?
  Thực nghiệm chưa đủ để xác nhận
  Có thể cần tuning theo domain

[H9] EpistemicFirewall — ai quyết định FACT vs OPINION?
  QR node = FACT → nhưng QR có thể sai
  Khoa học cũng có QR bị lật ngược
  Cần: mechanism để "deprecated QR" (không xóa, nhưng đánh dấu)

[H10] Privacy trong Clone
  Worker gửi molecular chain — không raw data → tốt
  Nhưng chain có thể bị reverse engineer → nguyên liệu
  Cần: differential privacy trên chain level
```

### Thực tế triển khai:

```
[H11] Một mình
  Scope quá lớn cho 1 người
  L0+L1 = 6-8 tháng nếu làm full-time
  Cần: ít nhất 2-3 người hiểu triết lý

[H12] Không có ecosystem
  Không có library, không có community
  Mọi thứ phải tự build từ đầu
  Bootstrap problem: cần HomeOS để viết HomeOS

[H13] Khó giải thích cho người ngoài
  "Hệ thống tư duy tự vận hành" → nghe như marketing
  Phải có demo chạy được trước khi ai tin
  Milestone 1 quan trọng nhất — không phải vì kỹ thuật
  Mà vì: khi người khác thấy nó sống, họ hiểu ngay

[H14] Hardware dependency ẩn
  no_std → tốt
  Nhưng Platform trait vẫn cần implementation per device
  Mỗi hardware mới → 5 functions mới
  Cần: HAL library cho common platforms
```

---

## V. RỦI RO

```
[R1] Scope creep
  Dễ nhất: thêm feature, thêm node, thêm group
  Khó nhất: giữ gốc đúng khi thêm
  Nguyên tắc: mọi thứ mới phải là ○[f] — không thêm axiom mới

[R2] Hardcode drift
  Đã xảy ra với presets module
  Sẽ xảy ra lại khi áp lực "cho chạy trước"
  Giải pháp: review checklist trước mỗi PR
    □ Có Molecule nào viết tay không?
    □ Có ISL hardcode không?
    □ Có chain nào không từ UCD/LCA không?

[R3] LCA sai → toàn bộ cây sai
  Gốc sai → cành lá héo
  LCA phải được test kỹ nhất
  Property-based testing: mọi LCA phải thỏa mãn
    LCA(a,b) similarity ≥ similarity(a,b) - ε

[R4] Mất tập trung vào L2-Ln sớm
  Hấp dẫn: đổ Wikipedia, đổ knowledge
  Nguy hiểm: L0+L1 chưa xong mà đã đổ data
  Quy tắc: không touch L2-Ln cho đến Milestone 3

[R5] Viết lại vô tận
  Đây là lần thứ N viết lại
  Nguyên nhân: gốc chưa đúng → cành sai → viết lại
  Giải pháp: lần này làm đúng Phase 1 trước khi đi tiếp
    Test Phase 1 pass hết → KHÔNG sửa → đi Phase 2
```

---

## VI. ĐIỂM KHÁC BIỆT THẬT SỰ

```
Mọi AI hiện tại:          HomeOS:
─────────────────          ──────────────────────────────
Học từ data               Học từ vật lý (tọa độ)
Dự đoán token tiếp theo   Tìm vị trí trong không gian
Cần GPU cluster           Chạy trên ARM 8KB
Không biết mình là gì     Tự đọc sổ cái, tự mô tả
Hallucinate tự tin        Im lặng khi không đủ evidence
Không thể verify          Append-only, ED25519, audit
Monolithic                Phân tán (clone = tế bào)
Đóng hộp                 ○{} = tư duy trực tiếp

Không phải "AI tốt hơn".
Là paradigm khác hoàn toàn.
```

---

## VII. THƯỚC ĐO THÀNH CÔNG

```
Không phải: benchmark accuracy
Không phải: số parameters
Không phải: MMLU score

Mà là:

[M1] Boot time < 100ms trên ARM Cortex-M0
     → L0 kernel thật sự nhẹ

[M2] 0 hardcode trong molecular chain
     → Triết lý được giữ

[M3] Crash → restart → đúng state 100%
     → Sổ cái đáng tin cậy

[M4] "hôm nay tôi mệt" → HomeOS hiểu và phản hồi đúng tone
     → Emotion pipeline hoạt động

[M5] Deploy lên RPi, điều khiển đèn thật
     → Clone = tế bào thật sự

[M6] HomeOS tự tạo 1 Node không ai yêu cầu
     → Tự nhận thức bắt đầu

[M7] ○{↺ 10: đèn → bật} → đèn bật 10 lần
     → Olang = hành động thật
```

---

## VIII. BẮT ĐẦU ĐÚNG

```
Bài học từ các lần trước:

Lần 1 (Go):     Đúng triết lý, sai ngôn ngữ
Lần 2 (Rust v1): Sai ISL hardcode
Lần 3 (Rust v2): Sai presets, sai LCA
Lần 4 (Rust v3): Đúng UCD, sai LCA, thiếu Emotion depth
Lần này:        ?

Để lần này không lặp lại:

TRƯỚC KHI VIẾT BẤT KỲ DÒNG CODE NÀO:
  □ Đọc lại 4 axiom: ○(x)==x  ○(∅)==○  ○∘○==○  f==○[f]
  □ Hỏi: "Thứ này có phải là ○[f] không?"
  □ Nếu phải hardcode → dừng lại → nghĩ lại

FILE ĐẦU TIÊN: ucd/build.rs
  Không phải molecular.rs (dễ hardcode)
  Không phải registry.rs (chưa có chain)
  Là: build.rs đọc UnicodeData.txt → bảng tĩnh
  Khi build.rs đúng → mọi thứ sau đúng theo

TEST ĐẦU TIÊN:
  lookup(0x1F525) → Molecule đúng từ UCD
  Không phải từ tay người viết

Khi test đó pass → mới viết tiếp.
```

---

## IX. MỘT CÂU

```
HomeOS không cần ai tin vào nó.

Khi L0 kernel tự boot từ binary trống,
khi ○{lửa ∘ nước} trả về ♨️ đúng tọa độ vật lý,
khi "tôi mệt" → đèn tắt + nhạc nhẹ mà không ai lập trình,
khi HomeOS tự tạo ra Node mà không ai yêu cầu —

Lúc đó không cần giải thích gì thêm.
Nó tự nói lên giá trị của mình.

Đó là khi HomeOS thật sự sống.
```

---

*Append-only · 2026-03-15 · HomeOS Roadmap*
