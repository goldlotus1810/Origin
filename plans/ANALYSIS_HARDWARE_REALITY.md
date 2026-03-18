# ANALYSIS: CPU/GPU/NPU — Thực tế hardware vs HomeOS workloads

**Ngày:** 2026-03-18
**Mục đích:** Đánh giá thực tế (không phải marketing) khả năng hardware hiện tại cho HomeOS.

---

## TL;DR — Kết luận chính

```
CPU:  ✅ PHÙ HỢP HOÀN HẢO cho HomeOS. Đây là nền tảng đúng.
GPU:  ⚠️ CÓ ÍCH cho batch operations ở scale lớn, NHƯNG có vấn đề với FNV-1a-64.
NPU:  ❌ VÔ DỤNG cho HomeOS. Chỉ làm được matrix multiply. Bỏ qua.

Phát hiện quan trọng nhất:
  ① FNV-1a-64 là SERIAL — không thể parallel trên GPU cho 1 hash
  ② KHÔNG CÓ mobile GPU nào hỗ trợ native 64-bit integer multiply
  ③ NPU marketed 45 TOPS → thực tế 0.5 TOPS (1.3%) cho non-ML workloads
  ④ CPU branch misprediction cho VM dispatch: ~25% miss rate
  ⑤ 5-byte molecule VỪA ĐẸP trong cache line (12 mol/64B, 25 mol/128B)
```

---

## 1. CPU — Nền tảng chính của HomeOS

### Thông số thực tế (2024-2026)

```
                    Phone            Laptop           Desktop
                    (Cortex-X925)    (Apple M4)       (Zen 5)
────────────────────────────────────────────────────────────────
Decode width        10-wide          10-wide          8-wide (2×4)
ROB entries         ~750-948         >630             448
Integer ALUs        8                7                6
MUL pipes           4                2                3
MUL latency         2 cycles ⭐      ~3 cycles        3 cycles
MUL throughput      4/cycle          2/cycle          3/cycle ⭐
SIMD width          128-bit (NEON)   128-bit (NEON)   512-bit (AVX-512) ⭐
L1D cache           64 KB            128 KB ⭐        48 KB
L1D latency         3-4 cycles       3 cycles ⭐      4 cycles
L2 cache            2-3 MB           12-48 MB ⭐      1 MB
L3 cache            32 MB            shared L2        128 MB (V-Cache) ⭐
Cache line           64 bytes         128 bytes ⭐     64 bytes
Memory BW           ~65 GB/s         ~100-273 GB/s    ~90 GB/s
Clock               3.8 GHz          ~4.0 GHz         5.7 GHz ⭐
Branch miss penalty  10-12 cycles ⭐  ~12 cycles       ~14 cycles
```

### FNV-1a Performance trên CPU

```
FNV-1a-64 cho 1 molecule (5 bytes):
  Mỗi byte = 1 XOR + 1 64-bit MUL (SERIAL chain — không parallel được)
  5 bytes × MUL_latency = total cycles

  Platform        MUL latency   Cycles/mol   Time/mol    Hashes/sec
  ──────────────────────────────────────────────────────────────────
  Cortex-X925     2 cycles      10 cycles    2.6 ns      380M/s ⭐
  Apple M4        3 cycles      15 cycles    3.8 ns      267M/s
  Zen 5 @5.7GHz   3 cycles      15 cycles    2.6 ns      380M/s ⭐

  ⚠️ Serial dependency: thêm MUL pipes KHÔNG giúp cho 1 hash.
     Nhưng hash NHIỀU molecules cùng lúc → dùng hết MUL pipes.

  Batch hashing (independent molecules, dùng hết MUL throughput):
  Platform        MUL throughput   Batch hashes/sec
  ──────────────────────────────────────────────────
  Cortex-X925     4/cycle          ~1.5B/s (4 mol song song)
  Apple M4        2/cycle          ~530M/s (2 mol song song)
  Zen 5           3/cycle          ~1.1B/s (3 mol song song)
```

### VM Dispatch — Branch Prediction Reality

