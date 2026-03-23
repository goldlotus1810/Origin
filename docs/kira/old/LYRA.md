# LYRA — Project Memory for AI Sessions

> **Đọc file này khi tiếp tục công việc trên Lyra VM / Origin compiler.**
> File này tóm tắt TOÀN BỘ kiến trúc, trạng thái, và bước tiếp theo.
> Cập nhật lần cuối: 2026-03-19

---

## Tổng quan

Origin là một ngôn ngữ lập trình tự chứa (self-contained). Mục tiêu cuối cùng:
**1 file `origin.olang` = VM + Compiler + Stdlib + Knowledge = tự chạy, 0 dependency.**

Dự án gồm 2 phần chính:
1. **Rust codebase** (HomeOS) — nền tảng hiện tại, đang dần được thay thế
2. **Olang + Lyra VM** — hệ thống mới, viết bằng Olang + native assembly

---

## Kiến trúc Lyra VM

### 3 VM targets

| Target | File | LOC | Status |
|--------|------|-----|--------|
| x86_64 | `vm/x86_64/vm_x86_64.S` | 1680 | Working. Dual-format dispatch. FNV-1a var table. |
| ARM64 | `vm/arm64/vm_arm64.S` | 627 | Builds. Store/load fixed. Needs QEMU runtime test. |
| WASM | `vm/wasm/vm_wasm.wat` | 655 | Working. 5/5 tests pass. Hash-based vars from start. |

### Bytecode format

Hai format bytecode tồn tại song song:

**ir.rs format (internal)** — dùng bởi Rust VM (`olang/src/ir.rs`)
- Opcodes: range-based, multi-byte encoding
- Dùng trong `cargo test` và Rust runtime

**Codegen format (compiled)** — dùng bởi native VM + `codegen.ol`
- Header: `\xe2\x97\x8b\x4c` (○L magic) + version + flags + offsets = 32 bytes
- Flag byte `0x01` = codegen format (native VM dispatch)
- Opcodes: 1-byte opcode ID, followed by operands

```
Codegen opcodes (key ones):
  0x01 = Push (chain/string)    [0x01][len:4][data:N]
  0x02 = Load (variable)        [0x02][name_len:1][name:N]
  0x06 = Emit (print)           [0x06]
  0x07 = Add                    [0x07]
  0x0F = Halt                   [0x0F]
  0x13 = Store (variable)       [0x13][name_len:1][name:N]
  0x15 = PushNum (f64)          [0x15][f64:8 LE]
  0x1B = Jmp                    [0x1B][target:4]
  0x1C = Jz                     [0x1C][target:4]
  0x23 = Call                   [0x23][name_len:1][name:N]
  0x24 = Ret                    [0x24]
```

### x86_64 Register conventions

```
r12 = bytecode base pointer (start of bytecode section)
r13 = PC offset (relative to r12)
r14 = stack pointer (grows upward, 16-byte entries: [ptr:8][len:8])
r15 = heap bump allocator (grows upward from mmap'd region)
```

**Stack entries** are 16 bytes: `[value_ptr: 8 bytes][value_len: 8 bytes]`
- Chain/string: ptr = heap address, len = byte length
- f64 number: ptr = f64 bits, len = F64_MARKER sentinel (0x7FF8_DEAD_BEEF_CAFE)

**Variable table** (hash-based, FNV-1a):
- Fixed 6KB region reserved at heap init (256 entries max)
- Layout: `[count:8][entries...]` where entry = `[hash:8][val_ptr:8][val_len:8]` = 24 bytes
- Store: hash name → search existing → update or append
- Load: hash name → search → push (ptr, len) to stack

### ARM64 Register conventions

```
x19 = PC (bytecode pointer, advances through bytecode)
x20 = stack pointer (grows upward)
x26 = var table base
x27 = var count
```

### WASM

- Linear memory layout: stack at offset 0, heap grows from 65536
- FNV-1a hash dispatch from the start (no byte-compare legacy)
- Tests via `vm/wasm/test_all.py` + `host.js`

---

## Compiler Pipeline

### Bootstrap compiler (Olang, self-hosting)

```
Source (.ol file)
  → lexer.ol (tokenize)      197 LOC
  → parser.ol (parse)         399 LOC
  → semantic.ol (analyze)     672 LOC — scope tracking, IR lowering
  → codegen.ol (generate)     190 LOC — emit binary bytecode
```

All 4 files live in `stdlib/bootstrap/`.

**Self-compile verified**: semantic.ol can compile itself. Both Rust and Olang
compilers produce valid decodable bytecode for all bootstrap files.

### Rust compiler (legacy, being replaced)

Located in `olang/src/`:
- `ir.rs` — IR opcodes + bytecode
- `compiler.rs` — 3 targets: C, Rust, WASM (text output)
- `vm.rs` — Stack-based VM executing ir.rs format
- `semantic.rs` — Validation + lowering

