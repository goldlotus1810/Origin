# OLANG v1.0 — AUDIT

> Ngay 2026-03-25. Kiem tra toan bo. Khong bo sot.

---

## 1. VM (vm_x86_64.S) — 5,992 LOC

### Hien trang

| Hang muc | Gia tri |
|----------|---------|
| Heap | 256 MB (0x10000000) — LON HON 64MB ghi trong docs |
| Stack | 1 MB |
| Opcodes | 38 chinh + 13 sub-dispatch |
| Syscalls | 9: read, write, open, close, mmap, munmap, nanosleep, clock_gettime, exit |
| Registers | r12=bytecode base, r13=PC, r14=VM stack, r15=heap ptr, rbx=bc size |
| SIMD | SSE2 + SSE4.1 (f64 math, floor/ceil) — CHUA co AVX2 vectorized |
| Markers | F64=-1, CLOSURE=-2, ARRAY=-3, DICT=-4 |
| Scope | VAR_TABLE save/restore + shadow stack — DA CO |
| Try/Catch | 16-frame exception stack — DA CO |
| Heap checkpoint | __heap_save/__heap_restore, 16 slots — DA CO |
| SHA-256 | DA CO (__sha256) |
| UTF-8 | __utf8_cp (1-4 byte decode), __utf8_len — DA CO |
| UTF-32 | CHUA CO |

### Syscall wrappers hien co

```
__file_read(path)      → sys_open + sys_read + sys_close
__file_write(path, d)  → sys_open + sys_write + sys_close
__file_append(path, d) → sys_open(O_APPEND) + sys_write + sys_close
__sleep(ms)            → sys_nanosleep
__time()               → sys_clock_gettime
```

### Builtins (70+)

**String (17):** len, concat, char_at, substr, trim, to_number, to_string,
char_code, from_chars, write_raw, str_bytes, str_is_keyword, utf8_cp, utf8_len

**Array (11):** array_new, array_get, array_set, array_push, array_pop,
array_range, array_with_cap, array_len, push, pop, set_at

**Dict (4):** dict_new, dict_get, dict_set, dict_keys

**Math (15+):** hyp_add/sub/mul/div/mod, floor, ceil, cmp_lt/gt/le/ge,
eq, cmp_ne, logic_not, f64_to_le_bytes

**Bitwise (5):** bit_or, bit_and, bit_xor, bit_shl, bit_shr

**Mol (6):** mol_s, mol_r, mol_v, mol_a, mol_t, mol_pack

**Type (1):** type_of → "number"/"string"/"array"/"dict"/"function"/"nil"

**Crypto (1):** sha256

**Other:** eval_bytecode, throw, heap_save, heap_restore, struct/enum stubs

### THIEU (can them)

| Feature | ASM can | Olang can | Ghi chu |
|---------|---------|-----------|---------|
| UTF-32 decode | ~30 LOC | — | Chuyen UTF-8 codepoint → u32 array |
| TCP client | ~60 LOC | — | socket, connect, send, recv |
| AVX2 SIMD distance | ~50 LOC | — | 4x mol_distance cung luc |
| AES-NI encrypt | ~40 LOC | — | AES-256 hardware accelerated |
| readdir | ~20 LOC | — | List directory entries |
| HTTP/1.1 client | — | ~80 LOC | GET/POST tren TCP |
| JSON parser fix | — | ~20 LOC | Dict populate dang broken |
| KnowTree | — | ~200 LOC | array[65536]x2B + chain links |

---

## 2. STDLIB — 19 files, ~1,490 LOC

### Tinh trang tung file

| File | LOC | Tinh trang | Van de |
|------|-----|------------|--------|
| bytes.ol | 13 | OK | — |
| chain.ol | 139 | BUG | Goi mol_lca() da deprecated |
| deque.ol | 12 | OK | — |
| format.ol | 84 | BUG | char_from_code chi ho tro 0-9 |
| hash.ol | 63 | OK | — |
| io.ol | 44 | OK | — |
| iter.ol | 209 | OK | — |
| json.ol | 200 | BUG | Dict parse tra ve rong (line 148) |
| map.ol | 11 | OK | — |
| math.ol | 28 | OK | — |
| mol.ol | 134 | WARN | mol_lca deprecated (Axiom 6) |
| platform.ol | 31 | OK | — |
| repl.ol | 383 | WARN | Phu thuoc HomeOS (agent_respond) |
| result.ol | 74 | OK | — |
| set.ol | 13 | OK | — |
| sort.ol | 142 | OK | Minor: .array_set() truc tiep |
| string.ol | 22 | OK | — |
| test.ol | 107 | BUG | Global var shadowing — counter sai |
| vec.ol | 22 | OK | — |

