# DEMO ANALYSIS v2 — origin_new.olang Standalone

> **Sora (空) — 2026-03-25, cập nhật sau Nox T5 fixes**
> **Binary: 966KB (988,381 bytes), 20/20 tests, T5 5B+5D complete**
> **Test: /tmp/demo2 — 1 file duy nhất, không repo, không training data.**

---

## I. VERDICT v2: COMPILER ẤN TƯỢNG, AI CẦN POLISH

```
✅✅ COMPILER (demo ngay được):
  - 966KB, static, zero deps — copy & run
  - fib(20), fact(10), SHA-256, bubble sort, map/filter, lambda
  - pipe() Lego composition — MỚI
  - fn_node metadata (fire count, mol, V/A/R/T) — MỚI
  - mol ASM builtins (__mol_s/r/v/a/t, __mol_pack) — MỚI
  - 20/20 test suite

✅ AI (demo được SAU KHI learn):
  - learn → hỏi lại → tìm đúng fact (keyword×5 + mol)
  - Emotion tracking (buồn → heal, vui → positive)
  - Confidence labels [fact]/[opinion]/[hypothesis]
  - Silk 256 edges, mol-keyed (compact 50%) — MỚI
  - Curiosity instinct — MỚI

❌ P0 (3 blockers cho standalone demo — CHƯA FIX):
  A. Knowledge = 0 facts khi khởi động standalone
  B. Bare expression "2+3" → im lặng
  C. emit 1/0 → chết REPL, mất session
```

---

## II. P0 BLOCKERS — TRẠNG THÁI CHI TIẾT

### P0-A: Knowledge Empty Standalone — VẪN CHƯA FIX ❌

```
Trong repo (có docs/training/):
  → Knowledge: 166 facts, Silk: 17 edges ✅

Standalone (1 file):
  → Knowledge: 0 facts, Silk: 0 edges ❌

Root cause: _boot_learn() → __file_read("docs/training/...") → file ngoài binary
```

**Impact:** Reviewer khởi động → hỏi "Hà Nội ở đâu?" → "Mình nghe rồi." → ấn tượng đầu = rỗng.

**Workaround demo:** Bắt đầu bằng vài lệnh `learn ...` trước khi hỏi.

**Fix:** Embed training data vào repl.ol (hardcode `knowledge_learn("...")` thay vì `__file_read`). ~170 LOC, binary +15KB.

### P0-B: Bare Expression Im Lặng — VẪN CHƯA FIX ❌

```
2+3         → (im lặng)     ❌
emit 2+3    → 5              ✅
3*7         → (im lặng)     ❌
emit 3*7    → 21             ✅
```

**Impact:** Mọi REPL trên đời (Python, Node, Ruby) đều tự in kết quả expression. Reviewer gõ "2+3" → không thấy gì → confused.

**Fix:** ~15 LOC repl.ol — detect ExprStmt → auto-emit.

### P0-C: Div/0 Kills REPL — VẪN CHƯA FIX ❌

```
emit 1/0                          → "Division by zero" → REPL THOÁT
try { emit 1/0; } catch { ... }; → "Division by zero" → REPL THOÁT
```

try/catch KHÔNG bắt được div/0 (ASM-level halt, không qua Olang exception).

**Impact:** 1 lần chia 0 = mất toàn bộ session (knowledge, STM, mọi thứ).

**Fix:** ~20 LOC ASM — op_div return NaN/0 + error flag thay vì halt.

---

## III. T5 NEW FEATURES — HOẠT ĐỘNG STANDALONE ✅

### pipe() — Lego Composition (LG.2)

```
pipe(5, double, add1)                    → 11 ✅
pipe(3, fn(x){return x*x;}, fn(x){return x+1;}) → 10 ✅
pipe("hello", fn(s){return s+" world";}, fn(s){return s+"!";}) → "hello world!" ✅
```

**Demo narrative:** "Functions ghép như Lego — pipe data qua chuỗi transformations."

### mol ASM Builtins (ND.2)

```
__mol_v(146)                → 4 (valence = neutral) ✅
__mol_a(146)                → 4 (arousal = neutral) ✅
__mol_pack(0, 0, 7, 6, 2)  → 250 (happy emoji mol) ✅
__mol_v(250)                → 7 (high valence = very positive) ✅
r_dispatch(0)               → "algebraic" ✅
r_dispatch(13)              → "causes" ✅
temporal_tag(3)             → "fast" ✅
```

