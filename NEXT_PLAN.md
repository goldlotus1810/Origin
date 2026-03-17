# HomeOS — Nguồn Sự Thật Duy Nhất

> **AI mới vào: ĐỌC FILE NÀY TRƯỚC TIÊN.**
> Đây là file duy nhất phản ánh trạng thái thật của dự án.
> Các file roadmap khác (docs/roadmap.md, HomeOS_Roadmap.md) là lịch sử — KHÔNG phải trạng thái hiện tại.

**Cập nhật lần cuối:** 2026-03-17
**Checkpoint an toàn:** commit `86dff81` · branch `backup/full-seed-2026-03-17`

---

## Trạng thái hiện tại

```
Score:    8.81/10 (A)
Tests:    1,753 pass · 0 fail · 0 clippy warnings
Deps:     0 external (native SHA-256, Ed25519, AES-256-GCM, homemath)
origin:   174 nodes (35 L0 + 139 domain) · 118 edges · 1181 aliases · 37KB
Phases:   1-9 hoàn thành (xem HomeOS_Roadmap.md Section X cho lịch sử)
Parser:   18/18 RelOps (hoàn thành)
Dream:    Auto-trigger Fibonacci + STM cleanup + disk flush (hoàn thành)
```

### Điểm khôi phục nếu hỏng:
```bash
git checkout backup/full-seed-2026-03-17   # quay lại điểm an toàn
# hoặc
git checkout backup-2026-03-17-full-seed   # tag cũng trỏ cùng commit
```

---

## Đã hoàn thành (KHÔNG CẦN LÀM LẠI)

```
Foundation (Phase 1-9):
  ✅ UCD Engine (5424 entries)
  ✅ Molecule/Chain 5D encoding
  ✅ LCA weighted + variance
  ✅ Registry append-only + crash recovery
  ✅ Silk Hebbian + φ⁻¹ decay
  ✅ Emotion Pipeline 7 tầng
  ✅ 7 Instinct Skills + 15 Domain Skills
  ✅ ISL messaging (AES-256-GCM native)
  ✅ HAL (x86/ARM/RISC-V/WASM)
  ✅ VSDF (18 SDF + FFR Fibonacci)
  ✅ Agent Hierarchy (AAM/Chief/Worker)
  ✅ Compiler backends (C/Rust/WASM)
  ✅ VM 31 opcodes (arithmetic, molecular literals)
  ✅ Graph traversal (why/explain/query)
  ✅ 246 domain nodes (math, physics, chemistry, biology, philosophy, algorithms)
  ✅ Zero external dependencies
  ✅ LeoAI tự lập trình (program/express/experiment)

Phiên 2026-03-17 (hôm nay):
  ✅ P1: Parser 18/18 RelOps — ⊥ ∖ ↔ ⟶ ⟳ ↑ ⚡ ∥ (commit 38e48df)
  ✅ P2: Dream STM cleanup — promoted observations tự xóa khỏi STM
  ✅ Dream pipeline đầy đủ: auto-trigger Fibonacci + disk flush + KnowTree L3
```

---

## Việc tiếp theo (theo thứ tự ưu tiên)

### 1. Multilingual Seeding [TRUNG BÌNH]
```
Hiện tại: multilang.olang có sẵn (6057 bytes) nhưng chưa integrate
Seed thêm aliases từ nhiều ngôn ngữ (Pháp, Nhật, Hàn, Đức...)

Files:
  tools/seeder/src/multilang.rs
  multilang.olang

Test: ○{feu} → resolve đúng node 🔥 (tiếng Pháp)
```

### 2. SkillProposal [TRUNG BÌNH]
```
Docs mô tả nhưng code chưa implement
DreamSkill detect pattern → SkillProposal → AAM approve → ComposedSkill

Files:
  crates/agents/src/domain_skills.rs — thêm SkillProposalSkill
  crates/agents/src/leo.rs           — wire vào dream cycle
  crates/memory/src/proposal.rs      — SkillProposal struct

Test: pattern lặp lại → LeoAI đề xuất Skill mới → AAM approve
```

### 3. WASM Browser Demo [THẤP]
```
homeos-wasm đã có bindings nhưng chưa có demo page
Tạo simple HTML page chạy HomeOS trong browser

Files:
  crates/wasm/src/lib.rs
  crates/wasm/index.html (mới)

Test: mở browser → gõ ○{fire} → thấy kết quả
```

### 4. Cải thiện code [THẤP — làm khi rảnh]
```
- Giảm unwrap() (hiện ~291, target <100)
- Thêm tests cho tools (inspector, server, bench)
- API documentation cho core crates
```

---

## Quy trình mỗi phiên làm việc

```
1. TRƯỚC KHI LÀM:
   Đọc file NEXT_PLAN.md này         # biết đang ở đâu
   git log --oneline -3               # xác nhận đúng branch
   cargo test --workspace             # confirm green

2. SAU KHI LÀM:
   cargo test --workspace             # phải 0 fail
   cargo clippy --workspace           # phải 0 warnings
   CẬP NHẬT FILE NEXT_PLAN.md NÀY    # đánh dấu gì đã xong, gì tiếp theo
   git add ... && git commit          # commit rõ ràng
   git push                           # push lên remote

3. NẾU PHIÊN KẾT THÚC ĐỘT NGỘT:
   AI mới vào → đọc NEXT_PLAN.md → biết ngay trạng thái
   git log --oneline -5               # xem commit cuối làm gì
   cargo test --workspace             # code có xanh không
   Nếu hỏng: git checkout backup/full-seed-2026-03-17
```

---

## Ghi chú quan trọng

- **NEXT_PLAN.md = nguồn sự thật duy nhất** — docs/roadmap.md và HomeOS_Roadmap.md là lịch sử
- origin.olang PHẢI được commit sau mỗi thay đổi
- 0 external deps — giữ nguyên, không thêm crate ngoài
- LeoAI biết tự lập trình — feature mạnh, chưa khai thác hết
- Người dùng KHÔNG biết lập trình — AI phải tự quyết định kỹ thuật, chỉ hỏi về hướng đi

---

## Lịch sử phiên làm việc

```
2026-03-17 phiên 1:
  - Đồng bộ docs (13 sai lệch sửa)
  - Reseed origin.olang (35 → 174 nodes)
  - Tạo backup branches + tags
  - Score: 8.66 → 8.81

2026-03-17 phiên 2:
  - P1: Parser 18/18 RelOps (commit 38e48df)
  - P2: Dream STM cleanup
  - Tests: 1,744 → 1,753
```

---

*HomeOS · 2026-03-17 · 1,753 tests · 174 nodes · ○(∅)==○*
