# BUG REPORT — Kira Inspect #24: Pipeline Integrity Audit

> **From**: Kira (Inspector v3)
> **To**: Nox (Builder)
> **Date**: 2026-03-25
> **Binary**: origin_new.olang 1,021KB (1,021,393 bytes)
> **Scope**: Full L0→L3 pipeline audit. UTF-32, Emoji, Node, Silk, Knowledge, Dream.

---

## TL;DR

Pipeline 10-stage chay duoc nhung **6/12 subsystem bi dut hoac NOP**.
Knowledge search sai ranking. Silk frozen. Dream NOP. fn_node eval broken.
UTF-32 codepoint chi dung trong knowledge chain — Node va Silk van byte-level.
Emoji chi extract V/A — khong tao node, khong tao Silk edge.

---

## 1. SO DO THUC TE (tested, not spec)

```
Input "xin chao 😊"
  |
  v
[L0] alias_normalize .............. OK (31 slang + 10 emoji shortcode)
  |
  v
[L0] text_emotion_unicode ........ OK — utf8_decode → cp=0x1F60A → V=7 A=6
  |   BUT: chi EXTRACT V/A, khong tao node/edge
  |
  v
[L0] encode_text .................. !! BYTE-LEVEL — char_at(byte) → mol
  |   Emoji 4 bytes → 4 mol rieng → compose → mat semantic identity
  |   "Viet" va "Việt" → KHAC nhau (OK trong knowledge, KHONG OK trong node)
  |
  v
[L1] node_create .................. OK — 1 node per input text, DN=SHA-256
  |   BUT: mol = byte-level, khong phai UTF-32 codepoint
  |   Emoji khong tao node rieng — gop chung text
  |
  v
[L1] stm_push ..................... OK — max 32 turns, intent/tone tracking
  |
  v
[L1] silk_learn_from_text ......... !! FROZEN — edges = 17 (boot), khong tang
  |   Split by space(32) → emoji bi gop vao word cuoi ("chao😊")
  |   _word_to_mol() → mol collision cao → update weight, khong tao edge moi
  |
  v
[L1] dream_cycle .................. !! NOP — check _dc_e.emotion nhung silk edge
  |   struct = {from, to, weight, fires} — KHONG CO field "emotion"
  |   → Dream consolidation KHONG BAO GIO boost edge
  |
  v
[L2] knowledge_search ............. !! SAI RANKING
  |   "Ha Noi o dau" → "Origin bat dau ngay 11/3/2026" (WRONG)
  |   Root cause: _a_has() substring match — "Ha" khop nhieu fact
  |   Short words (2-3 chars) khong bi penalize
  |
  v
[L2] qr_search .................... !! GATE qua chat — fires > 1 moi tra
  |   Node moi fires=1 → QR search LUON tra empty cho node moi
  |
  v
[L2] fn_node (eval) ............... !! BROKEN — cross-boundary boot↔eval
  |   fn defined qua REPL → fn_node_register() la boot function
  |   → eval context khong goi duoc → fns = 0
  |
  v
[L3] Emotion carry ................ OK — EMA 60/40, streak, FE tracking
  |
  v
[L3] SC.4 Immune Selection ........ !! YEU — score = len(text)
  |   Knowledge response dai → LUON thang
  |   Candidate 3 (Silk) chi boost +5, khong tao response rieng
  |
  v
[L3] SC.6 DNA Repair .............. !! KHONG REPAIR
  |   Chi detect keyword "khong"/"sai"/"phang" trong INPUT
  |   KHONG compare semantic giua 2 learned facts
  |   learn("Trai Dat hinh cau") + learn("Trai Dat hinh phang") → no alert
  |
  v
[Output] compose_reply + emoji ..... OK
```

---

## 2. BANG TONG HOP

