# T5 Analysis — UDC-native Olang

> **Sora (空) — 2026-03-25**
> **Nghiên cứu từ: TASKBOARD T5, docs/UDC_DOC (13 files, 8,232 LOC), plans/PLAN_FORMULA_ENGINE.md (2,114 LOC), docs/HomeOS_SPEC_v3.md (956 LOC)**

---

## I. HIỆN TRẠNG — Hệ thống đã có gì

```
Binary: 943KB → 965KB (mới nhất, BUG-INDEX fixed, lambda expressions)
Tests:  16/16 ✅
LOC:    ~20,000 (VM 5,634 + Bootstrap 3,542 + HomeOS 9,416 + stdlib)

Đã hoạt động:
  ✅ Encoder: text → mol u16 (block-range mapper, 59 blocks)
  ✅ Compose: mol_compose (LCA trung bình các chiều)
  ✅ Knowledge: 166 facts auto-learn, keyword + mol similarity search
  ✅ 10-stage respond pipeline (alias→encode→node→DN/QR→decode→output)
  ✅ STM 32, Silk 128, Knowledge 512, Nodes 256
  ✅ Self-build 3 generations
  ✅ 100% self-compile (44/44 HomeOS + 4/4 bootstrap)

Chưa hoạt động:
  ❌ Mol chỉ là SỐ — không dispatch theo R/V/A/T formulas
  ❌ Knowledge lưu nguyên STRING (10KB/fact) thay vì UDC chain (vài bytes)
  ❌ Fn = opaque bytecode blob, không phải inspectable node chain
  ❌ Silk = explicit bigrams thay vì implicit chain order
  ❌ 0/5 Checkpoints
  ❌ 0/7 Instincts (chỉ có SecurityGate)
  ❌ 0/3 Intelligence (Immune Selection, Homeostasis, DNA Repair)
```

---

## II. T5 LÀ GÌ — Triết lý cốt lõi

### Câu hỏi: Tại sao cần T5?

Hiện tại HomeOS **biết** 8,846 UDC formulas (18KB KnowTree) nhưng **không dùng**. Mol chỉ là con số so sánh bằng `>` `<` `==`. Giống như biết bảng tuần hoàn nhưng không biết hóa học.

T5 = **kích hoạt** 8,846 formulas. Mỗi u16 không chỉ là index — nó là **chương trình**.

### Phương trình thống nhất (Spec v3 §XI)

```
HomeOS(input) = self_correct(
                  splice(
                    chain( f(p₁), f(p₂), ..., f(pₙ) ),
                    position,
                    context
                  ),
                  φ⁻¹
                )

f(pᵢ) = SDF — 1 trong 8,846 hàm gốc
chain  = xâu chuỗi → 2 bytes/link (u16)
splice = cắt/ghép chuỗi
φ⁻¹ ≈ 0.618 = ngưỡng duy nhất
```

4 thao tác. Mọi thứ. Giống DNA chỉ cần 4 nucleotides.

### T×S Insight (PLAN_FORMULA_ENGINE)

```
S = WHAT shape (18 SDF primitives: sphere, box, cylinder, torus...)
T = HOW (amplitude, frequency, phase → spline parameters)
S × T = hình dạng CỤ THỂ với kích thước, vị trí, chuyển động

Ví dụ:
  S=SPHERE, T={amp=3.0}           → quả cầu lớn (r=3.0)
  S=BOX, T={amp=2.0, freq=0.1}   → hộp rung
  "nhà" = compose(BOX+T{4,3,4}, PYRAMID+T{5,2,5}, BOX+T{1,2,0.1})

BẤT KỲ concept nào cũng có thể RENDER thành hình 3D.
5D → 1 vật thể hoàn chỉnh: hình + quan hệ + cảm xúc + năng lượng + thời gian.
```

---

## III. 4 PHASE CỦA T5

### Phase 5A — Stabilize (ĐÃ GẦN XONG)

```
ST.1 BUG-INDEX/BUG-SORT:  ✅ DONE (vừa fix)
ST.2 map/filter/reduce:    TODO — stdlib functional pipeline
ST.3 min/max/any/all:      TODO — utility functions
ST.4 Regression tests:     TODO — 30+ tests

Đánh giá: ST.1 done, ST.2-4 là ~200 LOC Olang đơn giản.
Kira có thể làm trong 1 session.
```

### Phase 5B — Node-native (TRỌNG TÂM)

**Mục tiêu:** 1 LOC = 1 node = 16 bits. Mọi thứ = node trong KnowTree.

```
ND.1 Molecule literal:    mol(0x2100) hoặc @● → u16 trực tiếp trên VM stack
ND.2 Extract S/R/V/A/T:  mol.s(), mol.r() → bit extract (đã có trong Olang, cần ASM builtin)
ND.3 Chain = u16 array:  compact storage, mỗi link 2 bytes thay vì 16B
ND.4 Node type native:   node { dn, mol, fire, links } — mọi fn/var/fact = node
ND.5 Knowledge → chain:  thay string (10KB) → UDC chain (vài bytes) — tiết kiệm 100x
ND.6 Variable = node:    let x = 42 → node { dn=hash("x"), mol=encode(42) }
```

