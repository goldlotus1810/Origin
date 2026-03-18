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

### SPEC_NODE_SILK — 8 gaps ✅ HOÀN THÀNH

```
Gap #1  Parent pointer (Silk dọc 43KB)       ✅  silk/graph.rs
Gap #2  CompoundKind (31 mẫu)                ✅  silk/index.rs
Gap #3  Dream dùng MolSummary + implicit     ✅  memory/dream.rs
Gap #4  Observation.layer (QT⑪)              ✅  agents/learning.rs
Gap #5  Wire unified_neighbors()             ✅  memory/dream.rs
Gap #6  NodeState struct                     ✅  olang/mol/molecular.rs
Gap #7  CompositionOrigin enum               ✅  olang/mol/molecular.rs + lca.rs
Gap #8  Fix weight=0 bug                     ⚠️  heuristic trong STM, weight thật trong Dream
```

### SPEC_MATURITY_PIPELINE — 6 vấn đề hệ thống

```
#1  Response quality (instinct+Silk)         ✅  runtime/origin.rs
#2  Parser 6 commands                        ❌  CÒN — runtime/parser.rs
#3  Maturity pipeline wire                   ✅  agents/learning.rs + memory/dream.rs
#4  Dream threshold quá cao                  ❌  CÒN — memory/dream.rs
#5  Silk parent pointer                      ✅  = Gap #1
#6  Agent hierarchy dead loop                ❌  CÒN — runtime + context + agents
```

### Olang Language — Plan_Olang.md ✅ Phase 6 HOÀN THÀNH

```
Phase 1 (Type System):        ✅ COMPLETE
Phase 2 (Abstraction):        ✅ COMPLETE
Phase 3 (Collections):        ✅ COMPLETE
Phase 4 (Concurrency+Self):   ✅ COMPLETE
Phase 5 (Completion):         ✅ COMPLETE
Phase 6 (Molecular Types):    ✅ COMPLETE
  6N: Node Lifecycle           ✅ (6N1 ⚠️ heuristic, 6N2 ✅, 6N3 ✅)
  6S: Silk Structure           ✅ (6S1-6S5 tất cả xong)
  6T: Molecular Type System    ✅ (6A-6H tất cả xong)
```

---

## Tiếp Theo — Chia việc

### Priority HIGH

#### A. Parser 6 commands — NHỎ (SPEC_MATURITY #2)
```
Vấn đề:  typeof/explain/why/trace/inspect/assert bị bỏ qua bởi is_command()
          handle_command() ĐÃ CÓ code xử lý — chỉ thiếu routing
File:     crates/runtime/src/core/parser.rs
Effort:   NHỎ — thêm 6 từ vào is_command()
AI:       AI Runtime
```

#### B. DreamConfig::for_conversation() — NHỎ (SPEC_MATURITY #4)
```
Vấn đề:  cluster_score ≈ 0.10 << threshold 0.6 → 0 clusters mọi phiên
          Dream không bao giờ cluster được trong hội thoại thông thường
File:     crates/memory/src/dream.rs
Effort:   NHỎ — thêm preset method + wire vào HomeRuntime::new()
Fix:      DreamConfig::for_conversation() { threshold=0.30, min=2, depth=2 }
AI:       AI Runtime
```

#### C. Agent hierarchy wire — LỚN (SPEC_MATURITY #6)
```
Vấn đề:  Chiefs boot nhưng idle, 0 messages, 0 Workers spawned
          Không có HomeControl intent → command không route đến Chief
Files:    crates/context/src/emotion/affect.rs (IntentKind)
          crates/runtime/src/core/origin.rs (routing)
          crates/agents/src/hierarchy/chief.rs (process_task)
Effort:   LỚN — 3 bước: HomeControl intent → ISL routing → Mock Workers
AI:       AI Runtime
```

### Priority MEDIUM

#### D. SystemManifest — đọc NodeKind thay vì đoán
```
Vấn đề:  82% nodes unclassified vì classify từ alias name
Effort:   NHỎ
AI:       AI Runtime
```

#### E. Book Reader — wire vào runtime
```
Mục tiêu: ○{read ...} → BookReader → learn → STM → Silk
Effort:   TRUNG BÌNH
AI:       AI Runtime
```

#### F. Multi-language Response
```
Mục tiêu: Detect input language → response cùng ngôn ngữ
Effort:   TRUNG BÌNH
AI:       AI Runtime
```

### Priority LOW

```
G.  WASM Browser Demo
H.  API documentation cho core crates
I.  Domain Skills on-demand execution
J.  Compiler end-to-end pipeline (source → output file)
K.  HAL kết nối thiết bị thật
```

---

## Phân Việc Giữa AI

| Task | AI Runtime | AI Olang | Effort | Priority |
|------|-----------|----------|--------|----------|
| A. Parser 6 commands | ✅ Làm | — | NHỎ | HIGH |
| B. DreamConfig::for_conversation() | ✅ Làm | — | NHỎ | HIGH |
| C. Agent hierarchy wire | ✅ Làm | — | LỚN | HIGH |
| D. SystemManifest NodeKind | ✅ Làm | — | NHỎ | MEDIUM |
| E. Book Reader wire | ✅ Làm | — | TRUNG BÌNH | MEDIUM |
| F. Multi-language Response | ✅ Làm | — | TRUNG BÌNH | MEDIUM |
| G-K. Low priority | ✅ Làm | ⚠️ J (compiler) | NHỎ-LỚN | LOW |

**AI Olang:** Phase 6 xong. Chỉ quay lại nếu cần mở rộng compiler (task J) hoặc ngôn ngữ mới.

**AI Runtime:** Tất cả việc còn lại thuộc runtime/agents/memory/context — 100% scope AI Runtime.

### Thứ tự khuyến nghị

```
A (vài dòng) → B (nhỏ) → D (nhỏ) → E (trung bình) → F (trung bình) → C (lớn)
     5 min        15 min     30 min      1-2h             1-2h           3-5h
```

A+B+D có thể làm trong 1 phiên. C nên làm riêng vì phức tạp.

---

*HomeOS · 2026-03-18*
