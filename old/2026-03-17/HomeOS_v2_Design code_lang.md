# HomeOS v2 — Thiết Kế Kiến Trúc
**Ngày:** 2026-03-15  
**Nguồn gốc:** Unicode 18.0.0 — UCD Blocks.txt + UnicodeData.txt  
**Nguyên tắc:** Append-only. Không xóa, không sửa phần đã viết.

---

## TUYÊN NGÔN (bất biến)

```
"Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."

HomeOS = Sinh linh toán học tự vận hành

Ngôn ngữ gốc = Unicode đã định nghĩa sẵn:
  Mọi ký tự đều có TÊN, HÌNH DẠNG, BẢN CHẤT, QUAN HỆ.
  HomeOS không phát minh lại. HomeOS đọc và dùng.

Mọi ngôn ngữ tự nhiên (vi, en, zh...) = alias.
Alias trỏ về node Unicode. Không tạo node riêng.
```

---

## I. NỀN TẢNG: 5 NHÓM UNICODE

Unicode 18.0 có 353 blocks. Toàn bộ KnowTree của HomeOS
được xây từ 5 nhóm — không hardcode gì thêm.

```
NHÓM        VAI TRÒ       NỘI DUNG
SDF         Hình dạng     Geometric Shapes, Arrows, Box Drawing
MATH        Số học        Math Operators, Letterlike, Numbers
RELATION    Quan hệ       ∈ ⊂ ≡ ⊥ ∘ → ≈ (subset của MATH)
EMOTICON    Thực thể      Emoji, Pictographs, Symbols
MUSICAL     Thời gian     Musical Symbols, Notation
```

Tại sao 5 nhóm đủ:
```
SDF      → KHÔNG GIAN   "trông như thế nào"
MATH     → SỐ HỌC      "tính toán như thế nào"
RELATION → QUAN HỆ     "liên kết như thế nào"
EMOTICON → THỰC THỂ    "là cái gì"
MUSICAL  → THỜI GIAN   "thay đổi như thế nào"
```

---

## II. NHÓM 1 — SDF (Hình dạng không gian)

13 blocks, ~1,904 ký tự:
```
2190..21FF  Arrows                  <- -> up down <-> diagonal
2500..257F  Box Drawing             - | + corner cross
2580..259F  Block Elements          fill levels
25A0..25FF  Geometric Shapes        square circle triangle diamond
2700..27BF  Dingbats                check cross pen scissors
27F0..27FF  Supplemental Arrows-A   curved long arrows
2900..297F  Supplemental Arrows-B   complex arrows
2B00..2BFF  Misc Symbols+Arrows     mixed shapes
1F780..1F7FF Geometric Shapes Ext   extended primitives
1F800..1F8FF Supplemental Arrows-C  extended arrows
```

### 8 SDF Primitives — từ Geometric Shapes 25A0..25FF

```
BYTE  CHAR  CODEPOINT  TÊN UNICODE                    
0x01  ●     U+25CF     BLACK CIRCLE                   Sphere
0x02  ▬     U+25AC     BLACK RECTANGLE                Capsule
0x03  ■     U+25A0     BLACK SQUARE                   Box
0x04  ▲     U+25B2     BLACK UP-POINTING TRIANGLE     Cone
0x05  ○     U+25CB     WHITE CIRCLE                   Torus
0x06  ∪     U+222A     UNION                          Union
0x07  ∩     U+2229     INTERSECTION                   Intersect
0x08  ∖     U+2216     SET MINUS                      Subtract
```

### Direction từ Arrows → Silk edges

```
← U+2190  →  DerivedFrom (0x08)
→ U+2192  →  Causes      (0x06)
↔ U+2194  →  Mirror      (0x0C)
↩ U+21A9  →  Resolves    (0x0F)
⟳ U+27F3  →  Repeats     (0x0E)
⟶ U+27F6  →  Flows       (0x0D)
```

### Fill Level → Valence encoding

