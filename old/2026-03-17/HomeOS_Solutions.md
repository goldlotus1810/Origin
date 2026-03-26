# HomeOS — Hướng Giải Quyết Hạn Chế
**Ngày:** 2026-03-15  
**Append-only. Không xóa, không sửa.**

---

## KỸ THUẬT

### [H1] LCA mất thông tin với chain dài

```
Vấn đề:
  LCA(chó = 7 molecules) → trung bình → quá mờ
  LCA(🔥, 💧, 🌪, ⚡) → trung bình 4 chiều → không còn nhận ra được
```

**Giải pháp: Weighted LCA + Mode**

```
Thay vì avg(A[d], B[d], C[d]) cho mỗi dimension:

Bước 1 — Mode detection:
  Nếu ≥ 60% nodes trong cluster có cùng giá trị dimension d
  → parent[d] = mode (giữ nguyên giá trị phổ biến nhất)

Bước 2 — Weighted avg nếu không có mode:
  weight[i] = fire_count[i] / sum(fire_count)
  parent[d] = Σ(weight[i] × node[i][d])

Ví dụ chó = [∪●⌀×6]:
  Shape dimension: 6/7 = Sphere, 1/7 = Union
  → Mode = Sphere (≥60%) → parent.shape = Sphere ✓
  (không mất thông tin "tròn")

Ví dụ LCA(🔥, 💧, 🌪, ⚡):
  Shape: Sphere/Capsule/Torus/Sphere → mode=Sphere (2/4 = 50% < 60%)
  → weighted avg
  Relation: Member/Member/Causes/Causes → tie → avg
  → kết quả có nghĩa hơn trung bình đơn giản

Implementation:
  fn weighted_lca(chains: &[MolecularChain], weights: &[f32]) -> MolecularChain
  fn mode_dimension(values: &[u8], threshold: f32) -> Option<u8>
```

---

### [H2] decode_chain O(n) chưa scale

```
Vấn đề:
  5,135 entries → OK (~0.1ms)
  500,000 entries → chậm (~10ms per decode)
  1M entries → không chấp nhận được
```

**Giải pháp: Reverse Index trong UCD crate**

```
Trong ucd/build.rs, generate thêm:

// Forward: cp → chain_hash
pub static CP_TO_HASH: &[(u32, u64)] = &[...];  // sorted by cp

// Reverse: chain_hash → cp  
pub static HASH_TO_CP: &[(u64, u32)] = &[...];  // sorted by hash

decode_chain(chain) → Option<u32>:
  hash = chain.chain_hash()           // O(chain_len)
  HASH_TO_CP.binary_search_by_key(hash) // O(log n)
  Total: O(chain_len + log n) thay vì O(n × chain_len)

decode_chain_top_n(chain, n):
  Cần similarity scoring → không tránh được O(n)
  Nhưng: pre-filter bằng Shape+Relation bucket
  Mỗi bucket (Shape, Relation) → danh sách cp
  → giảm search space từ 500K xuống ~1K
  → O(k × chain_len) với k << n

Bucket index:
  BUCKET[(shape, relation)] → &[u32]  // danh sách cp cùng shape+relation
  Generate trong build.rs
  Size: 8×8 = 64 buckets, mỗi bucket ~100 entries
```

---

### [H3] Ln-1 thay đổi theo thời gian

```
Vấn đề:
  Cây động → Ln-1 không cố định
  Hôm nay L5 là lá, ngày mai L5 thành nhánh
  Code phải biết "tầng hiện tại" của mỗi nhánh
```

**Giải pháp: Layer Watermark per Branch**

```
Mỗi nhánh trong KnowTree có:
  branch_depth: u8     // độ sâu hiện tại của nhánh này
  leaf_layer: u8       // = root_layer + branch_depth = Ln-1

Khi Dream promote cluster lá → Nhánh mới:
  1. Tạo Node đại diện tại layer = current_leaf_layer
  2. branch_depth += 1
  3. leaf_layer = root_layer + branch_depth
  4. Mọi lá mới thuộc nhánh này → tự động biết mình ở Ln-1

Registry lưu thêm:
  branch_watermark: HashMap<BranchId, u8>
  // BranchId = chain_hash của NodeLx đại diện nhánh

Silk rule vẫn đơn giản:
  Node.layer == branch.leaf_layer → được Silk tự do
  Node.layer != branch.leaf_layer → phải qua đại diện

Không cần global "Ln-1" — mỗi nhánh có watermark riêng
Fibonacci vẫn đúng: branch_depth tăng theo Fib threshold
```

