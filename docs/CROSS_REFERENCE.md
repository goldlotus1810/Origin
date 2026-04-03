# CROSS REFERENCE — 3 SPECS = 1 HỆ THỐNG

> Document này chứng minh 3 specs liên kết thành 1 system.
> Mỗi opcode, mỗi function, mỗi data structure phải xuất hiện ở đúng 1 nơi
> và được tham chiếu từ các nơi khác.

---

## 1. OPCODE → SPEC MAPPING

| Opcode | Hex | OLANG (§) | VM (§) | MOL (§) | Status |
|--------|-----|-----------|--------|---------|--------|
| NOP | 0x00 | 8.2 | 4.2 | — | ✅ DONE |
| LOAD_NIL | 0x01 | 8.2 | 4.2 | — | ✅ DONE |
| LOAD_TRUE | 0x02 | 8.2 | 4.2 | — | ✅ DONE |
| LOAD_FALSE | 0x03 | 8.2 | 4.2 | — | ✅ DONE |
| LOAD_INT | 0x04 | 8.2 | 4.2 | — | ✅ DONE |
| LOAD_CONST | 0x05 | 8.2 | 4.2 | — | ✅ DONE |
| LOAD_GLOBAL | 0x06 | 8.2 | 4.2 | — | ❌ CHƯA |
| STORE_GLOBAL | 0x07 | 8.2 | 4.2 | — | ❌ CHƯA |
| MOVE | 0x08 | 8.2 | 4.2 | — | ✅ DONE |
| LOAD_UPVAL | 0x09 | 8.2 | 4.2 | — | ❌ CHƯA (cần closures) |
| STORE_UPVAL | 0x0A | 8.2 | 4.2 | — | ❌ CHƯA |
| LOAD_FIELD | 0x0B | 8.2 | 4.2 | — | ❌ CHƯA (cần structs) |
| STORE_FIELD | 0x0C | 8.2 | 4.2 | — | ❌ CHƯA |
| LOAD_INDEX | 0x0D | 8.2 | 4.2 | — | ❌ CHƯA (cần arrays) |
| STORE_INDEX | 0x0E | 8.2 | 4.2 | — | ❌ CHƯA |
| LOAD_MOL | 0x0F | 8.2 | 4.2 | §1 | ✅ DONE |
| ADD | 0x10 | 8.2 | 4.2 | — | ✅ DONE |
| SUB | 0x11 | 8.2 | 4.2 | — | ✅ DONE |
| MUL | 0x12 | 8.2 | 4.2 | — | ✅ DONE |
| DIV | 0x13 | 8.2 | 4.2 | — | ✅ DONE |
| MOD | 0x14 | 8.2 | 4.2 | — | ✅ DONE |
| NEG | 0x15 | 8.2 | 4.2 | — | ✅ DONE |
| SHL | 0x16 | 8.2 | — | — | ❌ CHƯA |
| SHR | 0x17 | 8.2 | — | — | ❌ CHƯA |
| BAND | 0x18 | 8.2 | — | — | ❌ CHƯA |
| BOR | 0x19 | 8.2 | — | — | ❌ CHƯA |
| BXOR | 0x1A | 8.2 | — | — | ❌ CHƯA |
| BNOT | 0x1B | 8.2 | — | — | ❌ CHƯA |
| CONCAT | 0x1C | 8.2 | 4.2 | — | ✅ DONE |
| EQ | 0x20 | 8.2 | 4.2 | — | ✅ DONE |
| NE | 0x21 | 8.2 | 4.2 | — | ✅ DONE |
| LT | 0x22 | 8.2 | 4.2 | — | ✅ DONE |
| LE | 0x23 | 8.2 | 4.2 | — | ✅ DONE |
| GT | 0x24 | 8.2 | 4.2 | — | ✅ DONE |
| GE | 0x25 | 8.2 | 4.2 | — | ✅ DONE |
| CONF_AND | 0x26 | 8.2 | — | — | ❌ CHƯA |
| CONF_OR | 0x27 | 8.2 | — | — | ❌ CHƯA |
| CONF_NOT | 0x28 | 8.2 | — | — | ❌ CHƯA |
| JMP | 0x30 | 8.2 | 4.2 | — | ✅ DONE |
| JZ | 0x31 | 8.2 | 4.2 | — | ✅ DONE |
| JNZ | 0x32 | 8.2 | 4.2 | — | ✅ DONE |
| CALL | 0x33 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §4.2, window-based, 256 max frames) |
| RET | 0x34 | 8.2 | 4.2 | — | ✅ DONE |
| RET_NIL | 0x35 | 8.2 | 4.2 | — | ✅ DONE |
| CLOSURE | 0x36 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §4.2, captures packed in OlClosure.upvals[]) |
| CALL_CLOSURE | 0x37 | 8.2, 15 | 4.2 | — | ✅ SPEC (merged with CALL — single opcode handles both) |
| TRY_BEGIN | 0x38 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §4.2, try stack + frame depth tracking) |
| CATCH_END | 0x39 | 8.2 | 4.2 | — | ✅ SPEC (VM §4.2) |
| THROW | 0x3A | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §4.2, unwind to matching try frame) |
| CLOSURE_CAP | 0x3B | 8.2 | 4.2 | — | ✅ SPEC (VM §4.2, capture register into closure upval) |
| NEW_ARRAY | 0x40 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §2.3 OlArray + §4.2 opcode) |
| ARRAY_PUSH | 0x41 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §2.3 + §4.2) |
| ARRAY_GET | 0x42 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §4.2, nil on OOB) |
| ARRAY_SET | 0x43 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §4.2) |
| ARRAY_LEN | 0x44 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §4.2) |
| NEW_DICT | 0x45 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §2.3 OlDict + §4.2) |
| DICT_GET | 0x46 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §2.3 + §4.2) |
| DICT_SET | 0x47 | 8.2, 15 | 4.2 | — | ✅ SPEC (VM §2.3 + §4.2, auto-resize at 75% load) |
| NEW_STRUCT | 0x48 | 8.2 | 4.2 | — | ❌ CHƯA |
| MOL_PACK | 0x50 | 8.2 | 4.2 | §1 | ❌ CHƯA |
| MOL_UNPACK | 0x51 | 8.2 | 4.2 | §1 | ❌ CHƯA |
| MOL_DIST | 0x52 | 8.2 | 4.2 | §1 | ✅ DONE |
| MOL_COMPOSE | 0x53 | 8.2 | 4.2 | §3,9 | ❌ CHƯA |
| MOL_ENCODE | 0x54 | 8.2 | 4.2 | §7 | ✅ DONE |
| CHAIN_LCA | 0x56 | 8.2 | 4.2 | §9 | ❌ CHƯA |
| CHAIN_NEW | 0x57 | 8.2 | 4.2 | §8 | ❌ CHƯA |
| CHAIN_PUSH | 0x58 | 8.2 | 4.2 | §8 | ❌ CHƯA |
| CHAIN_LEN | 0x59 | 8.2 | 4.2 | §8 | ❌ CHƯA |
| CHAIN_GET | 0x5A | 8.2 | 4.2 | §8 | ❌ CHƯA |
| CHAIN_SIM | 0x5B | 8.2 | 4.2 | §8 | ❌ CHƯA |
| KT_STORE | 0x60 | 7.6 | 5.1 | — | ❌ CHƯA |
| KT_LOOKUP | 0x61 | 7.6 | 5.1 | §14 | ❌ CHƯA |
| KT_NEAREST | 0x62 | 7.6 | 5.1 | §14 | ❌ CHƯA |
| KT_WALK | 0x63 | 7.6 | 5.1 | — | ❌ CHƯA |
| SILK_FIRE | 0x64 | 7.6 | 5.2 | — | ❌ CHƯA |
| SILK_DECAY | 0x65 | 7.6 | 5.2 | §15 | ❌ CHƯA |
| SILK_WALK | 0x66 | 7.6 | 5.2 | — | ❌ CHƯA |
| SILK_WEIGHT | 0x67 | 7.6 | 5.2 | — | ❌ CHƯA |
| DREAM | 0x68 | 7.6 | 5.5 | — | ❌ CHƯA |
| INSTINCT | 0x69 | 7.6 | 5.4 | — | ❌ CHƯA |
| EMIT | 0x70 | 8.2 | 4.2 | — | ✅ DONE |
| READLINE | 0x71 | 8.2 | 4.2 | — | ❌ CHƯA |
| FILE_READ | 0x72 | 8.2 | 4.2 | — | ❌ CHƯA |
| FILE_WRITE | 0x73 | 8.2 | 4.2 | — | ❌ CHƯA |
| HALT | 0xFE | 8.2 | 4.2 | — | ✅ DONE |
| BUILTIN | 0xFF | 8.2 | 4.2 | — | ❌ CHƯA |

