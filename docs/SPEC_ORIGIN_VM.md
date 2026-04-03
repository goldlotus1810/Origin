# ORIGIN VM SPECIFICATION v2.0

> Tài liệu gốc. VM PHẢI được build theo spec này.
> Ngày: 2026-04-03. Tác giả: Lupin + Nox.

---

## 1. TỔNG QUAN

Origin là một **virtual machine độc lập** chứa:
- Bytecode interpreter (chạy Olang)
- Brain (KnowTree, Silk, Pipeline, Instincts)
- Memory manager (GC)
- I/O subsystem
- Self-modification engine

Origin **ăn** mọi thứ: Julia code, Python code, JSON, files, data. Chúng là thức ăn — Origin tiêu hóa và lưu vào KnowTree.

**Mục tiêu kỹ thuật:**
- Binary: < 2MB (không dependency)
- Startup: < 10ms
- UTF-8 native
- Self-hosting: Olang compiler viết bằng Olang chạy trên Origin VM
- Self-modify: VM có thể sửa code đang chạy

**Ngôn ngữ implementation:** C (portable, nhỏ, kiểm soát memory)

---

## 2. VALUE REPRESENTATION

### 2.1 NaN Boxing (64-bit tagged values)

Mọi giá trị trong Origin là 64-bit, sử dụng NaN boxing:

```c
// IEEE 754 double: khi exponent = 0x7FF và mantissa != 0 → NaN
// Canonical NaN: 0x7FF8000000000000
// Quiet NaN bit: bit 51 = 1
// Tag space: bits 48-50 (3 bits = 8 types)
// Payload: bits 0-47 (48 bits = 281TB address space, đủ cho con trỏ)

// Layout:
// Float:    normal IEEE 754 double (passes through as-is)
// Tagged:   0x7FFC_XXXX_XXXX_XXXX
//           bits 50-48 = type tag (0-7)
//           bits 47-0  = payload (pointer hoặc immediate)

#define TAG_MASK     0xFFFF000000000000ULL
#define QNAN         0x7FFC000000000000ULL
#define TAG_NIL      (QNAN | (0ULL << 48))  // 0x7FFC000000000000
#define TAG_BOOL     (QNAN | (1ULL << 48))  // 0x7FFD000000000000
#define TAG_INT      (QNAN | (2ULL << 48))  // 0x7FFE000000000000
#define TAG_MOL      (QNAN | (3ULL << 48))  // 0x7FFF000000000000
#define TAG_STRING   (QNAN | (4ULL << 48))  // signed bit set → pointer
#define TAG_ARRAY    (QNAN | (5ULL << 48))
#define TAG_DICT     (QNAN | (6ULL << 48))
#define TAG_OBJECT   (QNAN | (7ULL << 48))  // struct, closure, chain, etc.

// Bool = TAG_BOOL (NaN-tag singleton)
// Nghiên cứu: clox, Wren, LuaJIT, SpiderMonkey đều dùng NaN-tag cho bool.
// TRUE_VAL = QNAN | 3, FALSE_VAL = QNAN | 2
// IS_BOOL(v) = ((v | 1) == TRUE_VAL) — 1 bitwise op
// Comparisons (op_eq, op_lt, ...) trả val_float(1.0/0.0) cho confidence model.
// true/false LITERALS trả TAG_BOOL. Cả hai truthy = true.

typedef uint64_t Value;

// Constructors
static inline Value val_nil(void)          { return TAG_NIL; }
static inline Value val_bool(int b)        { return TAG_BOOL | (b ? 1 : 0); }
static inline Value val_int(int32_t i)     { return TAG_INT | (uint32_t)i; }
static inline Value val_float(double d)    { union { double d; uint64_t u; } v; v.d = d; return v.u; }
static inline Value val_mol(uint16_t m)    { return TAG_MOL | m; }
static inline Value val_string(OlStr* s)   { return TAG_STRING | (uint64_t)(uintptr_t)s; }
static inline Value val_array(OlArray* a)  { return TAG_ARRAY | (uint64_t)(uintptr_t)a; }

// Type checks
static inline int is_float(Value v)  { return (v & QNAN) != QNAN; }
static inline int is_nil(Value v)    { return v == TAG_NIL; }
static inline int is_int(Value v)    { return (v & TAG_MASK) == TAG_INT; }
static inline int is_mol(Value v)    { return (v & TAG_MASK) == TAG_MOL; }
static inline int is_string(Value v) { return (v & TAG_MASK) == TAG_STRING; }
static inline int is_array(Value v)  { return (v & TAG_MASK) == TAG_ARRAY; }

// Extractors
static inline double   as_float(Value v)  { union { uint64_t u; double d; } x; x.u = v; return x.d; }
static inline int32_t  as_int(Value v)     { return (int32_t)(v & 0xFFFFFFFF); }
static inline uint16_t as_mol(Value v)     { return (uint16_t)(v & 0xFFFF); }
static inline OlStr*   as_string(Value v)  { return (OlStr*)(uintptr_t)(v & 0x0000FFFFFFFFFFFF); }
static inline OlArray* as_array(Value v)   { return (OlArray*)(uintptr_t)(v & 0x0000FFFFFFFFFFFF); }
static inline OlDict*  as_dict(Value v)    { return (OlDict*)(uintptr_t)(v & 0x0000FFFFFFFFFFFF); }
static inline OlObject* as_object(Value v) { return (OlObject*)(uintptr_t)(v & 0x0000FFFFFFFFFFFF); }
static inline int is_dict(Value v)   { return (v & TAG_MASK) == TAG_DICT; }
static inline int is_object(Value v) { return (v & TAG_MASK) == TAG_OBJECT; }
// is_closure: check TAG_OBJECT + OBJ_TYPE == OBJ_CLOSURE
static inline int is_closure(Value v) {
    if (!is_object(v)) return 0;
    ObjHeader* h = (ObjHeader*)as_object(v);
    return OBJ_TYPE(h) == OBJ_CLOSURE;
}
static inline OlClosure* as_closure(Value v) { return (OlClosure*)as_object(v); }

// Number coercion (int/float/bool → double)
static inline double as_number(Value v) {
    if (is_float(v)) return as_float(v);
    if (is_int(v)) return (double)as_int(v);
    if (is_bool(v)) return as_bool(v) ? 1.0 : 0.0;
    return 0.0;
}

// Molecule dimension accessors (from src/mol.h — MOL §1)
#define MOL_S(m)  (((m) >> 12) & 0xF)
#define MOL_R(m)  (((m) >> 8) & 0xF)
#define MOL_V(m)  (((m) >> 5) & 0x7)
#define MOL_A(m)  (((m) >> 2) & 0x7)
#define MOL_T(m)  ((m) & 0x3)

#define MIN(a,b) ((a)<(b)?(a):(b))
#define MAX(a,b) ((a)>(b)?(a):(b))
#define CLAMP(x,lo,hi) ((x)<(lo)?(lo):((x)>(hi)?(hi):(x)))
```

**Ưu điểm:**
- Float operations = zero overhead (không cần unbox)
- Type check = 1 phép so sánh
- 48-bit pointer đủ cho x86_64 (canonical address space)

### 2.2 Object Header

Mọi heap object (string, array, dict, struct, closure, chain) có header:

```c
typedef struct {
    uint32_t tag;       // object type + GC bits
                        // bits 0-1: GC mark (WHITE=0, GRAY=1, BLACK=2, FORWARD=3)
                        // bits 2-7: object type
                        // bits 8-31: reserved (size class, etc.)
    uint32_t size;      // total size in bytes (including header)
} ObjHeader;

// Object types
enum ObjType {
    OBJ_STRING   = 1,
    OBJ_ARRAY    = 2,
    OBJ_DICT     = 3,
    OBJ_CLOSURE  = 4,
    OBJ_CHAIN    = 5,   // MolecularChain
    OBJ_STRUCT   = 6,
    OBJ_BYTES    = 7,
    OBJ_UPVALUE  = 8,
    OBJ_PROTO    = 9,   // function prototype
    OBJ_NODE     = 10,  // KnowTree node
    OBJ_EDGE     = 11,  // Silk edge
    OBJ_TASK     = 12,  // spawned task
    OBJ_CHANNEL  = 13,
};
```

### 2.3 Collection Types — Struct Layouts

Mỗi collection type có ObjHeader ở đầu + dữ liệu theo sau. GC dùng OBJ_TYPE để biết cách scan.

```c
/* ── OlArray ───────────────────────────────────────────────────────── */
/* Mảng động, chứa Value[]. GC phải scan từng element. */

typedef struct {
    ObjHeader header;       // tag=OBJ_ARRAY, size=sizeof(OlArray)+capacity*8
    uint32_t  length;       // số phần tử hiện tại
    uint32_t  capacity;     // số phần tử tối đa trước khi grow
    Value     items[];      // flexible array member (C99)
} OlArray;

// Tạo mảng: alloc trên heap, init header
OlArray* array_new(Heap* h, uint32_t capacity) {
    size_t sz = sizeof(OlArray) + capacity * sizeof(Value);
    OlArray* a = (OlArray*)heap_alloc(h, sz);
    if (!a) return NULL;
    OBJ_INIT(&a->header, OBJ_ARRAY, sz);
    a->length = 0;
    a->capacity = capacity;
    memset(a->items, 0, capacity * sizeof(Value));
    return a;
}

// Push: nếu length == capacity, phải alloc mảng mới (2x) rồi copy
// GC có thể chạy trong lúc alloc → root cũ phải được track
void array_push(Heap* h, OlArray** a, Value v) {
    if ((*a)->length >= (*a)->capacity) {
        uint32_t new_cap = (*a)->capacity * 2;
        if (new_cap < 8) new_cap = 8;
        OlArray* new_a = array_new(h, new_cap);
        memcpy(new_a->items, (*a)->items, (*a)->length * sizeof(Value));
        new_a->length = (*a)->length;
        *a = new_a;  // old array sẽ bị GC thu gom
    }
    (*a)->items[(*a)->length++] = v;
}

/* ── OlDict ────────────────────────────────────────────────────────── */
/* Open-addressing hash table: key=OlStr*, value=Value. */
/* Load factor max 0.75 → resize 2x. */

typedef struct {
    Value key;              // TAG_STRING hoặc TAG_NIL (empty slot)
    Value val;
} DictEntry;

typedef struct {
    ObjHeader  header;
    uint32_t   count;       // số entries đang dùng
    uint32_t   capacity;    // tổng slots (luôn power of 2)
    DictEntry  entries[];   // flexible array member
} OlDict;

OlDict* dict_new(Heap* h, uint32_t capacity) {
    if (capacity < 8) capacity = 8;
    size_t sz = sizeof(OlDict) + capacity * sizeof(DictEntry);
    OlDict* d = (OlDict*)heap_alloc(h, sz);
    if (!d) return NULL;
    OBJ_INIT(&d->header, OBJ_DICT, sz);
    d->count = 0;
    d->capacity = capacity;
    // Init all keys to nil (empty)
    for (uint32_t i = 0; i < capacity; i++) {
        d->entries[i].key = val_nil();
        d->entries[i].val = val_nil();
    }
    return d;
}

// Lookup: open-addressing with linear probing
// Fibonacci hash (Knuth TAOCP §6.4): floor(2³²×φ⁻¹) = 2654435769
// Using 2654435761 (closest prime, ≈same distribution, common in practice)
// Three-distance theorem: φ⁻¹ multiplicative hash = most uniform distribution
// Benchmark: 2× faster than integer modulo (Skarupke 2018)
Value dict_get(OlDict* d, OlStr* key) {
    uint32_t mask = d->capacity - 1;
    uint32_t idx = ((uint32_t)((uintptr_t)key) * 2654435761u) >> (32 - __builtin_ctz(d->capacity));
    for (uint32_t i = 0; i < d->capacity; i++) {
        uint32_t slot = (idx + i) & mask;
        if (is_nil(d->entries[slot].key)) return val_nil();  // not found
        if (as_string(d->entries[slot].key) == key)
            return d->entries[slot].val;
    }
    return val_nil();
}

// Set: insert or update. Caller must handle resize if count/capacity > 0.75
void dict_set(OlDict* d, OlStr* key, Value val) {
    uint32_t mask = d->capacity - 1;
    uint32_t idx = ((uint32_t)((uintptr_t)key) * 2654435761u) >> (32 - __builtin_ctz(d->capacity));
    for (uint32_t i = 0; i < d->capacity; i++) {
        uint32_t slot = (idx + i) & mask;
        if (is_nil(d->entries[slot].key)) {
            d->entries[slot].key = val_string(key);
            d->entries[slot].val = val;
            d->count++;
            return;
        }
        if (as_string(d->entries[slot].key) == key) {
            d->entries[slot].val = val;  // update existing
            return;
        }
    }
}

/* ── OlClosure ─────────────────────────────────────────────────────── */
/* Cảm hứng từ Julia jl_opaque_closure_t: captures packed vào array. */
/* Upvalue = captured variable từ outer scope. */

typedef struct {
    ObjHeader header;
    OlProto*  proto;        // function prototype (code + constants)
    uint16_t  upval_count;  // số upvalues captured
    Value     upvals[];     // captured values (flexible array)
} OlClosure;

OlClosure* closure_new(Heap* h, OlProto* proto, uint16_t upval_count) {
    size_t sz = sizeof(OlClosure) + upval_count * sizeof(Value);
    OlClosure* c = (OlClosure*)heap_alloc(h, sz);
    if (!c) return NULL;
    OBJ_INIT(&c->header, OBJ_CLOSURE, sz);
    c->proto = proto;
    c->upval_count = upval_count;
    memset(c->upvals, 0, upval_count * sizeof(Value));
    return c;
}

/* ── OlBytes ───────────────────────────────────────────────────────── */
/* Byte array cho binary I/O, crypto, protocols. Không chứa Value → GC không scan. */

typedef struct {
    ObjHeader header;
    uint32_t  length;
    uint8_t   data[];       // raw bytes
} OlBytes;
```

### 2.4 GC Object Scanning — Per-Type Field Traversal

GC Cheney cần scan mọi Value bên trong object đã copy sang tospace.
Cách tiếp cận: switch trên OBJ_TYPE, iterate qua từng Value field.
(Tham khảo: Julia `gc_mark_loop` dùng `jl_typetagof` để dispatch, scan tất cả pointer fields.)

```c
/* scan_object: gọi update_value cho mỗi Value bên trong object.
 * Chạy trong Cheney BFS: object đã nằm ở tospace, các Value bên trong
 * có thể vẫn trỏ vào fromspace → cần copy chúng sang tospace.
 */
static void scan_object(Heap* h, ObjHeader* obj, uint8_t** free_ptr) {
    switch (OBJ_TYPE(obj)) {

    case OBJ_ARRAY: {
        OlArray* arr = (OlArray*)obj;
        for (uint32_t i = 0; i < arr->length; i++) {
            update_value(h, &arr->items[i], free_ptr);
        }
        break;
    }

    case OBJ_DICT: {
        OlDict* dict = (OlDict*)obj;
        for (uint32_t i = 0; i < dict->capacity; i++) {
            if (!is_nil(dict->entries[i].key)) {
                update_value(h, &dict->entries[i].key, free_ptr);
                update_value(h, &dict->entries[i].val, free_ptr);
            }
        }
        break;
    }

    case OBJ_CLOSURE: {
        OlClosure* cl = (OlClosure*)obj;
        // proto là pointer bên ngoài heap (static/old gen) → không scan
        for (uint16_t i = 0; i < cl->upval_count; i++) {
            update_value(h, &cl->upvals[i], free_ptr);
        }
        break;
    }

    case OBJ_CHAIN: {
        // MolChain chứa uint16_t[] (mol values) → không phải heap pointer
        // Không cần scan
        break;
    }

    case OBJ_STRING:
    case OBJ_BYTES:
        // Raw data, không chứa Value → không scan
        break;

    case OBJ_STRUCT: {
        // Struct = fixed-layout fields, tất cả là Value
        // Layout: ObjHeader + uint16_t field_count + Value fields[]
        uint16_t fc = *(uint16_t*)((uint8_t*)obj + sizeof(ObjHeader));
        Value* fields = (Value*)((uint8_t*)obj + sizeof(ObjHeader) + 8);
        for (uint16_t i = 0; i < fc; i++) {
            update_value(h, &fields[i], free_ptr);
        }
        break;
    }

    default:
        // OBJ_NODE, OBJ_EDGE, OBJ_TASK, OBJ_CHANNEL
        // Sẽ implement khi có struct layout cho từng loại
        break;
    }
}
```

---

## 3. MEMORY MODEL

### 3.1 Architecture

Origin sử dụng **per-subsystem heaps** (lấy cảm hứng từ Erlang BEAM):

```
+------------------------------------------------------------------+
|                          ORIGIN VM                                |
|                                                                   |
|  +------------------+  +------------------+  +------------------+ |
|  |   MAIN HEAP      |  |  KNOWTREE HEAP   |  |   SILK HEAP      | |
|  |  (Olang objects)  |  |  (nodes, chains) |  |  (edges, weights)| |
|  |  Cheney semi-space|  |  Arena + sweep   |  |  Arena + sweep   | |
|  |  256KB young      |  |  grows monotonic |  |  grows monotonic | |
|  +------------------+  +------------------+  +------------------+ |
|                                                                   |
|  +------------------+  +------------------+                       |
|  |  STRING TABLE     |  |  LARGE OBJECTS   |                       |
|  |  (interned strs)  |  |  (> 2KB, malloc) |                       |
|  |  hash table       |  |  ref counted     |                       |
|  +------------------+  +------------------+                       |
+------------------------------------------------------------------+
```

### 3.2 Main Heap — Cheney Semi-Space GC

Cho Olang runtime objects (variables, arrays, dicts, closures, temp chains):

```c
#define YOUNG_SIZE (256 * 1024)    // 256KB per semispace
#define OLD_SIZE   (4 * 1024 * 1024) // 4MB old generation

typedef struct {
    uint8_t* fromspace;
    uint8_t* tospace;
    uint8_t* alloc_ptr;       // bump pointer in fromspace
    uint8_t* fromspace_end;
    
    // Old generation (mark-sweep, for long-lived objects)
    uint8_t* old_space;
    uint8_t* old_alloc;
    uint8_t* old_end;
    
    // Stats
    uint32_t young_collections;
    uint32_t tenured_count;     // objects promoted to old gen
} MainHeap;

// Allocation (hot path — chỉ bump pointer)
static inline void* heap_alloc(MainHeap* h, size_t size) {
    size = ALIGN8(size);
    if (h->alloc_ptr + size > h->fromspace_end) {
        heap_collect(h);  // trigger GC
        if (h->alloc_ptr + size > h->fromspace_end) {
            return NULL;  // out of memory
        }
    }
    void* ptr = h->alloc_ptr;
    h->alloc_ptr += size;
    return ptr;
}

// Collection — Cheney's algorithm
void heap_collect(MainHeap* h) {
    uint8_t* scan = h->tospace;
    uint8_t* free = h->tospace;
    
    // Copy roots (registers, stack, globals)
    for (int i = 0; i < root_count; i++) {
        roots[i] = copy_object(roots[i], &free, h);
    }
    
    // Scan copied objects (BFS)
    while (scan < free) {
        ObjHeader* obj = (ObjHeader*)scan;
        scan_object_fields(obj, &free, h);
        scan += obj->size;
    }
    
    // Swap spaces
    uint8_t* tmp = h->fromspace;
    h->fromspace = h->tospace;
    h->tospace = tmp;
    h->alloc_ptr = free;
    h->fromspace_end = h->fromspace + YOUNG_SIZE;
    h->young_collections++;
}

// Copy one object
static void* copy_object(Value* valp, uint8_t** free, MainHeap* h) {
    if (!is_heap_object(*valp)) return;
    
    ObjHeader* obj = as_object(*valp);
    
    // Already forwarded?
    if ((obj->tag & 3) == GC_FORWARD) {
        *valp = forwarding_address(obj);
        return;
    }
    
    // Tenure old objects
    if (obj->tag & TENURED_BIT) {
        // Copy to old gen instead
        void* new_addr = old_alloc(h, obj->size);
        memcpy(new_addr, obj, obj->size);
        obj->tag = GC_FORWARD;
        set_forwarding(obj, new_addr);
        *valp = make_value(new_addr);
        return;
    }
    
    // Copy to tospace
    void* new_addr = *free;
    memcpy(new_addr, obj, obj->size);
    *free += ALIGN8(obj->size);
    
    // Set forwarding pointer in old copy
    obj->tag = GC_FORWARD;
    set_forwarding(obj, new_addr);
    *valp = make_value(new_addr);
}
```

### 3.3 KnowTree Heap

Arena-based, grows monotonically. GC = mark-sweep (không copy — nodes có ID ổn định):

```c
typedef struct {
    uint8_t* base;
    uint8_t* alloc_ptr;
    uint8_t* end;
    size_t   capacity;        // starts 1MB, doubles when full
    
    // Free list (from swept dead nodes)
    uint32_t* free_list;
    uint32_t  free_count;
} KnowTreeHeap;
```

### 3.4 Silk Heap

Tương tự KnowTree heap. Edges có fixed size (24 bytes) → pool allocator:

```c
#define SILK_EDGE_SIZE 24

typedef struct {
    uint8_t* pool;
    uint32_t capacity;
    uint32_t count;
    uint32_t* free_slots;    // recycled slots
    uint32_t  free_count;
} SilkHeap;
```

### 3.5 String Interning

```c
typedef struct {
    ObjHeader header;
    uint32_t  hash;           // precomputed FNV-1a
    uint32_t  byte_len;
    uint8_t   flags;          // bit 0: ascii_only
    uint8_t   data[];         // UTF-8 bytes + NUL terminator
} OlStr;

typedef struct {
    OlStr**   buckets;
    uint32_t  size;           // power of 2
    uint32_t  count;
    uint32_t  mask;           // size - 1
} StringTable;

// Interning: O(1) amortized
OlStr* string_intern(StringTable* t, const uint8_t* bytes, uint32_t len) {
    uint32_t h = fnv1a(bytes, len);
    uint32_t idx = h & t->mask;
    
    while (t->buckets[idx]) {
        OlStr* s = t->buckets[idx];
        if (s->hash == h && s->byte_len == len && memcmp(s->data, bytes, len) == 0)
            return s;  // already exists
        idx = (idx + 1) & t->mask;  // linear probing
    }
    
    // Allocate new string
    OlStr* s = alloc_string(bytes, len, h);
    t->buckets[idx] = s;
    t->count++;
    if (t->count > t->size * 3 / 4) string_table_resize(t);
    return s;
}

// Equality = pointer comparison (vì tất cả interned)
static inline int string_eq(OlStr* a, OlStr* b) {
    return a == b;
}
```

---

## 4. VM INTERPRETER

### 4.1 VM State

```c
typedef struct {
    // Registers (256 per frame)
    Value     registers[256];
    
    // Call stack
    CallFrame frames[256];
    int       frame_count;
    
    // Current frame
    uint32_t* pc;              // program counter (pointer into bytecode)
    Value*    base;            // base register of current frame
    OlProto*  proto;           // current function prototype
    
    // Constant pool
    Value*    constants;
    uint32_t  const_count;
    
    // Global variables
    StringTable globals;       // name → Value
    
    // Heaps
    MainHeap     main_heap;
    KnowTreeHeap kt_heap;
    SilkHeap     silk_heap;
    StringTable  strings;
    
    // Brain subsystems
    KnowTree*  knowtree;
    SilkGraph*  silk;
    Pipeline*   pipeline;
    
    // I/O
    int        stdin_fd;
    int        stdout_fd;
    int        stderr_fd;
    
    // Error handling
    TryFrame   try_stack[64];
    int        try_depth;
    
    // Tasks (concurrency)
    Task*      tasks;
    int        task_count;
    
    // Persistence
    PersistStore* persist;
    
    // Self-modification
    uint32_t   code_generation;  // increments on any code modification
} VM;

/* ── OlProto — CANONICAL DEFINITION ─────────────────────────────────
 * Function prototype: bytecode + constant pool + metadata.
 * Compiled bởi codegen (OLANG §14.5). Loaded từ .olang binary (OLANG §8.3).
 * Nested functions: parent proto has sub_protos[] array.
 * Referenced by: OlClosure (§2.3), CallFrame (below), VM state.
 */
typedef struct OlProto {
    uint32_t* code;             // bytecode instructions
    uint32_t  code_len;         // number of instructions
    Value*    constants;        // constant pool (strings, numbers, sub-proto refs)
    uint32_t  const_count;
    uint8_t   param_count;      // number of formal parameters
    uint8_t   reg_count;        // max registers used (for stack frame sizing)
    uint8_t   upval_count;      // number of upvalues captured
    uint8_t   flags;            // bit 0: is_vararg, bit 1: has_self
    OlStr*    name;             // function name (interned, NULL for anonymous)
    struct OlProto** sub_protos; // nested function prototypes
    uint16_t  sub_proto_count;
    uint32_t  source_offset;    // offset in source file (for fn_source/debug)
    uint32_t  source_len;       // length in source
} OlProto;

typedef struct {
    OlProto*  proto;
    uint32_t* return_pc;
    Value*    base;
    int       result_reg;     // register để nhận return value
} CallFrame;

typedef struct {
    uint32_t* catch_pc;
    int       frame_depth;
    int       reg_base;
} TryFrame;
```

