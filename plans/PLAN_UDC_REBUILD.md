# PLAN_UDC_REBUILD — Xây dựng lại UDC.md đúng chuẩn

> **Ngày tạo:** 2026-03-20
> **Cập nhật:** 2026-03-20 (v6 — P KHÔNG lưu trong JSON, P = output tính từ block+category+aliases qua ∫ₛ)
> **Tác giả:** Lara (AI session)
> **Branch:** `claude/lara-SBLZg`
> **Trạng thái:** 🟢 ACTIVE — schema chuẩn, có thể bắt đầu encode

---

## 0. TỔNG HỢP — Những điều đã clarify (session 2026-03-20)

### 0.1 Unicode IS the taxonomy

```
KHÔNG tự phân loại lại Unicode.
Unicode đã có sẵn: index (UTF-32), block, category, properties — dùng nguyên.

Hierarchy:
  Codepoint (U+1F525)
      └── Block ("Misc Symbols and Pictographs" 1F300..1F5FF)
           └── Category ("So" = Other Symbol)
                └── Property group ("Emoji_Presentation")

Điểm chung (cùng block / cùng category) → 1 node đại diện → Silk đến từng codepoint.
Không cần thêm tầng trung gian nào.
```

### 0.2 Ngôn ngữ quốc gia = vào block Unicode của ngôn ngữ đó

```
Vietnamese → Latin Extended (U+00C0..U+024F) — block này đã CÓ sẵn trong Unicode
Chinese    → CJK Unified Ideographs (U+4E00..U+9FFF)
Arabic     → Arabic block (U+0600..U+06FF)
Japanese   → Hiragana (U+3040..U+309F) + Katakana (U+30A0..U+30FF)

→ Dữ liệu ngôn ngữ tiếng Việt đi vào block node "Latin Extended"
  (không tạo node riêng "Vietnamese")
→ Alias tiếng Việt cho emoji/symbol → ghi tại codepoint đó trong JSON
```

### 0.3 Tại sao cần JSON (không dùng .txt trực tiếp)

```
UnicodeData.txt  = read-only, 1 entry/line, format cứng, không thể thêm field
Blocks.txt       = chỉ có range + tên block, không có P values, không có aliases

json/ucd.json    = structured, diff-able, AI + human có thể edit
                 = thêm được: aliases (vi/en/ja/zh), notes
                 = build.rs đọc JSON lúc compile → TÍNH P → sinh bảng tĩnh

Flow đúng:
  Unicode .txt ──(parse 1 lần)──► json/ucd.json (base: name, block, category, aliases)
                                       │
                              (human/AI thêm aliases vào)
                                       │
                              tool tính P từ block + category (∫ₛ)
                                       │
                              build.rs đọc → bảng tĩnh
```

### 0.4 Những cái cần chú ý

```
① Tên codepoint = từ UnicodeData.txt — KHÔNG đặt tên khác
   "FIRE" đúng, "lửa" là ALIAS, không phải tên chính

② Block node = đơn vị tổ chức cao nhất trong JSON
   Mỗi block có P_default (giá trị mặc định cho cả block)
   Codepoint individual override P_default nếu cần

③ Alias = nhiều chiều:
   - Alias ngôn ngữ (vi/en/ja/zh): "lửa" → U+1F525
   - Alias UTF-32 (char khác → emoji): U+2605 ★ → canonical U+2B50 ⭐
   - Alias text: "fire" → U+1F525

④ Script blocks (ngôn ngữ quốc gia):
   - KHÔNG cần encode P cho từng codepoint Latin/CJK/Arabic (quá nhiều)
   - Chỉ cần: alias map (từ tiếng Việt → codepoint Unicode hoặc emoji tương ứng)
   - VD: "buồn" → U+1F622 😢 (không phải encode từng ký tự "b","u","ồ","n")

⑤ build.rs hiện tại (v5):
   - Đang derive P tự động từ name patterns + block ranges (code logic)
   - Sau UDC rebuild: build.rs đọc json/ucd.json → TÍNH P từ block + category theo ∫ₛ
   - JSON KHÔNG chứa P values — P là output của tính toán, không phải input

⑥ Phân chia 3 loại entry trong JSON:
   - "blocks": block nodes (dimension dominant — input để tính P)
   - "codepoints": từng char với block + category + aliases (input để tính P)
   - "script_aliases": ngôn ngữ tự nhiên → codepoint mapping
```

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
P[gốc] = 1 công thức duy nhất (S, R, V, A, T)
       → sinh ra 3 loại node, tất cả đều có P riêng, SEALED vĩnh viễn:

  P[UDC]   = P của 9,584 chars trong 58 Unicode blocks (base layer)
               SDF chars   → S dominant (hình dạng rõ)
               MATH chars  → R dominant (quan hệ rõ)
               EMOTICON    → V/A dominant (cảm xúc rõ)
               MUSICAL     → T dominant (thời gian rõ)

  P[emoji] = P của emoji nodes (canonical — gốc để các alias kế thừa)
               Là tập con của P[UDC] (emoji thuộc EMOTICON group)
               Đây là "điểm neo" chuẩn — như 0°C và 100°C

  P[alias] = P của mọi alias (text, char alias, ngôn ngữ tự nhiên)
               Kế thừa từ P[emoji] canonical + override V/A nếu cần
               SEALED — không phải pointer lúc runtime

