# PLAN_UDC_REBUILD — Xây dựng lại UDC.md đúng chuẩn

> **Ngày tạo:** 2026-03-20
> **Tác giả:** Lara (AI session)
> **Branch:** `claude/lara-SBLZg`
> **Trạng thái:** 🟡 DRAFT — cần review trước khi thực thi

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
HomeOS:  ○{○{○{○{○{...}}}}}              (mỗi ký tự = 1 node có tọa độ 5D riêng)
```

- Mỗi ký tự Unicode trong tập 9,584 = **1 nucleotide** của HomeOS
- P = (S, R, V, A, T) = **tọa độ 5D** của nucleotide đó trong không gian tri thức
- Block xác định **chiều chủ đạo** (dim nào có giá trị có nghĩa nhất)
- Char vẫn có GIÁ TRỊ trong cả 5 chiều (không chỉ 1)
- Chuỗi ký tự → chuỗi tọa độ → **encode khái niệm** (như codon → amino acid)

---

## 2. CẤU TRÚC P ĐÚNG

### 5 Chiều và nguồn gốc:

| Chiều | Kiểu | Range | Nguồn Unicode | Ý nghĩa |
|-------|------|-------|---------------|---------|
| **S** (Shape)    | enum  | 8 giá trị | Geometric Shapes, Box Drawing, Arrows... | "Trông như thế nào" — hình học |
| **R** (Relation) | enum  | 8 giá trị | Math Operators, Letterlike, Number Forms... | "Liên kết thế nào" — quan hệ toán học |
| **V** (Valence)  | u8    | 0x00..0xFF | Emoticons, Pictographs, Misc Symbols... | "Dương/âm" — cảm xúc |
| **A** (Arousal)  | u8    | 0x00..0xFF | Emoticons (presentation), Emoji... | "Mạnh/yếu" — cường độ |
| **T** (Time)     | enum  | 5 giá trị | Musical Symbols, Byzantine Music... | "Nhanh/chậm" — nhịp thời gian |

### Enum values:
```
S: Sphere(0) | Line(1) | Square(2) | Triangle(3) | Empty(4) | Union(5) | Intersect(6) | SetMinus(7)
R: Member(0) | Subset(1) | Equiv(2) | Orthogonal(3) | Compose(4) | Causes(5) | Approximate(6) | Inverse(7)
T: Static(0) | Slow(1) | Medium(2) | Fast(3) | Instant(4)
V: 0x00 (cực âm) → 0x80 (trung tính) → 0xFF (cực dương)
A: 0x00 (tĩnh lặng) → 0x80 (trung bình) → 0xFF (kích động mạnh)
```

### Olang syntax cho từng ký tự:
```olang
"BLACK_SQUARE" == { S=Square R=Contains V=128 A=128 T=Static }
"FIRE"         == { S=Sphere R=Causes  V=192 A=192 T=Fast   }
"PLUS_SIGN"    == { S=Line   R=Compose V=128 A=128 T=Medium }
```

---

## 3. PHÂN NHÓM 58 BLOCKS (Tổng quan)

Từ `tmp_P_tree.md` và `old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md`:

```
Nhóm S (Shape)   — 13 blocks — ~2,059 ký tự
  S.01  Geometric Shapes           U+25A0..U+25FF   (96 chars)
  S.02  Box Drawing                U+2500..U+257F   (128 chars)
  S.03  Block Elements             U+2580..U+259F   (32 chars)
  S.04  Miscellaneous Technical    U+2300..U+23FF   (256 chars)
  S.05  Arrows                     U+2190..U+21FF   (112 chars)
  S.06  Supplemental Arrows-A      U+27F0..U+27FF   (16 chars)
  S.07  Supplemental Arrows-B      U+2900..U+297F   (128 chars)
  S.08  Supplemental Arrows-C      U+1F800..U+1F8FF (256 chars)
  S.09  Miscellaneous Symbols      U+2600..U+26FF   (256 chars)
  S.10  Dingbats                   U+2700..U+27BF   (192 chars)
  S.11  Braille Patterns           U+2800..U+28FF   (256 chars)
  S.12  Geometric Shapes Extended  U+1F780..U+1F7FF (128 chars)
  S.13  Supplemental Symbols       (TBD từ data)