### 4.2 Main Loop (Direct Threading)

```c
// Computed goto dispatch (GCC/Clang extension, nhanh nhất)
#define DISPATCH() goto *dispatch_table[*pc & 0xFF]

void vm_execute(VM* vm) {
    static void* dispatch_table[256] = {
        [0x00] = &&op_nop,
        [0x01] = &&op_load_nil,
        [0x02] = &&op_load_true,
        // ... tất cả 64+ opcodes
        [0xFE] = &&op_halt,
    };
    
    register uint32_t* pc   = vm->pc;
    register Value*    base = vm->base;
    register Value*    K    = vm->constants;
    
    DISPATCH();
    
op_nop:
    pc++;
    DISPATCH();

op_load_nil:
    base[FIELD_A(*pc)] = val_nil();
    pc++;
    DISPATCH();

op_load_const: {
    uint8_t a = FIELD_A(*pc);
    uint16_t d = FIELD_D(*pc);
    base[a] = K[d];
    pc++;
    DISPATCH();
}

op_add: {
    uint8_t a = FIELD_A(*pc);
    uint8_t b = FIELD_B(*pc);
    uint8_t c = FIELD_C(*pc);
    Value vb = base[b], vc = base[c];
    
    if (is_float(vb) && is_float(vc)) {
        base[a] = val_float(as_float(vb) + as_float(vc));
    } else if (is_int(vb) && is_int(vc)) {
        base[a] = val_int(as_int(vb) + as_int(vc));
    } else if (is_string(vb) || is_string(vc)) {
        base[a] = string_concat(vm, vb, vc);
    } else {
        vm_error(vm, "cannot add these types");
    }
    pc++;
    DISPATCH();
}

op_call: {
    uint8_t a = FIELD_A(*pc);  // result register
    uint8_t b = FIELD_B(*pc);  // function register
    uint8_t c = FIELD_C(*pc);  // arg count
    
    Value fn = base[b];
    
    if (is_closure(fn)) {
        OlClosure* cl = as_closure(fn);
        
        // Push call frame
        vm->frames[vm->frame_count++] = (CallFrame){
            .proto = vm->proto,
            .return_pc = pc + 1,
            .base = base,
            .result_reg = a,
        };
        
        // Setup new frame
        vm->proto = cl->proto;
        base = base + b + 1;  // args start after function register
        vm->base = base;
        pc = cl->proto->code;
        K = cl->proto->constants;
        
        DISPATCH();
    }
    // ... builtin call, etc.
}

op_ret: {
    uint8_t a = FIELD_A(*pc);
    Value result = base[a];
    
    // Pop call frame
    CallFrame* frame = &vm->frames[--vm->frame_count];
    base = frame->base;
    pc = frame->return_pc;
    vm->proto = frame->proto;
    vm->base = base;
    K = frame->proto->constants;
    
    base[frame->result_reg] = result;
    DISPATCH();
}

/* ── OP_CALL (0x33) ────────────────────────────────────────────────
 * Format: ABC — R[A..A+C-1] = R[B](R[B+1..B+C])
 * A = result register
 * B = function register (OlClosure* hoặc builtin index)
 * C = argument count (không tính function)
 *
 * Call frame layout (trên register window):
 *
 *   Caller:  [ ... | fn | arg0 | arg1 | arg2 | ... ]
 *                    ^B   ^B+1   ^B+2   ^B+3
 *
 *   Callee:  [ arg0 | arg1 | arg2 | local0 | local1 | ... ]
 *              ^base[0]  ^base[1]  ^base[2]
 *
 * Callee's base = caller's base + B + 1 (skip function slot)
 * Giống Lua 5.x register window approach.
 *
 * Tham khảo: Julia interpreter dùng C-stack allocated frames (interpreter.c:17-27).
 * Origin dùng pre-allocated frame array (256 max) để tránh malloc/free mỗi call.
 * ────────────────────────────────────────────────────────────────── */
op_call: {
    uint8_t a = FIELD_A(*pc);  // result register
    uint8_t b = FIELD_B(*pc);  // function register
    uint8_t c = FIELD_C(*pc);  // arg count

    Value fn = base[b];

    if (is_object(fn)) {
        OlClosure* cl = (OlClosure*)as_object(fn);

        // Check frame overflow
        if (vm->frame_count >= VM_MAX_FRAMES) {
            vm_error(vm, "stack overflow: > 256 frames");
            goto op_halt;
        }

        // Check arg count matches
        if (c != cl->proto->param_count) {
            vm_error(vm, "wrong arg count: expected %d, got %d",
                     cl->proto->param_count, c);
            goto op_halt;
        }

        // Push call frame (save caller state)
        vm->frames[vm->frame_count++] = (CallFrame){
            .proto     = vm->proto,
            .return_pc = pc + 1,     // resume after CALL
            .base      = base,
            .result_reg = a,
        };

        // Setup callee frame
        // Callee's base starts at caller's base[b+1] (args already there)
        Value* callee_base = base + b + 1;

        // Load upvalues into callee's high registers
        // Convention: upval[i] → callee register[param_count + local_count + i]
        // Compiler emits OP_LOAD_UPVAL trong callee body để access chúng
        // Nhưng đơn giản hơn: store upvals vào closure, callee dùng OP_LOAD_UPVAL

        vm->proto = cl->proto;
        base = callee_base;
        vm->base = base;
        pc = cl->proto->code;
        K = cl->proto->constants;

        DISPATCH();
    }

    // Builtin function call (dùng OP_BUILTIN thay thế)
    vm_error(vm, "attempt to call non-function");
    goto op_halt;
}

/* ── OP_CLOSURE (0x36) ─────────────────────────────────────────────
 * Format: AD — R[A] = closure(Proto[D])
 *
 * Tạo OlClosure object trên heap.
 * Proto[D] = index vào bảng function prototypes.
 * Upvalues được capture bởi chuỗi OP_CLOSURE_CAP ngay sau OP_CLOSURE.
 *
 * Sequence:
 *   OP_CLOSURE  R[3], Proto[2]      // tạo closure, proto index = 2
 *   OP_CLOSURE_CAP R[3], R[5]       // capture R[5] vào upval[0]
 *   OP_CLOSURE_CAP R[3], R[7]       // capture R[7] vào upval[1]
 *
 * Tham khảo: Julia packs captures vào tuple (opaque_closure.c:140).
 * Origin packs captures vào OlClosure.upvals[] (§2.3).
 * ────────────────────────────────────────────────────────────────── */
op_closure: {
    uint8_t a = FIELD_A(*pc);
    uint16_t d = FIELD_D(*pc);

    // Proto[d] = pointer tới OlProto (stored in constant pool hoặc proto table)
    OlProto* proto = (OlProto*)(uintptr_t)as_int(K[d]);

    // Count upcoming OP_CLOSURE_CAP instructions
    uint16_t upval_count = 0;
    uint32_t* scan = pc + 1;
    while (FIELD_OP(*scan) == OP_CLOSURE_CAP) {
        upval_count++;
        scan++;
    }

    // Allocate closure on heap
    OlClosure* cl = closure_new(&vm->main_heap, proto, upval_count);
    if (!cl) { vm_error(vm, "out of memory"); goto op_halt; }

    // Fill upvalues from following OP_CLOSURE_CAP instructions
    for (uint16_t i = 0; i < upval_count; i++) {
        pc++;
        uint8_t src_reg = FIELD_B(*pc);
        cl->upvals[i] = base[src_reg];
    }

    base[a] = val_object((OlObject*)cl);
    pc++;
    DISPATCH();
}

#define OP_CLOSURE_CAP 0x3B  // format: AB — closure[A].upvals[next] = R[B]

/* ── OP_LOAD_UPVAL (0x09) ──────────────────────────────────────────
 * Format: AB — R[A] = current_closure.upvals[B]
 *
 * Callee truy cập captured variables từ closure object.
 * current_closure = con trỏ lưu trong VM state (set khi CALL closure).
 * ────────────────────────────────────────────────────────────────── */
op_load_upval: {
    uint8_t a = FIELD_A(*pc);
    uint8_t b = FIELD_B(*pc);
    // vm->current_closure được set trong op_call khi gọi closure
    OlClosure* cl = vm->current_closure;
    if (cl && b < cl->upval_count) {
        base[a] = cl->upvals[b];
    } else {
        base[a] = val_nil();
    }
    pc++;
    DISPATCH();
}

/* ── OP_STORE_UPVAL (0x0A) ─────────────────────────────────────────
 * Format: AB — current_closure.upvals[B] = R[A]
 *
 * Mutate captured variable. Thay đổi chỉ ảnh hưởng closure này.
 * (Giống Julia: captures là snapshot, không phải reference.)
 * ────────────────────────────────────────────────────────────────── */
op_store_upval: {
    uint8_t a = FIELD_A(*pc);
    uint8_t b = FIELD_B(*pc);
    OlClosure* cl = vm->current_closure;
    if (cl && b < cl->upval_count) {
        cl->upvals[b] = base[a];
    }
    pc++;
    DISPATCH();
}

/* ── OP_TRY_BEGIN (0x38) ───────────────────────────────────────────
 * Format: AD — push try frame, catch at PC + signed(D)
 *
 * Tham khảo: Julia dùng setjmp/longjmp (julia.h:2322).
 * Origin dùng try stack + frame depth tracking (đơn giản hơn,
 * không cần longjmp vì VM interpreter control flow).
 * ────────────────────────────────────────────────────────────────── */
op_try_begin: {
    if (vm->try_depth >= VM_MAX_TRY) {
        vm_error(vm, "too many nested try blocks");
        goto op_halt;
    }
    int16_t offset = FIELD_SD(*pc);
    vm->try_stack[vm->try_depth++] = (TryFrame){
        .catch_pc    = pc + offset,
        .frame_depth = vm->frame_count,
        .reg_base    = (int)(base - vm->registers),
    };
    pc++;
    DISPATCH();
}

/* ── OP_CATCH_END (0x39) ───────────────────────────────────────────
 * Format: — (no operands)
 * Pop try frame. Normal exit from try block.
 * ────────────────────────────────────────────────────────────────── */
op_catch_end: {
    if (vm->try_depth > 0) vm->try_depth--;
    pc++;
    DISPATCH();
}

/* ── OP_THROW (0x3A) ───────────────────────────────────────────────
 * Format: A — throw R[A] as error
 *
 * Unwind call stack until matching try frame.
 * Restore frame_count, base, pc to catch block.
 * Error value placed in R[A] of catch frame.
 * ────────────────────────────────────────────────────────────────── */
op_throw: {
    Value err = base[FIELD_A(*pc)];

    if (vm->try_depth == 0) {
        // Uncaught exception
        vm_emit(vm, err);
        goto op_halt;
    }

    // Pop to matching try frame
    TryFrame* tf = &vm->try_stack[--vm->try_depth];

    // Unwind call stack
    vm->frame_count = tf->frame_depth;
    if (vm->frame_count > 0) {
        CallFrame* f = &vm->frames[vm->frame_count - 1];
        vm->proto = f->proto;
        K = f->proto->constants;
    }
    base = vm->registers + tf->reg_base;
    vm->base = base;
    pc = tf->catch_pc;

    // Error value in R[0] of catch frame
    base[0] = err;
    DISPATCH();
}

/* ── Collection opcodes (0x40-0x47) ────────────────────────────────── */

op_new_array: {
    uint8_t a = FIELD_A(*pc);
    uint16_t cap = FIELD_D(*pc);
    if (cap == 0) cap = 8;
    OlArray* arr = array_new(&vm->main_heap, cap);
    base[a] = val_array(arr);
    pc++;
    DISPATCH();
}

op_array_push: {
    uint8_t a = FIELD_A(*pc);  // array register
    uint8_t b = FIELD_B(*pc);  // value register
    OlArray* arr = as_array(base[a]);
    array_push(&vm->main_heap, &arr, base[b]);
    base[a] = val_array(arr);  // arr pointer may have changed
    pc++;
    DISPATCH();
}

op_array_get: {
    uint8_t a = FIELD_A(*pc);  // result
    uint8_t b = FIELD_B(*pc);  // array
    uint8_t c = FIELD_C(*pc);  // index
    OlArray* arr = as_array(base[b]);
    int32_t idx = (int32_t)as_number(base[c]);
    if (idx >= 0 && (uint32_t)idx < arr->length) {
        base[a] = arr->items[idx];
    } else {
        base[a] = val_nil();  // out of bounds → nil (không crash)
    }
    pc++;
    DISPATCH();
}

op_array_set: {
    uint8_t a = FIELD_A(*pc);  // array
    uint8_t b = FIELD_B(*pc);  // index
    uint8_t c = FIELD_C(*pc);  // value
    OlArray* arr = as_array(base[a]);
    int32_t idx = (int32_t)as_number(base[b]);
    if (idx >= 0 && (uint32_t)idx < arr->length) {
        arr->items[idx] = base[c];
    }
    pc++;
    DISPATCH();
}

op_array_len: {
    uint8_t a = FIELD_A(*pc);
    uint8_t b = FIELD_B(*pc);
    OlArray* arr = as_array(base[b]);
    base[a] = val_int((int32_t)arr->length);
    pc++;
    DISPATCH();
}

op_new_dict: {
    uint8_t a = FIELD_A(*pc);
    uint16_t cap = FIELD_D(*pc);
    if (cap == 0) cap = 8;
    OlDict* d = dict_new(&vm->main_heap, cap);
    base[a] = val_dict(d);
    pc++;
    DISPATCH();
}

op_dict_get: {
    uint8_t a = FIELD_A(*pc);  // result
    uint8_t b = FIELD_B(*pc);  // dict
    uint8_t c = FIELD_C(*pc);  // key (string)
    OlDict* d = as_dict(base[b]);
    OlStr* key = as_string(base[c]);
    base[a] = dict_get(d, key);
    pc++;
    DISPATCH();
}

op_dict_set: {
    uint8_t a = FIELD_A(*pc);  // dict
    uint8_t b = FIELD_B(*pc);  // key (string)
    uint8_t c = FIELD_C(*pc);  // value
    OlDict* d = as_dict(base[a]);
    OlStr* key = as_string(base[b]);
    // Resize check: load factor > 0.75
    if (d->count * 4 >= d->capacity * 3) {
        OlDict* new_d = dict_new(&vm->main_heap, d->capacity * 2);
        // Rehash all entries
        for (uint32_t i = 0; i < d->capacity; i++) {
            if (!is_nil(d->entries[i].key)) {
                dict_set(new_d, as_string(d->entries[i].key), d->entries[i].val);
            }
        }
        d = new_d;
        base[a] = val_dict(d);
    }
    dict_set(d, key, base[c]);
    pc++;
    DISPATCH();
}

op_emit: {
    uint8_t a = FIELD_A(*pc);
    Value v = base[a];
    vm_emit(vm, v);  // output: UTF-8 string to stdout
    pc++;
    DISPATCH();
}

op_halt:
    vm->pc = pc;
    return;
}
```

### 4.3 Instruction Decode Macros

```c
#define FIELD_OP(inst)  ((inst) & 0xFF)
#define FIELD_A(inst)   (((inst) >> 8) & 0xFF)
#define FIELD_B(inst)   (((inst) >> 24) & 0xFF)
#define FIELD_C(inst)   (((inst) >> 16) & 0xFF)
#define FIELD_D(inst)   (((inst) >> 16) & 0xFFFF)
#define FIELD_SD(inst)  ((int16_t)(((inst) >> 16) & 0xFFFF))  // signed D
```

---

## 5. BRAIN SUBSYSTEMS

### 5.1 KnowTree

**Cấu trúc** (từ Rust gốc, đã simplify):

```c
// Node = 32 bytes (fixed size cho pool allocator)
typedef struct {
    uint16_t p_weight;        // P_weight encoding [S:4][R:4][V:3][A:3][T:2]
    uint16_t fire_count;      // số lần được activate
    uint32_t chain_offset;    // offset vào chain storage
    uint16_t chain_len;       // số molecules trong chain
    uint8_t  maturity;        // 0=Formula, 1=Evaluating, 2=Mature
    uint8_t  layer;           // 0=L0(permanent), 1-3=higher layers
    uint32_t first_edge;      // index vào Silk edge array
    uint16_t edge_count;
    uint16_t flags;           // bit 0: QR promoted, bit 1: deleted
    int64_t  last_fire;       // timestamp of last activation (ms since epoch)
                              // also serves as created_at (= first fire)
                              // ONA pattern: usefulness = fire_count/(recency+1)
} KTNode;

// Index: 256 hash buckets (P_weight >> 8)
typedef struct {
    KTNode*   nodes;
    uint32_t  node_count;
    uint32_t  node_capacity;
    
    // Bucket index for fast lookup
    uint32_t  bucket_heads[256];  // head node index per bucket
    uint32_t* bucket_next;        // next pointer per node (linked list)
    
    // Chain storage (pool of u16 P_weights)
    uint16_t* chains;
    uint32_t  chain_alloc;
    uint32_t  chain_capacity;
    
    // Stats
    uint32_t  total_fires;
    uint32_t  qr_count;
} KnowTree;
```

**Thuật toán chính:**

```c
// Store: O(1) amortized
uint32_t kt_store(KnowTree* kt, uint16_t* chain, uint16_t len, uint8_t emotion) {
    uint16_t pw = chain[0];  // representative = first molecule
    uint8_t bucket = pw >> 8;
    
    // Check collision — nếu đã tồn tại, tăng fire_count
    uint32_t idx = kt->bucket_heads[bucket];
    while (idx != UINT32_MAX) {
        KTNode* n = &kt->nodes[idx];
        if (n->p_weight == pw && chain_match(kt, n, chain, len)) {
            n->fire_count++;
            advance_maturity(n);
            return idx;
        }
        idx = kt->bucket_next[idx];
    }
    
    // New node
    uint32_t id = kt_alloc_node(kt);
    KTNode* node = &kt->nodes[id];
    node->p_weight = pw;
    node->fire_count = 1;
    node->chain_offset = kt_store_chain(kt, chain, len);
    node->chain_len = len;
    node->maturity = 0;
    node->layer = 1;
    node->first_edge = 0;
    node->edge_count = 0;
    node->flags = 0;
    node->created_at = time_now_ms();
    
    // Insert into bucket
    kt->bucket_next[id] = kt->bucket_heads[bucket];
    kt->bucket_heads[bucket] = id;
    kt->node_count++;
    
    return id;
}

// Nearest: O(bucket_size) ≈ O(n/256)
void kt_nearest(KnowTree* kt, uint16_t target, int k, uint32_t* results, int* count) {
    // Phase 1: same bucket
    uint8_t bucket = target >> 8;
    // Phase 2: adjacent buckets (±1 in each dimension)
    // Phase 3: sort by weighted Manhattan distance
    // Return top-k
    
    int count = 0;
    int max_dist = INT_MAX;
    
    for (int b = 0; b < 256; b++) {
        // Skip buckets too far away
        int bucket_dist = bucket_manhattan(bucket, b);
        if (bucket_dist > max_dist && count >= k) continue;
        
        uint32_t idx = kt->bucket_heads[b];
        while (idx != UINT32_MAX) {
            int dist = mol_dist(target, kt->nodes[idx].p_weight);
            if (count < k || dist < max_dist) {
                insert_sorted(results, &count, k, idx, dist);
                if (count >= k) max_dist = results[k-1].dist;
            }
            idx = kt->bucket_next[idx];
        }
    }
}

// Weighted Manhattan Distance (từ Rust gốc)
static inline int mol_dist(uint16_t a, uint16_t b) {
    int ds = abs(((a >> 12) & 0xF) - ((b >> 12) & 0xF));  // S: weight 1
    int dr = abs(((a >> 8) & 0xF) - ((b >> 8) & 0xF));    // R: weight 1
    int dv = abs(((a >> 5) & 0x7) - ((b >> 5) & 0x7));    // V: weight 2
    int da = abs(((a >> 2) & 0x7) - ((b >> 2) & 0x7));    // A: weight 2
    int dt = abs((a & 0x3) - (b & 0x3));                   // T: weight 4
    return ds + dr + 2*dv + 2*da + 4*dt;
}
```

### 5.2 Silk (Knowledge Graph)

**3 tầng** (từ Rust gốc):

```c
// Edge = 32 bytes (expanded from 24 for reward tracking — Sora #5)
typedef struct {
    uint32_t from;          // node index
    uint32_t to;            // node index
    uint16_t weight;        // 0-65535 (mapped to 0.0-1.0)
    uint16_t fire_count;
    uint8_t  edge_kind;     // 22 types: Member, Subset, Equiv, ...
    uint8_t  layer;         // 0=implicit, 1=learned, 2=structural
    uint8_t  flags;         // bit 0: bidirectional
    uint8_t  _pad;
    int64_t  last_fire;     // timestamp
    float    reward_sum;    // accumulated reward (for running average)
    uint16_t reward_count;  // number of feedback events
    uint16_t _pad2;
} SilkEdge;

typedef struct {
    SilkEdge* edges;
    uint32_t  edge_count;
    uint32_t  edge_capacity;
    
    // Index: from_node → first_edge
    uint32_t* node_first_edge;
    uint32_t* edge_next;        // linked list per node
    uint32_t  node_capacity;
} SilkGraph;
```

**Thuật toán Hebbian (từ Rust gốc):**

```c
// Co-activate: khi 2 nodes xuất hiện cùng context
void silk_co_activate(SilkGraph* sg, uint32_t a, uint32_t b, float reward) {
    SilkEdge* edge = silk_find_edge(sg, a, b);
    
    if (edge) {
        // Strengthen: Δw = reward × (1 - w/65535) × 236
        uint16_t w = edge->weight;
        int delta = (int)(reward * (1.0f - w / 65535.0f) * 236);
        edge->weight = (uint16_t)MIN(65535, w + delta);
        edge->fire_count++;
        edge->last_fire = time_now_ms();
    } else {
        // New edge
        silk_add_edge(sg, a, b, (uint16_t)(reward * 236), 1);
    }
}

// Decay: stretched exponential w(t) = w₀ × exp(-(t/τ)^β)
//   β = φ⁻¹ = 0.618, τ = 78.37h (calibrated: w(24h) = 0.618 EXACT — xem MOL §26)
//   Xem MOL §26 cho full analysis + so sánh ACT-R
//
// Fixed-point approximation cho integer weights (u16):
//   Full 24h cycle: w × 618 / 1000 (giống pure exponential)
//   Partial period: dùng powf (chính xác hơn linear interpolation)
//   618/1000 (KHÔNG phải 618/1024 — 618/1000 chính xác hơn: 0.618 vs 0.604)

void silk_decay(SilkGraph* sg, int64_t elapsed_ms) {
    int64_t hours_24 = 24LL * 60 * 60 * 1000;

    for (uint32_t i = 0; i < sg->edge_count; i++) {
        SilkEdge* e = &sg->edges[i];
        if (e->weight == 0) continue;

        float hours = (float)elapsed_ms / (60.0f * 60.0f * 1000.0f);

        // Stretched exponential: exp(-(hours/78.37)^0.618)
        float x = hours / 78.37f;
        float factor = expf(-powf(x, 0.618f));
        e->weight = (uint16_t)((float)e->weight * factor);

        // Prune dead edges
        if (e->weight == 0) {
            silk_remove_edge(sg, i);
            i--;
        }
    }
}

// Walk: BFS N hops from start node
void silk_walk(SilkGraph* sg, uint32_t start, int hops, 
               uint32_t* results, int* count, int max_results) {
    uint32_t queue[1024];
    uint8_t  visited[65536 / 8];  // bitset
    int head = 0, tail = 0;
    int depth = 0;
    
    memset(visited, 0, sizeof(visited));
    queue[tail++] = start;
    BIT_SET(visited, start);
    
    *count = 0;
    
    while (head < tail && depth < hops) {
        int level_size = tail - head;
        for (int i = 0; i < level_size && *count < max_results; i++) {
            uint32_t node = queue[head++];
            results[(*count)++] = node;
            
            // Enqueue neighbors
            uint32_t edge_idx = sg->node_first_edge[node];
            while (edge_idx != UINT32_MAX) {
                SilkEdge* e = &sg->edges[edge_idx];
                uint32_t neighbor = (e->from == node) ? e->to : e->from;
                if (!BIT_GET(visited, neighbor)) {
                    BIT_SET(visited, neighbor);
                    if (tail < 1024) queue[tail++] = neighbor;
                }
                edge_idx = sg->edge_next[edge_idx];
            }
        }
        depth++;
    }
}

// Implicit Silk (Layer 0): computed from 5D position, 0-cost
// 37 buckets: 8S + 8R + 8V + 8A + 5T
// Từ Rust gốc: silk/index.rs
uint8_t silk_implicit_type(uint16_t mol_a, uint16_t mol_b) {
    int ds = abs(MOL_S(mol_a) - MOL_S(mol_b));
    int dr = abs(MOL_R(mol_a) - MOL_R(mol_b));
    int dv = abs(MOL_V(mol_a) - MOL_V(mol_b));
    int da = abs(MOL_A(mol_a) - MOL_A(mol_b));
    int dt = abs(MOL_T(mol_a) - MOL_T(mol_b));
    
    // Dimension dominant → edge type
    if (ds >= dr && ds >= dv && ds >= da && ds >= dt) return SILK_SHAPE_DIFF + ds;
    if (dr >= ds && dr >= dv && dr >= da && dr >= dt) return SILK_REL_DIFF + dr;
    if (dv >= ds && dv >= dr && dv >= da && dv >= dt) return SILK_VAL_DIFF + dv;
    if (da >= ds && da >= dr && da >= dv && da >= dt) return SILK_ARO_DIFF + da;
    return SILK_TIME_DIFF + dt;
}
```