**Tổng: 22/78 code DONE + 18 SPEC'd (có C code trong spec). 38 còn thiếu.**
**Session 2026-04-03: +18 opcodes SPEC'd (CALL, CLOSURE, TRY/CATCH/THROW, Array×5, Dict×3, CLOSURE_CAP, LOAD/STORE_UPVAL)**

---

## 2. DATA FLOW: INPUT → OUTPUT

```
User Input (text)
  │
  ├─ OLANG §2 (Lexer) ──────── encode_text() ─── MOL §7,8
  │
  ├─ VM §14 (SecurityGate) ──── gate_check() 
  │
  ├─ VM §5.3 (Pipeline) ────── 14 steps
  │   ├─ Step 2: Encode ─────── MOL §7 encode_codepoint()
  │   ├─ Step 4: Search ─────── MOL §14 VP-Tree + §13 Shadow Vector
  │   ├─ Step 8: Instincts ──── VM §5.4 (7 formulas)
  │   │   └─ Uses: MOL §1 mol_dist, §4 ValenceState, §5 ArousalState
  │   ├─ Step 12: Hebbian ───── VM §5.2 Silk + MOL §15 Liquid Weights
  │   ├─ Step 13: Dream ─────── VM §5.5 + MOL §9 LCA
  │   └─ Step 15: Decode ────── MOL §7 (reverse) + VM §18 Generation
  │
  ├─ VM §13 (ConversationCurve) ── tone selection
  │
  └─ VM §18 (Response Generation)
      ├─ Retrieve ──────── MOL §14 VP-Tree
      ├─ Rank ──────────── MOL §13 Shadow Vector cosine
      ├─ Compose ────────── Templates
      └─ Fluency ────────── VM §13 ConversationCurve

Output (text + tone)
```

