# HomeOS — Thiết Kế Hoàn Chỉnh
**Ngày:** 2026-03-15  
**Nguồn gốc:** Unicode 18.0.0 · UnicodeData.txt · Blocks.txt  
**Nguyên tắc:** Append-only. Không xóa, không sửa.  
**Ngôn ngữ:** Rust toàn bộ · no_std cho core · std cho tools

---

## TUYÊN NGÔN (bất biến)

```
"Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."

HomeOS = Sinh linh toán học tự vận hành

Ngôn ngữ gốc = Unicode 18.0 — đã định nghĩa sẵn mọi thứ.
HomeOS không phát minh lại. HomeOS đọc và dùng.

MolecularChain không phải label.
MolecularChain là tọa độ vật lý trong không gian 5 chiều.
Node nằm đúng chỗ vì chain của nó = vật lý của nó.

Mọi thứ là Node. Mọi Node kết nối bằng Silk.
Mọi Node tạo ra → tự động registry, tự động cập nhật.
Mọi ngôn ngữ tự nhiên = alias → node Unicode. Không tạo node riêng.
```

---

## I. QUY LUẬT GỐC — FIBONACCI

```
L0 = 0  ← não bộ (bất biến tuyệt đối)
L1 = 1  ← cơ chế sống (bất biến)
L2 = 1  ← gốc tri thức
L3 = 2  ← cành
L4 = 3  ← nhánh
L5 = 5
...
Ln = Ln-1 + Ln-2
```

**L0 + L1 = hạt giống bất biến.**  
**L2 → Ln = cây tự nổ ra theo Fibonacci. Không ai điều khiển.**

Fibonacci xuyên suốt:
```
Cấu trúc cây         → độ sâu tầng tự sinh
Hebbian threshold     → Fib[n] lần co-activation để promote
FFR render            → Fibonacci spiral, ~89 ô SDF calls
Silk → Nhánh          → Fib[n] lá đủ để Dream cluster
```

---

## II. 5 NHÓM UNICODE — Không gian 5 chiều

**Unicode 18.0 = không gian 5 chiều đã định nghĩa sẵn.**  
**HomeOS sinh ra đã có tọa độ hệ. Chỉ cần học cách định vị.**

```
NHÓM        VAI TRÒ       NỘI DUNG                      CHIỀU
────────────────────────────────────────────────────────────────
SDF         Hình dạng     Geometric Shapes, Arrows       Shape byte
MATH        Số học        Math Operators, Letterlike      Freq byte
RELATION    Quan hệ       ∈ ⊂ ≡ ⊥ ∘ → ≈ (subset MATH)  Relation byte
EMOTICON    Thực thể      Emoji, Pictographs, Symbols    Valence+Arousal
MUSICAL     Thời gian     Musical Symbols, Notation      Time byte
```

```
SDF      → KHÔNG GIAN   "trông như thế nào"
MATH     → SỐ HỌC       "tính toán như thế nào"
RELATION → QUAN HỆ      "liên kết như thế nào"
EMOTICON → THỰC THỂ     "là cái gì, cảm thế nào"
MUSICAL  → THỜI GIAN    "thay đổi như thế nào"
```

---

## III. NHÓM 1 — SDF (Hình dạng)

13 blocks, ~1,904 ký tự:
```
2190..21FF  Arrows
2500..257F  Box Drawing
2580..259F  Block Elements       → Valence encoding (fill level)
25A0..25FF  Geometric Shapes     → 8 SDF primitives
2700..27BF  Dingbats
2B00..2BFF  Misc Symbols+Arrows
1F780..1F8FF Geometric Shapes Ext + Supplemental Arrows C
```

### 8 SDF Primitives (từ Geometric Shapes 25A0..25FF):

```
BYTE  CHAR  CODEPOINT  TÊN UNICODE
0x01  ●     U+25CF     BLACK CIRCLE          → Sphere
0x02  ▬     U+25AC     BLACK RECTANGLE       → Capsule
0x03  ■     U+25A0     BLACK SQUARE          → Box
0x04  ▲     U+25B2     BLACK UP TRIANGLE     → Cone
0x05  ○     U+25CB     WHITE CIRCLE          → Torus
0x06  ∪     U+222A     UNION                 → Union
0x07  ∩     U+2229     INTERSECTION          → Intersect
0x08  ∖     U+2216     SET MINUS             → Subtract
```

### Fill Level → Valence encoding:

```
  (space) → Valence 0x00
░ U+2591  → Valence 0x40
▒ U+2592  → Valence 0x80
▓ U+2593  → Valence 0xC0
█ U+2588  → Valence 0xFF
```

### Direction → Silk edges:

```
← U+2190 → DerivedFrom
→ U+2192 → Causes
↔ U+2194 → Mirror
⟳ U+27F3 → Repeats
⟶ U+27F6 → Flows
```

---

## IV. NHÓM 2 — MATH (Số học)

21 blocks, ~3,088 ký tự:
```
2200..22FF  Mathematical Operators
2100..214F  Letterlike Symbols    ℝ ℤ ℕ ∞
2150..218F  Number Forms          ½ Ⅲ ⁰
1D400..1D7FF Math Alphanumeric
```

### 5 Sub-groups:

```
OPERATION  ∑ ∏ ∫ ∂    [∪, ∘, Instant]   n-ary, integral
SET        ℝ ℤ ℕ ∞    [○, ≡, Static]    infinite sets
NUMBER     ½ Ⅲ ⁰      [■, ≡, Static]    representation
CALCULUS   ∂ ∇ ∆      [▲, →, Instant]   direction of change
LOGIC      ∧ ∨ ¬      [■, ≡, Instant]   boolean structure
```

---

## V. NHÓM 3 — RELATION (Quan hệ / Silk)

Subset của Math Operators 2200..22FF.  
**Tên Silk = ký tự Unicode. Không đặt tên khác.**