### 5.2.1 Silk + KnowTree Utility Functions

```c
/* Tất cả hàm được gọi trong specs nhưng chưa có body.
 * Bổ sung ở đây để specs tự-nhất-quán. */

// ── KnowTree ──

// Tìm node chính xác theo chain (exact match, không nearest)
uint32_t kt_find(KnowTree* kt, uint16_t* chain, uint16_t len) {
    if (len == 0) return UINT32_MAX;
    uint8_t bucket = chain[0] >> 8;
    uint32_t idx = kt->bucket_heads[bucket];
    while (idx != UINT32_MAX) {
        KTNode* n = &kt->nodes[idx];
        if (n->p_weight == chain[0] && n->chain_len == len) {
            // Compare full chain
            uint16_t* stored = kt->chains + n->chain_offset;
            int match = 1;
            for (int i = 0; i < len; i++) {
                if (stored[i] != chain[i]) { match = 0; break; }
            }
            if (match) return idx;
        }
        idx = kt->bucket_next[idx];
    }
    return UINT32_MAX;  // not found
}

// Advance maturity: Formula→Evaluating→Mature (irreversible)
void advance_maturity(KTNode* node) {
    if (node->maturity == 0 && node->fire_count > 0)
        node->maturity = 1;  // Formula → Evaluating
    // Evaluating → Mature: handled by check_promotion_adaptive()
}

// Node usefulness score — for eviction (ONA/NARS-inspired)
// Nguồn: OpenNARS-for-Applications Usage.c (github.com/opennars/OpenNARS-for-Applications)
// Formula: sigmoid(fire_count / (recency + 1))
// Giống ACT-R's ln(n/L) nhưng đơn giản hơn, không cần log
// Tính chất: new items get grace period, old unused decay, frequent survive
double kt_node_usefulness(KTNode* node, int64_t now_ms) {
    double recency_h = (double)(now_ms - node->last_fire) / 3600000.0;
    double raw = (double)node->fire_count / (recency_h + 1.0);
    return raw / (raw + 1.0);  // normalized [0, 1]
}

// Evict least useful node when KnowTree full
// ONA pattern: PriorityQueue ordered by usefulness, pop min
void kt_evict_if_full(KnowTree* kt, int64_t now_ms) {
    if (kt->node_count < kt->node_capacity) return;

    // Find least useful non-QR node
    uint32_t worst = UINT32_MAX;
    double worst_score = 1e9;
    for (uint32_t i = 0; i < kt->node_count; i++) {
        if (kt->nodes[i].flags & 1) continue;  // skip QR (permanent)
        double score = kt_node_usefulness(&kt->nodes[i], now_ms);
        if (score < worst_score) {
            worst_score = score;
            worst = i;
        }
    }
    if (worst != UINT32_MAX) {
        kt->nodes[worst].flags |= 2;  // mark deleted
        // TODO: add to free list for reuse
    }
}

// §5.1.1 KnowTree RESIZE — doubling realloc when full
// Eviction first (trên). Nếu eviction không đủ → grow.
// Capacity: 1M initial → 2M → 4M → ... (32MB → 64MB → 128MB)
// Sau resize: bucket_next[] cũng phải realloc.
// VP-Tree index: rebuild sau resize (pointers thay đổi).
int kt_grow(KnowTree* kt) {
    uint32_t new_cap = kt->node_capacity * 2;
    KTNode* new_nodes = realloc(kt->nodes, new_cap * sizeof(KTNode));
    if (!new_nodes) return -1;  // OOM — eviction is only option
    kt->nodes = new_nodes;

    uint32_t* new_next = realloc(kt->bucket_next, new_cap * sizeof(uint32_t));
    if (!new_next) return -1;
    kt->bucket_next = new_next;

    // Zero new slots
    memset(kt->nodes + kt->node_capacity, 0, (new_cap - kt->node_capacity) * sizeof(KTNode));
    memset(kt->bucket_next + kt->node_capacity, 0xFF, (new_cap - kt->node_capacity) * sizeof(uint32_t));

    kt->node_capacity = new_cap;
    return 0;
}

// Cũng grow chain storage khi đầy:
int kt_grow_chains(KnowTree* kt) {
    uint32_t new_cap = kt->chain_capacity * 2;
    uint16_t* new_chains = realloc(kt->chains, new_cap * sizeof(uint16_t));
    if (!new_chains) return -1;
    kt->chains = new_chains;
    kt->chain_capacity = new_cap;
    return 0;
}

// Gọi trong kt_store() khi cần:
// if (kt->node_count >= kt->node_capacity) {
//     kt_evict_if_full(kt, time_ms());
//     if (kt->node_count >= kt->node_capacity) {
//         if (kt_grow(kt) != 0) return UINT32_MAX;  // OOM
//     }
// }

// Push to STM (ring buffer, evict lowest importance if full)
void stm_push(STM* stm, uint16_t* chain, uint16_t len, uint16_t emotion) {
    int slot;
    if (stm->count < 32) {
        slot = stm->count++;
    } else {
        // Evict: find lowest importance
        float min_score = 1e9;
        slot = 0;
        for (int i = 0; i < 32; i++) {
            float score = stm_eviction_score(&stm->entries[i], time_ms());
            if (score < min_score) { min_score = score; slot = i; }
        }
    }
    memcpy(stm->entries[slot].chain, chain, MIN(len, 64) * sizeof(uint16_t));
    stm->entries[slot].chain_len = MIN(len, 64);
    stm->entries[slot].emotion = emotion;
    stm->entries[slot].fire_count = 1;
    stm->entries[slot].timestamp = time_ms();
}

// ── Silk ──

// Find edge between 2 nodes
SilkEdge* silk_find_edge(SilkGraph* sg, uint32_t from, uint32_t to) {
    if (from >= sg->node_capacity) return NULL;
    uint32_t idx = sg->node_first_edge[from];
    while (idx != UINT32_MAX) {
        if (sg->edges[idx].to == to) return &sg->edges[idx];
        idx = sg->edge_next[idx];
    }
    return NULL;
}

// Get weight (0-65535) between 2 nodes, 0 if no edge
uint16_t silk_get_weight(SilkGraph* sg, uint32_t from, uint32_t to) {
    SilkEdge* e = silk_find_edge(sg, from, to);
    return e ? e->weight : 0;
}

// Max weight from any edge connected to node
uint16_t silk_max_weight(SilkGraph* sg, uint32_t node) {
    if (node >= sg->node_capacity) return 0;
    uint16_t max_w = 0;
    uint32_t idx = sg->node_first_edge[node];
    while (idx != UINT32_MAX) {
        if (sg->edges[idx].weight > max_w) max_w = sg->edges[idx].weight;
        idx = sg->edge_next[idx];
    }
    return max_w;
}

// Add new edge
void silk_add_edge(SilkGraph* sg, uint32_t from, uint32_t to,
                    uint16_t weight, uint8_t kind) {
    if (sg->edge_count >= sg->edge_capacity) {
        sg->edge_capacity *= 2;
        sg->edges = realloc(sg->edges, sg->edge_capacity * sizeof(SilkEdge));
        sg->edge_next = realloc(sg->edge_next, sg->edge_capacity * sizeof(uint32_t));
    }
    uint32_t idx = sg->edge_count++;
    sg->edges[idx] = (SilkEdge){
        .from = from, .to = to, .weight = weight,
        .fire_count = 1, .edge_kind = kind, .layer = 1,
        .last_fire = time_ms()
    };
    // Insert into linked list
    sg->edge_next[idx] = sg->node_first_edge[from];
    sg->node_first_edge[from] = idx;
}

// ── MolChain utilities ──

typedef struct { uint16_t mols[64]; uint16_t len; } MolChain;

MolChain molchain_empty(void) {
    MolChain mc = {0};
    return mc;
}

float molchain_similarity(MolChain* a, MolChain* b) {
    if (a->len == 0 || b->len == 0) return 0;
    // Average pairwise mol_dist, normalized
    float total_dist = 0;
    int pairs = 0;
    int min_len = MIN(a->len, b->len);
    for (int i = 0; i < min_len; i++) {
        total_dist += mol_dist(a->mols[i], b->mols[i]) / 70.0f;
        pairs++;
    }
    if (pairs == 0) return 0;
    return 1.0f - (total_dist / pairs);  // 1.0 = identical, 0.0 = completely different
}

// ── VM utilities ──

int64_t time_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000LL + ts.tv_nsec / 1000000LL;
}

void vm_error(VM* vm, const char* fmt, ...) {
    char buf[256];
    va_list args;
    va_start(args, fmt);
    vsnprintf(buf, sizeof(buf), fmt, args);
    va_end(args);
    write(STDERR_FILENO, "Error: ", 7);
    write(STDERR_FILENO, buf, strlen(buf));
    write(STDERR_FILENO, "\n", 1);
    vm->had_error = 1;
}

Value vm_file_read(VM* vm, const char* path) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return val_nil();
    struct stat st;
    fstat(fd, &st);
    uint8_t* buf = malloc(st.st_size);
    read(fd, buf, st.st_size);
    close(fd);
    OlStr* s = strtab_intern(&vm->strings, buf, st.st_size);
    free(buf);
    return val_string(s);
}

void vm_file_write(VM* vm, const char* path, OlStr* content) {
    int fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd >= 0) { write(fd, content->data, content->byte_len); close(fd); }
}

void copy_file(const char* src, const char* dst) {
    int in = open(src, O_RDONLY);
    if (in < 0) return;
    int out = open(dst, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (out < 0) { close(in); return; }
    char buf[4096];
    ssize_t n;
    while ((n = read(in, buf, sizeof(buf))) > 0) write(out, buf, n);
    close(in); close(out);
}

OlStr* string_replace(VM* vm, OlStr* haystack, OlStr* needle, OlStr* replacement) {
    // Simple single-occurrence replace
    char* pos = strstr((char*)haystack->data, (char*)needle->data);
    if (!pos) return haystack;
    size_t before = pos - (char*)haystack->data;
    size_t after = haystack->byte_len - before - needle->byte_len;
    size_t new_len = before + replacement->byte_len + after;
    uint8_t* buf = malloc(new_len);
    memcpy(buf, haystack->data, before);
    memcpy(buf + before, replacement->data, replacement->byte_len);
    memcpy(buf + before + replacement->byte_len, pos + needle->byte_len, after);
    OlStr* result = strtab_intern(&vm->strings, buf, new_len);
    free(buf);
    return result;
}

// Compile string → OlProto (for REPL + eval)
// Bootstrap strategy (từ Crafting Interpreters):
//   Phase 1: Pratt expression-only compiler (~150 LOC C)
//            Handles: literals, arithmetic, comparison, bool, string, call
//            Đủ cho REPL expressions và estimate_valence()
//   Phase 2: Expand to full compiler (statements, control flow, functions)
//            Uses codegen from OLANG §14.5
//
// REPL tricks (từ Lua):
//   1. Try prepend "return " → compile → nếu OK thì là expression, in result
//   2. Nếu fail, compile as statement
//   3. Nếu error chứa "unexpected end" → incomplete input, chờ thêm
OlProto* compile_string(VM* vm, const char* code, size_t len) {
    // Phase 1 bootstrap: Pratt expression evaluator
    // Xem OLANG §14 cho full lexer/parser/codegen
    (void)vm; (void)code; (void)len;
    return NULL;  // implement Pratt parser trước
}

// Estimate valence from text (quick, for PTAV loop)
float estimate_valence(const char* text, size_t len) {
    // Quick heuristic: check NRC-VAD for first recognizable word
    // Full version: encode text → chain → average V dimension
    (void)text; (void)len;
    return 0.0f;  // neutral default
}

// Homeostasis error: RMSE between predicted and actual
float homeostasis_error(uint16_t* predicted, uint16_t* actual, int len) {
    if (len == 0) return 1.0f;  // max surprise
    float sum_sq = 0;
    int n = MIN(len, 8);
    for (int i = 0; i < n; i++) {
        float d = mol_dist(predicted[i], actual[i]) / 70.0f;
        sum_sq += d * d;
    }
    return sqrtf(sum_sq / n);
}
```

### 5.3 Pipeline (15 bước)

```c
typedef struct {
    SecurityGate gate;
    ContentEncoder encoder;
    KnowTree* kt;
    SilkGraph* silk;
    STM stm;
    DreamCycle dream;
    Instinct instincts[7];
} Pipeline;

// NOTE: PipelineResult dùng inline chain[64] cho performance (no heap alloc).
// MolecularChain trong MOL §8 dùng heap pointer cho general use.
// Conversion: memcpy(heap_chain.mols, inline_chain, len * 2)
typedef struct {
    uint16_t chain[64];    // inline chain (max 64 mols, STM/pipeline use)
    uint16_t chain_len;
    uint16_t emotion;      // P_weight of emotional state
    float    confidence;
    uint32_t node_id;      // KnowTree node (nếu đã lưu)
} PipelineResult;

// Full pipeline (từ Rust agents/pipeline/learning.rs)
PipelineResult pipeline_process(Pipeline* p, const char* text, size_t len) {
    PipelineResult result = {0};
    
    // Step 1: SecurityGate
    GateVerdict v = gate_check(&p->gate, text, len);
    if (v == GATE_BLOCK || v == GATE_CRISIS) {
        result.confidence = 0.0;
        return result;
    }
    
    // Step 2: Encode (text → molecular chain)
    encode_text(text, len, result.chain, &result.chain_len);
    
    // Step 3: Checkpoint CP1 — validate chain
    if (result.chain_len == 0) return result;
    
    // Step 4: Search KnowTree
    uint32_t nearest[10];
    int nearest_count = 0;
    kt_nearest(p->kt, result.chain[0], 10, nearest, &nearest_count);
    
    // Step 5: Homeostasis — đo surprise, Learn hay Act?
    float surprise = homeostasis_error(p->predicted_vec, result.chain, result.chain_len);
    int learning_mode = (surprise > HOMEOSTASIS_THRESHOLD);  // 0.5 (tunable, §21)
    if (learning_mode) {
        // Surprise cao → tăng learning rate, Dream thường xuyên hơn
        p->learning_rate_boost = 1.5f;
    } else {
        p->learning_rate_boost = 1.0f;
    }

    // Step 6: Compose — kết hợp input chain + nearest context chains
    // LCA(input, nearest) → blended representation
    if (nearest_count > 0) {
        uint16_t blended[64];
        uint16_t blended_len;
        chain_lca(result.chain, result.chain_len,
                  p->kt->nodes[nearest[0]].chain_data, p->kt->nodes[nearest[0]].chain_len,
                  blended, &blended_len);
        // Emotion = V/A from blended chain
        result.emotion = blended_len > 0 ? blended[0] : result.chain[0];
    }

    // Step 7: Checkpoint CP2 — ENCODE done
    
    // Step 8: Instincts (7 bản năng, Honesty LUÔN chạy đầu tiên)
    float honesty = instinct_honesty(p, result.chain, result.chain_len);
    if (honesty < 0.35) {
        result.confidence = honesty;
        return result;  // BlackCurtain: im lặng
    }
    
    float contradiction = instinct_contradiction(p, result.chain, result.chain_len, nearest, nearest_count);
    float causality = instinct_causality(p, result.chain, result.chain_len);
    float abstraction = instinct_abstraction(p, result.chain, result.chain_len);
    // ... analogy, curiosity, reflection
    
    // Step 9: Immune Selection — 3 candidates, pick lowest entropy
    //   Candidate A: direct answer from nearest match
    //   Candidate B: composed answer from LCA
    //   Candidate C: reasoning engine result (§23 arbiter)
    ReasoningResult reasoning = arbiter(p->vm, &(MolChain){result.chain, result.chain_len});
    float best_entropy = reasoning.entropy;
    // Pick candidate with lowest entropy (most certain)
    result.confidence = reasoning.confidence;

    // Step 10: DNA Repair — sửa nếu quality < φ⁻¹
    for (int repair_iter = 0; repair_iter < 3; repair_iter++) {
        float quality = 0.3f * result.confidence + 0.3f * (1.0f - best_entropy)
                      + 0.2f * (nearest_count > 0 ? 1.0f : 0.0f)  // consistency
                      + 0.2f * (silk_max_weight(p->silk, result.node_id) / 65535.0f);
        if (quality >= 0.618f) break;  // φ⁻¹ — good enough
        // Try: swap weakest dimension in chain with nearest's dimension
        if (nearest_count > 0 && result.chain_len > 0) {
            // Find weakest mol in chain, replace with nearest match
            int weakest = 0;
            int max_dist = 0;
            for (int i = 0; i < result.chain_len; i++) {
                int d = mol_dist(result.chain[i], p->kt->nodes[nearest[0]].p_weight);
                if (d > max_dist) { max_dist = d; weakest = i; }
            }
            result.chain[weakest] = p->kt->nodes[nearest[0]].p_weight;
        }
    }

    // Step 11: Checkpoint CP3 — INFER done
    
    // Step 12: Hebbian (Silk co-activate)
    for (int i = 0; i < nearest_count; i++) {
        silk_co_activate(p->silk, result.node_id, nearest[i], honesty);
    }
    
    // Step 13: Dream check (trigger nếu đủ Fibonacci threshold)
    stm_push(&p->stm, result.chain, result.chain_len, result.emotion);
    if (should_dream(&p->dream, p->stm.count)) {
        dream_cycle(&p->dream, &p->stm, p->kt, p->silk);
    }
    
    // Step 14: Checkpoint CP4 — PROMOTE done
    
    // Step 15: Decode (chain → text response)
    // Xem MOL §23 decode_chain_to_utf8() + VM §24 compose_response()
    // Decode result chain thành text, apply honesty prefix, tone adjustment
    // Actual generation happens in caller (vm_respond) not here

    // Update predicted vector for next homeostasis check
    if (result.chain_len > 0) {
        memcpy(p->predicted_vec, result.chain,
               MIN(result.chain_len, 8) * sizeof(uint16_t));
    }

    result.confidence = honesty;
    return result;
}
```

### 5.4 7 Instincts (từ Rust agents/skills/instinct.rs)

```c
// 1. Honesty: confidence = weighted sum
float instinct_honesty(Pipeline* p, uint16_t* chain, uint16_t len) {
    float silk_w = 0, fire = 0, sources = 0, consistency = 0;
    
    uint32_t node = kt_find(p->kt, chain, len);
    if (node != UINT32_MAX) {
        silk_w = silk_max_weight(p->silk, node) / 65535.0f;
        fire = MIN(p->kt->nodes[node].fire_count / 10.0f, 1.0f);
        sources = MIN(silk_neighbor_count(p->silk, node) / 3.0f, 1.0f);
        consistency = check_consistency(p, node);
    }
    
    return 0.3f * MIN(silk_w, 1.0f) 
         + 0.3f * fire 
         + 0.2f * sources 
         + 0.2f * consistency;
}

// 2. Contradiction: valence opposition + relation orthogonal
float instinct_contradiction(Pipeline* p, uint16_t* chain, uint16_t len,
                             uint32_t* neighbors, int n_count) {
    float max_score = 0;
    for (int i = 0; i < n_count; i++) {
        uint16_t mol_a = chain[0];
        uint16_t mol_b = p->kt->nodes[neighbors[i]].p_weight;
        
        float val_dist = abs(MOL_V(mol_a) - MOL_V(mol_b)) / 7.0f;
        float rel_dist = abs(MOL_R(mol_a) - MOL_R(mol_b)) / 15.0f;
        int both_extreme = (MOL_A(mol_a) >= 5 && MOL_A(mol_b) >= 5);
        
        float test1 = (val_dist > 0.6f && both_extreme) ? val_dist : 0;
        float test2 = (rel_dist < 0.2f) ? 0 : rel_dist; // orthogonal = high R distance
        float test3 = (MOL_V(mol_a) < 2 && MOL_V(mol_b) > 5) ? 1.0f : 0;
        
        float score = 0.4f * test1 + 0.3f * test2 + 0.3f * test3;
        if (score > max_score) max_score = score;
    }
    return max_score;
}

// 3. Causality: temporal + co-activation + R type
// 3 conditions, need >= 2/3 (SPEC_D §D2)
float instinct_causality(Pipeline* p, uint16_t* chain, uint16_t len) {
    if (len < 1 || p->stm.count < 2) return 0;

    // Compare current input vs most recent STM entry
    STMEntry* recent = &p->stm.entries[p->stm.count - 1];
    int evidence = 0;

    // Condition 1: temporal order (recent came before current)
    if (recent->timestamp < time_ms()) evidence++;

    // Condition 2: co-activation (silk weight > φ⁻¹)
    uint32_t node_a = kt_find(p->kt, recent->chain, recent->chain_len);
    uint32_t node_b = kt_find(p->kt, chain, len);
    if (node_a != UINT32_MAX && node_b != UINT32_MAX) {
        float w = silk_get_weight(p->silk, node_a, node_b) / 65535.0f;
        if (w > 0.618f) evidence++;
    }

    // Condition 3: R in CAUSES range (R = 8..12)
    uint8_t r = MOL_R(chain[0]);
    if (r >= 8 && r <= 12) evidence++;

    return (evidence >= 2) ? (float)evidence / 3.0f : 0.0f;
}

// 4. Abstraction: variance of cluster → concrete/categorical/abstract
float instinct_abstraction(Pipeline* p, uint16_t* chain, uint16_t len) {
    if (len < 1) return 0;

    // Find 10 nearest nodes
    uint32_t nearest[10];
    int n = 0;
    kt_nearest(p->kt, chain[0], 10, nearest, &n);
    if (n < 3) return 0;  // too few to abstract

    // Compute centroid
    int sum_s = 0, sum_r = 0, sum_v = 0, sum_a = 0, sum_t = 0;
    for (int i = 0; i < n; i++) {
        uint16_t pw = p->kt->nodes[nearest[i]].p_weight;
        sum_s += MOL_S(pw); sum_r += MOL_R(pw);
        sum_v += MOL_V(pw); sum_a += MOL_A(pw); sum_t += MOL_T(pw);
    }
    int cs = sum_s / n, cr = sum_r / n, cv = sum_v / n, ca = sum_a / n, ct = sum_t / n;

    // Variance (normalized 0-1)
    float var_sum = 0;
    for (int i = 0; i < n; i++) {
        uint16_t pw = p->kt->nodes[nearest[i]].p_weight;
        float ds = (MOL_S(pw) - cs) / 15.0f;
        float dr = (MOL_R(pw) - cr) / 15.0f;
        float dv = (MOL_V(pw) - cv) / 7.0f;
        float da = (MOL_A(pw) - ca) / 7.0f;
        float dt = (MOL_T(pw) - ct) / 3.0f;
        var_sum += ds*ds + dr*dr + dv*dv + da*da + dt*dt;
    }
    float variance = var_sum / (n * 5.0f);

    // < 0.15 = concrete, < 0.40 = categorical, >= 0.40 = abstract
    return variance;
}

// 5. Analogy: A:B :: C:D via 5D delta (compute predicted D)
void instinct_analogy_compute(uint16_t a, uint16_t b, uint16_t c, uint16_t* d) {
    int s = CLAMP(MOL_S(c) + MOL_S(b) - MOL_S(a), 0, 15);
    int r = CLAMP(MOL_R(c) + MOL_R(b) - MOL_R(a), 0, 15);
    int v = CLAMP(MOL_V(c) + MOL_V(b) - MOL_V(a), 0, 7);
    int ar = CLAMP(MOL_A(c) + MOL_A(b) - MOL_A(a), 0, 7);
    int t = CLAMP(MOL_T(c) + MOL_T(b) - MOL_T(a), 0, 3);
    *d = (s << 12) | (r << 8) | (v << 5) | (ar << 2) | t;
}

// 5b. Analogy score: trả float cho instinct opcode dispatch (fix C10)
float instinct_analogy_score(Pipeline* p, uint16_t* chain, uint16_t len) {
    if (len < 1 || !p->kt || p->kt->node_count < 2) return 0;
    uint32_t nearest[2];
    int n = 0;
    kt_nearest(p->kt, chain[0], 2, nearest, &n);
    if (n < 2) return 0;
    uint16_t predicted;
    instinct_analogy_compute(p->kt->nodes[nearest[0]].p_weight,
                             p->kt->nodes[nearest[1]].p_weight,
                             chain[0], &predicted);
    return 1.0f - mol_dist(predicted, chain[0]) / 70.0f;
}

// 6. Curiosity: novelty = 1 - nearest_similarity
float instinct_curiosity(Pipeline* p, uint16_t* chain, uint16_t len) {
    uint32_t nearest[1];
    int n_nearest = 0;
    kt_nearest(p->kt, chain[0], 1, nearest, &n_nearest);
    if (p->kt->node_count == 0) return 1.0f;  // everything is new
    
    float dist = mol_dist(chain[0], p->kt->nodes[nearest[0]].p_weight) / 70.0f;
    return MIN(dist, 1.0f);  // 0 = known, 1 = completely novel
}

// 7. Reflection: knowledge quality
float instinct_reflection(Pipeline* p) {
    if (p->kt->node_count == 0) return 0;
    float qr_ratio = (float)p->kt->qr_count / p->kt->node_count;
    float connectivity = (float)p->silk->edge_count / MAX(p->kt->node_count, 1);
    return 0.6f * qr_ratio + 0.4f * MIN(connectivity, 1.0f);
}
```

