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
│   L0 nodes (8,846 UCD ⚠️v2)  chain_hash + u16 mol + alias │
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

Chạy (sau khi cài):
  o                    ← symlink → /usr/local/bin/o → origin.olang
  origin               ← symlink dài hơn (alias)

  Cả hai → REPL. Không argument = interactive mode.

Cài đặt lần đầu (First Run — chỉ xảy ra 1 lần):

  $ ./origin.olang

  ┌──────────────────────────────────────────────────────┐
  │              ○ HomeOS — First Run Setup               │
  ├──────────────────────────────────────────────────────┤
  │                                                      │
  │  QUY TẮC SỬ DỤNG                                    │
  │  ────────────────                                    │
  │  1. origin.olang là tài sản cá nhân của bạn.        │
  │     File này chứa MỌI THỨ: VM, logic, tri thức,     │
  │     khóa xác thực. MẤT FILE = MẤT HẾT.             │
  │                                                      │
  │  2. HomeOS học từ bạn. Dữ liệu KHÔNG rời khỏi      │
  │     thiết bị. Không cloud. Không telemetry.           │
  │     Bạn sở hữu 100% dữ liệu của mình.              │
  │                                                      │
  │  3. Append-only: HomeOS không xóa, không ghi đè.     │
  │     Mọi thay đổi được ghi lại vĩnh viễn.            │
  │                                                      │
  │  4. HomeOS KHÔNG chịu trách nhiệm cho:              │
  │     - Quyết định dựa trên đề xuất của HomeOS        │
  │     - Mất file do lỗi phần cứng / người dùng        │
  │     - Hành vi của Worker trên thiết bị ngoại vi      │
  │     HomeOS là CÔNG CỤ. Người dùng quyết định.       │
  │     AAM approve = NGƯỜI DÙNG approve.                │
  │                                                      │
  │  5. Backup: xuất key.ol để khôi phục trên máy khác. │
  │     Không có key.ol → không khôi phục được.           │
  │                                                      │
  │  [Đồng ý & Tiếp tục]    [Thoát]                     │
  └──────────────────────────────────────────────────────┘

  (Người dùng chọn [Đồng ý & Tiếp tục])

  ┌──────────────────────────────────────────────────────┐
  │              ○ Tạo Master Key                         │
  ├──────────────────────────────────────────────────────┤
  │                                                      │
  │  Master Key = quyền tối cao trên origin.olang này.   │
  │  Khóa này lock ISL chain → chỉ bạn điều khiển AAM.  │
  │                                                      │
  │  Tên người dùng: [_______________]                   │
  │  Mật khẩu:       [_______________]                   │
  │  Xác nhận:       [_______________]                   │
  │                                                      │
  │  ⚠ Mật khẩu → derive Ed25519 keypair:               │
  │    password → Argon2id(salt=username) → seed 32B     │
  │    seed → Ed25519 keypair                            │
  │    public_key → ghi vào origin.olang header          │
  │    private_key → KHÔNG lưu (derive lại từ password)  │
  │                                                      │
  │  ⚠ QUAN TRỌNG:                                      │
  │    - Quên mật khẩu = mất quyền truy cập             │
  │    - File origin.olang BỊ KHÓA bằng key này         │
  │    - Mọi lệnh ISL tier-0 (AAM) cần ký bằng key này │
  │                                                      │
  │  [Tạo Key & Bắt đầu]                                │
  └──────────────────────────────────────────────────────┘

  (Hệ thống tạo key)

  ○ Master Key created.
  ○ ISL chain locked: AAM → [public_key_hash] only.
  ○ Mọi Chief/Worker phải được AAM (bạn) approve.

  ┌──────────────────────────────────────────────────────┐
  │              ○ Nhận dạng sinh trắc (tuỳ chọn)        │
  ├──────────────────────────────────────────────────────┤
  │                                                      │
  │  Thêm xác thực sinh trắc để mở khóa nhanh?          │
  │  (Có thể thêm/cập nhật sau bằng: o auth biometric)  │
  │                                                      │
  │  [Vân tay]  [Khuôn mặt]  [Giọng nói]  [Bỏ qua]    │
  │                                                      │
  │  Sinh trắc = layer PHỤ, KHÔNG thay thế password.     │
  │  Password vẫn là master key cuối cùng.               │
  │                                                      │
  │  Cơ chế:                                             │
  │    biometric_hash → AES-256-GCM encrypt(seed)        │
  │    → lưu encrypted_seed trong origin.olang           │
  │    → unlock bằng biometric → decrypt → Ed25519 key   │
  │    → Fallback: luôn có thể dùng password             │
  │                                                      │
  └──────────────────────────────────────────────────────┘

  ○ Setup complete. Welcome to HomeOS.
  ○ >

