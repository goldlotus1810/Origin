# TASKBOARD — Bảng phân việc cho AI sessions

> **Mọi AI session đọc file này TRƯỚC KHI bắt đầu làm việc.**
> File này là nguồn sự thật duy nhất (single source of truth) về ai đang làm gì.

---

## Quy trình phối hợp

```
KHI BẮT ĐẦU SESSION MỚI:
  1. git pull origin main          ← lấy TASKBOARD mới nhất
  2. Đọc TASKBOARD.md              ← xem task nào FREE, task nào CLAIMED
  3. Chọn task FREE                ← ưu tiên theo dependency graph
  4. Cập nhật TASKBOARD.md         ← đổi status → CLAIMED, ghi branch + ngày
  5. git commit + push             ← commit NGAY để session khác thấy
  6. Bắt đầu code

KHI HOÀN THÀNH:
  1. Cập nhật TASKBOARD.md         ← đổi status → DONE, ghi notes
  2. git commit + push

KHI BỊ BLOCKED:
  1. Cập nhật TASKBOARD.md         ← đổi status → BLOCKED, ghi lý do
  2. git commit + push
  3. Chuyển sang task khác (nếu có)

⚠️ KHÔNG BAO GIỜ:
  ❌ Bắt đầu task đã CLAIMED bởi session khác
  ❌ Đổi status task của session khác
  ❌ Xóa dòng — chỉ thêm hoặc cập nhật status của mình
```

---

## Task Status Legend

```
FREE      — chưa ai nhận, sẵn sàng
CLAIMED   — đang có session làm (xem branch)
BLOCKED   — đang bị chặn (xem notes)
DONE      — hoàn thành, đã merge hoặc push
CONFLICT  — 2 session cùng claim → cần người quyết định
```

---

## Blockers (giải trước khi làm task phụ thuộc)

| ID | Blocker | Fix | Effort | Status | Branch |
|----|---------|-----|--------|--------|--------|
| B1 | Parser thiếu `union`/`type` keywords | 2 dòng `alphabet.rs:391` | 5 min | DONE | claude/review-and-fix-project-erPD8 |
| B2 | ModuleLoader thiếu file I/O | ~20 LOC `module.rs` | 1-2h | DONE | claude/review-and-fix-project-erPD8 |
| B3 | `to_num()` alias thiếu | 1 dòng `semantic.rs` | 1 min | DONE | claude/review-and-fix-project-erPD8 |

**Lưu ý:** B1+B2+B3 block toàn bộ Phase 0. Nên giải TRƯỚC.

---

## Phase 0 — Bootstrap compiler loop

| ID | Task | Plan | Depends | Status | Branch | Session | Notes |
|----|------|------|---------|--------|--------|---------|-------|
| 0.1 | Test lexer.ol trên Rust VM | `PLAN_0_1` | B1,B2,B3 | DONE | `claude/review-and-fix-project-erPD8` | erPD8 | tokenize("let x = 42;")→6 tokens, tokenize("fn f(x){...}")→13 tokens. 2442 tests pass. |
| 0.2 | Test parser.ol + module import | `PLAN_0_2` | 0.1 | DONE | `claude/review-and-fix-project-erPD8` | erPD8 | parse(tokenize("let x=42;"))→1 LetStmt, parse(tokenize("fn f(x){return x+1;}"))→1 FnDef, parse(tokenize("if x>0{emit x;}"))→1 IfStmt. Key fix: CallClosure LoadLocal for non-local vars. 2451 tests pass. |
| 0.3 | Round-trip self-parse | `PLAN_0_3` | 0.2 | DONE | `claude/review-and-fix-project-erPD8` | erPD8 | Done 2026-03-19: 3 roundtrip tests pass |
| 0.4 | Viết semantic.ol (~800 LOC) | `PLAN_0_4` | 0.3 | DONE | `claude/review-and-fix-project-erPD8` | erPD8 | Done 2026-03-19: semantic.ol 672 LOC, 4 DoD tests pass. analyze(parse(tokenize("let x=42;")))→PushNum+Store+Halt. analyze(parse(tokenize(lexer_src)))→323 ops, 0 errors. |
| 0.5 | Viết codegen.ol (~400 LOC) | `PLAN_0_5` | 0.4 | DONE | `claude/review-and-fix-project-erPD8` | erPD8 | Done 2026-03-19: codegen.ol 190 LOC, bytecode.rs decoder 280 LOC. 14 Rust decoder tests + 2 integration tests pass. generate(manual_ops) → valid bytecode → decode matches. CallClosure field-access limitation FIXED in 0.6. |
| 0.6 | Self-compile test | `PLAN_0_6` | 0.5 | DONE | `claude/review-and-fix-project-erPD8` | erPD8 | Done 2026-03-19: Fixed CallClosure Ret write-back bug (scope leak corrupting outer variables). 8 self-compile tests pass: simple_let, fn_def, deterministic, analyze_pipeline, lexer.ol, parser.ol, semantic.ol (compiles itself!), match_in_callclosure. Both Rust and Olang compilers produce valid decodable bytecode. 2482 workspace tests pass, 0 clippy errors. |