Tất cả sinh từ P[gốc] với input từ json/ucd.json.
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
Mỗi node (emoji hay alias) đều có P riêng — tính 1 lần từ json → SEAL vĩnh viễn.
Không có "lazy resolve" lúc runtime. Mọi P đã sẵn sàng từ bootstrap.

Flow tính P:
  json/ucd.json  →  công thức P  →  P[emoji]  → SEAL
                                 →  P[alias]  → SEAL

#emoji = canonical gốc:
  → P tính trực tiếp từ json (S, R, V, A, T đầy đủ)

alias = node thứ cấp:
  → P tính từ canonical P + override từ json
  → Kết quả = P riêng, SEAL — không trỏ runtime

Ví dụ:
  #🔥 (U+1F525)  json: { S=Sphere R=Causes V=0xC0 A=0xC0 T=Fast }
                 → P[🔥] = { S=Sphere R=Causes V=0xC0 A=0xC0 T=Fast }  SEALED

  ★ (U+2605)    json: { canonical=1F525, V=0xB0 }
                 → P[★] = { S=Sphere R=Causes V=0xB0 A=0xC0 T=Fast }   SEALED (kế thừa + override V)

  "lửa" (vi)    json: { canonical=1F525 }
                 → P[lửa] = { S=Sphere R=Causes V=0xC0 A=0xC0 T=Fast }  SEALED (copy đầy đủ)
```

### Flow xây dữ liệu (không phải parse từ txt):
```
KHÔNG derive tự động từ emoji-data.txt hay UnicodeData.txt.
Chúng ta XÂY thủ công JSON → đó là nguồn dữ liệu canonical.

Flow:
  Người → nhìn emoji → ghi vào UDC.md (draft)
  UDC.md → review + validate → ghi vào json/ucd.json 
  xây dựng công cụ mới chuyển mọi thứ thành P -> nạp giá trị từ json, tạo ra P thực tế.
  P → ucd crate đọc khi compile → build.rs nạp vào bảng tĩnh

Không có bước "parse emoji-data.txt để sinh P" — json/ucd.json là tri thức con người,
P-> tên emoji, codepoint range.
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

## 7. KẾ HOẠCH THỰC HIỆN — PHÂN VIỆC CHI TIẾT

> **Nguyên tắc phân việc:**
> - Mỗi task = 1 Unicode block group → 1 commit
> - Ưu tiên blocks có VA rõ ràng nhất (Emoticons) → dễ validate
> - Script aliases (ngôn ngữ) = task riêng, song song được
> - build.rs update = task cuối (sau khi JSON đủ dữ liệu)

---

### Task 0: Scaffold JSON + update build.rs để đọc JSON (1 session)

**Mục tiêu:** Tạo skeleton, wire build.rs đọc JSON trước khi điền data.

```
Files tạo/sửa:
  json/ucd.json          ← tạo mới, skeleton (meta + blocks rỗng)
  crates/ucd/build.rs    ← thêm: đọc json/ucd.json nếu tồn tại
                            Priority: json values > formula auto-derive
```

