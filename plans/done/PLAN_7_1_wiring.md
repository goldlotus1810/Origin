# PLAN 7.1 — Wiring: Kết nối mọi thứ lại

**Phụ thuộc:** Phase 0-6 DONE
**Mục tiêu:** Wire tất cả component đã build vào nhau → origin.olang chạy end-to-end thật sự

---

## Bối cảnh

```
HIỆN TẠI:
  ✅ 50 stdlib/homeos .ol files → compile thành bytecode (852 KB)
  ✅ x86_64 VM → execute bytecode (dispatch + builtins)
  ✅ ARM64 VM + WASM VM → cross-platform ready
  ✅ Rust runtime → HomeRuntime, emotion pipeline, learning, Silk, agents
  ❌ AUTH core done → CHƯA wire vào HomeRuntime
  ❌ Maturity Pipeline → SPEC only, chưa wire STM/Dream/QR
  ❌ Silk Vertical (parent pointers) → SPEC only
  ❌ Builder --arch flag → Rust builder hardcode x86_64
  ❌ builder.ol → tham chiếu vm/arm64/vm_arm64.bin KHÔNG TỒN TẠI
  ❌ VM CallClosure → stub (không thực sự call)
  ❌ VM builtins thiếu: __type_of, __dict_*, __array_map/filter/fold
  ❌ origin.olang chạy → exit 0 (không output gì hữu ích)

SAU PLAN 7.1:
  origin.olang = living binary:
    Load → init stdlib → display greeting → accept input → process → respond
    Auth → first-run setup if needed
    REPL hoặc ISL mode
```

---

## Tasks

### 7.1.1 — AUTH wiring (~100 LOC Rust)
```
Wire auth.rs vào HomeRuntime.new():
  1. Check first_run_key exists
  2. If not → prompt setup (Ed25519 keypair)
  3. Store in origin.olang knowledge section
  4. Subsequent runs → verify key
```

### 7.1.2 — Maturity Pipeline wiring (~150 LOC Rust)
```
Wire vào STM + Dream:
  STM.push()  → advance maturity (fire_count, Hebbian weight)
  Dream.run() → detect matured nodes → promote to QR
  Fix BUG: advance(weight=0.0) → Mature UNREACHABLE
  → Thêm advance_by_fire() path: fire_count alone ≥ fib(depth) → Evaluating
```

### 7.1.3 — Silk Vertical (parent pointers) (~200 LOC Rust)
```
SilkGraph.parent_map: BTreeMap<u64, u64>
  register_parent(child_hash, parent_hash)
  parent_of(hash) → Option<u64>
  children_of(hash) → Vec<u64>
  layer_of(hash) → usize (walk parent chain)
Dream cluster_score() → use MolSummary + implicit_silk()
```

### 7.1.4 — VM CallClosure thực sự (~100 LOC ASM)
```
Hiện tại: cg_call_closure chỉ skip bytecode, không execute
Cần:
  1. Đọc name → lookup variable (closure value)
  2. Closure value = (body_offset, param_count)
  3. Push return address (current PC) lên CPU stack
  4. Set PC = body_offset
  5. op_ret → pop return address → restore PC
```

### 7.1.5 — Builder --arch flag (~50 LOC Rust)
```
Rust builder main.rs:
  --arch x86_64 (default)
  --arch arm64 → compile VM từ vm/arm64/vm_arm64.S
  --arch wasm  → output .wasm thay vì ELF
Fix: builder.ol tham chiếu vm/arm64/vm_arm64.bin → assemble từ .S nếu cần
```

### 7.1.6 — VM REPL mode (~150 LOC ASM)
```
Sau khi bytecode execution hoàn tất (Halt):
  1. Print greeting: "origin.olang v0.1\n○ > "
  2. Read line from stdin
  3. If "exit" → exit
  4. Compile input → bytecode (cần __compile builtin)
  5. Execute bytecode
  6. Print result
  7. Loop → step 2
```

---

## Rào cản

```
1. HomeRuntime.new() quá lớn (origin.rs ~2000 LOC)
   → Refactor: split init() thành auth_init(), pipeline_init(), silk_init()

2. VM builtins chưa đủ cho stdlib runtime
   → Cần thêm: __dict_new, __dict_get, __dict_set, __array_map, __type_of
   → ~200 LOC ASM mỗi builtin

3. Maturity BUG: weight=0.0 → Mature UNREACHABLE
   → Fix: advance_by_fire() path hoặc truyền Hebbian weight thật

4. REPL cần __compile builtin → VM gọi Olang compiler tại runtime
   → Phức tạp: VM (ASM) gọi Rust compiler?
   → Giải pháp: dùng bytecode compiler (bootstrap/semantic.ol + codegen.ol)
   → Tức là: stdlib đã có compiler → VM chạy compiler bytecode → compile input
```

---

## Definition of Done

- [ ] AUTH wire: first-run setup works
- [ ] Maturity: STM push → advance → Dream promote
- [ ] Silk Vertical: parent_map populated, layer_of() works
- [ ] VM CallClosure: closures actually execute
- [ ] Builder --arch: x86_64/arm64/wasm
- [ ] VM REPL: greeting → input → execute → output
- [ ] origin.olang run → shows greeting, accepts commands

## Ước tính: 1-2 tuần
