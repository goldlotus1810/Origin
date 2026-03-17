# HomeOS — Kiến Trúc Kỹ Thuật Chi Tiết

**Ngày:** 2026-03-16

---

## Tổng quan

```
11 crates + 4 tools = 115 Rust files, ~66,000 LoC
Dependency: L0(ucd) → L1(olang) → L2(silk,context) → L3(agents,memory) → L4(runtime)
Không có circular dependency. L0 không import L1.
```

---

## Crate Map

### L0 — Nền tảng

**ucd** — Unicode → Molecule lookup
```
build.rs: Đọc UnicodeData.txt → UCD_TABLE tĩnh lúc compile
lib.rs:   lookup(cp) → Option<Molecule>, 5424 entries
          encode_codepoint(cp) → MolecularChain
          Phân nhóm: SDF(~1344), MATH(~1904), EMOTICON(~1760), MUSICAL(~416)
```

### L1 — Core Language

**olang** — Ngôn ngữ lõi
```
molecular.rs:  Molecule(5 bytes), MolecularChain, EmotionDim
encoder.rs:    encode_codepoint(), encode_text()
lca.rs:        lca(), lca_weighted(), lca_with_variance(), lca_many_with_variance()
registry.rs:   Registry — chain_index, lang_index, tree_index, branch_watermark
writer.rs:     OlangWriter — append-only binary format ○LNG v0.03
reader.rs:     OlangReader — parse_recoverable() cho crash recovery
hash.rs:       chain_hash tự sinh (FNV-based)
knowtree.rs:   KnowTree — L0-Ln hierarchical knowledge structure
vm.rs:         OlangVM — stack-based, Vec<VmEvent> output
ir.rs:         OlangProgram, 26 IR opcodes
compiler.rs:   Backends: C, Rust, WASM
syntax.rs:     Parser → Expr AST
semantic.rs:   Validation + IR generation
math.rs:       Symbolic math: solve/derive/integrate/simplify
constants.rs:  9 hằng số toán học (π,e,φ,√2,...) từ công thức sinh
compact.rs:    DeltaMolecule + ChainDictionary compression
clone.rs:      WorkerPackage export/import
self_model.rs: SystemManifest — ○{stats} self-description
```

### L2 — Neural & Emotion

**silk** — Mạng thần kinh
```
edge.rs:     SilkEdge — EmotionTag per edge, 9 EdgeKinds
graph.rs:    SilkGraph — sorted binary search, co_activate, co_activate_same_layer (QT11)
hebbian.rs:  Hebbian strengthen/decay (φ⁻¹), Fib threshold promote
walk.rs:     walk_weighted() — emotion amplification through graph
```

**context** — Cảm xúc & ngữ cảnh
```
emotion.rs:  EmotionTag (V/A/D/I), sentence_affect(), word_affect() (3000+ từ)
curve.rs:    ConversationCurve — f(x), f'(x), f''(x), window_variance
intent.rs:   IntentKind (Crisis/Learn/Command/Chat), estimate_intent()
fusion.rs:   Cross-modal fusion (Bio>Audio>Text>Image)
```

### L3 — Intelligence

**agents** — Bộ não + kỹ năng
```
encoder.rs:   ContentEncoder — text/audio/sensor/code → chain
learning.rs:  LearningLoop — 5 tầng text learning, Silk co-activation
gate.rs:      SecurityGate — Crisis detect, BlackCurtain, EpistemicFirewall
instinct.rs:  7 bản năng: Honesty→Contradiction→Causality→Abstraction→Analogy→Curiosity→Reflection
leo.rs:       LeoAI — KnowledgeChief + run_instincts() + run_dream()
chief.rs:     HomeChief/VisionChief/NetworkChief — domain routing
worker.rs:    Worker profiles (Camera/Sensor/Actuator/Network/Door)
skill.rs:     Skill trait + ExecContext (QT19-23)
domain_skills.rs: 15 domain skills
```