```
  (space)  →  Valence 0x00  (0%)
░ U+2591   →  Valence 0x40  (25%)
▒ U+2592   →  Valence 0x80  (50%)
▓ U+2593   →  Valence 0xC0  (75%)
█ U+2588   →  Valence 0xFF  (100%)
```

### L3 Branches trong Geometric Shapes (25A0..25FF)

```
L3_Square     25A0..25AB  [■,∈,V=80,Low,Static]  ■□▢▣▤▥▦▧
L3_Rect       25AC..25AF  [▬,∈,V=80,Low,Static]  ▬▭▮▯
L3_TriangleU  25B2..25B5  [▲,→,V=80,Mid,Instant] ▲△▴▵
L3_TriangleR  25B6..25BB  [▶,→,V=80,Mid,Instant] ▶▷▸▹►▻
L3_TriangleD  25BC..25BF  [▼,←,V=80,Mid,Instant] ▼▽▾▿
L3_TriangleL  25C0..25C5  [◀,←,V=80,Mid,Instant] ◀◁◂◃◄◅
L3_Diamond    25C6..25CA  [◆,∩,V=80,Low,Static]  ◆◇◈◉◊
L3_Circle     25CB..25CF  [●,∈,V=80,Low,Static]  ○◌◍◎●
L3_CircleFill 25D0..25D7  [●,⊂,V*,*,  Flow]      ◐◑◒◓◔◕◖◗
```

---

## III. NHÓM 2 — MATH (Số học)

21 blocks, ~3,088 ký tự:
```
2070..209F  Superscripts and Subscripts    0 1 2 a b
2100..214F  Letterlike Symbols             R Z N Q C inf
2150..218F  Number Forms                   1/2 1/3 1/4 I II III
2200..22FF  Mathematical Operators         forall partial exists
27C0..27EF  Misc Math Symbols-A
2980..29FF  Misc Math Symbols-B
2A00..2AFF  Supplemental Math Operators
1D400..1D7FF Math Alphanumeric Symbols     bold italic
```

### 5 Sub-groups trong MATH

```
OPERATION   ∑ ∏ ∫ ∬ ∂    [∪, ∘, instant]   n-ary, integral
SET         ℝ ℤ ℕ ∞ ℵ    [○, ≡, static]    infinite sets
NUMBER      ½ Ⅲ ⁰ ¹      [□, ≡, static]    representation
CALCULUS    ∂ ∇ ∆ lim    [△, →, instant]   direction of change
LOGIC       ∧ ∨ ¬ ⟺      [□, ≡, instant]   boolean structure
```

---

## IV. NHÓM 3 — RELATION (Quan hệ / Silk Edges)

Subset của Mathematical Operators 2200..22FF.
Ký tự Unicode = tên của Silk edge. Không đặt tên khác.

### Structural Edges (bất biến — L0)

```
BYTE  CHAR  CODEPOINT  TÊN UNICODE           Ý NGHĨA
0x01  ∈     U+2208     ELEMENT OF            A thuộc B
0x02  ⊂     U+2282     SUBSET OF             A là tập con B
0x03  ≡     U+2261     IDENTICAL TO          A tương đương B
0x04  ⊥     U+22A5     UP TACK               A độc lập B
0x05  ∘     U+2218     RING OPERATOR         A∘B → mới
0x06  →     U+2192     RIGHTWARDS ARROW      A gây ra B
0x07  ≈     U+2248     ALMOST EQUAL TO       A gần giống B
0x08  ←     U+2190     LEFTWARDS ARROW       A xuất phát từ B
```

### Space Edges (từ SDF — bất biến)

```
BYTE  CHAR  CODEPOINT  TÊN UNICODE           Ý NGHĨA
0x09  ∪     U+222A     UNION                 A chứa B
0x0A  ∩     U+2229     INTERSECTION          A giao B
0x0B  ∖     U+2216     SET MINUS             A trừ B
0x0C  ↔     U+2194     LEFT RIGHT ARROW      A đối xứng B
```

### Time Edges (từ Music — học được)