Nhóm R (Relation) — 20 blocks — ~2,824 ký tự
  R.01  Superscripts & Subscripts  U+2070..U+209F   (48 chars)
  R.02  Letterlike Symbols         U+2100..U+214F   (80 chars)
  R.03  Number Forms               U+2150..U+218F   (64 chars)
  R.04  Mathematical Operators     U+2200..U+22FF   (256 chars)
  R.05  Supplemental Math Ops      U+2A00..U+2AFF   (256 chars)
  R.06  Miscellaneous Math A       U+27C0..U+27EF   (48 chars)
  R.07  Miscellaneous Math B       U+2980..U+29FF   (128 chars)
  R.08  Math Alphanumeric Symbols  U+1D400..U+1D7FF (988 chars)
  R.09  Enclosed Alphanumerics     U+2460..U+24FF   (160 chars)
  R.10  Enclosed Alphanumerics Ext U+1F100..U+1F1FF (256 chars)
  ...   (10 blocks nữa từ ancient number systems)

Nhóm VA (Valence + Arousal) — 14 blocks — ~2,489 ký tự
  VA.01 Emoticons (Emoji Faces)    U+1F600..U+1F64F (80 chars)
  VA.02 Misc Symbols & Pictographs U+1F300..U+1F5FF (768 chars)
  VA.03 Supplemental Symbols      U+1F900..U+1F9FF (256 chars)
  VA.04 Symbols & Pictographs ExtA U+1FA00..U+1FA6F (112 chars)
  VA.05 Symbols & Pictographs ExtB U+1FA70..U+1FAFF (144 chars)
  VA.06 Mahjong Tiles              U+1F004..U+1F02F (44 chars)
  VA.07 Domino Tiles               U+1F030..U+1F09F (112 chars)
  VA.08 Playing Cards              U+1F0A0..U+1F0FF (96 chars)
  VA.09 Enclosed Ideographic Suppl U+1F200..U+1F2FF (256 chars)
  ...   (5 blocks nữa)

Nhóm T (Time) — ~11 blocks — ~2,212 ký tự
  T.01  Musical Symbols            U+1D100..U+1D1FF (256 chars)
  T.02  Byzantine Musical Symbols  U+1D000..U+1D0FF (256 chars)
  T.03  Ancient Greek Music        (subset của Misc Technical)
  ...   (8 blocks nữa — cần xác định từ data)
```

> ⚠️ Số liệu trên là ước tính. **File thật** để xác định: `json/Blocks.txt` + `json/UCD_18_INDEX.md`

---

## 4. SCHEMA UDC.md — Template

Mỗi section trong `UDC.md` theo format:

```markdown
## [Group].[Block_ID] — [Block Name]
**Unicode range:** U+XXXX..U+YYYY
**Chiều chủ đạo:** [S | R | V+A | T]
**Số ký tự:** N

### Formula chung (Block level):
```olang
P_[block_id] = { S=[val] R=[val] V=[val] A=[val] T=[val] }
```
> Mọi char trong block NẾU KHÔNG có quy tắc riêng → thừa kế formula này.

---

### [Group].[Block_ID].[Sub_ID] — [Sub-range name]
**Range:** U+XXXX..U+XXYY

#### Formula sub:
```olang
P_[sub_id] = { S=[val] R=[val] V=[val] A=[val] T=[val] }
```

---

#### [Group].[Block_ID].[Sub_ID].[Con_ID] — [Sub-category name]

