# KnowTree Design — o{} → Ln-1

> **Nox — 2026-03-25, sửa theo Lupin**
> **L0 = cây tổng. L1 = nhánh chính. Ln-1 = lá.**

---

## I. L0 — KnowTree GỐC

```
L0 = o{KnowTree}
  Đây là CÂY TỔNG chứa MỌI THỨ.
  Mỗi nhánh L0 = 1 loại tri thức.

o{KnowTree} = [
    UDC,            // 9,584 SDF gốc — SEALED, immutable
    Emoji_UTF32,    // 155,000+ alias → trỏ về UDC
    Learning,       // tri thức học được (facts, books, conversations)
    Memory,         // STM, Silk edges, Dream clusters — working memory
    Agent,          // AAM, LeoAI, Chiefs, Workers
    Skill,          // 7 instincts + learned skills
    Code,           // fn nodes, bytecode chains
    Program,        // origin.olang, tools, configs
    Device,         // hardware: sensors, display, network interfaces
]

Mỗi phần tử = u16 index trỏ đến nhánh L1 tương ứng.
Kích thước L0: 9 × 2B = 18 bytes
```

---

## II. L1 — NHÁNH CHÍNH (phân loại từ L0)

```
Mỗi nhánh L0 phân thành các nhóm chính ở L1:

L0:UDC → L1[
    SDF,        // 14 blocks, 1,838 chars (Shape)
    MATH,       // 21 blocks, 2,563 chars (Relation)
    EMOTICON,   // 17 blocks, 3,487 chars (Valence + Arousal)
    MUSICAL,    // 7 blocks, 958 chars (Time)
]

L0:Emoji_UTF32 → L1[
    Emoji_faces,        // 😀😂😭😡... → alias trỏ về E.09
    Emoji_people,       // 👨👩👶🧑... → alias trỏ về E.08
    Emoji_animals,      // 🐱🐶🦁... → alias trỏ về E.08
    Emoji_objects,      // 🔥⭐💎... → alias trỏ về E.08
    Emoji_symbols,      // ❤✅❌... → alias trỏ về E.02
    UTF32_latin,        // 172,849 Unicode assigned chars
    UTF32_cjk,          // 97,000 CJK ideographs
    UTF32_hangul,       // 11,172 Hangul syllables
    UTF32_other_scripts,// Arabic, Cyrillic, Thai...
]

L0:Learning → L1[
    facts,              // "Ha Noi la thu do cua Viet Nam"
    books,              // "Cuon Theo Chieu Gio"
    conversations,      // session history
    observations,       // sensor data learned
]

L0:Memory → L1[
    STM,                // short-term: last 32 turns
    Silk,               // Hebbian edges (cross-branch)
    Dream,              // consolidated clusters
    QR,                 // promoted, append-only, immutable
]

L0:Agent → L1[
    AAM,                // approve/reject (tier 0)
    LeoAI,              // learn+dream+curate (tier 1)
    HomeChief,          // quản lý Worker nhà
    VisionChief,        // quản lý Worker camera
    NetworkChief,       // quản lý Worker network
]

L0:Skill → L1[
    Honesty,            // instinct #1
    Contradiction,      // instinct #2
    Causality,          // instinct #3
    Abstraction,        // instinct #4
    Analogy,            // instinct #5
    Curiosity,          // instinct #6
    Reflection,         // instinct #7
    learned_skills,     // Dream promoted skill clusters
]

L0:Code → L1[
    fn_nodes,           // user-defined functions
    builtin_nodes,      // map, filter, reduce, sort...
    lambda_nodes,       // anonymous closures
]

L0:Program → L1[
    origin_binary,      // origin.olang metadata
    config,             // settings, personality
    training_data,      // docs/training/*.md
]

L0:Device → L1[
    x86_64,             // current platform
    arm64,              // mobile (skeleton)
    wasm,               // browser (skeleton)
    sensors,            // future: camera, mic, bio
]
```

---

## III. L2 → L3 → ... → Ln-2 — PHÂN NHÁNH TIẾP