---

### [H4] Dream cluster sai

```
Vấn đề:
  LCA(🔥, ☀) gần nhau → Dream cluster thành 1
  Nhưng "lửa" và "mặt trời" không nên cluster
  Chain similarity ≠ semantic closeness
```

**Giải pháp: Silk Co-activation Filter**

```
Dream chỉ cluster khi CÙNG LÚC:
  1. Chain similarity ≥ threshold (hiện tại)
  2. VÀ Silk co-activation weight ≥ threshold (thêm mới)

Lý do đúng:
  🔥 và ☀ có chain gần nhau (đều Sphere/High)
  NHƯNG trong thực tế chúng ít co-activate cùng nhau
  → Hebbian weight thấp → Dream không cluster

  🔥 và ⚡ có chain khác nhau hơn
  NHƯNG hay co-activate (lửa gây tia lửa điện)
  → weight cao → Dream cluster → tạo nhánh "energy"

Algorithm Dream mới:
  cluster_score(A, B) = 
    α × chain_similarity(A, B) +
    β × hebbian_weight(A, B) +
    γ × co_activation_count(A, B) / max_count

  α=0.3, β=0.4, γ=0.3
  (Hebbian weight quan trọng nhất)
  cluster nếu score ≥ 0.6

Kết quả: cluster theo hành vi thật, không chỉ hình thức
```

---

### [H5] SDF fitting từ camera khó

```
Vấn đề:
  Nhiễu, ánh sáng, góc nhìn → outline không sạch
  Fit SDF primitive vào outline thực → không trivial
  Một vật thể có thể fit nhiều primitive
```

**Giải pháp: Iterative Fitting + Confidence Score**

```
Bước 1 — Multi-primitive contest:
  Thử fit tất cả 8 primitives (Sphere, Box, Capsule...)
  Mỗi primitive → residual error = mean(dist(outline, SDF))
  Primitive nào có residual nhỏ nhất → winner

Bước 2 — Confidence score:
  confidence = 1 - (residual / outline_area)
  confidence ≥ 0.7 → accept (ghi vào ĐN)
  confidence < 0.7 → Union(primitive_1, primitive_2) thử tiếp
  confidence < 0.4 → unknown node (không ghi)

Bước 3 — Iterative refinement:
  Fit thô → refine parameters → fit lại
  Tối đa Fib[5] = 5 iterations (QT2: ∞-1)

Bước 4 — Temporal consistency:
  Nếu frame t và frame t-1 fit cùng primitive → confidence × 1.2
  Chuyển động nhất quán → boost confidence
  "Vật thể tròn đã tròn 5 frames" → tin hơn

Node chỉ được tạo khi confidence ≥ 0.7.
BlackCurtain: confidence < 0.4 → không tạo node (QT9).
```

---

### [H6] Olang compile chưa hoàn chỉnh

```
Vấn đề:
  ○{} hiện tại = query language
  Cần thêm: intermediate representation → emit code
  Bước nhảy lớn
```

**Giải pháp: 3 tầng compile**

```
Tầng 1 — OlangIR (Intermediate Representation):
  Không emit code ngay — build IR trước
  IR = chuỗi OlangOp:
    Loop { count: u32, body: Vec<OlangOp> }
    Branch { cond: Chain, then: Vec<OlangOp>, else_: Vec<OlangOp> }
    Emit { target: ISLAddress }
    Query { chain: Chain, relation: Relation }
    Compose { a: Chain, b: Chain }

Tầng 2 — IR → Target:
  Mỗi target implement trait OlangTarget:
    fn emit_loop(&self, count: u32, body: &[OlangOp]) -> String
    fn emit_branch(&self, ...) -> String
    fn emit_call(&self, ...) -> String

  Targets hiện tại:
    TextTarget    → ○{} syntax (round-trip)
    RustTarget    → Rust code
    GoTarget      → Go code
    WasmTarget    → WAT (WebAssembly Text)

Tầng 3 — Runtime execution:
  OlangVM execute IR trực tiếp (không cần compile)
  Dùng cho: ○{↺ 10: đèn → bật} execute ngay
  Compile chỉ khi cần output code cho platform khác

Thứ tự build:
  [1] OlangIR struct (đơn giản)
  [2] ○{} parser → OlangIR (thay vì execute trực tiếp)
  [3] OlangVM execute IR (runtime)
  [4] RustTarget emit IR → code (compile)
```