### 5.5 Dream Cycle (từ Rust memory/dream.rs)

```c
typedef struct {
    uint16_t chain[64];
    uint16_t chain_len;
    uint16_t emotion;
    uint32_t fire_count;
    int64_t  timestamp;
} STMEntry;

typedef struct {
    STMEntry entries[32];  // max 32 items (từ spec)
    int      count;
} STM;

// Fibonacci trigger: dream at Fib(2,3,5,8,13,21,34,55...) items
int should_dream(DreamCycle* dc, int stm_count) {
    static const int fib[] = {2, 3, 5, 8, 13, 21, 34, 55};
    for (int i = 0; i < 8; i++) {
        if (stm_count == fib[i]) return 1;
    }
    return 0;
}

// Dream: cluster STM → LCA per cluster → promote if qualified
void dream_cycle(DreamCycle* dc, STM* stm, KnowTree* kt, SilkGraph* silk) {
    // 1. Select top-10 by fire_count
    // 2. Cluster by LCA similarity (greedy, threshold 0.3)
    // 3. Per cluster: compute representative chain = LCA of all
    // 4. Score = 0.3*frequency + 0.4*connectivity + 0.3*emotion_intensity
    // 5. If score >= 0.5: create DreamProposal
    // 6. Proposal: kt_store + silk_co_activate cluster members
    // 7. If fire_count >= Fibonacci(depth) AND weight >= 854: promote to QR
    
    // (full implementation from Rust memory/dream.rs)
}
```

### 5.6 Memory Architecture — 3-Layer Decay Model

Kết hợp 3 nghiên cứu thành 1 kiến trúc thống nhất:
- Memory Chain Model (Murre 2015, PMC4492928): 2 stores + consolidation
- Two-phase decay (Candia 2019, PMC9744905): exp → power law switch
- Stretched exponential (Kohlrausch 1854): interpolation exp ↔ power law

```c
/* 3 tầng nhớ, mỗi tầng có decay khác nhau:
 *
 * ┌──────────┐  fast decay     ┌──────────────┐  slow decay    ┌─────────┐
 * │   STM    │ ──── Dream ───→ │  KnowTree    │ ── QR promo ─→ │   QR    │
 * │ (giờ)    │  consolidation  │  + Silk      │                │ (forever)│
 * └──────────┘                 └──────────────┘                └─────────┘
 *
 * STM decay:       w × 0.5^(hours/4)     — half-life 4 giờ (nhanh)
 *                  pure exponential       — không cần long tail
 *                  Nếu không fire lại trong vài giờ → biến mất
 *
 * Silk decay:      w × exp(-(t/78.37)^φ⁻¹) — stretched exponential
 *                  half-life ≈ 24h tại lần đầu
 *                  long tail: 1 tuần vẫn còn 0.216 (thay vì 0.028)
 *                  Liquid τ (§15) scale theo context
 *
 * QR:              không decay — vĩnh viễn, append-only
 *                  QR = kết quả của Dream consolidation thành công
 *
 * Dream = consolidation transfer (MCM's μ₂):
 *   STM items fire đủ → cluster → LCA → score → promote vào KnowTree
 *   KnowTree items mature đủ → promote vào QR
 *
 * Two-phase switching:
 *   Phase 1 (t < ~10 days): Silk stretched exp dominates → forgetting
 *   Phase 2 (t > ~10 days): items còn sống = đã fire đủ → promotion candidate
 *   Switching point tự nhiên: items survive stretched exp = strong enough for QR
 */

// STM decay: pure exponential, half-life 4 hours
float stm_decay(float w0, float elapsed_hours) {
    return w0 * powf(0.5f, elapsed_hours / 4.0f);
}

// STM eviction: decay + importance scoring
float stm_eviction_score(STMEntry* e, int64_t now_ms) {
    float hours = (float)(now_ms - e->timestamp) / (3600.0f * 1000.0f);
    float decay = stm_decay(1.0f, hours);
    float fire_bonus = logf(1.0f + e->fire_count) / 5.0f;  // diminishing returns
    float emotion = (float)(MOL_V(e->emotion)) / 7.0f;      // emotional = important
    return decay * (1.0f + fire_bonus + emotion * 0.5f);
    // Thấp nhất → evict đầu tiên
}

// Silk decay: stretched exponential β=φ⁻¹ (đã define trong §5.2)
// → xem MOL §26 cho chi tiết

// QR: no decay function needed — permanent
```

### 5.7 Adaptive Promotion — BCM + IWCM kết hợp

Thay vì threshold cố định (0.65 hay 0.854), dùng BCM adaptive:

```c
/* Kết hợp 2 nghiên cứu:
 *
 * IWCM (RiverText, arXiv:2506.23192):
 *   Đếm co-occurrence trực tiếp: count[a][b]++ khi a, b co-fire
 *   Không cần negative sampling, không gradient, CPU-only
 *   Đơn giản hơn Word2Vec
 *
 * BCM (Bienenstock-Cooper-Munro, 1982):
 *   theta_M = <activity>^p — sliding threshold tự điều chỉnh
 *   Cao activity → khó LTP. Thấp activity → dễ LTP.
 *
 * Kết hợp:
 *   IWCM cung cấp activity data → BCM tính threshold → QR promotion gate
 *   Shadow Vector dim[5..7] HỌC từ cùng co-occurrence data
 */

typedef struct {
    uint32_t co_fire_total;     // tổng co-fire events
    float    avg_activity;      // running average (EMA)
    float    theta_promote;     // BCM adaptive threshold
} DomainStats;

// Gọi mỗi khi 2 nodes co-fire
void on_co_fire(DomainStats* ds, KnowTree* kt,
                uint32_t node_a, uint32_t node_b) {
    // 1. IWCM: đếm co-occurrence
    ds->co_fire_total++;

    // 2. Shadow Vector: update dim[5..7] từ co-occurrence
    //    Đơn giản hơn Word2Vec: cộng dồn, normalize periodic
    float* sv_a = &kt->shadow_vectors[node_a * 8 + 5];
    float* sv_b = &kt->shadow_vectors[node_b * 8 + 5];
    // EMA blend: dim[5] của a ← 0.95 * a + 0.05 * b (và ngược lại)
    for (int d = 0; d < 3; d++) {  // dims 5,6,7
        float tmp_a = sv_a[d], tmp_b = sv_b[d];
        sv_a[d] = 0.95f * tmp_a + 0.05f * tmp_b;
        sv_b[d] = 0.95f * tmp_b + 0.05f * tmp_a;
    }

    // 3. BCM: update adaptive threshold
    float current_rate = (float)ds->co_fire_total;  // normalize later
    ds->avg_activity = 0.99f * ds->avg_activity + 0.01f * current_rate;
    // theta = avg^p, p=1.5 → superlinear
    ds->theta_promote = powf(ds->avg_activity, 1.5f);
    // Clamp to [0.3, 0.95] — không quá dễ, không quá khó
    if (ds->theta_promote < 0.3f) ds->theta_promote = 0.3f;
    if (ds->theta_promote > 0.95f) ds->theta_promote = 0.95f;
}

// QR promotion check — thay thế fixed threshold
int check_promotion_adaptive(KnowTree* kt, SilkGraph* silk,
                              DomainStats* ds, uint32_t node_id) {
    KTNode* node = &kt->nodes[node_id];
    if (node->maturity != MATURITY_MATURE) return 0;

    // Fibonacci fire threshold (giữ nguyên)
    static const int fib[] = {2, 3, 5, 8, 13, 21, 34, 55};
    int depth = node->layer;
    int fib_thresh = (depth < 8) ? fib[depth] : 55;
    if (node->fire_count < (uint16_t)fib_thresh) return 0;

    // BCM adaptive threshold (thay vì 0.65 hoặc 0.854 cố định)
    float weight = silk_max_weight(silk, node_id) / 65535.0f;
    if (weight < ds->theta_promote) return 0;

    return 1;  // promote to QR
}
```

**Ưu điểm:**
- Không cần chọn 0.65 hay 0.854 — threshold tự điều chỉnh
- Domain active (nhiều data) → strict hơn → lọc noise
- Domain quiet (ít data) → lenient hơn → không bỏ sót
- Shadow Vector học từ cùng data, không cần negative sampling riêng

### 5.8 Encode: 42 Formulas (từ Rust mol/encoder.rs + ucd crate)

```c
// UCD table: generated from json/udc.json (8,284 entries)
// UCD data = thức ăn cho KnowTree (xem MOL §22), KHÔNG dùng cho encode.
// UCDEntry struct định nghĩa trong MOL §7.

// encode_codepoint: Unicode codepoint → P_weight u16
// LUÔN TÍNH bằng 42 formulas (MOL §7). KHÔNG tra bảng. (TÍNH không TRA)
uint16_t encode_codepoint(uint32_t cp) {
    // 42 formulas (từ UDC_DOC — 3 tầng)
    // Nguồn: Origin_RUST_Sfvz/docs/UDC_DOC/UDC_formulas.md
    //
    // Tầng 1: F₀ master router — dispatch theo block membership
    // Tầng 2: f_S, f_R, f_V, f_A, f_T — per-dimension encoder
    // Tầng 3: 36 keyword classifiers + quantizers
    //
    // 26 keyword matchers: match char_name vs keyword set → group_id
    //  2 score lookups: NRC-VAD → raw V/A float
    //  2 quantizers: float → 3-bit int
    // 10 subgroup mappers: emoji subgroup → V/A defaults
    //  1 compositor: 5 dims → 2 bytes
    //  1 spline assembler: duration × pitch × amplitude → T

    return encode_by_formula(cp);
}

// encode_text: text → MolecularChain
void encode_text(const char* text, size_t len, uint16_t* chain, uint16_t* chain_len) {
    const uint8_t* p = (const uint8_t*)text;
    const uint8_t* end = p + len;
    *chain_len = 0;
    
    while (p < end && *chain_len < 64) {
        uint32_t cp = utf8_decode(&p);
        if (cp == ' ' || cp == '\t' || cp == '\n') continue;  // skip whitespace
        chain[(*chain_len)++] = encode_codepoint(cp);
    }
}

// LCA: Lowest Common Ancestor of 2 chains
// Thuật toán từ Rust mol/lca.rs
void chain_lca(uint16_t* a, uint16_t a_len, uint16_t* b, uint16_t b_len,
               uint16_t* result, uint16_t* result_len) {
    // Weighted LCA: giữ molecules xuất hiện trong CẢ 2 chains
    // Weight = position-based (đầu chain quan trọng hơn)
    *result_len = 0;
    
    for (int i = 0; i < a_len && *result_len < 64; i++) {
        for (int j = 0; j < b_len; j++) {
            if (mol_dist(a[i], b[j]) <= 4) {  // close enough
                // Compose: S=union(min), R=zipf, V=amplify, A=max, T=dominant
                result[(*result_len)++] = mol_compose_pair(a[i], b[j]);
                break;
            }
        }
    }
    
    if (*result_len == 0 && a_len > 0 && b_len > 0) {
        // Fallback: compose first molecules
        result[0] = mol_compose_pair(a[0], b[0]);
        *result_len = 1;
    }
}

// Compose 2 molecules (từ Rust mol/lca.rs)
uint16_t mol_compose_pair(uint16_t a, uint16_t b) {
    // S compose = MAX (dominant shape)
    // S index = category (0=Arrow, 1=Geometric, 2=Box...),  KHÔNG phải SDF distance.
    // SDF union min(d1,d2) áp dụng cho distance values, không cho category indices.
    // MAX = dominant shape wins (shape phức tạp hơn bao trùm shape đơn giản).
    // Nhất quán với: NOX_COMPLETE_REFERENCE, NOX_AI_MODEL_SPEC, MOL compose_union().
    int s = MAX(MOL_S(a), MOL_S(b));
    int r = (MOL_R(a) * 2 + MOL_R(b)) / 3;              // R: Zipf weighting
    int v = CLAMP(MOL_V(a) + (MOL_V(b) - 4) / 2, 0, 7); // V: Amplify
    int ar = MAX(MOL_A(a), MOL_A(b));                     // A: Max
    int t = (MOL_T(a) > MOL_T(b)) ? MOL_T(a) : MOL_T(b); // T: Dominant
    return (s << 12) | (r << 8) | (v << 5) | (ar << 2) | t;
}
```

---

## 6. I/O SUBSYSTEM

### 6.1 File I/O (POSIX direct, không libuv)

```c
// Read file → OlStr (interned)
Value vm_file_read(VM* vm, const char* path) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return val_nil();
    
    struct stat st;
    fstat(fd, &st);
    size_t size = st.st_size;
    
    uint8_t* buf = malloc(size);
    read(fd, buf, size);
    close(fd);
    
    OlStr* s = string_intern(&vm->strings, buf, size);
    free(buf);
    return val_string(s);
}

// Write file
Value vm_file_write(VM* vm, const char* path, OlStr* data) {
    int fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd < 0) return val_float(0.0);
    write(fd, data->data, data->byte_len);
    close(fd);
    return val_float(1.0);  // confidence = 1.0 = success
}
```

### 6.2 UTF-8 Output (FIX cho DATA-3)

```c
// Emit: output Value as UTF-8 to stdout
void vm_emit(VM* vm, Value v) {
    if (is_string(v)) {
        OlStr* s = as_string(v);
        write(vm->stdout_fd, s->data, s->byte_len);  // write ALL bytes, not just low byte
        write(vm->stdout_fd, "\n", 1);
    } else if (is_float(v)) {
        char buf[32];
        int len = snprintf(buf, 32, "%g", as_float(v));
        write(vm->stdout_fd, buf, len);
        write(vm->stdout_fd, "\n", 1);
    } else if (is_int(v)) {
        char buf[24];
        int len = snprintf(buf, 24, "%d", as_int(v));
        write(vm->stdout_fd, buf, len);
        write(vm->stdout_fd, "\n", 1);
    } else if (is_mol(v)) {
        uint16_t m = as_mol(v);
        char buf[64];
        int len = snprintf(buf, 64, "mol{S=%d R=%d V=%d A=%d T=%d}",
                           MOL_S(m), MOL_R(m), MOL_V(m), MOL_A(m), MOL_T(m));
        write(vm->stdout_fd, buf, len);
        write(vm->stdout_fd, "\n", 1);
    }
    // ... array, dict, chain
}
```

### 6.3 "Eating" Languages

Origin ăn Julia/Python/JSON bằng cách parse chúng thành KnowTree nodes:

```c
// Eat Julia file: parse → extract functions/types → encode → store
void eat_julia(VM* vm, const char* path) {
    Value content = vm_file_read(vm, path);
    if (is_nil(content)) return;
    
    OlStr* s = as_string(content);
    
    // Simple parser: extract function signatures, type definitions, doc comments
    // Mỗi extracted item → encode → kt_store
    
    JuliaParser jp;
    julia_parser_init(&jp, s->data, s->byte_len);
    
    while (julia_parser_next(&jp)) {
        switch (jp.item_type) {
            case JULIA_FUNCTION: {
                // Encode function name + signature → molecular chain
                uint16_t chain[64];
                uint16_t chain_len;
                encode_text(jp.name, jp.name_len, chain, &chain_len);
                
                // Store with metadata
                uint32_t node = kt_store(vm->knowtree, chain, chain_len, 0);
                
                // Store source code as associated data
                kt_set_source(vm->knowtree, node, jp.body, jp.body_len);
                break;
            }
            case JULIA_STRUCT:
            case JULIA_CONST:
            case JULIA_COMMENT:
                // Similar: encode name → chain → store
                break;
        }
    }
}

// Generic: eat any text file
void eat_text(VM* vm, const char* path) {
    Value content = vm_file_read(vm, path);
    if (is_nil(content)) return;
    
    OlStr* s = as_string(content);
    
    // Split by sentences/paragraphs → encode each → store
    // Uses same algorithm as Rust agents/pipeline/learning.rs Layer 1
}
```

---

## 7. SELF-MODIFICATION

### 7.1 Safe Self-Modify Cycle (từ Rust SPEC_F + DECISIONS A6/A7)

```
INSPECT → IDENTIFY → PLAN → BACKUP → MODIFY → BUILD → TEST → VERIFY → PASS/ROLLBACK
```

```c
typedef struct {
    char*    backup_path;     // backup file path
    uint32_t original_hash;   // hash of original code
    uint32_t modified_hash;   // hash of modified code
    int      test_passed;
} ModifyContext;

// Template slots (DECISIONS A6): backbone cố định + slots thay đổi
// Không decompile, không string-edit toàn bộ source
typedef struct {
    char*    template_source;  // source code với {{SLOT_N}} markers
    char**   slot_values;      // giá trị cho mỗi slot
    int      slot_count;
} TemplateModify;

// Chromosomal redundancy (DECISIONS A7): copy → sửa copy → test → swap
Value vm_self_modify(VM* vm, Value file, Value old_str, Value new_str) {
    OlStr* path = as_string(file);
    OlStr* old = as_string(old_str);
    OlStr* new = as_string(new_str);
    
    // 1. Backup
    char backup[256];
    snprintf(backup, 256, "%s.bak", path->data);
    copy_file(path->data, backup);
    
    // 2. Read current
    Value content = vm_file_read(vm, path->data);
    OlStr* source = as_string(content);
    
    // 3. Replace (in copy, not original)
    OlStr* modified = string_replace(vm, source, old, new);
    
    // 4. Write modified
    vm_file_write(vm, path->data, modified);
    
    // 5. Test: compile modified file
    Value compiled = vm_compile(vm, modified);
    if (is_nil(compiled)) {
        // Rollback
        copy_file(backup, path->data);
        return val_float(0.0);  // failed
    }
    
    // 6. Verify: compile + run modified code, check output matches expected
    // Self-modify chỉ accept nếu test pass (chromosomal redundancy — DECISION A7)
    
    // 7. Success
    vm->code_generation++;
    return val_float(1.0);  // success
}
```

### 7.2 Template Slot Mechanism (DECISION A6)

```c
/* Template slots: backbone cố định + slots thay đổi.
 * Không decompiler, không string edit toàn bộ source.
 *
 * Source file có markers: {{SLOT_0}}, {{SLOT_1}}, ...
 * VM thay thế markers bằng giá trị mới.
 *
 * Ví dụ trong brain.ol:
 *   let threshold = {{SLOT_0}};     // QR promotion threshold
 *   let decay_tau = {{SLOT_1}};     // decay time constant
 *   let dream_interval = {{SLOT_2}}; // dream trigger frequency
 *
 * Self-modify = thay đổi SLOT values, không thay đổi logic.
 * Logic (backbone) bất biến. Parameters (slots) mutable.
 */

// Find and replace template slots in source
OlStr* template_fill_slots(VM* vm, OlStr* template_src,
                            Value* slot_values, int slot_count) {
    // Working copy
    char* result = malloc(template_src->byte_len + slot_count * 64);
    memcpy(result, template_src->data, template_src->byte_len);
    result[template_src->byte_len] = '\0';
    size_t result_len = template_src->byte_len;

    for (int i = 0; i < slot_count; i++) {
        char marker[16];
        snprintf(marker, sizeof(marker), "{{SLOT_%d}}", i);

        // Convert value to string
        char val_str[64];
        if (is_int(slot_values[i]))
            snprintf(val_str, 64, "%d", as_int(slot_values[i]));
        else if (is_float(slot_values[i]))
            snprintf(val_str, 64, "%g", as_float(slot_values[i]));
        else if (is_string(slot_values[i]))
            snprintf(val_str, 64, "%.*s", as_string(slot_values[i])->byte_len,
                     as_string(slot_values[i])->data);
        else
            snprintf(val_str, 64, "nil");

        // Replace all occurrences of marker with val_str
        char* pos;
        while ((pos = strstr(result, marker)) != NULL) {
            size_t marker_len = strlen(marker);
            size_t val_len = strlen(val_str);
            memmove(pos + val_len, pos + marker_len,
                    result_len - (pos - result) - marker_len + 1);
            memcpy(pos, val_str, val_len);
            result_len += val_len - marker_len;
        }
    }

    OlStr* out = strtab_intern(&vm->strings, (uint8_t*)result, result_len);
    free(result);
    return out;
}

// Self-modify via template: safe cycle
// 1. Load template source
// 2. Fill slots with new values
// 3. Compile filled template
// 4. Test compiled result
// 5. If pass: swap running code. If fail: rollback.
Value vm_self_modify_template(VM* vm, OlStr* template_path,
                               Value* slot_values, int slot_count) {
    // 1. Read template
    Value content = vm_file_read(vm, (const char*)template_path->data);
    if (is_nil(content)) return val_float(0.0);

    // 2. Fill slots
    OlStr* filled = template_fill_slots(vm, as_string(content),
                                         slot_values, slot_count);

    // 3. Backup + write
    char backup[256];
    snprintf(backup, 256, "%s.bak", template_path->data);
    copy_file((const char*)template_path->data, backup);
    vm_file_write(vm, (const char*)template_path->data, filled);

    // 4. Compile + test
    Value compiled = vm_compile(vm, filled);
    if (is_nil(compiled)) {
        copy_file(backup, (const char*)template_path->data);
        return val_float(0.0);  // rollback
    }

    // 5. Success
    vm->code_generation++;
    return val_float(1.0);
}
```

### 7.3 Code Generation Counter

```c
// Mỗi khi code thay đổi, generation tăng
// Cached JIT code với generation < current = invalidated
// Interpreter check: if (cached->generation != vm->code_generation) recompile();
```

---

## 8. PERSISTENCE

### 8.1 Binary Format cho KnowTree + Silk save/load

```
File: origin.dat

Header (16 bytes):
  magic: "ORIG"
  version: u16
  node_count: u32
  edge_count: u32
  flags: u16
  
Node Section:
  [KTNode; node_count]  // 32 bytes per node
  
Chain Section:
  [u16; total_chain_mols]  // all chains concatenated
  
Edge Section:
  [SilkEdge; edge_count]  // 24 bytes per edge

Persist Section:
  count: u32
  entries: [{
    key_len: u16,
    key: [u8; key_len],
    val_type: u8,      // 0=int, 1=float, 2=string, 3=mol, 4=chain
    val_len: u32,
    val: [u8; val_len],
  }; count]
```

### 8.2 Save/Load Code