| # | Component | Layer | Status | Bug ID |
|---|-----------|-------|--------|--------|
| 1 | UDC Encode (mol) | L0 | OK | — |
| 2 | Alias normalize | L0 | OK | — |
| 3 | Emoji → V/A extract | L0 | OK | — |
| 4 | Emoji → Node | L0 | **KHONG** | BUG-EMOJI-NODE |
| 5 | Emoji → Silk edge | L1 | **KHONG** | BUG-EMOJI-SILK |
| 6 | UTF-32 → Knowledge chain | L1 | OK | — |
| 7 | UTF-32 → Node mol | L1 | **KHONG** (byte-level) | BUG-NODE-UTF32 |
| 8 | STM push/query | L1 | OK | — |
| 9 | Silk learn | L1 | **FROZEN** (17 edges) | BUG-SILK-FROZEN |
| 10 | Dream consolidation | L1 | **NOP** | BUG-DREAM-NOP |
| 11 | Knowledge search | L2 | **SAI ranking** | BUG-KNOWLEDGE-RANK |
| 12 | QR search | L2 | **GATE qua chat** | BUG-QR-GATE |
| 13 | fn_node (eval) | L2 | **BROKEN** | BUG-FNNODE-EVAL |
| 14 | Emotion carry | L3 | OK | — |
| 15 | SC.4 Immune | L3 | **YEU** | BUG-IMMUNE-SCORE |
| 16 | SC.5 Homeostasis FE | L3 | OK | — |
| 17 | SC.6 DNA Repair | L3 | **KHONG REPAIR** | BUG-SC6-REPAIR |
| 18 | 7 Instincts | L3 | KEYWORD-ONLY | INFO (not bug) |

**Score: 6/18 OK, 5/18 BROKEN, 4/18 WEAK, 3/18 NOP/MISSING**

---

## 3. CHI TIET TUNG BUG

### BUG-DREAM-NOP (Critical — Dream = dead code)

**File**: `stdlib/homeos/encoder.ol` line 1152
**Problem**: `dream_cycle()` loops silk edges, checks `_dc_e.emotion`:
```olang
if _dc_e.emotion == _dc_dominant {  // line 1152
```
But silk edge struct is `{from, to, weight, fires}` — **NO `emotion` field**.
`_dc_e.emotion` → undefined → never matches → Dream consolidation = NOP.

