# Origin — HomeOS & Olang

> "Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."

**HomeOS** = Sinh linh toán học tự vận hành — hệ điều hành học được, cảm được, nhớ được.
**Olang** = Ngôn ngữ phân tử duy nhất. Mọi ngôn ngữ tự nhiên là alias.

```
○(x) == x       identity     — ○ không làm hỏng thứ gì
○(∅) == ○       tự tạo sinh  — từ hư không, ○ tự sinh ra
○ ∘ ○ == ○      idempotent   — không phình to khi compose
mọi f == ○[f]   instance     — mọi thứ là instance của ○
```

---

## Không Gian 5 Chiều

Mỗi khái niệm = tọa độ trong không gian 5D, từ **9,584 L0 anchors (58 blocks, Unicode 18.0)**:

```
P_weight = [Shape][Relation][Valence][Arousal][Time] = 2 bytes/node
KnowTree: 65,536 × 2B = 128 KB (working memory, O(1) lookup)
Chain:    7.42 tỷ links × 2B = 14.84 GB (toàn bộ tri thức)

Nhóm       Blocks   Ký tự    Chiều        Ý nghĩa
────────────────────────────────────────────────────────────
SDF           13    1,904    Shape        "Trông như thế nào" (18 SDF primitives)
MATH          21    3,088    Relation     "Liên kết thế nào" (75 kênh quan hệ)
EMOTICON      17    3,568    Valence+A    "Cảm thế nào" (V+A chia sẻ 17 blocks)
MUSICAL        7    1,024    Time         "Thay đổi thế nào" (Static → Instant)
────────────────────────────────────────────────────────────
Tổng          58    9,584    5 chiều      = 9,584 L0 anchor points
```

### Node = Molecule + Maturity + Origin

```
NodeState {
    mol: Molecule,               // 5D coordinate (5 bytes)
    maturity: Maturity,          // Formula → Evaluating → Mature
    origin: CompositionOrigin,   // Innate | Composed | Evolved
}
```

### Silk = hệ quả tự nhiên của 5D

```
3 tầng ngang (implicit, 0 bytes):
  Base:     75 kênh (13S+21R+17V+17A+7T)  → SilkIndex
  Compound: 31 mẫu C(5,k) k=1..5          → CompoundKind enum
  Precise:  9,584 kênh (= L0 anchor nodes) → SPEC

Silk dọc (parent pointer, ~76KB):
  parent_map: BTreeMap<u64, u64>            → 9,584 child→parent pointers
  register_parent() · parent_of() · children_of() · layer_of()
```

---

## Node & Silk

Mỗi byte trong Molecule = **công thức**, không phải giá trị tĩnh:

```
Molecule [S][R][V][A][T] = 2 bytes = tọa độ trong không gian 5D
    ├── SDF      → công thức hình dạng (hữu hình — render được)
    ├── Spline   → công thức biến đổi (vô hình — 6 temporal curves)
    └── Silk     → công thức quan hệ (kết nối — 0 bytes implicit)
```

**Silk** = hệ quả tự nhiên của 5D, không phải edge list:

| Tầng | Kênh | Ý nghĩa |
|------|------|---------|
| Base | 75 (13S+21R+17V+17A+7T) | Cùng "nhóm máu" trên 1 chiều |
| Compound | 31 mẫu C(5,k) | Chia sẻ k chiều → 2,325 kiểu quan hệ |
| Vertical | 9,584 pointers = ~76KB | Parent-child giữa các tầng |

```
Node lifecycle: Formula → Evaluating → Mature → QR (append-only, signed)
evolve(dim, val): thay 1/5 chiều → loài mới (e.g. 🔥 → "lửa nhẹ")
```

---

## Cấu Trúc