```olang
# U+XXXX [CHAR_NAME]
"[CHAR_NAME]" == { S=[val] R=[val] V=[val] A=[val] T=[val] }

# U+XXXXY [CHAR_NAME_2]
"[CHAR_NAME_2]" == { S=[val] R=[val] V=[val] A=[val] T=[val] }
```
```

---

## 5. QUY TẮC GÁN GIÁ TRỊ P

### S (Shape) — Từ hình học ký tự:
```
Sphere(0)    → ký tự tròn: ●○◉◎⊙
Line(1)      → ký tự thẳng/sóng: ─│╌╍┄┅ hoặc dấu toán học ± −
Square(2)    → ký tự vuông: ■□▪▫▬
Triangle(3)  → ký tự tam giác: ▲▼◀▶▴▾
Empty(4)     → ký tự rỗng/khoảng: ◌ (combining placeholder)
Union(5)     → ký tự gộp/mở rộng: ∪ ∨ + ⋃
Intersect(6) → ký tự giao/thu hẹp: ∩ ∧ × ⋂
SetMinus(7)  → ký tự loại trừ: ∖ ÷ / −
```

### R (Relation) — Từ ý nghĩa toán học:
```
Member(0)      → ký tự "thuộc về": ∈ ∉ ∋ ∌
Subset(1)      → ký tự "bao hàm": ⊂ ⊃ ⊆ ⊇
Equiv(2)       → ký tự "tương đương": ≡ ≈ ≅ ↔ =
Orthogonal(3)  → ký tự "vuông góc/độc lập": ⊥ ∥
Compose(4)     → ký tự "ghép/kết hợp": ∘ ∗ ∙ ·
Causes(5)      → ký tự "gây ra/dẫn tới": → ⟶ ⇒ ↦
Approximate(6) → ký tự "xấp xỉ/gần": ≈ ∼ ≃
Inverse(7)     → ký tự "ngược lại": ← ⟵ ⇐ ↩
```

### V (Valence) — Thang 0x00..0xFF:
```
0x00..0x3F  → âm tính mạnh (buồn, nguy hiểm, xấu): ☠💀😢
0x40..0x7F  → âm tính nhẹ (bình thường/tối): ▪■◼
0x80        → trung tính: ○●□■ (geometric neutral)
0x81..0xBF  → dương tính nhẹ (tích cực nhẹ): ✓☑✅
0xC0..0xFF  → dương tính mạnh (vui, tốt, sáng): 😊🌟✨💛
```

### A (Arousal) — Thang 0x00..0xFF:
```
0x00..0x3F  → rất tĩnh lặng: ─ (dashes, quiet symbols)
0x40..0x7F  → tĩnh: □ ○ (neutral geometric)
0x80        → trung bình: phần lớn ký tự toán học
0x81..0xBF  → kích thích vừa: ⚡ ⚠ ! ?
0xC0..0xFF  → kích thích mạnh: 🔥 💥 ‼ 🚨
```

### T (Time) — Từ ký tự:
```
Static(0)   → ký tự hình học/toán học không biến đổi: ■ ○ ∈ ≡
Slow(1)     → ký tự âm nhạc chậm: 𝅗𝅥 (half note, whole note)
Medium(2)   → ký tự âm nhạc trung bình: ♩ ♪ (quarter note)
Fast(3)     → ký tự âm nhạc nhanh: ♫ ♬ (eighth notes)
Instant(4)  → ký tự kích hoạt tức thì: ⚡ ☇ (lightning)
```

---

## 6. KẾ HOẠCH THỰC HIỆN — Chia nhỏ theo Block

### Phase 0: Nền tảng (1 session)
- [ ] Tạo file `UDC.md` với header + schema template
- [ ] Viết phần "Giải thích P đúng" (khác gì Olang hiện tại)
- [ ] Viết bảng 5 chiều đầy đủ + quy tắc gán
- [ ] Commit: `docs: create UDC.md skeleton with P schema`

### Phase 1: Nhóm S — Shape (13 sessions, 1 block/session)
- [ ] S.01 Geometric Shapes (U+25A0..U+25FF) — 96 chars
- [ ] S.02 Box Drawing (U+2500..U+257F) — 128 chars
- [ ] S.03 Block Elements (U+2580..U+259F) — 32 chars
- [ ] S.04 Miscellaneous Technical (U+2300..U+23FF) — 256 chars
- [ ] S.05 Arrows (U+2190..U+21FF) — 112 chars
- [ ] S.06..S.13 — các block còn lại

### Phase 2: Nhóm R — Relation (20 sessions)
- [ ] R.01 Superscripts & Subscripts
- [ ] R.02 Letterlike Symbols
- [ ] R.03 Number Forms
- [ ] R.04 Mathematical Operators (lớn nhất — 256 chars)
- [ ] R.05..R.20 — các block còn lại

### Phase 3: Nhóm VA — Valence+Arousal (14 sessions)
- [ ] VA.01 Emoticons (U+1F600..U+1F64F)
- [ ] VA.02 Misc Symbols & Pictographs (lớn nhất — 768 chars)
- [ ] VA.03..VA.14 — các block còn lại

### Phase 4: Nhóm T — Time (11 sessions)
- [ ] T.01 Musical Symbols (U+1D100..U+1D1FF)
- [ ] T.02 Byzantine Musical Symbols
- [ ] T.03..T.11 — các block còn lại

### Phase 5: Sinh JSON
- [ ] Script đọc UDC.md → sinh `ucd_utf32.json` (codepoint → P object)
- [ ] Verify: 9,584 entries, mỗi entry có đủ 5 trường

---

## 7. OUTPUT CUỐI CÙNG

### File `UDC.md`:
```
docs/UDC.md
  ├── Giải thích P đúng
  ├── Bảng 5 chiều
  ├── Quy tắc gán
  ├── S.01..S.13 (với từng char)
  ├── R.01..R.20
  ├── VA.01..VA.14
  └── T.01..T.11
