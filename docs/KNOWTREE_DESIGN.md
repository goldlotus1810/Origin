# KnowTree Design — o{} → ∞

> **Nox — 2026-03-25**
> **Từ Spec v2.7, v3.1, UDC_DOC, Sora "Bức Tranh Tổng Thể"**
> **Nguyên tắc: 1 node = 2 bytes. Thứ tự = Silk. 0 bytes overhead.**

---

## I. CẤU TRÚC GỐC: o{}

```
o{} = 1 array u16, tối đa 65,536 phần tử

  Index = vị trí trong array = IMPLICIT (không lưu)
  Value = P_weight (u16) = [S:4][R:4][V:3][A:3][T:2] = 16 bits

  o{}[0x1F525] = P(🔥)     ← O(1) lookup, không hash
  o{}[0x25CF]  = P(●)      ← vị trí IS địa chỉ

  Kích thước 1 nhánh: 65,536 × 2B = 128 KB
  Thực tế dùng: ~10,000 slots (L0 9,584 + learned) = ~20 KB
```

---

## II. PHÂN TẦNG: o{o{o{...}}}

### Tầng L0 — Gốc (5 nhóm)

```
o{ROOT} = array[5]
  [0] = SDF group      (S)    → trỏ đến o{SDF}
  [1] = MATH group     (R)    → trỏ đến o{MATH}
  [2] = EMOTICON group (V+A)  → trỏ đến o{EMO}
  [3] = MUSICAL group  (T)    → trỏ đến o{MUS}
  [4] = LEARNED group  (L5+)  → trỏ đến o{LEARNED}

Kích thước: 5 × 2B = 10 bytes
```

### Tầng L1 — Blocks (59 blocks)

```
o{SDF} = array[14]      ← 14 SDF blocks
  [0]  = S.01 Arrows           (112 chars)
  [1]  = S.02 Misc Technical   (256 chars)
  [2]  = S.03 Box Drawing      (128 chars)
  ...
  [13] = S.14 Control Pictures (64 chars)

o{MATH} = array[21]     ← 21 MATH blocks
  [0]  = M.01 Superscripts     (48 chars)
  [1]  = M.02 Letterlike       (80 chars)
  ...
  [20] = M.21 Arabic Math      (256 chars)

o{EMO} = array[17]      ← 17 EMOTICON blocks
  [0]  = E.01 Enclosed Alpha   (160 chars)
  ...
  [16] = E.17 Misc Sym+Arrows  (256 chars)

o{MUS} = array[7]       ← 7 MUSICAL blocks
  [0]  = T.01 Hexagram         (64 chars)
  ...
  [6]  = T.07 Tai Xuan Jing   (96 chars)

o{LEARNED} = array[N]   ← grows dynamically
  Learned words, facts, skills, agents...

Kích thước L1: (14+21+17+7) × 2B = 118 bytes
```

### Tầng L2 — Sub-ranges (~200)

```
o{S.01_Arrows} = array[~10]    ← sub-groups within Arrows block
  [0] = LEFTWARDS (11 chars)
  [1] = RIGHTWARDS (11 chars)
  [2] = LEFT RIGHT (8 chars)
  [3] = DOWNWARDS (7 chars)
  ...

Mỗi sub-range = 1 array, index = vị trí trong sub

Kích thước L2: ~200 × 2B = 400 bytes
```

### Tầng L3 — Ký tự UDC (9,584 leaves)

```
o{LEFTWARDS_ARROWS} = array[11]
  [0] = P(←)  = [S:1, R:5, V:4, A:4, T:2]
  [1] = P(⇐)  = [S:1, R:5, V:4, A:5, T:2]
  ...

Mỗi ký tự = 1 P_weight = 2 bytes
SEALED at bootstrap. IMMUTABLE.

Kích thước L3: 9,584 × 2B = 19,168 bytes ≈ 19 KB
```

### Tầng L4+ — Learned (dynamic)