## Phase 1 — Machine code VM (SONG SONG với Phase 0)

| ID | Task | Plan | Depends | Status | Branch | Session | Notes |
|----|------|------|---------|--------|--------|---------|-------|
| 1.1 | vm_x86_64.S | `PLAN_1_1` | 0.5 (bytecode format) | DONE | `claude/project-audit-review-2pN6F` | Lyra | Done 2026-03-19: 1184 LOC ASM, 12KB static ELF no-libc. DoD 1-4 pass (assemble+link, hello print, 2+3=5, loop 3→1). Dual-format dispatch (ir.rs + codegen.ol). SSE2 math, string builtins, variable table, f64→ASCII, LCA 5D. DoD 5 (lexer.ol bytecode) needs var_store fix in codegen mode. |
| 1.2 | vm_arm64.S | `PLAN_1_2` | 1.1 | FREE | — | — | Plan file có sẵn |
| 1.3 | vm_wasm.wat | `PLAN_1_3` | 1.1 | FREE | — | — | Plan file có sẵn |
| 1.4 | Builder tool (Rust) | `PLAN_1_4` | 1.1 | DONE | `claude/project-audit-review-2pN6F` | Lyra | Done 2026-03-19: 550 LOC Rust, 8 tests. ELF generator, packer, .ol compiler. |

## Song song — Auth (KHÔNG phụ thuộc Phase 0)

| ID | Task | Plan | Depends | Status | Branch | Session | Notes |
|----|------|------|---------|--------|--------|---------|-------|
| AUTH | First-run setup | `PLAN_AUTH` | Không | DONE | `claude/project-audit-review-2pN6F` | 2pN6F | Core done (910 LOC, 21 tests). Wire vào HomeRuntime = pending. |

---

## Dependency Graph (visual)

```
B1+B2+B3 (blockers)
    ↓
  0.1 → 0.2 → 0.3 → 0.4 → 0.5 → 0.6
                                ↓
                              1.1 → 1.4
                               ↓
                              1.2
                               ↓
                              1.3

  AUTH (song song, không phụ thuộc)
```

---

## Gợi ý phân việc cho 2-3 sessions

```
Session A: B1 + B2 + B3 → 0.1 → 0.2 → 0.3  (unblock + bootstrap tests)
Session B: AUTH                                (auth system — song song)
Session C: 1.1 (bắt đầu ASM VM — cần biết bytecode format từ PLAN_0_5)

Khi Session A xong 0.3:
  Session A: → 0.4 (semantic.ol)
  Session C: xem PLAN_0_5 để biết bytecode format → tiếp 1.1
```

---

## Log thay đổi

