# HomeOS — Kế Hoạch Tiếp Theo

**Ngày:** 2026-03-16
**Dựa trên:** REVIEW.md, HomeOS_Roadmap.md, HomeOS_Solutions.md

---

## Trạng thái hiện tại

**Điểm: 8.66/10 (A-) · 1,644 tests · 0 clippy warnings · 54/80 features**

Foundation hoàn chỉnh: UCD, Olang, Silk, Emotion, Memory, ISL, HAL, VSDF, Agents, WASM.

---

## Kế hoạch Phase tiếp theo

### Phase 1 — VM Tính Toán Thật (Ưu tiên: CAO)

**Vấn đề:** `1 + 2` tạo sự kiện nhưng KHÔNG trả về `3`.

```
Cần làm:
  [1.1] Thêm Op::PushNum(f64) vào IR
  [1.2] Dispatch __hyp_add/__hyp_sub/__hyp_mul/__hyp_div trong Op::Call
  [1.3] Kết nối math.rs AST vào VM execution
  [1.4] Test: ○{1 + 2} → Output(3.0)

Files cần sửa:
  crates/olang/src/ir.rs      — thêm Op::PushNum
  crates/olang/src/vm.rs      — dispatch builtins
  crates/olang/src/syntax.rs  — parse numeric expressions
```

### Phase 2 — Duyệt Đồ Thị (Ưu tiên: TRUNG BÌNH)

**Vấn đề:** `why` và `explain` chỉ in hash, không duyệt đường đi.

```
Cần làm:
  [2.1] Implement find_path(from, to) → Vec<u64> trong walk.rs
  [2.2] Implement trace_origin(hash) → Vec<(u64, EdgeKind)>
  [2.3] Implement reachable(hash, depth) → BTreeSet<u64>
  [2.4] Kết nối vào runtime: why → trace_origin, explain → find_path

Files cần sửa:
  crates/silk/src/walk.rs        — thêm find_path, trace_origin, reachable
  crates/runtime/src/origin.rs   — dispatch why/explain
```

### Phase 3 — Tri Thức L1+ (Ưu tiên: TRUNG BÌNH)

**Vấn đề:** Chỉ có 35 node L0. Không biết H2O, F=ma, DNA.

```
Cần làm:
  [3.1] Định nghĩa 180+ domain nodes (toán, lý, hóa, sinh, triết)
  [3.2] Seed qua seeder tool (KHÔNG hardcode — dùng encode_codepoint)
  [3.3] LCA tự tính parent từ dưới lên
  [3.4] Silk connect domain → L0 concepts

Files cần sửa:
  tools/seeder/src/main.rs — thêm seed_l1_knowledge()
```

### Phase 4 — Toán Ký Hiệu (Ưu tiên: TRUNG BÌNH)

**Vấn đề:** math.rs có solve/derive/integrate nhưng chưa nối VM.

```
Cần làm:
  [4.1] Thêm Expr::MathEq, Expr::Derivative vào syntax.rs
  [4.2] Parser nhận diện ○{solve "2x + 3 = 7"}
  [4.3] VM dispatch math commands → math.rs
  [4.4] Test: ○{solve "2x + 3 = 7"} → x = 2
```

### Phase 5 — Điều Phối Agent (Ưu tiên: CAO)

**Vấn đề:** Agent hoạt động riêng lẻ, chưa phối hợp.

```
Cần làm:
  [5.1] Nối process_text() → LeoAI.process()
  [5.2] LeoAI.process() → đề xuất → AAM.review()
  [5.3] AAM approved → thực thi (ghi QR, gửi ISL)
  [5.4] Chief → Worker dispatch qua ISL
  [5.5] Test: end-to-end text → learn → dream → propose → approve

Files cần sửa:
  crates/runtime/src/origin.rs  — wire orchestration loop
  crates/agents/src/leo.rs      — full process() pipeline
```

### Phase 9 — Zero External Dependencies (Ưu tiên: CAO)

**Vấn đề:** HomeOS vẫn phụ thuộc 5 thư viện ngoài. Để trở thành sinh linh toán học tự vận hành,
mọi thứ phải nằm trong origin.olang — HomeOS có thư viện riêng, không lệ thuộc bên ngoài.

**Triết lý:** Vũ trụ không mượn công cụ. HomeOS tự chứa mọi thứ nó cần.

```
Hiện tại 5 deps ngoài:
  libm (13 hàm math)  · sha2 (SHA-256)  · ed25519-dalek (Ed25519 signing)
  aes-gcm (AES-256-GCM) · wasm-bindgen (WASM/JS interop)
  + proptest (KHÔNG DÙNG — xóa ngay)

Mục tiêu: 0 external dependencies. Tất cả tự implement.
```