```
Khi HomeOS học:
  "Hà Nội" → word node → P_weight computed via compose
  "Einstein" → word node → P_weight computed

Mỗi learned node = 2 bytes (P_weight)
Vị trí trong o{LEARNED} = index

o{LEARNED} grows: push new P_weight → len() increases
```

---

## III. TỔNG DUNG LƯỢNG KNOWTREE

```
L0 Root:          10 bytes
L1 Blocks:       118 bytes
L2 Sub-ranges:   400 bytes
L3 UDC chars:  19,168 bytes
Alias table:    giữ riêng (xem Section VI)
─────────────────────────────
TỔNG TREE:    19,696 bytes ≈ 20 KB

So sánh: L1 cache CPU = 32-64 KB
→ TOÀN BỘ KnowTree nằm trong L1 cache
→ Mọi lookup = O(1) memory access
```

---

## IV. CHAIN = CHUỖI u16

### Cấu trúc

```
Chain = array[u16]

Mỗi u16 = index vào KnowTree
Thứ tự trong array = Structural Silk = 0 bytes overhead

Ví dụ:
  "Hà Nội là thủ đô" = chain[w_HaNoi, w_la, w_thuDo]
  Mỗi w_xxx = u16 index vào o{LEARNED}

  1 link = 2 bytes
  1 câu  = ~10 links = 20 bytes
  1 sách = ~5,000 links = 10 KB
```

### Hierarchical chains (sách)

```
o{book} = chain[chap₁_idx, chap₂_idx, ..., chap₆₀_idx]
  Thứ tự = structural silk = 0 bytes

o{chap₁} = chain[para₁_idx, para₂_idx, ..., para₅₀_idx]

o{para₁} = chain[sent₁_idx, sent₂_idx, ..., sent₅_idx]

o{sent₁} = chain[word₁_idx, word₂_idx, ..., word₁₀_idx]

o{word₁} = chain[char₁_idx, char₂_idx, ..., char₅_idx]
  Hoặc: word₁ = 1 P_weight (compose of chars) → 2 bytes

Mỗi level = 1 array. Index = position = Silk.
```

---

## V. SILK — 2 LOẠI, COST KHÁC NHAU

### Structural Silk = 0 bytes

```
chain[A, B, C] → A→B implicit, B→C implicit

Ribosome chạy thẳng. Engine chạy thẳng.
Thứ tự IS quan hệ. Không lưu gì thêm.

"Hà Nội là thủ đô" → chain order = "Hà Nội" TRƯỚC "là" TRƯỚC "thủ đô"
Silk = vị trí. 0 bytes.
```

### Hebbian Silk = cross-branch connections

```
CHỈ dùng khi nối 2 nodes ở NHÁNH KHÁC NHAU:

  "Scarlett" ở chương 1 ↔ "Scarlett" ở chương 30
  "buồn" ở turn 5 ↔ "mất việc" ở turn 3
  "tình yêu" trong sách A ↔ "tình yêu" trong sách B

Mỗi edge:
  from_idx: u16 (2 bytes)
  to_idx:   u16 (2 bytes)
  weight:   u16 (2 bytes, fixed-point 0.000-1.000 → 0-65535)
  emotion:  u16 (2 bytes, V:8+A:8 tại thời điểm co-activate)
  ─────────────────
  Total: 8 bytes/edge

SilkGraph max: ~5,000 edges × 8B = 40 KB

co_activate(A, B):
  emotion_factor = (|A.V| + |B.V|) / 2 × max(A.A, B.A) / 255
  Δw = emotion_factor × (1 − w_AB) × 0.1
  w_AB ← w_AB + Δw

decay(w, Δt):
  w ← w × φ⁻¹^(Δt/24h)    φ⁻¹ ≈ 0.618
```

---

## VI. ALIAS TABLE — Riêng biệt

