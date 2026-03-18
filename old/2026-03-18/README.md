# Origin — HomeOS & Olang

> "Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."

**HomeOS** = Sinh linh toán học tự vận hành — hệ điều hành học được, cảm được, nhớ được.
**Olang** = Ngôn ngữ phân tử duy nhất. Mọi ngôn ngữ tự nhiên là alias.

```
○(x) == x       identity     — ○ không làm hỏng thứ gì
○(∅) == ○       tự tạo sinh  — từ hư không, ○ tự sinh ra
○ ∘ ○ == ○      idempotent   — không phình to khi compose
mọi f == ○[f]   instance     — mọi thứ là instance của ○
```

---

## Không gian 5 chiều

Mỗi khái niệm = tọa độ trong không gian 5D, từ **~5,400 ký tự Unicode 18.0** có semantic identity rõ ràng:

```
MolecularChain = [Shape][Relation][Valence][Arousal][Time]
                  1 byte  1 byte    1 byte   1 byte  1 byte = 5 bytes/molecule

Nhóm        Ký tự    Chiều        Ý nghĩa
────────────────────────────────────────────────────
SDF         ~1344    Shape        "Trông như thế nào" (● ▬ ■ ▲ ○ ∪ ∩ ∖)
MATH        ~1904    Relation     "Liên kết thế nào" (∈ ⊂ ≡ ⊥ ∘ → ≈ ←)
EMOTICON    ~1760    Valence+A    "Cảm thế nào" (0x00..0xFF × 2)
MUSICAL     ~416     Time         "Thay đổi thế nào" (Static → Instant)
```

---

## Kiến trúc

```
Người dùng → runtime::HomeRuntime.process_text()
                 │
                 ├─ ○{...} → Parser → IR → VM → Response
                 │
                 └─ text → Emotion Pipeline 7 tầng:
                      T1: infer_context()        ← điều kiện biên
                      T2: sentence_affect()      ← raw emotion từ từ ngữ
                      T3: ctx.apply()            ← scale theo ngữ cảnh
                      T4: estimate_intent()      ← Crisis/Learn/Command/Chat
                      T5: Crisis check           ← DỪNG NGAY nếu nguy hiểm
                      T6: learning.process_one() ← Encode → STM → Silk
                      T7: render response        ← tone từ ConversationCurve
```

### Agent Hierarchy

```
NGƯỜI DÙNG
    ↓
AAM  [tier 0] — stateless · approve · quyết định cuối
    ↓ ISL
LeoAI       [tier 1] — Knowledge + Learning + Dream + 7 bản năng
HomeChief   [tier 1] — quản lý Worker thiết bị nhà
VisionChief [tier 1] — quản lý Worker camera/sensor
NetworkChief[tier 1] — quản lý Worker network/security
    ↓ ISL
Workers     [tier 2] — SILENT · skill đúng việc · báo cáo chain

Giao tiếp:
  ✅ AAM ↔ Chief     ✅ Chief ↔ Chief     ✅ Chief ↔ Worker
  ❌ AAM ↔ Worker    ❌ Worker ↔ Worker
```

---

## Cấu trúc dự án

```
crates/
├── ucd/        Unicode → Molecule lookup (build.rs, 5424 entries)      21 tests
├── olang/      Core: Molecule · LCA · Registry · VM · Compact        594 tests
├── silk/       Hebbian learning · EmotionTag edges · Walk              82 tests
├── context/    Emotion V/A/D/I · ConversationCurve · Intent           168 tests
├── agents/     Encoder · Learning · Gate · Instinct · Chief/Worker    252 tests
├── memory/     STM · DreamCycle · Proposals · AAM                      62 tests
├── runtime/    HomeRuntime · ○{} Parser · MVHOS                       204 tests
├── hal/        Hardware Abstraction · Tier · FFI · Security            76 tests
├── isl/        Inter-System Link messaging (AES-256-GCM)               31 tests
├── vsdf/       18 SDF generators · FFR · Physics · SceneGraph         116 tests
└── wasm/       WebAssembly bindings · WebSocket-ISL bridge             23 tests

tools/
├── seeder/     Seed 35 L0 nodes từ UCD (0 hardcode)                    15 tests
├── server/     Terminal REPL (stdin/stdout)
├── inspector/  Đọc/verify origin.olang
└── bench/      Performance benchmarks
```

**~66,000 lines Rust · 1,744 tests · 0 clippy warnings · 0 external deps · no_std core**

---

## Đánh giá hiện tại

| Hạng mục | Điểm |
|----------|------|
| Thiết kế kiến trúc | 10/10 |
| Chất lượng code | 8.5/10 |
| Độ phủ test | 9.5/10 |
| Tuân thủ QT (23 quy tắc) | 9.5/10 (21 đầy đủ, 2 một phần) |
| Tính năng hoàn thiện | 7.5/10 |
| Bảo mật | 9/10 |
| Độc lập (0 external deps) | 10/10 |
| **Tổng** | **8.81/10 — A** |