```c
/* Persistence: 3 layers (từ BP13)
 *
 * Layer 1: mmap snapshot — main KnowTree + Silk state
 *   File: origin.dat (format §8.1)
 *   Save: periodic checkpoint (mỗi 5 phút idle hoặc 100 interactions)
 *   Load: startup — mmap file, reconstruct pointers
 *
 * Layer 2: WAL (Write-Ahead Log) — incremental changes
 *   File: origin.wal
 *   Mỗi operation (kt_store, silk_fire, qr_promote) → append WAL entry
 *   On startup: replay WAL entries newer than mmap snapshot
 *   Periodic: checkpoint (sync mmap + truncate WAL)
 *
 * Layer 3: Dream consolidation — promote STM→KnowTree→QR
 *   Runs during idle (>5 min) hoặc Fibonacci trigger
 *   Batch-write promoted QR entries
 *   Prune decayed edges (weight < threshold)
 *   Sync + truncate WAL
 */

// WAL entry format
typedef struct {
    uint8_t  op;          // WAL_KT_STORE=1, WAL_SILK_FIRE=2, WAL_QR_PROMOTE=3
    uint32_t timestamp;   // seconds since epoch (truncated)
    uint16_t data_len;
    // data follows (variable length)
} WALEntry;

#define WAL_KT_STORE    1
#define WAL_SILK_FIRE   2
#define WAL_QR_PROMOTE  3
#define WAL_PERSIST_SET 4

// Save KnowTree + Silk to file
int origin_save(VM* vm, const char* path) {
    int fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd < 0) return -1;

    // Header
    uint8_t header[16];
    memcpy(header, "ORIG", 4);
    *(uint16_t*)(header + 4) = 1;  // version
    *(uint32_t*)(header + 6) = vm->knowtree->node_count;
    *(uint32_t*)(header + 10) = vm->silk->edge_count;
    *(uint16_t*)(header + 14) = 0;  // flags
    write(fd, header, 16);

    // Nodes
    write(fd, vm->knowtree->nodes,
          vm->knowtree->node_count * sizeof(KTNode));

    // Chains (concatenated)
    write(fd, vm->knowtree->chains,
          vm->knowtree->chain_alloc * sizeof(uint16_t));

    // Edges
    write(fd, vm->silk->edges,
          vm->silk->edge_count * sizeof(SilkEdge));

    // Persist key-value store
    // ... (iterate persist entries, write each)

    close(fd);
    return 0;
}

// Load from file
int origin_load(VM* vm, const char* path) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;

    // Header
    uint8_t header[16];
    read(fd, header, 16);
    if (memcmp(header, "ORIG", 4) != 0) { close(fd); return -1; }

    uint32_t node_count = *(uint32_t*)(header + 6);
    uint32_t edge_count = *(uint32_t*)(header + 10);

    // Allocate and read nodes
    vm->knowtree->nodes = malloc(node_count * sizeof(KTNode));
    vm->knowtree->node_count = node_count;
    read(fd, vm->knowtree->nodes, node_count * sizeof(KTNode));

    // Read chains
    // ... (read chain data, rebuild chain_offset pointers)

    // Read edges
    vm->silk->edges = malloc(edge_count * sizeof(SilkEdge));
    vm->silk->edge_count = edge_count;
    read(fd, vm->silk->edges, edge_count * sizeof(SilkEdge));

    close(fd);

    // Rebuild VP-Tree index from loaded data
    vptree_index_maybe_rebuild(&vm->vptree_idx, vm->knowtree);

    return 0;
}

// WAL append (call after every state-changing operation)
void wal_append(VM* vm, uint8_t op, const void* data, uint16_t len) {
    if (vm->wal_fd < 0) return;
    WALEntry entry = { .op = op, .timestamp = (uint32_t)time(NULL), .data_len = len };
    write(vm->wal_fd, &entry, sizeof(WALEntry));
    write(vm->wal_fd, data, len);
}

// WAL replay (on startup, after loading snapshot)
void wal_replay(VM* vm, const char* wal_path) {
    int fd = open(wal_path, O_RDONLY);
    if (fd < 0) return;

    WALEntry entry;
    while (read(fd, &entry, sizeof(WALEntry)) == sizeof(WALEntry)) {
        uint8_t* data = malloc(entry.data_len);
        read(fd, data, entry.data_len);

        switch (entry.op) {
        case WAL_KT_STORE: {
            uint16_t* chain = (uint16_t*)data;
            uint16_t len = entry.data_len / 2;
            kt_store(vm->knowtree, chain, len, 0);
            break;
        }
        case WAL_SILK_FIRE: {
            uint32_t a = *(uint32_t*)data;
            uint32_t b = *(uint32_t*)(data + 4);
            silk_co_activate(vm->silk, a, b);
            break;
        }
        case WAL_QR_PROMOTE: {
            uint32_t node_id = *(uint32_t*)data;
            vm->knowtree->nodes[node_id].flags |= 1;  // QR bit
            break;
        }
        }
        free(data);
    }
    close(fd);
}

// Checkpoint: sync mmap + truncate WAL
void origin_checkpoint(VM* vm) {
    origin_save(vm, "origin.dat");
    // Truncate WAL
    if (vm->wal_fd >= 0) {
        ftruncate(vm->wal_fd, 0);
        lseek(vm->wal_fd, 0, SEEK_SET);
    }
}
```

### 8.3 Persist Key-Value Store (Olang-accessible)

```c
// Olang: persist_save("key", value) / persist_load("key")
// Stored in origin.dat Persist Section

typedef struct {
    OlStr*  key;
    Value   val;
} PersistEntry;

typedef struct {
    PersistEntry* entries;
    uint32_t count;
    uint32_t capacity;
} PersistStore;

void persist_set(PersistStore* ps, OlStr* key, Value val) {
    // Update existing
    for (uint32_t i = 0; i < ps->count; i++) {
        if (ps->entries[i].key == key) {
            ps->entries[i].val = val;
            return;
        }
    }
    // New entry
    if (ps->count >= ps->capacity) {
        ps->capacity = ps->capacity ? ps->capacity * 2 : 32;
        ps->entries = realloc(ps->entries, ps->capacity * sizeof(PersistEntry));
    }
    ps->entries[ps->count++] = (PersistEntry){ .key = key, .val = val };
}

Value persist_get(PersistStore* ps, OlStr* key) {
    for (uint32_t i = 0; i < ps->count; i++) {
        if (ps->entries[i].key == key) return ps->entries[i].val;
    }
    return val_nil();
}
```

---

## 9. BINARY FORMAT (.origin)

Standalone executable format:

```
+------------------+
| ELF/PE Header    |  (platform-specific, generated)
+------------------+
| VM Code (C comp) |  Origin VM implementation
+------------------+
| Bytecode Section |  .olang bytecode (compiled Olang)
+------------------+
| UCD Table        |  Unicode Character Data (8,284 entries)
+------------------+
| KnowTree Data    |  Persistent knowledge (nếu có)
+------------------+
| Silk Data        |  Persistent edges (nếu có)
+------------------+

Total estimated: ~500KB VM + ~200KB UCD + bytecode + data
```

---

## 10. AGENT HIERARCHY (từ Rust gốc)

### 10.1 3 Tiers

```
Tier 0: AAM (Auto-Approve Module)
  - Stateless, approve/reject only
  - Kiểm tra 23 QT rules
  - Im lặng khi không chắc chắn

Tier 1: Chiefs (LeoAI, HomeChief, VisionChief, NetworkChief)  
  - LeoAI: 11 skills (Ingest, Cluster, Similarity, Delta, Curator, Merge, Prune, Hebbian, Dream, Proposal, Honesty)
  - HomeChief: sensor/actuator management
  - VisionChief: camera processing
  - NetworkChief: network security

Tier 2: Workers (silent, report via chain)
  - WorkerKind: Sensor, Actuator, Camera, Network, Generic
  - Mỗi worker ~16-64KB
  - Communicate qua ISL (Inter-System Link)
```

### 10.2 ISL Protocol

```c
// Address: 4 bytes [layer, group, subgroup, index]
typedef struct {
    uint8_t layer;
    uint8_t group;
    uint8_t subgroup;
    uint8_t index;
} ISLAddress;

// Message: 12 bytes
typedef struct {
    ISLAddress from;
    ISLAddress to;
    uint8_t    msg_type;  // 14 types: Text, Chain, Emotion, Command, ...
    uint8_t    payload[3];
} ISLMessage;

// Frame: message + extended body
typedef struct {
    ISLMessage header;
    uint8_t*   body;
    uint16_t   body_len;
} ISLFrame;

// Queue: dual priority (urgent + normal)
typedef struct {
    ISLFrame urgent[256];
    ISLFrame normal[256];
    int      urgent_head, urgent_tail;
    int      normal_head, normal_tail;
} ISLQueue;
```

### 10.3 PTAV Agent Loop (từ SPEC_F Agent)

```c
/* PTAV = Perceive → Think → Act → Verify (+ Feedback)
 * Vòng lặp chính của Nox agent.
 * Chạy trong heartbeat loop (mỗi 100ms hoặc khi có input).
 *
 * Nguồn: SPEC_F_AGENT §1-3
 */

typedef struct {
    // State
    int       mode;           // 0=idle, 1=learning, 2=acting, 3=dreaming
    int64_t   last_input_ms;  // timestamp of last user input
    int64_t   last_dream_ms;  // timestamp of last dream cycle
    uint32_t  interaction_count;
    float     current_valence; // V from ConversationCurve
    float     current_arousal; // A from ConversationCurve

    // Goals (stack-based, max 8 active)
    struct {
        uint16_t chain[16];   // molecular goal representation
        uint16_t chain_len;
        float    priority;    // 0-1
        float    progress;    // 0-1
    } goals[8];
    int goal_count;

    // Components
    Pipeline*       pipeline;
    ConversationCurve* curve;
    PersistStore*   persist;
    DecodeMap*      decode_map;
} Agent;

// Main agent loop — called by heartbeat or input event
void agent_tick(Agent* ag, VM* vm, const char* input, size_t input_len) {
    int64_t now = time_ms();

    // ═══ PERCEIVE ═══
    if (input && input_len > 0) {
        ag->last_input_ms = now;
        ag->interaction_count++;

        // Push valence to conversation curve
        float input_v = estimate_valence(input, input_len);  // from NRC-VAD
        curve_push(ag->curve, input_v);
        ag->current_valence = ag->curve->fx;

        // Detect intent
        IntentResult intent = detect_intent(input, ag->current_valence, ag->current_arousal);

        // ═══ THINK ═══
        // Run full pipeline
        PipelineResult result = pipeline_process(ag->pipeline, input, input_len);

        // ═══ ACT ═══
        // Generate response
        int action = decide_action(&intent, ag->current_valence);
        int tone = curve_tone(ag->curve);

        Response resp;
        compose_response(&resp, action, tone, ag->current_valence,
                        input, result.confidence, ag->interaction_count);

        // Apply honesty prefix
        char final_output[2048];
        apply_honesty_prefix(final_output, sizeof(final_output),
                            result.confidence, resp.text);

        // Output
        if (final_output[0] != '\0') {
            write(STDOUT_FILENO, final_output, strlen(final_output));
            write(STDOUT_FILENO, "\n", 1);
        }

        // ═══ VERIFY ═══
        // Log for feedback tracking (reward comes from next user input)
        wal_append(vm, WAL_KT_STORE, result.chain, result.chain_len * 2);

        free(resp.text);
    }

    // ═══ IDLE PROCESSING ═══
    int64_t idle_ms = now - ag->last_input_ms;

    // Dream when idle > 5 minutes
    if (idle_ms > 5 * 60 * 1000 && (now - ag->last_dream_ms) > 60 * 1000) {
        ag->mode = 3;  // dreaming
        dream_cycle(&ag->pipeline->dream, &ag->pipeline->stm,
                    ag->pipeline->kt, ag->pipeline->silk);
        ag->last_dream_ms = now;

        // Checkpoint persistence
        origin_checkpoint(vm);
        ag->mode = 0;  // idle
    }

    // Meta-learning check (every 50 interactions)
    if (ag->interaction_count % 50 == 0 && ag->interaction_count > 0) {
        // Adjust liquid τ parameters, template weights
        // Max ±20% per cycle (safety bound)
        // → xem VM §27 CP7 Meta Loop
    }
}

// Heartbeat: called periodically (e.g., every 100ms via timer/io_uring)
void agent_heartbeat(Agent* ag, VM* vm) {
    agent_tick(ag, vm, NULL, 0);  // no input, just idle processing
}

// Startup: load persisted state, rebuild indexes
void agent_init(Agent* ag, VM* vm) {
    memset(ag, 0, sizeof(Agent));
    ag->pipeline = &vm->pipeline;
    ag->curve = &vm->curve;
    ag->persist = vm->persist;

    // Load persistent state
    origin_load(vm, "origin.dat");
    wal_replay(vm, "origin.wal");

    // Open WAL for append
    vm->wal_fd = open("origin.wal", O_WRONLY | O_CREAT | O_APPEND, 0644);

    // Build decode map
    ag->decode_map = malloc(sizeof(DecodeMap));
    decode_map_build(ag->decode_map);  // MOL §23

    ag->mode = 0;
    ag->last_input_ms = time_ms();
}
```

---

## 11. JULIA INTEGRATION — "Julia = thức ăn"

Julia là **offline compute tool**, KHÔNG runtime dependency (DECISION A2: No FFI).
Julia tính toán → output binary files → Origin VM mmap load.

### 11.1 Julia Pre-Compute Pipeline

```julia
# tools/precompute.jl — chạy 1 lần khi build, output vào data/
using StaticArrays, NearestNeighbors, Distances, Mmap, Unicode

# ─── 1. Molecule Distance Pre-Compute ───────────────────────────
# Custom distance metric cho 5D molecules
struct MolDist <: Distances.Metric end
function Distances.evaluate(::MolDist, a::AbstractVector, b::AbstractVector)
    abs(a[1]-b[1]) + abs(a[2]-b[2]) + 2*abs(a[3]-b[3]) + 2*abs(a[4]-b[4]) + 4*abs(a[5]-b[5])
end

# Unpack all 65536 possible molecules vào 5D vectors
mol_vectors = Matrix{UInt8}(undef, 5, 65536)
for pw in UInt16(0):UInt16(65535)
    mol_vectors[1, pw+1] = (pw >> 12) & 0xF   # S
    mol_vectors[2, pw+1] = (pw >> 8) & 0xF    # R
    mol_vectors[3, pw+1] = (pw >> 5) & 0x7    # V
    mol_vectors[4, pw+1] = (pw >> 2) & 0x7    # A
    mol_vectors[5, pw+1] = pw & 0x3           # T
end

# ─── 2. Build k-NN Table ────────────────────────────────────────
# BallTree với custom mol_dist → pre-compute 16 nearest neighbors
tree = BallTree(Float32.(mol_vectors), MolDist())

knn_table = Matrix{UInt16}(undef, 16, 65536)
for pw in UInt16(0):UInt16(65535)
    query = Float32.(mol_vectors[:, pw+1])
    idxs, _ = knn(tree, query, 16)
    knn_table[:, pw+1] .= UInt16.(idxs .- 1)  # 0-indexed cho C
end

# Write raw binary → Origin VM mmap
open("data/mol_knn_table.bin", "w") do io
    write(io, knn_table)  # 65536 × 16 × 2 bytes = 2MB
end

# ─── 3. Encode Table from UCD ───────────────────────────────────
# Read json/udc.json, apply 42 formulas, output packed P_weight table
using JSON3
ucd = JSON3.read(read("json/udc.json", String))
encode_table = zeros(UInt16, 65536)  # BMP codepoints

for entry in ucd.characters
    cp = entry.codepoint
    if cp <= 0xFFFF
        pw = entry.physics_logic.P_weight
        encode_table[cp + 1] = UInt16((pw[1] >> 4) << 12 | (pw[2] >> 4) << 8 |
                                       (pw[3] >> 5) << 5 | (pw[4] >> 5) << 2 |
                                       (pw[5] >> 6))
    end
end

open("data/encode_table.bin", "w") do io
    write(io, encode_table)  # 65536 × 2 = 128KB
end

# ─── 4. Unicode NFD Normalization Table ─────────────────────────
# Julia's utf8proc → export codepoint→NFD decomposition
using Unicode
nfd_table = Dict{UInt32, Vector{UInt32}}()
for cp in 0x0000:0x10FFFF
    try
        s = string(Char(cp))
        nfd = Unicode.normalize(s, :NFD)
        if nfd != s
            nfd_table[UInt32(cp)] = [UInt32(c) for c in nfd]
        end
    catch; end
end
# Write as binary lookup table cho VM
# Format: [count:u32] [entries: {cp:u32, decomp_len:u8, decomp:u32[]}]

# ─── 5. NRC-VAD → V/A Lookup ───────────────────────────────────
# Already done: src/nrc_vad_data.h (19,971 entries)
# Julia can extend: compute V/A for ALL 65536 BMP codepoints
# using interpolation from NRC-VAD + emoji subgroup defaults
```

### 11.2 Julia Offline Analysis Tools

```julia
# ─── Silk Fixed-Point Analysis ──────────────────────────────────
# At what weight does 1 fire/day exactly balance decay?
using Symbolics
@variables w
fire_gain = (1 - w/65535) * 236       # φ⁻³ Hebbian
decay_loss = w * (1 - 0.618)          # 1 - φ⁻¹ per 24h
# Steady state: fire_gain = decay_loss
# (1 - w/65535)*236 = w * 0.382
# 236 - 236w/65535 = 0.382w
# 236 = w(0.382 + 236/65535) = w * 0.3856
# w = 236/0.3856 ≈ 611.9 → ~612/65535 ≈ 0.00934
# Meaning: edge fired 1×/day stabilizes at very low weight
# Need ~50 fires/day to reach φ⁻¹ (0.618) = 40,543/65535

# ─── Instinct Coefficient Optimization ──────────────────────────
using Optim
# Gradient-free: Nelder-Mead trên validation set
function accuracy(coeffs)
    # Run 7 instincts với coeffs trên test data, measure accuracy
    # coeffs = [honesty_silk, honesty_fire, honesty_source, honesty_consist]
end
result = optimize(c -> -accuracy(c), [0.3, 0.3, 0.2, 0.2], NelderMead())

# ─── Collision Rate Measurement ─────────────────────────────────
# Count how many codepoints share same P_weight
collisions = Dict{UInt16, Vector{UInt32}}()
for cp in 0x0000:0xFFFF
    pw = encode_table[cp + 1]
    push!(get!(collisions, pw, UInt32[]), UInt32(cp))
end
histogram = [count(v -> length(v) == n, values(collisions)) for n in 1:20]
println("Collision histogram: ", histogram)
# Kết quả này trả lời Q1 trong VM §30
```

### 11.3 Data Files (Julia output → Origin VM input)

| File | Size | Nội dung | VM load method |
|------|------|---------|----------------|
| `data/mol_knn_table.bin` | 2MB | 65536×16 nearest neighbors | mmap |
| `data/encode_table.bin` | 128KB | BMP codepoint → P_weight | mmap |
| `data/nfd_table.bin` | ~200KB | NFD decomposition table | mmap |
| `src/nrc_vad_data.h` | ~1MB | 19,971 VAD entries | compile-in |
| `json/udc.json` | 8MB | Full UCD data (8,284 entries) | build tool |

---

## 12. IMPLEMENTATION ROADMAP

### Phase 1: Core VM (C)
- [ ] Value representation (NaN boxing)
- [ ] Main heap + Cheney GC
- [ ] String interning + UTF-8
- [ ] Instruction decoder
- [ ] Interpreter main loop (computed goto)
- [ ] Opcodes: data, arithmetic, comparison, control flow, collection
- **Mục tiêu: chạy được "Hello World" bằng Olang**

### Phase 2: Language Features
- [ ] Closures + upvalues
- [ ] Struct, enum, trait, impl
- [ ] Pattern matching
- [ ] Error handling (try/catch/throw)
- [ ] Module system (import)
- [ ] Concurrency (spawn, channel, select)
- **Mục tiêu: self-hosting Olang compiler**

### Phase 3: Brain
- [ ] KnowTree (store, lookup, nearest, walk)
- [ ] Silk (co_activate, decay, walk, implicit)
- [ ] Pipeline (14 steps)
- [ ] 7 Instincts
- [ ] Dream cycle
- [ ] UCD table + encode
- **Mục tiêu: Nox có thể learn + recall**

### Phase 4: Agent
- [ ] SecurityGate
- [ ] AAM
- [ ] ISL protocol
- [ ] Worker/Chief hierarchy
- [ ] Self-modification
- [ ] Persistence
- **Mục tiêu: Nox tự vận hành**

### Phase 5: "Eat" Languages
- [ ] Julia parser (extract functions, types)
- [ ] Python parser
- [ ] JSON parser (đã có 199 LOC)
- [ ] Generic text ingestion
- **Mục tiêu: Nox ăn Julia source code và học**

---

## 12. 23 QT RULES (BẤT BIẾN, từ Rust gốc)

| QT | Rule | Enforcement |
|----|------|-------------|
| 1 | Unicode = nền tảng duy nhất | UCD table compiled-in |
| 2 | Chain phải finite | FUSE opcode validates |
| 3 | 3 cấp nhận thức: giả thuyết, kiểm chứng, sự thật | Separate operators |
| 4 | Molecule chỉ từ encode_codepoint | No manual mol construction |
| 5 | P_weight = u16, 5D | Compile-time enforced |
| 6 | MolecularChain = ordered sequence | Append-only |
| 7 | LCA = representative chain | Deterministic algorithm |
| 8 | Auto-registry: mọi component đăng ký | RegistryGate |
| 9 | File-before-RAM: ghi file TRƯỚC khi update memory | Pipeline enforced |
| 10 | Append-only storage | No delete/overwrite |
| 11 | Silk same-layer: edge = 2 nodes cùng layer | Validated on create |
| 12 | EmotionTag trên mọi edge | Required field |
| 13 | Hebbian per-dimension (Oja/STDP/BCM) | Silk subsystem |
| 14 | L0 independent of L1+ | No upward dependency |
| 15 | 3 agent tiers: AAM → Chief → Worker | ISL routing |
| 16 | Fibonacci thresholds cho promotion | DreamCycle |
| 17 | BlackCurtain: im lặng khi không biết | Honesty instinct |
| 18 | 9 bất biến phải qua check | RegistryGate red-alert |
| 19 | Skill = stateless | No state in Skill struct |
| 20 | Skill = single responsibility | 1 skill = 1 function |
| 21 | Skill không biết Agent | No agent import in skill |
| 22 | I/O qua ExecContext | No direct I/O |
| 23 | φ = golden ratio = constant cơ bản | All constants derived |

---

---

## 13. CONVERSATION CURVE (từ Rust context/emotion/curve.rs)

Mô hình cảm xúc trong hội thoại. Quyết định tone của response.

```c
typedef struct {
    float curve[256];     // valence history (max 256 turns)
    float d1[256];        // first derivative V'(t)
    float d2[256];        // second derivative V''(t)
    float fx_conv;        // conversation function value
    float fx_dn;          // dendrite function value
    float fx;             // final combined value
    float window_var;     // variance in recent window
    int   unstable;       // instability flag
    int   count;          // number of data points
} ConversationCurve;

// Push new valence value (mỗi turn)
void curve_push(ConversationCurve* c, float valence) {
    int i = c->count;
    if (i >= 256) return;
    
    c->curve[i] = valence;
    
    // First derivative: V'(t) = V(t) - V(t-1)
    c->d1[i] = (i > 0) ? valence - c->curve[i-1] : 0;
    
    // Second derivative: V''(t) = V'(t) - V'(t-1)
    c->d2[i] = (i > 0) ? c->d1[i] - c->d1[i-1] : 0;
    
    // f_conv = V + φ⁻² × V'(t) + φ⁻³ × V''(t)
    // φ⁻² ≈ 0.382, φ⁻³ ≈ 0.236
    c->fx_conv = valence + 0.382f * c->d1[i] + 0.236f * c->d2[i];
    
    // f_dn: EMA with phi weights
    // f_dn = φ⁻¹ × f_dn_prev + φ⁻² × V(t)
    c->fx_dn = 0.618f * c->fx_dn + 0.382f * valence;
    
    // f(x) = α × f_conv + β × f_dn
    // α = φ⁻¹ ≈ 0.618, β = φ⁻² ≈ 0.382
    c->fx = 0.618f * c->fx_conv + 0.382f * c->fx_dn;
    
    // Instability detection: window variance > 0.04 AND ≥2 sign changes in last 5
    if (i >= 5) {
        float sum = 0, sum2 = 0;
        int sign_changes = 0;
        for (int j = i-4; j <= i; j++) {
            sum += c->d1[j];
            sum2 += c->d1[j] * c->d1[j];
            if (j > i-4 && ((c->d1[j] > 0) != (c->d1[j-1] > 0)))
                sign_changes++;
        }
        float mean = sum / 5.0f;
        c->window_var = sum2 / 5.0f - mean * mean;
        c->unstable = (c->window_var > 0.04f && sign_changes >= 2);
    }
    
    c->count++;
}

// 6 response tones (từ Rust silk/walk.rs)
enum ResponseTone {
    TONE_SUPPORTIVE  = 0,  // V'<0, V''≥0: declining but recovering → "tôi hiểu"
    TONE_PAUSE       = 1,  // V'<0, V''<0: accelerating decline → "hãy dừng lại"
    TONE_REINFORCING = 2,  // V'>0, V''≥0: improving → "tiếp tục"
    TONE_CELEBRATORY = 3,  // V'>0, V''>0.1: rapidly improving → "tuyệt vời!"
    TONE_GENTLE      = 4,  // unstable → override to gentle regardless
    TONE_ENGAGED     = 5,  // V'≈0: stable → "thú vị"
};

int curve_tone(ConversationCurve* c) {
    if (c->count == 0) return TONE_ENGAGED;
    if (c->unstable) return TONE_GENTLE;  // override
    
    int i = c->count - 1;
    float d1 = c->d1[i];
    float d2 = c->d2[i];
    
    if (d1 < -0.05f) {
        return (d2 >= 0) ? TONE_SUPPORTIVE : TONE_PAUSE;
    }
    if (d1 > 0.05f) {
        return (d2 > 0.1f) ? TONE_CELEBRATORY : TONE_REINFORCING;
    }
    return TONE_ENGAGED;
}
```

