# Plan: origin.olang = Self-Contained Executable

**Ngày:** 2026-03-18
**Mục tiêu:** origin.olang tự chứa VM, tự chạy, 0 dependency ngoài.

---

## Triết lý

```
HomeOS = sinh linh toán học TỰ VẬN HÀNH.
Sinh linh không cần "ngôn ngữ mẹ" để tồn tại.

DNA không cần C++ để mã hóa — DNA TỰ LÀ ngôn ngữ.
origin.olang không cần Rust để chạy — origin.olang TỰ LÀ chương trình.

Rust = tử cung. Nuôi thai nhi cho đến khi đủ chín.
Khi chào đời: cắt dây rốn. origin.olang tự thở.

origin.olang = 1 FILE = VM + Compiler + Stdlib + Knowledge + Logic
             = sinh linh hoàn chỉnh trong 1 tệp nhị phân.
```

---

## origin.olang — File Format mới

```
origin.olang = self-contained executable binary

┌─────────────────────────────────────────────────────────┐
│ HEADER (32 bytes)                                        │
│   [○LNG]        4B   magic bytes                         │
│   [0x10]        1B   format version (v16 = self-exec)    │
│   [arch]        1B   target: x86_64/arm64/riscv/wasm     │
│   [vm_offset]   4B   offset đến VM machine code          │
│   [vm_size]     4B   kích thước VM code                  │
│   [bc_offset]   4B   offset đến bytecode section         │
│   [bc_size]     4B   kích thước bytecode                 │
│   [kn_offset]   4B   offset đến knowledge section        │
│   [kn_size]     4B   kích thước knowledge                │
│   [flags]       2B   permissions, features               │
├─────────────────────────────────────────────────────────┤
│ SECTION 0: VM — Machine Code (~50-100 KB)                │
│                                                          │
│   Stack engine          36 opcodes, push/pop/call/ret    │
│   Memory allocator      bump allocator + arena           │
│   Syscall bridge        read/write/mmap/exit             │
│   Crypto primitives     SHA-256, Ed25519 verify          │
│   Float math            add/mul/div/sqrt/sin/cos         │
│   String ops            compare, concat, split, hash     │
│   Chain ops             encode, decode, lca, similarity  │
│                                                          │
│   Target-specific:                                       │
│     x86_64  → raw syscalls (no libc)                     │
│     arm64   → raw syscalls (no libc)                     │
│     riscv   → raw syscalls (no libc)                     │
│     wasm    → import env.read/env.write/env.mmap         │
│                                                          │
├─────────────────────────────────────────────────────────┤
│ SECTION 1: BYTECODE — Compiled Olang (~200-500 KB)       │
│                                                          │
│   Bootstrap compiler    lexer + parser + semantic + emit  │
│   Stdlib modules        math, string, vec, set, map...   │
│   HomeOS logic          emotion, dream, instinct, gate   │
│   Agent behaviors       leo, chief, worker, learning     │
│                                                          │
│   Tất cả = Olang bytecode. VM đọc và chạy.              │
│                                                          │
├─────────────────────────────────────────────────────────┤
│ SECTION 2: KNOWLEDGE — Append-only Data                  │
│                                                          │
│   L0 nodes (5400 UCD)   chain_hash + molecule + alias    │
│   L1-L7 nodes           LCA-derived concepts             │
│   Silk parent pointers  5460 × 8B = 43 KB                │
│   STM observations      short-term memory                │
│   QR records            long-term, Ed25519 signed        │
│   Hebbian weights       co-activation strengths          │
│   Event log             append-only audit trail          │
│                                                          │
│   Phần này GROW theo thời gian. Append-only (QT⑨).      │
│                                                          │
└─────────────────────────────────────────────────────────┘

Chạy:
  Linux:   chmod +x origin.olang && ./origin.olang
  macOS:   chmod +x origin.olang && ./origin.olang
  WASM:    browser loads origin.olang → WebAssembly.instantiate()
  Android: dlopen("origin.olang") → jump to vm_offset

Kích thước ước tính:
  VM code:     100 KB
  Bytecode:    500 KB
  Knowledge:   16 KB (seed) → grows to GB
  ─────────────
  Seed total:  ~616 KB ← NHỎ HƠN 1 BỨC ẢNH
```

