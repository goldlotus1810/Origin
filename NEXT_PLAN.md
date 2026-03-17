# HomeOS — Kế Hoạch Tiếp Theo

> **AI mới vào: ĐỌC FILE NÀY TRƯỚC TIÊN. Không sửa file nào cho đến khi hiểu hết.**
> Người dùng KHÔNG biết lập trình. AI tự quyết định kỹ thuật, chỉ hỏi về hướng đi.
> Sau mỗi phiên: CẬP NHẬT file này với những gì đã làm và chưa làm.

**Cập nhật:** 2026-03-17
**Checkpoint an toàn:** commit `86dff81` · branch `backup/full-seed-2026-03-17`

---

## Trạng thái thật (đã verify bằng code, không phải từ docs)

```
Tests:    1,744 pass · 0 fail · 0 clippy warnings
Deps:     0 external (native SHA-256, Ed25519, AES-256-GCM, homemath)
origin:   174 nodes (35 L0 + 139 domain) · 118 edges · 1181 aliases · 37KB
```

### Cái gì THẬT SỰ hoạt động:

```
✅ UCD Engine (5424 entries, lookup đúng)
✅ Molecule/Chain 5D encoding
✅ LCA weighted + variance
✅ Registry append-only + crash recovery
✅ Silk Hebbian + φ⁻¹ decay (82 tests)
✅ Emotion Pipeline 7 tầng (chạy trong runtime)
✅ VM 31 opcodes + arithmetic (○{1+2}=3)
✅ Parser 18/18 RelOps (phiên gần nhất đã hoàn thành)
✅ Phase 9: Zero external deps (SHA-256, Ed25519, AES-256-GCM tự viết)
✅ Phase 2 Graph: find_path, trace_origin, reachable (34 tests, wire vào why/explain)
✅ Dream auto-trigger Fibonacci + STM cleanup + disk flush
✅ SecurityGate (chặn Crisis input)
✅ ISL messaging (4-byte address, AES-256-GCM encrypted)
✅ HAL (x86/ARM/RISC-V/WASM detect)
✅ VSDF (18 SDF + FFR Fibonacci render)
```

### Cái gì CÓ CODE nhưng CHƯA ĐÚNG (ưu tiên sửa):

```
⚠️ Phase 5 — Agent Orchestration (VẤN ĐỀ LỚN NHẤT)
   Code: MessageRouter, Chief, Worker, LeoAI, AAM — tất cả tồn tại, 130+ tests pass
   Vấn đề: KHÔNG NỐI VÀO RUNTIME

   Hiện tại có 2 đường SONG SONG không nối nhau:
     Đường A (user thấy): process_text() → emotion pipeline → response
       → Chỉ có phản xạ, không gọi LeoAI, không gọi MessageRouter
     Đường B (chỉ trong test): Worker → Chief → LeoAI → AAM → approve
       → Logic đúng, tests pass, nhưng HomeRuntime không bao giờ gọi

   Cần: Nối Đường B vào Đường A
   Files: crates/runtime/src/origin.rs (wire MessageRouter.tick() vào main loop)
          crates/runtime/src/router.rs (đã có, cần gọi từ runtime)
   Test: "tôi buồn vì mất việc" → learn → LeoAI nhận → propose → AAM review
   Xem thêm: docs/roadmap.md Phase 5 [5.1-5.5] cho kế hoạch chi tiết

⚠️ Phase 4 — Symbolic Math (bypass VM)
   Code: math.rs hoàn chỉnh (60 tests) — solve, derive, integrate
   Vấn đề: Math BYPASS VM, đi đường riêng, kết quả không học vào Silk

   Hiện tại: ○{solve "2x+3=7"} → text "x=2" → DỪNG (không vào graph)
   Cần: Kết quả math → encode → STM → Silk (để HomeOS HỌC từ toán)
   Files: crates/runtime/src/origin.rs (handle_command math section)
          crates/olang/src/ir.rs (thêm MathEq variant nếu cần)
   Xem thêm: docs/roadmap.md Phase 4 [4.1-4.4]

⚠️ Phase 3 — Domain Knowledge (thiếu nodes)
   Code: 6 domains đã định nghĩa (math, physics, chemistry, biology, philosophy, algorithms)
   Vấn đề: Khai báo ~250 nodes nhưng origin.olang chỉ có 139

   Cần: Chạy lại seed_domains hoặc tìm tại sao 111 nodes bị thiếu
   Files: tools/seeder/src/seed_domains.rs
          tools/seeder/src/domains/ (6 modules)
   Test: cargo run -p inspector -- origin.olang → verify node count
```