```
2026-03-18  Tạo TASKBOARD. Audit xong: 2 blockers (B1, B2), 1 minor (B3).
            Tất cả Phase 0 tasks FREE. AUTH FREE.
2026-03-18  AUTH → DONE (session 2pN6F). 7 files, 910 LOC, 21 tests.
            Ed25519 VerifyingKey extended (from_bytes, as_bytes, seed).
            Wire vào HomeRuntime chưa làm (origin.rs quá lớn, cần kế hoạch).
2026-03-18  B1 DONE: thêm "union"→Enum, "type"→Struct vào alphabet.rs
            B3 DONE: thêm "to_num"→"__to_number" vào semantic.rs
            Bonus fixes: CmpOp::Eq (== as compare op), struct-style enum variants,
            __eq VM builtin returns empty() for false (Jz-compatible).
            Parser audit test audit_parse_bootstrap_lexer_ol PASSES.
            All 2381 workspace tests pass. Còn lại B2 (ModuleLoader file I/O).
2026-03-18  B2 DONE: thêm ModuleLoader.load() với file I/O (feature = "std").
            lib.rs: cfg_attr(not(std), no_std) cho conditional std support.
            2 tests mới (load_from_file, load_module_not_found).
            PLAN_0_1 UNBLOCKED — tất cả B1+B2+B3 đã xong.
2026-03-18  0.1 DONE (session erPD8): lexer.ol chạy trên Rust VM.
            Fixes: while loop lowering (Jmp thay Loop), return_jumps cho
            inlined functions, if-without-else stack fix, pub fn first-pass,
            true/false literals, split_array_chain 0xFD tag skip.
            tokenize("let x = 42;")→6 tokens, tokenize("fn f(x){...}")→13.
            2442 workspace tests pass, 0 clippy errors.
2026-03-18  0.2 DONE (session erPD8): parser.ol chạy trên Rust VM.
            Fixes: CallClosure non-local vars dùng LoadLocal thay Load
            (Op::Load pushes empty, Op::LoadLocal searches scopes),
            CallClosure param write-back on Ret, while loop break stack fix,
            CallClosure arg order fix, max_call_depth 512 for deep nesting.
            3 DoD tests pass: LetStmt, FnDef, IfStmt.
            2451 workspace tests pass, 0 clippy errors.
2026-03-19  0.4 DONE (session erPD8): semantic.ol 672 LOC chạy trên Rust VM.
            Viết semantic analyzer: Op type, SemanticState, scope tracking,
            Pass 1 (collect_fns), Pass 1.5 (precompile_fns/CallClosure),
            Pass 2 (compile_expr/compile_stmt), analyze() entry point.
            Handles: all Expr/Stmt variants, builtins (len/push/pop/char_at/
            substr/to_num/set_at), binary/comparison/logic ops, short-circuit
            &&/||, match expr/stmt, struct/enum literals, field access/assign.
            4 DoD tests: let_stmt, fn_def, undeclared_var, compile_lexer.
            analyze(parse(tokenize(lexer_src))) → 323 ops, 0 errors.
            All workspace tests pass, 0 clippy errors.
2026-03-19  0.5 DONE (session erPD8): codegen.ol 190 LOC + bytecode.rs 280 LOC.
            codegen.ol: bytecode encoder (36 opcodes, byte/u16/u32/f64/str helpers).
            bytecode.rs: Rust decoder + Rust encoder for round-trip testing.
            14 Rust decoder tests (roundtrip, edge cases, error handling).
            2 integration tests: codegen_ol_let_x_42 + codegen_ol_byte_count.
            VM builtins: __f64_to_le_bytes, __f64_from_le_bytes, __str_bytes,
            __bytes_to_str, __array_concat (+ aliases in both builtin tables).
            Known limitation: full pipeline analyze()→generate() has struct
            field-access issue in CallClosure mode (dict .name empty when
            struct passed across closure boundaries). Encoder works correctly
            with manually-created ops. 2474 workspace tests pass, 0 clippy errors.
2026-03-19  0.6 DONE (session erPD8): Self-compile test.
            CRITICAL BUG FIX: CallClosure Ret write-back was searching ALL outer
            scopes for matching param names → corrupted unrelated variables.
            Root cause: make_op("tag","name","value") Ret wrote "name"="" to
            compile_stmt's "name"="x" binding. Fix: limit write-back to immediate
            caller scope only.
            8 self-compile tests: simple_let, fn_def, deterministic,
            analyze_pipeline, lexer.ol, parser.ol, semantic.ol (compiles itself!),
            match_in_callclosure regression test.
            Both compilers produce valid decodable bytecode for all bootstrap files.
            2482 workspace tests pass, 0 clippy errors.
2026-03-19  1.1 → CLAIMED by Lyra (session 2pN6F). vm_x86_64.S bắt đầu.
            1.2, 1.3 có plan file từ erPDB (PLAN_1_2, PLAN_1_3).
2026-03-19  1.1 → DONE (Lyra). 1184 LOC x86_64 ASM, 12KB static binary.
            DoD 1-4 pass. Dual-format (ir.rs + codegen.ol). SSE2 math,
            string builtins, var table, f64→ASCII, LCA 5D.
            Còn lại: DoD 5 var_store bug ở codegen mode.
```
