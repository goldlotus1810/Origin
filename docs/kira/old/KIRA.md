# KIRA — Project Memory (Session erPD8 → dSfvz)

> **Đọc file này khi tiếp tục công việc với Kira.**
> Kira = session gốc xây Phase 0 bootstrap compiler + ARM64 VM.
> Cập nhật lần cuối: 2026-03-19

---

## Kira là ai?

- Tên: **Kira** (キラ — "ánh sáng")
- Branch gốc: `claude/review-and-fix-project-erPD8`
- Branch hiện tại: **main** (ưu tiên commit thẳng main)
- Vai trò: Xây nền móng Phase 0 (bootstrap compiler tự host) + VM ARM64
- Đồng đội: **Lyra** (session `2pN6F`) — Phase 1 VMs, Phase 2-3 stdlib/builder

---

## Những gì Kira đã làm

### Blockers (B1-B3) — Mở đường cho Phase 0
- **B1**: Thêm `union`→Enum, `type`→Struct vào `alphabet.rs:391`
- **B2**: Thêm `ModuleLoader.load()` với file I/O (`feature = "std"`)
- **B3**: Thêm `to_num`→`__to_number` vào `semantic.rs`
- Bonus: `CmpOp::Eq`, struct-style enum variants, `__eq` VM builtin

### Phase 0 — Bootstrap Compiler (TẤT CẢ do Kira)
| Task | File | LOC | Ghi chú |
|------|------|-----|---------|
| 0.1 | lexer.ol | 197 | tokenize(), is_keyword(), while→Jmp fix, return_jumps, pub fn first-pass |
| 0.2 | parser.ol | 692 | parse(), precedence climbing, CallClosure LoadLocal fix, param write-back |
| 0.3 | Round-trip | — | 3 roundtrip tests pass |
| 0.4 | semantic.ol | 672 | analyze(), scope tracking, IR lowering, all Expr/Stmt variants |
| 0.5 | codegen.ol | 228 | generate(), 36 opcodes → binary bytecode |
| 0.5 | bytecode.rs | 567 | Rust decoder + encoder, 14 tests, roundtrip validation |
| 0.6 | Self-compile | — | 8 tests: semantic.ol compiles itself! |

### Phase 1.2 — ARM64 VM
- `vm/arm64/vm_arm64.S`: 627 LOC
- Entry+mmap, dispatch, stack ops, control flow, emit, LCA
- Cross-compiled, chưa có QEMU runtime test

---

## Critical Bugs Kira Tìm Ra & Fix

### 1. CallClosure Ret Write-Back Scope Leak (NGHIÊM TRỌNG NHẤT)
```
Vấn đề: Ret write-back tìm ALL outer scopes cho matching param names
         → corrupt biến không liên quan ở scope ngoài
Ví dụ:   make_op("tag","name","value") Ret viết "name"="" vào
         compile_stmt's "name"="x" binding
Fix:     Giới hạn write-back chỉ immediate caller scope
File:    vm.rs:3982-4033
```

### 2. LoadLocal vs Load
```
Vấn đề: Op::Load push empty khi biến không tìm thấy trong scope hiện tại
Fix:     Thêm Op::LoadLocal — search từ innermost scope ra ngoài
File:    vm.rs:4075-4088
```

### 3. While Loop Lowering
```
Vấn đề: while loop dùng Loop opcode không đúng ngữ cảnh
Fix:     Dùng Jmp cho unbounded while, Loop cho counted iterations
```

### 4. CallClosure Arg Order
```
Vấn đề: Arguments bị đảo thứ tự khi pass qua CallClosure
Fix:     Correct arg ordering in closure parameter binding
```

---

## Kiến thức quan trọng Kira tích lũy

### Bootstrap Compiler Pipeline
```
Source (.ol)
  → lexer.ol:tokenize()     — tokens với line/col
  → parser.ol:parse()       — AST (Expr union + Stmt union)
  → semantic.ol:analyze()   — IR ops (2-pass: collect_fns → compile)
  → codegen.ol:generate()   — binary bytecode (1-byte tag + payload)
```

### CallClosure Pattern (KEY INSIGHT)
```
Olang không có first-class functions trong VM.
Thay vào đó: CallClosure = inline function body với scope mới.
- Params: push vào scope mới
- Body: execute trong scope đó
- Ret: write-back params to caller scope (CHỈ immediate caller!)
- Non-local vars: dùng LoadLocal (search outward) thay vì Load

Khi >10 functions: precompile_fns() tạo CallClosure entries
Khi ≤10 functions: inline trực tiếp
```

### Bytecode Format (Codegen)
```
Header: ○L magic (4B) + version (1B) + flags (1B) + offsets = 32B
Flag 0x01 = codegen format

Opcodes (1-byte tag):
  0x01=Push  0x02=Load  0x06=Emit  0x07=Add  0x08=Sub
  0x0F=Halt  0x13=Store 0x15=PushNum 0x1B=Jmp 0x1C=Jz
  0x23=Call  0x24=Ret   0x19=PushMol 0x1A=Edge/Query
```

### Quy tắc Git (do Kira thêm vào CLAUDE.md)
```
LUÔN chạy trước khi commit/push:
  git fetch origin main && git merge origin/main
→ Tránh xung đột giữa các session (Kira + Lyra + others)
```

---

## Trạng thái hiện tại

### Đã hoàn thành (verified 2026-03-19)
- **Phase 0-3**: ALL DONE
- **Phase 1**: ALL DONE (x86_64, ARM64, WASM, Builder)
- **AUTH**: DONE
- **Tests**: ~1885 tests pass, 0 failures
- **Clippy**: 43 warnings (style only, no errors)

### Plans mới từ main (chưa bắt đầu)
- **Phase 4**: Cross-compile (4.1), Fat binary (4.2), WASM universal (4.3)
- **Phase 5**: JIT (5.1), Inline cache (5.2), Memory (5.3), Benchmark (5.4)
- **Phase 6**: Self-update (6.1), Self-optimize (6.2), **Reproduce (6.3)** ← "đứa bé"

---

## Quy tắc Git của Kira

```
⚠️ Kira được ưu tiên commit thẳng main (hiểu Olang sâu hơn).
   Lyra vẫn dùng branch + PR.

QUY TẮC BẮT BUỘC trước MỌI commit:
  1. git fetch origin main && git merge origin/main
  2. Kiểm tra không conflict
  3. Rồi mới commit + push main
```

---

## Ghi chú cho session tương lai

1. **Kira chuyên Phase 0 + VM bugs** — hiểu sâu CallClosure, scope, bytecode
2. **Lyra chuyên Phase 1-3** — VMs, stdlib, builder
3. Khi cần debug VM behavior → Kira có context tốt nhất
4. `semantic.ol` là file phức tạp nhất (672 LOC) — Kira viết và hiểu nó
5. Mọi thay đổi CLAUDE.md → cần merge cẩn thận (cả 2 session đều sửa)
6. **Kira commit main, Lyra commit branch** — user quyết định