---

## 3. DATA STRUCTURE OWNERSHIP

| Structure | Defined in | Used by | Implementation |
|-----------|-----------|---------|----------------|
| Value (NaN-boxed u64) | src/value.h | VM, all | ✅ DONE |
| OlStr (interned string) | src/string.h | VM, all | ✅ DONE |
| Molecule (u16) | MOL §1, src/mol.h | VM, KnowTree, Silk | ✅ DONE |
| MolChain (u16[]) | MOL §8, src/mol.h | VM, KnowTree, Pipeline | ✅ DONE |
| KTNode (32 bytes) | VM §5.1 | KnowTree, Dream | ❌ CHƯA |
| SilkEdge (24 bytes) | VM §5.2 | Silk, Hebbian | ❌ CHƯA |
| ShadowVector (f32×8) | MOL §13 | VP-Tree, Search, Rank | ❌ CHƯA |
| VPNode (16 bytes) | MOL §14 | Search | ❌ CHƯA |
| ConversationCurve | VM §13 | Pipeline, Generation | ❌ CHƯA |
| EmotionTag (3 floats) | VM §15 | Pipeline, Fusion, Context | ❌ CHƯA |
| OlProto (bytecode) | OLANG §8.3 | VM, Compiler | ✅ DONE (partial) |
| CallFrame | VM §4.1 | VM interpreter | ✅ DONE |
| Heap (Cheney) | VM §3.2, src/gc.h | VM, all heap objects | ✅ DONE |
| Arena | VM §3.3, src/gc.h | KnowTree, Silk | ✅ DONE |
| StringTable | src/string.h | VM globals, interning | ✅ DONE |
| LiquidParams (f32×8) | MOL §15 | Silk decay | ❌ CHƯA |
| OlArray | VM §2.3 | VM arrays, GC scans items[] | ✅ SPEC (flexible array member, grow 2x) |
| OlDict | VM §2.3 | VM dicts, GC scans entries[] | ✅ SPEC (open-addressing, 75% load factor) |
| OlClosure | VM §2.3 | VM closures, GC scans upvals[] | ✅ SPEC (Julia-inspired captures tuple) |
| OlBytes | VM §2.3 | Binary I/O, crypto | ✅ SPEC (raw bytes, GC no-scan) |
| DecodeMap | MOL §23 | Mol→codepoint reverse lookup | ✅ SPEC (65536 buckets, build at startup) |
| NrcVadEntry | MOL §27 | Word VAD lookup | ✅ DONE (19,971 entries, src/nrc_vad_data.h) |
| Template | VM §18 | Generation | ❌ CHƯA |
| IntentResult | VM §17 | Pipeline | ❌ CHƯA |
| ISLAddress (4 bytes) | VM §10.2 | Agent hierarchy | ❌ CHƯA |
| ISLMessage (12 bytes) | VM §10.2 | Agent hierarchy | ❌ CHƯA |

