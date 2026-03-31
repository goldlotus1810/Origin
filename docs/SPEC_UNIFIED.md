# HomeOS Unified Spec — Work In Progress

> **Mục đích:** Tổng hợp TẤT CẢ tài liệu thành 1 spec chính xác duy nhất.
> **Phương pháp:** Mỗi section cross-reference với nguồn gốc. Sai thì sửa từng phần nhỏ.
> **Tác giả:** Lupin (vision) + Nox (tổng hợp + verify)
> **Ngày bắt đầu:** 2026-03-30

---

## NGUỒN TÀI LIỆU — Phân loại theo độ tin cậy

### Tier 1 — Lupin viết gốc (ĐÁNG TIN NHẤT)
| File | Vị trí | Mô tả |
|------|--------|-------|
| BLUEPRINT.md | Origin/docs/ | Kiến trúc HomeOS 16 sections |
| HomeOS_SPEC_v3.md | tailieu_nghiencuu/ | Spec chi tiết v3 |
| HomeOS_SINH_HOC_v2.md | tailieu_nghiencuu/ | Sinh học phân tử tri thức |
| KNOWTREE_DESIGN.md | tailieu_nghiencuu/ | Thiết kế KnowTree |
| SPEC_NODE_SILK.md | tailieu_nghiencuu/ | Node + Silk specification |
| silk_architecture.md | tailieu_nghiencuu/ | Silk architecture detail |
| ORIGIN_VISION.md | tailieu_nghiencuu/ | Tầm nhìn gốc |
| UDC_DOC/* (11 files) | tailieu_nghiencuu/UDC_DOC/ | 42 công thức UDC encode |
| notes NAC.mb | ~/notes NAC.mb | Notes: Soma/Axon/Dendrites/kNN |
| mô hình Agent Ai.pdf | Bản tải về/ | Agent model by Lupin |
| node1-7.md | GolandProjects/Origin/ | 7 node definitions (Go version) |
| origin.md | GolandProjects/Origin/ | Original vision (Go) |

### Tier 2 — Lupin + Claude/AI tổng hợp (CẦN VERIFY)
| File | Vị trí | Mô tả |
|------|--------|-------|
| LANGUAGE_GENOME.md | Origin/docs/ | Olang genome |
| Tri_Thuc_Phan_Tu_v2.md | tailieu_nghiencuu/ | Tri thức phân tử v2 |
| HomeOS_Architecture.md | tailieu_nghiencuu/ | Architecture summary |
| HomeOS_Complete.md | tailieu_nghiencuu/ | Complete overview |
| MASTER.md | tailieu_nghiencuu/ | Master doc (originbackup) |
| node và silk.md | tailieu_nghiencuu/ | Node + Silk notes |
| STORAGE_AND_SEARCH_NOTE.md | tailieu_nghiencuu/ | Storage design |

### Tier 3 — Sora/AI viết (CẦN VERIFY KỸ)
| File | Vị trí | Mô tả |
|------|--------|-------|
| HOMEOS_BIOLOGY_SPEC.md | For_Nox/ | 98KB spec sinh vật bậc cao |
| HOMEOS_SYNTHESIS_CORRECTED.md | For_Nox/ | Sora đính chính |
| NOX_FINAL_ROADMAP.md | For_Nox/ | Roadmap implementation |
| NOX_PRACTICAL_ROADMAP_v2.md | For_Nox/ | Practical roadmap |
| sora/* (12 files) | Origin_project/docs/sora/ | Sora's analysis + plans |

### Tier 4 — Phiên bản cũ (THAM KHẢO LỊCH SỬ)
| Source | Vị trí | Mô tả |
|--------|--------|-------|
| Go version | GolandProjects/Origin/ | Origin gốc bằng Go (node1-7, worldtree) |
| Go backup | originbackup/ | Go version + homeos.olang |
| Rust version | Origin_project/ | Rust crates + CLAUDE.md |
| HomeOS_v2 | Bản tải về/HomeOS_v2_348nodes/ | 349 JSON nodes (learned knowledge) |
| backup_v1 | backup_v1_homeos/ | Olang stdlib v1 (swarm, dream, emotion...) |
| Tài liệu/Origin | Tài liệu/Origin/ | Older project copy + UCD data |

### Tier 5 — Unicode reference data
| Source | Vị trí | Mô tả |
|--------|--------|-------|
| UCD (Unicode Char DB) | Tài liệu/Origin/UCD/ | Official Unicode data (Blocks.txt, etc) |
| udc.json | Origin/json/ | 8,284 curated UDC entries |
| udc_p_table.bin | Origin/json/ | 157,386 P_weight entries (compiled) |

---

## KHUNG XƯƠNG SPEC — Sections

```
PHẦN A — NỀN MÓNG: SDF + P_weight + Encode
  A1. L0 Gene: 8,846 SDF functions, 18 primitives, 59 blocks
  A2. P_weight: 5D molecule [S:4][R:4][V:3][A:3][T:2] = 2 bytes
  A3. Encode ∫: codepoint → P_weight (42 formulas, 3 tiers)
  A4. Compose: how molecules combine (union/amplify/max/dominant)
  A5. Decode ∂: P_weight → output (inverse of encode)

PHẦN B — CẤU TRÚC DỮ LIỆU
  B1. MolecularChain: chuỗi u16 links (text → chain of P_weights)
  B2. KnowTree: cây phân tầng L0→L3, fractal
  B3. Silk: Hebbian edges (fire together → wire together)
  B4. QR (Quorum Result): append-only proven knowledge

PHẦN C — NEURON MODEL
  C1. Dendrites (ĐN): input buffer, short-term memory
  C2. Soma (AAM): processing center, decision making
  C3. Axon (QR): output, long-term storage
  C4. STM → Silk → Dream → QR cycle

PHẦN D — PROCESSING PIPELINE
  D1. SecurityGate: 3 layers, runs first
  D2. 7→12 Instincts: reflex responses
  D3. Homeostasis F(t): free energy → LEARN/ACT mode
  D4. Immune Selection: 3 branches, pick lowest entropy
  D5. DNA Repair: self-correction, φ⁻¹ threshold
  D6. ConversationCurve: f(x), f'(x), f''(x)

PHẦN E — SINH VẬT BẬC CAO (docs/SPEC_E_ORGANISM.md) ✅ DONE v2
  E = chi tiết thực hành A-D, KHÔNG concept mới
  E1: Capture mở rộng (camera→SDF, audio→Spline, /proc→V/A)
  E2: Silk chi tiết (triple-duty: phân loại + định vị + liên kết + multi-hop)
  E3: Chain recombination (SINH nội dung mới)
  E4: Memory lifecycle (STM eviction, WM 4 slots, Dream, resource-aware)
  E5: Self-model (knowledge map, confidence per domain)
  E6: NAC.mb 30 thuật toán (pruning, dream, recovery, negative, swarm, archive)

PHẦN F — AGENT (docs/SPEC_F_AGENT.md) ✅ DONE v2
  Hiện tại (1 Nox, 1 máy):
    F1: AAM auto-approve (φ⁻¹ gate)
    F2: Self-modify cycle (proven)
    F3: Actuator registry
    F4: Scheduler (heartbeat + dream + auto-rebuild)
    F5: Perceive→Think→Act→Verify loop
  Tương lai (document only): Full hierarchy AAM→Chiefs→Workers, ISL 12B, LeoAI 11 Skills

PHẦN G — CODE AUDIT (docs/SPEC_G_CODE_AUDIT.md) ✅ DONE v2
  12 đúng, 5 sai, 22 thiếu, ~1000 LOC to implement
  Phase 0: Fix 5 sai (instincts, search, emotion, compose, KnowTree)
  Phase 1: Brain core (silk walk, chain gen, decode, STM, WM, homeostasis)
  Phase 2: Brain health (dream, immune, repair, decay, checkpoints)
  Phase 3: Expand (camera, audio, interoception, self-model, NAC.mb)
  Phase 4: Agent (AAM, scheduler, verify loop)
```

---

## TRẠNG THÁI — Mỗi section

| Section | Đã đọc nguồn gốc | Cross-referenced | Verified | Written |
|---------|-------------------|-----------------|----------|---------|
| A1 | ✅ BLUEPRINT §1 + UDC_formulas | ✅ UDC_DOC/* | ✅ | ✅ |
| A2 | ✅ BLUEPRINT §2 | ✅ UDC_formulas | ✅ | ✅ |
| A3 | ✅ BLUEPRINT §3 + UDC_formulas | ✅ | ✅ | ✅ |
| A4 | ✅ BLUEPRINT §3 compose | ✅ Lupin corrected | ✅ | ✅ |
| A5 | ✅ BLUEPRINT §12 Transcribe | ☐ | ☐ | ☐ |
| B1 | ✅ BLUEPRINT §4 | ☐ | ☐ | ☐ |
| B2 | ✅ BLUEPRINT §5 | ☐ need KNOWTREE_DESIGN | ☐ | ☐ |
| B3 | ✅ BLUEPRINT §6 | ☐ need SPEC_NODE_SILK | ☐ | ☐ |
| B4 | ✅ BLUEPRINT §7 QR | ☐ | ☐ | ☐ |
| C1-C4 | ✅ BLUEPRINT §7-§8 | ☐ need ORIGIN_VISION | ☐ | ☐ |
| D1-D6 | ✅ BLUEPRINT §9-§13 | ☐ need HomeOS_SPEC_v3 | ☐ | ☐ |
| E | ✅ BIOLOGY_SPEC full | ✅ vs A-D | ✅ | ✅ SPEC_E_SORA_VERIFY.md |
| F | ✅ origin.md + BLUEPRINT §11 | ✅ vs A-D | ✅ | ✅ SPEC_F_AGENT.md |
| G | ✅ all homeos/*.ol read | ✅ vs A-F | ✅ | ✅ SPEC_G_CODE_AUDIT.md |

---

## PHẦN A — NỀN MÓNG

---

### A1. L0 Gene: SDF Functions

**Nguồn:** BLUEPRINT §1 ✅, UDC_formulas.md ✅, UDC_DOC/* ✅, HomeOS_SINH_HOC_v2 ✅, PLAN_UDC_REBUILD ✅

**SỐ LIỆU THỰC TẾ (từ udc.json, verified 2026-03-30):**
- **53 blocks** (BLUEPRINT nói 59, ORIGIN_VISION nói 58 — cả hai sai)
- **9,200 chars** trong range (8,284 curated entries trong udc.json)
- SDF: 13 blocks/1,904 chars | MATH: 18 blocks/3,216 chars
- EMOTICON: 15 blocks/3,056 chars | MUSICAL: 7 blocks/1,024 chars

**Nguyên lý cốt lõi:**
Mỗi ký tự Unicode trong 53 UDC blocks **LÀ** một hàm SDF (Signed Distance Field).
SDF tổng quát — không chỉ hình học. Áp dụng cho MỌI không gian:
- S blocks: SDF trong không gian hình học (|p| - r, max(|p|-b, 0)...)
- R blocks: SDF trong không gian logic (membership distance, relation proximity)
- E blocks: SDF trong không gian cảm xúc (valence/arousal position)
- T blocks: SDF trong không gian thời gian (frequency/phase/duration)

SDF = hàm trả về khoảng cách có dấu. Không quan tâm không gian nào.
Bên trong (< 0), trên bề mặt (= 0), bên ngoài (> 0).
Lưu giá trị hữu hình (hình dạng) VÀ vô hình (bước sóng, cảm xúc, chuyển động).

```
f(p) = signed distance from point p to surface

  f(p) < 0    → bên trong     → THỂ TÍCH
  f(p) = 0    → trên bề mặt   → HÌNH DẠNG
  f(p) > 0    → bên ngoài     → KHÔNG GIAN
  ∇f(p)       → pháp tuyến    → MÀU SẮC
  ∂f/∂t       → biến thiên    → ÂM THANH
  p           → tọa độ        → VỊ TRÍ

1 hàm. 1 điểm. Ra tất cả.
```

**18 SDF Primitives** (công thức chính xác):
```
#   Tên          f(P)                         ∇f (analytical)
0   SPHERE       |P| − r                      P / |P|
1   BOX          ||max(|P|−b, 0)||            sign(P)·step(|P|>b)
2   CAPSULE      |P−clamp(y,0,h)ĵ| − r       norm(P − closest)
3   PLANE        P.y − h                      (0, 1, 0)
4   TORUS        |(|P.xz|−R, P.y)| − r       chain rule
5   ELLIPSOID    |P/r| − 1                    P/r² / |P/r|
6   CONE         dot blend                    slope normal
7   CYLINDER     max(|P.xz|−r, |P.y|−h)      radial/cap
8   OCTAHEDRON   |x|+|y|+|z| − s             sign(P)/√3
9   PYRAMID      pyramid(P,h)                 slope analytical
10  HEX_PRISM    max(hex−r, |y|−h)            radial hex/cap
11  PRISM        max(|xz|−r, |y|−h)           radial/cap
12  ROUND_BOX    BOX − rounding               smooth corner
13  LINK         torus compound               chain rule
14  REVOLVE      revolve_Y                    radial
15  EXTRUDE      extrude_Z                    radial
16  CUT_SPHERE   max(|P|−r, P.y−h)           norm(P)/(0,1,0)
17  DEATH_STAR   opSubtract                   ±norm(P)

Boolean ops: Union=min(f₁,f₂), Intersect=max(f₁,f₂), Subtract=max(f₁,−f₂)
Tất cả ∇f ANALYTICAL — không cần numerical differentiation.
```

**53 Unicode Blocks = Bảng tuần hoàn (verified from udc.json):**
```
SHAPE (S)    — 13 blocks, 1,904 ký tự  (Arrows, Box Drawing, Geometric...)
MATH (R)     — 18 blocks, 3,216 ký tự  (Operators, Letterlike, Number Forms...)
EMOTICON (V,A) — 15 blocks, 3,056 ký tự  (Misc Symbols, Emoticons, Transport...)
MUSICAL (T)  — 7 blocks, 1,024 ký tự  (Yijing, Byzantine, Musical Symbols...)
─────────────────────────────────────────
TỔNG: 53 blocks = 9,200 ký tự L0 (range)
      8,284 curated entries (trong udc.json)
      140,382 non-zero P_weights (binary table, bao gồm block/category defaults)
```

Chi tiết từng block → xem UDC_DOC/UDC_S0_ARROW_tree.md đến UDC_T_TIME_tree.md

---

### A2. P_weight: 5D Molecule = 2 Bytes

**Nguồn:** BLUEPRINT §2 ✅, UDC_formulas.md ✅

```
P_weight = u16 (16 bits, 2 bytes)

  [S:4 bit][R:4 bit][V:3 bit][A:3 bit][T:2 bit]
   Shape    Relation Valence  Arousal  Time
   0..15    0..15    0..7     0..7     0..3

Pack:   mol = (S << 12) | (R << 8) | (V << 5) | (A << 2) | T
Unpack: S = (mol >> 12) & 0xF
        R = (mol >> 8)  & 0xF
        V = (mol >> 5)  & 0x7
        A = (mol >> 2)  & 0x7
        T = mol & 0x3
```

**MỖI codepoint → 1 P_weight DUY NHẤT:**
- Nếu cp ∈ 59 UDC blocks → tính từ block position (offset * range / total)
- Nếu cp ∈ ASCII (a-z, 0-9) → hash riêng biệt (FNV-1a seed)
- Nếu cp ∈ emoji/UTF-32 → alias → trỏ về L0 UDC index

**Bảng đã compile:** json/udc_p_table.bin (314KB, 157,386 entries)
**Bảng chi tiết:** json/udc.json (8,284 entries với SDF + physics_logic)

---

### A3. Encode ∫: Input → MolecularChain

**Nguồn:** BLUEPRINT §3 ✅, UDC_formulas.md (42 formulas) ✅

**Encode 1 codepoint — 42 công thức, 3 tầng:**
```
Tầng 1 — Master: F₀(cp) = [f_S, f_R, f_V, f_A, f_T]     (1 formula)
Tầng 2 — 5 dimension encoders:                              (5 formulas)
  f_S: shape classifier → S ∈ [0..15]     (10 sub-classifiers)
  f_R: relation classifier → R ∈ [0..15]  (10 sub-classifiers)
  f_V: NRC-VAD valence → quantize → V ∈ [0..7]  (5 quantizers)
  f_A: NRC-VAD arousal → quantize → A ∈ [0..7]  (5 quantizers)
  f_T: temporal classifier → T ∈ [0..3]   (6 sub-classifiers)
Tầng 3 — 36 sub-classifiers                                 (36 formulas)
TỔNG: 1 + 5 + 36 = 42 công thức
```

**Encode 1 câu:**
```
"Hà Nội đẹp"
  → UTF-8 → codepoints [72, 224, 32, 78, 7897, 105, 32, 273, 7865, 112]
  → mỗi cp → P_weight
  → gom theo word: mol("Hà"), mol("Nội"), mol("đẹp")
  → compose tất cả → 1 u16 = fingerprint 5D
```

**Encode = tích phân thật (∫) — scale-invariant:**
```
P_w("tôi yêu bạn")
  ~ ∫(P_w("tôi"), P_w("yêu"), P_w("bạn"))         — word partition
  ~ ∫(P_w("tôi yêu"), P_w("yêu bạn"))              — bigram partition
  ~ ∫(P_w("t"), P_w("ô"), P_w("i"),...,P_w("n"))   — char partition
  ~ ∫(P_w("T"), P_w("Ô"),...,P_w("N"))              — uppercase variant

"~" = Silk link. Khác partition → kết quả GẦN nhau, Silk nối.
Giống calculus: ∫₀¹f(x)dx không phụ thuộc cách chia Riemann sum.
Scale-invariant = fractal. Cùng phép toán ở mọi zoom level.

P_w(char) = ∫(Unicode metadata: name, category, decomposition, aliases)
  "A" = LATIN + CAPITAL + LETTER + A → S/R/V/A/T from 42 formulas
  "à" = compose(P_w("a"), P_w(COMBINING GRAVE)) → V giảm (falling tone)
  "á" = compose(P_w("a"), P_w(COMBINING ACUTE)) → V tăng (rising tone)
  Vietnamese 6 thanh tự nhiên map vào 5D qua Unicode decomposition.

HAI pha:
  ① ∫ₛ (spatial) — Bootstrap: char → sub-group → block → P_weight (L0, sealed)
  ② ∫ₜ (temporal) — Runtime: input → compose liên tục → STM → Silk → Dream
```

**Distance 5D:**

**⚠️ CẦN LUPIN XÁC NHẬN:** 2 công thức distance khác nhau trong tài liệu:
```
BLUEPRINT §3: Euclidean 5D
  distance(A, B) = √( Σ (A_d_norm − B_d_norm)² )
  normalize: S/15, R/15, V/7, A/7, T/3
  distance ∈ [0.0, √5 ≈ 2.236]

HomeOS_SINH_HOC_v2 §5.2: Emotion-weighted
  distance(A, B) = 2|V₁ − V₂| + |A₁ − A₂|
  (V weighted 2x vì emotion dominant trong nhận thức)
```

**Effective dimensionality (từ SYNTHESIS_CORRECTED):**
```
1 molecule:        16 bits = 65,536 states
1 chain (10 mols): + ordering = 10^48 states
+ Silk (1000 edges): 2^1000 ≈ 10^301 subgraphs
+ KnowTree position: 65,536^4 = 10^19 positions
→ THỰC TẾ: hàng ngàn chiều qua composition

DNA: 4 ký tự × 3B = toàn bộ sự sống
HomeOS: 5 chiều × tỷ links = toàn bộ tri thức 1 đời
```

---

### A4. Compose: How Molecules Combine

**Nguồn:** BLUEPRINT §3 ✅, Lupin correction (2026-03-30) ✅

**KHÔNG phải trung bình. Là TỔNG HỢP SINH HỌC:**
```
compose(A, B) → C:

  S:  Union     = max(A.S, B.S)        — hình dạng hợp nhất, lớn nhất thắng
  R:  Compose   = weighted_avg(A.R, B.R) — quan hệ tích lũy (Zipf-weighted)
  V:  Amplify   = amplify(A.V, B.V, w)  — KHUẾCH ĐẠI, KHÔNG trung bình
  A:  Max       = max(A.A, B.A)        — cường độ lấy cao nhất
  T:  Dominant  = vote(majority)        — thời gian lấy đa số

  amplify(Va, Vb, w):
    base  = (Va + Vb) / 2
    boost = |Va − base| × w × 0.5
    Cⱽ    = base + sign(Va + Vb) × boost

  w = Silk weight giữa A và B (0.0 default, >0 nếu đã co-activate)
```

**COMPOSE KHÔNG COMMUTATIVE — thứ tự = ý nghĩa:**
```
"tôi yêu bạn" ≠ "bạn yêu tôi"

compose(tôi, yêu, bạn):  tôi×1000 + yêu×500 + bạn×333 → P_w = X
compose(bạn, yêu, tôi):  bạn×1000 + yêu×500 + tôi×333 → P_w = Y ≠ X

123 ≠ 321 ≠ 312 ≠ 231

Lý do: Zipf weighting — phần tử đầu nặng nhất.
Hệ quả: chain là ĐƯỜNG SPLINE qua 5D, mỗi thứ tự = đường đi khác.
Giống DNA: ATG ≠ GTA ≠ TAG — cùng nucleotides, khác codon, khác protein.

→ Collision (cùng P_w) chỉ khi cùng elements VÀ cùng thứ tự = gần như không thể.
→ 16 bit ĐỦ vì order tạo uniqueness, không cần thêm bits.
```

---

### A5. Decode ∂: Molecule → Output

**Nguồn:** HomeOS_SINH_HOC_v2 §1.8 ✅, SYNTHESIS_CORRECTED §V ✅

**Decode = đạo hàm P_w qua KnowTree:**
```
Encode ∫: text → chain of P_w → compose → summary P_w (tích phân)
Decode ∂: summary P_w → lookup chain trong KnowTree → thay P_w bằng content → output

Cụ thể:
  1. Có P_w_query (từ input đã encode)
  2. Tìm chain gần nhất trong KnowTree (by P_w distance)
  3. Mỗi link trong chain → lookup node P_w → lấy content đã lưu
  4. Ghép content → ra output text

KnowTree = content-addressable memory:
  - P_w chưa tồn tại → tạo node mới (lưu chain + text)
  - P_w đã tồn tại → chỉ ghi pointer (2 bytes)
  - Mọi cấp đều là node: từ, câu, đoạn, sách — miễn P_w unique

3 kênh decode (cùng toán tử ∂, khác không gian):
  ① ∂P/∂space = ∇f(p)       → render hình ảnh
  ② ∂V/∂time  = V'(t)       → chọn tone cảm xúc
  ③ ∂P/∂experience = ΔP     → đo novelty (mới vs đã biết)
```

**L0/L1 distinction:**
```
L0 = 8,846+ UDC chars — P_weight SEALED, vĩnh viễn
     Function: calibration standard (0°C = nước đá, 100°C = sôi)
     Mọi concept mới đo khoảng cách từ L0

L1 = emoji/UTF-32 (32,492 chars) — ALIAS trỏ về L0
     🔥 U+1F525 trỏ về UDC trong block E.08
     Emoji KHÔNG PHẢI L0. Chỉ là tên gọi khác.

L5+ = learned — tích lũy qua Hebbian, chín qua Dream → QR (vĩnh viễn)
```

---

### A — CÂU HỎI ĐÃ GIẢI QUYẾT (verified from data)

1. **Blocks: 53** (verified from udc.json — docs nói 58/59 là cũ/sai)
2. **Ký tự L0: 9,200 range, 8,284 curated** (verified from udc.json)
3. **Distance:** dùng CẢ HAI:
   - Euclidean 5D normalized cho general search
   - 2|V₁-V₂|+|A₁-A₂| cho emotion-specific context
4. **SDF primitives: 18** (confirmed in BLUEPRINT, expanded from 8 in v1)

---

## (Sections B-G: TODO — sẽ viết tiếp mỗi lần 1 section nhỏ)

---