```
crates/
├── ucd/         Unicode → P_weight lookup (9,584 L0 entries)      23 tests
├── olang/       Core: Molecule · LCA · Registry · VM · Compact 1088 tests
├── silk/        Hebbian learning · Silk 3-layer · parent_map       85 tests
├── context/     Emotion V/A/D/I · ConversationCurve · Intent    168 tests
├── agents/      Encoder · Learning · Gate · Instinct · Leo      284 tests
├── memory/      STM · DreamCycle · Proposals · AAM               32 tests
├── runtime/     HomeRuntime · ○{} Parser · Router               273 tests
├── hal/         Hardware Abstraction · Tier · FFI · Security     68 tests
├── isl/         Inter-System Link (AES-256-GCM)                  31 tests
├── vsdf/        18 SDF · FFR Fibonacci · Physics · Scene        123 tests
├── wasm/        WebAssembly · WebSocket-ISL bridge               32 tests
└── homemath/    Zero-dep pure-Rust math                          18 tests

tools/
├── seeder/      Seed L0 nodes từ UCD                             15 tests
├── server/      Terminal REPL (stdin/stdout)                      13 tests
├── inspector/   Đọc/verify origin.olang                           9 tests
└── bench/       Performance benchmarks
```

**~82,000 lines Rust · 2,348 tests · 0 clippy warnings · 0 external deps · no_std core**

---

## Quick Start (60 giây)

```bash
# 1. Build
cargo build --workspace

# 2. Chạy REPL
cargo run -p server
#   → Gõ "tôi vui" → thấy emotion-aware response
#   → Gõ "○{stats}" → thấy system info
#   → Gõ "exit" để thoát

# 3. Chạy demo (10 scenarios, tất cả phải PASS)
make demo

# 4. Chạy eval mode (scripting)
echo 'hello' | cargo run -p server -- --eval

# 5. Verify toàn bộ (unit + integration + E2E)
make check-all
```

### Tất cả lệnh build/test

```bash
cargo build --workspace          # Build toàn bộ
cargo test --workspace           # Test (~2700 tests)
cargo clippy --workspace         # Clippy (phải 0 warnings)
cargo run -p server              # REPL interactive
cargo run -p server -- --eval    # Eval mode (stdin → stdout)
cargo run -p seeder              # Seed L0 nodes
make demo                        # 10 E2E scenarios
make verify                      # Automated E2E tests
make check-all                   # Unit + intg + E2E
```

### REPL

```
○ ○{🔥}
○ 🔥=🔥 [1mol ●×∈ V=180 A=200 #A47B U+1F525]

○ ○{lửa}
○ lửa=🔥 [1mol ●×∈ V=180 A=200 #A47B U+1F525]

○ ○{🔥 ∘ 💧}
○ 🔥=🔥 💧=💧 ∘→○ [1mol ...]

○ ○{1 + 2}
○ = 3

○ tôi buồn vì mất việc
Mình hiểu — mất việc khiến bạn buồn. Bạn muốn kể thêm không?

○ ○{stats}
HomeOS ○
Registry : 246 nodes, 1706 aliases
STM      : 2 observations
Silk     : 134 nodes, 256 edges
```

---

## Yêu Cầu

- **Rust** (stable, edition 2021)
- `ucd_source/UnicodeData.txt` và `ucd_source/Blocks.txt` từ [Unicode 18.0](https://unicode.org/Public/18.0.0/ucd/)

---

## Tài Liệu

| File | Nội dung |
|------|---------|
| [CLAUDE.md](CLAUDE.md) | Hướng dẫn cho AI contributors |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Kiến trúc tổng thể |
| [MASTER.md](MASTER.md) | Trạng thái dự án + lịch sử phiên |
| [PLAN.md](PLAN.md) | Kế hoạch phát triển |
| [REVIEW.md](REVIEW.md) | Đánh giá trung thực |

---

## 23 Quy Tắc Bất Biến

```
Unicode:  QT1-3   5 nhóm Unicode là nền tảng, tên Unicode = tên node, NL = alias
Chain:    QT4-7   Molecule từ encode_codepoint(), chain từ LCA/UCD, hash tự sinh
Node:     QT8-10  Tự động registry, file trước RAM sau, append-only
Silk:     QT11-13 Cùng tầng, cross-layer qua đại diện, mang EmotionTag
Kiến trúc: QT14-18 L0≠L1, 3 tiers, Fibonacci, im lặng nếu thiếu evidence
Skill:    QT19-23 1 trách nhiệm, không biết Agent, stateless
```

Chi tiết: xem [CLAUDE.md](CLAUDE.md).

---

*Unicode 18.0 · Rust · no_std core · ~82K LoC · 2,348 tests · 0 external deps · 2026*