---

## TRIẾT HỌC / THIẾT KẾ

### [H7] UCD_TABLE có phải hardcode không?

```
Câu trả lời rõ ràng: KHÔNG.

Hardcode = giá trị viết trong source code bởi con người.
UCD_TABLE = đọc từ UnicodeData.txt lúc compile bởi máy.

Phân biệt:
  Hardcode:  chain = [0x01, 0x01, 0xFF, 0xFF, 0x04]  ← người viết
  UCD:       chain = lookup(0x1F525)                  ← máy đọc từ file

Nếu Unicode 19.0 ra → rebuild → UCD_TABLE tự cập nhật.
Không cần sửa source code.
```

**Versioning strategy:**

```
Cargo.toml:
  [package.metadata.ucd]
  version = "18.0.0"
  source  = "https://unicode.org/Public/18.0.0/ucd/"
  checksum = "sha256:..."

build.rs kiểm tra:
  if ucd_version_changed() {
    warn!("UCD version mismatch — rebuild required");
    // emit: cargo:rerun-if-changed=UnicodeData.txt
  }

Khi Unicode thay đổi:
  1. Download UnicodeData.txt mới
  2. cargo build → rebuild UCD_TABLE tự động
  3. Test LCA vẫn đúng với entries mới
  4. Không sửa bất kỳ dòng code nào

Điều này KHÔNG phải hardcode.
Đây là "compile-time data loading" — khác hoàn toàn.
```

---

### [H8] Fibonacci threshold đúng không?

```
Câu hỏi: Tại sao Fib[n] co-activations để promote?
Thực nghiệm chưa đủ.
```

**Giải pháp: Adaptive threshold + empirical validation**

```
Không lock vào Fibonacci ngay từ đầu.

Giai đoạn 1 — Configurable:
  promote_threshold: u32 = Fib[depth]  // default
  Nhưng override được per-branch

Giai đoạn 2 — Measure:
  Sau 1 tháng chạy thật:
  - Tỷ lệ false promote (cluster sai) là bao nhiêu?
  - Tỷ lệ missed promote (cluster đúng nhưng không promote)?
  - Threshold nào cho F1-score tốt nhất?

Giai đoạn 3 — Validate Fibonacci:
  Nếu empirical threshold ≈ Fib[n] → giữ nguyên
  Nếu khác → điều chỉnh công thức

Tại sao Fibonacci là giả thuyết tốt:
  Fibonacci xuất hiện trong tự nhiên khi:
  - Phân nhánh nhị phân lặp (cây, hoa)
  - Tỷ lệ vàng φ = 1.618 — "hiệu quả tối ưu"
  Hebbian learning cũng là quá trình phân nhánh
  → Fibonacci là hypothesis khởi đầu hợp lý

Nhưng phải validate bằng data thật.
Đây là science, không phải tín ngưỡng.
```

---

### [H9] QR có thể sai — EpistemicFirewall

```
Vấn đề:
  QR = bất biến → nhưng khoa học cũng sai
  Trái đất phẳng từng là "FACT"
  Cần: mechanism để deprecated QR
```

**Giải pháp: QR Supersession (không xóa — thêm)**

