# TASKBOARD — Bảng phân việc cho AI sessions

> **Mọi AI session đọc file này TRƯỚC KHI bắt đầu làm việc.**
> File này là nguồn sự thật duy nhất (single source of truth) về ai đang làm gì.

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
  1. Cập nhật TASKBOARD.md         ← đổi status → DONE, ghi notes
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

## Task Status Legend

```
FREE      — chưa ai nhận, sẵn sàng
CLAIMED   — đang có session làm (xem branch)
BLOCKED   — đang bị chặn (xem notes)
DONE      — hoàn thành, đã merge hoặc push
CONFLICT  — 2 session cùng claim → cần người quyết định
```

---

## Blockers (giải trước khi làm task phụ thuộc)

| ID | Blocker | Fix | Effort | Status | Branch |
|----|---------|-----|--------|--------|--------|
| B1 | Parser thiếu `union`/`type` keywords | 2 dòng `alphabet.rs:391` | 5 min | DONE | claude/review-and-fix-project-erPD8 |
| B2 | ModuleLoader thiếu file I/O | ~20 LOC `module.rs` | 1-2h | FREE | — |
| B3 | `to_num()` alias thiếu | 1 dòng `semantic.rs` | 1 min | DONE | claude/review-and-fix-project-erPD8 |

**Lưu ý:** B1+B2+B3 block toàn bộ Phase 0. Nên giải TRƯỚC.

---

## Phase 0 — Bootstrap compiler loop

| ID | Task | Plan | Depends | Status | Branch | Session | Notes |
|----|------|------|---------|--------|--------|---------|-------|
| 0.1 | Test lexer.ol trên Rust VM | `PLAN_0_1` | B1,B2,B3 | BLOCKED | — | — | B1+B3 DONE, chờ B2 |
| 0.2 | Test parser.ol + module import | `PLAN_0_2` | 0.1 | FREE | — | — | — |
| 0.3 | Round-trip self-parse | `PLAN_0_3` | 0.2 | FREE | — | — | — |
| 0.4 | Viết semantic.ol (~800 LOC) | `PLAN_0_4` | 0.3 | FREE | — | — | — |
| 0.5 | Viết codegen.ol (~400 LOC) | `PLAN_0_5` | 0.4 | FREE | — | — | — |
| 0.6 | Self-compile test | `PLAN_0_6` | 0.5 | FREE | — | — | — |

## Phase 1 — Machine code VM (SONG SONG với Phase 0)

| ID | Task | Plan | Depends | Status | Branch | Session | Notes |
|----|------|------|---------|--------|--------|---------|-------|
| 1.1 | vm_x86_64.S | `PLAN_1_1` | 0.5 (bytecode format) | FREE | — | — | Cần bytecode format cố định |
| 1.2 | vm_arm64.S | — | 1.1 | FREE | — | — | Chưa có plan chi tiết |
| 1.3 | vm_wasm.wat | — | 1.1 | FREE | — | — | Chưa có plan chi tiết |
| 1.4 | Builder tool (Rust) | `PLAN_1_4` | 1.1 | FREE | — | — | — |

## Song song — Auth (KHÔNG phụ thuộc Phase 0)

| ID | Task | Plan | Depends | Status | Branch | Session | Notes |
|----|------|------|---------|--------|--------|---------|-------|
| AUTH | First-run setup | `PLAN_AUTH` | Không | DONE | `claude/project-audit-review-2pN6F` | 2pN6F | Core done (910 LOC, 21 tests). Wire vào HomeRuntime = pending. |

---

## Dependency Graph (visual)

```
B1+B2+B3 (blockers)
    ↓
  0.1 → 0.2 → 0.3 → 0.4 → 0.5 → 0.6
                                ↓
                              1.1 → 1.4
                               ↓
                              1.2
                               ↓
                              1.3

  AUTH (song song, không phụ thuộc)
```

---

## Gợi ý phân việc cho 2-3 sessions

```
Session A: B1 + B2 + B3 → 0.1 → 0.2 → 0.3  (unblock + bootstrap tests)
Session B: AUTH                                (auth system — song song)
Session C: 1.1 (bắt đầu ASM VM — cần biết bytecode format từ PLAN_0_5)

Khi Session A xong 0.3:
  Session A: → 0.4 (semantic.ol)
  Session C: xem PLAN_0_5 để biết bytecode format → tiếp 1.1
```

---

## Log thay đổi

```
2026-03-18  Tạo TASKBOARD. Audit xong: 2 blockers (B1, B2), 1 minor (B3).
            Tất cả Phase 0 tasks FREE. AUTH FREE.
2026-03-18  AUTH → DONE (session 2pN6F). 7 files, 910 LOC, 21 tests.
            Ed25519 VerifyingKey extended (from_bytes, as_bytes, seed).
            Wire vào HomeRuntime chưa làm (origin.rs quá lớn, cần kế hoạch).
2026-03-18  B1 DONE: thêm "union"→Enum, "type"→Struct vào alphabet.rs
            B3 DONE: thêm "to_num"→"__to_number" vào semantic.rs
            Bonus fixes: CmpOp::Eq (== as compare op), struct-style enum variants,
            __eq VM builtin returns empty() for false (Jz-compatible).
            Parser audit test audit_parse_bootstrap_lexer_ol PASSES.
            All 2381 workspace tests pass. Còn lại B2 (ModuleLoader file I/O).
```