```

### File `ucd_utf32.json`:
```json
{
  "0x25A0": { "name": "BLACK_SQUARE", "S": 2, "R": 0, "V": 128, "A": 128, "T": 0 },
  "0x1F525": { "name": "FIRE",        "S": 0, "R": 5, "V": 192, "A": 192, "T": 3 },
  ...
}
```

---

## 8. NGUYÊN TẮC BẤT BIẾN (không phá vỡ)

```
① Mọi giá trị P phải có LÝ DO từ Unicode character properties
   → Không đặt V=200 cho ■ mà không có nguồn gốc từ UCD/emoji-data

② Block formula = lower bound, char formula = upper bound
   → Char KHÔNG được có P khác hoàn toàn so với block cha

③ Mỗi char có DUY NHẤT 1 dòng Olang
   → "CHAR_NAME" == { S=X R=Y V=Z A=W T=Q }
   → Không có 2 dòng cho cùng 1 codepoint

④ Tên char = tên từ Unicode (Unicode Character Name)
   → KHÔNG đặt tên tùy ý
   → "BLACK SQUARE" không phải "filled square" hay "dark square"

⑤ Olang dùng tên enum, không dùng số
   → { S=Square ... } ĐÚNG
   → { S=2 ... } SAI (dùng trong JSON, không dùng trong Olang source)
```

---

## 9. BẮT ĐẦU: Session đầu tiên làm gì?

**Thứ tự thực hiện ngay trong session này:**

1. Tạo `docs/UDC.md` với Phase 0 (skeleton + schema)
2. Điền Block **S.01 Geometric Shapes** (U+25A0..U+25FF) — 96 ký tự
3. Dùng `json/Index.txt` để tra tên chính xác từng char
4. Commit + push → kết thúc session

**Lý do chỉ làm S.01 trước:**
- 96 chars = vừa đủ để validate schema
- Geometric Shapes = trực quan nhất, dễ assign S values
- Xong S.01 → có template để làm nhanh các block sau

---

## 10. CÁC FILE THAM KHẢO

| File | Dùng để |
|------|---------|
| `json/Blocks.txt` | Xác định range hex chính xác của từng block |
| `json/Index.txt` | Tra tên Unicode chính xác của từng codepoint |
| `json/emoji/emoji-data.txt` | Xác định V và A cho emoji |
| `tmp_P_tree.md` | Cấu trúc cây đã có — dùng làm skeleton |
| `old/HomeOS_SINH_HOC_PHAN_TU_TRI_THUC_v2.md` | Triết lý P đúng — reference |

---

*Plan này cần được review và approve trước khi thực thi Phase 0.*