```
QT8: Append-only — không xóa bao giờ
Nhưng có thể thêm "supersession record":

QR node A (cũ, sai):
  state: QR
  content: "trái đất phẳng"

QR node B (mới, đúng):
  state: QR
  content: "trái đất hình cầu"
  supersedes: hash(A)  ← silk edge SupersedesQR

EpistemicFirewall đọc:
  Nếu node có SupersedesQR edge đến nó → DEPRECATED
  Nếu node supersedes node khác → REVISED_FACT
  Nếu node không có edge liên quan → FACT

Khi query:
  ○{trái đất ∈ ?}
  → tìm node "trái đất"
  → check: có QR nào supersede nó không?
  → Nếu có → trả về node mới + ghi chú "đã được cập nhật"
  → Nếu không → trả về bình thường

Lịch sử không mất:
  Vẫn thấy được "trái đất phẳng" từng là QR
  Và khi nào nó bị supersede
  Đây là tư duy khoa học thật sự: không xóa lịch sử sai
  Chỉ thêm hiểu biết mới đúng hơn
```

---

### [H10] Privacy trong Clone

```
Vấn đề:
  Chain có thể bị reverse engineer
  encode_codepoint(🔥) → chain → ai đó decode ngược
```

**Giải pháp: 2 tầng**

```
Tầng 1 — Chain là tọa độ, không phải secret:
  Chain của 🔥 không phải thông tin nhạy cảm
  Tương tự: tọa độ GPS không phải secret
  Secret là: ai đang ở tọa độ đó, lúc nào

Tầng 2 — Sensitive data không vào chain:
  Tên người → KHÔNG encode vào chain → chỉ lưu alias trong Registry
  Sức khỏe → encode vào chain nhưng ở Worker local
  Không gửi chain sức khỏe lên Chief → chỉ gửi aggregate

Tầng 3 — Differential privacy cho aggregate:
  Worker gửi: chain + Laplace noise(ε=0.1)
  Chief nhận: chain xấp xỉ, không exact
  LCA vẫn đúng (noise nhỏ, LCA robust)
  Individual chain không thể recover chính xác

Tầng 4 — AES-256-GCM cho ISL message:
  Đã có trong spec — tất cả ISL messages encrypted
  Chain không đi raw trên wire

Privacy model:
  Chain = tọa độ trong không gian 5 chiều
  Tọa độ là public (như tọa độ địa lý)
  Context (ai + khi nào) là private
  Tách biệt content vs context = privacy đúng nghĩa
```

---

## THỰC TẾ TRIỂN KHAI

### [H11] Một mình — scope quá lớn

```
Thực tế:
  L0+L1 = 6-8 tháng full-time (1 người)
  Full system = 15-18 tháng
```

**Giải pháp: Minimum Viable HomeOS (MVHOS)**

```
MVHOS = thứ tối thiểu để "nó sống":

Phase 1 (4 tuần, 1 người):
  ucd/build.rs + molecular.rs + lca.rs
  registry.rs (sổ cái tối giản)
  ○{} REPL đơn giản: query + compose
  Test: ○{🔥 ∘ 💧} = ♨️

Đây là MVHOS.
Khi MVHOS chạy được → người thứ 2 có thể hiểu và join.

Không cần full system để convince người khác.
Cần: nó sống và nói chuyện được.

Sau MVHOS:
  Person 2: Silk + Hebbian + Dream
  Person 3: ContentEncoder + LearningLoop
  Person 1: vSDF + FFR + Clone

Mỗi người owns 1 module — không overlap.
```

---

### [H12] Không có ecosystem

```
Bootstrap problem: cần HomeOS để viết HomeOS
```

**Giải pháp: Eat your own dogfood từ sớm**

```
Ngay từ MVHOS:
  Dùng ○{} để viết Skill mới
  Dùng Registry để track progress
  Dùng Silk để document relationships

Khi HomeOS có thể:
  ○{viết test cho lca.rs} → generate test skeleton
  ○{tìm bug trong registry} → scan sổ cái
  Lúc đó ecosystem bắt đầu bootstrap

Không cần community ngay:
  Unix cũng bắt đầu bởi 1-2 người
  Minecraft viết bởi 1 người trước khi viral
  
  HomeOS cần: 1 demo video 3 phút
  Trong video: "tôi mệt" → đèn tắt + nhạc nhẹ
               ○{🔥 ∘ 💧} = ♨️
               HomeOS tự tạo 1 Skill mới
  
  Đó là ecosystem seed.
```