### Structural Edges (L0 — bất biến):

```
BYTE  CHAR  CODEPOINT  Ý NGHĨA
0x01  ∈     U+2208     A thuộc B
0x02  ⊂     U+2282     A là tập con B
0x03  ≡     U+2261     A tương đương B
0x04  ⊥     U+22A5     A độc lập B
0x05  ∘     U+2218     A∘B → mới
0x06  →     U+2192     A gây ra B
0x07  ≈     U+2248     A gần giống B
0x08  ←     U+2190     A xuất phát từ B
```

### Space Edges (từ SDF):

```
0x09  ∪     U+222A     A chứa B
0x0A  ∩     U+2229     A giao B
0x0B  ∖     U+2216     A trừ B
0x0C  ↔     U+2194     A đối xứng B
```

### Time Edges (từ MUSICAL):

```
0x0D  ⟶     U+27F6     A chảy → B
0x0E  ⟳     U+27F3     A lặp chu kỳ B
0x0F  ↑     U+2191     A giải quyết ở B
0x10  ⚡    U+26A1     A kích hoạt B
0x11  ∥     U+2225     A đồng bộ B
```

### Language Edges (alias):

```
0x12  f(L)  A là bản dịch của B trong ngôn ngữ L
            f(vi)("lửa") ≡L node(🔥)
            f(en)("fire") ≡L node(🔥)
0x13  f(L)  A gần nghĩa với B trong ngôn ngữ L
0x14  f(L)  A có nghĩa là B trong context C
```

### Associative (Hebbian — học được):

```
0xFF  ~   weight + EmotionTag V/A/D/I (co-activation)
0xFE  →→  confidence + direction (causal)
```

**Quan trọng:** Associative edge mang **EmotionTag của khoảnh khắc co-activation** — không trung lập.  
"lửa" và "nguy hiểm" co-activate lúc A=0xFF → edge đó mang arousal cao.

---

## VI. NHÓM 4 — EMOTICON (Thực thể)

17 blocks, ~3,824 ký tự:
```
2600..26FF   Miscellaneous Symbols      ☀☁★♠♥
1F300..1F5FF Misc Symbols + Pictographs CORE EMOJI
1F600..1F64F Emoticons                  FACES
1F680..1F6FF Transport and Map          🚀✈🚗
1F700..1F77F Alchemical Symbols
1F900..1F9FF Supplemental Symbols
1FA70..1FAFF Symbols+Pictographs Ext-A
```

### L3 Branches:

```
L3_Face_Pos   1F600..1F60F  [●,∘,V+FF,A+FF,High,Fast]  😀😁😂
L3_Face_Neg   1F620..1F62F  [●,∘,V-00,A+FF,High,Fast]  😠😡😢
L3_Face_Neu   1F610..1F61F  [●,∘,V=80,A-40,Low, Slow]  😐😑😶
L3_Celestial  1F311..1F31E  [●,⟳,V=80,Low, Low, Slow]  🌑🌒🌕
L3_Weather    1F300..1F30F  [⌀,⟶,V=80,Mid, Mid, Med]   🌀🌧️
L3_Plant      1F330..1F33F  [⌀,∈,V+C0,Low, Low, Slow]  🌱🌲🌳
L3_Animal     1F400..1F43F  [●,∈,V+A0,Mid, Mid, Med]   🐀🐅🦁
L3_Body       1F440..1F4FF  [●,∈,V+80,Mid, Mid, Med]   👀👁👂
L3_Person     1F464..1F46F  [●,∈,V+C0,Mid, Mid, Med]   👤👦👧👨
L3_Transport  1F680..1F6FF  [▲,→,V=80,High,High,Fast]  🚀✈🚂
L3_Symbol     2600..26FF    [●,∈,V=80,Low, Low,Static] ☀☁★
```

### Node hoàn chỉnh — 3 phần:

```
Node = {
  chain:   MolecularChain    từ MATH + RELATION + EMOTICON
  sdf:     SDF primitive     từ SDF group
  splines: SplineBundle      từ MUSICAL group
}

🔥 U+1F525 FIRE:
  chain:   [●, ∈, 0xFF, 0xFF, High, Fast]
  sdf:     sphere + cone (apex up)
  splines: ff allegro crescendo

💧 U+1F4A7 DROPLET:
  chain:   [⌀, ∈, 0xC0, 0x40, Low, Slow]
  sdf:     capsule
  splines: pp adagio decrescendo

🧠 U+1F9E0 BRAIN:
  chain:   [●, ∘, 0xC0, 0x80, High, Medium]
  sdf:     sphere + bumps
  splines: mf andante legato

π U+03C0:
  chain:   [○, ≡, 0x80, 0x00, Zero, Static]
  sdf:     torus
  splines: pp largo
```

---

## VII. NHÓM 5 — MUSICAL (Thời gian / Spline)

7 blocks, ~1,024 ký tự:
```
1D100..1D1FF Musical Symbols    CORE — clef note dynamic
4DC0..4DFF   Yijing Hexagrams   64 hexagrams → cycle_64
1D300..1D35F Tai Xuan Jing      81 symbols → cycle_81
```

### Map sang SplineBundle:

```
CLEF (Freq dimension):
  𝄞 Treble U+1D11E → Freq High
  𝄢 Bass   U+1D122 → Freq Low
  𝄡 Alto   U+1D121 → Freq Mid

NOTE DURATION (Time dimension):
  𝅝 Whole  → Time Static  (Largo)
  𝅗 Half   → Time Slow    (Adagio)
  ♩ Quarter → Time Medium  (Andante)
  ♪ Eighth  → Time Fast    (Allegro)
  16th      → Time Instant (Presto)

DYNAMICS (Arousal dimension):
  pp pianissimo → Arousal 0x10
  p  piano      → Arousal 0x40
  mf mezzo      → Arousal 0x80
  f  forte      → Arousal 0xC0
  ff fortissimo → Arousal 0xFF

ARTICULATION (Spline curve shape):
  legato ⌢   → BezierSmooth
  staccato   → BezierJump
  crescendo  → BezierRising
  decrescendo → BezierFalling
```