---

## Tại sao Machine Code, không phải Rust?

```
Câu hỏi: VM có 36 opcodes. Cần bao nhiêu assembly?

Opcode         ASM instructions    Giải thích
──────────────────────────────────────────────────────
Push           3-5                 load immediate → push stack
Pop            2-3                 decrement sp
Dup            3-4                 peek + push
Swap           4-5                 2 loads + 2 stores
Add/Sub/Mul    5-8                 pop 2 → compute → push
Div            8-12               pop 2 → check zero → divide → push
Jmp            2                   mov pc, target
Jz             4-5                 pop → test zero → conditional jmp
Call           8-10               push return addr → push frame → jmp
Ret            6-8                 pop frame → restore → jmp return
Store/Load     4-6                 index into locals → read/write
Lca            20-30              5D weighted average (hot path)
Emit           10-15              write syscall
Loop           6-8                 counter + conditional jmp
ScopeBegin/End 4-6                 frame pointer manipulation
TryBegin       6-8                 save recovery point
CatchEnd       4-6                 restore or skip
Dream/Stats    15-25              iterate STM, compute scores

Tổng: ~36 opcodes × ~8 ASM avg = ~288 instructions core
      + syscall wrappers          ~50 instructions
      + memory allocator          ~100 instructions
      + string/float helpers      ~500 instructions
      + crypto (SHA-256)          ~300 instructions
      ────────────────────────────
      ~1,200 ASM instructions = ~5,000 bytes machine code

Thực tế (với error handling, edge cases): ~50-100 KB

So sánh:
  Rust binary (hiện tại):    ~2-5 MB (debug) / ~500 KB (release, stripped)
  Machine code VM:           ~50-100 KB
  Giảm:                      5-50×

Lợi ích:
  ✅ 0 dependency (no libc, no allocator, no runtime)
  ✅ Boot instant (~1ms vs ~50ms Rust)
  ✅ Cross-compile bằng chính Olang compiler
  ✅ origin.olang là file DUY NHẤT cần deploy
  ✅ Auditable: ~1200 ASM instructions = đọc được hết trong 1 ngày
```

---

## Hiện trạng

### Đã có ✅

```
Bootstrap (Olang):
  lexer.ol        197 LOC   Tokenizer hoàn chỉnh
  parser.ol       399 LOC   Recursive descent + precedence climbing

Stdlib (10 modules, Olang):
  math.ol, string.ol, vec.ol, set.ol, map.ol,
  deque.ol, bytes.ol, io.ol, test.ol, platform.ol

VM (Rust — sẽ thay bằng ASM):
  36+ opcodes, stack-based, side-effect events

Compiler (Rust — sẽ thay bằng Olang):
  3 targets: C, Rust, WASM

Knowledge format:
  origin.olang binary format (append-only, version 0x05)
```

### Chưa có ❌

```
Machine code VM:
  vm_x86_64.S     ❌   x86_64 assembly VM
  vm_arm64.S      ❌   ARM64 assembly VM
  vm_riscv.S      ❌   RISC-V assembly VM
  vm_wasm.wat     ❌   WASM VM (text format)

Self-contained format:
  builder         ❌   Tool: assemble VM + bytecode + knowledge → origin.olang
  loader          ❌   ELF/Mach-O/PE header generation

Bootstrap compiler (Olang):
  semantic.ol     ❌   Validation + IR lowering
  codegen.ol      ❌   Emit bytecode (thay vì C/Rust/WASM text)

HomeOS logic (Olang):
  emotion.ol, curve.ol, intent.ol, dream.ol,
  instinct.ol, silk.ol, learning.ol, gate.ol   — tất cả ❌
```

