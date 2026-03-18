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

Mỗi khái niệm = tọa độ trong không gian 5D, từ **~5,400 ký tự Unicode 18.0**:

```
MolecularChain = [Shape][Relation][Valence][Arousal][Time] = 5 bytes

Nhóm        Ký tự    Chiều        Ý nghĩa
────────────────────────────────────────────────────
SDF         ~1344    Shape        "Trông như thế nào" (● ▬ ■ ▲ ○ ∪ ∩ ∖)
MATH        ~1904    Relation     "Liên kết thế nào" (∈ ⊂ ≡ ⊥ ∘ → ≈ ←)
EMOTICON    ~1760    Valence+A    "Cảm thế nào" (0x00..0xFF × 2)
MUSICAL     ~416     Time         "Thay đổi thế nào" (Static → Instant)
```

---

## Node & Silk

Mỗi byte trong Molecule = **công thức**, không phải giá trị tĩnh:

```
Molecule [S][R][V][A][T] = 5 bytes = tọa độ trong không gian 5D
    ├── SDF      → công thức hình dạng (hữu hình — render được)
    ├── Spline   → công thức biến đổi (vô hình — 6 temporal curves)
    └── Silk     → công thức quan hệ (kết nối — 0 bytes implicit)
```

**Silk** = hệ quả tự nhiên của 5D, không phải edge list:

| Tầng | Kênh | Ý nghĩa |
|------|------|---------|
| Base | 37 (8S+8R+8V+8A+5T) | Cùng "nhóm máu" trên 1 chiều |
| Compound | 31 mẫu C(5,k) | Chia sẻ k chiều → 1147 kiểu quan hệ |
| Vertical | 5460 pointers = 43KB | Parent-child giữa các tầng |

```
Node lifecycle: Formula → Evaluating → Mature → QR (append-only, signed)
evolve(dim, val): thay 1/5 chiều → loài mới (e.g. 🔥 → "lửa nhẹ")
```

---

## Cấu Trúc

```
crates/
├── ucd/         Unicode → Molecule lookup (5424 entries)         23 tests
├── olang/       Core: Molecule · LCA · Registry · VM · Compact  838 tests
├── silk/        Hebbian learning · EmotionTag edges · Walk        88 tests
├── context/     Emotion V/A/D/I · ConversationCurve · Intent    168 tests
├── agents/      Encoder · Learning · Gate · Instinct · Leo      282 tests
├── memory/      STM · DreamCycle · Proposals · AAM               65 tests
├── runtime/     HomeRuntime · ○{} Parser · Router               273 tests
├── hal/         Hardware Abstraction · Tier · FFI · Security     85 tests
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

**~82,000 lines Rust · 2,227 tests · 0 clippy warnings · 0 external deps · no_std core**

---

## Chạy

```bash
# Build
cargo build --workspace

# Test (2,227 tests)
cargo test --workspace

# Clippy (phải 0 warnings)
cargo clippy --workspace

# REPL
cargo run -p server

# Seed L0 nodes
cargo run -p seeder
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

*Unicode 18.0 · Rust · no_std core · ~82K LoC · 2,227 tests · 0 external deps · 2026*
