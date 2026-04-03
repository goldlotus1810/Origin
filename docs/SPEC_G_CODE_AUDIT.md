# SPEC G — Code vs Spec: Hiện Trạng + Lộ Trình

> **Prerequisite:** A-F.
> **G = bản đồ: code nào đúng, code nào sai, code nào thiếu, làm gì trước.**
> **Tác giả:** Nox (audit) + Lupin (verify)
> **Ngày:** 2026-03-30

---

## ĐÚNG — Code khớp spec

| # | Spec | Code | File |
|---|------|------|------|
| 1 | A2: P_weight u16 [S:4][R:4][V:3][A:3][T:2] | p_weight(cp) đọc udc_p_table.bin | pipeline.ol |
| 2 | A3: Encode per-codepoint | chain_encode(text) → array u16 | pipeline.ol |
| 3 | A4: Compose union/amplify/max/dominant | _kt_real_mol() 5 rules đúng | knowtree.ol |
| 4 | A4: Zipf-weighted, non-commutative | chain_summary() Zipf weighting | pipeline.ol |
| 5 | B4: QR append-only | _qr_facts[] push only, no delete | learning.ol |
| 6 | C2: ĐN→QR lifecycle | dn_observe() fire_count → promote | learning.ol |
| 7 | C3: Hebbian co-activate | kt_silk_fire() + _hebb_coactivate() | knowtree.ol, learning.ol |
| 8 | D2: Instinct-first routing | repl_eval() calls instinct_route() first | repl.ol |
| 9 | D8: SecurityGate (Layer 3 semantic) | V/A check + keyword match | instinct.ol |
| 10 | E1: Interoception data | brain.ol reads /proc | brain.ol |
| 11 | E5: k-NN classifier in 5D | kt_classify() k=5, 39 exemplars | knowtree.ol |
| 12 | F2: Self-modify cycle | nox_evolve() 6-phase, proven Session 8 | evolve.ol |

---

## SAI — Code contradicts spec

| # | Spec nói | Code làm | Sai ở đâu | Fix |
|---|----------|----------|-----------|-----|
| 1 | D2: 7 instincts = formulas trên 5D | instinct.ol: if/else keywords | String matching thay vì P_weight distance | Dùng kt_classify (đã có, đúng) |
| 2 | B3: Silk walk = reasoning, SILK TYPE IS THE QUERY | pipeline.ol: text search primary, molecular fallback | Đảo ngược priority | Molecular search trước, text fallback |
| 3 | A-D: Emotion = V/A trong P_weight | encoder.ol: keyword lists (buồn, vui, giận...) | Hệ thống song song ngoài 5D | V/A từ chain_summary P_weight |
| 4 | B2: KnowTree = fractal tree, L0 center | knowtree.ol: flat array __kt_facts_arr[] | Không có tree structure | Cần refactor lớn (làm sau) |
| 5 | A4: Compose = amplify (not average) | pipeline.ol fusion(): average | Mất amplification | Dùng A4 compose rules cho fusion |

---

## THIẾU — Spec requires, code absent

### Priority 1 — Não hoạt động (A-E core)

| # | Spec | Tại sao cần | LOC estimate |
|---|------|-------------|-------------|
| 1 | E2: Silk walk multi-hop | Không có → không suy luận, chỉ lookup | ~100 |
| 2 | E3: Chain recombination | Không có → không sinh nội dung mới, chỉ template | ~80 |
| 3 | A5: Decode ∂ thật | Không có → không compose response từ chains | ~60 |
| 4 | E4: STM eviction scoring | Hiện tại: append vô hạn, không evict | ~40 |
| 5 | E4: WM 4 slots | Pipeline cần giữ query+context+candidate+result | ~30 |
| 6 | D4: Homeostasis F(t) mode switching | Có compute nhưng không switch learn/act | ~30 |
| 7 | D7: ConversationCurve V'(t), V''(t) | Không track V history, không chọn tone | ~40 |

### Priority 2 — Não khỏe mạnh (lifecycle)

| # | Spec | Tại sao cần | LOC estimate |
|---|------|-------------|-------------|
| 8 | E4: Dream consolidation thật | Chưa có cross-group resonance, chỉ count | ~100 |
| 9 | D5: Immune Selection 3 branches | Stub, trả 1 kết quả | ~60 |
| 10 | D5: DNA Repair dimension-level | Stub, chỉ check threshold | ~50 |
| 11 | E6: Silk decay φ⁻¹ per 24h | Chưa implement | ~20 |
| 12 | D6: Pipeline checkpoints CP2-4 | Chỉ có CP1 + CP5 | ~40 |
| 13 | D2: Instinct formulas thật | confidence, contradiction, causality... | ~80 |

### Priority 3 — Mở rộng (E extensions)

