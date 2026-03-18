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

### Priority HIGH

#### 1. Response Quality — "miệng" của HomeOS
```
Vấn đề: Response template quá nghèo → user nói gì cũng nhận cùng 1-2 câu
Mục tiêu:
  - Response phản ánh NỘI DUNG (topic, entities) không chỉ TONE
  - Instinct output (Causality, Abstraction) → surface vào text
  - Silk walk depth → enrich response
  - Dream insights → "Mình đã học được X từ những gì bạn nói"
Effort: TRUNG BÌNH
Impact: CỰC CAO — đây là thứ user chạm đầu tiên
```

#### 2. Command Parsing — typeof/explain/why/trace
```
Vấn đề: 6/14 commands bị parser bỏ qua (không có trong is_command())
Mục tiêu: Thêm typeof, explain, why, trace, inspect, assert vào parser
Effort: NHỎ
Impact: CAO — debug/reasoning commands cần hoạt động
```

#### 3. Agent Loop — Chiefs thật sự xử lý
```
Vấn đề: Chiefs boot nhưng idle, 0 messages, 0 Workers
Mục tiêu:
  - User nói về nhà → HomeChief nhận task qua ISL
  - User hỏi camera → VisionChief nhận
  - Chiefs xử lý → trả response qua ISL → runtime render
  - Mock Workers cho test
Effort: LỚN
Impact: CAO — "Home" trong "HomeOS"
```

### Priority MEDIUM

#### 4. Dream Threshold — Fibonacci quá cao
```
Vấn đề: STM chỉ 1-3 observations/session → không đủ để cluster
Mục tiêu: Giảm threshold cho real conversation patterns
Effort: NHỎ
```

#### 5. SystemManifest — đọc NodeKind thay vì đoán
```
Vấn đề: 82% nodes unclassified vì classify từ alias name
Mục tiêu: Đọc NodeKind metadata trực tiếp
Effort: NHỎ
```

#### 6. Book Reader — wire vào runtime
```
Mục tiêu: ○{read ...} → BookReader → learn → STM → Silk
Effort: TRUNG BÌNH
```

#### 7. Multi-language Response
```
Mục tiêu: Detect input language → response cùng ngôn ngữ
Effort: TRUNG BÌNH
```

### Priority LOW

```
8. WASM Browser Demo
9. API documentation cho core crates
10. Domain Skills on-demand execution
11. Compiler end-to-end pipeline (source → output file)
12. HAL kết nối thiết bị thật
```

---

## Phân Việc Giữa AI

| Scope | AI Runtime | AI Olang |
|-------|-----------|----------|
| `crates/runtime/` | ✅ Primary | ❌ |
| `crates/agents/` | ✅ Primary | ❌ |
| `crates/memory/` | ✅ Primary | ❌ |
| `crates/silk/` | ✅ Primary | ❌ |
| `crates/context/` | ✅ Primary | ❌ |
| `crates/isl/` | ✅ Primary | ❌ |
| `crates/olang/src/execution/` | ❌ | ✅ Primary |
| `crates/runtime/src/parser.rs` | ⚠️ Read only | ✅ Primary |
| `tools/server/` | ✅ Primary | ❌ |

---

*HomeOS · 2026-03-18*
