# Plans — Chi tiết triển khai

**Kim chỉ nam:** `PLAN_REWRITE.md` (root)
**Memory:** `LYRA.md` (project memory cho Lyra VM sessions)

---

## Dependency graph

```
Phase 0-3: ALL DONE ✅

PLAN_0_1 → 0_2 → 0_3 → 0_4 → 0_5 → 0_6     ← Bootstrap compiler ✅
PLAN_1_1 → 1_2 → 1_3 → 1_4                    ← Machine code VMs ✅
PLAN_2_1 → 2_2 → 2_3 → 2_4                    ← Stdlib + HomeOS logic ✅
PLAN_3_1 → 3_2 → 3_3 → 3_4                    ← Self-sufficient builder ✅
PLAN_AUTH                                       ← First-run setup ✅

Phase 4: Multi-architecture (TIẾP THEO)

PLAN_4_1 (cross-compile ARM64)     ← viết asm_emit_arm64.ol + fix op_call
    ↓
PLAN_4_2 (fat binary)              ← optional, multi-arch trong 1 file

PLAN_4_3 (WASM universal)          ← song song với 4_1, bytecode embed + browser

Phase 5: Optimization

PLAN_5_1 (JIT compilation)         ← hot loop → native code, cần asm_emit.ol
PLAN_5_2 (inline caching)          ← var IC + registry LRU + silk cache
PLAN_5_3 (memory optimization)     ← arena allocator + zero-copy + pool
PLAN_5_4 (benchmark)               ← đo lường, cần __time_ns() builtin

    5_1, 5_2, 5_3 song song được
    5_4 sau khi có ít nhất 1 optimization

Phase 6: Living system

PLAN_6_1 (self-update)             ← o install/update/learn, cần Phase 4
    ↓
PLAN_6_2 (self-optimize)           ← LeoAI profile + optimize, cần 5_1 + 6_1
    ↓
PLAN_6_3 (reproduce)               ← Worker clones, cần 4_1 + 6_1
```

## Phân việc

| Plan | Skill cần | Ước tính | Song song? | Status |
|------|-----------|----------|------------|--------|
| **4_1** | ARM64 ISA, ASM | 3-5 ngày | Song song với 4_3 | FREE |
| **4_2** | Binary format | 2-3 ngày | Sau 4_1 | FREE (optional) |
| **4_3** | WASM binary, JS | 3-5 ngày | Song song với 4_1 | FREE |
| **5_1** | x86 ASM, JIT design | 1-2 tuần | Song song với 5_2, 5_3 | FREE |
| **5_2** | Cache design, ASM | 3-5 ngày | Song song với 5_1, 5_3 | FREE |
| **5_3** | Memory management, ASM | 3-5 ngày | Song song với 5_1, 5_2 | FREE |
| **5_4** | Benchmarking | 3-5 ngày | Sau 5_1/5_2/5_3 | FREE |
| **6_1** | File I/O, self-modify | 1-2 tuần | Sau Phase 4 | FREE |
| **6_2** | AI/ML, profiling | 2-3 tuần | Sau 5_1 + 6_1 | FREE |
| **6_3** | IoT, ISL, security | 1-2 tuần | Sau 4_1 + 6_1 | FREE |

## Quick start cho developer mới

```bash
# Clone + build
git clone <repo> && cd Origin
cargo build --workspace

# Chạy tests (2,491 tests, tất cả pass)
cargo test --workspace

# Đọc bối cảnh (theo thứ tự)
1. CLAUDE.md                    ← hiến pháp, quy tắc bất biến
2. PLAN_REWRITE.md              ← kim chỉ nam tổng thể
3. LYRA.md                      ← project memory (VM, bytecode, status)
4. TASKBOARD.md                 ← xem task nào FREE
5. plans/PLAN_4_1_*.md          ← plan tiếp theo cần làm

# Build origin.olang (1.35 MB single-file executable)
make                               ← assemble VM + compile stdlib + pack

# File code quan trọng
crates/olang/src/exec/vm.rs       ← Rust VM (6,083 LOC)
crates/olang/src/exec/ir.rs       ← Op enum (36+ opcodes)
crates/olang/src/lang/semantic.rs  ← Compiler (8,994 LOC)
vm/x86_64/vm_x86_64.S             ← x86_64 ASM VM (~1,700 LOC)
vm/arm64/vm_arm64.S                ← ARM64 ASM VM (627 LOC)
vm/wasm/vm_wasm.wat                ← WASM VM (655 LOC)
tools/builder/                     ← Rust builder (pack VM+bytecode+knowledge)
stdlib/bootstrap/                  ← Self-hosting compiler
stdlib/homeos/                     ← HomeOS logic + builder
Makefile                           ← Build automation

# Vấn đề thực tế (xem TASKBOARD.md blockers B4-B7)
# 7/22 stdlib files không compile (parser limitations)
# VM exit ngay sau load (chưa có entry point dispatch)
```