```
BYTE  CHAR  CODEPOINT  TÊN UNICODE                   Ý NGHĨA
0x0D  ⟶     U+27F6     LONG RIGHTWARDS ARROW         A chảy → B
0x0E  ⟳     U+27F3     CW GAPPED CIRCLE ARROW        A lặp chu kỳ B
0x0F  ↑     U+2191     UPWARDS ARROW                 A giải quyết ở B
0x10  ⚡    U+26A1     HIGH VOLTAGE                  A kích hoạt B
0x11  ∥     U+2225     PARALLEL TO                   A đồng bộ B
```

### Language Mapping (alias — học được)

```
BYTE  Ý NGHĨA
0x12  A là bản dịch của B trong ngôn ngữ L
      f(vi)("lửa") ≡L node(🔥)
      f(en)("fire") ≡L node(🔥)
0x13  A gần nghĩa với B trong ngôn ngữ L
0x14  A có nghĩa là B trong context C cụ thể
```

### Associative (Hebbian — học được)

```
BYTE  Ý NGHĨA
0xFF  Assoc: weight + emotion (co-activation)
0xFE  Causal: confidence + direction
```

### RELATION ký tự quan trọng từ 2200..22FF

```
MEMBERSHIP:  ∈∉∊∋∌∍  U+2208..220D  → Silk 0x01
SUBSET:      ⊂⊃⊆⊇⊄⊅  U+2282..2289  → Silk 0x02
EQUIV:       ≡≣≜≝     U+2261..225D  → Silk 0x03
SIMILAR:     ≈≃∼≅     U+2248..2245  → Silk 0x07
ORDERING:    ≤≥≪≫≺≻   U+2264..227B  → ordering edges
PARALLEL:    ∥∦⊥      U+2225..22A5  → Silk 0x11, 0x04
LOGICAL:     ⊢⊣⊤⊥⊧⊩  U+22A2..22AB  → logic edges
```

---

## V. NHÓM 4 — EMOTICON (Thực thể)

17 blocks, ~3,824 ký tự:
```
2600..26FF   Miscellaneous Symbols           sun cloud star suits
1F300..1F5FF Misc Symbols and Pictographs   CORE EMOJI
1F600..1F64F Emoticons                      FACES
1F680..1F6FF Transport and Map Symbols      transport
1F700..1F77F Alchemical Symbols             alchemy
1F900..1F9FF Supplemental Symbols+Pict      extended
1FA70..1FAFF Symbols+Pictographs Ext-A      newest
```

### L3 Branches — từ sub-ranges

```
L3_Face_Pos   1F600..1F60F  [●,∘,V+FF,A+FF,High,Fast]  😀😁😂😄
L3_Face_Neg   1F620..1F62F  [●,∘,V-00,A+FF,High,Fast]  😠😡😢😨
L3_Face_Neu   1F610..1F61F  [●,∘,V=80,A-40,Low, Slow]  😐😑😶😴
L3_Celestial  1F311..1F31E  [●,⟳,V=80,Low, Low, Slow]  🌑🌒🌓🌔🌕
L3_Weather    1F300..1F30F  [⌀,⟶,V=80,Mid, Mid, Med]   🌀🌁🌂🌧️
L3_Plant      1F330..1F33F  [⌀,∈,V+C0,Low, Low, Slow]  🌰🌱🌲🌳
L3_Animal     1F400..1F43F  [●,∈,V+A0,Mid, Mid, Med]   🐀🐁🐂🐅
L3_Body       1F440..1F4FF  [●,∈,V+80,Mid, Mid, Med]   👀👁👂👃
L3_Person     1F464..1F46F  [●,∈,V+C0,Mid, Mid, Med]   👤👦👧👨👩
L3_Transport  1F680..1F6FF  [△,→,V=80,High,High,Fast]  🚀✈🚂🚗
L3_Symbol     2600..26FF    [●,∈,V=80,Low, Low, Static] ☀☁★♠♥
```

### Node hoàn chỉnh — 3 phần