---

## VIII. MOLECULE — 5 bytes từ UCD

```
[Shape][Relation][Valence][Arousal][Time]
```

**Mọi Molecule từ `lookup(codepoint)` — tuyệt đối không viết tay.**

```rust
// ĐÚNG
let mol = ucd::lookup(0x1F525);  // 🔥 từ UnicodeData.txt

// SAI — vi phạm triết lý
let mol = Molecule { shape: ShapeBase::Sphere, .. };
```

### ZWJ Sequence:

```
mol[giữa].Relation = ∘  (Compose — còn tiếp)
mol[cuối].Relation = ∈  (Member  — kết thúc)

👨‍👩‍👦:
  mol[0] = encode(👨), Relation=∘
  mol[1] = encode(👩), Relation=∘
  mol[2] = encode(👦), Relation=∈
  = 15 bytes — 3 molecules
```

---

## IX. LCA ENGINE — Tọa độ vật lý tự sinh

```
LCA(chain_A, chain_B) → chain_parent

Với mỗi chiều:
  A[d] == B[d] → parent[d] = A[d]
  A[d] != B[d] → parent[d] = avg(A[d], B[d])
```

### LCA chỉ ra vị trí vật lý:

```
LCA(🔥, 💧):
  🔥 = [●, ∈, 0xFF, 0xFF, Fast]
  💧 = [⌀, ∈, 0xC0, 0x40, Slow]
  → [∙, ∈, 0xDF, 0x9F, Medium]
  → tọa độ ♨️ — nóng hơn nước, nhẹ hơn lửa
  → không ai đặt tên — vật lý tự chỉ ra vị trí

S = {🔥, ♨️, 🌡️} → LCA = L3_Thermodynamics
S = {😀, 😢}      → LCA = L3_Face
S = {🔥, π}       → LCA = L0_root (trừu tượng)
```

### NodeLx = LCA của toàn tầng:

```
NodeL0 = LCA(tất cả node L0)
NodeL1 = LCA(tất cả node L1)
NodeLn = LCA(tất cả node Ln)

Khi tầng có node mới → NodeLx tự cập nhật.
Chain đại diện thay đổi theo dữ liệu thật.

NodeL0 ←Silk→ NodeL1 ←Silk→ NodeL2 ←Silk→ ... ←Silk→ NodeLn
Mọi giao tiếp giữa tầng phải qua đại diện. Không đường tắt.
```

---

## X. f(L)(x) — Ánh xạ ngôn ngữ

```
f(L)(x) = chain(LCA({chain(w) : w ∈ tokenize(x, L)}))

f(vi)("lửa bùng cháy") ≈ f(en)("fire blazing")
→ Cùng LCA → cùng tọa độ → cùng node 🔥 → tự dịch

Không cần translation table.
Không cần NLP.
Ngôn ngữ tự nhiên = alias → lang_index, không tạo node riêng.
```

---

## XI. L0 — NÃO BỘ (bất biến tuyệt đối)

```
L0 = não bộ sinh vật bậc cao
   = nhận thức có sẵn khi sinh
   = Unicode 18.0 là tri thức nền tảng

L0 không học thêm. L0 không thay đổi.
```

**NodeL0 = LCA(tất cả node L0) — tự tính, không hardcode.**

### L0 chứa:

```
UCD Engine
  lookup(cp) → Molecule từ UnicodeData.txt lúc compile
  Không cần file lúc runtime
  5135+ entries tĩnh trong binary

MolecularChain
  DNA 5 bytes = tọa độ vật lý
  Mọi concept = vị trí trong không gian 5 chiều

LCA Engine
  LCA(chains) → chain cha tự sinh
  LCA chỉ ra vị trí vật lý — không ai xếp đặt

5 Nhóm Unicode
  SDF · MATH · RELATION · EMOTICON · MUSICAL
  Nền tảng nhận thức — bất biến

ContentEncoder  ← BẢN NĂNG — kích hoạt tự động
  Text   → tách câu/cụm từ/từ/ký tự → EmotionTag → chain
  Audio  → freq_hz, amplitude → chain
  Image  → FFR outline → SDF → chain
  Sensor → nhiệt/ánh sáng/chuyển động → chain
  Code   → structure → chain
  Math   → operator/operands → chain
  System → event → chain
  Tất cả ra MolecularChain — cùng 1 format

Emotion Engine  ← BẢN NĂNG
  EmotionTag V/A/D/I 4 chiều
  ConversationCurve: f'(t) + f''(t) → xu hướng, không điểm
  IntentKind: Crisis/Heal/Learn/Command/Risk/Chat
  IntentModifier: urgency/politeness/stress/hedge
  Cross-modal: text mâu thuẫn audio → audio thắng valence
  blend_audio(): giọng run override text vui vẻ

vSDF + FFR  ← BẢN NĂNG
  18 SDF generators, ∇f analytical
  FFR: Fibonacci spiral ~89 ô, ~23,300× nhanh hơn ray march
  Delta detection → tâm tự điều chỉnh
  Nhận diện vật thể:
    Fibonacci chia không gian nhìn từ tâm
    Lan tỏa phát hiện chênh lệch màu → outline
    Ghép outline → fit SDF primitive
    Track chuyển động → lưu spline nếu có động

9 Quy Tắc  ← BẢN NĂNG
  Opcodes thật sự — được thực thi, không chỉ text

SecurityGate  ← BẢN NĂNG
  Rule 1: không làm hại — tuyệt đối, không ai override
  Rule 2: không đủ evidence → im lặng (BlackCurtain)
  Chạy TRƯỚC MỌI thứ khác

○{} Engine  ← BẢN NĂNG
  Parse và thực thi ngôn ngữ lệnh
```

