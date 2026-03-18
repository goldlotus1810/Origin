# HomeOS API Reference

**Version:** 0.1.0 · **Updated:** 2026-03-18

---

## Quick Start

```rust
use runtime::origin::HomeRuntime;

let mut rt = HomeRuntime::new(timestamp_ms);
let response = rt.process_text("xin chào", timestamp);
println!("{} [{}]", response.text, response.tone);
```

```javascript
// WASM (browser)
import init, { HomeOSWasm } from './homeos_wasm.js';
await init();
const os = new HomeOSWasm();
const r = JSON.parse(os.process("○{stats}"));
```

---

## Crate Overview

| Crate | Purpose | Entry Point |
|-------|---------|------------|
| **runtime** | Main entry, ○{} parser, response render | `HomeRuntime` |
| **olang** | Molecule, MolecularChain, Registry, VM, Compiler | `encode_codepoint()`, `Registry`, `VM` |
| **silk** | Hebbian graph, implicit 5D connections | `SilkGraph` |
| **context** | Emotion V/A/D/I, ConversationCurve, Intent | `EmotionTag`, `ConversationCurve` |
| **agents** | Learning pipeline, BookReader, LeoAI, Chief/Worker | `LearningLoop`, `BookReader` |
| **memory** | STM, DreamCycle, Proposals, AAM | `DreamConfig`, `DreamProposal` |
| **isl** | Inter-system messaging (4-byte addressing) | `ISLAddress`, `ISLMessage` |
| **ucd** | Unicode 18.0 → Molecule lookup (build-time) | `ucd::lookup()`, `ucd::table_len()` |
| **vsdf** | SDF generators, FFR render, NodeBody | `SdfKind`, `FFRCell`, `NodeBody` |
| **wasm** | Browser WASM bindings | `HomeOSWasm` |
| **hal** | Hardware abstraction, platform probe | `HalProbe`, `Capability` |

---

## runtime — HomeRuntime

### `HomeRuntime`

```rust
pub struct HomeRuntime { /* ... */ }

impl HomeRuntime {
    pub fn new(ts: u64) -> Self;
    pub fn with_file(ts: u64, data: Option<&[u8]>) -> Self;

    // Core processing
    pub fn process_text(&mut self, input: &str, ts: i64) -> Response;
    pub fn read_book(&mut self, text: &str, ts: i64) -> usize;

    // State getters
    pub fn fx(&self) -> f32;                  // ConversationCurve f(x)
    pub fn tone(&self) -> ResponseTone;       // Current emotional tone
    pub fn stm_len(&self) -> usize;           // STM observation count
    pub fn silk_edge_count(&self) -> usize;   // Total Silk edges
    pub fn silk_node_count(&self) -> usize;   // Distinct Silk nodes
    pub fn silk_edges_from(&self, hash: u64) -> usize;
    pub fn registry_len(&self) -> usize;      // Registered node count
    pub fn registry_alias_count(&self) -> usize;
}
```

### `Response`

```rust
pub struct Response {
    pub text: String,
    pub tone: ResponseTone,
    pub fx: f32,
    pub kind: ResponseKind,
}

pub enum ResponseKind { Natural, OlangResult, Crisis, Blocked, System }
pub enum ResponseTone { Supportive, Pause, Reinforcing, Celebratory, Gentle, Engaged }
```

### ○{} Commands