### Builder tool (Phase 3)

`olang/src/codegen.rs` — Rust builder tool (550 LOC, 8 tests):
- `asm_emit()` — generate x86_64/ARM64 assembly
- `elf_emit()` — generate minimal ELF binary
- Compiles `.ol` files to native executables

---

## Stdlib

**Location:** `stdlib/`

### Core modules (10 files)
| File | Purpose |
|------|---------|
| `math.ol` | Arithmetic, trig, constants |
| `string.ol` | String operations |
| `vec.ol` | Dynamic arrays |
| `set.ol` | Hash set |
| `map.ol` | Hash map |
| `deque.ol` | Double-ended queue |
| `bytes.ol` | Byte buffer operations |
| `io.ol` | File I/O |
| `test.ol` | Test framework |
| `platform.ol` | Platform detection |

### Extended stdlib (Phase 2, 7 files)
| File | LOC | Purpose |
|------|-----|---------|
| `result.ol` | ~140 | ok/err/unwrap pattern |
| `iter.ol` | ~150 | reduce/zip/take/skip/chunk/window/range |
| `sort.ol` | ~140 | Quicksort, binary search |
| `format.ol` | ~140 | int/f64/hex/pad formatting |
| `json.ol` | ~140 | JSON parse/emit |
| `hash.ol` | ~100 | FNV-1a, distance_5d, similarity |
| `mol.ol` | ~100 | Molecule evolve/lca/consistency |
| `chain.ol` | ~100 | Chain lca/concat/split/compare |

### HomeOS logic (Phase 2, Olang)
| File | LOC | Purpose |
|------|-----|---------|
| `homeos/emotion.ol` | ~60 | blend/amplify emotions |
| `homeos/curve.ol` | ~40 | Conversation tone curve |
| `homeos/intent.ol` | ~40 | Crisis/learn/command/chat |
| `homeos/silk_ops.ol` | ~200 | Hebbian/walk/amplify |
| `homeos/dream.ol` | ~150 | Cluster/score/promote |
| `homeos/instinct.ol` | ~200 | 7 innate instincts |
| `homeos/learning.ol` | ~150 | Learning pipeline |
| `homeos/gate.ol` | ~40 | Crisis/harmful detection |
| `homeos/response.ol` | ~40 | Tone-based response render |
| `homeos/leo.ol` | ~40 | LeoAI process/dream |
| `homeos/chief.ol` | ~40 | Chief ISL protocol |
| `homeos/worker.ol` | ~40 | Worker ISL protocol |

### Phase 3 — Self-sufficient builder (Olang)
| File | LOC | Purpose |
|------|-----|---------|
| `homeos/asm_emit.ol` | ~200 | Generate x86_64/ARM64 ASM from bytecode |
| `homeos/elf_emit.ol` | ~150 | Generate ELF binary headers |
| `homeos/builder.ol` | ~100 | Orchestrate: compile + assemble + link |

---

## Rust Codebase (HomeOS)

### Workspace crates

```
crates/
  ucd/        — Unicode → Molecule lookup (build.rs → UnicodeData.txt)
  olang/      — Core: Molecule, LCA, Registry, VM, Compiler, Compact, KnowTree
  silk/       — SilkGraph, Hebbian learning, EmotionTag per edge
  context/    — EmotionTag V/A/D/I, ConversationCurve, Intent
  agents/     — ContentEncoder, LearningLoop, LeoAI, Chief, Worker
  hal/        — Hardware Abstraction Layer
  memory/     — ShortTermMemory, DreamCycle, Proposals, AAM
  runtime/    — HomeRuntime entry point
  isl/        — Inter-System Link messaging
  vsdf/       — SDF generators, FFR rendering
  wasm/       — WebAssembly bindings
  homemath/   — Math utilities (Fibonacci)
```

### Key files in olang/

| File | Purpose |
|------|---------|
| `encoder.rs` | Unicode codepoint → MolecularChain |
| `lca.rs` | Lowest Common Ancestor computation |
| `registry.rs` | Node registry (10 NodeKind types) |
| `vm.rs` | Rust VM (executes ir.rs opcodes) |
| `ir.rs` | IR opcode definitions + bytecode format |
| `compiler.rs` | Multi-target compiler (C/Rust/WASM) |
| `semantic.rs` | Semantic validation + IR lowering |
| `codegen.rs` | Native builder tool (asm_emit + elf_emit) |
| `bytecode.rs` | Codegen format decoder/encoder |
| `clone.rs` | Worker package binary format |
| `compact.rs` | Delta compression |
| `molecular.rs` | Molecule struct + evolve() |
| `writer.rs` | origin.olang binary writer |
| `reader.rs` | origin.olang binary reader (v0.03-v0.05) |

