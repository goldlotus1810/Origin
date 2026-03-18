# HomeOS — Plan

**Cập nhật:** 2026-03-18

---

## Đã Hoàn Thành

```
Phase 1:  VM arithmetic (○{1+2}=3, ○{solve "2x+3=7"})
Phase 2:  Graph traversal (find_path, trace_origin, reachable)
Phase 3:  Domain Knowledge (6 domains, 313+ nodes seeded)
Phase 4:  Math → Silk (math results learned into graph)
Phase 5:  Agent Orchestration (Router + Chiefs wired)
Phase 6:  Perception pipeline (InverseRender, SensorSkill)
Phase 7:  LeoAI self-programming (program/compose/verify/experiment)
Phase 8:  Build layers (Compiler C/Rust/WASM targets)
Phase 9:  Zero External Dependencies (native crypto + homemath)
Phase 10: SkillPattern → AAM approval pipeline
Phase 11: Multilingual Seeding (7 languages)
Phase 12: Codebase restructuring (subdirectories per crate)
```

---

## Tiếp Theo

### Priority HIGH — SPEC_NODE_SILK Integration (Phase 6 trong Plan_Olang.md)

```
SPEC_NODE_SILK.md: 8 gaps — 8 thay đổi đề xuất
  Node (olang crate): Gap #6, #7, #8 → 6N1-6N3 trong Plan_Olang.md Phase 6
  Silk (silk/memory):  Gap #1-#5     → 6S1-6S5 trong Plan_Olang.md Phase 6

Thứ tự thực hiện:
  6N1 (fix weight=0 bug) → QUICK FIX đầu tiên
  6S1 (parent pointer) + 6N2 (NodeState) → song song
  6S2 (Observation.layer) → sau 6S1
  6S3 (CompoundKind) + 6N3 (CompositionOrigin) → song song
  6S4 (Dream MolSummary) → sau 6S1+6S3
  6S5 (unified_neighbors) → cuối cùng
```

### Priority HIGH — Spec Audit (6 vấn đề hệ thống)

```
Thứ tự thực hiện khuyến nghị:
  #2 → #3 → #4 → #1 → #5 → #6

SPEC_MATURITY_PIPELINE.md: covers #3, #4, maps all 6
SPEC_NODE_SILK.md:         covers #5 + 8 gaps (compound, Dream 5D, layer,
                           unified_neighbors, Molecule lifecycle, CompositionOrigin)
```

#### #2. Parser missing 6 commands (dễ nhất)
```
Vấn đề: typeof/explain/why/trace/inspect/assert bị bỏ qua bởi is_command()
File:    crates/runtime/src/core/parser.rs
Effort:  NHỎ (vài dòng) | Impact: CAO
```

#### #3. Maturity pipeline + advance() bug (CRITICAL)
```
Vấn đề: Maturity enum + advance() tồn tại nhưng KHÔNG wire vào STM/Dream
         advance(weight=0.0) → Mature UNREACHABLE (0.0 < 0.854 luôn)
Files:   agents/pipeline/learning.rs (Observation.maturity)
         memory/dream.rs (DreamResult.matured_nodes)
         runtime callers
Effort:  NHỎ | Impact: CAO — node không bao giờ chín
Fix:     Truyền Hebbian weight thật hoặc thêm advance_by_fire()
→ Mapped to Plan_Olang.md Phase 6: 6N1
```

#### #4. Dream threshold quá cao
```
Vấn đề: cluster_score ≈ 0.10 << threshold 0.6 → 0 clusters mọi phiên
         Dream bỏ qua 5D similarity (dùng chain bytes thay vì MolSummary)
Files:   memory/dream.rs (cluster_score, DreamConfig)
Effort:  NHỎ | Impact: CAO — Dream không học được gì
Fix:     DreamConfig::for_conversation() (threshold=0.30, min=2)
         cluster_score() dùng MolSummary + implicit_silk() bonus
→ Mapped to Plan_Olang.md Phase 6: 6S4
```

#### #1. Response Quality — "miệng" của HomeOS
```
Vấn đề: ~10 câu template cố định, bỏ qua instinct+Silk output
         User nói gì cũng nhận cùng 1-2 câu
Files:   runtime/core/origin.rs (render_response)
Effort:  TRUNG BÌNH | Impact: CỰC CAO
Fix:     Surface instinct results + Silk neighbors vào response text
```

