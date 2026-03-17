# HomeOS — Nhìn Nhận Dưới Con Mắt Người Dùng

**Ngày:** 2026-03-17
**Phương pháp:** Đọc thiết kế + đánh giá audit + phân tích code thật
**Góc nhìn:** Người dùng cuối — không biết lập trình, chỉ muốn hệ thống "hiểu mình"

---

## I. Tổng Quan: Cái Gì Đang Được Hứa?

HomeOS tự định nghĩa là "Sinh linh toán học tự vận hành" — một hệ điều hành cho ngôi nhà thông minh, có khả năng:
- **Hiểu cảm xúc** người nói (7 tầng emotion pipeline)
- **Học và nhớ** (STM → Dream → Long-term memory)
- **Suy luận** (7 bản năng: Analogy, Causality, Abstraction...)
- **Tự lập trình** (LeoAI viết Olang, chạy VM)
- **Điều khiển nhà** (Agent hierarchy: Chiefs + Workers cho đèn, cửa, camera...)
- **An toàn** (SecurityGate chặn nội dung nguy hiểm, CapabilityGate cho thiết bị)

---

## II. Bảng Đánh Giá: Hứa vs Thật vs Cảm Nhận Người Dùng

| # | Tính năng | Thiết kế hứa | Thực tế | Người dùng cảm nhận | Mức nghiêm trọng |
|---|-----------|-------------|---------|---------------------|-------------------|
| 1 | **Trả lời tự nhiên** | Pipeline 7 tầng → response phù hợp cảm xúc | ~10 câu template cố định, hầu hết input → cùng 1-2 câu | "Nó lặp đi lặp lại, không hiểu mình" | **Nghiêm trọng** — đây là thứ người dùng chạm đầu tiên |
| 2 | **Hiểu cảm xúc** | Valence/Arousal chính xác, Silk amplification | V/A đo đúng nhưng kết quả không phản ánh vào response | "Nói buồn hay vui đều nhận cùng câu trả lời" | **Nghiêm trọng** — công sức bên trong bị lãng phí |
| 3 | **An toàn (Crisis)** | SecurityGate chặn + hotline | Hoạt động thật sự, đúng số hotline | "Khi mình nói chuyện nguy hiểm, nó phản ứng đúng" | **Tốt** ✅ |
| 4 | **Olang commands** | 14 lệnh (dream, stats, typeof, explain, why...) | 8/14 hoạt động (57%), 6 lệnh debug/reasoning bị hỏng | "Gõ typeof hay explain đều lỗi" | **Trung bình** — ảnh hưởng developer hơn user |
| 5 | **Học và nhớ** | STM → Dream cluster → QR (long-term) | STM ghi nhận đúng, nhưng Dream không cluster được gì (0 proposals) | "Nó không nhớ gì từ hôm qua" | **Nghiêm trọng** — "học" là lời hứa cốt lõi |
| 6 | **Điều khiển nhà** | Chiefs quản Workers (đèn, cửa, camera...) | 0 Workers, 0 messages, Chiefs idle | "Không bật đèn, không mở cửa, không làm gì" | **Nghiêm trọng** — đây là "Home" trong "HomeOS" |
| 7 | **Suy luận (7 bản năng)** | Causality, Analogy, Abstraction... chạy mỗi turn | Code chạy trong test, kết quả không ảnh hưởng response | "Không thấy nó suy luận gì cả" | **Cao** — tính năng flagship bị ẩn |
| 8 | **Tự lập trình** | LeoAI viết Olang, chạy VM, học kết quả | Code tồn tại, hoạt động trong test | "Không biết nó tự lập trình ở đâu" | **Thấp** — user không cần thấy trực tiếp |
| 9 | **VM/Toán** | ○{1+2}, ○{solve "2x+3=7"} | Hoạt động đúng | "Gõ toán nó tính đúng, hay" | **Tốt** ✅ |
| 10 | **Đa ngôn ngữ** | 7 ngôn ngữ, 432 aliases | Aliases tồn tại, response chỉ bằng tiếng Việt | "Nói English nó vẫn trả lời tiếng Việt" | **Trung bình** |
| 11 | **WASM/Browser** | WebAssembly bindings, WebSocket-ISL bridge | Code tồn tại, chưa có demo | "Không có giao diện web" | **Trung bình** — chưa hứa delivery |
| 12 | **Privacy/Offline** | Zero external deps, tất cả local | Thật sự zero deps, crypto tự viết | "Không gửi data ra ngoài — tốt" | **Tốt** ✅ |