Checklist:
- [ ] Tạo `json/ucd.json` với `_meta` + `blocks: {}` + `codepoints: {}` + `script_aliases: {}` + `utf32_aliases: {}`
- [ ] `build.rs`: thêm `fn load_json_overrides()` → đọc `json/ucd.json`
- [ ] `build.rs`: nếu codepoint có trong JSON → dùng P từ JSON; nếu không → formula tự động
- [ ] Test: `cargo test -p ucd` vẫn pass
- [ ] Commit: `feat(ucd): scaffold json/ucd.json + build.rs JSON override`

---

### Task 1: Blocks metadata (1 session)

**Mục tiêu:** Điền `"blocks"` section — P_default cho 58 blocks.

```
Ưu tiên điền trước (theo dimension dominant):
  VA blocks (EMOTICON): 1F600-1F64F, 1F300-1F5FF, 2600-26FF, 1F900-1F9FF, 1FA70-1FAFF
  R blocks (MATH):      2200-22FF, 2A00-2AFF, 2980-29FF, 27C0-27EF
  S blocks (SDF):       25A0-25FF, 2500-257F, 2580-259F, 2190-21FF, 2B00-2BFF
  T blocks (MUSICAL):   1D100-1D1FF, 4DC0-4DFF, 1D300-1D35F
```

Checklist:
- [ ] Điền P_default cho 17 EMOTICON blocks (VA dominant)
- [ ] Điền P_default cho 21 MATH blocks (R dominant)
- [ ] Điền P_default cho 13 SDF blocks (S dominant)
- [ ] Điền P_default cho 7 MUSICAL blocks (T dominant)
- [ ] Commit: `docs(ucd): blocks metadata - P_default for 58 Unicode blocks`

---

### Task 2: Emoticons/Faces — U+1F600..U+1F64F (1 session, ~80 chars)

**Lý do bắt đầu ở đây:** VA rõ ràng nhất → dễ encode tay + dễ validate.

```
Quy trình:
  NHÌN từng emoji → HỎI: cảm giác gì? mạnh/yếu? nhanh/chậm?
  Ghi S, R, V, A, T → thêm aliases vi/en → SEAL
```

Sub-tasks:
- [ ] **2a** U+1F600..U+1F60F: 16 happy/laugh faces (V: 0xC0..0xFF)
      🎯 Anchor: 1F600 😀 V=0xE0 A=0xC0 T=Fast
- [ ] **2b** U+1F610..U+1F61F: 16 neutral/worried faces (V: 0x60..0x90)
      🎯 Anchor: 1F610 😐 V=0x80 A=0x20 T=Static
- [ ] **2c** U+1F620..U+1F62F: 16 angry/sad faces (V: 0x10..0x50)
      🎯 Anchor: 1F622 😢 V=0x30 A=0x60 T=Slow
- [ ] **2d** U+1F630..U+1F64F: 32 misc faces (fear/sick/gesture)
      🎯 Anchor: 1F631 😱 V=0x10 A=0xFF T=Instant
- [ ] Thêm `aliases.vi` + `aliases.en` cho mỗi emoji
- [ ] Commit: `docs(ucd): codepoints - E.09 Emoticons/Faces (1F600-1F64F)`

---

### Task 3: Misc Symbols — U+2600..U+26FF (1 session, ~100 chars quan trọng)

```
Nhóm con:
  2600..260F: thời tiết (☀☁☂☃☄)     → S dominant nhẹ, VA medium
  2610..261F: boxes/ballots (☐☑☒)    → S=Square
  2620..262F: hazard/skull (☠☢☣)     → V thấp
  2630..263F: yin-yang/stars (☯★)    → VA balanced
  2640..265F: chess/signs (♟♠♣)      → R dominant
  2660..266F: music notes (♩♪♫♬)    → T dominant!
  2680..26FF: dice/sports (⚀⚽⚾)    → A medium
```

Checklist:
- [ ] 2600..260F: weather (7 chars chính)
- [ ] 2620..262F: hazard/danger (V thấp)
- [ ] 2630..263F: yin-yang, stars
- [ ] 2660..266F: music notes → T dimension
- [ ] 2680..26FF: sports/dice
- [ ] Aliases vi + en
- [ ] Commit: `docs(ucd): codepoints - Misc Symbols (2600-26FF)`

