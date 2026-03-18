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
#2  Parser 6 commands                        ✅  runtime/parser.rs — is_command() + 6 prefixes
#3  Maturity pipeline wire                   ✅  agents/learning.rs + memory/dream.rs
#4  Dream threshold quá cao                  ✅  DreamConfig::for_conversation() threshold=0.30
#5  Silk parent pointer                      ✅  = Gap #1
#6  Agent hierarchy dead loop                ✅  HomeControl intent → ISL → HomeChief
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

## Audit Chi Tiết — 2026-03-18

### A. Parser 6 commands — RẤT NHỎ (SPEC_MATURITY #2)
```
Vấn đề:  typeof/explain/why/trace/inspect/assert bị bỏ qua bởi is_command()
          handle_command() ĐÃ CÓ code xử lý 100% — chỉ thiếu routing
          VM events đã defined + xử lý: TraceStep, InspectChain, AssertFailed,
          TypeInfo, WhyConnection, ExplainOrigin
          Help text đã liệt kê đủ 6 commands

File:     crates/runtime/src/core/parser.rs — is_command() dòng 1136-1159
Fix:      Thêm "trace" vào matches! + thêm is_reasoning_command() helper:
          s.starts_with("typeof ") || s.starts_with("inspect ")
          || s.starts_with("assert ") || s.starts_with("explain ")
          || s.starts_with("why ")
Effort:   ~10 dòng code
```

### B. DreamConfig::for_conversation() — NHỎ (SPEC_MATURITY #4)
```
Vấn đề:  cluster_score ≈ 0.10 << threshold 0.6 → 0 clusters mọi phiên
          DreamConfig chỉ có default() (threshold=0.6) và with_weights()
          Không có preset cho conversation context

File 1:   crates/memory/src/dream.rs — thêm method for_conversation()
          DreamConfig::for_conversation() { threshold=0.30, min=2, depth=2 }
File 2:   crates/runtime/src/core/origin.rs — dòng 262
          DreamConfig::default() → DreamConfig::for_conversation()
Effort:   ~15 dòng code
```

### C. Agent hierarchy wire — LỚN (SPEC_MATURITY #6)
```
Vấn đề:  Chiefs boot nhưng idle, 0 messages, 0 Workers spawned
          IntentKind::Command được detect (keywords sẵn) nhưng decide_action()
          fall-through → Proceed (xử lý như Chat bình thường)
          Không có routing từ Command intent → Chief qua ISL

Audit:
  ✅ IntentKind::Command — detect đúng ("tắt đèn", "turn off", ...)
  ✅ Chief.receive_frame() — sẵn sàng nhận ISL
  ✅ Chief.forward_command() — sẵn sàng gửi ActuatorCmd cho Worker
  ✅ MessageRouter — Phase 1/2/3 routing đã wire
  ✅ ISL MsgType::ActuatorCmd — protocol sẵn
  ❌ decide_action() — Command falls through → Proceed (dòng 721)
  ❌ Runtime dispatch — không có code gửi ISL đến Chief

3 bước:
  1. IntentAction::HomeControl { cmd } — variant mới trong context/intent.rs
  2. decide_action() match Command → return HomeControl — context/intent.rs dòng 691
  3. Runtime xử lý HomeControl → ISL message đến Chief — runtime/origin.rs

Files:    crates/context/src/analysis/intent.rs (IntentAction + decide_action)
          crates/runtime/src/core/origin.rs (dispatch to Chief)
Effort:   ~100-150 dòng code, cần test integration
```

### D. SystemManifest NodeKind — NHỎ-TB
```
Vấn đề:  82% nodes unclassified
          classify_node() → find_alias() quét bảng tĩnh ALIAS_CODEPOINTS (~70 entries)
          L1+ nodes tạo từ learning không có trong bảng tĩnh → Uncategorized

File:     crates/olang/src/system/startup.rs — dòng 1157-1268
Fix:      Registry cần expose reverse lookup (chain_hash → alias)
          Hoặc insert_with_kind() lúc seed → classify từ NodeKind thay vì alias
Effort:   ~50 dòng code
```

### E. Book Reader wire — TRUNG BÌNH
```
Vấn đề:  BookReader hoàn chỉnh (read → SentenceRecord → emotion)
          LearningLoop.learn_text() 5 tầng đã wire
          NHƯNG: parser không có "read" command → ○{read ...} bị bỏ qua

File 1:   crates/runtime/src/core/parser.rs — thêm "read" vào is_command()
File 2:   crates/runtime/src/core/origin.rs — handle_command() route "read" →
          BookReader.read(text) → filter top_significant() → learn_text() per sentence
File 3:   crates/agents/src/pipeline/book.rs — đã sẵn sàng, không cần sửa
Effort:   ~60-80 dòng code
```

### F. Multi-language Response — TRUNG BÌNH-LỚN
```
Vấn đề:  100% responses hardcoded Vietnamese
          Chỉ crisis_text_with_region("vi"/"en") có 2 ngôn ngữ
          ResponseParams không có language field
          Không có language detection

Files:    crates/runtime/src/output/response_template.rs — toàn bộ
          crates/runtime/src/core/origin.rs — truyền lang vào render()
Fix:
  1. Language detection function (keyword-based hoặc Unicode range)
  2. ResponseParams.language: String field
  3. Dịch 10+ template functions sang ít nhất en + vi
  4. Runtime: detect → store in ConversationCurve → pass to render()
Effort:   ~200-300 dòng code (phần lớn là dịch template)
```

---

## Phân Việc Giữa 2 AI

### AI 1 — Quick fixes (A + B + D)
```
Scope:    3 tasks nhỏ, ít conflict, có thể làm 1 phiên
Files:    parser.rs (A), dream.rs (B), origin.rs 1 dòng (B), startup.rs (D)
Effort:   ~1-2h tổng
Thứ tự:   A → B → D

Lưu ý:
  - A chạm parser.rs → làm TRƯỚC E (cùng file)
  - B chạm origin.rs 1 dòng (đổi DreamConfig init) → conflict nhỏ
  - D isolated trong olang/system/startup.rs → 0 conflict
```

### AI 2 — Complex wiring (C + E + F)
```
Scope:    3 tasks lớn, liên quan nhau, cần sửa nhiều trong origin.rs
Files:    intent.rs (C), origin.rs (C+E+F), parser.rs (E),
          response_template.rs (F), book.rs (E — chỉ đọc)
Effort:   ~5-8h tổng
Thứ tự:   E → F → C

Lưu ý:
  - E cần parser.rs → AI 1 phải merge A trước
  - C là task lớn nhất, nên làm cuối khi E+F đã ổn
  - F có thể làm song song với C (khác function trong origin.rs)
```

### Quy tắc merge
```
1. AI 1 làm xong A+B+D → merge vào main TRƯỚC
2. AI 2 pull main → bắt đầu E+F+C
3. Conflict zone duy nhất: origin.rs (B thay 1 dòng, C+E+F thêm logic)
   → AI 1 merge trước = tránh conflict
```

### Priority LOW (sau khi A-F xong)
```
G.  WASM Browser Demo           — AI nào rảnh
H.  API documentation           — AI nào rảnh
I.  Domain Skills on-demand     — AI Runtime
J.  Compiler end-to-end         — AI Olang (nếu cần)
K.  HAL kết nối thiết bị thật   — AI Runtime
```

---

*HomeOS · 2026-03-18*