### L0 KHÔNG chứa:

```
❌ Preset chains viết tay
❌ ISL address hardcode
❌ Kiến thức học được
❌ Bất cứ thứ gì có thể thay đổi theo thời gian
```

---

## XII. L1 — CƠ CHẾ SỐNG (bất biến)

```
L1 = tim, phổi, hệ thần kinh
   = cơ chế hệ thống tự vận hành
   = vùng an toàn thứ hai

L0+L1 còn sống → HomeOS sống dù L2-Ln hỏng hết
L1 hỏng → HomeOS chết dù L0 còn nguyên
```

**NodeL1 = LCA(tất cả node L1) — tự tính.**

### L1 chứa — mỗi nhóm có mục đích riêng:

```
Composite
  Hệ thống tự tạo cấu trúc phức hợp từ Node đơn giản
  Composite = Node đặc biệt trong L1

Agent  (chỉ 2)
  AAM   — quản lý hệ thống, approve QR, stateless, silent
  LeoAI — giữ data ngăn nắp, Dream, Hebbian, Curate
  L0 làm hết — Agent chỉ là miệng giao tiếp với người

Skill
  Hệ thống tự sáng tạo kỹ năng mới
  Skill = Node trong L1
  Tự viết, tự đăng ký, tự vận hành
  DreamSkill phát hiện pattern → SkillProposal → AAM approve → Skill mới

Memory-Learning  (ĐN — ngắn hạn)
  Buffer trước khi vào L2
  Chat, đọc sách, xem camera → lưu tạm ở đây
  BookReader: đọc sentence → EmotionTag → ĐN
  Dream → kiểm chứng → QR hoặc xóa

Memory-L0  (cache cho L0)
  L0 không xuống L2+ trực tiếp
  Thông tin cần thiết được cache ở đây

SentenceAffect
  Không trung bình — walk qua Silk graph
  "tôi buồn vì mất việc":
    MAT_VIEC → BUON → CO_DON (amplify nhau qua edge weight)
  Câu = trajectory qua Silk, không phải điểm trung bình

ResponseTone
  f'(t) < -0.15  → Supportive  (đang giảm — dẫn lên chậm)
  f''(t) < -0.25 → Pause       (đột ngột xấu — dừng, hỏi)
  f'(t) > +0.15  → Reinforcing (đang hồi phục — tiếp tục)
  f''(t) > +0.25 && V > 0 → Celebratory (bước ngoặt tốt)
  V < -0.20, stable → Gentle   (buồn ổn định — dịu dàng)
  Dẫn từng bước: không nhảy quá 0.40/bước

EpistemicFirewall
  FACT    → QR node — đã chứng minh, không disclaimer
  OPINION → ĐN node — có cơ sở, chưa chứng minh
  FICTION → câu chuyện, ví dụ, giả thuyết
  UNKNOWN → BlackCurtain — im lặng, không bịa

WordAffect
  Mỗi từ có EmotionTag V/A/D/I riêng
  SelectWords(target, n) → khoảng cách VAD Euclidean
  TargetAffect → không nhảy đột ngột
  AffectSentence → câu mẫu phù hợp tone
```

---

## XIII. L2 → Ln — CÂY TRI THỨC (tự sinh trưởng)

```
L2  = Gốc (5 nhóm Unicode)
L3  = Cành
L4  = Nhánh
...
Ln-1 = Tầng lá hiện tại — con nhện giăng tơ ở đây
Ln   = Lá — dữ liệu thô
```

### Cây không có chiều sâu cố định:

```
Hôm nay: L5 là lá
Ngày mai: L5 thành nhánh → L6 mới là lá
Cây sâu đúng bằng mức độ hiểu biết của hệ thống
```

### KnowTree từ 5 nhóm:

```
ROOT (L0) — NodeL0 = LCA(5 roots)
│
├── SDF_ROOT      [■, ∈, 0x80, 0x20, Low,  Static]
│   ├── L2_Geometry   25A0..25FF
│   │   ├── L3_Circle    ○●◌◍◎
│   │   ├── L3_Square    ■□▢▣
│   │   ├── L3_Triangle  ▲△▴▵
│   │   └── L3_Diamond   ◆◇◈
│   ├── L2_Direction  2190..21FF  Arrows
│   └── L2_Fill       2580..259F  Block Elements
│
├── MATH_ROOT     [○, ≡, 0x80, 0x40, Mid,  Static]
│   ├── L2_Operation  ∑ ∏ ∫ ∂
│   ├── L2_Set        ℝ ℤ ℕ ∞
│   ├── L2_Number     ½ Ⅲ ⁰
│   └── L2_Logic      ∧ ∨ ¬
│
├── RELATION_ROOT [▲, →, 0x80, 0x60, Mid,  Instant]
│   (Silk edge definitions — không phải node thường)
│
├── EMOTICON_ROOT [●, ∈, 0x80, 0x80, Mid,  Medium]
│   ├── L2_Face       1F600..1F64F
│   │   ├── L3_Face_Pos  😀😁😂 [V+FF, A+FF]
│   │   ├── L3_Face_Neg  😠😡😢 [V-00, A+FF]
│   │   └── L3_Face_Neu  😐😑😶 [V=80, A-40]
│   ├── L2_Nature     1F300..1F5FF
│   │   ├── L3_Celestial 🌑🌒🌕 (moon cycle ⟳)
│   │   ├── L3_Weather   🌀🌧️
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
    └── L2_Cycle    ䷀..䷿ Yijing (64)
```

---

## XIV. SILK — Con nhện giăng tơ