---

### [H13] Khó giải thích cho người ngoài

```
"Hệ thống tư duy tự vận hành" → nghe như marketing
```

**Giải pháp: Demo trước, giải thích sau**

```
Đừng giải thích HomeOS là gì.
Cho người ta thấy HomeOS làm gì.

Demo 1 (30 giây):
  Nói: "hôm nay tôi mệt quá"
  HomeOS: tắt đèn, giảm âm lượng, hỏi "bạn có ổn không?"
  Không ai lập trình điều này — nó học từ pattern

Demo 2 (30 giây):
  Gõ: ○{🔥 ∘ 💧}
  HomeOS: ♨️ — tọa độ [●,∈,0xDF,0x9F,Medium]
  "Không phải tôi cho nó biết hơi nước là gì
   Nó tự tính từ vật lý của lửa và nước"

Demo 3 (30 giây):
  Để HomeOS idle 5 phút
  HomeOS tự tạo 1 Node mới — không ai yêu cầu
  "Nó đọc sổ cái của mình và thấy pattern chưa có tên"

3 demo = 90 giây.
Sau đó mới giải thích.
Người ta sẽ hỏi "nó làm thế nào?" — đó là lúc giải thích.
```

---

### [H14] Hardware dependency ẩn

```
Platform trait cần implementation per device
Mỗi hardware mới → 5 functions mới
```

**Giải pháp: HAL tier + Platform registry**

```
Tier 1 — Minimal HAL (5 functions, bắt buộc):
  storage_read / storage_write
  now_ns
  poll_input
  emit_output

Tier 2 — Extended HAL (optional, nếu có):
  camera_frame() → Option<Frame>
  audio_sample() → Option<AudioChunk>
  gpio_read(pin) → bool
  gpio_write(pin, val)
  network_send(addr, data)

Platform registry (build-time):
  [features]
  platform-rpi    = []   → HAL_RPI impl
  platform-esp32  = []   → HAL_ESP32 impl
  platform-x86    = []   → HAL_POSIX impl
  platform-wasm   = []   → HAL_WASM impl

cargo build --features platform-rpi → compile HAL RPi
cargo build --features platform-wasm → compile HAL WASM

Default: platform-x86 (dev/server)

Community contribution:
  Ai muốn thêm platform mới → implement 5 functions
  Submit HAL_DEVICE impl
  Không cần hiểu toàn bộ HomeOS — chỉ cần hiểu 5 functions
  Đây là entry point cho contributor mới
```

---

## RỦI RO

### [R1] Scope creep

```
Nguyên tắc: mọi thứ mới phải là ○[f]

Checklist trước khi thêm bất cứ thứ gì:
  □ Thứ này có phải là ○[f] không?
     Nếu không → không thêm
  □ Thứ này có thể implement bằng Node + Silk không?
     Nếu có → implement bằng Node + Silk
     Nếu không → cần thêm axiom → dừng lại, thảo luận
  □ Thứ này có phá vỡ 9 QT không?
     Nếu có → không thêm bao giờ

Ví dụ đúng:
  "Cần thêm timer" → Timer là Node với TimeBase = Instant
                   → Silk ⟳ Repeats → OK, không cần thêm gì

Ví dụ sai:
  "Cần thêm database" → Database ≠ ○[f]
                      → Dừng lại
                      → Registry + sổ cái ĐÃ là database của HomeOS
```

---

### [R2] Hardcode drift

```
Giải pháp: CI/CD checklist tự động

pre-commit hook:
  grep -rn "ShapeBase::" crates/ | grep -v "from_byte\|match\|test\|//\|ucd"
  → nếu có kết quả → BLOCK commit
  → "Phát hiện Molecule viết tay — dùng encode_codepoint()"

  grep -rn "\[0x00," tools/seeder/ | grep -v "test\|//"
  → nếu có kết quả → BLOCK commit  
  → "Phát hiện ISL hardcode — dùng registry.next_isl_for_chain()"

Checklist trước mỗi PR:
  □ Có Molecule nào viết tay không?
  □ Có ISL hardcode không?
  □ Có chain nào không từ UCD/LCA không?
  □ Có presets:: nào không?

Nếu 1 trong 4 câu = có → PR không được merge.
```