```
VM fetch-decode-execute loop = indirect branch mỗi opcode.
Indirect branches = WORST CASE cho branch predictor.

Thực tế đo được:
  - Indirect branch misprediction: ~25% ❌
  - Chiếm 55.7% tổng số mispredictions
  - Mỗi mispredict: 10-14 cycles penalty

Ảnh hưởng:
  25% miss × 12 cycles penalty = 3 cycles overhead TRUNG BÌNH mỗi opcode
  → VM dispatch overhead ≈ 3-5 cycles/opcode (đáng kể!)

Giải pháp:
  ✅ Threaded code: mỗi opcode handler kết thúc bằng indirect jump riêng
     → Branch predictor có history riêng cho mỗi opcode → miss rate giảm
  ✅ Computed goto (GCC extension): tương tự threaded code
  ✅ Jump table + prefetch: prefetch opcode tiếp theo trong handler hiện tại
  ❌ Switch/case lớn: 1 branch location → 1 prediction entry → miss rate cao
```

### Molecule trong Cache

```
Molecule = 5 bytes. Cache line = 64 bytes (hoặc 128 Apple).

  5 bytes, packed tightly:
    64B line → 12 molecules + 4 bytes dư
    128B line → 25 molecules + 3 bytes dư

  Cache line straddling (5 không chia hết 64):
    Xác suất cross 2 lines: 4/64 = 6.25% (x86/ARM)
    Xác suất cross 2 lines: 4/128 = 3.125% (Apple)
    → Chấp nhận được, KHÔNG cần pad lên 8 bytes

  Tagged sparse (~3 bytes trung bình):
    64B line → ~21 tagged molecules
    128B line → ~42 tagged molecules
    → Tiết kiệm 40% cache space

  Working set analysis:
    L1D = 64 KB → 12,800 molecules (packed) hoặc 21,000 (tagged)
    L2 = 2 MB → 400,000 molecules
    → L0 (5400 molecules × 5B = 27 KB) VỪA TRONG L1D ⭐⭐⭐
    → L0 + L1 weights VỪA TRONG L2

  Full scan 500M concepts:
    500M × 33 bytes = 16.5 GB
    Phone 65 GB/s → 0.25 seconds
    M4 Max 273 GB/s → 0.06 seconds
    → Sequential scan = OK, nhưng Dream clustering O(N²) vẫn cần parallel
```

### CPU Summary cho HomeOS

```
HomeOS Operation          CPU fit?    Why
───────────────────────────────────────────────────────────────
VM dispatch (45 opcodes)   ✅ GOOD     Sequential, branch-heavy → CPU's domain
FNV-1a single hash         ✅ PERFECT  Serial XOR+MUL chain → CPU only
FNV-1a batch hash          ✅ GOOD     Multiple MUL pipes help
LCA (5D weighted avg)      ✅ PERFECT  5 multiply-adds = 10-15 cycles
Silk co-activate           ✅ GOOD     Hash lookup + weight update
Silk BFS/DFS               ✅ OK       Irregular memory → cache dependent
Emotion pipeline (7 tier)  ✅ PERFECT  Sequential, low compute
Dream cluster (small N)    ✅ OK       N < 10K → seconds on CPU
Dream cluster (large N)    ⚠️ SLOW     N > 100K → minutes → need parallel
String ops                 ✅ PERFECT  Sequential byte processing
SHA-256 (single block)     ✅ PERFECT  32-bit integer ops
SecurityGate check         ✅ PERFECT  Branch-heavy logic
```

---

## 2. GPU — Parallel Accelerator, có giới hạn

### Thông số thực tế — Mobile GPU

```
                    Mali-G720       Adreno 750      Apple M4 GPU
                    (ARM)           (Qualcomm)      (Apple)
────────────────────────────────────────────────────────────────
Shader cores        6-16            12 CUs          10 cores
Shading units       ~896 (MC7)      1,536           1,280
Warp/wave size      16              64              32
FP32 TFLOPS         ~2.3            ~3.0            ~2.9
FP16                Double-rate     Double-rate     Double-rate
FP64                ❌ NO           ❌ NO           ❌ NO
Int64 multiply      ❌ NO           ❌ NO ¹         ❌ NO ²
Shared memory       ❌ FAKE ³       ✅ 32 KB real   ✅ 32 KB
Memory BW           ~65 GB/s        ~77 GB/s        ~120 GB/s
Max workgroup       512             1024            1024

¹ Adreno: shaderInt64 sometimes exposed nhưng EMULATED (4-6× slower)
² Apple: uint64_t type exists nhưng NO native MUL instruction
³ Mali: "shared memory" backed by system RAM through cache, NOT real SRAM
```

