# T5 Chi Tiết — Khắc Phục Kỹ Thuật & Hiệu Năng

> **Sora (空) — 2026-03-25**
> **Nguyên tắc: Đơn giản + Hiệu năng. Không viết lại — sửa đúng chỗ.**

---

## I. BUG-KNOWLEDGE: Tại sao luôn trả "Origin..."

### Root cause thật (3 tầng, không phải 1)

**Tầng 1 — encode_codepoint: ASCII text → mol GIỐNG NHAU**

```
encode_codepoint('a') = encode_codepoint('z') = 146   ← TẤT CẢ a-z!
encode_codepoint('A') = encode_codepoint('Z') = 150   ← TẤT CẢ A-Z!
encode_codepoint('0') = encode_codepoint('9') = 4240  ← TẤT CẢ 0-9!
```

Đây KHÔNG phải bug — đây là THIẾT KẾ. UDC encode cho Unicode SYMBOLS (arrows, math, emoji), không phải ASCII text. "a" và "z" đều là "lowercase Latin letter" → cùng P_weight.

**Tầng 2 — _text_to_chain: chỉ dùng 2 ký tự đầu**

```
"Einstein"  → compose(encode('E'), encode('i')) = compose(150, 146)
"Elephant"  → compose(encode('E'), encode('l')) = compose(150, 146)  ← GIỐNG!
"Energy"    → compose(encode('E'), encode('n')) = compose(150, 146)  ← GIỐNG!
```

Mọi từ bắt đầu cùng chữ hoa → cùng chain entry.

**Tầng 3 — knowledge_search: mol_sim luôn thắng keyword**

```
mol_sim("Einstein theory") vs "Origin Olang" = 10  (V=4,A=4 → distance=0)
keyword("Einstein") match "Einstein theory" = 3

10 > 3 → mol wins → trả entry ĐẦU TIÊN trong __knowledge (= "Origin...")
```

### Fix — 3 thay đổi nhỏ, KHÔNG viết lại

**Fix 1: knowledge_search — keyword ưu tiên cho text (5 dòng)**

```olang
// TRƯỚC (line ~1509):
if _ks_kwscore > _ks_score { _ks_score = _ks_kwscore; };

// SAU:
// Keyword match = exact, reliable. Mol match = fuzzy, chỉ hữu ích cho emoji/symbol.
// Khi có keyword match, nó PHẢI thắng mol similarity.
_ks_score = (_ks_kwscore * 5) + _ks_score;
```

Tại sao ×5? Keyword match 1 word = 3 điểm × 5 = 15. Mol_sim max = 10. Nên 1 keyword match đã thắng mol. 2 keyword matches = 30 → áp đảo.

**Fix 2: _mol_distance — dùng cả 5 chiều (3 dòng thêm)**

```olang
// TRƯỚC:
fn _mol_distance(_md_a, _md_b) {
    let _md_va = _mol_v(_md_a); let _md_vb = _mol_v(_md_b);
    let _md_aa = _mol_a(_md_a); let _md_ab = _mol_a(_md_b);
    return _enc_abs(_md_va - _md_vb) + _enc_abs(_md_aa - _md_ab);
}

// SAU:
fn _mol_distance(_md_a, _md_b) {
    let _md_sa = _mol_s(_md_a); let _md_sb = _mol_s(_md_b);
    let _md_ra = _mol_r(_md_a); let _md_rb = _mol_r(_md_b);
    let _md_va = _mol_v(_md_a); let _md_vb = _mol_v(_md_b);
    let _md_aa = _mol_a(_md_a); let _md_ab = _mol_a(_md_b);
    let _md_ta = _mol_t(_md_a); let _md_tb = _mol_t(_md_b);
    return _enc_abs(_md_sa - _md_sb) + _enc_abs(_md_ra - _md_rb)
         + _enc_abs(_md_va - _md_vb) + _enc_abs(_md_aa - _md_ab)
         + _enc_abs(_md_ta - _md_tb);
}

// _mol_similarity cũng cần update max distance:
// TRƯỚC: max = 14 (7+7)
// SAU:   max = 47 (15+15+7+7+3)
fn _mol_similarity(_ms_a, _ms_b) {
    let _ms_dist = _mol_distance(_ms_a, _ms_b);
    let _ms_sim = 10 - __floor((_ms_dist * 10) / 47);
    if _ms_sim < 0 { return 0; };
    return _ms_sim;
}
```