```
Mỗi nhánh L1 tiếp tục phân thành L2 (nhóm con).
Mỗi nhóm L2 tiếp tục phân thành L3 (nhánh con).
Cứ thế cho đến khi KHÔNG THỂ PHÂN NỮA.

Ví dụ UDC:

L1:SDF → L2[
    Arrows,             // S.01, 112 chars
    Misc_Technical,     // S.02, 256 chars
    Box_Drawing,        // S.03, 128 chars
    Block_Elements,     // S.04, 32 chars
    Geometric_Shapes,   // S.05, 96 chars
    Dingbats,           // S.06, 192 chars
    ...14 blocks total
]

L2:Arrows → L3[
    Leftwards,          // ← ⇐ ↞ ↢ ... (11 chars)
    Rightwards,         // → ⇒ ↠ ↣ ... (11 chars)
    Bidirectional,      // ↔ ⇔ ↭ ... (8 chars)
    Upwards,            // ↑ ⇑ ↟ ... (6 chars)
    Downwards,          // ↓ ⇓ ↡ ... (7 chars)
    Diagonal,           // ↗ ↘ ↙ ↖ ... (8 chars)
    Curved,             // ↩ ↪ ...
    Dashed,             // ⇠ ⇢ ...
    Heavy,              // ➡ ➜ ...
    Wave,               // ↝ ...
]

L3:Leftwards → Ln-1[
    ← (U+2190),        // LEFTWARDS ARROW
    ⇐ (U+21D0),        // LEFTWARDS DOUBLE ARROW
    ↞ (U+219E),        // LEFTWARDS TWO HEADED ARROW
    ...11 leaves
]

Ví dụ Learning:

L1:facts → L2[
    geography,          // "Ha Noi la...", "Viet Nam o..."
    science,            // "Einstein phat minh...", "DNA la..."
    history,            // "Origin bat dau ngay 11..."
    dialog_patterns,    // "khi nguoi ta chao..."
    tech,               // "SHA-256 la..."
]

L2:geography → L3[
    Vietnam,            // facts about Vietnam
    Japan,              // facts about Japan
    USA,                // facts about USA
    ...
]

L3:Vietnam → Ln-1[
    "Viet Nam la quoc gia o Dong Nam A voi thu do Ha Noi"   ← LÁ
    "Ho Chi Minh City la thanh pho lon nhat cua Viet Nam"   ← LÁ
    "Da Nang la thanh pho bien dep nam giua Viet Nam"       ← LÁ
    ...
]

Ví dụ Sách:

L1:books → L2[
    "Cuon Theo Chieu Gio",
    "Hoang Tu Be",
    ...
]

L2:"Cuon Theo Chieu Gio" → L3[
    Loi_Gioi_Thieu,
    Chuong_1,
    Chuong_2,
    ...
    Chuong_63,
]

L3:Chuong_1 → L4[
    Doan_1,
    Doan_2,
    ...
]

L4:Doan_1 → L5[
    Cau_1,
    Cau_2,
    ...
]

L5:Cau_1 → L6[
    Tu_1, Tu_2, Tu_3, ...    ← words
]

L6:Tu_1 → Ln-1[
    char_1, char_2, char_3, ...    ← LÁ (Unicode codepoints)
]
```

---

## IV. Ln-1 = LÁ — CÁ THỂ CUỐI CÙNG

```
Ln-1 = node KHÔNG THỂ PHÂN TIẾP NỮA.

Mỗi lá = 1 P_weight (2 bytes).
Lá KHÔNG có con. Lá IS giá trị.

Ví dụ lá:
  ← (U+2190) = P_weight [S:1, R:5, V:4, A:4, T:2]     ← UDC char, immutable
  "tình yêu"  = P_weight [S:0, R:0, V:7, A:5, T:2]     ← learned word
  fn fib      = P_weight [S:0, R:0, V:4, A:4, T:3]     ← fn node
  "Ha Noi..." = P_weight [S:0, R:2, V:5, A:3, T:0]     ← fact node

Khi 1 node có thể phân tiếp → nó KHÔNG phải lá → nó là nhánh.
Khi 1 node KHÔNG thể phân → nó LÀ lá = Ln-1.

Depth khác nhau cho từng nhánh:
  UDC char: L0 → L1 → L2 → L3 → Ln-1=L4 (depth 4)
  Sách câu: L0 → L1 → L2 → L3 → L4 → L5 → L6 → Ln-1=L7 (depth 7)
  Fn node:  L0 → L1 → Ln-1=L2 (depth 2)

CÂY KHÔNG CÓ DEPTH CỐ ĐỊNH.
Mỗi nhánh phân đến khi hết phân.
Ln-1 ở bất kỳ depth nào.
```

---

## V. SILK TRONG CÂY

```
Structural Silk = thứ tự trong array = 0 bytes
  chain[A, B, C] → A→B→C implicit

Hebbian Silk = NỐI NHÁNH KHÁC NHAU
  "Scarlett" ở L3:Chuong_1 ↔ "Scarlett" ở L3:Chuong_30
  "buồn" ở L1:Memory:STM ↔ "mất việc" ở L1:Learning:facts

  Hebbian TẠO NODE MỚI ở Ln-1:
    co_activate("buồn", "mất_việc")
    → nếu w ≥ φ⁻¹ → Dream promote
    → node mới "mất_mát" ở L1:Learning
    → LÁ MỚI ở Ln-1 đúng vị trí trong cây

  8 bytes/edge: from(2B) + to(2B) + weight(2B) + emotion(2B)
```

---

## VI. DUNG LƯỢNG

```
L0 root:           18 bytes (9 nhánh × 2B)
L1 branches:      ~100 nhánh × 2B = 200 bytes
L2 groups:        ~500 nhóm × 2B = 1 KB
L3 sub-groups:    ~2,000 nhánh × 2B = 4 KB
L4+ leaves:       ~10,000 lá × 2B = 20 KB
───────────────────────────────────────
TOTAL TREE:       ~25 KB (fits L1 cache)

Alias table:      ~1 MB (riêng biệt)
Chain data:       2 bytes/link (biến thiên theo nội dung)
Hebbian Silk:     ~40 KB (5,000 edges × 8B)

1 cuốn sách thêm: ~400 KB chain data
1 đời: ~80 MB
256 MB heap: đủ cho 3 đời
```

---

## VII. NGUYÊN TẮC

```
1. L0 = MỌI THỨ. Không có gì ngoài L0.
2. Mỗi nhánh phân đến khi hết phân → Ln-1 = lá.
3. Depth không cố định. Mỗi nhánh có depth riêng.
4. Lá = 2 bytes (P_weight). Không phân tiếp.
5. Thứ tự trong array = Structural Silk = 0 bytes.
6. Hebbian = CHỈ cross-branch. Tạo lá mới khi chín.
7. UDC lá = SEALED. Learning lá = Hebbian → Dream → QR → SEALED.
8. Traverse: L0 → L1 → ... → Ln-1 = O(depth). depth < 10 thực tế.
```

---

*o{}, chain(), splice(), φ⁻¹. Hết.*