#### #5. Silk vertical — parent pointer
```
Vấn đề: Thiết kế 5460 pointers × 8B = 43KB vertical Silk, chưa implement
         Dream không biết 2 nodes cùng tầng → vi phạm QT⑪
Files:   silk/graph.rs (parent_map, register_parent, layer_of)
         agents/learning.rs (Observation.layer)
Effort:  TRUNG BÌNH | Impact: CAO — nền tảng layer-aware queries
→ Mapped to Plan_Olang.md Phase 6: 6S1 + 6S2
```

#### #6. Agent hierarchy dead loop
```
Vấn đề: Chiefs boot nhưng idle, 0 messages, 0 Workers spawned
Files:   runtime/core/origin.rs, agents/hierarchy/chief.rs, context/intent.rs
Effort:  LỚN | Impact: CAO — "Home" trong "HomeOS"
Fix:     Thêm HomeControl intent → route ISL → Chief → Mock Workers
```

### Priority MEDIUM

#### 7. SystemManifest — đọc NodeKind thay vì đoán
```
Vấn đề: 82% nodes unclassified vì classify từ alias name
Effort: NHỎ
```

#### 8. Book Reader — wire vào runtime
```
Mục tiêu: ○{read ...} → BookReader → learn → STM → Silk
Effort: TRUNG BÌNH
```

#### 9. Multi-language Response
```
Mục tiêu: Detect input language → response cùng ngôn ngữ
Effort: TRUNG BÌNH
```

### Priority LOW

```
10. WASM Browser Demo
11. API documentation cho core crates
12. Domain Skills on-demand execution
13. Compiler end-to-end pipeline (source → output file)
14. HAL kết nối thiết bị thật
```

---

## Olang Language — Plan_Olang.md Status

```
Phase 1 (Type System):        ✅ COMPLETE — struct, enum, Option/Result, impl, pub
Phase 2 (Abstraction):        ✅ COMPLETE — trait, generics, module system, closures
Phase 3 (Collections):        ✅ COMPLETE — Iterator, f"...", BitShift, String, Byte, Math
Phase 4 (Concurrency+Self):   ✅ COMPLETE — channel, spawn, select, self-hosting, migration
Phase 5 (Completion):         ✅ COMPLETE — ?, Option/Result, Set/Deque, slice, stdlib, compiler
Phase 6 (Molecular Types):    🔄 NEXT — 3 nhóm: Node lifecycle + Silk structure + ○{ } types

Phase 6 overview (3 nhóm):
  6N: Node Lifecycle (SPEC gaps #6,#7,#8)
    6N1. Fix weight=0 bug       — CRITICAL, quick fix
    6N2. NodeState wrapper       — Molecule + Maturity + Origin
    6N3. CompositionOrigin       — Track LCA/evolve nguồn gốc

  6S: Silk Structure (SPEC gaps #1-#5)
    6S1. Parent pointer (43KB)   — vertical Silk, layer_of()
    6S2. Observation.layer       — Dream same-layer filter (QT⑪)
    6S3. CompoundKind (31 mẫu)   — classify ImplicitSilk patterns
    6S4. Dream MolSummary+Silk   — unblock Dream clustering
    6S5. Wire unified_neighbors  — enrich response + Dream

  6T: Molecular Type System
    6A-6H. ○{ } constraints, immutability, CoW, exhaustive match, effects
```

---

## Phân Việc Giữa AI

| Scope | AI Runtime | AI Olang |
|-------|-----------|----------|
| `crates/runtime/` | ✅ Primary | ❌ |
| `crates/agents/` | ✅ Primary | ⚠️ Phase 6 (6N1, 6S2) |
| `crates/memory/` | ✅ Primary | ⚠️ Phase 6 (6S4, 6S5) |
| `crates/silk/` | ✅ Primary | ⚠️ Phase 6 (6S1, 6S3) |
| `crates/context/` | ✅ Primary | ❌ |
| `crates/isl/` | ✅ Primary | ❌ |
| `crates/olang/src/mol/` | ⚠️ Read only | ✅ Phase 6 (6N2, 6N3) |
| `crates/olang/src/exec/` | ❌ | ✅ Primary |
| `crates/runtime/src/parser.rs` | ⚠️ Read only | ✅ Primary |
| `tools/server/` | ✅ Primary | ❌ |

---

*HomeOS · 2026-03-18*