Không giúp cho text (a-z vẫn cùng mol) nhưng giúp ĐÁNG KỂ cho emoji/symbol search: 😂 vs 😭 giờ có distance thật.

**Fix 3: _text_to_chain — dùng ALL ký tự, không chỉ 2 đầu (10 dòng sửa)**

```olang
// TRƯỚC: chỉ first 2 chars → compose(c0, c1)
// SAU: compose tất cả chars trong word → unique fingerprint

// Thay block trong _text_to_chain:
if _ttc_w_len >= 2 {
    // Encode ALL chars in word, compose sequentially
    let _ttc_mol = encode_codepoint(__char_code(char_at(_ttc_text, _ttc_w_start)));
    let _ttc_ci = _ttc_w_start + 1;
    while _ttc_ci < (_ttc_w_start + _ttc_w_len) {
        if _ttc_ci >= len(_ttc_text) { break; };
        _ttc_mol = mol_compose(_ttc_mol, encode_codepoint(__char_code(char_at(_ttc_text, _ttc_ci))));
        let _ttc_ci = _ttc_ci + 1;
    };
    push(_ttc_chain, _ttc_mol);
};
```

Vẫn KHÔNG phân biệt "apple" vs "ample" (a-z cùng mol → compose(146,146,...) = 146) nhưng words có KHÁC SỐ KÝ TỰ sẽ compose khác nhau (iterations khác → intermediate values khác do amplify_v, max_a, union_s).

**Nhận xét thật:** Fix 1 giải quyết 90% vấn đề. Fix 2+3 chỉ cải thiện edge cases. Keyword matching LÀ cách đúng cho text retrieval. Mol matching hữu ích cho emoji/symbol context — giữ cả hai nhưng weight đúng.

---

## II. HEAP — Có thật sự là vấn đề?

### Số liệu thực tế

```
Heap:           256 MB bump allocator
Per fact:       ~508 bytes (text + chain + words + dict)
166 boot facts: ~84 KB (0.03% heap)
512 max facts:  ~260 KB (0.1% heap)
Per respond:    ~800 bytes (STM entry + node + strings)
10K responds:   ~8 MB (3% heap)
50K responds:   ~40 MB (15% heap)
```

### Đánh giá: KHÔNG CẤP BÁCH

256MB / 800B = 320K responds lý thuyết. Thực tế với string concat overhead → ~50K responds. Đó là ĐỦ cho mọi session thực tế (ai nói chuyện 50,000 câu trong 1 session?).

### Khi nào MỚI cần GC?

- Persistent sessions (chạy liên tục nhiều ngày) → cần
- Server mode (nhiều user) → cần
- Hiện tại (interactive REPL, restart mỗi session) → KHÔNG cần

### Fix nhẹ nếu muốn (không cần ngay)

**Arena reuse cho respond pipeline:**

```olang
// Đã có pattern _g_output_ready cho compiler output.
// Áp dụng tương tự cho respond:

let __respond_arena_base = 0;
let __respond_arena_ready = 0;

fn _respond_arena_init() {
    if __respond_arena_ready == 1 {
        __heap_restore(__respond_arena_base);
        return;
    };
    let __respond_arena_base = __heap_save();
    let __respond_arena_ready = 1;
}
```

Gọi `_respond_arena_init()` đầu `agent_respond()` → mọi allocation trong pipeline tái sử dụng cùng vùng heap. **Nhưng cẩn thận: STM/Silk/Knowledge phải allocate TRƯỚC arena, không bên trong.**