**Đánh giá Sora:**

ND.1-2: Straightforward. Mol literal cần parser support (`@●` syntax) + semantic emit PushMol opcode (đã có!). Extract đã có trong Olang (`_mol_s`, `_mol_v`...), chỉ cần ASM builtins cho performance.

ND.3: Chain = `[u16, u16, ...]`. Hiện tại array = 16B/entry (ptr:8 + len:8). Cần `u16_array` type mới hoặc pack 8 u16 vào 1 VM entry. **Lớn nhất của T5B.**

ND.4-5: **Paradigm shift.** Knowledge lưu string → lưu chain:
```
Hiện tại: { text: "Einstein published relativity in 1905", words: [...], mol: 146 }
           = ~50 bytes text + array overhead

T5B:      { chain: [mol("Einstein"), mol("published"), mol("relativity"), mol("1905")], mol: 146 }
           = 4 × 2 = 8 bytes
           Tiết kiệm 6x — và search bằng mol distance thay vì string match
```

ND.6: Variable = node — triết lý đẹp nhưng performance overhead lớn. SHA-256 mỗi variable access = chậm 1000x. **Gợi ý: defer, giữ flat var_table cho performance.**

### Phase 5C — Formula dispatch (PHỨC TẠP NHẤT)

**Mục tiêu:** Đọc mol → biết công thức → biết hành vi.

```
FE.1 R dispatch: 16 relation types
     R=0 Algebraic (group/ring/field)
     R=1 Order (partial/total)
     R=2 Representation (font transform)
     R=3 Numeral (positional encoding)
     R=4 Punctuation (PDA push/pop)
     R=5 Currency (linear map)
     R=6 Additive (Roman numerals)
     R=7 Automaton (state transition)
     R=8-15 Category morphisms (Member, Subset, Compose, Causes...)

FE.2 V/A physics
     V=0→flat=indifferent, V=6→deep well=approach, V=7→barrier=avoid
     A=0→dead=frozen, A=7→supercritical=explosive

FE.3 T spline parameters
     T=0→static, T=1→slow decay, T=2→linear, T=3→rhythmic (sin wave)

FE.4 S×T rendering
     18 SDF × T params = vô hạn hình dạng
     f(p)=|p|-r (sphere), f(p)=max(|p|-b,0) (box), ...

FE.5 42 UDC encode formulas
     Master encoder + 5 dimension encoders + 36 group classifiers
```

**Đánh giá Sora:**

FE.1: Cần ~120 LOC Olang. 16 nhánh switch trên R value. Mỗi nhánh trả về behavior descriptor. **Khả thi ngay** — PLAN_FORMULA_ENGINE đã design xong.

FE.2: ~80 LOC. V/A → physics model (potential energy, damped oscillator). Cần `__sqrt`, `__sin`, `__cos` builtins hoặc Taylor approximation trong Olang.

FE.3-4: ~160 LOC. T → spline knots. S × T → SDF evaluation. Cần float arithmetic (đã có f64 trong VM).

FE.5: **Đây là encoder.ol hiện tại** — block-range mapper đã encode 59 blocks → P_weight. Cần mở rộng từ block-level → character-level precision cho 8,846 UDC.

### Phase 5D — Lego composition (ĐẸP NHẤT)

**Mục tiêu:** fn = chain of nodes. Compose fn từ UDC blocks.

```
LG.1 fn → inspectable node chain
     fn body = [node(Load,"x"), node(PushNum,1), node(Call,"add")]
     Mỗi instruction = 1 node with mol → fn có cảm xúc!

LG.2 Compose fn từ UDC blocks
     fn = compose(node1, node2, node3) — Lego assembly
     "sort" = compose(compare_node, swap_node, loop_node)

LG.3 Silk = implicit từ chain order
     [A, B, C] → A→B (w=1.0), B→C (w=1.0) — 0 bytes overhead
     Thay explicit bigrams (hiện tại 128 edges × 5 fields each)

LG.4 Dream = cluster fn → skill
     Repeated fn patterns → promote to named skill node
     "greeting" skill = {parse_hello, detect_language, compose_reply}

LG.5 Self-describe: fn biết mình là gì
     fn.mol → cảm xúc (heal() V=6, delete() V=2)
     fn.fire → hot function (often called)
     fn.links → related fns (add↔subtract)
```

**Đánh giá Sora:**

LG.1-2: Cần bytecode introspection — decompile bytecode thành node chain. **Phức tạp** nhưng doable: iterate bytecode, mỗi opcode → 1 node.

LG.3: **Đây là optimization lớn nhất.** Silk hiện tại = 128 explicit edges × (from + to + weight + emotion + fires) = ~2KB. Chain-based Silk = 0 bytes (thứ tự trong chain IS the relationship). **Nhưng mất cross-chain connections** — cần hybrid: implicit intra-chain + explicit inter-chain.

LG.4-5: Downstream từ LG.1-3. Cần node infrastructure trước.

---

## IV. THỨ TỰ THỰC HIỆN ĐỀ XUẤT