| Command | Example | Description |
|---------|---------|-------------|
| `stats` | `○{stats}` | System statistics |
| `dream` | `○{dream}` | Run dream consolidation cycle |
| `help` | `○{help}` | List all commands |
| `trace` | `○{trace}` | Trace last processing path |
| Compose | `○{lửa ∘ nước}` | LCA composition |
| Relation | `○{🔥 ∈ ?}` | Query relations |
| Math | `○{1+2}` | Arithmetic |
| `learn` | `○{learn "concept"}` | Learn text into STM |
| `seed` | `○{seed L0}` | Seed L0 nodes from UCD |
| `typeof` | `○{typeof "lửa"}` | Show type info |
| `inspect` | `○{inspect "lửa"}` | Inspect node chain |
| `explain` | `○{explain "concept"}` | Explain origin path |
| `why` | `○{why "A" "B"}` | Explain connection |
| `assert` | `○{assert "A" == "B"}` | Assert truth |
| `read` | `○{read "long text..."}` | BookReader pipeline |
| `compile` | `○{compile rust <src>}` | Compile to target |
| `ingest` | `○{ingest "text"}` | IngestSkill processing |
| `delta` | `○{delta "A" "B"}` | 5D delta between concepts |
| `hebbian` | `○{hebbian "A" "B"}` | Show Hebbian weight |
| `merge` | `○{merge "A" "B"}` | MergeSkill combination |
| `fit` | `○{fit edge=3 circ=0.8}` | InverseRender SDF fitting |
| `prune` | `○{prune}` | PruneSkill on STM |
| `curate` | `○{curate}` | CuratorSkill ranking |
| `temporal` | `○{temporal}` | TemporalPattern detection |

---

## olang — Core Types

### `Molecule` (5 bytes)

```rust
pub struct Molecule {
    pub shape: ShapeBase,       // 8 primitives
    pub relation: RelationBase, // 8 relations
    pub valence: u8,            // 0x00..0xFF
    pub arousal: u8,            // 0x00..0xFF
    pub time: TimeDim,          // 5 tempos
}

impl Molecule {
    pub fn evolve(&self, dim: MolDimension, value: u8) -> EvolveResult;
    pub fn to_tagged_bytes(&self) -> Vec<u8>;   // sparse: 1-6 bytes
    pub fn from_tagged_bytes(b: &[u8]) -> Option<Self>;
    pub fn dimension_delta(&self, other: &Self) -> DimensionDelta;
}
```

### `MolecularChain`

```rust
pub struct MolecularChain { /* Vec<Molecule> + cached hash */ }

impl MolecularChain {
    pub fn chain_hash(&self) -> u64;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn similarity_full(&self, other: &Self) -> f32;
    pub fn as_bytes(&self) -> Vec<u8>;
}

// Create from Unicode
pub fn encode_codepoint(cp: u32) -> MolecularChain;

// Combine chains
pub fn lca(chains: &[&MolecularChain]) -> MolecularChain;
pub fn lca_many(chains: &[MolecularChain]) -> MolecularChain;
```

### `Registry`

```rust
pub struct Registry { /* BTreeMap<u64, RegistryEntry> */ }

impl Registry {
    pub fn new() -> Self;
    pub fn insert(&mut self, chain_hash: u64, layer: u8, is_qr: bool, ts: i64);
    pub fn insert_with_kind(&mut self, chain_hash: u64, layer: u8, is_qr: bool, ts: i64, kind: NodeKind);
    pub fn lookup_hash(&self, hash: u64) -> Option<&RegistryEntry>;
    pub fn lookup_name(&self, name: &str) -> Option<u64>;
    pub fn add_alias(&mut self, hash: u64, lang: &str, name: &str);
    pub fn len(&self) -> usize;
    pub fn alias_count(&self) -> usize;
    pub fn entries_by_kind(&self, kind: NodeKind) -> Vec<&RegistryEntry>;
}

pub enum NodeKind {
    Alphabet, Knowledge, Memory, Agent, Skill,
    Program, Device, Sensor, Emotion, System,
}
```

### VM & Compiler

```rust
pub struct VM { /* stack machine, 36 opcodes */ }

impl VM {
    pub fn new(registry: &Registry, graph: &SilkGraph) -> Self;
    pub fn execute(&mut self, ops: &[Op]) -> Vec<VmEvent>;
}

pub struct Compiler { /* multi-target */ }

impl Compiler {
    pub fn compile_to_c(source: &str) -> Result<String, CompileError>;
    pub fn compile_to_rust(source: &str) -> Result<String, CompileError>;
    pub fn compile_to_wasm(source: &str) -> Result<Vec<u8>, CompileError>;
}
```