**memory** — Trí nhớ
```
lib.rs:       ShortTermMemory (512 max, LFU eviction)
dream.rs:     DreamCycle — cluster STM → propose QR, dual-threshold
proposal.rs:  SkillProposal + InsightKind (Causal/Contradiction/...)
              AAM — review_proposal(), review_skill()
```

### L4 — Runtime

**runtime** — Entry point
```
origin.rs:    HomeRuntime — process_text(), ○{} dispatch
parser.rs:    ○{} parser — Query/Compose/Relation/Pipeline
response_template.rs: Render response từ tone + valence
```

**hal** — Hardware Abstraction
```
arch.rs:      Architecture detect (x86/ARM/RISC-V/WASM)
platform.rs:  Platform trait, probe
tier.rs:      Hardware tier system (Tier 1-4)
security.rs:  Security scan
driver.rs:    Driver abstraction
ffi.rs:       FFI bindings
```

**isl** — Inter-System Link
```
address.rs:   ISLAddress [layer:1B][group:1B][subgroup:1B][index:1B]
message.rs:   ISLMessage 12 bytes, 14 MsgTypes
codec.rs:     AES-256-GCM encryption (feature "encrypt")
queue.rs:     ISLQueue — dual deque (urgent + normal)
```

**vsdf** — Visual SDF
```
sdf.rs:       18 SDF generators, analytical gradient ∇f
ffr.rs:       FFR Fibonacci spiral rendering (~89 calls)
physics.rs:   Physics from SDF (collision O(1))
scene.rs:     3D scene graph
fit.rs:       SDF fitting from outline
```

**wasm** — Browser
```
lib.rs:       WASM bindings (wasm-bindgen)
bridge.rs:    WebSocket → ISL bridge
```

---

## Data Flow

```
Input (text/audio/sensor)
  │
  ▼
SecurityGate.check() ── Crisis? → helpline + bypass
  │
  ▼
ContentEncoder → MolecularChain + EmotionTag
  │
  ▼
ContextEngine (infer_context + estimate_intent + ConversationCurve)
  │
  ▼
LearningLoop (STM.push + Silk.co_activate — 5 tầng)
  │
  ▼
LeoAI.run_instincts() → 7 bản năng
  │
  ▼
ConversationCurve → ResponseTone (Supportive/Pause/Reinforcing/Celebratory/Gentle)
  │
  ▼
Response (render text + tone)

[Offline: mỗi 8 turns]
  DreamCycle → cluster STM → propose QR → AAM approve → LTM
```

---

## File Format

```
origin.olang — append-only binary
  Header: [○LNG] [0x03] [created_ts:8]  = 13 bytes
  Records:
    0x01 = Node  [chain_hash:8] [layer:1] [is_qr:1] [ts:8]
    0x02 = Edge  [from:8] [to:8] [rel:1] [ts:8]
    0x03 = Alias [chain_hash:8] [lang:2] [name_len:2] [name:N]

origin.olang.weights  — Hebbian weights (append-only)
origin.olang.registry — chain index (rebuild được từ origin.olang)
log.olang             — event log (append-only)
```

---

## QT Compliance Notes

**QT4 (Molecule from encode_codepoint):**
VM `PushMol`, VSDF `FFRCell::to_molecule()`, và LCA result construction
tạo Molecule ngoài `encode_codepoint()`. Đây là tính toán lúc chạy,
không phải giá trị viết tay — chấp nhận được.

**QT11 (Silk chỉ ở Ln-1):**
`SilkGraph::co_activate_same_layer()` kiểm tra tầng tại API boundary.
`SilkGraph::co_activate()` vẫn yêu cầu caller đảm bảo cùng tầng.
`co_activate_cross_layer()` cho phép kết nối khác tầng với Fib[n+2] threshold.

---

## Benchmark Targets

```
lookup()                < 1μs
LCA()                   < 10μs
boot                    < 100ms ARM
Silk walk 100 edges     < 1ms
Hebbian update          < 100μs
ConversationCurve       < 50μs
ContentEncoder text     < 5ms/sentence
word_affect()           < 1μs/word
FFR 89 calls            < 16ms (60fps)
SDF evaluate            < 100ns/point
```

---

*2026-03-16 · HomeOS Architecture*