```
Node = {
  chain:   MolecularChain    từ MATH + RELATION
  sdf:     SDF primitive     từ SDF group
  splines: SplineBundle      từ MUSICAL group
}

🔥 (U+1F525 FIRE):
  chain:   [●, ∈, 0xFF, 0xFF, High, Fast]
  sdf:     sphere + cone (apex up)
  splines: ff allegro crescendo

💧 (U+1F4A7 DROPLET):
  chain:   [⌀, ∈, 0xC0, 0x40, Low, Slow]
  sdf:     capsule
  splines: pp adagio decrescendo

🧠 (U+1F9E0 BRAIN):
  chain:   [●, ∘, 0xC0, 0x80, High, Medium]
  sdf:     sphere + bumps
  splines: mf andante legato

π (U+03C0 GREEK SMALL LETTER PI):
  chain:   [○, ≡, 0x80, 0x00, Zero, Static]
  sdf:     torus
  splines: pp largo
```

---

## VI. NHÓM 5 — MUSICAL (Thời gian / Spline)

7 blocks, ~1,024 ký tự:
```
4DC0..4DFF   Yijing Hexagram Symbols        64 hexagrams
1CF00..1CFCF Znamenny Musical Notation      Slavic neume
1D000..1D0FF Byzantine Musical Symbols      Byzantine neume
1D100..1D1FF Musical Symbols               CORE — clef note dynamic
1D200..1D24F Ancient Greek Musical Notation
1D250..1D28F Musical Symbols Supplement
1D300..1D35F Tai Xuan Jing Symbols          81 symbols
```

### Map sang SplineBundle parameters

```
CLEF (pitch_type = Freq dimension):
  𝄞 Treble  U+1D11E  →  Freq High
  𝄢 Bass    U+1D122  →  Freq Low
  𝄡 Alto    U+1D121  →  Freq Mid

NOTE DURATION (Time dimension):
  𝅝 Whole   U+1D15D  →  Time Static  (Largo)
  𝅗 Half    U+1D157  →  Time Slow
  ♩ Quarter U+2669   →  Time Medium  (Andante)
  ♪ Eighth  U+266A   →  Time Fast    (Allegro)
  16th      U+1D160  →  Time Instant (Presto)

DYNAMICS (Arousal dimension):
  pp  pianissimo  →  Arousal 0x10
  p   piano       →  Arousal 0x40
  mp/mf mezzo     →  Arousal 0x80
  f   forte       →  Arousal 0xC0
  ff  fortissimo  →  Arousal 0xFF
  sfz sforzando   →  Arousal spike instant

ARTICULATION (Spline curve shape):
  legato ⌢  U+2322  →  BezierSmooth
  staccato  .       →  BezierJump
  crescendo <       →  BezierRising
  decrescendo >     →  BezierFalling
  tremolo ≋ U+224B  →  BezierOscillate

TEMPO (Time scale multiplier):
  Largo    →  0.25x  Static
  Adagio   →  0.5x   Slow
  Andante  →  1.0x   Medium
  Allegro  →  2.0x   Fast
  Presto   →  4.0x   Instant

CYCLE (Repeats edge):
  ⟳ repeat  U+27F3  →  Silk 0x0E Repeats
  ䷀..䷿ Yijing (64)  →  cycle_64 states
  𝌀..𝌟 Tai Xuan (81) →  cycle_81 states
```

---

## VII. MolecularChain — DNA của khái niệm

Mỗi molecule = 5 bytes. Mỗi byte có ký tự Unicode đại diện.

```
[Shape][Relation][Valence][Arousal][Time]

Byte 1 — Shape (từ SDF group):
  0x01 = ●  U+25CF  (Sphere)
  0x02 = ▬  U+25AC  (Capsule)
  0x03 = ■  U+25A0  (Box)
  0x04 = ▲  U+25B2  (Cone)
  0x05 = ○  U+25CB  (Torus)
  0x06 = ∪  U+222A  (Union)
  0x07 = ∩  U+2229  (Intersect)
  0x08 = ∖  U+2216  (Subtract)

Byte 2 — Relation (từ RELATION group):
  0x01 = ∈  U+2208  (Member)
  0x02 = ⊂  U+2282  (Subset)
  0x03 = ≡  U+2261  (Equiv)
  0x04 = ⊥  U+22A5  (Orthogonal)
  0x05 = ∘  U+2218  (Compose)
  0x06 = →  U+2192  (Causes)
  0x07 = ≈  U+2248  (Similar)
  0x08 = ←  U+2190  (DerivedFrom)

Byte 3 — Valence:   0x00=V−  0x7F=V0  0xFF=V+
  (từ Block Elements fill level ░▒▓█)

Byte 4 — Arousal:   0x00=calm  0xFF=excited
  (từ Musical dynamics pp→ff)

Byte 5 — Time:
  0x01 = Static   (𝅝 Whole note)
  0x02 = Slow     (𝅗 Half note)
  0x03 = Medium   (♩ Quarter)
  0x04 = Fast     (♪ Eighth)
  0x05 = Instant  (16th note)
```