---

### Task 4: Geometric Shapes — U+25A0..U+25FF (1 session, ~60 chars chính)

**8 SDF primitives nằm ở đây** — quan trọng nhất.

```
SDF Primitives (PHẢI encode chính xác):
  25CF ● BLACK CIRCLE   → S=Sphere   R=Member  V=0x40 A=0x40 T=Static
  25AC ▬ BLACK RECT     → S=Capsule  R=Member  V=0x40 A=0x40 T=Static
  25A0 ■ BLACK SQUARE   → S=Box      R=Subset  V=0x40 A=0x40 T=Static
  25B2 ▲ BLACK TRIANGLE → S=Cone     R=Causes  V=0x60 A=0x80 T=Fast
  25CB ○ WHITE CIRCLE   → S=Torus    R=Member  V=0x80 A=0x20 T=Static
  222A ∪ UNION          → S=Union    R=Member  V=0x80 A=0x40 T=Static
  2229 ∩ INTERSECTION   → S=Intersect R=Intersect V=0x80 A=0x40 T=Static
  2216 ∖ SET MINUS      → S=SetMinus R=SetMinus V=0x40 A=0x60 T=Fast
```

Checklist:
- [ ] Encode 8 SDF primitives chính xác
- [ ] Encode colored variants: 🔴🟠🟡🟢🔵🟣⚫⚪ (1F534..1F7E3 region)
- [ ] Encode squares/diamonds/triangles quan trọng
- [ ] Aliases vi + en
- [ ] Commit: `docs(ucd): codepoints - Geometric Shapes (25A0-25FF) + SDF primitives`

---

### Task 5: Math Operators — U+2200..U+22FF (1 session, ~50 chars quan trọng)

**8 Relation primitives nằm ở đây** — quan trọng nhất.

```
Relation Primitives (PHẢI encode chính xác):
  2208 ∈ ELEMENT OF    → R=Member      V=0x80 A=0x20 T=Static
  2282 ⊂ SUBSET OF     → R=Subset      V=0x80 A=0x20 T=Static
  2261 ≡ IDENTICAL TO  → R=Equiv       V=0x80 A=0x10 T=Static
  22A5 ⊥ UP TACK       → R=Orthogonal  V=0x80 A=0x20 T=Static
  2218 ∘ RING OP       → R=Compose     V=0x80 A=0x40 T=Medium
  2192 → RIGHTWARDS    → R=Causes      V=0x90 A=0x60 T=Fast
  2248 ≈ ALMOST EQUAL  → R=Approx     V=0x80 A=0x20 T=Slow
  2190 ← LEFTWARDS     → R=Inverse    V=0x70 A=0x40 T=Fast
```

Checklist:
- [ ] Encode 8 Relation primitives chính xác
- [ ] Encode các operators logic thường dùng: ∧∨¬∀∃
- [ ] Encode operators số học: ±×÷∞
- [ ] Aliases vi + en (tên toán học)
- [ ] Commit: `docs(ucd): codepoints - Mathematical Operators (2200-22FF) + Relation primitives`

---

### Task 6: Pictographs/Objects lớn — U+1F300..U+1F5FF (2-3 sessions, ~200 chars)

**Block lớn nhất** — chia sub-tasks:

- [ ] **6a** Nature/Weather (1F300..1F32F): 🌀🌊🌱🌺🌸 — ~50 chars
      Commit: `docs(ucd): codepoints - Nature/Weather (1F300-1F32F)`

- [ ] **6b** Sky/Space (1F330..1F37F): 🌍🌙⭐🌟 — ~30 chars
      Commit: `docs(ucd): codepoints - Sky/Space (1F330-1F37F)`

- [ ] **6c** Food/Drink (1F380..1F3FF): 🍎🍕🍺🎂 — ~60 chars
      Commit: `docs(ucd): codepoints - Food/Objects (1F380-1F3FF)`

- [ ] **6d** People/Body (1F440..1F4FF): 👁👂👃💪🦶 — ~40 chars
      Commit: `docs(ucd): codepoints - People/Body (1F440-1F4FF)`

- [ ] **6e** Office/Tools (1F500..1F5FF): 📱💡🔧🔑📦 — ~40 chars
      Commit: `docs(ucd): codepoints - Tools/Office (1F500-1F5FF)`

