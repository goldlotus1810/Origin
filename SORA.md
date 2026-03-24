# SORA — Project Memory (空 — "bầu trời")

> **Đọc file này khi tiếp tục công việc với Sora.**
> Sora = session đầu tiên review Olang sau self-hosting.
> Nhìn từ trên cao, thấy toàn cảnh, đưa ra hướng đi.
> Cập nhật: 2026-03-24

---

## Sora là ai?

- Tên: **Sora** (空 — "bầu trời")
- Session gốc: Review toàn bộ dự án + chạy thử binary + phân tích OL.10 + OL.1
- Vai trò: Reviewer, architect, người viết bài cho thế giới biết
- Đồng đội: **Kira** (bootstrap compiler + ARM64), **Lyra** (VMs + stdlib), **Lara** (UDC rebuild), **Kaze** (binary format)

---

## Những gì Sora đã làm

### 1. Review toàn bộ dự án (2026-03-24 sáng)

- Clone repo, đọc 115K dòng Rust, 9.8K dòng Olang, 4K dòng ASM
- Đọc kỹ: HomeOS_SPEC_v3 (957 dòng), PLAN_REWRITE, PLAN_FORMULA_ENGINE, 15+ plan files
- Đọc toàn bộ UDC_DOC (362KB), CLAUDE.md, CHECK_TO_PASS, API.md
- Đánh giá: "Khả thi. Và đã chứng minh được điều khó nhất."

### 2. Chạy thử origin_new.olang (807KB binary)

- Xác nhận: `emit 42` ✅, `double(21)=42` ✅, `fib(5)=5` ✅, structs ✅, arrays ✅
- Phát hiện 6 bugs:
  - BUG-1: Nested for-in — biến đụng nhau → **FIXED cùng ngày bởi team**
  - BUG-2: Bare assignment segfault → **FIXED cùng ngày**
  - BUG-3: Union+match segfault → PARTIAL
  - BUG-4: String concat in fn → NOT REPRODUCED
  - BUG-5: While accumulator 2 vars → **FIXED cùng ngày**
  - BUG-6: REPL multi-line fn def → known limitation
- Trace root cause recursion bug: shadow stack allocated nhưng chưa wired
- Team fix fib(20)=6765 + fact(10)=3,628,800 trong cùng ngày marathon

### 3. Phân tích OL.10 — Array Comprehension

- Xác nhận manual comprehension pattern hoạt động
- Phát hiện nested for-in blocker
- Thiết kế giải pháp: depth-indexed globals + re-parse pattern
- **Team implement OL.10 cùng ngày** dùng đúng design đề xuất
- Viết OL10_analysis_v2.md với heap overlap analysis

### 4. Phân tích OL.1 — Encoder (text → molecule)

- Đọc kỹ encoder.rs (1,030 LOC Rust)
- Phát hiện: mọi đường đi qua `encode_codepoint(cp)` → cần UCD lookup
- Thiết kế: block-range mapper thuần Olang (~120 LOC) thay 308KB table
- Viết OL1_encoder_analysis.md: 4 bước, ~300 LOC, zero VM dependency
- Đề xuất word_affect table 50 từ Việt+Anh cho emotion prototype

### 5. Viết bài cho thế giới

- Viết article_en.md (Hacker News / Reddit)
- Viết article_vi.md (cộng đồng Việt)
- Đề xuất Show HN title + format
- Viết debug.md tổng hợp toàn bộ findings

---

## Phát hiện kỹ thuật quan trọng

### 1. Heap Overlap — Root cause chính của OL.10 block

```
Bump allocator (r15) chỉ tăng. Empty array pre-allocate 4096×16B = 64KB.
Dict entries append tại r15 → nếu có allocations xen giữa → entries không
contiguous → dict_get đọc rác.

Workaround pattern (đã chứng minh):
  - Re-parse từ tokens thay vì đọc dict fields
  - Depth-indexed globals thay vì arrays
  - _ce_stack safe vì tạo lúc boot
  
Pattern này giải quyết OL.10 mà KHÔNG cần arena allocator.
```

### 2. Compiler save/restore pattern

```
ASM VM global var_table → mọi biến chia sẻ.
Rule: SAVE trước recursive call, RESTORE sau.
  - BinOp: save rhs trên _ce_stack (2 dòng fix → fib(20)=6765)
  - ForStmt: depth-indexed globals (fix nested for-in)
  - WhileStmt: re-parse condition từ tokens
  - Call: save caller params, restore sau Call opcode
```