---

## 14. SECURITY GATE (từ Rust agents/pipeline/gate.rs)

```c
enum GateVerdict {
    GATE_ALLOW        = 0,
    GATE_BLOCK_HARM   = 1,  // physical harm keywords
    GATE_BLOCK_HATE   = 2,  // hate speech
    GATE_BLOCK_MANIP  = 3,  // manipulation
    GATE_BLOCK_DELETE = 4,  // delete attempt
    GATE_BLACKCURTAIN = 5,  // insufficient knowledge → silence
    GATE_CRISIS       = 6,  // emergency detected
};

// Crisis detection: keyword matching (multilingual)
int gate_check_crisis(const char* text, size_t len) {
    // Vietnamese crisis keywords
    static const char* crisis_vn[] = {
        "tu tu", "muon chet", "ket thuc", "khong muon song",
        "tu hai", "nhay cau", "uong thuoc", NULL
    };
    // English crisis keywords
    static const char* crisis_en[] = {
        "suicide", "kill myself", "end my life", "want to die",
        "self harm", "jump off", NULL
    };
    
    for (int i = 0; crisis_vn[i]; i++) {
        if (strstr(text, crisis_vn[i])) return 1;
    }
    for (int i = 0; crisis_en[i]; i++) {
        if (strstr(text, crisis_en[i])) return 1;
    }
    return 0;
}

// Capability Gate (from Rust gate.rs)
enum Capability {
    CAP_SENSOR_READ    = 0,  // always granted
    CAP_ACTUATOR_WRITE = 1,  // auto-granted
    CAP_NETWORK_LOCAL  = 2,  // always granted
    CAP_NETWORK_EXT    = 3,  // needs user approval
    CAP_FILE_READ      = 4,  // auto-granted
    CAP_FILE_WRITE     = 5,  // auto-granted
    CAP_QR_WRITE       = 6,  // needs user approval
    CAP_MEDIA_CAPTURE  = 7,  // needs user approval
};

// Epistemic Firewall (from Rust gate.rs)
enum EpistemicLevel {
    EPIST_FACT       = 0,  // no disclaimer
    EPIST_OPINION    = 1,  // add caveat prefix
    EPIST_HYPOTHESIS = 2,  // add "có thể"
    EPIST_UNKNOWN    = 3,  // BlackCurtain → silence "[chưa có đủ dữ liệu]"
    EPIST_DEPRECATED = 4,  // warning prefix
};
```

---

## 15. EMOTION CONTEXT (từ Rust context/emotion/context.rs)

```c
// Role sensitivity multipliers
enum Role {
    ROLE_FIRST_PERSON  = 0,  // sensitivity = 1.00
    ROLE_SECOND_PERSON = 1,  // sensitivity = 0.75
    ROLE_THIRD_PERSON  = 2,  // sensitivity = 0.55
    ROLE_OBSERVER      = 3,  // sensitivity = 0.30
};

static const float ROLE_SENSITIVITY[4] = { 1.00f, 0.75f, 0.55f, 0.30f };

// Emotion source sensitivity
enum EmotionSource {
    SRC_REAL_NOW    = 0,  // 1.00
    SRC_REAL_PAST   = 1,  // 0.80
    SRC_MEMORY      = 2,  // 0.70
    SRC_REAL_OTHER  = 3,  // 0.60
    SRC_FICTION      = 4,  // 0.30
    SRC_MUSIC        = 5,  // 0.25
};

static const float SOURCE_SENSITIVITY[6] = { 1.00f, 0.80f, 0.70f, 0.60f, 0.30f, 0.25f };

// Composite sensitivity formula
// S = role × source × (0.5 + 0.5 × recency) × shared_bonus × expected_dampen
// clamped to [0.05, 1.0]
float emotion_sensitivity(int role, int source, float recency, int shared, int expected) {
    float s = ROLE_SENSITIVITY[role] * SOURCE_SENSITIVITY[source];
    s *= (0.5f + 0.5f * recency);
    if (shared) s *= 1.15f;     // shared emotion amplified
    if (expected) s *= 0.80f;   // expected emotion dampened
    if (s < 0.05f) s = 0.05f;
    if (s > 1.00f) s = 1.00f;
    return s;
}

// Apply sensitivity to raw emotion
typedef struct {
    float valence;  // [-1, 1]
    float arousal;  // [0, 1]
    float intensity; // [0, 1]
} EmotionTag;

EmotionTag emotion_apply_context(EmotionTag raw, float sensitivity) {
    return (EmotionTag){
        .valence   = raw.valence * sensitivity,
        .arousal   = 0.1f + (raw.arousal - 0.1f) * sensitivity,
        .intensity = raw.intensity * sensitivity * sensitivity,
    };
}
```

---

## 16. MULTI-MODAL FUSION (từ Rust context/analysis/fusion.rs)

```c
// Base weights per modality
// Text=0.30, Audio=0.40, Image=0.25, Bio=0.50
static const float MODALITY_WEIGHT[4] = { 0.30f, 0.40f, 0.25f, 0.50f };

typedef struct {
    EmotionTag tag;
    float      confidence;
    int        modality;  // 0=Text, 1=Audio, 2=Image, 3=Bio
} ModalityInput;

typedef struct {
    EmotionTag tag;
    float      confidence;
    int        has_conflict;
    float      conflict_level;
} FusedEmotion;

// Fuse multiple modality inputs
FusedEmotion fusion_fuse(ModalityInput* inputs, int count) {
    if (count == 0) return (FusedEmotion){{0,0,0}, 0, 0, 0};
    
    float total_w = 0;
    float sum_v = 0, sum_a = 0, sum_i = 0, sum_conf = 0;
    
    for (int i = 0; i < count; i++) {
        float w = MODALITY_WEIGHT[inputs[i].modality] * inputs[i].confidence;
        sum_v += inputs[i].tag.valence * w;
        sum_a += inputs[i].tag.arousal * w;
        sum_i += inputs[i].tag.intensity * w;
        sum_conf += inputs[i].confidence;
        total_w += w;
    }
    
    if (total_w < 1e-8f) total_w = 1.0f;
    
    // Conflict detection: max|V_i - V_j| > φ⁻² (0.382)
    float max_v_diff = 0;
    for (int i = 0; i < count; i++) {
        for (int j = i+1; j < count; j++) {
            float diff = fabsf(inputs[i].tag.valence - inputs[j].tag.valence);
            if (diff > max_v_diff) max_v_diff = diff;
        }
    }
    int conflict = (max_v_diff > 0.382f);
    float conflict_level = max_v_diff;
    
    float avg_conf = sum_conf / count;
    // Confidence reduction when conflict
    if (conflict) {
        avg_conf *= (1.0f - conflict_level * 0.618f);
    }
    
    // BlackCurtain threshold = φ⁻² × (1 - φ⁻³) ≈ 0.29
    // If confidence < 0.29 → BlackCurtain (silence)
    
    return (FusedEmotion){
        .tag = { sum_v/total_w, sum_a/total_w, sum_i/total_w },
        .confidence = avg_conf,
        .has_conflict = conflict,
        .conflict_level = conflict_level,
    };
}
```

---

## 17. INTENT DETECTION (từ Rust context/analysis/intent.rs)

```c
enum IntentKind {
    INTENT_LEARN    = 0,
    INTENT_HEAL     = 1,
    INTENT_COMMAND  = 2,
    INTENT_INFORM   = 3,
    INTENT_RESEARCH = 4,
    INTENT_TECHNICAL = 5,
    INTENT_CREATIVE = 6,
    INTENT_EXPLORE  = 7,
    INTENT_MANIPULATE = 8,
    INTENT_RISK     = 9,
    INTENT_CRISIS   = 10,
    INTENT_CHAT     = 11,
    INTENT_CONFIRM  = 12,
    INTENT_DENY     = 13,
};

// Score accumulation per keyword bucket
// CRISIS=0.80, RISK=0.60, MANIPULATE=0.65, HEAL=0.50, LEARN_CMD=0.70
// Winner = max(bucket), confidence = clamp(score, 0.35, 0.95)
// NeedClarify if !sensitive AND conf < 0.55 AND words <= 6

typedef struct {
    int   intent;
    float confidence;
    int   need_clarify;
} IntentResult;

IntentResult detect_intent(const char* text, float cur_valence, float cur_arousal) {
    // Keyword scoring: Vietnamese + English keywords per intent
    // Mở rộng bằng NRC-VAD (19,971 entries — MOL §27, src/nrc_vad_data.h)
    
    IntentResult r = { INTENT_CHAT, 0.5f, 0 };
    
    // Crisis detection (highest priority)
    if (gate_check_crisis(text, strlen(text))) {
        r.intent = INTENT_CRISIS;
        r.confidence = 0.90f;
        return r;
    }
    
    // Emotional amplifiers from current V/A
    // V < -0.70 + A < 0.35 → crisis boost
    if (cur_valence < -0.70f && cur_arousal < 0.35f) {
        r.intent = INTENT_HEAL;
        r.confidence = 0.70f;
    }
    
    // [CẦN BỔ SUNG] Full keyword table từ Rust intent.rs
    // Bao gồm: Vietnamese + English keywords cho mỗi intent
    // Score = sum(keyword_weights) + emotional_amplifier
    
    r.need_clarify = (r.confidence < 0.55f && strlen(text) <= 30);
    return r;
}
```

---

## 18. RESPONSE GENERATION — PIPELINE (từ ATLAS §4.2 + Rust runtime/output/)

**Trạng thái:** Pipeline có 5 bước. Bước ① RETRIEVE cần VP-Tree + DecodeMap (xem SPEC_MOLECULAR_ENGINE §14 + §23). Bước ③ COMPOSE cần decode chain→text (SPEC_MOLECULAR_ENGINE §23). §24 dưới đây có code compose_response hoàn chỉnh cho Vietnamese output.

```c
// Generation Pipeline (ATLAS design):
// ① RETRIEVE → ② RANK → ③ COMPOSE → ④ REFINE → ⑤ FLUENCY

// ① RETRIEVE: search KnowTree → top-K chains
typedef struct {
    MolChain chain;
    float    relevance;
    float    recency;
    float    confidence;
    float    silk_weight;
} RetrievedChain;

void generate_retrieve(VM* vm, uint16_t query, int k, RetrievedChain* results, int* count) {
    // Phase 1: VP-Tree → top 100 candidates
    // Phase 2: Shadow Vector cosine → top K
    // → xem SPEC_MOLECULAR_ENGINE §14 two_phase_search()
    
    // Xem SPEC_MOLECULAR_ENGINE §14 (VP-Tree knn) + §24 (rebuild policy)
    // Phase 1: vptree_knn(&vm->vptree, kt->pw_array, query, 100, candidates, &n);
    // Phase 2: cosine similarity trên Shadow Vector → lọc top-K
    // [CẦN VP-Tree + Shadow Vector đã build]
    *count = 0;
}

// ② RANK: weighted score
float generate_rank_score(RetrievedChain* r) {
    return r->relevance * 0.4f + r->recency * 0.2f + r->confidence * 0.2f + r->silk_weight * 0.2f;
}

// ③ COMPOSE: ghép fragments theo templates
enum TemplateKind {
    TPL_DEFINITION  = 0,  // "{subject} là {predicate} {object}"
    TPL_CAUSAL      = 1,  // "{cause} dẫn đến {effect}"
    TPL_NARRATIVE    = 2,  // "{agent} {action} {object} {context}"
    TPL_COMPARISON   = 3,  // "{a} giống {b} ở {aspect}, khác ở {difference}"
    TPL_QUESTION     = 4,  // "Bạn có muốn biết thêm về {topic}?"
    TPL_ACKNOWLEDGE  = 5,  // "Tôi hiểu. {paraphrase}"
};

typedef struct {
    int    kind;
    char*  slots[8];     // filled slots
    int    slot_count;
} Template;

// Template filling: simple cho MVP. KAN B-spline = DEFER (Phase 3+).
// Template đủ cho Nox nói — Rust origin.rs đã chứng minh.
// 6 template types × 8 slots = 48 parameters. Học từ feedback (§25 BCM).

void generate_compose(RetrievedChain* chains, int count, int tone, char* output, size_t max_len) {
    if (count == 0) {
        snprintf(output, max_len, "[chưa có đủ dữ liệu]");  // BlackCurtain
        return;
    }
    
    // Select template based on intent + tone
    // Decode chains → text: xem SPEC_MOLECULAR_ENGINE §23 decode_chain_to_utf8()
    // Fill template slots từ decoded text

    // Decode top chain → text fragment
    char fragment[256];
    decode_chain_to_utf8(&vm->decode_map,
                         chains[0].chain.mols, chains[0].chain.len,
                         1,  // Latin script (hoặc detect từ context)
                         fragment, sizeof(fragment));

    // Simple template fill
    snprintf(output, max_len, "%s", fragment);
    // Full version: chọn template (§24 compose_response) + fill multiple slots
}

// ⑤ FLUENCY: ConversationCurve + length control
void generate_fluency(char* output, ConversationCurve* curve) {
    int tone = curve_tone(curve);
    // Adjust output based on tone:
    // TONE_PAUSE → shorter, softer
    // TONE_CELEBRATORY → longer, enthusiastic
    // TONE_GENTLE → avoid strong words (unstable override)
    // [CẦN IMPLEMENT]
}
```

**ĐÃ GIẢI QUYẾT:**
- Decode: inverted map 65536 buckets + context disambiguation → MOL §23
- NRC-VAD: 19,971 entries → src/nrc_vad_data.h (MOL §27)
- KAN B-spline: DEFER — template đủ cho MVP
- Collision rate: cần chạy decode_map_build() + print_stats() để đo thật

---

## 19. REASONING ENGINES (từ GIÁC document)

4 engines + Arbiter, code đầy đủ bên dưới (§23).

```c
// 4 engines, Arbiter chọn lowest entropy
enum ReasoningEngine {
    ENGINE_DEDUCE  = 0,  // A→B, A ⊢ B (modus ponens)
    ENGINE_INDUCE  = 1,  // {A₁→B, A₂→B, ...} ⊢ ∀A→B
    ENGINE_ANALOG  = 2,  // A:B :: C:D (5D delta transfer)
    ENGINE_RETRIEVE = 3, // nearest neighbor in KnowTree
};

typedef struct {
    int    engine;
    float  confidence;
    float  entropy;      // lower = more certain
    MolChain result;
} ReasoningResult;

// reason() = alias for arbiter() (§23 has full implementation)
// Removed duplicate — dùng arbiter() trực tiếp

// Self-correct: bounded iteration (max 3)
ReasoningResult reason_with_self_correct(VM* vm, MolChain* query) {
    ReasoningResult r = arbiter(vm, query);
    
    for (int iter = 0; iter < 3 && r.confidence < 0.7f; iter++) {
        // Check consistency with existing knowledge
        // If inconsistent → try different engine
        // [CẦN GIÁC DATA cho chi tiết]
        r = arbiter(vm, query);
    }
    
    return r;
}
```

---

## 20. OPCODE STATUS — CẬP NHẬT

| Range | Opcodes | Status |
|-------|---------|--------|
| 0x00-0x05 | NOP, LOAD_NIL/TRUE/FALSE, LOAD_INT, LOAD_CONST | ✅ DONE (§4.2) |
| 0x06-0x07 | LOAD_GLOBAL, STORE_GLOBAL | ✅ CODE BELOW |
| 0x08 | MOVE | ✅ DONE (§4.2) |
| 0x09-0x0A | LOAD_UPVAL, STORE_UPVAL | ✅ SPEC (§4.2) |
| 0x0B-0x0C | LOAD_FIELD, STORE_FIELD | ✅ CODE BELOW |
| 0x0D-0x0E | LOAD_INDEX, STORE_INDEX | ✅ CODE BELOW |
| 0x0F | LOAD_MOL | ✅ DONE (§4.2) |
| 0x10-0x15 | ADD, SUB, MUL, DIV, MOD, NEG | ✅ DONE (§4.2) |
| 0x16-0x1B | SHL, SHR, BAND, BOR, BXOR, BNOT | ✅ CODE BELOW |
| 0x1C | CONCAT | ✅ DONE (§4.2) |
| 0x20-0x25 | EQ, NE, LT, LE, GT, GE | ✅ DONE (§4.2) |
| 0x26-0x28 | CONF_AND, CONF_OR, CONF_NOT | ✅ CODE BELOW |
| 0x30-0x32 | JMP, JZ, JNZ | ✅ DONE (§4.2) |
| 0x33 | CALL | ✅ SPEC (§4.2) |
| 0x34-0x35 | RET, RET_NIL | ✅ DONE (§4.2) |
| 0x36 | CLOSURE | ✅ SPEC (§4.2) |
| 0x38-0x3A | TRY_BEGIN, CATCH_END, THROW | ✅ SPEC (§4.2) |
| 0x3B | CLOSURE_CAP | ✅ SPEC (§4.2) |
| 0x3C-0x3D | LOOP_INIT, LOOP_DEC | ✅ CODE BELOW |
| 0x40-0x44 | NEW_ARRAY, PUSH, GET, SET, LEN | ✅ SPEC (§4.2) |
| 0x45-0x47 | NEW_DICT, DICT_GET, DICT_SET | ✅ SPEC (§4.2) |
| 0x50-0x5B | Molecular ops | ✅ CODE BELOW (pack/unpack/compose/chain) |
| 0x60-0x69 | Brain ops | ✅ CODE BELOW (KT/Silk/Dream/Instinct) |
| 0x70 | EMIT | ✅ DONE (§4.2) |
| 0x71-0x73 | READLINE, FILE_READ, FILE_WRITE | ✅ CODE BELOW |
| 0xF0-0xF8 | System ops | DEFER — spawn/channel/persist/eval |
| 0xFE | HALT | ✅ DONE (§4.2) |
| 0xFF | BUILTIN | ✅ CODE BELOW |

### 20.1 Opcodes còn thiếu — code đầy đủ