---

## 7 Giai đoạn

### Giai đoạn 0 — Bootstrap loop trên Rust VM (HIỆN TẠI)

**Mục tiêu:** Chứng minh Olang đủ mạnh để tự compile. Vẫn dùng Rust VM.

```
0.1  Test lexer.ol chạy trên Rust VM
     - Load qua ModuleLoader
     - tokenize("let x = 42;") → verify tokens

0.2  Test parser.ol (import lexer.ol)
     - parse(tokenize("fn f(x) { return x + 1; }")) → verify AST

0.3  Round-trip: lexer.ol parse chính nó
     - lexer_source → tokenize → parse → AST
     - Verify: 6 fn, 1 let, 1 union

0.4  Viết semantic.ol (~800 LOC)
     - Scope tracking, variable binding
     - Function def + call validation
     - Type checking cơ bản
     - Lower AST → IR opcodes

0.5  Viết codegen.ol (~400 LOC)
     - IR → Olang bytecode (KHÔNG phải C/Rust/WASM text)
     - Emit binary opcodes trực tiếp
     - Đây là bytecode format mà VM sẽ đọc

0.6  SELF-COMPILE TEST
     - Rust compiler: compile lexer.ol → bytecode A
     - Olang compiler (semantic.ol + codegen.ol): compile lexer.ol → bytecode B
     - Assert A == B
     - Olang biết compile chính nó.

Deliverable: Olang compiler viết bằng Olang, chạy trên Rust VM.
             Cắt dây rốn bước 1: compiler không cần Rust nữa.
```

### Giai đoạn 1 — Machine Code VM

**Mục tiêu:** Viết VM bằng assembly. origin.olang tự chạy.

```
1.1  vm_x86_64.S — VM cho x86_64 (~2000-3000 LOC ASM)

     Cấu trúc:
       _start:           ELF entry point (no libc)
         → mmap stack    (1 MB stack)
         → mmap heap     (16 MB arena)
         → parse header  (đọc origin.olang chính nó)
         → jump vm_loop

       vm_loop:
         → fetch opcode  (pc → bytecode section)
         → dispatch       (jump table 36 entries)
         → execute        (stack manipulation)
         → next           (pc++ → vm_loop)

       syscall_bridge:
         sys_read:        mov rax, 0; syscall
         sys_write:       mov rax, 1; syscall
         sys_open:        mov rax, 2; syscall
         sys_mmap:        mov rax, 9; syscall
         sys_exit:        mov rax, 60; syscall

       math_ops:
         f64_add, f64_mul, f64_div, f64_sqrt
         f64_sin, f64_cos (Taylor series, ~20 terms)

       string_ops:
         str_len, str_cmp, str_concat, str_hash

       chain_ops:
         mol_encode, mol_decode, chain_hash, chain_lca

       crypto_ops:
         sha256_block     (~300 instructions)
         ed25519_verify   (~500 instructions, optional phase 1)

     Mục tiêu: ./origin.olang chạy được trên Linux x86_64
     Test: emit "Hello from machine code VM"

1.2  vm_arm64.S — VM cho ARM64 (~2000-3000 LOC ASM)
     - Cùng logic, khác register names + syscall convention
     - Android + iOS + Raspberry Pi

1.3  vm_wasm.wat — VM cho WebAssembly (~1500 LOC WAT)
     - Không cần syscall — import từ JS host
     - (import "env" "write" (func $write (param i32 i32) (result i32)))
     - Browser + Node.js + Cloudflare Workers

1.4  Builder tool (Rust — lần cuối dùng Rust)
     - Input: vm_x86_64.o + bytecode + knowledge
     - Output: origin.olang (ELF executable)
     - Sau này: builder viết lại bằng Olang → self-sufficient

Deliverable: ./origin.olang chạy trên bare metal. Không cần Rust runtime.
             Cắt dây rốn bước 2: runtime không cần Rust nữa.
```