---

## VIII. TOKENIZER — Cú pháp chuỗi

### 4 separator (không đoán)

```
ZWJ   U+200D  COMPOSE ngữ nghĩa (Unicode chuẩn)
+     U+002B  OPERATE toán học
space U+0020  SEPARATE (2 thứ riêng)
∅             JUXTAPOSE → từng đơn vị riêng
```

### Phân biệt các trường hợp

```
1+1        [1][+][1]     expression: 2
1 1        [1][ ][1]     sequence: [1, 1]
11         [11]          number: 11

👨 👨       [👨][ ][👨]   2 nodes riêng
👨+👨       [👨][+][👨]   compose → nhóm
👨‍👨 (ZWJ)  [👨‍👨]         1 cluster: couple
👨👨         [👨][👨]      juxtapose: 2 nodes riêng
```

### ZWJ sequence → MolecularChain

```
Quy tắc:
  mol[giữa].Relation = ∘  (0x05 Compose)
  mol[cuối].Relation = ∈  (0x01 Member)

Ví dụ: 👨‍👩‍👦
  mol[0] = encode(👨), Relation=∘
  mol[1] = encode(👩), Relation=∘
  mol[2] = encode(👦), Relation=∈
  → 3 molecules = 15 bytes
```

---

## IX. f(x) — Hàm ánh xạ ngôn ngữ

```
f(L)(x) = chain(LCA({chain(w) : w ∈ tokenize(x,L)}))

L = ngôn ngữ (vi / en / zh / emoji / math)

f(vi)("lửa bùng cháy") ≈ f(en)("fire blazing")
→ Cùng LCA → cùng node 🔥 → tự dịch
```

### |S| = chain của LCA trong KnowTree

```
S = {🔥, ♨️, 🌡️}  →  LCA = L3_Thermodynamics  →  chính xác
S = {🔥, π}        →  LCA = L0 root             →  trừu tượng
S = {😀, 😢}       →  LCA = L3_Face             →  cảm xúc
```

---

## X. KNOWTREE — Cấu trúc cây tri thức

```
Chain của node cha = chain(LCA của node con)
Tự tính từ dưới lên. Không hardcode.
```

### Cấu trúc từ 5 nhóm