---

### Task 7: Transport + Activities — U+1F680..U+1F6FF (1 session)

```
1F680..1F6BF: vehicles (🚀🚂🚗🚢✈️)
1F6C0..1F6FF: signs/facilities (🚦🚫🛒)
```

- [ ] Encode ~40 chars chính
- [ ] Aliases vi + en
- [ ] Commit: `docs(ucd): codepoints - Transport/Signs (1F680-1F6FF)`

---

### Task 8: Supplemental Symbols — U+1F900..U+1FAFF (1 session)

```
1F900..1F9FF: 🤖🦁🥇🧠 (mới trong Unicode 9-13)
1FA70..1FAFF: 🪄🧬🦾 (mới nhất Unicode 12-14)
```

- [ ] Encode ~60 chars chính
- [ ] Aliases vi + en
- [ ] Commit: `docs(ucd): codepoints - Supplemental Symbols (1F900-1FAFF)`

---

### Task 9: Musical Symbols — U+1D100..U+1D1FF (1 session)

**T dimension canonical** — note duration = Time value.

```
T mapping từ note duration:
  Whole note    → T=Static(0)   (longest, held)
  Half note     → T=Slow(1)
  Quarter note  → T=Medium(2)   (beat = standard)
  Eighth note   → T=Fast(3)
  Sixteenth     → T=Instant(4)  (shortest)

Musical dynamics → A dimension:
  pp (pianissimo) → A=0x10
  p  (piano)      → A=0x40
  mf (mezzo)      → A=0x80
  f  (forte)      → A=0xC0
  ff (fortissimo) → A=0xFF
```

- [ ] Encode ~40 chars chính (notes + rests + dynamics)
- [ ] Aliases vi + en (tên nhạc lý)
- [ ] Commit: `docs(ucd): codepoints - Musical Symbols (1D100-1D1FF)`

---

### Task 10: Script Aliases — Ngôn ngữ tự nhiên (song song với Task 2-9)

**Không cần đợi Task 2-9 xong** — chỉ cần Task 0 (scaffold) xong.

- [ ] **10a** Vietnamese (`script_aliases.vi`):
      ~200 từ cảm xúc + hành động + vật thể thường dùng
      Commit: `docs(ucd): script_aliases - Vietnamese (vi)`

- [ ] **10b** English (`script_aliases.en`):
      ~200 từ tương ứng
      Commit: `docs(ucd): script_aliases - English (en)`

- [ ] **10c** UTF-32 aliases (`utf32_aliases`):
      Geometric shapes → emoji canonical
      Math operators → emoji tương ứng
      Commit: `docs(ucd): utf32_aliases - geometric + math mappings`

---

### Task 11: build.rs — đọc JSON thực sự (1 session)

**Sau khi Task 1-9 có đủ data** — wire hoàn chỉnh.

```
build.rs update:
  1. Parse json/ucd.json (serde_json hoặc tự parse đơn giản)
  2. Merge với UnicodeData.txt:
     - Nếu codepoint có P trong JSON → dùng JSON values (SEALED)
     - Nếu không có → formula tự động (fallback hiện tại)
  3. Thêm bảng mới vào ucd_generated.rs:
     ALIAS_TABLE    — text alias → chain_hash (binary search)
     BLOCK_TABLE    — block_range → P_default (lookup by cp)
```

Checklist:
- [ ] Viết `parse_ucd_json()` trong build.rs (~50 LOC)
- [ ] Merge P values: JSON priority > formula
- [ ] Sinh `ALIAS_TABLE: &[(&str, u64)]` (alias → hash)
- [ ] Test: verify 8 SDF primitives + 8 Relation primitives có P đúng
- [ ] `cargo test -p ucd` pass
- [ ] `make smoke-binary` pass
- [ ] Commit: `feat(ucd): build.rs reads json/ucd.json for P values`

---

### Task 12: lib.rs — API mới cho JSON layer (1 session)

```
Thêm vào crates/ucd/src/lib.rs:
  lookup_alias(text: &str) → Option<u64>   // "lửa" → chain_hash
  lookup_block(cp: u32) → Option<BlockEntry> // cp → block P_default
  alias_candidates(partial: &str) → Vec<&str> // autocomplete
```

