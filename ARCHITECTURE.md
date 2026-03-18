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

*HomeOS · ~82K LoC Rust · 2,063 tests · 0 clippy warnings · 0 external deps · no_std core*