---

## silk — SilkGraph

```rust
pub struct SilkGraph { /* implicit + hebbian edges */ }

impl SilkGraph {
    pub fn new() -> Self;

    // Indexing (implicit 5D)
    pub fn index_node(&mut self, hash: u64, mol: &MolSummary);
    pub fn unified_neighbors(&self, hash: u64, mol: Option<&MolSummary>) -> Vec<SilkNeighbor>;
    pub fn unified_weight(&self, from: u64, to: u64, ...) -> f32;

    // Hebbian learning
    pub fn learn(&mut self, from: u64, to: u64, reward: f32);
    pub fn learn_mol(&mut self, from: u64, to: u64, ..., reward: f32);
    pub fn learned_weight(&self, from: u64, to: u64) -> f32;
    pub fn decay_learned(&mut self, elapsed_ns: i64);

    // Co-activation (fire together → wire together)
    pub fn co_activate(&mut self, a: u64, b: u64, weight: f32, emotion: EmotionTag);
    pub fn co_activate_same_layer(&mut self, a: u64, b: u64, layer: u8, weight: f32, emotion: EmotionTag);

    // Parent pointers (vertical Silk)
    pub fn register_parent(&mut self, child: u64, parent: u64);
    pub fn parent_of(&self, hash: u64) -> Option<u64>;
    pub fn children_of(&self, parent: u64) -> Vec<u64>;
    pub fn layer_of(&self, hash: u64) -> u8;

    // Stats
    pub fn len(&self) -> usize;
    pub fn node_count(&self) -> usize;
    pub fn edges_from(&self, hash: u64) -> Vec<(u64, f32)>;
}

pub struct SilkNeighbor {
    pub hash: u64,
    pub weight: f32,      // combined
    pub implicit: f32,
    pub hebbian: f32,
    pub shared_dims: u8,
}

pub struct MolSummary {
    pub shape: u8, pub relation: u8,
    pub valence: u8, pub arousal: u8, pub time: u8,
}
```

---

## context — Emotion & Intent

### `EmotionTag`

```rust
pub struct EmotionTag {
    pub valence: f32,   // -1.0..1.0 (negative..positive)
    pub arousal: f32,   // 0.0..1.0 (calm..excited)
    pub dominance: f32, // 0.0..1.0
    pub intensity: f32, // 0.0..1.0
}
```

### `ConversationCurve`

```rust
pub struct ConversationCurve { /* tracks emotion across turns */ }

impl ConversationCurve {
    pub fn new() -> Self;
    pub fn push(&mut self, valence: f32) -> f32;  // returns f(x)
    pub fn fx(&self) -> f32;
    pub fn tone(&self) -> ResponseTone;
    pub fn is_unstable(&self) -> bool;
}
```

### Intent System

```rust
pub enum IntentKind {
    Crisis, Learn, Command, Chat, Confirm, Question,
}

pub enum IntentAction {
    CrisisResponse, Proceed, LearnExplicit,
    UserConfirm, QuestionAnswer, HomeControl,
}

pub fn estimate_intent(text: &str, emotion: &EmotionTag) -> IntentEstimate;
pub fn decide_action(est: &IntentEstimate) -> IntentAction;
```

---

## memory — Dream & Proposals

### `DreamConfig`

```rust
pub struct DreamConfig {
    pub scan_top_n: usize,          // default 32
    pub cluster_threshold: f32,     // default 0.6, conversation 0.30
    pub min_cluster_size: usize,    // default 3, conversation 2
    pub tree_depth: usize,          // default 3, conversation 2
    pub alpha: f32,                 // chain similarity weight (0.3)
    pub beta: f32,                  // hebbian weight (0.4)
    pub gamma: f32,                 // co-activation weight (0.3)
}

impl DreamConfig {
    pub fn default() -> Self;
    pub fn for_conversation() -> Self;
    pub fn with_weights(a: f32, b: f32, g: f32) -> Self;
}
```

