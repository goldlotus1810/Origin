# PLAN_UDC_REBUILD — Xây dựng lại UDC.md đúng chuẩn

> **Ngày tạo:** 2026-03-20
> **Cập nhật:** 2026-03-20 (v3 — Unicode 18.0, emoji-data.txt canonical)
> **Tác giả:** Lara (AI session)
> **Branch:** `claude/lara-SBLZg`
> **Trạng thái:** 🟢 ACTIVE — data files sẵn sàng, có thể thực thi

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

### KnowTree structure (từ SINH_HOC_v2):
```
KnowTree = array 65,536 phần tử (u16 index):
  [gen: 2 bits][address: 14 bits]
    gen=00: UDC base L0 (0..9583)    — 9,584 slots
    gen=01: learned L5  (early)      — 16,384 slots
    gen=10: learned L6+ (mature)     — 16,384 slots
    gen=11: system/reserved          — 16,384 slots

  Mỗi phần tử = P_weight: Mol (5 bytes)
  KnowTree toàn bộ: 65,536 × 5B = 328 KB  ← vừa L1 cache!

  KnowTree[codepoint] → P_weight  — O(1), không cần hash
  Chain link = u16 (2 bytes) = đủ trỏ vào toàn bộ KnowTree
```

⚠️ **u16 là ĐÚNG** — không phải u32. v2 đã xác nhận rõ.

### Flow sinh UDC.md:
```
58 Unicode blocks (9,584 chars)
  → UDC.md: với mỗi char, người encode:
      NHÌN vào ký tự / emoji
      HỎI: "Nó trông ra sao? Nó làm gì? Cảm giác thế nào? Tốc độ?"
      → ghi P_weight = (S, R, V, A, T) → SEAL
  → alias_map: text/language → UDC node
  → ucd_utf32.json: { codepoint: P_weight }
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
  UDC.md          — P definition cho 1,447 emoji
  UDC_ALIAS.md    — UTF-32 → emoji alias map

json/
  ucd_utf32.json  — { "1F525": {S:0, R:5, V:192, A:192, T:3}, ... }
```

### JSON format:
```json
{
  "1F525": { "name": "FIRE",        "S": 0, "R": 5, "V": 192, "A": 192, "T": 3 },
  "1F600": { "name": "GRINNING_FACE","S": 0, "R": 0, "V": 230, "A": 180, "T": 2 },
  "25A0":  { "name": "BLACK_SQUARE", "alias": "1F7E5", "V_override": 64 }
}
```

---

## 9. NGUYÊN TẮC BẤT BIẾN

```
① Mọi P value phải có LÝ DO từ emoji-data.txt hoặc Unicode name
② Emoji name = tên chính xác từ emoji-data.txt (UPPERCASE, underscore)
③ Mỗi emoji có DUY NHẤT 1 dòng Olang
④ Alias không thay đổi S, R, T — chỉ có thể override V và/hoặc A
⑤ Olang dùng tên enum (S=Square), JSON dùng số (S=2)
⑥ Nhóm 7 (Pictographs) là phức tạp nhất — luôn làm sub-session nhỏ
```

---

## 10. BẮT ĐẦU: Session đầu tiên làm gì?

1. Tạo `docs/UDC.md` — Phase 0 skeleton
2. Điền **Nhóm 8: Emoticons/Faces** (U+1F600..U+1F64F) — ~80 emoji
   - Dùng `json/emoji/emoji-data.txt` để lấy tên chính xác
   - Assign V/A từ visual cue (😀=vui → V=0xE0, 😢=buồn → V=0x30)
3. Commit + push

**Lý do bắt đầu với Faces:**
- V và A rõ ràng nhất trong nhóm này (mặt người = cảm xúc trực quan)
- 80 chars = đủ để validate schema nhưng không quá tải
- Xong → có template cho mọi nhóm VA khác

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
UDC chars (58 blocks):         9,584  ← L0 KnowTree slots
KnowTree tổng slots:          65,536  (u16, 2^16)
KnowTree kích thước:           328 KB (65,536 × 5B)
Chain link size:                 2B   (u16)
Bootstrap: người encode tay → SEAL   (không compute từ emoji-data.txt)
Emoji trong UDC:             ~3,568  (EMOTICON group, E.01-E.17)
Data files có sẵn:           json/UnicodeData.txt, json/emoji/emoji-data.txt
```

---

*Plan v3 — Cập nhật 2026-03-20: Unicode 18.0 data files đã có trong ucd_source/,
emoji-data.txt là nguồn canonical cho EMOTICON group, 3 blocks mới cần thêm vào build.rs.*