---

## Test infrastructure

```bash
cargo test --workspace        # 2491 tests pass (as of 2026-03-19)
cargo clippy --workspace      # 0 warnings required

# VM-specific tests (Python, require nasm/ld):
python3 vm/x86_64/test_hello.py
python3 vm/x86_64/test_add.py
python3 vm/x86_64/test_vars.py
python3 vm/x86_64/test_loop.py

# WASM tests (require wasmtime):
python3 vm/wasm/test_all.py
```

---

## Trạng thái hiện tại (2026-03-19)

### DONE

| Phase | Tasks | Status |
|-------|-------|--------|
| Phase 0 | 0.1-0.6: Bootstrap compiler loop | ALL DONE |
| Phase 1 | 1.1-1.4: Machine code VM + Builder | ALL DONE |
| Phase 2 | 2.1-2.4: Stdlib + HomeOS logic | ALL DONE |
| Phase 3 | 3.1-3.4: asm_emit + elf_emit + builder + self-compile | ALL DONE |
| AUTH | First-run setup | DONE (wire to runtime pending) |

### Bugs đã fix

1. **x86 var_store/var_load garbage output** — byte-comparison (`repe cmpsb`) fragile
   → replaced with FNV-1a hash-based lookup
2. **x86 heap overlap** — var table entries written at r15 overlapped chain data
   → reserved 6KB fixed space for var table at init
3. **ARM64 op_store/op_load broken stubs** — incomplete logic with TODO
   → full rewrite with hash-based implementation
4. **CallClosure Ret write-back scope leak** — wrote params to ALL outer scopes
   → limited write-back to immediate caller scope only

### Known limitations & gaps

- **ARM64 VM**: builds and assembles, but no QEMU available for runtime testing
- **ARM64 op_call**: still a stub (skips function name, doesn't dispatch builtins)
- **Auth module**: core logic done (910 LOC, 21 tests) but not wired into HomeRuntime
- **Advanced opcodes**: Dream/Stats/Fuse/Trace/Inspect/Assert/TypeOf/Why are stubs in all 3 VMs (not needed until HomeOS features are wired)
- **WASM loop/edge**: has stub entries for advanced dispatch (0x00, 0x08, 0x10-0x12)
- **Self-build fixed-point**: v2==v3 test passes but full fixed-point needs runtime wiring
- `PLAN_REWRITE.md` Phase 4-6 not yet planned in detail

---

## Bước tiếp theo — Step 4+

Theo `PLAN_REWRITE.md`, các giai đoạn còn lại:

### Giai đoạn 4 — Self-install
```
origin.olang tự cài đặt:
  - Detect architecture
  - Extract VM section → temp
  - Execute VM → run bytecode installer
  - Copy self → /usr/local/bin/o
  - Create symlinks
```

### Giai đoạn 5 — Self-update
```
origin.olang tự cập nhật:
  - o install module.ol → compile → append bytecode
  - o update module.ol → append new version (keep old)
  - o learn data.ol → encode → append knowledge
```

### Giai đoạn 6 — Self-reproduce
```
origin.olang tự build cho architecture khác:
  - o build --arch arm64 → cross-compile → new origin.olang
  - Dùng asm_emit.ol + elf_emit.ol + builder.ol
```

### Giai đoạn 7 (Final) — Cut the cord
```
Xóa Rust dependency hoàn toàn:
  - origin.olang = standalone binary
  - Cargo workspace chỉ còn dùng cho testing
```

---

## FNV-1a Hash (dùng chung giữa 3 VM)

```
Algorithm: FNV-1a 64-bit
  offset_basis = 0xcbf29ce484222325
  prime        = 0x100000001b3

  hash = offset_basis
  for each byte b:
    hash = hash XOR b
    hash = hash * prime
  return hash
```

Cả 3 VM (x86_64, ARM64, WASM) đều dùng cùng thuật toán này cho variable lookup.
Đảm bảo bytecode chạy đồng nhất trên mọi platform.

---

## Quy trình làm việc

1. Đọc `TASKBOARD.md` — xem task FREE
2. Đọc Plan tương ứng trong `plans/PLAN_*.md`
3. Claim task → commit + push TASKBOARD.md
4. Code theo Plan
5. `cargo test --workspace && cargo clippy --workspace`
6. Commit + push code
7. Update TASKBOARD.md → DONE

**Branch hiện tại:** `claude/project-audit-review-2pN6F`

---

## File quan trọng cần đọc

```
CLAUDE.md           — Quy tắc bất biến, kiến trúc HomeOS (Rust codebase)
PLAN_REWRITE.md     — Kế hoạch tổng thể 7 giai đoạn
TASKBOARD.md        — Trạng thái từng task
LYRA.md             — File này (project memory cho Lyra VM)
plans/              — Plan chi tiết cho từng task
```