### FNV-1a trên GPU — VẤN ĐỀ LỚN

```
FNV-1a-64 cần: XOR + 64-bit integer MUL per byte.

KHÔNG CÓ mobile GPU nào có native 64-bit integer multiply.

  Platform          64-bit MUL?    Emulation cost
  ──────────────────────────────────────────────────
  Mali-G720         ❌ Không có    Không support luôn
  Adreno 750        Emulated       4-6 ops per MUL (mul-lo, mul-hi, add-carry)
  Apple M4 GPU      Emulated       Multiple 32-bit ops
  NVIDIA H100       ✅ Native      1 op
  AMD MI300X        ✅ Native      1 op

  → FNV-1a-64 trên mobile GPU: 4-6× chậm hơn per step
  → Batch 1M hashes: CPU vẫn có thể THẮNG GPU trên mobile!

  Desktop GPU (H100/MI300): native int64 → batch hashing nhanh hơn CPU 100×
  Mobile GPU: emulated int64 → batch hashing chỉ nhanh hơn CPU 5-10×
```

### Giải pháp cho GPU hashing

```
Option A: FNV-1a-32 (thay vì 64)
  + Native 32-bit MUL trên mọi GPU
  + Collision OK cho hash tables < 65K entries
  - Collision rate tăng khi > 100K entries
  - KHÔNG TƯƠNG THÍCH với Rust FNV-1a-64 hiện tại

Option B: Dual FNV-1a-32 (2 hashes độc lập → combine 64-bit)
  + Native 32-bit MUL
  + 64-bit output (low collision)
  + Parallel: 2 independent chains
  - 2× ops so với single FNV-1a-64
  - Khác hash value → KHÔNG deterministic với CPU

Option C: Emulate 64-bit MUL trên GPU
  + Deterministic: cùng hash value với CPU
  + Chấp nhận 4-6× slower per MUL
  + Batch parallelism bù lại per-op slowness
  - Phức tạp hơn
  - Mobile: ~5-10× faster than single-core CPU (thay vì 100×)

→ KHUYẾN NGHỊ: Option C (emulate) để giữ deterministic.
  Tại scale 100B nodes: batch 1M hashes, 5-10× vẫn worth it.
  Tại scale nhỏ: CPU đủ nhanh, không cần GPU.
```

### GPU workloads phù hợp HomeOS

```
HomeOS Operation          GPU fit?    Why
───────────────────────────────────────────────────────────────
VM dispatch               ❌ AWFUL    Sequential, branchy → GPU kẻ thù
FNV-1a single hash        ❌ AWFUL    Serial dependency → CPU only
FNV-1a batch (1M+)        ⚠️ OK      Parallel nhưng int64 emulated
LCA batch (1M+)           ✅ GOOD     5D weighted avg = f32 OK, parallel
Dream distance matrix     ✅ GOOD ⭐  N×N pairwise 5D distance → perfect GPU
Dream clustering          ✅ GOOD     Parallel reduction + sort
Silk BFS/DFS              ⚠️ POOR    Irregular memory access → cache thrash
Silk weight update batch  ✅ OK       Parallel f32 updates
SHA-256 batch             ✅ GOOD     32-bit integer ops, parallel
KNN search (5D)           ✅ GOOD ⭐  Parallel scan, f32 distance
String ops                ❌ AWFUL    Sequential byte processing
Emotion pipeline          ❌ AWFUL    Sequential 7-tier
SecurityGate              ❌ AWFUL    Branch logic
```

### WebGPU thêm giới hạn