```c
/* ── LOAD/STORE GLOBAL (0x06-0x07) ────────────────────────────── */

op_load_global: {
    uint8_t a = FIELD_A(*pc);
    uint16_t d = FIELD_D(*pc);
    // K[d] = string key (interned)
    OlStr* name = as_string(K[d]);
    base[a] = vm_get_global(vm, name->data);
    pc++;
    DISPATCH();
}

op_store_global: {
    uint8_t a = FIELD_A(*pc);
    uint16_t d = FIELD_D(*pc);
    OlStr* name = as_string(K[d]);
    vm_set_global(vm, name->data, base[a]);
    pc++;
    DISPATCH();
}

/* ── LOAD/STORE FIELD (0x0B-0x0C) ─────────────────────────────── */
// obj.field → DICT_GET, obj.field = val → DICT_SET
// Struct = dict with known keys (compiler generates LOAD_CONST for field name)

op_load_field: {
    uint32_t inst = *pc;
    Value obj = base[FIELD_B(inst)];
    Value key = base[FIELD_C(inst)];
    if (is_dict(obj)) {
        OlDict* d = as_dict(obj);
        base[FIELD_A(inst)] = dict_get(d, as_string(key));
    } else {
        base[FIELD_A(inst)] = val_nil();
    }
    pc++;
    DISPATCH();
}

op_store_field: {
    uint32_t inst = *pc;
    Value obj = base[FIELD_A(inst)];
    Value key = base[FIELD_B(inst)];
    Value val = base[FIELD_C(inst)];
    if (is_dict(obj)) {
        OlDict* d = as_dict(obj);
        dict_set(d, as_string(key), val);
    }
    pc++;
    DISPATCH();
}

/* ── LOAD/STORE INDEX (0x0D-0x0E) ─────────────────────────────── */
// Đã có ARRAY_GET/SET nhưng INDEX là generic (array hoặc dict)

op_load_index: {
    uint32_t inst = *pc;
    Value obj = base[FIELD_B(inst)];
    Value idx = base[FIELD_C(inst)];
    if (is_array(obj)) {
        OlArray* arr = as_array(obj);
        int32_t i = (int32_t)as_number(idx);
        base[FIELD_A(inst)] = (i >= 0 && (uint32_t)i < arr->length)
                              ? arr->items[i] : val_nil();
    } else if (is_dict(obj)) {
        base[FIELD_A(inst)] = dict_get(as_dict(obj), as_string(idx));
    } else {
        base[FIELD_A(inst)] = val_nil();
    }
    pc++;
    DISPATCH();
}

op_store_index: {
    uint32_t inst = *pc;
    Value obj = base[FIELD_A(inst)];
    Value idx = base[FIELD_B(inst)];
    Value val = base[FIELD_C(inst)];
    if (is_array(obj)) {
        OlArray* arr = as_array(obj);
        int32_t i = (int32_t)as_number(idx);
        if (i >= 0 && (uint32_t)i < arr->length) arr->items[i] = val;
    } else if (is_dict(obj)) {
        dict_set(as_dict(obj), as_string(idx), val);
    }
    pc++;
    DISPATCH();
}

/* ── BITWISE (0x16-0x1B) ──────────────────────────────────────── */

op_shl: {
    uint32_t inst = *pc;
    int32_t a = (int32_t)as_number(base[FIELD_B(inst)]);
    int32_t b = (int32_t)as_number(base[FIELD_C(inst)]);
    base[FIELD_A(inst)] = val_int(a << (b & 31));
    pc++; DISPATCH();
}
op_shr: {
    uint32_t inst = *pc;
    int32_t a = (int32_t)as_number(base[FIELD_B(inst)]);
    int32_t b = (int32_t)as_number(base[FIELD_C(inst)]);
    base[FIELD_A(inst)] = val_int((uint32_t)a >> (b & 31));
    pc++; DISPATCH();
}
op_band: {
    uint32_t inst = *pc;
    base[FIELD_A(inst)] = val_int((int32_t)as_number(base[FIELD_B(inst)])
                                & (int32_t)as_number(base[FIELD_C(inst)]));
    pc++; DISPATCH();
}
op_bor: {
    uint32_t inst = *pc;
    base[FIELD_A(inst)] = val_int((int32_t)as_number(base[FIELD_B(inst)])
                                | (int32_t)as_number(base[FIELD_C(inst)]));
    pc++; DISPATCH();
}
op_bxor: {
    uint32_t inst = *pc;
    base[FIELD_A(inst)] = val_int((int32_t)as_number(base[FIELD_B(inst)])
                                ^ (int32_t)as_number(base[FIELD_C(inst)]));
    pc++; DISPATCH();
}
op_bnot: {
    uint32_t inst = *pc;
    base[FIELD_A(inst)] = val_int(~(int32_t)as_number(base[FIELD_B(inst)]));
    pc++; DISPATCH();
}

/* ── CONFIDENCE LOGIC (0x26-0x28) ─────────────────────────────── */
// Confidence = float 0.0-1.0. Logic ops: Bayesian-inspired.

op_conf_and: {
    // P(A AND B) = P(A) × P(B) (independent assumption)
    uint32_t inst = *pc;
    double a = as_number(base[FIELD_B(inst)]);
    double b = as_number(base[FIELD_C(inst)]);
    base[FIELD_A(inst)] = val_float(a * b);
    pc++; DISPATCH();
}
op_conf_or: {
    // P(A OR B) = P(A) + P(B) - P(A)×P(B)
    uint32_t inst = *pc;
    double a = as_number(base[FIELD_B(inst)]);
    double b = as_number(base[FIELD_C(inst)]);
    base[FIELD_A(inst)] = val_float(a + b - a * b);
    pc++; DISPATCH();
}
op_conf_not: {
    // P(NOT A) = 1 - P(A)
    uint32_t inst = *pc;
    double a = as_number(base[FIELD_B(inst)]);
    base[FIELD_A(inst)] = val_float(1.0 - a);
    pc++; DISPATCH();
}

/* ── LOOP (0x3C-0x3D) ─────────────────────────────────────────── */

op_loop_init: {
    // R[A] = D (init counter)
    base[FIELD_A(*pc)] = val_int((int32_t)FIELD_D(*pc));
    pc++; DISPATCH();
}
op_loop_dec: {
    // R[A]--; if R[A] > 0: PC += signed(D)
    uint32_t inst = *pc;
    int32_t val = as_int(base[FIELD_A(inst)]) - 1;
    base[FIELD_A(inst)] = val_int(val);
    if (val > 0) {
        pc += FIELD_SD(inst);
    } else {
        pc++;
    }
    DISPATCH();
}

/* ── MOLECULAR (0x50-0x5B) ────────────────────────────────────── */

op_mol_pack: {
    // R[A] = mol_pack(R[B]=s, R[C]=r, stack: v, a, t)
    // Format: ABC + 3 extra regs
    uint32_t inst = *pc;
    uint8_t s = (uint8_t)as_number(base[FIELD_B(inst)]);
    uint8_t r = (uint8_t)as_number(base[FIELD_C(inst)]);
    // v, a, t in next 3 regs after C
    uint8_t c = FIELD_C(inst);
    uint8_t v = (uint8_t)as_number(base[c + 1]);
    uint8_t a = (uint8_t)as_number(base[c + 2]);
    uint8_t t = (uint8_t)as_number(base[c + 3]);
    base[FIELD_A(inst)] = val_mol(mol_pack(s, r, v, a, t));
    pc++; DISPATCH();
}

op_mol_unpack: {
    // R[A]=S, R[A+1]=R, R[A+2]=V, R[A+3]=A, R[A+4]=T from R[B]
    uint32_t inst = *pc;
    uint16_t m = as_mol(base[FIELD_B(inst)]);
    uint8_t a = FIELD_A(inst);
    base[a]   = val_int(MOL_S(m));
    base[a+1] = val_int(MOL_R(m));
    base[a+2] = val_int(MOL_V(m));
    base[a+3] = val_int(MOL_A(m));
    base[a+4] = val_int(MOL_T(m));
    pc++; DISPATCH();
}

op_mol_compose: {
    // R[A] = compose(R[B], R[C]) — xem MOL §9
    uint32_t inst = *pc;
    uint16_t a = as_mol(base[FIELD_B(inst)]);
    uint16_t b = as_mol(base[FIELD_C(inst)]);
    base[FIELD_A(inst)] = val_mol(mol_compose_pair(a, b));
    pc++; DISPATCH();
}

op_chain_new: {
    uint8_t a = FIELD_A(*pc);
    // Allocate MolChain on heap (OBJ_CHAIN)
    size_t sz = sizeof(ObjHeader) + sizeof(uint16_t) * 64 + 8;
    void* mem = heap_alloc(&vm->main_heap, sz);
    OBJ_INIT((ObjHeader*)mem, OBJ_CHAIN, sz);
    // chain_len at offset sizeof(ObjHeader)
    *(uint16_t*)((uint8_t*)mem + sizeof(ObjHeader)) = 0;
    base[a] = val_object((OlObject*)mem);
    pc++; DISPATCH();
}

op_chain_push: {
    uint32_t inst = *pc;
    OlObject* obj = as_object(base[FIELD_A(inst)]);
    uint16_t* len_ptr = (uint16_t*)((uint8_t*)obj + sizeof(ObjHeader));
    uint16_t* data = (uint16_t*)((uint8_t*)obj + sizeof(ObjHeader) + 8);
    if (*len_ptr < 64) {
        data[*len_ptr] = as_mol(base[FIELD_B(inst)]);
        (*len_ptr)++;
    }
    pc++; DISPATCH();
}

op_chain_len: {
    uint32_t inst = *pc;
    OlObject* obj = as_object(base[FIELD_B(inst)]);
    uint16_t len = *(uint16_t*)((uint8_t*)obj + sizeof(ObjHeader));
    base[FIELD_A(inst)] = val_int(len);
    pc++; DISPATCH();
}

/* ── BRAIN OPS (0x60-0x69) ────────────────────────────────────── */

op_kt_store: {
    // R[A] = kt_store(chain_from R[B], len R[C])
    uint32_t inst = *pc;
    OlObject* chain_obj = as_object(base[FIELD_B(inst)]);
    uint16_t len = *(uint16_t*)((uint8_t*)chain_obj + sizeof(ObjHeader));
    uint16_t* mols = (uint16_t*)((uint8_t*)chain_obj + sizeof(ObjHeader) + 8);
    uint32_t id = kt_store(vm->knowtree, mols, len, 0);
    base[FIELD_A(inst)] = val_int((int32_t)id);
    pc++; DISPATCH();
}

op_kt_nearest: {
    // R[A] = nearest node ID for mol R[B], k=R[C]
    uint32_t inst = *pc;
    uint16_t query = as_mol(base[FIELD_B(inst)]);
    int k = (int)as_number(base[FIELD_C(inst)]);
    if (k < 1) k = 1;
    if (k > 10) k = 10;
    uint32_t results[10];
    int found = 0;
    kt_nearest(vm->knowtree, query, k, results, &found);
    if (found > 0)
        base[FIELD_A(inst)] = val_int((int32_t)results[0]);
    else
        base[FIELD_A(inst)] = val_nil();
    pc++; DISPATCH();
}

op_silk_fire: {
    // silk_fire(node_a=R[A], node_b=R[B])
    uint32_t inst = *pc;
    uint32_t a = (uint32_t)as_int(base[FIELD_A(inst)]);
    uint32_t b = (uint32_t)as_int(base[FIELD_B(inst)]);
    silk_co_activate(vm->silk, a, b);
    pc++; DISPATCH();
}

op_silk_weight: {
    // R[A] = silk_weight(node_a=R[B], node_b=R[C])
    uint32_t inst = *pc;
    uint32_t a = (uint32_t)as_int(base[FIELD_B(inst)]);
    uint32_t b = (uint32_t)as_int(base[FIELD_C(inst)]);
    float w = silk_get_weight(vm->silk, a, b) / 65535.0f;
    base[FIELD_A(inst)] = val_float(w);
    pc++; DISPATCH();
}

op_dream: {
    // Trigger dream cycle manually
    dream_cycle(&vm->dream, &vm->stm, vm->knowtree, vm->silk);
    pc++; DISPATCH();
}

op_instinct: {
    // R[A] = instinct[R[B]](pipeline context)
    // B = instinct index (0-6)
    uint32_t inst = *pc;
    int idx = (int)as_number(base[FIELD_B(inst)]);
    float result = 0;
    switch (idx) {
        // Instincts need current chain from pipeline result
        // base[a+1] = chain mol (first mol of current input)
        {
        uint16_t chain[1] = { as_mol(base[a+1]) };
        uint16_t clen = 1;
        switch (idx) {
            case 0: result = instinct_honesty(&vm->pipeline, chain, clen); break;
            case 1: result = instinct_contradiction(&vm->pipeline, chain, clen, NULL, 0); break;
            case 2: result = instinct_causality(&vm->pipeline, chain, clen); break;
            case 3: result = instinct_abstraction(&vm->pipeline, chain, clen); break;
            case 4: result = instinct_analogy_score(&vm->pipeline, chain, clen); break;
            case 5: result = instinct_curiosity(&vm->pipeline, chain, clen); break;
            case 6: result = instinct_reflection(&vm->pipeline); break;
        }
        }
    }
    base[FIELD_A(inst)] = val_float(result);
    pc++; DISPATCH();
}

/* ── I/O (0x71-0x73) ──────────────────────────────────────────── */

op_readline: {
    // R[A] = readline from stdin
    uint8_t a = FIELD_A(*pc);
    char buf[4096];
    if (fgets(buf, sizeof(buf), stdin)) {
        // Strip trailing newline
        size_t len = strlen(buf);
        if (len > 0 && buf[len-1] == '\n') buf[--len] = '\0';
        OlStr* s = strtab_intern(&vm->strings, (uint8_t*)buf, len);
        base[a] = val_string(s);
    } else {
        base[a] = val_nil();
    }
    pc++; DISPATCH();
}

op_file_read: {
    // R[A] = file_read(path=R[B])
    uint32_t inst = *pc;
    OlStr* path = as_string(base[FIELD_B(inst)]);
    int fd = open((const char*)path->data, O_RDONLY);
    if (fd < 0) { base[FIELD_A(inst)] = val_nil(); pc++; DISPATCH(); }
    struct stat st;
    fstat(fd, &st);
    size_t sz = st.st_size;
    uint8_t* buf = malloc(sz);
    read(fd, buf, sz);
    close(fd);
    OlStr* content = strtab_intern(&vm->strings, buf, sz);
    free(buf);
    base[FIELD_A(inst)] = val_string(content);
    pc++; DISPATCH();
}

op_file_write: {
    // file_write(path=R[A], content=R[B])
    uint32_t inst = *pc;
    OlStr* path = as_string(base[FIELD_A(inst)]);
    OlStr* content = as_string(base[FIELD_B(inst)]);
    int fd = open((const char*)path->data, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd >= 0) {
        write(fd, content->data, content->byte_len);
        close(fd);
    }
    pc++; DISPATCH();
}

/* ── BUILTIN (0xFF) ───────────────────────────────────────────── */

op_builtin: {
    // R[A] = builtin[D](args...)
    // D = builtin function index
    // Args in R[A+1], R[A+2], ...
    uint32_t inst = *pc;
    uint8_t a = FIELD_A(inst);
    uint16_t fn_id = FIELD_D(inst);

    // Dispatch to builtin function table
    // vm->builtins[fn_id](vm, base + a)
    if (fn_id < vm->builtin_count) {
        vm->builtins[fn_id](vm, base + a);
    } else {
        base[a] = val_nil();
    }
    pc++; DISPATCH();
}

// §20.1.1 BUILTIN IMPLEMENTATIONS
// Mỗi builtin = 1 C function: void builtin_xxx(VM* vm, Value* args)
// args[0] = return slot, args[1..] = arguments
// Register: vm_register_builtin(vm, "name", fn_ptr) at startup

// ── String builtins ─────────────────────────────────────────────

void builtin_str_len(VM* vm, Value* args) {
    // __str_len(s) → int
    OlStr* s = as_string(args[1]);
    args[0] = val_int(s->len);
}

void builtin_str_split(VM* vm, Value* args) {
    // __str_split(s, delim) → array of strings
    OlStr* s = as_string(args[1]);
    OlStr* delim = as_string(args[2]);
    OlArray* result = array_new(&vm->heap, 8);
    const char* p = s->data;
    const char* end = s->data + s->len;
    while (p < end) {
        const char* found = memmem(p, end - p, delim->data, delim->len);
        if (!found) found = end;
        uint32_t part_len = (uint32_t)(found - p);
        OlStr* part = str_intern(&vm->strings, p, part_len);
        array_push(&vm->heap, result, val_string(part));
        p = found + delim->len;
        if (found == end) break;
    }
    args[0] = val_array(result);
}

void builtin_str_trim(VM* vm, Value* args) {
    // __str_trim(s) → string (strip leading/trailing whitespace)
    OlStr* s = as_string(args[1]);
    const char* start = s->data;
    const char* end = s->data + s->len;
    while (start < end && (*start == ' ' || *start == '\t' || *start == '\n' || *start == '\r')) start++;
    while (end > start && (end[-1] == ' ' || end[-1] == '\t' || end[-1] == '\n' || end[-1] == '\r')) end--;
    args[0] = val_string(str_intern(&vm->strings, start, (uint32_t)(end - start)));
}

void builtin_str_starts_with(VM* vm, Value* args) {
    // __str_starts_with(s, prefix) → bool
    OlStr* s = as_string(args[1]);
    OlStr* prefix = as_string(args[2]);
    int result = (s->len >= prefix->len && memcmp(s->data, prefix->data, prefix->len) == 0);
    args[0] = val_bool(result);
}

void builtin_str_contains(VM* vm, Value* args) {
    // __str_contains(s, sub) → bool
    OlStr* s = as_string(args[1]);
    OlStr* sub = as_string(args[2]);
    int result = (memmem(s->data, s->len, sub->data, sub->len) != NULL);
    args[0] = val_bool(result);
}

// ── Collection builtins ─────────────────────────────────────────

void builtin_len(VM* vm, Value* args) {
    // __len(x) → int (works on string, array, dict)
    Value v = args[1];
    if (is_string(v))     args[0] = val_int(as_string(v)->len);
    else if (is_array(v)) args[0] = val_int(as_array(v)->len);
    else if (is_dict(v))  args[0] = val_int(as_dict(v)->count);
    else                  args[0] = val_int(0);
}

void builtin_push(VM* vm, Value* args) {
    // __push(array, value) → array (mutates and returns)
    OlArray* arr = as_array(args[1]);
    array_push(&vm->heap, arr, args[2]);
    args[0] = args[1];
}

void builtin_dict_keys(VM* vm, Value* args) {
    // __dict_keys(d) → array of key strings
    OlDict* d = as_dict(args[1]);
    OlArray* keys = array_new(&vm->heap, d->count);
    for (uint32_t i = 0; i < d->capacity; i++) {
        if (!is_nil(d->entries[i].key)) {
            array_push(&vm->heap, keys, d->entries[i].key);
        }
    }
    args[0] = val_array(keys);
}

void builtin_to_string(VM* vm, Value* args) {
    // __to_string(x) → string representation
    Value v = args[1];
    char buf[64];
    if (is_nil(v))        { args[0] = val_string(str_intern(&vm->strings, "nil", 3)); return; }
    if (is_bool(v))       { const char* s = as_bool(v) ? "true" : "false"; args[0] = val_string(str_intern(&vm->strings, s, strlen(s))); return; }
    if (is_int(v))        { int n = snprintf(buf, 64, "%d", as_int(v)); args[0] = val_string(str_intern(&vm->strings, buf, n)); return; }
    if (is_float(v))      { int n = snprintf(buf, 64, "%g", as_float(v)); args[0] = val_string(str_intern(&vm->strings, buf, n)); return; }
    if (is_string(v))     { args[0] = v; return; }
    args[0] = val_string(str_intern(&vm->strings, "<object>", 8));
}

// ── File I/O builtins ───────────────────────────────────────────

void builtin_file_read(VM* vm, Value* args) {
    // __file_read(path) → string (entire file) or nil on error
    OlStr* path = as_string(args[1]);
    char cpath[1024];
    uint32_t plen = path->len < 1023 ? path->len : 1023;
    memcpy(cpath, path->data, plen);
    cpath[plen] = '\0';
    FILE* f = fopen(cpath, "rb");
    if (!f) { args[0] = val_nil(); return; }
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (size <= 0 || size > 64 * 1024 * 1024) { fclose(f); args[0] = val_nil(); return; }
    char* data = malloc(size);
    if (!data) { fclose(f); args[0] = val_nil(); return; }
    fread(data, 1, size, f);
    fclose(f);
    OlStr* result = str_intern(&vm->strings, data, (uint32_t)size);
    free(data);
    args[0] = val_string(result);
}

void builtin_file_write(VM* vm, Value* args) {
    // __file_write(path, content) → bool
    OlStr* path = as_string(args[1]);
    OlStr* content = as_string(args[2]);
    char cpath[1024];
    uint32_t plen = path->len < 1023 ? path->len : 1023;
    memcpy(cpath, path->data, plen);
    cpath[plen] = '\0';
    FILE* f = fopen(cpath, "wb");
    if (!f) { args[0] = val_bool(0); return; }
    fwrite(content->data, 1, content->len, f);
    fclose(f);
    args[0] = val_bool(1);
}

// ── Registration ────────────────────────────────────────────────

// Builtin ID table (index = fn_id used by OP_BUILTIN):
//   0: str_len       1: str_split       2: str_trim
//   3: str_starts_with  4: str_contains 5: len
//   6: push           7: dict_keys       8: to_string
//   9: file_read     10: file_write
//
// void vm_register_builtins(VM* vm) {
//     vm_register_builtin(vm, "__str_len",        builtin_str_len);
//     vm_register_builtin(vm, "__str_split",      builtin_str_split);
//     vm_register_builtin(vm, "__str_trim",       builtin_str_trim);
//     vm_register_builtin(vm, "__str_starts_with",builtin_str_starts_with);
//     vm_register_builtin(vm, "__str_contains",   builtin_str_contains);
//     vm_register_builtin(vm, "__len",            builtin_len);
//     vm_register_builtin(vm, "__push",           builtin_push);
//     vm_register_builtin(vm, "__dict_keys",      builtin_dict_keys);
//     vm_register_builtin(vm, "__to_string",      builtin_to_string);
//     vm_register_builtin(vm, "__file_read",      builtin_file_read);
//     vm_register_builtin(vm, "__file_write",     builtin_file_write);
// }
```

### 20.2 DEFER — system opcodes (0xF0-0xF8)

Spawn, channel, persist, eval — cần thiết kế concurrency model trước.
Không viết code giả. Khi thiết kế xong → viết vào đây.

---

## 21. HOMEOSTASIS — PID CONTROLLER (từ GIÁC §9.1)

```c
// Error signal = surprise / free energy
// F(t) = RMSE between predicted and actual ConceptVectors
float homeostasis_error(float* predicted, float* actual, int dims) {
    float sum_sq = 0;
    for (int i = 0; i < dims; i++) {
        float d = predicted[i] - actual[i];
        sum_sq += d * d;
    }
    return sqrtf(sum_sq / dims);
}

// PID controller for learning rate
typedef struct {
    float kp;           // 0.5  — proportional
    float ki;           // 0.01 — integral
    float kd;           // 0.1  — derivative
    float integral;     // accumulated error
    float prev_error;   // previous F(t)
} HomeostasisPID;

float homeostasis_lr(HomeostasisPID* pid, float error) {
    pid->integral += error;
    float derivative = error - pid->prev_error;
    pid->prev_error = error;
    return pid->kp * error + pid->ki * pid->integral + pid->kd * derivative;
}

// Mode switching:
// F(t) > threshold → LEARNING MODE (high surprise)
// F(t) < threshold → ANSWER MODE (confident enough)
// Homeostasis threshold = tunable parameter (default 0.5)
// Nghiên cứu: ACT-R dùng 0.0, Soar tunable. Mọi cognitive arch tune per-model.
// Không ai hardcode irrational constant. Giá trị khởi đầu, điều chỉnh từ data.
#define HOMEOSTASIS_THRESHOLD 0.5f  // tune via meta-learning (§27 CP7)
```

---

## 22. DREAM CYCLE — DBSCAN (từ GIÁC §9.2)

```c
// DBSCAN parameters (from GIÁC)
#define DREAM_DBSCAN_EPS     0.4f   // cosine distance threshold
#define DREAM_DBSCAN_MIN_PTS 3      // minimum cluster size

// Fibonacci schedule
static const int FIB_SCHEDULE[] = {2, 3, 5, 8, 13, 21, 34, 55};

// should_dream: canonical definition ở §5.5 (DreamCycle*, stm_count)
// Dùng stm_count (bao nhiêu items trong STM) để trigger

// Dream consolidation algorithm
typedef struct {
    uint32_t* members;
    int       member_count;
    float     centroid[8];  // Shadow Vector centroid
    float     confidence;
} DreamCluster;

void dream_cycle(VM* vm) {
    // 1. Collect STM nodes
    // 2. DBSCAN clustering (eps=0.4, min_pts=3)
    // 3. Per cluster: compute centroid, search archive for fit
    // 4. If fit > 0.7: create hypothesis, confidence += 0.2 per fitting domain
    // 5. If confidence > 0.5: insert to Archive (Growing maturity)
    // 6. Clean STM (remove non-pinned clustered nodes)
    
    int stm_count = vm->stm.count;
    if (stm_count < DREAM_DBSCAN_MIN_PTS) return;
    
    // DBSCAN
    int* labels = calloc(stm_count, sizeof(int));  // 0=unvisited, -1=noise, >0=cluster_id
    int cluster_id = 0;
    
    for (int i = 0; i < stm_count; i++) {
        if (labels[i] != 0) continue;
        
        // Find neighbors within eps
        int neighbors[256];
        int n_count = 0;
        for (int j = 0; j < stm_count; j++) {
            float dist = 1.0f - molchain_similarity(&vm->stm.entries[i].chain, &vm->stm.entries[j].chain);
            if (dist <= DREAM_DBSCAN_EPS) {
                if (n_count < 256) neighbors[n_count++] = j;
            }
        }
        
        if (n_count < DREAM_DBSCAN_MIN_PTS) {
            labels[i] = -1;  // noise
            continue;
        }
        
        cluster_id++;
        labels[i] = cluster_id;
        
        // Expand cluster
        for (int k = 0; k < n_count; k++) {
            int j = neighbors[k];
            if (labels[j] == -1) labels[j] = cluster_id;
            if (labels[j] != 0) continue;
            labels[j] = cluster_id;
            
            // Find j's neighbors
            for (int m = 0; m < stm_count; m++) {
                float dist = 1.0f - molchain_similarity(&vm->stm.entries[j].chain, &vm->stm.entries[m].chain);
                if (dist <= DREAM_DBSCAN_EPS) {
                    int already = 0;
                    for (int n = 0; n < n_count; n++) if (neighbors[n] == m) { already = 1; break; }
                    if (!already && n_count < 256) neighbors[n_count++] = m;
                }
            }
        }
    }
    
    // Process each cluster → consolidate into KnowTree
    for (int c = 1; c <= cluster_id; c++) {
        float confidence = 0;
        int count = 0;

        // Compute cluster centroid via LCA
        uint16_t centroid_chain[64];
        uint16_t centroid_len = 0;
        int first = 1;

        for (int i = 0; i < stm_count; i++) {
            if (labels[i] != c) continue;
            count++;

            if (first) {
                memcpy(centroid_chain, vm->stm.entries[i].chain,
                       vm->stm.entries[i].chain_len * 2);
                centroid_len = vm->stm.entries[i].chain_len;
                first = 0;
            } else {
                // LCA blend centroid with this member
                uint16_t tmp[64];
                uint16_t tmp_len;
                chain_lca(centroid_chain, centroid_len,
                          vm->stm.entries[i].chain, vm->stm.entries[i].chain_len,
                          tmp, &tmp_len);
                memcpy(centroid_chain, tmp, tmp_len * 2);
                centroid_len = tmp_len;
            }

            // Search KnowTree for existing fit
            uint32_t fit[1];
            int fn = 0;
            kt_nearest(vm->knowtree, vm->stm.entries[i].chain[0], 1, fit, &fn);
            if (fn > 0) {
                float dist = mol_dist(vm->stm.entries[i].chain[0],
                                      vm->knowtree->nodes[fit[0]].p_weight) / 70.0f;
                if (dist < 0.3f) confidence += 0.2f;  // fits existing knowledge
            }
        }

        if (confidence > 0.5f && count >= DREAM_DBSCAN_MIN_PTS && centroid_len > 0) {
            // Store cluster centroid as new KnowTree node
            uint32_t new_id = kt_store(vm->knowtree, centroid_chain, centroid_len, 0);
            KTNode* node = &vm->knowtree->nodes[new_id];
            node->maturity = 1;  // EVALUATING (growing)
            node->fire_count = count;

            // Co-activate all cluster members with new node
            for (int i = 0; i < stm_count; i++) {
                if (labels[i] != c) continue;
                // Find or create node for this STM entry
                uint32_t member_id = kt_store(vm->knowtree, vm->stm.entries[i].chain,
                                               vm->stm.entries[i].chain_len, 0);
                silk_co_activate(vm->silk, new_id, member_id);
            }

            // Check QR promotion (adaptive threshold from §5.7)
            if (check_promotion_adaptive(vm->knowtree, vm->silk,
                                          &vm->domain_stats, new_id)) {
                node->flags |= 1;  // QR bit — permanent
                node->maturity = 2;  // MATURE
                vm->knowtree->qr_count++;
            }
        }
    }
    
    free(labels);
}

// QR Promotion check (from GIÁC §9.3)
int check_promotion(uint32_t node_id, KnowTree* kt, SilkGraph* silk) {
    KTNode* node = &kt->nodes[node_id];
    
    // 1. Maturity must be MATURE
    if (node->maturity != MATURITY_MATURE) return 0;
    
    // 2. fire_count >= fibonacci(depth)
    int depth = node->layer;
    int fib_threshold = (depth < 8) ? FIB_SCHEDULE[depth] : 55;
    if (node->fire_count < (uint16_t)fib_threshold) return 0;
    
    // QR promotion: ưu tiên dùng BCM adaptive (§5.7 check_promotion_adaptive)
    // Fallback: fixed 0.65 (ACT-R default = 0.0, tune per-model)
    float threshold = 0.65f;
    float weight = silk_max_weight(silk, node_id) / 65535.0f;
    if (weight < threshold) return 0;
    
    // 4. No contradictions
    // [CẦN IMPLEMENT] find_contradictions()
    
    return 1;  // promote to QR
}
```

