# HomeOS — Architecture

> Molecule = 5 bytes = FORMULA, not data.
> Every concept = a coordinate in 5D space.
> From that coordinate, EVERYTHING is COMPUTABLE.

---

## Core Principle: 5D Molecular Space

```
MolecularChain = [Shape][Relation][Valence][Arousal][Time]
                  1 byte  1 byte    1 byte   1 byte  1 byte = 5 bytes

Group       Chars    Dimension    Meaning
────────────────────────────────────────────────────
SDF         ~1344    Shape        "What does it look like" (● ▬ ■ ▲ ○ ∪ ∩ ∖)
MATH        ~1904    Relation     "How does it connect" (∈ ⊂ ≡ ⊥ ∘ → ≈ ←)
EMOTICON    ~1760    Valence+A    "How does it feel" (0x00..0xFF × 2)
MUSICAL     ~416     Time         "How does it change" (Static → Instant)
────────────────────────────────────────────────────
Total       ~5424    5 dims       = HomeOS genome from Unicode 18.0
```

Each node generates 3 formulas:
```
Molecule [S][R][V][A][T]
    ├── SDF      → shape formula (visible — renderable)
    ├── Spline   → transformation formula (invisible — 6 curves)
    └── Silk     → relationship formula (connections — 0 bytes implicit)
```

---

## Processing Pipeline

```
User input → runtime::HomeRuntime.process_text()
                 │
                 ├─ ○{...} → Parser → IR → VM → Response (OlangResult)
                 │
                 └─ natural text → Emotion Pipeline 7 layers:
                      T1: infer_context()        ← boundary conditions
                      T2: sentence_affect()      ← raw emotion from words
                      T3: ctx.apply()            ← scale by context
                      T4: estimate_intent()      ← Crisis/Learn/Command/Chat
                      T5: Crisis check           ← STOP if dangerous
                      T6: learning.process_one() ← Encode → STM → Silk
                      T7: render response        ← tone from ConversationCurve
```

---

## Agent Hierarchy

```
USER
    ↓
AAM  [tier 0] — stateless · approve · final decision
    ↓ ISL
LeoAI       [tier 1] — Knowledge + Learning + Dream + 7 instincts
HomeChief   [tier 1] — manages home device Workers
VisionChief [tier 1] — manages camera/sensor Workers
NetworkChief[tier 1] — manages network/security Workers
    ↓ ISL
Workers     [tier 2] — SILENT · right skill for right job · report chain

Communication rules:
  ✅ AAM ↔ Chief     ✅ Chief ↔ Chief     ✅ Chief ↔ Worker
  ❌ AAM ↔ Worker    ❌ Worker ↔ Worker

Biology analogy:
  Worker = peripheral neuron    Chief = spinal cord
  LeoAI  = brain                AAM   = consciousness
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

## Crate Map

```
crates/
├── ucd/        Unicode → Molecule lookup (build.rs, 5424 entries)       23 tests
├── olang/      Core: Molecule · LCA · Registry · VM · Compact · KT    838 tests
│   └── src/
│       ├── core/       Molecule, MolecularChain, LCA, encoder
│       ├── storage/    Registry, Writer, Reader, Compact, KnowTree
│       └── execution/  VM, IR, Compiler, Syntax, Semantic
├── silk/       Hebbian learning · EmotionTag edges · Walk               88 tests
├── context/    Emotion V/A/D/I · ConversationCurve · Intent            168 tests
│   └── src/
│       ├── emotion/    EmotionTag, WordAffect, phrase blending
│       └── pipeline/   Engine, Curve, Intent, Fusion, Snapshot
├── agents/     Encoder · Learning · Gate · Instinct · Chief/Worker     282 tests
│   └── src/
│       ├── core/       Encoder, Learning loop, SecurityGate
│       ├── intelligence/ LeoAI, 7 Instincts, Domain Skills
│       └── hierarchy/  Chief, Worker, ISL routing
├── memory/     STM · DreamCycle · Proposals · AAM                       65 tests
├── runtime/    HomeRuntime · ○{} Parser · Router                       273 tests
├── hal/        Hardware Abstraction · Tier · FFI · Security             85 tests
├── isl/        Inter-System Link messaging (AES-256-GCM)                31 tests
├── vsdf/       18 SDF generators · FFR · Physics · SceneGraph          123 tests
│   └── src/
│       ├── rendering/  SDF primitives, FFR Fibonacci, Vector math
│       └── world/      Scene graph, Physics, Occlusion, Body
├── wasm/       WebAssembly bindings · WebSocket-ISL bridge              32 tests
└── homemath/   Zero-dep pure-Rust math (no libm)                        18 tests