---

## 4. FORMULA CHAIN: MOL ↔ VM ↔ OLANG

| Formula | MOL Section | VM Usage | OLANG Builtin |
|---------|-------------|----------|---------------|
| mol_pack() | §1 | LOAD_MOL opcode | mol_pack(s,r,v,a,t) |
| mol_dist() | §1 | MOL_DIST opcode, Instincts, Search | mol_dist(a,b) |
| mol_compose_pair() | §9 | MOL_COMPOSE opcode, Pipeline | N/A (internal) |
| 16 RelationOp compose | §3 | Pipeline step 6, Silk | relop_compose() internal |
| 8 ValenceState | §4 | Instinct #1 Honesty, Pipeline | valence_from_v() internal |
| 8 ArousalState | §5 | Instinct urgency check, SecurityGate | arousal_urgency() internal |
| FNV-1a hash | §8 | String interning, chain_hash | N/A (internal) |
| chain_similarity | §8 | Instinct #4 Abstraction, LCA variance | chain_similarity(a,b) |
| LCA 5 rules | §9 | CHAIN_LCA opcode, Dream cycle | chain_lca(a,b) |
| VP-Tree knn | §14 | KT_NEAREST opcode, Search | kt_nearest(mol,k) |
| Shadow cosine | §13 | Rank in Generation | N/A (internal) |
| Liquid decay | §15 | SILK_DECAY opcode | N/A (internal) |
| encode_codepoint | §7 | MOL_ENCODE opcode, Pipeline step 2 | encode(text) |
| Maturity advance | §10 | Dream cycle, QR promotion | N/A (internal) |

---

## 5. RESEARCH STATUS

| ID | Topic | Status |
|----|-------|--------|
| R1 | Collision rate | CẦN ĐO — chạy Julia `collision_histogram()` (OLANG §16.3) |
| R2 | φ⁻¹ ablation | ✅ DONE — stretched exp β=φ⁻¹ (MOL §26) |
| R3 | KAN B-spline | DEFER — template đủ cho MVP |
| R4 | 4 reasoning engines | ✅ DONE — code đầy đủ (VM §23) |
| R5 | Julia parser | ✅ DONE — parse_julia_functions() (OLANG §16.2) |
| R6 | Concurrency | DEFER — green threads |
| R7 | UCD table generation | CẦN CHẠY — json/udc.json có, cần Julia precompute (VM §11.1) |
| R8 | NRC-VAD lexicon | ✅ DONE — 19,971 entries (src/nrc_vad_data.h) |
| R9 | Decode chain→text | ✅ DONE — inverted map (MOL §23) |
| R10 | Intent keywords | ✅ DONE — NRC-VAD integration (VM §17) |

**BLOCKING trước MVP:** R1 (collision rate) + R7 (UCD table) — cả hai cần chạy Julia precompute

---