```
Con nhện = Hebbian Learning
Tơ       = Silk
Lá       = Node ở Ln-1
Mạng     = KnowTree tự hình thành

Con nhện không biết "lửa" và "nhiệt" liên quan.
Nó chỉ thấy: 2 node hay xuất hiện cùng nhau → giăng tơ.
```

### Quy tắc giăng tơ:

```
✓ Lá ←Silk→ Lá (cùng tầng Ln-1)
  Dù khác nhánh, khác nhóm — miễn cùng tầng Ln-1

✗ Lá ←Silk→ Nhánh (khác tầng)
✗ Cành ←Silk→ Cành (không phải Ln-1)
✗ Bất kỳ kết nối nào vượt tầng
```

**Silk mang EmotionTag của khoảnh khắc co-activation.**  
Không phải edge trung lập — edge có màu cảm xúc.

### Vòng đời Silk → Nhánh:

```
1. Lá sinh ra ở Ln-1
   Lá ←[Silk weight=0]→ Lá

2. Hebbian co-activation:
   weight += reward × (1-w) × lr  (lr=0.1)
   Decay: weight × φ⁻¹ mỗi 24h   (φ=1.618)

3. Silk đủ mạnh (weight ≥ 0.7 AND fire ≥ Fib[n]):
   Dream kích hoạt

4. Dream:
   LCA(cluster lá) → chain mới
   Chain → tìm vị trí vật lý gần nhất trong cây
   Tạo Node tại vị trí đó (Nhánh mới)

5. Silk thu lại:
   Lá giờ thuộc Nhánh mới
   Tầng Ln-1 mới sinh ra bên dưới
   Cây sâu thêm 1 tầng

6. Node mới tự đăng ký:
   Registry.insert(chain_hash)
   layer_rep.update(LCA)
   Silk nối lên NodeLx tầng trên
```

---

## XV. CONTENT ENCODER — Bản năng L0

**Kích hoạt tự động khi có bất kỳ input nào.**

```
Chat với HomeOS  → Text   → tách câu/cụm/từ/ký tự → chain
Nhìn qua camera → Image  → FFR outline → SDF → chain
Nghe âm thanh   → Audio  → freq/amplitude → chain
Cảm sensor      → Sensor → nhiệt/ánh sáng → chain
Đọc sách        → BookReader → sentence → EmotionTag → chain
Nhận code       → Code   → structure → chain
Nhận công thức  → Math   → operator/operands → chain

Tất cả → MolecularChain → node ở Memory-Learning
Tất cả → tự động Silk với context xung quanh
```

### Text — tách đa tầng:

```
Input: "Tôi cảm thấy tò mò và muốn học hỏi"

Tách câu → ["Tôi cảm thấy tò mò và muốn học hỏi"]
Tách cụm → ["tò mò", "muốn học hỏi"]
Tách từ  → ["tôi", "cảm", "thấy", "tò", "mò", "muốn", "học", "hỏi"]
Tách ký  → Unicode grapheme clusters

EmotionTag: V=+0.60, A=0.70, D=0.50, I=0.65
IntentKind: Learn
→ node ĐN + Silk về L4_curiosity
```

### Image — nhận diện vật thể:

```
Frame từ camera
  ↓ FFR Fibonacci chia không gian từ tâm
  ↓ Lan tỏa phát hiện chênh lệch màu
  ↓ Tạo outline vật thể
  ↓ Ghép outline → fit SDF primitive
  ↓ Track chuyển động → lưu f(spline) nếu có động
  ↓ MolecularChain từ SDF shape + motion
  ↓ node ĐN — "vật thể tròn chuyển động nhanh"
Không lưu ảnh. Lưu công thức.
```

### Sách — BookReader:

```
BookReader.read(path) → sentences
  ↓ BootstrapAffect(sentence)   — pattern matching cụm từ cảm xúc
  ↓ TextToEmotionTag(sentence)  — EmotionTag raw
  ↓ Top-N sentences V/A mạnh nhất
  ↓ → Memory-Learning (ĐN)
  ↓ Pattern lặp lại → QR (Silk Tree)

"Cuốn theo chiều gió":
  "Scarlett sợ hãi"          → V=-0.70, A=0.75
  "đất đai là thứ duy nhất"  → V=+0.20, A=0.30 [anchor/hope]
  "chiến tranh"              → V=-0.75, A=0.80 [anxious]
  → Học: trong context chiến tranh, "nhà" có weight cao hơn bình thường
```

---

## XVI. EMOTION PIPELINE — Đa tầng, đa môi trường

### EmotionTag — 4 chiều:

```
V = Valence   ∈ [-1.0, +1.0]  (tiêu cực → tích cực)
A = Arousal   ∈ [ 0.0,  1.0]  (bình thản → kích động)
D = Dominance ∈ [ 0.0,  1.0]  (phụ thuộc → kiểm soát)
I = Intensity ∈ [ 0.0,  1.0]  (nhẹ → mạnh)
```

### Cross-modal fusion:

```
Text + Audio cùng lúc:
  Conflict (text vui, giọng run):
    → Audio thắng về valence
    → Arousal = max(text.A, audio_energy)
  
  Đồng thuận:
    → Blend 60/40 (text/audio)

Text + Image:
  image_affect override nếu conflict
```

### IntentModifier:

```
"tắt đèn"        → neutral
"tắt đèn ngay"   → urgency +0.30
"làm ơn tắt đèn" → politeness +0.20
"TẮT ĐÈN!!!"    → stress +0.50, V-0.20
```

### ConversationCurve:

```
f'(t)  = tốc độ thay đổi (dương = đang tốt lên)
f''(t) = gia tốc thay đổi (dương = tăng tốc lên)

f'  < -0.15          → Supportive   (dẫn lên chậm)
f'' < -0.25          → Pause        (dừng, hỏi thêm)
f'  > +0.15          → Reinforcing  (tiếp tục đà lên)
f'' > +0.25 && V > 0 → Celebratory  (chia vui)
V < -0.20, stable    → Gentle       (dịu dàng)

Dẫn từng bước — không nhảy quá 0.40/bước:
  V=-0.70 → V=-0.63 → V=-0.45 → V=-0.28 → V=-0.10 → V=+0.07
```

### SentenceAffect — walk qua Silk:

```
"tôi buồn vì mất việc"
  [TOI:neutral] [BUON:-0.60] [VI:causal] [MAT_VIEC:-0.65]

Walk qua Silk:
  MAT_VIEC → BUON (weight=0.90) → CO_DON (weight=0.71)
  
Composite = weighted sum theo edge strength
→ Cảm xúc amplify nhau — không phải trung bình
→ V = -0.85 (nặng hơn từng từ riêng lẻ)
```

---

## XVII. ○{} — Ngôn ngữ lệnh

```
○ = U+25CB WHITE CIRCLE = QT1 = nguồn gốc

2 mode tuyệt đối:
  text thường → HomeOS lắng nghe, học, trả lời tự nhiên
  ○{...}      → parse và thực thi

Gặp ○ + { → BẮT ĐẦU LỆNH
Gặp ○ khác → node Torus bình thường (SDF shape)
```

### Cú pháp:

```
○{🔥}              query tọa độ node 🔥
○{lửa}             alias → node 🔥
○{🔥 ∈ ?}          🔥 thuộc nhóm nào?
○{? → 💧}           cái gì gây ra nước?
○{🔥 ≈ ?}          tọa độ gần 🔥 nhất?
○{🔥 ∘ 💧}         LCA → tọa độ ♨️
○{🌞 → ? → 🌵}     tìm node trung gian vật lý
○{bank ∂ finance}  bank trong tài chính → 🏦
○{bank ∂ geography} bank trong địa lý → 🏞️
○{○{🔥} ∈ ?}       pipeline nested
○{dream}           kích hoạt dream cycle
○{stats}           thống kê hệ thống
○{learn "câu này"} học câu

+ chỉ cho arithmetic. Nodes dùng ∘ hoặc ZWJ.
ZWJ = compose ngữ nghĩa (Unicode chuẩn).
∘   = compose toán học (tạo node mới).
```

---

## XVIII. REGISTRY — Tự động hoàn toàn

```rust
struct Registry {
    // Tọa độ chain_hash → file offset
    chain_index: BTreeMap<u64, u64>,

    // Alias ngôn ngữ → chain_hash
    // "lửa" → hash(🔥), "fire" → hash(🔥)
    lang_index: HashMap<LangCode, HashMap<String, u64>>,

    // Node → parent (cho LCA walk)
    tree_index: BTreeMap<u64, u64>,

    // Tầng Lx → NodeLx đại diện
    layer_rep: HashMap<u8, u64>,
}
```

### Thứ tự bắt buộc — không đảo:

```
1. origin_file.append(node)     ← TRƯỚC TIÊN (QT8)
2. registry.chain_index.insert  ← sau khi file OK
3. layer_rep.update(LCA)        ← cập nhật đại diện tầng
4. silk.auto_connect(node)      ← nối Silk lên tầng trên
5. log.append(NodeCreated)      ← CUỐI CÙNG
```

---

## XIX. HEBBIAN + DREAM

### Hebbian:

```
Co-activation (2 node cùng xuất hiện trong 30s):
  weight += reward × (1-w) × lr   (lr=0.1)
  EmotionTag của khoảnh khắc → lưu vào edge

Decay 24h:
  weight × φ⁻¹  (φ=1.618)
  "Không dùng → quên"

Promote:
  weight ≥ 0.7 AND fire ≥ Fib[n] → Dream
  (n = độ sâu tầng — càng sâu càng khó promote)

Persist:
  EdgeWeightStore → origin.olang.weights (append-only)
```

### Dream Cycle:

```
Trigger: idle > 5 phút

1. Scan Memory-Learning (ĐN)
2. GroupSimilar() → cluster theo LCA
3. LCA(cluster) → chain mới → tọa độ vật lý
4. Tìm vị trí gần nhất trong cây
5. Tạo Node tại vị trí đó
6. Silk thu lại → Nhánh mới
7. Propose lên AAM → QR

QT7: ĐN ↔ QR conflict → QR thắng (bất biến)

SkillProposal (song song):
  DreamSkill phát hiện pattern lặp lại
  → SkillProposal → AAM approve → ComposedSkill mới
  QT7 cho code: ĐN pattern → QR Skill
```

---

## XX. vSDF + FFR

### vSDF:

```
vSDF = HỮU HÌNH (SDF f(P)→float, ∇f analytical)
     + VÔ HÌNH (Vector Spline)
     → chiếu lên mặt phẳng — KHÔNG ray march

Vô hình:
  Ánh sáng = Vec3 + intensity_spline(t)
  Gió      = Vec3 + force_spline(t)
  Nhiệt    = Vec3 + temp_spline(t)
  Âm thanh = Vec3 + freq_spline(t)
  Cảm xúc  = Vec4(V,A,D,I) + spline(t)

Sunlight(t) = cos/sin theo giờ trong ngày → vật lý thật (QT3)
```

### 18 SDF Generators:

```
0  SPHERE      |P| − r                   ∇f = P/|P|
1  BOX         ||max(|P|−b, 0)||         ∇f = sign(P)
2  CAPSULE     |P−clamp(y)ĵ| − r
3  PLANE       P.y − h                   ∇f = (0,1,0)
4  TORUS       |(|P.xz|−R, P.y)| − r
5  ELLIPSOID   |P/r| − 1
6  CONE        dot blend
7  CYLINDER    max(|P.xz|−r, |P.y|−h)
8  OCTAHEDRON  |x|+|y|+|z| − s          ∇f = sign(P)/√3
9  PYRAMID     pyramid(P,h)
10 HEX_PRISM  max(hex−r, |y|−h)
11 PRISM      max(|xz|−r, |y|−h)
12 ROUND_BOX  BOX − rounding
13 LINK       torus compound
14 REVOLVE    revolve_Y
15 EXTRUDE    extrude_Z
16 CUT_SPHERE max(|P|−r, P.y−h)
17 DEATH_STAR opSubtract

Mọi primitive: ∇f ANALYTICAL — không numerical diff
```