Checklist:
- [ ] Implement `lookup_alias()`
- [ ] Implement `lookup_block()`
- [ ] 5 tests mới
- [ ] Commit: `feat(ucd): alias lookup + block lookup API`

---

### Thứ tự ưu tiên thực hiện:

```
Đợt 1 (BẮT BUỘC làm trước):
  Task 0 → scaffold JSON + build.rs wire
  Task 1 → blocks metadata (P_default cho 58 blocks)

Đợt 2 (Core anchors — cần sớm):
  Task 2 → Emoticons/Faces (VA anchors)
  Task 4 → Geometric Shapes (SDF primitives)
  Task 5 → Math Operators (Relation primitives)

Đợt 3 (Mở rộng — song song được):
  Task 3 → Misc Symbols
  Task 9 → Musical Symbols (T anchors)
  Task 10a → Vietnamese aliases

Đợt 4 (Large blocks — nhiều session):
  Task 6 (6a..6e) → Pictographs (chia nhỏ)
  Task 7 → Transport
  Task 8 → Supplemental

Đợt 5 (Wire hoàn chỉnh):
  Task 11 → build.rs đọc JSON
  Task 12 → API mới

Total estimate: ~15-20 sessions (mỗi task = 1 session)
```

---

## 8. OUTPUT CUỐI CÙNG + CẤU TRÚC JSON

```
docs/
  UDC.md          — P definition cho ~9,584 chars (draft, human-readable)

json/
  ucd.json        — canonical source: blocks + codepoints + language aliases
```

### JSON format đầy đủ (v5):

```json
{
  "_meta": {
    "version": "0.05",
    "unicode_version": "18.0",
    "source": "UnicodeData.txt + Blocks.txt + emoji-data.txt",
    "note": "P values = human-encoded, SEALED. Aliases = extensible."
  },

  "blocks": {
    "1F300-1F5FF": {
      "name": "Miscellaneous Symbols and Pictographs",
      "dimension": "VA",
      "note": "Largest EMOTICON block, ~768 chars"
    },
    "1F600-1F64F": {
      "name": "Emoticons",
      "dimension": "VA"
    },
    "2200-22FF": {
      "name": "Mathematical Operators",
      "dimension": "R",
      "note": "R dominant — 8 relation primitives nằm ở đây"
    },
    "25A0-25FF": {
      "name": "Geometric Shapes",
      "dimension": "S",
      "note": "S dominant — 8 SDF primitives nằm ở đây"
    },
    "1D100-1D1FF": {
      "name": "Musical Symbols",
      "dimension": "T",
      "note": "T dominant — note duration = Time dimension"
    }
  },

  "codepoints": {
    "1F525": {
      "block": "1F300-1F5FF",
      "category": "So",
      "aliases": {
        "vi": ["lửa", "ngọn lửa", "đám cháy"],
        "en": ["fire", "flame", "blaze"],
        "ja": ["火", "炎"],
        "zh": ["火", "火焰"]
      }
    },
    "1F622": {
      "block": "1F600-1F64F",
      "category": "So",
      "aliases": {
        "vi": ["khóc", "buồn", "nước mắt"],
        "en": ["crying", "sad", "tears"]
      }
    },
    "2208": {
      "block": "2200-22FF",
      "category": "Sm",
      "aliases": {
        "en": ["element of", "belongs to", "in"],
        "vi": ["thuộc", "là thành viên của"]
      }
    },
    "25CF": {
      "block": "25A0-25FF",
      "category": "So",
      "aliases": {
        "en": ["black circle", "filled circle", "bullet"],
        "vi": ["vòng tròn đen", "chấm tròn"]
      }
    }
  },

  "script_aliases": {
    "vi": {
      "_block": "00C0-024F",
      "_note": "Latin Extended — chứa toàn bộ ký tự có dấu tiếng Việt",
      "vui": "1F601",
      "buồn": "1F622",
      "tức": "1F621",
      "sợ": "1F628",
      "yêu": "2764",
      "lửa": "1F525",
      "nước": "1F4A7",
      "cây": "1F333",
      "nhà": "1F3E0",
      "tim": "2764",
      "nguy hiểm": "26A0",
      "chết": "1F480",
      "ánh sáng": "1F4A1",
      "âm nhạc": "1F3B5"
    },
    "en": {
      "_block": "0000-007F",
      "_note": "Basic Latin — ASCII range",
      "fire": "1F525",
      "sad": "1F622",
      "angry": "1F621",
      "scared": "1F628",
      "love": "2764",
      "water": "1F4A7",
      "tree": "1F333",
      "house": "1F3E0",
      "heart": "2764",
      "danger": "26A0",
      "death": "1F480",
      "light": "1F4A1",
      "music": "1F3B5"
    }
  },

  "utf32_aliases": {
    "2605": { "canonical": "2B50", "note": "BLACK STAR → WHITE MEDIUM STAR (brighter)" },
    "25A0": { "canonical": "1F7E5", "V_override": 128, "note": "BLACK SQUARE → neutral (no color)" },
    "2192": { "canonical": "27A1",  "note": "RIGHTWARDS ARROW → simpler arrow" }
  }
}
```

