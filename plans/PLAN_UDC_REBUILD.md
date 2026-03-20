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

### P đúng — DNA analogy:
```
DNA:     A — T — G — C — C — A — T...   (mỗi base = 1 nucleotide có danh tính riêng)
HomeOS:  ○{○{○{○{○{...}}}}}              (mỗi char = 1 node có tọa độ 5D riêng)
```

---

## 2. KIẾN TRÚC MỚI: Emoji = Canonical Nodes

### Nguyên lý:
```
EMOJI (1,447 chars)     = CANONICAL NODES   → mỗi emoji có P=(S,R,V,A,T) đầy đủ
UTF-32 chars            = ALIAS NODES       → trỏ vào emoji gần nhất về ngữ nghĩa
Text (ngôn ngữ tự nhiên) = ALIAS           → trỏ vào emoji/node

Ví dụ:
  🟥 RED SQUARE        → P = { S=Square R=Contains V=0xC0 A=0x80 T=Static }  ← node thật
  ■  BLACK SQUARE      → alias → 🟥 (cùng hình, khác màu/V)
  "hình vuông đỏ"      → alias → 🟥
  "red square"         → alias → 🟥
```

### Tại sao Emoji = root?
- Emoji có **ngữ nghĩa phổ quát** (cross-language, cross-culture)
- Emoji đã được Unicode gán ý nghĩa rõ ràng (tên, category, version)
- Emoji có **visual cue** trực tiếp → dễ assign V, A, S
- UTF-32 symbols (■ ∈ ♩) = biểu diễn trừu tượng → cần anchor vào emoji cụ thể

### Flow sinh JSON cuối:
```
emoji-data.txt
  → UDC.md (P cho từng emoji, nhóm theo semantic)
  → alias_map (UTF-32 → emoji codepoint)
  → ucd_utf32.json (mọi char, P trực tiếp hoặc qua alias)
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

## 4. PHÂN NHÓM EMOJI — Unicode 18.0 (3,966 fully-qualified)

> **Nguồn:** `ucd_source/emoji-data.txt` (Unicode 18.0, ngày 2026-01-30)
> **Số liệu thật:**
> - `Emoji_Presentation` base codepoints: **1,228** (single cp + ranges)
> - Fully-qualified sequences (từ emoji-test.txt): **3,966**
> - Tất cả qualified (incl. minimally, unqualified): **5,244**

Từ `ucd_source/emoji-data.txt` v18 — phân theo range codepoint:

```
Nhóm 0: ASCII/Latin emoji (U+0023..U+00AE)           — 5 ranges  ~14 cp
  # * 0-9 © ® — ký hiệu text thông thường có emoji variant

Nhóm 1: Geometric/Technical (U+2000..U+25FF)         — 21 ranges ~90 cp
  ⌚ ⌨ ⏏ ⏩ ⏰ ⚓ ⚡ — kỹ thuật + hình học

Nhóm 2: Misc Symbols (U+2600..U+26FF)                — 55 ranges ~180 cp
  ☀ ☁ ☔ ♈ ♥ ⚽ ⛄ — thời tiết, biểu tượng, thể thao

Nhóm 3: Dingbats/Arrows (U+2700..U+28FF)             — 24 ranges ~80 cp
  ✅ ✊ ✨ ❌ ➕ ➰ — dấu check, mũi tên, ký hiệu

Nhóm 4: Other BMP (U+3000..U+3FFF)                   — 9 ranges  ~30 cp
  Ⓜ ㊗ ㊙ — ký hiệu CJK enclosed

Nhóm 5: Mahjong/Cards (U+1F000..U+1F1FF)             — 7 ranges  ~50 cp
  🀄 🃏 🅰 🅱 🆎 — bài, mạt chược

Nhóm 6: Enclosed/Regional (U+1F200..U+1F2FF)         — 5 ranges  ~25 cp
  🈁 🈶 🉐 — ký hiệu Nhật Bản

Nhóm 7: Pictographs/Nature/Objects (U+1F300..U+1F5FF)— 120 ranges ~500 cp
  🌀 🌊 🌱 🍎 🎀 🎭 🏠 🐶 — lớn nhất, đa dạng nhất

Nhóm 8: Emoticons/Faces (U+1F600..U+1F64F)           — 31 ranges ~80 cp
  😀 😂 😍 😭 🙏 — biểu cảm khuôn mặt

Nhóm 9: Transport/Signs (U+1F680..U+1F8FF)           — 58 ranges ~200 cp
  🚀 🚗 🛒 🛸 — phương tiện, biển báo

Nhóm B: Supplemental (U+1F900..U+1F9FF)              — 45 ranges ~150 cp
  🤖 🤸 🤺 🥇 🦁 — người, động vật, biểu tượng mới

Nhóm C: Extended-A (U+1FA00..U+1FAFF)                — 44 ranges ~140 cp
  ♟ 🪄 🪅 🦾 🧬 — Chess (1FA00-1FA6F) + Pictographs (1FA70-1FAFF)
  ⚠️ MỚI: 1FA00..1FA6F (Chess Symbols) chưa có trong build.rs cũ

Nhóm D: Legacy Computing (U+1FB00..U+1FBFF) ← MỚI v18  — ~50 cp
  Block/terminal symbols, SDF-like geometric shapes
  ⚠️ Chưa có trong build.rs cũ
```

### Blocks mới trong v18 cần thêm vào build.rs GROUPS:
```
SDF group:
  1FA00..1FA6F   Chess Symbols (hình học: ♟ quân cờ, geometric)
  1FB00..1FBFF   Symbols for Legacy Computing (terminal blocks)

MUSICAL group:
  1D250..1D28F   Musical Symbols Supplement
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

### Phase 1: Nhóm 8 — Emoticons/Faces (1 session, ~80 chars)
**Lý do bắt đầu ở đây:** Faces = VA dominant → dễ nhất để assign V và A
- [ ] U+1F600..U+1F64F: 80 emoji khuôn mặt
- [ ] Commit: `docs: UDC.md - Emoticons group (1F600-1F64F)`

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

### Số liệu chuẩn Unicode 18.0:
```
Emoji_Presentation base codepoints: 1,228
Fully-qualified sequences:          3,966
Tất cả qualified:                   5,244
Unicode assigned codepoints tổng:  41,382 (từ UnicodeData.txt)
KnowTree key type cần dùng:        u32 (không phải u16)
Dung lượng toàn bộ ~41K entries:   ~1.4 MB static array (tối ưu)
```

---

*Plan v3 — Cập nhật 2026-03-20: Unicode 18.0 data files đã có trong ucd_source/,
emoji-data.txt là nguồn canonical cho EMOTICON group, 3 blocks mới cần thêm vào build.rs.*