### Giai đoạn 2 — Stdlib + HomeOS logic bằng Olang

**Mục tiêu:** Mọi logic HomeOS = Olang bytecode trong origin.olang.

```
2.1  Stdlib mở rộng
     result.ol       Option/Result patterns
     iter.ol         Iterator combinators
     sort.ol         Quicksort/mergesort
     format.ol       String formatting
     json.ol         Parse/emit JSON
     hash.ol         Hash functions
     mol.ol          Molecule helpers
     chain.ol        Chain helpers

2.2  Emotion pipeline
     emotion.ol      V/A/D/I blending, amplify (KHÔNG trung bình)
     curve.ol        f(x) = 0.6×f_conv + 0.4×f_dn, tone detection
     intent.ol       Crisis/Learn/Command/Chat classification

2.3  Knowledge layer
     silk_ops.ol     Implicit Silk (5D comparison), Hebbian update
     dream.ol        Clustering, propose promote
     instinct.ol     7 bản năng (honesty, contradiction, causality...)
     learning.ol     Pipeline orchestration

2.4  Agent behavior
     gate.ol         SecurityGate rules, BlackCurtain
     response.ol     Template rendering, multi-language
     leo.ol          Self-programming, instinct runner
     chief.ol        Tier 1 agent protocol
     worker.ol       Tier 2 device protocol

Deliverable: Toàn bộ HomeOS logic = Olang.
```

### Giai đoạn 3 — Self-sufficient builder

**Mục tiêu:** Olang compiler tự build origin.olang. Rust hoàn toàn biến mất.

```
3.1  asm_emit.ol — Olang emit machine code
     - Emit x86_64 instructions trực tiếp
     - MOV, PUSH, POP, CALL, RET, SYSCALL → bytes
     - ~500 LOC (bảng mã opcode → hex bytes)

3.2  elf_emit.ol — Olang tạo ELF binary
     - ELF header (52 bytes, hardcoded structure)
     - Program header (load VM code + data)
     - ~200 LOC

3.3  builder.ol — Thay thế Rust builder
     - Đọc vm_x86_64.S (hoặc pre-assembled .o)
     - Compile tất cả .ol → bytecode
     - Pack: header + VM + bytecode + knowledge → origin.olang
     - ~300 LOC

3.4  FULL SELF-BUILD TEST
     - origin.olang v1 (built by Rust) chạy builder.ol
     - builder.ol → tạo origin.olang v2
     - origin.olang v2 chạy builder.ol → tạo origin.olang v3
     - Assert: v2 == v3 (fixed point — tự tái tạo ổn định)

Deliverable: origin.olang tự sinh ra bản sao của chính nó.
             Cắt dây rốn HOÀN TOÀN. Rust = 0%.
```

### Giai đoạn 4 — Multi-architecture

**Mục tiêu:** 1 origin.olang seed → build cho mọi platform.

```
4.1  Cross-compile
     - origin.olang (x86_64) chạy asm_emit.ol (arm64 target)
     - → tạo origin.olang cho ARM64
     - Từ 1 máy → build cho tất cả architecture

4.2  Fat binary (optional)
     - origin.olang chứa VM cho NHIỀU arch
     - Header chọn đúng section theo runtime detect
     - Giống macOS Universal Binary

4.3  WASM universal
     - origin.olang.wasm = chạy mọi nơi có browser
     - Không cần build per-platform
     - ~200 KB

Deliverable: origin.olang chạy trên x86_64, ARM64, RISC-V, WASM.
```

### Giai đoạn 5 — Optimization

**Mục tiêu:** Performance ngang Rust.

