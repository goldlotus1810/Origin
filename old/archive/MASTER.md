# HomeOS — Master Document

> AI mới vào: ĐỌC FILE NÀY + CLAUDE.md TRƯỚC KHI LÀM GÌ.
> Sau mỗi phiên: CẬP NHẬT file này.

**Cập nhật:** 2026-03-18 (Phiên N)
**Tests:** 2,359 pass · 0 fail · 0 external deps
**Code:** ~84,000 lines Rust · 11 crates + 4 tools · no_std core

---

## Trạng Thái Thật (verified by running code)

### HOẠT ĐỘNG (wired + working):
```
✅ UCD Engine (5424 entries, hierarchical byte encoding)
✅ Molecule/Chain 5D + tagged sparse encoding (1-6 bytes)
✅ LCA weighted + variance + hierarchical base-aware mode
✅ Registry append-only + crash recovery
✅ Silk Hebbian + φ⁻¹ decay
✅ Emotion Pipeline 7 layers (end-to-end in runtime)
✅ VM 36 opcodes + arithmetic (○{1+2}=3, ○{solve "2x+3=7"})
✅ Parser 18/18 RelOps
✅ SecurityGate (Crisis detection + hotline)
✅ ISL messaging (4-byte address, AES-256-GCM)
✅ HAL (x86/ARM/RISC-V/WASM detect)
✅ VSDF (18 SDF + FFR Fibonacci render)
✅ NodeBody + BodyStore (chain_hash → SDF + Spline)
✅ Molecule.evolve() (mutation 1/5 dim → new species)
✅ Agent hierarchy (AAM/Chief/Worker boot)
✅ Compiler backends (C/Rust/WASM)
✅ WASM browser bindings
✅ Zero external deps (native SHA-256, Ed25519, AES-256-GCM, homemath)
✅ Dream auto-trigger + STM cleanup + cluster + QR promote
✅ Instincts wired into response flow
✅ LeoAI self-programming (○{program ...})
✅ KnowTree L2/L3 concepts
✅ SkillPattern → AAM approval pipeline
✅ Multilingual seeding (7 languages, 432+ aliases)
✅ Codebase restructured (subdirectories per crate)
✅ Silk parent_map vertical (child→parent pointer, 43KB design)
✅ CompoundKind 31 compound patterns (C(5,k), k=1..5)
✅ Dream 5D clustering (MolSummary + implicit Silk + layer filter QT⑪)
✅ NodeState wrapper (Molecule + Maturity + CompositionOrigin)
✅ CompositionOrigin tracking (Innate/Composed/Evolved) in LCA + evolve
✅ Maturity advance wired to STM + Dream (weight=0 bug fixed)
✅ unified_neighbors() wired into Dream neighbor_bonus
✅ Observation.layer field + Dream layer-aware clustering
✅ Olang Type System (6A-6H: TypeTag, TypedValue, pattern matching, generics)
✅ origin.olang = bộ nhớ duy nhất (9 record types, RAM = cache)
✅ STM persist/restore (RT_STM 0x06 — observation → file → boot replay)
✅ Hebbian persist/restore (RT_HEBBIAN 0x07 — learned weights → file)
✅ KnowTree persist/restore (RT_KNOWTREE 0x08 — L2+ compact nodes → file)
✅ ConversationCurve persist/restore (RT_CURVE 0x09 — emotion trajectory)
✅ Chiefs WIRED (domain automation: temp, motion, security escalation)
✅ Workers WIRED (4 kinds: Sensor, Actuator, Camera, Network + ISL)
✅ MessageRouter 7-phase pump (Workers→Chiefs→LeoAI→AAM→Dream)
✅ BookReader wired to runtime (○{read ...} → STM → Silk → KnowTree)
✅ Domain skills CALLABLE (15 skills, called in learning pipeline)
✅ Compiler backends WORKING (C/Rust/WASM generation)
✅ Module system (ModuleLoader + DepGraph + cycle detection)
✅ Olang stdlib (10 modules: math, string, vec, set, map, io, test...)
```