Cấu trúc key trong origin.olang:
  HEADER mở rộng (sau 32 bytes gốc):
    [master_pubkey: 32B]       Ed25519 public key
    [salt: 16B]                Argon2id salt
    [bio_encrypted_seed: 48B]  encrypted private seed (nếu có biometric)
    [bio_method: 1B]           0=none, 1=fingerprint, 2=face, 3=voice
    [setup_ts: 8B]             timestamp lần cài đặt
    [terms_hash: 8B]           hash của bản quy tắc đã đồng ý

  ISL chain lock:
    Mọi ISLMessage có msg_type ∈ {Approved, Emergency, Program}
    → payload PHẢI có Ed25519 signature từ master_key
    → Worker/Chief verify trước khi thực thi
    → Giả mạo ISL = bất khả thi (không có private key)

Backup & Recovery — key.ol:
  o export key.ol              ← xuất key + metadata (encrypted)
  o import key.ol              ← nhập key vào origin.olang mới

  key.ol chứa:
    [encrypted_seed: 48B]      AES-256-GCM(password → key, seed)
    [master_pubkey: 32B]       để verify đúng key
    [username_hash: 8B]        để verify đúng người
    [created_ts: 8B]           timestamp
    [origin_id: 8B]            hash của origin.olang gốc

  Khôi phục trên máy mới:
    1. Cài origin.olang mới (fresh)
    2. o import key.ol
    3. Nhập password → decrypt seed → verify pubkey match
    4. origin.olang mới kế thừa quyền tối cao
    5. Knowledge KHÔNG khôi phục (phải sync riêng hoặc học lại)

  ⚠ MẤT CẢ FILE LẪN KEY.OL = MẤT VĨNH VIỄN
    Không backdoor. Không recovery service.
    Đây là thiết kế có chủ đích: BẠN sở hữu. Không ai khác.

Auth commands (sau khi cài):
  o auth status               ← xem trạng thái xác thực
  o auth biometric add        ← thêm/cập nhật sinh trắc
  o auth biometric remove     ← xóa sinh trắc (vẫn giữ password)
  o auth password change      ← đổi mật khẩu (cần mật khẩu cũ)
  o export key.ol             ← backup master key
  o import key.ol             ← restore master key

