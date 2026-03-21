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

Phase 0-11 | Task 12 | Phase 14.1 | Phase 15 (6/6) | Phase 16 (4/4) | V2 Migration T1-T14, T16 | INTG
→ Chi tiết: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## Remaining FREE Tasks

| ID | Task | Spec ref | Depends | Status | Notes |
|----|------|----------|---------|--------|-------|
| 14.2 | Alias table tách riêng (= T15) | §1.7 | T14 ✅ | FREE | 41,338 emoji/UTF-32 → alias riêng, KHÔNG gộp KnowTree. |
| 14.3 | Silk vertical: parent_map 8,846 pointers | §2.3 | T14 ✅ | FREE | Đi từ lá lên gốc. register_parent() hook đã có ở 7.1. ~71 KB. |

---

## Dependency Graph

```
14.2 (Alias) — không phụ thuộc 14.3, làm song song được
14.3 (Silk vertical) — không phụ thuộc 14.2, làm song song được

Cả 2 đều unblocked (T14 DONE).
```

---

## Log thay đổi

```
2026-03-18  Tạo TASKBOARD. B1-B3, AUTH, Phase 0.1-0.2 DONE.
2026-03-19  Phase 0-9 ALL DONE. INTG ALL DONE. origin.olang 1.35MB ELF.
2026-03-21  V2 Migration T1-T12 DONE. Spec v3 audit → Phase 14-16 thêm.
2026-03-21  Session 2pN6F: Task 12 + Phase 15 (6) + Phase 16 (4) + T13/T14/T16 DONE.
2026-03-21  Chỉ còn 14.2 (Alias) + 14.3 (Silk vertical) FREE.
```
→ Full log: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)
