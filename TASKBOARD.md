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

## Task Status Legend

```
FREE      — chưa ai nhận, sẵn sàng
CLAIMED   — đang có session làm (xem branch)
BLOCKED   — đang bị chặn (xem notes)
DONE      — hoàn thành, đã merge hoặc push
CONFLICT  — 2 session cùng claim → cần người quyết định
```

---

## Phase 0-7 — ALL DONE ✅

B1-B7 ALL DONE | Phase 0 (0.1-0.6 compiler) ALL DONE | Phase 1-7 ALL DONE
→ Chi tiết: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

## Phase 8-11 — MOSTLY DONE (Task 12 CLAIMED)

8 Parser ✅ | 9 REPL ✅ | 10 Browser ✅ | 11 E2E ✅ | 13 Entropy ✅
→ Chi tiết: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

| ID | Task | Plan | Depends | Status | Branch | Session | Notes |
|----|------|------|---------|--------|--------|---------|-------|
| 12 | Response Intelligence | `PLAN_12_RESPONSE_INTELLIGENCE` | Phase 0 | CLAIMED | `claude/update-audit-context-2MKRJ` | 2MKRJ | Wire 5 mắt xích: walk_emotion, STM recall, intent v2, response composer, lang fix. ~560 LOC Rust. |

---

## Phase 14 — KnowTree + Silk Vertical (CRITICAL — kiến trúc)

> **T14 + T15 trong V2 Migration đã cover KnowTree + Alias. Phase này thêm Silk Vertical.**
> **Spec v3 §2.3:** "parent_map 8,846 pointers = ~71 KB (CHƯA implement)"

| ID | Task | Spec ref | Depends | Status | Branch | Session | Notes |
|----|------|----------|---------|--------|--------|---------|-------|
| 14.1 | → Xem T14 (V2 Migration) KnowTree cây phân tầng | §1.7 | T12 | FREE | | | Đã có task T14 ở V2 Migration section. |
| 14.2 | → Xem T15 (V2 Migration) Alias table tách riêng | §1.7 | T14 | FREE | | | Đã có task T15 ở V2 Migration section. |
| 14.3 | Silk vertical: parent_map 8,846 pointers | §2.3 | T14 | FREE | | | Silk dọc cho phép đi từ lá lên gốc. register_parent() hook đã có ở 7.1 nhưng chưa full impl. ~71 KB. |

## Phase 15 — Chain Optimization (Spec §IX — 6 thuật toán)

> Spec v3 liệt kê 8 thuật toán tối ưu. 2 đã implicit (Lazy Eval, Bloom Filter).
> 6 còn lại CHƯA CÓ TASK nào.

| ID | Task | Spec ref | Depends | Status | Branch | Session | Notes |
|----|------|----------|---------|--------|--------|---------|-------|
| 15.1 | Copy-on-Write chains | §IX.B | — | FREE | | | `cow_splice(chain_A, pos, new_link)`: Copy 200KB vs CoW 400B (500× hiệu quả). |
| 15.2 | Generational QR | §IX.D | — | FREE | | | 4 generations: gen0 (UDC bất tử), gen1 (read-mostly), gen2 (chuyên môn), gen3 (write-optimized). |
| 15.3 | Chain Compression | §IX.E | — | FREE | | | Detect repeats → ref + count. Tỉ lệ nén 40-60%. |
| 15.4 | Strand Complementarity | §IX.F | — | FREE | | | `complement(chain)`: invert Valence → anti-chain. Suy luận ngược, error detection. |
| 15.5 | Telomere — giới hạn sao chép | §IX.G | — | FREE | | | `chain_age += 1` mỗi lần ref. `age > threshold` → re-evaluate. |
| 15.6 | Intron/Exon marking | §IX.H | — | FREE | | | `mark_intron(chain, range)`: skip noise khi evaluate. Chain gốc không xóa. |

## Phase 16 — Fusion + Pipeline Gaps

> Fusion hiện chỉ có text modality. Spec yêu cầu 4 modalities + checkpoint 2,3,5 đầy đủ.

