# OLANG ROADMAP — Từ 7/10 → 10/10

> Mỗi Sprint = 2 tuần. Mỗi item = benchmark trước/sau + tests pass.
> Nox check mỗi ngày: `bash tools/nox_check.sh`
> Máy chạy 24/7: `nohup bash tools/evolve.sh &`

---

## SPRINT 1: Fix VM Bugs (Tuần 1-2)

Mục tiêu: 86/86 tests, không còn workaround.

```
S1.1  Fix JSON object parse (test_json_object FAIL)
      File: stdlib/json_parse.ol + test/test_json_object.ol
      Vấn đề: dict parse cần char_from_code (đã có builtin)
      Test: echo '{"x":42}' parse → d.x == 42
      Effort: ~30 phút

S1.2  Fix empty dict {} parse error
      File: stdlib/bootstrap/parser.ol
      Vấn đề: parser treats {} as block, not empty dict
      Fix: check token after { — if } follows immediately → empty dict
      Test: let d = {}; emit __type_of(d); → "dict"
      Effort: ~1 giờ

S1.3  Fix string concat >4 ops/line
      File: stdlib/bootstrap/semantic.ol hoặc vm_x86_64.S
      Vấn đề: chuỗi concat dài bị mất data
      Investigate: dump bytecode cho "a"+"b"+"c"+"d"+"e", xem opcode
      Test: let s = "a"+"b"+"c"+"d"+"e"; emit len(s); → 5
      Effort: ~2 giờ (investigate) + ~2 giờ (fix)

S1.4  Fix boot fn local reassignment
      File: vm_x86_64.S (var_store_hash)
      Vấn đề: let x = x+1 trong boot fn = shadow
      Root cause: bootstrap compiler emits Store thay StoreUpdate
      Fix: semantic.ol emit StoreUpdate khi var exists in locals
      Cẩn thận: StoreUpdate qua codegen dispatch khác Store
      Test: file test với while + let counter → counter == expected
      Effort: ~4 giờ (compiler + VM interaction)

DONE khi: 86/86 tests, JSON object PASS, {} works, concat OK
Benchmark: chạy tools/benchmark.sh trước/sau — không regression
```

---

## SPRINT 2: Auto-JIT (Tuần 3-4)

Mục tiêu: Olang tự detect hot function → compile native → replace.

```
S2.1  Profiler hook trong VM
      File: vm_x86_64.S
      Thêm: counter mỗi op_call → nếu function gọi > 1000 lần → flag hot
      BSS: hot_fn_hash[16] + hot_fn_count[16]
      Effort: ~2 giờ ASM

S2.2  JIT trigger trong interpreter
      File: vm_x86_64.S (op_call path)
      Logic: if hot_fn_count > threshold → check jit_cache
             if jit_cache[hash] exists → call native directly
             else → interpret as usual
      Effort: ~3 giờ ASM

S2.3  General function JIT compiler
      File: stdlib/jit_compile.ol (expand from fib-only)
      Logic: read function bytecode → translate to x86-64 via asm_emit.ol
             Handle: if/else → cmp+jcc, while → loop, let → reg alloc
             Start simple: integer arithmetic functions only
      Effort: ~1 tuần (biggest item)

S2.4  JIT cache + invalidation
      File: stdlib/jit_compile.ol
      Logic: cache[fn_hash] = native_ptr
             Invalidate when function redefined
      Effort: ~2 giờ

DONE khi: fib() auto-JIT'd after 1000 calls → subsequent calls = native speed
Benchmark: fib(30) first run = 350ms, after warmup = <10ms
```

---

## SPRINT 3: Module System (Tuần 5-6)

Mục tiêu: `use "math.ol"` loads file, makes pub functions available.

```
S3.1  Module loader
      File: stdlib/bootstrap/semantic.ol + vm_x86_64.S
      Syntax: use "file.ol"
      Logic: read file → compile → execute in isolated scope → export pub fns
      Cần thêm: __file_exists(path) builtin
      Effort: ~1 tuần

S3.2  Module cache
      Logic: đã load module → skip. Track loaded modules by path hash.
      Effort: ~2 giờ

S3.3  Stdlib module split
      Reorganize: stdlib/math.ol, stdlib/string.ol, stdlib/io.ol
      Each: standalone, use-able, no circular deps
      Effort: ~1 ngày

DONE khi: use "math.ol"; emit math_sqrt(144); → 12
```

---

## SPRINT 4: Error Handling (Tuần 7-8)

Mục tiêu: proper error types, no more silent crashes.

```
S4.1  Error type convention
      Design: functions return {ok: value} or {err: message}
      Helper: fn ok(v) { return {ok: v, err: ""}; }
              fn err(msg) { return {ok: 0, err: msg}; }
              fn is_ok(r) { return r.err == ""; }
      File: stdlib/result.ol (already exists, expand)
      Effort: ~2 giờ

S4.2  Stack traces on crash
      File: vm_x86_64.S
      Thêm: on segfault/error → print PC, last 5 function names
      BSS: call_stack[32] = last 32 function name hashes
      Effort: ~4 giờ ASM

S4.3  Builtin error returns
      Update: __tcp_connect returns {ok: fd} or {err: msg}
              __file_read returns {ok: content} or {err: msg}
      Backward compatible: old code still works (check return type)
      Effort: ~1 ngày per builtin group

DONE khi: errors show stack trace, builtins return Result type
```