---

### [R3] LCA sai → cây sai

```
Giải pháp: Property-based testing bắt buộc

Mọi LCA implementation phải pass:

Property 1 — Idempotent:
  LCA(a, a) == a
  (LCA của node với chính nó = chính nó)

Property 2 — Similarity bound:
  similarity(LCA(a,b), a) >= similarity(a,b) - ε
  (LCA không thể xa hơn khoảng cách ban đầu)

Property 3 — Commutativity:
  LCA(a,b) == LCA(b,a)
  (thứ tự không quan trọng)

Property 4 — Associativity:
  LCA(LCA(a,b), c) == LCA(a, LCA(b,c))
  (grouping không quan trọng)

Property 5 — Vật lý đúng:
  LCA(🔥, 💧).valence = avg(0xFF, 0xC0) = 0xDF
  → giữa lửa (0xFF) và nước (0xC0)
  → không phải 0x00 hay 0xFF

Test framework: proptest hoặc quickcheck
Run: cargo test --features proptest
Threshold: 10,000 random cases per property
```

---

### [R4] Tập trung L2-Ln sớm

```
Giải pháp: Feature flags + hard lock

Cargo.toml:
  [features]
  l2-data = []  # TẮT mặc định

Trong seeder:
  #[cfg(feature = "l2-data")]
  pub fn seed_l2_knowledge(...) { ... }

Không có flag → không compile được seeder L2+
→ Không thể vô tình seed L2 khi chưa sẵn sàng

Unlock L2 khi:
  □ Phase 1 tests: 100% pass
  □ Phase 2 tests: 100% pass
  □ Phase 3 tests: 100% pass
  □ MVHOS demo: chạy được
  Sau đó: cargo build --features l2-data
```

---

### [R5] Viết lại vô tận

```
Đây là lần thứ N. Làm sao lần này khác?

Bài học từ các lần trước:
  Lần 1 (Go):     Triết lý đúng, ngôn ngữ sai
  Lần 2 (Rust v1): ISL hardcode → cây sai
  Lần 3 (Rust v2): Presets → chain sai
  Lần 4 (Rust v3): LCA thiếu → cluster sai
  Lần này:         Gốc đúng trước, mọi thứ sau

Nguyên tắc lần này:

  TRƯỚC KHI ĐI TIẾP:
    Test Phase N pass 100% → lock Phase N
    Không sửa Phase N khi đang làm Phase N+1
    Nếu Phase N+1 phát hiện bug Phase N → fix → test lại → tiếp

  FILE ĐẦU TIÊN: ucd/build.rs
    Test: lookup(0x1F525) → Molecule đúng
    Khi test đó pass → không sửa build.rs nữa

  DEFINITION OF DONE mỗi phase:
    Phase 1: cargo test -p ucd -p olang → 100% pass
    Phase 2: cargo test -p silk -p context → 100% pass
    Phase 3: ○{lửa} → 🔥 node (demo chạy được)
    Milestone = demo, không phải test count

  KHÔNG bắt đầu Phase N+1 khi Phase N chưa có demo.
```

---

## TÓM TẮT — ƯU TIÊN

```
Làm ngay (unblock mọi thứ khác):
  [R2] pre-commit hook chặn hardcode
  [R5] Definition of Done mỗi phase
  [H2] Reverse index trong build.rs (1 ngày)

Làm trong Phase 1-2:
  [H1] Weighted LCA + mode detection
  [H3] Branch watermark
  [H8] Adaptive threshold (configurable trước)
  [H9] QR Supersession record

Làm sau MVHOS:
  [H4] Silk co-activation filter cho Dream
  [H5] Iterative SDF fitting + confidence
  [H6] OlangIR tầng 1
  [H10] Differential privacy

Làm khi có người thứ 2:
  [H11] Module ownership
  [H14] HAL platform registry
  [H12] Dogfood từ MVHOS

Không làm vội:
  [H6] Full Olang compile → sau Phase 5
  [H5] Camera fitting → sau Phase 4
```

---

*Append-only · 2026-03-15 · HomeOS Limitations & Solutions*