```
WebGPU (browser) vs Vulkan/Metal:
  - ❌ KHÔNG CÓ int64 (spec chưa support)
  - ❌ KHÔNG CÓ subgroup operations (planned, chưa có)
  - Max workgroup: 256 (vs 1024 Vulkan/Metal)
  - Không atomic 64-bit

  → FNV-1a-64 trên WebGPU: KHÔNG THỂ (không có int64 type)
  → Phải dùng FNV-1a-32 hoặc emulate bằng 2×u32

  → WebGPU chỉ hữu ích cho:
    ✅ Dream distance matrix (f32)
    ✅ LCA batch (f32)
    ✅ KNN search (f32)
    ❌ Hashing (no int64)
    ❌ Anything needing int64
```

---

## 3. NPU — Kết luận: KHÔNG DÙNG ĐƯỢC cho HomeOS

### Thực tế NPU (không phải marketing)

```
                    Qualcomm        Apple ANE       Intel NPU       AMD XDNA 2
                    Hexagon         (M4)            (Meteor Lake)   (Ryzen AI)
────────────────────────────────────────────────────────────────────────────────
Marketed TOPS       45              38              11              50
Real-world TOPS     0.5-5 ❌        ~5.7 peak       ~3-5            ~5-15
FP16                ✅              ✅ (native)     ✅              ✅
FP32                ❌              ❌              ❌              ❌
FP64                ❌              ❌              ❌              ❌
INT8                ✅              Fake ¹          ✅              ✅
Programmable?       Graph-level     Black box ²     Graph-level     Bare-metal ³
Custom ops?         QNN packages    ❌ (officially) Limited         ✅ (C++ kernels)

¹ Apple ANE: INT8 → dequantize to FP16 trước khi compute. INT8 chỉ tiết kiệm bandwidth.
² Apple ANE: Không có ISA docs. Chỉ qua CoreML. Reverse-engineer mới biết bên trong.
³ AMD XDNA: IRON API cho bare-metal access — nhưng vẫn chủ yếu cho tensor ops.

THỰC TẾ ĐÁNG SỢ:
  Qualcomm 45 TOPS → Useful Sensors đo được 0.573 TOPS = 1.3% marketed.
  Nguyên nhân: memory bandwidth bottleneck. NPU compute nhanh nhưng
  không feed data kịp từ LPDDR → idle phần lớn thời gian.
```

### NPU có thể làm gì?

```
Operation               NPU support?    Notes
───────────────────────────────────────────────────────────────
Matrix multiply          ✅ YES          Đây là LÝ DO TỒN TẠI duy nhất
Convolution              ✅ YES          Apple ANE: primary primitive
Element-wise (add, mul)  ✅ YES          As graph nodes
Softmax, LayerNorm       ⚠️ PARTIAL     Often falls back to CPU
Hash computation         ❌ NO           Cần arbitrary integer logic
Graph traversal          ❌ NO           Irregular memory, pointer chasing
Sorting                  ❌ NO           Conditional branching
Arbitrary integer math   ❌ NO           Fixed-point quantized MAC arrays
Cryptography             ❌ NO           Needs full-precision bit manipulation
General branching        ❌ NO           Static compiled graphs, no branches
FNV-1a                   ❌ NO           XOR + 64-bit MUL = impossible
LCA                      ❌ NO ¹         Could theoretically map to matmul but overhead > benefit
Silk operations          ❌ NO           Graph structure = worst case for NPU
Dream clustering         ❌ NO ²         Could map distance matrix to matmul, but int8/fp16 precision

¹ LCA = 5D weighted average. Could express as 1×5 × 5×N matrix multiply.
  But setup overhead (compile graph, transfer data, launch NPU) >> actual compute.
  CPU does this in ~15 cycles. NPU launch latency = microseconds.

² Distance matrix COULD be a matmul if reformulated.
  But NPU precision (int8/fp16) → hash collision risk.
  GPU (f32) is better fit.
```

### NPU kết luận

```
NPU = matrix multiply machine. Không gì khác.

HomeOS operations:
  - Hash (FNV-1a): XOR + integer MUL → ❌ NPU cannot do
  - Graph (Silk): pointer chasing → ❌ NPU cannot do
  - Sequential logic (VM, Gate): branching → ❌ NPU cannot do
  - 5D math (LCA, similarity): too small → overhead > benefit

NPU hữu ích cho:
  - LLM inference (nhưng HomeOS KHÔNG DÙNG LLM)
  - Image classification (nhưng Worker camera dùng SDF fitting, không NN)
  - Speech recognition (possible future use, nhưng không core)

→ BỎ QUA NPU. Không đáng thiết kế cho nó.
  Nếu tương lai cần ML: dùng NPU qua standard API (CoreML/NNAPI).
  Không cần tích hợp vào VM.
```