---

## SPRINT 5: REPL + Developer Experience (Tuần 9-10)

```
S5.1  REPL multiline
      Detect: unclosed { or [ → continue reading next line
      Effort: ~4 giờ ASM (REPL loop change)

S5.2  String interpolation in eval
      Syntax: $"Hello {name}" already parsed, fix eval emission
      Effort: ~2 giờ

S5.3  REPL history + tab completion
      History: save to ~/.olang_history
      Tab: complete function names from var_table
      Effort: ~1 ngày

S5.4  Documentation generator
      Parse: /// comment above pub fn → generate docs
      Output: markdown file
      Effort: ~1 ngày Olang

DONE khi: paste multiline code in REPL, tab-complete functions
```

---

## SPRINT 6: HTTPS + Real Crypto (Tuần 11-12)

```
S6.1  Real SHA-512 multi-block
      Current: single block only (< 112 bytes)
      Fix: loop over 128-byte blocks
      Effort: ~2 giờ ASM

S6.2  X25519 key exchange
      Need: modular arithmetic mod 2^255-19
      Implement in Olang (not ASM) using bigint arrays
      ~200 LOC Olang
      Effort: ~3 ngày

S6.3  HKDF-SHA256
      Standard key derivation. Pure Olang using __sha256.
      ~50 LOC
      Effort: ~2 giờ

S6.4  TLS 1.3 record layer
      ClientHello → ServerHello → encrypted data
      ~200 LOC Olang
      Effort: ~1 tuần

S6.5  Real Ed25519
      Need X25519 math (shared with S6.2)
      ~150 LOC Olang
      Effort: ~2 ngày

DONE khi: https_get("https://api.github.com") returns JSON
```

---

## SPRINT 7: Regex + Unicode (Tuần 13-14)

```
S7.1  Regex engine (NFA-based)
      File: stdlib/regex.ol
      Support: . * + ? [] () | ^ $
      Implement: Thompson NFA construction + simulation
      ~300 LOC Olang
      Effort: ~1 tuần

S7.2  Unicode normalization (NFC)
      File: stdlib/unicode.ol
      Table: decomposition mappings (~5KB data)
      Effort: ~3 ngày

DONE khi: regex_match("hello", "h.l+o") → 1
```

---

## SPRINT 8: Auto-JIT Complete + Swarm (Tuần 15-16)

```
S8.1  Auto-JIT for loops
      Detect: while loop with >10K iterations → JIT compile loop body
      Translate: loop bytecode → native x86-64 loop
      Effort: ~1 tuần

S8.2  Swarm evolution framework
      File: stdlib/evolve.ol
      Logic: spawn N Olang instances, each solves problem differently
             Compare results, keep best, mutate, repeat
      Effort: ~1 tuần

S8.3  Self-patching pipeline
      Logic: profile → detect bottleneck → generate patch → verify → deploy
      First target: auto-JIT hot functions without manual code
      Effort: ~1 tuần

DONE khi: 2 Olang instances tự optimize nhau, performance improves each gen
```

---

## TIMELINE SUMMARY

```
Sprint 1 (Wk 1-2):   Fix VM bugs          → 85/86 tests ✅ (3/4 bugs fixed)
Sprint 2 (Wk 3-4):   Auto-JIT             → fib/fact/sum JIT = C speed ✅
Sprint 3 (Wk 5-6):   Modules              → use "file.ol" works ✅ (text expansion)
Sprint 4 (Wk 7-8):   Error handling       → Result types ✅, crash PC trace ✅
Sprint 5 (Wk 9-10):  Developer UX         → multiline REPL, tab completion
Sprint 6 (Wk 11-12): HTTPS + crypto       → real TLS, Ed25519
Sprint 7 (Wk 13-14): Regex + Unicode      → pattern matching, NFC
Sprint 8 (Wk 15-16): Swarm evolution      → self-optimizing ecosystem

Total: 16 tuần (~4 tháng)
Result: Olang 10/10 mọi category
```

---

## SCORING TARGET

```
                    Now    After
Ngôn ngữ:          7/10 → 10/10 (modules, error, regex, unicode)
Performance:        9/10 → 10/10 (auto-JIT all hot paths)
Ecosystem:          4/10 →  8/10 (modules, docs, REPL, history)
Innovation:        10/10 → 10/10 (swarm evolution, self-patch)
```

---

## NOX DAILY ROUTINE

```
1. bash tools/nox_check.sh          → xem status
2. Kiểm tra logs/evolution_data.csv → performance trend
3. git log --oneline -5              → recent changes
4. Chọn 1 item từ sprint hiện tại   → implement
5. bash tests.sh                     → verify
6. bash tools/benchmark.sh           → no regression
7. git commit + push                 → save
8. Repeat
```