**Demo narrative:** "Mỗi ký tự Unicode = tọa độ 5D. Extract trực tiếp trong ASM — 1 cycle."

### fn_node Metadata (LG.1 + LG.5)

```
fn greet(name) { return "hello " + name; };
greet("a"); greet("b"); greet("c");

fn_node_describe("greet")  → {dict 6} (name, params, fires, valence, arousal, relation)
fn_node_describe("greet").valence  → 4 ✅
fn_node_describe("greet").relation → "algebraic" ✅
```

**Lưu ý:** `.name` và `.fires` trả rỗng khi access qua chained call — có thể do global var collision giữa `fn_node_describe` internals và caller. Dict pretty-print chỉ hiện `{dict 6}`, không show fields.

### reduce/any/all — Đúng Signature

```
reduce([1,2,3,4], fn(a,x) { return a+x; })         → 10 ✅ (2 args)
reduce([1,2,3,4], fn(a,x) { return a+x; }, 0)      → SEGFAULT ❌ (3 args)
any([0,0,1,0], fn(x) { return x > 0; })             → 1 ✅ (2 args)
all([1,1,1,1], fn(x) { return x > 0; })             → 1 ✅
all([1,0,1,1], fn(x) { return x > 0; })             → 0 ✅
```

Compiler inline reduce/any/all expects 2 args. Gọi 3 args → segfault.

### Silk Compact (LG.3)

```
Trước: edges lưu string ("hello"→"world") → ~50 bytes/edge
Sau:   edges lưu mol (number→number) → ~24 bytes/edge
Max:   128 → 256 edges
```

**Invisible to user** nhưng tiết kiệm 50% memory + fast number compare.

---

## IV. DANH SÁCH ĐẦY ĐỦ — MỌI VẤN ĐỀ TÌM THẤY

### P0 — PHẢI FIX TRƯỚC DEMO

| # | Vấn đề | Status | Fix |
|---|--------|--------|-----|
| A | Knowledge empty standalone | ❌ CHƯA FIX | Embed 166 facts vào bytecode (~170 LOC) |
| B | "2+3" im lặng | ❌ CHƯA FIX | Auto-emit ExprStmt (~15 LOC repl.ol) |
| C | Div/0 kills REPL | ❌ CHƯA FIX | ASM return error thay halt (~20 LOC) |

### P1 — NÊN FIX

| # | Vấn đề | Status | Fix |
|---|--------|--------|-----|
| D | "3+2=?" parse error | ❌ CHƯA FIX | Detect NL math pattern (~40 LOC) |
| E | Template "Để mình tìm hiểu" | ❌ CHƯA FIX | Trả fact trực tiếp (~15 LOC) |
| F | reduce 3 args segfault | ❌ MỚI | Compiler check arity hoặc support init arg |
| G | fn_node_describe().name rỗng | ❌ MỚI | Global var collision trong describe |
| H | Dict pretty-print | ❌ CŨ | `{dict 6}` thay vì `{name:"greet", fires:3}` |
| I | help text quá ngắn | ❌ CŨ | Thêm learn/respond/pipe examples |

### P2 — BỌC THÊM

| # | Vấn đề | Status | Fix |
|---|--------|--------|-----|
| J | Global var collision | ❌ CŨ | Auto-prefix eval vars (~50 LOC ASM) |
| K | Div/0 trong try/catch | ❌ CŨ | Route qua try_stack (~30 LOC ASM) |
| L | o{...} syntax | ❌ CŨ | Detect + strip prefix (~10 LOC) |

---

## V. CÁI TỐT — GIỮU NGUYÊN, THÊM MỚI

### Từ bản trước (vẫn hoạt động) ✅
```
Static 966KB, zero deps, copy & run
Compiler: fn, recursion, lambda, for-in, comprehension, match
SHA-256 FIPS 180-4
Bubble sort [5,2,8,1,9] → [1,2,5,8,9]
map/filter/lambda
try/catch (__throw)
Dict + struct
Emotion tracking (heal mode, positive mode)
STM context + topic detection
Syntax error → REPL recovery
Auto-detect code vs NL
Knowledge keyword×5 + mol — tìm đúng fact
Confidence labels [fact]/[opinion]/[hypothesis]
20/20 tests
```