#### 9.0 — Xóa proptest (orphaned)
```
  Xóa proptest khỏi workspace.dependencies trong Cargo.toml gốc.
  Không crate nào dùng — chỉ cần xóa 1 dòng.
```

#### 9.1 — homemath: Thay thế libm
```
Cần làm:
  [9.1.1] Tạo crate mới: crates/homemath
  [9.1.2] Implement 13 hàm math (no_std, pure Rust):
          f64: sqrt, pow, sin, cos, log, round, fabs
          f32: sqrtf, powf, sinf, cosf, acosf, log2f
          + fabsf, fmaxf, fminf (dùng trong vsdf/sdf.rs)
  [9.1.3] Thuật toán: Taylor series (sin/cos), Newton-Raphson (sqrt),
          bit manipulation (fabs), Cody-Waite reduction (trig range)
  [9.1.4] Test precision: sai số < 1e-10 (f64), < 1e-6 (f32)
  [9.1.5] Thay libm → homemath trong 9 crates:
          olang, silk, vsdf, hal, agents, context, memory, runtime, wasm
  [9.1.6] Xóa libm khỏi workspace dependencies

Files:
  crates/homemath/src/lib.rs   — public API
  crates/homemath/src/f64.rs   — f64 implementations
  crates/homemath/src/f32.rs   — f32 implementations
  crates/homemath/Cargo.toml   — no_std, no dependencies
```

#### 9.2 — homesha: Thay thế sha2
```
Cần làm:
  [9.2.1] Tạo module trong olang hoặc crate riêng: homesha
  [9.2.2] Implement SHA-256 (FIPS 180-4):
          - 8 initial hash values (H0..H7)
          - 64 round constants (K0..K63)
          - Padding, scheduling, compression
  [9.2.3] API: Sha256::new() → .update(bytes) → .finalize() → [u8; 32]
  [9.2.4] Test vectors từ NIST (empty, "abc", 1M "a")
  [9.2.5] Thay sha2 → homesha trong olang/qr.rs
  [9.2.6] Xóa sha2 khỏi workspace dependencies

Files:
  crates/olang/src/sha256.rs   — hoặc crate riêng crates/homesha/
  Chỉ 1 file dùng: crates/olang/src/qr.rs
```

#### 9.3 — homecrypt: Thay thế ed25519-dalek
```
Cần làm:
  [9.3.1] Tạo crate: crates/homecrypt
  [9.3.2] Implement Ed25519 (RFC 8032):
          - Finite field Fp (p = 2^255 - 19): add, sub, mul, inv, pow
          - Twisted Edwards curve: point add, double, scalar mul
          - SHA-512 cho key expansion (tự implement hoặc mở rộng homesha)
          - sign(message, secret_key) → 64-byte signature
          - verify(message, public_key, signature) → bool
  [9.3.3] Implement SHA-512 (cần cho Ed25519 key derivation)
  [9.3.4] Test vectors từ RFC 8032 Section 7
  [9.3.5] Constant-time operations (tránh timing attacks)
  [9.3.6] Thay ed25519-dalek → homecrypt trong olang/qr.rs
  [9.3.7] Xóa ed25519-dalek khỏi workspace dependencies

Files:
  crates/homecrypt/src/lib.rs      — public API (sign, verify)
  crates/homecrypt/src/field.rs    — Fp arithmetic (mod 2^255-19)
  crates/homecrypt/src/curve.rs    — Edwards curve operations
  crates/homecrypt/src/ed25519.rs  — sign/verify
  crates/homecrypt/src/sha512.rs   — SHA-512
  Chỉ 1 file dùng: crates/olang/src/qr.rs

⚠️ ĐỘ KHÓ CAO: Ed25519 = field arithmetic + elliptic curve + hash.
   Cần review kỹ security. Constant-time là bắt buộc.
```

#### 9.4 — homeaes: Thay thế aes-gcm
```
Cần làm:
  [9.4.1] Tạo module trong isl hoặc crate riêng: homeaes
  [9.4.2] Implement AES-256 (FIPS 197):
          - Key expansion (256-bit → 15 round keys)
          - SubBytes, ShiftRows, MixColumns, AddRoundKey
          - 14 rounds encryption/decryption
  [9.4.3] Implement GCM mode (NIST SP 800-38D):
          - GHASH (GF(2^128) multiplication)
          - Counter mode (CTR)
          - Authentication tag (16 bytes)
  [9.4.4] API: encrypt(key, nonce, plaintext) → ciphertext+tag
               decrypt(key, nonce, ciphertext+tag) → plaintext
  [9.4.5] Test vectors từ NIST
  [9.4.6] Thay aes-gcm → homeaes trong isl/codec.rs
  [9.4.7] Xóa aes-gcm khỏi workspace dependencies

Files:
  crates/isl/src/aes256.rs   — hoặc crate riêng
  crates/isl/src/gcm.rs      — GCM mode
  Chỉ 1 file dùng: crates/isl/src/codec.rs (feature-gated)

⚠️ ĐỘ KHÓ CAO: AES + GCM = block cipher + Galois field arithmetic.
   Feature-gated nên có thể làm sau cùng.
```