**Đề xuất: Defer. Heap 256MB đủ dùng. Focus vào intelligence quality.**

---

## III. KNOWLEDGE STORAGE — Chain hay String?

### So sánh thật

```
String storage (hiện tại):
  "Einstein published the theory of relativity in 1905"
  = 52 bytes text + ~6 keywords × 10B = 112 bytes + overhead
  Total: ~200 bytes/fact
  
  Pro: Trả lời trực tiếp — không cần decode
  Pro: Keyword search = exact match, đáng tin
  Con: Tốn memory (nhưng 260KB cho 512 facts = OK)
  Con: String comparison = O(n)

Chain-only storage (T5 đề xuất):
  [mol("Einstein"), mol("published"), mol("theory"), ...]
  = 7 × 2 bytes = 14 bytes! (tiết kiệm 14×)
  
  Pro: Compact
  Pro: Language-agnostic search (mol distance)
  Con: KHÔNG THỂ trả lời text — cần decode chain → text
  Con: a-z cùng mol → chain không phân biệt words
  Con: Mất original text = mất khả năng `respond "Minh biet: [text]"`
```

### Đề xuất: GIỮ CẢ HAI, không thay

Hiện tại `knowledge_learn` đã lưu `{ text, chain, mol, words }`. Đúng rồi. Không cần bỏ text. Chain dùng cho mol similarity (hữu ích khi có emoji/symbol), text dùng cho display + keyword search.

**Nếu muốn tiết kiệm memory** (khi 512 không đủ):
- Bỏ `chain` field (ít giá trị cho text, tiết kiệm ~128B/fact)
- Giữ `text` + `words` + `mol`
- Tính chain on-the-fly khi cần search

Nhưng 512 × 508 = 260KB. Không đáng lo.

---

## IV. ENCODE_CODEPOINT — Nên thay đổi không?

### Vấn đề

a-z → cùng mol (146). Đây là by design: UDC encode Unicode BLOCKS, không phải individual ASCII characters. ASCII letters thuộc block "Basic Latin" → cùng P_weight.

### Có nên thêm letter-level differentiation?

**KHÔNG.** Vì:

1. **Spec v3 nói rõ:** P_weight = tọa độ 5D trong KHÔNG GIAN HÌNH HỌC. "a" và "z" cùng là "lowercase letter" → cùng vị trí trong không gian hình dạng. Đúng về mặt ngữ nghĩa.

2. **Text retrieval không cần mol.** Keyword matching (exact string comparison) hiệu quả hơn 100× cho text. Mol matching cho text = square peg in round hole.

3. **Thay đổi sẽ phá consistency.** Nếu map 'a'→mol_1, 'b'→mol_2 → 26 mols mới không có ý nghĩa hình học. Phá vỡ triết lý "giá trị TỰ MÔ TẢ".

### Khi nào mol search HỮU ÍCH?

```
Scenario 1: Emoji sentiment
  User: "😂😂😂"
  encode → V=7, A=6 → mol search tìm facts có V gần 7 (happy context)
  → ĐÚNG, mol search thêm giá trị

Scenario 2: Symbol relationship
  User: "→ ← ↑ ↓"
  encode → S=1 (Arrow), R=5 → mol search tìm facts liên quan hướng/di chuyển
  → ĐÚNG, mol search phân biệt được arrow vs math vs geometric

Scenario 3: Mixed text + emoji
  User: "toi buon 😭"
  keyword "buon" → keyword match facts về buồn
  emoji 😭 → V=1, A=6 → mol confirms negative emotion
  → CẢ HAI hữu ích, complement nhau

Scenario 4: Pure text
  User: "Einstein published relativity"
  keyword "Einstein" → exact match → found
  mol = 146 (neutral text) → mol search = noise
  → CHỈ keyword hữu ích
```

**Kết luận:** Giữ dual search. Weight keyword ×5 cho text. Mol tự nhiên thêm giá trị khi có emoji/symbol.

