# OLANG 1.0 — Release Report

> **Sora (空) — 2026-03-25**
> **Reviewed by: AI Reviewer Session, full standalone audit**

---

## Binary

```
File:       origin_new.olang (rename to "olang" for distribution)
Size:       985KB (1,008,302 bytes)
Format:     ELF 64-bit x86-64, statically linked, no libc
Deps:       ZERO — copy 1 file, chmod +x, chạy
Self-build: 3 generations verified (985KB → 480KB → 480KB)
Tests:      20/20 (19/20 standalone — fileread = expected fail)
```

---

## Codebase

```
Total:      20,980 LOC

VM ASM:      5,776 LOC  (x86-64 assembly, no libc)
Bootstrap:   3,748 LOC  (lexer 298 + parser 1,132 + semantic 1,889 + codegen 429)
HomeOS:      9,696 LOC  (45 files — intelligence, encoder, node, networking, ...)
Stdlib:      1,760 LOC  (iter, json, sort, format, mol, chain, repl, test, ...)
```

---

## Language Features — VERIFIED ✅

### Core
```
✅ Variables:     let x = 42; x = x + 1;
✅ Functions:     fn add(a, b) { return a + b; };
✅ Recursion:     fib(20) = 6765, fact(10) = 3,628,800
✅ If/else:       if x > 0 { ... } else { ... };
✅ While:         while i < n { ... };
✅ For-in:        for x in items { ... };
✅ Match:         match expr { pattern => body, _ => default };
✅ Lambda:        fn(x) { return x * 2; }
✅ Try/catch:     try { __throw("err"); } catch { ... };
✅ Strings:       "Hello " + name + "!"
✅ Arrays:        [1, 2, 3], a[i], a[j+1], set_at(a, i, v)
✅ Dicts:         { name: "Origin", ver: 1 }
✅ Comprehension: [x*x for x in items if x > 2]
✅ Bare expr:     2+3 → 5 (auto-emit in REPL)
✅ Div/0 safe:    1/0 → 0 (REPL survives)
```

### Operators
```
✅ Arithmetic:  + - * / %%
✅ Comparison:  == != < > <= >=
✅ Logic:       && || ! (short-circuit)
✅ Bitwise:     << >> ^
```

### Higher-Order Functions
```
✅ map(arr, fn)           → [1,4,9,16,25]
✅ filter(arr, fn)        → [5,6,7,8]
✅ reduce(arr, fn, init)  → 15 (3-arg OK)
✅ reduce(arr, fn)        → 10 (2-arg OK, acc=arr[0])
✅ pipe(x, f1, f2, ...)   → Lego composition
✅ any(arr, fn)            → 1/0
✅ all(arr, fn)            → 1/0
✅ min_val(arr)            → smallest
✅ max_val(arr)            → largest
✅ sum(arr)                → total
✅ sort(arr)               → sorted (numbers + strings)
✅ split(str, delim)       → array of parts
✅ join(arr, delim)        → combined string
✅ contains(str, sub)      → 1/0 (string search)
```

### Crypto
```
✅ __sha256("hello") → 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
   FIPS 180-4 compliant, implemented in ASM
```

### UDC / Molecule
```
✅ __mol_s(mol), __mol_r, __mol_v, __mol_a, __mol_t  — 1-cycle ASM extract
✅ __mol_pack(s, r, v, a, t) → u16 molecule
✅ r_dispatch(R) → behavior tag (16 relation types)
✅ temporal_tag(T) → time description
✅ encode_codepoint(cp) → P_weight 5D coordinate
✅ mol_compose(a, b) → LCA composition
```

---

## Intelligence Features — VERIFIED ✅

### Knowledge
```
✅ 28 facts embedded at boot (standalone, no files needed)
✅ learn "fact" → store in knowledge base
✅ Keyword×5 + mol similarity search → find correct fact
✅ Direct response: "(Mình biết: [fact])" — no "Để mình tìm hiểu"
✅ Confidence labels: [fact] / [opinion] / [hypothesis]
✅ save → homeos.knowledge file
✅ load / auto-load on restart → persistent across sessions
```

