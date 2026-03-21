# PLAN 2.1 — Stdlib mở rộng (~1200 LOC Olang)

**Phụ thuộc:** Phase 0 DONE (compiler works), Phase 1 DONE (VMs work)
**Mục tiêu:** Thêm 8 stdlib modules cho HomeOS logic (Phase 2.2-2.4 cần)
**Song song:** Có thể chia cho 2-3 sessions

---

## Modules cần viết

### Group A — Data patterns (có thể làm song song)

| File | LOC | Mô tả | Cần cho |
|------|-----|-------|---------|
| `result.ol` | ~80 | Option/Result patterns: ok(), err(), unwrap(), map() | 2.2-2.4 error handling |
| `iter.ol` | ~150 | Iterator: map, filter, reduce, zip, enumerate, take, skip | 2.3 dream clustering |
| `sort.ol` | ~120 | Quicksort + mergesort cho Vec | 2.3 dream scoring |

### Group B — String/Format (có thể làm song song)

| File | LOC | Mô tả | Cần cho |
|------|-----|-------|---------|
| `format.ol` | ~150 | f64→string, int→string, template formatting | 2.2 response rendering |
| `json.ol` | ~200 | Parse/emit JSON (cho API, config) | 2.4 agent communication |

### Group C — Core helpers (phụ thuộc Group A)

| File | LOC | Mô tả | Cần cho |
|------|-----|-------|---------|
| `hash.ol` | ~100 | FNV-1a, chain_hash, similarity_hash | 2.3 silk ops |
| `mol.ol` | ~200 | Molecule encode/decode, evolve, dimension_delta | 2.2-2.4 mọi thứ |
| `chain.ol` | ~200 | MolecularChain helpers: lca, concat, split, compare | 2.3-2.4 knowledge |

---

## API Design

### result.ol
```
fn ok(val) { return { tag: "ok", val: val }; }
fn err(msg) { return { tag: "err", msg: msg }; }
fn is_ok(r) { return r.tag == "ok"; }
fn is_err(r) { return r.tag == "err"; }
fn unwrap(r) { if is_ok(r) { return r.val; } return 0; }
fn unwrap_or(r, default) { if is_ok(r) { return r.val; } return default; }
fn map_result(r, f) { if is_ok(r) { return ok(f(r.val)); } return r; }
```

### iter.ol
```
fn map(arr, f) { ... }      // [a] → [f(a)]
fn filter(arr, f) { ... }   // [a] → [a where f(a)]
fn reduce(arr, f, init) { ... } // [a] → single value
fn zip(a, b) { ... }        // [a],[b] → [[a,b]]
fn enumerate(arr) { ... }   // [a] → [[0,a],[1,a],...]
fn take(arr, n) { ... }     // first N elements
fn skip(arr, n) { ... }     // skip N elements
fn any(arr, f) { ... }      // true if any f(a)
fn all(arr, f) { ... }      // true if all f(a)
fn find(arr, f) { ... }     // first where f(a)
fn flat_map(arr, f) { ... } // map + flatten
```

### sort.ol
```
fn quicksort(arr, cmp) { ... }
fn mergesort(arr, cmp) { ... }
fn sort(arr) { ... }              // default: quicksort ascending
fn sort_by(arr, key_fn) { ... }   // sort by key function
fn is_sorted(arr, cmp) { ... }
fn binary_search(sorted, val, cmp) { ... }
```

### format.ol
```
fn f64_to_str(val, decimals) { ... }  // 3.14159 → "3.14"
fn int_to_str(val) { ... }            // 42 → "42"
fn pad_left(s, width, ch) { ... }     // "42" → "  42"
fn pad_right(s, width, ch) { ... }
fn fmt(template, args) { ... }        // "hello {}" → "hello world"
fn hex(val) { ... }                    // 255 → "ff"
```

### json.ol
```
fn json_parse(s) { ... }    // string → value (dict/vec/num/str)
fn json_emit(val) { ... }   // value → string
fn json_get(obj, key) { ... }
fn json_set(obj, key, val) { ... }
```

### hash.ol
```
fn fnv1a(data) { ... }           // bytes → u64 hash
fn chain_hash(chain) { ... }     // MolecularChain(Vec<u16>) → u64  ⚠️ v2: hash on 2B/link
fn similarity(a, b) { ... }      // 2 chains → 0.0-1.0 score
fn distance_5d(mol_a, mol_b) { ... } // 5D distance on packed u16 molecules
```

### mol.ol
```
// ⚠️ v2: Molecule = u16 packed [S:4][R:4][V:3][A:3][T:2]
fn pack(shape, rel, val, aro, time) { ... }     // → u16 packed Molecule
fn unpack(mol) { ... }                           // u16 → {s, r, v, a, t} via bit ops
fn evolve(mol, dim, new_val) { ... }           // mutate 1 dimension
fn dimension_delta(a, b) { ... }                // which dim differs most
fn mol_to_str(mol) { ... }                      // debug display
```

### chain.ol
```
fn chain_new() { ... }            // empty chain
fn chain_append(c, mol) { ... }   // add molecule
fn chain_lca(a, b) { ... }        // lowest common ancestor
fn chain_concat(a, b) { ... }     // join chains
fn chain_split(c, pos) { ... }    // split at position
fn chain_compare(a, b) { ... }    // -1/0/1
fn chain_len(c) { ... }           // molecule count
fn chain_get(c, idx) { ... }      // get nth molecule
```

---

## Test Plan

Mỗi module cần ≥ 3 tests:
```
// test_result.ol
assert(is_ok(ok(42)));
assert(!is_ok(err("fail")));
assert(unwrap(ok(42)) == 42);
assert(unwrap_or(err("x"), 0) == 0);

// test_sort.ol
let arr = [3, 1, 4, 1, 5];
let sorted = sort(arr);
assert(sorted[0] == 1);
assert(sorted[4] == 5);

// etc.
```

---

## Definition of Done

- [ ] 8 files tồn tại trong `stdlib/`
- [ ] Mỗi file compile thành công (`cargo test` qua Rust VM)
- [ ] Mỗi file có ≥ 3 tests pass
- [ ] `iter.ol` + `sort.ol` đủ cho dream clustering
- [ ] `format.ol` đủ cho response rendering
- [ ] `mol.ol` + `chain.ol` đủ cho emotion pipeline

## Phân việc gợi ý

```
Session A: result.ol + iter.ol + sort.ol (Group A)
Session B: format.ol + json.ol (Group B)
Session C: hash.ol + mol.ol + chain.ol (Group C, sau khi A xong)
```

## Ước tính: 1-2 ngày (3 sessions song song)