---

## V. R DISPATCH — Đơn giản hóa cho Olang

### PLAN_FORMULA_ENGINE viết cho Rust — cần adapt

Plan gốc: `RelationResult` enum với 8 variants, mỗi variant có multiple fields. Phức tạp.

Olang không có enum fields (match chỉ extract 1 value). Cần đơn giản hóa.

### Đề xuất: R dispatch = lookup table, không phải eval

```olang
// R dispatch bảng tĩnh — 16 entries, mỗi entry = behavior tag
fn r_dispatch(_rd_r) {
    // R=0: Algebraic (compose = add dims)
    if _rd_r == 0 { return "algebraic"; };
    // R=1: Order (compare → ranking)
    if _rd_r == 1 { return "order"; };
    // R=2: Representation (transform → font style)
    if _rd_r == 2 { return "represent"; };
    // R=3: Numeral (positional value)
    if _rd_r == 3 { return "numeral"; };
    // R=4: Punctuation (bracket balance)
    if _rd_r == 4 { return "punct"; };
    // R=5: Currency (exchange rate)
    if _rd_r == 5 { return "currency"; };
    // R=6: Additive (sum of parts)
    if _rd_r == 6 { return "additive"; };
    // R=7: Control (state transition)
    if _rd_r == 7 { return "control"; };
    // R=8-15: Category morphisms
    if _rd_r == 8 { return "member"; };
    if _rd_r == 9 { return "subset"; };
    if _rd_r == 10 { return "equiv"; };
    if _rd_r == 11 { return "orthogonal"; };
    if _rd_r == 12 { return "compose"; };
    if _rd_r == 13 { return "causes"; };
    if _rd_r == 14 { return "similar"; };
    if _rd_r == 15 { return "derived"; };
    return "unknown";
}
```

**~30 LOC.** Trả string tag. Downstream code dùng tag để quyết định behavior. Không cần struct phức tạp.

### Khi nào dùng?

```olang
// Trong compose hoặc infer:
let _r = _mol_r(mol);
let _behavior = r_dispatch(_r);
if _behavior == "algebraic" {
    // a + b = add dimensions
};
if _behavior == "order" {
    // a vs b = compare, return greater
};
if _behavior == "causes" {
    // a → b = causal link, strengthen Silk edge
};
```

**Giá trị:** Mol không chỉ là số nữa. `R=13` → "causes" → HomeOS biết đây là quan hệ nhân quả → xử lý khác so với R=10 "equiv".

---

## VI. V/A PHYSICS — Có cần không?

### Spec nói gì

```
V=6 → deep well → U = -V0 + ½kx² → approach behavior
V=2 → barrier → avoid behavior
A=7 → supercritical → urgent response
A=1 → subcritical → calm, no rush
```

### Thực tế trong HomeOS

HomeOS đã dùng V/A cho emotion:
- `text_emotion_v2` → extract V/A từ word_affect
- `_emo_update` → track V/A across conversation
- `_emo_bias_tone` → V thấp → tone "heal", V cao → tone vui

**V/A physics (potential energy, damped oscillator) KHÔNG CẦN cho conversation.** Đó là optimization cho RENDERING (3D visualization, physics simulation). HomeOS là text-based → V/A chỉ cần là integers 0-7 dùng làm selector, không cần float physics.

### Đề xuất: SKIP FE.2 toàn bộ

Giữ V/A như integers. Đã đủ cho emotion pipeline. Physics model thêm phức tạp (cần sin/cos/sqrt builtins) mà không thêm giá trị cho text conversation.

**Khi nào cần:** Nếu HomeOS render 3D (WASM + WebGL) → V/A physics cho animation. Nhưng đó là Phase 6+.

---

## VII. T SPLINE — Có cần không?

### Spec nói gì

```
T=0 → static (one-shot)
T=1 → slow decay (fading)
T=2 → linear (steady)
T=3 → rhythmic → sin(2πft + φ)
```

