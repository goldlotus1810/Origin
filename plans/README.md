# Plans — Chi tiết triển khai

**Kim chỉ nam:** `PLAN_REWRITE.md` (root)
**Memory:** `LYRA.md` (project memory cho Lyra VM sessions)

---

## Dependency graph

```
Phase 0-6: ALL DONE ✅

PLAN_0_1 → 0_2 → 0_3 → 0_4 → 0_5 → 0_6     ← Bootstrap compiler ✅
PLAN_1_1 → 1_2 → 1_3 → 1_4                    ← Machine code VMs ✅
PLAN_2_1 → 2_2 → 2_3 → 2_4                    ← Stdlib + HomeOS logic ✅
PLAN_3_1 → 3_2 → 3_3 → 3_4                    ← Self-sufficient builder ✅
PLAN_AUTH                                       ← First-run setup ✅
PLAN_4_1 → 4_3                                 ← Multi-architecture ✅
PLAN_5_1 → 5_2 → 5_3 → 5_4                    ← Optimization ✅
PLAN_6_1 → 6_2 → 6_3                          ← Living system ✅

Phase 7: Integration & Production ✅ (trừ 7.2 đang làm)

PLAN_7_1 (wiring)       ✅ DONE
PLAN_7_2 (mobile)       ← Android ARM64 + iOS WASM (Kira đang làm)
PLAN_7_3 (testing)      ✅ DONE — 140 intg tests
PLAN_7_4 (network)      ✅ DONE — 4 Olang files (~820 LOC)

Phase 8-11: End-to-End (MỚI — làm cho origin.olang THỰC SỰ chạy được)

PLAN_8 (parser upgrade)    ← Unlock 24/54 files: hex literals, ==, keywords
    ↓
PLAN_9 (native REPL)      ← ./origin → compile + execute user input
PLAN_10 (browser E2E)     ← origin.html → WASM compile + execute
    ↓
PLAN_11 (E2E verify)      ← make demo, make verify, CI/CD

    8 PHẢI xong trước 9, 10
    9, 10 song song được
    11 phần đầu (server --eval) làm song song với 8
```

## Phân việc

| Plan | Skill cần | Ước tính | Song song? | Status |
|------|-----------|----------|------------|--------|
| ~~4_2~~ | Binary format | — | — | DONE (Kaze) |
| ~~7_1~~ | Rust + ASM wiring | — | — | DONE (Kira) |
| **7_2** | Android/iOS | 2-3 tuần | Song song | CLAIMED (Kira) |
| ~~7_3~~ | Testing | — | — | DONE (Lyra) |
| ~~7_4~~ | Networking | — | — | DONE (Lyra) |
| **8** | Rust parser | 4-8h | ĐẦU TIÊN | FREE |
| **9** | x86_64 ASM | 12-20h | Sau 8 | FREE |
| **10** | WAT + JS + HTML | 10-16h | Song song với 9 | FREE |
| **11** | Shell + Rust + CI | 8-12h | Phần đầu song song với 8 | FREE |

## Quick start cho developer mới

```bash
# Clone + build
git clone <repo> && cd Origin
cargo build --workspace

# Chạy tests (2,500+ tests, tất cả pass)
cargo test --workspace

# Đọc bối cảnh (theo thứ tự)
1. CLAUDE.md                    ← hiến pháp, quy tắc bất biến
2. PLAN_REWRITE.md              ← kim chỉ nam tổng thể
3. LYRA.md                      ← project memory (VM, bytecode, status)
4. TASKBOARD.md                 ← xem task nào FREE
5. plans/PLAN_7_1_wiring.md     ← plan tiếp theo cần làm

# Build origin.olang
make                               ← assemble VM + compile stdlib + pack

# File code quan trọng
crates/olang/src/exec/vm.rs       ← Rust VM (6,083 LOC)
crates/olang/src/exec/ir.rs       ← Op enum (36+ opcodes)
crates/olang/src/lang/semantic.rs  ← Compiler (8,994 LOC)
vm/x86_64/vm_x86_64.S             ← x86_64 ASM VM (~1,700 LOC)
vm/arm64/vm_arm64.S                ← ARM64 ASM VM (627 LOC)
vm/wasm/vm_wasm.wat                ← WASM VM (830 LOC)
vm/wasm/vm_wasi.wat                ← WASI VM (643 LOC)
tools/builder/                     ← Rust builder (pack VM+bytecode+knowledge)
stdlib/bootstrap/                  ← Self-hosting compiler (4 files)
stdlib/                            ← Core stdlib (18 files)
stdlib/homeos/                     ← HomeOS modules (28 files)
Makefile                           ← Build automation

# Status: Phase 0-7 mostly DONE. 54 .ol files (30 compile, 24 known parse failures).
# Next: Phase 8-11 — Parser upgrade, native REPL, browser E2E, verification.
# Goal: anyone can run ./origin and SEE it work.
```