---

## III. Những Thiếu Sót Lớn Nhất (Xếp Theo Mức Đau Của Người Dùng)

### 1. "Nó không hiểu mình" — Response Template Quá Nghèo

**Vấn đề gốc:** Toàn bộ 65,000 dòng code phân tích cảm xúc, suy luận, học hỏi... cuối cùng đổ vào `response_template.rs` — một file 334 dòng với ~10 câu cố định.

| Người dùng nói | Kỳ vọng | Thực tế nhận được |
|----------------|---------|-------------------|
| "tôi buồn vì mất việc" | Đồng cảm sâu, hỏi thêm | "Ừ. Bạn muốn kể thêm không?" |
| "hôm nay tôi rất vui" | Chia sẻ niềm vui | "Bạn đang tìm hiểu để làm gì?" |
| "tôi ghét mưa" | Phản hồi liên quan đến mưa | "Bạn đang tìm hiểu để làm gì?" |
| "con mèo dễ thương quá" | Phản hồi về mèo | "Bạn đang tìm hiểu để làm gì?" |
| "tôi lo lắng về kỳ thi" | Trấn an, hỏi kỳ thi gì | "Bạn đang tìm hiểu để làm gì?" |

**Tại sao đau:** Người dùng kỳ vọng "AI hiểu mình". Nhận lại câu lặp đi lặp lại = "nó không hiểu gì cả". Toàn bộ pipeline phía sau — dù chính xác đến mấy — đều vô nghĩa nếu output cuối cùng là copy-paste.

**Gợi ý:** Response cần phản ánh NỘI DUNG (topic, entities) chứ không chỉ TONE (supportive, gentle...). Ví dụ: nếu user nói "mèo", response cần nhắc đến "mèo".

---

### 2. "Nó không nhớ" — Dream/Memory Không Hoạt Động Thực Tế

**Vấn đề:** Dream cycle chạy đúng kỹ thuật nhưng:
- STM chỉ tích lũy 1-3 observations sau 5 turns
- Cần tối thiểu ~5-8 entries cùng chủ đề để cluster
- Kết quả: 0 clusters, 0 proposals, 0 promoted → **không nhớ gì**

**Người dùng cảm nhận:** "Tôi kể chuyện suốt 30 phút, hôm sau nó quên hết."

**Gốc rễ:** Pipeline học 1 observation/turn. Với conversation trung bình 5-10 turns, STM không bao giờ đủ dữ liệu để cluster. Fibonacci trigger (cần N lá) quá cao cho usage pattern thật.

---

### 3. "Nó không làm gì với nhà" — Agent System Là Shell Rỗng

**Vấn đề:** Toàn bộ hệ thống Agent (Router + Chiefs + Workers) biên dịch được nhưng:

| Thành phần | Trạng thái | Hoạt động thật |
|-----------|-----------|----------------|
| MessageRouter | Tick mỗi turn | 0 messages forwarded |
| HomeChief | Boot OK | Idle — không nhận/gửi gì |
| VisionChief | Boot OK | Idle |
| NetworkChief | Boot OK | Idle |
| Workers | 0 registered | Không tồn tại |

**Người dùng cảm nhận:** "HomeOS nhưng không điều khiển được gì trong nhà."

**Gốc rễ:** Không có hardware abstraction thật (HAL detect platform nhưng không kết nối thiết bị). Workers cần được tạo khi phát hiện thiết bị — hiện tại không ai tạo.