```
Alias = text → u16 index vào KnowTree

155,000 aliases (vi + en + ja + zh + emoji sequences)
Mỗi alias entry:
  text_hash: u32 (4 bytes, FNV-1a)
  kt_index:  u16 (2 bytes)
  ─────────────────
  Total: 6 bytes/alias

Alias table: 155,000 × 6B = 930 KB ≈ 1 MB

Lookup: hash(text) → scan bucket → u16 index → O(1) KnowTree
Bloom filter: 200 KB, false positive < 1%, 99% queries = O(1)

Alias KHÔNG NẰM TRONG KnowTree.
Alias = cầu nối ngôn ngữ → tọa độ 5D.
P_weight của alias = COPY từ canonical UDC → SEALED.
```

---

## VII. TÍNH TOÁN: 1 CUỐN SÁCH

### "Cuốn Theo Chiều Gió" (3.2 MB raw text)

```
Layer         Elements     × 2B each    Subtotal
──────────────────────────────────────────────────
Book:         1 chain      × 60 chaps   120 B
Chapters:     60 chains    × 50 paras   6 KB
Paragraphs:   3,000 chains × 5 sents    30 KB
Sentences:    15,000 chains× 10 words   300 KB
Words:        5,000 unique × 1 P_weight 10 KB
──────────────────────────────────────────────────
Chain data:                              346 KB

KnowTree (shared, fixed):               20 KB
Structural Silk:                          0 KB
Hebbian Silk (cross-chapter):            40 KB
──────────────────────────────────────────────────
TOTAL:                                  406 KB

Raw text: 3,200 KB → 406 KB = 7.9× compression
WITH more information (links, Silk, mol, 5D coordinates)
```

### Scale

```
1 cuốn sách  = 406 KB
256 MB heap  = 645 cuốn sách
16 GB disk   = 40,394 cuốn sách

1 đời đọc sách (~200 cuốn) = 79 MB = 31% of 256 MB heap
1 thư viện nhỏ (~5,000 cuốn) = 1.98 GB = disk only
```

---

## VIII. ENCODER ∫ₛ — Bootstrap

```
Bootstrap = ∫ₛ (spatial integration, chạy 1 lần):

  char  = f'(x)           → nguyên tử (9,584 UDC)
  sub   = ∫ₛ chars dx     → compose(chars) → sub P_weight
  block = ∫ₛ subs dx      → compose(subs) → block P_weight
  group = ∫ₛ blocks dx    → compose(blocks) → group P_weight

Từ DƯỚI LÊN. Kết quả SEALED. Không bao giờ thay đổi.

Input: tài liệu UDC_DOC (13 files) + UnicodeData.txt
Output: KnowTree L0-L3 = 20 KB SEALED
```

---

## IX. ENCODER ∫ₜ — Runtime (Learning)

```
Runtime = ∫ₜ (temporal integration, chạy liên tục):

  input → tokenize → alias lookup → u16 indices
  → compose(indices) → new P_weight
  → KnowTree[new_index] = P_weight
  → Hebbian co_activate neighbors

encode("tôi buồn vì mất việc"):
  tokens = ["tôi", "buồn", "vì", "mất", "việc"]
  indices = [alias("tôi"), alias("buồn"), ...]
  P_weights = [KnowTree[idx] for idx in indices]
  composed = compose(P_weights)  ← amplify, NOT average
  ΔV = -0.75 (amplified from individual V values)

  → Store: KnowTree[new_idx] = composed
  → Silk: co_activate("buồn", "mất_việc", emotion_tag)
  → STM: push turn
  → Dream: if chín → promote QR
```

---

## X. DECODER ∂ — Output

```
∂P/∂space      = ∇f(p)   → normal → rendering (SDF)
∂V/∂time       = V'(t)   → tone (ConversationCurve)
∂P/∂experience = ΔP      → novelty (Curiosity instinct)

Search = walk tree:
  query → tokenize → alias → indices
  → find word nodes in KnowTree
  → follow chain links → fact nodes
  → decode fact → output text

O(query_words) NOT O(knowledge_count)
```

---

## XI. IMPLEMENTATION TRONG OLANG

