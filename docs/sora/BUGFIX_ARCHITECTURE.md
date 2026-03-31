# BUG REPORT: Kiến Trúc Nền Tảng — Origin/Olang

> **Sora (空) — 2026-03-24**
> **Scan: 13,547 LOC across 51 .ol files**
> **Findings: 434 CRITICAL bare variable issues, 1 confirmed infinite loop bug**

---

## I. BUG-VI — CONFIRMED INFINITE LOOP (respond hangs at 3rd call)

### Triệu chứng
```
respond one   → OK
respond two   → OK
respond three → HANGS (timeout, exit code 124)
```

### Root Cause
`_a_has()` dùng bare `i` → collide với caller `stm_topic_repeated()` cũng dùng bare `i`.

Khi `stm_topic_repeated` gọi `_a_has` lần thứ 3 (khi STM có 3 entries), `_a_has` match tại position 0 → return 1 với global `i` = 0. Caller `stm_topic_repeated` increment `i` → 1 → loop lại → i=2 → match → i=0 → **vĩnh viễn**.

### Fix Required
Rename ALL bare vars in `_a_has()`:
```olang
// TRƯỚC (BUG):
fn _a_has(text, word) {
    let i = 0;        // ← collides with caller's i
    let j = 0;        // ← collides with caller's j

// SAU (FIX):
fn _a_has(_ah_text, _ah_word) {
    let _ah_i = 0;
    let _ah_j = 0;
```

Cũng cần fix trong callers: `stm_topic_repeated`, `stm_find_related`, `silk_learn_from_text`, `silk_find_related`, `dream_cycle`.

---

## II. BARE VARIABLE SCAN — 434 CRITICAL ISSUES

### Vấn đề cốt lõi
ASM VM dùng **global var_table** — KHÔNG CÓ block scope. `let i = 0` trong function A sẽ bị function B overwrite nếu B cũng dùng `let i = 0`. Khi A gọi B, `i` của A bị B ghi đè.

### Hot Path (respond pipeline) — 23 functions, FIX NGAY

Đây là các function chạy **mỗi lần** user gõ `respond`:

| File | Function | Bare vars | Impact |
|------|----------|-----------|--------|
| **encoder.ol** | `_a_has()` | i, j, tlen, wlen | **BUG-VI ROOT CAUSE** |
| **encoder.ol** | `stm_topic_repeated()` | i, count | Caller of _a_has → infinite loop |
| **encoder.ol** | `stm_find_related()` | i, wi, ch | Caller of _a_has → infinite loop |
| **encoder.ol** | `silk_learn_from_text()` | i, j, ch, words, current | O(n) per respond, collision risk |
| **encoder.ol** | `silk_find_related()` | i, e, best, best_w | collision with silk_learn |
| **encoder.ol** | `silk_co_activate()` | i, e | collision with silk_learn |
| **encoder.ol** | `dream_cycle()` | i, heal_count, learn_count | every 5th respond |
| **encoder.ol** | `agent_respond()` | mol, intent, tone, reply, memory_context | MAIN ENTRY — all vars exposed |
| **encoder.ol** | `agent_process()` | mol, intent, tone | alt entry point |
| **encoder.ol** | `analyze_input()` | intent, tone | overwrites agent_respond's intent/tone |
| **encoder.ol** | `stm_push()` | i | collision during eviction |
| **encoder.ol** | `encode_text()` | i, n | encoder inner loop |
| **encoder.ol** | `mol_compose_many()` | i, result | encoder helper |
| **emotion.ol** | `sentence_affect()` | i, w, words, result | per respond |
| **emotion.ol** | `word_affect()` | result | per word lookup |
| **emotion.ol** | `apply_context()` | result | per respond |
| **intent.ol** | `contains()` | i | called per keyword check |
| **intent.ol** | `contains_any()` | i | called per keyword list |
| **intent.ol** | `list_contains()` | i | called per keyword list |
| **intent.ol** | `normalize()` | i, ch, result | text preprocessing |
| **intent.ol** | `detect_urgency()` | i | crisis detection |
| **node.ol** | `qr_search()` | (properly prefixed ✅) | OK |
| **repl.ol** | `repl_eval()` | src | entry point |

### Cold Path — 173 functions

Các function không chạy trong respond pipeline nhưng sẽ collide nếu gọi từ hot path trong tương lai. Xem scan output đầy đủ ở trên.

**Nguy hiểm nhất:**
- `bootstrap/parser.ol` — `tok` bare → self-compile sẽ crash nếu nested
- `bootstrap/semantic.ol` — `i` bare trong `collect_fns`, `lookup_fn` 
- `iter.ol` — 16 functions ALL bare → bất kỳ composition nào sẽ crash
- `json.ol` — 8 functions ALL bare → parse nested JSON sẽ crash
- `sort.ol` — `quicksort()` recursive với bare `i`, `n` → recursive = crash

---

## III. KIẾN TRÚC — CÁC VẤN ĐỀ THIẾT KẾ

### A. Global Var Table — Nợ kỹ thuật lớn nhất

**Hiện trạng:** VM dùng flat global var_table. Mọi `let x = val` ghi vào 1 bảng chung.

**Hệ quả:**
- Mọi function phải prefix tất cả locals
- 1 chỗ quên = infinite loop hoặc wrong value
- Không thể có libraries/modules gọi nhau an toàn
- 434 CRITICAL issues chưa fix = 434 potential bugs