### Sprint 1: 5A Complete + 5B Foundation (1-2 sessions)

```
① ST.2-4: map/filter/reduce/min/max + regression tests (~200 LOC)
② ND.1: Molecule literal syntax (@● hoặc mol(0x2100)) — parser + semantic
③ ND.2: ASM builtins __mol_s, __mol_r, __mol_v, __mol_a, __mol_t
         (extract từ u16, 5 functions × ~10 LOC ASM each = 50 LOC)
④ FE.1: R dispatch table trong Olang — 16 relation types (~120 LOC)
⑤ Fix BUG-KNOWLEDGE: _mol_distance dùng cả 5D, keyword weight 5×
```

### Sprint 2: Knowledge → Chain (1 session)

```
⑥ ND.3: u16 chain type (pack multiple u16 vào array entries)
⑦ ND.5: knowledge_learn → store chain instead of string
⑧ ND.5: knowledge_search → mol distance trên chain (language-agnostic!)
⑨ Test: learn "Einstein relativity" → chain [mol_E, mol_rel]
         respond "physics theory" → tìm Einstein qua mol proximity
```

### Sprint 3: Formula Engine Core (2 sessions)

```
⑩ FE.2: V/A physics — potential energy + damped oscillator behavior
⑪ FE.3: T spline — sin/cos approximation + temporal patterns
⑫ FE.4: S×T → SDF evaluation (18 primitives × T params)
⑬ SC.3: 7 Instincts (Honesty, Contradiction, Causality, Abstraction,
                       Analogy, Curiosity, Reflection)
⑭ SC.16: 5 Checkpoints (Gate, Encode, Infer, Promote, Response)
```

### Sprint 4: Lego Composition (2+ sessions)

```
⑮ LG.1: Bytecode → node chain decompiler
⑯ LG.3: Hybrid Silk (implicit intra-chain + explicit inter-chain)
⑰ LG.2: Compose API — compose(node1, node2) → new_fn
⑱ LG.4: Dream cluster → skill promotion
⑲ SC.4-6: Immune Selection + Homeostasis + DNA Repair
```

---

## V. RỦI RO VÀ CÂU HỎI MỞ

### Rủi ro kỹ thuật

1. **Heap exhaustion:** Chain-based knowledge cần nhiều small allocations. Bump allocator không free → leak nhanh hơn string storage. **Cần arena allocator hoặc GC.**

2. **u16 precision:** 16 bits = 65,536 buckets. Nhưng P_weight chỉ dùng 16 bits (S:4+R:4+V:3+A:3+T:2). Nhiều concept khác nhau → cùng mol → collision. **Cần chain (sequence of mols) để disambiguate.**

3. **Performance:** Mỗi mol operation = bit extract + lookup. Chain comparison = O(n×m). Với 512 knowledge entries × 8 mols/entry × search per respond → ~4000 comparisons. Acceptable? Chưa benchmark.

4. **ND.6 Variable = node:** SHA-256 mỗi variable access = ~1000 cycles. Hiện tại var_table lookup = ~50 cycles. 20x slowdown không chấp nhận được cho hot path. **Đề xuất: defer ND.6, giữ flat var_table.**

### Câu hỏi thiết kế

1. **Mol literal syntax:** `@●` (Unicode ● = 0x25CF) hay `mol(0x25CF)` hay `#0x25CF`? Cần consistent với Olang style.

2. **Chain interop:** Chain = `[u16]` nhưng VM stack entry = 16 bytes. Pack 8 u16 vào 1 entry (16B) hay dùng array of individual u16?

3. **Backward compatibility:** knowledge_learn hiện trả `{ text, words, mol, chain }`. Đổi sang chain-only → break `learn_file` + `respond` pipeline. Cần migration path.

4. **Formula purity:** Spec nói "giá trị TỰ MÔ TẢ — không cần ai giải thích". Nhưng R dispatch cần lookup table. Đó có phải "giải thích" không? Hay lookup table chính là "đọc giá trị → biết công thức"?

---

## VI. KẾT LUẬN

T5 là **paradigm shift** từ "text processing" sang "formula computation". Hiện tại HomeOS xử lý text → trả text. T5 biến nó thành: encode text → chain of formulas → dispatch by dimension → compose → decode → text.

**Khả thi?** Có. Foundation đã có: encoder, mol, compose, knowledge, 10-stage pipeline. T5 = thay nội dung bên trong mỗi stage, không đổi architecture.

**Effort?** 4 sprints × 1-2 sessions = 4-8 AI sessions. ~1000-1500 LOC Olang mới.

**Value?** Knowledge tiết kiệm 100x storage. Search language-agnostic (mol proximity thay vì keyword match). Fn có metadata (cảm xúc, fire count, links). HomeOS biến từ chatbot thành knowledge engine.

**Rủi ro lớn nhất:** Heap management. Bump allocator + chain storage = memory bomb. Cần ít nhất arena reuse (đã có `_g_output_ready` pattern) mở rộng cho knowledge.

```
Spec nói: "Vũ trụ không lưu hình dạng. Vũ trụ lưu công thức."
T5 = dạy HomeOS đọc công thức. 空
```