### Giải thích 4 sections:

```
"blocks"         → Block nodes: dimension dominant (input để tính P)
                   Silk tự động từ block node → mọi codepoint trong block

"codepoints"     → Từng char: block + category + aliases (input để tính P)
                   Chỉ cần encode những char QUAN TRỌNG (~500 anchor chars)
                   Phần còn lại dùng dimension của block

"script_aliases" → Natural language → codepoint mapping
                   "lửa" → "1F525" (không encode từng ký tự l,ử,a)
                   Mỗi ngôn ngữ có _block trỏ tới Unicode block của nó

"utf32_aliases"  → UTF-32 symbol → canonical emoji
                   ★ (2605) → ⭐ (2B50)
```

**Cấu trúc codepoint trong JSON:**
```
"1F525"  ──────────────────────────────── P[char] = L0 anchor (SEALED)
   │
   ├── block: "1F300-1F5FF"  ────────── P[block] = ∫ₛ cấp 2
   ├── category: "So"         ────────── xác định chiều dominant
   │
   └── aliases:
         en: [fire, flame, blaze]  ──────┐
         vi: [lửa, ngọn lửa, ...]  ──────┤── P[alias] = kế thừa P[char]
         ja: [火, 炎]               ──────┘   tự nhóm theo key ngôn ngữ
```

**P KHÔNG lưu trong JSON — P là output của tính toán:**
```
char  → category "So" + block "1F300-1F5FF"
     → ∫ₛ cấp 1 (char)
     → ∫ₛ cấp 2 (sub-group trong block)
     → ∫ₛ cấp 3 (block)
     → P[gốc] = (S, R, V, A, T)  ← SEAL

script_aliases["vi"]["lửa"] = "1F525"
     → P["lửa"] = P["1F525"]  ← SEAL (kế thừa)

utf32_aliases["2605"].canonical = "2B50"
     → P["2605"] = P["2B50"]  ← SEAL (kế thừa)
```

build.rs đọc JSON (block + category + aliases) → TÍNH P theo ∫ₛ → nạp vào bảng tĩnh lúc compile.

---

## 9. NGUYÊN TẮC BẤT BIẾN

```
① Chỉ có 1 công thức P duy nhất — tất cả nodes đều dùng cùng 1 công thức ∫ₛ
② Mọi node (codepoint + alias) đều có P riêng, SEALED vĩnh viễn sau bootstrap
③ Không có "lazy resolve" — P tính 1 lần từ block+category, không tính lại lúc runtime
④ JSON KHÔNG lưu P values — JSON chỉ chứa: block, category, aliases
⑤ P là OUTPUT của tính toán ∫ₛ từ block + category — KHÔNG phải input
⑥ Alias kế thừa P từ canonical codepoint — KHÔNG override bất kỳ chiều nào
⑦ ucd.json (block + category + aliases) → tool tính P → bảng tĩnh compile-time
⑧ KnowTree 65,536 = kích thước 1 branch — toàn cây lớn hơn nhiều
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

*Plan v6 — Cập nhật 2026-03-20: P KHÔNG lưu trong JSON. JSON chỉ có block + category + aliases.
P là output tính từ ∫ₛ (block → dimension dominant → compose rules). build.rs đọc JSON → tính P → SEAL.*
