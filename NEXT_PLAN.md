# HomeOS — NEXT_PLAN

> **AI mới vào: ĐỌC FILE NÀY TRƯỚC TIÊN. Không sửa file nào cho đến khi hiểu hết.**
> Người dùng KHÔNG biết lập trình. AI tự quyết định kỹ thuật, chỉ hỏi về hướng đi.
> Sau mỗi phiên: CẬP NHẬT file này với những gì đã làm và chưa làm.

**Cập nhật:** 2026-03-17
**Branch:** `claude/debug-github-issues-x8R9F`

---

## Trạng thái thật (verify bằng code)

```
Tests:    1,774 pass · 0 fail · 0 clippy warnings
Deps:     0 external runtime (native SHA-256, Ed25519, AES-256-GCM, homemath)
origin:   313 nodes (35 L0 + 278 domain) · 236 edges · 2246 aliases · 72KB
```

### HOẠT ĐỘNG:
```
✅ UCD Engine (5424 entries, hierarchical byte encoding)
✅ Molecule/Chain 5D + tagged sparse encoding (1-6 bytes)
✅ LCA weighted + variance + hierarchical base-aware mode
✅ Registry append-only + crash recovery
✅ Silk Hebbian + φ⁻¹ decay (82 tests)
✅ Emotion Pipeline 7 tầng (chạy trong runtime)
✅ VM 31 opcodes + arithmetic (○{1+2}=3)
✅ Parser 18/18 RelOps
✅ Phase 9: Zero external deps
✅ Phase 2: find_path, trace_origin, reachable (34 tests, wired vào why/explain)
✅ Phase 1: VM arithmetic
✅ Dream auto-trigger Fibonacci + STM cleanup + disk flush
✅ SecurityGate (chặn Crisis input)
✅ ISL messaging (4-byte address, AES-256-GCM encrypted)
✅ HAL (x86/ARM/RISC-V/WASM detect)
✅ VSDF (18 SDF + FFR Fibonacci render)
✅ NodeBody + BodyStore (chain_hash → SDF + Spline)
✅ Molecule.evolve() (mutation 1/5 chiều → loài mới)
✅ Phase 5: Agent Orchestration — MessageRouter + Chiefs wired vào runtime
✅ Phase 4: Math → Silk — solve/derive/integrate kết quả vào STM + Silk
✅ Phase 3: Domain Knowledge — 313 nodes seeded (6 domains)
✅ SkillProposal — ComposedSkill + SkillPattern + SkillPatternStore + wired vào LeoAI
✅ Unwrap reduction: 50 → 27 production unwraps (partial_cmp→total_cmp, try_into→array)
```

---

## ĐÃ HOÀN THÀNH (Phiên I)

### SkillProposal — ComposedSkill pipeline ✅
```
ComposedSkill: pipe N skills in sequence, respects QT4③
  execute_with() + resolver callback → skills don't know each other
  Pipeline: skill[0].output_chains → skill[1].input_chains → ...

SkillPattern: learned skill execution sequences
  observe_success/failure → running average effectiveness
  to_composed() when observations ≥ 3 AND effectiveness ≥ 0.6

SkillPatternStore: accumulate + auto-promote patterns
  record(steps, success, ts) → auto-promote effective patterns
  Sits in LeoAI.skill_patterns — wired into run_instincts()

AAM review_skill(InsightKind::SkillPattern):
  ≥3 obs + ≥0.6 eff → Approved
  <3 obs → Pending
  <0.6 eff → Rejected

15 new tests: ComposedSkill pipeline/error/insufficient,
  SkillPattern observe/promote/threshold,
  SkillPatternStore record/promote/dedup/mixed,
  AAM SkillPattern review approved/pending/rejected

Files đã sửa:
  crates/agents/src/skill.rs — ComposedSkill, SkillPattern, SkillPatternStore + tests
  crates/agents/src/leo.rs — skill_patterns field, wired into run_instincts()
  crates/memory/src/proposal.rs — InsightKind::SkillPattern + AAM review + tests
```

---

## ĐÃ HOÀN THÀNH (Phiên H)

### Phase 5 — Agent Orchestration ✅
```
Đường A (user) và Đường B (agent) ĐÃ NỐI:

process_input()
  → T6: learning.process_one()
  → T6f: ISL Learn message → LeoAI.receive_isl() → poll_inbox()
  → router.tick(workers, chiefs, leo, ts)
  → drain router pending_writes

HomeRuntime giờ có:
  router: MessageRouter     — central dispatcher
  chiefs: Vec<Chief>        — 3 chiefs (Home, Vision, Network) auto-boot
  workers: Vec<Worker>      — register_worker() API

Stats command hiển thị Router summary + Chief/Worker counts.
6 tests mới: router ticks, multi-turn, stats display.

Files đã sửa:
  crates/runtime/src/origin.rs — imports, struct fields, boot_chiefs(),
    process_input() T6f agent pump, handle_command() stats, accessors
```

### Phase 4 — Math → Silk ✅
```
handle_command() math section giờ gọi process_one(ContentInput::Math)
  → encode_math() → chain vào STM + Silk
  → math kết quả ĐƯỢC HỌC vào graph

2 tests mới: math_result_enters_learning, math_derive_enters_learning

Files đã sửa:
  crates/runtime/src/origin.rs — math handler gọi process_one()
```

### Phase 3 — Domain Knowledge ✅
```
Chạy seed_domains binary → 139 domain nodes seeded thêm vào origin.olang
origin.olang: 174 → 313 nodes, 118 → 236 edges, 1181 → 2246 aliases

6 domains: math(61), physics(26), chemistry(12), biology(10), philosophy(15), algorithms(15)
```

### Dọn dẹp docs ✅
```
Tất cả docs cũ → old/2026-03-17/ (13 files)
Root chỉ giữ: CLAUDE.md, NEXT_PLAN.md, README.md
docs/ chỉ giữ: olang_guide.md
```

---

## Tiếp theo (ưu tiên)

```
1. [TRUNG BÌNH] Multilingual Seeding — multilang.olang integrate
2. [THẤP]       WASM Browser Demo
3. [THẤP]       API documentation cho core crates
```

---

## Quy trình mỗi phiên

```
1. TRƯỚC: Đọc NEXT_PLAN.md → git log -5 → cargo test --workspace
2. TRONG: Commit thường xuyên (phiên có thể kết thúc bất ngờ)
3. SAU:   cargo test + clippy → CẬP NHẬT NEXT_PLAN.md → commit + push
```

---

## Lịch sử phiên

```
A: Viết Phase 1-8 liên tục → code tồn tại nhưng chất lượng kém
B: Molecule encoding, Dream pipeline
C: Phase 9 Zero deps (SHA-256, Ed25519, AES-256-GCM)
D: Tagged encoding, hierarchical bytes, NodeBody, evolve(), benchmark, Phase 1 VM arithmetic
E: Đồng bộ docs, reseed origin.olang
F: P1 RelOps 18/18, Dream STM cleanup
G: Verify Phase 2-5 thực trạng, cập nhật NEXT_PLAN
H: Dọn docs → old/, Phase 5+4+3 HOÀN THÀNH, 1759 tests
I: SkillProposal + unwrap reduction (50→27), 1774 tests
```

---

## Tài liệu cũ

```
old/2026-03-17/ chứa 13 files docs cũ
Chỉ giữ: CLAUDE.md, NEXT_PLAN.md, README.md, docs/olang_guide.md
```

---

*HomeOS · 2026-03-17 · 1,774 tests · 313 nodes · ○(∅)==○*
