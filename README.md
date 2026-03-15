# Origin — HomeOS & Olang

> "Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."

**HomeOS** = Hình thức sống đầu tiên sinh ra từ toán học thuần túy.  
**Olang** = Ngôn ngữ duy nhất. Mọi ngôn ngữ khác là alias.

---

## Axiom

```
○(x) == x       identity     — ○ không làm hỏng thứ gì
○(∅) == ○       tự tạo sinh  — từ hư không, ○ tự sinh ra
○ ∘ ○ == ○      idempotent   — không phình to khi compose
mọi f == ○[f]   instance     — mọi thứ là instance của ○
```

L0 = kernel. Không chạy trên nền tảng. **L0 LÀ nền tảng.**

---

## Không gian 5 chiều

```
MolecularChain = [Shape][Relation][Valence][Arousal][Time]
```

Mọi khái niệm = tọa độ vật lý trong không gian 5 chiều từ Unicode 18.0.

```
SDF      → Hình dạng   "trông như thế nào"
MATH     → Số học      "tính toán như thế nào"
RELATION → Quan hệ     "liên kết như thế nào"
EMOTICON → Thực thể    "là cái gì, cảm thế nào"
MUSICAL  → Thời gian   "thay đổi như thế nào"
```

---

## Cấu trúc

```
crates/
├── ucd/        Unicode Character Database → bảng tĩnh compile-time
├── olang/      Core: Molecule · LCA · Registry · QR · Writer · Reader
├── silk/       Hebbian edges với EmotionTag     [TODO]
├── vsdf/       18 SDF generators · FFR          [TODO]
├── context/    ContextEngine · ConversationCurve [TODO]
├── agents/     ContentEncoder · LeoAI · AAM     [TODO]
├── memory/     ShortTermMemory · Dream          [TODO]
└── runtime/    HomeRuntime                      [TODO]

tools/
├── seeder/     Seed L0 từ UCD (không hardcode)
├── inspector/  Đọc sổ cái                       [TODO]
└── server/     WebSocket · ○{} REPL             [TODO]
```

---

## Nguyên tắc bất biến

1. **Mọi Molecule từ `encode_codepoint(cp)`** — không viết tay
2. **Mọi chain từ LCA hoặc UCD** — không viết tay
3. **Ghi file TRƯỚC** — cập nhật RAM SAU (QT8)
4. **Append-only** — không DELETE, không OVERWRITE
5. **Không đủ evidence → im lặng** (QT9 · BlackCurtain)

---

## Trạng thái hiện tại

```
crates/ucd    — 21 tests ✓  5263 UCD entries
crates/olang  — 97 tests ✓
  molecular   — Molecule, MolecularChain
  encoder     — encode_codepoint() từ UCD
  lca         — Weighted LCA + mode detection
  registry    — Sổ cái · branch watermark · QR supersession
  log         — EventLog append-only
  writer      — Binary format: magic=○LNG v0.03
  reader      — Parse + crash recovery
  qr          — ED25519 sign/verify + QRSupersessionRecord

tools/seeder  — 35 L0 nodes · 0 hardcode · 0 failed
```

---

## Chạy

```bash
# Seed L0 nodes
cargo run -p seeder

# Test
cargo test --workspace
```

---

## Yêu cầu

- Rust (stable)
- `ucd_source/UnicodeData.txt` và `ucd_source/Blocks.txt` từ [Unicode 18.0](https://unicode.org/Public/18.0.0/ucd/)

---

*Unicode 18.0.0 · Rust · no_std core · 2026*