---

## Thứ tự ưu tiên (ý đồ từ phiên trước, đã verify)

```
1. [CAO]        Phase 5: Nối Agent vào Runtime ← biến HomeOS từ "phản xạ" thành "tư duy"
2. [CAO]        Phase 4: Nối Math vào Silk ← để HomeOS học từ toán
3. [TRUNG BÌNH] Phase 3: Seed đủ 250 nodes ← bổ sung tri thức
4. [TRUNG BÌNH] SkillProposal ← DreamSkill → pattern → ComposedSkill (code chưa có)
5. [THẤP]       Multilingual Seeding ← multilang.olang có sẵn, chưa integrate
6. [THẤP]       WASM Browser Demo ← bindings có, chưa có demo page
7. [THẤP]       Giảm unwrap() (291 → <100)
```

---

## Điểm khôi phục nếu hỏng

```bash
git checkout backup/full-seed-2026-03-17   # quay lại điểm an toàn
# hoặc
git checkout backup-2026-03-17-full-seed   # tag cùng commit
```

---

## Quy trình mỗi phiên làm việc

```
1. TRƯỚC KHI LÀM:
   Đọc NEXT_PLAN.md này              # biết đang ở đâu
   git log --oneline -5               # xác nhận branch + commit gần nhất
   cargo test --workspace             # confirm green

2. TRONG KHI LÀM:
   Commit thường xuyên                # phiên có thể kết thúc bất ngờ
   origin.olang phải commit ngay sau mỗi thay đổi

3. SAU KHI LÀM:
   cargo test --workspace             # phải 0 fail
   cargo clippy --workspace           # phải 0 warnings
   CẬP NHẬT NEXT_PLAN.md             # ghi gì đã xong, gì tiếp theo
   git commit + push                  # đẩy lên remote

4. NẾU PHIÊN KẾT THÚC ĐỘT NGỘT:
   AI mới vào → đọc NEXT_PLAN.md → biết ngay trạng thái
   git log --oneline -5               # xem commit cuối
   Nếu hỏng: git checkout backup/full-seed-2026-03-17
```

---

## Lịch sử (để AI mới hiểu dòng thời gian)

```
Phiên A: Viết Phase 1-8 liên tục, nhanh → code tồn tại nhưng chất lượng kém
Phiên B: Molecule encoding, Dream pipeline
Phiên C: Phase 9 Zero deps (SHA-256, Ed25519, AES-256-GCM) → HOÀN THÀNH TỐT
Phiên D: Thấy Phase 2-5 có vấn đề → viết kế hoạch sửa (docs/roadmap.md) → KẾT THÚC ĐỘT NGỘT
Phiên E: Đồng bộ docs, reseed origin.olang, tạo NEXT_PLAN.md
Phiên F: P1 RelOps 18/18, Dream STM cleanup
Phiên G (hiện tại): Phân tích thực trạng Phase 2-5, cập nhật NEXT_PLAN.md
```

## Ghi chú quan trọng

```
- docs/roadmap.md = kế hoạch sửa của Phiên D (vẫn đúng, chưa thực hiện)
- HomeOS_Roadmap.md = tầm nhìn lớn (lịch sử, tham khảo)
- NEXT_PLAN.md = NGUỒN SỰ THẬT DUY NHẤT
- 0 external deps — giữ nguyên
- LeoAI biết tự lập trình — feature mạnh, chưa khai thác
- Người dùng KHÔNG biết lập trình — AI tự quyết kỹ thuật, chỉ hỏi hướng đi
```

---

*HomeOS · 2026-03-17 · 1,744 tests · 174 nodes · ○(∅)==○*