**Giải pháp dài hạn:** Implement block scope trong VM (`op_call` push scope frame, `op_ret` pop). Đã thiết kế sẵn (scope stack 4MB, 256 depth) cho nested eval closures, nhưng boot closures vẫn flat.

**Giải pháp ngắn hạn:** Rename ALL bare vars. Ước tính: ~2000 dòng cần sửa.

### B. Heap — Bump Allocator, No GC

**Hiện trạng:** 256MB heap, bump only (`r15` grows up). Không free. Không GC.

**Hệ quả:**
- Long conversations exhaust heap
- `learn_file` với file lớn → heap full
- `respond` chain > ~50 calls → heap pressure
- Self-build phải `__heap_save`/`__heap_restore` manually

**Dấu hiệu:** Binary hoạt động tốt cho sessions ngắn (<30 interactions), nhưng production use (100+ turns) sẽ crash.

### C. String = u16 molecules — Mất diacritics

**Hiện trạng:** Mỗi char encode thành `0x2100 | byte`. Chỉ giữ low 8 bits.

**Hệ quả:**
- Tiếng Việt có dấu → mất dấu ("buồn" → "buon")
- Unicode characters > 255 → corrupt
- Emoji detection phải dùng UTF-8 byte sequence workaround

### D. Knowledge Store — Hard Limits

```
__knowledge_max = 512 (nhưng test cho thấy 128 max thực tế)
__silk_max = 64 edges
__stm_max = 32 turns (evict oldest)
__nodes_max = 256 nodes
```

**Vấn đề:** Không persistent. Mất khi tắt binary. Auto-learn ở boot compensate (~166 facts) nhưng user-taught knowledge vẫn mất.

### E. Respond Pipeline — O(n²) hoặc tệ hơn

Mỗi `respond` call đi qua 10+ stages, nhiều stage có nested loops:
1. `alias_normalize` — O(n) string scan
2. `text_emotion_unicode` — O(n) UTF-8 decode
3. `analyze_input` → `encode_text` — O(n×m) keyword matching
4. `text_emotion_v2` — O(n) word scan
5. `node_create` — O(nodes) dedup scan + SHA-256
6. `stm_push` — O(stm) eviction
7. `silk_learn_from_text` — O(words²) bigram wiring
8. `dream_cycle` — O(stm) scan
9. `stm_find_related` → `_a_has` — O(stm × words × text_len) **CRITICAL**
10. `qr_search` → `_qr_match_score` → `_qr_has` — O(nodes × words × text_len) **CRITICAL**
11. `knowledge_search` — O(knowledge × words)
12. `compose_reply` — O(1)

**Worst case:** With 166 knowledge, 64 silk, 32 stm, 256 nodes → mỗi respond = ~500K char_at operations.

---

## IV. ĐỀ XUẤT FIX — THỨ TỰ ƯU TIÊN

### P0 — BUG-VI fix (CẦN NGAY, blocks mọi conversation > 2 turns)

Fix 6 functions trong encoder.ol (đã viết patch ở trên):
1. `_a_has()` — rename i→_ah_i, j→_ah_j, tlen→_ah_tlen, wlen→_ah_wlen
2. `stm_topic_repeated()` — rename i→_str_i, count→_str_count
3. `stm_find_related()` — rename i→_sfr_i, wi→_sfr_wi, ch→_sfr_ch
4. `silk_learn_from_text()` — rename all bare vars
5. `silk_find_related()` — rename all bare vars
6. `dream_cycle()` — rename i→_dc_i, heal_count→_dc_heal

### P1 — Hot path prefix (blocks production use)

Fix remaining 17 functions in respond pipeline:
- encoder.ol: `agent_respond`, `agent_process`, `analyze_input`, `encode_text`, `mol_compose_many`, `silk_co_activate`, `stm_push`
- emotion.ol: `sentence_affect`, `word_affect`, `apply_context`
- intent.ol: `contains`, `contains_any`, `list_contains`, `normalize`, `detect_urgency`
- repl.ol: `repl_eval`, `is_olang_code`

### P2 — Critical stdlib (blocks self-compile + JSON + sort)

- `bootstrap/*.ol` — parser, semantic bare vars (blocks self-compile stability)
- `iter.ol` — 16 functions ALL bare (blocks any functional composition)
- `json.ol` — 8 functions ALL bare (blocks JSON parsing)
- `sort.ol` — recursive quicksort with bare vars (blocks sorting)

### P3 — Everything else (technical debt)

Remaining 173 cold-path functions across 40+ files. Lower risk but accumulates.

---

## V. METRICS

```
Total .ol files scanned:     51
Total LOC:                   13,547
Total functions with issues: 196
CRITICAL bare var issues:    434
WARN bare var issues:        256
Confirmed bugs:              1 (BUG-VI infinite loop)
Potential bugs:              ~50 (functions that call each other with colliding vars)
Hot path functions to fix:   23
Estimated fix effort:        ~2000 lines of renames
```

---

*Kiến trúc nền tảng vững — VM, compiler, self-build đều hoạt động.*
*Nhưng global var_table là nợ kỹ thuật lớn nhất. 434 quả mìn chờ nổ.*
*Fix BUG-VI trước — không ai nói chuyện được quá 2 câu.*