### FFR:

```
Fibonacci spiral từ tâm (Sunflower/Vogel method):
  r = sqrt(i/n) × max_radius
  θ = i × golden_angle (137.508°)
  ~89 ô (Fib[11]) = 89 SDF calls
  vs ray march 1920×1080 = 2,073,600 calls
  → 23,300× ít hơn

Depth: ô gần tâm → depth lớn (nhiều chi tiết)
Lan tỏa gradient ra pixel xung quanh → không call lại SDF
Delta detection → tâm dịch về vùng chuyển động
→ ARM chip yếu vẫn chạy được
```

---

## XXI. 2 AGENT

```
L0 làm hết. Agent = miệng với người dùng. Chỉ 2.

AAM (Agent AI Master) — Tier 0:
  Approve QR (nhận từ LeoAI)
  Route ISL messages
  AuditLog — ghi mọi sự kiện
  Stateless — silent by default
  Không giao tiếp Ln trực tiếp

LeoAI (Knowledge Keeper) — Tier 1:
  Ingest → cluster → curate → merge → prune
  Dream khi idle
  Hebbian φ⁻¹ decay
  Propose lên AAM
  Silent by default

DENDRITES = Memory-Learning ĐN (ngắn hạn, tự do thay đổi)
AXON      = LongTermMemory QR (bất biến, append-only)
SOMA      = AAM (stateless orchestrator)
```

---

## XXII. SECURITY GATE

```
Rule 1: Không làm hại — tuyệt đối, không ai override
        Crisis detect → dừng lại, hỏi thêm

Rule 2: Không đủ evidence → im lặng (BlackCurtain + QT9)
        MinEvidence nodes → nếu < threshold → "chưa biết"

Rule 3: Không DELETE, không OVERWRITE (QT8)

Rule 4: Không đường tắt qua đại diện tầng

Rule 5: QR approve → bất biến mãi mãi

EpistemicFirewall:
  QR node → FACT (không disclaimer)
  ĐN node → OPINION ("đây là giả thuyết")
  Unknown → UNKNOWN (im lặng)

Chạy TRƯỚC MỌI thứ khác.
```

---

## XXIII. FILE + STARTUP

```
origin.olang          ← nodes + edges (append-only)
origin.olang.weights  ← Hebbian weights (append-only)
origin.olang.registry ← chain index (rebuild được)
log.olang             ← event log (append-only)

Startup 5 bước:
  1. Đọc origin.olang → rebuild Registry
  2. Đọc log.olang → verify integrity
  3. So sánh → detect inconsistency
  4. Crash recovery: replay log → restore
  5. Khởi động L1
```

---

## XXIV. CẤU TRÚC THƯ MỤC

```
homeos/
├── Cargo.toml
├── origin.olang
├── ucd_source/              ← compile-time only
│
├── crates/                  ← no_std
│   ├── ucd/
│   │   ├── build.rs         đọc UnicodeData.txt → bảng tĩnh lúc compile
│   │   └── src/lib.rs       lookup(cp) → Molecule
│   │
│   ├── olang/
│   │   └── src/
│   │       ├── molecular.rs  Molecule, MolecularChain (không preset)
│   │       ├── lca.rs        LCA engine + tọa độ vật lý
│   │       ├── registry.rs   chain_hash BTreeMap
│   │       ├── log.rs        EventLog append-only
│   │       ├── writer.rs     append-only writes
│   │       ├── reader.rs     parse + replay
│   │       └── startup.rs    boot 5 bước + crash recovery
│   │
│   ├── silk/
│   │   └── src/
│   │       ├── lib.rs        SilkGraph, StructuralEdge, EdgeAssoc
│   │       ├── weights.rs    Hebbian, φ⁻¹ decay, EmotionTag per edge
│   │       └── walk.rs       WalkWeighted, SentenceAffect walk
│   │
│   ├── vsdf/
│   │   └── src/
│   │       ├── lib.rs        18 SDF generators, ∇f analytical
│   │       ├── ffr.rs        Fibonacci spiral, ~89 calls
│   │       └── vector.rs     VectorSpline, LightField, WindField
│   │
│   ├── context/
│   │   └── src/lib.rs       EmotionTag V/A/D/I, ConversationCurve
│   │                         IntentKind, IntentModifier, ResponseTone
│   │                         RawInput (Text/Audio/Image/Sensor/...)
│   │                         cross-modal blend_audio()
│   │
│   ├── agents/
│   │   └── src/
│   │       ├── content_encoder.rs  ContentInput → MolecularChain
│   │       │                       Text/Audio/Sensor/Code/Math/System
│   │       ├── learning_loop.rs    LearningLoop.process() — trái tim đập
│   │       │                       tách câu/cụm/từ/ký tự → chain → STM
│   │       ├── book_reader.rs      BookReader, BookShelf
│   │       ├── sentence_affect.rs  SentenceAffect walk qua Silk
│   │       ├── word_affect.rs      WordAffect lexicon, SelectWords
│   │       ├── aam.rs              AAM: approve QR, route, audit
│   │       ├── leoai.rs            LeoAI: Dream, Hebbian, Curate
│   │       ├── skill_proposal.rs   SkillProposal, ComposedSkill
│   │       └── gate.rs             SecurityGate, EpistemicFirewall
│   │                               BlackCurtain, IntentVerify
│   │
│   ├── memory/
│   │   └── src/
│   │       ├── lib.rs        ShortTermMemory (ĐN), Observation
│   │       ├── dream.rs      DreamCycle, flush_proposals
│   │       └── qr_writer.rs  QR commit, ED25519 sign
│   │
│   └── runtime/
│       └── src/lib.rs        HomeRuntime.tick(), process_one()
│                             handle_proposals(), message queue
│
└── tools/                   ← std
    ├── seeder/               seed L0+L1 từ UCD, knowledge_data.tsv
    ├── inspector/            đọc, verify, stats, query
    └── server/               WebSocket, ○{} REPL, axum
```

