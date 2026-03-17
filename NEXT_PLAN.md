# HomeOS — Kế Hoạch Tiếp Theo
**Ngày:** 2026-03-17
**Checkpoint:** commit `86dff81` · branch `backup/full-seed-2026-03-17`

---

## Trạng thái hiện tại

```
Score:    8.81/10 (A)
Tests:    1,744 pass · 0 fail · 0 clippy warnings
Deps:     0 external (native SHA-256, Ed25519, AES-256-GCM, homemath)
origin:   174 nodes (35 L0 + 139 domain) · 118 edges · 1181 aliases · 37KB
Phases:   1-9 hoàn thành
Docs:     đã đồng bộ với code thật (13 sai lệch đã sửa)
```

### Điểm khôi phục nếu hỏng:
```bash
git checkout backup/full-seed-2026-03-17   # quay lại điểm an toàn
# hoặc
git checkout backup-2026-03-17-full-seed   # tag cũng trỏ cùng commit
```

---

## Ưu tiên phát triển (theo thứ tự)

### P1: Olang Parser — Thêm RelOps thiếu [CAO]
```
Hiện tại: Parser hỗ trợ 10/18 RelOps
Thiếu:    ⊥(Ortho) ∖(SetMinus) ↔(Bidir) ⟶(Flows) ⟳(Repeats) ↑(Resolves) ⚡(Trigger) ∥(Parallel)

Files cần sửa:
  crates/runtime/src/parser.rs   — thêm token recognition
  crates/olang/src/syntax.rs     — thêm vào grammar
  crates/olang/src/compiler.rs   — compile xuống Edge opcode

Test: ○{fire ⊥ water} → tạo edge Orthogonal
```

### P2: Dream→KnowTree L3 Pipeline [CAO]
```
STM observations → Dream cluster → LCA → L3 node → KnowTree
Fibonacci trigger: Fib[depth] co-activations để promote

Files:
  crates/memory/src/dream.rs     — DreamCycle.run()
  crates/olang/src/knowtree.rs   — store_concept()
  crates/agents/src/leo.rs       — run_dream() integration
  crates/runtime/src/origin.rs   — wire dream into main loop

Test: learn 10 câu về "lửa" → dream → L3 node "thermodynamics" tự sinh
```

### P3: Multilingual Seeding [TRUNG BÌNH]
```
Hiện tại: multilang.olang có sẵn (6057 bytes) nhưng chưa integrate
Seed thêm aliases từ nhiều ngôn ngữ

Files:
  tools/seeder/src/multilang.rs
  multilang.olang

Test: ○{feu} → resolve đúng node 🔥 (tiếng Pháp)
```

### P4: SkillProposal [TRUNG BÌNH]
```
Docs mô tả nhưng code chưa implement
DreamSkill detect pattern → SkillProposal → AAM approve → ComposedSkill

Files:
  crates/agents/src/domain_skills.rs — thêm SkillProposalSkill
  crates/agents/src/leo.rs           — wire vào dream cycle
  crates/memory/src/proposal.rs      — SkillProposal struct

Test: pattern lặp lại → LeoAI đề xuất Skill mới → AAM approve
```

### P5: WASM Browser Demo [THẤP]
```
homeos-wasm đã có bindings nhưng chưa có demo page
Tạo simple HTML page chạy HomeOS trong browser

Files:
  crates/wasm/src/lib.rs
  crates/wasm/index.html (mới)

Test: mở browser → gõ ○{fire} → thấy kết quả
```

---

## Quy trình mỗi phiên làm việc

```
1. TRƯỚC KHI LÀM:
   git log --oneline -3              # xác nhận đúng branch
   cargo test --workspace            # confirm green
   cargo run -p inspector -- origin.olang  # verify data

2. SAU KHI LÀM:
   cargo test --workspace            # phải 0 fail
   cargo clippy --workspace          # phải 0 warnings
   cargo run -p inspector -- origin.olang  # verify intact
   git add ... && git commit         # commit rõ ràng
   git tag backup-YYYY-MM-DD-{desc}  # tag backup
   git push                          # push lên remote

3. NẾU HỎNG:
   git checkout backup/full-seed-2026-03-17  # quay lại
   cargo run -p seeder --bin seeder          # reseed L0 nếu cần
   cargo run -p seeder --bin seed_domains    # reseed domains nếu cần
```

---

## Ghi chú cho phiên sau

- origin.olang PHẢI được commit sau mỗi seed/learn — nếu phiên kết thúc bất ngờ, Git giữ data
- Backup branch + tag TRƯỚC KHI làm gì mới
- Docs đã đồng bộ tại commit `659880f` — không cần cập nhật trừ khi thêm feature mới
- LeoAI đã biết tự lập trình (program/express/experiment) — đây là feature mạnh, chưa khai thác hết
- 0 external deps — giữ nguyên, không thêm crate ngoài

---

*HomeOS · 2026-03-17 · 1,744 tests · 174 nodes · ○(∅)==○*