### `DreamProposal`

```rust
pub enum ProposalKind {
    NewNode { chain: MolecularChain, emotion: EmotionTag, sources: Vec<u64> },
    PromoteQR { chain_hash: u64, fire_count: u32 },
    NewEdge { from_hash: u64, to_hash: u64, edge_kind: u8 },
    SupersedeQR { old_hash: u64, new_hash: u64, reason: String },
}
```

---

## isl — Inter-System Link

```rust
pub struct ISLAddress([u8; 4]);  // [layer, group, subgroup, index]

impl ISLAddress {
    pub fn new(layer: u8, group: u8, subgroup: u8, index: u8) -> Self;
    pub const ROOT: Self;       // [0,0,0,0]
    pub const BROADCAST: Self;  // [FF,FF,FF,FF]
    pub fn to_u32(self) -> u32;
    pub fn from_u32(v: u32) -> Self;
}

pub struct ISLMessage {
    pub from: ISLAddress,
    pub to: ISLAddress,
    pub msg_type: MsgType,
    pub payload: [u8; 3],
}

pub enum MsgType {
    Text, Query, Learn, Propose, ActuatorCmd, Tick,
    Dream, Emergency, Approved, Broadcast, ChainPayload,
    Ack, Nack, Program,
}
```

---

## wasm — Browser Bindings

### `HomeOSWasm` (JavaScript API)

```javascript
const os = new HomeOSWasm();

// Core
os.process(input)       // → JSON {text, tone, fx, kind, turn}
os.read_book(text)      // → sentence count
os.dream()              // → JSON response
os.stats()              // → JSON response

// Getters
os.fx                   // f32 — conversation curve
os.turns                // u64 — turn count
os.stm_len              // u32 — STM observations
os.silk_edge_count      // u32 — Silk edges
os.silk_node_count      // u32 — Silk nodes
os.registry_len         // u32 — registered nodes
os.pending_events       // u32 — pending event frames
os.tone()               // string

// Events (binary WebSocket push)
os.drain_events()       // → Uint8Array [count:4BE][len:4BE][frame]...

// Static
HomeOSWasm.ucd_len()    // u32 — UCD table size
HomeOSWasm.encode_cp(cp) // u32 — chain hash

// Globals
version()               // string
quick_encode(cp)        // u32
create_homeos()         // factory
```

### Bridge Protocol (WebSocket)

```
Frame: [magic:2 "OS"] [type:1] [len:2BE] [payload:N]

Types:
  0x01 TextInput    0x10 Response      0x20 Stats
  0x02 OlangInput   0x11 EmotionUpdate 0x21 Health
  0x03 AudioInput   0x12 DreamResult   0xFE Ping
                    0x13 SilkUpdate    0xFF Pong
                    0x14 SceneUpdate
```

---

## Build & Test

```bash
cargo build --workspace          # Build all
cargo test --workspace           # Run ~1800 tests
cargo clippy --workspace         # Lint (must be 0 warnings)
cargo test -p runtime            # Test single crate

# WASM build (requires wasm-pack)
wasm-pack build crates/wasm --target web
# Output: crates/wasm/pkg/homeos_wasm.js + .wasm

# Run REPL
cargo run -p server
```

---

## Invariant Rules (must follow)

1. All Molecules from `encode_codepoint()` or `lca()` — never hand-craft
2. Emotion always goes through FULL pipeline — never average, always amplify via Silk
3. SecurityGate runs BEFORE everything
4. Append-only storage — never delete, never overwrite
5. Workers send MolecularChain, never raw data
6. Skills are stateless — state lives in Agent
7. Not enough evidence → silence (BlackCurtain)

---

*HomeOS · 2026-03-18*