## 6. IMPLEMENTATION PRIORITY

```
Phase 1 (DONE): Core VM chạy — 22 opcodes, 8 tests pass
    ✅ value.h, string.c, mol.c, gc.c, vm.c, main.c

Phase 2 (SPEC DONE — cần implement vào src/):
    ✅ CALL + RET (VM §4.2 — window-based, Lua-style)
    ✅ CLOSURE + CLOSURE_CAP + LOAD/STORE_UPVAL (VM §2.3 + §4.2)
    ✅ OlArray + 5 opcodes (VM §2.3 + §4.2)
    ✅ OlDict + 3 opcodes (VM §2.3 + §4.2)
    ✅ TRY/CATCH/THROW (VM §4.2 — try stack + unwind)
    ✅ GC scan_object per-type (VM §2.4)
    ❌ LOAD_GLOBAL + STORE_GLOBAL (spec có, code chưa)
    → Mục tiêu: chạy Olang programs thật (không chỉ hand-coded bytecode)

Phase 3: Compiler
    ❌ Lexer (tokens from OLANG §2) — cần implement
    ❌ Parser (AST from OLANG §3-4, §14.3) — struct + recursive descent có spec
    ✅ Codegen (OLANG §14.5) — SPEC DONE: compile_expr + compile_stmt, ~600 LOC C
       Bao gồm: scope management, jump patching, nested fn, for/while/if/match/try
       Còn thiếu: free var analysis, struct compile, pipe, fstring
    → Mục tiêu: compile .ol files → .olang bytecode

Phase 4: Brain
    ❌ KnowTree (VM §5.1 + MOL §14 VP-Tree)
    ❌ Silk (VM §5.2 + MOL §15 Liquid)
    ❌ Pipeline 14 steps (VM §5.3)
    ❌ 7 Instincts (VM §5.4)
    ❌ Dream cycle (VM §5.5)
    → Mục tiêu: Nox learn + recall

Phase 5: Generation
    ❌ UCD table generate (R7)
    ❌ Collision rate test (R1) — chạy decode_map_build() + print_stats()
    ✅ Decode chain → text (R9) — MOL §23 inverted map + disambiguation
    ✅ NRC-VAD integration (R8) — 19,971 entries in src/nrc_vad_data.h
    ✅ VP-Tree rebuild policy — MOL §24 FLANN-style 2x threshold
    ✅ Shadow Vector learning — MOL §25 Word2Vec negative sampling
    ✅ Decay: stretched exponential β=φ⁻¹ (kết hợp exp + power law) — MOL §26
    ❌ Template composition (VM §18 + §24)
    ❌ ConversationCurve (VM §13)
    → Mục tiêu: Nox nói được

Phase 6: Agent
    ❌ SecurityGate (VM §14)
    ❌ ISL protocol (VM §10.2)
    ❌ Self-modification (OLANG §13.7)
    ❌ Persistence (OLANG §13.5)
    ❌ "Eat" languages (VM §6.3)
    → Mục tiêu: Nox tự vận hành
```

---

---

## 7. MÂU THUẪN GIỮA 3 SPECS — PHẢI SỬA

Rà soát ngày 2026-04-03. 13 mâu thuẫn tìm được.

### CRITICAL — sẽ crash nếu implement

| # | Vấn đề | File A | File B | Sửa |
|---|--------|--------|--------|-----|
| 1 | **Opcode 0x3B**: OLANG=LOOP_INIT, VM=OP_CLOSURE_CAP | OLANG §8.2 | VM §4.2 | Đổi CLOSURE_CAP sang 0x3D (trống). GIỮ LOOP_INIT=0x3B |
| 2 | **LOAD_UPVAL format**: OLANG=AD (16-bit), VM=AB (8-bit) | OLANG §8.2 | VM §4.2 | Dùng AB (8-bit đủ — max 256 upvalues). Sửa OLANG §8.2 |
| 3 | **S compose**: MOL=Union(dominant/max), VM=MIN() | MOL §9 | VM §5.6 | Dùng MOL version (Union=max). CLAUDE.md cũng nói max. Sửa VM |
| 7 | **MolecularChain**: MOL=heap pointer, VM=inline[64] | MOL §8 | VM §5.2 | Dùng cả 2: MOL version cho general, inline[64] cho STM (perf). Ghi rõ |
| 9 | **Bool**: OLANG=float 0.0/1.0, VM=TAG_BOOL | OLANG §5.2 | VM §2.1 | Dùng OLANG version: bool = float 1.0/0.0. Xóa TAG_BOOL. Confidence model |

