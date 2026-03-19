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

Phase 7: Integration & Production (TIẾP THEO)

PLAN_7_1 (wiring)       ← Kết nối mọi thứ: AUTH, Maturity, Silk Vertical, REPL
    ↓
PLAN_7_2 (mobile)       ← Android ARM64 + iOS WASM
PLAN_7_3 (testing)      ← INTG-11/12, stress, fuzz, audit
PLAN_7_4 (network)      ← ISL over TCP/WebSocket/BLE

    7_2, 7_3, 7_4 song song được (sau 7_1)
    4_2 (fat binary) optional, có thể làm bất kỳ lúc nào
```

## Phân việc

| Plan | Skill cần | Ước tính | Song song? | Status |
|------|-----------|----------|------------|--------|
| **4_2** | Binary format | 2-3 ngày | Optional | FREE |
| **7_1** | Rust + ASM wiring | 1-2 tuần | ĐẦU TIÊN | FREE |
| **7_2** | Android/iOS, Swift | 2-3 tuần | Sau 7_1 | FREE |
| **7_3** | Testing, Rust | 3-5 ngày | Sau 7_1 | FREE |
| **7_4** | Networking, Olang | 1-2 tuần | Sau 7_1 | FREE |

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

# Status: Phase 0-6 DONE. 50 .ol files compile. B1-B7 fixed.
# Next: Phase 7 — wire everything, mobile, testing, network.
```