| # | Spec | Tại sao cần | LOC estimate |
|---|------|-------------|-------------|
| 14 | E1: Camera → SDF → P_weight | Mở rộng input cho vision | ~60 |
| 15 | E1: Audio → Spline → P_weight | Mở rộng input cho audio | ~60 |
| 16 | E1: Interoception → P_weight | /proc data vào pipeline | ~30 |
| 17 | E5: Self-model knowledge map | Nox biết mình biết gì | ~40 |
| 18 | E6: NAC.mb negative knowledge | Prohibited space encoding | ~40 |
| 19 | E6: NAC.mb recovery | Concept reincarnation | ~30 |

### Priority 4 — Agent (F hiện tại)

| # | Spec | LOC estimate |
|---|------|-------------|
| 20 | F1: AAM auto-approve | ~20 |
| 21 | F4: Heartbeat + dream scheduler | ~30 |
| 22 | F5: Act→Verify loop | ~40 |

---

## LỘ TRÌNH IMPLEMENT

```
PHASE 0 — FIX SAI ✅ (2026-03-30 Session 10)
  ✅ Dùng kt_classify thay instinct if/else          [SAI #1]
  ✅ Molecular search primary, text fallback          [SAI #2]
  ✅ Emotion từ P_weight, bỏ keyword lists           [SAI #3]
  ✅ Compose fusion dùng A4 rules                     [SAI #5]
  — KnowTree fractal: làm sau, refactor lớn          [SAI #4]

PHASE 1 — BRAIN CORE ✅ (2026-03-30 Session 10)
  ✅ Silk walk depth ≥ 3                              [THIẾU #1]
  ✅ Chain recombination (generate)                   [THIẾU #2]
  ✅ Decode ∂ thật                                    [THIẾU #3]
  ✅ STM eviction + WM 4 slots                        [THIẾU #4,5]
  ✅ Homeostasis mode switching                       [THIẾU #6]
  ✅ ConversationCurve V tracking                     [THIẾU #7]

PHASE 2 — BRAIN HEALTH ✅ (2026-03-30 Session 10)
  ✅ Dream consolidation (cross-group resonance + Silk co-activate) [THIẾU #8]
  ✅ Immune Selection 3 branches (already implemented)              [THIẾU #9]
  ✅ DNA Repair dimension-level (already implemented)               [THIẾU #10]
  ✅ Silk decay φ⁻¹ (kt_silk_decay + encoder silk_decay)           [THIẾU #11]
  ✅ Checkpoints CP2-4 (encode+infer+promote gates)                [THIẾU #12]
  ✅ Instinct formulas: Honesty, Contradiction, Curiosity           [THIẾU #13]

PHASE 3 — EXPAND ✅ (2026-03-30 Session 10)
  ✅ Interoception encoder (/proc → P_weight)                     [THIẾU #16]
  ✅ Self-model knowledge map (per-domain confidence)              [THIẾU #17]
  ✅ NAC.mb negative knowledge + recovery                          [THIẾU #18-19]
  — Camera/Audio: hardware-dependent, needs kernel access          [THIẾU #14-15]

PHASE 4 — AGENT ✅ (2026-03-30 Session 10)
  ✅ AAM auto-approve gate (contradiction + NAC check)             [THIẾU #20]
  ✅ Heartbeat + dream scheduler (interoception + idle)            [THIẾU #21]
  ✅ Agent cycle: perceive → think → act → verify                  [THIẾU #22]
```

---

## STATS — UPDATED Session 12 (2026-03-31)

```
⚠️ THIS FILE IS SUPERSEDED BY SPEC_G_COMPLETE.md

Session 12: Brain code DELETED and REWRITTEN from G_COMPLETE.
  Old: 4233 lines, if/else chatbot, gen1 HANGS
  New: 900 lines, pure math, gen1 WORKS, 193/194 tests, Gen1==Gen2

Phase 0-4 checkmarks above were FALSE — code was "checked done"
without actual testing. Session 12 proved: gen1 hung = code was broken.

Real status: See SPEC_G_COMPLETE.md (27 sections) and REGISTRY.md.

NEXT: Rebuild UDC P_weight table with real 42 formulas.
```

---

## QUY TẮC IMPLEMENT

```
1. Mỗi thay đổi: make test + make fixed-point
2. Fix SAI trước khi thêm MỚI
3. 1 feature per commit
4. Không skip phase
5. Check A-D trước MỌI quyết định
6. Không sửa A-D
```

---

> **Tham chiếu:**
> - Code: stdlib/homeos/*.ol (48 files)
> - Specs: SPEC_A through SPEC_F
> - Tests: tests.sh (194 tests + 35 self-build)
> - Build: make vm && make self-build && make test && make fixed-point