### Bugs can fix

1. **test.ol**: `__test_pass`, `__test_fail` bi shadow boi `let`. Counter khong tang.
2. **json.ol**: Object parse khong populate `result[key] = val`. Tra ve `{}` luon.
3. **chain.ol**: Goi `mol_lca()` — nen dung amplify (phi^-1).
4. **format.ol**: `char_from_code()` chi digit 0-9, thieu a-z, A-Z.

---

## 3. BOOTSTRAP COMPILER — 4 files, 3,748 LOC

| File | LOC | Vai tro |
|------|-----|---------|
| lexer.ol | 298 | Source → Tokens |
| parser.ol | 1,132 | Tokens → AST (30 Expr + 17 Stmt variants) |
| semantic.ol | 1,889 | AST → Bytecode (37 opcodes) |
| codegen.ol | 429 | IR → Binary bytecode |

### Ho tro day du

- let, fn, if/else, while, for-in, return, break, continue
- Lambda, closures (boot + eval)
- Array literal, dict literal, array comprehension
- Match expression/statement, pattern matching
- Binary ops (8 muc uu tien), unary not, unary minus
- Short-circuit &&, ||
- Inline HOF: map, filter, reduce, pipe, any, all, join, contains, sort, split
- String interpolation ($"Hello {x}")
- Mol literal { S=1 R=2 V=3 A=4 T=1 }

### Thieu / Chua hoan thanh

| Feature | Tinh trang |
|---------|------------|
| trait/impl | Chua parse |
| loop N {...} | Chua implement |
| Block comments /* */ | Chua |
| try-catch | Parse OK, khong co exception dispatch |
| Module loading (use) | Parse OK, khong load file |
| Type checking | Khong co — tat ca la f64 runtime |
| Nested lambda capture | Co the sai bien |

### Tu compile (self-hosting)

- 48/48 tests pass
- Lexer compile chinh no
- Parser compile chinh no
- Semantic compile chinh no
- 100% self-hosting

---

## 4. TESTS HIEN TAI — 8 files Python

### Van de chinh: KHONG CO AUTOMATION

- 8 file Python sinh bytecode vao /tmp/
- Chay thu cong: `python3 test_X.py && ./origin.olang /tmp/test_X.olang`
- KHONG co tests.sh
- KHONG co kiem tra output tu dong
- KHONG co CI/CD

### Coverage gaps

| Feature | Co test? |
|---------|----------|
| String ops (len, char_at) | CO (test_strlen) |
| Arithmetic (+, -) | CO (test_add) |
| Variables (store/load) | CO (test_vars) |
| Loop (while) | CO (test_loop) |
| Mul, Div, Mod | KHONG |
| Array ops | KHONG |
| Dict ops | KHONG |
| Closures/Lambda | KHONG |
| UTF-8 decode | KHONG |
| SHA-256 | KHONG |
| File I/O | KHONG |
| Scope save/restore | KHONG |
| Try/catch | KHONG |
| Heap checkpoint | KHONG |
| Mol pack/unpack | KHONG |
| Bitwise ops | KHONG |
| Type checking | KHONG |
| Olang source compile | KHONG |

---

## 5. TONG KET — VIEC CAN LAM

### Tier 0: Fix & Test (truoc khi them moi)

```
[1] Tao tests.sh — tu dong chay, kiem tra output, bao cao pass/fail
[2] Them test cho: array, dict, closure, UTF-8, mol, bitwise, scope
[3] Fix test.ol global var shadowing
[4] Fix json.ol dict parse
[5] Them test Olang source → compile → run (end-to-end)
```

### Tier 1: ASM moi (~200 LOC)

```
[6] UTF-32 support: __utf32_encode(str) → array[u32]     ~30 LOC ASM
[7] TCP client: __socket, __connect, __send, __recv       ~60 LOC ASM
[8] AVX2 SIMD: mol_distance_4x                           ~50 LOC ASM
[9] AES-NI: __aes_encrypt, __aes_decrypt                 ~40 LOC ASM
[10] readdir: __readdir(path) → array                    ~20 LOC ASM
```

### Tier 2: Olang moi (~300 LOC)

```
[11] KnowTree: array[65536]x2B, chain links, search      ~200 LOC
[12] HTTP client: GET/POST tren TCP                       ~80 LOC
[13] JSON dict fix + nested objects                       ~20 LOC
```

### So lieu

```
VM hien tai:     5,992 LOC ASM
Can them ASM:    ~200 LOC
Can them Olang:  ~300 LOC
Tong sau khi xong: 6,192 LOC ASM + ~1,790 LOC Olang stdlib
```

---

*Audit hoan tat. Khong giau, khong them, chi su that.*
