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
├── olang/      Core: Molecule · LCA · Registry · VM · Compact · KT   1088 tests
│   └── src/
│       ├── core/       Molecule, MolecularChain, LCA, encoder
│       ├── storage/    Registry, Writer, Reader, Compact, KnowTree
│       └── execution/  VM, IR, Compiler, Syntax, Semantic
├── silk/       Hebbian learning · EmotionTag edges · Walk · parent_map   85 tests
├── context/    Emotion V/A/D/I · ConversationCurve · Intent            168 tests
│   └── src/
│       ├── emotion/    EmotionTag, WordAffect, phrase blending
│       └── pipeline/   Engine, Curve, Intent, Fusion, Snapshot
├── agents/     Encoder · Learning · Gate · Instinct · Chief/Worker     284 tests
│   └── src/
│       ├── core/       Encoder, Learning loop, SecurityGate
│       ├── intelligence/ LeoAI, 7 Instincts, Domain Skills
│       └── hierarchy/  Chief, Worker, ISL routing
├── memory/     STM · DreamCycle · Proposals · AAM                       32 tests
├── runtime/    HomeRuntime · ○{} Parser · Router                       273 tests
├── hal/        Hardware Abstraction · Tier · FFI · Security             68 tests
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

### Nguyên lý: Silk = hệ quả tự nhiên của 5D

```
Silk KHÔNG CẦN LƯU EDGE.
Silk = "2 node chia sẻ cùng công thức trên chiều nào?"
     = lookup trong SilkIndex.

Emotion không phải metadata trên edge.
Emotion LÀ 2 TRONG 5 CHIỀU của node (V + A).
"Cùng cảm xúc" = cùng công thức V hoặc A = TỰ ĐỘNG Silk.

5400 công thức L0 → mỗi công thức = 1 "nhóm máu"
Cùng nhóm máu trên chiều nào → Silk trên chiều đó.
```

### 3 tầng Silk (horizontal, implicit, 0 bytes)

| Tầng | Tên | Cách hoạt động | Số lượng | Status |
|------|-----|----------------|---------|--------|
| Base | 37 kênh (8S+8R+8V+8A+5T) | Cùng base value = cùng "nhóm máu" | 37 | ✅ SilkIndex |
| Compound | 31 mẫu C(5,k) k=1..5 | Chia sẻ k chiều cùng lúc = kiểu quan hệ | 31 | ✅ CompoundKind enum |
| Precise | ~5400 kênh (= số L0 nodes) | Cùng variant chính xác = match hoàn hảo | ~5400 | SPEC — chưa implement |

```
Công thức sức mạnh kết nối:
  strength(A, B) = Σ match(dim) × precision(dim)
  match(dim)     = 1 nếu cùng base, 0 nếu khác
  precision(dim) = 1.0 nếu cùng variant, 0.5 nếu chỉ cùng base

Strength = number of shared dimensions:
  1 dim shared = 0.20 (loosely related)    → C(5,1) =  5 patterns
  2 dims       = 0.40 (clearly related)    → C(5,2) = 10 patterns
  3 dims       = 0.60 (near identical)     → C(5,3) = 10 patterns
  4 dims       = 0.80 (almost same concept)→ C(5,4) =  5 patterns
  5 dims       = 1.00 (same node)          → C(5,5) =  1 pattern

37 kênh × 31 mẫu = 1147 kiểu quan hệ có nghĩa

CompoundKind examples:
  ShapeValence     = "trông giống + cảm giống" → visual metaphor
  RelationValence  = "quan hệ giống + cảm giống" → moral analog
  ValenceArousal   = "cùng trạng thái cảm xúc" → empathy link
  AllButShape      = "khác hình, giống HẾT còn lại" → deep metaphor
```

### 2 Hướng Silk

| Hướng | Tên | Lưu trữ | Status |
|-------|-----|---------|--------|
| Ngang | Silk tự do (implicit, cùng tầng) | 0 bytes | ✅ SilkIndex |
| Dọc | Silk đại diện (parent pointer) | 5460 × 8B = 43 KB | ✅ parent_map |

### Vertical Silk (parent pointer, 43 KB)

```
SilkGraph.parent_map: BTreeMap<u64, u64>  (child_hash → parent_hash)

L1→L0:  5400 pointers  (each UCD atom → L1 representative)
L2→L1:    37 pointers
L3→L2:    12 pointers
L4→L3:     5 pointers
L5→L4:     3 pointers
L6→L5:     2 pointers
L7→L6:     1 pointer
─────────────────────
Tổng: 5460 × 8B = 43 KB

API:
  register_parent(child, parent)  → đăng ký parent pointer
  parent_of(hash)                 → Option<u64>
  children_of(parent)             → Vec<u64>
  layer_of(hash)                  → u8 (depth via parent chain)

Query: O(1) via parent chain traversal
  "🔥 related to ∈?" → compare 5D + check shared parent at L1
```

### Learned Silk (Hebbian, explicit)

```
Hebbian = phát hiện cái đã có, không tạo cái mới.
co_activate(a, b) → strengthen awareness of implicit relationship
HebbianLink: (hash_a, hash_b, weight, fire_count, emotion_tag)

3 query strategies merged by unified_neighbors():
  1. implicit_silk()    → 5D dimensional comparison (0 bytes, computed)
  2. learned (Hebbian)  → co-activation weights (explicit edges)
  3. structural         → legacy SilkEdge (explicit edges)
  → unified_neighbors() ranks and merges all 3 sources
```

---

## Node Architecture

### Molecule = Công thức, không phải Giá trị

```
Mỗi byte trong Molecule = tham chiếu đến công thức gốc L0:

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

### NodeState = Molecule + Maturity + Origin

```rust
NodeState {
    mol: Molecule,               // 5D coordinate
    maturity: Maturity,          // Formula → Evaluating → Mature
    origin: CompositionOrigin,   // how was this node created?
}
```

### CompositionOrigin — traceability

```
Innate(u32)                          → L0 node from encode_codepoint()
Composed { sources: [u64], op }      → LCA/Fuse/Program tổ hợp nhiều nodes
Evolved { source, dim, old, new }    → evolve() mutate 1/5 chiều

ComposeOp: Lca | Fuse | Program

Wire points:
  lca_with_origin()  → returns (LcaResult, CompositionOrigin::Composed)
  lca_to_node_state() → returns Option<NodeState>
  evolve()           → EvolveResult includes CompositionOrigin::Evolved
```

### Maturity Lifecycle (wired)

```
  Formula     → node created, no real input yet (5 potential functions)
  Evaluating  → fire_count ≥ fib(depth), accumulating evidence
  Mature      → weight ≥ 0.854 && fire_count ≥ fib(depth), ready for QR

  STM.push()  → fire_count++ → advance(fire_count, heuristic_weight, fib_threshold)
  Dream.run() → maturity check before QR promote + neighbor_bonus via unified_neighbors()
  QR promote  → only if Mature → append-only, signed, permanent knowledge
```

### Dream Clustering (5D-aware, layer-filtered)

```
cluster_score(a, b):
  α × MolSummary::similarity()      ← 5D molecular similarity (not byte-level)
    + implicit_silk bonus            ← 5D shared dimensions
  + β × Hebbian weight              ← co-activation strength
  + γ × co_activation score         ← fire together count

Layer enforcement (QT⑪):
  Observations grouped by layer before clustering
  → Dream never clusters L0 with L2

QR promote gate:
  maturity == Mature required
  neighbor_bonus from unified_neighbors() boosts confidence
```

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

*HomeOS · ~82K LoC Rust · 2,348 tests · 0 clippy warnings · 0 external deps · no_std core*