### FACADE → RESOLVED:
```
✅ Chiefs (3) — WIRED: domain automation, peer messaging, heartbeat (47 tests)
✅ Workers (4 kinds) — WIRED: sensor/actuator/camera/network, ISL frames
✅ ISL routing — ACTIVE: MessageRouter.tick() pumps 7 phases
✅ Compiler targets — WORKING: C/Rust/WASM backends generate correct code
✅ Book reader — WIRED: ○{read ...} → sentence parsing → STM → Silk
✅ Domain skills — CALLABLE: used in learning pipeline, not just tests
```

### Remaining Gaps:
```
⚠️ Response diversity — ~10 templates, not personalized per conversation
⚠️ Command execution — explain/why parsed but mostly stubs
⚠️ Compiler FFI stubs — generated code needs runtime stubs for execution
⚠️ Type system enforcement — semantic lowering done, not enforced at runtime
⚠️ Worker device integration — defined but no actual hardware drivers
```

### SPEC AUDIT — 6 Vấn Đề Hệ Thống (phiên L → N):
```
#1 Response template      — ~10 câu cố định, bỏ qua instinct+Silk output    [HIGH, MEDIUM effort]
#2 Parser missing 6 cmds  — typeof/explain/why/trace/inspect/assert           [HIGH, SMALL effort]
#3 Maturity pipeline       — ✅ DONE — advance wired, weight=0 fixed          [RESOLVED]
#4 Dream threshold         — ✅ DONE — MolSummary + implicit Silk bonus        [RESOLVED]
#5 Silk vertical (parent)  — ✅ DONE — parent_map in SilkGraph                 [RESOLVED]
#6 Agent hierarchy dead    — ✅ DONE — Chiefs wired, Workers wired, Router 7-phase pump [RESOLVED]
```

### QT8 AUDIT — Memory Persistence (phiên N):
```
✅ origin.olang = bộ nhớ duy nhất, RAM = cache tạm
✅ 9 record types: Node(0x01) Edge(0x02) Alias(0x03) Amend(0x04) NodeKind(0x05)
                   STM(0x06) Hebbian(0x07) KnowTree(0x08) Curve(0x09)
✅ Boot restore: startup.rs → BootResult → runtime wires all state back
✅ Runtime persist: every process_text() → append STM + Hebbian + Curve + KnowTree
✅ Roundtrip tests: reader.rs verifies all 9 record types
```

### Node & Silk — 8 Gaps (SPEC_NODE_SILK — ALL RESOLVED ✅):
```
Gap #1  Silk dọc (parent pointer)     — ✅ parent_map in SilkGraph
Gap #2  31 compound patterns          — ✅ CompoundKind enum implemented
Gap #3  Dream 5D similarity           — ✅ MolSummary + implicit Silk bonus
Gap #4  Dream layer filtering         — ✅ Observation.layer + layer grouping
Gap #5  unified_neighbors() wired     — ✅ Dream uses neighbor_bonus
Gap #6  Molecule = công thức          — ✅ NodeState = Molecule + Maturity + Origin
Gap #7  CompositionOrigin             — ✅ lca_with_origin(), evolve() tracks origin
Gap #8  Maturity advance weight=0     — ✅ heuristic_weight from Silk
```

### Silk thiết kế vs thực tế:
```
3 tầng:  Base 37 kênh ✅ | Compound 31 mẫu ✅ | Precise ~5400 (SPEC)
2 hướng: Ngang (implicit) ✅ | Dọc (parent 43KB) ✅
Hebbian = phát hiện cái đã có, không tạo mới ✅
```

### SPEC FILES:
```
SPEC_MATURITY_PIPELINE.md  — Wire Maturity vào Dream (covers #3, #4)
SPEC_NODE_SILK.md          — 8 Gaps: ALL 8 IMPLEMENTED ✅
PLAN_REWRITE.md            — origin.olang = self-contained executable (7 giai đoạn)
```

