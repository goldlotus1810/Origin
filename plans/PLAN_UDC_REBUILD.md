# PLAN_UDC_REBUILD — Xây dựng lại UDC.md đúng chuẩn

> **Ngày tạo:** 2026-03-20
> **Cập nhật:** 2026-03-20 (v4 — hierarchy #emoji→P→alias, KnowTree=tree not flat, json/ucd.json là canonical source)
> **Tác giả:** Lara (AI session)
> **Branch:** `claude/lara-SBLZg`
> **Trạng thái:** 🟢 ACTIVE — schema chuẩn, có thể bắt đầu encode

---

## 1. VẤN ĐỀ: Tại sao P hiện tại SAI

### P hiện tại trong Olang/ucd (SAI):
```
Molecule = 5 bytes [S][R][V][A][T]
encode_codepoint(cp) → mỗi ký tự được gán VÀO 1 chiều
→ ● được gán S=Sphere, các chiều còn lại = default
→ Olang hiểu: "● THUỘC chiều S" (phân loại)
```

### Vấn đề cốt lõi:
Olang đang dùng P như **nhãn phân loại** (char thuộc nhóm nào), thay vì dùng P như **tọa độ 5D** (char ở ĐÂU trong không gian tri thức).

### P đúng — theo SINH_HOC_v2:
```
P = (S, R, V, A, T) = TRỌNG SỐ ĐÃ TÍCH PHÂN — không phải công thức compute lại mỗi lần

  S = weight_s   Shape    (kết quả ∫ₛ cấp 3 từ char → sub → block)
  R = weight_r   Relation (kết quả ∫ₛ từ nhóm MATH)
  V = weight_v   Valence  (kết quả ∫ₛ từ nhóm EMOTICON)
  A = weight_a   Arousal  (kết quả ∫ₛ từ nhóm EMOTICON)
  T = weight_t   Time     (kết quả ∫ₛ từ nhóm MUSICAL)

Vòng đời:
  Bootstrap (1 lần): người đọc emoji → encode vào P_weight → SEAL
  Runtime:           đọc P_weight trực tiếp → O(1), không compute lại
  Learned:           Encoder ∫ₜ → ΔP_weight → Hebbian → CHÍN → QR

L0 emoji = calibration anchors (như 0°C và 100°C của nhiệt kế):
  🔥 (1F525): S=Sphere, R=Causes,  V=0xC0, A=0xC0, T=Fast
  😊 (1F60A): S=Sphere, R=Member,  V=0xE0, A=0x70, T=Medium
  💔 (1F494): S=Sphere, R=Causes,  V=0x10, A=0x50, T=Slow
  → Vĩnh viễn, không thay đổi. Mọi khái niệm khác = distance tới tập này.
```

### P đúng — DNA analogy:
```
DNA:     A — T — G — C — C — A — T...   (mỗi base = 1 nucleotide có danh tính riêng)
HomeOS:  ○{○{○{○{○{...}}}}}              (mỗi char = 1 node có tọa độ 5D riêng)
```

---

## 2. KIẾN TRÚC MỚI: UDC 9,584 = L0 KnowTree

### Nguyên lý — từ SINH_HOC_v2:
```
UDC 9,584 chars (58 blocks)  = L0 KnowTree  → mỗi char có P_weight đầy đủ, SEALED
Text (ngôn ngữ tự nhiên)     = ALIAS        → trỏ vào UDC node qua LCA

Phân cấp trong 9,584:
  Emoji (visual, E.xx blocks) = neo cảm xúc mạnh nhất (V/A rõ ràng nhất)
  Math (M.xx blocks)          = neo quan hệ (R rõ ràng nhất)
  SDF (S.xx blocks)           = neo hình dạng (S rõ ràng nhất)
  Musical (T.xx blocks)       = neo thời gian (T rõ ràng nhất)

Ví dụ:
  🟥 RED SQUARE        → P = { S=Square R=Contains V=0xC0 A=0x80 T=Static }  ← node thật
  ■  BLACK SQUARE      → alias → 🟥 (cùng hình, khác màu/V)
  "hình vuông đỏ"      → alias → 🟥
  "red square"         → alias → 🟥
```

### KnowTree là CÂY, không phải 1 flat array:
```
KnowTree = cây Fibonacci nhiều tầng:

  Root branch (L0 working memory):
    Array 65,536 × 5B = 328KB  ← ĐÂY CHỈ LÀ 1 NHÁNH (root branch)
    KnowTree[u16] → P_weight   — O(1), không cần hash
    9,584 slots đầu = UDC L0   — phần còn lại = learned/system

  Cây đầy đủ (nhiều tầng):
    L0: 9,584 UDC nodes (root)
    L1: LCA clusters của kinh nghiệm học
    L2: LCA clusters cấp cao hơn
    ...
    → Cây tăng trưởng theo Fibonacci khi học thêm
    → Mỗi tầng có root branch riêng (u16 indexed)
    → 65,536 = kích thước MỖI nhánh, không phải toàn bộ cây

  Ví dụ (v2 Section 4.2): 1 cuốn sách 100 trang:
    L0: 1,700 nodes (câu/ý)
    L1:    50 nodes (đoạn văn, gom Fib[8]=34)
    L2:     3 nodes (mục/phần)
    L3:     1 node  (cuốn sách)
    → Mỗi L có root branch riêng, mỗi branch ≤ 65,536 slots

Chain link = u16 (2 bytes) = index vào branch của tầng hiện tại
⚠️ u16 là ĐÚNG — không phải u32. v2 đã xác nhận rõ.

### Hierarchy: #emoji → P → alias → alias → ...
```
Mỗi emoji = canonical root node (#emoji):
  → có P_weight đầy đủ (S, R, V, A, T) — SEAL
  → mọi thứ khác ALIAS về nó

Ví dụ:
  #🔥 (U+1F525)  P = { S=Sphere R=Causes V=0xC0 A=0xC0 T=Fast }
    ↳ alias: U+2605 ★ (ngôi sao → lửa sáng, V override=0xB0)
    ↳ alias: "lửa" (tiếng Việt)
    ↳ alias: "fire" (tiếng Anh)
    ↳ alias: "feu" (tiếng Pháp)

  #😢 (U+1F622)  P = { S=Sphere R=Member V=0x30 A=0x60 T=Slow }
    ↳ alias: U+1F625 😥 (sad but relieved — gần nhau về VA)
    ↳ alias: "buồn" (tiếng Việt)
    ↳ alias: "sad" (tiếng Anh)

Chain: #emoji có P → alias chỉ lưu {canonical_cp, V_override?, A_override?}
Alias KHÔNG copy P — chỉ trỏ vào canonical, optional override V/A
```

### Flow xây dữ liệu (không phải parse từ txt):
```
KHÔNG derive tự động từ emoji-data.txt hay UnicodeData.txt.
Chúng ta XÂY thủ công JSON → đó là nguồn dữ liệu canonical.

Flow:
  Người → nhìn emoji → encode P tay → ghi vào UDC.md (draft)
  UDC.md → review + validate → ghi vào json/ucd.json (canonical source)
  json/ucd.json → ucd crate đọc khi compile → build.rs nạp vào bảng tĩnh

Không có bước "parse emoji-data.txt để sinh P" — P là tri thức con người,
không phải metadata Unicode. emoji-data.txt chỉ dùng để: tên emoji, codepoint range.
```

---

## 3. CẤU TRÚC P ĐÚNG

### 5 Chiều và nguồn gốc:

| Chiều | Kiểu | Range | Ý nghĩa |
|-------|------|-------|---------|
| **S** (Shape)    | enum  | 8 vals | "Trông như thế nào" — hình học |
| **R** (Relation) | enum  | 8 vals | "Liên kết thế nào" — quan hệ |
| **V** (Valence)  | u8    | 0x00..0xFF | "Dương/âm" — cảm xúc |
| **A** (Arousal)  | u8    | 0x00..0xFF | "Mạnh/yếu" — cường độ |
| **T** (Time)     | enum  | 5 vals | "Nhanh/chậm" — nhịp |

```
S: Sphere(0) | Line(1) | Square(2) | Triangle(3) | Empty(4) | Union(5) | Intersect(6) | SetMinus(7)
R: Member(0) | Subset(1) | Equiv(2) | Orthogonal(3) | Compose(4) | Causes(5) | Approximate(6) | Inverse(7)
T: Static(0) | Slow(1) | Medium(2) | Fast(3) | Instant(4)
V: 0x00 (cực âm) → 0x80 (trung tính) → 0xFF (cực dương)
A: 0x00 (tĩnh lặng) → 0x80 (trung bình) → 0xFF (kích động mạnh)
```

### Olang syntax:
```olang
"FIRE"         == { S=Sphere R=Causes  V=0xC0 A=0xC0 T=Fast   }
"RED_SQUARE"   == { S=Square R=Contains V=0xC0 A=0x80 T=Static }
"SPARKLES"     == { S=Empty  R=Compose  V=0xE0 A=0xC0 T=Fast   }
```

---

## 4. 58 UNICODE BLOCKS = 9,584 UDC (từ SINH_HOC_v2)

> **Nguồn:** `old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md` Section 1.4

```
SDF — 13 blocks, 1,904 chars (Shape)
  S.01  Arrows                 2190..21FF    112
  S.02  Box Drawing            2500..257F    128
  S.03  Block Elements         2580..259F     32
  S.04  Geometric Shapes       25A0..25FF     96
  S.05  Dingbats               2700..27BF    192
  S.06  Supp Arrows-A          27F0..27FF     16
  S.07  Supp Arrows-B          2900..297F    128
  S.08  Misc Symbols+Arrows    2B00..2BFF    256
  S.09  Geometric Shapes Ext   1F780..1F7FF  128
  S.10  Supp Arrows-C          1F800..1F8FF  256
  S.11  Ornamental Dingbats    1F650..1F67F   48
  S.12  Misc Technical         2300..23FF    256
  S.13  Braille Patterns       2800..28FF    256

MATH — 21 blocks, 3,088 chars (Relation)
  M.01  Superscripts+Subscripts   2070..209F     48
  M.02  Letterlike Symbols        2100..214F     80
  M.03  Number Forms              2150..218F     64
  M.04  Mathematical Operators    2200..22FF    256  ← ~35 Silk edges
  M.05  Misc Math Symbols-A       27C0..27EF     48
  M.06  Misc Math Symbols-B       2980..29FF    128
  M.07  Supp Math Operators       2A00..2AFF    256
  M.08  Math Alphanum Symbols     1D400..1D7FF 1024
  M.09–M.21  (Ancient numerics, Siyaq, Arab math...)  1,184

EMOTICON — 17 blocks, 3,568 chars (Valence + Arousal)
  E.01  Enclosed Alphanumerics    2460..24FF    160
  E.02  Misc Symbols              2600..26FF    256
  E.03–E.05  (Mahjong, Domino, Playing Cards)   256
  E.06–E.07  (Enclosed supp, Ideographic supp)  512
  E.08  Misc Sym+Pictographs     1F300..1F5FF  768  ← lớn nhất
  E.09  Emoticons                 1F600..1F64F   80
  E.10–E.17  (Transport, Alchemical, Chess...)  1,536

MUSICAL — 7 blocks, 1,024 chars (Time)
  T.01  Yijing Hexagram           4DC0..4DFF     64
  T.02  Znamenny Musical          1CF00..1CFCF  208
  T.03  Byzantine Musical         1D000..1D0FF  256
  T.04  Musical Symbols           1D100..1D1FF  256
  T.05–T.07  (Ancient Greek, Supp, Tai Xuan)    240

─────────────────────────────────────
TỔNG: 58 blocks = 9,584 UDC chars = L0 KnowTree
```

---

## 5. SCHEMA UDC.md — Template

```markdown
# UDC — Unicode Character Database for HomeOS
# Emoji = Canonical Nodes | UTF-32 = Alias

---

## Nhóm [N]: [Tên nhóm] (U+XXXX..U+YYYY)

**Chiều chủ đạo:** [S | R | VA | T | mix]
**Tổng:** ~N chars

### Formula chung (Nhóm):
```olang
P_group_N = { S=[val] R=[val] V=[val] A=[val] T=[val] }
```

---

### [N].[sub] [Tên sub-nhóm] (U+XXXX..U+XXYY)

**Semantic:** [mô tả ngắn]

```olang
# U+XXXX [CHAR] [EMOJI_NAME]
"EMOJI_NAME" == { S=[val] R=[val] V=[val] A=[val] T=[val] }

# U+XXXXY [CHAR] [EMOJI_NAME_2]
"EMOJI_NAME_2" == { S=[val] R=[val] V=[val] A=[val] T=[val] }
```

#### Alias từ UTF-32:
```
U+25A0 ■ → "BLACK_SQUARE" (alias vào RED_SQUARE với V override)
U+25CF ● → "BLACK_CIRCLE" (alias vào BLUE_CIRCLE với V=0x40)
```
```

---

## 6. QUY TẮC GÁN GIÁ TRỊ P

### S (Shape) — Từ hình học visual của emoji:
```
Sphere(0)    → ký tự tròn/bong bóng: ⚪🔵🌕⚽
Line(1)      → ký tự thẳng/kéo dài: ➖➡️〰️
Square(2)    → ký tự vuông/hộp: 🟥🟦⬜⬛
Triangle(3)  → ký tự tam giác/nhọn: 🔺🔻⚠️🎵
Empty(4)     → ký tự rỗng/trong suốt/placeholder: 🔲🔳
Union(5)     → ký tự gộp/mở rộng/nổ: 💥🎆🌟✨
Intersect(6) → ký tự giao/trùng/ghép: 🔗🤝⚔️✂️
SetMinus(7)  → ký tự loại trừ/cắt/phân chia: ➗🚫❌🗑️
```

### R (Relation) — Từ ngữ nghĩa chức năng của emoji:
```
Member(0)      → "thuộc về/là thành viên": 👤👥🏠🏡
Subset(1)      → "bao hàm/chứa đựng": 📦🗂️📁🗃️
Equiv(2)       → "bằng nhau/tương đương/cân bằng": ⚖️🤝🔄
Orthogonal(3)  → "độc lập/vuông góc/tự do": 🆓🔓🗺️
Compose(4)     → "ghép/kết hợp/tạo ra": 🔧🛠️🎨🍳
Causes(5)      → "gây ra/dẫn tới/liên quan nhân quả": ⚡💥🔥💣
Approximate(6) → "xấp xỉ/gần/khoảng chừng": 🌊🌫️〰️
Inverse(7)     → "ngược lại/phản chiều": 🔄🔃↩️↪️
```

### V (Valence) — Thang cảm xúc 0x00..0xFF:
```
0x00..0x1F  → cực âm (chết, đau, nguy hiểm): ☠️💀😵👿
0x20..0x3F  → âm mạnh (buồn, sợ, tức): 😢😰😡💔
0x40..0x5F  → âm nhẹ (lo, nghi ngờ, tối): 😟⚠️🌑
0x60..0x7F  → dưới trung tính (bình thường, trung lập): ⬛🖤😐
0x80        → trung tính hoàn toàn: ⚪○□ (geometric neutral)
0x81..0x9F  → trên trung tính (nhẹ tốt): 😊🌿
0xA0..0xBF  → dương nhẹ (vui, tốt, an toàn): ✅😄🌸💚
0xC0..0xDF  → dương mạnh (hạnh phúc, đẹp, sáng): 🌟😍💛🌈
0xE0..0xFF  → cực dương (tuyệt vời, thiêng liêng): ✨🎆👑💎
```

### A (Arousal) — Cường độ kích hoạt 0x00..0xFF:
```
0x00..0x3F  → rất tĩnh (ngủ, dừng, bình yên): 💤😴🌙
0x40..0x7F  → tĩnh (nhẹ nhàng, chậm): 🌿🍃☁️
0x80        → trung bình (bình thường): 🌤️🚶
0x81..0xBF  → kích thích vừa (hoạt động, chú ý): ⚠️🔔🏃
0xC0..0xFF  → kích thích mạnh (khẩn cấp, mãnh liệt): 🔥💥⚡🚨
```

### T (Time) — Nhịp thời gian:
```
Static(0)   → không biến đổi, vĩnh cửu: ⚪■⬛🗿
Slow(1)     → chậm, thiền định, tăng trưởng: 🌱🐢🌙
Medium(2)   → nhịp bình thường, đi bộ: 🚶🌤️🕐
Fast(3)     → nhanh, hành động, chuyển động: 🚀🏃⚡🎵
Instant(4)  → tức thì, kích nổ, bùng phát: 💥⚡☇💢
```

---

## 7. KẾ HOẠCH THỰC HIỆN

### Phase 0: Skeleton (1 session)
- [ ] Tạo `docs/UDC.md` với header + schema + bảng 5 chiều
- [ ] Commit: `docs: create UDC.md skeleton`

### Phase 1: E.09 — Emoticons/Faces (1 session, ~80 chars)
**Lý do bắt đầu ở đây:** Faces = VA dominant → dễ nhất để NHÌN và encode tay
**Quy trình bootstrap (từ SINH_HOC_v2):**
  - NHÌN vào từng emoji
  - HỎI: trông ra sao? làm gì? cảm giác thế nào? tốc độ?
  - Ghi S, R, V, A, T → SEAL (không derive từ code)
- [ ] U+1F600..U+1F64F: 80 emoji khuôn mặt → ghi vào UDC.md
- [ ] Commit: `docs: UDC.md - E.09 Emoticons group (1F600-1F64F)`

### Phase 2: Nhóm 2 — Misc Symbols (1 session, ~180 chars)
- [ ] U+2600..U+26FF: thời tiết, biểu tượng, thể thao
- [ ] Commit: `docs: UDC.md - Misc Symbols group (2600-26FF)`

### Phase 3: Nhóm 3 — Dingbats/Arrows (1 session, ~80 chars)
- [ ] U+2700..U+28FF
- [ ] Commit: `docs: UDC.md - Dingbats/Arrows group (2700-28FF)`

### Phase 4: Nhóm 1 — Geometric/Technical (1 session, ~90 chars)
- [ ] U+2000..U+25FF
- [ ] **+ alias map**: UTF-32 geometric → emoji

### Phase 5: Nhóm 7 — Pictographs/Nature/Objects (nhiều sessions, ~500 chars)
**Lớn nhất** — chia thành sub-sessions:
- [ ] 7a: Weather & Nature (🌀🌊🌱) — ~80 chars
- [ ] 7b: Food & Drink (🍎🍕🍺) — ~100 chars
- [ ] 7c: Animals (🐶🐱🦁) — ~100 chars
- [ ] 7d: Objects & Tools (🎀🏠🔧) — ~100 chars
- [ ] 7e: Activities (🎭🏃⚽) — ~120 chars

### Phase 6: Nhóm 9 — Transport/Signs (~200 chars)
- [ ] 9a: Vehicles (🚀🚗🚂) — ~80 chars
- [ ] 9b: Signs & Symbols (🛒🚦🚫) — ~120 chars

### Phase 7: Nhóm B+C — Supplemental + Extended (~290 chars)
- [ ] B: 1F900..1F9FF (🤖🦁🥇)
- [ ] C: 1FA00..1FAFF (🪄🧬🦾)

### Phase 8: Nhóm 0+4+5+6 — Misc small groups (~120 chars)
- [ ] ASCII emoji (#️ *️ 0️-9️ ©️ ®️)
- [ ] BMP misc (Ⓜ ㊗)
- [ ] Mahjong/Cards
- [ ] CJK enclosed

### Phase 9: Alias Map (1 session)
- [ ] Tạo `docs/UDC_ALIAS.md`: mọi UTF-32 symbol → emoji alias
- [ ] Geometric shapes → emoji hình dạng tương ứng
- [ ] Math operators → emoji quan hệ tương ứng
- [ ] Musical symbols → emoji thời gian tương ứng

### Phase 10: Sinh JSON (1 session)
- [ ] Script parse UDC.md + UDC_ALIAS.md
- [ ] Sinh `json/ucd_utf32.json`
- [ ] Verify: coverage, format, no duplicates

---

## 8. OUTPUT CUỐI CÙNG

```
docs/
  UDC.md          — P definition cho ~9,584 chars (draft, human-readable)

json/
  ucd.json        — CANONICAL SOURCE: canonical nodes + alias chain
```

### JSON format (canonical nodes + alias chain):
```json
{
  "nodes": {
    "1F525": { "name": "FIRE",         "S": 0, "R": 5, "V": 192, "A": 192, "T": 3 },
    "1F622": { "name": "CRYING_FACE",  "S": 0, "R": 0, "V": 48,  "A": 96,  "T": 1 },
    "1F7E5": { "name": "RED_SQUARE",   "S": 2, "R": 1, "V": 192, "A": 128, "T": 0 }
  },
  "aliases": {
    "2605":   { "canonical": "1F525", "V_override": 176 },
    "25A0":   { "canonical": "1F7E5" },
    "1F625":  { "canonical": "1F622", "A_override": 112 }
  },
  "text_aliases": {
    "vi": {
      "lửa":  "1F525",
      "buồn": "1F622"
    },
    "en": {
      "fire": "1F525",
      "sad":  "1F622"
    }
  }
}
```

⚠️ **ucd.json là CANONICAL SOURCE** — không phải output được generate.
build.rs đọc file này khi compile → sinh bảng tĩnh.

---

## 9. NGUYÊN TẮC BẤT BIẾN

```
① #emoji là CANONICAL NODE — có P đầy đủ, không derive từ code
② P value là tri thức con người (nhìn → encode) — không phải metadata Unicode
③ Alias KHÔNG copy P — chỉ trỏ canonical + optional V/A override
④ Alias chỉ override V và/hoặc A — KHÔNG override S, R, T
⑤ ucd.json là NGUỒN, không phải output — build.rs đọc nó
⑥ KnowTree 65,536 = kích thước 1 branch — toàn cây lớn hơn nhiều
⑦ Emoji name = tên chính xác từ Unicode (UPPERCASE, underscore)
⑧ Mỗi canonical node có DUY NHẤT 1 entry trong "nodes"
```

---

## 10. BẮT ĐẦU: Session đầu tiên làm gì?

1. Tạo `docs/UDC.md` — Phase 0 skeleton + schema
2. Tạo `json/ucd.json` — skeleton (nodes={}, aliases={}, text_aliases={})
3. Điền **E.09: Emoticons/Faces** (U+1F600..U+1F64F) — ~80 canonical nodes
   - NHÌN từng emoji → encode P tay
   - Ghi vào cả UDC.md (human-readable) VÀ json/ucd.json (canonical)
   - Tên lấy từ `ucd_source/emoji-data.txt` cho chính xác
4. Commit + push

**Lý do bắt đầu với Faces:**
- VA rõ ràng nhất (mặt người = cảm xúc trực quan) → dễ encode tay
- 80 chars = vừa để validate schema + flow json
- Xong → có template cho mọi nhóm khác

---

## 11. CÁC FILE THAM KHẢO

| File | Dùng để | Status |
|------|---------|--------|
| `ucd_source/UnicodeData.txt` | Tên + category mọi codepoint Unicode 18.0 | ✅ v18 sẵn sàng |
| `ucd_source/emoji-data.txt` | **Nguồn canonical** — Emoji_Presentation property | ✅ v18 mới tải |
| `ucd_source/emoji-test.txt` | 3,966 fully-qualified emoji list | ✅ v18 mới tải |
| `ucd_source/Blocks.txt` | Range blocks Unicode 18.0 | ✅ v18 sẵn sàng |
| `crates/ucd/build.rs` | Sinh bảng tĩnh lúc compile | 🔧 Cần thêm emoji-data.txt + v18 blocks |
| `tmp_P_tree.md` | Cấu trúc cây tham khảo | 📖 tham khảo |
| `old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md` | Triết lý P đúng | 📖 tham khảo |

### Kiến trúc 2 thế giới (từ session 2026-03-20):
```
Emoji  🔥         = SDF node thật   = thế giới HÌNH ẢNH (human)
Olang/UDC         = ngôn ngữ về emoji = thế giới TÍNH TOÁN (machine)
Môi trường chung  = Unicode 18.0   = không gian định nghĩa chung

Unicode codepoint U+1F525:
  → Người: 🔥 (nhìn thấy ngay)
  → Olang: "FIRE" == { S=Sphere R=Causes V=0xC0 A=0xC0 T=Fast }
  → SDF:   f(p) = sphere SDF, màu cam/đỏ, ánh sáng động
  → Cùng 1 codepoint, 3 cách nhìn, 1 Molecule
```

### Số liệu chuẩn (từ SINH_HOC_v2):
```
UDC chars (58 blocks):         9,584  ← L0 canonical nodes
KnowTree root branch:         65,536 slots (u16, 2^16) = 1 nhánh
KnowTree root branch size:     328 KB (65,536 × 5B)
KnowTree toàn cây:             nhiều tầng, tăng theo Fibonacci khi học
Chain link size:                 2B   (u16) = index vào branch hiện tại
Bootstrap: người encode tay → SEAL → json/ucd.json
Canonical emoji nodes:       ~3,568  (EMOTICON group E.01-E.17) + S/M/T groups
Reference files (tên/range): ucd_source/emoji-data.txt, ucd_source/UnicodeData.txt
JSON canonical source:       json/ucd.json (chúng ta xây thủ công)
```

---

*Plan v3 — Cập nhật 2026-03-20: Unicode 18.0 data files đã có trong ucd_source/,
emoji-data.txt là nguồn canonical cho EMOTICON group, 3 blocks mới cần thêm vào build.rs.*