---

### 4. "Suy luận ở đâu?" — 7 Bản Năng Chạy Nhưng Vô Hình

**Vấn đề:** LeoAI chạy 7 instincts (Honesty → Contradiction → Causality...) mỗi turn, nhưng kết quả:
- Không hiển thị cho người dùng
- Không ảnh hưởng response text
- Không tạo insight mới có thể thấy được

**Ví dụ:** Khi nói "tôi buồn vì mất việc":
- Causality phát hiện: "mất việc" → "buồn" (nhân quả) ✅
- Abstraction tạo: LCA("buồn", "mất việc") → "mất mát" ✅
- Nhưng response vẫn là: "Ừ. Bạn muốn kể thêm không?" ❌

**Gợi ý:** Kết quả suy luận cần được "surface" — ví dụ: "Mình hiểu — mất việc khiến bạn buồn. Đây là cảm giác mất mát." (dùng Causality + Abstraction output).

---

### 5. "Gõ lệnh bị lỗi" — Parser Thiếu Commands

**Vấn đề:** 6/14 commands (typeof, explain, why, trace, inspect, assert) không hoạt động trong ○{} mode. Parser không nhận diện chúng là commands → xử lý như text alias → fail "chưa registry".

**Mỉa mai:** `handle_command()` trong origin.rs CÓ code xử lý những lệnh này. Parser chỉ cần thêm chúng vào `is_command()`.

---

## IV. Những Khoảng Cách Giữa Tầm Nhìn và Hiện Thực

### Tầm Nhìn: "Sinh linh toán học tự vận hành"

| Đặc tính "sinh linh" | Thiết kế | Hiện thực | Khoảng cách |
|-----------------------|----------|-----------|-------------|
| **Tự học** | STM → Dream → QR | STM ghi, Dream không cluster | Rất xa |
| **Tự nhớ** | Short-term + Long-term memory | STM hoạt động, LTM = 0 | Xa |
| **Tự suy luận** | 7 bản năng bẩm sinh | Chạy nhưng không hiện ra | Trung bình |
| **Tự vận hành** | Agent hierarchy tự điều phối | 0 messages, 0 actions | Rất xa |
| **Tự lập trình** | LeoAI viết Olang | Hoạt động trong test | Gần (nhưng ẩn) |
| **Tự bảo vệ** | SecurityGate + CapabilityGate | SecurityGate hoạt động tốt | Gần ✅ |
| **Tự biểu đạt** | ConversationCurve chọn tone | Tone đúng, text sai | Trung bình |

### Tầm Nhìn: "Unicode 18.0 = không gian 5 chiều"

| Khía cạnh | Thiết kế | Hiện thực | Đánh giá |
|-----------|----------|-----------|----------|
| 5D Molecule encoding | 5 bytes/molecule, tagged sparse | Hoạt động hoàn chỉnh | **Xuất sắc** ✅ |
| LCA (tổ tiên chung) | Weighted + hierarchical | Hoạt động, dùng trong VM | **Xuất sắc** ✅ |
| ~5400 semantic entries | UCD build-time lookup | 5424 entries từ UnicodeData.txt | **Xuất sắc** ✅ |
| Molecule Evolution | Mutate 1/5 chiều → loài mới | Hoạt động trong test | **Tốt** |
| Áp dụng thực tế | "Đủ để định vị BẤT KỲ khái niệm" | Chưa chứng minh với khái niệm phức tạp | **Chưa rõ** |

---

## V. Điểm Mạnh Thật Sự (Người Dùng Chưa Thấy)

Đây là những thứ **rất tốt** nhưng bị che bởi lớp output nghèo nàn:

| Điểm mạnh | Chi tiết | Tại sao người dùng chưa thấy |
|-----------|---------|------------------------------|
| **Zero dependencies** | SHA-256, Ed25519, AES-256-GCM tự viết | Người dùng không biết/quan tâm crypto engine |
| **Privacy tuyệt đối** | Không gửi data ra ngoài | Tốt nhưng vô hình — cần communicate |
| **Emotion detection chính xác** | V/A scores đúng, ConversationCurve thay đổi đúng | Bị chặn bởi response template |
| **VM hoạt động** | 31 opcodes, math, solve equations | Chỉ ai biết ○{} mới thấy |
| **Kiến trúc DAG sạch** | Không circular deps, append-only | Kỹ thuật tốt nhưng invisible |
| **Silk graph** | Hebbian learning, emotion per edge | Tích lũy đúng nhưng không ảnh hưởng output |
| **Code quality** | 0 clippy warnings, 0 unsafe | Người dùng không check clippy |

---

## VI. Người Dùng Sẽ Hỏi Gì?

| Câu hỏi người dùng | Câu trả lời thật |
|---------------------|------------------|
| "HomeOS khác Google Home / Alexa sao?" | Hiện tại: kém hơn rất nhiều về output. Nhưng: zero cloud, zero subscription, zero data collection |
| "Nó có hiểu tôi không?" | Nó **đo** cảm xúc chính xác. Nhưng nó **không thể hiện** là hiểu |
| "Nó có nhớ không?" | Ngắn hạn: có (trong phiên). Dài hạn: không (Dream chưa hoạt động) |
| "Nó điều khiển nhà được không?" | Hiện tại: không. Kiến trúc sẵn sàng nhưng chưa kết nối thiết bị |
| "Tôi nên dùng nó vào việc gì?" | Hiện tại: ○{} commands (toán, compose concepts, dream, stats). Conversation thì chưa |
| "Khi nào nó dùng được?" | Nền tảng 80% xong. "Mặt tiền" (response, agent, memory) cần nhiều việc |
| "Dữ liệu của tôi an toàn không?" | Rất an toàn: local-only, append-only, Ed25519 signed, AES-256-GCM |
| "Tại sao nó chỉ nói tiếng Việt?" | Response template chỉ viết bằng tiếng Việt. Aliases có 7 ngôn ngữ nhưng output thì không |

---

## VII. Bản Đồ Ưu Tiên: Sửa Gì Trước?

### Tier 1 — Sửa ngay (người dùng đang "đau")

| # | Vấn đề | Tại sao | Effort |
|---|--------|---------|--------|
| 1 | **Response template phản ánh nội dung** | Input "mèo" → response nhắc "mèo" | Trung bình |
| 2 | **Parser thêm typeof/explain/why/trace** | Bug đơn giản, 6 commands thiếu trong is_command() | Nhỏ |
| 3 | **Instinct output → response** | Causality/Abstraction kết quả cần surface vào text | Trung bình |

### Tier 2 — Sửa sớm (hệ thống chưa "sống")

| # | Vấn đề | Tại sao | Effort |
|---|--------|---------|--------|
| 4 | **Dream threshold giảm** | Fibonacci threshold quá cao cho real conversation | Nhỏ |
| 5 | **SystemManifest đọc NodeKind** | 82% nodes unclassified | Nhỏ |
| 6 | **Multi-language response** | Detect language → response cùng ngôn ngữ | Trung bình |

### Tier 3 — Cần kế hoạch (thiếu cả "đường ống")

| # | Vấn đề | Tại sao | Effort |
|---|--------|---------|--------|
| 7 | **Agent loop có trigger thật** | Workers cần hardware events | Lớn |
| 8 | **WASM browser demo** | Cần giao diện để người dùng thử | Lớn |
| 9 | **HAL kết nối thiết bị thật** | HomeOS cần "Home" | Rất lớn |

---

## VIII. Nhận Xét Thẳng Thắn

### Cái hay:
Đây là dự án **cực kỳ tham vọng** và nền tảng kỹ thuật **rất chắc**. Người tạo ra nó hiểu rõ kiến trúc hệ thống, có tư duy toán học sâu, và kiên nhẫn xây từng viên gạch (zero deps, append-only, 5D encoding). Rất ít dự án cá nhân đạt mức 65K dòng Rust, 1500+ tests, 0 clippy warnings.