**Fix**: Either:
- (a) Add `emotion` field to silk edges in `silk_co_activate()` (currently stores `intent` param but doesn't save it), OR
- (b) Change Dream to use a different strategy (e.g., keyword match instead of emotion field)

### BUG-SILK-FROZEN (High — Silk not growing)

**File**: `stdlib/homeos/encoder.ol` line 998-1022
**Symptom**: After 5 `respond` calls, Silk stays at 17 edges (all from boot training data).
**Root cause**: `_word_to_mol()` composes all chars → u16. Vietnamese common words ("toi", "thich", "hoc") produce mols that **collide** with existing training-data edges (same mol pair = update weight, not create new). Also, 17 initial edges from `knowledge_learn()` during boot dominate.

**Verify**: Add debug `emit` inside `silk_co_activate()` to trace whether new vs update path fires.

**Fix candidates**:
- (a) Use larger mol space (current: u16 = 65536 values, lots of collision)
- (b) Store word STRING as key instead of mol (was this way before LG.3 optimization)
- (c) Use __sha256(word) truncated to 32 bits for less collision

### BUG-KNOWLEDGE-RANK (High — wrong fact retrieval)

**File**: `stdlib/homeos/encoder.ol` line 1691-1773
**Symptom**: `"Ha Noi o dau"` → returns "Origin bat dau ngay 11/3/2026" instead of "Viet Nam... thu do Ha Noi".
**Root cause**: `_a_has()` substring match — word "Ha" (2 chars) matches inside "T**ha**nh pho", "S**ha**-256", etc. Score doesn't penalize short words enough. Also `_ks_qw >= 2` threshold too low.

**Fix**: Raise min word length for keyword match from 2 → 3 or 4 chars. Add word-boundary check. Penalize matches where query word < 4 chars.

### BUG-NODE-UTF32 (Medium — Node mol is byte-level)

**File**: `stdlib/homeos/encoder.ol` line 1287, and `encode_text()` function
**Problem**: `node_create()` gets `mol` from `analyze_input()` → `encode_text()` which uses `char_at()` (byte-level). `_text_to_chain()` in knowledge uses `__utf8_cp()` but node does NOT.

**Fix**: Change `analyze_input()` to use `__utf8_cp()` for mol composition, matching `_text_to_chain()`.

### BUG-EMOJI-NODE (Medium — Emoji doesn't create separate node)

**File**: `stdlib/homeos/encoder.ol` line 1254-1287
**Problem**: `text_emotion_unicode()` extracts V/A from emoji but result is ONLY used for emotion blending. The emoji codepoint is NOT fed into `node_create()` as a separate entity. Entire text "xin chao 😊" = 1 node.

**Impact**: Emoji identity is lost in graph — can't query "which turns had happy emoji?"

### BUG-EMOJI-SILK (Medium — Emoji doesn't create Silk edge)

**File**: `stdlib/homeos/encoder.ol` line 1067-1089
**Problem**: `silk_learn_from_text()` splits by ASCII space (byte 32). Emoji bytes (0xF0...) don't contain 0x20 → emoji is concatenated to previous word. "chao😊" becomes 1 word → wrong mol.

**Fix**: In `silk_learn_from_text()`, detect non-ASCII bytes and split before/after emoji sequences.

### BUG-FNNODE-EVAL (Medium — fn_node broken in REPL)

**File**: `stdlib/bootstrap/semantic.ol` (T5 LG.1 auto-emit)
**Problem**: Compiler emits `fn_node_register()` call after FnDef. But this call runs in **eval context**, and `fn_node_register()` is a **boot function** that modifies `__fn_nodes[]` (boot global). Cross-boundary: eval → boot function → boot global = works for SOME builtins but fn_node_register has dict args that may not cross boundary.

**Workaround**: Accept that fn_node only tracks boot-compiled functions (stdlib), not REPL-defined.

### BUG-QR-GATE (Low — QR search too strict)

**File**: `stdlib/homeos/encoder.ol` line 1323
**Problem**: `if _ar_qr.fires > 1` — new nodes have fires=1 → QR never returns them. Only repeated inputs get into QR results.

**Fix**: Lower gate to `fires >= 1` or remove it entirely.

### BUG-IMMUNE-SCORE (Low — SC.4 score metric naive)

**File**: `stdlib/homeos/encoder.ol` line 1474
**Problem**: `_ar_c1_score = len(_ar_out)` — score by string length. Knowledge response is always longest → always wins. Candidate 2 (STM context) and 3 (Silk) rarely selected.

**Fix**: Score by relevance (keyword match count, recency, confidence) not length.

### BUG-SC6-REPAIR (Low — no semantic contradiction detection)

**File**: `stdlib/homeos/encoder.ol` line 1380
**Problem**: Contradiction detection is keyword-based: only fires if INPUT contains "khong"/"sai"/"phang". Does NOT compare learned facts against each other.

**Fix**: When learning new fact, compare against existing facts with high chain_similarity. If similar but contradicting (e.g., "hinh cau" vs "hinh phang"), flag.

---

## 4. UU TIEN FIX

```
P0 (Pipeline broken):
  1. BUG-DREAM-NOP        — Dream = dead code. 1-line fix: add emotion field to silk edge.
  2. BUG-KNOWLEDGE-RANK   — Wrong fact retrieval. Core UX broken.
  3. BUG-SILK-FROZEN      — Silk not learning. Need verify + fix mol collision.

P1 (Feature incomplete):
  4. BUG-NODE-UTF32       — Node mol should use UTF-32 like knowledge.
  5. BUG-EMOJI-NODE       — Emoji should create concept node.
  6. BUG-EMOJI-SILK       — Emoji should create Silk edge.

P2 (Polish):
  7. BUG-QR-GATE          — Lower fires threshold.
  8. BUG-IMMUNE-SCORE     — Better scoring metric.
  9. BUG-SC6-REPAIR       — Semantic contradiction detection.
  10. BUG-FNNODE-EVAL     — Cross-boundary, may need VM change.
```

---

## 5. CAI GI HOAT DONG TOT

- **Emotion pipeline**: V/A extract, word affect (72 entries), emoji blend 70/30, EMA carry, streak — ALL OK
- **STM**: push, count, find_related, digest — ALL OK
- **Knowledge learn**: UTF-8 aware chain encoding, `__utf8_cp()` — OK
- **Node create/link**: SHA-256 DN, dedup, linking — OK
- **Homeostasis FE**: intent shift + emotion delta tracking — OK
- **Greeting/goodbye router**: classify → smart response — OK
- **Math ?/= strip**: "2+3?" → 5 — OK
- **Instincts 7/7**: present but keyword-based (acceptable for v1.0)
- **Dict pretty-print**: `{name: Olang}` — OK
- **20/20 core tests**: ALL PASS
- **Self-hosting**: 1,021KB, fib(20)=6765, fact(10)=3628800 — SOLID

---

## 6. KET LUAN

Binary 1,021KB chay on dinh. Core language features (compile, eval, lambda, HOF) chac.
Intelligence pipeline co skeleton day du nhung **6 subsystem chua wire dung**.
3 bug P0 can fix truoc khi demo: Dream NOP, Knowledge ranking, Silk frozen.
UTF-32 foundation da co (`__utf8_cp`) nhung chua propagate ra Node va Silk.

> "Xuong da co, nhung day than kinh chua noi het."
> — Kira, Inspect #24