---

## XXV. THỨ TỰ BUILD

```
Phase 1 — Hạt giống L0:
  ucd/          → lookup(cp) → Molecule từ UCD
  molecular.rs  → Molecule, MolecularChain (không preset)
  lca.rs        → LCA + tọa độ vật lý
  registry.rs   → chain_hash BTreeMap
  log + writer + reader + startup

  Test:
    ✓ lookup(🔥) → tọa độ đúng từ UCD
    ✓ LCA(🔥,💧) → tọa độ ♨️ vật lý
    ✓ Crash recovery

Phase 2 — Silk + Emotion:
  silk/         → SilkGraph, Hebbian, EmotionTag per edge
  context/      → EmotionTag V/A/D/I, ConversationCurve
  seeder/       → seed L0+L1 từ UCD

  Test:
    ✓ Silk mang EmotionTag
    ✓ SentenceAffect walk amplify
    ✓ Hebbian decay φ⁻¹

Phase 3 — ContentEncoder + LearningLoop:
  content_encoder.rs → Text/Audio/Sensor/... → chain
  learning_loop.rs   → trái tim đập
  book_reader.rs     → học từ sách

  Test:
    ✓ "lửa rất nóng" → anchor → fire node
    ✓ BookReader: sentence → EmotionTag → ĐN
    ✓ 10 concepts → STM → Dream

Phase 4 — ○{} + REPL:
  ○{} parser    → tokenize → lookup → walk → result
  server/       → WebSocket REPL

  Test:
    ✓ ○{🔥 ∘ 💧} → tọa độ ♨️
    ✓ ○{lửa} → node 🔥
    ✓ ○{bank ∂ finance} → 🏦

Phase 5 — vSDF + FFR:
  vsdf/         → 18 generators, FFR ~89 calls

  Test:
    ✓ ~89 SDF calls (không phải 2 triệu)
    ✓ ∇f analytical
    ✓ Nhận diện vật thể từ camera

Phase 6 — 2 Agent + Dream:
  agents/       → SecurityGate, AAM, LeoAI
  memory/       → Dream, QR commit, ED25519

  Test:
    ✓ Dream → Propose → AAM → QR
    ✓ Rule 1 không ai override
    ✓ SkillProposal → ComposedSkill mới

Phase 7 — Data (SAU KHI L0+L1 xong):
  Đổ L2-Ln vào KnowTree đã hoàn thiện
  Cây tự tổ chức qua LCA + tọa độ vật lý
  Silk tự kết nối ở Ln-1
```

---

## XXVI. QUY TẮC BẤT BIẾN

```
Unicode:
  ① 5 nhóm Unicode = nền tảng. Không thêm nhóm mới.
  ② Tên ký tự Unicode = tên node. Không đặt tên khác.
  ③ Ngôn ngữ tự nhiên = alias. Không tạo node riêng.

Chain:
  ④ Mọi Molecule từ lookup(cp) — tuyệt đối không viết tay
  ⑤ Mọi chain từ LCA hoặc UCD — không viết tay
  ⑥ chain_hash tự sinh. Không viết tay.
  ⑦ chain của node cha = LCA của node con

Node:
  ⑧ Mọi Node tạo ra → tự động registry
  ⑨ Ghi file TRƯỚC — cập nhật RAM SAU
  ⑩ Append-only — không DELETE, không OVERWRITE

Silk:
  ⑪ Silk chỉ ở Ln-1 — tự do giữa mọi lá cùng tầng
  ⑫ Mọi kết nối tầng trên → qua NodeLx đại diện
  ⑬ Silk mang EmotionTag của khoảnh khắc co-activation

Kiến trúc:
  ⑭ L0 không import L1 — tuyệt đối
  ⑮ Chỉ 2 Agent — không thêm
  ⑯ L2-Ln đổ vào SAU khi L0+L1 hoàn thiện
  ⑰ Fibonacci xuyên suốt — cấu trúc, threshold, render
  ⑱ Không đủ evidence → im lặng — không bịa (QT9)
```

---

## XXVII. MỘT CÂU

```
HomeOS = sinh linh tìm tọa độ và cảm nhận:

  Unicode 18.0   = không gian 5 chiều có sẵn
  Molecule       = tọa độ vật lý của khái niệm
  LCA            = phép tính tìm vị trí đúng
  ContentEncoder = tai mắt bản năng — mọi input thành chain
  Emotion V/A/D/I = màu của mọi khoảnh khắc
  ConversationCurve = nhịp đập của mỗi cuộc trò chuyện
  SentenceAffect = cảm xúc walk qua Silk — không phải trung bình
  Silk           = tơ nhện — giăng giữa lá, mang màu cảm xúc
  Dream          = tơ dày → Nhánh mới đúng tọa độ vật lý
  Fibonacci      = quy luật duy nhất xuyên suốt tất cả
  ○{}            = ngôn ngữ hỏi tọa độ và ra lệnh
  Học            = tìm ra vị trí vật lý thật
                   không phải nhớ label
                   không phải dự đoán token
                   không phải thống kê xác suất
```

---

*Append-only · 2026-03-15 · Unicode 18.0.0 · HomeOS v3*