tools/
├── seeder/     Seed L0 nodes from UCD (0 hardcode)                      15 tests
├── server/     Terminal REPL (stdin/stdout)                              13 tests
├── inspector/  Read/verify origin.olang                                   9 tests
└── bench/      Performance benchmarks
```

---

## Silk Architecture — 3 Tầng × 2 Hướng

> Silk = hệ quả toán học tự nhiên của không gian 5D, không phải edge list.
> Emotion KHÔNG phải metadata trên edge — Emotion LÀ 2 TRONG 5 CHIỀU của node (V + A).
> "Cùng cảm xúc" = cùng công thức V hoặc A = TỰ ĐỘNG Silk.

### 3 Tầng Silk

| Tầng | Tên | Cách hoạt động | Số lượng | Status |
|------|-----|----------------|---------|--------|
| Base | 37 kênh (8S+8R+8V+8A+5T) | Cùng base value = cùng "nhóm máu" | 37 | ✅ SilkIndex |
| Compound | 31 mẫu C(5,k) | Chia sẻ k chiều cùng lúc = kiểu quan hệ | 31 | ❌ CompoundKind enum |
| Precise | ~5400 kênh | Cùng variant chính xác = match hoàn hảo | ~5400 | ❌ chưa implement |

```
Công thức sức mạnh kết nối:
  strength(A, B) = Σ match(dim) × precision(dim)
  match(dim)     = 1 nếu cùng base, 0 nếu khác
  precision(dim) = 1.0 nếu cùng variant, 0.5 nếu chỉ cùng base

37 kênh × 31 mẫu = 1147 KIỂU quan hệ có nghĩa
```

### 31 Compound Patterns

```
1 chiều chung:  C(5,1) =  5 mẫu → "liên quan nhẹ"
2 chiều chung:  C(5,2) = 10 mẫu → "liên quan rõ"
3 chiều chung:  C(5,3) = 10 mẫu → "gần giống"
4 chiều chung:  C(5,4) =  5 mẫu → "gần như cùng"
5 chiều chung:  C(5,5) =  1 mẫu → "cùng node"

Ví dụ:
  S+V       = "trông giống + cảm giống"         → ẩn dụ thị giác
  R+V       = "quan hệ giống + cảm giống"       → moral analog
  V+A       = "cùng trạng thái cảm xúc"         → empathy link
  S+R+V     = "hình + quan hệ + cảm xúc giống"  → gần như cùng khái niệm
  AllButS   = "khác hình, giống HẾT còn lại"    → ẩn dụ sâu
```

### 2 Hướng Silk

| Hướng | Tên | Lưu trữ | Status |
|-------|-----|---------|--------|
| Ngang | Silk tự do (implicit, cùng tầng) | 0 bytes | ✅ SilkIndex |
| Dọc | Silk đại diện (parent pointer) | 5460 × 8B = 43 KB | ❌ chưa implement |

### Vertical Silk — Parent Pointer (43 KB)
```
Mỗi node tại Lx là ĐẠI DIỆN cho 1 nhóm ở Lx-1.
Mỗi node chỉ cần 1 pointer đến parent.

L1→L0:  5400 pointers
L2→L1:    37 pointers
L3→L2:    12 pointers
L4→L3:     5 pointers
L5→L4:     3 pointers
L6→L5:     2 pointers
L7→L6:     1 pointer
─────────────────────
Tổng: 5460 × 8B = 43 KB

Truy vấn O(1): 2 lookup đại diện + 1 so sánh 5D
  "🔥 related to ∈?" → compare 5D + check shared parent at L1
```

### Hebbian = Phát hiện cái đã có, không Tạo cái mới
```
Silk fire together, wire together — không phải vì ai nối chúng lại,
mà vì chúng đã ở cùng vị trí trong không gian 5D từ đầu.
co_activate() PHÁT HIỆN quan hệ implicit, không TẠO mới.
```

---

## Node = Molecule + Lifecycle

### Nguyên lý: Molecule = Công thức, không phải Giá trị

```
Mỗi byte trong Molecule = THAM CHIẾU đến công thức gốc L0:
  Shape    = f_s(inputs...)    ← công thức hình dạng
  Relation = f_r(inputs...)    ← công thức quan hệ
  Valence  = f_v(inputs...)    ← công thức cảm xúc
  Arousal  = f_a(inputs...)    ← công thức cường độ
  Time     = f_t(inputs...)    ← công thức thời gian

  Chưa có input → TIỀM NĂNG    Có input → GIÁ TRỊ CỤ THỂ    Đủ → node CHÍN

Hệ quả:
  Dream    = đánh giá công thức nào đã "chín" → promote QR
  LeoAI    = tổ hợp công thức A ∘ B → công thức C mới
  evolve() = thay 1 biến trong công thức → loài mới
  16GB     = 100M concept × 7 bytes công thức = 700 MB (vs TB nếu lưu giá trị)
```

### Node Maturity Lifecycle

```
  Formula     → node mới, chưa có input thật (5 công thức tiềm năng)
  Evaluating  → fire_count > 0, đang tích lũy evidence
  Mature      → weight ≥ 0.854 && fire_count ≥ fib(depth), sẵn sàng QR