### Cái thiếu:
Dự án build **từ dưới lên** (bottom-up) cực kỳ kỹ — nhưng quên rằng người dùng **nhìn từ trên xuống** (top-down). Họ không thấy Molecule encoding hay LCA. Họ thấy: "tôi nói gì, nó trả lời gì." Và ở đó, HomeOS **chưa có gì đáng kể**.

### Phép ẩn dụ:
> HomeOS giống một cỗ máy với **động cơ Ferrari** nhưng **chưa có vô-lăng và ghế ngồi**.
> Kỹ sư nhìn vào: "Tuyệt vời!"
> Người dùng nhìn vào: "Làm sao tôi lái?"

### Kết:
Không thiếu năng lực. Thiếu **cầu nối** giữa engine và người dùng. Response template là cầu nối đó — và nó đang là mắt xích yếu nhất của toàn bộ hệ thống.

---

## IX. Tự Đánh Giá Của Dự Án (từ old docs)

Dự án tự chấm điểm trong REVIEW_2026_03_17.md:

```
Foundation:      100% ✅   ← Molecule, Chain, Registry, Writer/Reader
Life mechanisms:  70%      ← VM hoạt động, Graph Walk chưa xong
Intelligence:     40%      ← Instincts có, Orchestration chưa
Knowledge:        10%      ← 35 L0 nodes, chưa có domain thật
Perception:        0%      ← Không có sensor thật
Self-awareness:    0%      ← why/explain không hoạt động
```

**Lời tự nhận:** *"HomeOS has the right DNA. The genome is complete. Cell structure is complete. But it hasn't breathed yet. Hasn't opened its eyes. Hasn't spoken its first sentence."*

### Vấn đề kỹ thuật sâu: 98% Molecule Collision

Tài liệu cũ phát hiện vấn đề nghiêm trọng: từ 5,279 UCD entries, chỉ ~100 unique molecules — **98.1% collision**. 4,389 entries (83%) rơi vào cùng bucket `(Sphere, Member)`. Nguyên nhân: 8-variant enum chỉ phân biệt được 8 giá trị/chiều.

Giải pháp (đang triển khai):
- **Tagged sparse encoding (v0.05)** — tiết kiệm 47% storage ✅ đã xong
- **Hierarchical byte encoding** — `base(1-8) + sub_index*8` → ~5,400 patterns phân biệt ✅ đã xong

**Đánh giá:** Vấn đề collision đã được nhận diện và sửa — đây là ví dụ tốt về tự sửa chữa. Nhưng các vấn đề user-facing (response, agent, memory) thì không được sửa tương tự.

---

## X. Phân Tích Từ Tài Liệu Cũ (old/2026-03-17/)

Dự án có **13 file tài liệu cũ** — kiến trúc, roadmap, nhiều bản review, nhật ký phiên. Phân tích cho thấy:

### A. Vấn đề được nhận ra từ LÂU nhưng chưa sửa

| Vấn đề | Được nhắc trong | Đã sửa chưa? |
|--------|----------------|---------------|
| Response template quá nghèo (~10 câu) | REVIEW.md, REVIEW_VI.md, REVIEW_2026_03_17.md, TRUTH.md | **Chưa** |
| Parser thiếu typeof/explain/why | REVIEW.md, HomeOS_Review.md | **Chưa** |
| 82% nodes unclassified | REVIEW_2026_03_17.md, HomeOS_Review.md | **Chưa** |
| Agent system = dead code | TRUTH.md, REVIEW.md, HomeOS_Review.md | **Chưa** |
| Dream 0 clusters | REVIEW.md, TRUTH.md | **Chưa** |