Chi tiết: xem [docs/REVIEW_2026_03_17.md](docs/REVIEW_2026_03_17.md) (mới nhất), [REVIEW.md](REVIEW.md), [REVIEW_VI.md](REVIEW_VI.md).

---

## MVHOS — Minimum Viable HomeOS

```
✅ boot từ binary rỗng < 200ms
✅ ○{🔥}       → chain + human-readable info [1mol ●×∈ V=180 A=200 U+1F525]
✅ ○{🔥 ∘ 💧}  → LCA result (weighted mode detection)
✅ ○{lửa}      → alias resolve → node 🔥
✅ ○{stats}    → Registry nodes/aliases, Silk edges, KnowTree summary
✅ crash → restart → state giữ nguyên (serialize + parse_recoverable)
✅ 0 hardcoded Molecule (QT4 compliant)
```

---

## Dependency Graph

```
ucd (UnicodeData.txt → compile-time table)
 └→ olang (Molecule, Chain, LCA, Registry, VM, Compact, KnowTree)
     ├→ silk (SilkGraph, Hebbian, EmotionTag edges, WalkWeighted)
     │   └→ context (Emotion V/A/D/I, ConversationCurve, Intent, Fusion)
     │       └→ agents (Encoder, Learning, Gate, Instinct, LeoAI, Chief, Worker)
     │           ├→ hal (Architecture, Platform, Tier, FFI, Security)
     │           └→ memory (STM, Dream, Proposals, AAM)
     │               └→ runtime (HomeRuntime — entry point)
     │                   └→ wasm (WASM bindings, WebSocket bridge)
     ├→ isl (ISL messaging: 4-byte address, AES-256-GCM)
     └→ vsdf (18 SDF + FFR Fibonacci render + SceneGraph)
```

---

## 23 Nguyên tắc bất biến (QT)

```
Unicode:
  QT1:  5 nhóm Unicode = nền tảng. Không thêm nhóm.
  QT2:  Tên ký tự Unicode = tên node. Không đặt tên khác.
  QT3:  Ngôn ngữ tự nhiên = alias → node. Không tạo node riêng.

Chain:
  QT4:  Mọi Molecule từ encode_codepoint(cp) — KHÔNG viết tay ¹
  QT5:  Mọi chain từ LCA hoặc UCD — KHÔNG viết tay
  QT6:  chain_hash tự sinh. KHÔNG viết tay.
  QT7:  chain cha = LCA(chain con)

Node:
  QT8:  Mọi Node tạo ra → tự động registry
  QT9:  Ghi file TRƯỚC — cập nhật RAM SAU
  QT10: Append-only — KHÔNG DELETE, KHÔNG OVERWRITE

Silk:
  QT11: Silk chỉ ở Ln-1 — tự do giữa lá cùng tầng ²
  QT12: Kết nối tầng trên → qua NodeLx đại diện (Fib[n+2] threshold)
  QT13: Silk mang EmotionTag của khoảnh khắc co-activation

Kiến trúc:
  QT14: L0 không import L1 — tuyệt đối
  QT15: Agent tiers: AAM(tier 0) + Chiefs(tier 1) + Workers(tier 2)
  QT16: L2-Ln đổ vào SAU khi L0+L1 hoàn thiện
  QT17: Fibonacci xuyên suốt — cấu trúc, threshold, render
  QT18: Không đủ evidence → im lặng — KHÔNG bịa (BlackCurtain)

Skill:
  QT19: 1 Skill = 1 trách nhiệm
  QT20: Skill không biết Agent là gì
  QT21: Skill không biết Skill khác tồn tại
  QT22: Skill giao tiếp qua ExecContext.State
  QT23: Skill không giữ state — state nằm trong Agent

¹ QT4: VM PushMol, VSDF FFRCell::to_molecule(), và LCA tạo Molecule ngoài
  encode_codepoint(). Đây là tính toán lúc chạy, không phải giá trị viết tay
  — chấp nhận được.

² QT11: SilkGraph::co_activate_same_layer() kiểm tra tầng tại API.
  SilkGraph::co_activate() vẫn yêu cầu caller đảm bảo cùng tầng.
  co_activate_cross_layer() cho kết nối khác tầng có kiểm soát.
```

---

## Lộ trình phát triển

