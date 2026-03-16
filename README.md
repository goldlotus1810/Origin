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
LeoAI      [tier 1] — Knowledge + Learning + Dream + 7 bản năng
HomeChief  [tier 1] — quản lý Worker thiết bị nhà
    ↓ ISL
Workers    [tier 2] — SILENT · skill đúng việc · báo cáo chain
```

---

## Cấu trúc dự án

```
crates/
├── ucd/        Unicode → Molecule lookup (build.rs, 5263 entries)     21 tests
├── olang/      Core: Molecule · LCA · Registry · VM · Compact       286 tests
├── silk/       Hebbian learning · EmotionTag edges · Walk             57 tests
├── context/    Emotion V/A/D/I · ConversationCurve · Intent          160 tests
├── agents/     Encoder · Learning · Gate · Instinct · Chief/Worker   187 tests
├── memory/     STM · DreamCycle · Proposals · AAM                     32 tests
├── runtime/    HomeRuntime · ○{} Parser · MVHOS                      128 tests
├── hal/        Hardware Abstraction · Tier · FFI · Security           76 tests
├── isl/        Inter-System Link messaging (AES-256-GCM)              31 tests
├── vsdf/       18 SDF generators · FFR · Physics · SceneGraph        103 tests
└── wasm/       WebAssembly bindings · WebSocket-ISL bridge            23 tests

tools/
├── seeder/     Seed 35 L0 nodes từ UCD (0 hardcode)
├── server/     Terminal REPL (stdin/stdout)
├── inspector/  Đọc/verify origin.olang
└── bench/      Performance benchmarks
```

**38,000+ lines Rust · 1,104 tests · 0 clippy warnings · no_std core**

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

## Nguyên tắc bất biến

```
Unicode:
  ① Mọi Molecule từ encode_codepoint(cp) — KHÔNG viết tay
  ② Mọi chain từ LCA hoặc UCD — KHÔNG viết tay
  ③ Ngôn ngữ tự nhiên = alias → node

Architecture:
  ④ Ghi file TRƯỚC — cập nhật RAM SAU (QT9)
  ⑤ Append-only — KHÔNG DELETE, KHÔNG OVERWRITE
  ⑥ SecurityGate LUÔN chạy trước mọi input
  ⑦ Không đủ evidence → im lặng (BlackCurtain)

Design:
  ⑧ L0 không import L1
  ⑨ Fibonacci xuyên suốt — cấu trúc, threshold, render
  ⑩ Skill stateless — state nằm trong Agent
  ⑪ Worker gửi chain, KHÔNG gửi raw data
  ⑫ Silent by default — không polling, không heartbeat
```

---

## Chạy

```bash
# Build toàn bộ
cargo build --workspace

# Test (1104 tests)
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

## Scale

```
TieredStore: Hot/Warm/Cold tiers + LRU PageCache (Fibonacci: 55/233/610/2584)
LayerIndex:  Bloom filter (256B, 3 hashes) + sorted binary search O(log n)
Compact:     DeltaMolecule (1-6B vs 5B) + ChainDictionary (dedup)
Target:      1 tỷ nodes trên thiết bị phổ thông (RAM < 256MB, disk ~2GB)
```

---

*Unicode 18.0.0 · Rust · no_std core · 2026*
