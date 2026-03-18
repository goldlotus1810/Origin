# Plans — Chi tiết triển khai

**Kim chỉ nam:** `PLAN_REWRITE.md` (root)
**Yêu cầu:** Biết Rust. KHÔNG cần biết Olang.

---

## Dependency graph

```
PLAN_0_1 (lexer.ol test)
    ↓
PLAN_0_2 (parser.ol test)       ← cần module import hoạt động
    ↓
PLAN_0_3 (round-trip self-parse)
    ↓
PLAN_0_4 (semantic.ol)          ← viết ~800 LOC Olang
    ↓
PLAN_0_5 (codegen.ol)           ← viết ~400 LOC Olang + Rust decoder
    ↓
PLAN_0_6 (self-compile test)    ← MILESTONE: cắt dây rốn bước 1
    ↓
PLAN_1_1 (vm_x86_64.S)         ← x86_64 assembly, ~2000 LOC
    ↓
PLAN_1_4 (builder)              ← Rust tool: pack → ELF executable
    ↓
    ✓ origin.olang tự chạy      ← MILESTONE: cắt dây rốn bước 2

Song song:
PLAN_AUTH (first-run setup)     ← không phụ thuộc, làm bất kỳ lúc nào
```

## Phân việc

| Plan | Skill cần | Ước tính | Song song? |
|------|-----------|----------|------------|
| **0_1** | Rust, test | 2-4h → 2-3 ngày* | Không |
| **0_2** | Rust, module system | 4h → 2-3 ngày* | Không |
| **0_3** | Rust, test | 2-4h | Không |
| **0_4** | Rust + hiểu compiler | 3-5 ngày | Không |
| **0_5** | Rust, binary encoding | 2-3 ngày | Không |
| **0_6** | Rust, test | 3-5 ngày | Không |
| **1_1** | x86_64 ASM, Linux syscalls | 2-3 tuần | Song song với 0_4+ |
| **1_4** | Rust, ELF format | 1 tuần | Song song với 1_1 |
| **AUTH** | Rust, crypto | 1 tuần | Song song với tất cả |

*\* Tùy thuộc rào cản: FFI bridge, VM array support, module import*

## Rào cản chung (ảnh hưởng nhiều plan)

### 1. FFI Bridge cho built-in functions
**Ảnh hưởng:** 0_1, 0_2, 0_3, 0_4, 0_5
**Vấn đề:** lexer.ol dùng `len()`, `char_at()`, `substr()`, `push()`, `to_num()` — VM chưa có
**File:** `crates/olang/src/exec/vm.rs` hoặc tạo `builtins.rs`
**Ước tính:** 1-2 ngày

### 2. VM hỗ trợ arrays + strings natively
**Ảnh hưởng:** 0_1, 0_2, 0_3, 0_4
**Vấn đề:** `let tokens = [];` cần dynamic array; VM hiện chỉ có MolecularChain
**File:** `crates/olang/src/exec/vm.rs` — cần VmValue enum
**Ước tính:** 2-3 ngày

### 3. Module import (`use` statement)
**Ảnh hưởng:** 0_2+
**Vấn đề:** `use olang.bootstrap.lexer;` cần ModuleLoader resolve + inject
**File:** `crates/olang/src/exec/module.rs`
**Ước tính:** 1-2 ngày

---

## Quick start cho developer mới

```bash
# Clone + build
git clone <repo> && cd Origin
cargo build --workspace

# Chạy tests (2,093 tests, tất cả pass)
cargo test --workspace

# Đọc bối cảnh (theo thứ tự)
1. CLAUDE.md                    ← hiến pháp, quy tắc bất biến
2. PLAN_REWRITE.md              ← kim chỉ nam tổng thể
3. plans/PLAN_0_1_*.md          ← plan đầu tiên cần làm

# File code quan trọng
crates/olang/src/exec/vm.rs       ← VM (5,716 LOC)
crates/olang/src/exec/ir.rs       ← Op enum (36+ opcodes)
crates/olang/src/lang/semantic.rs  ← Compiler (288K)
crates/olang/src/exec/module.rs    ← Module loader
stdlib/bootstrap/lexer.ol          ← File Olang đầu tiên cần chạy
stdlib/bootstrap/parser.ol         ← File Olang thứ hai
```