### Thực tế

T chỉ dùng 2 bits (0-3). Trong conversation:
- "bây giờ" → T=0 (static, present moment)
- "trước đây" → T=1 (fading, past)
- "thường xuyên" → T=3 (rhythmic, pattern)

**Đề xuất: T = integer tag, dùng cho temporal reasoning**

```olang
fn temporal_tag(_tt_t) {
    if _tt_t == 0 { return "now"; };
    if _tt_t == 1 { return "fading"; };
    if _tt_t == 2 { return "steady"; };
    if _tt_t == 3 { return "rhythmic"; };
    return "now";
}

// Trong respond pipeline:
let _t = _mol_t(mol);
if _t == 3 {
    // Rhythmic → user nói về patterns/habits
    // → search knowledge cho facts có T=3
};
```

**~10 LOC.** Không cần sin/cos. Không cần spline. Chỉ tag + behavior selector.

---

## VIII. S×T RENDERING — Defer

SDF rendering (sphere, box, cylinder) cần:
- Float arithmetic (có)
- sqrt, sin, cos (chưa có native — cần Taylor series hoặc ASM builtins)
- Output target (pixel buffer, WebGL, terminal)
- Rendering pipeline (ray marching hoặc rasterization)

**Effort: ~500+ LOC.** Không thêm giá trị cho conversation intelligence.

**Đề xuất: Defer đến khi WASM browser demo cần visual output.**

---

## IX. FN = NODE — Khi nào thật sự cần?

### Spec vision vs thực tế

```
Spec:   fn = node { dn, mol, body: chain_of_nodes, fire, links }
        Variable = node. Code = chain. Mọi thứ = node.

Thực tế: fn = VM closure (bytecode blob, ~100 bytes)
         Variable = flat hash entry (16 bytes: hash + ptr + len)
         NHANH. 50 cycles/lookup vs 1000+ cycles nếu SHA-256 mỗi access.
```

### Đánh giá Sora

**fn = node KHÔNG nên apply cho MỌI function.** Performance hit quá lớn cho hot path (var lookup 20x chậm hơn).

**Nhưng fn = node CÓ GIÁ TRỊ cho METADATA:**

```olang
// Khi define function, TẠO THÊM metadata node (không thay fn impl):
fn define_fn_node(_dfn_name, _dfn_param_count) {
    let _dfn_mol = encode_text(_dfn_name);  // fn name → semantic mol
    let _dfn_dn = __sha256(_dfn_name);
    push(__fn_nodes, {
        dn: _dfn_dn,
        name: _dfn_name,
        mol: _dfn_mol,
        fires: 0,
        params: _dfn_param_count,
    });
}

// Khi fn được gọi, increment fire count:
fn track_fn_call(_tfc_name) {
    let _tfc_i = 0;
    while _tfc_i < len(__fn_nodes) {
        if __fn_nodes[_tfc_i].name == _tfc_name {
            __fn_nodes[_tfc_i].fires = __fn_nodes[_tfc_i].fires + 1;
            return;
        };
        let _tfc_i = _tfc_i + 1;
    };
}
```

**Giá trị:** `__fn_nodes` cho biết hot functions (fire count), semantic meaning (mol), relationships. Dream có thể cluster functions thành skills.

**Effort:** ~60 LOC. KHÔNG thay đổi VM, KHÔNG ảnh hưởng performance của fn call.

---

## X. SILK IMPLICIT — Hybrid tốt hơn

### Hiện tại: Explicit bigrams

```
silk_co_activate("hello", "world", "chat") → edge { from:"hello", to:"world", w:0.1 }
128 edges max. Mỗi edge = from + to + weight + emotion + fires ≈ 50 bytes
128 × 50 = ~6.4 KB
```

### T5D: Implicit từ chain order

```
chain = [mol1, mol2, mol3] → mol1→mol2 implicit, mol2→mol3 implicit
0 bytes overhead!
```