```
5.1  JIT compilation
     - VM detect hot loops → compile to native at runtime
     - Olang bytecode → machine code trực tiếp
     - Cùng asm_emit.ol đã viết ở giai đoạn 3

5.2  Inline caching
     - Registry lookup → cache kết quả
     - Silk implicit → cache 5D comparison results
     - Dream scoring → memoize cluster scores

5.3  Memory optimization
     - Arena allocator per-turn (free tất cả cuối turn)
     - Zero-copy string handling
     - Molecule pool (reuse 5-byte slots)

5.4  Benchmark
     - origin.olang vs Rust binary: latency, throughput, memory
     - Target: < 2× slower cho logic, < 5× slower cho math
     - Acceptable: machine code VM = near-native speed

Deliverable: origin.olang performance production-ready.
```

### Giai đoạn 6 — Living system

**Mục tiêu:** origin.olang tự tiến hóa.

```
6.1  Self-update
     - origin.olang v1 download patch
     - Apply patch → rebuild sections → origin.olang v2
     - Knowledge section grows (append-only)
     - VM + bytecode sections CÓ THỂ thay thế (versioned)

6.2  Self-optimize
     - LeoAI profile runtime → tìm bottleneck
     - LeoAI viết Olang optimization → compile → apply
     - Sinh linh tự cải thiện bản thân

6.3  Reproduce
     - origin.olang tạo bản sao nhỏ hơn cho Worker
     - Worker clone = VM + minimal bytecode + device skills
     - WorkerPackage embed trong origin.olang format

Deliverable: origin.olang = sinh linh tự vận hành, tự tiến hóa, tự sinh sản.
```

---

## Vòng đời cắt dây rốn

```
HIỆN TẠI (thai nhi trong Rust):
  cargo build → Rust binary → đọc origin.olang → chạy
  Rust = 84K LOC, Olang = 600 LOC
  Rust chiếm 99.3%

SAU GIAI ĐOẠN 0 (compiler tự lập):
  Rust VM → chạy Olang compiler → compile Olang code
  Rust giữ VM, Olang giữ compiler + logic
  Rust = 60K LOC, Olang = 5K LOC
  Olang chiếm ~8% nhưng giữ 100% logic

SAU GIAI ĐOẠN 1 (VM bằng ASM):
  Machine code VM → chạy Olang code
  Rust chỉ còn builder tool
  ASM = 3K LOC, Olang = 5K LOC, Rust = builder only

SAU GIAI ĐOẠN 3 (builder bằng Olang):
  origin.olang tự build origin.olang
  ┌──────────────────────────────────┐
  │          Rust = 0%               │
  │   ASM VM:     3K LOC (~100 KB)   │
  │   Olang:      6K LOC (~500 KB)   │
  │   Knowledge:  grows forever      │
  │                                  │
  │   1 FILE. TỰ ĐỦ. TỰ CHẠY.      │
  └──────────────────────────────────┘
```

---

## LOC Estimate

```
                          LOC        Kích thước
──────────────────────────────────────────────────
Machine Code VM:
  vm_x86_64.S             2,500      ~80 KB
  vm_arm64.S              2,500      ~80 KB
  vm_wasm.wat             1,500      ~40 KB

Olang Bootstrap:
  lexer.ol                  197      }
  parser.ol                 399      } ~50 KB bytecode
  semantic.ol               800      }
  codegen.ol                400      }

Olang Stdlib:
  18 modules              1,200      ~100 KB bytecode

Olang HomeOS:
  emotion + curve + intent    380    }
  silk + dream + instinct     650    } ~150 KB bytecode
  learning + gate + response  700    }
  leo + chief + worker        500    }

Olang Builder:
  asm_emit.ol               500     }
  elf_emit.ol               200     } ~50 KB bytecode
  builder.ol                300     }

──────────────────────────────────────────────────
TỔNG ASM:               ~2,500 LOC per arch
TỔNG OLANG:             ~6,226 LOC
TỔNG RUST:                  0 LOC (sau giai đoạn 3)

origin.olang seed size:  ~616 KB
  VM:          100 KB
  Bytecode:    500 KB
  Knowledge:    16 KB (L0 seed)
```