---

## Test Counts Per Crate

| Crate | Tests | Status |
|-------|-------|--------|
| olang | 1093 | Core engine + type system + storage (STM/Hebbian/KnowTree/Curve records) |
| agents | 284 | Encoder, learning, gate, instincts, chief, worker |
| runtime | 273 | HomeRuntime, parser, router, persist/restore |
| context | 168 | Emotion, intent, curve, fusion |
| vsdf | 123 | SDF, FFR, physics, scene |
| silk | 112 | Graph, hebbian, walk, edges, parent_map, restore_learned |
| hal | 85 | Arch, platform, security, tier |
| memory | 68 | STM, dream, proposals |
| wasm | 38 | WASM bindings, bridge |
| isl | 31 | Address, message, codec, queue |
| ucd | 23 | Unicode lookup table |
| homemath | 18 | Pure math functions |
| seeder | 15 | L0 node seeding |
| server | 13 | REPL boot/run |
| inspector | 9 | File verification |
| migrate | 6 | Data migration tools |
| **Total** | **2,359** | |

---

## Lịch Sử Phiên

```
A: Phase 1-8 code skeleton
B: Molecule encoding, Dream pipeline
C: Phase 9 Zero deps (SHA-256, Ed25519, AES-256-GCM)
D: Tagged encoding, hierarchical bytes, NodeBody, evolve(), VM arithmetic
E: Docs sync, reseed origin.olang
F: RelOps 18/18, Dream STM cleanup
G: Verify Phase 2-5 status, update docs
H: Docs → old/, Phase 5+4+3 complete, 1759 tests
I: SkillProposal + unwrap reduction + Multilingual Seeding, 1774 tests
J: SkillPattern → AAM pipeline + test coverage + warning cleanup, 1784 tests
K: Honest audit — response template, command parsing, agent orchestration issues identified
   Response generation improved, instincts wired to output, Dream fixed
   Codebase restructured into subdirectories, 2063 tests
L: Spec audit — 2 specs created (SPEC_MATURITY_PIPELINE, SPEC_NODE_SILK)
   6 systemic issues mapped, 1 critical bug found (advance weight=0.0)
   Phase 1-3 Olang features verified, 2227 tests
M: SPEC_NODE_SILK all 8 gaps implemented — parent_map, CompoundKind,
   Dream 5D+layer, NodeState+CompositionOrigin, Maturity wired.
   Phase 6T: Olang Type System (6A-6H). 2348 tests
N: QT8 origin.olang = bộ nhớ duy nhất. 4 new record types (STM/Hebbian/KnowTree/Curve).
   Full persist/restore pipeline. Agent hierarchy WIRED (Chiefs+Workers+Router).
   Module system + stdlib. PLAN_REWRITE: origin.olang self-contained executable. 2359 tests
```

---

## Quy Trình Mỗi Phiên

```
1. TRƯỚC: Đọc MASTER.md + CLAUDE.md → git log -10 → cargo test --workspace
2. TRONG: Commit thường xuyên (phiên có thể kết thúc bất ngờ)
3. SAU:   cargo test + clippy → CẬP NHẬT MASTER.md → commit + push
```

---

## Tài Liệu

| File | Nội dung |
|------|---------|
| [CLAUDE.md](CLAUDE.md) | Hướng dẫn cho AI contributors (bất biến) |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Kiến trúc tổng thể |
| [MASTER.md](MASTER.md) | File này — trạng thái dự án + lịch sử |
| [PLAN.md](PLAN.md) | Kế hoạch tiếp theo |
| [README.md](README.md) | Giới thiệu dự án |
| [REVIEW.md](REVIEW.md) | Đánh giá trung thực |
| [PLAN_REWRITE.md](PLAN_REWRITE.md) | Kế hoạch origin.olang self-contained |
| old/ | Tài liệu cũ (2026-03-17, 2026-03-18) |

---

*HomeOS · 2026-03-18 · 2,359 tests · ~84K LoC · ○(∅)==○*