| ID | Task | Spec ref | Depends | Status | Branch | Session | Notes |
|----|------|----------|---------|--------|--------|---------|-------|
| 16.1 | Fusion multi-modal stub (audio+image+bio) | §V.5 | 12 | FREE | | | Bio=0.50 > Audio=0.40 > Text=0.30 > Image=0.25. |
| 16.2 | Checkpoint 2 (ENCODE) enforcement | §X CP2 | 12 | FREE | | | entities≥1, chain_hash≠0, consistency≥0.75. Vi phạm → Honesty. |
| 16.3 | Checkpoint 3 (INFER) enforcement | §X CP3 | 12 | FREE | | | ≥1 nhánh valid≥0.75, rollback check. Vi phạm → BlackCurtain. |
| 16.4 | Checkpoint 5 (RESPONSE) enforcement | §X CP5 | 12 | FREE | | | SecurityGate.check(response), tone check, confidence<0.40→im lặng. |

---

## Dependency Graph

```
Phase 0-11: ALL DONE ✅ (trừ Task 12 CLAIMED)

Phase 14 (KnowTree — CRITICAL):
  T14 (tree refactor) → T15 (alias table) → 14.3 (silk vertical)

Phase 15 (Chain Optimization — song song được):
  15.1 (CoW) | 15.2 (GenQR) | 15.3 (Compress) | 15.4 (Complement) | 15.5 (Telomere) | 15.6 (Intron/Exon)

Phase 16 (Fusion + Checkpoints — cần Task 12 xong):
  12 (Response Intelligence) → 16.1 (Fusion) | 16.2 (CP2) | 16.3 (CP3) | 16.4 (CP5)

Ưu tiên:
  P0: Task 12 (đang CLAIMED)
  P1: T14 → T15 → 14.3  (kiến trúc sai = nợ lớn nhất)
  P2: 15.1 + 15.2          (CoW + GenQR = hiệu năng quan trọng)
  P3: 16.2 + 16.3 + 16.4   (checkpoint = an toàn pipeline)
  P4: 15.3-15.6 + 16.1     (nice to have)
```

---

## INTG — ALL DONE ✅

12 test files, 120+ tests, 0 failures. `make intg` OK.
→ Chi tiết: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

---

## V2 Migration — BIG BANG (PLAN_V2_MIGRATION.md)

> **T1-T12 DONE.** T13-T16 còn FREE.
> **Ref:** `plans/AUDIT_TONG_HOP.md`, `plans/PLAN_V2_MIGRATION.md`

T1-T12 ALL DONE. → Chi tiết: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)

| ID | Task | Depends | Status | Notes |
|----|------|---------|--------|-------|
| T13 | check_logic test_bit_shifts fix | T12 | FREE | parse_rs.rs:489 — assertion `r` bit shift 10 fails. |
| T14 | **KnowTree → cây phân tầng** | T12 | FREE | ⚠️ CRITICAL: mảng phẳng → CÂY L0→L1→L2→L3. Spec v3 §1.7. |
| T15 | Alias table tách riêng | T14 | FREE | 41,338 emoji/UTF-32 → alias riêng, KHÔNG gộp KnowTree. |
| T16 | olang_handbook.md update v2 | T3 | FREE | 3 xung đột: Molecule 5B→2B, Chain, Shape 8→18. |

---

## Spec v3 Audit Summary (2026-03-21)

```
14 cơ chế DNA:  12 DONE ✅ | 2 đang làm (⑪⑭ = Task 12)
5 checkpoint:   2 hoàn chỉnh (CP1, CP4) | 3 chờ Task 12 (CP2, CP3, CP5)
8 thuật toán:   2 implicit ✅ | 6 chưa có → Phase 15
Kiến trúc:      KnowTree sai (mảng→cây) → T14 | Silk vertical chưa → 14.3
Fusion:         Chỉ text → Phase 16.1

13 task mới: Phase 14 (3) + Phase 15 (6) + Phase 16 (4)
```

---

## Log thay đổi

```
2026-03-18  Tạo TASKBOARD. B1-B3, AUTH, Phase 0.1-0.2 DONE.
2026-03-19  Phase 0-9 ALL DONE. INTG ALL DONE. origin.olang 1.35MB ELF.
2026-03-21  V2 Migration T1-T12 DONE. Spec v3 audit → Phase 14-16 thêm.
```
→ Full log: [`docs/TASKBOARD_ARCHIVE.md`](docs/TASKBOARD_ARCHIVE.md)