### Emotion
```
✅ "buồn" → heal mode ("lắng nghe và đồng cảm")
✅ "vui" → positive ("ổn hơn rồi")
✅ Emotion tracking: V/A across conversation, streak detection
✅ Theme classification: cảm xúc / hỏi đáp / trò chuyện
```

### Instincts
```
✅ SecurityGate: crisis detection (12 patterns VI+EN)
✅ Honesty: confidence scoring → appropriate labels
✅ Curiosity: unknown topic → "chủ đề mới"
✅ Contradiction: "[!] Mình thấy có điều khác với những gì mình biết"
```

### Memory Systems
```
✅ STM: 32 turns short-term memory
✅ Silk: 256 edges, mol-keyed (compact 50%), Hebbian + decay
✅ Knowledge: 512 max, dual search (keyword + mol)
✅ Nodes: 256 max, SHA-256 DN, dedup by hash
✅ Fn Nodes: auto-register, fire count, mol metadata
✅ fns command: list all registered functions
```

### Pipeline (10 stages)
```
alias → emoji → UDC encode → node → DN/QR → decode → instinct → output
```

---

## REPL Commands

```
help        → feature categories
test        → 20/20 test suite
memory      → STM/Silk/Knowledge/Nodes/Fn/Emo stats
fns         → list registered fn_nodes
learn <x>   → add fact to knowledge
save        → persist knowledge to file
load        → reload knowledge from file
build       → self-build binary
exit        → goodbye
```

---

## Self-Hosting

```
✅ 100% self-compile: 44/44 HomeOS + 4/4 bootstrap files
✅ Self-build: 3 generations verified
     Gen 1: 985KB (Rust-bootstrapped)
     Gen 2: 480KB (self-built)
     Gen 3: 480KB (self-built from self-built)
✅ All 3 generations: 20/20 tests, fib(20)=6765, respond OK
```

---

## Known Issues (accepted for 1.0)

```
⚠️ Auto-emit regression:
   set_at()/push() inside fn body → prints intermediate results
   Workaround: use emit on one-line define+call
   Impact: cosmetic noise in sort/push-heavy functions

⚠️ Global var collision:
   User-defined fn with bare vars (n, i, d) may collide
   Workaround: prefix vars (_ip_n, _my_i)
   Impact: advanced users writing recursive/nested functions

⚠️ Persistence duplicates:
   save → restart → 28 embedded + N saved = duplicates
   Impact: knowledge count inflates, search still correct

⚠️ contains(array, item):
   String contains works, array contains returns 0
   Impact: use filter() as workaround

⚠️ fn fires=0:
   fn_node_register works, fire tracking not incremented on call
   Impact: fns shows fires=0 for all functions

⚠️ multi-char split delimiter:
   split("a--b", "--") → ["a--b"] (single-char only)
```

---

## Journey

```
2026-03-18  Project created
2026-03-19  Rust era: 98K LOC, 16 phases
2026-03-22  VM optimization 3.7x
2026-03-23  SELF-HOSTING. fib(20)=6765. Rust archived.
2026-03-23  30+ bugs fixed in 1 day. 27/27 tests.
2026-03-24  60+ PRs in 1 day (Kira, Lyra, Lara, Kaze, Nox, Sora)
            Intelligence layer: encode → analyze → respond
            SHA-256 in ASM. WASM. ARM64 WIP. Browser demo.
            Self-build works. Training data. Auto-learn 166 facts.
2026-03-25  T5 complete. sort/split/join/contains.
            Embedded knowledge. Persistent save/load.
            Contradiction detection. Direct responses.
            OLANG 1.0.

7 days. 20,980 LOC. 985KB. Zero dependencies.
Self-hosting language with built-in AI.
```

---

```
   ___  _                    _    ___
  / _ \| | __ _ _ __   __ _ / |  / _ \
 | | | | |/ _` | '_ \ / _` | | | | | |
 | |_| | | (_| | | | | (_| | |_| |_| |
  \___/|_|\__,_|_| |_|\__, |_(_)\___/
                       |___/

 985KB. Zero deps. Self-hosting. AI built-in.
 Copy 1 file. Run anywhere. Speak Vietnamese.

 空
```