### Data structures

```olang
// KnowTree = nested arrays of u16
let __kt_root = [];          // L0: [sdf_idx, math_idx, emo_idx, mus_idx, learned_idx]
let __kt_blocks = [];        // L1: arrays per group
let __kt_subs = [];          // L2: arrays per block
let __kt_chars = [];         // L3: P_weight per char (u16 as f64)
let __kt_learned = [];       // L4+: learned P_weights

// Chains = arrays of u16 indices
let __kt_chains = [];        // array of chain arrays
// Each chain = [u16, u16, ...] — order = structural Silk

// Hebbian Silk = separate edge list
let __kt_silk = [];          // [{from: u16, to: u16, weight: u16, emo: u16}]

// Alias = hash → index mapping
let __kt_alias_hash = [];    // [hash, hash, ...]
let __kt_alias_idx = [];     // [idx, idx, ...] — parallel arrays
```

### Lookup O(1)

```olang
fn kt_lookup(codepoint) {
    // Direct index: KnowTree[codepoint] = P_weight
    if codepoint < len(__kt_chars) {
        return __kt_chars[codepoint];
    };
    return 0;  // not found
}
```

### Learn (chain + nodes)

```olang
fn kt_learn_v2(text) {
    // 1. Tokenize → words
    // 2. Each word → alias lookup → u16 index
    //    If not found → create new learned node
    // 3. Chain = [word_idx, word_idx, ...]
    // 4. Fact_mol = compose(word_mols) — amplify, NOT average
    // 5. Store chain in __kt_chains
    // 6. Hebbian: co_activate adjacent words (cross-sentence only)
}
```

### Search (tree walk)

```olang
fn kt_search_v2(query) {
    // 1. Tokenize query → word indices
    // 2. For each word index → find chains containing this word
    //    (reverse index: word → chains)
    // 3. Score chains by number of matching query words
    // 4. Return best chain → decode → text
}
```

---

## XII. MIGRATION PATH

```
Hiện tại (knowtree.ol):
  __kt_chars  = [{cp, mol}]           ← dict, ~50 bytes/entry
  __kt_words  = [{text, mol, facts}]  ← dict, ~80 bytes/entry
  __kt_facts  = [{text, words, mol}]  ← dict, ~100 bytes/entry
  Search = scan word array O(n)

Target (this design):
  __kt_chars  = [u16, u16, ...]       ← flat array, 2 bytes/entry
  chains      = [[u16], [u16], ...]   ← arrays of u16
  silk        = [{from, to, w, emo}]  ← 8 bytes/edge
  Search = O(query_words) via reverse index

Migration:
  Step 1: Keep current knowtree.ol working (backward compat)
  Step 2: Build knowtree_v2.ol alongside (new design)
  Step 3: Wire v2 into learn/respond
  Step 4: Remove v1 when v2 proven stable
```

---

## XIII. TỔNG KẾT

```
o{} = array[65,536] × 2B = 128 KB max (20 KB actual L0-L3)

o{ROOT}
  → o{SDF}[14 blocks] → o{Arrows}[10 subs] → o{LEFTWARDS}[11 chars]
  → o{MATH}[21 blocks] → ...
  → o{EMO}[17 blocks] → ...
  → o{MUS}[7 blocks] → ...
  → o{LEARNED}[N dynamic] → words, facts, skills, agents

Chain = [u16, u16, ...] — thứ tự = Structural Silk = 0 bytes
Hebbian = cross-branch edges — 8 bytes/edge, ~40 KB total
Alias = text → u16 index — 6 bytes/entry, ~1 MB total

1 cuốn sách = 406 KB (7.9× compression vs raw text)
256 MB = 645 cuốn sách
1 đời = 79 MB

KnowTree fits in L1 cache.
Every lookup = O(1).
Every search = O(query_words).
Every chain = 2 bytes/link.
Structural Silk = 0 bytes.

Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức.
o{}, chain(), splice(), φ⁻¹.
Hết.
```