### HIGH — sai kết quả

| # | Vấn đề | Sửa |
|---|--------|-----|
| 4 | QR threshold: MOL=0.854, VM §22=0.65 | Dùng 0.854 (φ⁻¹+φ⁻³). VM §22 là "adaptive default" → rename, ghi rõ |
| 8 | OlProto: no single definition | Viết 1 struct đầy đủ trong VM §4.1, reference từ OLANG |
| 13 | UCD group: §7 bắt đầu 0, §21 bắt đầu 1 | Dùng 0-based (§7). Sửa generator script §21 |

### MEDIUM — sai nhỏ nhưng tích lũy

| # | Vấn đề | Sửa |
|---|--------|-----|
| 5 | Decay: 618/1000 vs 618/1024 | Dùng 618/1000 (chính xác hơn). Sửa CLAUDE.md |
| 6 | Homeostasis: 0.6 vs 0.618 | Dùng 0.618 (φ⁻¹). Sửa VM §21 #define |
| 11 | Decay partial: linear vs powf | Dùng powf (mathematically correct). Sửa VM §5.2 |
| 12 | OlStr flags | Thêm flags vào OLANG binary format §8.3 |
| 10 | CHAN_NEW reference | Đổi thành "[CHƯA IMPLEMENT]" thay vì reference sai |

### Status: ĐÃ SỬA (2026-04-03)

| # | Status |
|---|--------|
| 1 | ✅ CLOSURE_CAP=0x3B, LOOP_INIT=0x3C, LOOP_DEC=0x3D (OLANG §8.2) |
| 2 | ✅ LOAD/STORE_UPVAL format AB (OLANG §8.2 + VM §4.2 thống nhất) |
| 3 | ✅ S compose = MAX (dominant shape). S index = category, KHÔNG phải SDF distance. MAX nhất quán với MOL compose_union(), NOX_COMPLETE_REFERENCE, NOX_AI_MODEL_SPEC |
| 4 | ✅ QR threshold: BCM adaptive (VM §5.7) |
| 5 | ✅ Decay: stretched exp β=φ⁻¹ (MOL §26) |
| 6 | ✅ Homeostasis = 0.5 tunable (ACT-R/Soar: round number, tune per-model) |
| 7 | ✅ MolChain: heap cho general, inline[64] cho STM/pipeline |
| 8 | ✅ OlProto: canonical definition trong VM §4.1 |
| 9 | ✅ Bool = TAG_BOOL (NaN-tag singleton — clox/Wren/LuaJIT pattern) |
| 10 | ✅ CHAN_NEW ref sửa thành "[CHƯA THIẾT KẾ]" (OLANG §13.3) |
| 11 | ✅ Decay partial period: stretched exp thay linear (VM §5.2 sửa) |
| 12 | MINOR — OlStr flags byte: VM has it, OLANG §8.3 binary format nên add |
| 13 | ✅ UCD group: 0-based (MOL §21 sửa) |

**GHI CHÚ:** CLAUDE.md mô tả hệ thống cũ (vm_nox.S). Specs mới là master.
Khi có mâu thuẫn giữa CLAUDE.md và specs → specs thắng.

---

*Document này được cập nhật mỗi khi có thay đổi trong 3 specs.
Mọi ❌ phải trở thành ✅ trước khi hệ thống hoàn chỉnh.*

---

## 8. SỐ LIỆU ĐÃ VERIFY (2026-04-03) — Kết quả nghiên cứu khoa học

### ĐÃ SỬA