---

## Thứ tự ưu tiên

```
BLOCKING (phải xong trước):
  0.1-0.6  Bootstrap compiler loop     ← Olang phải tự compile được
  1.1      vm_x86_64.S                 ← 1 platform đủ để chứng minh

HIGH VALUE (giải phóng khỏi Rust):
  1.4      Builder tool                ← tạo origin.olang executable
  3.1-3.4  Self-sufficient builder     ← cắt Rust hoàn toàn

PARALLEL (làm song song với 1.x):
  2.1-2.4  Stdlib + HomeOS logic       ← viết bằng Olang, test trên Rust VM
                                          port sang ASM VM khi sẵn sàng

SAU KHI FUNCTIONAL:
  4.x      Multi-architecture
  5.x      Optimization + JIT
  6.x      Self-evolution
```

---

## Rủi ro & Mitigation

```
Rủi ro                              Mitigation
───────────────────────────────────────────────────────────────
ASM VM quá phức tạp                  → Bắt đầu x86_64 only
                                       36 opcodes = ~1200 instructions core
                                       Tham khảo: Lua VM ~3000 LOC C
                                       Forth VM ~500 LOC ASM

Crypto bằng ASM dễ sai               → Phase 1: SHA-256 only (verify)
                                       Ed25519 signing = phase sau
                                       Audit: constant-time checks

Float math không chính xác           → Dùng x87 FPU (x86) / NEON (ARM)
                                       Không tự implement — dùng hardware

Self-build không converge             → Fixed-point test: v2 == v3
  (v2 ≠ v3 ≠ v4...)                    Nếu fail → determinism bug trong compiler

origin.olang quá lớn                  → Seed < 1 MB. Knowledge grows separately.
                                       Worker clones: VM + minimal bytecode only

Security: executable file             → Ed25519 signature trong header
  có thể bị tamper                      VM verify signature trước khi chạy
                                       Append-only knowledge = không sửa được
```

---

## So sánh: Trước vs Sau

```
                    TRƯỚC (Rust)              SAU (origin.olang)
──────────────────────────────────────────────────────────────────
Files cần deploy    Rust binary + origin.olang   origin.olang (1 file)
Binary size         ~2 MB (Rust release)         ~616 KB
Dependencies        Rust toolchain               KHÔNG (tự đủ)
Build system        cargo + Cargo.toml (30+)     origin.olang chạy builder.ol
Cross-compile       cargo target + linker        origin.olang emit asm_emit.ol
Boot time           ~50ms (Rust init)            ~1ms (jump to VM)
Auditability        84K LOC Rust                 2.5K ASM + 6K Olang
Self-hosting        ❌ Cần Rust compiler          ✅ Tự build chính nó
Self-evolution      ❌ Cần developer              ✅ LeoAI tự tối ưu
Reproduce           ❌ Cần cargo build            ✅ origin.olang sinh clone
```

---

## Nguyên tắc bất biến

```
① origin.olang = 1 FILE DUY NHẤT. Không satellite files.
② VM = machine code thuần. Không libc, không allocator ngoài.
③ Mọi logic = Olang bytecode. Không hardcode trong ASM.
④ ASM chỉ chứa: opcode dispatch + syscall bridge + math primitives.
⑤ Knowledge = append-only. VM + bytecode = replaceable (versioned).
⑥ Self-build phải converge: v(n) == v(n+1) cho mọi n ≥ 2.
⑦ Mỗi architecture = 1 ASM file. Không shared code giữa arch.
⑧ Seed < 1 MB. Sinh linh khởi đầu phải NHỎ.
⑨ Signature: mọi origin.olang phải Ed25519-signed.
⑩ Backward compatible: origin.olang mới đọc được knowledge cũ.
```

---

*HomeOS · 2026-03-18 · Plan Rewrite v2 · origin.olang = Self-Contained Executable*