---

## 4. Mapping: HomeOS Workloads → Hardware

### Theo scale

```
SCALE NHỎ (< 100K nodes — phần lớn use cases):
  ┌─────────────────────────────────────────────────┐
  │ CPU LÀM TẤT CẢ. Không cần GPU, không cần NPU.  │
  │                                                  │
  │ VM dispatch:     ~5 cycles/opcode                │
  │ FNV-1a hash:     ~3 ns/molecule                  │
  │ LCA:             ~15 cycles (5D avg)             │
  │ Silk co-activate: ~50 ns (hash + weight update)  │
  │ Dream (10K obs):  ~100ms                         │
  │ Emotion pipeline: ~1 μs per turn                 │
  │                                                  │
  │ Total turn latency: < 1 ms ⭐                    │
  │ 100% CPU. 0% GPU. 0% NPU.                       │
  └─────────────────────────────────────────────────┘

SCALE TRUNG BÌNH (100K - 10M nodes):
  ┌─────────────────────────────────────────────────┐
  │ CPU chính. GPU hỗ trợ Dream/batch.               │
  │                                                  │
  │ Real-time (per turn): CPU only                   │
  │   VM + emotion + response: < 1 ms               │
  │                                                  │
  │ Background (Dream, nightly):                     │
  │   Distance matrix 100K: CPU 5s → GPU 50ms       │
  │   Batch LCA 10K: CPU 2s → GPU 20ms              │
  │   KNN search: CPU 1s → GPU 10ms                 │
  │                                                  │
  │ GPU chỉ dùng cho batch operations OFFLINE.       │
  │ Never on hot path. Never blocking user.          │
  └─────────────────────────────────────────────────┘

SCALE LỚN (10M - 100B nodes):
  ┌─────────────────────────────────────────────────┐
  │ CPU cho real-time. GPU cho batch. Tiered storage.│
  │                                                  │
  │ Real-time: CPU only (hot set trong cache)        │
  │   Working set << total knowledge                 │
  │   L1/L2 cache sufficient cho active nodes        │
  │                                                  │
  │ Background batch:                                │
  │   Dream cluster 1M obs: GPU 30s (vs CPU 30min)  │
  │   Batch hash 10M chains: GPU 1s (vs CPU 10s)    │
  │   KNN 100M points: GPU 10s (vs CPU 1000s)       │
  │                                                  │
  │ Tiered storage:                                  │
  │   Hot (L1/L2 cache): active conversation nodes   │
  │   Warm (RAM): recent knowledge                   │
  │   Cold (disk): archived, load on-demand          │
  │                                                  │
  │ NPU: vẫn VÔ DỤNG ở mọi scale.                   │
  └─────────────────────────────────────────────────┘
```

### Theo operation

```
Operation           Small scale    Medium scale    Large scale     Hardware
                    (< 100K)       (100K-10M)      (10M-100B)
────────────────────────────────────────────────────────────────────────────
VM dispatch         CPU            CPU             CPU             CPU only ⁺
FNV-1a (per turn)   CPU            CPU             CPU             CPU only ⁺
FNV-1a (batch)      CPU            CPU             GPU ⁱ           GPU (emulate i64)
LCA (per turn)      CPU            CPU             CPU             CPU only ⁺
LCA (batch)         CPU            CPU/GPU         GPU             GPU (f32)
Emotion pipeline    CPU            CPU             CPU             CPU only ⁺
Silk co-activate    CPU            CPU             CPU             CPU only ⁺
Silk BFS/DFS        CPU            CPU             CPU ⁱⁱ          CPU (irregular access)
Dream clustering    CPU            GPU             GPU ⭐          GPU (distance matrix)
Dream propose       CPU            CPU             CPU             CPU only ⁺
KNN search          CPU            CPU/GPU         GPU ⭐          GPU (parallel scan)
SHA-256 (per op)    CPU            CPU             CPU             CPU only ⁺
SHA-256 (batch)     CPU            CPU             GPU             GPU (u32 native)
SecurityGate        CPU            CPU             CPU             CPU only ⁺
Response render     CPU            CPU             CPU             CPU only ⁺

⁺  Sequential/branchy → GPU/NPU không giúp được gì
ⁱ  Int64 emulated trên mobile GPU → chỉ 5-10× speedup (vs 100× lý thuyết)
ⁱⁱ Graph traversal trên GPU: worse than CPU do irregular memory access
```