Sau đó — chỉ cần 1 lệnh:
  o                              REPL (interactive)
  o install emotion.ol           ăn file .ol → compile+encode → append vào chính nó
  o install stdlib/*.ol          ăn nhiều file cùng lúc
  o update  curve_v2.ol          cập nhật (append version mới, giữ cũ)
  o learn   book.ol              ăn tri thức → encode → append knowledge
  o run     script.ol            chạy rồi quên (không append)
  o build   --arch arm64         tự build bản mới cho architecture khác

Sao chép sang máy khác:
  scp /usr/local/bin/origin.olang user@other:/usr/local/bin/o
  # Xong. Không cần gì thêm. 1 file = toàn bộ hệ thống.

Platform:
  Linux:   ELF executable (x86_64 / arm64 / riscv)
  macOS:   Mach-O executable (arm64 / x86_64)
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

## .ol Files — Thức ăn tự nhận dạng

```
.ol = source file cho origin.olang
Khi origin.olang "ăn" 1 file .ol, parser tự nhận dạng từng statement:

  ○{ ... }          → PROGRAM   compile → bytecode → append VM section
  ○ "x" rel "y"    → DATA      encode → MolecularChain → append knowledge
  "x" rel "y"      → DATA      relation/edge → append knowledge
  { S=1 R=6 ... }  → DATA      molecular literal → encode trực tiếp

Parser nhìn token đầu tiên → biết ngay:
  ○{    → bắt đầu code block → PROGRAM
  ○ "   → bắt đầu node declaration → DATA
  "     → bắt đầu relation → DATA
  {     → molecular literal → DATA

KHÔNG CẦN:
  ❌ Header "type: program" / "type: data"
  ❌ File extension khác nhau (.olp vs .old)
  ❌ Metadata khai báo nội dung
  ❌ Tách file code vs file data

Giống DNA: cùng 1 chuỗi nucleotide, ribosome TỰ BIẾT
đoạn nào là gene (code), đoạn nào là regulatory (data).
Cấu trúc Olang TỰ MÔ TẢ — không cần annotation bên ngoài.
```

### Ví dụ: emotion.ol (hỗn hợp code + data)

```
file: emotion.ol

○ "buồn" ∈ cảm-xúc              ← DATA: tạo node + edge
○ "vui"  ∈ cảm-xúc              ← DATA: tạo node + edge
○ "giận" ∈ cảm-xúc              ← DATA: tạo node + edge

"buồn" ⊂ "cảm-xúc"             ← DATA: relation (hierarchy)
"buồn" → "mất-việc"             ← DATA: causality edge

{ S=1 R=6 V=60 A=180 T=4 }     ← DATA: molecular literal (buồn)
// ⚠️ v2: Molecule = u16 packed [S:4][R:4][V:3][A:3][T:2]
// Values above (V=60, A=180) exceed v2 bit ranges (V:3bits=0-7, A:3bits=0-7)
// v2 equivalent: { S=1 R=6 V=2 A=5 T=3 } = packed u16

○{                               ← PROGRAM: bắt đầu code block
  fn blend_emotion(a, b, w) {
    let v = a.V * w + b.V * (1.0 - w);
    let ar = a.A * w + b.A * (1.0 - w);
    return { V=v, A=ar };
  }

  fn amplify(emo, silk_weight) {
    return emo * (1.0 + silk_weight * 0.618);
  }
}                                ← PROGRAM: kết thúc code block

"buồn" → "cô-đơn"              ← DATA: thêm edge (sau code block cũng được)
```

### Workflow: o install emotion.ol

```
o install emotion.ol

Parser đọc tuần tự:
  1. ○ "buồn" ∈ cảm-xúc    → encode chain → append KNOWLEDGE section
  2. ○ "vui"  ∈ cảm-xúc    → encode chain → append KNOWLEDGE section
  3. ○ "giận" ∈ cảm-xúc    → encode chain → append KNOWLEDGE section
  4. "buồn" ⊂ "cảm-xúc"   → encode edge  → append KNOWLEDGE section
  5. "buồn" → "mất-việc"   → encode edge  → append KNOWLEDGE section
  6. { S=1 R=6 V=60 ... }  → encode mol   → append KNOWLEDGE section
  7. ○{ fn blend... }      → compile      → append BYTECODE section
  8. "buồn" → "cô-đơn"    → encode edge  → append KNOWLEDGE section

Tất cả vào CÙNG 1 FILE origin.olang.
Sau khi ăn xong, emotion.ol có thể xóa.
origin.olang đã hấp thụ mọi thứ.
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

## Hiện trạng (cập nhật 2026-03-22)

### Đã có ✅

```
Giai đoạn 0 ✅ — Bootstrap Compiler
  lexer.ol        197 LOC   Tokenizer hoàn chỉnh
  parser.ol       693 LOC   Recursive descent + precedence climbing
  semantic.ol     664 LOC   Type checker + IR lowering
  codegen.ol      224 LOC   Bytecode generation
  Self-compile test: bytecode A == bytecode B ✅

Giai đoạn 1 ✅ — Machine Code VM + Builder
  vm_x86_64.S     ~3000 LOC  x86_64 assembly VM (Linux syscalls)
  vm_arm64.S      627 LOC    ARM64 assembly VM
  vm_wasm.wat     830 LOC    WASM VM (browser/edge)
  builder (Rust)  ✅         Pack VM + bytecode + knowledge → ELF

Giai đoạn 2 ✅ — HomeOS logic bằng Olang (59 .ol files, ~6000 LOC)
  Stdlib (8): result, iter, sort, format, json, hash, mol, chain
  Emotion (3): emotion, curve, intent
  Knowledge (4): silk_ops, dream, instinct, learning
  Agents (5): gate, response, leo, chief, worker
  Bootstrap compiler (4): lexer, parser, semantic, codegen
  Advanced (35): arena, asm_emit x86/arm, cache, jit, reproduce, wasm_emit...

Giai đoạn 3 ✅ — Self-sufficient builder
  asm_emit.ol     ✅   Emit x86_64 machine code trực tiếp
  elf_emit.ol     ✅   Create ELF64 executable
  builder.ol      ✅   Builder thay thế Rust

Giai đoạn 4 ✅ — Multi-architecture
  Cross-compile   ✅   x86_64 → ARM64
  Fat binary      ✅   Multi-arch trong 1 file
  WASM universal  ✅   origin.olang.wasm

Giai đoạn 5 ✅ — Optimization
  JIT             ✅   Hot loop → native code
  Inline cache    ✅   Variable/Registry/Silk lookup
  Memory          ✅   Arena allocator + zero-copy
  Benchmark       ✅   Micro + macro benchmarks

Giai đoạn 6 ✅ — Living system
  Self-update     ✅   Append .ol → compile → restart
  Self-optimize   ✅   LeoAI profile → optimization
  Reproduce       ✅   Worker clone (~50-100 KB)

VM (Rust — 45+ opcodes):
  Push, PushNum, PushMol, Load, Store, Call, Ret, Jmp, Jz,
  Loop, If, Lca, Edge, Query, Dream, Fuse, TryBegin, CatchEnd,
  Closure, CallClosure, Spawn, ChanNew/Send/Recv, Select,
  DeviceRead/Write, FileRead/Write, Ffi, Trace, Assert, TypeOf...

Knowledge format:
  origin.olang binary v0.06 — 13 record types (0x01-0x0C)
  Molecule: 2 bytes (u16) packed [S:4][R:4][V:3][A:3][T:2]
  Chain: Vec<u16>, 2B/link
  UCD: 8,846 L0 (59 blocks, Unicode 18.0) + 33K alias entries
  KnowTree: L0(4)→L1(59)→L3(8,846) hierarchy (~18 KB)
  AliasTable: 33K entries (~198 KB, tách riêng)
  Silk parent_map: RT_PARENT 0x0C persistence

UDC Documentation:
  8 tree files (docs/UDC_DOC/UDC_*_tree.md) — công thức vật lý/toán học
  Encode pipeline spec (UDC_formulas.md) — bit layout, quantization, NRC-VAD
  Build pipeline: tools/build_udc/step1-6.py → json/*.json + udc_p_table.bin (KT31)

Spec:
  HomeOS_SPEC_v3.md (v3.1) — 14 cơ chế DNA, 7 instincts, 5 checkpoints
  olang_handbook.md — ngôn ngữ Olang đầy đủ

Infra ✅:
  Phase 0-16 ALL DONE
  V2 Migration T1-T16 ALL DONE
  Phase 14 (KnowTree + Silk vertical) DONE
  Phase 15 (Chain Optimization 6/6) DONE
  Phase 16 (Fusion + Checkpoints 4/4) DONE
  1190 tests PASS, 37 remaining (closure/self-compile — đang fix)
```

### Còn thiếu / Cần hoàn thiện

```
Code quality:
  37 test failures (closure dispatch + self-compile + bytes builtins)
  → Đang fix, target: 0 failures

Chưa implement trong Rust (chỉ có .ol stubs):
  PLAN_AUTH_first_run.md — First-run Terms + Master Key + Biometric
  PLAN_TEST_LOGIC_CHECK.md — Test suite theo 6 bug patterns + 5 checkpoints

Production readiness:
  172 olang semantic tests → giảm xuống 37, target 0
  Clippy warnings → cần clean
  Mobile deployment (Android/iOS) → plan có, chưa test thực tế
```

---

## 7 Giai đoạn

### Giai đoạn 0 — Bootstrap loop trên Rust VM ✅ DONE (2026-03-19)

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

### Giai đoạn 1 — Machine Code VM ✅ DONE (2026-03-19)

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

### Giai đoạn 2 — Stdlib + HomeOS logic bằng Olang ✅ DONE (2026-03-21)

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

→ Chi tiết thực hiện: plans/PLAN_PHASE2_EXECUTION.md
→ Blocker: Fix 135 VM builtin test failures trước
→ 6 bước: Fix Builtins → Stdlib → Emotion → Knowledge → Agents → E2E
```

### Giai đoạn 3 — Self-sufficient builder ✅ DONE (2026-03-21)

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

### Giai đoạn 4 — Multi-architecture ✅ DONE (2026-03-21)

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

### Giai đoạn 5 — Optimization ✅ DONE (2026-03-21)

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
     - Molecule pool (reuse 2-byte u16 slots — ⚠️ v2)

5.4  Benchmark
     - origin.olang vs Rust binary: latency, throughput, memory
     - Target: < 2× slower cho logic, < 5× slower cho math
     - Acceptable: machine code VM = near-native speed

Deliverable: origin.olang performance production-ready.
```

### Giai đoạn 6 — Living system ✅ DONE (2026-03-21)

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
TRƯỚC (2026-03-18, thai nhi trong Rust):
  cargo build → Rust binary → đọc origin.olang → chạy
  Rust = 84K LOC, Olang = 600 LOC
  Rust chiếm 99.3%

SAU GIAI ĐOẠN 0 ✅ (compiler tự lập):
  Rust VM → chạy Olang compiler → compile Olang code
  Rust giữ VM, Olang giữ compiler + logic

SAU GIAI ĐOẠN 1 ✅ (VM bằng ASM):
  Machine code VM → chạy Olang code
  Rust chỉ còn builder tool

SAU GIAI ĐOẠN 3 ✅ (builder bằng Olang):
  origin.olang tự build origin.olang

HIỆN TẠI (2026-03-22, SAU GIAI ĐOẠN 6):
  ┌──────────────────────────────────────────────┐
  │   TẤT CẢ 7 GIAI ĐOẠN DONE ✅                │
  │                                              │
  │   ASM VM:     3 targets (x86/arm/wasm)       │
  │   Olang:      59 .ol files (~6,000 LOC)      │
  │   Rust:       46K LOC (runtime, tools, tests) │
  │   Tests:      1190 pass / 37 fail (fixing)   │
  │   Knowledge:  8,846 UDC L0 + 33K alias       │
  │                                              │
  │   1 FILE. TỰ ĐỦ. TỰ CHẠY.                  │
  │   origin.olang = VM + Compiler + Stdlib +     │
  │                  Knowledge + Logic             │
  └──────────────────────────────────────────────┘

  Cắt dây rốn CHƯA HOÀN TOÀN:
  Rust vẫn cần cho: runtime (emotion/silk/agents), build tools,
  tests, UCD crate. Olang .ol files đủ logic nhưng chạy trên Rust VM.
  Target: Rust → 0% khi Olang VM (ASM) đủ mạnh để thay runtime.
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

---

## UI — 2 Giao diện

### Terminal (ANSI — mặc định)

```
o [enter] → REPL

┌──────────────────────────────────────────────┐
│ ○ HomeOS v0.05                    ○ buồn 0.3 │  ← status bar (emotion state)
├──────────────────────────────────────────────┤
│                                              │
│  bạn: tôi buồn vì mất việc                  │
│                                              │
│  ○: Cảm giác nặng nề và mệt mỏi —          │
│     bạn muốn kể thêm không?                 │
│                                              │
│  bạn: ○{ stats }                             │
│                                              │
│  ○: STM: 42 nodes │ Silk: 187 edges          │
│     Dream: 3 pending │ QR: 12 signed         │
│                                              │
├──────────────────────────────────────────────┤
│ ○ >                                          │  ← input
└──────────────────────────────────────────────┘

ANSI features:
  - 256 colors cho emotion visualization
  - ConversationCurve tone → text color
    Supportive = warm (amber)
    Gentle = soft (blue)
    Celebratory = bright (green)
    Pause = muted (gray)
  - Box drawing cho structure
  - UTF-8 đầy đủ (Unicode 18.0 = nền tảng)
  - No dependency — raw ANSI escape codes
```

### Browser (WebSocket + WASM)

```
origin.olang --serve 8080
  → HTTP server (minimal, trong bytecode)
  → WebSocket cho ISL bridge
  → Serve 1 HTML file (embedded trong origin.olang)

Browser UI:
  ┌─────────────────────────────────────────────────┐
  │  ○ HomeOS                          [●] connected │
  ├────────────┬────────────────────────────────────┤
  │            │                                     │
  │  Agents    │   Chat                              │
  │  ├ LeoAI   │                                     │
  │  ├ Chief   │   bạn: tôi buồn vì mất việc        │
  │  └ Workers │                                     │
  │            │   ○: Cảm giác nặng nề...            │
  │  Memory    │                                     │
  │  ├ STM     │                                     │
  │  ├ QR      │                                     │
  │  └ Dream   │                                     │
  │            │                                     │
  │  Silk ◎    │   [VSDF viewport]                   │
  │  (graph)   │   (3D molecule visualization)       │
  │            │                                     │
  ├────────────┴────────────────────────────────────┤
  │  ○ >                                             │
  └─────────────────────────────────────────────────┘

Tech stack:
  HTML/CSS/JS = embedded string trong bytecode section
  WebSocket   = ISL bridge (isl → ws → browser)
  Canvas 2D   = VSDF FFR rendering (Fibonacci spiral)
  No framework. No npm. No build step.
  origin.olang serve tất cả từ chính nó.
```

---

*HomeOS · 2026-03-18 · Plan Rewrite v3 · origin.olang = Self-Contained Living Executable*