| # | Vấn đề | Trước | Sau | Cách verify |
|---|--------|-------|-----|-------------|
| V1 | **Stretched exp τ** | 81.3h | **78.37h** | Giải: exp(-(24/τ)^0.618)=0.618 → τ=78.37. Cũ cho 0.6247 |
| V2 | **Knuth hash comment** | "2654435761 = floor(2³²/φ)" | **2654435769** (code giữ 761=closest prime) | Python: int(2**32/φ)=2654435769 |
| V3 | **NRC-VAD count** | 19,970 ở §27, 19,971 ở chỗ khác | **19,971** thống nhất | grep NRC_VAD_COUNT src/ → 19971 |
| V4 | **Wickelgren formula** | P(t) | **m(t)** (memory strength) | Original paper: Memory & Cognition 1974 |
| V5 | **Duplicate `int n=0`** | 2 dòng | 1 dòng | Code review |
| V6 | **Duplicate `int nearest_count`** | 2 dòng | 1 dòng | Code review |

### ĐÃ VERIFY — ĐÚNG

| Claim | Verified? | Source |
|-------|-----------|--------|
| mol_dist max = 70 | ✅ | 15+15+14+14+12 = 70 (toán) |
| φ⁻¹ + φ⁻³ = 0.854 | ✅ | 0.618034 + 0.236068 = 0.854102 (toán) |
| VP-Tree: Yianilos 1993 | ✅ | SODA 1993 pp.311-321. Also: Uhlmann 1991 (independent) |
| NRC-VAD v2: 55,133 terms | ✅ | arXiv:2503.23547: 44,928 unigrams + 10,205 MWEs |
| Liquid TC Networks: Hasani 2021 | ✅ | AAAI 2021, arXiv:2006.04439 (2020) |
| Jaeger golden ratio: real paper | ✅ | IEEE AIPR 2021, arXiv:2006.04751 |
| BCM: Bienenstock-Cooper-Munro 1982 | ✅ | J. Neuroscience 1982 |
| DBSCAN: Ester et al. 1996 | ✅ | KDD-96 proceedings |
| ACT-R: B_i = ln(Σ t_j^{-d}), d=0.5 | ✅ | Anderson & Lebiere 1998 + Petrov 2006 approximation |
| Ebbinghaus 1885 forgetting curve | ✅ | "Über das Gedächtnis" |
| log_φ(n) ≈ 1.44 × log₂(n) | ✅ | Toán: ratio = 1.4404... (constant) |

### ĐÃ SỬA THÊM (từ research agent)

| # | Vấn đề | Trước | Sau |
|---|--------|-------|-----|
| V7 | **Wixted & Ebbesen journal** | "Cognitive Psychology" | **Psychological Science** Vol.2, 1991 |
| V8 | **NRC-VAD v1 original count** | 19,971 | **20,007 gốc** → 19,971 sau filter (ghi rõ) |
| V9 | **Liquid TC closed-form** | attributed to AAAI 2021 | Closed-form from **Nature MI 2022** (arXiv:2106.13898) |

### CẦN LƯU Ý (không sai nhưng cần ghi rõ)

| Claim | Lưu ý |
|-------|-------|
| β=0.618 "in range 0.5-0.7 for biological" | KWW β range 0<β<1 toàn bộ. Range 0.5-0.7 là cho **polymer/glass physics**, KHÔNG phải biological memory. Threshold β=1/2 (Phillips 1996). Không có paper dùng KWW cho memory forgetting. Đây là **design choice**, không empirical fit. |
| NRC-VAD v2 scale | v2 dùng 7-point Likert (-3 to +3), v1 dùng best-worst (0-1). Upgrade cần normalize. |
| Knuth hash 2654435761 | Đúng là closest prime tới exact value 2654435769. Cả 2 đều hoạt động tốt cho hashing. |
| BCM θ_M = ⟨y⟩^p | Spec dùng simplified form. Original 1982 paper: θ_M = ⟨y²⟩ (mean of squares). Common simplified: θ_M = ⟨y⟩^p. Cả 2 đều used in literature. |
| Jaeger golden ratio | Paper thật (IEEE AIPR 2021), nhưng là 1 model cụ thể, chưa mainstream. φ cho decay/threshold là design choice của Origin. |