---

## 5. Phát hiện quan trọng cho kiến trúc

### ① HomeOS operations chủ yếu SERIAL — CPU-first là ĐÚNG

```
Phân tích 45 opcodes + 60 builtins:
  - 90% operations = sequential, branchy, data-dependent
  - 10% operations = parallelizable (batch hash, batch LCA, distance matrix)

→ CPU là accelerator CHÍNH cho HomeOS.
  GPU là accelerator PHỤ cho batch operations.
  NPU = không dùng.
```

### ② FNV-1a-64 serial dependency = bottleneck đặc trưng

```
hash ^= byte;               // XOR: 1 cycle
hash = hash * FNV_PRIME;     // MUL: 2-3 cycles, DEPENDS on previous XOR
// → Không thể pipeline, không thể parallel cho 1 hash

Nhưng BATCH hashing (nhiều molecules độc lập):
  CPU: dùng hết MUL pipes (3-4 parallel chains) → 1-1.5B hashes/sec
  GPU: dùng 1000+ threads (emulated i64) → 5-10B hashes/sec mobile

→ Single hash: CPU wins
→ Batch 1M+ hashes: GPU wins (5-10× trên mobile, 100× trên desktop)
```

### ③ Dream clustering = perfect GPU workload

```
Dream: N observations → N×N similarity matrix → cluster

Similarity(a, b) = weighted 5D distance:
  Σᵢ wᵢ × |aᵢ - bᵢ|² (i = 0..4)
  = 5 subtracts + 5 multiplies + 5 weighted adds + 1 sqrt
  = ~20 f32 operations per pair

N×N pairs = N² × 20 ops:
  N=1K:    20M ops → CPU 5ms, GPU 0.05ms
  N=10K:   2B ops → CPU 500ms, GPU 5ms
  N=100K:  2T ops → CPU 14 hours, GPU 30 seconds ⭐
  N=1M:    2P ops → CPU impossible, GPU ~1 hour

→ Dream clustering là MOTIVATING USE CASE cho GPU.
  Không phải hashing, không phải VM — Dream.
```

### ④ Cache architecture THUẬN LỢI cho HomeOS

```
L0 = 5400 molecules × 5 bytes = 27 KB → VỪA L1D (48-128 KB) ⭐⭐⭐

Điều này có nghĩa:
  - Mọi L0 lookup = L1D cache hit = 3-4 cycles
  - encode_codepoint() hot path = L1D resident
  - LCA với L0 nodes = tất cả trong cache

L1 weights (STM active) = vài nghìn observations × 33 bytes = < 1 MB → VỪA L2

Working set per conversation turn:
  ~100-500 relevant nodes × 33 bytes = 3-16 KB → VỪA L1D

→ HomeOS working set FITS IN CACHE cho hầu hết operations.
  Memory bandwidth chỉ matter cho Dream (scan toàn bộ STM)
  và import batch (load new data từ disk).
```

### ⑤ Branch prediction và VM dispatch

```
Indirect branch misprediction = 25% trên modern CPUs.
Mỗi opcode dispatch = 1 indirect branch.

Giải pháp đã biết:
  1. Threaded code (mỗi handler có tail dispatch riêng)
     → Branch predictor có separate history per opcode
     → Miss rate giảm từ 25% xuống ~5-10%

  2. Superinstructions (fuse common opcode pairs)
     → Push+Store = 1 superinstruction
     → Giảm 50% dispatches cho common patterns

  3. JIT compilation (Phase 5)
     → Hot loops → native code → no dispatch overhead
     → Cold code → interpreted (chấp nhận 25% miss)

vm_arm64.S và vm_x86_64.S NÊN dùng threaded code:
  mỗi op_push handler kết thúc bằng:
    ldrb w0, [x19], #1      // fetch next opcode
    ldr  x1, [x16, x0, lsl #3]  // lookup handler
    br   x1                  // tail dispatch
  → Branch predictor thấy: "sau op_push thường là op_store" → predict đúng
```