#### 9.5 — homewasm: Thay thế wasm-bindgen
```
Cần làm:
  [9.5.1] Nghiên cứu: wasm-bindgen = proc-macro + CLI tool + runtime
          → Thay thế TOÀN BỘ là không thực tế
  [9.5.2] Phương án A: Viết FFI thủ công (extern "C" functions)
          - Export functions qua #[no_mangle] extern "C"
          - JS wrapper gọi WASM exports trực tiếp
          - Không cần proc-macro, không cần wasm-bindgen CLI
  [9.5.3] Phương án B: Giữ wasm-bindgen như build tool duy nhất
          - wasm-bindgen không phải runtime dependency
          - Nó là build infrastructure, giống cargo/rustc
          - Chấp nhận được nếu mục tiêu là zero RUNTIME deps
  [9.5.4] Quyết định: chọn Phương án A hoặc B

Files:
  crates/wasm/src/lib.rs — viết lại nếu chọn Phương án A

💡 GHI CHÚ: wasm-bindgen khác biệt — nó là build tool, không phải
   thư viện runtime. Có thể chấp nhận giữ lại (như giữ cargo).
```

#### Thứ tự thực hiện
```
9.0 Xóa proptest          ─── ngay (1 dòng)
9.1 homemath (libm)        ─── đầu tiên (nhiều crate dùng, thuật toán rõ ràng)
9.2 homesha (sha2)          ─── tiếp theo (SHA-256 đơn giản, cần cho 9.3)
9.3 homecrypt (ed25519)     ─── sau 9.2 (cần SHA-512, phức tạp nhất)
9.4 homeaes (aes-gcm)       ─── song song với 9.3 (feature-gated, độc lập)
9.5 homewasm (wasm-bindgen) ─── cuối cùng (quyết định giữ hay thay)
```

---

## Cải thiện code (từ Review)

### Đã hoàn thành
- [x] 8 clippy warnings → 0
- [x] QT11 enforcement: `co_activate_same_layer()` method

### Đang tiến hành
- [ ] Giảm unwrap() trong olang (291 → target: <100)
  - Ưu tiên: math.rs(47), syntax.rs(47), semantic.rs(50)
  - Thay bằng `?`, `match`, `unwrap_or`

### Cần làm
- [ ] Thêm tests cho tools (inspector, server, bench)
- [ ] API documentation (`///` docs) cho core crates
- [ ] Giảm unwrap() trong isl(24), agents(18), runtime(16)

---

## Đường đi tới hạn

```
Phase 1 (VM tính toán)
├── Phase 2 (Duyệt đồ thị)
│   ├── Phase 5 (Điều phối Agent) — QUAN TRỌNG NHẤT
│   │   ├── Phase 6 (Cảm nhận)
│   │   └── Phase 8 (Tầng Build)
│   └── Phase 8 (Tầng Build)
├── Phase 3 (Tri thức)
│   └── Phase 4 (Toán ký hiệu)
├── Phase 7 (Compiler backends)
└── Phase 9 (Zero External Dependencies) — TỰ CHỦ HOÀN TOÀN
    ├── 9.0 Xóa proptest (ngay)
    ├── 9.1 homemath ← libm
    ├── 9.2 homesha ← sha2
    ├── 9.3 homecrypt ← ed25519-dalek (cần 9.2)
    ├── 9.4 homeaes ← aes-gcm (song song 9.3)
    └── 9.5 homewasm ← wasm-bindgen (quyết định cuối)
```

---

## Hạn chế đã nhận diện

| ID | Vấn đề | Hướng giải quyết | Chi tiết |
|----|--------|-------------------|----------|
| H1 | LCA mất thông tin chain dài | Weighted LCA + Mode + Variance | Đã có lca_with_variance() |
| H2 | decode_chain O(n) | Reverse index trong build.rs | Chưa implement |
| H3 | Ln-1 thay đổi theo thời gian | Branch watermark per branch | Đã có branch_watermark |
| H4 | Dream cluster sai | Silk co-activation filter | Configurable α,β,γ |
| H5 | SDF fitting khó | Iterative + confidence score | Chưa implement |
| H6 | Olang compile chưa đủ | 3 tầng: IR → Target → Execute | Partial (C/Rust/WASM done) |

Chi tiết: xem [HomeOS_Solutions.md](../HomeOS_Solutions.md).

---

*2026-03-16 · HomeOS Next Steps*