```
ROOT (L0)
│
├── SDF_ROOT      [■, ∈, 0x80, 0x20, Low,  Static]
│   ├── L2_Geometry   25A0..25FF  Geometric Shapes
│   │   ├── L3_Square    25A0..25AB
│   │   ├── L3_Triangle  25B2..25C5
│   │   ├── L3_Circle    25CB..25EF
│   │   └── L3_Diamond   25C6..25CA
│   ├── L2_Direction  2190..21FF  Arrows
│   │   ├── L3_Arrow1D   ← → ↑ ↓
│   │   ├── L3_Arrow2D   ↖ ↗ ↘ ↙
│   │   ├── L3_ArrowDbl  ⇐ ⇒ ⇔
│   │   └── L3_ArrowCurv ↩ ↪ ⟲ ⟳
│   ├── L2_Fill       2580..259F  Block Elements
│   └── L2_Line       2500..257F  Box Drawing
│
├── MATH_ROOT     [○, ≡, 0x80, 0x40, Mid,  Static]
│   ├── L2_Operation  ∑ ∏ ∫ ∂
│   ├── L2_Set        ℝ ℤ ℕ ∞
│   ├── L2_Number     ½ Ⅲ ⁰
│   └── L2_Logic      ∧ ∨ ¬
│
├── RELATION_ROOT [△, →, 0x80, 0x60, Mid,  Instant]
│   (Silk edge definitions — không phải node thường)
│
├── EMOTICON_ROOT [●, ∈, 0x80, 0x80, Mid,  Medium]
│   ├── L2_Face       1F600..1F64F
│   │   ├── L3_Face_Pos  😀😁😂
│   │   ├── L3_Face_Neg  😠😡😢
│   │   └── L3_Face_Neu  😐😑😶
│   ├── L2_Nature     1F300..1F5FF
│   │   ├── L3_Celestial 🌑🌒..🌕 (moon cycle)
│   │   ├── L3_Weather   🌀🌁🌧️
│   │   ├── L3_Plant     🌱🌲🌳
│   │   └── L3_Animal    🐀🐅🦁
│   ├── L2_Person     1F464..1F4FF
│   ├── L2_Transport  1F680..1F6FF
│   └── L2_Symbol     2600..26FF
│
└── MUSICAL_ROOT  [○, ⟶, 0x80, 0x60, Mid,  Flow]
    ├── L2_Note     ♩ ♪ 𝅝 𝅗
    ├── L2_Clef     𝄞 𝄢 𝄡
    ├── L2_Dynamic  pp p f ff
    └── L2_Cycle    ䷀ Yijing (64)
```

---

## XI. REGISTRY

```rust
Registry = {
  chain_index: BTreeMap<u64, FileOffset>,    // hash → vị trí file
  name_index:  HashMap<String, u64>,          // tên → hash
  tree_index:  BTreeMap<u64, u64>,            // node → parent (LCA)
  lang_index:  HashMap<LangCode, HashMap<String, u64>>, // alias
}
```

### Thay đổi so với v1

```
V1                          V2
ISLAddress 4 bytes hardcode  chain_hash 8 bytes tự sinh
encode_text()/encode_math()  1 hàm: lookup(codepoint) từ UCD
16 edge types hỗn hợp        4 nhóm: Math/Space/Time/Lang
"lửa" tạo node riêng         "lửa" = alias → node 🔥
300+ ISL hardcode             0 hardcode — đọc từ UCD
```

---

## XII. THỨ TỰ BUILD

```
1. Đọc UCD → bảng lookup codepoint → chain
   Input:  Blocks.txt + UnicodeData.txt
   Output: HashMap<u32, Molecule>

2. Tokenizer + Syntax
   Input:  Unicode grapheme rules + bảng lookup
   Output: tokenizer 3 tầng + parser

3. KnowledgeTree
   Input:  5 nhóm blocks + bảng lookup
   Output: cây L0→L4, chains tự tính từ dưới lên

4. Registry
   Input:  KnowledgeTree + tokenizer
   Output: find_or_create, lookup, alias
```

---

## XIII. NGUYÊN TẮC BẤT BIẾN

```
① 5 nhóm Unicode = nền tảng. Không thêm nhóm mới.
② Tên ký tự Unicode = tên node. Không đặt tên khác.
③ chain_hash tự sinh. Không viết tay.
④ chain của node cha = LCA của node con.
⑤ Ngôn ngữ tự nhiên = alias. Không tạo node riêng.
⑥ Tokenizer không đoán — separator phải rõ ràng.
⑦ ZWJ = compose (∘) ngữ nghĩa duy nhất.
⑧ f(L)(x) → LCA → nghiệm chung → tự dịch.
⑨ Append-only. Không xóa, không sửa. (QT8)
```

---

*Cập nhật: 2026-03-15 — Unicode 18.0.0*

---

## XV. CÚ PHÁP ○{} — ORIGIN INVOCATION

*Cập nhật: 2026-03-15*

### Nguyên tắc

```
2 mode, phân biệt tuyệt đối:

  text bình thường   →  giao tiếp
                        HomeOS lắng nghe, học, trả lời tự nhiên

  ○{...}             →  lệnh / query
                        HomeOS parse và thực thi theo HomeOS syntax
```