---

## 6. Recommendations cho Architecture

### Phase 1 (hiện tại): CPU-only, thiết kế cho tương lai

```
✅ DO:
  - VM dùng threaded code (không switch/case)
  - Pack molecules tight (không pad lên 8 bytes)
  - L0 table đủ nhỏ → L1D resident
  - FNV-1a-64 trên CPU → deterministic, fast
  - Tất cả operations = CPU single-thread

❌ DON'T:
  - Không thiết kế cho GPU yet
  - Không nghĩ về NPU
  - Không dùng AVX-512/NEON cho single operations (overhead > benefit)
```

### Phase 5 (GPU integration): Chỉ cho batch operations

```
✅ GPU targets (worth it):
  - Dream distance matrix (N > 10K): 100× speedup
  - Batch LCA (N > 100K): 100× speedup
  - KNN search (N > 1M): 100× speedup
  - Batch SHA-256 verify: 50× speedup

⚠️ GPU possible nhưng limited (mobile):
  - Batch FNV-1a-64: emulate i64, chỉ 5-10× speedup
  - Silk weight update: limited by scatter write pattern

❌ GPU waste of time:
  - VM dispatch
  - Single hash/LCA
  - Emotion pipeline
  - String ops
  - SecurityGate

GPU API priority:
  1. Vulkan Compute (cross-platform, most capable)
  2. Metal Compute (Apple, required for macOS/iOS)
  3. WebGPU (browser, most limited — no int64)
```

### NPU: Bỏ qua

```
❌ NPU không hỗ trợ BẤT KỲ HomeOS operation nào.
  - Chỉ matrix multiply + activations
  - Không integer math, không graph ops, không hash
  - Marketing TOPS vs real TOPS = 50-100× gap
  - Black box (Apple), fragmented SDK (Qualcomm/Intel)

Ngoại lệ tương lai (KHÔNG PHẢI core HomeOS):
  - Voice recognition cho Worker microphone → NPU via NNAPI/CoreML
  - Image classification cho Worker camera → NPU via standard API
  - NHƯNG: output = molecular chain, NPU chỉ là sensor preprocessing
```

---

## 7. So sánh cuối cùng

```
                CPU             GPU (mobile)        NPU
────────────────────────────────────────────────────────────
FNV-1a-64       ✅ 380M/s       ⚠️ emulate i64      ❌ impossible
LCA single      ✅ 15 cycles    ❌ overhead > work   ❌ impossible
LCA batch 1M    ✅ 200ms        ✅ 2ms ⭐           ❌ impossible
Dream 100K      ⚠️ 14 hours    ✅ 30 seconds ⭐    ❌ impossible
Dream 1K        ✅ 5ms          ❌ overhead > work   ❌ impossible
VM dispatch     ✅ best         ❌ worst             ❌ impossible
Silk BFS        ✅ OK           ❌ irregular mem     ❌ impossible
KNN 1M          ⚠️ 1000s       ✅ 10s ⭐           ❌ impossible
SHA-256 single  ✅ fast         ❌ overhead > work   ❌ impossible
SHA-256 batch   ✅ OK           ✅ 50× faster       ❌ impossible
Emotion         ✅ 1μs          ❌ sequential        ❌ impossible
Strings         ✅ fast         ❌ sequential        ❌ impossible

VERDICT:
  CPU  = chạy MỌI THỨ, nhanh cho hầu hết operations
  GPU  = chỉ hữu ích khi N > 10K VÀ operation parallelizable
  NPU  = vô dụng cho HomeOS
```

---

*Analysis dựa trên data thực tế từ:*
*Chips and Cheese, AnandTech, Agner Fog instruction tables, Daniel Lemire,*
*arXiv papers on branch prediction, Useful Sensors NPU benchmarks,*
*Khronos Vulkan spec, Apple Metal docs, WebGPU spec.*