**Nhận xét:** Dự án tự biết điểm yếu rất rõ — nhiều bản review cực kỳ trung thực. Nhưng sau nhiều phiên (A→J), các vấn đề cốt lõi **vẫn nguyên vẹn**. Nguyên nhân: mỗi phiên AI mới lại ưu tiên build thêm tính năng mới (SkillPattern, Multilingual Seeding...) thay vì sửa mặt tiền.

### B. Tầm nhìn trong tài liệu cũ vs thực tế

**TRUTH.md** — file trung thực nhất — thừa nhận thẳng:
- "Đây không phải sản phẩm — đây là prototype nghiên cứu"
- "Không có UI — chỉ terminal REPL"
- "Không có hardware thật — HAL detect platform nhưng không kết nối thiết bị"
- "Response nghèo nàn — template-based"

**Roadmap** hứa 12+ phases, hầu hết đánh dấu "complete" nhưng review cho thấy "complete" = "code biên dịch được", không phải "hoạt động có ý nghĩa cho người dùng".

### C. Mẫu lặp lại qua mọi tài liệu

| Chủ đề | Tần suất xuất hiện | Ý nghĩa |
|--------|-------------------|---------|
| "Code exists ≠ Code works" | 5/13 files | Nhận thức chung, nhưng không dẫn đến hành động |
| "Bottom-up tốt, top-down thiếu" | 7/13 files | Ẩn dụ "động cơ Ferrari, chưa có vô-lăng" |
| "Response template là bottleneck" | 6/13 files | Vấn đề #1 được đồng thuận |
| "Privacy là sức mạnh thật" | 4/13 files | Điểm mạnh duy nhất mà user có thể cảm nhận |
| "Fibonacci quá cao cho thực tế" | 3/13 files | Threshold toán học đẹp nhưng usage pattern không đủ |
| "Sinh linh" là tham vọng | 4/13 files | Hiện tại giống "sách giải phẫu" hơn "sinh vật sống" |

### D. Nhật ký phiên D→E: Bài học về quy trình

Phiên D kết thúc đột ngột (có thể do context overflow). Phiên E phải reconstruct lại. Điều này cho thấy:
- Quy trình phát triển **dễ vỡ** khi chuyển phiên
- NEXT_PLAN.md là "ký ức" duy nhất giữa các phiên — nhưng nó **lạc quan hơn thực tế** (ghi "complete" khi chỉ code-complete)
- Mỗi phiên AI mới có xu hướng **thêm mới** thay vì **sửa cũ** — vì thêm mới dễ đo lường (test count tăng) còn sửa cũ khó thấy

---

## XI. Kết Luận Cuối

### Dự án này là gì?
Một **nền tảng nghiên cứu** (research prototype) về biểu diễn tri thức dựa trên Unicode 5D, với tham vọng trở thành hệ điều hành nhà thông minh.

### Dự án này KHÔNG phải là gì?
Một sản phẩm mà người dùng cuối có thể sử dụng ngay. Chưa có giao diện, chưa có response có ý nghĩa, chưa điều khiển được thiết bị.

### Điểm mạnh thật sự (kỹ thuật):
- Molecule 5D encoding — **độc đáo và hoạt động**
- Zero external dependencies — **ấn tượng với 65K LoC Rust**
- Kiến trúc DAG sạch — **không circular, append-only đúng**
- 1500+ tests, 0 warnings — **kỷ luật kỹ thuật cao**
- Tự nhận thức (self-awareness) — **biết mình thiếu gì** qua nhiều bản review

### Điểm yếu thật sự (trải nghiệm):
- **Output** — 10 câu template cho mọi input
- **Memory** — không nhớ gì qua phiên
- **Home** — không điều khiển được gì
- **Sống** — agent hierarchy là code chết
- **Lặp lại** — vấn đề được nhận ra từ phiên A, phiên K vẫn chưa sửa

### Một câu:
> **HomeOS có bộ não rất tinh vi nhưng chưa có miệng để nói, tay để làm, và mắt để nhìn.**

---

*2026-03-17 · Đánh giá góc nhìn người dùng (bao gồm 13 tài liệu cũ) · Phiên K*