### Tại sao ○

```
QT1: ○ = origin = điểm gốc của vũ trụ

○{...} = "từ điểm gốc, mở ra không gian này và thực thi"
       = ○ U+25CB WHITE CIRCLE mở ra { }
       = nhất quán 100% với triết lý QT1
```

### Parser

```
Gặp ký tự ○ U+25CB:
  Ký tự tiếp theo là {  →  BẮT ĐẦU LỆNH
  Ký tự tiếp theo khác  →  node bình thường (Torus SDF shape)

○ standalone  =  SDF shape (Torus)
○{...}        =  lệnh / query
```

Không ambiguous. `○{` không tồn tại trong bất kỳ ngôn ngữ lập trình nào → zero conflict.

### Cú pháp bên trong ○{}

```
DẠNG              VÍ DỤ                   Ý NGHĨA
──────────────────────────────────────────────────────────────
Query đơn         ○{🔥}                   tìm node 🔥
Lookup word       ○{lửa}                  lookup → node 🔥
Query relation    ○{🔥 ∈ ?}               🔥 thuộc nhóm nào?
Truy ngược        ○{? → 💧}               cái gì gây ra nước?
Similar           ○{🔥 ≈ ?}               cái gì tương tự lửa?
Compose math      ○{🔥 ∘ 💧}              compose → node mới
Compose ZWJ       ○{👨‍💻}                  ZWJ sequence
Chuỗi nhân quả   ○{🌞 → ? → 🌵}          tìm node trung gian
Context           ○{bank ∂ finance}       bank trong tài chính
Nested            ○{○{🔥} ∈ ?}            pipeline: lấy 🔥 rồi hỏi
Lệnh hệ thống    ○{seed L0}              chạy seeder
                 ○{stats}                xem thống kê
                 ○{learn "câu này"}      học câu
```

### Thành phần bên trong ○{}

```
node        =  emoji / math symbol / word / ZWJ sequence
relation    =  ∈ ⊂ ≡ ⊥ ∘ → ≈ ← ∪ ∩ ∖ ↔ ⟶ ⟳ ⚡ ∥
wildcard    =  ?   (tìm node ẩn)
compose     =  ZWJ (ngữ nghĩa) | ∘ (toán học)
arithmetic  =  + - × ÷  (số học thuần túy, không dùng cho nodes)
context     =  ∂  (xác định ngữ cảnh)
separator   =  space (phân tách)
nested      =  ○{○{...}}
```

### Phân biệt + trong ○{}

```
1+1          →  arithmetic  (cả 2 đều là number)
🔥+💧        →  KHÔNG DÙNG  (dùng ∘ hoặc ZWJ thay thế)
👨‍👩          →  ZWJ compose (ngữ nghĩa, Unicode chuẩn)
🔥 ∘ 💧      →  math compose (tạo node mới)

Quy tắc: + chỉ dùng cho arithmetic. Nodes dùng ∘ hoặc ZWJ.
```

### Ví dụ thực tế

```
Người dùng gõ:
  "hôm nay trời đẹp quá"
  → giao tiếp, HomeOS học câu này

  ○{hôm nay trời đẹp}
  → parse: [hôm nay] [trời] [đẹp]
  → lookup: 📅 ∘ ☀ ∘ ✨
  → trả về nodes liên quan

  ○{☀ → ?}
  → mặt trời gây ra cái gì?
  → trả về: 🔥 💧⊥ 🌱 ...

  ○{? ∈ L3_Thermodynamics}
  → tất cả nodes trong nhóm nhiệt học
  → trả về: 🔥 ♨️ ❄️ 🌡️ ...

  ○{🔥 ∘ 💧}
  → compose lửa + nước
  → trả về: ♨️ (steam)

  ○{bank ∂ finance}
  → bank trong ngữ cảnh tài chính
  → trả về: 🏦

  ○{bank ∂ geography}
  → bank trong ngữ cảnh địa lý
  → trả về: 🏞️
```