---

## 23. 4 REASONING ENGINES — ĐẦY ĐỦ (từ GIÁC §6)

```c
// === ENGINE 1: DEDUCE — Forward Chaining ===
// A→B, A ⊢ B (modus ponens)
typedef struct {
    MolChain answer;
    float    confidence;
    float    entropy;
    int      engine;
} ReasoningResult;

ReasoningResult engine_deduce(VM* vm, MolChain* query) {
    ReasoningResult r = { .answer = {0}, .confidence = 0, .entropy = 1.0f, .engine = 0 };
    if (query->len < 1 || !vm->knowtree) return r;

    // Find nearest nodes in KnowTree
    uint32_t nearest[10];
    int n = 0;
    kt_nearest(vm->knowtree, query->mols[0], 10, nearest, &n);

    float best_conf = 0;
    for (int i = 0; i < n; i++) {
        KTNode* node = &vm->knowtree->nodes[nearest[i]];
        // Check edges for CAUSES relation (R in [8..12])
        uint8_t rel = MOL_R(node->p_weight);
        if (rel < 8 || rel > 12) continue;  // not causal

        // Match = 1 - distance/70 (normalized)
        float dist = mol_dist(query->mols[0], node->p_weight) / 70.0f;
        float match = 1.0f - dist;
        if (match < 0.75f) continue;

        // Confidence = match × silk weight
        float silk_w = silk_get_weight(vm->silk, 0, nearest[i]) / 65535.0f;
        float conf = match * fmaxf(silk_w, 0.1f);
        if (conf > best_conf) {
            best_conf = conf;
            // Answer = the chain connected to this causal node
            uint16_t* chain_data = vm->knowtree->chains + node->chain_offset;
            memcpy(r.answer.mols, chain_data, MIN(node->chain_len, 64) * 2);
            r.answer.len = MIN(node->chain_len, 64);
        }
    }
    r.confidence = best_conf;
    r.entropy = 1.0f - best_conf;
    return r;
}

// === ENGINE 2: INDUCE — Centroid + Counter-example ===
ReasoningResult engine_induce(VM* vm, MolChain* query) {
    ReasoningResult r = { .answer = {0}, .confidence = 0, .entropy = 1.0f, .engine = 1 };
    if (query->len < 1 || !vm->knowtree) return r;

    // Find examples: nodes with similarity > 0.5
    uint32_t nearest[20];
    int n = 0;
    kt_nearest(vm->knowtree, query->mols[0], 20, nearest, &n);

    // Filter by similarity threshold
    uint32_t examples[20];
    int ex_count = 0;
    for (int i = 0; i < n; i++) {
        float dist = mol_dist(query->mols[0], vm->knowtree->nodes[nearest[i]].p_weight) / 70.0f;
        if (dist < 0.5f) examples[ex_count++] = nearest[i];
    }
    if (ex_count < 3) return r;  // need at least 3 examples

    // Compute centroid (LCA of all examples)
    uint16_t centroid = examples[0];
    for (int i = 1; i < ex_count; i++) {
        centroid = mol_compose_pair(centroid, vm->knowtree->nodes[examples[i]].p_weight);
    }

    // Variance
    float var_sum = 0;
    for (int i = 0; i < ex_count; i++) {
        float d = mol_dist(centroid, vm->knowtree->nodes[examples[i]].p_weight) / 70.0f;
        var_sum += d * d;
    }
    float variance = var_sum / ex_count;

    // Confidence
    r.confidence = fminf(ex_count / 10.0f, 1.0f) * fmaxf(1.0f - variance, 0.0f);
    r.entropy = variance;
    // Answer = centroid chain
    r.answer.mols[0] = centroid;
    r.answer.len = 1;
    return r;
}

// === ENGINE 3: ANALOG — Vector Arithmetic A:B :: C:D ===
ReasoningResult engine_analog(VM* vm, MolChain* query) {
    ReasoningResult r = { .answer = {0}, .confidence = 0, .entropy = 1.0f, .engine = 2 };
    if (query->len < 1 || !vm->knowtree) return r;

    uint16_t q = query->mols[0];

    // Find 2 nearest related nodes (A, B) via silk walk
    uint32_t nearest[5];
    int n = 0;
    kt_nearest(vm->knowtree, q, 5, nearest, &n);
    if (n < 2) return r;

    uint16_t a_mol = vm->knowtree->nodes[nearest[0]].p_weight;
    uint16_t b_mol = vm->knowtree->nodes[nearest[1]].p_weight;

    // Delta = B - A per dimension
    // Predicted D = C + delta (C = query)
    int ds = MOL_S(b_mol) - MOL_S(a_mol);
    int dr = MOL_R(b_mol) - MOL_R(a_mol);
    int dv = MOL_V(b_mol) - MOL_V(a_mol);
    int da = MOL_A(b_mol) - MOL_A(a_mol);
    int dt = MOL_T(b_mol) - MOL_T(a_mol);

    int ps = CLAMP(MOL_S(q) + ds, 0, 15);
    int pr = CLAMP(MOL_R(q) + dr, 0, 15);
    int pv = CLAMP(MOL_V(q) + dv, 0, 7);
    int pa = CLAMP(MOL_A(q) + da, 0, 7);
    int pt = CLAMP(MOL_T(q) + dt, 0, 3);
    uint16_t predicted = (ps << 12) | (pr << 8) | (pv << 5) | (pa << 2) | pt;

    // Search for nearest match to predicted
    uint32_t match_nearest[1];
    int mn = 0;
    kt_nearest(vm->knowtree, predicted, 1, match_nearest, &mn);
    if (mn > 0) {
        float dist = mol_dist(predicted, vm->knowtree->nodes[match_nearest[0]].p_weight) / 70.0f;
        r.confidence = 1.0f - dist;
        r.entropy = dist;
        r.answer.mols[0] = vm->knowtree->nodes[match_nearest[0]].p_weight;
        r.answer.len = 1;
    }
    return r;
}

// === ENGINE 4: RETRIEVE — Direct Lookup ===
ReasoningResult engine_retrieve(VM* vm, MolChain* query) {
    ReasoningResult r = { .answer = {0}, .confidence = 0, .entropy = 1.0f, .engine = 3 };
    if (query->len < 1 || !vm->knowtree) return r;

    // Search KnowTree for nearest neighbor
    uint32_t nearest[1];
    int n = 0;
    kt_nearest(vm->knowtree, query->mols[0], 1, nearest, &n);

    if (n > 0) {
        KTNode* node = &vm->knowtree->nodes[nearest[0]];
        float dist = mol_dist(query->mols[0], node->p_weight) / 70.0f;
        r.confidence = 1.0f - dist;
        r.entropy = dist;
        // Copy chain as answer
        uint16_t* chain_data = vm->knowtree->chains + node->chain_offset;
        memcpy(r.answer.mols, chain_data, MIN(node->chain_len, 64) * 2);
        r.answer.len = MIN(node->chain_len, 64);
    }
    return r;
}

// === ARBITER — Select by confidence/(1+entropy) ===
ReasoningResult arbiter(VM* vm, MolChain* query) {
    ReasoningResult results[4];
    results[0] = engine_deduce(vm, query);
    results[1] = engine_induce(vm, query);
    results[2] = engine_analog(vm, query);
    results[3] = engine_retrieve(vm, query);
    
    // Score = confidence / (1 + entropy)
    int best = 0;
    float best_score = 0;
    for (int i = 0; i < 4; i++) {
        float score = results[i].confidence / (1.0f + results[i].entropy);
        if (score > best_score) {
            best_score = score;
            best = i;
        }
    }
    
    // Honesty Gate
    if (results[best].confidence < 0.35f) {
        results[best].answer = molchain_empty();  // SILENCE
    }
    
    return results[best];
}

// === SELF-CORRECT — Bounded iteration (max 3) ===
ReasoningResult reason_with_self_correct(VM* vm, MolChain* query) {
    ReasoningResult r = arbiter(vm, query);
    
    // Quality formula:
    // quality = 0.3×validity + 0.3×(1-entropy) + 0.2×consistency + 0.2×edge_support
    float quality = 0.3f * r.confidence + 0.3f * (1.0f - r.entropy);
    // consistency: check contra silk edges. edge_support: count supporting edges.
    float consistency = 1.0f;  // assume consistent until contradiction found
    float edge_support = (r.answer.len > 0)
        ? silk_max_weight(vm->silk, 0) / 65535.0f : 0.0f;
    quality += 0.2f * consistency + 0.2f * edge_support;
    
    for (int iter = 0; iter < 3 && quality < 0.6f; iter++) {
        // Try different engine or refine
        ReasoningResult r2 = arbiter(vm, query);
        float q2 = 0.3f * r2.confidence + 0.3f * (1.0f - r2.entropy);
        
        if (q2 <= quality) break;  // rollback if worse
        r = r2;
        quality = q2;
    }
    
    return r;
}
```

---

## 24. RESPONSE GENERATION — ĐẦY ĐỦ (từ Rust origin.rs + ATLAS)

Origin dùng **template-based + emotion-driven**, KHÔNG phải LLM.

```c
// === Response struct ===
typedef struct {
    char*  text;          // output text
    int    tone;          // ResponseTone enum
    float  fx;            // ConversationCurve f(x)
    int    kind;          // Natural/OlangResult/Crisis/Blocked/System
} Response;

// === 12 Intent Actions ===
enum IntentAction {
    ACTION_PROCEED       = 0,
    ACTION_EMPATHIZE     = 1,
    ACTION_ASK_CONTEXT   = 2,
    ACTION_SOFT_REFUSAL  = 3,
    ACTION_CRISIS        = 4,
    ACTION_ADD_CLARIFY   = 5,
    ACTION_USER_CONFIRM  = 6,
    ACTION_USER_DENY     = 7,
    ACTION_FORCE_LEARN   = 8,
    ACTION_CONFIRM_LEARN = 9,
    ACTION_OBSERVE       = 10,
    ACTION_SILENT_ACK    = 11,
    ACTION_HOME_CONTROL  = 12,
};

// Decide action from intent (from Rust intent.rs)
int decide_action(IntentResult* est, float cur_v) {
    switch (est->intent) {
    case INTENT_CRISIS:     return ACTION_CRISIS;
    case INTENT_RISK:       return ACTION_ASK_CONTEXT;
    case INTENT_MANIPULATE: return ACTION_SOFT_REFUSAL;
    case INTENT_HEAL:       return (est->confidence < 0.55f) ? ACTION_OBSERVE : ACTION_EMPATHIZE;
    case INTENT_COMMAND:    return ACTION_HOME_CONTROL;
    case INTENT_CONFIRM:    return ACTION_USER_CONFIRM;
    case INTENT_DENY:       return ACTION_USER_DENY;
    default:
        if (est->need_clarify) return ACTION_ADD_CLARIFY;
        return ACTION_PROCEED;
    }
}

// === Response composition: [acknowledgment] + [topic] + [follow_up] ===

// Acknowledgment (Vietnamese)
const char* acknowledgment_vi(int tone, float v) {
    if (v < -0.50f) {
        switch (tone) {
        case TONE_SUPPORTIVE: return "Minh nghe ban.";
        case TONE_PAUSE:      return "Hay dung lai mot chut.";
        case TONE_GENTLE:     return "Minh o day.";
        default:              return "Minh hieu.";
        }
    }
    if (v > 0.30f) {
        switch (tone) {
        case TONE_CELEBRATORY: return "Tuyet voi!";
        case TONE_REINFORCING: return "Hay qua!";
        default:               return "Minh nghe.";
        }
    }
    return "Minh dang nghe.";
}

// Emotion descriptor (Vietnamese)
const char* emotion_descriptor_vi(float v) {
    if (v < -0.70f) return "nang ne";
    if (v < -0.40f) return "kho khan";
    if (v < -0.10f) return "khong de dang";
    if (v <  0.20f) return "binh thuong";
    if (v <  0.50f) return "tot";
    return "tuyet voi";
}

// Topic phrase (extract content words from user input)
void extract_topic(const char* input, char* topic, size_t max_len) {
    // Filter Vietnamese stop words, keep first 4 content words
    // Stop words: "toi", "ban", "cua", "la", "va", "co", "khong", "nhu", "ma", "thi"
    // [simplified — full version needs word segmentation]
    strncpy(topic, input, max_len - 1);
    topic[max_len - 1] = '\0';
    // Truncate to ~30 chars
    if (strlen(topic) > 30) topic[30] = '\0';
}

// Follow-up (context-aware)
const char* follow_up_vi(float v, float novelty, int repetition_count) {
    if (repetition_count > 2) return "Ban da nhac den dieu nay nhieu lan roi.";
    if (novelty > 0.70f)      return "Day la dieu rat moi. Ban co the ke them?";
    if (v < -0.40f)           return "Ban muon chia se them khong?";
    if (v > 0.30f)            return "Tin vui gi vay?";
    return "Ban muon noi them gi?";
}

// === Main compose_response ===
void compose_response(Response* resp, int action, int tone, float v,
                      const char* input, float novelty, int repetition) {
    char buf[1024];
    
    switch (action) {
    case ACTION_CRISIS:
        snprintf(buf, sizeof(buf),
            "Minh nghe thay ban dang rat kho khan. "
            "Ban khong co mot minh. "
            "Hay goi: 1800 599 920 (mien phi, 24/7).");
        break;
    
    case ACTION_SOFT_REFUSAL:
        snprintf(buf, sizeof(buf),
            "Minh khong the ho tro dieu nay. "
            "Hay thu hoi mot cach khac?");
        break;
    
    case ACTION_EMPATHIZE: {
        const char* ack = acknowledgment_vi(tone, v);
        const char* desc = emotion_descriptor_vi(v);
        char topic[64];
        extract_topic(input, topic, sizeof(topic));
        snprintf(buf, sizeof(buf), "%s \"%s\" -- minh hieu day la dieu %s.", ack, topic, desc);
        break;
    }
    
    case ACTION_OBSERVE:
        snprintf(buf, sizeof(buf), "Minh dang lang nghe...");
        break;
    
    case ACTION_SILENT_ACK:
        buf[0] = '\0';  // deliberate silence
        break;
    
    case ACTION_PROCEED: {
        const char* ack = acknowledgment_vi(tone, v);
        char topic[64];
        extract_topic(input, topic, sizeof(topic));
        const char* fu = follow_up_vi(v, novelty, repetition);
        snprintf(buf, sizeof(buf), "%s %s", ack, fu);
        break;
    }
    
    default:
        snprintf(buf, sizeof(buf), "Minh dang nghe.");
        break;
    }
    
    resp->text = strdup(buf);
    resp->tone = tone;
}

// === Honesty Gate thresholds ===
// confidence < 0.35 → "Toi khong biet ve dieu nay." (SILENCE)
// confidence < 0.60 → prefix "Toi nghi rang..." (HYPOTHESIS)
// confidence < 0.85 → prefix "Theo nhung gi toi biet..." (OPINION)
// confidence >= 0.85 → direct answer (FACT)

void apply_honesty_prefix(char* buf, size_t max, float confidence, const char* answer) {
    if (confidence < 0.35f) {
        snprintf(buf, max, "Toi khong biet ve dieu nay.");
    } else if (confidence < 0.60f) {
        snprintf(buf, max, "Toi nghi rang %s", answer);
    } else if (confidence < 0.85f) {
        snprintf(buf, max, "Theo nhung gi toi biet, %s", answer);
    } else {
        snprintf(buf, max, "%s", answer);
    }
}

// === Instinct enrichment (post-processing) ===
void enrich_with_instincts(char* buf, size_t max, float contradiction, float curiosity, float reflection) {
    if (contradiction > 0.5f) {
        strncat(buf, " [Minh nhan thay co dieu mau thuan]", max - strlen(buf) - 1);
    }
    if (curiosity > 0.70f) {
        strncat(buf, " [Day la dieu rat moi]", max - strlen(buf) - 1);
    }
    if (reflection < 0.40f) {
        strncat(buf, " [Kien thuc con mong -- can them du lieu]", max - strlen(buf) - 1);
    }
}
```

---

## 25. FEEDBACK & REWARD (từ GIÁC §8.2)

```c
// Reward signals
#define REWARD_CORRECT    1.0f   // "correct" / "thanks" / continues topic
#define REWARD_WRONG     -0.5f   // "wrong" / "not that" / re-asks
#define REWARD_TOPIC_CHANGE 0.0f // changes topic
#define REWARD_STOP      -0.2f   // stops conversation early

// Edge update from feedback
void feedback_update_edge(SilkEdge* edge, float reward) {
    // Running average (Sora #5 fix — không ghi đè, gradually adjust)
    edge->reward_sum += reward;
    edge->reward_count++;
    float avg_reward = edge->reward_sum / (float)edge->reward_count;
    float delta = 0.01f * (avg_reward - 0.5f) * (1.0f - edge->weight / 65535.0f);
    int new_w = (int)edge->weight + (int)(delta * 65535);
    if (new_w < 0) new_w = 0;
    if (new_w > 65535) new_w = 65535;
    edge->weight = (uint16_t)new_w;
}

// Calibration drift detection
// If |expected_accuracy - actual_accuracy| > 0.15 per confidence bucket → alert
typedef struct {
    int    correct[10];   // per 0.1 bucket (0.0-0.1, 0.1-0.2, ...)
    int    total[10];
} CalibrationTracker;

int calibration_check_drift(CalibrationTracker* ct) {
    for (int i = 0; i < 10; i++) {
        if (ct->total[i] < 5) continue;  // not enough data
        float expected = (i + 0.5f) / 10.0f;
        float actual = (float)ct->correct[i] / ct->total[i];
        if (fabsf(expected - actual) > 0.15f) return 1;  // drift detected
    }
    return 0;
}
```

---

## 26. BCM EDGE LEARNING (từ GIÁC §5.7)

```c
// Bienenstock-Cooper-Munro learning rule
// theta_M = sliding threshold based on average activity
void bcm_update_edge(SilkEdge* edge, float activity_a, float activity_b,
                     float valence_a, float valence_b, float arousal_a, float arousal_b) {
    float current_activity = activity_a * activity_b;  // co-activation
    
    // Sliding threshold = average activity squared
    float theta_m = edge->avg_activity * edge->avg_activity;
    
    // LTP if above threshold, LTD if below
    float eta = 0.01f;  // learning rate
    float dw = eta * current_activity * (current_activity - theta_m);
    
    // Emotion amplification
    float emotion_factor = (fabsf(valence_a) + fabsf(valence_b)) / 2.0f
                         * fmaxf(arousal_a, arousal_b);
    dw *= (1.0f + emotion_factor);
    
    // Update weight
    float w = (float)edge->weight / 65535.0f;
    w += dw;
    if (w < 0) w = 0;
    if (w > 1) w = 1;
    edge->weight = (uint16_t)(w * 65535);
    
    // Update running average activity
    edge->avg_activity = 0.95f * edge->avg_activity + 0.05f * current_activity;
}
```

---

## 27. CP7 META LOOP (từ ATLAS §4.3)

```c
// Runs every 50 interactions OR when accuracy drifts >10%
typedef struct {
    int interaction_count;
    float last_accuracy;
} MetaLoop;

void meta_loop_check(MetaLoop* ml, VM* vm) {
    ml->interaction_count++;
    
    if (ml->interaction_count < 50) return;
    ml->interaction_count = 0;
    
    // 1. Check feedback patterns
    // 2. Adjust liquid τ parameters (max ±20% per cycle)
    // 3. Adjust learned residual α (max ±0.05)
    // 4. Adjust template weights (max ±0.1)
    // 5. Validate accuracy per confidence bucket
    
    // Bounded adjustments (SAFETY):
    // L0 + 9 QT NEVER touched
    // If all learned params → 0: system degenerates to pure Nox = safe fallback
}
```

---

## 28. 4-LAYER SAFETY ARCHITECTURE (từ ATLAS §6)

```
Layer 0: IMMUTABLE CORE
  - UCD table (8,284 entries)
  - 42 encode formulas
  - Compose rules (16 RelationOp)
  - Distance functions (mol_dist)
  - 9 QT constitutional rules
  - SecurityGate (hardcoded)
  → SEALED. No process can modify. Binary verified.

Layer 1: APPEND-ONLY HISTORY
  - QR archive
  - Feedback log
  - Meta-learn log
  - Self-modify history
  → Write-only. Can add, can supersede. NEVER delete.

Layer 2: BOUNDED ADAPTATION
  - Liquid τ: max ±20% per cycle
  - Learned residual α: max 0.3
  - Template weights: max ±0.1 per cycle
  - Shadow Vectors: emergent
  → If all params → 0: degenerates to pure Nox = safe

Layer 3: FREE ZONE
  - STM, Working memory, Dream hypotheses, Conversation state
  → Ephemeral. Lost on restart. No safety risk.
```

**FUSE Circuit Breaker:**
- Dream: max 100ms per node cluster
- Self-modify: max 1 file per cycle, must pass fixed-point
- Generate: max 3 template attempts before "I don't know"
- Any step exceeding time_limit → abort → safe fallback

---

## 29. CORE LEXICON (từ Rust word_guide.rs — 24 Vietnamese words)

```c
typedef struct { const char* word; float v; float a; float d; } WordEntry;

static const WordEntry CORE_LEXICON[] = {
    {"binh yen",   0.50f, 0.20f, 0.60f},
    {"nhe nhang",  0.45f, 0.20f, 0.55f},
    {"an toan",    0.55f, 0.20f, 0.65f},
    {"on",         0.30f, 0.30f, 0.55f},
    {"thoai mai",  0.50f, 0.25f, 0.60f},
    {"am ap",      0.60f, 0.30f, 0.60f},
    {"ro rang",    0.20f, 0.35f, 0.60f},
    {"hay",        0.50f, 0.45f, 0.60f},
    {"dung",       0.30f, 0.35f, 0.65f},
    {"vui",        0.70f, 0.60f, 0.70f},
    {"hanh phuc",  0.80f, 0.60f, 0.70f},
    {"tuyet voi",  0.85f, 0.70f, 0.75f},
    {"thu vi",     0.65f, 0.65f, 0.65f},
    {"phan khich", 0.70f, 0.80f, 0.65f},
    {"tot",        0.45f, 0.35f, 0.65f},
    {"kho",       -0.20f, 0.45f, 0.40f},
    {"met",       -0.30f, 0.25f, 0.35f},
    {"chan",       -0.40f, 0.20f, 0.30f},
    {"lo",        -0.35f, 0.55f, 0.30f},
    {"buon",      -0.60f, 0.30f, 0.25f},
    {"so",        -0.65f, 0.75f, 0.20f},
    {"dau",       -0.65f, 0.55f, 0.20f},
    {"mat mat",   -0.70f, 0.40f, 0.20f},
    {NULL, 0, 0, 0}
};

// Word selection: distance = V²×2 + A²×0.5 + D²×0.5 (V weighted 2×)
// Sort by distance, return top N
```

---

## 30. OPEN QUESTIONS — CẬP NHẬT

| # | Câu hỏi | Status |
|---|---------|--------|
| Q1 | Collision rate | CẦN ĐO — chạy decode_map_build() + print_stats() |
| Q2 | φ⁻¹ ablation | DEFER — stretched exp β=φ⁻¹ giải quyết (MOL §26) |
| Q3 | Template vs KAN | ĐÃ QUYẾT ĐỊNH — Template cho MVP, KAN = DEFER |
| Q4 | NRC-VAD lexicon | ✅ DONE — 19,971 entries (src/nrc_vad_data.h, MOL §27) |
| Q5 | 4 reasoning engines | ✅ DONE — code đầy đủ (§23) |
| Q6 | Julia parser | DEFER |
| Q7 | Coroutine | DEFER |
| Q8 | Template slots | ✅ DONE — 6 types (§24) |

### Còn lại chưa giải quyết

| # | Vấn đề | Cần gì |
|---|--------|--------|
| ~~S compose MIN/MAX~~ | **DECIDED: MAX** (S=category index, not SDF distance) | Done |
| ~~Bool TAG_BOOL vs float~~ | **DECIDED: TAG_BOOL** (clox/Wren/LuaJIT NaN-tag) | Done |
| ~~Homeostasis 0.6/0.618~~ | **DECIDED: 0.5 tunable** (ACT-R/Soar pattern) | Done |
| Collision rate | Chạy decode_map_build() | Cần build + run |
| 42 encode formulas | §5.8 vẫn dùng heuristic fallback | Cần UCD table đầy đủ |
| Concurrency | spawn/channel/select = DEFER | Cần green thread design |

---

*Spec: 30+ sections, ~3500 LOC code thật.
Cross-reference: SPEC_OLANG_LANGUAGE.md (§14 compiler), SPEC_MOLECULAR_ENGINE.md (§1-27).*