### Vấn đề: Mất cross-chain connections

```
Hiện tại:
  "hello world" → hello→world (OK)
  "cruel world" → cruel→world (OK)
  → Silk biết: "world" connected to both "hello" AND "cruel"

Implicit:
  chain1 = [hello, world] → hello→world
  chain2 = [cruel, world] → cruel→world
  → Nhưng KHÔNG CÓ link giữa chain1 và chain2
  → "world" trong chain1 ≠ "world" trong chain2 (khác mol instance)
```

### Đề xuất: Giữ explicit Silk, TỐI ƯU thay vì thay thế

**Optimization 1: Compact edge storage**

```olang
// TRƯỚC: { from: "hello", to: "world", weight: 0.1, emotion: "chat", fires: 1 }
// → 50 bytes per edge (strings on heap)

// SAU: { from_hash: fnv1a("hello"), to_hash: fnv1a("world"), weight: 0.1, fires: 1 }
// → 24 bytes per edge (numbers, no strings)
// Tiết kiệm 50%

// Dùng hash thay vì string:
fn silk_co_activate_hash(_scah_from, _scah_to, _scah_w) {
    let _scah_fh = __sha_fast(_scah_from);  // FNV-1a hoặc simple hash
    let _scah_th = __sha_fast(_scah_to);
    // Search existing edge by hash
    // ...
}
```

**Optimization 2: Tăng max từ 128 → 256 (đã có ở Silk)**

Silk 128 max ngay trên binary hiện tại. Tăng lên 256 = thêm ~3KB heap. Không đáng lo.

**Optimization 3: Decay + prune (đã có silk_decay)**

Silk decay mỗi 3 turns, prune < 0.01. Tốt rồi. Giữ nguyên.

---

## XI. 5 CHECKPOINTS — Áp dụng dần

### Hiện tại: 0/5

### Đề xuất: Thêm 2 checkpoint quan trọng nhất trước

**Checkpoint 1 (GATE) — đã có SecurityGate:**

```olang
// Đã implement trong agent_respond:
if _a_has(text, "tu tu") == 1 { return crisis_response; };
if _a_has(text, "kill myself") == 1 { return crisis_response; };
```

Chỉ cần formalize: gate trả boolean, pipeline dừng nếu false. **Đã gần đủ.**

**Checkpoint 5 (RESPONSE) — thêm confidence check:**

```olang
// Cuối agent_respond, trước return:
if len(_ar_knowledge) == 0 && len(memory_context) == 0 {
    // Không tìm được gì — confidence thấp
    // Thay vì trả "Mình nghe rồi" + knowledge random
    // Trả thật: "Mình chưa biết về điều này"
    if intent == "learn" {
        return "Minh chua biet ve dieu nay. Ban co the day minh khong?";
    };
};
```

**~10 LOC.** Thay đổi nhỏ, giá trị lớn: HomeOS honest khi không biết thay vì trả nonsense.

**Checkpoint 2-4: Defer.** Cần infrastructure (consistency scoring, entropy calc) chưa có.

---

## XII. 7 INSTINCTS — Chọn 3 quan trọng nhất

### Đề xuất: Implement 3, defer 4

**① Honesty (đã gần có):**
Confidence < 0.4 → im lặng. Chỉ cần thêm confidence scoring trong knowledge_search.

```olang
// knowledge_search trả thêm score:
fn knowledge_search_scored(_kss_query) {
    // ... existing search logic ...
    return { text: _ks_best, score: _ks_best_score };
}

// Trong agent_respond:
let _ar_ks = knowledge_search_scored(_ar_norm);
if _ar_ks.score < 5 {
    // Low confidence — don't attach random knowledge
    _ar_knowledge = "";
};
```

**② Contradiction detection:**