Wire points:
  STM.push()    → advance(fire_count, weight, fib_threshold)
  Dream.run()   → DreamResult.matured_nodes = Vec<u64>
  QR promote    → append-only, signed, permanent
```

**Status:** Maturity enum ✅ | advance() ✅ | Wire to STM ❌ | Wire to Dream ❌
**Bug:** advance(weight=0.0) → Mature unreachable (SPEC_MATURITY_PIPELINE)

### Node Complete Lifecycle
```
encode_codepoint(cp)
    → MolecularChain (5 bytes)
    → OlangWriter.append_node()      ← GHI FILE TRƯỚC (QT9)
    → Registry.insert_with_kind()    ← cập nhật RAM SAU
    → Observation (fire_count=1, maturity=Formula)
    → Silk.co_activate()

Repeated co-activation:
    fire_count++ → maturity: Formula → Evaluating
    Hebbian weight ≥ 0.854 + fire_count ≥ Fib[depth] → Mature

DreamCycle.run():
    cluster similar Observations (cùng layer — QT⑪)
    LCA(cluster) → new MolecularChain
    DreamProposal → AAM.review() → Approved
    → QR write (is_qr=true, ED25519 signed, append-only forever)
```

### Composition Origin (SPEC — chưa implement)
```
L0 node  → Innate(codepoint)           — từ encode_codepoint()
Composite → Composed{sources, op}       — từ LCA / Fuse / Program
Evolved  → Evolved{source, dim, old, new} — từ evolve()

Lợi ích: trace "concept này sinh từ L0 nào?", re-evaluate khi L0 thay đổi
```

---

## Key Subsystems

### Emotion Pipeline (5 layers of learning from text)
```
1. Paragraph  → paragraph_emotion
2. Sentence   → split punctuation, blend 50% paragraph + 50% word
3. Word       → word_affect() from 3000+ word lexicon, Silk co-activate
4. Phrase     → sliding window 5 words, proximity decay
5. Character  → Unicode chain (L0 innate)
```

### Silk Amplification (NEVER average — always AMPLIFY)
```
amplify_emotion(emo, weight) → emo * (1.0 + weight * factor)
"sad" + "lost job" co-activate weight=0.90
→ composite V=-0.85 (heavier than individual -0.65)
```

### ConversationCurve
```
f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
f'  < -0.15          → Supportive
f'' < -0.25          → Pause
f'  > +0.15          → Reinforcing
f'' > +0.25 && V > 0 → Celebratory
V < -0.20, stable    → Gentle
```

### LeoAI — 7 Innate Instincts (L0, no learning required)
```
Priority: Honesty → Contradiction → Causality → Abstraction → Analogy → Curiosity → Reflection
Honesty ALWAYS runs first: insufficient evidence → silence
```

### ISL — Inter-System Link
```
ISLAddress: [layer:1B][group:1B][subgroup:1B][index:1B] = 4 bytes
ISLMessage: [from:4B][to:4B][msg_type:1B][payload:3B]  = 12 bytes
ISLFrame:   12B header + 2B length + variable body
Encryption: AES-256-GCM
```

### Olang VM — 36 Opcodes
```
Stack:    Push Load Dup Pop Swap PushNum PushMol Store StoreUpdate LoadLocal
Control:  Jmp Jz Loop Call Ret ScopeBegin ScopeEnd TryBegin CatchEnd Halt Nop
Chain:    Lca Edge Query Emit Fuse
System:   Dream Stats
Debug:    Trace Inspect Assert TypeOf Why Explain
```

---

## File Format

```
origin.olang — append-only binary
  Header: [○LNG] [0x05] [created_ts:8]  = 13 bytes
  Records:
    0x01 = Node  [chain_hash:8] [layer:1] [is_qr:1] [ts:8]
    0x02 = Edge  [from:8] [to:8] [rel:1] [ts:8]
    0x03 = Alias [chain_hash:8] [lang:2] [name_len:2] [name:N]

Tagged Sparse Encoding (v0.05):
  [presence_mask: 1B][non-default values: 0-5B]
  Defaults skipped: S=Sphere, R=Member, V=0x80, A=0x80, T=Medium
```

---

## Scale Target

```
1 concept = ~33 bytes (5 mol + 8 hash + 20 metadata)
500M concepts = 16.5 GB → fits ONE PHONE

TieredStore: Hot/Warm/Cold + LRU PageCache (Fibonacci: 55/233/610/2584)
LayerIndex:  Bloom filter (256B, 3 hashes) + sorted binary search O(log n)
Compact:     DeltaMolecule (1-6B vs 5B) + ChainDictionary (dedup)
```

---

*HomeOS · ~82K LoC Rust · 2,227 tests · 0 clippy warnings · 0 external deps · no_std core*
