# TASKBOARD — Bảng phân việc cho AI sessions

> **Mọi AI session đọc file này TRƯỚC KHI bắt đầu làm việc.**
> File này là nguồn sự thật duy nhất (single source of truth) về ai đang làm gì.
> **Chi tiết đầy đủ (debug/kiểm tra lỗi):** [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Quy trình phối hợp

```
KHI BẮT ĐẦU SESSION MỚI:
  1. git pull origin main          ← lấy TASKBOARD mới nhất
  2. Đọc TASKBOARD.md              ← xem task nào FREE, task nào CLAIMED
  3. Chọn task FREE                ← ưu tiên theo dependency graph
  4. Cập nhật TASKBOARD.md         ← đổi status → CLAIMED, ghi branch + ngày
  5. git commit + push             ← commit NGAY để session khác thấy
  6. Bắt đầu code

KHI HOÀN THÀNH:
  1. Tải cập nhật main.            ← cập nhật thay đổi mới nhất.
  2. Cập nhật TASKBOARD.md         ← đổi status → DONE, ghi notes
  2. git commit + push

KHI BỊ BLOCKED:
  1. Cập nhật TASKBOARD.md         ← đổi status → BLOCKED, ghi lý do
  2. git commit + push
  3. Chuyển sang task khác (nếu có)

⚠️ KHÔNG BAO GIỜ:
  ❌ Bắt đầu task đã CLAIMED bởi session khác
  ❌ Đổi status task của session khác
  ❌ Xóa dòng — chỉ thêm hoặc cập nhật status của mình
```

---

## ALL DONE ✅

Phase 0-11 | Task 12 | Phase 14.1-14.3 | Phase 15 (6/6) | Phase 16 (4/4) | V2 Migration T1-T14, T16 | INTG
→ Chi tiết: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Recently DONE (Phase 14)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 14.2 | Alias table tách riêng (T15) | DONE ✅ | 33,054 entries, 6B/entry, ~198KB. |
| 14.3 | Silk vertical parent_map persistence | DONE ✅ | RT_PARENT 0x0C (25B/record). |

---

## FREE Tasks — Ưu tiên cao → thấp

### Tier 1 — Unblocked, làm ngay được

| ID | Task | Plan | Effort | Depends | Status | Notes |
|----|------|------|--------|---------|--------|-------|
| P2.0 | Fix 135 VM builtin test failures | PLAN_PHASE2 | ~200 LOC | — | FREE | Array/Dict/String builtins. BLOCKER cho Phase 2. |
| 8.1 | Parser: hex literals (0xFF) | PLAN_8 | ~80 LOC | — | FREE | 13 .ol files fail vì hex. |
| 8.2 | Parser: == trong match/struct | PLAN_8 | ~200 LOC | — | FREE | 9 .ol files fail vì ==. |
| 8.3 | Parser: keywords as ident + struct colon | PLAN_8 | ~100 LOC | — | FREE | intent.ol "learn", silk_ops.ol struct. |
| 12.1 | Wire walk_emotion() vào response | PLAN_12 | ~100 LOC | — | FREE | Trả None hiện tại → implement Silk walk. |
| 12.2 | Context recall trong response | PLAN_12 | ~80 LOC | 12.1 | FREE | ResponseContext struct. |
| 12.3 | Intent estimation dùng context | PLAN_12 | ~120 LOC | 12.2 | FREE | Thay keyword-only bằng context-aware. |
| 12.4 | Response composer thay template | PLAN_12 | ~200 LOC | 12.3 | FREE | 4-part response. |
| 12.5 | Language detection + instinct wire | PLAN_12 | ~60 LOC | 12.4 | FREE | "xin chào" → Việt. |
| 11.3 | Server --eval mode | PLAN_11 | ~80 LOC | — | FREE | stdin → process → output → exit. |
| 11.2 | Rust E2E test suite | PLAN_11 | ~300 LOC | 11.3 | FREE | t16_e2e_demo.rs. |
| 11.5 | Makefile targets (demo/verify) | PLAN_11 | ~50 LOC | 11.2 | FREE | make demo, make verify. |

### Tier 2 — Cần Tier 1 xong trước

| ID | Task | Plan | Effort | Depends | Status | Notes |
|----|------|------|--------|---------|--------|-------|
| P2.2 | Emotion pipeline (.ol) | PLAN_PHASE2 | ~200 LOC | P2.0 | BLOCKED | emotion/curve/intent .ol hoàn thiện. |
| P2.3 | Knowledge layer (.ol) | PLAN_PHASE2 | ~150 LOC | P2.0 | BLOCKED | silk_ops/dream/instinct/learning .ol. |
| P2.4 | Agent behavior (.ol) | PLAN_PHASE2 | ~300 LOC | P2.0 | BLOCKED | response/leo/chief/worker .ol. |
| P2.5 | E2E integration test | PLAN_PHASE2 | ~50 LOC | P2.2-4 | BLOCKED | 5 test cases end-to-end. |
| 9 | Native REPL | PLAN_9 | ~600 LOC | 8.1-8.3 | BLOCKED | ./origin → REPL. Cần parser fix trước. |
| 10 | Browser E2E | PLAN_10 | ~500 LOC | 9 | BLOCKED | origin.html, WASM compile+execute. |
| 11.1 | Demo script (10 scenarios) | PLAN_11 | ~300 LOC | 8, 9 | BLOCKED | Cần parser + REPL. |
| 11.4 | Native binary --eval | PLAN_11 | ~50 LOC | 9 | BLOCKED | Cần native REPL. |

### Tier 3 — Lớn, cần kế hoạch riêng

| ID | Task | Plan | Effort | Status | Notes |
|----|------|------|--------|--------|-------|
| 7.2 | Mobile (Android + iOS) | PLAN_7_2 | 2-3 tuần | FREE | ARM64 native + WASM iOS. Ưu tiên thấp. |
| PW | P_weight migration 5B→2B | PLAN_PWEIGHT | LỚN | DRAFT | Data=2B, Code=5B mismatch. Ảnh hưởng tất cả crates. |
| V2 | V2 Migration BIG BANG | PLAN_V2 | RẤT LỚN | DRAFT | 12 tasks, 5 layers. Molecule→u16, Chain→Vec<u16>. |
| UDC | UDC Rebuild (59 blocks) | PLAN_UDC | Nhiều sessions | IN_PROGRESS | 8,846 entries từ Unicode 18.0. |
| TLC | Test Logic Check (6 patterns) | PLAN_TEST_LOGIC | Trung bình | PARTIAL | 6 test files cần viết. |

---

## Dependency Graph

```
                    ┌─────────────────────────────────────────┐
                    │         TIER 1 — Làm ngay               │
                    │                                         │
  P2.0 (Fix VM) ───┤  8.1-8.3 (Parser)   12.1→12.5 (Response)
       │            │  11.3→11.2→11.5 (E2E Server)           │
       │            └─────────────────────────────────────────┘
       │                    │
       ▼                    ▼
  ┌────────────┐    ┌───────────────┐
  │ TIER 2     │    │ TIER 2        │
  │ P2.2 Emot  │    │ 9 REPL        │
  │ P2.3 Know  │    │   ↓           │
  │ P2.4 Agent │    │ 10 Browser    │
  │   ↓        │    │ 11.1 Demo     │
  │ P2.5 E2E   │    │ 11.4 Native   │
  └────────────┘    └───────────────┘

  TIER 3 (song song, kế hoạch riêng):
    7.2 Mobile | PW Migration | V2 BIG BANG | UDC Rebuild | Test Logic
```

---

## Log thay đổi

```
2026-03-18  Tạo TASKBOARD. B1-B3, AUTH, Phase 0.1-0.2 DONE.
2026-03-19  Phase 0-9 ALL DONE. INTG ALL DONE. origin.olang 1.35MB ELF.
2026-03-21  V2 Migration T1-T12 DONE. Spec v3 audit → Phase 14-16 thêm.
2026-03-21  Session 2pN6F: Task 12 + Phase 15 (6) + Phase 16 (4) + T13/T14/T16 DONE.
2026-03-21  Chỉ còn 14.2 (Alias) + 14.3 (Silk vertical) FREE.
2026-03-21  14.2 (Alias table) DONE. 33K entries tách riêng khỏi KnowTree. Dọn plans/done/.
2026-03-21  14.3 (Silk vertical parent_map persistence) DONE.
2026-03-21  TẤT CẢ TASKS DONE. Bắt đầu lên kế hoạch Giai đoạn 2.
2026-03-21  Thêm 30+ tasks từ Plans còn lại. 3 tiers ưu tiên. Dependency graph.
```
→ Full log: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)