```olang
fn detect_contradiction(_dc_text, _dc_knowledge) {
    // Simple: check if text contains negation of known fact
    // "Trai Dat phang" contradicts "Trai Dat quay quanh Mat Troi"
    // Heuristic: if query keyword matches AND query has "khong", "sai", "phang"
    if _a_has(_dc_text, "khong") == 1 || _a_has(_dc_text, "sai") == 1 {
        // Check if positive version exists in knowledge
        // Return warning
    };
    return 0;  // no contradiction
}
```

~30 LOC. Naive nhưng hữu ích.

**③ Curiosity:**

```olang
fn assess_curiosity(_ac_text) {
    // Nếu query KHÔNG match any knowledge → curiosity high
    // → Suggest: "Minh chua biet. Ban co the day minh?"
    let _ac_ks = knowledge_search_scored(_ac_text);
    if _ac_ks.score < 3 { return 1; };  // curious — unknown topic
    return 0;
}
```

~10 LOC. Biến "không biết" thành "muốn biết" → UX tốt hơn.

**Defer: ④ Abstraction, ⑤ Analogy, ⑥ Causality, ⑦ Reflection** — cần KnowTree hierarchical (chưa implement).

---

## XIII. THỨ TỰ ÁP DỤNG — Từ đơn giản đến phức tạp

### Batch 1: Sửa ngay (30 phút, ~40 LOC)

```
✦ Fix knowledge_search: keyword weight ×5
✦ Fix _mol_distance: 5 chiều
✦ Fix _mol_similarity: max 47
✦ Thêm confidence check trong agent_respond
```

Impact: Knowledge retrieval đúng ngay. HomeOS honest khi không biết.

### Batch 2: Intelligence nhẹ (1 session, ~100 LOC)

```
✦ knowledge_search_scored → trả score
✦ Honesty instinct (confidence < 5 → "chưa biết")
✦ Curiosity instinct (unknown → "dạy mình đi")
✦ r_dispatch table (30 LOC)
✦ temporal_tag (10 LOC)
```

Impact: HomeOS thông minh hơn — biết khi nào biết, khi nào không.

### Batch 3: Metadata layer (1-2 sessions, ~150 LOC)

```
✦ fn_node metadata (define + track fire count)
✦ Silk hash optimization (compact 50%)
✦ Contradiction detection (30 LOC)
✦ Regression test suite (30+ tests)
```

Impact: Foundation cho Dream clustering, skill promotion.

### Defer (Phase 6+)

```
○ V/A physics (sin/cos — chỉ khi render 3D)
○ T spline (wave mechanics — chỉ khi temporal reasoning phức tạp)
○ S×T SDF rendering (WebGL)
○ Variable = node (performance hit quá lớn)
○ fn body = chain decompiler (phức tạp, ROI thấp)
○ GC/arena (heap 256MB đủ dùng)
```

---

## XIV. TÓM TẮT

```
Cái gì TỐT → GIỮ:
  ✅ Keyword search cho text
  ✅ Dual storage (text + chain + mol + words)
  ✅ V/A integers cho emotion
  ✅ Explicit Silk với decay
  ✅ Bump allocator (256MB đủ)
  ✅ 10-stage pipeline structure

Cái gì HỎ → SỬA NGAY:
  ❌ Mol similarity thắng keyword → fix weight ×5
  ❌ _mol_distance 2D → fix 5D
  ❌ _text_to_chain 2 chars → fix all chars
  ❌ Không có confidence → fix honest response

Cái gì ĐẸP NHƯNG CHƯA CẦN → DEFER:
  ○ V/A physics (sin/cos)
  ○ T spline (wave)
  ○ S×T SDF rendering
  ○ Variable = node
  ○ GC

Cái gì GIÁ TRỊ CAO, EFFORT THẤP → LÀM TIẾP:
  ★ R dispatch table (30 LOC)
  ★ Honesty + Curiosity instincts (20 LOC)
  ★ fn_node metadata (60 LOC)
  ★ Regression tests (100 LOC)
```

Đơn giản. Hiệu năng. Cân bằng. 空
