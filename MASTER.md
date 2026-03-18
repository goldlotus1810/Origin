# HomeOS — Master Document

> AI mới vào: ĐỌC FILE NÀY + CLAUDE.md TRƯỚC KHI LÀM GÌ.
> Sau mỗi phiên: CẬP NHẬT file này.

**Cập nhật:** 2026-03-18
**Tests:** 2,227 pass · 0 fail · 0 clippy warnings · 0 external deps
**Code:** ~82,000 lines Rust · 11 crates + 4 tools · no_std core

---

## Trạng Thái Thật (verified by running code)

### HOẠT ĐỘNG (wired + working):
```
✅ UCD Engine (5424 entries, hierarchical byte encoding)
✅ Molecule/Chain 5D + tagged sparse encoding (1-6 bytes)
✅ LCA weighted + variance + hierarchical base-aware mode
✅ Registry append-only + crash recovery
✅ Silk Hebbian + φ⁻¹ decay
✅ Emotion Pipeline 7 layers (end-to-end in runtime)
✅ VM 36 opcodes + arithmetic (○{1+2}=3, ○{solve "2x+3=7"})
✅ Parser 18/18 RelOps
✅ SecurityGate (Crisis detection + hotline)
✅ ISL messaging (4-byte address, AES-256-GCM)
✅ HAL (x86/ARM/RISC-V/WASM detect)
✅ VSDF (18 SDF + FFR Fibonacci render)
✅ NodeBody + BodyStore (chain_hash → SDF + Spline)
✅ Molecule.evolve() (mutation 1/5 dim → new species)
✅ Agent hierarchy (AAM/Chief/Worker boot)
✅ Compiler backends (C/Rust/WASM)
✅ WASM browser bindings
✅ Zero external deps (native SHA-256, Ed25519, AES-256-GCM, homemath)
✅ Dream auto-trigger + STM cleanup + cluster + QR promote
✅ Instincts wired into response flow
✅ LeoAI self-programming (○{program ...})
✅ KnowTree L2/L3 concepts
✅ SkillPattern → AAM approval pipeline
✅ Multilingual seeding (7 languages, 432+ aliases)
✅ Codebase restructured (subdirectories per crate)
```

### FACADE (code exists, not functionally active):
```
⚠️ Chiefs (3) — boot but idle, 0 messages processed
⚠️ Workers (0) — register_worker() exists, never called
⚠️ ISL routing between agents — LeoAI receives, Chiefs/Workers don't participate
⚠️ Compiler targets — code exists, no end-to-end pipeline
⚠️ Book reader — code exists, not wired to runtime
⚠️ Domain skills (15) — structs exist, only instincts call them
⚠️ Maturity pipeline — enum+advance() exist, NOT wired to STM/Dream
⚠️ Silk parent pointer — thiết kế 43KB vertical Silk, chưa implement
```

### SPEC AUDIT — 6 Vấn Đề Hệ Thống (phiên L):
```
#1 Response template      — ~10 câu cố định, bỏ qua instinct+Silk output    [HIGH, MEDIUM effort]
#2 Parser missing 6 cmds  — typeof/explain/why/trace/inspect/assert           [HIGH, SMALL effort]
#3 Maturity pipeline       — advance(weight=0.0) BUG → Mature unreachable     [HIGH, SMALL effort]
#4 Dream threshold         — cluster_score ≈ 0.10 << threshold 0.6            [MEDIUM, SMALL effort]
#5 Silk vertical (parent)  — 5460 pointers × 8B = 43KB, chưa có              [HIGH, MEDIUM effort]
#6 Agent hierarchy dead    — Chiefs idle, 0 Workers, 0 ISL messages           [HIGH, LARGE effort]
```

### SPEC FILES:
```
SPEC_MATURITY_PIPELINE.md  — Wire Maturity vào Dream (covers #3, #4, maps #1-#6)
SPEC_NODE_SILK.md          — 5 Gaps: parent pointer, compound, Dream 5D, layer, unified_neighbors
```

---

## Test Counts Per Crate

| Crate | Tests | Status |
|-------|-------|--------|
| olang | 838 | Core engine, fully tested |
| agents | 282 | Encoder, learning, gate, instincts |
| runtime | 273 | HomeRuntime, parser, router |
| context | 168 | Emotion, intent, curve, fusion |
| vsdf | 123 | SDF, FFR, physics, scene |
| silk | 88 | Graph, hebbian, walk, edges |
| hal | 85 | Arch, platform, security, tier |
| memory | 65 | STM, dream, proposals |
| wasm | 32 | WASM bindings, bridge |
| isl | 31 | Address, message, codec, queue |
| ucd | 23 | Unicode lookup table |
| homemath | 18 | Pure math functions |
| seeder | 15 | L0 node seeding |
| server | 13 | REPL boot/run |
| inspector | 9 | File verification |
| **Total** | **2,227** | |

---

## Lịch Sử Phiên

```
A: Phase 1-8 code skeleton
B: Molecule encoding, Dream pipeline
C: Phase 9 Zero deps (SHA-256, Ed25519, AES-256-GCM)
D: Tagged encoding, hierarchical bytes, NodeBody, evolve(), VM arithmetic
E: Docs sync, reseed origin.olang
F: RelOps 18/18, Dream STM cleanup
G: Verify Phase 2-5 status, update docs
H: Docs → old/, Phase 5+4+3 complete, 1759 tests
I: SkillProposal + unwrap reduction + Multilingual Seeding, 1774 tests
J: SkillPattern → AAM pipeline + test coverage + warning cleanup, 1784 tests
K: Honest audit — response template, command parsing, agent orchestration issues identified
   Response generation improved, instincts wired to output, Dream fixed
   Codebase restructured into subdirectories, 2063 tests
L: Spec audit — 2 specs created (SPEC_MATURITY_PIPELINE, SPEC_NODE_SILK)
   6 systemic issues mapped, 1 critical bug found (advance weight=0.0)
   Phase 1-3 Olang features verified, 2227 tests
```

---

## Quy Trình Mỗi Phiên

```
1. TRƯỚC: Đọc MASTER.md + CLAUDE.md → git log -10 → cargo test --workspace
2. TRONG: Commit thường xuyên (phiên có thể kết thúc bất ngờ)
3. SAU:   cargo test + clippy → CẬP NHẬT MASTER.md → commit + push
```

---

## Tài Liệu

| File | Nội dung |
|------|---------|
| [CLAUDE.md](CLAUDE.md) | Hướng dẫn cho AI contributors (bất biến) |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Kiến trúc tổng thể |
| [MASTER.md](MASTER.md) | File này — trạng thái dự án + lịch sử |
| [PLAN.md](PLAN.md) | Kế hoạch tiếp theo |
| [README.md](README.md) | Giới thiệu dự án |
| [REVIEW.md](REVIEW.md) | Đánh giá trung thực |
| old/ | Tài liệu cũ (2026-03-17, 2026-03-18) |

---

*HomeOS · 2026-03-18 · 2,227 tests · ~82K LoC · ○(∅)==○*