```
HOÀN THÀNH (Foundation):
  ✅ UCD Engine (5424 entries, 0 collision)
  ✅ Molecule/Chain 5D encoding
  ✅ LCA + Weighted + Variance
  ✅ Registry (append-only, crash recovery)
  ✅ Silk + Hebbian + φ⁻¹ decay
  ✅ Emotion Pipeline 7 tầng
  ✅ 7 Instinct Skills (Honesty → Reflection)
  ✅ 15 Domain Skills (QT4 compliant)
  ✅ ISL messaging (AES-256-GCM)
  ✅ HAL (x86/ARM/RISC-V/WASM)
  ✅ VSDF (18 SDF + FFR Fibonacci)
  ✅ Agent Hierarchy (AAM/Chief/Worker)
  ✅ Compiler backends (C/Rust/WASM)
  ✅ WASM browser bindings

HOÀN THÀNH (Phases):
  ✅ Phase 1: VM tính toán thật (○{1 + 2} = 3)
  ✅ Phase 2: Duyệt đồ thị (why/explain)
  ✅ Phase 3: Tri thức L1+ (246 domain nodes)
  ✅ Phase 4: Toán ký hiệu kết nối VM
  ✅ Phase 5: Điều phối Agent đầy đủ
  ✅ Phase 6-8: Perception, Programming, Build layers
  ✅ Phase 9: Zero External Dependencies (native SHA-256, Ed25519, AES-256-GCM)

TIẾP THEO:
  Phase 10: Dream→KnowTree L3 pipeline          — ưu tiên CAO
  Phase 11: Olang Parser — thêm RelOps thiếu    — ưu tiên CAO
  Phase 12: Data L2-Ln seeding                   — ưu tiên TRUNG BÌNH
```

Chi tiết: xem [HomeOS_Roadmap.md](HomeOS_Roadmap.md).

---

## Chạy

```bash
# Build toàn bộ
cargo build --workspace

# Test (1744+ tests)
cargo test --workspace

# Clippy (phải 0 warnings)
cargo clippy --workspace

# Chạy REPL
cargo run -p server

# Seed L0 nodes
cargo run -p seeder
```

### Ví dụ REPL

```
○ ○{🔥}
○ 🔥=🔥 [1mol ●×∈ V=180 A=200 #A47B U+1F525]

○ ○{lửa}
○ lửa=🔥 [1mol ●×∈ V=180 A=200 #A47B U+1F525]

○ ○{🔥 ∘ 💧}
○ 🔥=🔥 💧=💧 ∘→○ [1mol ...]

○ tôi buồn vì mất việc
Cảm giác nặng nề và mệt mỏi — bạn muốn kể thêm không?

○ ○{stats}
HomeOS ○
Turns    : 2
Registry : 39 nodes, 35 aliases
STM      : 2 observations
Silk     : 5 nodes, 8 edges
f(x)     : -0.312
```

---

## Yêu cầu

- **Rust** (stable, edition 2021)
- `ucd_source/UnicodeData.txt` và `ucd_source/Blocks.txt` từ [Unicode 18.0](https://unicode.org/Public/18.0.0/ucd/)

---

## Tài liệu

| File | Nội dung |
|------|---------|
| [CLAUDE.md](CLAUDE.md) | Hướng dẫn cho AI contributors |
| [HomeOS_Architecture.md](HomeOS_Architecture.md) | Kiến trúc tổng thể |
| [HomeOS_Roadmap.md](HomeOS_Roadmap.md) | Kế hoạch phát triển (Phase 1-9 hoàn thành) |
| [HomeOS_Solutions.md](HomeOS_Solutions.md) | Hướng giải quyết hạn chế |
| [HomeOS_Complete.md](HomeOS_Complete.md) | Thiết kế hoàn chỉnh |
| [REVIEW.md](REVIEW.md) | Đánh giá dự án (English) |
| [REVIEW_VI.md](REVIEW_VI.md) | Đánh giá dự án (Tiếng Việt) |
| [docs/olang_guide.md](docs/olang_guide.md) | Hướng dẫn ngôn ngữ Olang |
| [docs/architecture.md](docs/architecture.md) | Kiến trúc kỹ thuật chi tiết |
| [docs/roadmap.md](docs/roadmap.md) | Kế hoạch tiếp theo |
| [docs/REVIEW_2026_03_17.md](docs/REVIEW_2026_03_17.md) | Đánh giá mới nhất (8.81/10) |

---

## Scale

```
TieredStore: Hot/Warm/Cold tiers + LRU PageCache (Fibonacci: 55/233/610/2584)
LayerIndex:  Bloom filter (256B, 3 hashes) + sorted binary search O(log n)
Compact:     DeltaMolecule (1-6B vs 5B) + ChainDictionary (dedup)
Target:      1 tỷ nodes trên thiết bị phổ thông (RAM < 256MB, disk ~2GB)
```

---

*Unicode 18.0.0 · Rust · no_std core · ~66K LoC · 1,744 tests · 0 clippy warnings · 0 external deps · 2026*