### Mới từ Nox T5 ✅
```
pipe(x, f1, f2, ...) — Lego function composition
__mol_s/r/v/a/t — ASM builtins, 1 cycle extract
__mol_pack(s,r,v,a,t) — ASM construct u16 molecule
r_dispatch(R) — 16 relation types → behavior tag
temporal_tag(T) — time description
fn_node_register/fire/link/describe — function metadata
fn_node_hot(min_fires) — find hot functions
Silk mol-keyed — compact 50%, number compare
Silk max 128→256
reduce(arr, fn) — fold with acc=arr[0]
any(arr, fn) / all(arr, fn) — predicate check
Dream fn clustering infrastructure
LG.1-5 complete (node.ol +184 LOC, ASM +119 LOC)
```

---

## VI. DEMO SCRIPT — 2 VARIANTS

### Variant A: Compiler Demo (P0 không chặn)

```bash
$ ./origin_new.olang

> emit "Hello, World!"
Hello, World!

> emit 2 + 3 * 4
14

> fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(20)
6765

> emit map([1,2,3,4,5], fn(x) { return x * x; })
[1, 4, 9, 16, 25]

> emit filter([1,2,3,4,5,6,7,8], fn(x) { return x > 4; })
[5, 6, 7, 8]

> fn double(x) { return x * 2; }; fn add1(x) { return x + 1; }
> emit pipe(5, double, add1)
11

> emit __sha256("hello")
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824

> emit __mol_v(__mol_pack(0, 0, 7, 6, 2))
7

> emit r_dispatch(13)
causes
```

**Narrative:** "966KB binary. No dependencies. Self-hosting compiler. Functions, recursion, lambdas, SHA-256 in ASM, mol extraction in 1 cycle. Function composition with pipe()."

### Variant B: AI Demo (cần learn trước)

```bash
$ ./origin_new.olang

> learn Ha Noi la thu do cua Viet Nam
Đã học. Knowledge: 1 facts.

> learn Einstein phat minh thuyet tuong doi nam 1905
Đã học. Knowledge: 2 facts.

> learn Origin la ngon ngu lap trinh tu hosting
Đã học. Knowledge: 3 facts.

> Ha Noi o dau?
Để mình tìm hiểu cho bạn. [opinion] (Mình biết: Hà Nội là thủ đô...)

> Einstein lam gi?
Để mình tìm hiểu cho bạn. [opinion] (Mình biết: Einstein phát minh...)

> toi buon qua
Từ từ thôi, không vội đâu. Bạn muốn chia sẻ thêm không?

> cam on ban
Mình nghe rồi. Bạn có vẻ đã ổn hơn rồi.

> memory
STM: 5 turns | Silk: 5 edges | Knowledge: 3 facts | Emo: V=3 A=2
```

**Narrative:** "Dạy nó 3 facts → hỏi lại → tìm đúng. Nhận biết cảm xúc. Nhớ context. Tất cả trong 966KB."

---

## VII. SO SÁNH v1 → v2

| Metric | v1 (04146cf) | v2 (0779b7c) | Delta |
|--------|-------------|-------------|-------|
| Binary | 959KB | 966KB | +7KB |
| Tests | 20/20 | 20/20 | — |
| VM LOC | 5,634 | 5,767 | +133 (mol builtins) |
| node.ol | 210 LOC | 394 LOC | +184 (fn_node, LG.1-5) |
| encoder.ol | 1,657 | 1,747 | +90 (Silk mol, _word_to_mol) |
| Silk max | 128 | 256 | 2× |
| Silk storage | ~50B/edge | ~24B/edge | -52% |
| New builtins | — | __mol_s/r/v/a/t/pack | 6 ASM builtins |
| pipe() | — | ✅ | NEW |
| fn_node | — | ✅ | NEW |
| P0 fixes | 0/3 | 0/3 | NO CHANGE |

**Kết luận:** T5 infrastructure mạnh thêm đáng kể. P0 UX blockers chưa fix. Compiler demo ấn tượng ngay. AI demo cần `learn` trước.