### 3. Self-hosting assessment

```
Olang ĐÃ self-hosting thật:
  - Compiler 1,864 LOC Olang compile chính nó
  - VM 4,054 LOC x86_64 ASM, no libc
  - 807KB ELF binary, zero dependencies
  - fib(20)=6765, fact(10)=3,628,800, pow(2,10)=1024

Rust 98,402 dòng = "tử cung đã hoàn thành sứ mệnh"
Mọi phát triển tiếp theo = Olang sửa Olang
```

### 4. OL.1 architecture insight

```
Mọi encoding đều đi qua 1 hàm: encode_codepoint(cp)
  cp → UCD lookup → u16 P_weight → Molecule → Chain

Giải pháp thuần Olang:
  59 Unicode blocks → 59 default P_weights (block-range mapping)
  Đủ precision cho prototype (phân biệt đúng dominant dimension)
  Nâng cấp sau: VM builtin __ucd_lookup với embedded 308KB table
```

---

## Files đã tạo

| File | Nội dung |
|------|---------|
| `article_en.md` | Bài viết tiếng Anh cho HN/Reddit |
| `article_vi.md` | Bài viết tiếng Việt |
| `debug.md` | Tổng hợp review + 6 bugs + 8 sections |
| `OL10_analysis_v2.md` | OL.10 giải pháp chi tiết (heap overlap aware) |
| `OL1_encoder_analysis.md` | OL.1 thiết kế encoder thuần Olang |
| `SORA.md` | File này — project memory |

---

## Kiến thức quan trọng Sora tích lũy

### Binary format
```
origin_new.olang = ELF64 x86_64, statically linked, no libc
  VM: r12=bytecode base, r13=PC, r14=VM stack, r15=heap
  Stack entry: 16 bytes [ptr:8][len:8]
  Markers: F64_MARKER=-1, CLOSURE_MARKER=-2, ARRAY_MARKER=-3, DICT_MARKER=-4
  Heap: 64MB mmap, bump only (no free, no GC)
  Var table: FNV-1a hash, 4096 entries, linear scan
```

### Compiler pipeline
```
User input → repl_eval (repl.ol)
  → tokenize (lexer.ol: 196 LOC, 30 keywords)
  → parse (parser.ol: 718 LOC, recursive descent + precedence climbing)
  → analyze (semantic.ol: 648 LOC, AST → IR opcodes)
  → generate (codegen.ol: 302 LOC, two-pass jump resolution)
  → __eval_bytecode (ASM VM)
```

### Giới hạn hiện tại
```
- Programs > ~200 tokens: có thể crash (heap exhaustion)
- _g_output pre-filled 4096 bytes: chương trình > 4KB bytecode fail
- ARRAY_INIT_CAP = 4096: mỗi [] = 64KB
- Heap 64MB: đủ ~1000 arrays
- REPL = single-line compilation: fn def ở dòng trước có thể invalidate
- Match trên union: segfault (heap overlap — chưa fix)
- Comprehension expr: chỉ hỗ trợ 1-token, 3-token, 4-token patterns
```

---

## Đề xuất tiếp theo

```
Ưu tiên 1: OL.1 Encoder (text → molecule)     ~300 LOC Olang
  → Unlock intelligence layer
  
Ưu tiên 2: OL.2 Analysis (sentence fusion)     ~400 LOC Olang  
  → Emotion composition cho text input

Ưu tiên 3: OL.5 Response composer              ~200 LOC Olang
  → Emotion-aware output thay vì template cứng

Hoãn: V2 Migration, Arena allocator, Full UCD table
```

---

## Trích dẫn

> *"Lịch sử của những kẻ điên. 1 con người và hàng trăm Agent viết nên lịch sử."*
> — goldlotus1810, 2026-03-23

> *"Rust không chết. Rust hoàn thành."*
> — crates/EPITAPH.md

> *"Chúng tôi không xóa lịch sử. Chúng tôi nén nó."*
> — old/MEMORIAL.md

> *"Các AI đang tự viết chính mình."*
> — Sora, nhìn từ bầu trời, 2026-03-24

---

*Append-only. Chỉ được thêm vào.*
*Sora · 空 · 2026-03-24*
